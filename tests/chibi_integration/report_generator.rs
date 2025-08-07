//! Report Generator Module
//!
//! This module generates comprehensive compliance reports in multiple formats:
//! - HTML reports with interactive visualizations
//! - JSON reports for programmatic consumption
//! - Markdown summaries for documentation
//! - CSV exports for data analysis

use super::{TestSuiteResults, TestResult, TestStatus};
use super::compliance_analyzer::{ComplianceAnalysis, R7RSFeature, GapSeverity, EstimatedEffort};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json;

/// Report generator for Chibi-Scheme integration test results
pub struct ReportGenerator<'a> {
    results: &'a TestSuiteResults,
    analysis: Option<ComplianceAnalysis>,
}

impl<'a> ReportGenerator<'a> {
    /// Create a new report generator
    pub fn new(results: &'a TestSuiteResults) -> Self {
        Self {
            results,
            analysis: None,
        }
    }
    
    /// Create a new report generator with compliance analysis
    pub fn with_analysis(results: &'a TestSuiteResults, analysis: ComplianceAnalysis) -> Self {
        Self {
            results,
            analysis: Some(analysis),
        }
    }
    
    /// Generate comprehensive JSON report
    pub fn generate_json_report(&self, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("📄 Generating JSON report...");
        
        let report = JsonReport {
            metadata: ReportMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                lambdust_version: "0.1.0".to_string(),
                chibi_integration_version: "1.0.0".to_string(),
                test_suite_version: "chibi-scheme-0.10.0".to_string(),
            },
            execution_summary: ExecutionSummary {
                total_tests: self.results.total_tests,
                passed: self.results.passed,
                failed: self.results.failed,
                errors: self.results.errors,
                timeouts: self.results.timeouts,
                skipped: self.results.skipped,
                execution_duration_seconds: self.results.execution_duration.as_secs_f64(),
                pass_rate: if self.results.total_tests > 0 {
                    (self.results.passed as f64 / self.results.total_tests as f64) * 100.0
                } else {
                    0.0
                },
            },
            compliance_summary: self.results.compliance_summary.clone(),
            test_results: self.results.test_results.clone(),
            feature_coverage: self.results.feature_coverage.clone(),
            detailed_analysis: self.analysis.clone(),
            performance_metrics: self.generate_performance_metrics(),
            recommendations: self.generate_json_recommendations(),
        };
        
        let json = serde_json::to_string_pretty(&report)?;
        fs::write(output_path, json)?;
        
