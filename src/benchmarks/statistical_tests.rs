//! Statistical Test Results and Comparisons
//!
//! This module contains structures and types for various statistical tests
//! including t-tests, non-parametric tests, ANOVA, and pairwise comparisons.

use serde::{Deserialize, Serialize};
use super::effect_analysis::{EffectSize, ConfidenceInterval};

/// Statistical comparison between two performance implementations.
/// 
/// Performs multiple statistical tests (parametric and non-parametric)
/// to determine significant differences with effect size analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairwiseComparison {
    /// First implementation
    pub group_a: String,
    /// Second implementation
    pub group_b: String,
    /// T-test results
    pub t_test: TTestResult,
    /// Mann-Whitney U test results (non-parametric)
    pub mann_whitney: MannWhitneyResult,
    /// Welch's t-test (unequal variances)
    pub welch_test: WelchTestResult,
    /// Effect size measures
    pub effect_size: EffectSize,
    /// Confidence interval for difference
    pub difference_ci: ConfidenceInterval,
    /// Overall statistical significance
    pub is_significant: bool,
    /// Practical significance
    pub is_practically_significant: bool,
}

/// Student's t-test statistical analysis results.
/// 
/// Provides t-statistic, degrees of freedom, and significance
/// for parametric comparison of performance means.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTestResult {
    /// T-statistic
    pub t_statistic: f64,
    /// Degrees of freedom
    pub degrees_of_freedom: f64,
    /// P-value
    pub p_value: f64,
    /// Test type
    pub test_type: TTestType,
    /// Whether result is statistically significant
    pub significant: bool,
}

/// Classification of different t-test variants and assumptions.
/// 
/// Specifies the type of t-test used based on sample characteristics
/// and variance assumptions for appropriate statistical inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TTestType {
    /// One-sample t-test
    OneSample,
    /// Two-sample t-test with equal variances assumed
    TwoSampleEqual,    // Equal variances assumed
    /// Two-sample t-test with unequal variances (Welch's test)
    TwoSampleUnequal,  // Unequal variances (Welch's test)
    /// Paired t-test for dependent samples
    Paired,
}

/// Mann-Whitney U test results for non-parametric comparison.
/// 
/// Provides robust statistical comparison without normality assumptions
/// using rank-based methods for performance data analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MannWhitneyResult {
    /// U statistic
    pub u_statistic: f64,
    /// Z-score (for large samples)
    pub z_score: f64,
    /// P-value
    pub p_value: f64,
    /// Whether result is statistically significant
    pub significant: bool,
}

/// Welch's t-test results (unequal variances)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelchTestResult {
    /// T-statistic
    pub t_statistic: f64,
    /// Welch-Satterthwaite degrees of freedom
    pub degrees_of_freedom: f64,
    /// P-value
    pub p_value: f64,
    /// Whether result is statistically significant
    pub significant: bool,
}

/// ANOVA results for multiple group comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnovaResult {
    /// F-statistic
    pub f_statistic: f64,
    /// Between-groups degrees of freedom
    pub df_between: f64,
    /// Within-groups degrees of freedom
    pub df_within: f64,
    /// Between-groups sum of squares
    pub ss_between: f64,
    /// Within-groups sum of squares
    pub ss_within: f64,
    /// Mean square between
    pub ms_between: f64,
    /// Mean square within
    pub ms_within: f64,
    /// P-value
    pub p_value: f64,
    /// Whether result is statistically significant
    pub significant: bool,
    /// Post-hoc test results
    pub post_hoc: Option<PostHocTest>,
}

/// Post-hoc test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostHocTest {
    /// Test method used
    pub method: PostHocMethod,
    /// Pairwise comparisons
    pub comparisons: Vec<PostHocComparison>,
}

/// Post-hoc test methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostHocMethod {
    /// Tukey's Honestly Significant Difference
    Tukey,
    /// Bonferroni correction
    Bonferroni,
    /// Holm-Bonferroni correction
    Holm,
    /// Duncan's multiple range test
    Duncan,
    /// Student-Newman-Keuls test
    StudentNewmanKeuls,
}

/// Individual post-hoc comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostHocComparison {
    /// First group
    pub group_a: String,
    /// Second group
    pub group_b: String,
    /// Mean difference
    pub mean_difference: f64,
    /// Adjusted p-value
    pub adjusted_p_value: f64,
    /// Whether significant after adjustment
    pub significant: bool,
}