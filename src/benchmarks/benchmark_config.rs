//! Configuration structures for comprehensive benchmark suite.
//!
//! This module defines all configuration-related structures used to set up
//! and customize the behavior of the comprehensive benchmark suite, including
//! test parameters, statistical analysis settings, and output configurations.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::external_integration::ExternalReporting;

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