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
pub mod statistical_analyzer;
pub mod statistical_tests;
pub mod effect_analysis;
pub mod outlier_normality;
pub mod correlation_statistics;
pub mod regression_detection;
pub mod regression_detector;
pub mod baseline_manager;
pub mod performance_measurement;
pub mod trend_analyzer;
pub mod anomaly_detector;
pub mod detection_results;
pub mod analysis_support;
pub mod external_integration;
pub mod benchmark_config;
pub mod system_metadata;
pub mod results_measurements;
pub mod statistical_analysis_results;
pub mod regression_optimization;
pub mod execution_management;

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
    ComprehensiveBenchmarkSuite, BenchmarkSuiteResult,
    run_comprehensive_benchmarks, load_benchmark_config, save_benchmark_config,
};

pub use benchmark_config::{
    BenchmarkSuiteConfig, ImplementationConfig, RuntimeConfig, TestCategory, 
    TestCase, TestParameter, ParameterValue, ScalingBehavior, ResultType,
    TestResourceLimits, PerformanceHints, StatisticalConfig, OutlierDetection,
    OutputConfig, OutputFormat, ChartType, ResourceConfig, SystemResourceLimits,
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

pub use external_integration::{
    ExternalReporting, GitHubConfig, DashboardConfig, NotificationConfig,
};

pub use system_metadata::{
    BenchmarkMetadata, SystemInfo, TestFailure, FailureReason, ResourceStats,
    CPUStats, MemoryStats, DiskIOStats, NetworkIOStats, BenchmarkResult,
};

pub use results_measurements::{
    ImplementationResult, CategoryResult, TestResult, TimingMeasurements,
    MemoryMeasurements, ConfidenceInterval as TimingConfidenceInterval, 
    ValidationResult, CategoryStatistics,
};

pub use statistical_analysis_results::{
    ImplementationComparison, StatisticalSignificance, CategoryComparison,
    StatisticalSummary, PerformanceRanking, DistributionStats, DistributionShape,
    CorrelationAnalysis,
};

pub use regression_optimization::{
    RegressionAnalysis as BenchmarkRegressionAnalysis, 
    PerformanceRegression as BenchmarkPerformanceRegression, 
    PerformanceImprovement as BenchmarkPerformanceImprovement,
    RegressionSeverity, TrendAnalysis as BenchmarkTrendAnalysis, 
    TrendDirection, PerformanceForecast, 
    OptimizationRecommendation as BenchmarkOptimizationRecommendation,
};

pub use execution_management::{
    SystemResourceUsage, ResourceSnapshot, ResourceEfficiency,
};