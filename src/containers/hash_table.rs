//! High-performance hash table implementation (SRFI-125 compliant).
//!
//! This module provides both single-threaded and thread-safe hash table
//! implementations using a hybrid approach of open addressing with
//! Robin Hood hashing and chaining for collision resolution.

use crate::eval::value::Value;
use super::comparator::HashComparator;
use super::{Container, ContainerError, ContainerResult, load_factors, capacities};
use std::sync::{Arc, RwLock};

/// Entry in the hash table bucket
#[derive(Clone, Debug)]
struct Entry {
    key: Value,
    value: Value,
    hash: u64,
    distance: usize, // Distance from ideal position (for Robin Hood hashing)
}

impl Entry {
    fn new(key: Value, value: Value, hash: u64, distance: usize) -> Self {
        Self {
            key,
            value,
            hash,
            distance,
        }
    }
}

/// Single-threaded hash table implementation
#[derive(Clone, Debug)]
pub struct HashTable {
    /// Storage buckets
    buckets: Vec<Option<Entry>>,
    /// Number of entries
    size: usize,
    /// Load factor threshold
    load_factor: f64,
    /// Comparator for keys
    comparator: HashComparator,
    /// Name for debugging
    name: Option<String>,
}

impl HashTable {
    /// Creates a new hash table with default capacity and comparator
    pub fn new() -> Self {
        Self::with_capacity_and_comparator(
            capacities::DEFAULT_HASH_TABLE_CAPACITY,
            HashComparator::with_default(),
        )
    }
    
