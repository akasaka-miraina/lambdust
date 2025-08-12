//! Performance monitoring and metrics for JIT compilation
//!
//! This module provides comprehensive performance tracking for the JIT system,
//! including execution counters, timing measurements, compilation statistics,
//! and performance trend analysis.

use crate::jit::compilation_tiers::CompilationTier;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Main JIT performance metrics collector
#[derive(Debug)]
pub struct JitMetrics {
    /// General execution statistics
    execution_stats: ExecutionStats,
    /// Compilation statistics by tier
    compilation_stats: HashMap<CompilationTier, CompilationStats>,
    /// Performance counters
    performance_counters: PerformanceCounters,
    /// Timing measurements
    timing_measurements: TimingMeasurements,
    /// Memory usage statistics
    memory_stats: MemoryStats,
    /// Cache statistics
    cache_stats: CacheStats,
    /// Start time for overall metrics collection
    start_time: Instant,
}

impl Default for JitMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl JitMetrics {
    /// Creates a new metrics collector
    pub fn new() -> Self {
        let mut compilation_stats = HashMap::new();
        compilation_stats.insert(CompilationTier::Interpreter, CompilationStats::new());
        compilation_stats.insert(CompilationTier::Bytecode, CompilationStats::new());
        compilation_stats.insert(CompilationTier::JitBasic, CompilationStats::new());
        compilation_stats.insert(CompilationTier::JitOptimized, CompilationStats::new());

        JitMetrics {
            execution_stats: ExecutionStats::new(),
            compilation_stats,
            performance_counters: PerformanceCounters::new(),
            timing_measurements: TimingMeasurements::new(),
            memory_stats: MemoryStats::new(),
            cache_stats: CacheStats::new(),
            start_time: Instant::now(),
        }
    }

    /// Records execution of a function
    pub fn record_execution(&mut self, execution_time: Duration) {
        self.execution_stats.record_execution(execution_time);
        self.timing_measurements.record_execution_time(execution_time);
    }

    /// Records a compilation event
    pub fn record_compilation(&mut self, compilation_time: Duration, tier: CompilationTier) {
        if let Some(stats) = self.compilation_stats.get_mut(&tier) {
            stats.record_compilation(compilation_time);
        }
        self.timing_measurements.record_compilation_time(tier, compilation_time);
        self.performance_counters.increment_compilations();
    }

    /// Records a compilation failure
    pub fn record_compilation_failure(&mut self, tier: CompilationTier) {
        if let Some(stats) = self.compilation_stats.get_mut(&tier) {
            stats.record_failure();
        }
        self.performance_counters.increment_compilation_failures();
    }

