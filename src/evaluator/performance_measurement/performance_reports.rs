//! Performance visualization and report generation
//!
//! This module provides comprehensive performance reporting and visualization
//! capabilities for `RuntimeExecutor` optimization effectiveness analysis.

use crate::evaluator::performance_measurement::{
    ComparisonResult, PerformanceCategory
};
use crate::evaluator::RuntimeOptimizationLevel;
use std::collections::HashMap;
use std::io::Write;
use std::time::Instant;

/// Performance report generator
#[derive(Debug)]
pub struct PerformanceReportGenerator {
    /// Report configuration
    config: ReportConfig,
    /// Accumulated performance data
    performance_data: Vec<PerformanceDataPoint>,
    /// Report templates
    templates: HashMap<ReportType, ReportTemplate>,
}

/// Report generation configuration
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Include detailed benchmark breakdowns
    pub include_detailed_breakdowns: bool,
    /// Include statistical analysis
    pub include_statistical_analysis: bool,
    /// Include trend analysis
    pub include_trend_analysis: bool,
    /// Include optimization recommendations
    pub include_recommendations: bool,
    /// Report format preference
    pub preferred_format: ReportFormat,
    /// Output precision for numerical values
    pub decimal_precision: usize,
}

/// Report format types
#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    /// Markdown format for documentation
    Markdown,
    /// JSON format for programmatic access
    Json,
    /// CSV format for data analysis
    Csv,
    /// HTML format with interactive elements
    Html,
}

/// Report type categories
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReportType {
    /// Executive summary for management
    ExecutiveSummary,
    /// Detailed technical analysis
    TechnicalAnalysis,
    /// Performance trend report
    TrendAnalysis,
    /// Optimization effectiveness report
    OptimizationReport,
    /// Regression analysis report
    RegressionReport,
}

/// Single performance data point
#[derive(Debug, Clone)]
pub struct PerformanceDataPoint {
    /// Timestamp of measurement
    pub timestamp: Instant,
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Optimization level used
    pub optimization_level: RuntimeOptimizationLevel,
    /// Performance metrics
    pub metrics: PerformanceMetrics,
    /// Comparison results
    pub comparison_data: Option<ComparisonResult>,
}

/// Performance metrics for reporting
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Execution time in microseconds
    pub execution_time_us: u64,
    /// Speedup factor vs semantic evaluator
    pub speedup_factor: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Performance category
    pub performance_category: PerformanceCategory,
    /// Correctness verification result
    pub correctness_verified: bool,
}

/// Report template for different report types
#[derive(Debug, Clone)]
pub struct ReportTemplate {
    /// Template name
    pub name: String,
    /// Header template
    pub header_template: String,
    /// Section templates
    pub section_templates: HashMap<String, String>,
    /// Footer template
    pub footer_template: String,
}

/// Generated performance report
#[derive(Debug)]
pub struct PerformanceReport {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// Report content
    pub content: String,
    /// Report format
    pub format: ReportFormat,
    /// Generation timestamp
    pub generated_at: Instant,
    /// Report size in bytes
    pub size_bytes: usize,
}

/// Report metadata
#[derive(Debug, Clone)]
pub struct ReportMetadata {
    /// Report title
    pub title: String,
    /// Report type
    pub report_type: ReportType,
    /// Data time range
    pub time_range: TimeRange,
    /// Number of data points included
    pub data_points: usize,
    /// Report author/system
    pub author: String,
    /// Report version
    pub version: String,
}

/// Time range for report data
#[derive(Debug, Clone)]
pub struct TimeRange {
    /// Start time
    pub start: Instant,
    /// End time
    pub end: Instant,
}

