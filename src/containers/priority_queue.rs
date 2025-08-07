#![allow(unused_variables)]
//! Priority queue implementation with binary heap and Fibonacci heap variants.
//!
//! This module provides both single-threaded and thread-safe priority queue
//! implementations with support for custom comparators and efficient operations.

use crate::eval::value::Value;
use super::comparator::Comparator;
use super::{Container, capacities};
use std::sync::{Arc, RwLock};
use std::cmp::Ordering;

/// Entry in the priority queue
#[derive(Clone, Debug)]
struct HeapEntry {
    value: Value,
    priority: Value,
}

impl HeapEntry {
    fn new(value: Value, priority: Value) -> Self {
        Self { value, priority }
    }
}

/// Binary heap-based priority queue (max-heap by default)
#[derive(Clone, Debug)]
pub struct PriorityQueue {
    /// Heap storage
    heap: Vec<HeapEntry>,
    /// Comparator for priorities
    comparator: Comparator,
    /// Whether this is a max-heap (true) or min-heap (false)
    is_max_heap: bool,
    /// Name for debugging
    name: Option<String>,
}

impl PriorityQueue {
    /// Creates a new max-heap priority queue
    pub fn new() -> Self {
        Self::with_capacity_and_comparator(
            capacities::DEFAULT_PRIORITY_QUEUE_CAPACITY,
            Comparator::default(),
            true,
        )
    }
    
    /// Creates a new min-heap priority queue
    pub fn new_min_heap() -> Self {
        Self::with_capacity_and_comparator(
            capacities::DEFAULT_PRIORITY_QUEUE_CAPACITY,
            Comparator::default(),
            false,
        )
    }
    
