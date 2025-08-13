//! Performance trend analysis and forecasting.
//!
//! This module provides trend analysis capabilities including linear regression,
//! trend detection, and performance forecasting.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use super::regression_detector::RegressionDetectionConfig;
use super::baseline_manager::BaselineData;
use super::detection_results::StatisticalSignificance;

/// Analyzes performance trends over time
pub struct TrendAnalyzer {
    /// Configuration for trend analysis
    config: RegressionDetectionConfig,
}

/// Trend analysis for a specific test/implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Test identifier
    pub test_id: String,
    /// Implementation identifier
    pub implementation: String,
    /// Overall trend direction
    pub trend_direction: TrendDirection,
    /// Trend strength (0-100)
    pub trend_strength: f64,
    /// Linear regression slope
    pub slope: f64,
    /// R-squared value
    pub r_squared: f64,
    /// Trend significance
    pub significance: StatisticalSignificance,
    /// Forecast for next period
    pub forecast: PerformanceForecast,
}

/// Direction of performance trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Strongly improving performance trend
    StronglyImproving,
    /// Improving performance trend
    Improving,
    /// Stable performance (no significant trend)
    Stable,
    /// Declining performance trend
    Declining,
    /// Strongly declining performance trend
    StronglyDeclining,
    /// Volatile performance with no clear trend
    Volatile,
}

/// Performance forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceForecast {
    /// Predicted value for next measurement
    pub predicted_value: f64,
    /// Prediction interval
    pub prediction_interval: (f64, f64),
    /// Confidence in prediction
    pub confidence: f64,
    /// Time horizon for prediction
    pub time_horizon: Duration,
}

impl TrendAnalyzer {
    /// Creates a new trend analyzer with the specified configuration.
    pub fn new(config: RegressionDetectionConfig) -> Self {
        Self { config }
    }
    
    /// Analyzes performance trends across all baseline data sets.
    pub fn analyze_trends(&self, baselines: &HashMap<String, BaselineData>) -> HashMap<String, TrendAnalysis> {
        let mut trends = HashMap::new();
        
        for (key, baseline) in baselines {
            if baseline.measurements.len() < self.config.trend_window_size {
                continue;
            }
            
            let trend = self.analyze_single_trend(baseline);
            trends.insert(key.clone(), trend);
        }
        
        trends
    }
    
    fn analyze_single_trend(&self, baseline: &BaselineData) -> TrendAnalysis {
        let recent_measurements: Vec<_> = baseline.measurements
            .iter()
            .rev()
            .take(self.config.trend_window_size)
            .collect();
        
        // Simple linear regression
        let n = recent_measurements.len() as f64;
        let x_values: Vec<f64> = (0..recent_measurements.len()).map(|i| i as f64).collect();
        let y_values: Vec<f64> = recent_measurements.iter().map(|m| m.value).collect();
        
        let x_mean = x_values.iter().sum::<f64>() / n;
        let y_mean = y_values.iter().sum::<f64>() / n;
        
        let numerator: f64 = x_values.iter().zip(&y_values)
            .map(|(x, y)| (x - x_mean) * (y - y_mean))
            .sum();
        let denominator: f64 = x_values.iter()
            .map(|x| (x - x_mean).powi(2))
            .sum();
        
        let slope = if denominator != 0.0 { numerator / denominator } else { 0.0 };
        
        // Calculate R-squared
        let y_pred: Vec<f64> = x_values.iter().map(|x| y_mean + slope * (x - x_mean)).collect();
        let ss_tot: f64 = y_values.iter().map(|y| (y - y_mean).powi(2)).sum();
        let ss_res: f64 = y_values.iter().zip(&y_pred).map(|(y, y_p)| (y - y_p).powi(2)).sum();
        let r_squared = if ss_tot != 0.0 { 1.0 - (ss_res / ss_tot) } else { 0.0 };
        
        // Determine trend direction
        let trend_direction = if slope.abs() < 0.1 {
            TrendDirection::Stable
        } else if slope > 2.0 {
            TrendDirection::StronglyImproving
        } else if slope > 0.5 {
            TrendDirection::Improving
        } else if slope < -2.0 {
            TrendDirection::StronglyDeclining
        } else if slope < -0.5 {
            TrendDirection::Declining
        } else {
            TrendDirection::Volatile
        };
        
        let trend_strength = (slope.abs() * 10.0).min(100.0);
        
        // Generate forecast
        let predicted_value = y_mean + slope * n;
        let prediction_error = (ss_res / (n - 2.0)).sqrt();
        let prediction_interval = (
            predicted_value - 1.96 * prediction_error,
            predicted_value + 1.96 * prediction_error,
        );
        
        let forecast = PerformanceForecast {
            predicted_value,
            prediction_interval,
            confidence: (r_squared * 100.0).min(95.0),
            time_horizon: Duration::from_secs(86400), // 1 day
        };
        
        TrendAnalysis {
            test_id: baseline.test_id.clone(),
            implementation: baseline.implementation_id.clone(),
            trend_direction,
            trend_strength,
            slope,
            r_squared,
            significance: StatisticalSignificance {
                p_value: 0.05, // Placeholder
                is_significant: r_squared > 0.5,
                test_method: "Linear regression".to_string(),
                effect_size: slope,
            },
            forecast,
        }
    }
}