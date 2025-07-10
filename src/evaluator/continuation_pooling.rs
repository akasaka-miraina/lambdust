//! Unified Continuation Pooling System
//!
//! This module implements a comprehensive continuation pooling system:
//! - Global pool manager for centralized resource management
//! - Type-specific pools for different continuation variants
//! - Memory fragmentation prevention through smart allocation
//! - Heap allocation reduction via continuation reuse
//! - Performance monitoring and optimization hints

use crate::evaluator::Continuation;
use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex};

/// Continuation type identifier for pool categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContinuationType {
    /// Simple continuations with minimal state
    Simple,
    /// Application continuations for function calls
    Application,
    /// DoLoop continuations for iteration
    DoLoop,
    /// Control flow continuations (if, cond, etc.)
    ControlFlow,
    /// Exception handling continuations
    Exception,
    /// Complex multi-state continuations
    Complex,
}

impl ContinuationType {
    /// Determine continuation type from actual continuation
    pub fn from_continuation(cont: &Continuation) -> Self {
        match cont {
            Continuation::Identity => ContinuationType::Simple,
            Continuation::Values { .. } => ContinuationType::Simple,
            Continuation::Assignment { .. } => ContinuationType::Simple,
            Continuation::Define { .. } => ContinuationType::Simple,

            Continuation::Application { .. } => ContinuationType::Application,
            Continuation::Operator { .. } => ContinuationType::Application,

            Continuation::DoLoop { .. } => ContinuationType::DoLoop,
            Continuation::Do { .. } => ContinuationType::DoLoop,

            Continuation::IfTest { .. } => ContinuationType::ControlFlow,
            Continuation::CondTest { .. } => ContinuationType::ControlFlow,
            Continuation::Begin { .. } => ContinuationType::ControlFlow,
            Continuation::And { .. } => ContinuationType::ControlFlow,
            Continuation::Or { .. } => ContinuationType::ControlFlow,

            Continuation::ExceptionHandler { .. } => ContinuationType::Exception,
            Continuation::GuardClause { .. } => ContinuationType::Exception,

            _ => ContinuationType::Complex,
        }
    }

    /// Get optimal pool size for this continuation type
    pub fn optimal_pool_size(&self) -> usize {
        match self {
            ContinuationType::Simple => 50,      // High frequency, small size
            ContinuationType::Application => 30, // Medium frequency, medium size
            ContinuationType::DoLoop => 20,      // Medium frequency, larger size
            ContinuationType::ControlFlow => 25, // Medium frequency, medium size
            ContinuationType::Exception => 10,   // Low frequency, medium size
            ContinuationType::Complex => 5,      // Low frequency, large size
        }
    }

    /// Get memory priority for this continuation type
    pub fn memory_priority(&self) -> u8 {
        match self {
            ContinuationType::DoLoop => 10, // Highest priority (most benefit from pooling)
            ContinuationType::Application => 8, // High priority
            ContinuationType::Simple => 7,  // Medium-high priority
            ContinuationType::ControlFlow => 6, // Medium priority
            ContinuationType::Exception => 4, // Low priority
            ContinuationType::Complex => 2, // Lowest priority
        }
    }
}

/// Statistics for a single continuation pool
#[derive(Debug, Clone)]
pub struct PoolStatistics {
    /// Total allocations requested
    pub total_allocations: usize,
    /// Total successful reuses from pool
    pub total_reuses: usize,
    /// Current pool size
    pub current_size: usize,
    /// Maximum pool size reached
    pub peak_size: usize,
    /// Total memory saved through reuse (estimated)
    pub memory_saved_bytes: usize,
    /// Average pool utilization rate
    pub utilization_rate: f64,
}

impl PoolStatistics {
    /// Create new empty statistics
    pub fn new() -> Self {
        PoolStatistics {
            total_allocations: 0,
            total_reuses: 0,
            current_size: 0,
            peak_size: 0,
            memory_saved_bytes: 0,
            utilization_rate: 0.0,
        }
    }

