//! Set implementation for SRFI-113: Set basic data structures.
//!
//! This module provides a hash-based Set implementation with full SRFI-113 compliance,
//! designed for integration with Lambdust's Value system and SRFI-128 comparators.

use crate::eval::value::Value;
use crate::containers::{Container, ContainerError, ContainerResult};
use crate::containers::comparator::HashComparator;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A hash-based set implementation using SRFI-128 comparators.
///
/// This set uses a HashMap<Value, ()> internally for O(1) average-case operations
/// and integrates with SRFI-128 comparators for element equality testing.
#[derive(Debug, Clone)]
pub struct Set {
    /// Internal hash table storage using HashMap<Value, ()>
    elements: HashMap<Value, ()>,
    /// Comparator for element equality and hashing
    comparator: HashComparator,
    /// Debug name for easier identification
    debug_name: Option<String>,
}

impl Set {
    /// Creates a new empty set with the default comparator.
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            comparator: HashComparator::with_default(),
            debug_name: None,
        }
    }
    
    /// Creates a new empty set with a custom comparator.
    pub fn with_comparator(comparator: HashComparator) -> Self {
        Self {
            elements: HashMap::new(),
            comparator,
            debug_name: None,
        }
    }
    
    
    /// Creates a set from an iterator with a custom comparator.
    pub fn from_iter_with_comparator<I>(iter: I, comparator: HashComparator) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        let mut set = Self::with_comparator(comparator);
        for value in iter {
            set.adjoin(value);
        }
        set
    }
    
    /// Adds an element to the set. Returns true if the element was newly inserted.
    pub fn adjoin(&mut self, value: Value) -> bool {
        self.elements.insert(value, ()).is_none()
    }
    
    /// Removes an element from the set. Returns true if the element was present.
    pub fn delete(&mut self, value: &Value) -> bool {
        self.elements.remove(value).is_some()
    }
    
    /// Tests whether an element is in the set.
    pub fn contains(&self, value: &Value) -> bool {
        self.elements.contains_key(value)
    }
    
    /// Returns the number of elements in the set.
    pub fn size(&self) -> usize {
        self.elements.len()
    }
    
    /// Checks if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
    
    /// Returns an iterator over the elements in the set.
    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.elements.keys()
    }
    
    /// Converts the set to a vector of values.
    pub fn to_vec(&self) -> Vec<Value> {
        self.elements.keys().cloned().collect()
    }
    
    /// Returns the comparator used by this set.
    pub fn comparator(&self) -> &HashComparator {
        &self.comparator
    }
    
    /// Sets the debug name for this set.
    pub fn set_debug_name(&mut self, name: impl Into<String>) {
        self.debug_name = Some(name.into());
    }
    
    /// Gets the debug name of this set.
    pub fn debug_name(&self) -> Option<&str> {
        self.debug_name.as_deref()
    }
    
    /// Clears the debug name.
    pub fn clear_debug_name(&mut self) {
        self.debug_name = None;
    }
    
    /// Creates a union of this set with another set.
    pub fn union(&self, other: &Set) -> Set {
        let mut result = self.clone();
        for value in other.iter() {
            result.adjoin(value.clone());
        }
        result
    }
    
    /// Creates an intersection of this set with another set.
    pub fn intersection(&self, other: &Set) -> Set {
        let mut result = Set::with_comparator(self.comparator.clone());
        for value in self.iter() {
            if other.contains(value) {
                result.adjoin(value.clone());
            }
        }
        result
    }
    
    /// Creates a difference of this set with another set (elements in self but not in other).
    pub fn difference(&self, other: &Set) -> Set {
        let mut result = Set::with_comparator(self.comparator.clone());
        for value in self.iter() {
            if !other.contains(value) {
                result.adjoin(value.clone());
            }
        }
        result
    }
    
    /// Tests if this set is a subset of another set.
    pub fn is_subset(&self, other: &Set) -> bool {
        self.iter().all(|value| other.contains(value))
    }
    
    /// Tests if this set is a superset of another set.
    pub fn is_superset(&self, other: &Set) -> bool {
        other.is_subset(self)
    }
    
    /// Tests if this set is disjoint from another set (no common elements).
    pub fn is_disjoint(&self, other: &Set) -> bool {
        !self.iter().any(|value| other.contains(value))
    }
}

