//! Performance profiling and measurement tools for Lambdust.
//!
//! This module provides comprehensive performance monitoring capabilities including
//! CPU profiling, memory tracking, operation counting, and benchmarking infrastructure.

use std::collections::{HashMap, VecDeque};
use std::sync::{RwLock, Mutex};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

/// Unique identifier for profiling sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProfileId(u64);

/// Categories of operations that can be profiled.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProfileCategory {
    /// Lexical analysis operations
    Lexing,
    /// Parsing operations
    Parsing,
    /// Macro expansion operations
    MacroExpansion,
    /// Type checking operations
    TypeChecking,
    /// Expression evaluation operations
    Evaluation,
    /// Fast path operations
    FastPath,
    /// Garbage collection operations
    GarbageCollection,
    /// Memory allocation operations
    MemoryAllocation,
    /// I/O operations
    IO,
    /// FFI operations
    FFI,
    /// List processing operations
    ListOperations,
    /// Symbol interning operations
    SymbolInterning,
    /// Environment access operations
    EnvironmentAccess,
    /// Custom user-defined category
    Custom(String),
}

/// Detailed information about a single profiled operation.
#[derive(Debug, Clone)]
pub struct ProfileEntry {
    /// Unique identifier for this entry
    pub id: u64,
    /// Category of operation
    pub category: ProfileCategory,
    /// Name of the specific operation
    pub operation: String,
    /// Time when operation started
    pub start_time: Instant,
    /// Duration of the operation
    pub duration: Duration,
    /// Memory allocated during operation (bytes)
    pub memory_allocated: usize,
    /// Memory freed during operation (bytes)
    pub memory_freed: usize,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Thread ID where operation occurred
    pub thread_id: std::thread::ThreadId,
}

/// Aggregated statistics for a category of operations.
#[derive(Debug, Clone)]
pub struct CategoryStats {
    /// Category name
    pub category: ProfileCategory,
    /// Total number of operations
    pub operation_count: usize,
    /// Total time spent in this category
    pub total_duration: Duration,
    /// Average duration per operation
    pub average_duration: Duration,
    /// Minimum duration observed
    pub min_duration: Duration,
    /// Maximum duration observed
    pub max_duration: Duration,
    /// Total memory allocated
    pub total_memory_allocated: usize,
    /// Total memory freed
    pub total_memory_freed: usize,
    /// Net memory usage change
    pub net_memory_change: i64,
    /// Operations per second (based on recent activity)
    pub ops_per_second: f64,
}

/// System-wide performance metrics.
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// Total CPU time used by profiled operations
    pub total_cpu_time: Duration,
    /// Peak memory usage observed
    pub peak_memory_usage: usize,
    /// Current memory usage
    pub current_memory_usage: usize,
    /// Number of garbage collections triggered
    pub gc_count: usize,
    /// Time spent in garbage collection
    pub gc_time: Duration,
    /// Fast path hit rate percentage
    pub fast_path_hit_rate: f64,
    /// Memory pool efficiency score
    pub memory_pool_efficiency: f64,
    /// String interning hit rate percentage
    pub string_interning_hit_rate: f64,
}

/// Configuration for the profiler.
#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    /// Maximum number of entries to keep in memory
    pub max_entries: usize,
    /// Whether to track memory allocations
    pub track_memory: bool,
    /// Whether to enable detailed operation tracking
    pub detailed_tracking: bool,
    /// Minimum duration to record (operations shorter than this are ignored)
    pub min_duration_ns: u64,
    /// Whether to enable CPU profiling
    pub cpu_profiling: bool,
    /// Whether to track call stacks
    pub track_call_stacks: bool,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            track_memory: true,
            detailed_tracking: false,
            min_duration_ns: 1000, // 1 microsecond
            cpu_profiling: false,
            track_call_stacks: false,
        }
    }
}