impl PerformanceReportGenerator {
    /// Create a new report generator
    #[must_use] pub fn new() -> Self {
        let mut generator = Self {
            config: ReportConfig::default(),
            performance_data: Vec::new(),
            templates: HashMap::new(),
        };
        generator.initialize_templates();
        generator
    }

    /// Create with custom configuration
    #[must_use] pub fn with_config(config: ReportConfig) -> Self {
        let mut generator = Self {
            config,
            performance_data: Vec::new(),
            templates: HashMap::new(),
        };
        generator.initialize_templates();
        generator
    }

    /// Initialize report templates
    fn initialize_templates(&mut self) {
        // Executive summary template
        let exec_summary = ReportTemplate {
            name: "Executive Summary".to_string(),
            header_template: "# RuntimeExecutor Performance Summary\n\n".to_string(),
            section_templates: {
                let mut sections = HashMap::new();
                sections.insert("overview".to_string(), "## Performance Overview\n\n".to_string());
                sections.insert("key_metrics".to_string(), "## Key Metrics\n\n".to_string());
                sections.insert("recommendations".to_string(), "## Recommendations\n\n".to_string());
                sections
            },
            footer_template: "\n---\n*Generated by Lambdust Performance Measurement System*\n".to_string(),
        };
        self.templates.insert(ReportType::ExecutiveSummary, exec_summary);

        // Technical analysis template
        let tech_analysis = ReportTemplate {
            name: "Technical Analysis".to_string(),
            header_template: "# RuntimeExecutor Technical Performance Analysis\n\n".to_string(),
            section_templates: {
                let mut sections = HashMap::new();
                sections.insert("methodology".to_string(), "## Methodology\n\n".to_string());
                sections.insert("detailed_results".to_string(), "## Detailed Results\n\n".to_string());
                sections.insert("statistical_analysis".to_string(), "## Statistical Analysis\n\n".to_string());
                sections.insert("optimization_breakdown".to_string(), "## Optimization Breakdown\n\n".to_string());
                sections
            },
            footer_template: "\n---\n*Technical Analysis - Lambdust Performance System*\n".to_string(),
        };
        self.templates.insert(ReportType::TechnicalAnalysis, tech_analysis);
    }

    /// Add performance data point
    pub fn add_data_point(&mut self, data_point: PerformanceDataPoint) {
        self.performance_data.push(data_point);
    }

    /// Add comparison results
    pub fn add_comparison_results(&mut self, results: &[ComparisonResult]) {
        for result in results {
            let data_point = PerformanceDataPoint {
                timestamp: result.timestamp,
                benchmark_id: result.expression_summary.clone(),
                optimization_level: result.optimization_effectiveness.optimization_level,
                metrics: PerformanceMetrics {
                    execution_time_us: result.runtime_metrics.execution_time.as_micros() as u64,
                    speedup_factor: result.performance_comparison.speedup_factor,
                    memory_usage_bytes: result.runtime_metrics.memory_usage_bytes,
                    performance_category: result.performance_comparison.performance_category.clone(),
                    correctness_verified: result.correctness_check.results_equivalent,
                },
                comparison_data: Some(result.clone()),
            };
            self.add_data_point(data_point);
        }
    }

    /// Generate executive summary report
    #[must_use] pub fn generate_executive_summary(&self) -> PerformanceReport {
        let mut content = String::new();
        
        if let Some(template) = self.templates.get(&ReportType::ExecutiveSummary) {
            content.push_str(&template.header_template);
            
            // Overview section
            content.push_str(&template.section_templates["overview"]);
            content.push_str(&self.generate_overview_section());
            
            // Key metrics section
            content.push_str(&template.section_templates["key_metrics"]);
            content.push_str(&self.generate_key_metrics_section());
            
            // Recommendations section
            if self.config.include_recommendations {
                content.push_str(&template.section_templates["recommendations"]);
                content.push_str(&self.generate_recommendations_section());
            }
            
            content.push_str(&template.footer_template);
        }

        let metadata = ReportMetadata {
            title: "RuntimeExecutor Performance Executive Summary".to_string(),
            report_type: ReportType::ExecutiveSummary,
            time_range: self.get_data_time_range(),
            data_points: self.performance_data.len(),
            author: "Lambdust Performance System".to_string(),
            version: "1.0".to_string(),
        };

        PerformanceReport {
            metadata,
            size_bytes: content.len(),
            content,
            format: self.config.preferred_format.clone(),
            generated_at: Instant::now(),
        }
    }

    /// Generate technical analysis report
    #[must_use] pub fn generate_technical_analysis(&self) -> PerformanceReport {
        let mut content = String::new();
        
        if let Some(template) = self.templates.get(&ReportType::TechnicalAnalysis) {
            content.push_str(&template.header_template);
            
            // Methodology section
            content.push_str(&template.section_templates["methodology"]);
            content.push_str(&self.generate_methodology_section());
            
            // Detailed results section
            content.push_str(&template.section_templates["detailed_results"]);
            content.push_str(&self.generate_detailed_results_section());
            
            // Statistical analysis section
            if self.config.include_statistical_analysis {
                content.push_str(&template.section_templates["statistical_analysis"]);
                content.push_str(&self.generate_statistical_analysis_section());
            }
            
            // Optimization breakdown section
            content.push_str(&template.section_templates["optimization_breakdown"]);
            content.push_str(&self.generate_optimization_breakdown_section());
            
            content.push_str(&template.footer_template);
        }

        let metadata = ReportMetadata {
            title: "RuntimeExecutor Technical Performance Analysis".to_string(),
            report_type: ReportType::TechnicalAnalysis,
            time_range: self.get_data_time_range(),
            data_points: self.performance_data.len(),
            author: "Lambdust Performance System".to_string(),
            version: "1.0".to_string(),
        };

        PerformanceReport {
            metadata,
            size_bytes: content.len(),
            content,
            format: self.config.preferred_format.clone(),
            generated_at: Instant::now(),
        }
    }

    /// Generate overview section content
    fn generate_overview_section(&self) -> String {
        let mut content = String::new();
        
        let total_measurements = self.performance_data.len();
        let speedups: Vec<f64> = self.performance_data
            .iter()
            .map(|d| d.metrics.speedup_factor)
            .filter(|&s| s.is_finite())
            .collect();
        
        if speedups.is_empty() {
            content.push_str("*No valid performance data available for analysis.*\n\n");
        } else {
            let avg_speedup = speedups.iter().sum::<f64>() / speedups.len() as f64;
            let max_speedup = speedups.iter().fold(0.0f64, |a, &b| a.max(b));
            
            content.push_str(&format!("**Total Measurements:** {total_measurements}\n\n"));
            content.push_str(&format!("**Average Speedup:** {avg_speedup:.2}x\n\n"));
            content.push_str(&format!("**Maximum Speedup:** {max_speedup:.2}x\n\n"));
            
            let correct_count = self.performance_data
                .iter()
                .filter(|d| d.metrics.correctness_verified)
                .count();
            let correctness_rate = (correct_count as f64 / total_measurements as f64) * 100.0;
            content.push_str(&format!("**Correctness Rate:** {correctness_rate:.1}%\n\n"));
        }
        
        content
    }

    /// Generate key metrics section content
    fn generate_key_metrics_section(&self) -> String {
        let mut content = String::new();
        
        // Performance distribution
        let mut distribution = HashMap::new();
        for data_point in &self.performance_data {
            *distribution.entry(&data_point.metrics.performance_category).or_insert(0) += 1;
        }
        
        content.push_str("### Performance Distribution\n\n");
        for (category, count) in &distribution {
            let percentage = (f64::from(*count) / self.performance_data.len() as f64) * 100.0;
            content.push_str(&format!("- **{category:?}:** {count} ({percentage:.1}%)\n"));
        }
        content.push('\n');
        
        // Optimization level analysis
        let mut opt_level_dist = HashMap::new();
        for data_point in &self.performance_data {
            *opt_level_dist.entry(&data_point.optimization_level).or_insert(0) += 1;
        }
        
        content.push_str("### Optimization Level Usage\n\n");
        for (level, count) in &opt_level_dist {
            let percentage = (f64::from(*count) / self.performance_data.len() as f64) * 100.0;
            content.push_str(&format!("- **{level:?}:** {count} ({percentage:.1}%)\n"));
        }
        content.push('\n');
        
        content
    }

    /// Generate recommendations section content
    fn generate_recommendations_section(&self) -> String {
        let mut content = String::new();
        let mut recommendations: Vec<String> = Vec::new();
        
        // Analyze performance and generate recommendations
        let speedups: Vec<f64> = self.performance_data
            .iter()
            .map(|d| d.metrics.speedup_factor)
            .filter(|&s| s.is_finite())
            .collect();
        
        if !speedups.is_empty() {
            let avg_speedup = speedups.iter().sum::<f64>() / speedups.len() as f64;
            
            if avg_speedup < 1.5 {
                recommendations.push("🔧 **Consider Higher Optimization Levels**: Average speedup is below 1.5x. Experiment with Aggressive optimization level for better performance.".to_string());
            }
            
            if avg_speedup > 3.0 {
                recommendations.push("✅ **Excellent Performance**: RuntimeExecutor is showing significant optimization benefits. Consider documenting and sharing successful optimization strategies.".to_string());
            }
        }
        
        let incorrect_count = self.performance_data
            .iter()
            .filter(|d| !d.metrics.correctness_verified)
            .count();
        
        if incorrect_count > 0 {
            let error_rate = (incorrect_count as f64 / self.performance_data.len() as f64) * 100.0;
            let message = format!("⚠️ **Correctness Issues**: {error_rate:.1}% of measurements show correctness issues. Investigate and fix optimization bugs.");
            recommendations.push(message);
        }
        
        // Memory usage analysis
        let high_memory_count = self.performance_data
            .iter()
            .filter(|d| d.metrics.memory_usage_bytes > 10_000_000) // 10MB threshold
            .count();
        
        if high_memory_count > self.performance_data.len() / 4 {
            recommendations.push("💾 **Memory Usage Concern**: Many benchmarks show high memory usage. Consider implementing memory optimizations.".to_string());
        }
        
        if recommendations.is_empty() {
            content.push_str("✅ **System Performing Well**: No specific recommendations at this time. Continue monitoring performance trends.\n\n");
        } else {
            for rec in &recommendations {
                content.push_str(&format!("{rec}\n\n"));
            }
        }
        
        content
    }

    /// Generate methodology section content
    fn generate_methodology_section(&self) -> String {
        let mut content = String::new();
        
        content.push_str("This analysis compares RuntimeExecutor performance against SemanticEvaluator using:\n\n");
        content.push_str("- **Semantic Evaluator**: Pure R7RS reference implementation for correctness verification\n");
        content.push_str("- **Runtime Executor**: Optimized implementation with multiple optimization levels\n");
        content.push_str("- **Metrics**: Execution time, speedup factor, memory usage, correctness verification\n");
        content.push_str("- **Statistical Analysis**: Mean, median, standard deviation, confidence intervals\n\n");
        
        content
    }

    /// Generate detailed results section content
    fn generate_detailed_results_section(&self) -> String {
        let mut content = String::new();
        
        if self.config.include_detailed_breakdowns {
            content.push_str("### Benchmark Performance Breakdown\n\n");
            content.push_str("| Benchmark | Optimization Level | Speedup | Execution Time | Memory Usage | Correct |\n");
            content.push_str("|-----------|-------------------|---------|----------------|--------------|----------|\n");
            
            for data_point in &self.performance_data {
                let speedup = if data_point.metrics.speedup_factor.is_finite() {
                    format!("{:.2}x", data_point.metrics.speedup_factor)
                } else {
                    "∞".to_string()
                };
                
                let exec_time = format!("{}μs", data_point.metrics.execution_time_us);
                let memory = format!("{}B", data_point.metrics.memory_usage_bytes);
                let correct = if data_point.metrics.correctness_verified { "✅" } else { "❌" };
                
                content.push_str(&format!(
                    "| {} | {:?} | {} | {} | {} | {} |\n",
                    data_point.benchmark_id,
                    data_point.optimization_level,
                    speedup,
                    exec_time,
                    memory,
                    correct
                ));
            }
            content.push('\n');
        }
        
        content
    }

    /// Generate statistical analysis section content
    fn generate_statistical_analysis_section(&self) -> String {
        let mut content = String::new();
        
        let speedups: Vec<f64> = self.performance_data
            .iter()
            .map(|d| d.metrics.speedup_factor)
            .filter(|&s| s.is_finite())
            .collect();
        
        if !speedups.is_empty() {
            let mean = speedups.iter().sum::<f64>() / speedups.len() as f64;
            let variance = speedups.iter()
                .map(|&s| (s - mean).powi(2))
                .sum::<f64>() / speedups.len() as f64;
            let std_dev = variance.sqrt();
            
            let mut sorted_speedups = speedups.clone();
            sorted_speedups.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = if sorted_speedups.len() % 2 == 0 {
                f64::midpoint(sorted_speedups[sorted_speedups.len() / 2 - 1], sorted_speedups[sorted_speedups.len() / 2])
            } else {
                sorted_speedups[sorted_speedups.len() / 2]
            };
            
            content.push_str("### Speedup Factor Statistics\n\n");
            content.push_str(&format!("- **Mean:** {mean:.3}\n"));
            content.push_str(&format!("- **Median:** {median:.3}\n"));
            content.push_str(&format!("- **Standard Deviation:** {std_dev:.3}\n"));
            content.push_str(&format!("- **Minimum:** {:.3}\n", sorted_speedups[0]));
            content.push_str(&format!("- **Maximum:** {:.3}\n", sorted_speedups[sorted_speedups.len() - 1]));
            content.push('\n');
        }
        
        content
    }

    /// Generate optimization breakdown section content
    fn generate_optimization_breakdown_section(&self) -> String {
        let mut content = String::new();
        
        // Group by optimization level
        let mut by_opt_level: HashMap<RuntimeOptimizationLevel, Vec<&PerformanceDataPoint>> = HashMap::new();
        for data_point in &self.performance_data {
            by_opt_level.entry(data_point.optimization_level).or_default().push(data_point);
        }
        
        content.push_str("### Performance by Optimization Level\n\n");
        
        for (level, data_points) in &by_opt_level {
            let speedups: Vec<f64> = data_points
                .iter()
                .map(|d| d.metrics.speedup_factor)
                .filter(|&s| s.is_finite())
                .collect();
            
            if !speedups.is_empty() {
                let avg_speedup = speedups.iter().sum::<f64>() / speedups.len() as f64;
                let correct_count = data_points
                    .iter()
                    .filter(|d| d.metrics.correctness_verified)
                    .count();
                let correctness_rate = (correct_count as f64 / data_points.len() as f64) * 100.0;
                
                content.push_str(&format!("#### {level:?}\n\n"));
                content.push_str(&format!("- **Measurements:** {}\n", data_points.len()));
                content.push_str(&format!("- **Average Speedup:** {avg_speedup:.2}x\n"));
                content.push_str(&format!("- **Correctness Rate:** {correctness_rate:.1}%\n\n"));
            }
        }
        
        content
    }

    /// Get time range of performance data
    fn get_data_time_range(&self) -> TimeRange {
        if self.performance_data.is_empty() {
            let now = Instant::now();
            return TimeRange { start: now, end: now };
        }
        
        let start = self.performance_data
            .iter()
            .map(|d| d.timestamp)
            .min()
            .unwrap_or_else(Instant::now);
        
        let end = self.performance_data
            .iter()
            .map(|d| d.timestamp)
            .max()
            .unwrap_or_else(Instant::now);
        
        TimeRange { start, end }
    }

    /// Export report to writer
    pub fn export_report<W: Write>(&self, report: &PerformanceReport, writer: &mut W) -> std::io::Result<()> {
        match report.format {
            ReportFormat::Markdown => {
                write!(writer, "{}", report.content)?;
            }
            ReportFormat::Json => {
                // Simple JSON export (could be enhanced with serde)
                writeln!(writer, "{{")?;
                writeln!(writer, "  \"title\": \"{}\",", report.metadata.title)?;
                writeln!(writer, "  \"type\": \"{:?}\",", report.metadata.report_type)?;
                writeln!(writer, "  \"data_points\": {},", report.metadata.data_points)?;
                writeln!(writer, "  \"content\": \"{}\"", report.content.replace('\n', "\\n").replace('"', "\\\""))?;
                writeln!(writer, "}}")?;
            }
            ReportFormat::Csv => {
                writeln!(writer, "Metric,Value")?;
                writeln!(writer, "Title,{}", report.metadata.title)?;
                writeln!(writer, "Data Points,{}", report.metadata.data_points)?;
                writeln!(writer, "Generated At,{:?}", report.generated_at)?;
            }
            ReportFormat::Html => {
                writeln!(writer, "<!DOCTYPE html>")?;
                writeln!(writer, "<html><head><title>{}</title></head>", report.metadata.title)?;
                writeln!(writer, "<body><pre>{}</pre></body></html>", report.content)?;
            }
        }
        Ok(())
    }
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_detailed_breakdowns: true,
            include_statistical_analysis: true,
            include_trend_analysis: false,
            include_recommendations: true,
            preferred_format: ReportFormat::Markdown,
            decimal_precision: 3,
        }
    }
}

