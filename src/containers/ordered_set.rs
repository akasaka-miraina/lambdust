//! Ordered set implementation based on red-black tree.
//!
//! This module provides both single-threaded and thread-safe ordered set
//! implementations with efficient set operations and custom comparators.

use crate::eval::value::Value;
use super::comparator::Comparator;
use super::Container;
use std::sync::{Arc, RwLock};
use std::cmp::Ordering;

/// Colors for red-black tree nodes
#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    Red,
    Black,
}

/// Node in the red-black tree
#[derive(Clone, Debug)]
struct Node {
    value: Value,
    color: Color,
    left: Option<Arc<Node>>,
    right: Option<Arc<Node>>,
    size: usize, // Size of subtree rooted at this node
}

impl Node {
    /// Creates a new red node with the given value
    fn new_red(value: Value) -> Self {
        Self {
            value,
            color: Color::Red,
            left: None,
            right: None,
            size: 1,
        }
    }
    
    /// Creates a new black node with the given value
    fn new_black(value: Value) -> Self {
        Self {
            value,
            color: Color::Black,
            left: None,
            right: None,
            size: 1,
        }
    }
    
    /// Creates a new node with specified children and color
    fn new_with_children(
        value: Value,
        color: Color,
        left: Option<Arc<Node>>,
        right: Option<Arc<Node>>,
    ) -> Self {
        let size = 1 
            + left.as_ref().map(|n| n.size).unwrap_or(0)
            + right.as_ref().map(|n| n.size).unwrap_or(0);
        
        Self {
            value,
            color,
            left,
            right,
            size,
        }
    }
    
    /// Updates the size of this node based on its children
    fn update_size(&mut self) {
        self.size = 1 
            + self.left.as_ref().map(|n| n.size).unwrap_or(0)
            + self.right.as_ref().map(|n| n.size).unwrap_or(0);
    }
    
    /// Checks if this node is red
    fn is_red(&self) -> bool {
        self.color == Color::Red
    }
    
    /// Checks if this node is black
    fn is_black(&self) -> bool {
        self.color == Color::Black
    }
    
    /// Makes this node red
    fn make_red(mut self) -> Self {
        self.color = Color::Red;
        self
    }
    
    /// Makes this node black
    fn make_black(mut self) -> Self {
        self.color = Color::Black;
        self
    }
}

/// Helper function to check if a node is red (None is considered black)
fn is_red(node: &Option<Arc<Node>>) -> bool {
    node.as_ref().map(|n| n.is_red()).unwrap_or(false)
}

/// Helper function to get the size of a node (None has size 0)
fn size(node: &Option<Arc<Node>>) -> usize {
    node.as_ref().map(|n| n.size).unwrap_or(0)
}

/// Red-black tree implementation
#[derive(Clone, Debug)]
struct RedBlackTree {
    root: Option<Arc<Node>>,
    comparator: Comparator,
}

impl RedBlackTree {
    /// Creates a new empty red-black tree
    fn new(comparator: Comparator) -> Self {
        Self {
            root: None,
            comparator,
        }
    }
    
    /// Gets the size of the tree
    fn size(&self) -> usize {
        size(&self.root)
    }
    
    /// Checks if the tree is empty
    fn is_empty(&self) -> bool {
        self.root.is_none()
    }
    
    /// Searches for a value in the tree
    fn contains(&self, value: &Value) -> bool {
        self.search_node(&self.root, value).is_some()
    }
    
    /// Helper function to search for a node
    fn search_node(&self, node: &Option<Arc<Node>>, value: &Value) -> Option<Arc<Node>> {
        match node {
            None => None,
            Some(n) => {
                match self.comparator.compare(value, &n.value) {
                    Ordering::Equal => Some(n.clone()),
                    Ordering::Less => self.search_node(&n.left, value),
                    Ordering::Greater => self.search_node(&n.right, value),
                }
            }
        }
    }
    
    /// Inserts a value into the tree
    fn insert(&mut self, value: Value) -> bool {
        let root = self.root.take();
        let (new_root, inserted) = self.insert_recursive(root, value);
        self.root = new_root.map(|n| Arc::new(n.make_black()));
        inserted
    }
    
