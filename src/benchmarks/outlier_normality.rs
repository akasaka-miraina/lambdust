//! Outlier Detection and Normality Testing
//!
//! This module contains structures for outlier detection, normality testing,
//! and related statistical assessments for performance data quality analysis.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Outlier analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierAnalysis {
    /// Outliers detected by each method
    pub outliers_by_method: HashMap<OutlierMethod, Vec<OutlierPoint>>,
    /// Consensus outliers (detected by multiple methods)
    pub consensus_outliers: Vec<OutlierPoint>,
    /// Outlier summary statistics
    pub summary: OutlierSummary,
}

/// Outlier detection methods
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum OutlierMethod {
    /// Interquartile range method
    IQR,              // Interquartile range method
    /// Z-score method
    ZScore,           // Z-score method
    /// Modified Z-score method
    ModifiedZScore,   // Modified Z-score method
    /// Grubbs' test for outliers
    Grubbs,           // Grubbs' test
    /// Dixon's test for outliers
    Dixon,            // Dixon's test
    /// Isolation forest method
    Isolation,        // Isolation forest
}

/// Individual outlier point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierPoint {
    /// Index in original dataset
    pub index: usize,
    /// Value
    pub value: f64,
    /// Outlier score (method-dependent)
    pub score: f64,
    /// Dataset/group this outlier belongs to
    pub group: String,
}

/// Summary of outlier analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierSummary {
    /// Total outliers found
    pub total_outliers: usize,
    /// Outliers as percentage of data
    pub outlier_percentage: f64,
    /// Most extreme outlier
    pub most_extreme: Option<OutlierPoint>,
    /// Recommended action
    pub recommendation: OutlierRecommendation,
}

/// Recommended action for outliers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutlierRecommendation {
    /// No action needed for outliers
    NoAction,
    /// Investigate outlier causes
    Investigate,
    /// Remove outliers and rerun analysis
    RemoveAndRerun,
    /// Transform data to reduce outlier impact
    TransformData,
    /// Use robust statistical methods
    UseRobustMethods,
}

/// Normality test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalityTest {
    /// Dataset name
    pub dataset: String,
    /// Shapiro-Wilk test
    pub shapiro_wilk: Option<NormalityTestResult>,
    /// Anderson-Darling test
    pub anderson_darling: Option<NormalityTestResult>,
    /// Kolmogorov-Smirnov test
    pub kolmogorov_smirnov: Option<NormalityTestResult>,
    /// Jarque-Bera test
    pub jarque_bera: Option<NormalityTestResult>,
    /// Overall normality assessment
    pub overall_assessment: NormalityAssessment,
}

/// Individual normality test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalityTestResult {
    /// Test statistic
    pub statistic: f64,
    /// P-value
    pub p_value: f64,
    /// Whether data appears normal
    pub is_normal: bool,
}

/// Overall normality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalityAssessment {
    /// Data follows normal distribution
    Normal,
    /// Data approximately follows normal distribution
    QuasiNormal,
    /// Data does not follow normal distribution
    NonNormal,
    /// Normality assessment is inconclusive
    Inconclusive,
}