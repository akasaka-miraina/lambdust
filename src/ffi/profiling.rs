//! Debugging and profiling support for FFI operations.
//!
//! This module provides comprehensive debugging tools, performance profiling,
//! and monitoring capabilities for FFI function calls and memory operations.

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::{RwLock, Mutex};
use std::time::{Duration, Instant};
use std::thread;

use crate::eval::Value;
use crate::diagnostics::Error;

/// FFI debugging and profiling errors
#[derive(Debug, Clone)]
pub enum ProfilingError {
    /// Profiler not initialized
    NotInitialized,
    /// Invalid profiling configuration
    InvalidConfig {
        parameter: String,
        reason: String,
    },
    /// Profiling data collection failed
    CollectionFailed {
        operation: String,
        error: String,
    },
    /// Report generation failed
    ReportGenerationFailed {
        format: String,
        error: String,
    },
}

impl fmt::Display for ProfilingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfilingError::NotInitialized => {
                write!(f, "FFI profiler not initialized")
            }
            ProfilingError::InvalidConfig { parameter, reason } => {
                write!(f, "Invalid profiling configuration for '{parameter}': {reason}")
            }
            ProfilingError::CollectionFailed { operation, error } => {
                write!(f, "Profiling data collection failed for '{operation}': {error}")
            }
            ProfilingError::ReportGenerationFailed { format, error } => {
                write!(f, "Report generation failed for format '{format}': {error}")
            }
        }
    }
}

impl std::error::Error for ProfilingError {}

impl From<ProfilingError> for Error {
    fn from(profiling_error: ProfilingError) -> Self {
        Error::runtime_error(profiling_error.to_string(), None)
    }
}

/// FFI function call event
#[derive(Debug, Clone)]
pub struct FfiCallEvent {
    /// Unique event ID
    pub id: u64,
    /// Function name
    pub function_name: String,
    /// Library name
    pub library_name: String,
    /// Thread ID
    pub thread_id: thread::ThreadId,
    /// Call start time
    pub start_time: Instant,
    /// Call end time (if completed)
    pub end_time: Option<Instant>,
    /// Call duration
    pub duration: Option<Duration>,
    /// Arguments passed to the function
    pub arguments: Vec<String>, // Serialized argument descriptions
    /// Return value (if available)
    pub return_value: Option<String>,
    /// Success status
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Memory allocations during call
    pub memory_allocations: Vec<MemoryAllocationEvent>,
    /// Stack trace (if enabled)
    pub stack_trace: Option<Vec<String>>,
}

/// Memory allocation event
#[derive(Debug, Clone)]
pub struct MemoryAllocationEvent {
    /// Allocation ID
    pub id: u64,
    /// Pointer address
    pub pointer: usize,
    /// Size allocated
    pub size: usize,
    /// Allocation type
    pub allocation_type: AllocationType,
    /// Timestamp
    pub timestamp: Instant,
    /// Thread ID
    pub thread_id: thread::ThreadId,
    /// Stack trace (if enabled)
    pub stack_trace: Option<Vec<String>>,
}

/// Type of memory allocation
#[derive(Debug, Clone, PartialEq)]
pub enum AllocationType {
    /// FFI memory allocation
    FfiAllocation,
    /// Pool allocation
    PoolAllocation,
    /// System allocation
    SystemAllocation,
    /// Deallocation
    Deallocation,
}

/// Performance metrics for FFI operations
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Total number of FFI calls
    pub total_calls: u64,
    /// Successful calls
    pub successful_calls: u64,
    /// Failed calls
    pub failed_calls: u64,
    /// Total time spent in FFI calls
    pub total_time: Duration,
    /// Average call time
    pub average_time: Duration,
    /// Minimum call time
    pub min_time: Duration,
    /// Maximum call time
    pub max_time: Duration,
    /// Calls per second
    pub calls_per_second: f64,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Top slowest functions
    pub slowest_functions: Vec<FunctionPerformance>,
    /// Most called functions
    pub most_called_functions: Vec<FunctionPerformance>,
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Total memory allocated
    pub total_allocated: usize,
    /// Peak memory usage
    pub peak_usage: usize,
    /// Current memory usage
    pub current_usage: usize,
    /// Number of allocations
    pub allocation_count: u64,
    /// Number of deallocations
    pub deallocation_count: u64,
    /// Memory leaks detected
    pub leaks_detected: u64,
}

