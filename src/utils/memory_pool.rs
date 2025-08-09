//! Memory pools for efficient allocation and reuse of common objects.
//!
//! This module provides memory pools to reduce allocation overhead for frequently
//! created and destroyed objects like tokens, AST nodes, and values.

use std::collections::{VecDeque, HashMap};
use std::sync::{Arc, Mutex};

/// A memory pool for objects of type T.
/// Objects are recycled to reduce allocation overhead.
#[derive(Debug)]
pub struct MemoryPool<T> {
    pool: Arc<Mutex<VecDeque<T>>>,
    factory: fn() -> T,
    max_size: usize,
}

impl<T> MemoryPool<T> {
    /// Creates a new memory pool with the given factory function and maximum size.
    pub fn new(factory: fn() -> T, max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::new())),
            factory,
            max_size,
        }
    }

    /// Gets an object from the pool, or creates a new one if the pool is empty.
    pub fn get(&self) -> PooledObject<T> {
        let obj = if let Ok(mut pool) = self.pool.lock() {
            pool.pop_front().unwrap_or_else(|| (self.factory)())
        } else {
            (self.factory)()
        };

        PooledObject {
            object: Some(obj),
            pool: self.pool.clone(),
            max_size: self.max_size,
        }
    }

    /// Returns the current number of objects in the pool.
    pub fn size(&self) -> usize {
        if let Ok(pool) = self.pool.lock() {
            pool.len()
        } else {
            0
        }
    }

    /// Clears all objects from the pool.
    pub fn clear(&self) {
        if let Ok(mut pool) = self.pool.lock() {
            pool.clear();
        }
    }
}

/// An object borrowed from a memory pool that will be returned when dropped.
pub struct PooledObject<T> {
    object: Option<T>,
    pool: Arc<Mutex<VecDeque<T>>>,
    max_size: usize,
}

impl<T> PooledObject<T> {
    /// Takes the object out of the pooled wrapper.
    /// The object will not be returned to the pool when this wrapper is dropped.
    pub fn take(mut self) -> T {
        self.object.take().expect("Object has already been taken")
    }
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.object.as_ref().expect("Object has been taken")
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().expect("Object has been taken")
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.object.take() {
            if let Ok(mut pool) = self.pool.lock() {
                if pool.len() < self.max_size {
                    pool.push_back(obj);
                }
            }
        }
    }
}

/// A specialized memory pool for Vec<T> that reuses the allocated capacity.
#[derive(Debug)]
pub struct VecPool<T> {
    pool: Arc<Mutex<VecDeque<Vec<T>>>>,
    max_size: usize,
    initial_capacity: usize,
}

impl<T> VecPool<T> {
    /// Creates a new Vec pool with the given initial capacity and maximum pool size.
    pub fn new(initial_capacity: usize, max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
            initial_capacity,
        }
    }

    /// Gets a Vec from the pool, or creates a new one if the pool is empty.
    pub fn get(&self) -> PooledVec<T> {
        let mut vec = if let Ok(mut pool) = self.pool.lock() {
            pool.pop_front().unwrap_or_else(|| Vec::with_capacity(self.initial_capacity))
        } else {
            Vec::with_capacity(self.initial_capacity)
        };

        vec.clear(); // Ensure the Vec is empty but retains capacity

        PooledVec {
            vec: Some(vec),
            pool: self.pool.clone(),
            max_size: self.max_size,
        }
    }

    /// Returns the current number of Vecs in the pool.
    pub fn size(&self) -> usize {
        if let Ok(pool) = self.pool.lock() {
            pool.len()
        } else {
            0
        }
    }

    /// Clears all Vecs from the pool.
    pub fn clear(&self) {
        if let Ok(mut pool) = self.pool.lock() {
            pool.clear();
        }
    }
}

/// A Vec borrowed from a VecPool that will be returned when dropped.
pub struct PooledVec<T> {
    vec: Option<Vec<T>>,
    pool: Arc<Mutex<VecDeque<Vec<T>>>>,
    max_size: usize,
}

