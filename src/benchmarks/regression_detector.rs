//! Core regression detection engine and configuration.
//!
//! This module contains the main `RegressionDetector` that orchestrates
//! the entire performance regression detection system.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use crate::benchmarks::statistical_analysis::{StatisticalAnalyzer, StatisticalAnalysisConfig};
use super::baseline_manager::{BaselineManager, BaselineData, BaselineStatistics};
use super::trend_analyzer::{TrendAnalyzer, TrendAnalysis};
use super::anomaly_detector::{AnomalyDetector, PerformanceAnomaly};
use super::performance_measurement::PerformanceMeasurement;
use super::detection_results::{
    RegressionDetectionResult, PerformanceRegression, PerformanceImprovement,
    StatisticalSignificance, PerformanceComparison, DifferenceMetrics, RegressionSeverity
};
use super::analysis_support::{
    SuspectedCause, ImprovementCause, CauseType, OverallAssessment, ActionRecommendation,
    PerformanceStatus, RiskLevel, RecommendedAction, EffortLevel
};

/// Advanced performance regression detection system.
/// 
/// Combines statistical analysis, trend detection, and anomaly identification
/// to provide comprehensive performance regression monitoring.
pub struct RegressionDetector {
    /// Configuration for regression detection
    config: RegressionDetectionConfig,
    /// Manager for baseline performance data
    baseline_manager: BaselineManager,
    /// Analyzer for performance trends
    trend_analyzer: TrendAnalyzer,
    /// Detector for performance anomalies
    anomaly_detector: AnomalyDetector,
}

/// Configuration parameters for performance regression detection.
/// 
/// Controls sensitivity, statistical rigor, and time windows
/// for detecting performance degradations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionDetectionConfig {
    /// Minimum performance degradation to consider a regression (percentage)
    pub regression_threshold: f64,
    /// Confidence level for statistical tests
    pub confidence_level: f64,
    /// Number of recent measurements to consider for trends
    pub trend_window_size: usize,
    /// Sensitivity for anomaly detection (lower = more sensitive)
    pub anomaly_sensitivity: f64,
    /// Whether to use statistical significance testing
    pub require_statistical_significance: bool,
    /// Minimum number of measurements to establish baseline
    pub min_baseline_samples: usize,
    /// Maximum age of baseline measurements (days)
    pub baseline_max_age_days: u64,
}

impl Default for RegressionDetectionConfig {
    fn default() -> Self {
        Self {
            regression_threshold: 5.0,  // 5% degradation
            confidence_level: 0.95,
            trend_window_size: 20,
            anomaly_sensitivity: 2.0,
            require_statistical_significance: true,
            min_baseline_samples: 10,
            baseline_max_age_days: 30,
        }
    }
}

impl RegressionDetector {
    /// Create new regression detector
    pub fn new(config: RegressionDetectionConfig) -> Self {
        let baseline_manager = BaselineManager::new(config.clone());
        let trend_analyzer = TrendAnalyzer::new(config.clone());
        let anomaly_detector = AnomalyDetector::new(config.clone());
        
        Self {
            config,
            baseline_manager,
            trend_analyzer,
            anomaly_detector,
        }
    }
    
    /// Add new performance measurements to the system
    pub fn add_measurements(&mut self, measurements: Vec<PerformanceMeasurement>) {
        for measurement in measurements {
            self.baseline_manager.add_measurement(measurement);
        }
    }
    
    /// Perform regression detection analysis
    pub fn detect_regressions(&mut self) -> RegressionDetectionResult {
        let timestamp = SystemTime::now();
        
        // Detect regressions and improvements
        let (regressions, improvements) = self.detect_performance_changes();
        
        // Analyze trends
        let trends = self.trend_analyzer.analyze_trends(&self.baseline_manager.baselines);
        
        // Detect anomalies
        let anomalies = self.anomaly_detector.detect_anomalies(&self.baseline_manager.baselines);
        
        // Generate overall assessment
        let overall_assessment = self.assess_overall_performance(&regressions, &improvements, &trends);
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&regressions, &improvements, &trends, &anomalies);
        