/// Per-function performance data
#[derive(Debug, Clone)]
pub struct FunctionPerformance {
    /// Function name
    pub name: String,
    /// Library name
    pub library: String,
    /// Number of calls
    pub call_count: u64,
    /// Total time
    pub total_time: Duration,
    /// Average time
    pub average_time: Duration,
    /// Minimum time
    pub min_time: Duration,
    /// Maximum time
    pub max_time: Duration,
    /// Success rate
    pub success_rate: f64,
    /// Memory usage
    pub memory_usage: usize,
}

/// Profiling configuration
#[derive(Debug, Clone)]
pub struct ProfilingConfig {
    /// Enable call tracing
    pub enable_call_tracing: bool,
    /// Enable memory profiling
    pub enable_memory_profiling: bool,
    /// Enable stack traces
    pub enable_stack_traces: bool,
    /// Maximum number of events to keep
    pub max_events: usize,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Enable detailed argument logging
    pub log_arguments: bool,
    /// Enable return value logging
    pub log_return_values: bool,
    /// Profile hot functions threshold (microseconds)
    pub hot_function_threshold: u64,
    /// Enable real-time monitoring
    pub real_time_monitoring: bool,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            enable_call_tracing: true,
            enable_memory_profiling: true,
            enable_stack_traces: false,
            max_events: 10000,
            sampling_rate: 1.0,
            log_arguments: false,
            log_return_values: false,
            hot_function_threshold: 1000, // 1ms
            real_time_monitoring: false,
        }
    }
}

/// FFI profiler
#[derive(Debug)]
pub struct FfiProfiler {
    /// Profiling configuration
    config: RwLock<ProfilingConfig>,
    /// Call events history
    events: RwLock<VecDeque<FfiCallEvent>>,
    /// Memory allocation events
    memory_events: RwLock<VecDeque<MemoryAllocationEvent>>,
    /// Per-function performance data
    function_metrics: RwLock<HashMap<String, FunctionPerformance>>,
    /// Global metrics
    global_metrics: RwLock<PerformanceMetrics>,
    /// Event counter
    event_counter: Mutex<u64>,
    /// Start time for rate calculations
    start_time: Instant,
    /// Whether profiler is active
    active: RwLock<bool>,
}

impl Default for FfiProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl FfiProfiler {
    /// Create a new FFI profiler
    pub fn new() -> Self {
        Self {
            config: RwLock::new(ProfilingConfig::default()),
            events: RwLock::new(VecDeque::new()),
            memory_events: RwLock::new(VecDeque::new()),
            function_metrics: RwLock::new(HashMap::new()),
            global_metrics: RwLock::new(PerformanceMetrics {
                total_calls: 0,
                successful_calls: 0,
                failed_calls: 0,
                total_time: Duration::new(0, 0),
                average_time: Duration::new(0, 0),
                min_time: Duration::from_secs(u64::MAX),
                max_time: Duration::new(0, 0),
                calls_per_second: 0.0,
                memory_stats: MemoryStats {
                    total_allocated: 0,
                    peak_usage: 0,
                    current_usage: 0,
                    allocation_count: 0,
                    deallocation_count: 0,
                    leaks_detected: 0,
                },
                slowest_functions: Vec::new(),
                most_called_functions: Vec::new(),
            }),
            event_counter: Mutex::new(0),
            start_time: Instant::now(),
            active: RwLock::new(false),
        }
    }

    /// Configure the profiler
    pub fn configure(&self, config: ProfilingConfig) -> std::result::Result<(), ProfilingError> {
        // Validate configuration
        if config.sampling_rate < 0.0 || config.sampling_rate > 1.0 {
            return Err(ProfilingError::InvalidConfig {
                parameter: "sampling_rate".to_string(),
                reason: "Must be between 0.0 and 1.0".to_string(),
            });
        }

        if config.max_events == 0 {
            return Err(ProfilingError::InvalidConfig {
                parameter: "max_events".to_string(),
                reason: "Must be greater than 0".to_string(),
            });
        }

        let mut current_config = self.config.write().unwrap();
        *current_config = config;
        Ok(())
    }

    /// Start profiling
    pub fn start(&self) -> std::result::Result<(), ProfilingError> {
        let mut active = self.active.write().unwrap();
        *active = true;
        Ok(())
    }