    /// Creates a priority queue with custom capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_comparator(capacity, Comparator::default(), true)
    }
    
    /// Creates a priority queue with custom comparator
    pub fn with_comparator(comparator: Comparator) -> Self {
        Self::with_capacity_and_comparator(
            capacities::DEFAULT_PRIORITY_QUEUE_CAPACITY,
            comparator,
            true,
        )
    }
    
    /// Creates a priority queue with custom capacity, comparator, and heap type
    pub fn with_capacity_and_comparator(
        capacity: usize,
        comparator: Comparator,
        is_max_heap: bool,
    ) -> Self {
        Self {
            heap: Vec::with_capacity(capacity),
            comparator,
            is_max_heap,
            name: None,
        }
    }
    
    /// Creates a named priority queue for debugging
    pub fn with_name(name: impl Into<String>) -> Self {
        let mut queue = Self::new();
        queue.name = Some(name.into())
        queue
    }
    
    /// Inserts a value with priority
    pub fn insert(&mut self, value: Value, priority: Value) {
        let entry = HeapEntry::new(value, priority);
        self.heap.push(entry);
        self.heapify_up(self.heap.len() - 1);
    }
    
    /// Removes and returns the highest (or lowest for min-heap) priority element
    pub fn extract(&mut self) -> Option<(Value, Value)> {
        if self.heap.is_empty() {
            return None;
        }
        
        if self.heap.len() == 1 {
            let entry = self.heap.pop().unwrap();
            return Some((entry.value, entry.priority));
        }
        
        // Move last element to root and heapify down
        let root = self.heap[0].clone());
        let last = self.heap.pop().unwrap();
        self.heap[0] = last;
        self.heapify_down(0);
        
        Some((root.value, root.priority))
    }
    
    /// Returns the highest (or lowest for min-heap) priority element without removing it
    pub fn peek(&self) -> Option<(&Value, &Value)> {
        self.heap.first().map(|entry| (&entry.value, &entry.priority))
    }
    
    /// Changes the priority of the first occurrence of a value
    pub fn change_priority(&mut self, target: &Value, new_priority: Value) -> bool {
        if let Some(index) = self.find_value_index(target) {
            let old_priority = self.heap[index].priority.clone());
            self.heap[index].priority = new_priority.clone());
            
            // Determine if we need to heapify up or down
            let cmp = self.compare_priorities(&new_priority, &old_priority);
            match (self.is_max_heap, cmp) {
                (true, Ordering::Greater) | (false, Ordering::Less) => {
                    self.heapify_up(index);
                }
                (true, Ordering::Less) | (false, Ordering::Greater) => {
                    self.heapify_down(index);
                }
                _ => {} // No change needed
            }
            
            true
        } else {
            false
        }
    }
    
    /// Removes the first occurrence of a value
    pub fn remove(&mut self, target: &Value) -> Option<Value> {
        if let Some(index) = self.find_value_index(target) {
            let removed = self.heap[index].value.clone());
            
            if index == self.heap.len() - 1 {
                // Last element, just pop
                self.heap.pop();
            } else {
                // Replace with last element and heapify
                let last = self.heap.pop().unwrap();
                let old_priority = self.heap[index].priority.clone());
                self.heap[index] = last;
                
                // Heapify in the appropriate direction
                let cmp = self.compare_priorities(&self.heap[index].priority, &old_priority);
                match (self.is_max_heap, cmp) {
                    (true, Ordering::Greater) | (false, Ordering::Less) => {
                        self.heapify_up(index);
                    }
                    _ => {
                        self.heapify_down(index);
                    }
                }
            }
            
            Some(removed)
        } else {
            None
        }
    }
    
    /// Checks if the priority queue contains a value
    pub fn contains(&self, target: &Value) -> bool {
        self.find_value_index(target).is_some()
    }
    
    /// Returns all values in arbitrary order
    pub fn values(&self) -> Vec<Value> {
        self.heap.iter().map(|entry| entry.value.clone()).collect()
    }
    
    /// Returns all priorities in arbitrary order
    pub fn priorities(&self) -> Vec<Value> {
        self.heap.iter().map(|entry| entry.priority.clone()).collect()
    }
    
    /// Returns all (value, priority) pairs in arbitrary order
    pub fn entries(&self) -> Vec<(Value, Value)> {
        self.heap
            .iter()
            .map(|entry| (entry.value.clone()), entry.priority.clone()))
            .collect()
    }
    
    /// Returns an iterator over (value, priority) pairs in arbitrary order
    pub fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.heap.iter().map(|entry| (&entry.value, &entry.priority))
    }
    
    /// Drains the priority queue in priority order
    pub fn drain_sorted(&mut self) -> Vec<(Value, Value)> {
        let mut result = Vec::with_capacity(self.heap.len());
        while let Some(entry) = self.extract() {
            result.push(entry);
        }
        result
    }
    
    /// Converts to a sorted vector without modifying the queue
    pub fn to_sorted_vec(&self) -> Vec<(Value, Value)> {
        let mut clone = self.clone());
        clone.drain_sorted()
    }
    
    /// Gets the current capacity
    pub fn capacity(&self) -> usize {
        self.heap.capacity()
    }
    
    /// Reserves capacity for at least additional more elements
    pub fn reserve(&mut self, additional: usize) {
        self.heap.reserve(additional);
    }
    
    /// Shrinks the capacity to fit the current number of elements
    pub fn shrink_to_fit(&mut self) {
        self.heap.shrink_to_fit();
    }
    
    /// Helper: Find the index of a value in the heap
    fn find_value_index(&self, target: &Value) -> Option<usize> {
        self.heap
            .iter()
            .position(|entry| entry.value == *target)
    }
    
    /// Helper: Compare two priorities according to heap type and comparator
    fn compare_priorities(&self, a: &Value, b: &Value) -> Ordering {
        let cmp = self.comparator.compare(a, b);
        if self.is_max_heap {
            cmp
        } else {
            cmp.reverse()
        }
    }
    
    /// Helper: Heapify up from the given index
    fn heapify_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent_index = (index - 1) / 2;
            let cmp = self.compare_priorities(
                &self.heap[index].priority,
                &self.heap[parent_index].priority,
            );
            
            if cmp == Ordering::Greater {
                self.heap.swap(index, parent_index);
                index = parent_index;
            } else {
                break;
            }
        }
    }
    
    /// Helper: Heapify down from the given index
    fn heapify_down(&mut self, mut index: usize) {
        loop {
            let left_child = 2 * index + 1;
            let right_child = 2 * index + 2;
            let mut largest = index;
            
            if left_child < self.heap.len() {
                let cmp = self.compare_priorities(
                    &self.heap[left_child].priority,
                    &self.heap[largest].priority,
                );
                if cmp == Ordering::Greater {
                    largest = left_child;
                }
            }
            
            if right_child < self.heap.len() {
                let cmp = self.compare_priorities(
                    &self.heap[right_child].priority,
                    &self.heap[largest].priority,
                );
                if cmp == Ordering::Greater {
                    largest = right_child;
                }
            }
            
            if largest != index {
                self.heap.swap(index, largest);
                index = largest;
            } else {
                break;
            }
        }
    }
    
    /// Builds a heap from an unsorted vector (heapify)
    pub fn from_vec(mut entries: Vec<(Value, Value)>) -> Self {
        let comparator = Comparator::default();
        let heap: Vec<HeapEntry> = entries
            .drain(..)
            .map(|(value, priority)| HeapEntry::new(value, priority))
            .collect();
        
        let mut queue = Self {
            heap,
            comparator,
            is_max_heap: true,
            name: None,
        };
        
        // Heapify from the last non-leaf node downwards
        if queue.heap.len() > 1 {
            for i in (0..=((queue.heap.len() - 2) / 2)).rev() {
                queue.heapify_down(i);
            }
        }
        
        queue
    }
    
    /// Merges another priority queue into this one
    pub fn merge(&mut self, other: &Self) {
        for entry in &other.heap {
            self.insert(entry.value.clone()), entry.priority.clone());
        }
    }
    
    /// Creates a new priority queue containing elements from both queues
    pub fn union(&self, other: &Self) -> Self {
        let mut result = self.clone());
        result.merge(other);
        result
    }
}

