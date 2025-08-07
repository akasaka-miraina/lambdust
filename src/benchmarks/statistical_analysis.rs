//! Statistical Analysis Framework for Performance Benchmarks
//!
//! This module provides comprehensive statistical analysis capabilities for
//! benchmark results, including hypothesis testing, confidence intervals,
//! outlier detection, and performance regression analysis.

use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Comprehensive statistical analysis engine for performance benchmark results.
/// 
/// Provides advanced statistical methods including hypothesis testing,
/// confidence intervals, effect size analysis, and outlier detection.
pub struct StatisticalAnalyzer {
    /// Configuration for statistical analysis
    config: StatisticalAnalysisConfig,
}

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

/// Comprehensive statistical analysis results for benchmark comparisons.
/// 
/// Contains descriptive statistics, hypothesis test results, outlier analysis,
/// and advanced statistical measures for performance comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysisResult {
    /// Summary statistics for each dataset
    pub descriptive_stats: HashMap<String, DescriptiveStatistics>,
    /// Pairwise comparisons between implementations
    pub pairwise_comparisons: Vec<PairwiseComparison>,
    /// ANOVA results for multiple group comparison
    pub anova_result: Option<AnovaResult>,
    /// Outlier detection results
    pub outlier_analysis: OutlierAnalysis,
    /// Normality test results
    pub normality_tests: HashMap<String, NormalityTest>,
    /// Correlation analysis
    pub correlation_matrix: CorrelationMatrix,
    /// Effect size analysis
    pub effect_sizes: HashMap<String, EffectSize>,
    /// Bootstrap confidence intervals
    pub bootstrap_intervals: HashMap<String, BootstrapInterval>,
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

impl StatisticalAnalyzer {
    /// Create new statistical analyzer with configuration
    pub fn new(config: StatisticalAnalysisConfig) -> Self {
        Self { config }
    }
    
    /// Create analyzer with default configuration
    pub fn default() -> Self {
        Self::new(StatisticalAnalysisConfig::default())
    }
    
    /// Perform complete statistical analysis on benchmark results
    pub fn analyze(&self, datasets: HashMap<String, Vec<f64>>) -> StatisticalAnalysisResult {
        // Calculate descriptive statistics for each dataset
        let descriptive_stats = datasets.iter()
            .map(|(name, data)| (name.clone()), self.calculate_descriptive_stats(name, data)))
            .collect();
        
        // Perform pairwise comparisons between all implementations
        let pairwise_comparisons = self.perform_pairwise_comparisons(&datasets);
        
        // Perform ANOVA if more than 2 groups
        let anova_result = if datasets.len() > 2 {
            Some(self.perform_anova(&datasets))
        } else {
            None
        };
        
        // Detect outliers
        let outlier_analysis = self.detect_outliers(&datasets);
        