    /// Recursive insertion helper
    fn insert_recursive(&self, node: Option<Arc<Node>>, value: Value) -> (Option<Node>, bool) {
        match node {
            None => (Some(Node::new_red(value)), true),
            Some(n) => {
                match self.comparator.compare(&value, &n.value) {
                    Ordering::Equal => (Some((*n).clone()), false), // Already exists
                    Ordering::Less => {
                        let (new_left, inserted) = self.insert_recursive(n.left.clone(), value);
                        let mut new_node = Node::new_with_children(
                            n.value.clone(),
                            n.color,
                            new_left.map(Arc::new),
                            n.right.clone(),
                        );
                        new_node = self.fix_up(new_node);
                        (Some(new_node), inserted)
                    }
                    Ordering::Greater => {
                        let (new_right, inserted) = self.insert_recursive(n.right.clone(), value);
                        let mut new_node = Node::new_with_children(
                            n.value.clone(),
                            n.color,
                            n.left.clone(),
                            new_right.map(Arc::new),
                        );
                        new_node = self.fix_up(new_node);
                        (Some(new_node), inserted)
                    }
                }
            }
        }
    }
    
    /// Removes a value from the tree
    fn remove(&mut self, value: &Value) -> bool {
        if !self.contains(value) {
            return false;
        }
        
        // Make root red if both children are black
        if let Some(ref root) = self.root {
            if !is_red(&root.left) && !is_red(&root.right) {
                let cloned_root = (**root).clone();
                self.root = Some(Arc::new(cloned_root.make_red()));
            }
        }
        
        let root = self.root.take();
        self.root = self.delete_recursive(root, value);
        
        // Make root black
        if let Some(root) = self.root.take() {
            self.root = Some(Arc::new((*root).clone().make_black()));
        }
        
        true
    }
    
    /// Recursive deletion helper
    fn delete_recursive(&self, node: Option<Arc<Node>>, value: &Value) -> Option<Arc<Node>> {
        match node {
            None => None,
            Some(n) => {
                match self.comparator.compare(value, &n.value) {
                    Ordering::Less => {
                        let mut new_node = (*n).clone();
                        
                        if !is_red(&n.left) && n.left.as_ref().map(|l| !is_red(&l.left)).unwrap_or(true) {
                            new_node = self.move_red_left(new_node);
                        }
                        
                        new_node.left = self.delete_recursive(new_node.left, value);
                        new_node.update_size();
                        Some(Arc::new(self.fix_up(new_node)))
                    }
                    _ => {
                        if is_red(&n.left) {
                            let mut new_node = self.rotate_right((*n).clone());
                            new_node.left = self.delete_recursive(new_node.left, value);
                            new_node.update_size();
                            Some(Arc::new(self.fix_up(new_node)))
                        } else {
                            if self.comparator.compare(value, &n.value) == Ordering::Equal && n.right.is_none() {
                                return None;
                            }
                            
                            let mut new_node = (*n).clone();
                            
                            if !is_red(&n.right) && n.right.as_ref().map(|r| !is_red(&r.left)).unwrap_or(true) {
                                new_node = self.move_red_right(new_node);
                            }
                            
                            if self.comparator.compare(value, &new_node.value) == Ordering::Equal {
                                // Replace with minimum from right subtree
                                if let Some(min_val) = Self::find_min(&new_node.right) {
                                    new_node.value = min_val;
                                    new_node.right = self.delete_min(new_node.right);
                                } else {
                                    return None;
                                }
                            } else {
                                new_node.right = self.delete_recursive(new_node.right, value);
                            }
                            
                            new_node.update_size();
                            Some(Arc::new(self.fix_up(new_node)))
                        }
                    }
                }
            }
        }
    }
    
    /// Finds the minimum value in a subtree
    fn find_min(node: &Option<Arc<Node>>) -> Option<Value> {
        match node {
            None => None,
            Some(n) => {
                if n.left.is_none() {
                    Some(n.value.clone())
                } else {
                    Self::find_min(&n.left)
                }
            }
        }
    }
    