/// The main performance profiler.
pub struct Profiler {
    /// Configuration
    config: ProfilerConfig,
    /// Profile entries (circular buffer)
    entries: RwLock<VecDeque<ProfileEntry>>,
    /// Aggregated statistics by category
    category_stats: RwLock<HashMap<ProfileCategory, CategoryStats>>,
    /// System-wide metrics
    system_metrics: RwLock<SystemMetrics>,
    /// Next entry ID
    next_id: std::sync::atomic::AtomicU64,
    /// Active profile sessions
    active_sessions: RwLock<HashMap<ProfileId, Instant>>,
    /// CPU profiler (if enabled)
    cpu_profiler: Mutex<Option<CpuProfiler>>,
}

/// CPU profiler implementation.
#[derive(Debug)]
struct CpuProfiler {
    /// Sampling interval in microseconds
    sampling_interval_us: u64,
    /// Sample data
    samples: Vec<CpuSample>,
    /// Last sample time
    last_sample: Instant,
}

/// A single CPU usage sample.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CpuSample {
    /// Timestamp of sample
    timestamp: Instant,
    /// CPU usage percentage (0.0 to 100.0)
    cpu_usage: f64,
    /// Memory usage in bytes
    memory_usage: usize,
    /// Active category being profiled
    active_category: Option<ProfileCategory>,
}

impl Profiler {
    /// Creates a new profiler with the given configuration.
    pub fn new(config: ProfilerConfig) -> Self {
        let cpu_profiler = if config.cpu_profiling {
            Some(CpuProfiler {
                sampling_interval_us: 10000, // 10ms
                samples: Vec::new(),
                last_sample: Instant::now(),
            })
        } else {
            None
        };
        
        Self {
            config,
            entries: RwLock::new(VecDeque::new()),
            category_stats: RwLock::new(HashMap::new()),
            system_metrics: RwLock::new(SystemMetrics {
                total_cpu_time: Duration::ZERO,
                peak_memory_usage: 0,
                current_memory_usage: 0,
                gc_count: 0,
                gc_time: Duration::ZERO,
                fast_path_hit_rate: 0.0,
                memory_pool_efficiency: 0.0,
                string_interning_hit_rate: 0.0,
            }),
            next_id: std::sync::atomic::AtomicU64::new(1),
            active_sessions: RwLock::new(HashMap::new()),
            cpu_profiler: Mutex::new(cpu_profiler),
        }
    }
    
    /// Starts profiling an operation.
    pub fn start_profile(&self, category: ProfileCategory, operation: &str) -> ProfileSession<'_> {
        let id = ProfileId(self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
        let start_time = Instant::now();
        
        if let Ok(mut sessions) = self.active_sessions.write() {
            sessions.insert(id, start_time);
        }
        
        ProfileSession {
            id,
            category,
            operation: operation.to_string(),
            start_time,
            metadata: HashMap::new(),
            profiler: self,
        }
    }
    
    /// Records a completed profile entry.
    pub fn record_entry(&self, entry: ProfileEntry) {
        // Check minimum duration threshold
        if entry.duration.as_nanos() < self.config.min_duration_ns as u128 {
            return;
        }
        
        // Update entries
        if let Ok(mut entries) = self.entries.write() {
            if entries.len() >= self.config.max_entries {
                entries.pop_front();
            }
            entries.push_back(entry.clone());
        }
        
        // Update category statistics
        self.update_category_stats(&entry);
        
        // Update system metrics
        self.update_system_metrics(&entry);
        
        // Sample CPU if profiling is enabled
        if let Ok(mut cpu_profiler) = self.cpu_profiler.lock() {
            if let Some(ref mut profiler) = cpu_profiler.as_mut() {
                profiler.sample_if_needed(&entry.category);
            }
        }
    }
    