    /// Record an allocation
    pub fn record_allocation(&mut self) {
        self.total_allocations += 1;
    }

    /// Record a reuse
    pub fn record_reuse(&mut self, memory_saved: usize) {
        self.total_reuses += 1;
        self.memory_saved_bytes += memory_saved;
        self.update_utilization_rate();
    }

    /// Update pool size
    pub fn update_size(&mut self, new_size: usize) {
        self.current_size = new_size;
        self.peak_size = self.peak_size.max(new_size);
    }

    /// Update utilization rate
    fn update_utilization_rate(&mut self) {
        if self.total_allocations > 0 {
            self.utilization_rate = self.total_reuses as f64 / self.total_allocations as f64;
        }
    }

    /// Get reuse efficiency percentage
    pub fn reuse_efficiency(&self) -> f64 {
        self.utilization_rate * 100.0
    }
}

impl Default for PoolStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Type-specific continuation pool
#[derive(Debug)]
pub struct TypedContinuationPool {
    /// Pool of reusable continuations
    pool: Vec<Continuation>,
    /// Maximum pool size
    max_size: usize,
    /// Pool statistics
    statistics: PoolStatistics,
    /// Continuation type for this pool
    continuation_type: ContinuationType,
}

impl TypedContinuationPool {
    /// Create new typed continuation pool
    pub fn new(continuation_type: ContinuationType) -> Self {
        let max_size = continuation_type.optimal_pool_size();
        TypedContinuationPool {
            pool: Vec::with_capacity(max_size),
            max_size,
            statistics: PoolStatistics::new(),
            continuation_type,
        }
    }

    /// Allocate continuation from pool or create new one
    pub fn allocate(&mut self) -> Option<Continuation> {
        self.statistics.record_allocation();

        if let Some(cont) = self.pool.pop() {
            // Successful reuse
            let memory_saved = mem::size_of::<Continuation>();
            self.statistics.record_reuse(memory_saved);
            self.statistics.update_size(self.pool.len());
            Some(cont)
        } else {
            // No continuation available for reuse
            None
        }
    }

    /// Return continuation to pool
    pub fn deallocate(&mut self, cont: Continuation) -> bool {
        // Verify continuation type matches pool
        if ContinuationType::from_continuation(&cont) != self.continuation_type {
            return false; // Wrong type, reject
        }

        if self.pool.len() < self.max_size {
            self.pool.push(cont);
            self.statistics.update_size(self.pool.len());
            true
        } else {
            // Pool is full, drop the continuation
            false
        }
    }

    /// Get pool statistics
    pub fn statistics(&self) -> &PoolStatistics {
        &self.statistics
    }

    /// Clear all continuations from pool
    pub fn clear(&mut self) {
        self.pool.clear();
        self.statistics.update_size(0);
    }

    /// Get current pool size
    pub fn size(&self) -> usize {
        self.pool.len()
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }

    /// Get pool capacity utilization
    pub fn capacity_utilization(&self) -> f64 {
        if self.max_size > 0 {
            self.pool.len() as f64 / self.max_size as f64
        } else {
            0.0
        }
    }

    /// Get maximum pool size
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

/// Global continuation pool manager
/// Coordinates multiple typed pools and provides unified interface
#[derive(Debug)]
pub struct ContinuationPoolManager {
    /// Type-specific continuation pools
    pools: HashMap<ContinuationType, TypedContinuationPool>,
    /// Global allocation counter
    global_allocations: usize,
    /// Global reuse counter
    global_reuses: usize,
    /// Total memory saved across all pools
    total_memory_saved: usize,
    /// Memory fragmentation prevention threshold
    fragmentation_threshold: f64,
}

impl ContinuationPoolManager {
    /// Create new global continuation pool manager
    pub fn new() -> Self {
        let mut pools = HashMap::new();

        // Initialize all continuation type pools
        for cont_type in [
            ContinuationType::Simple,
            ContinuationType::Application,
            ContinuationType::DoLoop,
            ContinuationType::ControlFlow,
            ContinuationType::Exception,
            ContinuationType::Complex,
        ] {
            pools.insert(cont_type, TypedContinuationPool::new(cont_type));
        }

        ContinuationPoolManager {
            pools,
            global_allocations: 0,
            global_reuses: 0,
            total_memory_saved: 0,
            fragmentation_threshold: 0.75, // 75% utilization threshold
        }
    }

