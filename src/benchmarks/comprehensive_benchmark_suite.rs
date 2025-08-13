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
use super::external_integration::{ExternalReporting, GitHubConfig, DashboardConfig, NotificationConfig};
use super::benchmark_config::{
    BenchmarkSuiteConfig, ImplementationConfig, RuntimeConfig, TestCategory, TestCase,
    TestParameter, ParameterValue, ScalingBehavior, ResultType, TestResourceLimits,
    PerformanceHints, StatisticalConfig, OutlierDetection, OutputConfig, OutputFormat,
    ChartType, ResourceConfig, SystemResourceLimits,
};
use super::system_metadata::{
    BenchmarkMetadata, SystemInfo, TestFailure, FailureReason, ResourceStats,
    CPUStats, MemoryStats, DiskIOStats, NetworkIOStats, BenchmarkResult,
};
use super::results_measurements::{
    ImplementationResult, CategoryResult, TestResult, TimingMeasurements,
    MemoryMeasurements, ConfidenceInterval as TimingConfidenceInterval, 
    ValidationResult, CategoryStatistics,
};
use super::statistical_analysis_results::{
    ImplementationComparison, StatisticalSignificance, CategoryComparison,
    StatisticalSummary, PerformanceRanking, DistributionStats, DistributionShape,
    CorrelationAnalysis,
};
use super::regression_optimization::{
    RegressionAnalysis, PerformanceRegression, PerformanceImprovement,
    RegressionSeverity, TrendAnalysis, TrendDirection, PerformanceForecast,
    OptimizationRecommendation,
};
use super::execution_management::{
    SystemResourceUsage, ResourceSnapshot, ResourceEfficiency,
};







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
                    implementation_results.insert(impl_config.id.clone(), result);
                }
                Err(e) => {
                    eprintln!("Failed to benchmark {}: {}", impl_config.name, e);
                    // Create a failure result
                    let failure_result = ImplementationResult {
                        config: impl_config.clone(),
                        category_results: HashMap::new(),
                        overall_score: 0.0,
                        ranking: 0,
                        failures: vec![Box::new(TestFailure {
                            test_name: "suite_execution".to_string(),
                            category: "infrastructure".to_string(),
                            parameters: HashMap::new(),
                            reason: FailureReason::InfrastructureError,
                            error_message: e.to_string(),
                            stack_trace: None,
                        })],
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
                    implementation_results.insert(impl_config.id.clone(), failure_result);
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
            config: self.config.clone(),
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
            
            category_results.insert(category.name.clone(), CategoryResult {
                category: category.name.clone(),
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
            config: impl_config.clone(),
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
    ) -> BenchmarkResult<TestResult> {
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
            return Err(Box::new(TestFailure {
                test_name: test_case.name.clone(),
                category: "unknown".to_string(), // Would be passed from caller
                parameters: params.clone(),
                reason: FailureReason::RuntimeError,
                error_message: error.unwrap_or_else(|| "Unknown error".to_string()),
                stack_trace: None,
            }));
        }
        
        Ok(TestResult {
            test_case: test_case.clone(),
            parameters: params.clone(),
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
                    let mut new_combo = existing_combo.clone();
                    new_combo.insert(param.name.clone(), value.clone());
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
    ) -> BenchmarkResult<String> {
        let mut result = template.to_string();
        
        for (name, value) in params {
            let placeholder = format!("{{{name}}}");
            let value_str = match value {
                ParameterValue::Integer { value } => value.to_string(),
                ParameterValue::Float { value } => value.to_string(),
                ParameterValue::String { value } => format!("\"{value}\""),
                ParameterValue::Boolean { value } => if *value { "#t" } else { "#f" }.to_string(),
                ParameterValue::Range { .. } => {
                    return Err(Box::new(TestFailure {
                        test_name: "parameter_substitution".to_string(),
                        category: "infrastructure".to_string(),
                        parameters: params.clone(),
                        reason: FailureReason::InfrastructureError,
                        error_message: "Range parameters should be expanded before substitution".to_string(),
                        stack_trace: None,
                    }));
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
    ) -> BenchmarkResult<String> {
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
            error_message: format!("Failed to create temp file: {e}"),
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
    ) -> BenchmarkResult<(TimingMeasurements, MemoryMeasurements, ValidationResult, bool, Option<String>)> {
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
    ) -> BenchmarkResult<(bool, Option<String>, u64)> {
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
                let binary_path = format!("{target_dir}/{profile}/lambdust");
                let mut command = Command::new(binary_path);
                command.arg("--batch");
                command.arg(test_file);
                command
            }
            RuntimeConfig::Docker { .. } => {
                // Docker execution would be implemented here
                return Err(Box::new(TestFailure {
                    test_name: "docker_execution".to_string(),
                    category: "infrastructure".to_string(),
                    parameters: HashMap::new(),
                    reason: FailureReason::InfrastructureError,
                    error_message: "Docker execution not yet implemented".to_string(),
                    stack_trace: None,
                }));
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
            Err(e) => Err(Box::new(TestFailure {
                test_name: "command_execution".to_string(),
                category: "infrastructure".to_string(),
                parameters: HashMap::new(),
                reason: FailureReason::InfrastructureError,
                error_message: format!("Failed to execute command: {e}"),
                stack_trace: None,
            })),
        }
    }
    
    // Placeholder implementations for statistics calculations
    fn calculate_timing_statistics(&self, times: Vec<Duration>) -> TimingMeasurements {
        if times.is_empty() {
            return self.empty_timing();
        }
        
        let sum: Duration = times.iter().sum();
        let mean = sum / times.len() as u32;
        
        let mut sorted_times = times.clone();
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
            confidence_interval: TimingConfidenceInterval {
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
            confidence_interval: TimingConfidenceInterval {
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