    /// Stop profiling
    pub fn stop(&self) -> std::result::Result<(), ProfilingError> {
        let mut active = self.active.write().unwrap();
        *active = false;
        Ok(())
    }

    /// Check if profiler is active
    pub fn is_active(&self) -> bool {
        *self.active.read().unwrap()
    }

    /// Record the start of an FFI call
    pub fn record_call_start(
        &self,
        function_name: &str,
        library_name: &str,
        args: &[Value],
    ) -> Option<u64> {
        if !self.is_active() {
            return None;
        }

        let config = self.config.read().unwrap();

        // Check sampling rate
        if config.sampling_rate < 1.0 {
            let random_value: f64 = rand::random();
            if random_value > config.sampling_rate {
                return None;
            }
        }

        let event_id = {
            let mut counter = self.event_counter.lock().unwrap();
            *counter += 1;
            *counter
        };

        let arguments = if config.log_arguments {
            args.iter().map(|arg| format!("{arg:?}")).collect()
        } else {
            vec!["<hidden>".to_string(); args.len()]
        };

        let event = FfiCallEvent {
            id: event_id,
            function_name: function_name.to_string(),
            library_name: library_name.to_string(),
            thread_id: thread::current().id(),
            start_time: Instant::now(),
            end_time: None,
            duration: None,
            arguments,
            return_value: None,
            success: false,
            error: None,
            memory_allocations: Vec::new(),
            stack_trace: if config.enable_stack_traces {
                Some(self.capture_stack_trace())
            } else {
                None
            },
        };

        // Store the event
        {
            let mut events = self.events.write().unwrap();
            events.push_back(event);

            // Limit event history size
            while events.len() > config.max_events {
                events.pop_front();
            }
        }

        Some(event_id)
    }

    /// Record the completion of an FFI call
    pub fn record_call_end(
        &self,
        event_id: u64,
        result: &std::result::Result<Value, String>,
    ) {
        if !self.is_active() {
            return;
        }

        let config = self.config.read().unwrap();
        let end_time = Instant::now();

        // Find and update the event
        {
            let mut events = self.events.write().unwrap();
            if let Some(event) = events.iter_mut().find(|e| e.id == event_id) {
                event.end_time = Some(end_time);
                event.duration = Some(end_time.duration_since(event.start_time));
                
                match result {
                    Ok(value) => {
                        event.success = true;
                        if config.log_return_values {
                            event.return_value = Some(format!("{value:?}"));
                        }
                    }
                    Err(error) => {
                        event.success = false;
                        event.error = Some(error.clone());
                    }
                }

                // Update function metrics
                self.update_function_metrics(event);
            }
        }

        // Update global metrics
        self.update_global_metrics(result.is_ok(), end_time);
    }

    /// Record a memory allocation
    pub fn record_memory_allocation(
        &self,
        pointer: usize,
        size: usize,
        allocation_type: AllocationType,
    ) {
        if !self.is_active() {
            return;
        }

        let config = self.config.read().unwrap();
        if !config.enable_memory_profiling {
            return;
        }

        let event_id = {
            let mut counter = self.event_counter.lock().unwrap();
            *counter += 1;
            *counter
        };

        let event = MemoryAllocationEvent {
            id: event_id,
            pointer,
            size,
            allocation_type: allocation_type.clone(),
            timestamp: Instant::now(),
            thread_id: thread::current().id(),
            stack_trace: if config.enable_stack_traces {
                Some(self.capture_stack_trace())
            } else {
                None
            },
        };

        {
            let mut memory_events = self.memory_events.write().unwrap();
            memory_events.push_back(event);

            // Limit memory event history size
            while memory_events.len() > config.max_events {
                memory_events.pop_front();
            }
        }

        // Update global memory stats
        self.update_memory_stats(size, &allocation_type);
    }

