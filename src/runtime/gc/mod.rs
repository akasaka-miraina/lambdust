//! Parallel generational garbage collector for Lambdust
//!
//! This module provides a comprehensive parallel generational garbage collection
//! system designed for high-performance Scheme evaluation. The GC features:
//!
//! - **Generational Collection**: Young, old, and permanent generations with
//!   appropriate collection strategies for each
//! - **Parallel Coordination**: Thread-safe coordination between mutator and
//!   collector threads with safepoint management
//! - **Multiple Collection Algorithms**: Stop-the-world copying for young
//!   generation, concurrent mark-and-sweep for old generation, and incremental
//!   collection for low-latency scenarios
//! - **Advanced Allocation**: Thread-local allocation buffers (TLABs), 
//!   generation-aware allocation, and allocation sampling
//! - **Performance Integration**: Comprehensive metrics collection and
//!   integration with JIT performance monitoring
//! - **Adaptive Tuning**: Dynamic adjustment of collection parameters based
//!   on allocation patterns and performance metrics
//!
//! # Architecture
//!
//! The GC is built around several key components:
//!
//! 1. **ParallelGc**: Main coordinator that manages collection phases and
//!    thread synchronization
//! 2. **GenerationManager**: Manages young, old, and permanent generations
//!    with appropriate allocation and collection strategies
//! 3. **AllocationCoordinator**: Handles thread-local allocation buffers,
//!    allocation sampling, and generation-aware allocation decisions
//! 4. **Collection Algorithms**: Copying collector for young generation,
//!    mark-and-sweep for old generation, and incremental collection
//! 5. **Performance Monitoring**: Comprehensive metrics and adaptive tuning
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use lambdust::runtime::gc::{ParallelGc, ParallelGcConfig};
//!
//! // Create GC configuration
//! let config = ParallelGcConfig {
//!     young_generation_size: 64 * 1024 * 1024,  // 64MB
//!     old_generation_size: 256 * 1024 * 1024,   // 256MB
//!     target_minor_pause_ms: 10,                // 10ms target
//!     adaptive_tuning: true,
//!     ..Default::default()
//! };
//!
//! // Initialize GC
//! let mut gc = ParallelGc::new(config);
//! gc.initialize(None)?;
//!
//! // Register mutator thread
//! gc.register_mutator_thread();
//!
//! // Allocate objects (handled by AllocationCoordinator)
//! let coordinator = AllocationCoordinator::new(
//!     generation_manager,
//!     32 * 1024,    // Default TLAB size
//!     128 * 1024,   // Max TLAB size
//!     32 * 1024,    // Large object threshold
//! );
//!
//! let obj = coordinator.allocate(value, size)?;
//!
//! // Collections are triggered automatically or manually
//! gc.request_minor_collection()?;
//! ```
//!
//! # Performance Characteristics
//!
//! The GC is designed to achieve:
//! - Minor collection pause times: < 10ms
//! - Major collection pause times: < 50ms (concurrent)
//! - Thread scaling: Efficient use of available CPU cores
//! - Memory overhead: < 10% for GC metadata
//! - Allocation throughput: High-performance thread-local allocation

// Public module declarations
pub mod parallel_gc;
pub mod generation;
pub mod allocator;
pub mod collector;

// Re-export main types for convenient access
pub use parallel_gc::{
    ParallelGc, 
    ParallelGcConfig, 
    CollectionPhase, 
    GcStatistics, 
    SafepointCoordinator,
    AdaptiveTuningParams,
    CollectionRequest,
};

pub use generation::{
    GenerationManager,
    YoungGeneration,
    OldGeneration,
    Generation,
    ObjectHeader,
    GenerationId,
    GenerationStatistics,
    CollectionResult,
    MemoryRegion,
    HeapStatistics,
};

pub use allocator::{
    AllocationCoordinator,
    TlabManager,
    Tlab,
    AllocationSampler,
    AllocationSample,
    AllocationStatistics,
    TlabStatistics,
};

pub use collector::{
    CopyingCollector,
    MarkSweepCollector,
    IncrementalCollector,
    RootSet,
    WriteBarrier,
    ObjectMarker,
};

// Additional convenience types and functions

/// Result type for GC operations
pub type GcResult<T> = Result<T, String>;

/// GC configuration builder for convenient setup
pub struct GcConfigBuilder {
    config: ParallelGcConfig,
}

