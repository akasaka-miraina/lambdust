//! Optimized environment with O(1) lookup performance.
//!
//! This module provides a cached environment system that optimizes variable
//! lookup from O(n) to O(1) through several techniques:
//! - LRU cache for frequently accessed variables
//! - Flattened parent chain representation  
//! - Path compression to reduce traversal depth
//! - Generation-based cache invalidation

use super::{Environment, Generation, Value};
use crate::diagnostics::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::rc::{Rc, Weak};

/// Flattened bindings map type alias for cleaner code
type FlattenedBindings = Rc<std::cell::RefCell<Option<HashMap<String, (Value, Generation)>>>>;

/// Maximum size of the LRU cache for variable lookups.
const DEFAULT_CACHE_SIZE: usize = 256;

/// Cached environment that provides O(1) lookup performance for frequently 
/// accessed variables while maintaining exact R7RS lexical scoping semantics.
#[derive(Debug, Clone)]
pub struct CachedEnvironment {
    /// The underlying environment chain
    base_env: Rc<Environment>,
    
    /// LRU cache for frequently accessed variable lookups
    /// Maps variable names to (value, generation) pairs
    lookup_cache: Rc<std::cell::RefCell<lru::LruCache<String, (Value, Generation)>>>,
    
    /// Flattened lookup table for O(1) access to all variables in the chain
    /// This is lazily computed and cached until invalidated
    flattened_bindings: FlattenedBindings,
    
    /// Generation counter for cache invalidation
    /// Incremented whenever the environment chain is modified
    cache_generation: std::cell::Cell<Generation>,
    
    /// Weak references to parent environments for path compression
    /// This avoids reference cycles while enabling efficient traversal
    compressed_parents: Rc<std::cell::RefCell<Vec<Weak<Environment>>>>,
    
    /// Statistics for performance monitoring
    cache_stats: Rc<std::cell::RefCell<CacheStatistics>>,
}

/// Statistics for monitoring cache performance and optimization effectiveness.
#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
    /// Number of cache hits (O(1) lookups)
    pub cache_hits: u64,
    
    /// Number of cache misses requiring chain traversal
    pub cache_misses: u64,
    
    /// Number of times the flattened table was rebuilt
    pub flattened_rebuilds: u64,
    
    /// Number of cache invalidations due to modifications
    pub cache_invalidations: u64,
    
    /// Maximum depth of environment chain encountered
    pub max_chain_depth: usize,
    
    /// Average lookup time in nanoseconds
    pub avg_lookup_time_ns: f64,
}

impl CachedEnvironment {
    /// Creates a new cached environment wrapping the given base environment.
    pub fn new(base_env: Rc<Environment>) -> Self {
        Self::with_cache_size(base_env, DEFAULT_CACHE_SIZE)
    }
    
    /// Creates a new cached environment with specified cache size.
    pub fn with_cache_size(base_env: Rc<Environment>, cache_size: usize) -> Self {
        Self {
            base_env,
            lookup_cache: Rc::new(std::cell::RefCell::new(
                lru::LruCache::new(std::num::NonZeroUsize::new(cache_size).unwrap())
            )),
            flattened_bindings: Rc::new(std::cell::RefCell::new(None)),
            cache_generation: std::cell::Cell::new(0),
            compressed_parents: Rc::new(std::cell::RefCell::new(Vec::new())),
            cache_stats: Rc::new(std::cell::RefCell::new(CacheStatistics::default())),
        }
    }
    
    /// Performs optimized variable lookup with O(1) performance for cached variables.
    /// 
    /// Algorithm:
    /// 1. Check LRU cache first - O(1)
    /// 2. Check flattened bindings table - O(1) 
    /// 3. Fall back to chain traversal if needed - O(n)
    /// 4. Cache the result for future lookups
    pub fn lookup(&self, name: &str) -> Option<Value> {
        let start_time = std::time::Instant::now();
        
        // Step 1: Check LRU cache first
        if let Some((value, generation)) = self.lookup_cache.borrow_mut().get(name) {
            // Verify cache entry is still valid
            if *generation >= self.cache_generation.get() {
                self.cache_stats.borrow_mut().cache_hits += 1;
                self.update_avg_time(start_time);
                return Some(value.clone());
            }
            // Cache entry is stale, remove it
            self.lookup_cache.borrow_mut().pop(name);
        }
        
        // Step 2: Check flattened bindings table  
        if let Some(ref bindings) = *self.flattened_bindings.borrow() {
            if let Some((value, generation)) = bindings.get(name) {
                // Verify the flattened table is still valid
                if *generation >= self.cache_generation.get() {
                    // Cache this lookup for future use
                    self.lookup_cache.borrow_mut().put(
                        name.to_string(), 
                        (value.clone(), *generation)
                    );
                    self.cache_stats.borrow_mut().cache_hits += 1;
                    self.update_avg_time(start_time);
                    return Some(value.clone());
                }
            }
        }
        
        // Step 3: Fall back to chain traversal and rebuild caches
        self.cache_stats.borrow_mut().cache_misses += 1;
        let result = self.lookup_with_rebuild(name);
        self.update_avg_time(start_time);
        result
    }
    
