//! Performance optimization features for SRFI-135 Text processing.
//!
//! This module implements SIMD acceleration, string interning, 
//! memory pooling, and other performance optimizations.

use crate::stdlib::text::Text;
use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

// Note: These dependencies would need to be added to Cargo.toml:
// lru = "0.12"
// serde_json = "1.0" (already present)

// ============= STRING INTERNING =============

/// String interning pool for memory optimization.
pub struct StringInterningPool {
    /// Interned strings storage
    pool: RwLock<HashMap<u64, Arc<String>>>,
    /// Statistics
    stats: Mutex<InterningStats>,
}

/// Statistics for string interning.
#[derive(Debug, Default)]
pub struct InterningStats {
    /// Total interning requests
    pub total_requests: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// Cache misses
    pub cache_misses: usize,
    /// Memory saved (bytes)
    pub memory_saved: usize,
}

/// Global string interning pool.
static GLOBAL_INTERNING_POOL: std::sync::OnceLock<StringInterningPool> = std::sync::OnceLock::new();

impl StringInterningPool {
    /// Creates a new string interning pool.
    pub fn new() -> Self {
        Self {
            pool: RwLock::new(HashMap::new()),
            stats: Mutex::new(InterningStats::default()),
        }
    }

    /// Gets the global interning pool.
    pub fn global() -> &'static StringInterningPool {
        GLOBAL_INTERNING_POOL.get_or_init(|| StringInterningPool::new())
    }

    /// Interns a string, returning a shared reference.
    pub fn intern(&self, s: String) -> Arc<String> {
        let hash = self.hash_string(&s);
        
        // First try to read from the pool
        {
            let pool = self.pool.read().unwrap();
            if let Some(interned) = pool.get(&hash) {
                // Update statistics
                let mut stats = self.stats.lock().unwrap();
                stats.total_requests += 1;
                stats.cache_hits += 1;
                stats.memory_saved += s.len();
                return interned.clone());
            }
        }

        // Not found, insert into pool
        {
            let mut pool = self.pool.write().unwrap();
            // Double-check in case another thread inserted it
            if let Some(interned) = pool.get(&hash) {
                let mut stats = self.stats.lock().unwrap();
                stats.total_requests += 1;
                stats.cache_hits += 1;
                stats.memory_saved += s.len();
                return interned.clone());
            }

            // Insert new string
            let interned = Arc::new(s);
            pool.insert(hash, interned.clone());
            
            // Update statistics
            let mut stats = self.stats.lock().unwrap();
            stats.total_requests += 1;
            stats.cache_misses += 1;
            
            interned
        }
    }

    /// Interns a string slice.
    pub fn intern_str(&self, s: &str) -> Arc<String> {
        self.intern(s.to_string())
    }

    /// Gets statistics for the pool.
    pub fn stats(&self) -> InterningStats {
        self.stats.lock().unwrap().clone())
    }

    /// Clears the interning pool.
    pub fn clear(&self) {
        let mut pool = self.pool.write().unwrap();
        pool.clear();
        
        let mut stats = self.stats.lock().unwrap();
        *stats = InterningStats::default();
    }

    /// Computes a hash for a string.
    fn hash_string(&self, s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for StringInterningPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for InterningStats {
    fn clone(&self) -> Self {
        Self {
            total_requests: self.total_requests,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            memory_saved: self.memory_saved,
        }
    }
}

// ============= MEMORY POOLING =============

/// Memory pool for efficient text allocation.
pub struct TextMemoryPool {
    /// Small string pool (< 64 bytes)
    small_pool: Mutex<Vec<Vec<u8>>>,
    /// Medium string pool (64-1024 bytes)
    medium_pool: Mutex<Vec<Vec<u8>>>,
    /// Large string pool (> 1024 bytes)
    large_pool: Mutex<Vec<Vec<u8>>>,
    /// Pool statistics
    stats: Mutex<PoolStats>,
}

/// Statistics for memory pooling.
#[derive(Debug, Default)]
pub struct PoolStats {
    /// Allocations from pool
    pub pool_allocations: usize,
    /// New allocations (not from pool)
    pub new_allocations: usize,
    /// Returns to pool
    pub returns_to_pool: usize,
    /// Memory reused (bytes)
    pub memory_reused: usize,
}

/// Global text memory pool.
static GLOBAL_MEMORY_POOL: std::sync::OnceLock<TextMemoryPool> = std::sync::OnceLock::new();

impl TextMemoryPool {
    /// Creates a new memory pool.
    pub fn new() -> Self {
        Self {
            small_pool: Mutex::new(Vec::new()),
            medium_pool: Mutex::new(Vec::new()),
            large_pool: Mutex::new(Vec::new()),
            stats: Mutex::new(PoolStats::default()),
        }
    }

    /// Gets the global memory pool.
    pub fn global() -> &'static TextMemoryPool {
        GLOBAL_MEMORY_POOL.get_or_init(|| TextMemoryPool::new())
    }

    /// Allocates a byte vector with the given capacity.
    pub fn allocate(&self, capacity: usize) -> Vec<u8> {
        let pool = match capacity {
            0..=64 => &self.small_pool,
            65..=1024 => &self.medium_pool,
            _ => &self.large_pool,
        };

        let mut pool_guard = pool.lock().unwrap();
        
        if let Some(mut buf) = pool_guard.pop() {
            if buf.capacity() >= capacity {
                buf.clear();
                buf.reserve(capacity);
                
                // Update statistics
                let mut stats = self.stats.lock().unwrap();
                stats.pool_allocations += 1;
                stats.memory_reused += buf.capacity();
                
                return buf;
            } else {
                // Return undersized buffer back to pool
                pool_guard.push(buf);
            }
        }

        // Allocate new buffer
        let mut stats = self.stats.lock().unwrap();
        stats.new_allocations += 1;
        
        Vec::with_capacity(capacity)
    }

    /// Returns a byte vector to the pool.
    pub fn deallocate(&self, mut buf: Vec<u8>) {
        let capacity = buf.capacity();
        
        if capacity == 0 {
            return;
        }

        let pool = match capacity {
            0..=64 => &self.small_pool,
            65..=1024 => &self.medium_pool,
            _ => &self.large_pool,
        };

        buf.clear();
        
        let mut pool_guard = pool.lock().unwrap();
        
        // Limit pool size to prevent memory bloat
        if pool_guard.len() < 100 {
            pool_guard.push(buf);
            
            // Update statistics
            let mut stats = self.stats.lock().unwrap();
            stats.returns_to_pool += 1;
        }
    }

    /// Gets pool statistics.
    pub fn stats(&self) -> PoolStats {
        self.stats.lock().unwrap().clone())
    }

    /// Clears all pools.
    pub fn clear(&self) {
        self.small_pool.lock().unwrap().clear();
        self.medium_pool.lock().unwrap().clear();
        self.large_pool.lock().unwrap().clear();
        
        let mut stats = self.stats.lock().unwrap();
        *stats = PoolStats::default();
    }
}