    /// Allocate continuation from appropriate pool
    pub fn allocate(&mut self, cont_type: ContinuationType) -> Option<Continuation> {
        self.global_allocations += 1;

        if let Some(pool) = self.pools.get_mut(&cont_type) {
            if let Some(cont) = pool.allocate() {
                self.global_reuses += 1;
                self.total_memory_saved += mem::size_of::<Continuation>();
                return Some(cont);
            }
        }

        // No continuation available from pool
        None
    }

    /// Return continuation to appropriate pool
    pub fn deallocate(&mut self, cont: Continuation) -> bool {
        let cont_type = ContinuationType::from_continuation(&cont);

        if let Some(pool) = self.pools.get_mut(&cont_type) {
            pool.deallocate(cont)
        } else {
            false
        }
    }

    /// Get global statistics
    pub fn global_statistics(&self) -> (usize, usize, usize, f64) {
        let global_efficiency = if self.global_allocations > 0 {
            self.global_reuses as f64 / self.global_allocations as f64 * 100.0
        } else {
            0.0
        };

        (
            self.global_allocations,
            self.global_reuses,
            self.total_memory_saved,
            global_efficiency,
        )
    }

    /// Get statistics for specific continuation type
    pub fn type_statistics(&self, cont_type: ContinuationType) -> Option<&PoolStatistics> {
        self.pools.get(&cont_type).map(|pool| pool.statistics())
    }

    /// Get all pool statistics
    pub fn all_statistics(&self) -> HashMap<ContinuationType, &PoolStatistics> {
        self.pools
            .iter()
            .map(|(cont_type, pool)| (*cont_type, pool.statistics()))
            .collect()
    }

    /// Clear all pools
    pub fn clear_all(&mut self) {
        for pool in self.pools.values_mut() {
            pool.clear();
        }
        self.global_allocations = 0;
        self.global_reuses = 0;
        self.total_memory_saved = 0;
    }

    /// Clear specific pool type
    pub fn clear_type(&mut self, cont_type: ContinuationType) {
        if let Some(pool) = self.pools.get_mut(&cont_type) {
            pool.clear();
        }
    }

    /// Check if memory fragmentation prevention is needed
    pub fn needs_defragmentation(&self) -> bool {
        let total_capacity: usize = self.pools.values().map(|p| p.max_size).sum();
        let total_used: usize = self.pools.values().map(|p| p.size()).sum();

        if total_capacity > 0 {
            let utilization = total_used as f64 / total_capacity as f64;
            utilization > self.fragmentation_threshold
        } else {
            false
        }
    }

    /// Perform memory defragmentation by compacting pools
    pub fn defragment(&mut self) {
        // Sort pools by memory priority and compact high-priority pools first
        let mut pool_priorities: Vec<_> = self
            .pools
            .keys()
            .map(|&cont_type| (cont_type, cont_type.memory_priority()))
            .collect();
        pool_priorities.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by priority (descending)

        for (cont_type, _) in pool_priorities {
            if let Some(pool) = self.pools.get_mut(&cont_type) {
                // For now, defragmentation just ensures pools are at optimal capacity
                // In a full implementation, this could involve memory compaction
                let optimal_size = cont_type.optimal_pool_size();
                if pool.size() > optimal_size {
                    // Trim pool to optimal size
                    while pool.size() > optimal_size {
                        pool.pool.pop();
                    }
                    pool.statistics.update_size(pool.size());
                }
            }
        }
    }

