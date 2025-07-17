//! Advanced JIT System Type Definitions
//!
//! This module contains type definitions for the advanced JIT system
//! that were missing from the main implementation.

use std::time::Duration;

/// Execution pattern classification for optimization strategy selection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionPattern {
    /// Compute-intensive operations
    ComputeHeavy,
    /// Memory-intensive operations
    MemoryHeavy,
    /// High-frequency calls
    HighFrequency,
    /// Balanced execution profile
    Balanced,
}

/// Dynamic profiling statistics summary
#[derive(Debug, Clone)]
pub struct DynamicProfilingStats {
    /// Total number of executions recorded
    pub total_executions: u64,
    /// Number of identified hot paths
    pub hot_path_count: usize,
    /// Total memory allocated (bytes)
    pub total_memory_allocated: usize,
    /// Average execution time across all executions
    pub average_execution_time: Duration,
    /// Branch prediction accuracy (0.0 to 1.0)
    pub branch_prediction_accuracy: f64,
    /// Call graph complexity (number of edges)
    pub call_graph_complexity: usize,
}

/// Adaptive compilation trigger criteria
#[derive(Debug, Clone)]
pub struct AdaptiveCompilationCriteria {
    /// Execution count threshold
    pub execution_threshold: u64,
    /// Average execution time threshold
    pub time_threshold: Duration,
    /// Memory allocation threshold
    pub memory_threshold: usize,
    /// Compilation priority score
    pub priority_score: f64,
}

/// Compilation strategy effectiveness tracking
#[derive(Debug, Clone)]
pub struct StrategyEffectiveness {
    /// Strategy name
    pub strategy_name: String,
    /// Number of times applied
    pub application_count: u64,
    /// Average speedup achieved
    pub average_speedup: f64,
    /// Compilation time overhead
    pub compilation_overhead: Duration,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}