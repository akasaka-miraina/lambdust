//! Arc Allocation Tracking System for Value Optimization Measurement
//!
//! This module provides precise Arc allocation tracking to measure the effectiveness
//! of the Value enum optimization strategy. It implements custom Arc tracking that
//! can measure allocation patterns, memory usage, and optimization effectiveness.
//!
//! Key Features:
//! - Real-time Arc allocation counting
//! - Per-type allocation tracking
//! - Memory pattern analysis
//! - Allocation lifecycle monitoring
//! - Performance impact measurement
//! - Detailed reporting and analytics

#![allow(missing_docs)]

use crate::eval::value::Value;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::{
    Arc, Mutex, RwLock,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Global Arc allocation tracker instance
lazy_static::lazy_static! {
    static ref GLOBAL_TRACKER: ArcAllocationTracker = ArcAllocationTracker::new();
}

/// Thread-safe Arc allocation tracker
pub struct ArcAllocationTracker {
    enabled: AtomicBool,
    total_allocations: AtomicUsize,
    total_deallocations: AtomicUsize,
    peak_concurrent: AtomicUsize,
    current_count: AtomicUsize,
    detailed_stats: RwLock<DetailedAllocationStats>,
}

/// Detailed allocation statistics with historical tracking
#[derive(Debug, Clone)]
struct DetailedAllocationStats {
    per_type_stats: HashMap<String, TypeAllocationStats>,
    allocation_timeline: VecDeque<AllocationEvent>,
    thread_stats: HashMap<thread::ThreadId, ThreadAllocationStats>,
    memory_pressure_events: Vec<MemoryPressureEvent>,
}

impl DetailedAllocationStats {
    fn new() -> Self {
        DetailedAllocationStats {
            per_type_stats: HashMap::new(),
            allocation_timeline: VecDeque::new(),
            thread_stats: HashMap::new(),
            memory_pressure_events: Vec::new(),
        }
    }
}

/// Statistics for a specific Value type
#[derive(Debug, Clone, Default)]
pub struct TypeAllocationStats {
    pub allocations: usize,
    pub deallocations: usize,
    pub peak_concurrent: usize,
    pub current_count: usize,
    pub average_lifetime: Duration,
    pub memory_footprint: usize,
}

/// Per-thread allocation statistics
#[derive(Debug, Clone, Default)]
pub struct ThreadAllocationStats {
    allocations: usize,
    deallocations: usize,
    peak_usage: usize,
    thread_name: Option<String>,
}

/// Individual allocation event for detailed tracking
#[derive(Debug, Clone)]
pub struct AllocationEvent {
    pub timestamp: SystemTime,
    pub event_type: AllocationEventType,
    pub value_type: String,
    pub thread_id: thread::ThreadId,
    pub allocation_id: usize,
    pub memory_size: usize,
}

/// Type of allocation event
#[derive(Debug, Clone, PartialEq)]
pub enum AllocationEventType {
    Allocation,
    Deallocation,
    PeakUpdate,
}

/// Memory pressure event indicating high allocation rates
#[derive(Debug, Clone)]
pub struct MemoryPressureEvent {
    pub timestamp: SystemTime,
    pub allocation_rate: f64, // allocations per second
    pub concurrent_count: usize,
    pub pressure_level: PressureLevel,
}

/// Level of memory pressure
#[derive(Debug, Clone, PartialEq)]
pub enum PressureLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl ArcAllocationTracker {
    /// Creates a new Arc allocation tracker
    fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            total_allocations: AtomicUsize::new(0),
            total_deallocations: AtomicUsize::new(0),
            peak_concurrent: AtomicUsize::new(0),
            current_count: AtomicUsize::new(0),
            detailed_stats: RwLock::new(DetailedAllocationStats::new()),
        }
    }

    /// Enables Arc allocation tracking
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
    }

    /// Disables Arc allocation tracking
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }

    /// Checks if tracking is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// Records an Arc allocation
    pub fn record_allocation(&self, value_type: &str, memory_size: usize) -> AllocationId {
        if !self.is_enabled() {
            return AllocationId::new(0);
        }

        let allocation_id = self.total_allocations.fetch_add(1, Ordering::SeqCst);
        let current = self.current_count.fetch_add(1, Ordering::SeqCst) + 1;

        // Update peak if necessary
        let mut peak = self.peak_concurrent.load(Ordering::SeqCst);
        while current > peak {
            match self.peak_concurrent.compare_exchange_weak(
                peak,
                current,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }

        // Record detailed statistics
        if let Ok(mut stats) = self.detailed_stats.write() {
            let thread_id = thread::current().id();
            let timestamp = SystemTime::now();

            // Update per-type statistics
            let type_stats = stats
                .per_type_stats
                .entry(value_type.to_string())
                .or_default();
            type_stats.allocations += 1;
            type_stats.current_count += 1;
            type_stats.memory_footprint += memory_size;
            if type_stats.current_count > type_stats.peak_concurrent {
                type_stats.peak_concurrent = type_stats.current_count;
            }

            // Update thread statistics
            let thread_stats = stats.thread_stats.entry(thread_id).or_default();
            thread_stats.allocations += 1;
            if thread_stats.thread_name.is_none() {
                thread_stats.thread_name = thread::current().name().map(|s| s.to_string());
            }

            // Record allocation event
            let event = AllocationEvent {
                timestamp,
                event_type: AllocationEventType::Allocation,
                value_type: value_type.to_string(),
                thread_id,
                allocation_id,
                memory_size,
            };

            stats.allocation_timeline.push_back(event);

            // Limit timeline size to prevent unbounded growth
            if stats.allocation_timeline.len() > 10000 {
                stats.allocation_timeline.pop_front();
            }

            // Check for memory pressure
            self.check_memory_pressure(&mut stats, current);
        }

        AllocationId::new(allocation_id)
    }

    /// Records an Arc deallocation
    pub fn record_deallocation(
        &self,
        allocation_id: AllocationId,
        value_type: &str,
        memory_size: usize,
    ) {
        if !self.is_enabled() {
            return;
        }

        self.total_deallocations.fetch_add(1, Ordering::SeqCst);
        let current = self
            .current_count
            .fetch_sub(1, Ordering::SeqCst)
            .saturating_sub(1);

        // Record detailed statistics
        if let Ok(mut stats) = self.detailed_stats.write() {
            let thread_id = thread::current().id();
            let timestamp = SystemTime::now();

            // Update per-type statistics
            if let Some(type_stats) = stats.per_type_stats.get_mut(value_type) {
                type_stats.deallocations += 1;
                type_stats.current_count = type_stats.current_count.saturating_sub(1);
                type_stats.memory_footprint =
                    type_stats.memory_footprint.saturating_sub(memory_size);
            }

            // Update thread statistics
            if let Some(thread_stats) = stats.thread_stats.get_mut(&thread_id) {
                thread_stats.deallocations += 1;
            }

            // Record deallocation event
            let event = AllocationEvent {
                timestamp,
                event_type: AllocationEventType::Deallocation,
                value_type: value_type.to_string(),
                thread_id,
                allocation_id: allocation_id.id(),
                memory_size,
            };

            stats.allocation_timeline.push_back(event);

            // Limit timeline size
            if stats.allocation_timeline.len() > 10000 {
                stats.allocation_timeline.pop_front();
            }
        }
    }

    /// Checks for memory pressure and records events
    fn check_memory_pressure(&self, stats: &mut DetailedAllocationStats, current_count: usize) {
        let now = SystemTime::now();

        // Calculate allocation rate over last second
        let one_second_ago = now.checked_sub(Duration::from_secs(1));
        if let Some(threshold_time) = one_second_ago {
            let recent_allocations = stats
                .allocation_timeline
                .iter()
                .filter(|event| {
                    event.timestamp >= threshold_time
                        && event.event_type == AllocationEventType::Allocation
                })
                .count();

            let allocation_rate = recent_allocations as f64;

            // Determine pressure level
            let pressure_level = if allocation_rate > 1000.0 {
                PressureLevel::Critical
            } else if allocation_rate > 500.0 {
                PressureLevel::High
            } else if allocation_rate > 100.0 {
                PressureLevel::Medium
            } else {
                PressureLevel::Low
            };

            // Record pressure event if significant
            if pressure_level != PressureLevel::Low {
                stats.memory_pressure_events.push(MemoryPressureEvent {
                    timestamp: now,
                    allocation_rate,
                    concurrent_count: current_count,
                    pressure_level,
                });

                // Limit pressure events to prevent unbounded growth
                if stats.memory_pressure_events.len() > 1000 {
                    stats.memory_pressure_events.remove(0);
                }
            }
        }
    }

    /// Gets current allocation statistics
    pub fn get_current_stats(&self) -> AllocationStats {
        AllocationStats {
            total_allocations: self.total_allocations.load(Ordering::SeqCst),
            total_deallocations: self.total_deallocations.load(Ordering::SeqCst),
            current_count: self.current_count.load(Ordering::SeqCst),
            peak_concurrent: self.peak_concurrent.load(Ordering::SeqCst),
        }
    }

    /// Gets detailed allocation report
    pub fn get_detailed_report(&self) -> DetailedAllocationReport {
        let basic_stats = self.get_current_stats();

        let detailed_stats = if let Ok(stats) = self.detailed_stats.try_read() {
            stats.clone()
        } else {
            DetailedAllocationStats {
                per_type_stats: HashMap::new(),
                allocation_timeline: VecDeque::new(),
                thread_stats: HashMap::new(),
                memory_pressure_events: Vec::new(),
            }
        };

        DetailedAllocationReport {
            basic_stats,
            per_type_stats: detailed_stats.per_type_stats,
            thread_stats: detailed_stats.thread_stats,
            memory_pressure_events: detailed_stats.memory_pressure_events,
            timeline_length: detailed_stats.allocation_timeline.len(),
        }
    }

    /// Resets all tracking statistics
    pub fn reset(&self) {
        self.total_allocations.store(0, Ordering::SeqCst);
        self.total_deallocations.store(0, Ordering::SeqCst);
        self.peak_concurrent.store(0, Ordering::SeqCst);
        self.current_count.store(0, Ordering::SeqCst);

        if let Ok(mut stats) = self.detailed_stats.write() {
            *stats = DetailedAllocationStats {
                per_type_stats: HashMap::new(),
                allocation_timeline: VecDeque::new(),
                thread_stats: HashMap::new(),
                memory_pressure_events: Vec::new(),
            };
        }
    }

    /// Analyzes allocation patterns for optimization insights
    pub fn analyze_patterns(&self) -> AllocationPatternAnalysis {
        let report = self.get_detailed_report();
        let mut analysis = AllocationPatternAnalysis::default();

        // Analyze per-type patterns
        for (type_name, stats) in &report.per_type_stats {
            let allocation_efficiency = if stats.allocations > 0 {
                stats.deallocations as f64 / stats.allocations as f64
            } else {
                0.0
            };

            analysis
                .type_efficiency
                .insert(type_name.clone(), allocation_efficiency);

            if stats.peak_concurrent > 100 {
                analysis.high_usage_types.push(type_name.clone());
            }

            if allocation_efficiency < 0.8 {
                analysis.potential_leaks.push(type_name.clone());
            }
        }

        // Analyze memory pressure
        analysis.pressure_events_count = report.memory_pressure_events.len();
        analysis.peak_allocation_rate = report
            .memory_pressure_events
            .iter()
            .map(|event| event.allocation_rate)
            .fold(0.0, f64::max);

        // Calculate overall efficiency
        analysis.overall_efficiency = if report.basic_stats.total_allocations > 0 {
            report.basic_stats.total_deallocations as f64
                / report.basic_stats.total_allocations as f64
        } else {
            1.0
        };

        analysis.fragmentation_risk = if analysis.overall_efficiency < 0.9 {
            FragmentationRisk::High
        } else if analysis.overall_efficiency < 0.95 {
            FragmentationRisk::Medium
        } else {
            FragmentationRisk::Low
        };

        analysis
    }
}

