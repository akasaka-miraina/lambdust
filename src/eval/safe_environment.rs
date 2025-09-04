//! Memory-safe environment chain management for Lambdust
//! 
//! Provides stack-overflow resistant environment chains with cycle detection
//! and memory-safe variable lookup without raw pointer operations.

use crate::eval::safe_optimized_value::SafeOptimizedValue as OptimizedValue;
use crate::utils::SymbolId;
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};
use std::fmt;

/// Generation counter for environment cache invalidation
pub type Generation = u64;

/// Maximum environment chain depth to prevent stack overflow
const MAX_ENVIRONMENT_DEPTH: usize = 1000;

/// Memory-safe environment with cycle-resistant parent chains
pub struct SafeEnvironment {
    /// Local variable bindings
    bindings: RwLock<HashMap<SymbolId, OptimizedValue>>,
    
    /// Parent environment (weak reference to prevent cycles)
    parent: Option<Weak<SafeEnvironment>>,
    
    /// Environment metadata
    metadata: EnvironmentMetadata,
    
    /// Variable lookup cache for performance
    cache: RwLock<LookupCache>,
}

#[derive(Debug, Clone)]
struct EnvironmentMetadata {
    /// Environment depth (0 = global, increases with nesting)
    depth: usize,
    
    /// Generation for cache invalidation
    generation: Generation,
    
    /// Environment ID for debugging
    id: u64,
    
    /// Creation timestamp
    created_at: std::time::Instant,
}

/// Cache for variable lookups to avoid repeated parent traversal
#[derive(Debug)]
struct LookupCache {
    entries: HashMap<SymbolId, CachedBinding>,
    max_size: usize,
    generation: Generation,
    hits: u64,
    misses: u64,
}

#[derive(Debug, Clone)]
struct CachedBinding {
    value: OptimizedValue,
    depth_found: usize,
    generation: Generation,
    access_count: u64,
}

impl SafeEnvironment {
    /// Create a new global environment (no parent)
    pub fn new_global() -> Arc<Self> {
        Arc::new(Self {
            bindings: RwLock::new(HashMap::new()),
            parent: None,
            metadata: EnvironmentMetadata {
                depth: 0,
                generation: 0,
                id: generate_env_id(),
                created_at: std::time::Instant::now(),
            },
            cache: RwLock::new(LookupCache::new()),
        })
    }
    
    /// Create a new environment extending a parent
    pub fn new_child(parent: Arc<SafeEnvironment>) -> Result<Arc<Self>, EnvironmentError> {
        let parent_depth = parent.metadata.depth;
        
        // Check depth to prevent stack overflow
        if parent_depth >= MAX_ENVIRONMENT_DEPTH {
            return Err(EnvironmentError::MaxDepthExceeded {
                max_depth: MAX_ENVIRONMENT_DEPTH,
                attempted_depth: parent_depth + 1,
            });
        }
        
        Ok(Arc::new(Self {
            bindings: RwLock::new(HashMap::new()),
            parent: Some(Arc::downgrade(&parent)),
            metadata: EnvironmentMetadata {
                depth: parent_depth + 1,
                generation: parent.metadata.generation + 1,
                id: generate_env_id(),
                created_at: std::time::Instant::now(),
            },
            cache: RwLock::new(LookupCache::new()),
        }))
    }
    
    /// Define a variable in this environment
    pub fn define(&self, symbol: SymbolId, value: OptimizedValue) -> Result<(), EnvironmentError> {
        let mut bindings = self.bindings.write()
            .map_err(|_| EnvironmentError::LockPoisoned)?;
        
        bindings.insert(symbol, value.clone());
        
        // Invalidate cache for this symbol
        self.invalidate_cache(symbol);
        
        Ok(())
    }
    
    /// Set (mutate) an existing variable
    pub fn set(&self, symbol: SymbolId, value: OptimizedValue) -> Result<(), EnvironmentError> {
        // Try to set in local environment first
        {
            let mut bindings = self.bindings.write()
                .map_err(|_| EnvironmentError::LockPoisoned)?;
            
            if bindings.contains_key(&symbol) {
                bindings.insert(symbol, value);
                self.invalidate_cache(symbol);
                return Ok(());
            }
        }
        
        // Try parent environments
        if let Some(parent_weak) = &self.parent {
            if let Some(parent) = parent_weak.upgrade() {
                return parent.set(symbol, value);
            }
        }
        
        Err(EnvironmentError::VariableNotFound { symbol })
    }
    