    /// Update function-specific metrics
    fn update_function_metrics(&self, event: &FfiCallEvent) {
        if let Some(duration) = event.duration {
            let mut metrics = self.function_metrics.write().unwrap();
            let key = format!("{}::{}", event.library_name, event.function_name);
            
            let func_perf = metrics.entry(key).or_insert_with(|| FunctionPerformance {
                name: event.function_name.clone(),
                library: event.library_name.clone(),
                call_count: 0,
                total_time: Duration::new(0, 0),
                average_time: Duration::new(0, 0),
                min_time: Duration::from_secs(u64::MAX),
                max_time: Duration::new(0, 0),
                success_rate: 0.0,
                memory_usage: 0,
            });

            func_perf.call_count += 1;
            func_perf.total_time += duration;
            func_perf.average_time = func_perf.total_time / func_perf.call_count as u32;
            
            if duration < func_perf.min_time {
                func_perf.min_time = duration;
            }
            if duration > func_perf.max_time {
                func_perf.max_time = duration;
            }

            // Update success rate
            let successful_calls = if event.success { 1 } else { 0 };
            func_perf.success_rate = (func_perf.success_rate * (func_perf.call_count - 1) as f64 + successful_calls as f64) / func_perf.call_count as f64;
        }
    }

    /// Update global metrics
    fn update_global_metrics(&self, success: bool, _end_time: Instant) {
        let mut metrics = self.global_metrics.write().unwrap();
        metrics.total_calls += 1;
        
        if success {
            metrics.successful_calls += 1;
        } else {
            metrics.failed_calls += 1;
        }

        // Calculate calls per second
        let elapsed = self.start_time.elapsed();
        metrics.calls_per_second = metrics.total_calls as f64 / elapsed.as_secs_f64();
    }

    /// Update memory statistics
    fn update_memory_stats(&self, size: usize, allocation_type: &AllocationType) {
        let mut metrics = self.global_metrics.write().unwrap();
        
        match allocation_type {
            AllocationType::FfiAllocation | AllocationType::PoolAllocation | AllocationType::SystemAllocation => {
                metrics.memory_stats.total_allocated += size;
                metrics.memory_stats.current_usage += size;
                metrics.memory_stats.allocation_count += 1;
                
                if metrics.memory_stats.current_usage > metrics.memory_stats.peak_usage {
                    metrics.memory_stats.peak_usage = metrics.memory_stats.current_usage;
                }
            }
            AllocationType::Deallocation => {
                metrics.memory_stats.current_usage = metrics.memory_stats.current_usage.saturating_sub(size);
                metrics.memory_stats.deallocation_count += 1;
            }
        }
    }

    /// Capture stack trace (simplified implementation)
    fn capture_stack_trace(&self) -> Vec<String> {
        // This is a simplified implementation
        // In practice, you'd use a proper stack trace library
        vec![
            "frame_0: ffi_call".to_string(),
            "frame_1: scheme_eval".to_string(),
            "frame_2: main".to_string(),
        ]
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        let mut metrics = self.global_metrics.read().unwrap().clone();
        
        // Update top functions lists
        let function_metrics = self.function_metrics.read().unwrap();
        
        // Sort by call count for most called
        let mut most_called: Vec<_> = function_metrics.values().cloned().collect();
        most_called.sort_by(|a, b| b.call_count.cmp(&a.call_count));
        metrics.most_called_functions = most_called.into_iter().take(10).collect();
        
        // Sort by average time for slowest
        let mut slowest: Vec<_> = function_metrics.values().cloned().collect();
        slowest.sort_by(|a, b| b.average_time.cmp(&a.average_time));
        metrics.slowest_functions = slowest.into_iter().take(10).collect();
        
        metrics
    }

    /// Get call events
    pub fn get_call_events(&self) -> Vec<FfiCallEvent> {
        let events = self.events.read().unwrap();
        events.iter().cloned().collect()
    }

    /// Get memory events
    pub fn get_memory_events(&self) -> Vec<MemoryAllocationEvent> {
        let events = self.memory_events.read().unwrap();
        events.iter().cloned().collect()
    }

    /// Generate a profiling report
    pub fn generate_report(&self, format: &str) -> std::result::Result<String, ProfilingError> {
        match format.to_lowercase().as_str() {
            "text" => self.generate_text_report(),
            "json" => self.generate_json_report(),
            "html" => self.generate_html_report(),
            _ => Err(ProfilingError::ReportGenerationFailed {
                format: format.to_string(),
                error: "Unsupported format".to_string(),
            }),
        }
    }

