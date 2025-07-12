//! Benchmarking and Performance Comparison Module
//! Comprehensive benchmarking system for evaluating Lambdust performance
//! against other language implementations, particularly GHC Haskell

pub mod ghc_comparison;

// Re-export key types
pub use ghc_comparison::{
    GHCComparisonSuite, GHCComparisonResult, GHCReferenceMetrics,
    PerformanceRatio, GHCBenchmarkCategory, GHCOptimizationLevel,
    TestConfiguration, StatisticalMethod, ComplexityClass
};

use crate::error::Result;
use std::collections::HashMap;

/// Main benchmarking coordinator
pub struct BenchmarkCoordinator {
    /// GHC comparison suite
    ghc_suite: GHCComparisonSuite,
    /// Custom benchmark suites
    custom_suites: HashMap<String, Box<dyn BenchmarkSuite>>,
}

/// Trait for custom benchmark suites
pub trait BenchmarkSuite {
    /// Name of the benchmark suite
    fn name(&self) -> &str;
    
    /// Run the benchmark suite
    fn run(&mut self) -> Result<BenchmarkSuiteResult>;
    
    /// Get configuration options
    fn get_config(&self) -> BenchmarkConfig;
    
    /// Set configuration
    fn set_config(&mut self, config: BenchmarkConfig);
}

/// Generic benchmark suite result
#[derive(Debug, Clone)]
pub struct BenchmarkSuiteResult {
    /// Suite name
    pub suite_name: String,
    /// Individual test results
    pub test_results: Vec<TestResult>,
    /// Overall statistics
    pub summary: BenchmarkSummary,
}

/// Individual test result
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Test name
    pub test_name: String,
    /// Test category
    pub category: String,
    /// Performance metrics
    pub metrics: TestMetrics,
    /// Pass/fail status
    pub status: TestStatus,
}

/// Test performance metrics
#[derive(Debug, Clone)]
pub struct TestMetrics {
    /// Execution time
    pub execution_time_ms: f64,
    /// Memory usage
    pub memory_usage_mb: f64,
    /// Throughput (operations per second)
    pub throughput_ops_per_sec: f64,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

/// Test status
#[derive(Debug, Clone)]
pub enum TestStatus {
    /// Test passed
    Passed,
    /// Test failed with error
    Failed(String),
    /// Test skipped
    Skipped(String),
    /// Test completed with warnings
    Warning(String),
}

/// Benchmark summary statistics
#[derive(Debug, Clone)]
pub struct BenchmarkSummary {
    /// Total tests run
    pub total_tests: usize,
    /// Number of passed tests
    pub passed_tests: usize,
    /// Number of failed tests
    pub failed_tests: usize,
    /// Overall execution time
    pub total_execution_time_ms: f64,
    /// Average performance score
    pub average_performance_score: f64,
}

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of iterations per test
    pub iterations: u32,
    /// Warmup iterations
    pub warmup_iterations: u32,
    /// Timeout per test (seconds)
    pub timeout_seconds: u32,
    /// Output verbosity level
    pub verbosity: VerbosityLevel,
    /// Enable performance profiling
    pub enable_profiling: bool,
}

/// Verbosity levels for benchmark output
#[derive(Debug, Clone)]
pub enum VerbosityLevel {
    /// Minimal output
    Quiet,
    /// Normal output
    Normal,
    /// Detailed output
    Verbose,
    /// Debug output
    Debug,
}

impl BenchmarkCoordinator {
    /// Create new benchmark coordinator
    pub fn new() -> Self {
        Self {
            ghc_suite: GHCComparisonSuite::new(),
            custom_suites: HashMap::new(),
        }
    }

    /// Register a custom benchmark suite
    pub fn register_suite(&mut self, name: String, suite: Box<dyn BenchmarkSuite>) {
        self.custom_suites.insert(name, suite);
    }

    /// Run GHC comparison benchmarks
    pub fn run_ghc_comparison(&mut self) -> Result<Vec<GHCComparisonResult>> {
        self.ghc_suite.run_comprehensive_comparison()
    }

    /// Run all registered benchmark suites
    pub fn run_all_suites(&mut self) -> Result<Vec<BenchmarkSuiteResult>> {
        let mut results = Vec::new();

        for (name, suite) in &mut self.custom_suites {
            println!("Running benchmark suite: {}", name);
            match suite.run() {
                Ok(result) => results.push(result),
                Err(e) => println!("Suite {} failed: {}", name, e),
            }
        }

        Ok(results)
    }