impl GcConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        GcConfigBuilder {
            config: ParallelGcConfig::default(),
        }
    }

    /// Set the number of collector threads
    pub fn collector_threads(mut self, threads: usize) -> Self {
        self.config.max_collector_threads = threads;
        self
    }

    /// Set young generation size in MB
    pub fn young_generation_mb(mut self, mb: usize) -> Self {
        self.config.young_generation_size = mb * 1024 * 1024;
        self
    }

    /// Set old generation size in MB
    pub fn old_generation_mb(mut self, mb: usize) -> Self {
        self.config.old_generation_size = mb * 1024 * 1024;
        self
    }

    /// Set large object threshold in KB
    pub fn large_object_threshold_kb(mut self, kb: usize) -> Self {
        self.config.large_object_threshold = kb * 1024;
        self
    }

    /// Set target minor pause time in milliseconds
    pub fn target_minor_pause_ms(mut self, ms: u64) -> Self {
        self.config.target_minor_pause_ms = ms;
        self
    }

    /// Set target major pause time in milliseconds
    pub fn target_major_pause_ms(mut self, ms: u64) -> Self {
        self.config.target_major_pause_ms = ms;
        self
    }

    /// Enable or disable NUMA-aware allocation
    pub fn numa_aware(mut self, enabled: bool) -> Self {
        self.config.numa_aware = enabled;
        self
    }

    /// Enable or disable adaptive tuning
    pub fn adaptive_tuning(mut self, enabled: bool) -> Self {
        self.config.adaptive_tuning = enabled;
        self
    }

    /// Build the configuration
    pub fn build(self) -> ParallelGcConfig {
        self.config
    }
}

impl Default for GcConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive GC system that combines all components
pub struct GcSystem {
    /// Main GC coordinator
    pub parallel_gc: ParallelGc,
    /// Generation manager
    pub generation_manager: Arc<GenerationManager>,
    /// Allocation coordinator
    pub allocation_coordinator: Arc<AllocationCoordinator>,
    /// Copying collector
    pub copying_collector: Arc<CopyingCollector>,
    /// Mark-sweep collector
    pub mark_sweep_collector: Arc<MarkSweepCollector>,
    /// Incremental collector
    pub incremental_collector: Arc<IncrementalCollector>,
    /// Root set
    pub root_set: Arc<RootSet>,
}

use std::sync::Arc;

impl GcSystem {
    /// Create a new comprehensive GC system
    pub fn new(config: ParallelGcConfig) -> GcResult<Self> {
        // Create generation manager
        let generation_manager = Arc::new(GenerationManager::new(
            config.young_generation_size,
            config.old_generation_size,
        )?);

        // Create allocation coordinator
        let allocation_coordinator = Arc::new(AllocationCoordinator::new(
            Arc::clone(&generation_manager),
            32 * 1024,  // Default TLAB size: 32KB
            128 * 1024, // Max TLAB size: 128KB
            config.large_object_threshold,
        ));

        // Create root set
        let root_set = Arc::new(RootSet::new());

        // Create main GC coordinator
        let parallel_gc = ParallelGc::new(config);
        let safepoint = Arc::new(SafepointCoordinator::new());
        let statistics = Arc::new(GcStatistics::new());

        // Create collectors
        let copying_collector = Arc::new(CopyingCollector::new(
            Arc::clone(&safepoint),
            Arc::clone(&root_set),
            Arc::clone(&statistics),
        ));

        let mark_sweep_collector = Arc::new(MarkSweepCollector::new(
            Arc::clone(&root_set),
            Arc::clone(&statistics),
        ));

        let incremental_collector = Arc::new(IncrementalCollector::new(
            Arc::clone(&copying_collector),
            Arc::clone(&mark_sweep_collector),
            1000, // 1ms step budget
        ));

        Ok(GcSystem {
            parallel_gc,
            generation_manager,
            allocation_coordinator,
            copying_collector,
            mark_sweep_collector,
            incremental_collector,
            root_set,
        })
    }

    /// Initialize the GC system with optional JIT metrics
    pub fn initialize(&mut self, jit_metrics: Option<Arc<std::sync::RwLock<crate::jit::metrics::JitMetrics>>>) -> GcResult<()> {
        self.parallel_gc.initialize(jit_metrics)
    }

    /// Perform a minor collection
    pub fn collect_minor(&self) -> GcResult<CollectionResult> {
        self.copying_collector.collect()
    }

    /// Perform a major collection
    pub fn collect_major(&self, concurrent: bool) -> GcResult<CollectionResult> {
        self.mark_sweep_collector.collect(concurrent)
    }

    /// Perform an incremental collection step
    pub fn collect_incremental_step(&self) -> GcResult<bool> {
        self.incremental_collector.perform_incremental_step()
    }

    /// Allocate a new object
    pub fn allocate(&self, value: crate::eval::value::Value, size: usize) -> GcResult<Arc<ObjectHeader>> {
        self.allocation_coordinator.allocate(value, size)
    }

    /// Register the current thread as a mutator
    pub fn register_mutator_thread(&self) {
        self.parallel_gc.register_mutator_thread();
    }

    /// Unregister the current thread as a mutator
    pub fn unregister_mutator_thread(&self) {
        self.parallel_gc.unregister_mutator_thread();
    }

