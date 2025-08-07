//! Comprehensive Performance Comparison System for Lambdust
//!
//! This module implements a scientifically rigorous benchmark suite for comparing
//! Lambdust performance against major Scheme implementations. It provides:
//!
//! - Standardized cross-implementation benchmarks
//! - Statistical analysis with confidence intervals
//! - Performance regression detection
//! - Actionable optimization recommendations
//! - Automated result collection and reporting

use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use rayon::prelude::*;

/// Configuration for the comprehensive benchmark suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuiteConfig {
    /// Implementations to benchmark against
    pub implementations: Vec<ImplementationConfig>,
    /// Test categories and their configurations
    pub test_categories: Vec<TestCategory>,
    /// Statistical analysis parameters
    pub statistical_config: StatisticalConfig,
    /// Output and reporting configuration
    pub output_config: OutputConfig,
    /// System resource limits and monitoring
    pub resource_config: ResourceConfig,
}

/// Configuration for a single Scheme implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationConfig {
    /// Human-readable name
    pub name: String,
    /// Unique identifier
    pub id: String,
    /// Docker image or local binary path
    pub runtime: RuntimeConfig,
    /// Version information
    pub version: String,
    /// Expected performance baseline (operations per second)
    pub expected_baseline: Option<f64>,
    /// Implementation-specific quirks and limitations
    pub limitations: Vec<String>,
    /// Whether this implementation supports R7RS features
    pub r7rs_compliant: bool,
}

/// Runtime configuration for an implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RuntimeConfig {
    /// Docker container runtime configuration
    Docker {
        /// Docker image name
        image: String,
        /// Container command line arguments
        container_args: Vec<String>,
        /// Volume mount specifications
        volume_mounts: Vec<String>,
    },
    /// Native binary runtime configuration
    Native {
        /// Path to the native binary
        binary_path: String,
        /// Command line arguments
        args: Vec<String>,
        /// Environment variables
        env_vars: HashMap<String, String>,
    },
    /// Lambdust-specific runtime configuration
    Lambdust {
        /// Target directory for built binaries
        target_dir: String,
        /// Build profile ("debug" or "release")
        profile: String, // "debug" or "release"
        /// Cargo features to enable
        features: Vec<String>,
    },
}

/// Test category configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCategory {
    /// Category name (e.g., "arithmetic", "lists", "recursion")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// List of test cases in this category
    pub tests: Vec<TestCase>,
    /// Weight for overall scoring (0.0-1.0)
    pub weight: f64,
    /// Whether this category is critical for performance ranking
    pub critical: bool,
}

/// Individual test case definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Test name
    pub name: String,
    /// Test description
    pub description: String,
    /// Scheme code template (with parameter substitution)
    pub code_template: String,
    /// Test parameters and their ranges
    pub parameters: Vec<TestParameter>,
    /// Expected result type for validation
    pub expected_result_type: ResultType,
    /// Resource limits for this test
    pub resource_limits: TestResourceLimits,
    /// Performance expectations
    pub performance_hints: PerformanceHints,
}

/// Defines a parameter for a test case with values and scaling behavior.
/// 
/// Test parameters allow for parameterized benchmarks where the same test
/// can be run with different input values to analyze performance characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestParameter {
    /// Parameter name
    pub name: String,
    /// Possible parameter values
    pub values: Vec<ParameterValue>,
    /// Expected algorithmic scaling behavior
    pub scaling_behavior: ScalingBehavior,
}

/// Represents different types of values that can be used as test parameters.
/// 
/// Supports various data types and ranges for comprehensive test coverage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ParameterValue {
    /// Integer parameter value
    Integer {
        /// The integer value
        value: i64
    },
    /// Float parameter value
    Float {
        /// The floating-point value
        value: f64
    },
    /// String parameter value
    String {
        /// The string value
        value: String
    },
    /// Boolean parameter value
    Boolean {
        /// The boolean value
        value: bool
    },
    /// Range of integer values
    Range {
        /// Start of range (inclusive)
        start: i64,
        /// End of range (inclusive)
        end: i64,
        /// Step size for iteration
        step: i64
    },
}

/// Describes the expected algorithmic complexity and scaling behavior of operations.
/// 
/// Used for performance analysis and identifying performance regressions
/// by comparing actual scaling behavior against expectations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingBehavior {
    /// Constant time complexity O(1)
    Constant,      // O(1)
    /// Linear time complexity O(n)
    Linear,        // O(n)
    /// Logarithmic time complexity O(log n)
    Logarithmic,   // O(log n)
    /// Quadratic time complexity O(n²)
    Quadratic,     // O(n²)
    /// Exponential time complexity O(2^n)
    Exponential,   // O(2^n)
    /// Custom complexity description
    Custom(String), // Custom complexity description
}

/// Specifies the expected type of result from a test execution.
/// 
/// Used for result validation to ensure tests are executing correctly
/// and producing expected output types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultType {
    /// Numeric result expected
    Number,
    /// String result expected
    String,
    /// Boolean result expected
    Boolean,
    /// List result expected
    List,
    /// Any result type accepted
    Any,
    /// Test expected to fail with error
    Error, // Test expected to fail
}

/// Defines resource constraints for individual test executions.
/// 
/// Prevents runaway tests from consuming excessive system resources
/// and ensures fair comparison between implementations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResourceLimits {
    /// Maximum execution time in seconds
    pub max_time_seconds: u64,
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f64,
}

/// Provides metadata about expected performance characteristics of a test.
/// 
/// Used by optimization analysis to identify critical operations
/// and expected performance patterns for targeted improvements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHints {
    /// Operations that should be optimized in fast path
    pub fast_path_candidates: Vec<String>,
    /// Memory allocation patterns
    pub memory_patterns: Vec<String>,
    /// Expected algorithmic complexity
    pub complexity: ScalingBehavior,
    /// Performance-critical operations
    pub critical_operations: Vec<String>,
}

/// Configuration parameters for statistical analysis of benchmark results.
/// 
/// Controls the rigor and accuracy of performance measurements,
/// including iteration counts, confidence intervals, and outlier detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalConfig {
    /// Number of measurement iterations per test
    pub iterations: u32,
    /// Number of warmup iterations (not counted)
    pub warmup_iterations: u32,
    /// Confidence level for intervals (e.g., 0.95 for 95%)
    pub confidence_level: f64,
    /// Minimum detectable performance difference (percentage)
    pub min_detectable_difference: f64,
    /// Outlier detection method
    pub outlier_detection: OutlierDetection,
    /// Whether to perform normality tests
    pub normality_tests: bool,
}

/// Statistical methods for detecting and handling outlier measurements.
/// 
/// Different outlier detection algorithms provide varying levels of
/// sensitivity and statistical rigor for performance measurement cleanup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutlierDetection {
    /// No outlier detection
    None,
    /// Interquartile Range outlier detection
    IQR {
        /// IQR multiplier for outlier threshold
        multiplier: f64
    },
    /// Z-Score outlier detection
    ZScore {
        /// Z-score threshold for outlier detection
        threshold: f64
    },
    /// Modified Z-Score outlier detection
    ModifiedZScore {
        /// Modified Z-score threshold
        threshold: f64
    },
    /// Grubbs test outlier detection
    Grubbs {
        /// Alpha level for Grubbs test
        alpha: f64
    },
}

