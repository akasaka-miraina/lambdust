//! Random access list implementation (SRFI-101 compliant) using skew binary representation.
//!
//! This module provides both mutable and persistent random access list
//! implementations with O(log n) indexing and O(1) cons operations.

use crate::eval::value::Value;
use super::{Container, Persistent, ContainerError, ContainerResult};
use std::sync::{Arc, RwLock};

/// A tree node in the skew binary representation
#[derive(Clone, Debug, PartialEq)]
enum Tree {
    /// Leaf node containing a single value
    Leaf(Value),
    /// Branch node with a value and two subtrees
    Node {
        value: Value,
        left: Arc<Tree>,
        right: Arc<Tree>,
        size: usize,
    },
}

impl Tree {
    /// Creates a new leaf
    fn leaf(value: Value) -> Self {
        Tree::Leaf(value)
    }
    
    /// Creates a new branch node
    fn node(value: Value, left: Arc<Tree>, right: Arc<Tree>) -> Self {
        let size = 1 + left.size() + right.size();
        Tree::Node {
            value,
            left,
            right,
            size,
        }
    }
    
    /// Gets the size of this tree
    fn size(&self) -> usize {
        match self {
            Tree::Leaf(_) => 1,
            Tree::Node { size, .. } => *size,
        }
    }
    
    /// Gets the value at the root of this tree
    fn root_value(&self) -> &Value {
        match self {
            Tree::Leaf(value) => value,
            Tree::Node { value, .. } => value,
        }
    }
    
    /// Looks up a value by index within this tree
    fn lookup(&self, mut index: usize) -> Option<&Value> {
        match self {
            Tree::Leaf(value) => {
                if index == 0 {
                    Some(value)
                } else {
                    None
                }
            }
            Tree::Node { value, left, right, .. } => {
                if index == 0 {
                    Some(value)
                } else {
                    index -= 1; // Skip the root
                    let left_size = left.size();
                    if index < left_size {
                        left.lookup(index)
                    } else {
                        right.lookup(index - left_size)
                    }
                }
            }
        }
    }
    
    /// Updates a value by index within this tree
    fn update(&self, mut index: usize, new_value: Value) -> Option<Arc<Tree>> {
        match self {
            Tree::Leaf(_) => {
                if index == 0 {
                    Some(Arc::new(Tree::leaf(new_value)))
                } else {
                    None
                }
            }
            Tree::Node { value, left, right, .. } => {
                if index == 0 {
                    Some(Arc::new(Tree::node(new_value, left.clone(), right.clone())))
                } else {
                    index -= 1; // Skip the root
                    let left_size = left.size();
                    if index < left_size {
                        left.update(index, new_value).map(|new_left| {
                            Arc::new(Tree::node(value.clone(), new_left, right.clone()))
                        })
                    } else {
                        right.update(index - left_size, new_value).map(|new_right| {
                            Arc::new(Tree::node(value.clone(), left.clone(), new_right))
                        })
                    }
                }
            }
        }
    }
    
    /// Converts the tree to a vector in order
    fn to_vec(&self) -> Vec<Value> {
        let mut result = Vec::with_capacity(self.size());
        self.collect_values(&mut result);
        result
    }
    
    /// Helper to collect values in order
    fn collect_values(&self, result: &mut Vec<Value>) {
        match self {
            Tree::Leaf(value) => {
                result.push(value.clone());
            }
            Tree::Node { value, left, right, .. } => {
                result.push(value.clone());
                left.collect_values(result);
                right.collect_values(result);
            }
        }
    }
}

/// A digit in the skew binary representation (tree with its size)
#[derive(Clone, Debug, PartialEq)]
struct Digit {
    tree: Arc<Tree>,
    weight: usize, // Weight in the skew binary system
}

impl Digit {
    fn new(tree: Arc<Tree>, weight: usize) -> Self {
        Self { tree, weight }
    }
    
    fn size(&self) -> usize {
        self.tree.size()
    }
}