    /// Updates category statistics with a new entry.
    fn update_category_stats(&self, entry: &ProfileEntry) {
        if let Ok(mut stats) = self.category_stats.write() {
            let category_stats = stats.entry(entry.category.clone()).or_insert_with(|| {
                CategoryStats {
                    category: entry.category.clone(),
                    operation_count: 0,
                    total_duration: Duration::ZERO,
                    average_duration: Duration::ZERO,
                    min_duration: Duration::MAX,
                    max_duration: Duration::ZERO,
                    total_memory_allocated: 0,
                    total_memory_freed: 0,
                    net_memory_change: 0,
                    ops_per_second: 0.0,
                }
            });
            
            category_stats.operation_count += 1;
            category_stats.total_duration += entry.duration;
            category_stats.average_duration = category_stats.total_duration / category_stats.operation_count as u32;
            category_stats.min_duration = category_stats.min_duration.min(entry.duration);
            category_stats.max_duration = category_stats.max_duration.max(entry.duration);
            category_stats.total_memory_allocated += entry.memory_allocated;
            category_stats.total_memory_freed += entry.memory_freed;
            category_stats.net_memory_change += entry.memory_allocated as i64 - entry.memory_freed as i64;
            
            // Calculate ops per second based on recent activity (last 10 seconds)
            let recent_cutoff = Instant::now() - Duration::from_secs(10);
            if entry.start_time >= recent_cutoff {
                let elapsed_secs = (Instant::now() - recent_cutoff).as_secs_f64();
                if elapsed_secs > 0.0 {
                    category_stats.ops_per_second = category_stats.operation_count as f64 / elapsed_secs;
                }
            }
        }
    }
    
    /// Updates system-wide metrics with a new entry.
    fn update_system_metrics(&self, entry: &ProfileEntry) {
        if let Ok(mut metrics) = self.system_metrics.write() {
            metrics.total_cpu_time += entry.duration;
            
            let current_memory = entry.memory_allocated.saturating_sub(entry.memory_freed);
            metrics.current_memory_usage = metrics.current_memory_usage
                .saturating_add(current_memory);
            metrics.peak_memory_usage = metrics.peak_memory_usage.max(metrics.current_memory_usage);
            
            // Update derived metrics from other systems
            metrics.fast_path_hit_rate = crate::eval::get_fast_path_stats().hit_rate;
            
            if let Some(pool_stats) = self.get_memory_pool_stats() {
                metrics.memory_pool_efficiency = pool_stats.overall_efficiency();
            }
            
            if let Some(interner_stats) = self.get_string_interner_stats() {
                let total_symbols = interner_stats.total_symbols;
                let common_symbols = interner_stats.common_symbols;
                if total_symbols > 0 {
                    metrics.string_interning_hit_rate = (common_symbols as f64 / total_symbols as f64) * 100.0;
                }
            }
        }
    }
    
    /// Gets category statistics.
    pub fn get_category_stats(&self) -> HashMap<ProfileCategory, CategoryStats> {
        if let Ok(stats) = self.category_stats.read() {
            stats.clone()
        } else {
            HashMap::new()
        }
    }
    
    /// Gets system metrics.
    pub fn get_system_metrics(&self) -> SystemMetrics {
        if let Ok(metrics) = self.system_metrics.read() {
            metrics.clone()
        } else {
            SystemMetrics {
                total_cpu_time: Duration::ZERO,
                peak_memory_usage: 0,
                current_memory_usage: 0,
                gc_count: 0,
                gc_time: Duration::ZERO,
                fast_path_hit_rate: 0.0,
                memory_pool_efficiency: 0.0,
                string_interning_hit_rate: 0.0,
            }
        }
    }
    