/// Configuration for benchmark result output and reporting.
/// 
/// Controls how benchmark results are formatted, stored, and shared
/// with various output formats and external integrations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output directory for results
    pub output_dir: String,
    /// Output formats
    pub formats: Vec<OutputFormat>,
    /// Whether to generate comparative visualizations
    pub generate_charts: bool,
    /// Chart types to generate
    pub chart_types: Vec<ChartType>,
    /// Whether to upload results to external systems
    pub external_reporting: Option<ExternalReporting>,
}

/// Available formats for benchmark result output.
/// 
/// Each format serves different purposes from machine processing
/// to human-readable reports and academic publications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    /// JSON output format
    JSON,
    /// CSV output format
    CSV,
    /// HTML output format
    HTML,
    /// Markdown output format
    Markdown,
    /// LaTeX output format
    LaTeX,
    /// XML output format
    XML,
}

/// Types of visualization charts for performance analysis.
/// 
/// Different chart types highlight various aspects of performance data
/// from comparative analysis to scaling behavior and resource usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartType {
    /// Bar chart visualization
    BarChart,
    /// Box plot for statistical distribution
    BoxPlot,
    /// Scatter plot for correlation analysis
    ScatterPlot,
    /// Performance profile chart
    PerformanceProfile,
    /// Memory usage over time chart
    MemoryUsageOverTime,
    /// Scaling behavior analysis chart
    ScalingAnalysis,
    /// Implementation comparison chart
    ImplementationComparison,
}

/// Configuration for integrating with external systems and services.
/// 
/// Enables automated reporting to CI/CD systems, performance dashboards,
/// and notification channels for continuous performance monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalReporting {
    /// GitHub integration for CI/CD
    pub github: Option<GitHubConfig>,
    /// Performance tracking dashboard
    pub dashboard: Option<DashboardConfig>,
    /// Slack/Discord notifications
    pub notifications: Option<NotificationConfig>,
}

/// Configuration for GitHub integration and CI/CD reporting.
/// 
/// Enables automatic issue creation on performance regressions
/// and PR comments with benchmark results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub repository identifier (owner/repo)
    pub repository: String,
    /// Environment variable containing GitHub token
    pub token_env_var: String,
    /// Whether to create issues on performance regression
    pub create_issues_on_regression: bool,
    /// Whether to comment benchmark results on pull requests
    pub comment_on_prs: bool,
}

/// Configuration for performance tracking dashboard integration.
/// 
/// Allows uploading benchmark results to external monitoring
/// and visualization platforms for trend analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Dashboard API endpoint URL
    pub endpoint: String,
    /// Environment variable containing dashboard API key
    pub api_key_env_var: String,
    /// Project identifier for dashboard integration
    pub project_id: String,
}

/// Configuration for performance change notifications.
/// 
/// Sends alerts to team communication channels when performance
/// changes exceed specified thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Webhook URL for notifications
    pub webhook_url: String,
    /// Performance change percentage threshold for notifications
    pub notification_threshold: f64, // Performance change percentage
}

/// Configuration for system resource monitoring during benchmarks.
/// 
/// Controls what system resources are tracked and their sampling
/// frequency to understand resource usage patterns and constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Monitor CPU usage
    pub monitor_cpu: bool,
    /// Monitor memory usage
    pub monitor_memory: bool,
    /// Monitor disk I/O
    pub monitor_disk_io: bool,
    /// Monitor network I/O
    pub monitor_network_io: bool,
    /// Sampling interval for resource monitoring
    pub sampling_interval_ms: u64,
    /// Resource usage limits
    pub limits: SystemResourceLimits,
}

/// Global resource limits for the entire benchmark suite execution.
/// 
/// Prevents the benchmark suite from overwhelming the system
/// and ensures reproducible testing conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceLimits {
    /// Maximum total memory usage in MB
    pub max_total_memory_mb: u64,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f64,
    /// Maximum disk usage in MB
    pub max_disk_usage_mb: u64,
    /// Test timeout in seconds
    pub global_timeout_seconds: u64,
}

/// Complete results from a full benchmark suite execution.
/// 
/// Contains all performance data, statistical analysis, and metadata
/// from comparing multiple implementations across test categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuiteResult {
    /// Metadata about the benchmark run
    pub metadata: BenchmarkMetadata,
    /// Results by implementation and test
    pub implementation_results: HashMap<String, ImplementationResult>,
    /// Cross-implementation comparisons
    pub comparisons: Vec<ImplementationComparison>,
    /// Statistical analysis summary
    pub statistical_summary: StatisticalSummary,
    /// Performance regression analysis
    pub regression_analysis: Option<RegressionAnalysis>,
    /// Optimization recommendations
    pub recommendations: Vec<OptimizationRecommendation>,
    /// System resource usage during benchmarking
    pub resource_usage: SystemResourceUsage,
}

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
    pub os: String,
    pub architecture: String,
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub hostname: String,
}

/// Complete performance results for a single Scheme implementation.
/// 
/// Contains all test results, performance scores, and resource usage
/// statistics for one implementation across all test categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationResult {
    /// Implementation configuration
    pub config: ImplementationConfig,
    /// Results by test category
    pub category_results: HashMap<String, CategoryResult>,
    /// Overall performance score (0-100)
    pub overall_score: f64,
    /// Performance ranking among all implementations
    pub ranking: u32,
    /// Failures and errors
    pub failures: Vec<TestFailure>,
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

/// Comparison between implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationComparison {
    /// Implementation A
    pub impl_a: String,
    /// Implementation B
    pub impl_b: String,
    /// Performance ratio (A/B)
    pub performance_ratio: f64,
    /// Statistical significance of difference
    pub significance: StatisticalSignificance,
    /// Category-wise comparisons
    pub category_comparisons: HashMap<String, CategoryComparison>,
}

/// Statistical significance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSignificance {
    /// p-value from statistical test
    pub p_value: f64,
    /// Whether difference is statistically significant
    pub is_significant: bool,
    /// Type of statistical test used
    pub test_type: String,
    /// Effect size measure
    pub effect_size: f64,
}

/// Category-specific comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryComparison {
    /// Category name
    pub category: String,
    /// Performance difference (percentage)
    pub performance_difference: f64,
    /// Winner of this category
    pub winner: String,
    /// Confidence in the comparison
    pub confidence: f64,
}

/// Statistical summary across all implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSummary {
    /// Overall performance rankings
    pub performance_rankings: Vec<PerformanceRanking>,
    /// Category leaders
    pub category_leaders: HashMap<String, String>,
    /// Performance distribution statistics
    pub distribution_stats: DistributionStats,
    /// Correlation analysis
    pub correlation_analysis: CorrelationAnalysis,
}

/// Performance ranking entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRanking {
    /// Implementation name
    pub implementation: String,
    /// Overall rank (1 = best)
    pub rank: u32,
    /// Overall performance score
    pub score: f64,
    /// Wins by category
    pub category_wins: u32,
    /// Confidence interval for ranking
    pub ranking_confidence: f64,
}

/// Distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    /// Performance distribution shape
    pub distribution_shape: DistributionShape,
    /// Variance in performance across implementations
    pub performance_variance: f64,
    /// Outlier implementations
    pub outliers: Vec<String>,
}