    /// Deletes the minimum node from a subtree
    fn delete_min(&self, node: Option<Arc<Node>>) -> Option<Arc<Node>> {
        match node {
            None => None,
            Some(n) => {
                if n.left.is_none() {
                    return n.right.clone();
                }
                
                let mut new_node = (*n).clone();
                
                if !is_red(&n.left) && n.left.as_ref().map(|l| !is_red(&l.left)).unwrap_or(true) {
                    new_node = self.move_red_left(new_node);
                }
                
                new_node.left = self.delete_min(new_node.left);
                new_node.update_size();
                Some(Arc::new(self.fix_up(new_node)))
            }
        }
    }
    
    /// Rotates left
    fn rotate_left(&self, mut node: Node) -> Node {
        if let Some(right) = node.right.take() {
            let new_right = right.left.clone();
            let mut new_root = (*right).clone();
            new_root.left = Some(Arc::new(Node::new_with_children(
                node.value,
                Color::Red,
                node.left,
                new_right,
            )));
            new_root.color = node.color;
            new_root.update_size();
            new_root
        } else {
            node
        }
    }
    
    /// Rotates right
    fn rotate_right(&self, mut node: Node) -> Node {
        if let Some(left) = node.left.take() {
            let new_left = left.right.clone();
            let mut new_root = (*left).clone();
            new_root.right = Some(Arc::new(Node::new_with_children(
                node.value,
                Color::Red,
                new_left,
                node.right,
            )));
            new_root.color = node.color;
            new_root.update_size();
            new_root
        } else {
            node
        }
    }
    
    /// Flips colors
    fn flip_colors(&self, mut node: Node) -> Node {
        node.color = match node.color {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        };
        
        if let Some(left) = &node.left {
            let cloned_left = (**left).clone();
            node.left = Some(Arc::new(cloned_left.make_red()));
        }
        
        if let Some(right) = &node.right {
            let cloned_right = (**right).clone();
            node.right = Some(Arc::new(cloned_right.make_red()));
        }
        
        node
    }
    
    /// Fixes up red-black tree invariants
    fn fix_up(&self, mut node: Node) -> Node {
        // Rotate left if right child is red and left child is not
        if is_red(&node.right) && !is_red(&node.left) {
            node = self.rotate_left(node);
        }
        
        // Rotate right if left child and left-left grandchild are both red
        if is_red(&node.left) && node.left.as_ref().map(|l| is_red(&l.left)).unwrap_or(false) {
            node = self.rotate_right(node);
        }
        
        // Flip colors if both children are red
        if is_red(&node.left) && is_red(&node.right) {
            node = self.flip_colors(node);
        }
        
        node.update_size();
        node
    }
    
    /// Moves red link to the left
    fn move_red_left(&self, mut node: Node) -> Node {
        node = self.flip_colors(node);
        
        if node.right.as_ref().map(|r| is_red(&r.left)).unwrap_or(false) {
            if let Some(right) = node.right.take() {
                node.right = Some(Arc::new(self.rotate_right((*right).clone())));
            }
            node = self.rotate_left(node);
            node = self.flip_colors(node);
        }
        
        node
    }
    
    /// Moves red link to the right
    fn move_red_right(&self, mut node: Node) -> Node {
        node = self.flip_colors(node);
        
        if node.left.as_ref().map(|l| is_red(&l.left)).unwrap_or(false) {
            node = self.rotate_right(node);
            node = self.flip_colors(node);
        }
        
        node
    }
    
    /// Converts the tree to a sorted vector
    fn to_vec(&self) -> Vec<Value> {
        let mut result = Vec::new();
        Self::inorder_traversal(&self.root, &mut result);
        result
    }
    
    /// In-order traversal helper
    fn inorder_traversal(node: &Option<Arc<Node>>, result: &mut Vec<Value>) {
        if let Some(n) = node {
            Self::inorder_traversal(&n.left, result);
            result.push(n.value.clone());
            Self::inorder_traversal(&n.right, result);
        }
    }
    
    /// Gets the minimum value
    fn min(&self) -> Option<Value> {
        Self::find_min(&self.root)
    }
    
    /// Gets the maximum value
    fn max(&self) -> Option<Value> {
        Self::find_max(&self.root)
    }
    
    /// Finds the maximum value in a subtree
    fn find_max(node: &Option<Arc<Node>>) -> Option<Value> {
        match node {
            None => None,
            Some(n) => {
                if n.right.is_none() {
                    Some(n.value.clone())
                } else {
                    Self::find_max(&n.right)
                }
            }
        }
    }
    
