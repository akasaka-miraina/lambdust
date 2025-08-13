//! Baseline management for performance regression detection.
//!
//! This module manages historical performance data, baseline statistics,
//! and quality assessment for performance measurements.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use super::regression_detector::RegressionDetectionConfig;
use super::performance_measurement::PerformanceMeasurement;

/// Manages baseline performance measurements
pub struct BaselineManager {
    /// Baseline data indexed by test ID
    pub baselines: HashMap<String, BaselineData>,
    /// Configuration for regression detection
    config: RegressionDetectionConfig,
}

/// Baseline performance data and historical measurements for a test.
/// 
/// Maintains historical performance data, statistics, and quality metrics
/// for establishing performance expectations and detecting regressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineData {
    /// Test identifier
    pub test_id: String,
    /// Implementation identifier
    pub implementation_id: String,
    /// Historical measurements
    pub measurements: Vec<PerformanceMeasurement>,
    /// Baseline statistics
    pub statistics: BaselineStatistics,
    /// When baseline was last updated
    pub last_updated: SystemTime,
    /// Quality indicators
    pub quality: BaselineQuality,
}

/// Comprehensive statistical analysis of baseline performance data.
/// 
/// Provides statistical measures and confidence intervals
/// for establishing performance expectations and variance bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineStatistics {
    /// Number of measurements
    pub count: usize,
    /// Mean performance value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Median value
    pub median: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
    /// Coefficient of variation
    pub coefficient_of_variation: f64,
    /// Confidence interval for mean
    pub confidence_interval: (f64, f64),
}

/// Quality assessment metrics for baseline performance data.
/// 
/// Evaluates the reliability and trustworthiness of baseline data
/// across multiple quality dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineQuality {
    /// Overall quality score (0-100)
    pub score: f64,
    /// Stability rating (low variance = high stability)
    pub stability: QualityRating,
    /// Freshness rating (recent data = high freshness)
    pub freshness: QualityRating,
    /// Coverage rating (many samples = high coverage)
    pub coverage: QualityRating,
    /// Issues affecting quality
    pub issues: Vec<String>,
}

/// Quality rating scale for various baseline assessment dimensions.
/// 
/// Provides standardized quality levels for evaluating
/// different aspects of performance data quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityRating {
    /// Excellent quality baseline data
    Excellent,
    /// Good quality baseline data
    Good,
    /// Fair quality baseline data
    Fair,
    /// Poor quality baseline data
    Poor,
    /// Insufficient data for quality assessment
    Insufficient,
}

impl BaselineManager {
    /// Creates a new baseline manager with the specified configuration.
    pub fn new(config: RegressionDetectionConfig) -> Self {
        Self {
            baselines: HashMap::new(),
            config,
        }
    }
    
    /// Adds a new performance measurement to the appropriate baseline data.
    pub fn add_measurement(&mut self, measurement: PerformanceMeasurement) {
        let key = format!("{}_{}", measurement.parameters.get("test_id").unwrap_or(&"unknown".to_string()), 
                         measurement.parameters.get("implementation").unwrap_or(&"unknown".to_string()));
        
        let baseline = self.baselines.entry(key.clone()).or_insert_with(|| BaselineData {
            test_id: measurement.parameters.get("test_id").unwrap_or(&"unknown".to_string()).clone(),
            implementation_id: measurement.parameters.get("implementation").unwrap_or(&"unknown".to_string()).clone(),
            measurements: Vec::new(),
            statistics: BaselineStatistics {
                count: 0,
                mean: 0.0,
                std_dev: 0.0,
                median: 0.0,
                p95: 0.0,
                p99: 0.0,
                coefficient_of_variation: 0.0,
                confidence_interval: (0.0, 0.0),
            },
            last_updated: SystemTime::now(),
            quality: BaselineQuality {
                score: 0.0,
                stability: QualityRating::Insufficient,
                freshness: QualityRating::Excellent,
                coverage: QualityRating::Insufficient,
                issues: Vec::new(),
            },
        });
        
        baseline.measurements.push(measurement);
        baseline.last_updated = SystemTime::now();
        
        // Update statistics
        self.update_baseline_statistics(&key);
        self.update_baseline_quality(&key);
    }
    
    fn update_baseline_statistics(&mut self, key: &str) {
        // First, extract values using immutable borrow
        let values = {
            if let Some(baseline) = self.baselines.get(key) {
                baseline.measurements.iter().map(|m| m.value).collect::<Vec<f64>>()
            } else {
                return;
            }
        };
        
        // Calculate statistics
        let statistics = self.calculate_statistics(&values);
        
        // Update baseline using mutable borrow
        if let Some(baseline) = self.baselines.get_mut(key) {
            baseline.statistics = statistics;
        }
    }
    