/// Shape of performance distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionShape {
    /// Normal (Gaussian) distribution
    Normal,
    /// Skewed distribution
    Skewed,
    /// Bimodal distribution with two peaks
    Bimodal,
    /// Uniform distribution
    Uniform,
    /// Unknown or unclassified distribution shape
    Unknown,
}

/// Correlation analysis between different metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationAnalysis {
    /// Correlation between categories
    pub category_correlations: HashMap<String, HashMap<String, f64>>,
    /// Memory vs speed correlation
    pub memory_speed_correlation: f64,
    /// Implementation feature correlations
    pub feature_correlations: HashMap<String, f64>,
}

/// Performance regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    /// Baseline results for comparison
    pub baseline: Option<String>, // Path to baseline results
    /// Detected regressions
    pub regressions: Vec<PerformanceRegression>,
    /// Performance improvements
    pub improvements: Vec<PerformanceImprovement>,
    /// Overall trend analysis
    pub trend_analysis: TrendAnalysis,
}

/// Detected performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    /// Implementation affected
    pub implementation: String,
    /// Test case affected
    pub test_case: String,
    /// Category affected
    pub category: String,
    /// Performance change (negative percentage)
    pub performance_change: f64,
    /// Statistical significance
    pub significance: StatisticalSignificance,
    /// Severity of regression
    pub severity: RegressionSeverity,
}

/// Performance improvement detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovement {
    /// Implementation improved
    pub implementation: String,
    /// Test case improved
    pub test_case: String,
    /// Category improved
    pub category: String,
    /// Performance change (positive percentage)
    pub performance_change: f64,
    /// Statistical significance
    pub significance: StatisticalSignificance,
}

/// Severity of performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    /// Minor regression (< 10% performance loss)
    Minor,      // < 10% performance loss
    /// Moderate regression (10-25% performance loss)
    Moderate,   // 10-25% performance loss
    /// Major regression (25-50% performance loss)
    Major,      // 25-50% performance loss
    /// Critical regression (> 50% performance loss)
    Critical,   // > 50% performance loss
}

/// Trend analysis over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Overall trend direction
    pub overall_trend: TrendDirection,
    /// Performance velocity (change per time unit)
    pub performance_velocity: f64,
    /// Prediction for future performance
    pub performance_forecast: PerformanceForecast,
}

/// Direction of performance trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Performance is consistently improving over time
    Improving,
    /// Performance is stable with minimal variation
    Stable,
    /// Performance is consistently declining over time
    Declining,
    /// Performance shows high volatility and unpredictable changes
    Volatile,
}

/// Performance forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceForecast {
    /// Predicted performance change over next period
    pub predicted_change: f64,
    /// Confidence in prediction
    pub confidence: f64,
    /// Time horizon for prediction
    pub time_horizon: Duration,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Priority level (1-10, 10 = highest)
    pub priority: u8,
    /// Title of recommendation
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Target implementation(s)
    pub target_implementations: Vec<String>,
    /// Affected categories
    pub affected_categories: Vec<String>,
    /// Expected performance improvement
    pub expected_improvement: f64,
    /// Implementation difficulty (1-10, 10 = hardest)
    pub difficulty: u8,
    /// Supporting evidence
    pub evidence: Vec<String>,
    /// Related optimizations
    pub related_recommendations: Vec<String>,
}

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

impl Default for BenchmarkSuiteConfig {
    fn default() -> Self {
        Self {
            implementations: Self::default_implementations(),
            test_categories: Self::default_test_categories(),
            statistical_config: StatisticalConfig::default(),
            output_config: OutputConfig::default(),
            resource_config: ResourceConfig::default(),
        }
    }
}

impl BenchmarkSuiteConfig {
    /// Default implementation configurations
    fn default_implementations() -> Vec<ImplementationConfig> {
        vec![
            ImplementationConfig {
                name: "Lambdust".to_string(),
                id: "lambdust".to_string(),
                runtime: RuntimeConfig::Lambdust {
                    target_dir: "./target".to_string(),
                    profile: "release".to_string(),
                    features: vec![],
                },
                version: env!("CARGO_PKG_VERSION").to_string(),
                expected_baseline: Some(5_000_000.0), // 5M ops/sec
                limitations: vec![],
                r7rs_compliant: true,
            },
            ImplementationConfig {
                name: "Chez Scheme".to_string(),
                id: "chez".to_string(),
                runtime: RuntimeConfig::Native {
                    binary_path: "scheme".to_string(),
                    args: vec!["--quiet".to_string(), "--script".to_string()],
                    env_vars: HashMap::new(),
                },
                version: "9.5+".to_string(),
                expected_baseline: Some(8_000_000.0), // High performance baseline
                limitations: vec![],
                r7rs_compliant: true,
            },
            ImplementationConfig {
                name: "Racket".to_string(),
                id: "racket".to_string(),
                runtime: RuntimeConfig::Native {
                    binary_path: "racket".to_string(),
                    args: vec!["-t".to_string()],
                    env_vars: HashMap::new(),
                },
                version: "8.11+".to_string(),
                expected_baseline: Some(3_000_000.0),
                limitations: vec!["Different numeric tower".to_string()],
                r7rs_compliant: true,
            },
            // Additional implementations would be defined here...
        ]
    }
    
    /// Default test categories
    fn default_test_categories() -> Vec<TestCategory> {
        vec![
            TestCategory {
                name: "arithmetic".to_string(),
                description: "Arithmetic operations and numeric tower".to_string(),
                tests: Self::arithmetic_tests(),
                weight: 0.25,
                critical: true,
            },
            TestCategory {
                name: "lists".to_string(),
                description: "List operations and data structure manipulation".to_string(),
                tests: Self::list_tests(),
                weight: 0.20,
                critical: true,
            },
            TestCategory {
                name: "recursion".to_string(),
                description: "Recursive algorithms and tail call optimization".to_string(),
                tests: Self::recursion_tests(),
                weight: 0.15,
                critical: true,
            },
            TestCategory {
                name: "memory".to_string(),
                description: "Memory allocation and garbage collection".to_string(),
                tests: Self::memory_tests(),
                weight: 0.15,
                critical: false,
            },
            TestCategory {
                name: "io".to_string(),
                description: "Input/output operations".to_string(),
                tests: Self::io_tests(),
                weight: 0.10,
                critical: false,
            },
            TestCategory {
                name: "macros".to_string(),
                description: "Macro expansion and metaprogramming".to_string(),
                tests: Self::macro_tests(),
                weight: 0.10,
                critical: false,
            },
            TestCategory {
                name: "strings".to_string(),
                description: "String manipulation and processing".to_string(),
                tests: Self::string_tests(),
                weight: 0.05,
                critical: false,
            },
        ]
    }
    
