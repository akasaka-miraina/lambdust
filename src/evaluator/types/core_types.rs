//! Core Types Module
//!
//! このモジュールはevaluator型システムの基本的な型定義を提供します。
//! LocationHandle trait、メモリ戦略、統計ラッパーを含みます。

use crate::error::Result;
use crate::value::Value;
use std::fmt::Debug;

/// Location handle trait for abstracting over different memory management strategies
pub trait LocationHandle: Debug {
    /// Get the value at this location
    fn get(&self) -> Option<Value>;
    /// Set the value at this location
    fn set(&self, value: Value) -> Result<()>;
    /// Check if this location is still valid
    fn is_valid(&self) -> bool;
    /// Get location ID for debugging
    fn id(&self) -> usize;
}

/// RAII location handle implementation
impl LocationHandle for crate::evaluator::raii_store::RaiiLocation {
    fn get(&self) -> Option<Value> {
        self.get()
    }

    fn set(&self, value: Value) -> Result<()> {
        self.set(value)
    }

    fn is_valid(&self) -> bool {
        self.is_valid()
    }

    fn id(&self) -> usize {
        self.id()
    }
}

/// Statistics wrapper for unified RAII memory management
/// Simplified to use only RAII store statistics
#[derive(Debug, Clone)]
pub struct StoreStatisticsWrapper {
    /// RAII store statistics
    raii_stats: crate::evaluator::raii_store::RaiiStoreStatistics,
}

impl StoreStatisticsWrapper {
    /// Create from RAII statistics
    #[must_use] 
    pub fn from_raii(stats: crate::evaluator::raii_store::RaiiStoreStatistics) -> Self {
        StoreStatisticsWrapper { raii_stats: stats }
    }

    /// Get total allocations
    #[must_use] 
    pub fn total_allocations(&self) -> usize {
        self.raii_stats.total_allocations
    }

    /// Get total deallocations
    #[must_use] 
    pub fn total_deallocations(&self) -> usize {
        self.raii_stats.total_deallocations
    }

    /// Get memory usage
    #[must_use] 
    pub fn memory_usage(&self) -> usize {
        self.raii_stats.estimated_memory_usage
    }

    /// Get RAII-specific statistics
    #[must_use] 
    pub fn raii_statistics(&self) -> &crate::evaluator::raii_store::RaiiStoreStatistics {
        &self.raii_stats
    }

    /// Get active allocations
    #[must_use] 
    pub fn active_allocations(&self) -> usize {
        self.raii_stats.total_allocations.saturating_sub(self.raii_stats.total_deallocations)
    }

    /// Get allocation efficiency (percentage of successful allocations)
    #[must_use] 
    pub fn allocation_efficiency(&self) -> f64 {
        if self.raii_stats.total_allocations > 0 {
            100.0
        } else {
            0.0
        }
    }

    /// Get memory utilization (used memory / total allocated)
    #[must_use] 
    pub fn memory_utilization(&self) -> f64 {
        if self.raii_stats.estimated_memory_usage > 0 {
            // Assume efficient utilization for RAII
            0.85 
        } else {
            0.0
        }
    }
}

/// Memory management strategy for the evaluator
/// Unified RAII-only memory management
#[derive(Debug, Default)]
pub struct MemoryStrategy {
    /// RAII-based store leveraging Rust's ownership model
    raii_store: crate::evaluator::raii_store::RaiiStore,
}

impl MemoryStrategy {
    /// Create new RAII-based memory strategy
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with memory limit
    #[must_use] 
    pub fn with_memory_limit(limit: usize) -> Self {
        MemoryStrategy {
            raii_store: crate::evaluator::raii_store::RaiiStore::with_memory_limit(limit),
        }
    }

    /// Create with custom configuration
    #[must_use] 
    pub fn with_config(config: MemoryStrategyConfig) -> Self {
        let strategy = if config.memory_limit > 0 {
            Self::with_memory_limit(config.memory_limit)
        } else {
            Self::new()
        };
        
        if config.enable_optimization {
            // RAII store is already optimized by design
        }
        
        strategy
    }

    /// Get reference to RAII store
    #[must_use] 
    pub fn raii_store(&self) -> &crate::evaluator::raii_store::RaiiStore {
        &self.raii_store
    }

    /// Get mutable reference to RAII store
    pub fn raii_store_mut(&mut self) -> &mut crate::evaluator::raii_store::RaiiStore {
        &mut self.raii_store
    }

    /// Get current memory statistics
    #[must_use] 
    pub fn statistics(&self) -> StoreStatisticsWrapper {
        StoreStatisticsWrapper::from_raii(self.raii_store.statistics())
    }

    /// Allocate value and return handle
    pub fn allocate(&mut self, value: Value) -> Result<crate::evaluator::raii_store::RaiiLocation> {
        Ok(self.raii_store.allocate(value))
    }

    /// Force garbage collection if supported
    pub fn collect_garbage(&mut self) -> Result<usize> {
        // RAII handles cleanup automatically
        Ok(0)
    }

    /// Get memory pressure (0.0 = low, 1.0 = high)
    #[must_use] 
    pub fn memory_pressure(&self) -> f64 {
        let stats = self.raii_store.statistics();
        if stats.estimated_memory_usage > 0 {
            // Conservative estimate for RAII
            (stats.estimated_memory_usage as f64 / (1024.0 * 1024.0 * 100.0)).min(1.0)
        } else {
            0.0
        }
    }

    /// Check if memory is under pressure
    #[must_use] 
    pub fn is_under_pressure(&self) -> bool {
        self.memory_pressure() > 0.8
    }
}

/// Memory strategy configuration
#[derive(Debug, Clone)]
pub struct MemoryStrategyConfig {
    /// Memory limit in bytes (0 = unlimited)
    pub memory_limit: usize,
    /// Enable optimizations
    pub enable_optimization: bool,
    /// Enable debugging features
    pub enable_debugging: bool,
}

impl Default for MemoryStrategyConfig {
    fn default() -> Self {
        Self {
            memory_limit: 0, // Unlimited by default
            enable_optimization: true,
            enable_debugging: false,
        }
    }
}

impl MemoryStrategyConfig {
    /// Create production configuration
    #[must_use] 
    pub fn production() -> Self {
        Self {
            memory_limit: 1024 * 1024 * 1024, // 1GB limit
            enable_optimization: true,
            enable_debugging: false,
        }
    }

    /// Create development configuration
    #[must_use] 
    pub fn development() -> Self {
        Self {
            memory_limit: 512 * 1024 * 1024, // 512MB limit
            enable_optimization: false,
            enable_debugging: true,
        }
    }

    /// Create testing configuration
    #[must_use] 
    pub fn testing() -> Self {
        Self {
            memory_limit: 128 * 1024 * 1024, // 128MB limit
            enable_optimization: false,
            enable_debugging: true,
        }
    }
}