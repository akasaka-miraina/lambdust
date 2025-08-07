//! Comprehensive Benchmarking and Performance Analysis for Lambdust.
//!
//! This module provides a scientifically rigorous benchmarking system including:
//! - Cross-implementation performance comparison (Lambdust vs major Scheme implementations)
//! - Statistical analysis with confidence intervals and hypothesis testing  
//! - Performance regression detection with trend analysis
//! - Memory usage analysis and garbage collection profiling
//! - Actionable optimization recommendations
//! - Automated result collection and reporting

pub mod scheme_comparison;
pub mod performance_analysis;
pub mod performance_tester;
pub mod scheme_benchmark_suite;
pub mod comprehensive_benchmark_suite;
pub mod statistical_analysis;
pub mod regression_detection;

pub use performance_analysis::{
    PerformanceAnalyzer, PerformanceAnalysis, AnalysisConfig,
    AnalysisCategory, CategoryAnalysis, PerformanceBottleneck,
    HotPath, MemoryAnalysis, OptimizationRecommendation,
    BaselineComparison, BaselineMetrics,
};

pub use performance_tester::{
    PerformanceTester, PerformanceTestConfig, PerformanceTestResults,
    MicroBenchmarkResults, MacroBenchmarkResults, SimdOptimizationResults,
    MemoryPoolResults, EnvironmentOptimizationResults,
};

pub use scheme_benchmark_suite::{
    SchemeBenchmarkSuite, SchemeBenchmark, SchemeBenchmarkResult,
};

// New comprehensive benchmarking system
pub use comprehensive_benchmark_suite::{
    BenchmarkSuiteConfig, ComprehensiveBenchmarkSuite, BenchmarkSuiteResult,
    ImplementationConfig, TestCategory, TestCase, StatisticalConfig,
    OutputConfig, ResourceConfig, run_comprehensive_benchmarks,
    load_benchmark_config, save_benchmark_config,
};

pub use statistical_analysis::{
    StatisticalAnalyzer, StatisticalAnalysisConfig, StatisticalAnalysisResult,
    DescriptiveStatistics, PairwiseComparison, EffectSize, ConfidenceInterval,
    AnovaResult, OutlierAnalysis, NormalityTest, CorrelationMatrix,
    generate_statistical_report,
};

pub use regression_detection::{
    RegressionDetector, RegressionDetectionConfig, RegressionDetectionResult,
    PerformanceRegression, PerformanceImprovement, TrendAnalysis, PerformanceAnomaly,
    BaselineData, PerformanceMeasurement, OverallAssessment, ActionRecommendation,
    generate_regression_report,
};