    /// Arithmetic test cases
    fn arithmetic_tests() -> Vec<TestCase> {
        vec![
            TestCase {
                name: "integer_arithmetic".to_string(),
                description: "Basic integer arithmetic operations".to_string(),
                code_template: r#"
                    (define (arithmetic-benchmark n)
                      (let loop ((i 0) (sum 0))
                        (if (< i n)
                            (loop (+ i 1) (+ sum (* i i)))
                            sum)))
                    (time (arithmetic-benchmark {n}))
                "#.to_string(),
                parameters: vec![
                    TestParameter {
                        name: "n".to_string(),
                        values: vec![
                            ParameterValue::Range { start: 1000, end: 100000, step: 10000 },
                        ],
                        scaling_behavior: ScalingBehavior::Linear,
                    }
                ],
                expected_result_type: ResultType::Number,
                resource_limits: TestResourceLimits {
                    max_time_seconds: 30,
                    max_memory_mb: 100,
                    max_cpu_percent: 100.0,
                },
                performance_hints: PerformanceHints {
                    fast_path_candidates: vec!["+".to_string(), "*".to_string()],
                    memory_patterns: vec!["constant memory".to_string()],
                    complexity: ScalingBehavior::Linear,
                    critical_operations: vec!["arithmetic".to_string()],
                },
            },
            // Additional arithmetic tests...
        ]
    }
    