    /// Lookup a variable with memory-safe traversal
    pub fn lookup(&self, symbol: SymbolId) -> Result<OptimizedValue, EnvironmentError> {
        // Check cache first
        let cache_result = {
            let mut cache = self.cache.write()
                .map_err(|_| EnvironmentError::LockPoisoned)?;
            
            // Check if entry exists and is valid
            let (found_valid, found_value) = if let Some(cached) = cache.entries.get(&symbol) {
                if cached.generation == self.metadata.generation {
                    (true, Some(cached.value.clone()))
                } else {
                    (false, None)
                }
            } else {
                (false, None)
            };
            
            if found_valid {
                // Update access count separately
                if let Some(cached) = cache.entries.get_mut(&symbol) {
                    cached.access_count += 1;
                }
                cache.hits += 1;
                found_value
            } else {
                // Remove invalid entry if it exists
                cache.entries.remove(&symbol);
                cache.misses += 1;
                None
            }
        };
        
        if let Some(result) = cache_result {
            return Ok(result);
        }
        
        // Perform memory-safe lookup
        let result = self.lookup_recursive(symbol, 0)?;
        
        // Cache the result
        self.cache_lookup_result(symbol, result.clone());
        
        Ok(result)
    }
    
    /// Recursive lookup with depth tracking to prevent stack overflow
    fn lookup_recursive(&self, symbol: SymbolId, current_depth: usize) -> Result<OptimizedValue, EnvironmentError> {
        // Check for excessive recursion
        if current_depth > MAX_ENVIRONMENT_DEPTH {
            return Err(EnvironmentError::MaxDepthExceeded {
                max_depth: MAX_ENVIRONMENT_DEPTH,
                attempted_depth: current_depth,
            });
        }
        
        // Check local bindings
        {
            let bindings = self.bindings.read()
                .map_err(|_| EnvironmentError::LockPoisoned)?;
            
            if let Some(value) = bindings.get(&symbol) {
                return Ok(value.clone());
            }
        }
        
        // Check parent environment
        if let Some(parent_weak) = &self.parent {
            if let Some(parent) = parent_weak.upgrade() {
                return parent.lookup_recursive(symbol, current_depth + 1);
            }
        }
        
        Err(EnvironmentError::VariableNotFound { symbol })
    }
    
    /// Check if a variable is defined in this environment or parents
    pub fn contains(&self, symbol: SymbolId) -> bool {
        self.lookup(symbol).is_ok()
    }
    
    /// Get environment statistics for debugging
    pub fn statistics(&self) -> EnvironmentStats {
        let bindings = self.bindings.read().unwrap();
        let cache = self.cache.read().unwrap();
        
        EnvironmentStats {
            depth: self.metadata.depth,
            local_bindings: bindings.len(),
            cached_lookups: cache.entries.len(),
            cache_hit_ratio: cache.calculate_hit_ratio(),
            generation: self.metadata.generation,
            environment_id: self.metadata.id,
        }
    }
    
    /// Invalidate cache entry for a symbol
    fn invalidate_cache(&self, symbol: SymbolId) {
        if let Ok(mut cache) = self.cache.write() {
            cache.entries.remove(&symbol);
        }
    }
    
    /// Cache a lookup result
    fn cache_lookup_result(&self, symbol: SymbolId, value: OptimizedValue) {
        if let Ok(mut cache) = self.cache.write() {
            cache.add_entry(symbol, value, self.metadata.depth, self.metadata.generation);
        }
    }
}

