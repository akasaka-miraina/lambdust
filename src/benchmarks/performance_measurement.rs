//! Performance measurement data models.
//!
//! This module defines the core data structures for capturing
//! performance measurements with system context and metadata.

use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

/// Single performance measurement with context and parameters.
/// 
/// Captures a point-in-time performance measurement with
/// associated system context and test configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasurement {
    /// Timestamp of measurement
    pub timestamp: SystemTime,
    /// Performance value (e.g., operations per second)
    pub value: f64,
    /// Test parameters (if any)
    pub parameters: HashMap<String, String>,
    /// System context
    pub context: SystemContext,
    /// Confidence in this measurement
    pub confidence: f64,
}

/// System environment context during performance measurement.
/// 
/// Captures relevant system state that may influence performance
/// to provide context for measurement interpretation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    /// Git commit hash
    pub commit_hash: Option<String>,
    /// Compiler version
    pub compiler_version: String,
    /// System load during measurement
    pub system_load: f64,
    /// Available memory during measurement
    pub available_memory_mb: u64,
    /// Temperature (if available)
    pub temperature: Option<f64>,
}