//! Performance Detection Module
//!
//! このモジュールはパフォーマンス回帰検出と最適化推奨システムを実装します。
//! 統計的回帰検出、自動アラート、最適化推奨を含みます。

use super::core_types::{
    PerformanceRegressionAlert, RegressionSeverity, OptimizationRecommendation, OptimizationType,
};
use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

/// Performance regression detection system
#[derive(Debug)]
pub struct PerformanceRegressionDetector {
    /// Historical performance data
    performance_history: std::collections::HashMap<String, VecDeque<PerformanceMeasurement>>,
    
    /// Active alerts
    active_alerts: Vec<PerformanceRegressionAlert>,
    
    /// Configuration parameters
    config: RegressionDetectionConfig,
}

/// Individual performance measurement
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    /// Execution time
    pub execution_time: Duration,
    
    /// Memory usage
    pub memory_usage: usize,
    
    /// Timestamp
    pub timestamp: SystemTime,
    
    /// Additional metrics
    pub cpu_cycles: Option<u64>,
    
    /// Cache misses
    pub cache_misses: Option<u64>,
}

/// Configuration for regression detection
#[derive(Debug, Clone)]
pub struct RegressionDetectionConfig {
    /// Minimum number of samples needed for regression detection
    pub min_samples: usize,
    
    /// Window size for baseline calculation
    pub baseline_window: usize,
    
    /// Threshold for minor regression (percentage)
    pub minor_threshold: f64,
    
    /// Threshold for moderate regression (percentage)
    pub moderate_threshold: f64,
    
    /// Threshold for major regression (percentage)
    pub major_threshold: f64,
    
    /// Threshold for critical regression (percentage)
    pub critical_threshold: f64,
    
    /// Statistical confidence level required
    pub confidence_level: f64,
}

/// Performance trend analysis
#[derive(Debug, Clone)]
pub struct PerformanceTrend {
    /// Trend direction
    pub direction: TrendDirection,
    
    /// Magnitude of trend (percentage change per time unit)
    pub magnitude: f64,
    
    /// Statistical confidence in trend
    pub confidence: f64,
    
    /// Trend duration
    pub duration: Duration,
}

/// Trend direction
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    /// Performance is improving
    Improving,
    
    /// Performance is stable
    Stable,
    
    /// Performance is degrading
    Degrading,
    
    /// Performance is highly variable
    Volatile,
}

impl PerformanceRegressionDetector {
    #[must_use] 
    /// Create a new performance regression detector with default configuration
    pub fn new() -> Self { 
        Self {
            performance_history: std::collections::HashMap::new(),
            active_alerts: Vec::new(),
            config: RegressionDetectionConfig::default(),
        }
    }
    
    /// Record a performance measurement
    pub fn record_measurement(&mut self, expr_hash: &str, measurement: PerformanceMeasurement) {
        let history = self.performance_history.entry(expr_hash.to_string())
            .or_default();
        
        history.push_back(measurement);
        
        // Keep history within reasonable bounds
        while history.len() > 1000 {
            history.pop_front();
        }
        
        // Check for regressions after each new measurement
        if let Some(alert) = self.detect_regression(expr_hash) {
            self.active_alerts.push(alert);
        }
    }
    
    /// Detect performance regression for an expression
    fn detect_regression(&self, expr_hash: &str) -> Option<PerformanceRegressionAlert> {
        let history = self.performance_history.get(expr_hash)?;
        
        if history.len() < self.config.min_samples {
            return None;
        }
        
        // Calculate baseline performance (average of oldest measurements)
        let baseline_size = self.config.baseline_window.min(history.len() / 2);
        let baseline_measurements: Vec<_> = history.iter().take(baseline_size).collect();
        let baseline_avg = self.calculate_average_performance(&baseline_measurements);
        
        // Calculate recent performance (average of newest measurements)
        let recent_size = self.config.baseline_window.min(history.len() / 2);
        let recent_measurements: Vec<_> = history.iter().rev().take(recent_size).collect();
        let recent_avg = self.calculate_average_performance(&recent_measurements);
        
        // Calculate regression factor
        let regression_factor = recent_avg.as_nanos() as f64 / baseline_avg.as_nanos() as f64;
        let regression_percentage = (regression_factor - 1.0) * 100.0;
        
        // Determine severity
        let severity = if regression_percentage >= self.config.critical_threshold {
            RegressionSeverity::Critical
        } else if regression_percentage >= self.config.major_threshold {
            RegressionSeverity::Major
        } else if regression_percentage >= self.config.moderate_threshold {
            RegressionSeverity::Moderate
        } else if regression_percentage >= self.config.minor_threshold {
            RegressionSeverity::Minor
        } else {
            return None; // No significant regression
        };
        
        Some(PerformanceRegressionAlert {
            expression: expr_hash.to_string(),
            severity,
            previous_performance: baseline_avg,
            current_performance: recent_avg,
            regression_factor,
            timestamp: SystemTime::now(),
            potential_causes: self.analyze_potential_causes(expr_hash, &baseline_measurements, &recent_measurements),
        })
    }
    
