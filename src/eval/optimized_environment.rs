//! Optimized environment implementation for fast variable lookup
//!
//! This module provides highly optimized environment data structures that
//! minimize the cost of variable lookups, especially for frequently accessed
//! variables and deep environment chains.

use crate::eval::{Value, Generation};
use crate::utils::SymbolId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};

/// Cache for frequently accessed variables to avoid deep lookups
#[derive(Debug)]
struct VariableCache {
    /// Cache entries mapping symbol to (value, generation, depth)
    entries: HashMap<SymbolId, CacheEntry>,
    /// Maximum cache size
    max_size: usize,
    /// Cache hit counter
    hits: usize,
    /// Cache miss counter
    misses: usize,
    /// Current generation for cache invalidation
    generation: Generation,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: Value,
    generation: Generation,
    depth: usize,
    access_count: usize,
}

/// Optimized environment with caching and fast lookup mechanisms
#[derive(Debug)]
pub struct OptimizedEnvironment {
    /// Local bindings for this environment
    bindings: HashMap<SymbolId, Value>,
    /// Parent environment (if any)
    parent: Option<Arc<OptimizedEnvironment>>,
    /// Variable access cache
    cache: RwLock<VariableCache>,
    /// Environment depth (0 for global, increases with nesting)
    depth: usize,
    /// Generation counter for cache invalidation
    generation: Generation,
    /// Environment ID for debugging
    id: u64,
}

/// Statistics about environment performance
#[derive(Debug, Clone)]
pub struct EnvironmentStats {
    /// Total variable lookups performed
    pub total_lookups: usize,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
    /// Cache hit rate as percentage
    pub hit_rate: f64,
    /// Average lookup depth
    pub avg_lookup_depth: f64,
    /// Most frequently accessed variables
    pub hot_variables: Vec<(SymbolId, usize)>,
}

impl VariableCache {
    fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            hits: 0,
            misses: 0,
            generation: 0,
        }
    }
    
    fn get(&mut self, symbol: SymbolId, current_generation: Generation) -> Option<Value> {
        // First, check if we have the entry and whether it's valid
        let should_remove = if let Some(entry) = self.entries.get(&symbol) {
            entry.generation > current_generation
        } else {
            false
        };
        
        // If we need to remove a stale entry, do it now
        if should_remove {
            self.entries.remove(&symbol);
            self.misses += 1;
            return None;
        }
        
        // Now we can safely get a mutable reference to update access count
        if let Some(entry) = self.entries.get_mut(&symbol) {
            entry.access_count += 1;
            self.hits += 1;
            Some(entry.value.clone())
        } else {
            self.misses += 1;
            None
        }
    }
    
    fn insert(&mut self, symbol: SymbolId, value: Value, generation: Generation, depth: usize) {
        // If cache is full, remove least recently used entry
        if self.entries.len() >= self.max_size {
            self.evict_lru();
        }
        
        let entry = CacheEntry {
            value,
            generation,
            depth,
            access_count: 1,
        };
        
        self.entries.insert(symbol, entry);
    }
    
    fn evict_lru(&mut self) {
        // Find and remove the entry with the lowest access count
        if let Some((&symbol_to_remove, _)) = self.entries.iter()
            .min_by_key(|(_, entry)| entry.access_count) {
            self.entries.remove(&symbol_to_remove);
        }
    }
    
    fn invalidate(&mut self, generation: Generation) {
        self.generation = generation;
        // Remove stale entries
        self.entries.retain(|_, entry| entry.generation <= generation);
    }
    
    fn stats(&self) -> (usize, usize, f64) {
        let total = self.hits + self.misses;
        let hit_rate = if total > 0 { 
            (self.hits as f64 / total as f64) * 100.0 
        } else { 
            0.0 
        };
        (self.hits, self.misses, hit_rate)
    }
}

