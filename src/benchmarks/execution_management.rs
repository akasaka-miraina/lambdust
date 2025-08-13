//! Execution management and resource monitoring structures.
//!
//! This module provides structures for monitoring system resource usage
//! during benchmark execution, tracking resource utilization efficiency,
//! and managing execution state throughout the benchmarking process.

use std::time::SystemTime;
use serde::{Deserialize, Serialize};

/// System resource usage during benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceUsage {
    /// Resource usage timeline
    pub timeline: Vec<ResourceSnapshot>,
    /// Peak resource usage
    pub peak_usage: ResourceSnapshot,
    /// Average resource usage
    pub average_usage: ResourceSnapshot,
    /// Resource utilization efficiency
    pub efficiency_metrics: ResourceEfficiency,
}

/// Resource usage at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    /// Timestamp
    pub timestamp: SystemTime,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Available memory in bytes
    pub available_memory_bytes: u64,
    /// Disk I/O rate (bytes/sec)
    pub disk_io_rate: f64,
    /// Network I/O rate (bytes/sec)
    pub network_io_rate: f64,
    /// Active processes count
    pub process_count: u32,
}

/// Resource utilization efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEfficiency {
    /// CPU utilization efficiency (0-1)
    pub cpu_efficiency: f64,
    /// Memory utilization efficiency (0-1)
    pub memory_efficiency: f64,
    /// Overall system efficiency (0-1)
    pub overall_efficiency: f64,
    /// Bottleneck identification
    pub bottlenecks: Vec<String>,
}