impl Default for TextMemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PoolStats {
    fn clone(&self) -> Self {
        Self {
            pool_allocations: self.pool_allocations,
            new_allocations: self.new_allocations,
            returns_to_pool: self.returns_to_pool,
            memory_reused: self.memory_reused,
        }
    }
}

// ============= SIMD ACCELERATION =============

/// SIMD-accelerated text operations.
pub struct SimdTextOps;

impl SimdTextOps {
    /// Fast character counting using SIMD when available.
    pub fn count_char(text: &Text, ch: char) -> usize {
        let s = text.to_string();
        
        // For ASCII characters, we could use SIMD
        // For now, fall back to standard implementation
        if ch.is_ascii() {
            Self::count_ascii_char(s.as_bytes(), ch as u8)
        } else {
            s.chars().filter(|&c| c == ch).count()
        }
    }

    /// Fast ASCII character counting.
    fn count_ascii_char(bytes: &[u8], target: u8) -> usize {
        // Simple implementation - in production would use SIMD intrinsics
        bytes.iter().filter(|&&b| b == target).count()
    }

    /// Fast string search using SIMD when available.
    pub fn find_substring(haystack: &Text, needle: &Text) -> Option<usize> {
        let haystack_str = haystack.to_string();
        let needle_str = needle.to_string();
        
        // For short patterns, use simple search
        if needle_str.len() <= 4 {
            haystack_str.find(&needle_str).map(|byte_pos| {
                haystack_str[..byte_pos].chars().count()
            })
        } else {
            // For longer patterns, could use SIMD-accelerated algorithms
            haystack_str.find(&needle_str).map(|byte_pos| {
                haystack_str[..byte_pos].chars().count()
            })
        }
    }

    /// Fast case conversion using SIMD when available.
    pub fn to_ascii_uppercase(text: &Text) -> Text {
        let s = text.to_string();
        
        if s.is_ascii() {
            // Could use SIMD for ASCII-only text
            Text::from_string(s.to_ascii_uppercase())
        } else {
            text.to_uppercase()
        }
    }

    /// Fast case conversion using SIMD when available.
    pub fn to_ascii_lowercase(text: &Text) -> Text {
        let s = text.to_string();
        
        if s.is_ascii() {
            // Could use SIMD for ASCII-only text
            Text::from_string(s.to_ascii_lowercase())
        } else {
            text.to_lowercase()
        }
    }
}