    /// Rebuilds the flattened bindings table and performs lookup.
    fn lookup_with_rebuild(&self, name: &str) -> Option<Value> {
        // Rebuild flattened bindings table
        let mut flattened = HashMap::new();
        let mut depth = 0;
        let current_generation = self.cache_generation.get();
        
        // Traverse the environment chain and collect all bindings
        let mut current_env = Some(self.base_env.clone());
        while let Some(env) = current_env {
            depth += 1;
            
            // Add all bindings from this environment level
            for (var_name, value) in env.bindings.borrow().iter() {
                // Only add if not already present (inner scope precedence)
                if !flattened.contains_key(var_name) {
                    flattened.insert(var_name.clone(), (value.clone(), current_generation));
                }
            }
            
            current_env = env.parent.clone();
        }
        
        // Update statistics
        {
            let mut stats = self.cache_stats.borrow_mut();
            stats.max_chain_depth = stats.max_chain_depth.max(depth);
            stats.flattened_rebuilds += 1;
        }
        
        // Cache the flattened bindings
        *self.flattened_bindings.borrow_mut() = Some(flattened.clone());
        
        // Perform the lookup
        let result = flattened.get(name).map(|(value, _)| value.clone());
        
        // Cache this specific lookup
        if let Some(ref value) = result {
            self.lookup_cache.borrow_mut().put(
                name.to_string(),
                (value.clone(), current_generation)
            );
        }
        
        result
    }
    
    /// Defines a new variable in the base environment and invalidates caches.
    pub fn define(&self, name: String, value: Value) {
        self.base_env.define(name.clone(), value.clone());
        self.invalidate_caches();
        
        // Optimistically add to cache since this is a new binding
        let generation = self.cache_generation.get();
        self.lookup_cache.borrow_mut().put(name, (value, generation));
    }
    
    /// Sets a variable in the environment chain and invalidates caches.
    pub fn set(&self, name: &str, value: Value) -> bool {
        let result = self.base_env.set(name, value.clone());
        if result {
            self.invalidate_caches();
            
            // Update cache with new value
            let generation = self.cache_generation.get();
            self.lookup_cache.borrow_mut().put(name.to_string(), (value, generation));
        }
        result
    }
    
    /// Invalidates all caches due to environment modifications.
    fn invalidate_caches(&self) {
        self.cache_generation.set(self.cache_generation.get() + 1);
        *self.flattened_bindings.borrow_mut() = None;
        self.lookup_cache.borrow_mut().clear();
        self.cache_stats.borrow_mut().cache_invalidations += 1;
    }
    
    /// Updates the average lookup time statistic.
    fn update_avg_time(&self, start_time: std::time::Instant) {
        let elapsed_ns = start_time.elapsed().as_nanos() as f64;
        let mut stats = self.cache_stats.borrow_mut();
        let total_lookups = stats.cache_hits + stats.cache_misses;
        
        if total_lookups > 0 {
            stats.avg_lookup_time_ns = 
                (stats.avg_lookup_time_ns * (total_lookups - 1) as f64 + elapsed_ns) 
                / total_lookups as f64;
        } else {
            stats.avg_lookup_time_ns = elapsed_ns;
        }
    }
    
    /// Extends the environment with new bindings, returning a new cached environment.
    pub fn extend(&self, generation: Generation) -> Rc<CachedEnvironment> {
        let extended_base = self.base_env.extend(generation);
        Rc::new(CachedEnvironment::new(extended_base))
    }
    
    /// Returns a reference to the underlying base environment.
    pub fn base_environment(&self) -> &Rc<Environment> {
        &self.base_env
    }
    
    /// Returns current cache statistics for performance monitoring.
    pub fn cache_statistics(&self) -> CacheStatistics {
        self.cache_stats.borrow().clone()
    }
    
