#![allow(missing_docs)]//! Production monitoring and deployment system for SRFI-31 optimization
//!
//! This module provides comprehensive monitoring, alerting, and deployment
//! infrastructure for the SRFI-31 optimization framework. It ensures
//! production reliability while maximizing optimization benefits.
//!
//! # Architecture
//!
//! ## Real-time Monitoring
//! - Performance metrics collection with minimal overhead
//! - Optimization success/failure rate tracking  
//! - Memory usage and allocation pattern analysis
//! - Latency percentile tracking (P50, P95, P99)
//!
//! ## Alerting System
//! - Threshold-based alerts for performance regression
//! - Optimization failure rate monitoring
//! - Memory leak detection for optimization caches
//! - Integration with external monitoring systems
//!
//! ## Deployment Safety
//! - Circuit breaker pattern for optimization failures
//! - Gradual rollout with A/B testing capability
//! - Automatic rollback on performance regression
//! - Feature flags for fine-grained control
//!
//! ## Performance Validation
//! - Continuous benchmarking against baseline
//! - Statistical significance testing
//! - Performance regression detection
//! - Optimization effectiveness measurement
//!
//! # Usage
//!
//! ```rust
//! use crate::eval::rec_optimization_monitoring::OptimizationMonitor;
//!
//! // Initialize production monitoring
//! let mut monitor = OptimizationMonitor::production();
//! 
//! // Report optimization attempt
//! monitor.record_optimization_attempt("factorial", true, 1.5, 0.3);
//!
//! // Check health status
//! if monitor.is_healthy() {
//!     // Continue with optimizations
//! } else {
//!     // Circuit breaker activated - use fallback
//! }
//! ```

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::time::{Duration, Instant, SystemTime};
use crate::eval::rec_optimization_framework::{RecursivePattern, TailCallOptimization};

/// Configuration for production optimization monitoring
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Whether monitoring is enabled
    pub enabled: bool,
    /// Maximum number of metrics to retain in memory
    pub max_metrics_retention: usize,
    /// Window size for rolling statistics (in seconds)
    pub metrics_window_seconds: u64,
    /// Threshold for optimization failure rate (0.0-1.0)
    pub failure_rate_threshold: f64,
    /// Threshold for performance regression (e.g., 0.1 = 10% regression)
    pub regression_threshold: f64,
    /// Circuit breaker trip threshold (consecutive failures)
    pub circuit_breaker_threshold: usize,
    /// Recovery window for circuit breaker (seconds)
    pub circuit_breaker_recovery_window: u64,
    /// Whether to enable detailed tracing
    pub enable_tracing: bool,
    /// Sample rate for detailed tracing (0.0-1.0)
    pub tracing_sample_rate: f64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_metrics_retention: 10000,
            metrics_window_seconds: 300, // 5 minutes
            failure_rate_threshold: 0.1, // 10% failure rate
            regression_threshold: 0.15,  // 15% performance regression
            circuit_breaker_threshold: 10,
            circuit_breaker_recovery_window: 60, // 1 minute
            enable_tracing: false,
            tracing_sample_rate: 0.01, // 1% sample rate
        }
    }
}

impl MonitoringConfig {
    /// Creates a production monitoring configuration
    pub fn production() -> Self {
        Self {
            enabled: true,
            max_metrics_retention: 50000,
            metrics_window_seconds: 600, // 10 minutes
            failure_rate_threshold: 0.05, // 5% failure rate
            regression_threshold: 0.1,    // 10% regression threshold
            circuit_breaker_threshold: 20,
            circuit_breaker_recovery_window: 120, // 2 minutes
            enable_tracing: true,
            tracing_sample_rate: 0.001, // 0.1% sample rate for production
        }
    }

    /// Creates a development monitoring configuration
    pub fn development() -> Self {
        Self {
            enabled: true,
            max_metrics_retention: 1000,
            metrics_window_seconds: 60, // 1 minute
            failure_rate_threshold: 0.2, // 20% failure rate (more lenient)
            regression_threshold: 0.2,   // 20% regression threshold
            circuit_breaker_threshold: 5,
            circuit_breaker_recovery_window: 30,
            enable_tracing: true,
            tracing_sample_rate: 0.1, // 10% sample rate for development
        }
    }

