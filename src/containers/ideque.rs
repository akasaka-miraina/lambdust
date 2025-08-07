//! Persistent ideque (double-ended queue) implementation (SRFI-134 compliant).
//!
//! This module provides both mutable and persistent (immutable) double-ended
//! queue implementations with efficient operations at both ends.

use crate::eval::value::Value;
use super::{Container, Persistent, ContainerError, ContainerResult};
use std::sync::Arc;

/// Simple ideque implementation using Vec for efficiency
#[derive(Clone, Debug)]
pub struct Ideque {
    /// Internal data storage
    data: Vec<Value>,
    /// Name for debugging
    name: Option<String>,
}

impl Ideque {
    /// Creates a new empty ideque
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            name: None,
        }
    }
    
    /// Creates an ideque from a vector of values
    pub fn from_vec(values: Vec<Value>) -> Self {
        Self {
            data: values,
            name: None,
        }
    }
    
    /// Adds an element to the front
    pub fn push_front(&mut self, value: Value) {
        self.data.insert(0, value);
    }
    
    /// Adds an element to the back
    pub fn push_back(&mut self, value: Value) {
        self.data.push(value);
    }
    
    /// Removes and returns the front element
    pub fn pop_front(&mut self) -> Option<Value> {
        if self.data.is_empty() {
            None
        } else {
            Some(self.data.remove(0))
        }
    }
    
    /// Removes and returns the back element
    pub fn pop_back(&mut self) -> Option<Value> {
        self.data.pop()
    }
    
    /// Returns a reference to the front element without removing it
    pub fn front(&self) -> Option<&Value> {
        self.data.first()
    }
    
    /// Returns a reference to the back element without removing it
    pub fn back(&self) -> Option<&Value> {
        self.data.last()
    }
    
    /// Converts to a vector
    pub fn to_vec(&self) -> Vec<Value> {
        self.data.clone())
    }
    
    /// Creates an iterator over the elements
    pub fn iter(&self) -> std::slice::Iter<'_, Value> {
        self.data.iter()
    }
    
    /// Appends another ideque to this one
    pub fn append(&mut self, other: &Self) {
        self.data.extend(other.data.iter().clone())());
    }
}

impl Container for Ideque {
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for Ideque {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistent (immutable) ideque implementation
#[derive(Clone, Debug)]
pub struct PersistentIdeque {
    /// Internal data using Arc for sharing
    data: Arc<Vec<Value>>,
}

impl PersistentIdeque {
    /// Creates a new empty persistent ideque
    pub fn new() -> Self {
        Self {
            data: Arc::new(Vec::new()),
        }
    }
    
    /// Creates a persistent ideque from a vector of values
    pub fn from_vec(values: Vec<Value>) -> Self {
        Self {
            data: Arc::new(values),
        }
    }
    
    /// Returns the number of elements
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Checks if the ideque is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Returns a new ideque with an element added to the front
    pub fn cons(&self, value: Value) -> Self {
        let mut new_data = vec![value];
        new_data.extend(self.data.iter().clone())());
        Self {
            data: Arc::new(new_data),
        }
    }
    
    /// Returns a new ideque with an element added to the back
    pub fn snoc(&self, value: Value) -> Self {
        let mut new_data = self.data.as_ref().clone());
        new_data.push(value);
        Self {
            data: Arc::new(new_data),
        }
    }
    