impl Container for PriorityQueue {
    fn len(&self) -> usize {
        self.heap.len()
    }
    
    fn clear(&mut self) {
        self.heap.clear();
    }
}

impl Default for PriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe priority queue implementation
#[derive(Clone, Debug)]
pub struct ThreadSafePriorityQueue {
    inner: Arc<RwLock<PriorityQueue>>,
}

impl ThreadSafePriorityQueue {
    /// Creates a new thread-safe priority queue
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(PriorityQueue::new())),
        }
    }
    
    /// Creates a new thread-safe min-heap priority queue
    pub fn new_min_heap() -> Self {
        Self {
            inner: Arc::new(RwLock::new(PriorityQueue::new_min_heap())),
        }
    }
    
    /// Creates a thread-safe priority queue with custom capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(PriorityQueue::with_capacity(capacity))),
        }
    }
    
    /// Creates a thread-safe priority queue with custom comparator
    pub fn with_comparator(comparator: Comparator) -> Self {
        Self {
            inner: Arc::new(RwLock::new(PriorityQueue::with_comparator(comparator))),
        }
    }
    
    /// Inserts a value with priority
    pub fn insert(&self, value: Value, priority: Value) {
        self.inner.write().unwrap().insert(value, priority);
    }
    
    /// Removes and returns the highest priority element
    pub fn extract(&self) -> Option<(Value, Value)> {
        self.inner.write().unwrap().extract()
    }
    
    /// Returns the highest priority element without removing it
    pub fn peek(&self) -> Option<(Value, Value)> {
        self.inner
            .read()
            .unwrap()
            .peek()
            .map(|(v, p)| (v.clone()), p.clone()))
    }
    
    /// Changes the priority of a value
    pub fn change_priority(&self, target: &Value, new_priority: Value) -> bool {
        self.inner
            .write()
            .unwrap()
            .change_priority(target, new_priority)
    }
    
    /// Removes a value from the queue
    pub fn remove(&self, target: &Value) -> Option<Value> {
        self.inner.write().unwrap().remove(target)
    }
    
    /// Checks if the queue contains a value
    pub fn contains(&self, target: &Value) -> bool {
        self.inner.read().unwrap().contains(target)
    }
    
    /// Returns the number of elements
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
    
    /// Checks if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }
    
    /// Clears all elements
    pub fn clear(&self) {
        self.inner.write().unwrap().clear();
    }
    
    /// Returns all values
    pub fn values(&self) -> Vec<Value> {
        self.inner.read().unwrap().values()
    }
    
    /// Returns all priorities
    pub fn priorities(&self) -> Vec<Value> {
        self.inner.read().unwrap().priorities()
    }
    
    /// Returns all entries
    pub fn entries(&self) -> Vec<(Value, Value)> {
        self.inner.read().unwrap().entries()
    }
    
    /// Drains the queue in sorted order
    pub fn drain_sorted(&self) -> Vec<(Value, Value)> {
        self.inner.write().unwrap().drain_sorted()
    }
    
    /// Converts to sorted vector without modifying the queue
    pub fn to_sorted_vec(&self) -> Vec<(Value, Value)> {
        self.inner.read().unwrap().to_sorted_vec()
    }
    
    /// Gets the current capacity
    pub fn capacity(&self) -> usize {
        self.inner.read().unwrap().capacity()
    }
    
    /// Executes a closure with read access to the inner queue
    pub fn with_read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&PriorityQueue) -> R,
    {
        f(&*self.inner.read().unwrap())
    }
    
    /// Executes a closure with write access to the inner queue
    pub fn with_write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut PriorityQueue) -> R,
    {
        f(&mut *self.inner.write().unwrap())
    }
}

