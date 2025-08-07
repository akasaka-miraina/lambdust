//! Performance Regression Detection System
//!
//! This module implements sophisticated algorithms for detecting performance
//! regressions in benchmark results over time. It provides early warning
//! systems for performance degradation and trend analysis.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use crate::benchmarks::statistical_analysis::{StatisticalAnalyzer, StatisticalAnalysisConfig};

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

/// Manages baseline performance measurements
struct BaselineManager {
    /// Baseline data indexed by test ID
    baselines: HashMap<String, BaselineData>,
    /// Configuration for regression detection
    config: RegressionDetectionConfig,
}

/// Analyzes performance trends over time
struct TrendAnalyzer {
    /// Configuration for trend analysis
    config: RegressionDetectionConfig,
}

/// Detects performance anomalies using various algorithms
struct AnomalyDetector {
    /// Configuration for anomaly detection
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

/// Single performance measurement with context and parameters.
/// 
/// Captures a point-in-time performance measurement with
/// associated system context and test configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasurement {
    /// Timestamp of measurement
    pub timestamp: SystemTime,
    /// Performance value (e.g., operations per second)
    pub value: f64,
    /// Test parameters (if any)
    pub parameters: HashMap<String, String>,
    /// System context
    pub context: SystemContext,
    /// Confidence in this measurement
    pub confidence: f64,
}

/// System environment context during performance measurement.
/// 
/// Captures relevant system state that may influence performance
/// to provide context for measurement interpretation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    /// Git commit hash
    pub commit_hash: Option<String>,
    /// Compiler version
    pub compiler_version: String,
    /// System load during measurement
    pub system_load: f64,
    /// Available memory during measurement
    pub available_memory_mb: u64,
    /// Temperature (if available)
    pub temperature: Option<f64>,
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

/// Comprehensive results from performance regression analysis.
/// 
/// Contains all detected regressions, improvements, trends, and anomalies
/// with recommendations for addressing performance issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionDetectionResult {
    /// Timestamp of analysis
    pub timestamp: SystemTime,
    /// Configuration used
    pub config: RegressionDetectionConfig,
    /// Detected regressions
    pub regressions: Vec<PerformanceRegression>,
    /// Detected improvements
    pub improvements: Vec<PerformanceImprovement>,
    /// Trend analysis results
    pub trends: HashMap<String, TrendAnalysis>,
    /// Anomaly detection results
    pub anomalies: Vec<PerformanceAnomaly>,
    /// Overall assessment
    pub overall_assessment: OverallAssessment,
    /// Recommendations for action
    pub recommendations: Vec<ActionRecommendation>,
}

/// Detected performance regression with detailed analysis.
/// 
/// Represents a statistically significant performance degradation
/// with severity assessment and suspected root causes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    /// Test identifier
    pub test_id: String,
    /// Implementation affected
    pub implementation: String,
    /// Performance degradation percentage
    pub degradation_percent: f64,
    /// Statistical significance
    pub statistical_significance: StatisticalSignificance,
    /// Severity level
    pub severity: RegressionSeverity,
    /// Current vs baseline comparison
    pub current_vs_baseline: PerformanceComparison,
    /// Suspected causes
    pub suspected_causes: Vec<SuspectedCause>,
    /// First detected timestamp
    pub first_detected: SystemTime,
    /// Confidence in detection
    pub confidence: f64,
}

/// Detected performance improvement with analysis details.
/// 
/// Represents a statistically significant performance improvement
/// with likely contributing factors and impact assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovement {
    /// Test identifier
    pub test_id: String,
    /// Implementation improved
    pub implementation: String,
    /// Performance improvement percentage
    pub improvement_percent: f64,
    /// Statistical significance
    pub statistical_significance: StatisticalSignificance,
    /// Current vs baseline comparison
    pub current_vs_baseline: PerformanceComparison,
    /// Likely causes
    pub likely_causes: Vec<ImprovementCause>,
    /// First detected timestamp
    pub first_detected: SystemTime,
    /// Confidence in detection
    pub confidence: f64,
}