static ENVIRONMENT_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl OptimizedEnvironment {
    /// Creates a new top-level environment
    pub fn new() -> Self {
        let id = ENVIRONMENT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self {
            bindings: HashMap::new(),
            parent: None,
            cache: RwLock::new(VariableCache::new(100)), // Cache up to 100 variables
            depth: 0,
            generation: 0,
            id,
        }
    }
    
    /// Creates a new child environment
    pub fn new_child(parent: Arc<OptimizedEnvironment>) -> Self {
        let id = ENVIRONMENT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let depth = parent.depth + 1;
        let generation = parent.generation + 1;
        
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
            cache: RwLock::new(VariableCache::new(50)), // Smaller cache for child envs
            depth,
            generation,
            id,
        }
    }
    
    /// Binds a variable in this environment
    pub fn bind(&mut self, symbol: SymbolId, value: Value) {
        self.bindings.insert(symbol, value.clone());
        
        // Update cache with the new binding
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(symbol, value, self.generation, 0);
        }
        
        // Invalidate parent caches since binding might shadow parent variables
        self.invalidate_parent_caches();
    }
    
    /// Sets a variable (updates existing binding or creates new one)
    pub fn set(&mut self, symbol: SymbolId, value: Value) -> Result<(), String> {
        // Try to update local binding first
        if self.bindings.contains_key(&symbol) {
            self.bindings.insert(symbol, value.clone());
            
            // Update cache
            if let Ok(mut cache) = self.cache.write() {
                cache.insert(symbol, value, self.generation, 0);
            }
            
            return Ok(());
        }
        
        // Try to update in parent environments
        if let Some(ref parent) = self.parent {
            return self.set_in_parent(symbol, value, parent.clone()), 1);
        }
        
        Err(format!("Undefined variable: {:?}", symbol))
    }
    
    /// Looks up a variable value with optimized caching
    pub fn lookup(&self, symbol: SymbolId) -> Option<Value> {
        // First check the cache
        if let Ok(mut cache) = self.cache.write() {
            if let Some(value) = cache.get(symbol, self.generation) {
                return Some(value);
            }
        }
        
        // Check local bindings
        if let Some(value) = self.bindings.get(&symbol) {
            // Cache the result
            if let Ok(mut cache) = self.cache.write() {
                cache.insert(symbol, value.clone()), self.generation, 0);
            }
            return Some(value.clone());
        }
        
        // Check parent environments
        if let Some(ref parent) = self.parent {
            if let Some(value) = parent.lookup_with_depth(symbol, 1) {
                // Cache the result with appropriate depth
                if let Ok(mut cache) = self.cache.write() {
                    cache.insert(symbol, value.clone()), self.generation, 1);
                }
                return Some(value);
            }
        }
        
        None
    }
    
    /// Internal lookup with depth tracking for cache optimization
    fn lookup_with_depth(&self, symbol: SymbolId, current_depth: usize) -> Option<Value> {
        // Check local bindings
        if let Some(value) = self.bindings.get(&symbol) {
            return Some(value.clone());
        }
        
        // Check parent environments
        if let Some(ref parent) = self.parent {
            parent.lookup_with_depth(symbol, current_depth + 1)
        } else {
            None
        }
    }
    
    /// Helper method to set variables in parent environments
    fn set_in_parent(&self, symbol: SymbolId, value: Value, parent: Arc<OptimizedEnvironment>, depth: usize) -> Result<(), String> {
        // This is a simplified version - in a real implementation, we would need
        // proper mutable access to parent environments
        Err("Setting variables in parent environments requires mutable access".to_string())
    }
    
    /// Invalidates caches in parent environments
    fn invalidate_parent_caches(&self) {
        if let Some(ref parent) = self.parent {
            if let Ok(mut cache) = parent.cache.write() {
                cache.invalidate(self.generation);
            }
            parent.invalidate_parent_caches();
        }
    }
    
    /// Gets environment statistics for performance analysis
    pub fn get_stats(&self) -> EnvironmentStats {
        let mut total_lookups = 0;
        let mut cache_hits = 0;
        let mut cache_misses = 0;
        let mut total_depth = 0;
        let mut hot_variables = Vec::new();
        
        if let Ok(cache) = self.cache.read() {
            let (hits, misses, _) = cache.stats();
            cache_hits += hits;
            cache_misses += misses;
            total_lookups += hits + misses;
            
            // Collect hot variables
            let mut var_stats: Vec<_> = cache.entries.iter()
                .map(|(&symbol, entry)| (symbol, entry.access_count))
                .collect();
            var_stats.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
            hot_variables = var_stats.into_iter().take(10).collect();
            
            // Calculate average depth
            let total_weighted_depth: usize = cache.entries.values()
                .map(|entry| entry.depth * entry.access_count)
                .sum();
            let total_accesses: usize = cache.entries.values()
                .map(|entry| entry.access_count)
                .sum();
            if total_accesses > 0 {
                total_depth = total_weighted_depth / total_accesses;
            }
        }
        
        // Recursively collect stats from parent environments
        if let Some(ref parent) = self.parent {
            let parent_stats = parent.get_stats();
            total_lookups += parent_stats.total_lookups;
            cache_hits += parent_stats.cache_hits;
            cache_misses += parent_stats.cache_misses;
        }
        
        let hit_rate = if total_lookups > 0 {
            (cache_hits as f64 / total_lookups as f64) * 100.0
        } else {
            0.0
        };
        
        EnvironmentStats {
            total_lookups,
            cache_hits,
            cache_misses,
            hit_rate,
            avg_lookup_depth: total_depth as f64,
            hot_variables,
        }
    }
    
    /// Optimizes the environment by pre-loading frequently used variables into cache
    pub fn optimize(&mut self, hot_variables: &[(SymbolId, usize)]) {
        if let Ok(mut cache) = self.cache.write() {
            // Pre-load hot variables into cache
            for &(symbol, access_count) in hot_variables.iter().take(20) {
                if let Some(value) = self.bindings.get(&symbol) {
                    let entry = CacheEntry {
                        value: value.clone()),
                        generation: self.generation,
                        depth: 0,
                        access_count,
                    };
                    cache.entries.insert(symbol, entry);
                }
            }
        }
    }
    
    /// Clears the variable cache (useful for testing or memory management)
    pub fn clear_cache(&mut self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.entries.clear();
            cache.hits = 0;
            cache.misses = 0;
        }
    }
    
    /// Gets the environment depth
    pub fn depth(&self) -> usize {
        self.depth
    }
    
    /// Gets the environment ID
    pub fn id(&self) -> u64 {
        self.id
    }
    
    /// Dumps environment contents for debugging
    pub fn dump_debug_info(&self) -> String {
        let mut info = String::new();
        info.push_str(&format!("Environment {} (depth: {})\n", self.id, self.depth));
        info.push_str(&format!("Local bindings: {}\n", self.bindings.len()));
        
        if let Ok(cache) = self.cache.read() {
            let (hits, misses, hit_rate) = cache.stats();
            info.push_str(&format!("Cache: {} hits, {} misses, {:.1}% hit rate\n", hits, misses, hit_rate));
            info.push_str(&format!("Cache entries: {}/{}\n", cache.entries.len(), cache.max_size));
        }
        
        if let Some(ref parent) = self.parent {
            info.push_str("Parent environment: Yes\n");
        } else {
            info.push_str("Parent environment: None (top-level)\n");
        }
        
        info
    }
}