impl Default for Set {
    fn default() -> Self {
        Self::new()
    }
}

impl Container for Set {
    fn len(&self) -> usize {
        self.size()
    }
    
    fn clear(&mut self) {
        self.elements.clear();
    }
}

impl PartialEq for Set {
    fn eq(&self, other: &Self) -> bool {
        self.size() == other.size() && self.is_subset(other)
    }
}

impl Eq for Set {}

/// Thread-safe wrapper for Set operations.
///
/// This provides a thread-safe interface to Set operations using Arc<RwLock<Set>>.
#[derive(Debug, Clone)]
pub struct ThreadSafeSet {
    inner: Arc<RwLock<Set>>,
}

impl ThreadSafeSet {
    /// Creates a new thread-safe empty set.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Set::new())),
        }
    }
    
    /// Creates a new thread-safe set with a custom comparator.
    pub fn with_comparator(comparator: HashComparator) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Set::with_comparator(comparator))),
        }
    }
    
    
    /// Adds an element to the set. Returns true if the element was newly inserted.
    pub fn adjoin(&self, value: Value) -> ContainerResult<bool> {
        self.inner
            .write()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire write lock".to_string(),
            })?
            .adjoin(value)
            .then_some(true)
            .ok_or_else(|| ContainerError::EmptyContainer {
                operation: "adjoin".to_string(),
            })
    }
    
    /// Removes an element from the set. Returns true if the element was present.
    pub fn delete(&self, value: &Value) -> ContainerResult<bool> {
        Ok(self
            .inner
            .write()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire write lock".to_string(),
            })?
            .delete(value))
    }
    
    /// Tests whether an element is in the set.
    pub fn contains(&self, value: &Value) -> ContainerResult<bool> {
        Ok(self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?
            .contains(value))
    }
    
    /// Returns the number of elements in the set.
    pub fn size(&self) -> ContainerResult<usize> {
        Ok(self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?
            .size())
    }
    
    /// Checks if the set is empty.
    pub fn is_empty(&self) -> ContainerResult<bool> {
        Ok(self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?
            .is_empty())
    }
    
    /// Converts the set to a vector of values.
    pub fn to_vec(&self) -> ContainerResult<Vec<Value>> {
        Ok(self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?
            .to_vec())
    }
    
    /// Clears all elements from the set.
    pub fn clear(&self) -> ContainerResult<()> {
        self.inner
            .write()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire write lock".to_string(),
            })?
            .clear();
        Ok(())
    }
    
    /// Sets the debug name for this set.
    pub fn set_debug_name(&self, name: impl Into<String>) -> ContainerResult<()> {
        self.inner
            .write()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire write lock".to_string(),
            })?
            .set_debug_name(name);
        Ok(())
    }
    
    /// Gets the debug name of this set.
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
    
    /// Creates a union of this set with another set.
    pub fn union(&self, other: &ThreadSafeSet) -> ContainerResult<ThreadSafeSet> {
        let self_set = self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        let other_set = other
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        
        let result = self_set.union(&other_set);
        Ok(ThreadSafeSet {
            inner: Arc::new(RwLock::new(result)),
        })
    }
    
    /// Creates an intersection of this set with another set.
    pub fn intersection(&self, other: &ThreadSafeSet) -> ContainerResult<ThreadSafeSet> {
        let self_set = self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        let other_set = other
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        
        let result = self_set.intersection(&other_set);
        Ok(ThreadSafeSet {
            inner: Arc::new(RwLock::new(result)),
        })
    }
    
    /// Tests if this set is a subset of another set.
    pub fn is_subset(&self, other: &ThreadSafeSet) -> ContainerResult<bool> {
        let self_set = self
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        let other_set = other
            .inner
            .read()
            .map_err(|_| ContainerError::InvalidComparator {
                message: "Failed to acquire read lock".to_string(),
            })?;
        
        Ok(self_set.is_subset(&other_set))
    }
}

