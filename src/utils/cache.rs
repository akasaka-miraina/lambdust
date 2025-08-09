//! Generic caching utilities for performance optimization.
//!
//! This module provides thread-safe caching mechanisms for memoizing
//! expensive computations, including LRU cache and simple hash-based cache.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// A thread-safe LRU cache for expensive computations.
#[derive(Debug)]
pub struct LruCache<K, V>
where
    K: Clone + Hash + Eq,
    V: Clone,
{
    capacity: usize,
    cache: RwLock<HashMap<K, CacheEntry<V>>>,
    access_order: RwLock<Vec<K>>,
}

/// Entry in the cache with metadata.
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    access_count: u64,
}

impl<K, V> LruCache<K, V>
where
    K: Clone + Hash + Eq,
    V: Clone,
{
    /// Creates a new LRU cache with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: RwLock::new(HashMap::new()),
            access_order: RwLock::new(Vec::new()),
        }
    }

    /// Gets a value from the cache, returning None if not present.
    pub fn get(&self, key: &K) -> Option<V> {
        if let Ok(mut cache) = self.cache.write() {
            if let Some(entry) = cache.get_mut(key) {
                entry.access_count += 1;
                self.update_access_order(key);
                return Some(entry.value.clone());
            }
        }
        None
    }

    /// Inserts or updates a value in the cache.
    pub fn insert(&self, key: K, value: V) {
        if let Ok(mut cache) = self.cache.write() {
            // If at capacity and key doesn't exist, evict LRU
            if cache.len() >= self.capacity && !cache.contains_key(&key) {
                self.evict_lru();
            }

            let entry = CacheEntry {
                value,
                created_at: Instant::now(),
                access_count: 1,
            };

            cache.insert(key.clone(), entry);
            self.update_access_order(&key);
        }
    }

    /// Gets cache statistics.
    pub fn stats(&self) -> CacheStats {
        if let Ok(cache) = self.cache.read() {
            let total_entries = cache.len();
            let total_accesses: u64 = cache.values().map(|entry| entry.access_count).sum();
            let oldest_entry = cache.values()
                .min_by_key(|entry| entry.created_at)
                .map(|entry| entry.created_at);

            CacheStats {
                total_entries,
                capacity: self.capacity,
                total_accesses,
                oldest_entry_age: oldest_entry.map(|instant| instant.elapsed()),
            }
        } else {
            CacheStats::default()
        }
    }

    /// Clears all entries from the cache.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
        if let Ok(mut order) = self.access_order.write() {
            order.clear();
        }
    }

    /// Updates the access order for LRU eviction.
    fn update_access_order(&self, key: &K) {
        if let Ok(mut order) = self.access_order.write() {
            // Remove existing entry if present
            if let Some(pos) = order.iter().position(|k| k == key) {
                order.remove(pos);
            }
            // Add to end (most recently used)
            order.push(key.clone());
        }
    }

    /// Evicts the least recently used entry.
    fn evict_lru(&self) {
        if let Ok(mut order) = self.access_order.write() {
            if let Some(lru_key) = order.first().cloned() {
                order.remove(0);
                if let Ok(mut cache) = self.cache.write() {
                    cache.remove(&lru_key);
                }
            }
        }
    }
}

/// Simple function memoization cache.
#[derive(Debug)]
pub struct MemoCache<K, V>
where
    K: Clone + Hash + Eq,
    V: Clone,
{
    cache: RwLock<HashMap<K, V>>,
    max_size: usize,
    hits: RwLock<u64>,
    misses: RwLock<u64>,
}

impl<K, V> MemoCache<K, V>
where
    K: Clone + Hash + Eq,
    V: Clone,
{
    /// Creates a new memoization cache.
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_size,
            hits: RwLock::new(0),
            misses: RwLock::new(0),
        }
    }

    /// Gets a value or computes it using the provided function.
    pub fn get_or_compute<F>(&self, key: K, compute: F) -> V
    where
        F: FnOnce() -> V,
    {
        // Fast path: check if already cached
        if let Ok(cache) = self.cache.read() {
            if let Some(value) = cache.get(&key) {
                if let Ok(mut hits) = self.hits.write() {
                    *hits += 1;
                }
                return value.clone();
            }
        }

        // Slow path: compute and cache
        let value = compute();
        
        if let Ok(mut misses) = self.misses.write() {
            *misses += 1;
        }

        // Insert into cache
        if let Ok(mut cache) = self.cache.write() {
            // Simple eviction: clear if at max size
            if cache.len() >= self.max_size {
                cache.clear();
            }
            cache.insert(key, value.clone());
        }

        value
    }

    /// Gets cache hit rate as a percentage.
    pub fn hit_rate(&self) -> f64 {
        let hits = if let Ok(hits) = self.hits.read() { *hits } else { 0 };
        let misses = if let Ok(misses) = self.misses.read() { *misses } else { 0 };
        let total = hits + misses;
        
        if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Clears the cache and resets statistics.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
        if let Ok(mut hits) = self.hits.write() {
            *hits = 0;
        }
        if let Ok(mut misses) = self.misses.write() {
            *misses = 0;
        }
    }
}

/// Cache statistics for monitoring performance.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total number of entries currently in the cache
    pub total_entries: usize,
    /// Maximum capacity of the cache
    pub capacity: usize,
    /// Total number of access operations performed
    pub total_accesses: u64,
    /// Age of the oldest entry in the cache
    pub oldest_entry_age: Option<Duration>,
}


/// Global cache instances for commonly used expensive computations.
pub mod global {
    use super::*;
    use once_cell::sync::Lazy;
    
    /// Cache for string to symbol conversion.
    pub static SYMBOL_CACHE: Lazy<MemoCache<String, u64>> = 
        Lazy::new(|| MemoCache::new(1000));
    
    /// Cache for numeric calculations.
    pub static NUMERIC_CACHE: Lazy<MemoCache<String, f64>> = 
        Lazy::new(|| MemoCache::new(500));
    
    /// Cache for type checking results.
    pub static TYPE_CACHE: Lazy<LruCache<String, String>> = 
        Lazy::new(|| LruCache::new(200));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic() {
        let cache = LruCache::new(2);
        
        cache.insert(1, "one");
        cache.insert(2, "two");
        
        assert_eq!(cache.get(&1), Some("one"));
        assert_eq!(cache.get(&2), Some("two"));
        assert_eq!(cache.get(&3), None);
    }

    #[test]
    fn test_lru_cache_eviction() {
        let cache = LruCache::new(2);
        
        cache.insert(1, "one");
        cache.insert(2, "two");
        cache.insert(3, "three"); // Should evict 1
        
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some("two"));
        assert_eq!(cache.get(&3), Some("three"));
    }

    #[test]
    fn test_memo_cache() {
        let cache = MemoCache::new(10);
        
        let mut call_count = 0;
        let compute = || {
            call_count += 1;
            format!("computed_{}", call_count)
        };
        
        let result1 = cache.get_or_compute(1, compute);
        let result2 = cache.get_or_compute(1, compute);
        
        assert_eq!(result1, result2);
        assert_eq!(call_count, 1); // Should only compute once
        assert!(cache.hit_rate() > 0.0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = LruCache::new(5);
        cache.insert(1, "test");
        cache.get(&1);
        cache.get(&1);
        
        let stats = cache.stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.capacity, 5);
        assert_eq!(stats.total_accesses, 3); // 1 insert + 2 gets
    }
}