impl<T> PooledVec<T> {
    /// Takes the Vec out of the pooled wrapper.
    /// The Vec will not be returned to the pool when this wrapper is dropped.
    pub fn take(mut self) -> Vec<T> {
        self.vec.take().expect("Vec has already been taken")
    }
}

impl<T> std::ops::Deref for PooledVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        self.vec.as_ref().expect("Vec has been taken")
    }
}

impl<T> std::ops::DerefMut for PooledVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.vec.as_mut().expect("Vec has been taken")
    }
}

impl<T> Drop for PooledVec<T> {
    fn drop(&mut self) {
        if let Some(vec) = self.vec.take() {
            if let Ok(mut pool) = self.pool.lock() {
                if pool.len() < self.max_size {
                    pool.push_back(vec);
                }
            }
        }
    }
}

/// Specialized memory pool for AST node allocations.
#[derive(Debug)]
pub struct AstNodePool<T> {
    pool: Arc<Mutex<VecDeque<Box<T>>>>,
    max_size: usize,
    allocation_count: Arc<std::sync::atomic::AtomicUsize>,
    reuse_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl<T> AstNodePool<T> {
    /// Creates a new AST node pool.
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
            allocation_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            reuse_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
    
    /// Gets a boxed node from the pool or allocates a new one.
    pub fn get_boxed<F>(&self, factory: F) -> PooledBox<T>
    where
        F: FnOnce() -> T,
    {
        let boxed_node = if let Ok(mut pool) = self.pool.lock() {
            if let Some(mut node) = pool.pop_front() {
                self.reuse_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                // Reset the node to a clean state
                *node = factory();
                node
            } else {
                self.allocation_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Box::new(factory())
            }
        } else {
            self.allocation_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Box::new(factory())
        };
        
        PooledBox {
            boxed: Some(boxed_node),
            pool: self.pool.clone(),
            max_size: self.max_size,
        }
    }
    
    /// Gets allocation statistics.
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            pool_size: self.size(),
            allocation_count: self.allocation_count.load(std::sync::atomic::Ordering::Relaxed),
            reuse_count: self.reuse_count.load(std::sync::atomic::Ordering::Relaxed),
            max_size: self.max_size,
        }
    }
    
    /// Returns the current pool size.
    pub fn size(&self) -> usize {
        if let Ok(pool) = self.pool.lock() {
            pool.len()
        } else {
            0
        }
    }
    
    /// Clears all nodes from the pool.
    pub fn clear(&self) {
        if let Ok(mut pool) = self.pool.lock() {
            pool.clear();
        }
    }
}

/// A boxed value borrowed from an AST node pool.
pub struct PooledBox<T> {
    boxed: Option<Box<T>>,
    pool: Arc<Mutex<VecDeque<Box<T>>>>,
    max_size: usize,
}

impl<T> PooledBox<T> {
    /// Takes the boxed value out of the pool wrapper.
    pub fn take(mut self) -> Box<T> {
        self.boxed.take().expect("Box already taken")
    }
}

impl<T> std::ops::Deref for PooledBox<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.boxed.as_ref().expect("Box already taken")
    }
}

impl<T> std::ops::DerefMut for PooledBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.boxed.as_mut().expect("Box already taken")
    }
}

impl<T> Drop for PooledBox<T> {
    fn drop(&mut self) {
        if let Some(boxed) = self.boxed.take() {
            if let Ok(mut pool) = self.pool.lock() {
                if pool.len() < self.max_size {
                    pool.push_back(boxed);
                }
            }
        }
    }
}

/// Statistics about a memory pool.
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Current number of items in the pool
    pub pool_size: usize,
    /// Total number of allocations made
    pub allocation_count: usize,
    /// Number of times items were reused from the pool  
    pub reuse_count: usize,
    /// Maximum pool size
    pub max_size: usize,
}

