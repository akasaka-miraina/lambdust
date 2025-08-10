//! Detection results for performance regression analysis.
//!
//! This module contains the result structures and data types
//! for reporting performance regressions, improvements, and comparisons.

use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use super::regression_detector::RegressionDetectionConfig;
use super::baseline_manager::BaselineStatistics;
use super::trend_analyzer::TrendAnalysis;
use super::anomaly_detector::PerformanceAnomaly;
use super::analysis_support::{OverallAssessment, ActionRecommendation, SuspectedCause, ImprovementCause};

/// Comprehensive results from performance regression analysis.
/// 
/// Contains all detected regressions, improvements, trends, and anomalies
/// with recommendations for addressing performance issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionDetectionResult {
    /// Timestamp of analysis
    pub timestamp: SystemTime,
    /// Configuration used
    pub config: RegressionDetectionConfig,
    /// Detected regressions
    pub regressions: Vec<PerformanceRegression>,
    /// Detected improvements
    pub improvements: Vec<PerformanceImprovement>,
    /// Trend analysis results
    pub trends: HashMap<String, TrendAnalysis>,
    /// Anomaly detection results
    pub anomalies: Vec<PerformanceAnomaly>,
    /// Overall assessment
    pub overall_assessment: OverallAssessment,
    /// Recommendations for action
    pub recommendations: Vec<ActionRecommendation>,
}

/// Detected performance regression with detailed analysis.
/// 
/// Represents a statistically significant performance degradation
/// with severity assessment and suspected root causes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    /// Test identifier
    pub test_id: String,
    /// Implementation affected
    pub implementation: String,
    /// Performance degradation percentage
    pub degradation_percent: f64,
    /// Statistical significance
    pub statistical_significance: StatisticalSignificance,
    /// Severity level
    pub severity: RegressionSeverity,
    /// Current vs baseline comparison
    pub current_vs_baseline: PerformanceComparison,
    /// Suspected causes
    pub suspected_causes: Vec<SuspectedCause>,
    /// First detected timestamp
    pub first_detected: SystemTime,
    /// Confidence in detection
    pub confidence: f64,
}

/// Detected performance improvement with analysis details.
/// 
/// Represents a statistically significant performance improvement
/// with likely contributing factors and impact assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovement {
    /// Test identifier
    pub test_id: String,
    /// Implementation improved
    pub implementation: String,
    /// Performance improvement percentage
    pub improvement_percent: f64,
    /// Statistical significance
    pub statistical_significance: StatisticalSignificance,
    /// Current vs baseline comparison
    pub current_vs_baseline: PerformanceComparison,
    /// Likely causes
    pub likely_causes: Vec<ImprovementCause>,
    /// First detected timestamp
    pub first_detected: SystemTime,
    /// Confidence in detection
    pub confidence: f64,
}

/// Statistical significance assessment for performance changes.
/// 
/// Provides statistical test results and effect size measurements
/// to validate the reliability of detected performance changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSignificance {
    /// P-value from statistical test
    pub p_value: f64,
    /// Whether statistically significant
    pub is_significant: bool,
    /// Type of test used
    pub test_method: String,
    /// Effect size
    pub effect_size: f64,
}

/// Severity of performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    /// Minor performance regression (5-15% degradation)
    Minor,      // 5-15% degradation
    /// Moderate performance regression (15-30% degradation)
    Moderate,   // 15-30% degradation
    /// Major performance regression (30-50% degradation)
    Major,      // 30-50% degradation
    /// Critical performance regression (>50% degradation)
    Critical,   // >50% degradation
}

/// Comparison between current and baseline performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    /// Baseline statistics
    pub baseline: BaselineStatistics,
    /// Current measurements
    pub current_values: Vec<f64>,
    /// Current statistics
    pub current_statistics: BaselineStatistics,
    /// Difference metrics
    pub difference: DifferenceMetrics,
}

/// Metrics describing the difference between baseline and current
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferenceMetrics {
    /// Mean difference (current - baseline)
    pub mean_difference: f64,
    /// Percentage change
    pub percent_change: f64,
    /// Standard error of difference
    pub standard_error: f64,
    /// Cohen's d effect size
    pub effect_size: f64,
    /// Confidence interval for difference
    pub confidence_interval: (f64, f64),
}