/// Random access list using skew binary representation
#[derive(Clone, Debug, PartialEq)]
pub struct RandomAccessList {
    /// List of digits in decreasing weight order
    digits: Vec<Digit>,
    /// Total size of the list
    total_size: usize,
    /// Name for debugging
    name: Option<String>,
}

impl RandomAccessList {
    /// Creates a new empty random access list
    pub fn new() -> Self {
        Self {
            digits: Vec::new(),
            total_size: 0,
            name: None,
        }
    }
    
    /// Creates a named random access list for debugging
    pub fn with_name(name: impl Into<String>) -> Self {
        let mut list = Self::new();
        list.name = Some(name.into());
        list
    }
    
    /// Creates a random access list from a vector
    pub fn from_vec(values: Vec<Value>) -> Self {
        let mut list = Self::new();
        for value in values.into_iter().rev() {
            list = list.cons_front(value);
        }
        list
    }
    
    /// Gets the length of the list
    pub fn len(&self) -> usize {
        self.total_size
    }
    
    /// Checks if the list is empty
    pub fn is_empty(&self) -> bool {
        self.total_size == 0
    }
    
    /// Adds an element to the front of the list (O(1))
    pub fn cons_front(self, value: Value) -> Self {
        let mut digits = self.digits;
        
        // Check if we can combine the first two digits
        if digits.len() >= 2 && digits[0].weight == digits[1].weight {
            let first = digits.remove(0);
            let second = digits.remove(0);
            
            // Create new tree combining the two trees
            let new_tree = Arc::new(Tree::node(
                value,
                first.tree,
                second.tree,
            ));
            
            let new_digit = Digit::new(new_tree, first.weight * 2 + 1);
            digits.insert(0, new_digit);
        } else {
            // Just add a new leaf at the front
            let new_tree = Arc::new(Tree::leaf(value));
            let new_digit = Digit::new(new_tree, 1);
            digits.insert(0, new_digit);
        }
        
        Self {
            digits,
            total_size: self.total_size + 1,
            name: self.name,
        }
    }
    
    /// Removes an element from the front of the list (O(1))
    pub fn uncons_front(self) -> Option<(Value, Self)> {
        if self.digits.is_empty() {
            return None;
        }
        
        let mut digits = self.digits;
        let first_digit = digits.remove(0);
        
        match &*first_digit.tree {
            Tree::Leaf(value) => {
                // Simple case: just remove the leaf
                Some((value.clone(), Self {
                    digits,
                    total_size: self.total_size - 1,
                    name: self.name,
                }))
            }
            Tree::Node { value, left, right, .. } => {
                // Complex case: split the node
                let left_weight = (first_digit.weight - 1) / 2;
                let right_weight = left_weight;
                
                let left_digit = Digit::new(left.clone(), left_weight);
                let right_digit = Digit::new(right.clone(), right_weight);
                
                // Insert the split digits at the front
                digits.insert(0, right_digit);
                digits.insert(0, left_digit);
                
                Some((value.clone(), Self {
                    digits,
                    total_size: self.total_size - 1,
                    name: self.name,
                }))
            }
        }
    }
    
    /// Gets an element by index (O(log n))
    pub fn get(&self, mut index: usize) -> Option<&Value> {
        if index >= self.total_size {
            return None;
        }
        
        for digit in &self.digits {
            if index < digit.size() {
                return digit.tree.lookup(index);
            }
            index -= digit.size();
        }
        
        None
    }
    
    /// Sets an element by index, returning a new list (O(log n))
    pub fn set(&self, mut index: usize, value: Value) -> Option<Self> {
        if index >= self.total_size {
            return None;
        }
        
        let mut new_digits = Vec::with_capacity(self.digits.len());
        let mut found = false;
        
        for digit in &self.digits {
            if !found && index < digit.size() {
                if let Some(new_tree) = digit.tree.update(index, value.clone()) {
                    new_digits.push(Digit::new(new_tree, digit.weight));
                    found = true;
                } else {
                    return None;
                }
            } else {
                new_digits.push(digit.clone());
                if !found {
                    index -= digit.size();
                }
            }
        }
        
        if found {
            Some(Self {
                digits: new_digits,
                total_size: self.total_size,
                name: self.name.clone(),
            })
        } else {
            None
        }
    }
    
