//! Performance regression detection system
//!
//! This module provides automated detection of performance regressions in the
//! `RuntimeExecutor` optimization system, comparing current performance against
//! historical baselines and identifying significant degradations.

use crate::error::Result;
use crate::evaluator::performance_measurement::{
    ComparisonResult, PerformanceCategory
};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Performance regression detection system
#[derive(Debug)]
pub struct RegressionDetector {
    /// Historical performance baselines
    baselines: HashMap<String, PerformanceBaseline>,
    /// Recent measurement history (sliding window)
    measurement_history: VecDeque<PerformanceMeasurement>,
    /// Detection configuration
    config: DetectionConfig,
    /// Regression alerts
    alerts: Vec<RegressionAlert>,
    /// Detection statistics
    detection_stats: DetectionStatistics,
}

/// Performance baseline for a specific benchmark or expression type
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    /// Baseline name/identifier
    pub name: String,
    /// Expected performance metrics
    pub expected_metrics: BaselineMetrics,
    /// Confidence interval
    pub confidence_interval: ConfidenceInterval,
    /// Baseline establishment date
    pub established_at: Instant,
    /// Number of measurements used to establish baseline
    pub sample_size: usize,
    /// Baseline stability (how consistent the measurements are)
    pub stability_score: f64,
}

/// Baseline performance metrics
#[derive(Debug, Clone)]
pub struct BaselineMetrics {
    /// Average execution time
    pub avg_execution_time: Duration,
    /// Standard deviation of execution time
    pub execution_time_std_dev: Duration,
    /// Average speedup factor vs semantic evaluator
    pub avg_speedup_factor: f64,
    /// Standard deviation of speedup factor
    pub speedup_std_dev: f64,
    /// Average memory usage
    pub avg_memory_usage: usize,
    /// Memory usage standard deviation
    pub memory_std_dev: f64,
}

/// Statistical confidence interval
#[derive(Debug, Clone)]
pub struct ConfidenceInterval {
    /// Lower bound of performance (worst acceptable)
    pub lower_bound: f64,
    /// Upper bound of performance (best expected)
    pub upper_bound: f64,
    /// Confidence level (e.g., 0.95 for 95%)
    pub confidence_level: f64,
}

/// Individual performance measurement
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    /// Measurement identifier
    pub measurement_id: String,
    /// Benchmark or expression type
    pub benchmark_type: String,
    /// Measured execution time
    pub execution_time: Duration,
    /// Speedup factor vs semantic evaluator
    pub speedup_factor: f64,
    /// Memory usage
    pub memory_usage: usize,
    /// Measurement timestamp
    pub timestamp: Instant,
    /// Performance category
    pub performance_category: PerformanceCategory,
}

/// Regression detection configuration
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// Minimum sample size for baseline establishment
    pub min_baseline_samples: usize,
    /// Maximum history window size
    pub max_history_size: usize,
    /// Regression threshold (performance degradation percentage)
    pub regression_threshold: f64,
    /// Statistical significance threshold
    pub significance_threshold: f64,
    /// Minimum measurements before detection
    pub min_measurements_for_detection: usize,
    /// Alert cooldown period
    pub alert_cooldown: Duration,
}

/// Performance regression alert
#[derive(Debug, Clone)]
pub struct RegressionAlert {
    /// Alert unique identifier
    pub alert_id: String,
    /// Alert severity level
    pub severity: AlertSeverity,
    /// Benchmark that triggered the alert
    pub benchmark_name: String,
    /// Detected performance degradation
    pub performance_degradation: PerformanceDegradation,
    /// Alert timestamp
    pub timestamp: Instant,
    /// Recommended actions
    pub recommendations: Vec<String>,
    /// Whether the alert has been acknowledged
    pub acknowledged: bool,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    /// Minor performance degradation
    Minor,
    /// Moderate performance degradation
    Moderate,
    /// Severe performance degradation
    Severe,
    /// Critical performance degradation
    Critical,
}

/// Performance degradation details
#[derive(Debug, Clone)]
pub struct PerformanceDegradation {
    /// Current performance metric
    pub current_performance: f64,
    /// Baseline performance metric
    pub baseline_performance: f64,
    /// Degradation percentage
    pub degradation_percentage: f64,
    /// Statistical significance
    pub statistical_significance: f64,
    /// Affected metrics
    pub affected_metrics: Vec<String>,
}