    /// Gets all values in a range [low, high]
    fn range(&self, low: &Value, high: &Value) -> Vec<Value> {
        let mut result = Vec::new();
        self.range_search(&self.root, low, high, &mut result);
        result
    }
    
    /// Range search helper
    fn range_search(&self, node: &Option<Arc<Node>>, low: &Value, high: &Value, result: &mut Vec<Value>) {
        if let Some(n) = node {
            let cmp_low = self.comparator.compare(&n.value, low);
            let cmp_high = self.comparator.compare(&n.value, high);
            
            if cmp_low != Ordering::Less {
                self.range_search(&n.left, low, high, result);
            }
            
            if cmp_low != Ordering::Less && cmp_high != Ordering::Greater {
                result.push(n.value.clone());
            }
            
            if cmp_high != Ordering::Greater {
                self.range_search(&n.right, low, high, result);
            }
        }
    }
}

/// Ordered set implementation
#[derive(Clone, Debug)]
pub struct OrderedSet {
    tree: RedBlackTree,
    name: Option<String>,
}

impl OrderedSet {
    /// Creates a new ordered set with default comparator
    pub fn new() -> Self {
        Self {
            tree: RedBlackTree::new(Comparator::with_default()),
            name: None,
        }
    }
    
    /// Creates a new ordered set with custom comparator
    pub fn with_comparator(comparator: Comparator) -> Self {
        Self {
            tree: RedBlackTree::new(comparator),
            name: None,
        }
    }
    
    /// Creates a named ordered set for debugging
    pub fn with_name(name: impl Into<String>) -> Self {
        let mut set = Self::new();
        set.name = Some(name.into());
        set
    }
    
    /// Creates an ordered set from a vector of values
    pub fn from_vec(values: Vec<Value>) -> Self {
        let mut set = Self::new();
        for value in values {
            set.insert(value);
        }
        set
    }
    
    /// Inserts a value into the set
    pub fn insert(&mut self, value: Value) -> bool {
        self.tree.insert(value)
    }
    
    /// Removes a value from the set
    pub fn remove(&mut self, value: &Value) -> bool {
        self.tree.remove(value)
    }
    
    /// Checks if the set contains a value
    pub fn contains(&self, value: &Value) -> bool {
        self.tree.contains(value)
    }
    
    /// Gets the minimum value
    pub fn min(&self) -> Option<Value> {
        self.tree.min()
    }
    
    /// Gets the maximum value
    pub fn max(&self) -> Option<Value> {
        self.tree.max()
    }
    
    /// Gets all values in a range [low, high]
    pub fn range(&self, low: &Value, high: &Value) -> Vec<Value> {
        self.tree.range(low, high)
    }
    
    /// Converts to a sorted vector
    pub fn to_vec(&self) -> Vec<Value> {
        self.tree.to_vec()
    }
    