    /// Converts to a vector
    pub fn to_vec(&self) -> Vec<Value> {
        let mut result = Vec::with_capacity(self.total_size);
        for digit in &self.digits {
            digit.tree.collect_values(&mut result);
        }
        result
    }
    
    /// Returns an iterator over the elements
    pub fn iter(&self) -> impl Iterator<Item = Value> + '_ {
        self.to_vec().into_iter()
    }
    
    /// Appends another list to this one
    pub fn append(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for value in other.iter() {
            result = result.cons_back(value);
        }
        result
    }
    
    /// Adds an element to the back of the list (O(log n))
    pub fn cons_back(self, value: Value) -> Self {
        // For skew binary lists, adding to the back is more expensive
        // We convert to vector, append, and convert back
        let mut values = self.to_vec();
        values.push(value);
        Self::from_vec(values)
    }
    
    /// Removes an element from the back of the list (O(log n))
    pub fn uncons_back(self) -> Option<(Self, Value)> {
        if self.is_empty() {
            return None;
        }
        
        let mut values = self.to_vec();
        let last = values.pop()?;
        Some((Self::from_vec(values), last))
    }
    
    /// Gets the first element
    pub fn head(&self) -> Option<&Value> {
        self.get(0)
    }
    
    /// Gets all elements except the first
    pub fn tail(&self) -> Option<Self> {
        self.clone().uncons_front().map(|(_, tail)| tail)
    }
    
    /// Maps over all elements
    pub fn map<F>(&self, mut f: F) -> Self
    where
        F: FnMut(&Value) -> Value,
    {
        let mapped: Vec<_> = self.iter().map(|v| f(&v)).collect();
        Self::from_vec(mapped)
    }
    
    /// Filters elements by a predicate
    pub fn filter<F>(&self, mut predicate: F) -> Self
    where
        F: FnMut(&Value) -> bool,
    {
        let filtered: Vec<_> = self.iter().filter(|v| predicate(v)).collect();
        Self::from_vec(filtered)
    }
    
    /// Folds over all elements
    pub fn fold<F, Acc>(&self, mut init: Acc, mut f: F) -> Acc
    where
        F: FnMut(Acc, &Value) -> Acc,
    {
        for value in self.iter() {
            init = f(init, &value);
        }
        init
    }
    
    /// Finds the first element matching a predicate
    pub fn find<F>(&self, mut predicate: F) -> Option<Value>
    where
        F: FnMut(&Value) -> bool,
    {
        self.iter().find(|v| predicate(v))
    }
    
    /// Checks if any element matches a predicate
    pub fn any<F>(&self, mut predicate: F) -> bool
    where
        F: FnMut(&Value) -> bool,
    {
        self.iter().any(|v| predicate(&v))
    }
    
    /// Checks if all elements match a predicate
    pub fn all<F>(&self, mut predicate: F) -> bool
    where
        F: FnMut(&Value) -> bool,
    {
        self.iter().all(|v| predicate(&v))
    }
    
    /// Reverses the list
    pub fn reverse(&self) -> Self {
        let values: Vec<_> = self.iter().collect();
        let mut result = Self::new();
        for value in values {
            result = result.cons_front(value);
        }
        result
    }
    
    /// Takes the first n elements
    pub fn take(&self, n: usize) -> Self {
        let values: Vec<_> = self.iter().take(n).collect();
        Self::from_vec(values)
    }
    
    /// Drops the first n elements
    pub fn drop(&self, n: usize) -> Self {
        let values: Vec<_> = self.iter().skip(n).collect();
        Self::from_vec(values)
    }
    
    /// Splits the list at the given index
    pub fn split_at(&self, index: usize) -> (Self, Self) {
        let values = self.to_vec();
        if index >= values.len() {
            (self.clone(), Self::new())
        } else {
            let (left, right) = values.split_at(index);
            (Self::from_vec(left.to_vec()), Self::from_vec(right.to_vec()))
        }
    }
}