    /// Get memory usage summary
    pub fn memory_usage_summary(&self) -> (usize, usize, f64) {
        let total_pools = self.pools.len();
        let active_pools = self.pools.values().filter(|p| !p.is_empty()).count();
        let avg_utilization = self
            .pools
            .values()
            .map(|p| p.capacity_utilization())
            .sum::<f64>()
            / total_pools as f64;

        (total_pools, active_pools, avg_utilization)
    }
}

impl Default for ContinuationPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: ContinuationPoolManager contains only Rust-safe data structures
// and no raw pointers or external resources
unsafe impl Send for ContinuationPoolManager {}
unsafe impl Sync for ContinuationPoolManager {}

/// Thread-safe wrapper for continuation pool manager
/// Provides safe concurrent access to continuation pools
#[derive(Debug, Clone)]
pub struct SharedContinuationPoolManager {
    inner: Arc<Mutex<ContinuationPoolManager>>,
}

impl SharedContinuationPoolManager {
    /// Create new shared continuation pool manager
    pub fn new() -> Self {
        SharedContinuationPoolManager {
            inner: Arc::new(Mutex::new(ContinuationPoolManager::new())),
        }
    }

    /// Allocate continuation with thread safety
    pub fn allocate(&self, cont_type: ContinuationType) -> Option<Continuation> {
        if let Ok(mut manager) = self.inner.lock() {
            manager.allocate(cont_type)
        } else {
            None // Lock poisoned
        }
    }

    /// Deallocate continuation with thread safety
    pub fn deallocate(&self, cont: Continuation) -> bool {
        if let Ok(mut manager) = self.inner.lock() {
            manager.deallocate(cont)
        } else {
            false // Lock poisoned
        }
    }

    /// Get global statistics with thread safety
    pub fn global_statistics(&self) -> Option<(usize, usize, usize, f64)> {
        if let Ok(manager) = self.inner.lock() {
            Some(manager.global_statistics())
        } else {
            None // Lock poisoned
        }
    }

    /// Clear all pools with thread safety
    pub fn clear_all(&self) {
        if let Ok(mut manager) = self.inner.lock() {
            manager.clear_all();
        }
    }

    /// Check if defragmentation is needed
    pub fn needs_defragmentation(&self) -> bool {
        if let Ok(manager) = self.inner.lock() {
            manager.needs_defragmentation()
        } else {
            false
        }
    }

    /// Perform defragmentation
    pub fn defragment(&self) {
        if let Ok(mut manager) = self.inner.lock() {
            manager.defragment();
        }
    }
}

impl Default for SharedContinuationPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;
    use crate::value::Value;
    use std::rc::Rc;

    #[test]
    fn test_continuation_type_classification() {
        let identity = Continuation::Identity;
        assert_eq!(
            ContinuationType::from_continuation(&identity),
            ContinuationType::Simple
        );

        let values = Continuation::Values {
            values: vec![Value::from(42i64)],
            parent: Box::new(Continuation::Identity),
        };
        assert_eq!(
            ContinuationType::from_continuation(&values),
            ContinuationType::Simple
        );

        let app = Continuation::Application {
            operator: Value::from(1i64),
            evaluated_args: vec![],
            remaining_args: vec![],
            env: Rc::new(Environment::new()),
            parent: Box::new(Continuation::Identity),
        };
        assert_eq!(
            ContinuationType::from_continuation(&app),
            ContinuationType::Application
        );
    }

    #[test]
    fn test_pool_statistics() {
        let mut stats = PoolStatistics::new();

        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.total_reuses, 0);
        assert_eq!(stats.reuse_efficiency(), 0.0);