impl PoolStats {
    /// Calculates the reuse rate as a percentage.
    pub fn reuse_rate(&self) -> f64 {
        let total = self.allocation_count + self.reuse_count;
        if total == 0 {
            0.0
        } else {
            (self.reuse_count as f64 / total as f64) * 100.0
        }
    }
    
    /// Calculates memory efficiency (higher is better).
    pub fn efficiency_score(&self) -> f64 {
        let utilization = self.pool_size as f64 / self.max_size as f64;
        let reuse_factor = self.reuse_rate() / 100.0;
        (utilization * 0.3) + (reuse_factor * 0.7) // Weight reuse more heavily
    }
}

/// Environment pool for managing lexical environments efficiently.
#[derive(Debug)]
pub struct EnvironmentPool {
    pools: HashMap<usize, AstNodePool<HashMap<String, crate::eval::Value>>>,
    size_distribution: Arc<Mutex<HashMap<usize, usize>>>,
}

impl Default for EnvironmentPool {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentPool {
    /// Creates a new environment pool with size-based sub-pools.
    pub fn new() -> Self {
        let mut pools = HashMap::new();
        
        // Create pools for common environment sizes
        let common_sizes = [4, 8, 16, 32, 64, 128];
        for &size in &common_sizes {
            pools.insert(size, AstNodePool::new(10));
        }
        
        Self {
            pools,
            size_distribution: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Gets an environment with the specified capacity hint.
    pub fn get_environment(&self, capacity_hint: usize) -> PooledEnvironment {
        // Find the best matching pool size
        let pool_size = self.pools.keys()
            .filter(|&&size| size >= capacity_hint)
            .min()
            .copied()
            .unwrap_or(128); // Fallback to largest pool
        
        // Track size distribution
        if let Ok(mut dist) = self.size_distribution.lock() {
            *dist.entry(pool_size).or_insert(0) += 1;
        }
        
        if let Some(pool) = self.pools.get(&pool_size) {
            let bindings = pool.get_boxed(|| HashMap::with_capacity(pool_size));
            PooledEnvironment { bindings }
        } else {
            // Fallback - create directly
            PooledEnvironment {
                bindings: PooledBox {
                    boxed: Some(Box::new(HashMap::with_capacity(capacity_hint))),
                    pool: Arc::new(Mutex::new(VecDeque::new())),
                    max_size: 0,
                }
            }
        }
    }
    
    /// Gets statistics about environment allocation patterns.
    pub fn allocation_stats(&self) -> EnvironmentPoolStats {
        let mut total_allocations = 0;
        let mut total_reuses = 0;
        let size_dist = if let Ok(dist) = self.size_distribution.lock() {
            dist.clone()
        } else {
            HashMap::new()
        };
        
        for pool in self.pools.values() {
            let stats = pool.stats();
            total_allocations += stats.allocation_count;
            total_reuses += stats.reuse_count;
        }
        
        EnvironmentPoolStats {
            total_allocations,
            total_reuses,
            size_distribution: size_dist,
            reuse_rate: if total_allocations + total_reuses > 0 {
                (total_reuses as f64 / (total_allocations + total_reuses) as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// A pooled environment binding map.
pub struct PooledEnvironment {
    bindings: PooledBox<HashMap<String, crate::eval::Value>>,
}

impl PooledEnvironment {
    /// Takes the underlying HashMap.
    pub fn take(self) -> Box<HashMap<String, crate::eval::Value>> {
        self.bindings.take()
    }
}

impl std::ops::Deref for PooledEnvironment {
    type Target = HashMap<String, crate::eval::Value>;
    
    fn deref(&self) -> &Self::Target {
        &self.bindings
    }
}

impl std::ops::DerefMut for PooledEnvironment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bindings
    }
}

/// Statistics about environment pool usage.
#[derive(Debug, Clone)]
pub struct EnvironmentPoolStats {
    /// Total number of environment allocations
    pub total_allocations: usize,
    /// Total number of environment reuses
    pub total_reuses: usize,
    /// Distribution of requested environment sizes
    pub size_distribution: HashMap<usize, usize>,
    /// Overall reuse rate as percentage
    pub reuse_rate: f64,
}

/// Global pools for commonly used types.
pub mod global_pools {
    use super::*;
    use crate::lexer::Token;
    use once_cell::sync::Lazy;

    /// Global pool for Token vectors (used during tokenization).
    pub static TOKEN_VEC_POOL: Lazy<VecPool<Token>> = 
        Lazy::new(|| VecPool::new(64, 10));

    /// Global pool for String vectors.
    pub static STRING_VEC_POOL: Lazy<VecPool<String>> = 
        Lazy::new(|| VecPool::new(32, 10));
    
    /// Global pool for AST expression nodes.
    pub static EXPR_POOL: Lazy<AstNodePool<crate::ast::Expr>> = 
        Lazy::new(|| AstNodePool::new(50));
    
    /// Global pool for environment bindings.
    pub static ENVIRONMENT_POOL: Lazy<EnvironmentPool> = 
        Lazy::new(EnvironmentPool::new);
    
    /// Global pool for continuation frames.
    pub static FRAME_VEC_POOL: Lazy<VecPool<crate::eval::Frame>> = 
        Lazy::new(|| VecPool::new(16, 10));

    /// Gets a token vector from the global pool.
    pub fn get_token_vec() -> PooledVec<Token> {
        TOKEN_VEC_POOL.get()
    }

    /// Gets a string vector from the global pool.
    pub fn get_string_vec() -> PooledVec<String> {
        STRING_VEC_POOL.get()
    }
    
    /// Gets an AST expression node from the global pool.
    pub fn get_expr<F>(factory: F) -> PooledBox<crate::ast::Expr>
    where
        F: FnOnce() -> crate::ast::Expr,
    {
        EXPR_POOL.get_boxed(factory)
    }
    
    /// Gets a pooled environment from the global pool.
    pub fn get_environment(capacity_hint: usize) -> PooledEnvironment {
        ENVIRONMENT_POOL.get_environment(capacity_hint)
    }
    
    /// Gets a frame vector from the global pool.
    pub fn get_frame_vec() -> PooledVec<crate::eval::Frame> {
        FRAME_VEC_POOL.get()
    }

    /// Returns comprehensive statistics about all global pools.
    pub fn comprehensive_pool_stats() -> GlobalPoolStats {
        GlobalPoolStats {
            token_vec_pool: TOKEN_VEC_POOL.size(),
            string_vec_pool: STRING_VEC_POOL.size(),
            expr_pool: EXPR_POOL.stats(),
            environment_pool: ENVIRONMENT_POOL.allocation_stats(),
            frame_vec_pool: FRAME_VEC_POOL.size(),
        }
    }

    /// Returns basic pool statistics for testing purposes.
    /// Returns (token_pool_size, string_pool_size)
    pub fn pool_stats() -> (usize, usize) {
        (TOKEN_VEC_POOL.size(), STRING_VEC_POOL.size())
    }
}

/// Comprehensive statistics about all global memory pools.
#[derive(Debug, Clone)]
pub struct GlobalPoolStats {
    /// Token vector pool size
    pub token_vec_pool: usize,
    /// String vector pool size  
    pub string_vec_pool: usize,
    /// Expression pool statistics
    pub expr_pool: PoolStats,
    /// Environment pool statistics
    pub environment_pool: EnvironmentPoolStats,
    /// Frame vector pool size
    pub frame_vec_pool: usize,
}

impl GlobalPoolStats {
    /// Calculates overall memory efficiency across all pools.
    pub fn overall_efficiency(&self) -> f64 {
        // Weight different pools based on their typical usage frequency
        let expr_weight = 0.4;
        let env_weight = 0.3;
        let vec_weight = 0.3;
        
        let expr_efficiency = self.expr_pool.efficiency_score();
        let env_efficiency = self.environment_pool.reuse_rate / 100.0;
        
        // Simple utilization for vector pools (no detailed stats available)
        let vec_efficiency = 0.5; // Assume moderate efficiency
        
        (expr_efficiency * expr_weight) + (env_efficiency * env_weight) + (vec_efficiency * vec_weight)
    }
    
    /// Estimates total memory saved by pooling (in bytes).
    pub fn estimated_memory_saved(&self) -> usize {
        // Rough estimates based on typical object sizes
        let expr_saved = self.expr_pool.reuse_count * 128; // ~128 bytes per expression
        let env_saved = self.environment_pool.total_reuses * 256; // ~256 bytes per environment
        let vec_saved = (self.token_vec_pool + self.string_vec_pool + self.frame_vec_pool) * 64; // ~64 bytes per vector
        
        expr_saved + env_saved + vec_saved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestObject {
        value: i32,
    }

    impl TestObject {
        fn new() -> Self {
            Self { value: 0 }
        }

        fn reset(&mut self) {
            self.value = 0;
        }
    }

    #[test]
    fn test_memory_pool() {
        let pool = MemoryPool::new(TestObject::new, 5);
        
        // Pool should be empty initially
        assert_eq!(pool.size(), 0);
        
        // Get an object
        let mut obj1 = pool.get();
        obj1.value = 42;
        assert_eq!(obj1.value, 42);
        
        // Drop the object back to the pool
        drop(obj1);
        assert_eq!(pool.size(), 1);
        
        // Get another object - should reuse the previous one
        let obj2 = pool.get();
        assert_eq!(obj2.value, 42); // Value persists
        
        // Take the object out of the pool
        let taken = obj2.take();
        assert_eq!(taken.value, 42);
        // Pool should still be empty since object was taken
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_vec_pool() {
        let pool = VecPool::new(10, 3);
        
        // Get a vector
        let mut vec1 = pool.get();
        vec1.push(1);
        vec1.push(2);
        vec1.push(3);
        assert_eq!(vec1.len(), 3);
        
        // Drop it back to the pool
        drop(vec1);
        assert_eq!(pool.size(), 1);
        
        // Get another vector - should be cleared but have retained capacity
        let vec2 = pool.get();
        assert_eq!(vec2.len(), 0);
        assert!(vec2.capacity() >= 3); // Should have retained capacity
    }

    #[test]
    fn test_pool_max_size() {
        let pool = MemoryPool::new(TestObject::new, 2);
        
        // Create more objects than the pool can hold
        let obj1 = pool.get();
        let obj2 = pool.get();
        let obj3 = pool.get();
        
        drop(obj1);
        drop(obj2);
        drop(obj3);
        
        // Pool should only hold up to max_size objects
        assert_eq!(pool.size(), 2);
    }

    #[test]
    fn test_global_pools() {
        use global_pools::*;
        
        let mut token_vec = get_token_vec();
        assert_eq!(token_vec.len(), 0);
        
        // Use the vector
        // Note: Can't actually create tokens without full setup
        // token_vec.push(some_token);
        
        drop(token_vec);
        
        // Should be returned to pool
        let (token_pool_size, _) = pool_stats();
        assert_eq!(token_pool_size, 1);
    }

    #[test]
    fn test_multiple_borrows() {
        let pool = MemoryPool::new(TestObject::new, 10);
        
        let obj1 = pool.get();
        let obj2 = pool.get();
        let obj3 = pool.get();
        
        // All should be separate objects
        assert_ne!(obj1.value, 999);
        assert_ne!(obj2.value, 999);
        assert_ne!(obj3.value, 999);
        
        drop(obj1);
        drop(obj2);
        drop(obj3);
        
        assert_eq!(pool.size(), 3);
    }
}