impl Default for PerformanceReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_generator_creation() {
        let generator = PerformanceReportGenerator::new();
        assert!(generator.performance_data.is_empty());
        assert!(!generator.templates.is_empty());
    }

    #[test]
    fn test_data_point_addition() {
        let mut generator = PerformanceReportGenerator::new();
        
        let data_point = PerformanceDataPoint {
            timestamp: Instant::now(),
            benchmark_id: "test_benchmark".to_string(),
            optimization_level: RuntimeOptimizationLevel::Balanced,
            metrics: PerformanceMetrics {
                execution_time_us: 1000,
                speedup_factor: 2.0,
                memory_usage_bytes: 1024,
                performance_category: PerformanceCategory::ModerateImprovement,
                correctness_verified: true,
            },
            comparison_data: None,
        };
        
        generator.add_data_point(data_point);
        assert_eq!(generator.performance_data.len(), 1);
    }

    #[test]
    fn test_executive_summary_generation() {
        let mut generator = PerformanceReportGenerator::new();
        
        // Add some test data
        let data_point = PerformanceDataPoint {
            timestamp: Instant::now(),
            benchmark_id: "test".to_string(),
            optimization_level: RuntimeOptimizationLevel::Balanced,
            metrics: PerformanceMetrics {
                execution_time_us: 1000,
                speedup_factor: 2.5,
                memory_usage_bytes: 1024,
                performance_category: PerformanceCategory::ModerateImprovement,
                correctness_verified: true,
            },
            comparison_data: None,
        };
        generator.add_data_point(data_point);
        
        let report = generator.generate_executive_summary();
        assert_eq!(report.metadata.report_type, ReportType::ExecutiveSummary);
        assert!(report.content.contains("Performance Overview"));
        assert!(report.content.contains("Key Metrics"));
    }
}