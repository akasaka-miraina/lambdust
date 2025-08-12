//! Bag (multiset) implementation for SRFI-113: Bag basic data structures.
//!
//! This module provides a hash-based Bag implementation with full SRFI-113 compliance,
//! designed for integration with Lambdust's Value system and SRFI-128 comparators.
//! A bag is a collection that can contain duplicate elements, where each element
//! has an associated multiplicity (count).

use crate::eval::value::Value;
use crate::containers::{Container, ContainerError, ContainerResult};
use crate::containers::comparator::HashComparator;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A hash-based bag (multiset) implementation using SRFI-128 comparators.
///
/// This bag uses a HashMap<Value, usize> internally where the usize represents
/// the multiplicity (count) of each element. It provides O(1) average-case operations
/// and integrates with SRFI-128 comparators for element equality testing.
#[derive(Debug, Clone)]
pub struct Bag {
    /// Internal hash table storage using HashMap<Value, usize> for multiplicities
    elements: HashMap<Value, usize>,
    /// Comparator for element equality and hashing
    comparator: HashComparator,
    /// Debug name for easier identification
    debug_name: Option<String>,
}

impl Bag {
    /// Creates a new empty bag with the default comparator.
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            comparator: HashComparator::with_default(),
            debug_name: None,
        }
    }
    
    /// Creates a new empty bag with a custom comparator.
    pub fn with_comparator(comparator: HashComparator) -> Self {
        Self {
            elements: HashMap::new(),
            comparator,
            debug_name: None,
        }
    }
    
    
    /// Creates a bag from an iterator with a custom comparator.
    pub fn from_iter_with_comparator<I>(iter: I, comparator: HashComparator) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        let mut bag = Self::with_comparator(comparator);
        for value in iter {
            bag.adjoin(value);
        }
        bag
    }
    
    /// Adds an element to the bag, incrementing its count by 1.
    /// Returns the new count of the element.
    pub fn adjoin(&mut self, value: Value) -> usize {
        let count = self.elements.entry(value).or_insert(0);
        *count += 1;
        *count
    }
    
    /// Adds an element to the bag with a specific count.
    /// Returns the new total count of the element.
    pub fn adjoin_count(&mut self, value: Value, count: usize) -> usize {
        if count == 0 {
            return self.element_count(&value);
        }
        let current_count = self.elements.entry(value).or_insert(0);
        *current_count += count;
        *current_count
    }
    
    /// Removes one instance of an element from the bag.
    /// Returns true if the element was present (and thus decremented).
    pub fn delete(&mut self, value: &Value) -> bool {
        if let Some(count) = self.elements.get_mut(value) {
            if *count > 1 {
                *count -= 1;
                true
            } else {
                self.elements.remove(value);
                true
            }
        } else {
            false
        }
    }
    
    /// Removes all instances of an element from the bag.
    /// Returns the count that was removed.
    pub fn delete_all(&mut self, value: &Value) -> usize {
        self.elements.remove(value).unwrap_or(0)
    }
    
    /// Tests whether an element is in the bag (count > 0).
    pub fn contains(&self, value: &Value) -> bool {
        self.elements.get(value).is_some_and(|&count| count > 0)
    }
    
    /// Returns the count (multiplicity) of an element in the bag.
    pub fn element_count(&self, value: &Value) -> usize {
        self.elements.get(value).copied().unwrap_or(0)
    }
    
    /// Returns the number of unique elements in the bag.
    pub fn unique_size(&self) -> usize {
        self.elements.len()
    }
    
    /// Returns the total number of elements in the bag (sum of all multiplicities).
    pub fn total_size(&self) -> usize {
        self.elements.values().sum()
    }
    
    /// Alias for total_size() to match Container trait.
    pub fn size(&self) -> usize {
        self.total_size()
    }
    
    /// Checks if the bag is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
    
    /// Returns an iterator over the unique elements in the bag.
    pub fn unique_iter(&self) -> impl Iterator<Item = &Value> {
        self.elements.keys()
    }
    
    /// Returns an iterator over (element, count) pairs.
    pub fn iter_with_counts(&self) -> impl Iterator<Item = (&Value, &usize)> {
        self.elements.iter()
    }
    
    /// Returns a vector of all elements including duplicates.
    pub fn to_vec(&self) -> Vec<Value> {
        let mut result = Vec::with_capacity(self.total_size());
        for (value, &count) in &self.elements {
            for _ in 0..count {
                result.push(value.clone());
            }
        }
        result
    }
    
    /// Returns a vector of unique elements (no duplicates).
    pub fn unique_to_vec(&self) -> Vec<Value> {
        self.elements.keys().cloned().collect()
    }
    
    /// Converts this bag to a set (discards multiplicities).
    pub fn to_set(&self) -> crate::containers::set::Set {
        crate::containers::set::Set::from_iter_with_comparator(
            self.elements.keys().cloned(),
            self.comparator.clone()
        )
    }
    
    /// Returns the comparator used by this bag.
    pub fn comparator(&self) -> &HashComparator {
        &self.comparator
    }
    
    /// Sets the debug name for this bag.
    pub fn set_debug_name(&mut self, name: impl Into<String>) {
        self.debug_name = Some(name.into());
    }
    
    /// Gets the debug name of this bag.
    pub fn debug_name(&self) -> Option<&str> {
        self.debug_name.as_deref()
    }
    
    /// Clears the debug name.
    pub fn clear_debug_name(&mut self) {
        self.debug_name = None;
    }
    
    /// Increments the count of an element by 1. Returns the new count.
    pub fn increment(&mut self, value: &Value) -> usize {
        let count = self.elements.entry(value.clone()).or_insert(0);
        *count += 1;
        *count
    }
    
    /// Decrements the count of an element by 1. Returns the new count.
    /// If the count reaches 0, the element is removed.
    pub fn decrement(&mut self, value: &Value) -> usize {
        if let Some(count) = self.elements.get_mut(value) {
            if *count > 1 {
                *count -= 1;
                *count
            } else {
                self.elements.remove(value);
                0
            }
        } else {
            0
        }
    }
    
    /// Creates a union of this bag with another bag (sum of multiplicities).
    pub fn union(&self, other: &Bag) -> Bag {
        let mut result = self.clone();
        for (value, &count) in &other.elements {
            result.adjoin_count(value.clone(), count);
        }
        result
    }
    
    /// Creates an intersection of this bag with another bag (minimum multiplicities).
    pub fn intersection(&self, other: &Bag) -> Bag {
        let mut result = Bag::with_comparator(self.comparator.clone());
        for (value, &self_count) in &self.elements {
            if let Some(&other_count) = other.elements.get(value) {
                let min_count = self_count.min(other_count);
                if min_count > 0 {
                    result.adjoin_count(value.clone(), min_count);
                }
            }
        }
        result
    }
    
    /// Creates a difference of this bag with another bag (subtract multiplicities).
    pub fn difference(&self, other: &Bag) -> Bag {
        let mut result = Bag::with_comparator(self.comparator.clone());
        for (value, &self_count) in &self.elements {
            let other_count = other.element_count(value);
            if self_count > other_count {
                result.adjoin_count(value.clone(), self_count - other_count);
            }
        }
        result
    }
    
    /// Creates a sum of this bag with another bag (alias for union).
    pub fn sum(&self, other: &Bag) -> Bag {
        self.union(other)
    }
    
    /// Creates a product of this bag with another bag.
    /// For each element, multiplies its counts in both bags.
    pub fn product(&self, other: &Bag) -> Bag {
        let mut result = Bag::with_comparator(self.comparator.clone());
        for (value, &self_count) in &self.elements {
            if let Some(&other_count) = other.elements.get(value) {
                let product_count = self_count * other_count;
                if product_count > 0 {
                    result.adjoin_count(value.clone(), product_count);
                }
            }
        }
        result
    }
    
    /// Tests if this bag is a subbag of another bag.
    /// Every element in this bag must have count <= count in other bag.
    pub fn is_subbag(&self, other: &Bag) -> bool {
        self.elements.iter().all(|(value, &count)| {
            other.element_count(value) >= count
        })
    }
    
    /// Tests if this bag is disjoint from another bag (no common elements).
    pub fn is_disjoint(&self, other: &Bag) -> bool {
        !self.elements.keys().any(|value| other.contains(value))
    }
}