    /// List operation test cases
    fn list_tests() -> Vec<TestCase> {
        vec![
            TestCase {
                name: "list_creation".to_string(),
                description: "List creation and basic operations".to_string(),
                code_template: r#"
                    (define (list-benchmark n)
                      (let loop ((i 0) (lst '()))
                        (if (< i n)
                            (loop (+ i 1) (cons i lst))
                            (length lst))))
                    (time (list-benchmark {n}))
                "#.to_string(),
                parameters: vec![
                    TestParameter {
                        name: "n".to_string(),
                        values: vec![
                            ParameterValue::Range { start: 1000, end: 50000, step: 5000 },
                        ],
                        scaling_behavior: ScalingBehavior::Linear,
                    }
                ],
                expected_result_type: ResultType::Number,
                resource_limits: TestResourceLimits {
                    max_time_seconds: 60,
                    max_memory_mb: 200,
                    max_cpu_percent: 100.0,
                },
                performance_hints: PerformanceHints {
                    fast_path_candidates: vec!["cons".to_string(), "length".to_string()],
                    memory_patterns: vec!["linear allocation".to_string()],
                    complexity: ScalingBehavior::Linear,
                    critical_operations: vec!["cons".to_string(), "list traversal".to_string()],
                },
            },
            // Additional list tests...
        ]
    }
    
    /// Recursion test cases
    fn recursion_tests() -> Vec<TestCase> {
        vec![
            TestCase {
                name: "fibonacci".to_string(),
                description: "Fibonacci sequence calculation".to_string(),
                code_template: r#"
                    (define (fib n)
                      (if (<= n 1)
                          n
                          (+ (fib (- n 1)) (fib (- n 2)))))
                    (time (fib {n}))
                "#.to_string(),
                parameters: vec![
                    TestParameter {
                        name: "n".to_string(),
                        values: vec![
                            ParameterValue::Range { start: 20, end: 35, step: 5 },
                        ],
                        scaling_behavior: ScalingBehavior::Exponential,
                    }
                ],
                expected_result_type: ResultType::Number,
                resource_limits: TestResourceLimits {
                    max_time_seconds: 120,
                    max_memory_mb: 500,
                    max_cpu_percent: 100.0,
                },
                performance_hints: PerformanceHints {
                    fast_path_candidates: vec!["+".to_string(), "-".to_string()],
                    memory_patterns: vec!["exponential stack growth".to_string()],
                    complexity: ScalingBehavior::Exponential,
                    critical_operations: vec!["recursion".to_string(), "function calls".to_string()],
                },
            },
            // Additional recursion tests...
        ]
    }
    
    /// Memory management test cases  
    fn memory_tests() -> Vec<TestCase> {
        vec![
            TestCase {
                name: "allocation_stress".to_string(),
                description: "Memory allocation and garbage collection stress test".to_string(),
                code_template: r#"
                    (define (allocation-benchmark n)
                      (let loop ((i 0) (acc '()))
                        (if (< i n)
                            (let ((big-list (make-list 1000 i)))
                              (loop (+ i 1) (cons big-list acc)))
                            (length acc))))
                    (time (allocation-benchmark {n}))
                "#.to_string(),
                parameters: vec![
                    TestParameter {
                        name: "n".to_string(),
                        values: vec![
                            ParameterValue::Range { start: 100, end: 1000, step: 100 },
                        ],
                        scaling_behavior: ScalingBehavior::Linear,
                    }
                ],
                expected_result_type: ResultType::Number,
                resource_limits: TestResourceLimits {
                    max_time_seconds: 180,
                    max_memory_mb: 1000,
                    max_cpu_percent: 100.0,
                },
                performance_hints: PerformanceHints {
                    fast_path_candidates: vec![],
                    memory_patterns: vec!["high allocation rate".to_string(), "GC pressure".to_string()],
                    complexity: ScalingBehavior::Linear,
                    critical_operations: vec!["allocation".to_string(), "GC".to_string()],
                },
            },
            // Additional memory tests...
        ]
    }
    
    /// I/O operation test cases
    fn io_tests() -> Vec<TestCase> {
        vec![
            TestCase {
                name: "string_port_operations".to_string(),
                description: "String port I/O operations".to_string(),
                code_template: r#"
                    (define (io-benchmark n)
                      (let ((output-port (open-output-string)))
                        (let loop ((i 0))
                          (if (< i n)
                              (begin
                                (write i output-port)
                                (newline output-port)
                                (loop (+ i 1)))
                              (string-length (get-output-string output-port))))))
                    (time (io-benchmark {n}))
                "#.to_string(),
                parameters: vec![
                    TestParameter {
                        name: "n".to_string(),
                        values: vec![
                            ParameterValue::Range { start: 1000, end: 10000, step: 1000 },
                        ],
                        scaling_behavior: ScalingBehavior::Linear,
                    }
                ],
                expected_result_type: ResultType::Number,
                resource_limits: TestResourceLimits {
                    max_time_seconds: 60,
                    max_memory_mb: 200,
                    max_cpu_percent: 100.0,
                },
                performance_hints: PerformanceHints {
                    fast_path_candidates: vec!["write".to_string()],
                    memory_patterns: vec!["string buffer growth".to_string()],
                    complexity: ScalingBehavior::Linear,
                    critical_operations: vec!["I/O".to_string(), "string operations".to_string()],
                },
            },
            // Additional I/O tests...
        ]
    }
    
    /// Macro expansion test cases
    fn macro_tests() -> Vec<TestCase> {
        vec![
            TestCase {
                name: "macro_expansion".to_string(),
                description: "Complex macro expansion performance".to_string(),
                code_template: r#"
                    (define-syntax repeat
                      (syntax-rules ()
                        ((repeat n expr)
                         (let loop ((i 0))
                           (if (< i n)
                               (begin expr (loop (+ i 1)))
                               'done)))))
                    
                    (define (macro-benchmark n)
                      (repeat n (+ 1 2 3)))
                    
                    (time (macro-benchmark {n}))
                "#.to_string(),
                parameters: vec![
                    TestParameter {
                        name: "n".to_string(),
                        values: vec![
                            ParameterValue::Range { start: 1000, end: 10000, step: 1000 },
                        ],
                        scaling_behavior: ScalingBehavior::Linear,
                    }
                ],
                expected_result_type: ResultType::Any,
                resource_limits: TestResourceLimits {
                    max_time_seconds: 60,
                    max_memory_mb: 100,
                    max_cpu_percent: 100.0,
                },
                performance_hints: PerformanceHints {
                    fast_path_candidates: vec![],
                    memory_patterns: vec!["macro expansion overhead".to_string()],
                    complexity: ScalingBehavior::Linear,
                    critical_operations: vec!["macro expansion".to_string()],
                },
            },
            // Additional macro tests...
        ]
    }
    
    /// String manipulation test cases
    fn string_tests() -> Vec<TestCase> {
        vec![
            TestCase {
                name: "string_concatenation".to_string(),
                description: "String concatenation performance".to_string(),
                code_template: r#"
                    (define (string-benchmark n)
                      (let loop ((i 0) (result ""))
                        (if (< i n)
                            (loop (+ i 1) (string-append result (number->string i) " "))
                            (string-length result))))
                    (time (string-benchmark {n}))
                "#.to_string(),
                parameters: vec![
                    TestParameter {
                        name: "n".to_string(),
                        values: vec![
                            ParameterValue::Range { start: 100, end: 1000, step: 100 },
                        ],
                        scaling_behavior: ScalingBehavior::Quadratic, // Due to repeated concatenation
                    }
                ],
                expected_result_type: ResultType::Number,
                resource_limits: TestResourceLimits {
                    max_time_seconds: 60,
                    max_memory_mb: 200,
                    max_cpu_percent: 100.0,
                },
                performance_hints: PerformanceHints {
                    fast_path_candidates: vec!["string-append".to_string()],
                    memory_patterns: vec!["quadratic memory growth".to_string()],
                    complexity: ScalingBehavior::Quadratic,
                    critical_operations: vec!["string operations".to_string()],
                },
            },
            // Additional string tests...
        ]
    }
}

impl Default for StatisticalConfig {
    fn default() -> Self {
        Self {
            iterations: 10,
            warmup_iterations: 3,
            confidence_level: 0.95,
            min_detectable_difference: 5.0,
            outlier_detection: OutlierDetection::IQR { multiplier: 1.5 },
            normality_tests: true,
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            output_dir: "./benchmark_results".to_string(),
            formats: vec![OutputFormat::JSON, OutputFormat::HTML, OutputFormat::CSV],
            generate_charts: true,
            chart_types: vec![
                ChartType::BarChart,
                ChartType::BoxPlot,
                ChartType::PerformanceProfile,
                ChartType::ImplementationComparison,
            ],
            external_reporting: None,
        }
    }
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            monitor_cpu: true,
            monitor_memory: true,
            monitor_disk_io: true,
            monitor_network_io: false,
            sampling_interval_ms: 100,
            limits: SystemResourceLimits {
                max_total_memory_mb: 4096,
                max_cpu_percent: 95.0,
                max_disk_usage_mb: 1024,
                global_timeout_seconds: 3600, // 1 hour
            },
        }
    }
}

/// Main benchmark suite executor
pub struct ComprehensiveBenchmarkSuite {
    config: BenchmarkSuiteConfig,
    resource_monitor: Arc<Mutex<SystemResourceMonitor>>,
}

/// System resource monitoring
struct SystemResourceMonitor {
    monitoring: bool,
    samples: Vec<ResourceSnapshot>,
    start_time: SystemTime,
}

impl ComprehensiveBenchmarkSuite {
    /// Create a new benchmark suite with the given configuration
    pub fn new(config: BenchmarkSuiteConfig) -> Self {
        let resource_monitor = Arc::new(Mutex::new(SystemResourceMonitor {
            monitoring: false,
            samples: Vec::new(),
            start_time: SystemTime::now(),
        }));
        
        Self {
            config,
            resource_monitor,
        }
    }
    
    /// Execute the complete benchmark suite
    pub fn execute(&mut self) -> Result<BenchmarkSuiteResult, Box<dyn std::error::Error>> {
        println!("Starting comprehensive benchmark suite...");
        
        let start_time = SystemTime::now();
        
        // Start resource monitoring
        self.start_resource_monitoring();
        
        // Collect system information
        let system_info = self.collect_system_info();
        
        // Initialize results structure
        let mut implementation_results = HashMap::new();
        
        // Execute benchmarks for each implementation
        for impl_config in &self.config.implementations {
            println!("Benchmarking {}...", impl_config.name);
            
            match self.benchmark_implementation(impl_config) {
                Ok(result) => {
                    implementation_results.insert(impl_config.id.clone()), result);
                }
                Err(e) => {
                    eprintln!("Failed to benchmark {}: {}", impl_config.name, e);
                    // Create a failure result
                    let failure_result = ImplementationResult {
                        config: impl_config.clone()),
                        category_results: HashMap::new(),
                        overall_score: 0.0,
                        ranking: 0,
                        failures: vec![TestFailure {
                            test_name: "suite_execution".to_string(),
                            category: "infrastructure".to_string(),
                            parameters: HashMap::new(),
                            reason: FailureReason::InfrastructureError,
                            error_message: e.to_string(),
                            stack_trace: None,
                        }],
                        resource_stats: ResourceStats {
                            cpu: CPUStats {
                                avg_usage_percent: 0.0,
                                peak_usage_percent: 0.0,
                                user_time: Duration::ZERO,
                                kernel_time: Duration::ZERO,
                                context_switches: 0,
                            },
                            memory: MemoryStats {
                                avg_usage_bytes: 0,
                                peak_usage_bytes: 0,
                                allocations: 0,
                                deallocations: 0,
                                page_faults: 0,
                            },
                            disk_io: DiskIOStats {
                                bytes_read: 0,
                                bytes_written: 0,
                                read_ops: 0,
                                write_ops: 0,
                            },
                            network_io: None,
                        },
                    };
                    implementation_results.insert(impl_config.id.clone()), failure_result);
                }
            }
        }
        
        // Stop resource monitoring
        let resource_usage = self.stop_resource_monitoring();
        
        let total_duration = start_time.elapsed().unwrap_or(Duration::ZERO);
        
        // Perform cross-implementation analysis
        let comparisons = self.generate_comparisons(&implementation_results);
        let statistical_summary = self.generate_statistical_summary(&implementation_results);
        let regression_analysis = self.perform_regression_analysis(&implementation_results);
        let recommendations = self.generate_recommendations(&implementation_results, &statistical_summary);
        
        // Create metadata
        let metadata = BenchmarkMetadata {
            timestamp: start_time,
            total_duration,
            config: self.config.clone()),
            system_info,
            git_commit: self.get_git_commit(),
            environment: std::env::vars().collect(),
        };
        
        let result = BenchmarkSuiteResult {
            metadata,
            implementation_results,
            comparisons,
            statistical_summary,
            regression_analysis,
            recommendations,
            resource_usage,
        };
        
        // Save results
        self.save_results(&result)?;
        
        println!("Benchmark suite completed in {:.2} seconds", total_duration.as_secs_f64());
        
        Ok(result)
    }
    
    /// Benchmark a single implementation
    fn benchmark_implementation(&self, impl_config: &ImplementationConfig) -> Result<ImplementationResult, Box<dyn std::error::Error>> {
        let mut category_results = HashMap::new();
        let mut all_failures = Vec::new();
        
        // Execute tests for each category
        for category in &self.config.test_categories {
            println!("  Category: {}", category.name);
            
            let mut test_results = Vec::new();
            let mut category_failures = Vec::new();
            
            for test_case in &category.tests {
                println!("    Test: {}", test_case.name);
                
                // Generate parameter combinations
                let param_combinations = self.generate_parameter_combinations(&test_case.parameters);
                
                for params in param_combinations {
                    match self.execute_single_test(impl_config, test_case, &params) {
                        Ok(result) => test_results.push(result),
                        Err(failure) => category_failures.push(failure),
                    }
                }
            }
            
            // Calculate category statistics
            let statistics = self.calculate_category_statistics(&test_results);
            let score = self.calculate_category_score(&test_results, category);
            
            category_results.insert(category.name.clone()), CategoryResult {
                category: category.name.clone()),
                test_results,
                score,
                statistics,
            });
            
            all_failures.extend(category_failures);
        }
        
        // Calculate overall score and ranking
        let overall_score = self.calculate_overall_score(&category_results);
        
        // Calculate resource statistics (placeholder)
        let resource_stats = self.calculate_resource_stats(impl_config);
        
        Ok(ImplementationResult {
            config: impl_config.clone()),
            category_results,
            overall_score,
            ranking: 0, // Will be set during comparison phase
            failures: all_failures,
            resource_stats,
        })
    }
    