    /// Gets recent profile entries.
    pub fn get_recent_entries(&self, count: usize) -> Vec<ProfileEntry> {
        if let Ok(entries) = self.entries.read() {
            entries.iter().rev().take(count).cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    /// Gets CPU profiling samples.
    #[allow(private_interfaces)]
    pub fn get_cpu_samples(&self) -> Vec<CpuSample> {
        if let Ok(cpu_profiler) = self.cpu_profiler.lock() {
            if let Some(profiler) = cpu_profiler.as_ref() {
                profiler.samples.clone()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
    
    /// Generates a comprehensive performance report.
    pub fn generate_report(&self) -> PerformanceReport {
        let category_stats = self.get_category_stats();
        let system_metrics = self.get_system_metrics();
        let recent_entries = self.get_recent_entries(100);
        let cpu_samples = self.get_cpu_samples();
        
        // Calculate some derived metrics
        let total_operations: usize = category_stats.values().map(|s| s.operation_count).sum();
        let average_op_duration = if total_operations > 0 {
            system_metrics.total_cpu_time / total_operations as u32
        } else {
            Duration::ZERO
        };
        
        // Find hotspots (categories with highest total time)
        let mut hotspots: Vec<_> = category_stats.values().collect();
        hotspots.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));
        let top_hotspots: Vec<_> = hotspots.into_iter().take(5).cloned().collect();
        
        PerformanceReport {
            timestamp: Instant::now(),
            total_operations,
            average_op_duration,
            category_stats,
            system_metrics,
            recent_entries,
            cpu_samples,
            top_hotspots,
            memory_recommendations: self.generate_memory_recommendations(),
            optimization_suggestions: self.generate_optimization_suggestions(),
        }
    }
    
    /// Generates memory optimization recommendations.
    fn generate_memory_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let metrics = self.get_system_metrics();
        
        if metrics.memory_pool_efficiency < 0.5 {
            recommendations.push("Consider tuning memory pool sizes for better efficiency".to_string());
        }
        
        if metrics.string_interning_hit_rate < 70.0 {
            recommendations.push("String interning hit rate is low, consider pre-interning more common symbols".to_string());
        }
        
        if metrics.peak_memory_usage > 100 * 1024 * 1024 { // 100MB
            recommendations.push("High memory usage detected, consider enabling more aggressive garbage collection".to_string());
        }
        
        recommendations
    }
    
    /// Generates performance optimization suggestions.
    fn generate_optimization_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        let metrics = self.get_system_metrics();
        let category_stats = self.get_category_stats();
        
        if metrics.fast_path_hit_rate < 80.0 {
            suggestions.push("Fast path hit rate is low, consider optimizing more common operations".to_string());
        }
        
        // Check for slow categories
        for stats in category_stats.values() {
            if stats.average_duration > Duration::from_millis(10) {
                suggestions.push(format!("Category {:?} has high average duration, consider optimization", stats.category));
            }
        }
        
        if metrics.gc_time > metrics.total_cpu_time / 10 {
            suggestions.push("Garbage collection overhead is high, consider tuning GC parameters".to_string());
        }
        
        suggestions
    }
    
    /// Helper to get memory pool statistics (placeholder).
    fn get_memory_pool_stats(&self) -> Option<crate::utils::advanced_memory_pool::GlobalPoolStats> {
        Some(crate::utils::advanced_memory_pool::comprehensive_pool_stats())
    }
    
    /// Helper to get string interner statistics (placeholder).
    fn get_string_interner_stats(&self) -> Option<crate::utils::SymbolInternerStats> {
        Some(crate::utils::global_symbol_interner_stats())
    }
    
    /// Clears all profiling data.
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
        if let Ok(mut stats) = self.category_stats.write() {
            stats.clear();
        }
        if let Ok(mut sessions) = self.active_sessions.write() {
            sessions.clear();
        }
    }
}

/// A profiling session that automatically records timing when dropped.
pub struct ProfileSession<'a> {
    id: ProfileId,
    category: ProfileCategory,
    operation: String,
    start_time: Instant,
    metadata: HashMap<String, String>,
    profiler: &'a Profiler,
}

impl<'a> ProfileSession<'a> {
    /// Adds metadata to this profile session.
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Records memory allocation during this session.
    pub fn record_allocation(&mut self, bytes: usize) {
        self.metadata.insert("allocated_bytes".to_string(), bytes.to_string());
    }
    
    /// Records memory deallocation during this session.
    pub fn record_deallocation(&mut self, bytes: usize) {
        self.metadata.insert("freed_bytes".to_string(), bytes.to_string());
    }
}

