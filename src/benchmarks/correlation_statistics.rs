//! Correlation Analysis and Core Statistical Data Structures
//!
//! This module contains structures for correlation analysis, descriptive statistics,
//! and core configuration/result structures for statistical analysis.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Configuration parameters for statistical analysis procedures.
/// 
/// Controls the rigor and behavior of statistical tests including
/// significance levels, effect size thresholds, and correction methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysisConfig {
    /// Confidence level for statistical tests (e.g., 0.95 for 95%)
    pub confidence_level: f64,
    /// Alpha level for hypothesis tests
    pub alpha_level: f64,
    /// Minimum effect size to consider meaningful
    pub min_effect_size: f64,
    /// Outlier detection sensitivity
    pub outlier_sensitivity: f64,
    /// Bootstrap samples for non-parametric statistics
    pub bootstrap_samples: u32,
    /// Whether to apply multiple comparison corrections
    pub multiple_comparison_correction: bool,
}

impl Default for StatisticalAnalysisConfig {
    fn default() -> Self {
        Self {
            confidence_level: 0.95,
            alpha_level: 0.05,
            min_effect_size: 0.2,
            outlier_sensitivity: 1.5,
            bootstrap_samples: 1000,
            multiple_comparison_correction: true,
        }
    }
}

/// Comprehensive statistical analysis results for benchmark comparisons.
/// 
/// Contains descriptive statistics, hypothesis test results, outlier analysis,
/// and advanced statistical measures for performance comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysisResult {
    /// Summary statistics for each dataset
    pub descriptive_stats: HashMap<String, DescriptiveStatistics>,
    /// Pairwise comparisons between implementations
    pub pairwise_comparisons: Vec<super::statistical_tests::PairwiseComparison>,
    /// ANOVA results for multiple group comparison
    pub anova_result: Option<super::statistical_tests::AnovaResult>,
    /// Outlier detection results
    pub outlier_analysis: super::outlier_normality::OutlierAnalysis,
    /// Normality test results
    pub normality_tests: HashMap<String, super::outlier_normality::NormalityTest>,
    /// Correlation analysis
    pub correlation_matrix: CorrelationMatrix,
    /// Effect size analysis
    pub effect_sizes: HashMap<String, super::effect_analysis::EffectSize>,
    /// Bootstrap confidence intervals
    pub bootstrap_intervals: HashMap<String, super::effect_analysis::BootstrapInterval>,
}

/// Comprehensive descriptive statistics for a performance dataset.
/// 
/// Provides central tendency, variability, distribution shape,
/// and percentile information for statistical analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptiveStatistics {
    /// Dataset identifier
    pub name: String,
    /// Number of observations
    pub n: usize,
    /// Arithmetic mean
    pub mean: f64,
    /// Median (50th percentile)
    pub median: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Variance
    pub variance: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Range (max - min)
    pub range: f64,
    /// Interquartile range
    pub iqr: f64,
    /// Percentiles
    pub percentiles: HashMap<u8, f64>,
    /// Skewness measure
    pub skewness: f64,
    /// Kurtosis measure
    pub kurtosis: f64,
    /// Coefficient of variation
    pub coefficient_of_variation: f64,
    /// Standard error of the mean
    pub standard_error: f64,
}

/// Correlation analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    /// Variable names
    pub variables: Vec<String>,
    /// Pearson correlation coefficients
    pub pearson: Vec<Vec<f64>>,
    /// Spearman rank correlations
    pub spearman: Vec<Vec<f64>>,
    /// Kendall's tau correlations
    pub kendall: Vec<Vec<f64>>,
    /// Significance matrix (p-values)
    pub p_values: Vec<Vec<f64>>,
    /// Strong correlations (|r| > threshold)
    pub strong_correlations: Vec<CorrelationPair>,
}

/// Correlation between two variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationPair {
    /// First variable
    pub var_a: String,
    /// Second variable
    pub var_b: String,
    /// Correlation coefficient
    pub correlation: f64,
    /// Correlation type
    pub correlation_type: CorrelationType,
    /// P-value
    pub p_value: f64,
    /// Strength interpretation
    pub strength: CorrelationStrength,
}

/// Type of correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrelationType {
    /// Pearson product-moment correlation
    Pearson,
    /// Spearman rank correlation
    Spearman,
    /// Kendall's tau correlation
    Kendall,
}

/// Correlation strength interpretation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrelationStrength {
    /// Negligible correlation strength
    Negligible,
    /// Weak correlation strength
    Weak,
    /// Moderate correlation strength
    Moderate,
    /// Strong correlation strength
    Strong,
    /// Very strong correlation strength
    VeryStrong,
}