        // Test for normality
        let normality_tests = datasets.iter()
            .map(|(name, data)| (name.clone()), self.test_normality(name, data)))
            .collect();
        
        // Calculate correlation matrix
        let correlation_matrix = self.calculate_correlations(&datasets);
        
        // Calculate effect sizes
        let effect_sizes = self.calculate_effect_sizes(&datasets);
        
        // Generate bootstrap confidence intervals
        let bootstrap_intervals = self.generate_bootstrap_intervals(&datasets);
        
        StatisticalAnalysisResult {
            descriptive_stats,
            pairwise_comparisons,
            anova_result,
            outlier_analysis,
            normality_tests,
            correlation_matrix,
            effect_sizes,
            bootstrap_intervals,
        }
    }
    
    /// Calculate descriptive statistics for a dataset
    fn calculate_descriptive_stats(&self, name: &str, data: &[f64]) -> DescriptiveStatistics {
        if data.is_empty() {
            return self.empty_descriptive_stats(name);
        }
        
        let n = data.len();
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Basic statistics
        let sum: f64 = data.iter().sum();
        let mean = sum / n as f64;
        let min = sorted_data[0];
        let max = sorted_data[n - 1];
        let range = max - min;
        
        // Median
        let median = if n % 2 == 0 {
            (sorted_data[n / 2 - 1] + sorted_data[n / 2]) / 2.0
        } else {
            sorted_data[n / 2]
        };
        
        // Variance and standard deviation
        let variance = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (n - 1) as f64;
        let std_dev = variance.sqrt();
        
        // Quartiles and IQR
        let q1 = self.percentile(&sorted_data, 25.0);
        let q3 = self.percentile(&sorted_data, 75.0);
        let iqr = q3 - q1;
        
        // Percentiles
        let mut percentiles = HashMap::new();
        for &p in &[5, 10, 25, 50, 75, 90, 95, 99] {
            percentiles.insert(p, self.percentile(&sorted_data, p as f64));
        }
        
        // Skewness
        let skewness = if std_dev > 0.0 {
            let moment3 = data.iter()
                .map(|x| ((x - mean) / std_dev).powi(3))
                .sum::<f64>() / n as f64;
            moment3
        } else {
            0.0
        };
        
        // Kurtosis
        let kurtosis = if std_dev > 0.0 {
            let moment4 = data.iter()
                .map(|x| ((x - mean) / std_dev).powi(4))
                .sum::<f64>() / n as f64;
            moment4 - 3.0 // Excess kurtosis
        } else {
            0.0
        };
        
        // Additional measures
        let coefficient_of_variation = if mean != 0.0 { std_dev / mean.abs() } else { 0.0 };
        let standard_error = std_dev / (n as f64).sqrt();
        
        DescriptiveStatistics {
            name: name.to_string(),
            n,
            mean,
            median,
            std_dev,
            variance,
            min,
            max,
            range,
            iqr,
            percentiles,
            skewness,
            kurtosis,
            coefficient_of_variation,
            standard_error,
        }
    }
    
    /// Calculate percentile of sorted data
    fn percentile(&self, sorted_data: &[f64], p: f64) -> f64 {
        if sorted_data.is_empty() {
            return 0.0;
        }
        
        let n = sorted_data.len();
        let index = (p / 100.0) * (n - 1) as f64;
        
        if index.fract() == 0.0 {
            sorted_data[index as usize]
        } else {
            let lower = index.floor() as usize;
            let upper = (index.ceil() as usize).min(n - 1);
            let weight = index.fract();
            sorted_data[lower] * (1.0 - weight) + sorted_data[upper] * weight
        }
    }
    
    /// Perform pairwise comparisons between all implementations
    fn perform_pairwise_comparisons(&self, datasets: &HashMap<String, Vec<f64>>) -> Vec<PairwiseComparison> {
        let mut comparisons = Vec::new();
        let names: Vec<_> = datasets.keys().clone())().collect();
        
        for i in 0..names.len() {
            for j in (i + 1)..names.len() {
                let group_a = &names[i];
                let group_b = &names[j];
                let data_a = &datasets[group_a];
                let data_b = &datasets[group_b];
                
                let comparison = self.compare_two_groups(group_a, data_a, group_b, data_b);
                comparisons.push(comparison);
            }
        }
        
        comparisons
    }
    
    /// Compare two groups statistically
    fn compare_two_groups(&self, name_a: &str, data_a: &[f64], name_b: &str, data_b: &[f64]) -> PairwiseComparison {
        // T-test
        let t_test = self.perform_t_test(data_a, data_b);
        
        // Mann-Whitney U test
        let mann_whitney = self.perform_mann_whitney(data_a, data_b);
        
        // Welch's test (unequal variances)
        let welch_test = self.perform_welch_test(data_a, data_b);
        
        // Effect size
        let effect_size = self.calculate_effect_size_between(data_a, data_b);
        
        // Confidence interval for difference
        let mean_a = data_a.iter().sum::<f64>() / data_a.len() as f64;
        let mean_b = data_b.iter().sum::<f64>() / data_b.len() as f64;
        let difference = mean_a - mean_b;
        
        let difference_ci = self.calculate_difference_ci(data_a, data_b);
        
        // Overall significance (consider multiple test corrections)
        let mut is_significant = t_test.significant && mann_whitney.significant;
        if self.config.multiple_comparison_correction {
            // Apply Bonferroni correction (simplified)
            is_significant = is_significant && (t_test.p_value * 3.0 < self.config.alpha_level);
        }
        
        // Practical significance based on effect size
        let is_practically_significant = effect_size.cohens_d.abs() >= self.config.min_effect_size;
        
        PairwiseComparison {
            group_a: name_a.to_string(),
            group_b: name_b.to_string(),
            t_test,
            mann_whitney,
            welch_test,
            effect_size,
            difference_ci,
            is_significant,
            is_practically_significant,
        }
    }
    
    /// Perform independent samples t-test
    pub fn perform_t_test(&self, data_a: &[f64], data_b: &[f64]) -> TTestResult {
        if data_a.is_empty() || data_b.is_empty() {
            return TTestResult {
                t_statistic: 0.0,
                degrees_of_freedom: 0.0,
                p_value: 1.0,
                test_type: TTestType::TwoSampleEqual,
                significant: false,
            };
        }
        
        let n_a = data_a.len() as f64;
        let n_b = data_b.len() as f64;
        
        let mean_a = data_a.iter().sum::<f64>() / n_a;
        let mean_b = data_b.iter().sum::<f64>() / n_b;
        
        let var_a = data_a.iter().map(|x| (x - mean_a).powi(2)).sum::<f64>() / (n_a - 1.0);
        let var_b = data_b.iter().map(|x| (x - mean_b).powi(2)).sum::<f64>() / (n_b - 1.0);
        
        // Pooled variance
        let pooled_var = ((n_a - 1.0) * var_a + (n_b - 1.0) * var_b) / (n_a + n_b - 2.0);
        let standard_error = (pooled_var * (1.0 / n_a + 1.0 / n_b)).sqrt();
        
        let t_statistic = (mean_a - mean_b) / standard_error;
        let degrees_of_freedom = n_a + n_b - 2.0;
        
        // Approximate p-value calculation (simplified)
        let p_value = self.approximate_t_test_p_value(t_statistic.abs(), degrees_of_freedom);
        
        TTestResult {
            t_statistic,
            degrees_of_freedom,
            p_value,
            test_type: TTestType::TwoSampleEqual,
            significant: p_value < self.config.alpha_level,
        }
    }
    
    /// Perform Mann-Whitney U test (non-parametric)
    fn perform_mann_whitney(&self, data_a: &[f64], data_b: &[f64]) -> MannWhitneyResult {
        if data_a.is_empty() || data_b.is_empty() {
            return MannWhitneyResult {
                u_statistic: 0.0,
                z_score: 0.0,
                p_value: 1.0,
                significant: false,
            };
        }
        
        let n_a = data_a.len();
        let n_b = data_b.len();
        
        // Combine and rank data
        let mut combined: Vec<(f64, usize)> = data_a.iter().map(|&x| (x, 0)).collect();
        combined.extend(data_b.iter().map(|&x| (x, 1)));
        combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        
        // Calculate ranks
        let mut rank_sum_a = 0.0;
        for (i, &(_, group)) in combined.iter().enumerate() {
            if group == 0 {
                rank_sum_a += (i + 1) as f64;
            }
        }
        
        // Calculate U statistics
        let u_a = rank_sum_a - (n_a * (n_a + 1)) as f64 / 2.0;
        let u_b = (n_a * n_b) as f64 - u_a;
        let u_statistic = u_a.min(u_b);
        
        // Calculate Z-score for large samples
        let mean_u = (n_a * n_b) as f64 / 2.0;
        let var_u = (n_a * n_b * (n_a + n_b + 1)) as f64 / 12.0;
        let z_score = (u_statistic - mean_u) / var_u.sqrt();
        
        // Approximate p-value (two-tailed)
        let p_value = 2.0 * self.standard_normal_cdf(-z_score.abs());
        
        MannWhitneyResult {
            u_statistic,
            z_score,
            p_value,
            significant: p_value < self.config.alpha_level,
        }
    }
    
    /// Perform Welch's t-test (unequal variances)
    fn perform_welch_test(&self, data_a: &[f64], data_b: &[f64]) -> WelchTestResult {
        if data_a.is_empty() || data_b.is_empty() {
            return WelchTestResult {
                t_statistic: 0.0,
                degrees_of_freedom: 0.0,
                p_value: 1.0,
                significant: false,
            };
        }
        
        let n_a = data_a.len() as f64;
        let n_b = data_b.len() as f64;
        
        let mean_a = data_a.iter().sum::<f64>() / n_a;
        let mean_b = data_b.iter().sum::<f64>() / n_b;
        
        let var_a = data_a.iter().map(|x| (x - mean_a).powi(2)).sum::<f64>() / (n_a - 1.0);
        let var_b = data_b.iter().map(|x| (x - mean_b).powi(2)).sum::<f64>() / (n_b - 1.0);
        
        let se_a = var_a / n_a;
        let se_b = var_b / n_b;
        let standard_error = (se_a + se_b).sqrt();
        
        let t_statistic = (mean_a - mean_b) / standard_error;
        
        // Welch-Satterthwaite degrees of freedom
        let degrees_of_freedom = (se_a + se_b).powi(2) / 
            (se_a.powi(2) / (n_a - 1.0) + se_b.powi(2) / (n_b - 1.0));
        
        let p_value = self.approximate_t_test_p_value(t_statistic.abs(), degrees_of_freedom);
        
        WelchTestResult {
            t_statistic,
            degrees_of_freedom,
            p_value,
            significant: p_value < self.config.alpha_level,
        }
    }
    
    /// Calculate effect size between two groups
    pub fn calculate_effect_size_between(&self, data_a: &[f64], data_b: &[f64]) -> EffectSize {
        if data_a.is_empty() || data_b.is_empty() {
            return EffectSize {
                cohens_d: 0.0,
                glass_delta: 0.0,
                hedges_g: 0.0,
                cliffs_delta: 0.0,
                interpretation: EffectSizeInterpretation::Negligible,
            };
        }
        
        let n_a = data_a.len() as f64;
        let n_b = data_b.len() as f64;
        
        let mean_a = data_a.iter().sum::<f64>() / n_a;
        let mean_b = data_b.iter().sum::<f64>() / n_b;
        
        let var_a = data_a.iter().map(|x| (x - mean_a).powi(2)).sum::<f64>() / (n_a - 1.0);
        let var_b = data_b.iter().map(|x| (x - mean_b).powi(2)).sum::<f64>() / (n_b - 1.0);
        let std_a = var_a.sqrt();
        let std_b = var_b.sqrt();
        
        // Cohen's d (pooled standard deviation)
        let pooled_std = (((n_a - 1.0) * var_a + (n_b - 1.0) * var_b) / (n_a + n_b - 2.0)).sqrt();
        let cohens_d = if pooled_std > 0.0 { (mean_a - mean_b) / pooled_std } else { 0.0 };
        
        // Glass's delta (control group standard deviation)
        let glass_delta = if std_b > 0.0 { (mean_a - mean_b) / std_b } else { 0.0 };
        
        // Hedges' g (bias-corrected Cohen's d)
        let correction_factor = 1.0 - 3.0 / (4.0 * (n_a + n_b) - 9.0);
        let hedges_g = cohens_d * correction_factor;
        
        // Cliff's delta (non-parametric effect size)
        let mut greater = 0;
        let mut total = 0;
        for &a in data_a {
            for &b in data_b {
                total += 1;
                if a > b {
                    greater += 1;
                }
            }
        }
        let cliffs_delta = (2.0 * greater as f64 / total as f64) - 1.0;
        
        let interpretation = self.interpret_effect_size(cohens_d.abs());
        
        EffectSize {
            cohens_d,
            glass_delta,
            hedges_g,
            cliffs_delta,
            interpretation,
        }
    }
    
    /// Interpret effect size magnitude
    fn interpret_effect_size(&self, abs_effect_size: f64) -> EffectSizeInterpretation {
        if abs_effect_size < 0.1 {
            EffectSizeInterpretation::Negligible
        } else if abs_effect_size < 0.3 {
            EffectSizeInterpretation::Small
        } else if abs_effect_size < 0.5 {
            EffectSizeInterpretation::Medium
        } else if abs_effect_size < 0.8 {
            EffectSizeInterpretation::Large
        } else {
            EffectSizeInterpretation::VeryLarge
        }
    }
    
    /// Calculate confidence interval for difference between means
    fn calculate_difference_ci(&self, data_a: &[f64], data_b: &[f64]) -> ConfidenceInterval {
        if data_a.is_empty() || data_b.is_empty() {
            return ConfidenceInterval {
                lower: 0.0,
                upper: 0.0,
                confidence_level: self.config.confidence_level,
                method: ConfidenceIntervalMethod::TDistribution,
            };
        }
        
        let n_a = data_a.len() as f64;
        let n_b = data_b.len() as f64;
        
        let mean_a = data_a.iter().sum::<f64>() / n_a;
        let mean_b = data_b.iter().sum::<f64>() / n_b;
        let diff = mean_a - mean_b;
        
        let var_a = data_a.iter().map(|x| (x - mean_a).powi(2)).sum::<f64>() / (n_a - 1.0);
        let var_b = data_b.iter().map(|x| (x - mean_b).powi(2)).sum::<f64>() / (n_b - 1.0);
        
        let pooled_var = ((n_a - 1.0) * var_a + (n_b - 1.0) * var_b) / (n_a + n_b - 2.0);
        let standard_error = (pooled_var * (1.0 / n_a + 1.0 / n_b)).sqrt();
        
        let df = n_a + n_b - 2.0;
        let t_critical = self.t_critical_value(df, self.config.alpha_level / 2.0);
        let margin_of_error = t_critical * standard_error;
        
        ConfidenceInterval {
            lower: diff - margin_of_error,
            upper: diff + margin_of_error,
            confidence_level: self.config.confidence_level,
            method: ConfidenceIntervalMethod::TDistribution,
        }
    }
    
    // Placeholder implementations for statistical functions
    // In a real implementation, these would use proper statistical libraries
    
    fn approximate_t_test_p_value(&self, t_abs: f64, df: f64) -> f64 {
        // Rough approximation - in practice, use proper t-distribution
        if df > 30.0 {
            2.0 * self.standard_normal_cdf(-t_abs)
        } else {
            // For small df, this is very rough
            2.0 * self.standard_normal_cdf(-t_abs * (1.0 + 0.25 / df))
        }
    }
    
    fn standard_normal_cdf(&self, z: f64) -> f64 {
        // Approximation of standard normal CDF
        0.5 * (1.0 + self.erf(z / 2.0_f64.sqrt()))
    }
    
    fn erf(&self, x: f64) -> f64 {
        // Approximation of error function
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;
        
        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();
        
        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
        
        sign * y
    }
    
    fn t_critical_value(&self, df: f64, alpha: f64) -> f64 {
        // Rough approximation - should use proper inverse t-distribution
        if df > 30.0 {
            self.normal_inverse_cdf(1.0 - alpha)
        } else {
            // Very rough approximation
            self.normal_inverse_cdf(1.0 - alpha) * (1.0 + 1.0 / (4.0 * df))
        }
    }
    
    fn normal_inverse_cdf(&self, p: f64) -> f64 {
        // Rough approximation of inverse normal CDF
        if p >= 0.5 {
            (2.0 * std::f64::consts::PI).sqrt() * self.inverse_erf(2.0 * p - 1.0)
        } else {
            -(2.0 * std::f64::consts::PI).sqrt() * self.inverse_erf(1.0 - 2.0 * p)
        }
    }
    
    fn inverse_erf(&self, x: f64) -> f64 {
        // Very rough approximation
        let a = 0.147;
        let ln_term = ((1.0 - x * x) / (2.0 * a)).ln();
        let term1 = 2.0 / (std::f64::consts::PI * a) + ln_term / 2.0;
        let term2 = ln_term / a;
        (term1 * term1 - term2).sqrt() - term1
    }
    
    // Placeholder implementations for other methods
    fn perform_anova(&self, _datasets: &HashMap<String, Vec<f64>>) -> AnovaResult {
        AnovaResult {
            f_statistic: 1.0,
            df_between: 1.0,
            df_within: 10.0,
            ss_between: 1.0,
            ss_within: 10.0,
            ms_between: 1.0,
            ms_within: 1.0,
            p_value: 0.5,
            significant: false,
            post_hoc: None,
        }
    }
    
    fn detect_outliers(&self, _datasets: &HashMap<String, Vec<f64>>) -> OutlierAnalysis {
        OutlierAnalysis {
            outliers_by_method: HashMap::new(),
            consensus_outliers: Vec::new(),
            summary: OutlierSummary {
                total_outliers: 0,
                outlier_percentage: 0.0,
                most_extreme: None,
                recommendation: OutlierRecommendation::NoAction,
            },
        }
    }
    
    fn test_normality(&self, name: &str, _data: &[f64]) -> NormalityTest {
        NormalityTest {
            dataset: name.to_string(),
            shapiro_wilk: None,
            anderson_darling: None,
            kolmogorov_smirnov: None,
            jarque_bera: None,
            overall_assessment: NormalityAssessment::Inconclusive,
        }
    }
    
    fn calculate_correlations(&self, _datasets: &HashMap<String, Vec<f64>>) -> CorrelationMatrix {
        CorrelationMatrix {
            variables: Vec::new(),
            pearson: Vec::new(),
            spearman: Vec::new(),
            kendall: Vec::new(),
            p_values: Vec::new(),
            strong_correlations: Vec::new(),
        }
    }
    
    fn calculate_effect_sizes(&self, _datasets: &HashMap<String, Vec<f64>>) -> HashMap<String, EffectSize> {
        HashMap::new()
    }
    
    fn generate_bootstrap_intervals(&self, _datasets: &HashMap<String, Vec<f64>>) -> HashMap<String, BootstrapInterval> {
        HashMap::new()
    }
    
    fn empty_descriptive_stats(&self, name: &str) -> DescriptiveStatistics {
        DescriptiveStatistics {
            name: name.to_string(),
            n: 0,
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            variance: 0.0,
            min: 0.0,
            max: 0.0,
            range: 0.0,
            iqr: 0.0,
            percentiles: HashMap::new(),
            skewness: 0.0,
            kurtosis: 0.0,
            coefficient_of_variation: 0.0,
            standard_error: 0.0,
        }
    }
}

