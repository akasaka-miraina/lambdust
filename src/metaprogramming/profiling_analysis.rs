//! Runtime profiling and performance analysis components.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Profiling information.
#[derive(Debug, Clone)]
pub struct ProfilingInfo {
    /// Function call counts
    pub call_counts: HashMap<String, usize>,
    /// Function execution times
    pub execution_times: HashMap<String, Duration>,
    /// Memory allocation tracking
    pub allocations: HashMap<String, AllocationInfo>,
    /// Hot spots (most time-consuming functions)
    pub hot_spots: Vec<HotSpot>,
}

/// Memory allocation information.
#[derive(Debug, Clone)]
pub struct AllocationInfo {
    /// Number of allocations
    pub count: usize,
    /// Total bytes allocated
    pub bytes: usize,
    /// Average allocation size
    pub avg_size: f64,
}

/// Hot spot in profiling.
#[derive(Debug, Clone)]
pub struct HotSpot {
    /// Function name
    pub function: String,
    /// Percentage of total execution time
    pub time_percentage: f64,
    /// Number of calls
    pub call_count: usize,
    /// Average time per call
    pub avg_time: Duration,
}

/// Profiler for runtime performance analysis.
#[derive(Debug)]
pub struct Profiler {
    /// Profiling data
    profiling_info: ProfilingInfo,
    /// Start time
    start_time: Instant,
}

impl Profiler {
    /// Creates a new profiler.
    pub fn new() -> Self {
        Self {
            profiling_info: ProfilingInfo {
                call_counts: HashMap::new(),
                execution_times: HashMap::new(),
                allocations: HashMap::new(),
                hot_spots: Vec::new(),
            },
            start_time: Instant::now(),
        }
    }

    /// Records a function call.
    pub fn record_call(&mut self, function_name: String, duration: Duration) {
        *self.profiling_info.call_counts.entry(function_name.clone()).or_insert(0) += 1;
        *self.profiling_info.execution_times.entry(function_name).or_insert(Duration::from_secs(0)) += duration;
    }

    /// Gets profiling results.  
    pub fn get_results(&self) -> &ProfilingInfo {
        &self.profiling_info
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}