/// Statistical significance assessment for performance changes.
/// 
/// Provides statistical test results and effect size measurements
/// to validate the reliability of detected performance changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSignificance {
    /// P-value from statistical test
    pub p_value: f64,
    /// Whether statistically significant
    pub is_significant: bool,
    /// Type of test used
    pub test_method: String,
    /// Effect size
    pub effect_size: f64,
}

/// Severity of performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    /// Minor performance regression (5-15% degradation)
    Minor,      // 5-15% degradation
    /// Moderate performance regression (15-30% degradation)
    Moderate,   // 15-30% degradation
    /// Major performance regression (30-50% degradation)
    Major,      // 30-50% degradation
    /// Critical performance regression (>50% degradation)
    Critical,   // >50% degradation
}

/// Comparison between current and baseline performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    /// Baseline statistics
    pub baseline: BaselineStatistics,
    /// Current measurements
    pub current_values: Vec<f64>,
    /// Current statistics
    pub current_statistics: BaselineStatistics,
    /// Difference metrics
    pub difference: DifferenceMetrics,
}

/// Metrics describing the difference between baseline and current
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferenceMetrics {
    /// Mean difference (current - baseline)
    pub mean_difference: f64,
    /// Percentage change
    pub percent_change: f64,
    /// Standard error of difference
    pub standard_error: f64,
    /// Cohen's d effect size
    pub effect_size: f64,
    /// Confidence interval for difference
    pub confidence_interval: (f64, f64),
}

/// Suspected cause of performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspectedCause {
    /// Type of suspected cause
    pub cause_type: CauseType,
    /// Description
    pub description: String,
    /// Confidence in this cause (0-100)
    pub confidence: f64,
    /// Supporting evidence
    pub evidence: Vec<String>,
}

/// Types of causes for performance changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CauseType {
    /// Code or implementation changes
    CodeChange,
    /// Compiler version or optimization changes
    CompilerChange,
    /// System configuration modifications
    SystemConfiguration,
    /// External environmental factors
    ExternalFactors,
    /// Test methodology or setup changes
    TestMethodology,
    /// Hardware or infrastructure changes
    HardwareChange,
    /// Unknown or unidentified cause
    Unknown,
}

/// Likely cause of performance improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementCause {
    /// Type of likely cause
    pub cause_type: CauseType,
    /// Description
    pub description: String,
    /// Confidence in this cause (0-100)
    pub confidence: f64,
    /// Supporting evidence
    pub evidence: Vec<String>,
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

/// Overall assessment of performance state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallAssessment {
    /// Overall performance health score (0-100)
    pub health_score: f64,
    /// Performance status
    pub status: PerformanceStatus,
    /// Key findings
    pub key_findings: Vec<String>,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Confidence in assessment
    pub confidence: f64,
}

/// Overall performance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceStatus {
    /// Excellent performance (all metrics improving or stable)
    Excellent,      // All metrics improving or stable
    /// Good performance (minor issues, mostly stable)
    Good,           // Minor issues, mostly stable
    /// Concerning performance (some notable regressions)
    Concerning,     // Some notable regressions
    /// Poor performance (multiple significant regressions)
    Poor,           // Multiple significant regressions
    /// Critical performance (severe widespread regressions)
    Critical,       // Severe widespread regressions
}

/// Risk level for performance issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk level
    Low,
    /// Medium risk level
    Medium,
    /// High risk level
    High,
    /// Critical risk level
    Critical,
}

/// Action recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecommendation {
    /// Priority level (1-10, 10 = highest)
    pub priority: u8,
    /// Recommended action
    pub action: RecommendedAction,
    /// Description
    pub description: String,
    /// Expected impact
    pub expected_impact: String,
    /// Effort required
    pub effort_level: EffortLevel,
    /// Timeline for action
    pub timeline: String,
}

