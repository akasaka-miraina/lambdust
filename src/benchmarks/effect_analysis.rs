//! Effect Size and Confidence Interval Analysis
//!
//! This module contains structures for effect size measures, confidence intervals,
//! and bootstrap analysis for statistical significance assessment.

use serde::{Deserialize, Serialize};

/// Effect size measures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSize {
    /// Cohen's d
    pub cohens_d: f64,
    /// Glass's delta
    pub glass_delta: f64,
    /// Hedges' g
    pub hedges_g: f64,
    /// Cliff's delta (non-parametric)
    pub cliffs_delta: f64,
    /// Effect size interpretation
    pub interpretation: EffectSizeInterpretation,
}

/// Interpretation of effect size magnitude
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectSizeInterpretation {
    /// Negligible effect size
    Negligible,
    /// Small effect size
    Small,
    /// Medium effect size
    Medium,
    /// Large effect size
    Large,
    /// Very large effect size
    VeryLarge,
}

/// Confidence interval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Lower bound
    pub lower: f64,
    /// Upper bound
    pub upper: f64,
    /// Confidence level
    pub confidence_level: f64,
    /// Method used to calculate interval
    pub method: ConfidenceIntervalMethod,
}

/// Method for calculating confidence intervals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceIntervalMethod {
    /// Normal distribution assumption
    Normal,
    /// Bootstrap confidence interval
    Bootstrap,
    /// T-distribution confidence interval
    TDistribution,
    /// Percentile-based confidence interval
    Percentile,
}

/// Bootstrap confidence interval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapInterval {
    /// Statistic name
    pub statistic: String,
    /// Original estimate
    pub original_estimate: f64,
    /// Bootstrap mean
    pub bootstrap_mean: f64,
    /// Bootstrap standard error
    pub bootstrap_se: f64,
    /// Bootstrap bias
    pub bias: f64,
    /// Bias-corrected estimate
    pub bias_corrected: f64,
    /// Percentile confidence interval
    pub percentile_ci: ConfidenceInterval,
    /// Bias-corrected and accelerated (BCa) interval
    pub bca_ci: Option<ConfidenceInterval>,
}