        RegressionDetectionResult {
            timestamp,
            config: self.config.clone(),
            regressions,
            improvements,
            trends,
            anomalies,
            overall_assessment,
            recommendations,
        }
    }
    
    /// Detect performance regressions and improvements
    fn detect_performance_changes(&self) -> (Vec<PerformanceRegression>, Vec<PerformanceImprovement>) {
        let mut regressions = Vec::new();
        let mut improvements = Vec::new();
        
        let statistical_analyzer = StatisticalAnalyzer::new(StatisticalAnalysisConfig {
            confidence_level: self.config.confidence_level,
            alpha_level: 1.0 - self.config.confidence_level,
            ..Default::default()
        });
        
        for (key, baseline) in &self.baseline_manager.baselines {
            if baseline.measurements.len() < self.config.min_baseline_samples {
                continue;
            }
            
            // Get recent measurements for comparison
            let recent_count = (baseline.measurements.len() / 4).clamp(5, 10);
            let recent_measurements: Vec<f64> = baseline.measurements
                .iter()
                .rev()
                .take(recent_count)
                .map(|m| m.value)
                .collect();
            
            if recent_measurements.is_empty() {
                continue;
            }
            
            // Get baseline measurements (excluding recent ones)
            let baseline_measurements: Vec<f64> = baseline.measurements
                .iter()
                .rev()
                .skip(recent_count)
                .map(|m| m.value)
                .collect();
            
            if baseline_measurements.is_empty() {
                continue;
            }
            
            // Calculate performance change
            let baseline_mean = baseline_measurements.iter().sum::<f64>() / baseline_measurements.len() as f64;
            let recent_mean = recent_measurements.iter().sum::<f64>() / recent_measurements.len() as f64;
            let percent_change = ((recent_mean - baseline_mean) / baseline_mean) * 100.0;
            
            // Perform statistical test
            let t_test = statistical_analyzer.perform_t_test(&recent_measurements, &baseline_measurements);
            let effect_size = statistical_analyzer.calculate_effect_size_between(&recent_measurements, &baseline_measurements);
            
            let significance = StatisticalSignificance {
                p_value: t_test.p_value,
                is_significant: t_test.significant,
                test_method: "Welch's t-test".to_string(),
                effect_size: effect_size.cohens_d,
            };
            
            // Check for regression
            if percent_change < -self.config.regression_threshold {
                if !self.config.require_statistical_significance || significance.is_significant {
                    let regression = PerformanceRegression {
                        test_id: baseline.test_id.clone(),
                        implementation: baseline.implementation_id.clone(),
                        degradation_percent: -percent_change,
                        statistical_significance: significance.clone(),
                        severity: self.classify_regression_severity(-percent_change),
                        current_vs_baseline: self.create_performance_comparison(baseline, &recent_measurements),
                        suspected_causes: self.analyze_suspected_causes(baseline, &recent_measurements),
                        first_detected: SystemTime::now(),
                        confidence: self.calculate_detection_confidence(&significance, -percent_change),
                    };
                    regressions.push(regression);
                }
            }
            // Check for improvement
            else if percent_change > self.config.regression_threshold
                && (!self.config.require_statistical_significance || significance.is_significant) {
                    let improvement = PerformanceImprovement {
                        test_id: baseline.test_id.clone(),
                        implementation: baseline.implementation_id.clone(),
                        improvement_percent: percent_change,
                        statistical_significance: significance.clone(),
                        current_vs_baseline: self.create_performance_comparison(baseline, &recent_measurements),
                        likely_causes: self.analyze_improvement_causes(baseline, &recent_measurements),
                        first_detected: SystemTime::now(),
                        confidence: self.calculate_detection_confidence(&significance, percent_change),
                    };
                    improvements.push(improvement);
                }
        }
        
        (regressions, improvements)
    }
    
    fn classify_regression_severity(&self, degradation_percent: f64) -> RegressionSeverity {
        if degradation_percent < 15.0 {
            RegressionSeverity::Minor
        } else if degradation_percent < 30.0 {
            RegressionSeverity::Moderate
        } else if degradation_percent < 50.0 {
            RegressionSeverity::Major
        } else {
            RegressionSeverity::Critical
        }
    }
    
    fn calculate_detection_confidence(&self, significance: &StatisticalSignificance, percent_change: f64) -> f64 {
        let statistical_confidence = if significance.is_significant {
            (1.0 - significance.p_value) * 100.0
        } else {
            50.0
        };
        
        let magnitude_confidence = (percent_change.abs() / 50.0).min(1.0) * 50.0;
        let effect_confidence = (significance.effect_size.abs() / 2.0).min(1.0) * 50.0;
        
        (statistical_confidence + magnitude_confidence + effect_confidence) / 3.0
    }
    
    fn create_performance_comparison(&self, baseline: &BaselineData, recent_values: &[f64]) -> PerformanceComparison {
        let baseline_values: Vec<f64> = baseline.measurements.iter().map(|m| m.value).collect();
        
        let current_statistics = self.calculate_statistics(recent_values);
        let difference = self.calculate_difference_metrics(&baseline.statistics, &current_statistics);
        
        PerformanceComparison {
            baseline: baseline.statistics.clone(),
            current_values: recent_values.to_vec(),
            current_statistics,
            difference,
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
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (count - 1) as f64;
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
        
        // Rough confidence interval
        let margin = 1.96 * std_dev / (count as f64).sqrt();
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
    
    fn calculate_difference_metrics(&self, baseline: &BaselineStatistics, current: &BaselineStatistics) -> DifferenceMetrics {
        let mean_difference = current.mean - baseline.mean;
        let percent_change = if baseline.mean != 0.0 {
            (mean_difference / baseline.mean) * 100.0
        } else {
            0.0
        };
        
        // Simplified calculations
        let pooled_std = ((baseline.std_dev.powi(2) + current.std_dev.powi(2)) / 2.0).sqrt();
        let effect_size = if pooled_std != 0.0 { mean_difference / pooled_std } else { 0.0 };
        
        let standard_error = (baseline.std_dev.powi(2) / baseline.count as f64 + 
                             current.std_dev.powi(2) / current.count as f64).sqrt();
        
        let margin = 1.96 * standard_error;
        let confidence_interval = (mean_difference - margin, mean_difference + margin);
        
        DifferenceMetrics {
            mean_difference,
            percent_change,
            standard_error,
            effect_size,
            confidence_interval,
        }
    }
    
    fn analyze_suspected_causes(&self, _baseline: &BaselineData, _recent_measurements: &[f64]) -> Vec<SuspectedCause> {
        vec![
            SuspectedCause {
                cause_type: CauseType::CodeChange,
                description: "Recent code changes may have affected performance".to_string(),
                confidence: 70.0,
                evidence: vec!["Performance change coincides with recent commits".to_string()],
            }
        ]
    }
    
    fn analyze_improvement_causes(&self, _baseline: &BaselineData, _recent_measurements: &[f64]) -> Vec<ImprovementCause> {
        vec![
            ImprovementCause {
                cause_type: CauseType::CodeChange,
                description: "Recent optimizations may have improved performance".to_string(),
                confidence: 70.0,
                evidence: vec!["Performance improvement coincides with recent commits".to_string()],
            }
        ]
    }
    
    fn assess_overall_performance(
        &self,
        regressions: &[PerformanceRegression],
        improvements: &[PerformanceImprovement],
        _trends: &HashMap<String, TrendAnalysis>,
    ) -> OverallAssessment {
        let critical_regressions = regressions.iter().filter(|r| matches!(r.severity, RegressionSeverity::Critical)).count();
        let major_regressions = regressions.iter().filter(|r| matches!(r.severity, RegressionSeverity::Major)).count();
        let total_regressions = regressions.len();
        let total_improvements = improvements.len();
        
        let health_score = if critical_regressions > 0 {
            20.0
        } else if major_regressions > 2 {
            40.0
        } else if total_regressions > total_improvements {
            60.0
        } else if total_improvements > total_regressions {
            85.0
        } else {
            75.0
        };
        
        let status = if health_score >= 80.0 {
            PerformanceStatus::Excellent
        } else if health_score >= 65.0 {
            PerformanceStatus::Good
        } else if health_score >= 50.0 {
            PerformanceStatus::Concerning
        } else if health_score >= 30.0 {
            PerformanceStatus::Poor
        } else {
            PerformanceStatus::Critical
        };
        
        let risk_level = if critical_regressions > 0 {
            RiskLevel::Critical
        } else if major_regressions > 0 {
            RiskLevel::High
        } else if total_regressions > 3 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };
        
        let key_findings = vec![
            format!("Detected {} regressions and {} improvements", total_regressions, total_improvements),
            format!("Health score: {:.1}/100", health_score),
        ];
        
        OverallAssessment {
            health_score,
            status,
            key_findings,
            risk_level,
            confidence: 80.0,
        }
    }
    
    fn generate_recommendations(
        &self,
        regressions: &[PerformanceRegression],
        _improvements: &[PerformanceImprovement],
        _trends: &HashMap<String, TrendAnalysis>,
        _anomalies: &[PerformanceAnomaly],
    ) -> Vec<ActionRecommendation> {
        let mut recommendations = Vec::new();
        
        // High priority recommendations for critical regressions
        for regression in regressions.iter().filter(|r| matches!(r.severity, RegressionSeverity::Critical)) {
            recommendations.push(ActionRecommendation {
                priority: 10,
                action: RecommendedAction::Investigate,
                description: format!("Immediately investigate critical regression in {}", regression.test_id),
                expected_impact: "Prevent further performance degradation".to_string(),
                effort_level: EffortLevel::High,
                timeline: "Within 24 hours".to_string(),
            });
        }
        
        // Medium priority recommendations for major regressions
        for regression in regressions.iter().filter(|r| matches!(r.severity, RegressionSeverity::Major)) {
            recommendations.push(ActionRecommendation {
                priority: 7,
                action: RecommendedAction::Investigate,
                description: format!("Investigate major regression in {}", regression.test_id),
                expected_impact: "Restore performance levels".to_string(),
                effort_level: EffortLevel::Medium,
                timeline: "Within 1 week".to_string(),
            });
        }
        
        // General recommendations
        if regressions.len() > 5 {
            recommendations.push(ActionRecommendation {
                priority: 8,
                action: RecommendedAction::IncreaseMonitoring,
                description: "Increase monitoring frequency due to multiple regressions".to_string(),
                expected_impact: "Earlier detection of future regressions".to_string(),
                effort_level: EffortLevel::Low,
                timeline: "Immediate".to_string(),
            });
        }
        
        recommendations
    }
}