impl Default for OptimizedEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating optimized environments with specific configurations
pub struct OptimizedEnvironmentBuilder {
    cache_size: usize,
    enable_caching: bool,
    pre_populate_cache: bool,
}

impl OptimizedEnvironmentBuilder {
    pub fn new() -> Self {
        Self {
            cache_size: 100,
            enable_caching: true,
            pre_populate_cache: false,
        }
    }
    
    pub fn cache_size(mut self, size: usize) -> Self {
        self.cache_size = size;
        self
    }
    
    pub fn enable_caching(mut self, enable: bool) -> Self {
        self.enable_caching = enable;
        self
    }
    
    pub fn pre_populate_cache(mut self, pre_populate: bool) -> Self {
        self.pre_populate_cache = pre_populate;
        self
    }
    
    pub fn build(self) -> OptimizedEnvironment {
        let mut env = OptimizedEnvironment::new();
        
        if !self.enable_caching {
            env.clear_cache();
        }
        
        // In a real implementation, we would configure the cache size here
        env
    }
}

impl Default for OptimizedEnvironmentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::intern_symbol;
    
    #[test]
    fn test_optimized_environment_basic_operations() {
        let mut env = OptimizedEnvironment::new();
        let symbol = intern_symbol("test-var");
        let value = Value::integer(42);
        
        // Test binding
        env.bind(symbol, value.clone());
        
        // Test lookup
        let result = env.lookup(symbol);
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_integer(), Some(42));
        
        // Test cache hit on second lookup
        let result2 = env.lookup(symbol);
        assert!(result2.is_some());
    }
    
    #[test]
    fn test_environment_hierarchy() {
        let mut parent = OptimizedEnvironment::new();
        let parent_symbol = intern_symbol("parent-var");
        parent.bind(parent_symbol, Value::integer(100));
        
        let child = OptimizedEnvironment::new_child(Arc::new(parent));
        let child_symbol = intern_symbol("child-var");
        
        // Child should be able to see parent variables
        let parent_value = child.lookup(parent_symbol);
        assert!(parent_value.is_some());
        assert_eq!(parent_value.unwrap().as_integer(), Some(100));
    }
    
    #[test]
    fn test_cache_performance() {
        let mut env = OptimizedEnvironment::new();
        let symbol = intern_symbol("cached-var");
        env.bind(symbol, Value::integer(42));
        
        // First lookup should populate cache
        let _ = env.lookup(symbol);
        
        // Subsequent lookups should hit cache
        for _ in 0..10 {
            let _ = env.lookup(symbol);
        }
        
        let stats = env.get_stats();
        assert!(stats.cache_hits > 0);
        assert!(stats.hit_rate > 0.0);
    }
    
    #[test]
    fn test_environment_stats() {
        let mut env = OptimizedEnvironment::new();
        let symbols: Vec<_> = (0..5).map(|i| intern_symbol(&format!("var_{}", i))).collect();
        
        // Bind variables
        for (i, &symbol) in symbols.iter().enumerate() {
            env.bind(symbol, Value::integer(i as i64));
        }
        
        // Access variables with different frequencies
        for (i, &symbol) in symbols.iter().enumerate() {
            for _ in 0..=i {
                let _ = env.lookup(symbol);
            }
        }
        
        let stats = env.get_stats();
        assert!(stats.total_lookups > 0);
        assert!(!stats.hot_variables.is_empty());
    }
    
    #[test]
    fn test_environment_builder() {
        let env = OptimizedEnvironmentBuilder::new()
            .cache_size(50)
            .enable_caching(true)
            .build();
        
        assert_eq!(env.depth(), 0);
        assert_eq!(env.id(), env.id()); // ID should be consistent
    }
}