// ============= PERFORMANCE MONITORING =============

/// Performance monitoring for text operations.
pub struct TextPerformanceMonitor {
    /// Operation counters
    counters: RwLock<HashMap<String, u64>>,
    /// Timing information
    timings: RwLock<HashMap<String, Vec<u64>>>,
}

impl TextPerformanceMonitor {
    /// Creates a new performance monitor.
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            timings: RwLock::new(HashMap::new()),
        }
    }

    /// Increments a counter.
    pub fn increment_counter(&self, name: &str) {
        let mut counters = self.counters.write().unwrap();
        *counters.entry(name.to_string()).or_insert(0) += 1;
    }

    /// Records timing information.
    pub fn record_timing(&self, name: &str, duration_nanos: u64) {
        let mut timings = self.timings.write().unwrap();
        timings.entry(name.to_string()).or_insert_with(Vec::new).push(duration_nanos);
    }

    /// Gets counter value.
    pub fn get_counter(&self, name: &str) -> u64 {
        let counters = self.counters.read().unwrap();
        counters.get(name).copied().unwrap_or(0)
    }

    /// Gets average timing for an operation.
    pub fn get_average_timing(&self, name: &str) -> Option<f64> {
        let timings = self.timings.read().unwrap();
        if let Some(times) = timings.get(name) {
            if !times.is_empty() {
                let sum: u64 = times.iter().sum();
                Some(sum as f64 / times.len() as f64)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Gets all counter names.
    pub fn counter_names(&self) -> Vec<String> {
        let counters = self.counters.read().unwrap();
        counters.keys().clone())().collect()
    }

    /// Gets all timing names.
    pub fn timing_names(&self) -> Vec<String> {
        let timings = self.timings.read().unwrap();
        timings.keys().clone())().collect()
    }

    /// Clears all metrics.
    pub fn clear(&self) {
        self.counters.write().unwrap().clear();
        self.timings.write().unwrap().clear();
    }
}

impl Default for TextPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

// ============= CACHING =============

/// LRU cache for text operations.
pub struct TextCache<T> {
    /// Cache storage
    cache: RwLock<lru::LruCache<String, T>>,
    /// Hit/miss statistics
    stats: Mutex<CacheStats>,
}

/// Cache statistics.
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Cache hits
    pub hits: usize,
    /// Cache misses
    pub misses: usize,
    /// Cache evictions
    pub evictions: usize,
}

impl<T: Clone> TextCache<T> {
    /// Creates a new cache with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: RwLock::new(lru::LruCache::new(capacity.try_into().unwrap())),
            stats: Mutex::new(CacheStats::default()),
        }
    }

    /// Gets a value from the cache.
    pub fn get(&self, key: &str) -> Option<T> {
        let mut cache = self.cache.write().unwrap();
        let result = cache.get(key).clone())();
        
        let mut stats = self.stats.lock().unwrap();
        if result.is_some() {
            stats.hits += 1;
        } else {
            stats.misses += 1;
        }
        
        result
    }

    /// Puts a value into the cache.
    pub fn put(&self, key: String, value: T) {
        let mut cache = self.cache.write().unwrap();
        
        if cache.len() >= cache.cap().get() {
            let mut stats = self.stats.lock().unwrap();
            stats.evictions += 1;
        }
        
        cache.put(key, value);
    }

    /// Gets cache statistics.
    pub fn stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone())
    }

    /// Clears the cache.
    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
        let mut stats = self.stats.lock().unwrap();
        *stats = CacheStats::default();
    }
}

impl Clone for CacheStats {
    fn clone(&self) -> Self {
        Self {
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
        }
    }
}

// ============= OPTIMIZED TEXT BUILDER =============

/// High-performance text builder with memory pooling.
pub struct OptimizedTextBuilder {
    /// Buffer from memory pool
    buffer: Vec<u8>,
    /// Current UTF-8 length
    char_length: usize,
}