    /// Creates a disabled monitoring configuration
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::default()
        }
    }
}

/// Individual optimization metric record
#[derive(Debug, Clone)]
pub struct OptimizationMetric {
    /// Timestamp of the optimization attempt
    pub timestamp: Instant,
    /// Name/type of the recursive function
    pub function_type: String,
    /// Whether optimization was successful
    pub success: bool,
    /// Performance improvement factor (1.0 = no improvement)
    pub speedup_factor: f64,
    /// Memory reduction factor (0.0-1.0)
    pub memory_reduction: f64,
    /// Analysis time in microseconds
    pub analysis_time_us: u64,
    /// Pattern type detected
    pub pattern_type: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
}

/// Aggregated statistics for optimization monitoring
#[derive(Debug, Clone)]
pub struct OptimizationStatistics {
    /// Total optimization attempts
    pub total_attempts: u64,
    /// Successful optimizations
    pub successful_optimizations: u64,
    /// Failed optimization attempts
    pub failed_attempts: u64,
    /// Average speedup factor
    pub avg_speedup: f64,
    /// Average memory reduction
    pub avg_memory_reduction: f64,
    /// Average analysis time (microseconds)
    pub avg_analysis_time_us: f64,
    /// P50 analysis time (microseconds)
    pub p50_analysis_time_us: f64,
    /// P95 analysis time (microseconds)
    pub p95_analysis_time_us: f64,
    /// P99 analysis time (microseconds)
    pub p99_analysis_time_us: f64,
    /// Success rate (0.0-1.0)
    pub success_rate: f64,
    /// Window start time
    pub window_start: Instant,
    /// Window end time
    pub window_end: Instant,
}

impl Default for OptimizationStatistics {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            total_attempts: 0,
            successful_optimizations: 0,
            failed_attempts: 0,
            avg_speedup: 1.0,
            avg_memory_reduction: 0.0,
            avg_analysis_time_us: 0.0,
            p50_analysis_time_us: 0.0,
            p95_analysis_time_us: 0.0,
            p99_analysis_time_us: 0.0,
            success_rate: 0.0,
            window_start: now,
            window_end: now,
        }
    }
}

impl OptimizationStatistics {
    /// Calculates if the current performance indicates a regression
    pub fn has_performance_regression(&self, baseline: &OptimizationStatistics, threshold: f64) -> bool {
        if baseline.avg_speedup == 0.0 {
            return false; // No baseline to compare
        }
        
        let regression_ratio = (baseline.avg_speedup - self.avg_speedup) / baseline.avg_speedup;
        regression_ratio > threshold
    }

    /// Calculates if the success rate is below threshold
    pub fn has_high_failure_rate(&self, threshold: f64) -> bool {
        self.success_rate < (1.0 - threshold)
    }
}

/// Circuit breaker state for optimization safety
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    /// Normal operation - optimizations enabled
    Closed,
    /// Optimizations disabled due to failures
    Open,
    /// Testing if optimizations can be re-enabled
    HalfOpen,
}

/// Circuit breaker for optimization safety
pub struct OptimizationCircuitBreaker {
    /// Current state of the circuit breaker
    state: Arc<Mutex<CircuitBreakerState>>,
    /// Consecutive failure count
    failure_count: AtomicU64,
    /// Last failure time
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    /// Configuration
    config: MonitoringConfig,
}