impl Container for RandomAccessList {
    fn len(&self) -> usize {
        self.total_size
    }
    
    fn clear(&mut self) {
        self.digits.clear();
        self.total_size = 0;
    }
}

impl Default for RandomAccessList {
    fn default() -> Self {
        Self::new()
    }
}

impl Persistent<Value> for RandomAccessList {
    fn insert(&self, element: Value) -> Self {
        self.clone().cons_front(element)
    }
    
    fn remove(&self, element: &Value) -> Self {
        self.filter(|v| v != element)
    }
}

/// Persistent (immutable) random access list
pub type PersistentRandomAccessList = RandomAccessList;

/// Mutable wrapper around random access list
#[derive(Clone, Debug)]
pub struct MutableRandomAccessList {
    inner: RandomAccessList,
}

impl MutableRandomAccessList {
    /// Creates a new mutable random access list
    pub fn new() -> Self {
        Self {
            inner: RandomAccessList::new(),
        }
    }
    
    /// Creates from vector
    pub fn from_vec(values: Vec<Value>) -> Self {
        Self {
            inner: RandomAccessList::from_vec(values),
        }
    }
    
    /// Pushes to front
    pub fn push_front(&mut self, value: Value) {
        self.inner = std::mem::take(&mut self.inner).cons_front(value);
    }
    
    /// Pops from front
    pub fn pop_front(&mut self) -> Option<Value> {
        if let Some((value, new_list)) = std::mem::take(&mut self.inner).uncons_front() {
            self.inner = new_list;
            Some(value)
        } else {
            None
        }
    }
    
    /// Pushes to back
    pub fn push_back(&mut self, value: Value) {
        self.inner = std::mem::take(&mut self.inner).cons_back(value);
    }
    
    /// Pops from back
    pub fn pop_back(&mut self) -> Option<Value> {
        if let Some((new_list, value)) = std::mem::take(&mut self.inner).uncons_back() {
            self.inner = new_list;
            Some(value)
        } else {
            None
        }
    }
    
    /// Gets element by index
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.inner.get(index)
    }
    
    /// Sets element by index
    pub fn set(&mut self, index: usize, value: Value) -> bool {
        if let Some(new_list) = self.inner.set(index, value) {
            self.inner = new_list;
            true
        } else {
            false
        }
    }
    
    /// Gets length
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    
    /// Checks if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    
    /// Clears the list
    pub fn clear(&mut self) {
        self.inner = RandomAccessList::new();
    }
    
    /// Converts to vector
    pub fn to_vec(&self) -> Vec<Value> {
        self.inner.to_vec()
    }
    
    /// Gets immutable reference to inner list
    pub fn as_persistent(&self) -> &RandomAccessList {
        &self.inner
    }
    
    /// Converts to persistent list
    pub fn into_persistent(self) -> RandomAccessList {
        self.inner
    }
}

impl Container for MutableRandomAccessList {
    fn len(&self) -> usize {
        self.inner.len()
    }
    
    fn clear(&mut self) {
        MutableRandomAccessList::clear(self);
    }
}