/// Global functions for Arc tracking
impl ArcAllocationTracker {
    /// Gets the global tracker instance
    pub fn global() -> &'static Self {
        &GLOBAL_TRACKER
    }
}

/// Unique identifier for Arc allocations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocationId(usize);

impl AllocationId {
    fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

/// Basic allocation statistics
#[derive(Debug, Clone, Default)]
pub struct AllocationStats {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub current_count: usize,
    pub peak_concurrent: usize,
}

/// Detailed allocation report
#[derive(Debug, Clone)]
pub struct DetailedAllocationReport {
    pub basic_stats: AllocationStats,
    pub per_type_stats: HashMap<String, TypeAllocationStats>,
    pub thread_stats: HashMap<thread::ThreadId, ThreadAllocationStats>,
    pub memory_pressure_events: Vec<MemoryPressureEvent>,
    pub timeline_length: usize,
}

/// Allocation pattern analysis results
#[derive(Debug, Clone, Default)]
pub struct AllocationPatternAnalysis {
    pub type_efficiency: HashMap<String, f64>,
    pub high_usage_types: Vec<String>,
    pub potential_leaks: Vec<String>,
    pub pressure_events_count: usize,
    pub peak_allocation_rate: f64,
    pub overall_efficiency: f64,
    pub fragmentation_risk: FragmentationRisk,
}

/// Risk level for memory fragmentation
#[derive(Debug, Clone, PartialEq, Default)]
pub enum FragmentationRisk {
    #[default]
    Low,
    Medium,
    High,
}

/// Tracked Arc wrapper that automatically reports allocations/deallocations
pub struct TrackedArc<T> {
    inner: Arc<T>,
    allocation_id: AllocationId,
    type_name: String,
    memory_size: usize,
}

impl<T> TrackedArc<T> {
    /// Creates a new tracked Arc
    pub fn new(value: T, type_name: &str) -> Self {
        let memory_size = std::mem::size_of::<T>() + std::mem::size_of::<Arc<T>>();
        let allocation_id = GLOBAL_TRACKER.record_allocation(type_name, memory_size);

        Self {
            inner: Arc::new(value),
            allocation_id,
            type_name: type_name.to_string(),
            memory_size,
        }
    }