    /// Creates a new hash table with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_comparator(capacity, HashComparator::with_default())
    }
    
    /// Creates a new hash table with custom comparator
    pub fn with_comparator(comparator: HashComparator) -> Self {
        Self::with_capacity_and_comparator(
            capacities::DEFAULT_HASH_TABLE_CAPACITY,
            comparator,
        )
    }
    
    /// Creates a new hash table with specified capacity and comparator
    pub fn with_capacity_and_comparator(capacity: usize, comparator: HashComparator) -> Self {
        let capacity = super::utils::next_power_of_two(capacity.max(1));
        Self {
            buckets: vec![None; capacity],
            size: 0,
            load_factor: load_factors::MAX_LOAD_FACTOR,
            comparator,
            name: None,
        }
    }
    
    /// Creates a named hash table for debugging
    pub fn with_name(name: impl Into<String>) -> Self {
        let mut table = Self::new();
        table.name = Some(name.into());
        table
    }
    
    /// Gets the current capacity
    pub fn capacity(&self) -> usize {
        self.buckets.len()
    }
    
    /// Gets the current load factor
    pub fn load_factor(&self) -> f64 {
        if self.buckets.is_empty() {
            0.0
        } else {
            self.size as f64 / self.buckets.len() as f64
        }
    }
    
    /// Inserts a key-value pair into the hash table
    pub fn insert(&mut self, key: Value, value: Value) -> Option<Value> {
        if self.needs_resize() {
            self.resize();
        }
        
        let hash = self.comparator.hash(&key);
        let ideal_pos = (hash as usize) & (self.buckets.len() - 1);
        let mut distance = 0;
        let new_entry = Entry::new(key.clone(), value, hash, distance);
        
        loop {
            let pos = (ideal_pos + distance) & (self.buckets.len() - 1);
            
            match &mut self.buckets[pos] {
                None => {
                    // Empty slot found
                    self.buckets[pos] = Some(Entry::new(key, new_entry.value, hash, distance));
                    self.size += 1;
                    return None;
                }
                Some(existing_entry) => {
                    // Check if key already exists
                    if existing_entry.hash == hash && self.comparator.eq(&existing_entry.key, &key) {
                        // Replace existing value
                        let old_value = existing_entry.value.clone();
                        existing_entry.value = new_entry.value;
                        return Some(old_value);
                    }
                    
                    // Robin Hood hashing: if our distance is greater than the existing entry's,
                    // swap them and continue with the displaced entry
                    if distance > existing_entry.distance {
                        let displaced = existing_entry.clone();
                        *existing_entry = Entry::new(key, new_entry.value, hash, distance);
                        
                        // Continue inserting the displaced entry
                        return self.insert_displaced(displaced, pos + 1);
                    }
                    
                    distance += 1;
                }
            }
        }
    }
    
    /// Helper function to insert a displaced entry
    fn insert_displaced(&mut self, mut entry: Entry, start_pos: usize) -> Option<Value> {
        let mut pos = start_pos;
        
        loop {
            pos &= self.buckets.len() - 1;
            entry.distance = self.distance_from_ideal(entry.hash, pos);
            
            match &mut self.buckets[pos] {
                None => {
                    self.buckets[pos] = Some(entry);
                    self.size += 1;
                    return None;
                }
                Some(existing_entry) => {
                    if entry.distance > existing_entry.distance {
                        std::mem::swap(&mut entry, existing_entry);
                    }
                    pos += 1;
                }
            }
        }
    }
    
    /// Gets a value by key
    pub fn get(&self, key: &Value) -> Option<&Value> {
        let hash = self.comparator.hash(key);
        let mut pos = (hash as usize) & (self.buckets.len() - 1);
        let mut distance = 0;
        
        loop {
            match &self.buckets[pos] {
                None => return None,
                Some(entry) => {
                    if entry.hash == hash && self.comparator.eq(&entry.key, key) {
                        return Some(&entry.value);
                    }
                    
                    // If we've gone farther than this entry's distance, key doesn't exist
                    if distance > entry.distance {
                        return None;
                    }
                    
                    distance += 1;
                    pos = (pos + 1) & (self.buckets.len() - 1);
                }
            }
        }
    }
    
    /// Removes a key-value pair from the hash table
    pub fn remove(&mut self, key: &Value) -> Option<Value> {
        let hash = self.comparator.hash(key);
        let mut pos = (hash as usize) & (self.buckets.len() - 1);
        let mut distance = 0;
        
        loop {
            match &self.buckets[pos] {
                None => return None,
                Some(entry) => {
                    if entry.hash == hash && self.comparator.eq(&entry.key, key) {
                        let removed_value = entry.value.clone();
                        
                        // Shift back entries to fill the gap
                        self.shift_back(pos);
                        self.size -= 1;
                        
                        return Some(removed_value);
                    }
                    
                    if distance > entry.distance {
                        return None;
                    }
                    
                    distance += 1;
                    pos = (pos + 1) & (self.buckets.len() - 1);
                }
            }
        }
    }
    
    /// Shifts entries back after removal to maintain Robin Hood invariant
    fn shift_back(&mut self, mut pos: usize) {
        self.buckets[pos] = None;
        pos = (pos + 1) & (self.buckets.len() - 1);
        
        while let Some(entry) = self.buckets[pos].take() {
            if entry.distance == 0 {
                // Entry is in its ideal position, stop shifting
                self.buckets[pos] = Some(entry);
                break;
            }
            
            // Move entry one position back
            let new_pos = (pos + self.buckets.len() - 1) & (self.buckets.len() - 1);
            let new_distance = entry.distance - 1;
            self.buckets[new_pos] = Some(Entry::new(
                entry.key,
                entry.value,
                entry.hash,
                new_distance,
            ));
            
            pos = (pos + 1) & (self.buckets.len() - 1);
        }
    }
    
    /// Checks if the hash table contains a key
    pub fn contains_key(&self, key: &Value) -> bool {
        self.get(key).is_some()
    }
    
    /// Returns all keys in the hash table
    pub fn keys(&self) -> Vec<Value> {
        self.buckets
            .iter()
            .filter_map(|bucket| bucket.as_ref())
            .map(|entry| entry.key.clone())
            .collect()
    }
    
    /// Returns all values in the hash table
    pub fn values(&self) -> Vec<Value> {
        self.buckets
            .iter()
            .filter_map(|bucket| bucket.as_ref())
            .map(|entry| entry.value.clone())
            .collect()
    }
    
    /// Returns all key-value pairs
    pub fn entries(&self) -> Vec<(Value, Value)> {
        self.buckets
            .iter()
            .filter_map(|bucket| bucket.as_ref())
            .map(|entry| (entry.key.clone(), entry.value.clone()))
            .collect()
    }
    
    /// Iterator over key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.buckets
            .iter()
            .filter_map(|bucket| bucket.as_ref())
            .map(|entry| (&entry.key, &entry.value))
    }
    
    /// Checks if resize is needed
    fn needs_resize(&self) -> bool {
        self.load_factor() > self.load_factor
    }
    
    /// Resizes the hash table
    fn resize(&mut self) {
        let old_capacity = self.buckets.len();
        let old_buckets = std::mem::replace(&mut self.buckets, vec![None; old_capacity * 2]);
        self.size = 0;
        
        for entry in old_buckets.into_iter().flatten() {
            self.insert(entry.key, entry.value);
        }
    }
    
    /// Calculates distance from ideal position
    fn distance_from_ideal(&self, hash: u64, pos: usize) -> usize {
        let ideal_pos = (hash as usize) & (self.buckets.len() - 1);
        if pos >= ideal_pos {
            pos - ideal_pos
        } else {
            pos + self.buckets.len() - ideal_pos
        }
    }
    
    /// Gets statistics about the hash table
    pub fn stats(&self) -> HashTableStats {
        let mut max_distance = 0;
        let mut total_distance = 0;
        let mut chain_lengths = Vec::new();
        let mut current_chain = 0;
        
        for bucket in &self.buckets {
            match bucket {
                Some(entry) => {
                    max_distance = max_distance.max(entry.distance);
                    total_distance += entry.distance;
                    current_chain += 1;
                }
                None => {
                    if current_chain > 0 {
                        chain_lengths.push(current_chain);
                        current_chain = 0;
                    }
                }
            }
        }
        
        if current_chain > 0 {
            chain_lengths.push(current_chain);
        }
        
        let avg_distance = if self.size > 0 {
            total_distance as f64 / self.size as f64
        } else {
            0.0
        };
        
        HashTableStats {
            size: self.size,
            capacity: self.capacity(),
            load_factor: self.load_factor(),
            max_distance,
            avg_distance,
            chain_count: chain_lengths.len(),
            max_chain_length: chain_lengths.iter().copied().max().unwrap_or(0),
        }
    }
}