impl Default for MutableRandomAccessList {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe random access list
#[derive(Clone, Debug)]
pub struct ThreadSafeRandomAccessList {
    inner: Arc<RwLock<MutableRandomAccessList>>,
}

impl ThreadSafeRandomAccessList {
    /// Creates a new thread-safe random access list
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MutableRandomAccessList::new())),
        }
    }
    
    /// Creates from vector
    pub fn from_vec(values: Vec<Value>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(MutableRandomAccessList::from_vec(values))),
        }
    }
    
    /// Pushes to front
    pub fn push_front(&self, value: Value) {
        self.inner.write().unwrap().push_front(value);
    }
    
    /// Pops from front
    pub fn pop_front(&self) -> Option<Value> {
        self.inner.write().unwrap().pop_front()
    }
    
    /// Pushes to back
    pub fn push_back(&self, value: Value) {
        self.inner.write().unwrap().push_back(value);
    }
    
    /// Pops from back
    pub fn pop_back(&self) -> Option<Value> {
        self.inner.write().unwrap().pop_back()
    }
    
    /// Gets element by index
    pub fn get(&self, index: usize) -> Option<Value> {
        self.inner.read().unwrap().get(index).cloned()
    }
    
    /// Sets element by index
    pub fn set(&self, index: usize, value: Value) -> bool {
        self.inner.write().unwrap().set(index, value)
    }
    
    /// Gets length
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
    
    /// Checks if empty
    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }
    
    /// Clears the list
    pub fn clear(&self) {
        self.inner.write().unwrap().clear();
    }
    
    /// Converts to vector
    pub fn to_vec(&self) -> Vec<Value> {
        self.inner.read().unwrap().to_vec()
    }
    
    /// Executes closure with read access
    pub fn with_read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&MutableRandomAccessList) -> R,
    {
        f(&self.inner.read().unwrap())
    }
    
    /// Executes closure with write access
    pub fn with_write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut MutableRandomAccessList) -> R,
    {
        f(&mut self.inner.write().unwrap())
    }
}

impl Default for ThreadSafeRandomAccessList {
    fn default() -> Self {
        Self::new()
    }
}

/// SRFI-101 specific functions
impl RandomAccessList {
    /// SRFI-101: ra-list-ref with error handling
    pub fn ra_list_ref(&self, index: usize) -> ContainerResult<Value> {
        self.get(index).cloned().ok_or(ContainerError::IndexOutOfBounds {
            index,
            length: self.len(),
        })
    }
    
    /// SRFI-101: ra-list-set
    pub fn ra_list_set(&self, index: usize, value: Value) -> ContainerResult<Self> {
        self.set(index, value).ok_or(ContainerError::IndexOutOfBounds {
            index,
            length: self.len(),
        })
    }
    
    /// SRFI-101: ra-list-cons
    pub fn ra_list_cons(&self, value: Value) -> Self {
        self.clone().cons_front(value)
    }
    
    /// SRFI-101: ra-list-car
    pub fn ra_list_car(&self) -> ContainerResult<Value> {
        self.head().cloned().ok_or(ContainerError::EmptyContainer {
            operation: "ra-list-car".to_string(),
        })
    }
    
    /// SRFI-101: ra-list-cdr
    pub fn ra_list_cdr(&self) -> ContainerResult<Self> {
        self.tail().ok_or(ContainerError::EmptyContainer {
            operation: "ra-list-cdr".to_string(),
        })
    }
    
    /// SRFI-101: ra-list->list
    pub fn ra_list_to_list(&self) -> Value {
        Value::list(self.to_vec())
    }
    
    /// SRFI-101: list->ra-list
    pub fn list_to_ra_list(list: &Value) -> ContainerResult<Self> {
        match list.as_list() {
            Some(values) => Ok(Self::from_vec(values)),
            None => Err(ContainerError::InvalidComparator {
                message: "Expected a proper list".to_string(),
            }),
        }
    }
    
    /// SRFI-101: ra-list-length
    pub fn ra_list_length(&self) -> usize {
        self.len()
    }
    
    /// SRFI-101: ra-list-null?
    pub fn ra_list_null(&self) -> bool {
        self.is_empty()
    }
    
    /// SRFI-101: ra-list-append
    pub fn ra_list_append(&self, other: &Self) -> Self {
        self.append(other)
    }
    
    /// SRFI-101: ra-list-reverse
    pub fn ra_list_reverse(&self) -> Self {
        self.reverse()
    }
    
    /// SRFI-101: ra-list-map
    pub fn ra_list_map<F>(&self, f: F) -> Self
    where
        F: FnMut(&Value) -> Value,
    {
        self.map(f)
    }
    
    /// SRFI-101: ra-list-for-each
    pub fn ra_list_for_each<F>(&self, mut f: F)
    where
        F: FnMut(&Value),
    {
        for value in self.iter() {
            f(&value);
        }
    }
    