    /// Returns an iterator over the values in sorted order
    pub fn iter(&self) -> impl Iterator<Item = Value> + '_ {
        self.to_vec().into_iter()
    }
    
    /// Union of two sets
    pub fn union(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for value in other.iter() {
            result.insert(value);
        }
        result
    }
    
    /// Intersection of two sets
    pub fn intersection(&self, other: &Self) -> Self {
        let mut result = Self::with_comparator(self.tree.comparator.clone());
        for value in self.iter() {
            if other.contains(&value) {
                result.insert(value);
            }
        }
        result
    }
    
    /// Difference of two sets (elements in self but not in other)
    pub fn difference(&self, other: &Self) -> Self {
        let mut result = Self::with_comparator(self.tree.comparator.clone());
        for value in self.iter() {
            if !other.contains(&value) {
                result.insert(value);
            }
        }
        result
    }
    
    /// Symmetric difference of two sets
    pub fn symmetric_difference(&self, other: &Self) -> Self {
        let mut result = Self::with_comparator(self.tree.comparator.clone());
        
        for value in self.iter() {
            if !other.contains(&value) {
                result.insert(value);
            }
        }
        
        for value in other.iter() {
            if !self.contains(&value) {
                result.insert(value);
            }
        }
        
        result
    }
    
    /// Checks if this set is a subset of another
    pub fn is_subset(&self, other: &Self) -> bool {
        self.iter().all(|value| other.contains(&value))
    }
    
    /// Checks if this set is a superset of another
    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }
    
    /// Checks if two sets are disjoint
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.iter().all(|value| !other.contains(&value))
    }
    
    /// Filters the set by a predicate
    pub fn filter<F>(&self, mut predicate: F) -> Self
    where
        F: FnMut(&Value) -> bool,
    {
        let mut result = Self::with_comparator(self.tree.comparator.clone());
        for value in self.iter() {
            if predicate(&value) {
                result.insert(value);
            }
        }
        result
    }
    
    /// Maps the set through a function
    pub fn map<F>(&self, mut f: F) -> Self
    where
        F: FnMut(&Value) -> Value,
    {
        let mut result = Self::with_comparator(self.tree.comparator.clone());
        for value in self.iter() {
            result.insert(f(&value));
        }
        result
    }
    
    /// Folds the set
    pub fn fold<F, Acc>(&self, init: Acc, mut f: F) -> Acc
    where
        F: FnMut(Acc, &Value) -> Acc,
    {
        self.iter().fold(init, |acc, value| f(acc, &value))
    }
    
    /// Partitions the set by a predicate
    pub fn partition<F>(&self, mut predicate: F) -> (Self, Self)
    where
        F: FnMut(&Value) -> bool,
    {
        let mut true_set = Self::with_comparator(self.tree.comparator.clone());
        let mut false_set = Self::with_comparator(self.tree.comparator.clone());
        
        for value in self.iter() {
            if predicate(&value) {
                true_set.insert(value);
            } else {
                false_set.insert(value);
            }
        }
        
        (true_set, false_set)
    }
}

impl Container for OrderedSet {
    fn len(&self) -> usize {
        self.tree.size()
    }
    
    fn clear(&mut self) {
        self.tree.root = None;
    }
}

impl Default for OrderedSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe ordered set implementation
#[derive(Clone, Debug)]
pub struct ThreadSafeOrderedSet {
    inner: Arc<RwLock<OrderedSet>>,
}

impl ThreadSafeOrderedSet {
    /// Creates a new thread-safe ordered set
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(OrderedSet::new())),
        }
    }
    
    /// Creates a new thread-safe ordered set with custom comparator
    pub fn with_comparator(comparator: Comparator) -> Self {
        Self {
            inner: Arc::new(RwLock::new(OrderedSet::with_comparator(comparator))),
        }
    }
    
    /// Inserts a value
    pub fn insert(&self, value: Value) -> bool {
        self.inner.write().unwrap().insert(value)
    }
    
    /// Removes a value
    pub fn remove(&self, value: &Value) -> bool {
        self.inner.write().unwrap().remove(value)
    }
    
    /// Checks if the set contains a value
    pub fn contains(&self, value: &Value) -> bool {
        self.inner.read().unwrap().contains(value)
    }
    
    /// Gets the minimum value
    pub fn min(&self) -> Option<Value> {
        self.inner.read().unwrap().min()
    }
    
    /// Gets the maximum value
    pub fn max(&self) -> Option<Value> {
        self.inner.read().unwrap().max()
    }
    
    /// Gets all values in a range
    pub fn range(&self, low: &Value, high: &Value) -> Vec<Value> {
        self.inner.read().unwrap().range(low, high)
    }
    
    /// Returns the number of elements
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
    
    /// Checks if the set is empty
    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }
    
    /// Clears all elements
    pub fn clear(&self) {
        self.inner.write().unwrap().clear();
    }
    
    /// Converts to a vector
    pub fn to_vec(&self) -> Vec<Value> {
        self.inner.read().unwrap().to_vec()
    }
    
    /// Union with another set
    pub fn union(&self, other: &Self) -> Self {
        let self_set = self.inner.read().unwrap();
        let other_set = other.inner.read().unwrap();
        Self {
            inner: Arc::new(RwLock::new(self_set.union(&other_set))),
        }
    }
    
    /// Intersection with another set
    pub fn intersection(&self, other: &Self) -> Self {
        let self_set = self.inner.read().unwrap();
        let other_set = other.inner.read().unwrap();
        Self {
            inner: Arc::new(RwLock::new(self_set.intersection(&other_set))),
        }
    }
    
    /// Difference with another set
    pub fn difference(&self, other: &Self) -> Self {
        let self_set = self.inner.read().unwrap();
        let other_set = other.inner.read().unwrap();
        Self {
            inner: Arc::new(RwLock::new(self_set.difference(&other_set))),
        }
    }
    
    /// Checks if this is a subset of another set
    pub fn is_subset(&self, other: &Self) -> bool {
        let self_set = self.inner.read().unwrap();
        let other_set = other.inner.read().unwrap();
        self_set.is_subset(&other_set)
    }
    
    /// Executes a closure with read access to the inner set
    pub fn with_read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&OrderedSet) -> R,
    {
        f(&self.inner.read().unwrap())
    }
    
    /// Executes a closure with write access to the inner set
    pub fn with_write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut OrderedSet) -> R,
    {
        f(&mut self.inner.write().unwrap())
    }
}