    fn calculate_statistics(&self, values: &[f64]) -> BaselineStatistics {
        if values.is_empty() {
            return BaselineStatistics {
                count: 0,
                mean: 0.0,
                std_dev: 0.0,
                median: 0.0,
                p95: 0.0,
                p99: 0.0,
                coefficient_of_variation: 0.0,
                confidence_interval: (0.0, 0.0),
            };
        }
        
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let count = values.len();
        let mean = values.iter().sum::<f64>() / count as f64;
        let variance = if count > 1 {
            values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (count - 1) as f64
        } else {
            0.0
        };
        let std_dev = variance.sqrt();
        
        let median = if count % 2 == 0 {
            (sorted_values[count / 2 - 1] + sorted_values[count / 2]) / 2.0
        } else {
            sorted_values[count / 2]
        };
        
        let p95_index = ((count as f64 * 0.95) as usize).min(count - 1);
        let p99_index = ((count as f64 * 0.99) as usize).min(count - 1);
        let p95 = sorted_values[p95_index];
        let p99 = sorted_values[p99_index];
        
        let coefficient_of_variation = if mean != 0.0 { std_dev / mean.abs() } else { 0.0 };
        
        let margin = if count > 1 { 1.96 * std_dev / (count as f64).sqrt() } else { 0.0 };
        let confidence_interval = (mean - margin, mean + margin);
        
        BaselineStatistics {
            count,
            mean,
            std_dev,
            median,
            p95,
            p99,
            coefficient_of_variation,
            confidence_interval,
        }
    }
    
    fn update_baseline_quality(&mut self, key: &str) {
        // First, calculate scores using immutable borrows
        let (count, stability_score, freshness_score, coverage_score) = {
            if let Some(baseline) = self.baselines.get(key) {
                let count = baseline.measurements.len();
                let stability_score = self.calculate_stability_score(&baseline.statistics);
                let freshness_score = self.calculate_freshness_score(baseline.last_updated);
                let coverage_score = self.calculate_coverage_score(count);
                (count, stability_score, freshness_score, coverage_score)
            } else {
                return;
            }
        };
        
        // Calculate ratings using immutable borrow
        let stability_rating = self.score_to_rating(stability_score);
        let freshness_rating = self.score_to_rating(freshness_score);
        let coverage_rating = self.score_to_rating(coverage_score);
        
        // Then, update the baseline using mutable borrow
        if let Some(baseline) = self.baselines.get_mut(key) {
            baseline.quality.score = (stability_score + freshness_score + coverage_score) / 3.0;
            baseline.quality.stability = stability_rating;
            baseline.quality.freshness = freshness_rating;
            baseline.quality.coverage = coverage_rating;
        }
    }
    
    fn calculate_stability_score(&self, stats: &BaselineStatistics) -> f64 {
        if stats.coefficient_of_variation < 0.1 {
            100.0
        } else if stats.coefficient_of_variation < 0.2 {
            80.0
        } else if stats.coefficient_of_variation < 0.3 {
            60.0
        } else if stats.coefficient_of_variation < 0.5 {
            40.0
        } else {
            20.0
        }
    }
    
    fn calculate_freshness_score(&self, last_updated: SystemTime) -> f64 {
        let age = SystemTime::now().duration_since(last_updated).unwrap_or(Duration::ZERO);
        let age_days = age.as_secs() / 86400;
        
        if age_days <= 1 {
            100.0
        } else if age_days <= 7 {
            80.0
        } else if age_days <= 30 {
            60.0
        } else if age_days <= 90 {
            40.0
        } else {
            20.0
        }
    }
    
    fn calculate_coverage_score(&self, count: usize) -> f64 {
        if count >= 50 {
            100.0
        } else if count >= 20 {
            80.0
        } else if count >= 10 {
            60.0
        } else if count >= 5 {
            40.0
        } else {
            20.0
        }
    }
    
    fn score_to_rating(&self, score: f64) -> QualityRating {
        if score >= 90.0 {
            QualityRating::Excellent
        } else if score >= 70.0 {
            QualityRating::Good
        } else if score >= 50.0 {
            QualityRating::Fair
        } else if score >= 30.0 {
            QualityRating::Poor
        } else {
            QualityRating::Insufficient
        }
    }
}