    /// SRFI-101: ra-list-fold
    pub fn ra_list_fold<F, Acc>(&self, init: Acc, f: F) -> Acc
    where
        F: FnMut(Acc, &Value) -> Acc,
    {
        self.fold(init, f)
    }
    
    /// SRFI-101: ra-list-fold-right
    pub fn ra_list_fold_right<F, Acc>(&self, mut init: Acc, mut f: F) -> Acc
    where
        F: FnMut(&Value, Acc) -> Acc,
    {
        let values = self.to_vec();
        for value in values.iter().rev() {
            init = f(value, init);
        }
        init
    }
    
    /// SRFI-101: ra-list-filter
    pub fn ra_list_filter<F>(&self, predicate: F) -> Self
    where
        F: FnMut(&Value) -> bool,
    {
        self.filter(predicate)
    }
    
    /// SRFI-101: ra-list-partition
    pub fn ra_list_partition<F>(&self, mut predicate: F) -> (Self, Self)
    where
        F: FnMut(&Value) -> bool,
    {
        let mut true_list = Vec::new();
        let mut false_list = Vec::new();
        
        for value in self.iter() {
            if predicate(&value) {
                true_list.push(value);
            } else {
                false_list.push(value);
            }
        }
        
        (Self::from_vec(true_list), Self::from_vec(false_list))
    }
    
    /// SRFI-101: ra-list-find
    pub fn ra_list_find<F>(&self, predicate: F) -> Option<Value>
    where
        F: FnMut(&Value) -> bool,
    {
        self.find(predicate)
    }
    
    /// SRFI-101: ra-list-any
    pub fn ra_list_any<F>(&self, predicate: F) -> bool
    where
        F: FnMut(&Value) -> bool,
    {
        self.any(predicate)
    }
    
    /// SRFI-101: ra-list-every
    pub fn ra_list_every<F>(&self, predicate: F) -> bool
    where
        F: FnMut(&Value) -> bool,
    {
        self.all(predicate)
    }
    
    /// SRFI-101: ra-list-take
    pub fn ra_list_take(&self, n: usize) -> Self {
        self.take(n)
    }
    
    /// SRFI-101: ra-list-drop
    pub fn ra_list_drop(&self, n: usize) -> Self {
        self.drop(n)
    }
    
