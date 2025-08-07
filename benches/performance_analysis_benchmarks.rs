//! Performance Analysis and Reporting Benchmarks
//!
//! Comprehensive performance analysis infrastructure with visualization,
//! statistical analysis, and actionable recommendations for optimization.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lambdust::{LambdustRuntime, MultithreadedLambdust};
use lambdust::runtime::{BootstrapIntegrationConfig, BootstrapMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime as TokioRuntime;

/// Performance analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysisConfig {
    pub output_formats: Vec<OutputFormat>,
    pub analysis_types: Vec<AnalysisType>,
    pub visualization_types: Vec<VisualizationType>,
    pub statistical_tests: Vec<StatisticalTest>,
    pub comparison_baselines: Vec<ComparisonBaseline>,
    pub optimization_recommendations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    JSON,
    HTML,
    CSV,
    Markdown,
    PlotlyJSON,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    TrendAnalysis,
    PerformanceProfile,
    BottleneckIdentification,
    ScalabilityAnalysis,
    MigrationImpactAnalysis,
    ResourceUtilizationAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationType {
    TimeSeriesChart,
    PerformanceHeatmap,
    ScalabilityPlot,
    ComparisonBarChart,
    DistributionHistogram,
    CorrelationMatrix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatisticalTest {
    TTest,
    WilcoxonRankSum,
    AnovaTest,
    RegressionAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonBaseline {
    PreviousVersion,
    RustImplementation,
    OptimalTheoretical,
    IndustryBenchmark,
}

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub metadata: ReportMetadata,
    pub executive_summary: ExecutiveSummary,
    pub detailed_analysis: DetailedAnalysis,
    pub visualizations: Vec<VisualizationData>,
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
    pub appendices: Vec<Appendix>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub generated_at: String,
    pub version: String,
    pub git_commit: Option<String>,
    pub benchmark_duration: Duration,
    pub system_info: SystemInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_info: String,
    pub memory_info: String,
    pub os_info: String,
    pub rust_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub overall_performance_score: f64,
    pub key_findings: Vec<String>,
    pub critical_issues: Vec<String>,
    pub performance_improvements: Vec<String>,
    pub migration_impact_summary: MigrationImpactSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationImpactSummary {
    pub operations_migrated: usize,
    pub average_performance_change: f64,
    pub worst_case_degradation: f64,
    pub best_case_improvement: f64,
    pub acceptable_performance_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedAnalysis {
    pub benchmark_categories: HashMap<String, CategoryAnalysis>,
    pub statistical_analysis: StatisticalAnalysisResults,
    pub scalability_analysis: ScalabilityAnalysisResults,
    pub trend_analysis: TrendAnalysisResults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAnalysis {
    pub category_name: String,
    pub benchmark_count: usize,
    pub average_performance: Duration,
    pub performance_variance: f64,
    pub outliers: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysisResults {
    pub confidence_intervals: HashMap<String, ConfidenceInterval>,
    pub significance_tests: HashMap<String, SignificanceTestResult>,
    pub correlation_analysis: CorrelationAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    pub lower_bound: Duration,
    pub upper_bound: Duration,
    pub confidence_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificanceTestResult {
    pub test_type: StatisticalTest,
    pub p_value: f64,
    pub effect_size: f64,
    pub is_significant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationAnalysis {
    pub correlations: HashMap<String, f64>,
    pub strong_correlations: Vec<(String, String, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityAnalysisResults {
    pub thread_scaling: ThreadScalingAnalysis,
    pub data_scaling: DataScalingAnalysis,
    pub memory_scaling: MemoryScalingAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadScalingAnalysis {
    pub efficiency_by_thread_count: HashMap<usize, f64>,
    pub optimal_thread_count: usize,
    pub scalability_coefficient: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataScalingAnalysis {
    pub complexity_analysis: HashMap<String, ComplexityClassification>,
    pub scaling_coefficients: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityClassification {
    Constant,      // O(1)
    Logarithmic,   // O(log n)
    Linear,        // O(n)
    Linearithmic,  // O(n log n)
    Quadratic,     // O(n²)
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryScalingAnalysis {
    pub memory_efficiency: f64,
    pub gc_impact: f64,
    pub memory_leaks_detected: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysisResults {
    pub performance_trends: HashMap<String, TrendDirection>,
    pub regression_periods: Vec<RegressionPeriod>,
    pub improvement_periods: Vec<ImprovementPeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Degrading,
    Stable,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionPeriod {
    pub start_date: String,
    pub end_date: String,
    pub affected_benchmarks: Vec<String>,
    pub severity: RegressionSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Minor,
    Moderate,
    Severe,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementPeriod {
    pub start_date: String,
    pub end_date: String,
    pub improved_benchmarks: Vec<String>,
    pub improvement_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationData {
    pub visualization_type: VisualizationType,
    pub title: String,
    pub data: serde_json::Value,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub category: OptimizationCategory,
    pub priority: Priority,
    pub description: String,
    pub expected_impact: ExpectedImpact,
    pub implementation_effort: ImplementationEffort,
    pub specific_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationCategory {
    AlgorithmOptimization,
    DataStructureOptimization,
    MemoryOptimization,
    ConcurrencyOptimization,
    IOOptimization,
    CompilerOptimization,
    ArchitecturalOptimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedImpact {
    pub performance_improvement: f64,
    pub confidence_level: f64,
    pub affected_operations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Minimal,   // Hours
    Low,       // Days
    Medium,    // Weeks
    High,      // Months
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appendix {
    pub title: String,
    pub content: AppendixContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppendixContent {
    RawData(serde_json::Value),
    BenchmarkConfiguration(serde_json::Value),
    SystemConfiguration(serde_json::Value),
    StatisticalDetails(serde_json::Value),
}

impl Default for PerformanceAnalysisConfig {
    fn default() -> Self {
        Self {
            output_formats: vec![OutputFormat::HTML, OutputFormat::JSON, OutputFormat::Markdown],
            analysis_types: vec![
                AnalysisType::TrendAnalysis,
                AnalysisType::PerformanceProfile,
                AnalysisType::BottleneckIdentification,
                AnalysisType::MigrationImpactAnalysis,
            ],
            visualization_types: vec![
                VisualizationType::TimeSeriesChart,
                VisualizationType::ComparisonBarChart,
                VisualizationType::DistributionHistogram,
            ],
            statistical_tests: vec![StatisticalTest::TTest, StatisticalTest::WilcoxonRankSum],
            comparison_baselines: vec![ComparisonBaseline::PreviousVersion, ComparisonBaseline::RustImplementation],
            optimization_recommendations: true,
        }
    }
}

/// Performance analysis engine
pub struct PerformanceAnalysisEngine {
    config: PerformanceAnalysisConfig,
    benchmark_results: Vec<BenchmarkResult>,
    historical_data: Vec<HistoricalDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub category: String,
    pub bootstrap_mode: String,
    pub thread_count: usize,
    pub data_size: usize,
    pub measurements: Vec<Duration>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataPoint {
    pub timestamp: u64,
    pub benchmark_name: String,
    pub performance: Duration,
    pub git_commit: Option<String>,
}

impl PerformanceAnalysisEngine {
    pub fn new(config: PerformanceAnalysisConfig) -> Self {
        Self {
            config,
            benchmark_results: Vec::new(),
            historical_data: Vec::new(),
        }
    }
    
    pub fn add_benchmark_result(&mut self, result: BenchmarkResult) {
        self.benchmark_results.push(result);
    }
    
    pub fn load_historical_data(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if Path::new(file_path).exists() {
            let data = fs::read_to_string(file_path)?;
            self.historical_data = serde_json::from_str(&data)?;
        }
        Ok(())
    }
    
    pub fn generate_comprehensive_report(&self) -> PerformanceReport {
        let metadata = self.generate_report_metadata();
        let executive_summary = self.generate_executive_summary();
        let detailed_analysis = self.generate_detailed_analysis();
        let visualizations = self.generate_visualizations();
        let optimization_recommendations = self.generate_optimization_recommendations();
        let appendices = self.generate_appendices();
        
        PerformanceReport {
            metadata,
            executive_summary,
            detailed_analysis,
            visualizations,
            optimization_recommendations,
            appendices,
        }
    }
    
    fn generate_report_metadata(&self) -> ReportMetadata {
        ReportMetadata {
            generated_at: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: std::env::var("GIT_COMMIT").ok(),
            benchmark_duration: Duration::from_secs(3600), // Placeholder
            system_info: SystemInfo {
                cpu_info: "Unknown CPU".to_string(), // Would use system detection
                memory_info: "Unknown Memory".to_string(),
                os_info: std::env::consts::OS.to_string(),
                rust_version: env!("RUSTC_VERSION").to_string(),
            },
        }
    }
    
    fn generate_executive_summary(&self) -> ExecutiveSummary {
        let overall_score = self.calculate_overall_performance_score();
        let key_findings = self.identify_key_findings();
        let critical_issues = self.identify_critical_issues();
        let improvements = self.identify_performance_improvements();
        let migration_summary = self.analyze_migration_impact();
        
        ExecutiveSummary {
            overall_performance_score: overall_score,
            key_findings,
            critical_issues,
            performance_improvements: improvements,
            migration_impact_summary: migration_summary,
        }
    }
    
    fn calculate_overall_performance_score(&self) -> f64 {
        if self.benchmark_results.is_empty() {
            return 0.0;
        }
        
        let mut total_score = 0.0;
        let mut benchmark_count = 0;
        
        for result in &self.benchmark_results {
            if !result.measurements.is_empty() {
                let avg_time = result.measurements.iter().sum::<Duration>().as_nanos() as f64 / result.measurements.len() as f64;
                let normalized_score = 1.0 / (1.0 + avg_time / 1_000_000.0); // Normalize to 0-1 range
                total_score += normalized_score;
                benchmark_count += 1;
            }
        }
        
        if benchmark_count > 0 {
            (total_score / benchmark_count as f64) * 100.0
        } else {
            0.0
        }
    }
    
    fn identify_key_findings(&self) -> Vec<String> {
        let mut findings = Vec::new();
        
        // Analyze performance distribution
        let mut category_performance = HashMap::new();
        for result in &self.benchmark_results {
            if !result.measurements.is_empty() {
                let avg_time = result.measurements.iter().sum::<Duration>().as_nanos() as f64 / result.measurements.len() as f64;
                category_performance.entry(result.category.clone())
                    .or_insert_with(Vec::new)
                    .push(avg_time);
            }
        }
        
        for (category, times) in category_performance {
            let avg_category_time = times.iter().sum::<f64>() / times.len() as f64;
            findings.push(format!("Average performance for {} category: {:.2}ms", 
                category, avg_category_time / 1_000_000.0));
        }
        
        // Identify outliers
        let mut all_times = Vec::new();
        for result in &self.benchmark_results {
            for measurement in &result.measurements {
                all_times.push(measurement.as_nanos() as f64);
            }
        }
        
        if !all_times.is_empty() {
            all_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let q3_index = (all_times.len() as f64 * 0.75) as usize;
            let q1_index = (all_times.len() as f64 * 0.25) as usize;
            let q3 = all_times[q3_index.min(all_times.len() - 1)];
            let q1 = all_times[q1_index];
            let iqr = q3 - q1;
            let outlier_threshold = q3 + 1.5 * iqr;
            
            let outlier_count = all_times.iter().filter(|&&time| time > outlier_threshold).count();
            if outlier_count > 0 {
                findings.push(format!("Found {} performance outliers (>1.5×IQR above Q3)", outlier_count));
            }
        }
        
        findings
    }
    
    fn identify_critical_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check for extremely slow operations
        for result in &self.benchmark_results {
            if let Some(max_time) = result.measurements.iter().max() {
                if max_time.as_millis() > 1000 {
                    issues.push(format!("Critical performance issue in {}: maximum execution time {}ms", 
                        result.name, max_time.as_millis()));
                }
            }
        }
        
        // Check for high variance
        for result in &self.benchmark_results {
            if result.measurements.len() > 1 {
                let mean = result.measurements.iter().sum::<Duration>().as_nanos() as f64 / result.measurements.len() as f64;
                let variance = result.measurements.iter()
                    .map(|d| (d.as_nanos() as f64 - mean).powi(2))
                    .sum::<f64>() / result.measurements.len() as f64;
                let coefficient_of_variation = (variance.sqrt()) / mean;
                
                if coefficient_of_variation > 0.5 {
                    issues.push(format!("High performance variance in {}: CV={:.2}", 
                        result.name, coefficient_of_variation));
                }
            }
        }
        
        issues
    }
    
    fn identify_performance_improvements(&self) -> Vec<String> {
        let mut improvements = Vec::new();
        
        // Compare with historical data
        for result in &self.benchmark_results {
            if let Some(historical) = self.historical_data.iter()
                .filter(|h| h.benchmark_name == result.name)
                .max_by_key(|h| h.timestamp) {
                
                if !result.measurements.is_empty() {
                    let current_avg = result.measurements.iter().sum::<Duration>().as_nanos() as f64 / result.measurements.len() as f64;
                    let historical_avg = historical.performance.as_nanos() as f64;
                    
                    if current_avg < historical_avg * 0.9 {
                        let improvement = ((historical_avg - current_avg) / historical_avg) * 100.0;
                        improvements.push(format!("Performance improvement in {}: {:.1}% faster", 
                            result.name, improvement));
                    }
                }
            }
        }
        
        improvements
    }
    
    fn analyze_migration_impact(&self) -> MigrationImpactSummary {
        let migration_results: Vec<_> = self.benchmark_results.iter()
            .filter(|r| r.category.contains("migration") || r.name.contains("migration"))
            .collect();
        
        let mut performance_changes = Vec::new();
        let mut rust_times = Vec::new();
        let mut scheme_times = Vec::new();
        
        for result in &migration_results {
            if result.bootstrap_mode == "Fallback" && !result.measurements.is_empty() {
                let avg_time = result.measurements.iter().sum::<Duration>().as_nanos() as f64 / result.measurements.len() as f64;
                rust_times.push(avg_time);
            } else if result.bootstrap_mode == "Minimal" && !result.measurements.is_empty() {
                let avg_time = result.measurements.iter().sum::<Duration>().as_nanos() as f64 / result.measurements.len() as f64;
                scheme_times.push(avg_time);
            }
        }
        
        let mut worst_degradation = 0.0;
        let mut best_improvement = 0.0;
        let mut acceptable_count = 0;
        
        for (rust_time, scheme_time) in rust_times.iter().zip(scheme_times.iter()) {
            let change = (scheme_time - rust_time) / rust_time;
            performance_changes.push(change);
            
            if change > worst_degradation {
                worst_degradation = change;
            }
            if change < best_improvement {
                best_improvement = change;
            }
            if change.abs() <= 0.2 { // Within 20% is acceptable
                acceptable_count += 1;
            }
        }
        
        let average_change = if !performance_changes.is_empty() {
            performance_changes.iter().sum::<f64>() / performance_changes.len() as f64
        } else {
            0.0
        };
        
        let acceptable_ratio = if !performance_changes.is_empty() {
            acceptable_count as f64 / performance_changes.len() as f64
        } else {
            0.0
        };
        
        MigrationImpactSummary {
            operations_migrated: migration_results.len(),
            average_performance_change: average_change,
            worst_case_degradation: worst_degradation,
            best_case_improvement: best_improvement,
            acceptable_performance_ratio: acceptable_ratio,
        }
    }
    
    fn generate_detailed_analysis(&self) -> DetailedAnalysis {
        let benchmark_categories = self.analyze_by_category();
        let statistical_analysis = self.perform_statistical_analysis();
        let scalability_analysis = self.analyze_scalability();
        let trend_analysis = self.analyze_trends();
        
        DetailedAnalysis {
            benchmark_categories,
            statistical_analysis,
            scalability_analysis,
            trend_analysis,
        }
    }
    
    fn analyze_by_category(&self) -> HashMap<String, CategoryAnalysis> {
        let mut categories = HashMap::new();
        
        for result in &self.benchmark_results {
            let category = categories.entry(result.category.clone()).or_insert_with(|| CategoryAnalysis {
                category_name: result.category.clone(),
                benchmark_count: 0,
                average_performance: Duration::ZERO,
                performance_variance: 0.0,
                outliers: Vec::new(),
                recommendations: Vec::new(),
            });
            
            category.benchmark_count += 1;
            
            if !result.measurements.is_empty() {
                let avg_time = result.measurements.iter().sum::<Duration>() / result.measurements.len() as u32;
                category.average_performance = (category.average_performance + avg_time) / 2;
            }
        }
        
        categories
    }
    
    fn perform_statistical_analysis(&self) -> StatisticalAnalysisResults {
        let confidence_intervals = HashMap::new(); // Placeholder
        let significance_tests = HashMap::new(); // Placeholder
        let correlation_analysis = CorrelationAnalysis {
            correlations: HashMap::new(),
            strong_correlations: Vec::new(),
        };
        
        StatisticalAnalysisResults {
            confidence_intervals,
            significance_tests,
            correlation_analysis,
        }
    }
    
    fn analyze_scalability(&self) -> ScalabilityAnalysisResults {
        // Placeholder implementation
        ScalabilityAnalysisResults {
            thread_scaling: ThreadScalingAnalysis {
                efficiency_by_thread_count: HashMap::new(),
                optimal_thread_count: 4,
                scalability_coefficient: 0.8,
            },
            data_scaling: DataScalingAnalysis {
                complexity_analysis: HashMap::new(),
                scaling_coefficients: HashMap::new(),
            },
            memory_scaling: MemoryScalingAnalysis {
                memory_efficiency: 0.85,
                gc_impact: 0.15,
                memory_leaks_detected: Vec::new(),
            },
        }
    }
    
    fn analyze_trends(&self) -> TrendAnalysisResults {
        // Placeholder implementation
        TrendAnalysisResults {
            performance_trends: HashMap::new(),
            regression_periods: Vec::new(),
            improvement_periods: Vec::new(),
        }
    }
    
    fn generate_visualizations(&self) -> Vec<VisualizationData> {
        let mut visualizations = Vec::new();
        
        // Generate time series chart
        if self.config.visualization_types.contains(&VisualizationType::TimeSeriesChart) {
            visualizations.push(self.generate_time_series_chart());
        }
        
        // Generate comparison bar chart
        if self.config.visualization_types.contains(&VisualizationType::ComparisonBarChart) {
            visualizations.push(self.generate_comparison_bar_chart());
        }
        
        // Generate distribution histogram
        if self.config.visualization_types.contains(&VisualizationType::DistributionHistogram) {
            visualizations.push(self.generate_distribution_histogram());
        }
        
        visualizations
    }
    
    fn generate_time_series_chart(&self) -> VisualizationData {
        let mut data = serde_json::Map::new();
        data.insert("type".to_string(), serde_json::Value::String("time_series".to_string()));
        data.insert("title".to_string(), serde_json::Value::String("Performance Over Time".to_string()));
        
        VisualizationData {
            visualization_type: VisualizationType::TimeSeriesChart,
            title: "Performance Trends Over Time".to_string(),
            data: serde_json::Value::Object(data),
            description: "Shows performance trends across benchmark runs".to_string(),
        }
    }
    
    fn generate_comparison_bar_chart(&self) -> VisualizationData {
        let mut data = serde_json::Map::new();
        data.insert("type".to_string(), serde_json::Value::String("bar_chart".to_string()));
        data.insert("title".to_string(), serde_json::Value::String("Performance Comparison".to_string()));
        
        VisualizationData {
            visualization_type: VisualizationType::ComparisonBarChart,
            title: "Performance Comparison by Category".to_string(),
            data: serde_json::Value::Object(data),
            description: "Compares performance across different benchmark categories".to_string(),
        }
    }
    
    fn generate_distribution_histogram(&self) -> VisualizationData {
        let mut data = serde_json::Map::new();
        data.insert("type".to_string(), serde_json::Value::String("histogram".to_string()));
        data.insert("title".to_string(), serde_json::Value::String("Performance Distribution".to_string()));
        
        VisualizationData {
            visualization_type: VisualizationType::DistributionHistogram,
            title: "Performance Distribution Analysis".to_string(),
            data: serde_json::Value::Object(data),
            description: "Shows the distribution of performance measurements".to_string(),
        }
    }
    
    fn generate_optimization_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        if !self.config.optimization_recommendations {
            return recommendations;
        }
        
        // Analyze for algorithmic optimizations
        for result in &self.benchmark_results {
            if !result.measurements.is_empty() {
                let avg_time = result.measurements.iter().sum::<Duration>() / result.measurements.len() as u32;
                if avg_time.as_millis() > 100 {
                    recommendations.push(OptimizationRecommendation {
                        category: OptimizationCategory::AlgorithmOptimization,
                        priority: Priority::High,
                        description: format!("Optimize algorithm for benchmark: {}", result.name),
                        expected_impact: ExpectedImpact {
                            performance_improvement: 0.3,
                            confidence_level: 0.8,
                            affected_operations: vec![result.name.clone()],
                        },
                        implementation_effort: ImplementationEffort::Medium,
                        specific_actions: vec![
                            "Profile the hot paths in the algorithm".to_string(),
                            "Consider more efficient data structures".to_string(),
                            "Investigate caching opportunities".to_string(),
                        ],
                    });
                }
            }
        }
        
        // Memory optimization recommendations
        let high_memory_benchmarks: Vec<_> = self.benchmark_results.iter()
            .filter(|r| r.category.contains("memory") || r.name.contains("allocation"))
            .collect();
        
        if !high_memory_benchmarks.is_empty() {
            recommendations.push(OptimizationRecommendation {
                category: OptimizationCategory::MemoryOptimization,
                priority: Priority::Medium,
                description: "Optimize memory allocation patterns".to_string(),
                expected_impact: ExpectedImpact {
                    performance_improvement: 0.2,
                    confidence_level: 0.7,
                    affected_operations: high_memory_benchmarks.iter().map(|r| r.name.clone()).collect(),
                },
                implementation_effort: ImplementationEffort::Low,
                specific_actions: vec![
                    "Implement object pooling for frequently allocated objects".to_string(),
                    "Optimize garbage collection settings".to_string(),
                    "Consider using more efficient memory layouts".to_string(),
                ],
            });
        }
        
        recommendations
    }
    
    fn generate_appendices(&self) -> Vec<Appendix> {
        let mut appendices = Vec::new();
        
        // Raw benchmark data appendix
        let raw_data = serde_json::to_value(&self.benchmark_results).unwrap_or_default();
        appendices.push(Appendix {
            title: "Raw Benchmark Data".to_string(),
            content: AppendixContent::RawData(raw_data),
        });
        
        // System configuration appendix
        let mut system_config = serde_json::Map::new();
        system_config.insert("os".to_string(), serde_json::Value::String(std::env::consts::OS.to_string()));
        system_config.insert("arch".to_string(), serde_json::Value::String(std::env::consts::ARCH.to_string()));
        appendices.push(Appendix {
            title: "System Configuration".to_string(),
            content: AppendixContent::SystemConfiguration(serde_json::Value::Object(system_config)),
        });
        
        appendices
    }
    
    pub fn export_report(&self, report: &PerformanceReport, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(output_dir)?;
        
        for format in &self.config.output_formats {
            match format {
                OutputFormat::JSON => {
                    let json_data = serde_json::to_string_pretty(report)?;
                    fs::write(format!("{}/performance_report.json", output_dir), json_data)?;
                }
                OutputFormat::HTML => {
                    let html_report = self.generate_html_report(report);
                    fs::write(format!("{}/performance_report.html", output_dir), html_report)?;
                }
                OutputFormat::Markdown => {
                    let markdown_report = self.generate_markdown_report(report);
                    fs::write(format!("{}/performance_report.md", output_dir), markdown_report)?;
                }
                OutputFormat::CSV => {
                    let csv_data = self.generate_csv_data(report);
                    fs::write(format!("{}/benchmark_data.csv", output_dir), csv_data)?;
                }
                OutputFormat::PlotlyJSON => {
                    let plotly_data = self.generate_plotly_data(report);
                    fs::write(format!("{}/visualizations.json", output_dir), plotly_data)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn generate_html_report(&self, report: &PerformanceReport) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>Lambdust Performance Report</title>\n");
        html.push_str("<style>body { font-family: Arial, sans-serif; margin: 40px; }</style>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str(&format!("<h1>Lambdust Performance Analysis Report</h1>\n"));
        html.push_str(&format!("<p>Generated: {}</p>\n", report.metadata.generated_at));
        html.push_str(&format!("<p>Overall Performance Score: {:.1}/100</p>\n", report.executive_summary.overall_performance_score));
        
        html.push_str("<h2>Executive Summary</h2>\n");
        html.push_str("<h3>Key Findings</h3>\n<ul>\n");
        for finding in &report.executive_summary.key_findings {
            html.push_str(&format!("<li>{}</li>\n", finding));
        }
        html.push_str("</ul>\n");
        
        if !report.executive_summary.critical_issues.is_empty() {
            html.push_str("<h3>Critical Issues</h3>\n<ul>\n");
            for issue in &report.executive_summary.critical_issues {
                html.push_str(&format!("<li style='color: red;'>{}</li>\n", issue));
            }
            html.push_str("</ul>\n");
        }
        
        html.push_str("<h2>Optimization Recommendations</h2>\n<ol>\n");
        for rec in &report.optimization_recommendations {
            html.push_str(&format!("<li><strong>{:?} Priority:</strong> {} <br><em>Expected Impact: {:.1}% improvement</em></li>\n", 
                rec.priority, rec.description, rec.expected_impact.performance_improvement * 100.0));
        }
        html.push_str("</ol>\n");
        
        html.push_str("</body>\n</html>");
        html
    }
    
    fn generate_markdown_report(&self, report: &PerformanceReport) -> String {
        let mut md = String::new();
        md.push_str("# Lambdust Performance Analysis Report\n\n");
        md.push_str(&format!("**Generated:** {}\n", report.metadata.generated_at));
        md.push_str(&format!("**Overall Performance Score:** {:.1}/100\n\n", report.executive_summary.overall_performance_score));
        
        md.push_str("## Executive Summary\n\n");
        md.push_str("### Key Findings\n\n");
        for finding in &report.executive_summary.key_findings {
            md.push_str(&format!("- {}\n", finding));
        }
        
        if !report.executive_summary.critical_issues.is_empty() {
            md.push_str("\n### ⚠️ Critical Issues\n\n");
            for issue in &report.executive_summary.critical_issues {
                md.push_str(&format!("- **{}**\n", issue));
            }
        }
        
        md.push_str("\n## 🎯 Optimization Recommendations\n\n");
        for (i, rec) in report.optimization_recommendations.iter().enumerate() {
            md.push_str(&format!("{}. **{:?} Priority:** {}\n", i + 1, rec.priority, rec.description));
            md.push_str(&format!("   - Expected Impact: {:.1}% improvement\n", rec.expected_impact.performance_improvement * 100.0));
            md.push_str(&format!("   - Implementation Effort: {:?}\n\n", rec.implementation_effort));
        }
        
        md
    }
    
    fn generate_csv_data(&self, _report: &PerformanceReport) -> String {
        let mut csv = String::new();
        csv.push_str("benchmark_name,category,bootstrap_mode,thread_count,data_size,avg_time_ms,min_time_ms,max_time_ms\n");
        
        for result in &self.benchmark_results {
            if !result.measurements.is_empty() {
                let avg_time = result.measurements.iter().sum::<Duration>().as_millis() as f64 / result.measurements.len() as f64;
                let min_time = result.measurements.iter().min().unwrap().as_millis();
                let max_time = result.measurements.iter().max().unwrap().as_millis();
                
                csv.push_str(&format!("{},{},{},{},{},{:.2},{},{}\n",
                    result.name, result.category, result.bootstrap_mode,
                    result.thread_count, result.data_size, avg_time, min_time, max_time));
            }
        }
        
        csv
    }
    
    fn generate_plotly_data(&self, _report: &PerformanceReport) -> String {
        let mut plotly_data = serde_json::Map::new();
        plotly_data.insert("version".to_string(), serde_json::Value::String("1.0".to_string()));
        serde_json::to_string_pretty(&plotly_data).unwrap_or_default()
    }
}

// ============================================================================
// ANALYSIS BENCHMARKS
// ============================================================================

fn bench_performance_analysis_generation(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("performance_analysis_generation");
    group.measurement_time(Duration::from_secs(10));

    let analysis_configs = vec![
        ("minimal_analysis", PerformanceAnalysisConfig {
            output_formats: vec![OutputFormat::JSON],
            analysis_types: vec![AnalysisType::PerformanceProfile],
            visualization_types: vec![VisualizationType::ComparisonBarChart],
            statistical_tests: vec![],
            comparison_baselines: vec![],
            optimization_recommendations: false,
        }),
        ("comprehensive_analysis", PerformanceAnalysisConfig::default()),
    ];

    for (analysis_name, config) in analysis_configs {
        let benchmark_id = BenchmarkId::from_parameter(analysis_name);
        
        group.bench_with_input(benchmark_id, &config, |b, config| {
            b.to_async(&rt).iter(|| async move {
                let mut engine = PerformanceAnalysisEngine::new(config.clone());
                
                // Generate sample benchmark results
                for i in 0..10 {
                    let measurements = vec![
                        Duration::from_millis(100 + i * 10),
                        Duration::from_millis(95 + i * 10),
                        Duration::from_millis(105 + i * 10),
                    ];
                    
                    let result = BenchmarkResult {
                        name: format!("test_benchmark_{}", i),
                        category: "test_category".to_string(),
                        bootstrap_mode: "Minimal".to_string(),
                        thread_count: 1,
                        data_size: 1000,
                        measurements,
                        metadata: HashMap::new(),
                    };
                    
                    engine.add_benchmark_result(result);
                }
                
                let start = Instant::now();
                let _report = engine.generate_comprehensive_report();
                let generation_time = start.elapsed();
                
                generation_time
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    analysis_benches,
    bench_performance_analysis_generation
);

criterion_main!(analysis_benches);