impl<'a> Drop for ProfileSession<'a> {
    fn drop(&mut self) {
        let end_time = Instant::now();
        let duration = end_time.duration_since(self.start_time);
        
        // Remove from active sessions
        if let Ok(mut sessions) = self.profiler.active_sessions.write() {
            sessions.remove(&self.id);
        }
        
        // Parse memory info from metadata
        let memory_allocated = self.metadata.get("allocated_bytes")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let memory_freed = self.metadata.get("freed_bytes")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        
        let entry = ProfileEntry {
            id: self.id.0,
            category: self.category.clone(),
            operation: self.operation.clone(),
            start_time: self.start_time,
            duration,
            memory_allocated,
            memory_freed,
            metadata: self.metadata.clone(),
            thread_id: std::thread::current().id(),
        };
        
        self.profiler.record_entry(entry);
    }
}

impl CpuProfiler {
    /// Samples CPU usage if enough time has passed.
    fn sample_if_needed(&mut self, active_category: &ProfileCategory) {
        let now = Instant::now();
        if now.duration_since(self.last_sample).as_micros() >= self.sampling_interval_us as u128 {
            let sample = CpuSample {
                timestamp: now,
                cpu_usage: self.get_cpu_usage(),
                memory_usage: self.get_memory_usage(),
                active_category: Some(active_category.clone()),
            };
            
            self.samples.push(sample);
            self.last_sample = now;
            
            // Keep only recent samples (last 1000)
            if self.samples.len() > 1000 {
                self.samples.remove(0);
            }
        }
    }
    
    /// Gets current CPU usage (placeholder implementation).
    fn get_cpu_usage(&self) -> f64 {
        // In a real implementation, this would read from /proc/stat or similar
        0.0
    }
    
    /// Gets current memory usage (placeholder implementation).
    fn get_memory_usage(&self) -> usize {
        // In a real implementation, this would read from /proc/self/status or similar
        0
    }
}

/// Comprehensive performance report.
#[derive(Debug, Clone)]
#[allow(private_interfaces)]
pub struct PerformanceReport {
    /// When this report was generated
    pub timestamp: Instant,
    /// Total number of operations profiled
    pub total_operations: usize,
    /// Average operation duration
    pub average_op_duration: Duration,
    /// Statistics by category
    pub category_stats: HashMap<ProfileCategory, CategoryStats>,
    /// System-wide metrics
    pub system_metrics: SystemMetrics,
    /// Recent profile entries
    pub recent_entries: Vec<ProfileEntry>,
    /// CPU usage samples
    pub cpu_samples: Vec<CpuSample>,
    /// Top performance hotspots
    pub top_hotspots: Vec<CategoryStats>,
    /// Memory optimization recommendations
    pub memory_recommendations: Vec<String>,
    /// Performance optimization suggestions
    pub optimization_suggestions: Vec<String>,
}

impl PerformanceReport {
    /// Formats the report as a human-readable string.
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Lambdust Performance Report ===\n\n");
        
        // Summary
        report.push_str(&format!("Total Operations: {}\n", self.total_operations));
        report.push_str(&format!("Average Duration: {:?}\n", self.average_op_duration));
        report.push_str(&format!("Peak Memory: {} MB\n", self.system_metrics.peak_memory_usage / 1024 / 1024));
        report.push_str(&format!("Fast Path Hit Rate: {:.1}%\n", self.system_metrics.fast_path_hit_rate));
        report.push('\n');
        
        // Top hotspots
        if !self.top_hotspots.is_empty() {
            report.push_str("=== Performance Hotspots ===\n");
            for (i, hotspot) in self.top_hotspots.iter().enumerate() {
                report.push_str(&format!("{}. {:?}: {} ops, {:?} total\n", 
                    i + 1, hotspot.category, hotspot.operation_count, hotspot.total_duration));
            }
            report.push('\n');
        }
        
        // Recommendations
        if !self.memory_recommendations.is_empty() {
            report.push_str("=== Memory Recommendations ===\n");
            for rec in &self.memory_recommendations {
                report.push_str(&format!("• {rec}\n"));
            }
            report.push('\n');
        }
        