    /// Records cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_stats.record_hit();
        self.performance_counters.increment_cache_hits();
    }

    /// Records cache miss
    pub fn record_cache_miss(&mut self) {
        self.cache_stats.record_miss();
        self.performance_counters.increment_cache_misses();
    }

    /// Records memory allocation
    pub fn record_memory_allocation(&mut self, size: usize) {
        self.memory_stats.record_allocation(size);
    }

    /// Records memory deallocation
    pub fn record_memory_deallocation(&mut self, size: usize) {
        self.memory_stats.record_deallocation(size);
    }

    /// Gets execution statistics
    pub fn execution_stats(&self) -> &ExecutionStats {
        &self.execution_stats
    }

    /// Gets compilation statistics for a tier
    pub fn compilation_stats(&self, tier: CompilationTier) -> Option<&CompilationStats> {
        self.compilation_stats.get(&tier)
    }

    /// Gets performance counters
    pub fn performance_counters(&self) -> &PerformanceCounters {
        &self.performance_counters
    }

    /// Gets timing measurements
    pub fn timing_measurements(&self) -> &TimingMeasurements {
        &self.timing_measurements
    }

    /// Gets memory statistics
    pub fn memory_stats(&self) -> &MemoryStats {
        &self.memory_stats
    }

    /// Gets cache statistics
    pub fn cache_stats(&self) -> &CacheStats {
        &self.cache_stats
    }

    /// Gets total executions across all tiers
    pub fn total_executions(&self) -> u64 {
        self.execution_stats.total_executions
    }

    /// Gets total compilation time across all tiers
    pub fn total_compilation_time(&self) -> Duration {
        self.compilation_stats
            .values()
            .map(|stats| stats.total_compilation_time)
            .sum()
    }

    /// Gets uptime since metrics collection started
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Calculates overall performance summary
    pub fn performance_summary(&self) -> PerformanceSummary {
        let uptime = self.uptime();
        let total_executions = self.total_executions();
        let total_compilation_time = self.total_compilation_time();
        
        let executions_per_second = if uptime.as_secs() > 0 {
            total_executions as f64 / uptime.as_secs_f64()
        } else {
            0.0
        };

        let average_execution_time = if total_executions > 0 {
            self.execution_stats.total_execution_time / total_executions as u32
        } else {
            Duration::ZERO
        };

        let cache_hit_rate = self.cache_stats.hit_rate();

        PerformanceSummary {
            uptime,
            total_executions,
            executions_per_second,
            average_execution_time,
            total_compilation_time,
            cache_hit_rate,
            memory_usage: self.memory_stats.current_usage(),
        }
    }

    /// Generates performance report
    pub fn generate_report(&self) -> String {
        let summary = self.performance_summary();
        
        let mut report = String::new();
        report.push_str("=== JIT Performance Report ===\n");
        report.push_str(&format!("Uptime: {:.2}s\n", summary.uptime.as_secs_f64()));
        report.push_str(&format!("Total Executions: {}\n", summary.total_executions));
        report.push_str(&format!("Executions/sec: {:.2}\n", summary.executions_per_second));
        report.push_str(&format!("Avg Execution Time: {:.2}μs\n", 
            summary.average_execution_time.as_micros()));
        report.push_str(&format!("Total Compilation Time: {:.2}ms\n", 
            summary.total_compilation_time.as_millis()));
        report.push_str(&format!("Cache Hit Rate: {:.2}%\n", summary.cache_hit_rate * 100.0));
        report.push_str(&format!("Memory Usage: {} bytes\n", summary.memory_usage));
        
        report.push_str("\n=== Compilation Stats by Tier ===\n");
        for (tier, stats) in &self.compilation_stats {
            report.push_str(&format!("{}: {} compilations, {:.2}ms avg, {:.2}% success\n",
                tier.name(),
                stats.total_compilations,
                stats.average_compilation_time().as_millis(),
                stats.success_rate() * 100.0
            ));
        }

        report.push_str("\n=== Performance Counters ===\n");
        let counters = &self.performance_counters;
        report.push_str(&format!("Hotspots Detected: {}\n", counters.hotspots_detected()));
        report.push_str(&format!("Tier Promotions: {}\n", counters.tier_promotions()));
        report.push_str(&format!("Deoptimizations: {}\n", counters.deoptimizations()));
        report.push_str(&format!("Code Cache Evictions: {}\n", counters.cache_evictions()));

        report
    }

    /// Resets all metrics
    pub fn reset(&mut self) {
        *self = JitMetrics::new();
    }
}

impl Clone for JitMetrics {
    fn clone(&self) -> Self {
        // Create a new instance with current values
        let mut new_metrics = JitMetrics::new();
        
        // Copy over the statistics (atomic values will be read at current state)
        new_metrics.execution_stats = self.execution_stats.clone();
        new_metrics.compilation_stats = self.compilation_stats.clone();
        new_metrics.timing_measurements = self.timing_measurements.clone();
        new_metrics.memory_stats = self.memory_stats.clone();
        new_metrics.cache_stats = self.cache_stats.clone();
        new_metrics.start_time = self.start_time;
        
        // Performance counters are atomic, so we need to read and create new ones
        new_metrics.performance_counters = PerformanceCounters::new();
        
        new_metrics
    }
}

/// General execution statistics
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    /// Total number of executions
    pub total_executions: u64,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Minimum execution time observed
    pub min_execution_time: Duration,
    /// Maximum execution time observed
    pub max_execution_time: Duration,
    /// Recent execution times for trend analysis
    pub recent_execution_times: Vec<Duration>,
    /// Maximum size of recent times buffer
    pub recent_times_buffer_size: usize,
}