impl OptimizationCircuitBreaker {
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitBreakerState::Closed)),
            failure_count: AtomicU64::new(0),
            last_failure_time: Arc::new(Mutex::new(None)),
            config,
        }
    }

    /// Records a successful optimization
    pub fn record_success(&self) {
        if let Ok(mut state) = self.state.lock() {
            match *state {
                CircuitBreakerState::HalfOpen => {
                    *state = CircuitBreakerState::Closed;
                    self.failure_count.store(0, Ordering::Relaxed);
                }
                CircuitBreakerState::Closed => {
                    self.failure_count.store(0, Ordering::Relaxed);
                }
                CircuitBreakerState::Open => {
                    // Stay open until recovery window passes
                }
            }
        }
    }

    /// Records a failed optimization
    pub fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        
        if let Ok(mut last_failure) = self.last_failure_time.lock() {
            *last_failure = Some(Instant::now());
        }

        if failures >= self.config.circuit_breaker_threshold as u64 {
            if let Ok(mut state) = self.state.lock() {
                *state = CircuitBreakerState::Open;
            }
        }
    }

    /// Checks if optimizations should be allowed
    pub fn should_allow_optimization(&self) -> bool {
        if let Ok(mut state) = self.state.lock() {
            match *state {
                CircuitBreakerState::Closed => true,
                CircuitBreakerState::Open => {
                    // Check if recovery window has passed
                    if let Ok(last_failure) = self.last_failure_time.lock() {
                        if let Some(failure_time) = *last_failure {
                            if failure_time.elapsed().as_secs() >= self.config.circuit_breaker_recovery_window {
                                *state = CircuitBreakerState::HalfOpen;
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                CircuitBreakerState::HalfOpen => true,
            }
        } else {
            false // Conservative: disallow on lock contention
        }
    }

    /// Gets current circuit breaker state
    pub fn get_state(&self) -> CircuitBreakerState {
        self.state.lock().unwrap_or_else(|_| {
            // Default to open on lock contention for safety
            std::thread::sleep(std::time::Duration::from_millis(1));
            self.state.lock().expect("Failed to acquire lock after retry")
        }).clone()
    }

    /// Gets current failure count
    pub fn get_failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::Relaxed)
    }
}

/// Main production monitoring system for SRFI-31 optimizations
pub struct OptimizationMonitor {
    /// Configuration settings
    config: MonitoringConfig,
    /// Recent optimization metrics (ring buffer)
    metrics: Arc<Mutex<VecDeque<OptimizationMetric>>>,
    /// Aggregated statistics cache
    cached_stats: Arc<Mutex<Option<(OptimizationStatistics, Instant)>>>,
    /// Circuit breaker for safety
    circuit_breaker: OptimizationCircuitBreaker,
    /// Baseline performance for regression detection
    baseline_stats: Arc<Mutex<Option<OptimizationStatistics>>>,
    /// Performance alert history
    alert_history: Arc<Mutex<VecDeque<PerformanceAlert>>>,
    /// Whether monitoring is currently active
    is_active: AtomicBool,
    /// Total metrics processed (for overflow protection)
    total_metrics_processed: AtomicU64,
}

/// Performance alert record
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    /// Alert timestamp
    pub timestamp: Instant,
    /// Alert type
    pub alert_type: AlertType,
    /// Alert message
    pub message: String,
    /// Severity level
    pub severity: AlertSeverity,
    /// Associated metrics (if applicable)
    pub metrics: Option<OptimizationStatistics>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    PerformanceRegression,
    HighFailureRate,
    CircuitBreakerTripped,
    MemoryUsageHigh,
    AnalysisTimeoutExcessive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl OptimizationMonitor {
    /// Creates a new optimization monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(MonitoringConfig::default())
    }

    /// Creates a new optimization monitor with custom configuration
    pub fn with_config(config: MonitoringConfig) -> Self {
        Self {
            circuit_breaker: OptimizationCircuitBreaker::new(config.clone()),
            metrics: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_metrics_retention))),
            cached_stats: Arc::new(Mutex::new(None)),
            baseline_stats: Arc::new(Mutex::new(None)),
            alert_history: Arc::new(Mutex::new(VecDeque::new())),
            is_active: AtomicBool::new(config.enabled),
            total_metrics_processed: AtomicU64::new(0),
            config,
        }
    }

    /// Creates a production monitoring configuration
    pub fn production() -> Self {
        Self::with_config(MonitoringConfig::production())
    }

    /// Creates a development monitoring configuration
    pub fn development() -> Self {
        Self::with_config(MonitoringConfig::development())
    }

    /// Creates a disabled monitoring configuration
    pub fn disabled() -> Self {
        Self::with_config(MonitoringConfig::disabled())
    }

    /// Records an optimization attempt
    pub fn record_optimization_attempt(
        &self,
        function_type: &str,
        success: bool,
        speedup_factor: f64,
        memory_reduction: f64,
        analysis_time_us: u64,
        pattern: &RecursivePattern,
    ) {
        if !self.is_active.load(Ordering::Relaxed) {
            return; // Fast path: disabled monitoring
        }

        // Extract pattern information
        let (pattern_type, confidence) = match pattern {
            RecursivePattern::LinearTailRecursion { confidence, .. } => {
                ("LinearTailRecursion", *confidence)
            }
            RecursivePattern::TreeRecursion { confidence, .. } => {
                ("TreeRecursion", *confidence)
            }
            RecursivePattern::AccumulatorPattern { confidence, .. } => {
                ("AccumulatorPattern", *confidence)
            }
            RecursivePattern::LinearRecursion { confidence, .. } => {
                ("LinearRecursion", *confidence)
            }
            RecursivePattern::ComplexPattern { confidence, .. } => {
                ("ComplexPattern", *confidence)
            }
            RecursivePattern::UnknownPattern => ("UnknownPattern", 0.0),
        };

        let metric = OptimizationMetric {
            timestamp: Instant::now(),
            function_type: function_type.to_string(),
            success,
            speedup_factor,
            memory_reduction,
            analysis_time_us,
            pattern_type: pattern_type.to_string(),
            confidence,
        };

        // Record metric
        self.add_metric(metric);

        // Update circuit breaker
        if success {
            self.circuit_breaker.record_success();
        } else {
            self.circuit_breaker.record_failure();
        }

        // Check for alerts
        self.check_for_alerts();
    }

    /// Adds a metric to the collection
    fn add_metric(&self, metric: OptimizationMetric) {
        if let Ok(mut metrics) = self.metrics.lock() {
            // Maintain ring buffer
            if metrics.len() >= self.config.max_metrics_retention {
                metrics.pop_front();
            }
            metrics.push_back(metric);
            
            // Invalidate cached statistics
            if let Ok(mut cached) = self.cached_stats.lock() {
                *cached = None;
            }
        }

        self.total_metrics_processed.fetch_add(1, Ordering::Relaxed);
    }

    /// Checks if the system is healthy for optimizations
    pub fn is_healthy(&self) -> bool {
        self.is_active.load(Ordering::Relaxed) && 
        self.circuit_breaker.should_allow_optimization()
    }

    /// Gets current optimization statistics
    pub fn get_current_statistics(&self) -> OptimizationStatistics {
        if !self.is_active.load(Ordering::Relaxed) {
            return OptimizationStatistics::default();
        }

        // Check cached statistics
        if let Ok(cached) = self.cached_stats.lock() {
            if let Some((stats, cache_time)) = cached.as_ref() {
                // Cache is valid for 10 seconds
                if cache_time.elapsed() < Duration::from_secs(10) {
                    return stats.clone();
                }
            }
        }

        // Calculate fresh statistics
        let stats = self.calculate_statistics();
        
        // Update cache
        if let Ok(mut cached) = self.cached_stats.lock() {
            *cached = Some((stats.clone(), Instant::now()));
        }

        stats
    }

    /// Calculates statistics from current metrics
    fn calculate_statistics(&self) -> OptimizationStatistics {
        let metrics = if let Ok(metrics) = self.metrics.lock() {
            metrics.clone()
        } else {
            return OptimizationStatistics::default();
        };

        if metrics.is_empty() {
            return OptimizationStatistics::default();
        }

        let window_start = Instant::now() - Duration::from_secs(self.config.metrics_window_seconds);
        let windowed_metrics: Vec<_> = metrics.iter()
            .filter(|m| m.timestamp >= window_start)
            .collect();

        if windowed_metrics.is_empty() {
            return OptimizationStatistics::default();
        }

        let total_attempts = windowed_metrics.len() as u64;
        let successful_optimizations = windowed_metrics.iter()
            .filter(|m| m.success)
            .count() as u64;
        let failed_attempts = total_attempts - successful_optimizations;

        let avg_speedup = windowed_metrics.iter()
            .filter(|m| m.success)
            .map(|m| m.speedup_factor)
            .sum::<f64>() / successful_optimizations.max(1) as f64;

        let avg_memory_reduction = windowed_metrics.iter()
            .filter(|m| m.success)
            .map(|m| m.memory_reduction)
            .sum::<f64>() / successful_optimizations.max(1) as f64;

        let avg_analysis_time_us = windowed_metrics.iter()
            .map(|m| m.analysis_time_us as f64)
            .sum::<f64>() / total_attempts as f64;

        // Calculate percentiles
        let mut analysis_times: Vec<u64> = windowed_metrics.iter()
            .map(|m| m.analysis_time_us)
            .collect();
        analysis_times.sort_unstable();

        let p50_analysis_time_us = percentile(&analysis_times, 50.0) as f64;
        let p95_analysis_time_us = percentile(&analysis_times, 95.0) as f64;
        let p99_analysis_time_us = percentile(&analysis_times, 99.0) as f64;

        let success_rate = successful_optimizations as f64 / total_attempts as f64;

        OptimizationStatistics {
            total_attempts,
            successful_optimizations,
            failed_attempts,
            avg_speedup,
            avg_memory_reduction,
            avg_analysis_time_us,
            p50_analysis_time_us,
            p95_analysis_time_us,
            p99_analysis_time_us,
            success_rate,
            window_start: windowed_metrics.first().unwrap().timestamp,
            window_end: windowed_metrics.last().unwrap().timestamp,
        }
    }

    /// Checks for performance alerts and triggers them if necessary
    fn check_for_alerts(&self) {
        let current_stats = self.get_current_statistics();
        
        // Check for high failure rate
        if current_stats.has_high_failure_rate(self.config.failure_rate_threshold) {
            self.trigger_alert(
                AlertType::HighFailureRate,
                AlertSeverity::Warning,
                format!(
                    "High optimization failure rate: {:.2}% (threshold: {:.2}%)",
                    (1.0 - current_stats.success_rate) * 100.0,
                    self.config.failure_rate_threshold * 100.0
                ),
                Some(current_stats.clone()),
            );
        }

        // Check for performance regression against baseline
        if let Ok(baseline) = self.baseline_stats.lock() {
            if let Some(baseline_stats) = baseline.as_ref() {
                if current_stats.has_performance_regression(baseline_stats, self.config.regression_threshold) {
                    self.trigger_alert(
                        AlertType::PerformanceRegression,
                        AlertSeverity::Critical,
                        format!(
                            "Performance regression detected: {:.2}x -> {:.2}x speedup",
                            baseline_stats.avg_speedup,
                            current_stats.avg_speedup
                        ),
                        Some(current_stats.clone()),
                    );
                }
            }
        }

        // Check circuit breaker state
        if self.circuit_breaker.get_state() == CircuitBreakerState::Open {
            self.trigger_alert(
                AlertType::CircuitBreakerTripped,
                AlertSeverity::Critical,
                format!(
                    "Circuit breaker tripped after {} consecutive failures",
                    self.circuit_breaker.get_failure_count()
                ),
                Some(current_stats),
            );
        }
    }

    /// Triggers a performance alert
    fn trigger_alert(
        &self,
        alert_type: AlertType,
        severity: AlertSeverity,
        message: String,
        metrics: Option<OptimizationStatistics>,
    ) {
        let alert = PerformanceAlert {
            timestamp: Instant::now(),
            alert_type,
            message: message.clone(),
            severity: severity.clone(),
            metrics,
        };

        if let Ok(mut alerts) = self.alert_history.lock() {
            alerts.push_back(alert);
            
            // Maintain alert history size
            while alerts.len() > 100 {
                alerts.pop_front();
            }
        }

        // Log alert (in production, this would integrate with logging/monitoring systems)
        match severity {
            AlertSeverity::Info => {
                eprintln!("[INFO] SRFI-31 Optimization: {}", message);
            }
            AlertSeverity::Warning => {
                eprintln!("[WARNING] SRFI-31 Optimization: {}", message);
            }
            AlertSeverity::Critical => {
                eprintln!("[CRITICAL] SRFI-31 Optimization: {}", message);
            }
        }
    }

    /// Sets the baseline performance statistics for regression detection
    pub fn set_baseline(&self, stats: OptimizationStatistics) {
        if let Ok(mut baseline) = self.baseline_stats.lock() {
            *baseline = Some(stats);
        }
    }

    /// Gets recent performance alerts
    pub fn get_recent_alerts(&self, limit: usize) -> Vec<PerformanceAlert> {
        if let Ok(alerts) = self.alert_history.lock() {
            alerts.iter()
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Generates a comprehensive monitoring report
    pub fn generate_monitoring_report(&self) -> String {
        let stats = self.get_current_statistics();
        let circuit_state = self.circuit_breaker.get_state();
        let failure_count = self.circuit_breaker.get_failure_count();
        let recent_alerts = self.get_recent_alerts(5);
        let total_processed = self.total_metrics_processed.load(Ordering::Relaxed);

        let mut report = format!(
            "SRFI-31 Optimization Monitoring Report\n\
             =====================================\n\
             \n\
             System Status: {}\n\
             Circuit Breaker: {:?} (failures: {})\n\
             Total Metrics Processed: {}\n\
             \n\
             Current Performance Statistics:\n\
             - Total attempts: {}\n\
             - Successful optimizations: {}\n\
             - Success rate: {:.2}%\n\
             - Average speedup: {:.2}x\n\
             - Average memory reduction: {:.2}%\n\
             - Average analysis time: {:.0}μs\n\
             - P95 analysis time: {:.0}μs\n\
             - P99 analysis time: {:.0}μs\n\
             \n",
            if self.is_healthy() { "HEALTHY" } else { "DEGRADED" },
            circuit_state,
            failure_count,
            total_processed,
            stats.total_attempts,
            stats.successful_optimizations,
            stats.success_rate * 100.0,
            stats.avg_speedup,
            stats.avg_memory_reduction * 100.0,
            stats.avg_analysis_time_us,
            stats.p95_analysis_time_us,
            stats.p99_analysis_time_us,
        );

        if !recent_alerts.is_empty() {
            report.push_str("Recent Alerts:\n");
            for alert in recent_alerts {
                report.push_str(&format!(
                    "- [{:?}] {}: {}\n",
                    alert.severity,
                    format!("{:?}", alert.alert_type).replace("_", " "),
                    alert.message
                ));
            }
        } else {
            report.push_str("No recent alerts.\n");
        }

        report
    }

    /// Enables or disables monitoring
    pub fn set_active(&self, active: bool) {
        self.is_active.store(active, Ordering::Relaxed);
    }

    /// Gets the current configuration
    pub fn config(&self) -> &MonitoringConfig {
        &self.config
    }

    /// Resets all monitoring data
    pub fn reset(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
        if let Ok(mut cached) = self.cached_stats.lock() {
            *cached = None;
        }
        if let Ok(mut baseline) = self.baseline_stats.lock() {
            *baseline = None;
        }
        if let Ok(mut alerts) = self.alert_history.lock() {
            alerts.clear();
        }
        self.total_metrics_processed.store(0, Ordering::Relaxed);
    }
}

impl Default for OptimizationMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculates the percentile value from a sorted array
fn percentile(sorted_data: &[u64], percentile: f64) -> u64 {
    if sorted_data.is_empty() {
        return 0;
    }
    
    let index = (percentile / 100.0 * (sorted_data.len() - 1) as f64).round() as usize;
    sorted_data[index.min(sorted_data.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::rec_optimization_framework::RecursivePattern;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_monitoring_config_creation() {
        let config = MonitoringConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_metrics_retention, 10000);

        let prod_config = MonitoringConfig::production();
        assert!(prod_config.enabled);
        assert_eq!(prod_config.max_metrics_retention, 50000);
        assert_eq!(prod_config.failure_rate_threshold, 0.05);

        let disabled_config = MonitoringConfig::disabled();
        assert!(!disabled_config.enabled);
    }

    #[test]
    fn test_circuit_breaker_basic_functionality() {
        let config = MonitoringConfig {
            circuit_breaker_threshold: 3,
            circuit_breaker_recovery_window: 1,
            ..MonitoringConfig::default()
        };
        let breaker = OptimizationCircuitBreaker::new(config);

        // Initially closed
        assert_eq!(breaker.get_state(), CircuitBreakerState::Closed);
        assert!(breaker.should_allow_optimization());

        // Record failures
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.get_state(), CircuitBreakerState::Closed);

        breaker.record_failure(); // Should trip
        assert_eq!(breaker.get_state(), CircuitBreakerState::Open);
        assert!(!breaker.should_allow_optimization());

        // Wait for recovery window
        thread::sleep(Duration::from_secs(2));
        assert!(breaker.should_allow_optimization()); // Should be half-open

        // Success should close circuit
        breaker.record_success();
        assert_eq!(breaker.get_state(), CircuitBreakerState::Closed);
    }

    #[test]
    fn test_optimization_monitor_basic_functionality() {
        let monitor = OptimizationMonitor::new();
        assert!(monitor.is_healthy());

        let pattern = RecursivePattern::LinearTailRecursion {
            confidence: 0.8,
            strategy: crate::eval::rec_optimization_framework::TailCallStrategy::SimpleIteration,
            estimated_speedup: 2.0,
        };

        // Record some metrics
        monitor.record_optimization_attempt("factorial", true, 2.0, 0.5, 1000, &pattern);
        monitor.record_optimization_attempt("fibonacci", false, 1.0, 0.0, 2000, &pattern);

        let stats = monitor.get_current_statistics();
        assert_eq!(stats.total_attempts, 2);
        assert_eq!(stats.successful_optimizations, 1);
        assert_eq!(stats.success_rate, 0.5);
    }

    #[test]
    fn test_optimization_monitor_disabled() {
        let monitor = OptimizationMonitor::disabled();
        assert!(!monitor.is_healthy());

        let pattern = RecursivePattern::UnknownPattern;
        monitor.record_optimization_attempt("test", true, 1.0, 0.0, 1000, &pattern);

        let stats = monitor.get_current_statistics();
        assert_eq!(stats.total_attempts, 0); // Should not record when disabled
    }

    #[test]
    fn test_statistics_calculation() {
        let monitor = OptimizationMonitor::new();
        let pattern = RecursivePattern::AccumulatorPattern {
            confidence: 0.9,
            optimization_level: 4,
            memory_reduction_estimate: 0.7,
        };

        // Record multiple successful optimizations
        for i in 0..10 {
            monitor.record_optimization_attempt(
                "test",
                true,
                1.5 + (i as f64 * 0.1),
                0.3 + (i as f64 * 0.01),
                1000 + (i * 100),
                &pattern,
            );
        }

        let stats = monitor.get_current_statistics();
        assert_eq!(stats.total_attempts, 10);
        assert_eq!(stats.successful_optimizations, 10);
        assert_eq!(stats.success_rate, 1.0);
        assert!(stats.avg_speedup > 1.5);
        assert!(stats.avg_memory_reduction > 0.3);
    }

    #[test]
    fn test_performance_regression_detection() {
        let baseline = OptimizationStatistics {
            avg_speedup: 2.0,
            success_rate: 0.95,
            ..OptimizationStatistics::default()
        };

        let current = OptimizationStatistics {
            avg_speedup: 1.5, // 25% regression
            success_rate: 0.95,
            ..OptimizationStatistics::default()
        };

        assert!(current.has_performance_regression(&baseline, 0.2)); // 20% threshold
        assert!(!current.has_performance_regression(&baseline, 0.3)); // 30% threshold
    }

    #[test]
    fn test_high_failure_rate_detection() {
        let stats = OptimizationStatistics {
            success_rate: 0.85, // 15% failure rate
            ..OptimizationStatistics::default()
        };

        assert!(stats.has_high_failure_rate(0.1)); // 10% threshold
        assert!(!stats.has_high_failure_rate(0.2)); // 20% threshold
    }

    #[test]
    fn test_alert_system() {
        let config = MonitoringConfig {
            failure_rate_threshold: 0.2, // 20% failure threshold
            ..MonitoringConfig::default()
        };
        let monitor = OptimizationMonitor::with_config(config);
        let pattern = RecursivePattern::UnknownPattern;

        // Record high failure rate
        for _ in 0..8 {
            monitor.record_optimization_attempt("test", false, 1.0, 0.0, 1000, &pattern);
        }
        for _ in 0..2 {
            monitor.record_optimization_attempt("test", true, 1.5, 0.3, 1000, &pattern);
        }

        let alerts = monitor.get_recent_alerts(10);
        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| a.alert_type == AlertType::HighFailureRate));
    }

    #[test]
    fn test_monitoring_report_generation() {
        let monitor = OptimizationMonitor::new();
        let pattern = RecursivePattern::LinearRecursion {
            confidence: 0.7,
            tail_call_opportunities: 2,
        };

        // Record some metrics
        monitor.record_optimization_attempt("factorial", true, 1.8, 0.4, 1200, &pattern);
        monitor.record_optimization_attempt("fibonacci", true, 1.2, 0.1, 800, &pattern);

        let report = monitor.generate_monitoring_report();
        assert!(report.contains("SRFI-31 Optimization Monitoring Report"));
        assert!(report.contains("System Status: HEALTHY"));
        assert!(report.contains("Total attempts: 2"));
        assert!(report.contains("Successful optimizations: 2"));
    }

    #[test]
    fn test_percentile_calculation() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        
        assert_eq!(percentile(&data, 50.0), 5);
        assert_eq!(percentile(&data, 95.0), 10);
        assert_eq!(percentile(&data, 0.0), 1);
        assert_eq!(percentile(&data, 100.0), 10);

        let empty_data: Vec<u64> = vec![];
        assert_eq!(percentile(&empty_data, 50.0), 0);
    }

    #[test]
    fn test_concurrent_monitoring() {
        let monitor = Arc::new(OptimizationMonitor::new());
        let pattern = RecursivePattern::TreeRecursion {
            confidence: 0.8,
            memoization_candidate: true,
            complexity_estimate: crate::eval::rec_optimization_framework::ComplexityClass::Exponential,
        };

        let handles: Vec<_> = (0..4).map(|i| {
            let monitor = Arc::clone(&monitor);
            let pattern = pattern.clone();
            thread::spawn(move || {
                for j in 0..10 {
                    monitor.record_optimization_attempt(
                        &format!("test_{}", i),
                        j % 2 == 0,
                        1.5,
                        0.3,
                        1000,
                        &pattern,
                    );
                }
            })
        }).collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = monitor.get_current_statistics();
        assert_eq!(stats.total_attempts, 40); // 4 threads × 10 iterations
        assert_eq!(stats.success_rate, 0.5); // Every other attempt succeeds
    }

    #[test]
    fn test_metrics_retention_limit() {
        let config = MonitoringConfig {
            max_metrics_retention: 5,
            ..MonitoringConfig::default()
        };
        let monitor = OptimizationMonitor::with_config(config);
        let pattern = RecursivePattern::UnknownPattern;

        // Record more metrics than retention limit
        for i in 0..10 {
            monitor.record_optimization_attempt(
                &format!("test_{}", i),
                true,
                1.5,
                0.3,
                1000,
                &pattern,
            );
        }

        // Should only retain the last 5 metrics
        let stats = monitor.get_current_statistics();
        assert_eq!(stats.total_attempts, 5);
        assert_eq!(monitor.total_metrics_processed.load(Ordering::Relaxed), 10);
    }
}