    /// Generate a text report
    fn generate_text_report(&self) -> std::result::Result<String, ProfilingError> {
        let metrics = self.get_metrics();
        let mut report = String::new();

        report.push_str("FFI Profiling Report\n");
        report.push_str("===================\n\n");

        // Global statistics
        report.push_str("Global Statistics:\n");
        report.push_str(&format!("  Total calls: {}\n", metrics.total_calls));
        report.push_str(&format!("  Successful calls: {}\n", metrics.successful_calls));
        report.push_str(&format!("  Failed calls: {}\n", metrics.failed_calls));
        report.push_str(&format!("  Success rate: {:.2}%\n", 
            (metrics.successful_calls as f64 / metrics.total_calls as f64) * 100.0));
        report.push_str(&format!("  Calls per second: {:.2}\n", metrics.calls_per_second));
        report.push_str(&format!("  Average call time: {:?}\n", metrics.average_time));
        report.push('\n');

        // Memory statistics
        report.push_str("Memory Statistics:\n");
        report.push_str(&format!("  Total allocated: {} bytes\n", metrics.memory_stats.total_allocated));
        report.push_str(&format!("  Peak usage: {} bytes\n", metrics.memory_stats.peak_usage));
        report.push_str(&format!("  Current usage: {} bytes\n", metrics.memory_stats.current_usage));
        report.push_str(&format!("  Allocations: {}\n", metrics.memory_stats.allocation_count));
        report.push_str(&format!("  Deallocations: {}\n", metrics.memory_stats.deallocation_count));
        report.push('\n');

        // Top functions
        report.push_str("Most Called Functions:\n");
        for (i, func) in metrics.most_called_functions.iter().take(5).enumerate() {
            report.push_str(&format!("  {}. {}::{} ({} calls, avg: {:?})\n", 
                i + 1, func.library, func.name, func.call_count, func.average_time));
        }
        report.push('\n');

        report.push_str("Slowest Functions:\n");
        for (i, func) in metrics.slowest_functions.iter().take(5).enumerate() {
            report.push_str(&format!("  {}. {}::{} (avg: {:?}, {} calls)\n", 
                i + 1, func.library, func.name, func.average_time, func.call_count));
        }

        Ok(report)
    }

    /// Generate a JSON report
    fn generate_json_report(&self) -> std::result::Result<String, ProfilingError> {
        let metrics = self.get_metrics();
        
        // This is a simplified JSON generation
        // In practice, you'd use serde_json
        let json = format!(r#"{{
  "total_calls": {},
  "successful_calls": {},
  "failed_calls": {},
  "calls_per_second": {},
  "memory_stats": {{
    "total_allocated": {},
    "peak_usage": {},
    "current_usage": {}
  }}
}}"#, 
            metrics.total_calls,
            metrics.successful_calls, 
            metrics.failed_calls,
            metrics.calls_per_second,
            metrics.memory_stats.total_allocated,
            metrics.memory_stats.peak_usage,
            metrics.memory_stats.current_usage
        );

        Ok(json)
    }

    /// Generate an HTML report
    fn generate_html_report(&self) -> std::result::Result<String, ProfilingError> {
        let metrics = self.get_metrics();
        
        let html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>FFI Profiling Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; }}
        .metric {{ margin: 10px 0; }}
        .section {{ margin: 20px 0; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <h1>FFI Profiling Report</h1>
    
    <div class="section">
        <h2>Global Statistics</h2>
        <div class="metric">Total calls: {}</div>
        <div class="metric">Successful calls: {}</div>
        <div class="metric">Failed calls: {}</div>
        <div class="metric">Calls per second: {:.2}</div>
    </div>
    
    <div class="section">
        <h2>Memory Statistics</h2>
        <div class="metric">Total allocated: {} bytes</div>
        <div class="metric">Peak usage: {} bytes</div>
        <div class="metric">Current usage: {} bytes</div>
    </div>
</body>
</html>"#,
            metrics.total_calls,
            metrics.successful_calls,
            metrics.failed_calls,
            metrics.calls_per_second,
            metrics.memory_stats.total_allocated,
            metrics.memory_stats.peak_usage,
            metrics.memory_stats.current_usage
        );

        Ok(html)
    }

    /// Clear all profiling data
    pub fn clear(&self) {
        {
            let mut events = self.events.write().unwrap();
            events.clear();
        }
        
        {
            let mut memory_events = self.memory_events.write().unwrap();
            memory_events.clear();
        }
        
        {
            let mut function_metrics = self.function_metrics.write().unwrap();
            function_metrics.clear();
        }
        
        {
            let mut global_metrics = self.global_metrics.write().unwrap();
            *global_metrics = PerformanceMetrics {
                total_calls: 0,
                successful_calls: 0,
                failed_calls: 0,
                total_time: Duration::new(0, 0),
                average_time: Duration::new(0, 0),
                min_time: Duration::from_secs(u64::MAX),
                max_time: Duration::new(0, 0),
                calls_per_second: 0.0,
                memory_stats: MemoryStats {
                    total_allocated: 0,
                    peak_usage: 0,
                    current_usage: 0,
                    allocation_count: 0,
                    deallocation_count: 0,
                    leaks_detected: 0,
                },
                slowest_functions: Vec::new(),
                most_called_functions: Vec::new(),
            };
        }
    }
}