    /// Generate comprehensive benchmark report
    pub fn generate_comprehensive_report(&mut self) -> Result<String> {
        let mut report = String::new();
        
        report.push_str("# Lambdust Comprehensive Benchmark Report\n\n");
        
        // Run GHC comparison
        report.push_str("## GHC Haskell Comparison\n\n");
        match self.run_ghc_comparison() {
            Ok(ghc_results) => {
                let ghc_report = self.ghc_suite.generate_performance_report(&ghc_results);
                report.push_str(&ghc_report);
            }
            Err(e) => {
                report.push_str(&format!("GHC comparison failed: {}\n\n", e));
            }
        }

        // Run custom suites
        report.push_str("## Custom Benchmark Suites\n\n");
        match self.run_all_suites() {
            Ok(suite_results) => {
                for suite_result in suite_results {
                    report.push_str(&format!("### {}\n", suite_result.suite_name));
                    report.push_str(&format!("- Total tests: {}\n", suite_result.summary.total_tests));
                    report.push_str(&format!("- Passed: {}\n", suite_result.summary.passed_tests));
                    report.push_str(&format!("- Failed: {}\n", suite_result.summary.failed_tests));
                    report.push_str(&format!("- Execution time: {:.2}ms\n", suite_result.summary.total_execution_time_ms));
                    report.push_str(&format!("- Performance score: {:.2}\n\n", suite_result.summary.average_performance_score));
                }
            }
            Err(e) => {
                report.push_str(&format!("Custom suites failed: {}\n\n", e));
            }
        }

        Ok(report)
    }

    /// Get GHC comparison suite for configuration
    pub fn ghc_suite_mut(&mut self) -> &mut GHCComparisonSuite {
        &mut self.ghc_suite
    }

    /// List registered benchmark suites
    pub fn list_suites(&self) -> Vec<&str> {
        self.custom_suites.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for BenchmarkCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 10,
            warmup_iterations: 3,
            timeout_seconds: 30,
            verbosity: VerbosityLevel::Normal,
            enable_profiling: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBenchmarkSuite {
        name: String,
        config: BenchmarkConfig,
    }

    impl BenchmarkSuite for MockBenchmarkSuite {
        fn name(&self) -> &str {
            &self.name
        }

        fn run(&mut self) -> Result<BenchmarkSuiteResult> {
            Ok(BenchmarkSuiteResult {
                suite_name: self.name.clone(),
                test_results: vec![
                    TestResult {
                        test_name: "mock_test_1".to_string(),
                        category: "unit".to_string(),
                        metrics: TestMetrics {
                            execution_time_ms: 10.0,
                            memory_usage_mb: 1.0,
                            throughput_ops_per_sec: 100.0,
                            custom_metrics: HashMap::new(),
                        },
                        status: TestStatus::Passed,
                    }
                ],
                summary: BenchmarkSummary {
                    total_tests: 1,
                    passed_tests: 1,
                    failed_tests: 0,
                    total_execution_time_ms: 10.0,
                    average_performance_score: 95.0,
                },
            })
        }

        fn get_config(&self) -> BenchmarkConfig {
            self.config.clone()
        }

        fn set_config(&mut self, config: BenchmarkConfig) {
            self.config = config;
        }
    }

    #[test]
    fn test_benchmark_coordinator_creation() {
        let coordinator = BenchmarkCoordinator::new();
        assert_eq!(coordinator.list_suites().len(), 0);
    }

    #[test]
    fn test_custom_suite_registration() {
        let mut coordinator = BenchmarkCoordinator::new();
        
        let mock_suite = MockBenchmarkSuite {
            name: "test_suite".to_string(),
            config: BenchmarkConfig::default(),
        };
        
        coordinator.register_suite("test_suite".to_string(), Box::new(mock_suite));
        
        let suites = coordinator.list_suites();
        assert_eq!(suites.len(), 1);
        assert!(suites.contains(&"test_suite"));
    }

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.iterations, 10);
        assert_eq!(config.warmup_iterations, 3);
        assert_eq!(config.timeout_seconds, 30);
        assert!(matches!(config.verbosity, VerbosityLevel::Normal));
        assert!(!config.enable_profiling);
    }

    #[test]
    fn test_test_status_variants() {
        let passed = TestStatus::Passed;
        let failed = TestStatus::Failed("error".to_string());
        let skipped = TestStatus::Skipped("reason".to_string());
        let warning = TestStatus::Warning("warning".to_string());

        assert!(matches!(passed, TestStatus::Passed));
        assert!(matches!(failed, TestStatus::Failed(_)));
        assert!(matches!(skipped, TestStatus::Skipped(_)));
        assert!(matches!(warning, TestStatus::Warning(_)));
    }
}