    /// Calculate average performance from measurements
    fn calculate_average_performance(&self, measurements: &[&PerformanceMeasurement]) -> Duration {
        if measurements.is_empty() {
            return Duration::ZERO;
        }
        
        let total_nanos: u64 = measurements.iter()
            .map(|m| m.execution_time.as_nanos() as u64)
            .sum();
        
        Duration::from_nanos(total_nanos / measurements.len() as u64)
    }
    
    /// Analyze potential causes of performance regression
    fn analyze_potential_causes(&self, _expr_hash: &str, baseline: &[&PerformanceMeasurement], recent: &[&PerformanceMeasurement]) -> Vec<String> {
        let mut causes = Vec::new();
        
        // Memory usage increase
        let baseline_memory: f64 = baseline.iter().map(|m| m.memory_usage as f64).sum::<f64>() / baseline.len() as f64;
        let recent_memory: f64 = recent.iter().map(|m| m.memory_usage as f64).sum::<f64>() / recent.len() as f64;
        
        if recent_memory > baseline_memory * 1.2 {
            causes.push("Increased memory usage detected".to_string());
        }
        
        // Cache miss increase (if available)
        let baseline_cache_misses = baseline.iter()
            .filter_map(|m| m.cache_misses)
            .collect::<Vec<_>>();
        let recent_cache_misses = recent.iter()
            .filter_map(|m| m.cache_misses)
            .collect::<Vec<_>>();
        
        if !baseline_cache_misses.is_empty() && !recent_cache_misses.is_empty() {
            let baseline_avg_misses = baseline_cache_misses.iter().sum::<u64>() as f64 / baseline_cache_misses.len() as f64;
            let recent_avg_misses = recent_cache_misses.iter().sum::<u64>() as f64 / recent_cache_misses.len() as f64;
            
            if recent_avg_misses > baseline_avg_misses * 1.3 {
                causes.push("Increased cache misses detected".to_string());
            }
        }
        
        // CPU cycles increase (if available)
        let baseline_cycles = baseline.iter()
            .filter_map(|m| m.cpu_cycles)
            .collect::<Vec<_>>();
        let recent_cycles = recent.iter()
            .filter_map(|m| m.cpu_cycles)
            .collect::<Vec<_>>();
        
        if !baseline_cycles.is_empty() && !recent_cycles.is_empty() {
            let baseline_avg_cycles = baseline_cycles.iter().sum::<u64>() as f64 / baseline_cycles.len() as f64;
            let recent_avg_cycles = recent_cycles.iter().sum::<u64>() as f64 / recent_cycles.len() as f64;
            
            if recent_avg_cycles > baseline_avg_cycles * 1.2 {
                causes.push("Increased CPU cycle count detected".to_string());
            }
        }
        
        if causes.is_empty() {
            causes.push("Unknown cause - requires further investigation".to_string());
        }
        
        causes
    }
    