impl Container for HashTable {
    fn len(&self) -> usize {
        self.size
    }
    
    fn clear(&mut self) {
        self.buckets.fill(None);
        self.size = 0;
    }
}

impl Default for HashTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe hash table implementation
#[derive(Clone, Debug)]
pub struct ThreadSafeHashTable {
    inner: Arc<RwLock<HashTable>>,
}

impl ThreadSafeHashTable {
    /// Creates a new thread-safe hash table
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashTable::new())),
        }
    }
    
    /// Creates a new thread-safe hash table with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashTable::with_capacity(capacity))),
        }
    }
    
    /// Creates a new thread-safe hash table with comparator
    pub fn with_comparator(comparator: HashComparator) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashTable::with_comparator(comparator))),
        }
    }
    
    /// Inserts a key-value pair
    pub fn insert(&self, key: Value, value: Value) -> Option<Value> {
        self.inner.write().unwrap().insert(key, value)
    }
    
    /// Gets a value by key
    pub fn get(&self, key: &Value) -> Option<Value> {
        self.inner.read().unwrap().get(key).cloned()
    }
    
    /// Removes a key-value pair
    pub fn remove(&self, key: &Value) -> Option<Value> {
        self.inner.write().unwrap().remove(key)
    }
    
    /// Checks if the hash table contains a key
    pub fn contains_key(&self, key: &Value) -> bool {
        self.inner.read().unwrap().contains_key(key)
    }
    
    /// Returns the number of entries
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
    
    /// Checks if the hash table is empty
    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }
    
    /// Clears all entries
    pub fn clear(&self) {
        self.inner.write().unwrap().clear()
    }
    
    /// Returns all keys
    pub fn keys(&self) -> Vec<Value> {
        self.inner.read().unwrap().keys()
    }
    
    /// Returns all values
    pub fn values(&self) -> Vec<Value> {
        self.inner.read().unwrap().values()
    }
    
    /// Returns all key-value pairs
    pub fn entries(&self) -> Vec<(Value, Value)> {
        self.inner.read().unwrap().entries()
    }
    
    /// Gets statistics
    pub fn stats(&self) -> HashTableStats {
        self.inner.read().unwrap().stats()
    }
    
    /// Executes a closure with read access to the inner hash table
    pub fn with_read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashTable) -> R,
    {
        f(&self.inner.read().unwrap())
    }
    
    /// Executes a closure with write access to the inner hash table
    pub fn with_write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashTable) -> R,
    {
        f(&mut self.inner.write().unwrap())
    }
}

