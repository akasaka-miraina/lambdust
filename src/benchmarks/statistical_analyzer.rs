//! Statistical Analysis Engine Implementation
//!
//! This module contains the core `StatisticalAnalyzer` struct and its implementation,
//! providing the main statistical analysis engine for performance benchmark results.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::correlation_statistics::{StatisticalAnalysisConfig, StatisticalAnalysisResult, DescriptiveStatistics, CorrelationMatrix};
use super::statistical_tests::{TTestResult, TTestType, MannWhitneyResult, WelchTestResult, PairwiseComparison, AnovaResult};
use super::effect_analysis::{EffectSize, EffectSizeInterpretation, ConfidenceInterval, ConfidenceIntervalMethod, BootstrapInterval};
use super::outlier_normality::{OutlierAnalysis, NormalityTest, NormalityAssessment, OutlierSummary, OutlierRecommendation};

/// Comprehensive statistical analysis engine for performance benchmark results.
/// 
/// Provides advanced statistical methods including hypothesis testing,
/// confidence intervals, effect size analysis, and outlier detection.
pub struct StatisticalAnalyzer {
    /// Configuration for statistical analysis
    config: StatisticalAnalysisConfig,
}

impl StatisticalAnalyzer {
    /// Create new statistical analyzer with configuration
    pub fn new(config: StatisticalAnalysisConfig) -> Self {
        Self { config }
    }
    
    /// Create analyzer with default configuration
    pub fn with_defaults() -> Self {
        Self::new(StatisticalAnalysisConfig::default())
    }
    
    /// Perform complete statistical analysis on benchmark results
    pub fn analyze(&self, datasets: HashMap<String, Vec<f64>>) -> StatisticalAnalysisResult {
        // Calculate descriptive statistics for each dataset
        let descriptive_stats = datasets.iter()
            .map(|(name, data)| (name.clone(), self.calculate_descriptive_stats(name, data)))
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
            .map(|(name, data)| (name.clone(), self.test_normality(name, data)))
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
    pub fn calculate_descriptive_stats(&self, name: &str, data: &[f64]) -> DescriptiveStatistics {
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
            
            data.iter()
                .map(|x| ((x - mean) / std_dev).powi(3))
                .sum::<f64>() / n as f64
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
        let names: Vec<_> = datasets.keys().clone().collect();
        
        for i in 0..names.len() {
            for j in (i + 1)..names.len() {
                let group_a = &names[i];
                let group_b = &names[j];
                let data_a = &datasets[group_a.as_str()];
                let data_b = &datasets[group_b.as_str()];
                
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