impl Default for ThreadSafeSet {
    fn default() -> Self {
        Self::new()
    }
}

impl std::iter::FromIterator<Value> for Set {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        let mut set = Self::new();
        for value in iter {
            set.adjoin(value);
        }
        set
    }
}

impl std::iter::FromIterator<Value> for ThreadSafeSet {
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
    fn test_set_new() {
        let set = Set::new();
        assert!(set.is_empty());
        assert_eq!(set.size(), 0);
    }
    
    #[test]
    fn test_set_adjoin() {
        let mut set = Set::new();
        let v1 = Value::number(42.0);
        let v2 = Value::string("hello");
        
        assert!(set.adjoin(v1.clone()));
        assert_eq!(set.size(), 1);
        assert!(set.contains(&v1));
        
        // Adding the same element again should return false
        assert!(!set.adjoin(v1.clone()));
        assert_eq!(set.size(), 1);
        
        assert!(set.adjoin(v2.clone()));
        assert_eq!(set.size(), 2);
        assert!(set.contains(&v2));
    }
    
    #[test]
    fn test_set_delete() {
        let mut set = Set::new();
        let v1 = Value::number(42.0);
        let v2 = Value::string("hello");
        
        set.adjoin(v1.clone());
        set.adjoin(v2.clone());
        assert_eq!(set.size(), 2);
        
        assert!(set.delete(&v1));
        assert_eq!(set.size(), 1);
        assert!(!set.contains(&v1));
        assert!(set.contains(&v2));
        
        // Deleting a non-existent element should return false
        assert!(!set.delete(&v1));
        assert_eq!(set.size(), 1);
    }
    
    #[test]
    fn test_set_operations() {
        let mut set1 = Set::new();
        let mut set2 = Set::new();
        
        let v1 = Value::number(1.0);
        let v2 = Value::number(2.0);
        let v3 = Value::number(3.0);
        
        set1.adjoin(v1.clone());
        set1.adjoin(v2.clone());
        
        set2.adjoin(v2.clone());
        set2.adjoin(v3.clone());
        
        // Union
        let union = set1.union(&set2);
        assert_eq!(union.size(), 3);
        assert!(union.contains(&v1));
        assert!(union.contains(&v2));
        assert!(union.contains(&v3));
        
        // Intersection
        let intersection = set1.intersection(&set2);
        assert_eq!(intersection.size(), 1);
        assert!(intersection.contains(&v2));
        
        // Difference
        let diff = set1.difference(&set2);
        assert_eq!(diff.size(), 1);
        assert!(diff.contains(&v1));
        assert!(!diff.contains(&v2));
    }
    
    #[test]
    fn test_set_subset_superset() {
        let mut set1 = Set::new();
        let mut set2 = Set::new();
        
        let v1 = Value::number(1.0);
        let v2 = Value::number(2.0);
        
        set1.adjoin(v1.clone());
        set2.adjoin(v1.clone());
        set2.adjoin(v2.clone());
        
        assert!(set1.is_subset(&set2));
        assert!(!set1.is_superset(&set2));
        assert!(set2.is_superset(&set1));
        assert!(!set2.is_subset(&set1));
    }
    
    #[test]
    fn test_thread_safe_set() {
        let set = ThreadSafeSet::new();
        let v1 = Value::number(42.0);
        
        assert!(set.adjoin(v1.clone()).unwrap());
        assert_eq!(set.size().unwrap(), 1);
        assert!(set.contains(&v1).unwrap());
        
        assert!(set.delete(&v1).unwrap());
        assert_eq!(set.size().unwrap(), 0);
        assert!(!set.contains(&v1).unwrap());
    }
}