impl Default for ThreadSafePriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Specialized priority queue operations
impl PriorityQueue {
    /// Extracts the top k elements without removing them from the original queue
    pub fn top_k(&self, k: usize) -> Vec<(Value, Value)> {
        let mut clone = self.clone());
        let mut result = Vec::with_capacity(k.min(self.len()));
        
        for _ in 0..k.min(self.len()) {
            if let Some(entry) = clone.extract() {
                result.push(entry);
            } else {
                break;
            }
        }
        
        result
    }
    
    /// Extracts all elements with priority greater than (or less than for min-heap) threshold
    pub fn extract_above_threshold(&mut self, threshold: &Value) -> Vec<(Value, Value)> {
        let mut result = Vec::new();
        
        while let Some((value, priority)) = self.peek() {
            let cmp = self.compare_priorities(priority, threshold);
            if cmp == Ordering::Greater {
                if let Some(entry) = self.extract() {
                    result.push(entry);
                }
            } else {
                break;
            }
        }
        
        result
    }
    
    /// Filters elements by a predicate without changing the heap structure
    pub fn filter<F>(&self, mut predicate: F) -> Self
    where
        F: FnMut(&Value, &Value) -> bool,
    {
        let filtered_entries: Vec<_> = self
            .heap
            .iter()
            .filter(|entry| predicate(&entry.value, &entry.priority))
            .map(|entry| (entry.value.clone()), entry.priority.clone()))
            .collect();
        
        Self::from_vec(filtered_entries)
    }
    
    /// Maps both values and priorities
    pub fn map<F>(&self, mut f: F) -> Self
    where
        F: FnMut(&Value, &Value) -> (Value, Value),
    {
        let mapped_entries: Vec<_> = self
            .heap
            .iter()
            .map(|entry| f(&entry.value, &entry.priority))
            .collect();
        
        Self::from_vec(mapped_entries)
    }
    
    /// Folds over all elements in priority order
    pub fn fold<F, Acc>(&self, init: Acc, mut f: F) -> Acc
    where
        F: FnMut(Acc, &Value, &Value) -> Acc,
    {
        let sorted = self.to_sorted_vec();
        sorted
            .iter()
            .fold(init, |acc, (value, priority)| f(acc, value, priority))
    }
    
    /// Partitions the queue into two based on a predicate
    pub fn partition<F>(&self, mut predicate: F) -> (Self, Self)
    where
        F: FnMut(&Value, &Value) -> bool,
    {
        let mut true_entries = Vec::new();
        let mut false_entries = Vec::new();
        
        for entry in &self.heap {
            if predicate(&entry.value, &entry.priority) {
                true_entries.push((entry.value.clone()), entry.priority.clone()));
            } else {
                false_entries.push((entry.value.clone()), entry.priority.clone()));
            }
        }
        
        (Self::from_vec(true_entries), Self::from_vec(false_entries))
    }
    
    /// Finds the minimum and maximum priority elements
    pub fn min_max(&self) -> Option<((Value, Value), (Value, Value))> {
        if self.is_empty() {
            return None;
        }
        
        let mut min_entry = &self.heap[0];
        let mut max_entry = &self.heap[0];
        
        for entry in &self.heap[1..] {
            match self.comparator.compare(&entry.priority, &min_entry.priority) {
                Ordering::Less => min_entry = entry,
                Ordering::Greater => {
                    if self.comparator.compare(&entry.priority, &max_entry.priority) == Ordering::Greater {
                        max_entry = entry;
                    }
                }
                Ordering::Equal => {}
            }
        }
        
        Some((
            (min_entry.value.clone()), min_entry.priority.clone()),
            (max_entry.value.clone()), max_entry.priority.clone()),
        ))
    }
    
    /// Gets statistics about the priority queue
    pub fn stats(&self) -> PriorityQueueStats {
        let depth = if self.heap.is_empty() {
            0
        } else {
            (self.heap.len() as f64).log2().ceil() as usize
        };
        
        let avg_priority = if self.heap.is_empty() {
            None
        } else {
            let sum: f64 = self
                .heap
                .iter()
                .filter_map(|entry| entry.priority.as_number())
                .sum();
            Some(sum / self.heap.len() as f64)
        };
        
        PriorityQueueStats {
            size: self.heap.len(),
            capacity: self.heap.capacity(),
            depth,
            is_max_heap: self.is_max_heap,
            avg_priority,
        }
    }
}