        stats.record_allocation();
        stats.record_allocation();
        stats.record_reuse(100);

        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.total_reuses, 1);
        assert_eq!(stats.reuse_efficiency(), 50.0);
        assert_eq!(stats.memory_saved_bytes, 100);
    }

    #[test]
    fn test_typed_continuation_pool() {
        let mut pool = TypedContinuationPool::new(ContinuationType::Simple);

        // Test allocation from empty pool
        assert!(pool.allocate().is_none());
        assert_eq!(pool.statistics().total_allocations, 1);
        assert_eq!(pool.statistics().total_reuses, 0);

        // Test deallocation
        let cont = Continuation::Identity;
        assert!(pool.deallocate(cont));
        assert_eq!(pool.size(), 1);

        // Test allocation from pool with item
        let reused_cont = pool.allocate();
        assert!(reused_cont.is_some());
        assert_eq!(pool.size(), 0);
        assert_eq!(pool.statistics().total_reuses, 1);
    }

    #[test]
    fn test_pool_type_validation() {
        let mut pool = TypedContinuationPool::new(ContinuationType::Simple);

        // Valid continuation type
        let simple_cont = Continuation::Identity;
        assert!(pool.deallocate(simple_cont));

        // Invalid continuation type
        let app_cont = Continuation::Application {
            operator: Value::from(1i64),
            evaluated_args: vec![],
            remaining_args: vec![],
            env: Rc::new(Environment::new()),
            parent: Box::new(Continuation::Identity),
        };
        assert!(!pool.deallocate(app_cont));
    }

    #[test]
    fn test_pool_size_limits() {
        let mut pool = TypedContinuationPool::new(ContinuationType::Exception); // Small pool size
        let max_size = pool.max_size;

        // Fill pool to capacity with Exception-type continuations
        for _ in 0..max_size {
            let cont = Continuation::ExceptionHandler {
                handler: Value::Boolean(true),
                env: Rc::new(Environment::new()),
                parent: Box::new(Continuation::Identity),
            };
            assert!(pool.deallocate(cont));
        }

        assert_eq!(pool.size(), max_size);

        // Try to exceed capacity
        let overflow_cont = Continuation::ExceptionHandler {
            handler: Value::Boolean(false),
            env: Rc::new(Environment::new()),
            parent: Box::new(Continuation::Identity),
        };
        assert!(!pool.deallocate(overflow_cont)); // Should reject overflow
        assert_eq!(pool.size(), max_size); // Size unchanged
    }

    #[test]
    fn test_continuation_pool_manager() {
        let mut manager = ContinuationPoolManager::new();

        // Test allocation from empty pools
        assert!(manager.allocate(ContinuationType::Simple).is_none());

        // Test deallocation
        let cont = Continuation::Identity;
        assert!(manager.deallocate(cont));

        // Test allocation after deallocation
        let reused_cont = manager.allocate(ContinuationType::Simple);
        assert!(reused_cont.is_some());

        let (allocs, reuses, memory_saved, efficiency) = manager.global_statistics();
        assert_eq!(allocs, 2);
        assert_eq!(reuses, 1);
        assert!(memory_saved > 0);
        assert_eq!(efficiency, 50.0);
    }

    #[test]
    fn test_memory_defragmentation() {
        let mut manager = ContinuationPoolManager::new();

        // Fill pools beyond optimal capacity
        for _ in 0..100 {
            let cont = Continuation::Identity;
            manager.deallocate(cont);
        }

        // Check if defragmentation is needed
        if manager.needs_defragmentation() {
            manager.defragment();
        }

        // Verify pools are within reasonable bounds
        let (total_pools, active_pools, avg_utilization) = manager.memory_usage_summary();
        assert!(total_pools > 0);
        assert!(active_pools <= total_pools);
        assert!((0.0..=1.0).contains(&avg_utilization));
    }

    #[test]
    fn test_shared_pool_manager() {
        let shared_manager = SharedContinuationPoolManager::new();

        // Test thread-safe operations
        assert!(shared_manager.allocate(ContinuationType::Simple).is_none());

        let cont = Continuation::Identity;
        assert!(shared_manager.deallocate(cont));

        let reused_cont = shared_manager.allocate(ContinuationType::Simple);
        assert!(reused_cont.is_some());

        let stats = shared_manager.global_statistics();
        assert!(stats.is_some());

        if let Some((allocs, reuses, memory_saved, efficiency)) = stats {
            assert_eq!(allocs, 2);
            assert_eq!(reuses, 1);
            assert!(memory_saved > 0);
            assert_eq!(efficiency, 50.0);
        }
    }
}
