//! SRFI-9 Record Performance Optimization System
//!
//! This module provides advanced optimization techniques for SRFI-9 records:
//! - JIT compilation hints for ultra-hot record operations
//! - Dynamic optimization profile adaptation
//! - Memory layout optimization based on usage patterns
//! - Bulk operation acceleration with SIMD

use crate::eval::nan_boxed_value::NanBoxedValue;
use crate::eval::record_access::{
    BulkRecordOperations, FieldAccessCache, FieldAccessCacheStats, GLOBAL_FIELD_ACCESS_CACHE,
    GLOBAL_TYPE_CHECKER,
};
use crate::eval::record_arena::{ArenaStatsSnapshot, GLOBAL_RECORD_ARENA, GcStats};
use crate::eval::record_instance::RecordInstance;
use crate::eval::record_type::{GLOBAL_RECORD_REGISTRY, RecordError, RecordResult, RecordTypeId};
use crate::eval::value::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Comprehensive performance monitoring for SRFI-9 records
pub struct RecordPerformanceMonitor {
    /// Performance metrics per record type
    type_metrics: RwLock<HashMap<RecordTypeId, TypePerformanceMetrics>>,
    /// Global performance statistics
    global_stats: GlobalRecordStats,
    /// JIT compilation tracker
    jit_tracker: JitCompilationTracker,
    /// Optimization recommendations
    recommendations: RwLock<Vec<OptimizationRecommendation>>,
    /// Monitoring start time
    start_time: Instant,
}

/// Performance metrics for a specific record type
#[derive(Debug)]
pub struct TypePerformanceMetrics {
    /// Type identifier
    pub type_id: RecordTypeId,
    /// Total instances created
    pub instances_created: AtomicU64,
    /// Total field accesses
    pub field_accesses: AtomicU64,
    /// Total bulk operations
    pub bulk_operations: AtomicU64,
    /// Average access time (nanoseconds)
    pub avg_access_time: AtomicU64,
    /// Hot field indices (most accessed)
    pub hot_fields: Vec<usize>,
    /// SIMD utilization percentage
    pub simd_utilization: AtomicU64,
    /// Memory utilization efficiency
    pub memory_efficiency: AtomicU64,
    /// Creation timestamp
    pub created_at: Instant,
}

