//! High-Performance Feature Evaluation Cache
//!
//! Implements LRU cache with Bloom filter for negative lookups to minimize
//! expensive feature evaluations during compilation.

use crate::ast::FeatureRequirement;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::{Arc, Mutex};

/// Configuration for feature cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Enable Bloom filter for negative caching
    pub use_bloom_filter: bool,
    /// Bloom filter size in bits
    pub bloom_filter_size: usize,
    /// Number of hash functions for Bloom filter
    pub bloom_hash_functions: usize,
    /// Whether to use thread-safe cache (for concurrent access)
    pub thread_safe: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1024,
            use_bloom_filter: true,
            bloom_filter_size: 8192,
            bloom_hash_functions: 3,
            thread_safe: false,
        }
    }
}

/// Cache statistics for monitoring and debugging
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of cache lookups
    pub lookups: u64,
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of entries currently in cache
    pub entries: usize,
    /// Number of evictions due to capacity
    pub evictions: u64,
    /// Bloom filter false positives
    pub bloom_false_positives: u64,
    /// Bloom filter true negatives
    pub bloom_true_negatives: u64,
}

impl CacheStats {
    /// Calculates cache hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        if self.lookups == 0 {
            0.0
        } else {
            (self.hits as f64 / self.lookups as f64) * 100.0
        }
    }

    /// Calculates Bloom filter effectiveness
    pub fn bloom_effectiveness(&self) -> f64 {
        let total_bloom_checks = self.bloom_false_positives + self.bloom_true_negatives;
        if total_bloom_checks == 0 {
            0.0
        } else {
            (self.bloom_true_negatives as f64 / total_bloom_checks as f64) * 100.0
        }
    }
}

/// Simple Bloom filter implementation
#[derive(Debug, Clone)]
struct BloomFilter {
    bits: Vec<bool>,
    size: usize,
    hash_functions: usize,
}

impl BloomFilter {
    fn new(size: usize, hash_functions: usize) -> Self {
        Self {
            bits: vec![false; size],
            size,
            hash_functions,
        }
    }

    fn add(&mut self, item: &FeatureRequirement) {
        for i in 0..self.hash_functions {
            let hash = self.hash_item(item, i);
            let index = hash % self.size;
            self.bits[index] = true;
        }
    }

    fn contains(&self, item: &FeatureRequirement) -> bool {
        for i in 0..self.hash_functions {
            let hash = self.hash_item(item, i);
            let index = hash % self.size;
            if !self.bits[index] {
                return false;
            }
        }
        true
    }

    fn hash_item(&self, item: &FeatureRequirement, seed: usize) -> usize {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        item.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn clear(&mut self) {
        self.bits.fill(false);
    }
}

/// LRU cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    key: FeatureRequirement,
    value: bool,
}

/// High-performance feature evaluation cache
pub struct FeatureCache {
    config: CacheConfig,
    // Single-threaded cache
    cache: Option<LruCache>,
    // Thread-safe cache
    thread_safe_cache: Option<Arc<Mutex<LruCache>>>,
}

/// Internal LRU cache implementation
struct LruCache {
    entries: HashMap<FeatureRequirement, usize>,
    order: VecDeque<CacheEntry>,
    bloom_filter: Option<BloomFilter>,
    stats: CacheStats,
    max_entries: usize,
}

