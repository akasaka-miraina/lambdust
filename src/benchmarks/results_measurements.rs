//! Results and measurement structures for benchmark execution.
//!
//! This module provides comprehensive data structures for capturing,
//! analyzing, and reporting benchmark execution results, including
//! timing measurements, memory usage, validation results, and statistics.

use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use super::benchmark_config::{TestCase, ParameterValue};
use super::system_metadata::{TestFailure, ResourceStats};

/// Complete performance results for a single Scheme implementation.
/// 
/// Contains all test results, performance scores, and resource usage
/// statistics for one implementation across all test categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationResult {
    /// Implementation configuration
    pub config: super::benchmark_config::ImplementationConfig,
    /// Results by test category
    pub category_results: HashMap<String, CategoryResult>,
    /// Overall performance score (0-100)
    pub overall_score: f64,
    /// Performance ranking among all implementations
    pub ranking: u32,
    /// Failures and errors
    pub failures: Vec<Box<TestFailure>>,
    /// Resource usage statistics
    pub resource_stats: ResourceStats,
}

/// Performance results for a specific test category within an implementation.
/// 
/// Aggregates individual test results within a category (e.g., arithmetic,
/// list operations) with category-level statistics and analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResult {
    /// Category name
    pub category: String,
    /// Individual test results
    pub test_results: Vec<TestResult>,
    /// Category performance score
    pub score: f64,
    /// Statistical summary for the category
    pub statistics: CategoryStatistics,
}

/// Complete execution results and measurements for a single test case.
/// 
/// Contains timing, memory usage, validation results, and metadata
/// for one parameterized test execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test case information
    pub test_case: TestCase,
    /// Parameter values used for this test
    pub parameters: HashMap<String, ParameterValue>,
    /// Execution timing measurements
    pub timing: TimingMeasurements,
    /// Memory usage measurements
    pub memory: MemoryMeasurements,
    /// Result correctness validation
    pub validation: ValidationResult,
    /// Whether the test completed successfully
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Comprehensive timing statistics for test execution.
/// 
/// Provides detailed timing analysis including statistical measures,
/// confidence intervals, and performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingMeasurements {
    /// Individual iteration times
    pub iteration_times: Vec<Duration>,
    /// Mean execution time
    pub mean: Duration,
    /// Median execution time
    pub median: Duration,
    /// Standard deviation
    pub std_dev: Duration,
    /// Minimum time
    pub min: Duration,
    /// Maximum time
    pub max: Duration,
    /// Percentiles
    pub percentiles: HashMap<u8, Duration>, // 50th, 90th, 95th, 99th
    /// Confidence interval
    pub confidence_interval: ConfidenceInterval,
    /// Operations per second
    pub ops_per_second: f64,
}

/// Memory usage analysis and statistics for test execution.
/// 
/// Tracks memory consumption patterns, peak usage, and efficiency
/// metrics for memory performance analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMeasurements {
    /// Peak memory usage in bytes
    pub peak_usage: u64,
    /// Memory usage over time
    pub usage_timeline: Vec<(Duration, u64)>,
    /// Memory allocation rate
    pub allocation_rate: f64,
    /// Memory efficiency (operations per MB)
    pub efficiency: f64,
}

/// Statistical confidence interval for timing measurements.
/// 
/// Provides uncertainty bounds for performance measurements
/// at a specified confidence level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Lower bound
    pub lower: Duration,
    /// Upper bound  
    pub upper: Duration,
    /// Confidence level
    pub confidence_level: f64,
}

/// Validation results for test output correctness.
/// 
/// Compares actual test results against expected values
/// to ensure implementation correctness alongside performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the result type matches expectation
    pub type_correct: bool,
    /// Whether the result value is correct (if deterministic)
    pub value_correct: Option<bool>,
    /// Actual result (truncated for large results)
    pub actual_result: String,
    /// Expected result (if known)
    pub expected_result: Option<String>,
}

/// Aggregate statistical analysis for a test category.
/// 
/// Provides comprehensive statistics across all tests within
/// a category for performance trend analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStatistics {
    /// Number of successful tests
    pub successful_tests: u32,
    /// Total number of tests
    pub total_tests: u32,
    /// Success rate
    pub success_rate: f64,
    /// Average performance across tests
    pub avg_performance: f64,
    /// Performance variance
    pub performance_variance: f64,
    /// Ranking among implementations for this category
    pub category_ranking: u32,
}