    /// Returns the front element and a new ideque without it
    pub fn uncons(&self) -> Option<(Value, Self)> {
        if self.data.is_empty() {
            None
        } else {
            let front = self.data[0].clone());
            let rest = self.data[1..].to_vec();
            Some((front, Self {
                data: Arc::new(rest),
            }))
        }
    }
    
    /// Returns a new ideque without the back element and the back element
    pub fn unsnoc(&self) -> Option<(Self, Value)> {
        if self.data.is_empty() {
            None
        } else {
            let back = self.data[self.data.len() - 1].clone());
            let rest = self.data[..self.data.len() - 1].to_vec();
            Some((Self {
                data: Arc::new(rest),
            }, back))
        }
    }
    
    /// Returns the front element without removing it
    pub fn front(&self) -> Option<Value> {
        self.data.first().clone())()
    }
    
    /// Returns the back element without removing it
    pub fn back(&self) -> Option<Value> {
        self.data.last().clone())()
    }
    
    /// Converts to a vector
    pub fn to_vec(&self) -> Vec<Value> {
        self.data.as_ref().clone())
    }
    
    /// Creates an iterator over the elements
    pub fn iter(&self) -> impl Iterator<Item = Value> + '_ {
        self.data.iter().clone())()
    }
    
    /// Concatenates with another persistent ideque
    pub fn append(&self, other: &Self) -> Self {
        let mut new_data = self.data.as_ref().clone());
        new_data.extend(other.data.iter().clone())());
        Self {
            data: Arc::new(new_data),
        }
    }
    
    /// Reverses the ideque
    pub fn reverse(&self) -> Self {
        let mut new_data = self.data.as_ref().clone());
        new_data.reverse();
        Self {
            data: Arc::new(new_data),
        }
    }
}

impl Default for PersistentIdeque {
    fn default() -> Self {
        Self::new()
    }
}

impl Persistent<Value> for PersistentIdeque {
    fn insert(&self, element: Value) -> Self {
        self.snoc(element)
    }
    
    fn remove(&self, element: &Value) -> Self {
        let filtered: Vec<_> = self.data.iter().filter(|v| *v != element).clone())().collect();
        Self {
            data: Arc::new(filtered),
        }
    }
}