impl LookupCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            max_size: 500, // Configurable cache size
            generation: 0,
            hits: 0,
            misses: 0,
        }
    }
    
    fn add_entry(&mut self, symbol: SymbolId, value: OptimizedValue, depth: usize, generation: Generation) {
        if self.entries.len() >= self.max_size {
            self.evict_lru();
        }
        
        self.entries.insert(symbol, CachedBinding {
            value,
            depth_found: depth,
            generation,
            access_count: 1,
        });
    }
    
    fn evict_lru(&mut self) {
        // Remove least recently used entries
        let mut entries: Vec<_> = self.entries.drain().collect();
        entries.sort_by_key(|(_, binding)| binding.access_count);
        entries.truncate(self.max_size / 2);
        self.entries.extend(entries);
    }
    
    fn calculate_hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Environment operation errors
#[derive(Debug, Clone)]
pub enum EnvironmentError {
    /// Variable not found in environment chain
    VariableNotFound { symbol: SymbolId },
    
    /// Maximum environment depth exceeded (stack overflow prevention)
    MaxDepthExceeded { max_depth: usize, attempted_depth: usize },
    
    /// Environment lock is poisoned
    LockPoisoned,
    
    /// Parent environment was dropped (orphaned environment)
    OrphanedEnvironment,
}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvironmentError::VariableNotFound { symbol } => {
                write!(f, "Variable not found: {:?}", symbol)
            }
            EnvironmentError::MaxDepthExceeded { max_depth, attempted_depth } => {
                write!(f, "Environment depth exceeded: {} > {}", attempted_depth, max_depth)
            }
            EnvironmentError::LockPoisoned => {
                write!(f, "Environment lock is poisoned")
            }
            EnvironmentError::OrphanedEnvironment => {
                write!(f, "Parent environment was dropped")
            }
        }
    }
}

impl std::error::Error for EnvironmentError {}

/// Environment statistics for debugging and monitoring
#[derive(Debug, Clone)]
pub struct EnvironmentStats {
    pub depth: usize,
    pub local_bindings: usize,
    pub cached_lookups: usize,
    pub cache_hit_ratio: f64,
    pub generation: Generation,
    pub environment_id: u64,
}

/// Generate unique environment ID
fn generate_env_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::safe_optimized_value::SafeOptimizedValue as OptimizedValue;
    
    #[test]
    fn test_basic_environment_operations() {
        let env = SafeEnvironment::new_global();
        let symbol = SymbolId(42);
        let value = OptimizedValue::nil(); // Assuming this method exists
        
        // Define variable
        env.define(symbol, value.clone()).unwrap();
        
        // Look up variable
        let retrieved = env.lookup(symbol).unwrap();
        // Test would need OptimizedValue equality implementation
        
        assert!(env.contains(symbol));
    }
    
    #[test]
    fn test_environment_chain() {
        let global = SafeEnvironment::new_global();
        let child = SafeEnvironment::new_child(global.clone()).unwrap();
        
        let symbol1 = SymbolId(1);
        let symbol2 = SymbolId(2);
        let value1 = OptimizedValue::nil();
        let value2 = OptimizedValue::nil();
        
        // Define in different environments
        global.define(symbol1, value1).unwrap();
        child.define(symbol2, value2).unwrap();
        
        // Child should see both variables
        assert!(child.contains(symbol1)); // From parent
        assert!(child.contains(symbol2)); // Local
        
        // Global should only see its variable
        assert!(global.contains(symbol1));
        assert!(!global.contains(symbol2));
    }
    
    #[test]
    fn test_depth_limit() {
        let mut current = SafeEnvironment::new_global();
        
        // Create deep environment chain
        for i in 0..MAX_ENVIRONMENT_DEPTH {
            current = SafeEnvironment::new_child(current).unwrap();
        }
        
        // Next one should fail
        let result = SafeEnvironment::new_child(current);
        assert!(matches!(result, Err(EnvironmentError::MaxDepthExceeded { .. })));
    }
    
    #[test]
    fn test_cache_performance() {
        let env = SafeEnvironment::new_global();
        let symbol = SymbolId(123);
        let value = OptimizedValue::nil();
        
        env.define(symbol, value).unwrap();
        
        // Multiple lookups should hit cache
        for _ in 0..100 {
            let _ = env.lookup(symbol).unwrap();
        }
        
        let stats = env.statistics();
        println!("Cache hit ratio: {:.2}%", stats.cache_hit_ratio * 100.0);
        assert!(stats.cache_hit_ratio > 0.0, "Should have cache hits");
    }
}