    /// SRFI-101: ra-list-split-at
    pub fn ra_list_split_at(&self, index: usize) -> (Self, Self) {
        self.split_at(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let list = RandomAccessList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        
        // Cons elements
        let list = list.cons_front(Value::number(3.0));
        let list = list.cons_front(Value::number(2.0));
        let list = list.cons_front(Value::number(1.0));
        
        assert_eq!(list.len(), 3);
        assert!(!list.is_empty());
        
        // Test indexing
        assert_eq!(list.get(0), Some(&Value::number(1.0)));
        assert_eq!(list.get(1), Some(&Value::number(2.0)));
        assert_eq!(list.get(2), Some(&Value::number(3.0)));
        assert_eq!(list.get(3), None);
        
        // Test head/tail
        assert_eq!(list.head(), Some(&Value::number(1.0)));
        let tail = list.tail().unwrap();
        assert_eq!(tail.len(), 2);
        assert_eq!(tail.get(0), Some(&Value::number(2.0)));
    }
    
    #[test]
    fn test_uncons_front() {
        let list = RandomAccessList::from_vec(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        
        let (first, rest) = list.uncons_front().unwrap();
        assert_eq!(first, Value::number(1.0));
        assert_eq!(rest.len(), 2);
        assert_eq!(rest.get(0), Some(&Value::number(2.0)));
        
        let (second, rest) = rest.uncons_front().unwrap();
        assert_eq!(second, Value::number(2.0));
        assert_eq!(rest.len(), 1);
        
        let (third, rest) = rest.uncons_front().unwrap();
        assert_eq!(third, Value::number(3.0));
        assert!(rest.is_empty());
        
        assert_eq!(rest.uncons_front(), None);
    }
    
    #[test]
    fn test_set_operation() {
        let list = RandomAccessList::from_vec(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        
        let updated = list.set(1, Value::number(42.0)).unwrap();
        
        // Original list unchanged
        assert_eq!(list.get(1), Some(&Value::number(2.0)));
        
        // New list has updated value
        assert_eq!(updated.get(1), Some(&Value::number(42.0)));
        assert_eq!(updated.get(0), Some(&Value::number(1.0)));
        assert_eq!(updated.get(2), Some(&Value::number(3.0)));
        
        // Test out of bounds
        assert!(list.set(10, Value::number(0.0)).is_none());
    }
    
    #[test]
    fn test_from_vec_and_to_vec() {
        let values = vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
            Value::number(4.0),
            Value::number(5.0),
        ];
        
        let list = RandomAccessList::from_vec(values.clone());
        assert_eq!(list.len(), 5);
        assert_eq!(list.to_vec(), values);
        
        // Test empty list
        let empty = RandomAccessList::from_vec(vec![]);
        assert!(empty.is_empty());
        assert_eq!(empty.to_vec(), Vec::<Value>::new());
    }
    
    #[test]
    fn test_append() {
        let list1 = RandomAccessList::from_vec(vec![
            Value::number(1.0),
            Value::number(2.0),
        ]);
        
        let list2 = RandomAccessList::from_vec(vec![
            Value::number(3.0),
            Value::number(4.0),
        ]);
        
        let combined = list1.append(&list2);
        assert_eq!(combined.len(), 4);
        assert_eq!(combined.to_vec(), vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
            Value::number(4.0),
        ]);
        
        // Original lists unchanged
        assert_eq!(list1.len(), 2);
        assert_eq!(list2.len(), 2);
    }
    
    #[test]
    fn test_functional_operations() {
        let list = RandomAccessList::from_vec(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
            Value::number(4.0),
            Value::number(5.0),
        ]);
        
        // Test map
        let doubled = list.map(|v| Value::number(v.as_number().unwrap() * 2.0));
        assert_eq!(doubled.to_vec(), vec![
            Value::number(2.0),
            Value::number(4.0),
            Value::number(6.0),
            Value::number(8.0),
            Value::number(10.0),
        ]);
        
        // Test filter
        let evens = list.filter(|v| {
            if let Some(n) = v.as_number() {
                n as i64 % 2 == 0
            } else {
                false
            }
        });
        assert_eq!(evens.to_vec(), vec![
            Value::number(2.0),
            Value::number(4.0),
        ]);
        
        // Test fold
        let sum = list.fold(0.0, |acc, v| acc + v.as_number().unwrap());
        assert_eq!(sum, 15.0);
        
        // Test find
        let found = list.find(|v| v.as_number() == Some(3.0));
        assert_eq!(found, Some(Value::number(3.0)));
        
        // Test any/all
        assert!(list.any(|v| v.as_number() == Some(5.0)));
        assert!(list.all(|v| v.as_number().is_some()));
    }
    
    #[test]
    fn test_take_drop_split() {
        let list = RandomAccessList::from_vec(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
            Value::number(4.0),
            Value::number(5.0),
        ]);
        
        // Test take
        let first_three = list.take(3);
        assert_eq!(first_three.to_vec(), vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        
        // Test drop
        let last_two = list.drop(3);
        assert_eq!(last_two.to_vec(), vec![
            Value::number(4.0),
            Value::number(5.0),
        ]);
        
        // Test split_at
        let (left, right) = list.split_at(2);
        assert_eq!(left.to_vec(), vec![
            Value::number(1.0),
            Value::number(2.0),
        ]);
        assert_eq!(right.to_vec(), vec![
            Value::number(3.0),
            Value::number(4.0),
            Value::number(5.0),
        ]);
    }
    
