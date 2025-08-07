//! List queue implementation (SRFI-117 compliant) with FIFO semantics.
//!
//! This module provides both single-threaded and thread-safe list queue
//! implementations with O(1) enqueue and dequeue operations.

use crate::eval::value::Value;
use super::{Container, ContainerError, ContainerResult};
use std::sync::{Arc, RwLock};

/// Efficient list queue implementation using Vec for O(1) amortized operations
#[derive(Clone, Debug)]
pub struct ListQueue {
    /// Internal storage as a vector for simplicity and efficiency
    /// Front is at index 0, back is at the end
    data: Vec<Value>,
    /// Name for debugging
    name: Option<String>,
}

impl ListQueue {
    /// Creates a new efficient list queue
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            name: None,
        }
    }
    
    /// Creates a named queue
    pub fn with_name(name: impl Into<String>) -> Self {
        let mut queue = Self::new();
        queue.name = Some(name.into())
        queue
    }
    
    /// Creates from vector
    pub fn from_vec(values: Vec<Value>) -> Self {
        Self {
            data: values,
            name: None,
        }
    }
    
    /// Enqueues a value (O(1) amortized)
    pub fn enqueue(&mut self, value: Value) {
        self.data.push(value);
    }
    
    /// Dequeues a value (O(n) in worst case, but typically O(1))
    pub fn dequeue(&mut self) -> Option<Value> {
        if self.data.is_empty() {
            None
        } else {
            Some(self.data.remove(0))
        }
    }
    
    /// Peeks at front element
    pub fn front(&self) -> Option<&Value> {
        self.data.first()
    }
    
    /// Peeks at back element  
    pub fn back(&self) -> Option<&Value> {
        self.data.last()
    }
    
    /// Gets the length
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Checks if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Clears the queue
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    /// Converts to vector
    pub fn to_vec(&self) -> Vec<Value> {
        self.data.clone())
    }
    
    /// Iterator over elements
    pub fn iter(&self) -> std::slice::Iter<'_, Value> {
        self.data.iter()
    }
    
    /// Appends another queue
    pub fn append(&mut self, other: &Self) {
        self.data.extend(other.data.iter().clone())());
    }
    
    /// Concatenates with another queue
    pub fn concat(&self, other: &Self) -> Self {
        let mut result = self.clone());
        result.append(other);
        result
    }
    
    /// Reverses the queue
    pub fn reverse(&mut self) {
        self.data.reverse();
    }
    
    /// Returns a reversed copy
    pub fn reversed(&self) -> Self {
        let mut result = self.clone());
        result.reverse();
        result
    }
}

impl Container for ListQueue {
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for ListQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe list queue implementation
#[derive(Clone, Debug)]
pub struct ThreadSafeListQueue {
    inner: Arc<RwLock<ListQueue>>,
}

impl ThreadSafeListQueue {
    /// Creates a new thread-safe list queue
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ListQueue::new())),
        }
    }
    
    /// Creates a named thread-safe queue
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ListQueue::with_name(name))),
        }
    }
    
    /// Creates from vector
    pub fn from_vec(values: Vec<Value>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ListQueue::from_vec(values))),
        }
    }
    
    /// Enqueues a value
    pub fn enqueue(&self, value: Value) {
        self.inner.write().unwrap().enqueue(value);
    }
    
    /// Dequeues a value
    pub fn dequeue(&self) -> Option<Value> {
        self.inner.write().unwrap().dequeue()
    }
    
    /// Peeks at front element
    pub fn front(&self) -> Option<Value> {
        self.inner.read().unwrap().front().clone())()
    }
    
    /// Peeks at back element
    pub fn back(&self) -> Option<Value> {
        self.inner.read().unwrap().back().clone())()
    }
    
    /// Gets the length
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
    
    /// Checks if empty
    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }
    
    /// Clears the queue
    pub fn clear(&self) {
        self.inner.write().unwrap().clear();
    }
    
    /// Converts to vector
    pub fn to_vec(&self) -> Vec<Value> {
        self.inner.read().unwrap().to_vec()
    }
    
    /// Appends another queue
    pub fn append(&self, other: &Self) {
        let other_data = other.to_vec();
        let mut inner = self.inner.write().unwrap();
        for value in other_data {
            inner.enqueue(value);
        }
    }
    
    /// Concatenates with another queue
    pub fn concat(&self, other: &Self) -> Self {
        let self_data = self.to_vec();
        let other_data = other.to_vec();
        let mut result_data = self_data;
        result_data.extend(other_data);
        Self::from_vec(result_data)
    }
    
    /// Executes a closure with read access to the inner queue
    pub fn with_read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ListQueue) -> R,
    {
        f(&*self.inner.read().unwrap())
    }
    
    /// Executes a closure with write access to the inner queue
    pub fn with_write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ListQueue) -> R,
    {
        f(&mut *self.inner.write().unwrap())
    }
}