lazy_static::lazy_static! {
    /// Global FFI profiler instance
    pub static ref GLOBAL_FFI_PROFILER: FfiProfiler = FfiProfiler::new();
}

/// Convenience functions for global profiler
pub fn start_profiling() -> std::result::Result<(), ProfilingError> {
    GLOBAL_FFI_PROFILER.start()
}

pub fn stop_profiling() -> std::result::Result<(), ProfilingError> {
    GLOBAL_FFI_PROFILER.stop()
}

pub fn record_call_start(function_name: &str, library_name: &str, args: &[Value]) -> Option<u64> {
    GLOBAL_FFI_PROFILER.record_call_start(function_name, library_name, args)
}

pub fn record_call_end(event_id: u64, result: &std::result::Result<Value, String>) {
    GLOBAL_FFI_PROFILER.record_call_end(event_id, result)
}

pub fn get_profiling_metrics() -> PerformanceMetrics {
    GLOBAL_FFI_PROFILER.get_metrics()
}

pub fn generate_profiling_report(format: &str) -> std::result::Result<String, ProfilingError> {
    GLOBAL_FFI_PROFILER.generate_report(format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_profiler_creation() {
        let profiler = FfiProfiler::new();
        assert!(!profiler.is_active());
    }

    #[test]
    fn test_profiler_start_stop() {
        let profiler = FfiProfiler::new();
        
        profiler.start().unwrap();
        assert!(profiler.is_active());
        
        profiler.stop().unwrap();
        assert!(!profiler.is_active());
    }

    #[test]
    fn test_call_recording() {
        let profiler = FfiProfiler::new();
        profiler.start().unwrap();
        
        let args = vec![Value::Literal(Literal::Number(42.0))];
        let event_id = profiler.record_call_start("test_func", "test_lib", &args);
        assert!(event_id.is_some());
        
        let result = Ok(Value::Literal(Literal::Number(84.0)));
        profiler.record_call_end(event_id.unwrap(), &result);
        
        let metrics = profiler.get_metrics();
        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.successful_calls, 1);
    }

    #[test]
    fn test_memory_recording() {
        let profiler = FfiProfiler::new();
        profiler.start().unwrap();
        
        profiler.record_memory_allocation(0x1000, 64, AllocationType::FfiAllocation);
        
        let metrics = profiler.get_metrics();
        assert_eq!(metrics.memory_stats.allocation_count, 1);
        assert_eq!(metrics.memory_stats.total_allocated, 64);
    }

    #[test]
    fn test_report_generation() {
        let profiler = FfiProfiler::new();
        profiler.start().unwrap();
        
        // Add some test data
        let args = vec![Value::Literal(Literal::Number(42.0))];
        let event_id = profiler.record_call_start("test_func", "test_lib", &args).unwrap();
        profiler.record_call_end(event_id, &Ok(Value::Literal(Literal::Number(84.0))));
        
        let text_report = profiler.generate_report("text").unwrap();
        assert!(text_report.contains("FFI Profiling Report"));
        assert!(text_report.contains("Total calls: 1"));
        
        let json_report = profiler.generate_report("json").unwrap();
        assert!(json_report.contains("total_calls"));
        
        let html_report = profiler.generate_report("html").unwrap();
        assert!(html_report.contains("<html>"));
    }

    #[test]
    fn test_configuration_validation() {
        let profiler = FfiProfiler::new();
        
        let invalid_config = ProfilingConfig {
            sampling_rate: -0.5, // Invalid
            ..Default::default()
        };
        
        let result = profiler.configure(invalid_config);
        assert!(matches!(result, Err(ProfilingError::InvalidConfig { .. })));
    }
}