impl Default for ThreadSafeHashTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about hash table performance
#[derive(Debug, Clone)]
pub struct HashTableStats {
    /// Current number of elements in the hash table
    pub size: usize,
    /// Total capacity of the hash table
    pub capacity: usize,
    /// Current load factor (size / capacity)
    pub load_factor: f64,
    /// Maximum probe distance for any element
    pub max_distance: usize,
    /// Average probe distance across all elements
    pub avg_distance: f64,
    /// Number of collision chains
    pub chain_count: usize,
    /// Length of the longest collision chain
    pub max_chain_length: usize,
}

impl std::fmt::Display for HashTableStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HashTable Stats: size={}, capacity={}, load_factor={:.2}, max_distance={}, avg_distance={:.2}, chain_count={}, max_chain_length={}",
            self.size,
            self.capacity,
            self.load_factor,
            self.max_distance,
            self.avg_distance,
            self.chain_count,
            self.max_chain_length
        )
    }
}

/// SRFI-125 specific functions
impl HashTable {
    /// SRFI-125: hash-table-ref with optional default
    pub fn hash_table_ref(&self, key: &Value, default: Option<Value>) -> ContainerResult<Value> {
        match self.get(key) {
            Some(value) => Ok(value.clone()),
            None => match default {
                Some(default_value) => Ok(default_value),
                None => Err(ContainerError::KeyNotFound {
                    key: format!("{key}"),
                }),
            },
        }
    }
    
    /// SRFI-125: hash-table-set! (multiple key-value pairs)
    pub fn hash_table_set(&mut self, pairs: &[(Value, Value)]) {
        for (key, value) in pairs {
            self.insert(key.clone(), value.clone());
        }
    }
    
    /// SRFI-125: hash-table-delete! (multiple keys)
    pub fn hash_table_delete(&mut self, keys: &[Value]) -> usize {
        let mut deleted = 0;
        for key in keys {
            if self.remove(key).is_some() {
                deleted += 1;
            }
        }
        deleted
    }
    
    /// SRFI-125: hash-table-update!
    pub fn hash_table_update<F>(&mut self, key: &Value, updater: F, default: Option<Value>) -> ContainerResult<()>
    where
        F: FnOnce(&Value) -> Value,
    {
        let current_value = match self.get(key) {
            Some(value) => value.clone(),
            None => match default {
                Some(default_value) => default_value,
                None => return Err(ContainerError::KeyNotFound {
                    key: format!("{key}"),
                }),
            },
        };
        
        let new_value = updater(&current_value);
        self.insert(key.clone(), new_value);
        Ok(())
    }
    
    /// SRFI-125: hash-table-fold
    pub fn hash_table_fold<F, Acc>(&self, mut init: Acc, mut f: F) -> Acc
    where
        F: FnMut(Acc, &Value, &Value) -> Acc,
    {
        for (key, value) in self.iter() {
            init = f(init, key, value);
        }
        init
    }
    
    /// SRFI-125: hash-table-map->list
    pub fn hash_table_map_to_list<F>(&self, mut f: F) -> Vec<Value>
    where
        F: FnMut(&Value, &Value) -> Value,
    {
        self.iter().map(|(k, v)| f(k, v)).collect()
    }
    
