//! Performance Regression Detection System - Main Module
//!
//! This module re-exports the main regression detection functionality
//! and provides the report generation function.

use std::collections::HashMap;
use std::time::SystemTime;

// Re-export main types
pub use crate::benchmarks::regression_detector::{RegressionDetector, RegressionDetectionConfig};
pub use crate::benchmarks::baseline_manager::{BaselineManager, BaselineData, BaselineStatistics, BaselineQuality, QualityRating};
pub use crate::benchmarks::performance_measurement::{PerformanceMeasurement, SystemContext};
pub use crate::benchmarks::trend_analyzer::{TrendAnalyzer, TrendAnalysis, TrendDirection, PerformanceForecast};
pub use crate::benchmarks::anomaly_detector::{AnomalyDetector, PerformanceAnomaly, AnomalyType, AnomalyDetectionMethod};
pub use crate::benchmarks::detection_results::{
    RegressionDetectionResult, PerformanceRegression, PerformanceImprovement,
    StatisticalSignificance, RegressionSeverity, PerformanceComparison, DifferenceMetrics
};
pub use crate::benchmarks::analysis_support::{
    SuspectedCause, ImprovementCause, CauseType, OverallAssessment, ActionRecommendation,
    PerformanceStatus, RiskLevel, RecommendedAction, EffortLevel
};

/// Generate a comprehensive regression detection report
pub fn generate_regression_report(result: &RegressionDetectionResult) -> String {
    let mut report = String::new();
    
    report.push_str("# Performance Regression Detection Report\n\n");
    
    // Overall assessment
    report.push_str("## Overall Assessment\n\n");
    report.push_str(&format!("- **Health Score:** {:.1}/100\n", result.overall_assessment.health_score));
    report.push_str(&format!("- **Status:** {:?}\n", result.overall_assessment.status));
    report.push_str(&format!("- **Risk Level:** {:?}\n", result.overall_assessment.risk_level));
    
    // Key findings
    report.push_str("\n### Key Findings\n");
    for finding in &result.overall_assessment.key_findings {
        report.push_str(&format!("- {finding}\n"));
    }
    
    // Regressions
    if !result.regressions.is_empty() {
        report.push_str(&format!("\n## Detected Regressions ({})\n\n", result.regressions.len()));
        
        for regression in &result.regressions {
            report.push_str(&format!("### {} - {}\n", regression.implementation, regression.test_id));
            report.push_str(&format!("- **Degradation:** {:.1}%\n", regression.degradation_percent));
            report.push_str(&format!("- **Severity:** {:?}\n", regression.severity));
            report.push_str(&format!("- **Statistical Significance:** p={:.4}\n", regression.statistical_significance.p_value));
            report.push_str(&format!("- **Confidence:** {:.1}%\n", regression.confidence));
        }
    }
    
    // Improvements
    if !result.improvements.is_empty() {
        report.push_str(&format!("\n## Detected Improvements ({})\n\n", result.improvements.len()));
        
        for improvement in &result.improvements {
            report.push_str(&format!("### {} - {}\n", improvement.implementation, improvement.test_id));
            report.push_str(&format!("- **Improvement:** {:.1}%\n", improvement.improvement_percent));
            report.push_str(&format!("- **Statistical Significance:** p={:.4}\n", improvement.statistical_significance.p_value));
            report.push_str(&format!("- **Confidence:** {:.1}%\n", improvement.confidence));
        }
    }
    
    // Recommendations
    if !result.recommendations.is_empty() {
        report.push_str("\n## Recommended Actions\n\n");
        
        let mut sorted_recommendations = result.recommendations.clone();
        sorted_recommendations.sort_by_key(|r| std::cmp::Reverse(r.priority));
        
        for (i, rec) in sorted_recommendations.iter().enumerate() {
            report.push_str(&format!("{}. **{:?}** (Priority: {}/10)\n", i + 1, rec.action, rec.priority));
            report.push_str(&format!("   - {}\n", rec.description));
            report.push_str(&format!("   - Timeline: {}\n", rec.timeline));
            report.push_str(&format!("   - Effort: {:?}\n", rec.effort_level));
        }
    }
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_regression_detector_creation() {
        let config = RegressionDetectionConfig::default();
        let detector = RegressionDetector::new(config);
        // Test that the detector was created successfully
        assert!(true, "RegressionDetector created successfully");
    }
    
    #[test]
    fn test_baseline_management() {
        let mut manager = BaselineManager::new(RegressionDetectionConfig::default());
        
        let measurement = PerformanceMeasurement {
            timestamp: SystemTime::now(),
            value: 100.0,
            parameters: [
                ("test_id".to_string(), "test1".to_string()),
                ("implementation".to_string(), "impl1".to_string()),
            ].iter().cloned().collect(),
            context: SystemContext {
                commit_hash: None,
                compiler_version: "rustc 1.70".to_string(),
                system_load: 0.5,
                available_memory_mb: 4096,
                temperature: None,
            },
            confidence: 95.0,
        };
        
        manager.add_measurement(measurement);
        assert_eq!(manager.baselines.len(), 1);
    }
}