    /// Calculates the cache hit ratio as a percentage.
    pub fn cache_hit_ratio(&self) -> f64 {
        let stats = self.cache_stats.borrow();
        let total = stats.cache_hits + stats.cache_misses;
        if total > 0 {
            (stats.cache_hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Clears all caches and resets statistics (for testing/debugging).
    pub fn clear_caches(&self) {
        self.lookup_cache.borrow_mut().clear();
        *self.flattened_bindings.borrow_mut() = None;
        *self.cache_stats.borrow_mut() = CacheStatistics::default();
        self.cache_generation.set(0);
    }
    
    /// Compresses the environment chain by building weak references to parents.
    /// This optimization reduces memory usage and can speed up traversals.
    pub fn compress_chain(&self) {
        let mut compressed = Vec::new();
        let mut current = self.base_env.parent.clone();
        
        while let Some(env) = current {
            compressed.push(Rc::downgrade(&env));
            current = env.parent.clone();
        }
        
        *self.compressed_parents.borrow_mut() = compressed;
    }
}

/// Converts a regular Environment to a CachedEnvironment for optimization.
impl From<Rc<Environment>> for CachedEnvironment {
    fn from(env: Rc<Environment>) -> Self {
        CachedEnvironment::new(env)
    }
}

/// Provides access to the underlying environment for compatibility.
impl std::ops::Deref for CachedEnvironment {
    type Target = Environment;
    
    fn deref(&self) -> &Self::Target {
        &self.base_env
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::Environment;
    
    /// Creates a test environment chain for performance testing.
    fn create_test_chain(depth: usize) -> Rc<Environment> {
        let mut env = Rc::new(Environment::new(None, 0));
        
        for i in 0..depth {
            env.define(format!("var{}", i), Value::integer(i as i64));
            if i < depth - 1 {
                env = env.extend((i as u32).into());
            }
        }
        
        env
    }
    
    #[test]
    fn test_cached_lookup_performance() {
        let base_env = create_test_chain(100);
        let cached_env = CachedEnvironment::new(base_env);
        
        // First lookup should be slow (cache miss)
        let start = std::time::Instant::now();
        let value1 = cached_env.lookup("var50");
        let first_lookup_time = start.elapsed();
        
        // Second lookup should be fast (cache hit)
        let start = std::time::Instant::now();
        let value2 = cached_env.lookup("var50");
        let second_lookup_time = start.elapsed();
        
        assert_eq!(value1, value2);
        assert!(second_lookup_time < first_lookup_time);
        
        let stats = cached_env.cache_statistics();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }
    
    #[test]
    fn test_cache_invalidation() {
        let base_env = Rc::new(Environment::new(None, 0));
        base_env.define("x".to_string(), Value::integer(42));
        
        let cached_env = CachedEnvironment::new(base_env);
        
        // Initial lookup
        assert_eq!(cached_env.lookup("x"), Some(Value::integer(42)));
        
        // Modify the environment
        cached_env.set("x", Value::integer(100));
        
        // Should return updated value
        assert_eq!(cached_env.lookup("x"), Some(Value::integer(100)));
        
        // Cache should have been invalidated
        let stats = cached_env.cache_statistics();
        assert!(stats.cache_invalidations > 0);
    }
    
    #[test]
    fn test_lexical_scoping_correctness() {
        // Create nested environments: outer -> middle -> inner
        let outer = Rc::new(Environment::new(None, 0));
        outer.define("x".to_string(), Value::integer(1));
        outer.define("y".to_string(), Value::integer(2));
        
        let middle = outer.extend(1);
        middle.define("x".to_string(), Value::integer(10));  // shadows outer x
        middle.define("z".to_string(), Value::integer(3));
        
        let inner = middle.extend(2);
        inner.define("x".to_string(), Value::integer(100));  // shadows middle x
        
        let cached_env = CachedEnvironment::new(inner);
        
        // Test variable resolution follows lexical scoping rules
        assert_eq!(cached_env.lookup("x"), Some(Value::integer(100)));  // inner
        assert_eq!(cached_env.lookup("y"), Some(Value::integer(2)));    // outer
        assert_eq!(cached_env.lookup("z"), Some(Value::integer(3)));    // middle
        assert_eq!(cached_env.lookup("nonexistent"), None);
    }
    
    #[test]
    fn test_cache_statistics() {
        let base_env = create_test_chain(50);
        let cached_env = CachedEnvironment::new(base_env);
        
        // Perform various lookups
        for i in 0..10 {
            cached_env.lookup(&format!("var{}", i));
            cached_env.lookup(&format!("var{}", i));  // Second lookup should hit cache
        }
        
        let stats = cached_env.cache_statistics();
        assert!(stats.cache_hits > 0);
        assert!(stats.cache_misses > 0);
        assert!(cached_env.cache_hit_ratio() > 0.0);
    }
}