    /// Get comprehensive GC statistics
    pub fn get_statistics(&self) -> GcSystemStatistics {
        let gc_stats = self.parallel_gc.get_statistics();
        let heap_stats = self.generation_manager.get_heap_statistics();
        let allocation_stats = self.allocation_coordinator.get_statistics();
        let tlab_stats = self.allocation_coordinator.get_tlab_statistics();

        GcSystemStatistics {
            // GC statistics
            minor_collections: gc_stats.minor_collections.load(std::sync::atomic::Ordering::Relaxed),
            major_collections: gc_stats.major_collections.load(std::sync::atomic::Ordering::Relaxed),
            avg_minor_pause_ms: gc_stats.avg_minor_pause_ns.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1_000_000.0,
            avg_major_pause_ms: gc_stats.avg_major_pause_ns.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1_000_000.0,
            
            // Heap statistics
            total_allocations: heap_stats.total_allocations,
            total_allocated_bytes: heap_stats.total_allocated_bytes,
            young_utilization: heap_stats.young_utilization,
            old_utilization: heap_stats.old_utilization,
            
            // Allocation statistics
            allocation_rate: self.allocation_coordinator.allocation_rate(),
            failure_rate: allocation_stats.failure_rate(),
            
            // TLAB statistics
            tlab_utilization: tlab_stats.average_utilization(),
            tlab_waste_percentage: tlab_stats.waste_percentage(),
        }
    }

    /// Shutdown the GC system
    pub fn shutdown(self) -> GcResult<()> {
        self.parallel_gc.shutdown()
    }
}

/// Comprehensive GC system statistics
#[derive(Debug)]
pub struct GcSystemStatistics {
    // Collection statistics
    /// Total minor collections performed
    pub minor_collections: u64,
    /// Total major collections performed
    pub major_collections: u64,
    /// Average minor collection pause time (milliseconds)
    pub avg_minor_pause_ms: f64,
    /// Average major collection pause time (milliseconds)  
    pub avg_major_pause_ms: f64,
    
    // Heap statistics
    /// Total allocations across all generations
    pub total_allocations: u64,
    /// Total bytes allocated across all generations
    pub total_allocated_bytes: u64,
    /// Young generation utilization percentage
    pub young_utilization: f64,
    /// Old generation utilization percentage
    pub old_utilization: f64,
    
    // Performance statistics
    /// Current allocation rate (objects per second)
    pub allocation_rate: f64,
    /// Allocation failure rate percentage
    pub failure_rate: f64,
    
    // TLAB statistics
    /// Average TLAB utilization percentage
    pub tlab_utilization: f64,
    /// TLAB waste percentage
    pub tlab_waste_percentage: f64,
}

impl GcSystemStatistics {
    /// Check if GC performance is healthy
    pub fn is_healthy(&self) -> bool {
        // Define health criteria
        self.avg_minor_pause_ms < 15.0  // Minor pauses under 15ms
            && self.avg_major_pause_ms < 75.0   // Major pauses under 75ms
            && self.failure_rate < 1.0          // Failure rate under 1%
            && self.tlab_utilization > 70.0     // Good TLAB utilization
            && self.young_utilization < 95.0    // Not overwhelmed
            && self.old_utilization < 90.0      // Not overwhelmed
    }

    /// Get overall performance score (0.0 to 1.0)
    pub fn performance_score(&self) -> f64 {
        let pause_score = 1.0 - (self.avg_minor_pause_ms / 50.0).min(1.0); // Normalize against 50ms
        let utilization_score = (self.tlab_utilization / 100.0).min(1.0);
        let failure_score = 1.0 - (self.failure_rate / 10.0).min(1.0); // Normalize against 10%
        let allocation_score = if self.allocation_rate > 0.0 { 1.0 } else { 0.0 };

        (pause_score + utilization_score + failure_score + allocation_score) / 4.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::Value;
    use crate::ast::Literal;

    #[test]
    fn test_gc_config_builder() {
        let config = GcConfigBuilder::new()
            .young_generation_mb(32)
            .old_generation_mb(128)
            .target_minor_pause_ms(8)
            .adaptive_tuning(true)
            .build();

        assert_eq!(config.young_generation_size, 32 * 1024 * 1024);
        assert_eq!(config.old_generation_size, 128 * 1024 * 1024);
        assert_eq!(config.target_minor_pause_ms, 8);
        assert!(config.adaptive_tuning);
    }

    #[test]
    fn test_gc_system_creation() {
        let config = ParallelGcConfig::default();
        let gc_system = GcSystem::new(config);
        assert!(gc_system.is_ok());
    }

    #[test]
    fn test_basic_allocation() {
        let config = GcConfigBuilder::new()
            .young_generation_mb(16)
            .old_generation_mb(64)
            .build();
            
        let gc_system = GcSystem::new(config).unwrap();
        let value = Value::Literal(Literal::Number(42.0));
        
        let result = gc_system.allocate(value, 64);
        assert!(result.is_ok());
        
        let header = result.unwrap();
        assert_eq!(header.size, 64);
        assert_eq!(header.generation, GenerationId::Young);
    }

    #[test]
    fn test_statistics_health_check() {
        let stats = GcSystemStatistics {
            minor_collections: 100,
            major_collections: 10,
            avg_minor_pause_ms: 8.5,
            avg_major_pause_ms: 45.0,
            total_allocations: 10000,
            total_allocated_bytes: 1024 * 1024,
            young_utilization: 75.0,
            old_utilization: 60.0,
            allocation_rate: 1000.0,
            failure_rate: 0.1,
            tlab_utilization: 85.0,
            tlab_waste_percentage: 5.0,
        };

        assert!(stats.is_healthy());
        assert!(stats.performance_score() > 0.8);
    }
}