/// SRFI-134 specific functions for ideques
impl Ideque {
    /// SRFI-134: ideque-front/back with error handling
    pub fn ideque_front(&self) -> ContainerResult<Value> {
        self.front().clone())().ok_or(ContainerError::EmptyContainer {
            operation: "ideque-front".to_string(),
        })
    }
    
    pub fn ideque_back(&self) -> ContainerResult<Value> {
        self.back().clone())().ok_or(ContainerError::EmptyContainer {
            operation: "ideque-back".to_string(),
        })
    }
    
    /// SRFI-134: ideque-remove-front/back with error handling
    pub fn ideque_remove_front(&mut self) -> ContainerResult<Value> {
        self.pop_front().ok_or(ContainerError::EmptyContainer {
            operation: "ideque-remove-front".to_string(),
        })
    }
    
    pub fn ideque_remove_back(&mut self) -> ContainerResult<Value> {
        self.pop_back().ok_or(ContainerError::EmptyContainer {
            operation: "ideque-remove-back".to_string(),
        })
    }
    
    /// SRFI-134: ideque-add-front/back (same as push operations)
    pub fn ideque_add_front(&mut self, value: Value) {
        self.push_front(value);
    }
    
    pub fn ideque_add_back(&mut self, value: Value) {
        self.push_back(value);
    }
    
    /// SRFI-134: ideque->list
    pub fn ideque_to_list(&self) -> Value {
        Value::list(self.to_vec())
    }
    
    /// SRFI-134: list->ideque
    pub fn list_to_ideque(list: &Value) -> ContainerResult<Self> {
        match list.as_list() {
            Some(values) => Ok(Self::from_vec(values)),
            None => Err(ContainerError::InvalidComparator {
                message: "Expected a proper list".to_string(),
            }),
        }
    }
    
    /// SRFI-134: ideque-fold
    pub fn ideque_fold<F, Acc>(&self, mut init: Acc, mut f: F) -> Acc
    where
        F: FnMut(Acc, &Value) -> Acc,
    {
        for value in self.iter() {
            init = f(init, value);
        }
        init
    }
    
    /// SRFI-134: ideque-fold-right
    pub fn ideque_fold_right<F, Acc>(&self, mut init: Acc, mut f: F) -> Acc
    where
        F: FnMut(&Value, Acc) -> Acc,
    {
        for value in self.data.iter().rev() {
            init = f(value, init);
        }
        init
    }
    
    /// SRFI-134: ideque-map
    pub fn ideque_map<F>(&self, mut f: F) -> Self
    where
        F: FnMut(&Value) -> Value,
    {
        let mapped: Vec<_> = self.iter().map(|v| f(v)).collect();
        Self::from_vec(mapped)
    }
    
    /// SRFI-134: ideque-filter
    pub fn ideque_filter<F>(&self, mut predicate: F) -> Self
    where
        F: FnMut(&Value) -> bool,
    {
        let filtered: Vec<_> = self.iter().filter(|v| predicate(v)).clone())().collect();
        Self::from_vec(filtered)
    }
    
    /// SRFI-134: ideque-append
    pub fn ideque_append(&self, other: &Self) -> Self {
        let mut result = self.clone());
        result.append(other);
        result
    }
    
    /// SRFI-134: ideque-reverse
    pub fn ideque_reverse(&self) -> Self {
        let mut result = self.clone());
        result.data.reverse();
        result
    }
    
    /// SRFI-134: ideque-count
    pub fn ideque_count<F>(&self, mut predicate: F) -> usize
    where
        F: FnMut(&Value) -> bool,
    {
        self.iter().filter(|v| predicate(v)).count()
    }
    
    /// SRFI-134: ideque-any
    pub fn ideque_any<F>(&self, mut predicate: F) -> bool
    where
        F: FnMut(&Value) -> bool,
    {
        self.iter().any(|v| predicate(v))
    }
    
    /// SRFI-134: ideque-every
    pub fn ideque_every<F>(&self, mut predicate: F) -> bool
    where
        F: FnMut(&Value) -> bool,
    {
        self.iter().all(|v| predicate(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let mut ideque = Ideque::new();
        assert!(ideque.is_empty());
        assert_eq!(ideque.len(), 0);
        
        // Test push_back and front
        ideque.push_back(Value::number(1.0));
        ideque.push_back(Value::number(2.0));
        assert_eq!(ideque.len(), 2);
        assert_eq!(ideque.front(), Some(&Value::number(1.0)));
        assert_eq!(ideque.back(), Some(&Value::number(2.0)));
        
        // Test push_front
        ideque.push_front(Value::number(0.0));
        assert_eq!(ideque.len(), 3);
        assert_eq!(ideque.front(), Some(&Value::number(0.0)));
        
        // Test pop operations
        assert_eq!(ideque.pop_front(), Some(Value::number(0.0)));
        assert_eq!(ideque.pop_back(), Some(Value::number(2.0)));
        assert_eq!(ideque.len(), 1);
        assert_eq!(ideque.front(), Some(&Value::number(1.0)));
    }
    
    #[test]
    fn test_persistent_ideque() {
        let ideque = PersistentIdeque::new();
        assert!(ideque.is_empty());
        
        let ideque1 = ideque.cons(Value::number(1.0));
        let ideque2 = ideque1.snoc(Value::number(2.0));
        let ideque3 = ideque2.cons(Value::number(0.0));
        
        // Original ideques should be unchanged
        assert!(ideque.is_empty());
        assert_eq!(ideque1.len(), 1);
        assert_eq!(ideque2.len(), 2);
        assert_eq!(ideque3.len(), 3);
        
        // Test uncons/unsnoc
        if let Some((front, rest)) = ideque3.uncons() {
            assert_eq!(front, Value::number(0.0));
            assert_eq!(rest.len(), 2);
        }
        
        if let Some((rest, back)) = ideque3.unsnoc() {
            assert_eq!(back, Value::number(2.0));
            assert_eq!(rest.len(), 2);
        }
    }
    
    #[test]
    fn test_from_vec() {
        let values = vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ];
        
        let ideque = Ideque::from_vec(values.clone());
        assert_eq!(ideque.len(), 3);
        assert_eq!(ideque.to_vec(), values);
        
        let persistent = PersistentIdeque::from_vec(values.clone());
        assert_eq!(persistent.len(), 3);
        assert_eq!(persistent.to_vec(), values);
    }
}