impl Default for ThreadSafeOrderedSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let mut set = OrderedSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        
        // Insert elements
        assert!(set.insert(Value::number(3.0)));
        assert!(set.insert(Value::number(1.0)));
        assert!(set.insert(Value::number(2.0)));
        assert!(!set.insert(Value::number(2.0))); // Duplicate
        
        assert_eq!(set.len(), 3);
        assert!(set.contains(&Value::number(2.0)));
        assert!(!set.contains(&Value::number(4.0)));
        
        // Check ordering
        let values = set.to_vec();
        assert_eq!(values, vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        
        // Remove element
        assert!(set.remove(&Value::number(2.0)));
        assert!(!set.remove(&Value::number(4.0))); // Not present
        assert_eq!(set.len(), 2);
        assert!(!set.contains(&Value::number(2.0)));
    }
    
    #[test]
    fn test_min_max() {
        let mut set = OrderedSet::new();
        assert_eq!(set.min(), None);
        assert_eq!(set.max(), None);
        
        set.insert(Value::number(5.0));
        set.insert(Value::number(1.0));
        set.insert(Value::number(9.0));
        
        assert_eq!(set.min(), Some(Value::number(1.0)));
        assert_eq!(set.max(), Some(Value::number(9.0)));
    }
    
    #[test]
    fn test_range() {
        let mut set = OrderedSet::new();
        for i in 1..=10 {
            set.insert(Value::number(i as f64));
        }
        
        let range = set.range(&Value::number(3.0), &Value::number(7.0));
        assert_eq!(range, vec![
            Value::number(3.0),
            Value::number(4.0),
            Value::number(5.0),
            Value::number(6.0),
            Value::number(7.0),
        ]);
    }
    
    #[test]
    fn test_set_operations() {
        let mut set1 = OrderedSet::new();
        let mut set2 = OrderedSet::new();
        
        for i in 1..=5 {
            set1.insert(Value::number(i as f64));
        }
        
        for i in 3..=7 {
            set2.insert(Value::number(i as f64));
        }
        
        // Union
        let union = set1.union(&set2);
        assert_eq!(union.len(), 7);
        assert!(union.contains(&Value::number(1.0)));
        assert!(union.contains(&Value::number(7.0)));
        
        // Intersection
        let intersection = set1.intersection(&set2);
        assert_eq!(intersection.len(), 3);
        assert!(intersection.contains(&Value::number(3.0)));
        assert!(intersection.contains(&Value::number(4.0)));
        assert!(intersection.contains(&Value::number(5.0)));
        
        // Difference
        let difference = set1.difference(&set2);
        assert_eq!(difference.len(), 2);
        assert!(difference.contains(&Value::number(1.0)));
        assert!(difference.contains(&Value::number(2.0)));
        
        // Symmetric difference
        let sym_diff = set1.symmetric_difference(&set2);
        assert_eq!(sym_diff.len(), 4);
        assert!(sym_diff.contains(&Value::number(1.0)));
        assert!(sym_diff.contains(&Value::number(2.0)));
        assert!(sym_diff.contains(&Value::number(6.0)));
        assert!(sym_diff.contains(&Value::number(7.0)));
    }
    
    #[test]
    fn test_subset_operations() {
        let mut set1 = OrderedSet::new();
        let mut set2 = OrderedSet::new();
        let mut set3 = OrderedSet::new();
        
        for i in 1..=3 {
            set1.insert(Value::number(i as f64));
        }
        
        for i in 1..=5 {
            set2.insert(Value::number(i as f64));
        }
        
        for i in 6..=8 {
            set3.insert(Value::number(i as f64));
        }
        
        assert!(set1.is_subset(&set2));
        assert!(set2.is_superset(&set1));
        assert!(!set1.is_subset(&set3));
        assert!(set1.is_disjoint(&set3));
        assert!(!set1.is_disjoint(&set2));
    }
    
    #[test]
    fn test_functional_operations() {
        let mut set = OrderedSet::new();
        for i in 1..=5 {
            set.insert(Value::number(i as f64));
        }
        
        // Filter
        let evens = set.filter(|v| {
            if let Some(n) = v.as_number() {
                n as i64 % 2 == 0
            } else {
                false
            }
        });
        assert_eq!(evens.len(), 2);
        assert!(evens.contains(&Value::number(2.0)));
        assert!(evens.contains(&Value::number(4.0)));
        
        // Map
        let doubled = set.map(|v| {
            if let Some(n) = v.as_number() {
                Value::number(n * 2.0)
            } else {
                v.clone()
            }
        });
        assert_eq!(doubled.len(), 5);
        assert!(doubled.contains(&Value::number(2.0)));
        assert!(doubled.contains(&Value::number(10.0)));
        
        // Fold
        let sum = set.fold(0.0, |acc, v| {
            acc + v.as_number().unwrap_or(0.0)
        });
        assert_eq!(sum, 15.0);
        
        // Partition
        let (odds, evens) = set.partition(|v| {
            if let Some(n) = v.as_number() {
                n as i64 % 2 == 1
            } else {
                false
            }
        });
        assert_eq!(odds.len(), 3);
        assert_eq!(evens.len(), 2);
    }
    
    #[test]
    fn test_large_set() {
        let mut set = OrderedSet::new();
        
        // Insert many elements
        for i in 0..1000 {
            assert!(set.insert(Value::number((i * 17) as f64 % 1000.0)));
        }
        
        assert_eq!(set.len(), 1000);
        
        // Verify ordering
        let values = set.to_vec();
        for i in 1..values.len() {
            let prev = values[i - 1].as_number().unwrap();
            let curr = values[i].as_number().unwrap();
            assert!(prev < curr);
        }
        
        // Remove half the elements
        for i in 0..500 {
            set.remove(&Value::number(i as f64));
        }
        
        assert_eq!(set.len(), 500);
    }
    
    #[test]
    fn test_thread_safe_ordered_set() {
        let set = ThreadSafeOrderedSet::new();
        
        assert!(set.insert(Value::number(1.0)));
        assert!(set.insert(Value::number(3.0)));
        assert!(set.insert(Value::number(2.0)));
        
        assert_eq!(set.len(), 3);
        assert!(set.contains(&Value::number(2.0)));
        
        let values = set.to_vec();
        assert_eq!(values, vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        
        assert!(set.remove(&Value::number(2.0)));
        assert_eq!(set.len(), 2);
    }
    
    #[test]
    fn test_custom_comparator() {
        // Create set with reverse numeric comparator
        let comparator = Comparator::new(
            "reverse-numeric",
            |a, b| {
                match (a.as_number(), b.as_number()) {
                    (Some(n1), Some(n2)) => n2.partial_cmp(&n1).unwrap_or(std::cmp::Ordering::Equal),
                    _ => std::cmp::Ordering::Equal,
                }
            },
            |v| v.as_number().map(|n| n.to_bits()).unwrap_or(0),
        );
        
        let mut set = OrderedSet::with_comparator(comparator);
        set.insert(Value::number(1.0));
        set.insert(Value::number(3.0));
        set.insert(Value::number(2.0));
        
        // Should be in reverse order
        let values = set.to_vec();
        assert_eq!(values, vec![
            Value::number(3.0),
            Value::number(2.0),
            Value::number(1.0),
        ]);
    }
}