/// Types of recommended actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    /// Investigate the performance issue
    Investigate,
    /// Rollback recent changes
    Rollback,
    /// Optimize code performance
    OptimizeCode,
    /// Update performance baseline
    UpdateBaseline,
    /// Increase monitoring frequency
    IncreaseMonitoring,
    /// Change test methodology
    ChangeTestMethod,
    /// No action required
    NoAction,
}

/// Effort level for implementing recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    /// Low effort level (< 1 day)
    Low,      // < 1 day
    /// Medium effort level (1-5 days)
    Medium,   // 1-5 days
    /// High effort level (> 5 days)
    High,     // > 5 days
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
            config: self.config.clone()),
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
            let recent_count = (baseline.measurements.len() / 4).max(5).min(10);
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
                        test_id: baseline.test_id.clone()),
                        implementation: baseline.implementation_id.clone()),
                        degradation_percent: -percent_change,
                        statistical_significance: significance.clone()),
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
            else if percent_change > self.config.regression_threshold {
                if !self.config.require_statistical_significance || significance.is_significant {
                    let improvement = PerformanceImprovement {
                        test_id: baseline.test_id.clone()),
                        implementation: baseline.implementation_id.clone()),
                        improvement_percent: percent_change,
                        statistical_significance: significance.clone()),
                        current_vs_baseline: self.create_performance_comparison(baseline, &recent_measurements),
                        likely_causes: self.analyze_improvement_causes(baseline, &recent_measurements),
                        first_detected: SystemTime::now(),
                        confidence: self.calculate_detection_confidence(&significance, percent_change),
                    };
                    improvements.push(improvement);
                }
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
        
        let current_statistics = self.calculate_statistics(&recent_values);
        let difference = self.calculate_difference_metrics(&baseline.statistics, &current_statistics);
        
        PerformanceComparison {
            baseline: baseline.statistics.clone()),
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

impl BaselineManager {
    fn new(config: RegressionDetectionConfig) -> Self {
        Self {
            baselines: HashMap::new(),
            config,
        }
    }
    
    fn add_measurement(&mut self, measurement: PerformanceMeasurement) {
        let key = format!("{}_{}", measurement.parameters.get("test_id").unwrap_or(&"unknown".to_string()), 
                         measurement.parameters.get("implementation").unwrap_or(&"unknown".to_string()));
        
        let baseline = self.baselines.entry(key.clone()).or_insert_with(|| BaselineData {
            test_id: measurement.parameters.get("test_id").unwrap_or(&"unknown".to_string()).clone()),
            implementation_id: measurement.parameters.get("implementation").unwrap_or(&"unknown".to_string()).clone()),
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

impl TrendAnalyzer {
    fn new(config: RegressionDetectionConfig) -> Self {
        Self { config }
    }
    
    fn analyze_trends(&self, baselines: &HashMap<String, BaselineData>) -> HashMap<String, TrendAnalysis> {
        let mut trends = HashMap::new();
        
        for (key, baseline) in baselines {
            if baseline.measurements.len() < self.config.trend_window_size {
                continue;
            }
            
            let trend = self.analyze_single_trend(baseline);
            trends.insert(key.clone()), trend);
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
            test_id: baseline.test_id.clone()),
            implementation: baseline.implementation_id.clone()),
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

impl AnomalyDetector {
    fn new(config: RegressionDetectionConfig) -> Self {
        Self { config }
    }
    
    fn detect_anomalies(&self, baselines: &HashMap<String, BaselineData>) -> Vec<PerformanceAnomaly> {
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
                    test_id: baseline.test_id.clone()),
                    implementation: baseline.implementation_id.clone()),
                    anomaly_type,
                    anomaly_score: z_score,
                    anomalous_measurement: measurement.clone()),
                    detection_method: AnomalyDetectionMethod::StatisticalOutlier,
                    detected_at: SystemTime::now(),
                });
            }
        }
        
        anomalies
    }
}