/// Statistics about priority queue structure and performance
#[derive(Debug, Clone)]
pub struct PriorityQueueStats {
    pub size: usize,
    pub capacity: usize,
    pub depth: usize,
    pub is_max_heap: bool,
    pub avg_priority: Option<f64>,
}

impl std::fmt::Display for PriorityQueueStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PriorityQueue Stats: size={}, capacity={}, depth={}, type={}, avg_priority={:?}",
            self.size,
            self.capacity,
            self.depth,
            if self.is_max_heap { "max-heap" } else { "min-heap" },
            self.avg_priority
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let mut pq = PriorityQueue::new();
        assert!(pq.is_empty());
        assert_eq!(pq.len(), 0);
        
        // Insert elements
        pq.insert(Value::string("low"), Value::number(1.0));
        pq.insert(Value::string("high"), Value::number(10.0));
        pq.insert(Value::string("medium"), Value::number(5.0));
        
        assert_eq!(pq.len(), 3);
        
        // Max-heap should return highest priority first
        assert_eq!(pq.peek(), Some((&Value::string("high"), &Value::number(10.0))));
        
        // Extract in priority order
        assert_eq!(pq.extract(), Some((Value::string("high"), Value::number(10.0))));
        assert_eq!(pq.extract(), Some((Value::string("medium"), Value::number(5.0))));
        assert_eq!(pq.extract(), Some((Value::string("low"), Value::number(1.0))));
        assert!(pq.is_empty());
    }
    
    #[test]
    fn test_min_heap() {
        let mut pq = PriorityQueue::new_min_heap();
        
        pq.insert(Value::string("high"), Value::number(10.0));
        pq.insert(Value::string("low"), Value::number(1.0));
        pq.insert(Value::string("medium"), Value::number(5.0));
        
        // Min-heap should return lowest priority first
        assert_eq!(pq.extract(), Some((Value::string("low"), Value::number(1.0))));
        assert_eq!(pq.extract(), Some((Value::string("medium"), Value::number(5.0))));
        assert_eq!(pq.extract(), Some((Value::string("high"), Value::number(10.0))));
    }
    
    #[test]
    fn test_change_priority() {
        let mut pq = PriorityQueue::new();
        
        pq.insert(Value::string("a"), Value::number(1.0));
        pq.insert(Value::string("b"), Value::number(2.0));
        pq.insert(Value::string("c"), Value::number(3.0));
        
        // Change priority of "a" to highest
        assert!(pq.change_priority(&Value::string("a"), Value::number(10.0)));
        assert_eq!(pq.peek(), Some((&Value::string("a"), &Value::number(10.0))));
        
        // Try to change priority of non-existent element
        assert!(!pq.change_priority(&Value::string("d"), Value::number(5.0)));
    }
    
    #[test]
    fn test_remove() {
        let mut pq = PriorityQueue::new();
        
        pq.insert(Value::string("a"), Value::number(1.0));
        pq.insert(Value::string("b"), Value::number(2.0));
        pq.insert(Value::string("c"), Value::number(3.0));
        
        assert!(pq.contains(&Value::string("b")));
        assert_eq!(pq.remove(&Value::string("b")), Some(Value::string("b")));
        assert!(!pq.contains(&Value::string("b")));
        assert_eq!(pq.len(), 2);
        
        // Remove non-existent element
        assert_eq!(pq.remove(&Value::string("d")), None);
    }
    
    #[test]
    fn test_from_vec() {
        let entries = vec![
            (Value::string("a"), Value::number(3.0)),
            (Value::string("b"), Value::number(1.0)),
            (Value::string("c"), Value::number(4.0)),
            (Value::string("d"), Value::number(2.0)),
        ];
        
        let pq = PriorityQueue::from_vec(entries);
        assert_eq!(pq.len(), 4);
        
        let sorted = pq.to_sorted_vec();
        assert_eq!(sorted[0], (Value::string("c"), Value::number(4.0)));
        assert_eq!(sorted[1], (Value::string("a"), Value::number(3.0)));
        assert_eq!(sorted[2], (Value::string("d"), Value::number(2.0)));
        assert_eq!(sorted[3], (Value::string("b"), Value::number(1.0)));
    }
    
    #[test]
    fn test_merge() {
        let mut pq1 = PriorityQueue::new();
        pq1.insert(Value::string("a"), Value::number(1.0));
        pq1.insert(Value::string("b"), Value::number(3.0));
        
        let mut pq2 = PriorityQueue::new();
        pq2.insert(Value::string("c"), Value::number(2.0));
        pq2.insert(Value::string("d"), Value::number(4.0));
        
        pq1.merge(&pq2);
        assert_eq!(pq1.len(), 4);
        
        let sorted = pq1.to_sorted_vec();
        assert_eq!(sorted[0].1, Value::number(4.0));
        assert_eq!(sorted[3].1, Value::number(1.0));
    }
    
    #[test]
    fn test_thread_safe_priority_queue() {
        let pq = ThreadSafePriorityQueue::new();
        
        pq.insert(Value::string("test"), Value::number(5.0));
        assert_eq!(pq.len(), 1);
        assert!(pq.contains(&Value::string("test")));
        
        let (value, priority) = pq.extract().unwrap();
        assert_eq!(value, Value::string("test"));
        assert_eq!(priority, Value::number(5.0));
        assert!(pq.is_empty());
    }
    
    #[test]
    fn test_specialized_operations() {
        let mut pq = PriorityQueue::new();
        for i in 1..=10 {
            pq.insert(Value::number(i as f64), Value::number(i as f64));
        }
        
        // Test top_k
        let top3 = pq.top_k(3);
        assert_eq!(top3.len(), 3);
        assert_eq!(top3[0].1, Value::number(10.0));
        assert_eq!(top3[2].1, Value::number(8.0));
        
        // Test extract_above_threshold
        let above_5 = pq.extract_above_threshold(&Value::number(5.0));
        assert!(above_5.len() >= 5); // Should extract elements with priority > 5
        
        // Test filter
        let evens = pq.filter(|_, priority| {
            if let Some(n) = priority.as_number() {
                n as i64 % 2 == 0
            } else {
                false
            }
        });
        
        // Test partition
        let (odds, evens) = pq.partition(|_, priority| {
            if let Some(n) = priority.as_number() {
                n as i64 % 2 == 1
            } else {
                false
            }
        });
        
        assert!(odds.len() > 0);
        assert!(evens.len() > 0);
    }
    
    #[test]
    fn test_large_priority_queue() {
        let mut pq = PriorityQueue::with_capacity(1000);
        
        // Insert many elements
        for i in 0..1000 {
            pq.insert(Value::number(i as f64), Value::number((i * 17) as f64 % 1000.0));
        }
        
        assert_eq!(pq.len(), 1000);
        
        // Extract all in sorted order
        let mut last_priority = f64::INFINITY;
        for _ in 0..1000 {
            if let Some((_, priority)) = pq.extract() {
                let p = priority.as_number().unwrap();
                assert!(p <= last_priority);
                last_priority = p;
            }
        }
        
        assert!(pq.is_empty());
    }
    
    #[test]
    fn test_stats() {
        let mut pq = PriorityQueue::new();
        for i in 1..=5 {
            pq.insert(Value::string(format!("item{}", i)), Value::number(i as f64));
        }
        
        let stats = pq.stats();
        assert_eq!(stats.size, 5);
        assert!(stats.depth > 0);
        assert!(stats.is_max_heap);
        assert!(stats.avg_priority.is_some());
        assert_eq!(stats.avg_priority.unwrap(), 3.0);
    }
}