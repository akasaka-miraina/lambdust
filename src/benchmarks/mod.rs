//! Comprehensive Benchmarking and Performance Analysis for Lambdust.
//!
//! This module provides a scientifically rigorous benchmarking system including:
//! - Cross-implementation performance comparison (Lambdust vs major Scheme implementations)
//! - Statistical analysis with confidence intervals and hypothesis testing
//! - Performance regression detection with trend analysis
//! - Memory usage analysis and garbage collection profiling
//! - Actionable optimization recommendations
//! - Automated result collection and reporting

pub mod analysis_support;
pub mod benchmark_config;
pub mod comprehensive_benchmark_suite;
pub mod effect_analysis;
pub mod environment_optimization;
pub mod execution_management;
pub mod external_integration;
pub mod outlier_normality;
pub mod performance_analysis;
pub mod performance_measurement;
pub mod regression_optimization;
pub mod results_measurements;
pub mod scheme_benchmark_suite;
pub mod scheme_comparison;
pub mod statistical_analysis_results;
pub mod system_metadata;

pub use performance_analysis::{
    AnalysisCategory, AnalysisConfig, BaselineComparison, BaselineMetrics, CategoryAnalysis,
    HotPath, MemoryAnalysis, OptimizationRecommendation, PerformanceAnalysis, PerformanceAnalyzer,
    PerformanceBottleneck,
};

pub use scheme_benchmark_suite::{SchemeBenchmark, SchemeBenchmarkResult, SchemeBenchmarkSuite};

// New comprehensive benchmarking system
pub use comprehensive_benchmark_suite::{
    BenchmarkSuiteResult, ComprehensiveBenchmarkSuite, load_benchmark_config,
    run_comprehensive_benchmarks, save_benchmark_config,
};

pub use environment_optimization::{
    BenchmarkConfig as EnvBenchmarkConfig, BenchmarkResults as EnvBenchmarkResults,
    run_comprehensive_benchmark as run_env_benchmark,
    run_performance_tests as run_env_performance_tests,
    verify_correctness as verify_env_correctness,
};

pub use benchmark_config::{
    BenchmarkSuiteConfig, ChartType, ImplementationConfig, OutlierDetection, OutputConfig,
    OutputFormat, ParameterValue, PerformanceHints, ResourceConfig, ResultType, RuntimeConfig,
    ScalingBehavior, StatisticalConfig, SystemResourceLimits, TestCase, TestCategory,
    TestParameter, TestResourceLimits,
};

pub use external_integration::{
    DashboardConfig, ExternalReporting, GitHubConfig, NotificationConfig,
};

pub use system_metadata::{
    BenchmarkMetadata, BenchmarkResult, CPUStats, DiskIOStats, FailureReason, MemoryStats,
    NetworkIOStats, ResourceStats, SystemInfo, TestFailure,
};

pub use results_measurements::{
    CategoryResult, CategoryStatistics, ConfidenceInterval as TimingConfidenceInterval,
    ImplementationResult, MemoryMeasurements, TestResult, TimingMeasurements, ValidationResult,
};

pub use statistical_analysis_results::{
    CategoryComparison, CorrelationAnalysis, DistributionShape, DistributionStats,
    ImplementationComparison, PerformanceRanking, StatisticalSignificance, StatisticalSummary,
};

pub use regression_optimization::{
    OptimizationRecommendation as BenchmarkOptimizationRecommendation, PerformanceForecast,
    PerformanceImprovement as BenchmarkPerformanceImprovement,
    PerformanceRegression as BenchmarkPerformanceRegression,
    RegressionAnalysis as BenchmarkRegressionAnalysis, RegressionSeverity,
    TrendAnalysis as BenchmarkTrendAnalysis, TrendDirection,
};

pub use execution_management::{ResourceEfficiency, ResourceSnapshot, SystemResourceUsage};