/// Detection system statistics
#[derive(Debug, Clone)]
pub struct DetectionStatistics {
    /// Total measurements processed
    pub total_measurements: usize,
    /// Number of baselines established
    pub baselines_established: usize,
    /// Total alerts generated
    pub total_alerts: usize,
    /// Alerts by severity
    pub alerts_by_severity: HashMap<AlertSeverity, usize>,
    /// False positive rate
    pub false_positive_rate: f64,
    /// Detection accuracy
    pub detection_accuracy: f64,
}

impl RegressionDetector {
    /// Create a new regression detector
    #[must_use] pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
            measurement_history: VecDeque::new(),
            config: DetectionConfig::default(),
            alerts: Vec::new(),
            detection_stats: DetectionStatistics::default(),
        }
    }

    /// Create with custom configuration
    #[must_use] pub fn with_config(config: DetectionConfig) -> Self {
        Self {
            baselines: HashMap::new(),
            measurement_history: VecDeque::new(),
            config,
            alerts: Vec::new(),
            detection_stats: DetectionStatistics::default(),
        }
    }

    /// Process a new performance measurement
    pub fn process_measurement(&mut self, measurement: PerformanceMeasurement) -> Result<()> {
        // Add to history
        self.measurement_history.push_back(measurement.clone());
        
        // Maintain history window size
        while self.measurement_history.len() > self.config.max_history_size {
            self.measurement_history.pop_front();
        }

        self.detection_stats.total_measurements += 1;

        // Check if we have a baseline for this benchmark type
        if let Some(baseline) = self.baselines.get(&measurement.benchmark_type) {
            // Check for regression
            if let Some(alert) = self.check_for_regression(&measurement, baseline)? {
                self.alerts.push(alert);
                self.detection_stats.total_alerts += 1;
            }
        } else {
            // Try to establish a baseline
            self.try_establish_baseline(&measurement.benchmark_type)?;
        }

        Ok(())
    }

    /// Process multiple comparison results
    pub fn process_comparison_results(&mut self, results: &[ComparisonResult]) -> Result<()> {
        for result in results {
            let measurement = PerformanceMeasurement {
                measurement_id: format!("comp_{}", self.detection_stats.total_measurements),
                benchmark_type: result.expression_summary.clone(),
                execution_time: result.runtime_metrics.execution_time,
                speedup_factor: result.performance_comparison.speedup_factor,
                memory_usage: result.runtime_metrics.memory_usage_bytes,
                timestamp: result.timestamp,
                performance_category: result.performance_comparison.performance_category.clone(),
            };
            
            self.process_measurement(measurement)?;
        }
        Ok(())
    }

    /// Try to establish a baseline for a benchmark type
    fn try_establish_baseline(&mut self, benchmark_type: &str) -> Result<()> {
        let measurements: Vec<_> = self.measurement_history
            .iter()
            .filter(|m| m.benchmark_type == benchmark_type)
            .collect();

        if measurements.len() >= self.config.min_baseline_samples {
            let baseline = self.calculate_baseline(benchmark_type, &measurements)?;
            self.baselines.insert(benchmark_type.to_string(), baseline);
            self.detection_stats.baselines_established += 1;
        }

        Ok(())
    }

    /// Calculate baseline from measurements
    fn calculate_baseline(
        &self,
        benchmark_type: &str,
        measurements: &[&PerformanceMeasurement],
    ) -> Result<PerformanceBaseline> {
        let execution_times: Vec<f64> = measurements
            .iter()
            .map(|m| m.execution_time.as_nanos() as f64)
            .collect();

        let speedup_factors: Vec<f64> = measurements
            .iter()
            .map(|m| m.speedup_factor)
            .filter(|&s| s.is_finite())
            .collect();

        let memory_usages: Vec<f64> = measurements
            .iter()
            .map(|m| m.memory_usage as f64)
            .collect();

        let avg_execution_time = Duration::from_nanos(
            (execution_times.iter().sum::<f64>() / execution_times.len() as f64) as u64
        );

        let execution_time_variance = execution_times
            .iter()
            .map(|&t| {
                let diff = t - avg_execution_time.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>() / execution_times.len() as f64;
        
        let execution_time_std_dev = Duration::from_nanos(execution_time_variance.sqrt() as u64);

        let avg_speedup_factor = speedup_factors.iter().sum::<f64>() / speedup_factors.len() as f64;
        let speedup_variance = speedup_factors
            .iter()
            .map(|&s| (s - avg_speedup_factor).powi(2))
            .sum::<f64>() / speedup_factors.len() as f64;
        let speedup_std_dev = speedup_variance.sqrt();

        let avg_memory_usage = memory_usages.iter().sum::<f64>() / memory_usages.len() as f64;
        let memory_variance = memory_usages
            .iter()
            .map(|&m| (m - avg_memory_usage).powi(2))
            .sum::<f64>() / memory_usages.len() as f64;
        let memory_std_dev = memory_variance.sqrt();

        // Calculate confidence interval (95% confidence)
        let confidence_margin = 1.96 * speedup_std_dev / (speedup_factors.len() as f64).sqrt();
        let confidence_interval = ConfidenceInterval {
            lower_bound: (avg_speedup_factor - confidence_margin).max(0.0),
            upper_bound: avg_speedup_factor + confidence_margin,
            confidence_level: 0.95,
        };

        // Calculate stability score
        let stability_score = 1.0 - (speedup_std_dev / avg_speedup_factor).min(1.0);

        Ok(PerformanceBaseline {
            name: benchmark_type.to_string(),
            expected_metrics: BaselineMetrics {
                avg_execution_time,
                execution_time_std_dev,
                avg_speedup_factor,
                speedup_std_dev,
                avg_memory_usage: avg_memory_usage as usize,
                memory_std_dev,
            },
            confidence_interval,
            established_at: Instant::now(),
            sample_size: measurements.len(),
            stability_score,
        })
    }

    /// Check for regression against baseline
    fn check_for_regression(
        &self,
        measurement: &PerformanceMeasurement,
        baseline: &PerformanceBaseline,
    ) -> Result<Option<RegressionAlert>> {
        // Check if performance is significantly below baseline
        let performance_drop = (baseline.expected_metrics.avg_speedup_factor - measurement.speedup_factor) 
            / baseline.expected_metrics.avg_speedup_factor;

        if performance_drop > self.config.regression_threshold {
            // Calculate statistical significance
            let z_score = (baseline.expected_metrics.avg_speedup_factor - measurement.speedup_factor) 
                / baseline.expected_metrics.speedup_std_dev;
            let statistical_significance = 1.0 - (z_score / 3.0).min(1.0).max(0.0); // Simplified

            if statistical_significance > self.config.significance_threshold {
                let severity = self.determine_severity(performance_drop);
                
                let degradation = PerformanceDegradation {
                    current_performance: measurement.speedup_factor,
                    baseline_performance: baseline.expected_metrics.avg_speedup_factor,
                    degradation_percentage: performance_drop * 100.0,
                    statistical_significance,
                    affected_metrics: vec!["speedup_factor".to_string()],
                };

                let alert = RegressionAlert {
                    alert_id: format!("alert_{}_{}", measurement.benchmark_type, measurement.timestamp.elapsed().as_millis()),
                    severity,
                    benchmark_name: measurement.benchmark_type.clone(),
                    performance_degradation: degradation,
                    timestamp: Instant::now(),
                    recommendations: self.generate_recommendations(&measurement.performance_category),
                    acknowledged: false,
                };

                return Ok(Some(alert));
            }
        }

        Ok(None)
    }

    /// Determine alert severity based on performance drop
    fn determine_severity(&self, performance_drop: f64) -> AlertSeverity {
        if performance_drop > 0.5 {
            AlertSeverity::Critical
        } else if performance_drop > 0.3 {
            AlertSeverity::Severe
        } else if performance_drop > 0.15 {
            AlertSeverity::Moderate
        } else {
            AlertSeverity::Minor
        }
    }

    /// Generate recommendations based on performance category
    fn generate_recommendations(&self, category: &PerformanceCategory) -> Vec<String> {
        match category {
            PerformanceCategory::SignificantRegression => vec![
                "Investigate recent code changes".to_string(),
                "Check for optimization regressions".to_string(),
                "Review algorithm implementations".to_string(),
            ],
            PerformanceCategory::ModerateRegression => vec![
                "Review optimization settings".to_string(),
                "Check for new bottlenecks".to_string(),
            ],
            PerformanceCategory::MinorRegression => vec![
                "Monitor for trend continuation".to_string(),
                "Consider minor optimizations".to_string(),
            ],
            _ => vec!["No immediate action required".to_string()],
        }
    }

    /// Get active alerts
    #[must_use] pub fn get_active_alerts(&self) -> Vec<&RegressionAlert> {
        self.alerts.iter().filter(|a| !a.acknowledged).collect()
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> Result<()> {
        if let Some(alert) = self.alerts.iter_mut().find(|a| a.alert_id == alert_id) {
            alert.acknowledged = true;
        }
        Ok(())
    }

    /// Get baseline for a benchmark type
    #[must_use] pub fn get_baseline(&self, benchmark_type: &str) -> Option<&PerformanceBaseline> {
        self.baselines.get(benchmark_type)
    }

    /// Get detection statistics
    #[must_use] pub fn get_statistics(&self) -> &DetectionStatistics {
        &self.detection_stats
    }

    /// Generate regression report
    #[must_use] pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# Performance Regression Detection Report\n\n");
        
        report.push_str("## Detection Statistics\n\n");
        report.push_str(&format!("- Total Measurements: {}\n", self.detection_stats.total_measurements));
        report.push_str(&format!("- Baselines Established: {}\n", self.detection_stats.baselines_established));
        report.push_str(&format!("- Total Alerts: {}\n\n", self.detection_stats.total_alerts));
        
        let active_alerts = self.get_active_alerts();
        report.push_str(&format!("## Active Alerts ({})\n\n", active_alerts.len()));
        
        for alert in active_alerts {
            report.push_str(&format!("### {} - {:?}\n\n", alert.benchmark_name, alert.severity));
            report.push_str(&format!("- Performance Drop: {:.1}%\n", alert.performance_degradation.degradation_percentage));
            report.push_str(&format!("- Statistical Significance: {:.3}\n", alert.performance_degradation.statistical_significance));
            report.push_str("- Recommendations:\n");
            for rec in &alert.recommendations {
                report.push_str(&format!("  - {rec}\n"));
            }
            report.push('\n');
        }
        
        report.push_str("## Established Baselines\n\n");
        for (name, baseline) in &self.baselines {
            report.push_str(&format!("### {name}\n\n"));
            report.push_str(&format!("- Average Speedup: {:.2}x\n", baseline.expected_metrics.avg_speedup_factor));
            report.push_str(&format!("- Stability Score: {:.3}\n", baseline.stability_score));
            report.push_str(&format!("- Sample Size: {}\n\n", baseline.sample_size));
        }
        
        report
    }
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            min_baseline_samples: 10,
            max_history_size: 1000,
            regression_threshold: 0.1, // 10% performance drop
            significance_threshold: 0.95,
            min_measurements_for_detection: 5,
            alert_cooldown: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl Default for DetectionStatistics {
    fn default() -> Self {
        Self {
            total_measurements: 0,
            baselines_established: 0,
            total_alerts: 0,
            alerts_by_severity: HashMap::new(),
            false_positive_rate: 0.0,
            detection_accuracy: 0.0,
        }
    }
}

impl Default for RegressionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regression_detector_creation() {
        let detector = RegressionDetector::new();
        assert!(detector.baselines.is_empty());
        assert!(detector.alerts.is_empty());
    }

    #[test]
    fn test_severity_determination() {
        let detector = RegressionDetector::new();
        
        assert_eq!(detector.determine_severity(0.6), AlertSeverity::Critical);
        assert_eq!(detector.determine_severity(0.4), AlertSeverity::Severe);
        assert_eq!(detector.determine_severity(0.2), AlertSeverity::Moderate);
        assert_eq!(detector.determine_severity(0.1), AlertSeverity::Minor);
    }

    #[test]
    fn test_measurement_processing() {
        let mut detector = RegressionDetector::new();
        
        let measurement = PerformanceMeasurement {
            measurement_id: "test_1".to_string(),
            benchmark_type: "arithmetic".to_string(),
            execution_time: Duration::from_millis(100),
            speedup_factor: 2.0,
            memory_usage: 1024,
            timestamp: Instant::now(),
            performance_category: PerformanceCategory::ModerateImprovement,
        };

        assert!(detector.process_measurement(measurement).is_ok());
        assert_eq!(detector.detection_stats.total_measurements, 1);
    }
}