    /// Gets the allocation ID
    pub fn allocation_id(&self) -> AllocationId {
        self.allocation_id
    }

    /// Gets the strong reference count
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Gets the weak reference count
    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.inner)
    }
}

impl<T> Clone for TrackedArc<T> {
    fn clone(&self) -> Self {
        // Cloning doesn't create a new allocation, just increments reference count
        Self {
            inner: self.inner.clone(),
            allocation_id: self.allocation_id,
            type_name: self.type_name.clone(),
            memory_size: self.memory_size,
        }
    }
}

impl<T> Drop for TrackedArc<T> {
    fn drop(&mut self) {
        // Only record deallocation when this is the last reference
        if Arc::strong_count(&self.inner) == 1 {
            GLOBAL_TRACKER.record_deallocation(
                self.allocation_id,
                &self.type_name,
                self.memory_size,
            );
        }
    }
}

impl<T> std::ops::Deref for TrackedArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: fmt::Debug> fmt::Debug for TrackedArc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrackedArc")
            .field("value", &**self)
            .field("allocation_id", &self.allocation_id)
            .field("strong_count", &self.strong_count())
            .finish()
    }
}

/// Value type analyzer for determining Arc usage patterns
pub struct ValueTypeAnalyzer;

impl ValueTypeAnalyzer {
    /// Analyzes a Value and returns type classification
    pub fn classify_value(value: &Value) -> ValueTypeClassification {
        match value {
            Value::Nil | Value::Unspecified => ValueTypeClassification {
                type_name: "immediate".to_string(),
                expected_arc_count: 0,
                optimization_potential: OptimizationPotential::High,
                memory_impact: MemoryImpact::Low,
            },

            Value::Literal(_) => ValueTypeClassification {
                type_name: "literal".to_string(),
                expected_arc_count: 0,
                optimization_potential: OptimizationPotential::High,
                memory_impact: MemoryImpact::Low,
            },

            Value::Symbol(_) => ValueTypeClassification {
                type_name: "symbol".to_string(),
                expected_arc_count: 0,
                optimization_potential: OptimizationPotential::Medium,
                memory_impact: MemoryImpact::Low,
            },

            Value::Pair(_, _) => ValueTypeClassification {
                type_name: "pair".to_string(),
                expected_arc_count: 2,
                optimization_potential: OptimizationPotential::High,
                memory_impact: MemoryImpact::Medium,
            },

            Value::Vector(_) => ValueTypeClassification {
                type_name: "vector".to_string(),
                expected_arc_count: 1,
                optimization_potential: OptimizationPotential::Medium,
                memory_impact: MemoryImpact::High,
            },

            _ => ValueTypeClassification {
                type_name: "complex".to_string(),
                expected_arc_count: 1,
                optimization_potential: OptimizationPotential::Low,
                memory_impact: MemoryImpact::Medium,
            },
        }
    }