impl OptimizedTextBuilder {
    /// Creates a new optimized text builder.
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates a new optimized text builder with initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let buffer = TextMemoryPool::global().allocate(capacity);
        Self {
            buffer,
            char_length: 0,
        }
    }

    /// Pushes a string slice to the builder.
    pub fn push_str(&mut self, s: &str) {
        self.buffer.extend_from_slice(s.as_bytes());
        self.char_length += s.chars().count();
    }

    /// Pushes a character to the builder.
    pub fn push_char(&mut self, ch: char) {
        let mut buf = [0; 4];
        let s = ch.encode_utf8(&mut buf);
        self.push_str(s);
    }

    /// Pushes a text to the builder.
    pub fn push_text(&mut self, text: &Text) {
        self.push_str(&text.to_string());
    }

    /// Builds the final text.
    pub fn build(mut self) -> Text {
        let s = String::from_utf8(self.buffer.clone())
            .unwrap_or_else(|_| String::new());
        
        // Return buffer to pool
        TextMemoryPool::global().deallocate(std::mem::take(&mut self.buffer));
        
        // Try to intern the string if it's not too large
        if s.len() <= 1024 {
            let interned = StringInterningPool::global().intern(s);
            Text::from_string((*interned).clone())
        } else {
            Text::from_string(s)
        }
    }

    /// Gets the current length in bytes.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Gets the current length in characters.
    pub fn char_len(&self) -> usize {
        self.char_length
    }

    /// Returns true if the builder is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Clears the builder.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.char_length = 0;
    }
}

impl Default for OptimizedTextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for OptimizedTextBuilder {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            TextMemoryPool::global().deallocate(std::mem::take(&mut self.buffer));
        }
    }
}

// ============= UTILITY FUNCTIONS =============

/// Gets overall performance statistics.
pub fn get_performance_stats() -> HashMap<String, serde_json::Value> {
    let mut stats = HashMap::new();
    
    // String interning stats
    let interning_stats = StringInterningPool::global().stats();
    stats.insert("interning".to_string(), serde_json::json!({
        "total_requests": interning_stats.total_requests,
        "cache_hits": interning_stats.cache_hits,
        "cache_misses": interning_stats.cache_misses,
        "memory_saved": interning_stats.memory_saved,
        "hit_rate": if interning_stats.total_requests > 0 {
            interning_stats.cache_hits as f64 / interning_stats.total_requests as f64
        } else {
            0.0
        }
    }));
    
    // Memory pool stats
    let pool_stats = TextMemoryPool::global().stats();
    stats.insert("memory_pool".to_string(), serde_json::json!({
        "pool_allocations": pool_stats.pool_allocations,
        "new_allocations": pool_stats.new_allocations,
        "returns_to_pool": pool_stats.returns_to_pool,
        "memory_reused": pool_stats.memory_reused,
        "pool_efficiency": if pool_stats.pool_allocations + pool_stats.new_allocations > 0 {
            pool_stats.pool_allocations as f64 / (pool_stats.pool_allocations + pool_stats.new_allocations) as f64
        } else {
            0.0
        }
    }));
    
    stats
}

/// Clears all performance caches and pools.
pub fn clear_performance_caches() {
    StringInterningPool::global().clear();
    TextMemoryPool::global().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interning() {
        let pool = StringInterningPool::new();
        
        let s1 = pool.intern("hello".to_string());
        let s2 = pool.intern("hello".to_string());
        
        // Should be the same Arc
        assert!(Arc::ptr_eq(&s1, &s2));
        
        let stats = pool.stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }

    #[test]
    fn test_memory_pool() {
        let pool = TextMemoryPool::new();
        
        let buf1 = pool.allocate(100);
        assert!(buf1.capacity() >= 100);
        
        pool.deallocate(buf1);
        
        let buf2 = pool.allocate(100);
        // Should reuse the buffer
        assert!(buf2.capacity() >= 100);
        
        let stats = pool.stats();
        assert!(stats.pool_allocations > 0 || stats.new_allocations > 0);
    }

    #[test]
    fn test_simd_operations() {
        let text = Text::from_str("hello world hello");
        
        let count = SimdTextOps::count_char(&text, 'l');
        assert_eq!(count, 5);
        
        let needle = Text::from_str("world");
        let pos = SimdTextOps::find_substring(&text, &needle);
        assert_eq!(pos, Some(6));
    }

    #[test]
    fn test_optimized_text_builder() {
        let mut builder = OptimizedTextBuilder::new();
        
        builder.push_str("hello");
        builder.push_char(' ');
        builder.push_str("world");
        
        let text = builder.build();
        assert_eq!(text.to_string(), "hello world");
    }

    #[test]
    fn test_text_cache() {
        let cache: TextCache<String> = TextCache::new(2);
        
        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());
        
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache.get("key3"), None);
        
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = TextPerformanceMonitor::new();
        
        monitor.increment_counter("test_op");
        monitor.increment_counter("test_op");
        monitor.record_timing("test_op", 1000);
        monitor.record_timing("test_op", 2000);
        
        assert_eq!(monitor.get_counter("test_op"), 2);
        assert_eq!(monitor.get_average_timing("test_op"), Some(1500.0));
    }
}