    /// SRFI-125: hash-table-for-each
    pub fn hash_table_for_each<F>(&self, mut f: F)
    where
        F: FnMut(&Value, &Value),
    {
        for (key, value) in self.iter() {
            f(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let mut table = HashTable::new();
        
        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
        
        let key1 = Value::string("key1");
        let value1 = Value::number(42.0);
        
        assert_eq!(table.insert(key1.clone(), value1.clone()), None);
        assert_eq!(table.len(), 1);
        assert!(!table.is_empty());
        
        assert_eq!(table.get(&key1), Some(&value1));
        assert!(table.contains_key(&key1));
        
        let value2 = Value::number(24.0);
        assert_eq!(table.insert(key1.clone(), value2.clone()), Some(value1));
        assert_eq!(table.get(&key1), Some(&value2));
        
        assert_eq!(table.remove(&key1), Some(value2));
        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
    }
    
    #[test]
    fn test_resize() {
        let mut table = HashTable::with_capacity(2);
        
        // Insert enough elements to trigger resize
        for i in 0..10 {
            let key = Value::string(format!("key{}", i));
            let value = Value::number(i as f64);
            table.insert(key, value);
        }
        
        assert_eq!(table.len(), 10);
        assert!(table.capacity() > 2);
        
        // Verify all elements are still accessible
        for i in 0..10 {
            let key = Value::string(format!("key{}", i));
            let expected = Value::number(i as f64);
            assert_eq!(table.get(&key), Some(&expected));
        }
    }
    
    #[test]
    fn test_collision_handling() {
        let table = HashTable::with_capacity(4); // Small capacity to force collisions
        
        // This test would need a way to create hash collisions
        // For now, just test that many insertions work correctly
        let mut table = table;
        for i in 0..20 {
            let key = Value::number(i as f64);
            let value = Value::string(format!("value{}", i));
            table.insert(key, value);
        }
        
        assert_eq!(table.len(), 20);
        
        for i in 0..20 {
            let key = Value::number(i as f64);
            let expected = Value::string(format!("value{}", i));
            assert_eq!(table.get(&key), Some(&expected));
        }
    }
    
    #[test]
    fn test_thread_safe_hash_table() {
        let table = ThreadSafeHashTable::new();
        
        let key = Value::string("test");
        let value = Value::number(123.0);
        
        assert_eq!(table.insert(key.clone(), value.clone()), None);
        assert_eq!(table.get(&key), Some(value.clone()));
        assert!(table.contains_key(&key));
        assert_eq!(table.len(), 1);
        
        assert_eq!(table.remove(&key), Some(value));
        assert!(table.is_empty());
    }
    
    #[test]
    fn test_srfi_125_operations() {
        let mut table = HashTable::new();
        
        let key1 = Value::string("key1");
        let key2 = Value::string("key2");
        let value1 = Value::number(1.0);
        let value2 = Value::number(2.0);
        let default = Value::number(0.0);
        
        // Test hash-table-ref with default
            assert_eq!(table.hash_table_ref(&key1, Some(default.clone())), Ok(default.clone()));
        
        // Test hash-table-set! with multiple pairs
        table.hash_table_set(&[(key1.clone(), value1.clone()), (key2.clone(), value2.clone())]);
        assert_eq!(table.len(), 2);
        
        // Test hash-table-update!
        table.hash_table_update(&key1, |v| Value::number(v.as_number().unwrap() + 10.0), None).unwrap();
        assert_eq!(table.get(&key1), Some(&Value::number(11.0)));
        
        // Test hash-table-fold
        let sum = table.hash_table_fold(0.0, |acc, _key, value| {
            acc + value.as_number().unwrap_or(0.0)
        });
        assert_eq!(sum, 13.0); // 11.0 + 2.0
        
        // Test hash-table-delete! with multiple keys
        let deleted = table.hash_table_delete(&[key1, key2]);
        assert_eq!(deleted, 2);
        assert!(table.is_empty());
    }
    
    #[test]
    fn test_stats() {
        let mut table = HashTable::with_capacity(8);
        
        for i in 0..5 {
            let key = Value::number(i as f64);
            let value = Value::string(format!("value{}", i));
            table.insert(key, value);
        }
        
        let stats = table.stats();
        assert_eq!(stats.size, 5);
        assert_eq!(stats.capacity, 8);
        assert!(stats.load_factor < 1.0);
    }
}