/// Generate a comprehensive regression detection report
pub fn generate_regression_report(result: &RegressionDetectionResult) -> String {
    let mut report = String::new();
    
    report.push_str("# Performance Regression Detection Report\n\n");
    
    // Overall assessment
    report.push_str(&format!("## Overall Assessment\n\n"));
    report.push_str(&format!("- **Health Score:** {:.1}/100\n", result.overall_assessment.health_score));
    report.push_str(&format!("- **Status:** {:?}\n", result.overall_assessment.status));
    report.push_str(&format!("- **Risk Level:** {:?}\n", result.overall_assessment.risk_level));
    
    // Key findings
    report.push_str(&format!("\n### Key Findings\n"));
    for finding in &result.overall_assessment.key_findings {
        report.push_str(&format!("- {}\n", finding));
    }
    
    // Regressions
    if !result.regressions.is_empty() {
        report.push_str(&format!("\n## Detected Regressions ({})\n\n", result.regressions.len()));
        
        for regression in &result.regressions {
            report.push_str(&format!("### {} - {}\n", regression.implementation, regression.test_id));
            report.push_str(&format!("- **Degradation:** {:.1}%\n", regression.degradation_percent));
            report.push_str(&format!("- **Severity:** {:?}\n", regression.severity));
            report.push_str(&format!("- **Statistical Significance:** p={:.4}\n", regression.statistical_significance.p_value));
            report.push_str(&format!("- **Confidence:** {:.1}%\n", regression.confidence));
        }
    }
    
    // Improvements
    if !result.improvements.is_empty() {
        report.push_str(&format!("\n## Detected Improvements ({})\n\n", result.improvements.len()));
        
        for improvement in &result.improvements {
            report.push_str(&format!("### {} - {}\n", improvement.implementation, improvement.test_id));
            report.push_str(&format!("- **Improvement:** {:.1}%\n", improvement.improvement_percent));
            report.push_str(&format!("- **Statistical Significance:** p={:.4}\n", improvement.statistical_significance.p_value));
            report.push_str(&format!("- **Confidence:** {:.1}%\n", improvement.confidence));
        }
    }
    
    // Recommendations
    if !result.recommendations.is_empty() {
        report.push_str(&format!("\n## Recommended Actions\n\n"));
        
        let mut sorted_recommendations = result.recommendations.clone());
        sorted_recommendations.sort_by_key(|r| std::cmp::Reverse(r.priority));
        
        for (i, rec) in sorted_recommendations.iter().enumerate() {
            report.push_str(&format!("{}. **{:?}** (Priority: {}/10)\n", i + 1, rec.action, rec.priority));
            report.push_str(&format!("   - {}\n", rec.description));
            report.push_str(&format!("   - Timeline: {}\n", rec.timeline));
            report.push_str(&format!("   - Effort: {:?}\n", rec.effort_level));
        }
    }
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_regression_detector_creation() {
        let config = RegressionDetectionConfig::default();
        let detector = RegressionDetector::new(config);
        assert_eq!(detector.config.regression_threshold, 5.0);
    }
    
    #[test]
    fn test_baseline_management() {
        let mut manager = BaselineManager::new(RegressionDetectionConfig::default());
        
        let measurement = PerformanceMeasurement {
            timestamp: SystemTime::now(),
            value: 100.0,
            parameters: [
                ("test_id".to_string(), "test1".to_string()),
                ("implementation".to_string(), "impl1".to_string()),
            ].iter().clone())().collect(),
            context: SystemContext {
                commit_hash: None,
                compiler_version: "rustc 1.70".to_string(),
                system_load: 0.5,
                available_memory_mb: 4096,
                temperature: None,
            },
            confidence: 95.0,
        };
        
        manager.add_measurement(measurement);
        assert_eq!(manager.baselines.len(), 1);
    }
}