impl ExecutionStats {
    fn new() -> Self {
        ExecutionStats {
            total_executions: 0,
            total_execution_time: Duration::ZERO,
            min_execution_time: Duration::MAX,
            max_execution_time: Duration::ZERO,
            recent_execution_times: Vec::new(),
            recent_times_buffer_size: 1000,
        }
    }

    fn record_execution(&mut self, execution_time: Duration) {
        self.total_executions += 1;
        self.total_execution_time += execution_time;
        
        if execution_time < self.min_execution_time {
            self.min_execution_time = execution_time;
        }
        if execution_time > self.max_execution_time {
            self.max_execution_time = execution_time;
        }
        
        self.recent_execution_times.push(execution_time);
        if self.recent_execution_times.len() > self.recent_times_buffer_size {
            self.recent_execution_times.remove(0);
        }
    }

    /// Gets average execution time
    pub fn average_execution_time(&self) -> Duration {
        if self.total_executions > 0 {
            self.total_execution_time / self.total_executions as u32
        } else {
            Duration::ZERO
        }
    }

    /// Calculates execution time variance
    pub fn execution_time_variance(&self) -> f64 {
        if self.recent_execution_times.len() < 2 {
            return 0.0;
        }

        let mean = self.average_execution_time().as_nanos() as f64;
        let variance: f64 = self.recent_execution_times
            .iter()
            .map(|t| {
                let diff = t.as_nanos() as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / self.recent_execution_times.len() as f64;

        variance
    }
}

/// Compilation statistics for a specific tier
#[derive(Debug, Clone)]
pub struct CompilationStats {
    /// Total number of compilation attempts
    pub total_compilations: u64,
    /// Total number of compilation failures
    pub compilation_failures: u64,
    /// Total compilation time
    pub total_compilation_time: Duration,
    /// Minimum compilation time observed
    pub min_compilation_time: Duration,
    /// Maximum compilation time observed
    pub max_compilation_time: Duration,
}

impl CompilationStats {
    fn new() -> Self {
        CompilationStats {
            total_compilations: 0,
            compilation_failures: 0,
            total_compilation_time: Duration::ZERO,
            min_compilation_time: Duration::MAX,
            max_compilation_time: Duration::ZERO,
        }
    }

    fn record_compilation(&mut self, compilation_time: Duration) {
        self.total_compilations += 1;
        self.total_compilation_time += compilation_time;
        
        if compilation_time < self.min_compilation_time {
            self.min_compilation_time = compilation_time;
        }
        if compilation_time > self.max_compilation_time {
            self.max_compilation_time = compilation_time;
        }
    }

    fn record_failure(&mut self) {
        self.total_compilations += 1;
        self.compilation_failures += 1;
    }

    /// Gets average compilation time
    pub fn average_compilation_time(&self) -> Duration {
        if self.total_compilations > 0 {
            self.total_compilation_time / self.total_compilations as u32
        } else {
            Duration::ZERO
        }
    }

    /// Gets compilation success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_compilations > 0 {
            (self.total_compilations - self.compilation_failures) as f64 / self.total_compilations as f64
        } else {
            1.0
        }
    }
}

/// Performance counters using atomic operations for thread safety
#[derive(Debug)]
pub struct PerformanceCounters {
    compilations: AtomicU64,
    compilation_failures: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    hotspots_detected: AtomicU64,
    tier_promotions: AtomicU64,
    deoptimizations: AtomicU64,
    cache_evictions: AtomicU64,
}

impl PerformanceCounters {
    fn new() -> Self {
        PerformanceCounters {
            compilations: AtomicU64::new(0),
            compilation_failures: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            hotspots_detected: AtomicU64::new(0),
            tier_promotions: AtomicU64::new(0),
            deoptimizations: AtomicU64::new(0),
            cache_evictions: AtomicU64::new(0),
        }
    }