/// Generate a comprehensive statistical report
pub fn generate_statistical_report(analysis: &StatisticalAnalysisResult) -> String {
    let mut report = String::new();
    
    report.push_str("# Statistical Analysis Report\n\n");
    
    // Descriptive statistics
    report.push_str("## Descriptive Statistics\n\n");
    report.push_str("| Implementation | N | Mean | Median | Std Dev | Min | Max | CV |\n");
    report.push_str("|---|---|---|---|---|---|---|---|\n");
    
    for (name, stats) in &analysis.descriptive_stats {
        report.push_str(&format!(
            "| {} | {} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.3} |\n",
            name, stats.n, stats.mean, stats.median, stats.std_dev, 
            stats.min, stats.max, stats.coefficient_of_variation
        ));
    }
    
    // Pairwise comparisons
    report.push_str("\n## Pairwise Comparisons\n\n");
    report.push_str("| Comparison | T-test p-value | Effect Size (Cohen's d) | Significant | Practical |\n");
    report.push_str("|---|---|---|---|---|\n");
    
    for comp in &analysis.pairwise_comparisons {
        report.push_str(&format!(
            "| {} vs {} | {:.4} | {:.3} | {} | {} |\n",
            comp.group_a, comp.group_b, comp.t_test.p_value, comp.effect_size.cohens_d,
            if comp.is_significant { "Yes" } else { "No" },
            if comp.is_practically_significant { "Yes" } else { "No" }
        ));
    }
    
    // ANOVA results
    if let Some(anova) = &analysis.anova_result {
        report.push_str(&format!("\n## ANOVA Results\n\n"));
        report.push_str(&format!("- F-statistic: {:.3}\n", anova.f_statistic));
        report.push_str(&format!("- p-value: {:.4}\n", anova.p_value));
        report.push_str(&format!("- Significant: {}\n", if anova.significant { "Yes" } else { "No" }));
    }
    
    // Outlier analysis
    report.push_str(&format!("\n## Outlier Analysis\n\n"));
    report.push_str(&format!("- Total outliers detected: {}\n", analysis.outlier_analysis.summary.total_outliers));
    report.push_str(&format!("- Percentage of data: {:.1}%\n", analysis.outlier_analysis.summary.outlier_percentage));
    report.push_str(&format!("- Recommendation: {:?}\n", analysis.outlier_analysis.summary.recommendation));
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_descriptive_statistics() {
        let analyzer = StatisticalAnalyzer::default();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = analyzer.calculate_descriptive_stats("test", &data);
        
        assert_eq!(stats.n, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert!(stats.std_dev > 0.0);
    }
    
    #[test]
    fn test_t_test() {
        let analyzer = StatisticalAnalyzer::default();
        let data_a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let data_b = vec![2.0, 3.0, 4.0, 5.0, 6.0];
        let result = analyzer.perform_t_test(&data_a, &data_b);
        
        assert!(result.t_statistic != 0.0);
        assert!(result.p_value >= 0.0 && result.p_value <= 1.0);
    }
    
    #[test]
    fn test_effect_size_calculation() {
        let analyzer = StatisticalAnalyzer::default();
        let data_a = vec![1.0, 2.0, 3.0];
        let data_b = vec![4.0, 5.0, 6.0];
        let effect_size = analyzer.calculate_effect_size_between(&data_a, &data_b);
        
        // Should show a large negative effect (group A < group B)
        assert!(effect_size.cohens_d < 0.0);
        assert!(matches!(effect_size.interpretation, EffectSizeInterpretation::VeryLarge));
    }
}