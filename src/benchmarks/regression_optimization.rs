//! Regression detection and optimization recommendation structures.
//!
//! This module provides structures for detecting performance regressions,
//! analyzing performance trends over time, and generating actionable
//! optimization recommendations based on benchmark results.

use std::time::Duration;
use serde::{Deserialize, Serialize};
use super::statistical_analysis_results::StatisticalSignificance;

/// Performance regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    /// Baseline results for comparison
    pub baseline: Option<String>, // Path to baseline results
    /// Detected regressions
    pub regressions: Vec<PerformanceRegression>,
    /// Performance improvements
    pub improvements: Vec<PerformanceImprovement>,
    /// Overall trend analysis
    pub trend_analysis: TrendAnalysis,
}

/// Detected performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    /// Implementation affected
    pub implementation: String,
    /// Test case affected
    pub test_case: String,
    /// Category affected
    pub category: String,
    /// Performance change (negative percentage)
    pub performance_change: f64,
    /// Statistical significance
    pub significance: StatisticalSignificance,
    /// Severity of regression
    pub severity: RegressionSeverity,
}

/// Performance improvement detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovement {
    /// Implementation improved
    pub implementation: String,
    /// Test case improved
    pub test_case: String,
    /// Category improved
    pub category: String,
    /// Performance change (positive percentage)
    pub performance_change: f64,
    /// Statistical significance
    pub significance: StatisticalSignificance,
}

/// Severity of performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    /// Minor regression (< 10% performance loss)
    Minor,      // < 10% performance loss
    /// Moderate regression (10-25% performance loss)
    Moderate,   // 10-25% performance loss
    /// Major regression (25-50% performance loss)
    Major,      // 25-50% performance loss
    /// Critical regression (> 50% performance loss)
    Critical,   // > 50% performance loss
}

/// Trend analysis over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Overall trend direction
    pub overall_trend: TrendDirection,
    /// Performance velocity (change per time unit)
    pub performance_velocity: f64,
    /// Prediction for future performance
    pub performance_forecast: PerformanceForecast,
}

/// Direction of performance trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Performance is consistently improving over time
    Improving,
    /// Performance is stable with minimal variation
    Stable,
    /// Performance is consistently declining over time
    Declining,
    /// Performance shows high volatility and unpredictable changes
    Volatile,
}

/// Performance forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceForecast {
    /// Predicted performance change over next period
    pub predicted_change: f64,
    /// Confidence in prediction
    pub confidence: f64,
    /// Time horizon for prediction
    pub time_horizon: Duration,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Priority level (1-10, 10 = highest)
    pub priority: u8,
    /// Title of recommendation
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Target implementation(s)
    pub target_implementations: Vec<String>,
    /// Affected categories
    pub affected_categories: Vec<String>,
    /// Expected performance improvement
    pub expected_improvement: f64,
    /// Implementation difficulty (1-10, 10 = hardest)
    pub difficulty: u8,
    /// Supporting evidence
    pub evidence: Vec<String>,
    /// Related optimizations
    pub related_recommendations: Vec<String>,
}