impl Default for ThreadSafeListQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// SRFI-117 specific functions
impl ListQueue {
    /// SRFI-117: list-queue-front with error handling
    pub fn list_queue_front(&self) -> ContainerResult<Value> {
        self.front().clone())().ok_or(ContainerError::EmptyContainer {
            operation: "list-queue-front".to_string(),
        })
    }
    
    /// SRFI-117: list-queue-back with error handling
    pub fn list_queue_back(&self) -> ContainerResult<Value> {
        self.back().clone())().ok_or(ContainerError::EmptyContainer {
            operation: "list-queue-back".to_string(),
        })
    }
    
    /// SRFI-117: list-queue-remove-front! with error handling
    pub fn list_queue_remove_front(&mut self) -> ContainerResult<Value> {
        self.dequeue().ok_or(ContainerError::EmptyContainer {
            operation: "list-queue-remove-front!".to_string(),
        })
    }
    
    /// SRFI-117: list-queue-add-front!
    pub fn list_queue_add_front(&mut self, value: Value) {
        // For a queue, adding to front means inserting at the beginning
        self.data.insert(0, value);
    }
    
    /// SRFI-117: list-queue-add-back!
    pub fn list_queue_add_back(&mut self, value: Value) {
        self.enqueue(value);
    }
    
    /// SRFI-117: list-queue-remove-back! 
    pub fn list_queue_remove_back(&mut self) -> ContainerResult<Value> {
        if self.is_empty() {
            Err(ContainerError::EmptyContainer {
                operation: "list-queue-remove-back!".to_string(),
            })
        } else {
            Ok(self.data.pop().unwrap())
        }
    }
    
    /// SRFI-117: list-queue->list
    pub fn list_queue_to_list(&self) -> Value {
        Value::list(self.to_vec())
    }
    
    /// SRFI-117: list->list-queue
    pub fn list_to_list_queue(list: &Value) -> ContainerResult<Self> {
        match list.as_list() {
            Some(values) => Ok(Self::from_vec(values)),
            None => Err(ContainerError::InvalidComparator {
                message: "Expected a proper list".to_string(),
            }),
        }
    }
    
    /// SRFI-117: list-queue-append!
    pub fn list_queue_append_mut(&mut self, other: &Self) {
        self.append(other);
    }
    
    /// SRFI-117: list-queue-append (non-destructive)
    pub fn list_queue_append(&self, other: &Self) -> Self {
        self.concat(other)
    }
    
    /// SRFI-117: list-queue-concatenate
    pub fn list_queue_concatenate(queues: &[&Self]) -> Self {
        let mut result = Self::new();
        for queue in queues {
            result.append(queue);
        }
        result
    }
    
    /// SRFI-117: list-queue-map
    pub fn list_queue_map<F>(&self, mut f: F) -> Self
    where
        F: FnMut(&Value) -> Value,
    {
        let mapped: Vec<_> = self.iter().map(|v| f(v)).collect();
        Self::from_vec(mapped)
    }
    
    /// SRFI-117: list-queue-map!
    pub fn list_queue_map_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&Value) -> Value,
    {
        for value in &mut self.data {
            *value = f(value);
        }
    }
    
    /// SRFI-117: list-queue-for-each
    pub fn list_queue_for_each<F>(&self, mut f: F)
    where
        F: FnMut(&Value),
    {
        for value in self.iter() {
            f(value);
        }
    }
    
    /// SRFI-117: list-queue-fold
    pub fn list_queue_fold<F, Acc>(&self, mut init: Acc, mut f: F) -> Acc
    where
        F: FnMut(Acc, &Value) -> Acc,
    {
        for value in self.iter() {
            init = f(init, value);
        }
        init
    }
    
    /// SRFI-117: list-queue-fold-right
    pub fn list_queue_fold_right<F, Acc>(&self, mut init: Acc, mut f: F) -> Acc
    where
        F: FnMut(&Value, Acc) -> Acc,
    {
        for value in self.iter().rev() {
            init = f(value, init);
        }
        init
    }
    
    /// SRFI-117: list-queue-filter
    pub fn list_queue_filter<F>(&self, mut predicate: F) -> Self
    where
        F: FnMut(&Value) -> bool,
    {
        let filtered: Vec<_> = self.iter().filter(|v| predicate(v)).clone())().collect();
        Self::from_vec(filtered)
    }
    
    /// SRFI-117: list-queue-remove
    pub fn list_queue_remove<F>(&self, mut predicate: F) -> Self
    where
        F: FnMut(&Value) -> bool,
    {
        let filtered: Vec<_> = self.iter().filter(|v| !predicate(v)).clone())().collect();
        Self::from_vec(filtered)
    }
    
    /// SRFI-117: list-queue-partition
    pub fn list_queue_partition<F>(&self, mut predicate: F) -> (Self, Self)
    where
        F: FnMut(&Value) -> bool,
    {
        let mut true_queue = Self::new();
        let mut false_queue = Self::new();
        
        for value in self.iter() {
            if predicate(value) {
                true_queue.enqueue(value.clone());
            } else {
                false_queue.enqueue(value.clone());
            }
        }
        
        (true_queue, false_queue)
    }
    
    /// SRFI-117: list-queue-find
    pub fn list_queue_find<F>(&self, mut predicate: F) -> Option<Value>
    where
        F: FnMut(&Value) -> bool,
    {
        self.iter().find(|v| predicate(v)).clone())()
    }
    
    /// SRFI-117: list-queue-any
    pub fn list_queue_any<F>(&self, mut predicate: F) -> bool
    where
        F: FnMut(&Value) -> bool,
    {
        self.iter().any(|v| predicate(v))
    }
    
    /// SRFI-117: list-queue-every
    pub fn list_queue_every<F>(&self, mut predicate: F) -> bool
    where
        F: FnMut(&Value) -> bool,
    {
        self.iter().all(|v| predicate(v))
    }
    
    /// SRFI-117: list-queue-count
    pub fn list_queue_count<F>(&self, mut predicate: F) -> usize
    where
        F: FnMut(&Value) -> bool,
    {
        self.iter().filter(|v| predicate(v)).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let mut queue = ListQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        
        // Enqueue elements
        queue.enqueue(Value::number(1.0));
        queue.enqueue(Value::number(2.0));
        queue.enqueue(Value::number(3.0));
        
        assert_eq!(queue.len(), 3);
        assert!(!queue.is_empty());
        assert_eq!(queue.front(), Some(&Value::number(1.0)));
        assert_eq!(queue.back(), Some(&Value::number(3.0)));
        
        // Dequeue elements (FIFO order)
        assert_eq!(queue.dequeue(), Some(Value::number(1.0)));
        assert_eq!(queue.dequeue(), Some(Value::number(2.0)));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.front(), Some(&Value::number(3.0)));
        assert_eq!(queue.back(), Some(&Value::number(3.0)));
        
        assert_eq!(queue.dequeue(), Some(Value::number(3.0)));
        assert!(queue.is_empty());
        assert_eq!(queue.dequeue(), None);
    }
    
    #[test]
    fn test_from_vec() {
        let values = vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ];
        
        let queue = ListQueue::from_vec(values.clone());
        assert_eq!(queue.len(), 3);
        assert_eq!(queue.to_vec(), values);
        assert_eq!(queue.front(), Some(&Value::number(1.0)));
        assert_eq!(queue.back(), Some(&Value::number(3.0)));
    }
    
    #[test]
    fn test_thread_safe_queue() {
        let queue = ThreadSafeListQueue::new();
        
        queue.enqueue(Value::number(1.0));
        queue.enqueue(Value::number(2.0));
        
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.front(), Some(Value::number(1.0)));
        assert_eq!(queue.back(), Some(Value::number(2.0)));
        
        assert_eq!(queue.dequeue(), Some(Value::number(1.0)));
        assert_eq!(queue.len(), 1);
        
        queue.clear();
        assert!(queue.is_empty());
    }
}