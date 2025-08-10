//! Performance anomaly detection algorithms.
//!
//! This module implements various anomaly detection algorithms
//! to identify unusual performance patterns and outliers.

use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use super::regression_detector::RegressionDetectionConfig;
use super::baseline_manager::BaselineData;
use super::performance_measurement::PerformanceMeasurement;

/// Detects performance anomalies using various algorithms
pub struct AnomalyDetector {
    /// Configuration for anomaly detection
    config: RegressionDetectionConfig,
}

/// Performance anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnomaly {
    /// Test identifier
    pub test_id: String,
    /// Implementation identifier
    pub implementation: String,
    /// Anomaly type
    pub anomaly_type: AnomalyType,
    /// Anomaly score (higher = more anomalous)
    pub anomaly_score: f64,
    /// Measurement that triggered the anomaly
    pub anomalous_measurement: PerformanceMeasurement,
    /// Detection method used
    pub detection_method: AnomalyDetectionMethod,
    /// Timestamp when detected
    pub detected_at: SystemTime,
}

/// Types of performance anomalies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Unusually high performance outlier
    OutlierHigh,     // Unusually high performance
    /// Unusually low performance outlier
    OutlierLow,      // Unusually low performance
    /// Sudden sustained improvement
    ShiftUp,         // Sudden sustained improvement
    /// Sudden sustained degradation
    ShiftDown,       // Sudden sustained degradation
    /// Unusual performance variance
    Volatility,      // Unusual variance
    /// Systematic performance bias
    Systematic,      // Systematic bias
}

/// Anomaly detection methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyDetectionMethod {
    /// Statistical outlier detection method
    StatisticalOutlier,
    /// Isolation Forest anomaly detection
    IsolationForest,
    /// DBSCAN clustering-based detection
    DBSCAN,
    /// Local Outlier Factor method
    LocalOutlierFactor,
    /// One-class SVM method
    OneSVM,
    /// Change point detection algorithm
    ChangePointDetection,
}

impl AnomalyDetector {
    /// Creates a new anomaly detector with the specified configuration.
    pub fn new(config: RegressionDetectionConfig) -> Self {
        Self { config }
    }
    
    /// Detects performance anomalies across all baseline data sets.
    pub fn detect_anomalies(&self, baselines: &HashMap<String, BaselineData>) -> Vec<PerformanceAnomaly> {
        let mut anomalies = Vec::new();
        
        for baseline in baselines.values() {
            anomalies.extend(self.detect_statistical_outliers(baseline));
        }
        
        anomalies
    }
    
    fn detect_statistical_outliers(&self, baseline: &BaselineData) -> Vec<PerformanceAnomaly> {
        let mut anomalies = Vec::new();
        
        if baseline.measurements.len() < 10 {
            return anomalies;
        }
        
        let mean = baseline.statistics.mean;
        let std_dev = baseline.statistics.std_dev;
        let threshold = self.config.anomaly_sensitivity;
        
        for measurement in &baseline.measurements {
            let z_score = (measurement.value - mean).abs() / std_dev;
            
            if z_score > threshold {
                let anomaly_type = if measurement.value > mean {
                    AnomalyType::OutlierHigh
                } else {
                    AnomalyType::OutlierLow
                };
                
                anomalies.push(PerformanceAnomaly {
                    test_id: baseline.test_id.clone(),
                    implementation: baseline.implementation_id.clone(),
                    anomaly_type,
                    anomaly_score: z_score,
                    anomalous_measurement: measurement.clone(),
                    detection_method: AnomalyDetectionMethod::StatisticalOutlier,
                    detected_at: SystemTime::now(),
                });
            }
        }
        
        anomalies
    }
}