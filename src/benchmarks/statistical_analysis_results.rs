//! Statistical analysis and comparison structures for benchmark results.
//!
//! This module provides structures for cross-implementation performance comparisons,
//! statistical significance testing, performance rankings, and correlation analysis
//! to enable comprehensive benchmark result interpretation and insights.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Comparison between implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationComparison {
    /// Implementation A
    pub impl_a: String,
    /// Implementation B
    pub impl_b: String,
    /// Performance ratio (A/B)
    pub performance_ratio: f64,
    /// Statistical significance of difference
    pub significance: StatisticalSignificance,
    /// Category-wise comparisons
    pub category_comparisons: HashMap<String, CategoryComparison>,
}

/// Statistical significance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSignificance {
    /// p-value from statistical test
    pub p_value: f64,
    /// Whether difference is statistically significant
    pub is_significant: bool,
    /// Type of statistical test used
    pub test_type: String,
    /// Effect size measure
    pub effect_size: f64,
}

/// Category-specific comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryComparison {
    /// Category name
    pub category: String,
    /// Performance difference (percentage)
    pub performance_difference: f64,
    /// Winner of this category
    pub winner: String,
    /// Confidence in the comparison
    pub confidence: f64,
}

/// Statistical summary across all implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSummary {
    /// Overall performance rankings
    pub performance_rankings: Vec<PerformanceRanking>,
    /// Category leaders
    pub category_leaders: HashMap<String, String>,
    /// Performance distribution statistics
    pub distribution_stats: DistributionStats,
    /// Correlation analysis
    pub correlation_analysis: CorrelationAnalysis,
}

/// Performance ranking entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRanking {
    /// Implementation name
    pub implementation: String,
    /// Overall rank (1 = best)
    pub rank: u32,
    /// Overall performance score
    pub score: f64,
    /// Wins by category
    pub category_wins: u32,
    /// Confidence interval for ranking
    pub ranking_confidence: f64,
}

/// Distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    /// Performance distribution shape
    pub distribution_shape: DistributionShape,
    /// Variance in performance across implementations
    pub performance_variance: f64,
    /// Outlier implementations
    pub outliers: Vec<String>,
}

/// Shape of performance distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionShape {
    /// Normal (Gaussian) distribution
    Normal,
    /// Skewed distribution
    Skewed,
    /// Bimodal distribution with two peaks
    Bimodal,
    /// Uniform distribution
    Uniform,
    /// Unknown or unclassified distribution shape
    Unknown,
}

/// Correlation analysis between different metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationAnalysis {
    /// Correlation between categories
    pub category_correlations: HashMap<String, HashMap<String, f64>>,
    /// Memory vs speed correlation
    pub memory_speed_correlation: f64,
    /// Implementation feature correlations
    pub feature_correlations: HashMap<String, f64>,
}