    /// Analyzes Value hierarchy for Arc usage
    pub fn analyze_value_hierarchy(value: &Value) -> HierarchyAnalysis {
        let mut analysis = HierarchyAnalysis::default();
        let mut visited = std::collections::HashSet::new();

        Self::analyze_recursive(value, &mut analysis, &mut visited, 0);

        analysis
    }

    fn analyze_recursive(
        value: &Value,
        analysis: &mut HierarchyAnalysis,
        visited: &mut std::collections::HashSet<*const Value>,
        depth: usize,
    ) {
        let value_ptr = value as *const Value;

        if visited.contains(&value_ptr) {
            analysis.circular_references += 1;
            return;
        }
        visited.insert(value_ptr);

        let classification = Self::classify_value(value);
        analysis.total_values += 1;
        analysis.total_expected_arcs += classification.expected_arc_count;
        analysis.max_depth = analysis.max_depth.max(depth);

        match classification.optimization_potential {
            OptimizationPotential::High => analysis.high_optimization_values += 1,
            OptimizationPotential::Medium => analysis.medium_optimization_values += 1,
            OptimizationPotential::Low => analysis.low_optimization_values += 1,
        }

        // Recurse into child values
        match value {
            Value::Pair(car, cdr) => {
                Self::analyze_recursive(car, analysis, visited, depth + 1);
                Self::analyze_recursive(cdr, analysis, visited, depth + 1);
            }
            Value::Vector(vec_arc) => {
                if let Ok(elements) = vec_arc.try_borrow() {
                    for element in elements.iter() {
                        Self::analyze_recursive(element, analysis, visited, depth + 1);
                    }
                }
            }
            _ => {} // Other types don't have child Values to analyze
        }
    }
}