impl LruCache {
    fn new(config: &CacheConfig) -> Self {
        let bloom_filter = if config.use_bloom_filter {
            Some(BloomFilter::new(
                config.bloom_filter_size,
                config.bloom_hash_functions,
            ))
        } else {
            None
        };

        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            bloom_filter,
            stats: CacheStats::default(),
            max_entries: config.max_entries,
        }
    }

    fn get(&mut self, key: &FeatureRequirement) -> Option<bool> {
        self.stats.lookups += 1;

        // Check Bloom filter first for negative lookups
        if let Some(ref bloom) = self.bloom_filter {
            if !bloom.contains(key) {
                self.stats.bloom_true_negatives += 1;
                self.stats.misses += 1;
                return None;
            }
        }

        if let Some(&index) = self.entries.get(key) {
            // Move to front (most recently used)
            let entry = self.order.remove(index).unwrap();
            self.order.push_front(entry.clone());

            // Update indices
            self.update_indices();

            self.stats.hits += 1;
            Some(entry.value)
        } else {
            if self.bloom_filter.is_some() {
                self.stats.bloom_false_positives += 1;
            }
            self.stats.misses += 1;
            None
        }
    }

    fn insert(&mut self, key: FeatureRequirement, value: bool) {
        // Check if already exists
        if self.entries.contains_key(&key) {
            self.remove(&key);
        }

        // Add to Bloom filter
        if let Some(ref mut bloom) = self.bloom_filter {
            bloom.add(&key);
        }

        // Create new entry
        let entry = CacheEntry {
            key: key.clone(),
            value,
        };

        // Add to front
        self.order.push_front(entry);
        self.entries.insert(key, 0);

        // Update indices
        self.update_indices();

        // Evict if necessary
        if self.order.len() > self.max_entries {
            if let Some(evicted) = self.order.pop_back() {
                self.entries.remove(&evicted.key);
                self.stats.evictions += 1;
            }
        }

        self.stats.entries = self.order.len();
    }

    fn remove(&mut self, key: &FeatureRequirement) -> Option<bool> {
        if let Some(&index) = self.entries.get(key) {
            let entry = self.order.remove(index).unwrap();
            self.entries.remove(key);
            self.update_indices();
            self.stats.entries = self.order.len();
            Some(entry.value)
        } else {
            None
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
        if let Some(ref mut bloom) = self.bloom_filter {
            bloom.clear();
        }
        self.stats = CacheStats::default();
    }

    fn stats(&self) -> CacheStats {
        self.stats.clone()
    }

    fn update_indices(&mut self) {
        for (i, entry) in self.order.iter().enumerate() {
            self.entries.insert(entry.key.clone(), i);
        }
    }
}

impl FeatureCache {
    /// Creates a new feature cache with the given configuration
    pub fn new(config: CacheConfig) -> Self {
        if config.thread_safe {
            Self {
                thread_safe_cache: Some(Arc::new(Mutex::new(LruCache::new(&config)))),
                cache: None,
                config,
            }
        } else {
            Self {
                cache: Some(LruCache::new(&config)),
                thread_safe_cache: None,
                config,
            }
        }
    }

    /// Gets a cached evaluation result
    pub fn get(&self, key: &FeatureRequirement) -> Option<bool> {
        if let Some(ref cache) = self.cache {
            // SAFETY: We're using interior mutability pattern here
            // In single-threaded mode, this is safe
            let cache_ptr = cache as *const LruCache as *mut LruCache;
            unsafe { (*cache_ptr).get(key) }
        } else if let Some(ref cache) = self.thread_safe_cache {
            let mut cache_guard = cache.lock().ok()?;
            cache_guard.get(key)
        } else {
            None
        }
    }

    /// Inserts an evaluation result into the cache
    pub fn insert(&self, key: FeatureRequirement, value: bool) {
        if let Some(ref cache) = self.cache {
            // SAFETY: Same as above
            let cache_ptr = cache as *const LruCache as *mut LruCache;
            unsafe { (*cache_ptr).insert(key, value) };
        } else if let Some(ref cache) = self.thread_safe_cache {
            if let Ok(mut cache_guard) = cache.lock() {
                cache_guard.insert(key, value);
            }
        }
    }

    /// Removes an entry from the cache
    pub fn remove(&self, key: &FeatureRequirement) -> Option<bool> {
        if let Some(ref cache) = self.cache {
            let cache_ptr = cache as *const LruCache as *mut LruCache;
            unsafe { (*cache_ptr).remove(key) }
        } else if let Some(ref cache) = self.thread_safe_cache {
            let mut cache_guard = cache.lock().ok()?;
            cache_guard.remove(key)
        } else {
            None
        }
    }

    /// Clears the cache
    pub fn clear(&self) {
        if let Some(ref cache) = self.cache {
            let cache_ptr = cache as *const LruCache as *mut LruCache;
            unsafe { (*cache_ptr).clear() };
        } else if let Some(ref cache) = self.thread_safe_cache {
            if let Ok(mut cache_guard) = cache.lock() {
                cache_guard.clear();
            }
        }
    }

    /// Gets cache statistics
    pub fn stats(&self) -> CacheStats {
        if let Some(ref cache) = self.cache {
            cache.stats()
        } else if let Some(ref cache) = self.thread_safe_cache {
            cache.lock().map(|guard| guard.stats()).unwrap_or_default()
        } else {
            CacheStats::default()
        }
    }