    fn increment_compilations(&self) -> u64 {
        self.compilations.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn increment_compilation_failures(&self) -> u64 {
        self.compilation_failures.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn increment_cache_hits(&self) -> u64 {
        self.cache_hits.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn increment_cache_misses(&self) -> u64 {
        self.cache_misses.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Increments hotspots detected counter
    pub fn increment_hotspots_detected(&self) -> u64 {
        self.hotspots_detected.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Increments tier promotions counter
    pub fn increment_tier_promotions(&self) -> u64 {
        self.tier_promotions.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Increments deoptimizations counter
    pub fn increment_deoptimizations(&self) -> u64 {
        self.deoptimizations.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Increments cache evictions counter
    pub fn increment_cache_evictions(&self) -> u64 {
        self.cache_evictions.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Gets current compilations count
    pub fn compilations(&self) -> u64 {
        self.compilations.load(Ordering::Relaxed)
    }

    /// Gets current compilation failures count
    pub fn compilation_failures(&self) -> u64 {
        self.compilation_failures.load(Ordering::Relaxed)
    }

    /// Gets current cache hits count
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }

    /// Gets current cache misses count
    pub fn cache_misses(&self) -> u64 {
        self.cache_misses.load(Ordering::Relaxed)
    }

    /// Gets current hotspots detected count
    pub fn hotspots_detected(&self) -> u64 {
        self.hotspots_detected.load(Ordering::Relaxed)
    }

    /// Gets current tier promotions count
    pub fn tier_promotions(&self) -> u64 {
        self.tier_promotions.load(Ordering::Relaxed)
    }

    /// Gets current deoptimizations count
    pub fn deoptimizations(&self) -> u64 {
        self.deoptimizations.load(Ordering::Relaxed)
    }

    /// Gets current cache evictions count
    pub fn cache_evictions(&self) -> u64 {
        self.cache_evictions.load(Ordering::Relaxed)
    }

    /// Calculates cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits();
        let misses = self.cache_misses();
        let total = hits + misses;
        
        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

/// Timing measurements and analysis
#[derive(Debug, Clone)]
pub struct TimingMeasurements {
    /// Compilation times by tier
    compilation_times: HashMap<CompilationTier, Vec<Duration>>,
    /// Recent execution times
    execution_times: Vec<Duration>,
    /// Maximum number of timing samples to keep
    max_samples: usize,
}

impl TimingMeasurements {
    fn new() -> Self {
        let mut compilation_times = HashMap::new();
        compilation_times.insert(CompilationTier::Bytecode, Vec::new());
        compilation_times.insert(CompilationTier::JitBasic, Vec::new());
        compilation_times.insert(CompilationTier::JitOptimized, Vec::new());

        TimingMeasurements {
            compilation_times,
            execution_times: Vec::new(),
            max_samples: 1000,
        }
    }

    fn record_compilation_time(&mut self, tier: CompilationTier, time: Duration) {
        if let Some(times) = self.compilation_times.get_mut(&tier) {
            times.push(time);
            if times.len() > self.max_samples {
                times.remove(0);
            }
        }
    }

    fn record_execution_time(&mut self, time: Duration) {
        self.execution_times.push(time);
        if self.execution_times.len() > self.max_samples {
            self.execution_times.remove(0);
        }
    }

    /// Gets compilation time percentiles for a tier
    pub fn compilation_time_percentiles(&self, tier: CompilationTier) -> Option<Percentiles> {
        self.compilation_times.get(&tier).map(|times| {
            calculate_percentiles(times)
        })
    }

    /// Gets execution time percentiles
    pub fn execution_time_percentiles(&self) -> Percentiles {
        calculate_percentiles(&self.execution_times)
    }
}

/// Memory usage statistics
#[derive(Debug)]
pub struct MemoryStats {
    /// Current allocated bytes
    current_allocated: AtomicUsize,
    /// Peak allocated bytes
    peak_allocated: AtomicUsize,
    /// Total allocations
    total_allocations: AtomicU64,
    /// Total deallocations
    total_deallocations: AtomicU64,
}

impl MemoryStats {
    fn new() -> Self {
        MemoryStats {
            current_allocated: AtomicUsize::new(0),
            peak_allocated: AtomicUsize::new(0),
            total_allocations: AtomicU64::new(0),
            total_deallocations: AtomicU64::new(0),
        }
    }

    /// Creates a snapshot of current memory stats
    fn clone(&self) -> Self {
        use std::sync::atomic::Ordering;
        MemoryStats {
            current_allocated: AtomicUsize::new(self.current_allocated.load(Ordering::Relaxed)),
            peak_allocated: AtomicUsize::new(self.peak_allocated.load(Ordering::Relaxed)),
            total_allocations: AtomicU64::new(self.total_allocations.load(Ordering::Relaxed)),
            total_deallocations: AtomicU64::new(self.total_deallocations.load(Ordering::Relaxed)),
        }
    }

    fn record_allocation(&self, size: usize) {
        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        let new_current = self.current_allocated.fetch_add(size, Ordering::Relaxed) + size;
        
        // Update peak if necessary
        let mut current_peak = self.peak_allocated.load(Ordering::Relaxed);
        while new_current > current_peak {
            match self.peak_allocated.compare_exchange_weak(
                current_peak, new_current, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }
    }

    fn record_deallocation(&self, size: usize) {
        self.total_deallocations.fetch_add(1, Ordering::Relaxed);
        self.current_allocated.fetch_sub(size.min(self.current_usage()), Ordering::Relaxed);
    }

    /// Gets current memory usage
    pub fn current_usage(&self) -> usize {
        self.current_allocated.load(Ordering::Relaxed)
    }

    /// Gets peak memory usage
    pub fn peak_usage(&self) -> usize {
        self.peak_allocated.load(Ordering::Relaxed)
    }

    /// Gets total allocations count
    pub fn total_allocations(&self) -> u64 {
        self.total_allocations.load(Ordering::Relaxed)
    }

    /// Gets total deallocations count
    pub fn total_deallocations(&self) -> u64 {
        self.total_deallocations.load(Ordering::Relaxed)
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    /// Cache hits
    hits: AtomicU64,
    /// Cache misses
    misses: AtomicU64,
    /// Cache evictions
    evictions: AtomicU64,
}

impl CacheStats {
    fn new() -> Self {
        CacheStats {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Creates a snapshot of current cache stats
    fn clone(&self) -> Self {
        use std::sync::atomic::Ordering;
        CacheStats {
            hits: AtomicU64::new(self.hits.load(Ordering::Relaxed)),
            misses: AtomicU64::new(self.misses.load(Ordering::Relaxed)),
            evictions: AtomicU64::new(self.evictions.load(Ordering::Relaxed)),
        }
    }

    fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Records cache eviction
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Gets cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Gets total cache accesses
    pub fn total_accesses(&self) -> u64 {
        self.hits.load(Ordering::Relaxed) + self.misses.load(Ordering::Relaxed)
    }

    /// Gets evictions count
    pub fn evictions(&self) -> u64 {
        self.evictions.load(Ordering::Relaxed)
    }
}

/// Performance summary snapshot
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    /// Total system uptime
    pub uptime: Duration,
    /// Total number of executions performed
    pub total_executions: u64,
    /// Executions per second rate
    pub executions_per_second: f64,
    /// Average time per execution
    pub average_execution_time: Duration,
    /// Total time spent in compilation
    pub total_compilation_time: Duration,
    /// Cache hit rate percentage (0.0-1.0)
    pub cache_hit_rate: f64,
    /// Current memory usage in bytes
    pub memory_usage: usize,
}

/// Statistical percentiles
#[derive(Debug, Clone)]
pub struct Percentiles {
    /// 50th percentile (median)
    pub p50: Duration,
    /// 90th percentile
    pub p90: Duration,
    /// 95th percentile
    pub p95: Duration,
    /// 99th percentile
    pub p99: Duration,
    /// Minimum value
    pub min: Duration,
    /// Maximum value
    pub max: Duration,
}

/// Calculates percentiles from duration samples
fn calculate_percentiles(samples: &[Duration]) -> Percentiles {
    if samples.is_empty() {
        return Percentiles {
            p50: Duration::ZERO,
            p90: Duration::ZERO,
            p95: Duration::ZERO,
            p99: Duration::ZERO,
            min: Duration::ZERO,
            max: Duration::ZERO,
        };
    }

    let mut sorted_samples: Vec<Duration> = samples.to_vec();
    sorted_samples.sort();

    let len = sorted_samples.len();
    let p50_idx = (len as f64 * 0.50) as usize;
    let p90_idx = (len as f64 * 0.90) as usize;
    let p95_idx = (len as f64 * 0.95) as usize;
    let p99_idx = (len as f64 * 0.99) as usize;

    Percentiles {
        p50: sorted_samples[p50_idx.min(len - 1)],
        p90: sorted_samples[p90_idx.min(len - 1)],
        p95: sorted_samples[p95_idx.min(len - 1)],
        p99: sorted_samples[p99_idx.min(len - 1)],
        min: sorted_samples[0],
        max: sorted_samples[len - 1],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_metrics_creation() {
        let metrics = JitMetrics::new();
        assert_eq!(metrics.total_executions(), 0);
        assert_eq!(metrics.total_compilation_time(), Duration::ZERO);
    }

    #[test]
    fn test_execution_recording() {
        let mut metrics = JitMetrics::new();
        metrics.record_execution(Duration::from_micros(100));
        
        assert_eq!(metrics.total_executions(), 1);
        assert_eq!(metrics.execution_stats().average_execution_time(), Duration::from_micros(100));
    }

    #[test]
    fn test_compilation_recording() {
        let mut metrics = JitMetrics::new();
        metrics.record_compilation(Duration::from_millis(5), CompilationTier::JitBasic);
        
        let stats = metrics.compilation_stats(CompilationTier::JitBasic).unwrap();
        assert_eq!(stats.total_compilations, 1);
        assert_eq!(stats.average_compilation_time(), Duration::from_millis(5));
    }

    #[test]
    fn test_cache_statistics() {
        let mut metrics = JitMetrics::new();
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        
        assert_eq!(metrics.cache_stats().hit_rate(), 2.0 / 3.0);
        assert_eq!(metrics.cache_stats().total_accesses(), 3);
    }

    #[test]
    fn test_performance_counters_thread_safety() {
        let counters = PerformanceCounters::new();
        
        // Test atomic operations
        assert_eq!(counters.increment_compilations(), 1);
        assert_eq!(counters.increment_compilations(), 2);
        assert_eq!(counters.compilations(), 2);
        
        assert_eq!(counters.increment_cache_hits(), 1);
        assert_eq!(counters.cache_hits(), 1);
    }

    #[test]
    fn test_memory_stats() {
        let stats = MemoryStats::new();
        
        stats.record_allocation(1000);
        assert_eq!(stats.current_usage(), 1000);
        assert_eq!(stats.peak_usage(), 1000);
        
        stats.record_allocation(2000);
        assert_eq!(stats.current_usage(), 3000);
        assert_eq!(stats.peak_usage(), 3000);
        
        stats.record_deallocation(1000);
        assert_eq!(stats.current_usage(), 2000);
        assert_eq!(stats.peak_usage(), 3000); // Peak should remain
    }

    #[test]
    fn test_percentiles_calculation() {
        let samples = vec![
            Duration::from_micros(100),
            Duration::from_micros(200),
            Duration::from_micros(300),
            Duration::from_micros(400),
            Duration::from_micros(500),
        ];
        
        let percentiles = calculate_percentiles(&samples);
        assert_eq!(percentiles.min, Duration::from_micros(100));
        assert_eq!(percentiles.max, Duration::from_micros(500));
        assert_eq!(percentiles.p50, Duration::from_micros(300));
    }

    #[test]
    fn test_performance_summary() {
        let mut metrics = JitMetrics::new();
        metrics.record_execution(Duration::from_micros(100));
        metrics.record_execution(Duration::from_micros(200));
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        
        let summary = metrics.performance_summary();
        assert_eq!(summary.total_executions, 2);
        assert_eq!(summary.average_execution_time, Duration::from_micros(150));
        assert_eq!(summary.cache_hit_rate, 0.5);
    }

    #[test]
    fn test_report_generation() {
        let mut metrics = JitMetrics::new();
        metrics.record_execution(Duration::from_micros(100));
        metrics.record_compilation(Duration::from_millis(5), CompilationTier::JitBasic);
        
        let report = metrics.generate_report();
        assert!(report.contains("JIT Performance Report"));
        assert!(report.contains("Total Executions: 1"));
        assert!(report.contains("jit_basic"));
    }
}