        println!("  ✅ JSON report saved to: {}", output_path.display());
        Ok(())
    }
    
    /// Generate HTML report with interactive visualizations
    pub fn generate_html_report(&self, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("📄 Generating HTML report...");
        
        let html_content = self.build_html_content()?;
        fs::write(output_path, html_content)?;
        
        println!("  ✅ HTML report saved to: {}", output_path.display());
        Ok(())
    }
    
    /// Generate markdown summary report
    pub fn generate_markdown_summary(&self, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("📄 Generating Markdown summary...");
        
        let markdown_content = self.build_markdown_content()?;
        fs::write(output_path, markdown_content)?;
        
        println!("  ✅ Markdown summary saved to: {}", output_path.display());
        Ok(())
    }
    
    /// Generate CSV export for data analysis
    pub fn generate_csv_export(&self, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("📄 Generating CSV exports...");
        
        // Test results CSV
        let test_results_csv = self.build_test_results_csv();
        let csv_path = output_dir.join("test_results.csv");
        fs::write(&csv_path, test_results_csv)?;
        println!("  ✅ Test results CSV: {}", csv_path.display());
        
        // Feature coverage CSV
        if let Some(analysis) = &self.analysis {
            let feature_csv = self.build_feature_coverage_csv(&analysis.feature_analysis);
            let feature_path = output_dir.join("feature_coverage.csv");
            fs::write(&feature_path, feature_csv)?;
            println!("  ✅ Feature coverage CSV: {}", feature_path.display());
        }
        
        Ok(())
    }
    
    /// Build complete HTML content with embedded CSS and JavaScript
    fn build_html_content(&self) -> Result<String, Box<dyn std::error::Error>> {
        let title = "Lambdust R7RS Compliance Report";
        let summary_stats = self.build_html_summary();
        let test_results_section = self.build_html_test_results();
        let feature_analysis_section = self.build_html_feature_analysis();
        let recommendations_section = self.build_html_recommendations();
        let charts_section = self.build_html_charts();
        
        let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        {css}
    </style>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
    <div class="container">
        <header>
            <h1>{title}</h1>
            <p class="subtitle">Generated: {timestamp}</p>
        </header>
        
        <main>
            {summary_stats}
            {charts_section}
            {test_results_section}
            {feature_analysis_section}
            {recommendations_section}
        </main>
        
        <footer>
            <p>Generated by Lambdust Chibi-Scheme Integration Suite</p>
        </footer>
    </div>
    
    <script>
        {javascript}
    </script>
</body>
</html>"#,
            title = title,
            css = self.get_embedded_css(),
            timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC"),
            summary_stats = summary_stats,
            charts_section = charts_section,
            test_results_section = test_results_section,
            feature_analysis_section = feature_analysis_section,
            recommendations_section = recommendations_section,
            javascript = self.get_embedded_javascript(),
        );
        
        Ok(html)
    }
    
    /// Build HTML summary statistics section
    fn build_html_summary(&self) -> String {
        let pass_rate = if self.results.total_tests > 0 {
            (self.results.passed as f64 / self.results.total_tests as f64) * 100.0
        } else {
            0.0
        };
        
        let compliance_grade = match self.results.compliance_summary.overall_percentage {
            p if p >= 95.0 => ("A+", "excellent"),
            p if p >= 90.0 => ("A", "very-good"),
            p if p >= 85.0 => ("B+", "good"),
            p if p >= 80.0 => ("B", "satisfactory"),
            p if p >= 70.0 => ("C+", "needs-work"),
            p if p >= 60.0 => ("C", "major-gaps"),
            _ => ("D", "incomplete"),
        };
        
        format!(r#"
        <section class="summary">
            <h2>Executive Summary</h2>
            <div class="summary-grid">
                <div class="summary-card">
                    <h3>Overall Compliance</h3>
                    <div class="metric-value">{:.1}%</div>
                    <div class="grade-badge {}">{}</div>
                </div>
                <div class="summary-card">
                    <h3>Test Results</h3>
                    <div class="metric-value">{}/{}</div>
                    <div class="sub-metric">Pass Rate: {:.1}%</div>
                </div>
                <div class="summary-card">
                    <h3>Execution Time</h3>
                    <div class="metric-value">{:.2}s</div>
                    <div class="sub-metric">{:.2} tests/sec</div>
                </div>
                <div class="summary-card">
                    <h3>Feature Coverage</h3>
                    <div class="metric-value">{}</div>
                    <div class="sub-metric">{} features analyzed</div>
                </div>
            </div>
        </section>
        "#,
            self.results.compliance_summary.overall_percentage,
            compliance_grade.1,
            compliance_grade.0,
            self.results.passed,
            self.results.total_tests,
            pass_rate,
            self.results.execution_duration.as_secs_f64(),
            self.results.total_tests as f64 / self.results.execution_duration.as_secs_f64(),
            self.results.feature_coverage.len(),
            self.results.feature_coverage.len()
        )
    }
    
    /// Build HTML test results section
    fn build_html_test_results(&self) -> String {
        let mut table_rows = String::new();
        
        for result in &self.results.test_results {
            let status_class = match result.status {
                TestStatus::Passed => "status-passed",
                TestStatus::Failed => "status-failed",
                TestStatus::Error => "status-error",
                TestStatus::Timeout => "status-timeout",
                TestStatus::Skipped => "status-skipped",
                TestStatus::Adapted => "status-adapted",
            };
            
            let status_icon = match result.status {
                TestStatus::Passed => "✅",
                TestStatus::Failed => "❌",
                TestStatus::Error => "💥",
                TestStatus::Timeout => "⏱️",
                TestStatus::Skipped => "⏭️",
                TestStatus::Adapted => "🔧",
            };
            
            let error_msg = result.error_message.as_deref().unwrap_or("-");
            let duration_ms = result.execution_time.as_millis();
            let truncated_error = if error_msg.len() > 50 { 
                format!("{}...", &error_msg[..50]) 
            } else { 
                error_msg.to_string() 
            };
            
            table_rows.push_str(&format!(r#"
                <tr class="{}">
                    <td>{} {:?}</td>
                    <td>{}</td>
                    <td>{}ms</td>
                    <td class="error-cell" title="{}">{}</td>
                </tr>
            "#,
                status_class,
                status_icon,
                result.status,
                result.test_name,
                duration_ms,
                error_msg,
                truncated_error
            ));
        }
        
        format!(r#"
        <section class="test-results">
            <h2>Test Results Details</h2>
            <div class="table-container">
                <table class="results-table">
                    <thead>
                        <tr>
                            <th>Status</th>
                            <th>Test Name</th>
                            <th>Duration</th>
                            <th>Error/Notes</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>
        </section>
        "#, table_rows)
    }
    
    /// Build HTML feature analysis section
    fn build_html_feature_analysis(&self) -> String {
        if let Some(analysis) = &self.analysis {
            let mut feature_cards = String::new();
            
            for (feature, results) in &analysis.feature_analysis {
                let status_class = match results.implementation_status {
                    super::ImplementationStatus::Complete => "status-complete",
                    super::ImplementationStatus::Partial => "status-partial",
                    super::ImplementationStatus::Minimal => "status-minimal",
                    super::ImplementationStatus::Missing => "status-missing",
                    super::ImplementationStatus::Planned => "status-planned",
                };
                
                let status_icon = match results.implementation_status {
                    super::ImplementationStatus::Complete => "✅",
                    super::ImplementationStatus::Partial => "🟡",
                    super::ImplementationStatus::Minimal => "🟠",
                    super::ImplementationStatus::Missing => "❌",
                    super::ImplementationStatus::Planned => "📋",
                };
                
                feature_cards.push_str(&format!(r#"
                    <div class="feature-card {}">
                        <h4>{} {}</h4>
                        <div class="feature-stats">
                            <div class="stat">
                                <span class="stat-label">Coverage:</span>
                                <span class="stat-value">{:.1}%</span>
                            </div>
                            <div class="stat">
                                <span class="stat-label">Tests:</span>
                                <span class="stat-value">{}/{}</span>
                            </div>
                        </div>
                        <div class="progress-bar">
                            <div class="progress-fill" style="width: {:.1}%"></div>
                        </div>
                        {}
                    </div>
                "#,
                    status_class,
                    status_icon,
                    feature.as_str(),
                    results.coverage_percentage,
                    results.passed_tests,
                    results.total_tests,
                    results.coverage_percentage,
                    if !results.missing_procedures.is_empty() {
                        format!("<div class=\"missing-procedures\">Missing: {}</div>", 
                               results.missing_procedures.join(", "))
                    } else {
                        String::new()
                    }
                ));
            }
            
            format!(r#"
            <section class="feature-analysis">
                <h2>R7RS Feature Analysis</h2>
                <div class="feature-grid">
                    {}
                </div>
            </section>
            "#, feature_cards)
        } else {
            String::new()
        }
    }
    
    /// Build HTML recommendations section
    fn build_html_recommendations(&self) -> String {
        if let Some(analysis) = &self.analysis {
            let mut recommendations_html = String::new();
            
            for (index, rec) in analysis.recommendations.iter().enumerate() {
                let priority_class = match rec.priority {
                    super::compliance_analyzer::RecommendationPriority::Critical => "priority-critical",
                    super::compliance_analyzer::RecommendationPriority::High => "priority-high",
                    super::compliance_analyzer::RecommendationPriority::Medium => "priority-medium",
                    super::compliance_analyzer::RecommendationPriority::Low => "priority-low",
                };
                
                let effort_text = match rec.estimated_effort {
                    EstimatedEffort::Small => "1-2 weeks",
                    EstimatedEffort::Medium => "2-4 weeks",
                    EstimatedEffort::Large => "4-8 weeks",
                    EstimatedEffort::XLarge => "8+ weeks",
                };
                
                let actions_html = rec.specific_actions.iter()
                    .map(|action| format!("<li>{}</li>", action))
                    .collect::<Vec<_>>()
                    .join("");
                
                recommendations_html.push_str(&format!(r#"
                    <div class="recommendation-card {}">
                        <h4>{}. {}</h4>
                        <p>{}</p>
                        <div class="recommendation-meta">
                            <span class="effort">Effort: {}</span>
                            <span class="impact">Impact: {:.1}/10</span>
                        </div>
                        {}
                    </div>
                "#,
                    priority_class,
                    index + 1,
                    rec.title,
                    rec.description,
                    effort_text,
                    rec.expected_impact,
                    if !rec.specific_actions.is_empty() {
                        format!("<details><summary>Specific Actions</summary><ul>{}</ul></details>", actions_html)
                    } else {
                        String::new()
                    }
                ));
            }
            
            format!(r#"
            <section class="recommendations">
                <h2>Implementation Recommendations</h2>
                <div class="recommendations-container">
                    {}
                </div>
            </section>
            "#, recommendations_html)
        } else {
            String::new()
        }
    }
    
    /// Build HTML charts section
    fn build_html_charts(&self) -> String {
        format!(r#"
        <section class="charts">
            <h2>Visual Analysis</h2>
            <div class="charts-grid">
                <div class="chart-container">
                    <canvas id="complianceChart" width="400" height="200"></canvas>
                </div>
                <div class="chart-container">
                    <canvas id="featureChart" width="400" height="200"></canvas>
                </div>
            </div>
        </section>
        "#)
    }
    
    /// Build markdown content
    fn build_markdown_content(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut content = String::new();
        
        // Header
        content.push_str(&format!(r#"# Lambdust R7RS Compliance Report

**Generated:** {}  
**Test Suite:** Chibi-Scheme Integration  
**Lambdust Version:** 0.1.0  

## Executive Summary

| Metric | Value |
|--------|-------|
| Overall Compliance | {:.1}% |
| Tests Passed | {}/{} ({:.1}%) |
| Execution Time | {:.2}s |
| Features Analyzed | {} |

"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC"),
            self.results.compliance_summary.overall_percentage,
            self.results.passed,
            self.results.total_tests,
            if self.results.total_tests > 0 {
                (self.results.passed as f64 / self.results.total_tests as f64) * 100.0
            } else { 0.0 },
            self.results.execution_duration.as_secs_f64(),
            self.results.feature_coverage.len()
        ));
        
        // Compliance Grade
        let grade = match self.results.compliance_summary.overall_percentage {
            p if p >= 95.0 => "🏆 A+ (Excellent)",
            p if p >= 90.0 => "🥇 A (Very Good)",
            p if p >= 85.0 => "🥈 B+ (Good)",
            p if p >= 80.0 => "🥉 B (Satisfactory)",
            p if p >= 70.0 => "⚠️ C+ (Needs Work)",
            p if p >= 60.0 => "❌ C (Major Gaps)",
            _ => "💥 D (Incomplete)",
        };
        
        content.push_str(&format!("## Compliance Grade\n\n{}\n\n", grade));
        
        // Test Results Summary
        content.push_str("## Test Results Summary\n\n");
        content.push_str("| Status | Count | Percentage |\n");
        content.push_str("|--------|-------|-----------|\n");
        content.push_str(&format!("| ✅ Passed | {} | {:.1}% |\n", self.results.passed,
            (self.results.passed as f64 / self.results.total_tests as f64) * 100.0));
        content.push_str(&format!("| ❌ Failed | {} | {:.1}% |\n", self.results.failed,
            (self.results.failed as f64 / self.results.total_tests as f64) * 100.0));
        content.push_str(&format!("| 💥 Errors | {} | {:.1}% |\n", self.results.errors,
            (self.results.errors as f64 / self.results.total_tests as f64) * 100.0));
        content.push_str(&format!("| ⏱️ Timeouts | {} | {:.1}% |\n", self.results.timeouts,
            (self.results.timeouts as f64 / self.results.total_tests as f64) * 100.0));
        content.push_str("\n");
        
        // Feature Analysis
        if let Some(analysis) = &self.analysis {
            content.push_str("## R7RS Feature Analysis\n\n");
            content.push_str("| Feature | Status | Coverage | Tests | Missing Procedures |\n");
            content.push_str("|---------|--------|----------|-------|-----------------|\n");
            
            for (feature, results) in &analysis.feature_analysis {
                let status_icon = match results.implementation_status {
                    super::ImplementationStatus::Complete => "✅",
                    super::ImplementationStatus::Partial => "🟡",
                    super::ImplementationStatus::Minimal => "🟠",
                    super::ImplementationStatus::Missing => "❌",
                    super::ImplementationStatus::Planned => "📋",
                };
                
                let missing_procs = if results.missing_procedures.len() > 3 {
                    format!("{} (+{} more)", 
                           results.missing_procedures[..3].join(", "),
                           results.missing_procedures.len() - 3)
                } else {
                    results.missing_procedures.join(", ")
                };
                
                content.push_str(&format!(
                    "| {} {} | {:?} | {:.1}% | {}/{} | {} |\n",
                    status_icon,
                    feature.as_str(),
                    results.implementation_status,
                    results.coverage_percentage,
                    results.passed_tests,
                    results.total_tests,
                    if missing_procs.is_empty() { "-" } else { &missing_procs }
                ));
            }
            
            content.push_str("\n");
            
            // Critical Gaps
            if !analysis.critical_gaps.is_empty() {
                content.push_str("## Critical Gaps\n\n");
                for gap in &analysis.critical_gaps {
                    content.push_str(&format!("### {} - {} Priority\n\n", gap.feature.as_str(), gap.severity.as_str()));
                    content.push_str(&format!("**Impact:** {:.1}/10  \n", gap.impact));
                    content.push_str(&format!("**Action:** {}  \n", gap.recommended_action));
                    if !gap.missing_procedures.is_empty() {
                        content.push_str(&format!("**Missing:** {}  \n", gap.missing_procedures.join(", ")));
                    }
                    content.push_str("\n");
                }
            }
            
            // Recommendations
            if !analysis.recommendations.is_empty() {
                content.push_str("## Top Recommendations\n\n");
                for (index, rec) in analysis.recommendations.iter().take(10).enumerate() {
                    let priority_icon = match rec.priority {
                        super::compliance_analyzer::RecommendationPriority::Critical => "🔴",
                        super::compliance_analyzer::RecommendationPriority::High => "🟠",
                        super::compliance_analyzer::RecommendationPriority::Medium => "🟡",
                        super::compliance_analyzer::RecommendationPriority::Low => "🟢",
                    };
                    
                    content.push_str(&format!("{}. {} **{}**\n", index + 1, priority_icon, rec.title));
                    content.push_str(&format!("   {}\n\n", rec.description));
                }
            }
        }
        
        // Footer
        content.push_str("---\n\n");
        content.push_str("*Report generated by Lambdust Chibi-Scheme Integration Suite*\n");
        
        Ok(content)
    }
    
    /// Build test results CSV
    fn build_test_results_csv(&self) -> String {
        let mut csv = String::new();
        csv.push_str("Test Name,Status,Duration (ms),Error Message\n");
        
        for result in &self.results.test_results {
            let error_msg = result.error_message.as_deref().unwrap_or("").replace(',', ";");
            csv.push_str(&format!("{},{:?},{},{}\n",
                result.test_name,
                result.status,
                result.execution_time.as_millis(),
                error_msg
            ));
        }
        
        csv
    }
    
    /// Build feature coverage CSV
    fn build_feature_coverage_csv(&self, feature_analysis: &HashMap<R7RSFeature, super::compliance_analyzer::FeatureTestResults>) -> String {
        let mut csv = String::new();
        csv.push_str("Feature,Status,Coverage %,Total Tests,Passed Tests,Failed Tests,Missing Procedures\n");
        
        for (feature, results) in feature_analysis {
            let missing_procs = results.missing_procedures.join(";");
            csv.push_str(&format!("{},{:?},{:.1},{},{},{},{}\n",
                feature.as_str(),
                results.implementation_status,
                results.coverage_percentage,
                results.total_tests,
                results.passed_tests,
                results.failed_tests,
                missing_procs
            ));
        }
        
        csv
    }
    
    /// Generate performance metrics for JSON report
    fn generate_performance_metrics(&self) -> PerformanceMetrics {
        let total_time = self.results.execution_duration.as_secs_f64();
        let tests_per_second = if total_time > 0.0 {
            self.results.total_tests as f64 / total_time
        } else {
            0.0
        };
        
        // Calculate test duration statistics
        let mut durations: Vec<f64> = self.results.test_results.iter()
            .map(|r| r.execution_time.as_secs_f64())
            .collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let avg_duration = if !durations.is_empty() {
            durations.iter().sum::<f64>() / durations.len() as f64
        } else {
            0.0
        };
        
        let median_duration = if !durations.is_empty() {
            let mid = durations.len() / 2;
            if durations.len() % 2 == 0 {
                (durations[mid - 1] + durations[mid]) / 2.0
            } else {
                durations[mid]
            }
        } else {
            0.0
        };
        
        PerformanceMetrics {
            total_execution_time: total_time,
            tests_per_second,
            average_test_duration: avg_duration,
            median_test_duration: median_duration,
            min_test_duration: durations.first().copied().unwrap_or(0.0),
            max_test_duration: durations.last().copied().unwrap_or(0.0),
        }
    }
    
    /// Generate recommendations for JSON report
    fn generate_json_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.results.compliance_summary.overall_percentage < 70.0 {
            recommendations.push("Focus on implementing core R7RS features to improve overall compliance".to_string());
        }
        
        if self.results.failed > self.results.passed {
            recommendations.push("Address failing tests as a priority - more tests are failing than passing".to_string());
        }
        
        if self.results.errors > 0 {
            recommendations.push(format!("Fix {} test execution errors which indicate parser or evaluator issues", self.results.errors));
        }
        
        if self.results.timeouts > 0 {
            recommendations.push(format!("Investigate {} timeout issues which may indicate performance problems", self.results.timeouts));
        }
        
        recommendations.extend(self.results.compliance_summary.recommendations.clone());
        
        recommendations
    }
    
    /// Get embedded CSS for HTML reports
    fn get_embedded_css(&self) -> &'static str {
        r#"
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 0; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        header { text-align: center; margin-bottom: 40px; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #333; margin: 0; }
        .subtitle { color: #666; margin: 10px 0 0 0; }
        
        .summary { background: white; padding: 30px; border-radius: 8px; margin-bottom: 30px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .summary-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin-top: 20px; }
        .summary-card { background: #f8f9fa; padding: 20px; border-radius: 6px; text-align: center; }
        .metric-value { font-size: 2.5em; font-weight: bold; color: #2c3e50; }
        .sub-metric { color: #666; margin-top: 5px; }
        
        .grade-badge { display: inline-block; padding: 5px 15px; border-radius: 20px; font-weight: bold; margin-top: 10px; }
        .excellent { background: #d4edda; color: #155724; }
        .very-good { background: #cce5ff; color: #004085; }
        .good { background: #fff3cd; color: #856404; }
        .satisfactory { background: #f8d7da; color: #721c24; }
        .needs-work { background: #f5c6cb; color: #721c24; }
        .major-gaps { background: #f5c6cb; color: #721c24; }
        .incomplete { background: #f5c6cb; color: #721c24; }
        
        .test-results, .feature-analysis, .recommendations, .charts { background: white; padding: 30px; border-radius: 8px; margin-bottom: 30px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        
        .table-container { overflow-x: auto; }
        .results-table { width: 100%; border-collapse: collapse; }
        .results-table th, .results-table td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }
        .results-table th { background: #f8f9fa; font-weight: bold; }
        
        .status-passed { background: #d4edda; }
        .status-failed { background: #f8d7da; }
        .status-error { background: #f5c6cb; }
        .status-timeout { background: #fff3cd; }
        .status-skipped { background: #e9ecef; }
        
        .feature-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; }
        .feature-card { border: 1px solid #ddd; border-radius: 6px; padding: 20px; }
        .feature-stats { display: flex; justify-content: space-between; margin: 15px 0; }
        .stat-label { color: #666; }
        .stat-value { font-weight: bold; }
        .progress-bar { height: 8px; background: #e9ecef; border-radius: 4px; overflow: hidden; margin-top: 10px; }
        .progress-fill { height: 100%; background: #007bff; transition: width 0.3s ease; }
        
        .status-complete { border-left: 4px solid #28a745; }
        .status-partial { border-left: 4px solid #ffc107; }
        .status-minimal { border-left: 4px solid #fd7e14; }
        .status-missing { border-left: 4px solid #dc3545; }
        
        .recommendations-container { display: grid; gap: 20px; }
        .recommendation-card { border: 1px solid #ddd; border-radius: 6px; padding: 20px; }
        .priority-critical { border-left: 4px solid #dc3545; }
        .priority-high { border-left: 4px solid #fd7e14; }
        .priority-medium { border-left: 4px solid #ffc107; }
        .priority-low { border-left: 4px solid #28a745; }
        
        .recommendation-meta { display: flex; gap: 20px; margin: 15px 0; color: #666; font-size: 0.9em; }
        
        .charts-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 30px; }
        .chart-container { text-align: center; }
        
        footer { text-align: center; color: #666; margin-top: 40px; }
        "#
    }
    
    /// Get embedded JavaScript for HTML reports
    fn get_embedded_javascript(&self) -> String {
        let passed = self.results.passed;
        let failed = self.results.failed;
        let errors = self.results.errors;
        let timeouts = self.results.timeouts;
        
        format!(r#"
        // Compliance Chart
        const complianceCtx = document.getElementById('complianceChart').getContext('2d');
        new Chart(complianceCtx, {{
            type: 'doughnut',
            data: {{
                labels: ['Passed', 'Failed', 'Errors', 'Timeouts'],
                datasets: [{{
                    data: [{}, {}, {}, {}],
                    backgroundColor: ['#28a745', '#dc3545', '#6c757d', '#ffc107']
                }}]
            }},
            options: {{
                responsive: true,
                plugins: {{
                    title: {{
                        display: true,
                        text: 'Test Results Distribution'
                    }}
                }}
            }}
        }});
        
        // Feature Chart (placeholder - would need feature data)
        const featureCtx = document.getElementById('featureChart').getContext('2d');
        new Chart(featureCtx, {{
            type: 'bar',
            data: {{
                labels: ['Core', 'Numbers', 'Lists', 'Strings', 'I/O'],
                datasets: [{{
                    label: 'Coverage %',
                    data: [75, 60, 80, 45, 30],
                    backgroundColor: '#007bff'
                }}]
            }},
            options: {{
                responsive: true,
                scales: {{
                    y: {{
                        beginAtZero: true,
                        max: 100
                    }}
                }},
                plugins: {{
                    title: {{
                        display: true,
                        text: 'Feature Coverage'
                    }}
                }}
            }}
        }});
        "#, passed, failed, errors, timeouts)
    }
}

// Data structures for JSON report

#[derive(serde::Serialize)]
struct JsonReport {
    metadata: ReportMetadata,
    execution_summary: ExecutionSummary,
    compliance_summary: super::ComplianceSummary,
    test_results: Vec<TestResult>,
    feature_coverage: HashMap<String, super::FeatureCoverage>,
    detailed_analysis: Option<ComplianceAnalysis>,
    performance_metrics: PerformanceMetrics,
    recommendations: Vec<String>,
}

#[derive(serde::Serialize)]
struct ReportMetadata {
    generated_at: String,
    lambdust_version: String,
    chibi_integration_version: String,
    test_suite_version: String,
}

#[derive(serde::Serialize)]
struct ExecutionSummary {
    total_tests: usize,
    passed: usize,
    failed: usize,
    errors: usize,
    timeouts: usize,
    skipped: usize,
    execution_duration_seconds: f64,
    pass_rate: f64,
}

#[derive(serde::Serialize)]
struct PerformanceMetrics {
    total_execution_time: f64,
    tests_per_second: f64,
    average_test_duration: f64,
    median_test_duration: f64,
    min_test_duration: f64,
    max_test_duration: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_report_generator_creation() {
        let results = create_mock_results();
        let _generator = ReportGenerator::new(&results);
        // Generator should be created without error
    }
    
    #[test]
    fn test_csv_generation() {
        let results = create_mock_results();
        let _generator = ReportGenerator::new(&results);
        let csv = generator.build_test_results_csv();
        assert!(csv.contains("Test Name,Status,Duration"));
    }
    
    #[test]
    fn test_markdown_generation() {
        let results = create_mock_results();
        let _generator = ReportGenerator::new(&results);
        let markdown = generator.build_markdown_content().unwrap();
        assert!(markdown.contains("# Lambdust R7RS Compliance Report"));
        assert!(markdown.contains("## Executive Summary"));
    }
    
    fn create_mock_results() -> TestSuiteResults {
        use super::super::*;
        
        TestSuiteResults {
            config: ChibiIntegrationConfig::default(),
            execution_start: chrono::Utc::now().to_rfc3339(),
            execution_duration: Duration::from_secs(10),
            total_tests: 2,
            passed: 1,
            failed: 1,
            errors: 0,
            timeouts: 0,
            skipped: 0,
            adapted: 0,
            test_results: vec![
                TestResult {
                    test_name: "test1".to_string(),
                    test_file: "file1".to_string(),
                    status: TestStatus::Passed,
                    execution_time: Duration::from_millis(100),
                    error_message: None,
                    stack_trace: None,
                    expected_output: None,
                    actual_output: None,
                },
                TestResult {
                    test_name: "test2".to_string(),
                    test_file: "file2".to_string(),
                    status: TestStatus::Failed,
                    execution_time: Duration::from_millis(200),
                    error_message: Some("Mock error".to_string()),
                    stack_trace: None,
                    expected_output: None,
                    actual_output: None,
                },
            ],
            feature_coverage: HashMap::new(),
            compliance_summary: ComplianceSummary {
                overall_percentage: 50.0,
                core_language_percentage: 60.0,
                standard_library_percentage: 40.0,
                syntax_compliance: 70.0,
                procedure_compliance: 30.0,
                critical_gaps: vec![],
                recommendations: vec![],
            },
        }
    }
}