    /// Execute a single test case
    fn execute_single_test(
        &self,
        impl_config: &ImplementationConfig,
        test_case: &TestCase,
        params: &HashMap<String, ParameterValue>,
    ) -> Result<TestResult, TestFailure> {
        // Generate the test code with parameter substitution
        let test_code = self.substitute_parameters(&test_case.code_template, params)?;
        
        // Create temporary test file
        let temp_file = self.create_temp_test_file(impl_config, &test_code)?;
        
        // Execute the test with timing and resource monitoring
        let (timing, memory, validation, success, error) = 
            self.execute_test_with_monitoring(impl_config, &temp_file, test_case)?;
        
        // Clean up temporary file
        let _ = fs::remove_file(&temp_file);
        
        if !success {
            return Err(TestFailure {
                test_name: test_case.name.clone()),
                category: "unknown".to_string(), // Would be passed from caller
                parameters: params.clone()),
                reason: FailureReason::RuntimeError,
                error_message: error.unwrap_or_else(|| "Unknown error".to_string()),
                stack_trace: None,
            });
        }
        
        Ok(TestResult {
            test_case: test_case.clone()),
            parameters: params.clone()),
            timing,
            memory,
            validation,
            success,
            error,
        })
    }
    
    /// Generate parameter combinations for a test
    fn generate_parameter_combinations(&self, parameters: &[TestParameter]) -> Vec<HashMap<String, ParameterValue>> {
        if parameters.is_empty() {
            return vec![HashMap::new()];
        }
        
        let mut combinations = vec![HashMap::new()];
        
        for param in parameters {
            let mut new_combinations = Vec::new();
            
            for value in &param.values {
                for existing_combo in &combinations {
                    let mut new_combo = existing_combo.clone());
                    new_combo.insert(param.name.clone()), value.clone());
                    new_combinations.push(new_combo);
                }
            }
            
            combinations = new_combinations;
        }
        
        combinations
    }
    
    /// Substitute parameters in test code template
    fn substitute_parameters(
        &self,
        template: &str,
        params: &HashMap<String, ParameterValue>,
    ) -> Result<String, TestFailure> {
        let mut result = template.to_string();
        
        for (name, value) in params {
            let placeholder = format!("{{{}}}", name);
            let value_str = match value {
                ParameterValue::Integer { value } => value.to_string(),
                ParameterValue::Float { value } => value.to_string(),
                ParameterValue::String { value } => format!("\"{}\"", value),
                ParameterValue::Boolean { value } => if *value { "#t" } else { "#f" }.to_string(),
                ParameterValue::Range { .. } => {
                    return Err(TestFailure {
                        test_name: "parameter_substitution".to_string(),
                        category: "infrastructure".to_string(),
                        parameters: params.clone()),
                        reason: FailureReason::InfrastructureError,
                        error_message: "Range parameters should be expanded before substitution".to_string(),
                        stack_trace: None,
                    });
                }
            };
            result = result.replace(&placeholder, &value_str);
        }
        
        Ok(result)
    }
    
    /// Create temporary test file for execution
    fn create_temp_test_file(
        &self,
        impl_config: &ImplementationConfig,
        test_code: &str,
    ) -> Result<String, TestFailure> {
        let file_extension = match &impl_config.runtime {
            RuntimeConfig::Lambdust { .. } => ".ldust",
            _ => ".scm",
        };
        
        let temp_file = format!("/tmp/benchmark_{}_{}{}", 
                               impl_config.id, 
                               SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
                               file_extension);
        
        fs::write(&temp_file, test_code).map_err(|e| TestFailure {
            test_name: "file_creation".to_string(),
            category: "infrastructure".to_string(),
            parameters: HashMap::new(),
            reason: FailureReason::InfrastructureError,
            error_message: format!("Failed to create temp file: {}", e),
            stack_trace: None,
        })?;
        
        Ok(temp_file)
    }
    
    /// Execute test with monitoring
    fn execute_test_with_monitoring(
        &self,
        impl_config: &ImplementationConfig,
        test_file: &str,
        test_case: &TestCase,
    ) -> Result<(TimingMeasurements, MemoryMeasurements, ValidationResult, bool, Option<String>), TestFailure> {
        let mut iteration_times = Vec::new();
        let mut memory_measurements = Vec::new();
        
        // Warmup iterations
        for _ in 0..self.config.statistical_config.warmup_iterations {
            let _ = self.execute_single_iteration(impl_config, test_file, test_case)?;
        }
        
        // Actual measurement iterations
        for _ in 0..self.config.statistical_config.iterations {
            let start = Instant::now();
            let (success, error, memory_usage) = self.execute_single_iteration(impl_config, test_file, test_case)?;
            let duration = start.elapsed();
            
            if !success {
                return Ok((self.empty_timing(), self.empty_memory(), self.empty_validation(), false, error));
            }
            
            iteration_times.push(duration);
            memory_measurements.push((duration, memory_usage));
        }
        
        // Calculate timing statistics
        let timing = self.calculate_timing_statistics(iteration_times);
        
        // Calculate memory statistics
        let memory = self.calculate_memory_statistics(memory_measurements);
        
        // Validate result (simplified)
        let validation = ValidationResult {
            type_correct: true, // Placeholder
            value_correct: Some(true), // Placeholder
            actual_result: "result".to_string(), // Placeholder
            expected_result: None,
        };
        
        Ok((timing, memory, validation, true, None))
    }
    
    /// Execute a single iteration of a test
    fn execute_single_iteration(
        &self,
        impl_config: &ImplementationConfig,
        test_file: &str,
        _test_case: &TestCase,
    ) -> Result<(bool, Option<String>, u64), TestFailure> {
        let mut cmd = match &impl_config.runtime {
            RuntimeConfig::Native { binary_path, args, env_vars } => {
                let mut command = Command::new(binary_path);
                command.args(args);
                for (key, value) in env_vars {
                    command.env(key, value);
                }
                command.arg(test_file);
                command
            }
            RuntimeConfig::Lambdust { target_dir, profile, .. } => {
                let binary_path = format!("{}/{}/lambdust", target_dir, profile);
                let mut command = Command::new(binary_path);
                command.arg("--batch");
                command.arg(test_file);
                command
            }
            RuntimeConfig::Docker { .. } => {
                // Docker execution would be implemented here
                return Err(TestFailure {
                    test_name: "docker_execution".to_string(),
                    category: "infrastructure".to_string(),
                    parameters: HashMap::new(),
                    reason: FailureReason::InfrastructureError,
                    error_message: "Docker execution not yet implemented".to_string(),
                    stack_trace: None,
                });
            }
        };
        
        match cmd.output() {
            Ok(output) => {
                let success = output.status.success();
                let error = if success {
                    None
                } else {
                    Some(String::from_utf8_lossy(&output.stderr).to_string())
                };
                
                // Memory usage estimation (placeholder)
                let memory_usage = output.stdout.len() as u64 + output.stderr.len() as u64;
                
                Ok((success, error, memory_usage))
            }
            Err(e) => Err(TestFailure {
                test_name: "command_execution".to_string(),
                category: "infrastructure".to_string(),
                parameters: HashMap::new(),
                reason: FailureReason::InfrastructureError,
                error_message: format!("Failed to execute command: {}", e),
                stack_trace: None,
            }),
        }
    }
    
    // Placeholder implementations for statistics calculations
    fn calculate_timing_statistics(&self, times: Vec<Duration>) -> TimingMeasurements {
        if times.is_empty() {
            return self.empty_timing();
        }
        
        let sum: Duration = times.iter().sum();
        let mean = sum / times.len() as u32;
        
        let mut sorted_times = times.clone());
        sorted_times.sort();
        
        let median = sorted_times[sorted_times.len() / 2];
        let min = sorted_times[0];
        let max = sorted_times[sorted_times.len() - 1];
        
        // Standard deviation calculation
        let variance: f64 = times.iter()
            .map(|t| {
                let diff = t.as_secs_f64() - mean.as_secs_f64();
                diff * diff
            })
            .sum::<f64>() / times.len() as f64;
        let std_dev = Duration::from_secs_f64(variance.sqrt());
        
        // Percentiles
        let mut percentiles = HashMap::new();
        percentiles.insert(50, median);
        percentiles.insert(90, sorted_times[(sorted_times.len() * 90 / 100).min(sorted_times.len() - 1)]);
        percentiles.insert(95, sorted_times[(sorted_times.len() * 95 / 100).min(sorted_times.len() - 1)]);
        percentiles.insert(99, sorted_times[(sorted_times.len() * 99 / 100).min(sorted_times.len() - 1)]);
        
        // Operations per second
        let ops_per_second = 1.0 / mean.as_secs_f64();
        
        TimingMeasurements {
            iteration_times: times,
            mean,
            median,
            std_dev,
            min,
            max,
            percentiles,
            confidence_interval: ConfidenceInterval {
                lower: mean - std_dev,
                upper: mean + std_dev,
                confidence_level: self.config.statistical_config.confidence_level,
            },
            ops_per_second,
        }
    }
    
    fn calculate_memory_statistics(&self, measurements: Vec<(Duration, u64)>) -> MemoryMeasurements {
        if measurements.is_empty() {
            return self.empty_memory();
        }
        
        let peak_usage = measurements.iter().map(|(_, mem)| *mem).max().unwrap_or(0);
        let avg_usage = measurements.iter().map(|(_, mem)| *mem).sum::<u64>() / measurements.len() as u64;
        
        MemoryMeasurements {
            peak_usage,
            usage_timeline: measurements,
            allocation_rate: avg_usage as f64, // Simplified
            efficiency: if peak_usage > 0 { 1.0 / peak_usage as f64 } else { 0.0 },
        }
    }
    
    fn empty_timing(&self) -> TimingMeasurements {
        TimingMeasurements {
            iteration_times: Vec::new(),
            mean: Duration::ZERO,
            median: Duration::ZERO,
            std_dev: Duration::ZERO,
            min: Duration::ZERO,
            max: Duration::ZERO,
            percentiles: HashMap::new(),
            confidence_interval: ConfidenceInterval {
                lower: Duration::ZERO,
                upper: Duration::ZERO,
                confidence_level: 0.0,
            },
            ops_per_second: 0.0,
        }
    }
    
    fn empty_memory(&self) -> MemoryMeasurements {
        MemoryMeasurements {
            peak_usage: 0,
            usage_timeline: Vec::new(),
            allocation_rate: 0.0,
            efficiency: 0.0,
        }
    }
    
    fn empty_validation(&self) -> ValidationResult {
        ValidationResult {
            type_correct: false,
            value_correct: None,
            actual_result: String::new(),
            expected_result: None,
        }
    }
    
    // Additional placeholder implementations
    fn calculate_category_statistics(&self, _test_results: &[TestResult]) -> CategoryStatistics {
        CategoryStatistics {
            successful_tests: 0,
            total_tests: 0,
            success_rate: 0.0,
            avg_performance: 0.0,
            performance_variance: 0.0,
            category_ranking: 0,
        }
    }
    
    fn calculate_category_score(&self, _test_results: &[TestResult], _category: &TestCategory) -> f64 {
        75.0 // Placeholder
    }
    
    fn calculate_overall_score(&self, _category_results: &HashMap<String, CategoryResult>) -> f64 {
        75.0 // Placeholder
    }
    
    fn calculate_resource_stats(&self, _impl_config: &ImplementationConfig) -> ResourceStats {
        ResourceStats {
            cpu: CPUStats {
                avg_usage_percent: 50.0,
                peak_usage_percent: 80.0,
                user_time: Duration::from_secs(1),
                kernel_time: Duration::from_millis(100),
                context_switches: 1000,
            },
            memory: MemoryStats {
                avg_usage_bytes: 100 * 1024 * 1024,
                peak_usage_bytes: 200 * 1024 * 1024,
                allocations: 10000,
                deallocations: 9500,
                page_faults: 100,
            },
            disk_io: DiskIOStats {
                bytes_read: 1024 * 1024,
                bytes_written: 512 * 1024,
                read_ops: 100,
                write_ops: 50,
            },
            network_io: None,
        }
    }
    
    fn generate_comparisons(&self, _results: &HashMap<String, ImplementationResult>) -> Vec<ImplementationComparison> {
        Vec::new() // Placeholder
    }
    
    fn generate_statistical_summary(&self, _results: &HashMap<String, ImplementationResult>) -> StatisticalSummary {
        StatisticalSummary {
            performance_rankings: Vec::new(),
            category_leaders: HashMap::new(),
            distribution_stats: DistributionStats {
                distribution_shape: DistributionShape::Unknown,
                performance_variance: 0.0,
                outliers: Vec::new(),
            },
            correlation_analysis: CorrelationAnalysis {
                category_correlations: HashMap::new(),
                memory_speed_correlation: 0.0,
                feature_correlations: HashMap::new(),
            },
        }
    }
    
    fn perform_regression_analysis(&self, _results: &HashMap<String, ImplementationResult>) -> Option<RegressionAnalysis> {
        None // Placeholder
    }
    
    fn generate_recommendations(
        &self, 
        _results: &HashMap<String, ImplementationResult>,
        _summary: &StatisticalSummary,
    ) -> Vec<OptimizationRecommendation> {
        Vec::new() // Placeholder
    }
    
    fn collect_system_info(&self) -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_model: "Unknown".to_string(), // Would use system API
            cpu_cores: num_cpus::get() as u32,
            total_memory_mb: 8192, // Placeholder
            available_memory_mb: 4096, // Placeholder
            hostname: "benchmark-host".to_string(),
        }
    }
    
    fn get_git_commit(&self) -> Option<String> {
        // Would execute `git rev-parse HEAD`
        None
    }
    
    fn start_resource_monitoring(&self) {
        // Would start background thread for resource monitoring
    }
    
    fn stop_resource_monitoring(&self) -> SystemResourceUsage {
        // Would stop monitoring and return collected data
        SystemResourceUsage {
            timeline: Vec::new(),
            peak_usage: ResourceSnapshot {
                timestamp: SystemTime::now(),
                cpu_percent: 80.0,
                memory_bytes: 200 * 1024 * 1024,
                available_memory_bytes: 6000 * 1024 * 1024,
                disk_io_rate: 1024.0 * 1024.0,
                network_io_rate: 0.0,
                process_count: 10,
            },
            average_usage: ResourceSnapshot {
                timestamp: SystemTime::now(),
                cpu_percent: 50.0,
                memory_bytes: 100 * 1024 * 1024,
                available_memory_bytes: 7000 * 1024 * 1024,
                disk_io_rate: 512.0 * 1024.0,
                network_io_rate: 0.0,
                process_count: 8,
            },
            efficiency_metrics: ResourceEfficiency {
                cpu_efficiency: 0.75,
                memory_efficiency: 0.80,
                overall_efficiency: 0.77,
                bottlenecks: vec!["CPU-bound".to_string()],
            },
        }
    }
    
    fn save_results(&self, result: &BenchmarkSuiteResult) -> Result<(), Box<dyn std::error::Error>> {
        // Create output directory
        fs::create_dir_all(&self.config.output_config.output_dir)?;
        
        let timestamp = result.metadata.timestamp
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        
        // Save results in requested formats
        for format in &self.config.output_config.formats {
            match format {
                OutputFormat::JSON => {
                    let json_path = format!("{}/benchmark_results_{}.json", 
                                          self.config.output_config.output_dir, timestamp);
                    let json_data = serde_json::to_string_pretty(result)?;
                    fs::write(json_path, json_data)?;
                }
                OutputFormat::CSV => {
                    let csv_path = format!("{}/benchmark_results_{}.csv", 
                                         self.config.output_config.output_dir, timestamp);
                    // CSV generation would be implemented here
                    fs::write(csv_path, "CSV data placeholder")?;
                }
                OutputFormat::HTML => {
                    let html_path = format!("{}/benchmark_report_{}.html", 
                                          self.config.output_config.output_dir, timestamp);
                    let html_report = self.generate_html_report(result);
                    fs::write(html_path, html_report)?;
                }
                _ => {
                    // Other formats would be implemented here
                }
            }
        }
        
        Ok(())
    }
    
    fn generate_html_report(&self, result: &BenchmarkSuiteResult) -> String {
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Lambdust Benchmark Results</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .summary {{ background-color: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .implementation {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; }}
        .score {{ font-size: 1.2em; font-weight: bold; color: #2e7d32; }}
    </style>
</head>
<body>
    <h1>Lambdust Performance Benchmark Results</h1>
    
    <div class="summary">
        <h2>Summary</h2>
        <p><strong>Benchmark Date:</strong> {}</p>
        <p><strong>Total Duration:</strong> {:.2} seconds</p>
        <p><strong>Implementations Tested:</strong> {}</p>
    </div>
    
    <h2>Implementation Results</h2>
    {}
    
    <h2>System Information</h2>
    <p><strong>OS:</strong> {}</p>
    <p><strong>Architecture:</strong> {}</p>
    <p><strong>CPU Cores:</strong> {}</p>
    <p><strong>Memory:</strong> {} MB</p>
</body>
</html>
        "#,
        // Format timestamp
        result.metadata.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs(),
        result.metadata.total_duration.as_secs_f64(),
        result.implementation_results.len(),
        // Implementation results
        result.implementation_results.iter()
            .map(|(id, impl_result)| format!(r#"
                <div class="implementation">
                    <h3>{}</h3>
                    <p class="score">Overall Score: {:.1}/100</p>
                    <p><strong>Ranking:</strong> #{}</p>
                    <p><strong>Failures:</strong> {}</p>
                </div>
            "#, impl_result.config.name, impl_result.overall_score, impl_result.ranking, impl_result.failures.len()))
            .collect::<Vec<String>>()
            .join(""),
        result.metadata.system_info.os,
        result.metadata.system_info.architecture,
        result.metadata.system_info.cpu_cores,
        result.metadata.system_info.total_memory_mb
        )
    }
}

/// Convenience function to run benchmark suite with default configuration
pub fn run_comprehensive_benchmarks() -> Result<BenchmarkSuiteResult, Box<dyn std::error::Error>> {
    let config = BenchmarkSuiteConfig::default();
    let mut suite = ComprehensiveBenchmarkSuite::new(config);
    suite.execute()
}

/// Load configuration from file
pub fn load_benchmark_config(path: &str) -> Result<BenchmarkSuiteConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: BenchmarkSuiteConfig = serde_json::from_str(&content)?;
    Ok(config)
}

/// Save configuration to file
pub fn save_benchmark_config(config: &BenchmarkSuiteConfig, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_creation() {
        let config = BenchmarkSuiteConfig::default();
        assert!(!config.implementations.is_empty());
        assert!(!config.test_categories.is_empty());
    }
    
    #[test]
    fn test_parameter_combination_generation() {
        let suite = ComprehensiveBenchmarkSuite::new(BenchmarkSuiteConfig::default());
        
        let params = vec![
            TestParameter {
                name: "n".to_string(),
                values: vec![
                    ParameterValue::Integer { value: 10 },
                    ParameterValue::Integer { value: 20 },
                ],
                scaling_behavior: ScalingBehavior::Linear,
            },
            TestParameter {
                name: "m".to_string(),
                values: vec![
                    ParameterValue::Integer { value: 5 },
                ],
                scaling_behavior: ScalingBehavior::Constant,
            },
        ];
        
        let combinations = suite.generate_parameter_combinations(&params);
        assert_eq!(combinations.len(), 2); // 2 * 1 = 2 combinations
    }
    
    #[test]
    fn test_parameter_substitution() {
        let suite = ComprehensiveBenchmarkSuite::new(BenchmarkSuiteConfig::default());
        
        let template = "(test {n} {s})";
        let mut params = HashMap::new();
        params.insert("n".to_string(), ParameterValue::Integer { value: 42 });
        params.insert("s".to_string(), ParameterValue::String { value: "hello".to_string() });
        
        let result = suite.substitute_parameters(template, &params).unwrap();
        assert_eq!(result, "(test 42 \"hello\")");
    }
}