    /// Get all performance regression alerts
    /// 
    /// Returns a list of detected performance regressions
    /// that exceed configured thresholds.
    #[must_use] pub fn get_alerts(&self) -> Vec<PerformanceRegressionAlert> {
        self.active_alerts.clone()
    }
    
    /// Clear alerts older than specified duration
    pub fn clear_old_alerts(&mut self, max_age: Duration) {
        let cutoff_time = SystemTime::now() - max_age;
        self.active_alerts.retain(|alert| alert.timestamp >= cutoff_time);
    }
    
    /// Get performance trend for an expression
    #[must_use] pub fn get_performance_trend(&self, expr_hash: &str) -> Option<PerformanceTrend> {
        let history = self.performance_history.get(expr_hash)?;
        
        if history.len() < 10 {
            return None;
        }
        
        // Simple linear regression on execution times
        let measurements: Vec<(f64, f64)> = history.iter()
            .enumerate()
            .map(|(i, m)| (i as f64, m.execution_time.as_nanos() as f64))
            .collect();
        
        let (slope, r_squared) = self.linear_regression(&measurements);
        
        let direction = if slope.abs() < 1000.0 { // Threshold for stable
            TrendDirection::Stable
        } else if slope > 0.0 {
            TrendDirection::Degrading
        } else {
            TrendDirection::Improving
        };
        
        // Calculate trend magnitude as percentage change per measurement
        let first_time = measurements.first()?.1;
        let magnitude = if first_time > 0.0 {
            (slope / first_time) * 100.0
        } else {
            0.0
        };
        
        Some(PerformanceTrend {
            direction,
            magnitude: magnitude.abs(),
            confidence: r_squared,
            duration: SystemTime::now().duration_since(history.front()?.timestamp).unwrap_or(Duration::ZERO),
        })
    }
    
    /// Simple linear regression
    fn linear_regression(&self, points: &[(f64, f64)]) -> (f64, f64) {
        if points.len() < 2 {
            return (0.0, 0.0);
        }
        
        let n = points.len() as f64;
        let sum_x: f64 = points.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = points.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = points.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = points.iter().map(|(x, _)| x * x).sum();
        let _sum_y2: f64 = points.iter().map(|(_, y)| y * y).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        
        // Calculate R-squared
        let mean_y = sum_y / n;
        let ss_tot: f64 = points.iter().map(|(_, y)| (y - mean_y).powi(2)).sum();
        let ss_res: f64 = points.iter().map(|(x, y)| {
            let predicted = slope * x + (sum_y - slope * sum_x) / n;
            (y - predicted).powi(2)
        }).sum();
        
        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };
        
        (slope, r_squared)
    }
    
    /// Generate optimization recommendations based on performance data
    #[must_use] pub fn generate_optimization_recommendations(&self, expr_hash: &str) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        if let Some(history) = self.performance_history.get(expr_hash) {
            if history.len() < 5 {
                return recommendations;
            }
            
            let recent_measurements: Vec<_> = history.iter().rev().take(10).collect();
            let avg_execution_time = self.calculate_average_performance(&recent_measurements);
            let avg_memory = recent_measurements.iter()
                .map(|m| m.memory_usage)
                .sum::<usize>() as f64 / recent_measurements.len() as f64;
            
            // High execution time -> JIT compilation
            if avg_execution_time > Duration::from_millis(10) {
                recommendations.push(OptimizationRecommendation {
                    optimization_type: OptimizationType::JITCompilation,
                    confidence: 0.8,
                    expected_speedup: 2.0,
                    description: "High execution time detected - JIT compilation recommended".to_string(),
                });
            }
            
            // High memory usage -> memory layout optimization
            if avg_memory > 10_000_000.0 { // 10MB
                recommendations.push(OptimizationRecommendation {
                    optimization_type: OptimizationType::MemoryLayoutOptimization,
                    confidence: 0.7,
                    expected_speedup: 1.3,
                    description: "High memory usage detected - memory layout optimization recommended".to_string(),
                });
            }
            
            // High cache miss rate -> cache optimization
            let high_cache_misses = recent_measurements.iter()
                .any(|m| m.cache_misses.unwrap_or(0) > 1000);
            
            if high_cache_misses {
                recommendations.push(OptimizationRecommendation {
                    optimization_type: OptimizationType::CacheOptimization,
                    confidence: 0.6,
                    expected_speedup: 1.5,
                    description: "High cache miss rate detected - cache optimization recommended".to_string(),
                });
            }
        }
        
        recommendations
    }
}

impl Default for RegressionDetectionConfig {
    fn default() -> Self {
        Self {
            min_samples: 10,
            baseline_window: 20,
            minor_threshold: 10.0,      // 10% regression
            moderate_threshold: 25.0,   // 25% regression
            major_threshold: 50.0,      // 50% regression
            critical_threshold: 100.0,  // 100% regression (2x slower)
            confidence_level: 0.95,
        }
    }
}

impl Default for PerformanceRegressionDetector {
    fn default() -> Self {
        Self::new()
    }
}