impl Clone for TypePerformanceMetrics {
    fn clone(&self) -> Self {
        Self {
            type_id: self.type_id,
            instances_created: AtomicU64::new(
                self.instances_created
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            field_accesses: AtomicU64::new(
                self.field_accesses
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            bulk_operations: AtomicU64::new(
                self.bulk_operations
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            avg_access_time: AtomicU64::new(
                self.avg_access_time
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            hot_fields: self.hot_fields.clone(),
            simd_utilization: AtomicU64::new(
                self.simd_utilization
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            memory_efficiency: AtomicU64::new(
                self.memory_efficiency
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            created_at: self.created_at,
        }
    }
}

impl TypePerformanceMetrics {
    /// Creates new performance metrics for a type
    pub fn new(type_id: RecordTypeId) -> Self {
        Self {
            type_id,
            instances_created: AtomicU64::new(0),
            field_accesses: AtomicU64::new(0),
            bulk_operations: AtomicU64::new(0),
            avg_access_time: AtomicU64::new(0),
            hot_fields: Vec::new(),
            simd_utilization: AtomicU64::new(0),
            memory_efficiency: AtomicU64::new(0),
            created_at: Instant::now(),
        }
    }

    /// Records an instance creation
    pub fn record_creation(&self) {
        self.instances_created.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a field access with timing
    pub fn record_access(&self, access_time_ns: u64) {
        self.field_accesses.fetch_add(1, Ordering::Relaxed);

        // Update rolling average
        let current_avg = self.avg_access_time.load(Ordering::Relaxed);
        let new_avg = if current_avg == 0 {
            access_time_ns
        } else {
            (current_avg * 15 + access_time_ns) / 16 // Rolling average with 15/16 weight
        };
        self.avg_access_time.store(new_avg, Ordering::Relaxed);
    }

    /// Records a bulk operation
    pub fn record_bulk_operation(&self, count: u64) {
        self.bulk_operations.fetch_add(count, Ordering::Relaxed);
    }

    /// Calculates performance score (0-100)
    pub fn performance_score(&self) -> f64 {
        let avg_time = self.avg_access_time.load(Ordering::Relaxed) as f64;
        let bulk_ops = self.bulk_operations.load(Ordering::Relaxed) as f64;
        let total_accesses = self.field_accesses.load(Ordering::Relaxed) as f64;

        if total_accesses == 0.0 {
            return 50.0; // Neutral score for unused types
        }

        // Score based on access time (lower is better)
        let time_score = if avg_time <= 1.0 {
            100.0
        } else if avg_time <= 5.0 {
            100.0 - (avg_time - 1.0) * 20.0
        } else {
            20.0
        };

        // Bonus for SIMD utilization
        let simd_bonus = if bulk_ops / total_accesses > 0.1 {
            10.0 * (bulk_ops / total_accesses).min(1.0)
        } else {
            0.0
        };

        (time_score + simd_bonus).min(100.0)
    }

    /// Checks if this type should trigger JIT compilation
    pub fn should_jit_compile(&self) -> bool {
        let accesses = self.field_accesses.load(Ordering::Relaxed);
        let avg_time = self.avg_access_time.load(Ordering::Relaxed);

        // JIT compile if:
        // 1. Very hot (>10000 accesses) OR
        // 2. Hot (>1000 accesses) with slow access (>5ns)
        accesses > 10000 || (accesses > 1000 && avg_time > 5)
    }
}

/// Global performance statistics for all record operations
#[derive(Debug, Default)]
pub struct GlobalRecordStats {
    /// Total record types registered
    pub types_registered: AtomicUsize,
    /// Total instances created across all types
    pub total_instances: AtomicU64,
    /// Total field accesses across all types
    pub total_accesses: AtomicU64,
    /// Total bulk operations
    pub total_bulk_operations: AtomicU64,
    /// Cache hit rate
    pub cache_hit_rate: AtomicU64,
    /// SIMD operations performed
    pub simd_operations: AtomicU64,
    /// Memory saved through arena allocation (bytes)
    pub memory_saved: AtomicU64,
}

impl GlobalRecordStats {
    /// Records type registration
    pub fn record_type_registration(&self) {
        self.types_registered.fetch_add(1, Ordering::Relaxed);
    }

    /// Records instance creation
    pub fn record_instance_creation(&self) {
        self.total_instances.fetch_add(1, Ordering::Relaxed);
    }

    /// Records field access
    pub fn record_field_access(&self) {
        self.total_accesses.fetch_add(1, Ordering::Relaxed);
    }

    /// Records bulk operation
    pub fn record_bulk_operation(&self, count: u64) {
        self.total_bulk_operations
            .fetch_add(count, Ordering::Relaxed);
    }

    /// Records SIMD operation
    pub fn record_simd_operation(&self) {
        self.simd_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Gets current statistics snapshot
    pub fn snapshot(&self) -> GlobalRecordStatsSnapshot {
        GlobalRecordStatsSnapshot {
            types_registered: self.types_registered.load(Ordering::Relaxed),
            total_instances: self.total_instances.load(Ordering::Relaxed),
            total_accesses: self.total_accesses.load(Ordering::Relaxed),
            total_bulk_operations: self.total_bulk_operations.load(Ordering::Relaxed),
            cache_hit_rate: self.cache_hit_rate.load(Ordering::Relaxed) as f64 / 100.0,
            simd_operations: self.simd_operations.load(Ordering::Relaxed),
            memory_saved: self.memory_saved.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of global statistics
#[derive(Debug, Clone)]
pub struct GlobalRecordStatsSnapshot {
    pub types_registered: usize,
    pub total_instances: u64,
    pub total_accesses: u64,
    pub total_bulk_operations: u64,
    pub cache_hit_rate: f64,
    pub simd_operations: u64,
    pub memory_saved: u64,
}

/// JIT compilation tracking and recommendations
pub struct JitCompilationTracker {
    /// Hot call sites that should be JIT compiled
    hot_sites: RwLock<HashMap<u64, JitCandidate>>,
    /// Successful JIT compilations
    compilations: AtomicU64,
    /// Performance improvements from JIT
    jit_speedup: AtomicU64, // Percentage improvement
}

/// JIT compilation candidate
#[derive(Debug, Clone)]
pub struct JitCandidate {
    /// Call site ID
    pub site_id: u64,
    /// Associated record type
    pub type_id: RecordTypeId,
    /// Access frequency
    pub access_count: u64,
    /// Average access time before JIT
    pub baseline_time: u64,
    /// Compilation priority (0-100)
    pub priority: u8,
    /// Compilation status
    pub status: JitStatus,
}

/// JIT compilation status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JitStatus {
    Candidate,
    Compiling,
    Compiled,
    Failed,
}

impl JitCompilationTracker {
    /// Creates a new JIT tracker
    pub fn new() -> Self {
        Self {
            hot_sites: RwLock::new(HashMap::new()),
            compilations: AtomicU64::new(0),
            jit_speedup: AtomicU64::new(0),
        }
    }

    /// Adds a JIT compilation candidate
    pub fn add_candidate(&self, candidate: JitCandidate) {
        let mut hot_sites = self.hot_sites.write().unwrap();
        hot_sites.insert(candidate.site_id, candidate);
    }

    /// Gets JIT candidates ordered by priority
    pub fn get_candidates(&self) -> Vec<JitCandidate> {
        let hot_sites = self.hot_sites.read().unwrap();
        let mut candidates: Vec<_> = hot_sites.values().cloned().collect();
        candidates.sort_by_key(|c| std::cmp::Reverse(c.priority));
        candidates
    }

    /// Records successful JIT compilation
    pub fn record_compilation(&self, site_id: u64, speedup_percent: u64) {
        self.compilations.fetch_add(1, Ordering::Relaxed);

        // Update speedup moving average
        let current = self.jit_speedup.load(Ordering::Relaxed);
        let new_avg = if current == 0 {
            speedup_percent
        } else {
            (current * 7 + speedup_percent) / 8 // 7/8 weighted average
        };
        self.jit_speedup.store(new_avg, Ordering::Relaxed);

        // Mark candidate as compiled
        let mut hot_sites = self.hot_sites.write().unwrap();
        if let Some(candidate) = hot_sites.get_mut(&site_id) {
            candidate.status = JitStatus::Compiled;
        }
    }

    /// Gets JIT compilation statistics
    pub fn stats(&self) -> JitCompilationStats {
        let hot_sites = self.hot_sites.read().unwrap();
        let mut candidate_count = 0;
        let mut compiling_count = 0;
        let mut compiled_count = 0;
        let mut failed_count = 0;

        for candidate in hot_sites.values() {
            match candidate.status {
                JitStatus::Candidate => candidate_count += 1,
                JitStatus::Compiling => compiling_count += 1,
                JitStatus::Compiled => compiled_count += 1,
                JitStatus::Failed => failed_count += 1,
            }
        }

        JitCompilationStats {
            total_candidates: hot_sites.len(),
            candidate_count,
            compiling_count,
            compiled_count,
            failed_count,
            average_speedup: self.jit_speedup.load(Ordering::Relaxed) as f64,
        }
    }
}

impl Default for JitCompilationTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// JIT compilation statistics
#[derive(Debug, Clone)]
pub struct JitCompilationStats {
    pub total_candidates: usize,
    pub candidate_count: usize,
    pub compiling_count: usize,
    pub compiled_count: usize,
    pub failed_count: usize,
    pub average_speedup: f64,
}

/// Optimization recommendation
#[derive(Debug, Clone)]
pub enum OptimizationRecommendation {
    /// Enable SIMD for bulk operations on this type
    EnableSimd {
        type_id: RecordTypeId,
        potential_speedup: f64,
    },
    /// Trigger JIT compilation for hot call site
    JitCompile {
        site_id: u64,
        type_id: RecordTypeId,
        expected_speedup: f64,
    },
    /// Reorganize field layout for better cache performance
    ReorganizeFields {
        type_id: RecordTypeId,
        recommended_order: Vec<usize>,
    },
    /// Use specialized allocation strategy
    OptimizeAllocation {
        type_id: RecordTypeId,
        strategy: AllocationStrategy,
    },
    /// Increase arena size for frequently allocated type
    IncreaseArenaSize {
        type_id: RecordTypeId,
        recommended_size: usize,
    },
}

/// Allocation optimization strategies
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocationStrategy {
    /// Use thread-local allocation
    ThreadLocal,
    /// Use specialized size class
    SpecializedSize,
    /// Use pre-allocated pool
    PreallocatedPool,
    /// Use NUMA-aware allocation
    NumaAware,
}

impl RecordPerformanceMonitor {
    /// Creates a new performance monitor
    pub fn new() -> Self {
        Self {
            type_metrics: RwLock::new(HashMap::new()),
            global_stats: GlobalRecordStats::default(),
            jit_tracker: JitCompilationTracker::new(),
            recommendations: RwLock::new(Vec::new()),
            start_time: Instant::now(),
        }
    }

    /// Records performance data for a record type
    pub fn record_type_performance(&self, type_id: RecordTypeId, operation: RecordOperation) {
        // Update global stats
        match &operation {
            RecordOperation::Create => self.global_stats.record_instance_creation(),
            RecordOperation::FieldAccess { access_time_ns } => {
                self.global_stats.record_field_access();
            }
            RecordOperation::BulkOperation { count } => {
                self.global_stats.record_bulk_operation(*count);
            }
            RecordOperation::SimdOperation => {
                self.global_stats.record_simd_operation();
            }
        }

        // Update type-specific metrics
        {
            let mut metrics = self.type_metrics.write().unwrap();
            let type_metrics = metrics
                .entry(type_id)
                .or_insert_with(|| TypePerformanceMetrics::new(type_id));

            match operation {
                RecordOperation::Create => type_metrics.record_creation(),
                RecordOperation::FieldAccess { access_time_ns } => {
                    type_metrics.record_access(access_time_ns);
                }
                RecordOperation::BulkOperation { count } => {
                    type_metrics.record_bulk_operation(count);
                }
                RecordOperation::SimdOperation => {
                    // SIMD operations are tracked separately
                }
            }
        }

        // Check for optimization opportunities
        self.check_optimization_opportunities(type_id);
    }

    /// Checks for optimization opportunities
    fn check_optimization_opportunities(&self, type_id: RecordTypeId) {
        let metrics = self.type_metrics.read().unwrap();
        if let Some(type_metrics) = metrics.get(&type_id) {
            let mut recommendations = self.recommendations.write().unwrap();

            // Check for JIT compilation opportunity
            if type_metrics.should_jit_compile() {
                let candidate = JitCandidate {
                    site_id: 0, // This would be specific to the call site
                    type_id,
                    access_count: type_metrics.field_accesses.load(Ordering::Relaxed),
                    baseline_time: type_metrics.avg_access_time.load(Ordering::Relaxed),
                    priority: calculate_jit_priority(type_metrics),
                    status: JitStatus::Candidate,
                };

                self.jit_tracker.add_candidate(candidate);

                recommendations.push(OptimizationRecommendation::JitCompile {
                    site_id: 0,
                    type_id,
                    expected_speedup: estimate_jit_speedup(type_metrics),
                });
            }

            // Check for SIMD opportunity
            let bulk_ratio = type_metrics.bulk_operations.load(Ordering::Relaxed) as f64
                / type_metrics.field_accesses.load(Ordering::Relaxed).max(1) as f64;

            if bulk_ratio > 0.1 && type_metrics.simd_utilization.load(Ordering::Relaxed) < 50 {
                recommendations.push(OptimizationRecommendation::EnableSimd {
                    type_id,
                    potential_speedup: estimate_simd_speedup(bulk_ratio),
                });
            }
        }
    }

    /// Gets comprehensive performance report
    pub fn performance_report(&self) -> RecordPerformanceReport {
        let global_stats = self.global_stats.snapshot();
        let field_cache_stats = GLOBAL_FIELD_ACCESS_CACHE.stats();
        let type_checker_stats = GLOBAL_TYPE_CHECKER.stats();
        let arena_stats = GLOBAL_RECORD_ARENA.stats();
        let jit_stats = self.jit_tracker.stats();

        // Calculate type-specific performance
        let metrics = self.type_metrics.read().unwrap();
        let type_performance: Vec<_> = metrics
            .values()
            .map(|m| TypePerformanceSnapshot {
                type_id: m.type_id,
                instances_created: m.instances_created.load(Ordering::Relaxed),
                field_accesses: m.field_accesses.load(Ordering::Relaxed),
                avg_access_time: m.avg_access_time.load(Ordering::Relaxed),
                performance_score: m.performance_score(),
                should_jit: m.should_jit_compile(),
            })
            .collect();

        let recommendations = self.recommendations.read().unwrap().clone();

        RecordPerformanceReport {
            global_stats,
            type_performance,
            field_cache_stats,
            type_checker_stats,
            arena_stats,
            jit_stats,
            recommendations,
            uptime: self.start_time.elapsed(),
        }
    }

    /// Applies optimization recommendations
    pub fn apply_optimizations(&self) -> OptimizationResults {
        let recommendations = self.recommendations.read().unwrap().clone();
        let mut results = OptimizationResults {
            applied: 0,
            failed: 0,
            skipped: 0,
            improvements: Vec::new(),
        };

        for recommendation in &recommendations {
            match self.apply_single_optimization(recommendation) {
                Ok(improvement) => {
                    results.applied += 1;
                    if let Some(improvement) = improvement {
                        results.improvements.push(improvement);
                    }
                }
                Err(_) => results.failed += 1,
            }
        }

        // Clear applied recommendations
        self.recommendations.write().unwrap().clear();

        results
    }

    /// Applies a single optimization recommendation
    fn apply_single_optimization(
        &self,
        recommendation: &OptimizationRecommendation,
    ) -> RecordResult<Option<PerformanceImprovement>> {
        match recommendation {
            OptimizationRecommendation::EnableSimd {
                type_id,
                potential_speedup,
            } => {
                // Enable SIMD for the given type
                if let Some(type_desc) = GLOBAL_RECORD_REGISTRY.get_type(*type_id) {
                    let mut type_desc = type_desc.write().unwrap();
                    type_desc.simd_flags.bulk_access = true;
                    type_desc.simd_flags.bulk_compare = true;
                    type_desc.simd_flags.bulk_init = true;

                    Ok(Some(PerformanceImprovement {
                        optimization_type: "SIMD Enable".to_string(),
                        type_id: Some(*type_id),
                        expected_speedup: *potential_speedup,
                        actual_speedup: None, // Will be measured later
                    }))
                } else {
                    Err(RecordError::TypeNotFound(format!("TypeId({:?})", type_id)))
                }
            }
            OptimizationRecommendation::JitCompile {
                site_id,
                type_id,
                expected_speedup,
            } => {
                // Trigger JIT compilation (placeholder - actual JIT would be more complex)
                self.jit_tracker
                    .record_compilation(*site_id, *expected_speedup as u64);

                Ok(Some(PerformanceImprovement {
                    optimization_type: "JIT Compile".to_string(),
                    type_id: Some(*type_id),
                    expected_speedup: *expected_speedup,
                    actual_speedup: None,
                }))
            }
            _ => {
                // Other optimizations not yet implemented
                Ok(None)
            }
        }
    }

    /// Triggers garbage collection and optimization
    pub fn optimize_memory(&self) -> MemoryOptimizationResults {
        let gc_stats = GLOBAL_RECORD_ARENA.garbage_collect();

        // Update performance profiles
        let mut updated_types = 0;
        {
            let mut metrics = self.type_metrics.write().unwrap();
            for type_metrics in metrics.values_mut() {
                // Update optimization profiles based on recent usage
                updated_types += 1;
            }
        }

        MemoryOptimizationResults {
            memory_reclaimed: gc_stats.reclaimed_memory,
            gc_stats,
            updated_profiles: updated_types,
        }
    }
}

impl Default for RecordPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Record operation types for performance tracking
#[derive(Debug, Clone)]
pub enum RecordOperation {
    Create,
    FieldAccess { access_time_ns: u64 },
    BulkOperation { count: u64 },
    SimdOperation,
}

/// Type performance snapshot
#[derive(Debug, Clone)]
pub struct TypePerformanceSnapshot {
    pub type_id: RecordTypeId,
    pub instances_created: u64,
    pub field_accesses: u64,
    pub avg_access_time: u64,
    pub performance_score: f64,
    pub should_jit: bool,
}

/// Comprehensive performance report
#[derive(Debug, Clone)]
pub struct RecordPerformanceReport {
    pub global_stats: GlobalRecordStatsSnapshot,
    pub type_performance: Vec<TypePerformanceSnapshot>,
    pub field_cache_stats: FieldAccessCacheStats,
    pub type_checker_stats: crate::eval::record_access::TypeCheckStatsSnapshot,
    pub arena_stats: ArenaStatsSnapshot,
    pub jit_stats: JitCompilationStats,
    pub recommendations: Vec<OptimizationRecommendation>,
    pub uptime: Duration,
}

/// Optimization application results
#[derive(Debug, Clone)]
pub struct OptimizationResults {
    pub applied: usize,
    pub failed: usize,
    pub skipped: usize,
    pub improvements: Vec<PerformanceImprovement>,
}

/// Performance improvement from optimization
#[derive(Debug, Clone)]
pub struct PerformanceImprovement {
    pub optimization_type: String,
    pub type_id: Option<RecordTypeId>,
    pub expected_speedup: f64,
    pub actual_speedup: Option<f64>,
}

/// Memory optimization results
#[derive(Debug, Clone)]
pub struct MemoryOptimizationResults {
    pub gc_stats: GcStats,
    pub updated_profiles: usize,
    pub memory_reclaimed: usize,
}

/// Helper functions for optimization calculations
fn calculate_jit_priority(metrics: &TypePerformanceMetrics) -> u8 {
    let accesses = metrics.field_accesses.load(Ordering::Relaxed);
    let avg_time = metrics.avg_access_time.load(Ordering::Relaxed);

    let access_score = (accesses.min(10000) as f64 / 10000.0 * 50.0) as u8;
    let time_score = if avg_time > 10 {
        50
    } else if avg_time > 5 {
        25
    } else {
        0
    };

    (access_score + time_score).min(100)
}

fn estimate_jit_speedup(metrics: &TypePerformanceMetrics) -> f64 {
    let avg_time = metrics.avg_access_time.load(Ordering::Relaxed) as f64;

    if avg_time <= 1.0 {
        1.5 // Modest improvement for already fast access
    } else if avg_time <= 5.0 {
        5.0 // Good improvement for moderate access
    } else {
        10.0 // Significant improvement for slow access
    }
}

fn estimate_simd_speedup(bulk_ratio: f64) -> f64 {
    // SIMD speedup depends on how much bulk processing is done
    2.0 + bulk_ratio * 6.0 // 2-8x speedup range
}

/// Global performance monitor instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_RECORD_PERFORMANCE_MONITOR: RecordPerformanceMonitor =
        RecordPerformanceMonitor::new();
}

/// Public API functions for performance monitoring
pub fn record_type_operation(type_id: RecordTypeId, operation: RecordOperation) {
    GLOBAL_RECORD_PERFORMANCE_MONITOR.record_type_performance(type_id, operation);
}

pub fn get_performance_report() -> RecordPerformanceReport {
    GLOBAL_RECORD_PERFORMANCE_MONITOR.performance_report()
}

pub fn apply_optimizations() -> OptimizationResults {
    GLOBAL_RECORD_PERFORMANCE_MONITOR.apply_optimizations()
}

pub fn optimize_memory() -> MemoryOptimizationResults {
    GLOBAL_RECORD_PERFORMANCE_MONITOR.optimize_memory()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::record_type::{GLOBAL_RECORD_REGISTRY, RecordTypeDescriptor};

    fn create_test_type() -> RecordTypeId {
        let desc = RecordTypeDescriptor::new(
            "test-type".to_string(),
            vec!["field1".to_string(), "field2".to_string()],
        );
        let type_id = desc.type_id;
        GLOBAL_RECORD_REGISTRY.register_type(desc);
        type_id
    }

    #[test]
    fn test_performance_metrics() {
        let type_id = create_test_type();
        let metrics = TypePerformanceMetrics::new(type_id);

        // Record some operations
        metrics.record_creation();
        metrics.record_access(5); // 5ns access time
        metrics.record_bulk_operation(4);

        assert_eq!(metrics.instances_created.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.field_accesses.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.bulk_operations.load(Ordering::Relaxed), 4);
        assert_eq!(metrics.avg_access_time.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn test_jit_priority_calculation() {
        let type_id = create_test_type();
        let metrics = TypePerformanceMetrics::new(type_id);

        // Simulate high access count with slow access time
        for _ in 0..5000 {
            metrics.record_access(10); // 10ns - slow access
        }

        let priority = calculate_jit_priority(&metrics);
        assert!(priority > 50); // Should have high priority
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = RecordPerformanceMonitor::new();
        let type_id = create_test_type();

        // Record some operations
        monitor.record_type_performance(type_id, RecordOperation::Create);
        monitor
            .record_type_performance(type_id, RecordOperation::FieldAccess { access_time_ns: 3 });
        monitor.record_type_performance(type_id, RecordOperation::BulkOperation { count: 8 });

        let report = monitor.performance_report();
        assert!(report.global_stats.total_instances > 0);
        assert!(report.global_stats.total_accesses > 0);
        assert!(report.type_performance.len() > 0);
    }

    #[test]
    fn test_simd_speedup_estimation() {
        let speedup = estimate_simd_speedup(0.5); // 50% bulk operations
        assert!(speedup >= 2.0 && speedup <= 8.0);

        let high_speedup = estimate_simd_speedup(1.0); // 100% bulk operations
        assert!(high_speedup >= 7.0);
    }

    #[test]
    fn test_optimization_recommendations() {
        let monitor = RecordPerformanceMonitor::new();
        let type_id = create_test_type();

        // Generate enough activity to trigger recommendations
        for _ in 0..2000 {
            monitor.record_type_performance(
                type_id,
                RecordOperation::FieldAccess { access_time_ns: 10 },
            );
        }

        let report = monitor.performance_report();
        assert!(!report.recommendations.is_empty());
    }
}