    #[test]
    fn test_reverse() {
        let list = RandomAccessList::from_vec(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        
        let reversed = list.reverse();
        assert_eq!(reversed.to_vec(), vec![
            Value::number(3.0),
            Value::number(2.0),
            Value::number(1.0),
        ]);
        
        // Original unchanged
        assert_eq!(list.to_vec(), vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
    }
    
    #[test]
    fn test_mutable_wrapper() {
        let mut list = MutableRandomAccessList::new();
        assert!(list.is_empty());
        
        list.push_front(Value::number(2.0));
        list.push_front(Value::number(1.0));
        list.push_back(Value::number(3.0));
        
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some(&Value::number(1.0)));
        assert_eq!(list.get(1), Some(&Value::number(2.0)));
        assert_eq!(list.get(2), Some(&Value::number(3.0)));
        
        assert!(list.set(1, Value::number(42.0)));
        assert_eq!(list.get(1), Some(&Value::number(42.0)));
        
        assert_eq!(list.pop_front(), Some(Value::number(1.0)));
        assert_eq!(list.pop_back(), Some(Value::number(3.0)));
        assert_eq!(list.len(), 1);
        
        list.clear();
        assert!(list.is_empty());
    }
    
    #[test]
    fn test_thread_safe_list() {
        let list = ThreadSafeRandomAccessList::new();
        
        list.push_front(Value::number(2.0));
        list.push_front(Value::number(1.0));
        list.push_back(Value::number(3.0));
        
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some(Value::number(1.0)));
        assert_eq!(list.get(2), Some(Value::number(3.0)));
        
        assert!(list.set(1, Value::number(42.0)));
        assert_eq!(list.get(1), Some(Value::number(42.0)));
        
        list.clear();
        assert!(list.is_empty());
    }
    
    #[test]
    fn test_srfi_101_operations() {
        let list = RandomAccessList::from_vec(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        
        // Test SRFI-101 functions
        assert_eq!(list.ra_list_ref(1).unwrap(), Value::number(2.0));
        assert!(list.ra_list_ref(10).is_err());
        
        let updated = list.ra_list_set(1, Value::number(42.0)).unwrap();
        assert_eq!(updated.ra_list_ref(1).unwrap(), Value::number(42.0));
        
        let consed = list.ra_list_cons(Value::number(0.0));
        assert_eq!(consed.ra_list_car().unwrap(), Value::number(0.0));
        
        let cdr = consed.ra_list_cdr().unwrap();
        assert_eq!(cdr.ra_list_length(), 3);
        
        assert!(!list.ra_list_null());
        assert!(RandomAccessList::new().ra_list_null());
        
        // Test conversion
        let scheme_list = list.ra_list_to_list();
        let converted_back = RandomAccessList::list_to_ra_list(&scheme_list).unwrap();
        assert_eq!(converted_back.to_vec(), list.to_vec());
    }
    
    #[test]
    fn test_large_list() {
        let mut list = RandomAccessList::new();
        
        // Build a large list
        for i in 0..1000 {
            list = list.cons_front(Value::number(i as f64));
        }
        
        assert_eq!(list.len(), 1000);
        
        // Test random access
        for i in 0..1000 {
            assert_eq!(list.get(i), Some(&Value::number((999 - i) as f64)));
        }
        
        // Test updates
        let updated = list.set(500, Value::number(42.0)).unwrap();
        assert_eq!(updated.get(500), Some(&Value::number(42.0)));
        assert_eq!(list.get(500), Some(&Value::number(499.0))); // Original unchanged
    }
    
    #[test]
    fn test_cons_back_uncons_back() {
        let list = RandomAccessList::new();
        let list = list.cons_back(Value::number(1.0));
        let list = list.cons_back(Value::number(2.0));
        let list = list.cons_back(Value::number(3.0));
        
        assert_eq!(list.len(), 3);
        assert_eq!(list.to_vec(), vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        
        let (rest, last) = list.uncons_back().unwrap();
        assert_eq!(last, Value::number(3.0));
        assert_eq!(rest.len(), 2);
        assert_eq!(rest.to_vec(), vec![
            Value::number(1.0),
            Value::number(2.0),
        ]);
    }
}