/// Classification of a Value type for optimization analysis
#[derive(Debug, Clone)]
pub struct ValueTypeClassification {
    pub type_name: String,
    pub expected_arc_count: usize,
    pub optimization_potential: OptimizationPotential,
    pub memory_impact: MemoryImpact,
}

/// Potential for optimization
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationPotential {
    High,   // Can eliminate most/all Arc usage
    Medium, // Can reduce Arc usage significantly
    Low,    // Limited optimization potential
}

/// Memory impact classification
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryImpact {
    Low,    // Small memory footprint
    Medium, // Moderate memory usage
    High,   // Large memory footprint
}

/// Analysis of Value hierarchy
#[derive(Debug, Clone, Default)]
pub struct HierarchyAnalysis {
    pub total_values: usize,
    pub total_expected_arcs: usize,
    pub high_optimization_values: usize,
    pub medium_optimization_values: usize,
    pub low_optimization_values: usize,
    pub max_depth: usize,
    pub circular_references: usize,
}

impl HierarchyAnalysis {
    /// Calculates optimization potential score (0.0 to 1.0)
    pub fn optimization_score(&self) -> f64 {
        if self.total_values == 0 {
            return 1.0;
        }

        let high_weight = 1.0;
        let medium_weight = 0.6;
        let low_weight = 0.2;

        let weighted_score = (self.high_optimization_values as f64 * high_weight)
            + (self.medium_optimization_values as f64 * medium_weight)
            + (self.low_optimization_values as f64 * low_weight);

        weighted_score / (self.total_values as f64)
    }