impl Default for Bag {
    fn default() -> Self {
        Self::new()
    }
}

impl Container for Bag {
    fn len(&self) -> usize {
        self.total_size()
    }
    
    fn clear(&mut self) {
        self.elements.clear();
    }
}

impl PartialEq for Bag {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

impl Eq for Bag {}

/// Thread-safe wrapper for Bag operations.
///
/// This provides a thread-safe interface to Bag operations using Arc<RwLock<Bag>>.
#[derive(Debug, Clone)]
pub struct ThreadSafeBag {
    inner: Arc<RwLock<Bag>>,
}

impl ThreadSafeBag {
    /// Creates a new thread-safe empty bag.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Bag::new())),
        }
    }
    
    /// Creates a new thread-safe bag with a custom comparator.
    pub fn with_comparator(comparator: HashComparator) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Bag::with_comparator(comparator))),
        }
    }
    
    
    /// Adds an element to the bag. Returns the new count.
    pub fn adjoin(&self, value: Value) -> ContainerResult<usize> {
        Ok(self
            .inner
            .write()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire write lock".to_string(),
            })?
            .adjoin(value))
    }
    
    /// Adds an element with a specific count. Returns the new total count.
    pub fn adjoin_count(&self, value: Value, count: usize) -> ContainerResult<usize> {
        Ok(self
            .inner
            .write()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire write lock".to_string(),
            })?
            .adjoin_count(value, count))
    }
    
    /// Removes one instance of an element from the bag.
    pub fn delete(&self, value: &Value) -> ContainerResult<bool> {
        Ok(self
            .inner
            .write()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire write lock".to_string(),
            })?
            .delete(value))
    }
    
    /// Tests whether an element is in the bag.
    pub fn contains(&self, value: &Value) -> ContainerResult<bool> {
        Ok(self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?
            .contains(value))
    }
    
    /// Returns the count of an element in the bag.
    pub fn element_count(&self, value: &Value) -> ContainerResult<usize> {
        Ok(self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?
            .element_count(value))
    }
    
    /// Returns the number of unique elements in the bag.
    pub fn unique_size(&self) -> ContainerResult<usize> {
        Ok(self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?
            .unique_size())
    }
    
    /// Returns the total size of the bag.
    pub fn total_size(&self) -> ContainerResult<usize> {
        Ok(self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?
            .total_size())
    }
    
    /// Alias for total_size().
    pub fn size(&self) -> ContainerResult<usize> {
        self.total_size()
    }
    
    /// Checks if the bag is empty.
    pub fn is_empty(&self) -> ContainerResult<bool> {
        Ok(self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?
            .is_empty())
    }
    
    /// Converts the bag to a vector including duplicates.
    pub fn to_vec(&self) -> ContainerResult<Vec<Value>> {
        Ok(self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?
            .to_vec())
    }
    
    /// Increments an element's count. Returns the new count.
    pub fn increment(&self, value: &Value) -> ContainerResult<usize> {
        Ok(self
            .inner
            .write()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire write lock".to_string(),
            })?
            .increment(value))
    }
    
    /// Decrements an element's count. Returns the new count.
    pub fn decrement(&self, value: &Value) -> ContainerResult<usize> {
        Ok(self
            .inner
            .write()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire write lock".to_string(),
            })?
            .decrement(value))
    }
    
    /// Clears all elements from the bag.
    pub fn clear(&self) -> ContainerResult<()> {
        self.inner
            .write()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire write lock".to_string(),
            })?
            .clear();
        Ok(())
    }
    
    /// Sets the debug name for this bag.
    pub fn set_debug_name(&self, name: impl Into<String>) -> ContainerResult<()> {
        self.inner
            .write()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire write lock".to_string(),
            })?
            .set_debug_name(name);
        Ok(())
    }
    
    /// Gets the debug name of this bag.
    pub fn debug_name(&self) -> ContainerResult<Option<String>> {
        Ok(self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?
            .debug_name()
            .map(|s| s.to_string()))
    }
    
    /// Creates a union of this bag with another bag.
    pub fn union(&self, other: &ThreadSafeBag) -> ContainerResult<ThreadSafeBag> {
        let self_bag = self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        let other_bag = other
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        
        let result = self_bag.union(&other_bag);
        Ok(ThreadSafeBag {
            inner: Arc::new(RwLock::new(result)),
        })
    }
    
    /// Creates an intersection of this bag with another bag.
    pub fn intersection(&self, other: &ThreadSafeBag) -> ContainerResult<ThreadSafeBag> {
        let self_bag = self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        let other_bag = other
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        
        let result = self_bag.intersection(&other_bag);
        Ok(ThreadSafeBag {
            inner: Arc::new(RwLock::new(result)),
        })
    }
    
    /// Creates a difference of this bag with another bag.
    pub fn difference(&self, other: &ThreadSafeBag) -> ContainerResult<ThreadSafeBag> {
        let self_bag = self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        let other_bag = other
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        
        let result = self_bag.difference(&other_bag);
        Ok(ThreadSafeBag {
            inner: Arc::new(RwLock::new(result)),
        })
    }
    
    /// Creates a product of this bag with another bag.
    pub fn product(&self, other: &ThreadSafeBag) -> ContainerResult<ThreadSafeBag> {
        let self_bag = self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        let other_bag = other
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        
        let result = self_bag.product(&other_bag);
        Ok(ThreadSafeBag {
            inner: Arc::new(RwLock::new(result)),
        })
    }

    /// Tests if this bag is a subbag of another bag.
    pub fn is_subbag(&self, other: &ThreadSafeBag) -> ContainerResult<bool> {
        let self_bag = self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        let other_bag = other
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        
        Ok(self_bag.is_subbag(&other_bag))
    }
}

