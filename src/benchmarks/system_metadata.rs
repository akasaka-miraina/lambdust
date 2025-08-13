//! System metadata and failure tracking structures.
//!
//! This module provides structures for capturing system information,
//! test failures, and resource usage statistics for comprehensive
//! benchmark result analysis and debugging.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use super::benchmark_config::{BenchmarkSuiteConfig, ParameterValue};

/// Metadata and contextual information about a benchmark execution.
/// 
/// Captures execution environment, configuration, and timing information
/// necessary for result interpretation and reproducibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    /// Timestamp when benchmark started
    pub timestamp: SystemTime,
    /// Total duration of benchmark suite
    pub total_duration: Duration,
    /// Configuration used
    pub config: BenchmarkSuiteConfig,
    /// System information
    pub system_info: SystemInfo,
    /// Git commit hash (if available)
    pub git_commit: Option<String>,
    /// Environment variables
    pub environment: HashMap<String, String>,
}

/// Hardware and system configuration information.
/// 
/// Provides context about the execution environment for understanding
/// performance results and comparing across different systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system name and version.
    pub os: String,
    /// System architecture (e.g., x86_64, aarch64).
    pub architecture: String,
    /// CPU model name and details.
    pub cpu_model: String,
    /// Number of CPU cores.
    pub cpu_cores: u32,
    /// Total system memory in megabytes.
    pub total_memory_mb: u64,
    /// Available system memory in megabytes.
    pub available_memory_mb: u64,
    /// System hostname.
    pub hostname: String,
}

/// Test failure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    /// Test that failed
    pub test_name: String,
    /// Category of the failed test
    pub category: String,
    /// Parameters used
    pub parameters: HashMap<String, ParameterValue>,
    /// Failure reason
    pub reason: FailureReason,
    /// Error message
    pub error_message: String,
    /// Stack trace (if available)
    pub stack_trace: Option<String>,
}

/// Reason for test failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureReason {
    /// Test execution exceeded time limit
    Timeout,
    /// Test exceeded memory allocation limits
    OutOfMemory,
    /// Runtime error during test execution
    RuntimeError,
    /// Code compilation or parsing error
    CompilationError,
    /// Result validation failed
    ValidationFailure,
    /// Test infrastructure or environment error
    InfrastructureError,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    /// CPU usage statistics
    pub cpu: CPUStats,
    /// Memory usage statistics  
    pub memory: MemoryStats,
    /// Disk I/O statistics
    pub disk_io: DiskIOStats,
    /// Network I/O statistics (if monitored)
    pub network_io: Option<NetworkIOStats>,
}

/// CPU usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUStats {
    /// Average CPU usage percentage
    pub avg_usage_percent: f64,
    /// Peak CPU usage percentage
    pub peak_usage_percent: f64,
    /// CPU time spent in user mode
    pub user_time: Duration,
    /// CPU time spent in kernel mode
    pub kernel_time: Duration,
    /// Number of context switches
    pub context_switches: u64,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Average memory usage in bytes
    pub avg_usage_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_usage_bytes: u64,
    /// Memory allocation count
    pub allocations: u64,
    /// Memory deallocation count
    pub deallocations: u64,
    /// Page faults
    pub page_faults: u64,
}

/// Disk I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIOStats {
    /// Bytes read from disk
    pub bytes_read: u64,
    /// Bytes written to disk
    pub bytes_written: u64,
    /// Number of read operations
    pub read_ops: u64,
    /// Number of write operations
    pub write_ops: u64,
}

/// Network I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIOStats {
    /// Bytes received
    pub bytes_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Number of packets received
    pub packets_received: u64,
    /// Number of packets sent
    pub packets_sent: u64,
}

/// Type alias to reduce large error size
pub type BenchmarkResult<T> = Result<T, Box<TestFailure>>;