        if !self.optimization_suggestions.is_empty() {
            report.push_str("=== Optimization Suggestions ===\n");
            for suggestion in &self.optimization_suggestions {
                report.push_str(&format!("• {suggestion}\n"));
            }
            report.push('\n');
        }
        
        report
    }
    
    /// Exports the report as JSON.
    pub fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        // In a real implementation, this would use serde_json
        Ok(format!("{{\"total_operations\": {}, \"average_duration_ns\": {}}}", 
                   self.total_operations, self.average_op_duration.as_nanos()))
    }
}

/// Global profiler instance.
static GLOBAL_PROFILER: Lazy<Profiler> = Lazy::new(|| {
    Profiler::new(ProfilerConfig::default())
});

/// Starts profiling an operation using the global profiler.
pub fn profile(category: ProfileCategory, operation: &str) -> ProfileSession<'_> {
    GLOBAL_PROFILER.start_profile(category, operation)
}

/// Gets the global profiler instance.
pub fn global_profiler() -> &'static Profiler {
    &GLOBAL_PROFILER
}

/// Generates a performance report using the global profiler.
pub fn generate_report() -> PerformanceReport {
    GLOBAL_PROFILER.generate_report()
}

/// Convenience macros for profiling.
#[macro_export]
macro_rules! profile_scope {
    ($category:expr, $operation:expr) => {
        let _profile_session = $crate::utils::profiler::profile($category, $operation);
    };
    ($category:expr, $operation:expr, $session:ident) => {
        let mut $session = $crate::utils::profiler::profile($category, $operation);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_basic_profiling() {
        let profiler = Profiler::new(ProfilerConfig::default());
        
        {
            let mut session = profiler.start_profile(ProfileCategory::Evaluation, "test_operation");
            session.add_metadata("test_key".to_string(), "test_value".to_string());
            thread::sleep(Duration::from_millis(1));
        }
        
        let entries = profiler.get_recent_entries(10);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].operation, "test_operation");
        assert!(entries[0].duration > Duration::ZERO);
    }
    
    #[test]
    fn test_category_stats() {
        let profiler = Profiler::new(ProfilerConfig::default());
        
        // Profile multiple operations in the same category
        for i in 0..5 {
            let _session = profiler.start_profile(ProfileCategory::Parsing, &format!("operation_{}", i).as_str());
            thread::sleep(Duration::from_millis(1));
        }
        
        let stats = profiler.get_category_stats();
        let parsing_stats = stats.get(&ProfileCategory::Parsing).unwrap();
        assert_eq!(parsing_stats.operation_count, 5);
        assert!(parsing_stats.total_duration > Duration::ZERO);
    }
    
    #[test]
    fn test_memory_tracking() {
        let profiler = Profiler::new(ProfilerConfig::default());
        
        {
            let mut session = profiler.start_profile(ProfileCategory::MemoryAllocation, "alloc_test");
            session.record_allocation(1024);
            session.record_deallocation(512);
        }
        
        let entries = profiler.get_recent_entries(1);
        assert_eq!(entries[0].memory_allocated, 1024);
        assert_eq!(entries[0].memory_freed, 512);
    }
    
    #[test]
    fn test_performance_report() {
        let profiler = Profiler::new(ProfilerConfig::default());
        
        // Generate some profile data
        for _ in 0..10 {
            let _session = profiler.start_profile(ProfileCategory::Evaluation, "eval_test");
            thread::sleep(Duration::from_millis(1));
        }
        
        let report = profiler.generate_report();
        assert!(report.total_operations > 0);
        assert!(report.average_op_duration > Duration::ZERO);
        assert!(!report.format_report().is_empty());
    }
    
    #[test]
    fn test_profiling_macro() {
        profile_scope!(ProfileCategory::Evaluation, "macro_test");
        
        let stats = global_profiler().get_category_stats();
        assert!(stats.contains_key(&ProfileCategory::Evaluation));
    }
}