impl Default for ThreadSafeBag {
    fn default() -> Self {
        Self::new()
    }
}

impl std::iter::FromIterator<Value> for Bag {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        let mut bag = Self::new();
        for value in iter {
            bag.adjoin(value);
        }
        bag
    }
}

impl std::iter::FromIterator<Value> for ThreadSafeBag {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        Self {
            inner: Arc::new(RwLock::new(iter.into_iter().collect())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bag_new() {
        let bag = Bag::new();
        assert!(bag.is_empty());
        assert_eq!(bag.total_size(), 0);
        assert_eq!(bag.unique_size(), 0);
    }
    
    #[test]
    fn test_bag_adjoin() {
        let mut bag = Bag::new();
        let v1 = Value::number(42.0);
        let v2 = Value::string("hello");
        
        assert_eq!(bag.adjoin(v1.clone()), 1);
        assert_eq!(bag.total_size(), 1);
        assert_eq!(bag.unique_size(), 1);
        assert!(bag.contains(&v1));
        assert_eq!(bag.element_count(&v1), 1);
        
        // Adding the same element again increments count
        assert_eq!(bag.adjoin(v1.clone()), 2);
        assert_eq!(bag.total_size(), 2);
        assert_eq!(bag.unique_size(), 1);
        assert_eq!(bag.element_count(&v1), 2);
        
        assert_eq!(bag.adjoin(v2.clone()), 1);
        assert_eq!(bag.total_size(), 3);
        assert_eq!(bag.unique_size(), 2);
        assert!(bag.contains(&v2));
    }
    
    #[test]
    fn test_bag_delete() {
        let mut bag = Bag::new();
        let v1 = Value::number(42.0);
        
        bag.adjoin(v1.clone());
        bag.adjoin(v1.clone());
        bag.adjoin(v1.clone());
        assert_eq!(bag.element_count(&v1), 3);
        
        assert!(bag.delete(&v1));
        assert_eq!(bag.element_count(&v1), 2);
        assert_eq!(bag.total_size(), 2);
        
        assert!(bag.delete(&v1));
        assert_eq!(bag.element_count(&v1), 1);
        
        assert!(bag.delete(&v1));
        assert_eq!(bag.element_count(&v1), 0);
        assert!(!bag.contains(&v1));
        assert_eq!(bag.total_size(), 0);
        
        // Deleting a non-existent element should return false
        assert!(!bag.delete(&v1));
    }
    
    #[test]
    fn test_bag_increment_decrement() {
        let mut bag = Bag::new();
        let v1 = Value::number(42.0);
        
        assert_eq!(bag.increment(&v1), 1);
        assert_eq!(bag.increment(&v1), 2);
        assert_eq!(bag.element_count(&v1), 2);
        
        assert_eq!(bag.decrement(&v1), 1);
        assert_eq!(bag.element_count(&v1), 1);
        
        assert_eq!(bag.decrement(&v1), 0);
        assert!(!bag.contains(&v1));
        
        // Decrementing non-existent element
        assert_eq!(bag.decrement(&v1), 0);
    }
    
    #[test]
    fn test_bag_operations() {
        let mut bag1 = Bag::new();
        let mut bag2 = Bag::new();
        
        let v1 = Value::number(1.0);
        let v2 = Value::number(2.0);
        let v3 = Value::number(3.0);
        
        bag1.adjoin_count(v1.clone(), 2);
        bag1.adjoin_count(v2.clone(), 3);
        
        bag2.adjoin_count(v2.clone(), 1);
        bag2.adjoin_count(v3.clone(), 2);
        
        // Union (sum)
        let union = bag1.union(&bag2);
        assert_eq!(union.element_count(&v1), 2);
        assert_eq!(union.element_count(&v2), 4); // 3 + 1
        assert_eq!(union.element_count(&v3), 2);
        assert_eq!(union.total_size(), 8);
        
        // Intersection (minimum)
        let intersection = bag1.intersection(&bag2);
        assert_eq!(intersection.element_count(&v1), 0); // min(2, 0)
        assert_eq!(intersection.element_count(&v2), 1); // min(3, 1)
        assert_eq!(intersection.element_count(&v3), 0); // min(0, 2)
        assert_eq!(intersection.total_size(), 1);
        
        // Difference
        let diff = bag1.difference(&bag2);
        assert_eq!(diff.element_count(&v1), 2); // 2 - 0
        assert_eq!(diff.element_count(&v2), 2); // 3 - 1
        assert_eq!(diff.element_count(&v3), 0); // 0 - 2 = 0
        assert_eq!(diff.total_size(), 4);
    }
    
    #[test]
    fn test_bag_product() {
        let mut bag1 = Bag::new();
        let mut bag2 = Bag::new();
        
        let v1 = Value::number(1.0);
        let v2 = Value::number(2.0);
        
        bag1.adjoin_count(v1.clone(), 3);
        bag1.adjoin_count(v2.clone(), 2);
        
        bag2.adjoin_count(v1.clone(), 2);
        bag2.adjoin_count(v2.clone(), 4);
        
        let product = bag1.product(&bag2);
        assert_eq!(product.element_count(&v1), 6); // 3 * 2
        assert_eq!(product.element_count(&v2), 8); // 2 * 4
        assert_eq!(product.total_size(), 14);
    }
    
    #[test]
    fn test_bag_subbag() {
        let mut bag1 = Bag::new();
        let mut bag2 = Bag::new();
        
        let v1 = Value::number(1.0);
        let v2 = Value::number(2.0);
        
        bag1.adjoin_count(v1.clone(), 1);
        bag1.adjoin_count(v2.clone(), 2);
        
        bag2.adjoin_count(v1.clone(), 2);
        bag2.adjoin_count(v2.clone(), 3);
        
        assert!(bag1.is_subbag(&bag2));
        assert!(!bag2.is_subbag(&bag1));
    }
    
    #[test]
    fn test_bag_to_set() {
        let mut bag = Bag::new();
        let v1 = Value::number(1.0);
        let v2 = Value::number(2.0);
        
        bag.adjoin_count(v1.clone(), 3);
        bag.adjoin_count(v2.clone(), 2);
        
        let set = bag.to_set();
        assert_eq!(set.size(), 2);
        assert!(set.contains(&v1));
        assert!(set.contains(&v2));
    }
    
    #[test]
    fn test_thread_safe_bag() {
        let bag = ThreadSafeBag::new();
        let v1 = Value::number(42.0);
        
        assert_eq!(bag.adjoin(v1.clone()).unwrap(), 1);
        assert_eq!(bag.total_size().unwrap(), 1);
        assert!(bag.contains(&v1).unwrap());
        
        assert_eq!(bag.increment(&v1).unwrap(), 2);
        assert_eq!(bag.element_count(&v1).unwrap(), 2);
        
        assert!(bag.delete(&v1).unwrap());
        assert_eq!(bag.element_count(&v1).unwrap(), 1);
    }
}