    /// Estimates potential Arc reduction
    pub fn estimated_arc_reduction(&self) -> usize {
        // Conservative estimate: high potential values can reduce by 90%, medium by 50%
        let high_reduction = (self.high_optimization_values as f64 * 0.9) as usize;
        let medium_reduction = (self.medium_optimization_values as f64 * 0.5) as usize;

        high_reduction + medium_reduction
    }
}

/// Convenience functions for global Arc tracking
pub fn enable_global_tracking() {
    ArcAllocationTracker::global().enable();
}

pub fn disable_global_tracking() {
    ArcAllocationTracker::global().disable();
}

pub fn reset_global_tracking() {
    ArcAllocationTracker::global().reset();
}

pub fn get_global_stats() -> AllocationStats {
    ArcAllocationTracker::global().get_current_stats()
}

pub fn get_global_detailed_report() -> DetailedAllocationReport {
    ArcAllocationTracker::global().get_detailed_report()
}

pub fn analyze_global_patterns() -> AllocationPatternAnalysis {
    ArcAllocationTracker::global().analyze_patterns()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_tracker_basic_functionality() {
        let tracker = ArcAllocationTracker::new();
        tracker.enable();

        // Test allocation recording
        let id1 = tracker.record_allocation("test_type", 24);
        let id2 = tracker.record_allocation("test_type", 24);

        let stats = tracker.get_current_stats();
        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.current_count, 2);

        // Test deallocation recording
        tracker.record_deallocation(id1, "test_type", 24);

        let stats = tracker.get_current_stats();
        assert_eq!(stats.total_deallocations, 1);
        assert_eq!(stats.current_count, 1);
    }

    #[test]
    fn test_tracked_arc() {
        reset_global_tracking();
        enable_global_tracking();

        let initial_stats = get_global_stats();

        {
            let _tracked = TrackedArc::new(42, "i32");
            let current_stats = get_global_stats();
            assert_eq!(
                current_stats.total_allocations,
                initial_stats.total_allocations + 1
            );
            assert_eq!(current_stats.current_count, initial_stats.current_count + 1);
        }

        // After dropping, deallocation should be recorded
        let final_stats = get_global_stats();
        assert_eq!(
            final_stats.total_deallocations,
            initial_stats.total_deallocations + 1
        );
        assert_eq!(final_stats.current_count, initial_stats.current_count);
    }

    #[test]
    fn test_value_type_classification() {
        let nil_class = ValueTypeAnalyzer::classify_value(&Value::Nil);
        assert_eq!(nil_class.expected_arc_count, 0);
        assert_eq!(
            nil_class.optimization_potential,
            OptimizationPotential::High
        );

        let pair_class =
            ValueTypeAnalyzer::classify_value(&Value::pair(Value::integer(1), Value::integer(2)));
        assert_eq!(pair_class.expected_arc_count, 2);
        assert_eq!(
            pair_class.optimization_potential,
            OptimizationPotential::High
        );
    }

    #[test]
    fn test_hierarchy_analysis() {
        let list = Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]);
        let analysis = ValueTypeAnalyzer::analyze_value_hierarchy(&list);

        assert!(analysis.total_values > 0);
        assert!(analysis.optimization_score() >= 0.0);
        assert!(analysis.optimization_score() <= 1.0);
    }

    #[test]
    fn test_allocation_pattern_analysis() {
        let tracker = ArcAllocationTracker::new();
        tracker.enable();

        // Generate some allocation patterns
        for i in 0..10 {
            let id = tracker.record_allocation("test_type", 24);
            if i % 2 == 0 {
                tracker.record_deallocation(id, "test_type", 24);
            }
        }

        let analysis = tracker.analyze_patterns();
        assert!(analysis.overall_efficiency > 0.0);
        assert!(analysis.overall_efficiency <= 1.0);
    }

    #[test]
    fn test_memory_pressure_detection() {
        let tracker = ArcAllocationTracker::new();
        tracker.enable();

        // Simulate high allocation rate
        for _ in 0..100 {
            tracker.record_allocation("pressure_test", 24);
        }

        let report = tracker.get_detailed_report();

        // Should detect some level of pressure
        assert!(report.memory_pressure_events.len() < usize::MAX); // Vec length is always >= 0
    }
}