    /// Gets cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}

// Thread-safe clone implementation
impl Clone for FeatureCache {
    fn clone(&self) -> Self {
        if let Some(ref thread_safe_cache) = self.thread_safe_cache {
            Self {
                config: self.config.clone(),
                cache: None,
                thread_safe_cache: Some(Arc::clone(thread_safe_cache)),
            }
        } else {
            // For single-threaded cache, create a new one
            Self::new(self.config.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let config = CacheConfig::default();
        let cache = FeatureCache::new(config);

        let key1 = FeatureRequirement::Feature("test".to_string());
        let key2 = FeatureRequirement::Feature("test2".to_string());

        // Initially empty
        assert_eq!(cache.get(&key1), None);

        // Insert and retrieve
        cache.insert(key1.clone(), true);
        assert_eq!(cache.get(&key1), Some(true));

        // Different key
        assert_eq!(cache.get(&key2), None);
        cache.insert(key2.clone(), false);
        assert_eq!(cache.get(&key2), Some(false));

        // Original key still there
        assert_eq!(cache.get(&key1), Some(true));
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut config = CacheConfig::default();
        config.max_entries = 2;
        let cache = FeatureCache::new(config);

        let key1 = FeatureRequirement::Feature("test1".to_string());
        let key2 = FeatureRequirement::Feature("test2".to_string());
        let key3 = FeatureRequirement::Feature("test3".to_string());

        // Fill cache to capacity
        cache.insert(key1.clone(), true);
        cache.insert(key2.clone(), true);

        // Both should be there
        assert_eq!(cache.get(&key1), Some(true));
        assert_eq!(cache.get(&key2), Some(true));

        // Add third item, should evict least recently used
        cache.insert(key3.clone(), true);

        // key3 should be there, key1 or key2 should be evicted
        assert_eq!(cache.get(&key3), Some(true));
        let stats = cache.stats();
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn test_cache_stats() {
        let config = CacheConfig::default();
        let cache = FeatureCache::new(config);

        let key = FeatureRequirement::Feature("test".to_string());

        // Initial stats
        let stats = cache.stats();
        assert_eq!(stats.lookups, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // Miss
        cache.get(&key);
        let stats = cache.stats();
        assert_eq!(stats.lookups, 1);
        assert_eq!(stats.misses, 1);

        // Insert and hit
        cache.insert(key.clone(), true);
        cache.get(&key);
        let stats = cache.stats();
        assert_eq!(stats.lookups, 2);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_bloom_filter_operations() {
        let mut bloom = BloomFilter::new(1000, 3);

        let key1 = FeatureRequirement::Feature("test1".to_string());
        let key2 = FeatureRequirement::Feature("test2".to_string());

        // Initially empty
        assert!(!bloom.contains(&key1));
        assert!(!bloom.contains(&key2));

        // Add key1
        bloom.add(&key1);
        assert!(bloom.contains(&key1));

        // key2 might or might not be positive (could be false positive)
        // but key1 should always be positive
        assert!(bloom.contains(&key1));
    }

    #[test]
    fn test_cache_with_bloom_filter() {
        let mut config = CacheConfig::default();
        config.use_bloom_filter = true;
        config.bloom_filter_size = 100;

        let cache = FeatureCache::new(config);

        let key1 = FeatureRequirement::Feature("test1".to_string());
        let key2 = FeatureRequirement::Feature("test2".to_string());

        // First miss should update Bloom filter stats
        cache.get(&key1);
        let stats = cache.stats();
        assert!(stats.bloom_true_negatives > 0 || stats.bloom_false_positives > 0);

        // Insert and check Bloom filter effectiveness
        cache.insert(key1.clone(), true);
        cache.get(&key1); // Should hit

        cache.get(&key2); // Should miss, might affect Bloom stats
        let stats = cache.stats();
        assert!(stats.lookups >= 3);
    }

    #[test]
    fn test_thread_safe_cache() {
        let mut config = CacheConfig::default();
        config.thread_safe = true;

        let cache = FeatureCache::new(config);
        let key = FeatureRequirement::Feature("test".to_string());

        cache.insert(key.clone(), true);
        assert_eq!(cache.get(&key), Some(true));

        // Test that it can be cloned (Arc<Mutex<>> behavior)
        let cache2 = cache.clone();
        assert_eq!(cache2.get(&key), Some(true));
    }

    #[test]
    fn test_complex_feature_requirement_caching() {
        let config = CacheConfig::default();
        let cache = FeatureCache::new(config);

        let complex_req = FeatureRequirement::And(vec![
            FeatureRequirement::Feature("lambdust".to_string()),
            FeatureRequirement::Not(Box::new(FeatureRequirement::Feature("debug".to_string()))),
        ]);

        cache.insert(complex_req.clone(), true);
        assert_eq!(cache.get(&complex_req), Some(true));
    }
}
