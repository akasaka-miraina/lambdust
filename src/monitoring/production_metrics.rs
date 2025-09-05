//! Production Metrics System for Phase 8.3 Rollout
//!
//! Real-time monitoring system that tracks optimization effectiveness,
//! system performance, and alerts on regression or failures.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};
use crate::feature::optimization_features::{OptimizationFeature, FeatureStatsSnapshot};

/// Production metrics collector
pub struct ProductionMetrics {
    /// Performance counters
    performance_counters: Arc<RwLock<PerformanceCounters>>,
    /// Optimization effectiveness tracking
    optimization_metrics: Arc<RwLock<OptimizationMetrics>>,
    /// Alert thresholds and configuration
    alert_config: AlertConfiguration,
    /// Metric history for trend analysis
    metric_history: Arc<Mutex<MetricHistory>>,
    /// Start time for uptime calculation
    start_time: Instant,
}

/// Core performance counters
#[derive(Debug, Clone)]
pub struct PerformanceCounters {
    /// Total operations processed
    pub total_operations: u64,
    /// Operations per second (rolling average)
    pub operations_per_second: f64,
    /// Average response time in microseconds
    pub avg_response_time_micros: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// CPU utilization percentage
    pub cpu_utilization_percent: f32,
    /// Garbage collection pressure
    pub gc_pressure_ratio: f32,
    /// Error rate (errors per 1000 operations)
    pub error_rate: f32,
}

/// Optimization-specific metrics
#[derive(Debug, Clone)]
pub struct OptimizationMetrics {
    /// NaN-boxing statistics
    pub nan_boxing: OptimizationEffectiveness,
    /// String interning statistics
    pub string_interning: OptimizationEffectiveness,
    /// Arena allocation statistics
    pub arena_allocation: OptimizationEffectiveness,
    /// Zero-copy parsing statistics
    pub zero_copy_parsing: OptimizationEffectiveness,
    /// Feature rollout percentages
    pub rollout_percentages: HashMap<OptimizationFeature, f32>,
}

/// Effectiveness metrics for a specific optimization
#[derive(Debug, Clone)]
pub struct OptimizationEffectiveness {
    /// Hit rate (0.0 to 1.0)
    pub hit_rate: f32,
    /// Memory savings in bytes
    pub memory_savings_bytes: u64,
    /// Performance improvement ratio
    pub performance_improvement: f32,
    /// Fallback rate (0.0 to 1.0)
    pub fallback_rate: f32,
    /// Usage count
    pub usage_count: u64,
    /// Last updated timestamp
    pub last_updated: SystemTime,
}

/// Alert configuration and thresholds
#[derive(Debug, Clone)]
pub struct AlertConfiguration {
    /// Memory usage spike threshold (percentage increase)
    pub memory_spike_threshold: f32,
    /// Response time degradation threshold (multiplier)
    pub latency_degradation_threshold: f32,
    /// Error rate increase threshold (percentage)
    pub error_rate_threshold: f32,
    /// CPU utilization alert threshold
    pub cpu_threshold: f32,
    /// Minimum time between identical alerts
    pub alert_cooldown: Duration,
}

/// Historical metrics for trend analysis
#[derive(Debug)]
pub struct MetricHistory {
    /// Performance snapshots over time
    performance_snapshots: Vec<(SystemTime, PerformanceCounters)>,
    /// Optimization effectiveness over time
    optimization_snapshots: Vec<(SystemTime, OptimizationMetrics)>,
    /// Maximum history size
    max_history_size: usize,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert message
#[derive(Debug, Clone)]
pub struct Alert {
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub timestamp: SystemTime,
    pub suggested_action: String,
}

impl Default for PerformanceCounters {
    fn default() -> Self {
        Self {
            total_operations: 0,
            operations_per_second: 0.0,
            avg_response_time_micros: 0.0,
            memory_usage_bytes: 0,
            peak_memory_bytes: 0,
            cpu_utilization_percent: 0.0,
            gc_pressure_ratio: 0.0,
            error_rate: 0.0,
        }
    }
}

impl Default for OptimizationEffectiveness {
    fn default() -> Self {
        Self {
            hit_rate: 0.0,
            memory_savings_bytes: 0,
            performance_improvement: 0.0,
            fallback_rate: 0.0,
            usage_count: 0,
            last_updated: SystemTime::now(),
        }
    }
}

impl Default for AlertConfiguration {
    fn default() -> Self {
        Self {
            memory_spike_threshold: 0.20,  // 20% memory increase
            latency_degradation_threshold: 1.10,  // 10% latency increase
            error_rate_threshold: 0.05,   // 5% error rate increase
            cpu_threshold: 0.80,          // 80% CPU utilization
            alert_cooldown: Duration::from_secs(60),  // 1 minute cooldown
        }
    }
}

impl ProductionMetrics {
    /// Create new production metrics system
    pub fn new() -> Self {
        Self {
            performance_counters: Arc::new(RwLock::new(PerformanceCounters::default())),
            optimization_metrics: Arc::new(RwLock::new(OptimizationMetrics::default())),
            alert_config: AlertConfiguration::default(),
            metric_history: Arc::new(Mutex::new(MetricHistory::new())),
            start_time: Instant::now(),
        }
    }

    /// Update performance counters
    pub fn update_performance(&self, counters: PerformanceCounters) -> Result<Vec<Alert>, Box<dyn std::error::Error>> {
        let mut alerts = Vec::new();
        
        // Check for alerts before updating
        {
            let current = self.performance_counters.read().unwrap();
            alerts.extend(self.check_performance_alerts(&current, &counters)?);
        }
        
        // Update counters
        {
            let mut perf = self.performance_counters.write().unwrap();
            *perf = counters.clone();
        }
        
        // Store in history
        {
            let mut history = self.metric_history.lock().unwrap();
            history.add_performance_snapshot(counters);
        }
        
        Ok(alerts)
    }

    /// Update optimization metrics
    pub fn update_optimization_metrics(&self, metrics: OptimizationMetrics) -> Result<Vec<Alert>, Box<dyn std::error::Error>> {
        let mut alerts = Vec::new();
        
        // Check optimization-specific alerts
        alerts.extend(self.check_optimization_alerts(&metrics)?);
        
        // Update metrics
        {
            let mut opt = self.optimization_metrics.write().unwrap();
            *opt = metrics.clone();
        }
        
        // Store in history
        {
            let mut history = self.metric_history.lock().unwrap();
            history.add_optimization_snapshot(metrics);
        }
        
        Ok(alerts)
    }

    /// Check for performance-related alerts
    fn check_performance_alerts(&self, current: &PerformanceCounters, new: &PerformanceCounters) -> Result<Vec<Alert>, Box<dyn std::error::Error>> {
        let mut alerts = Vec::new();
        
        // Memory spike alert
        if new.memory_usage_bytes > 0 && current.memory_usage_bytes > 0 {
            let memory_increase = (new.memory_usage_bytes as f32 - current.memory_usage_bytes as f32) 
                / current.memory_usage_bytes as f32;
            
            if memory_increase > self.alert_config.memory_spike_threshold {
                alerts.push(Alert {
                    severity: AlertSeverity::Warning,
                    title: "Memory Usage Spike Detected".to_string(),
                    message: format!("Memory usage increased by {:.1}%", memory_increase * 100.0),
                    metric_name: "memory_usage_bytes".to_string(),
                    current_value: new.memory_usage_bytes as f64,
                    threshold_value: (current.memory_usage_bytes as f32 * (1.0 + self.alert_config.memory_spike_threshold)) as f64,
                    timestamp: SystemTime::now(),
                    suggested_action: "Consider enabling arena allocation fallback or investigating memory leaks".to_string(),
                });
            }
        }
        
        // Response time degradation alert
        if new.avg_response_time_micros > current.avg_response_time_micros * self.alert_config.latency_degradation_threshold {
            alerts.push(Alert {
                severity: AlertSeverity::Warning,
                title: "Response Time Degradation".to_string(),
                message: format!("Average response time increased to {:.1}μs", new.avg_response_time_micros),
                metric_name: "avg_response_time_micros".to_string(),
                current_value: new.avg_response_time_micros,
                threshold_value: (current.avg_response_time_micros * self.alert_config.latency_degradation_threshold),
                timestamp: SystemTime::now(),
                suggested_action: "Consider reducing optimization rollout percentage or enabling circuit breakers".to_string(),
            });
        }
        
        // Error rate alert
        if new.error_rate > current.error_rate + self.alert_config.error_rate_threshold {
            let severity = if new.error_rate > 0.10 { AlertSeverity::Critical } else { AlertSeverity::Warning };
            
            alerts.push(Alert {
                severity,
                title: "Error Rate Increase Detected".to_string(),
                message: format!("Error rate increased to {:.2}%", new.error_rate * 100.0),
                metric_name: "error_rate".to_string(),
                current_value: new.error_rate as f64,
                threshold_value: (current.error_rate + self.alert_config.error_rate_threshold) as f64,
                timestamp: SystemTime::now(),
                suggested_action: "Investigate recent changes and consider immediate rollback if critical".to_string(),
            });
        }
        
        // CPU utilization alert
        if new.cpu_utilization_percent > self.alert_config.cpu_threshold {
            alerts.push(Alert {
                severity: AlertSeverity::Info,
                title: "High CPU Utilization".to_string(),
                message: format!("CPU utilization at {:.1}%", new.cpu_utilization_percent),
                metric_name: "cpu_utilization_percent".to_string(),
                current_value: new.cpu_utilization_percent as f64,
                threshold_value: self.alert_config.cpu_threshold as f64,
                timestamp: SystemTime::now(),
                suggested_action: "Monitor for sustained high usage and consider load balancing".to_string(),
            });
        }
        
        Ok(alerts)
    }

    /// Check for optimization-specific alerts
    fn check_optimization_alerts(&self, metrics: &OptimizationMetrics) -> Result<Vec<Alert>, Box<dyn std::error::Error>> {
        let mut alerts = Vec::new();
        
        // NaN-boxing fallback rate alert
        if metrics.nan_boxing.fallback_rate > 0.10 {  // 10% fallback rate
            alerts.push(Alert {
                severity: AlertSeverity::Warning,
                title: "High NaN-Boxing Fallback Rate".to_string(),
                message: format!("NaN-boxing fallback rate at {:.1}%", metrics.nan_boxing.fallback_rate * 100.0),
                metric_name: "nan_boxing_fallback_rate".to_string(),
                current_value: metrics.nan_boxing.fallback_rate as f64,
                threshold_value: 0.10,
                timestamp: SystemTime::now(),
                suggested_action: "Investigate value types causing fallbacks and consider optimization adjustments".to_string(),
            });
        }
        
        // String interning effectiveness alert
        if metrics.string_interning.hit_rate < 0.80 && metrics.string_interning.usage_count > 1000 {
            alerts.push(Alert {
                severity: AlertSeverity::Info,
                title: "String Interning Hit Rate Low".to_string(),
                message: format!("String interning hit rate at {:.1}%", metrics.string_interning.hit_rate * 100.0),
                metric_name: "string_interning_hit_rate".to_string(),
                current_value: metrics.string_interning.hit_rate as f64,
                threshold_value: 0.80,
                timestamp: SystemTime::now(),
                suggested_action: "Consider expanding pre-interned symbol set or adjusting interning strategy".to_string(),
            });
        }
        
        Ok(alerts)
    }

    /// Get current performance snapshot
    pub fn get_performance_snapshot(&self) -> PerformanceCounters {
        self.performance_counters.read().unwrap().clone()
    }

    /// Get current optimization metrics
    pub fn get_optimization_snapshot(&self) -> OptimizationMetrics {
        self.optimization_metrics.read().unwrap().clone()
    }

    /// Get system uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Generate comprehensive metrics report
    pub fn generate_report(&self) -> MetricsReport {
        let performance = self.get_performance_snapshot();
        let optimization = self.get_optimization_snapshot();
        let uptime = self.uptime();
        
        MetricsReport {
            performance,
            optimization,
            uptime,
            timestamp: SystemTime::now(),
            system_health: self.calculate_system_health(),
        }
    }

    /// Calculate overall system health score (0.0 to 1.0)
    fn calculate_system_health(&self) -> f32 {
        let performance = self.get_performance_snapshot();
        let optimization = self.get_optimization_snapshot();
        
        let mut health_score = 1.0;
        
        // Deduct for high error rate
        if performance.error_rate > 0.01 {  // > 1%
            health_score -= (performance.error_rate - 0.01) * 10.0;
        }
        
        // Deduct for high memory usage (if over 1GB)
        if performance.memory_usage_bytes > 1_000_000_000 {
            let excess_gb = (performance.memory_usage_bytes - 1_000_000_000) as f32 / 1_000_000_000.0;
            health_score -= excess_gb * 0.1;
        }
        
        // Bonus for effective optimizations
        let avg_hit_rate = (optimization.nan_boxing.hit_rate + optimization.string_interning.hit_rate) / 2.0;
        if avg_hit_rate > 0.90 {
            health_score += 0.05;  // 5% bonus for high optimization effectiveness
        }
        
        health_score.clamp(0.0, 1.0)
    }
}

impl OptimizationMetrics {
    fn default() -> Self {
        Self {
            nan_boxing: OptimizationEffectiveness::default(),
            string_interning: OptimizationEffectiveness::default(),
            arena_allocation: OptimizationEffectiveness::default(),
            zero_copy_parsing: OptimizationEffectiveness::default(),
            rollout_percentages: HashMap::new(),
        }
    }
}

impl MetricHistory {
    fn new() -> Self {
        Self {
            performance_snapshots: Vec::new(),
            optimization_snapshots: Vec::new(),
            max_history_size: 1440,  // 24 hours of minute-level data
        }
    }
    
    fn add_performance_snapshot(&mut self, counters: PerformanceCounters) {
        self.performance_snapshots.push((SystemTime::now(), counters));
        
        // Maintain history size limit
        if self.performance_snapshots.len() > self.max_history_size {
            self.performance_snapshots.remove(0);
        }
    }
    
    fn add_optimization_snapshot(&mut self, metrics: OptimizationMetrics) {
        self.optimization_snapshots.push((SystemTime::now(), metrics));
        
        // Maintain history size limit
        if self.optimization_snapshots.len() > self.max_history_size {
            self.optimization_snapshots.remove(0);
        }
    }
}

/// Comprehensive metrics report
#[derive(Debug, Clone)]
pub struct MetricsReport {
    pub performance: PerformanceCounters,
    pub optimization: OptimizationMetrics,
    pub uptime: Duration,
    pub timestamp: SystemTime,
    pub system_health: f32,
}

impl MetricsReport {
    /// Generate human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "=== Production Metrics Report ===\n\
            Timestamp: {:?}\n\
            Uptime: {:.1} hours\n\
            System Health: {:.1}%\n\
            \n\
            Performance:\n\
            - Operations: {} total, {:.1}/sec\n\
            - Response Time: {:.1}μs average\n\
            - Memory Usage: {:.1}MB current, {:.1}MB peak\n\
            - CPU: {:.1}%\n\
            - Error Rate: {:.3}%\n\
            \n\
            Optimizations:\n\
            - NaN-boxing: {:.1}% hit rate, {:.1}% fallback\n\
            - String Interning: {:.1}% hit rate, {} operations\n\
            - Arena Allocation: {:.1}MB saved\n\
            - Zero-copy Parsing: {:.1}% effectiveness\n",
            self.timestamp,
            self.uptime.as_secs_f64() / 3600.0,
            self.system_health * 100.0,
            self.performance.total_operations,
            self.performance.operations_per_second,
            self.performance.avg_response_time_micros,
            self.performance.memory_usage_bytes as f64 / 1_000_000.0,
            self.performance.peak_memory_bytes as f64 / 1_000_000.0,
            self.performance.cpu_utilization_percent,
            self.performance.error_rate * 100.0,
            self.optimization.nan_boxing.hit_rate * 100.0,
            self.optimization.nan_boxing.fallback_rate * 100.0,
            self.optimization.string_interning.hit_rate * 100.0,
            self.optimization.string_interning.usage_count,
            self.optimization.arena_allocation.memory_savings_bytes as f64 / 1_000_000.0,
            self.optimization.zero_copy_parsing.hit_rate * 100.0
        )
    }
}

/// Global production metrics instance
static GLOBAL_METRICS: std::sync::OnceLock<ProductionMetrics> = std::sync::OnceLock::new();

/// Get global production metrics instance
pub fn global_metrics() -> &'static ProductionMetrics {
    GLOBAL_METRICS.get_or_init(|| ProductionMetrics::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_metrics_creation() {
        let metrics = ProductionMetrics::new();
        let report = metrics.generate_report();
        assert!(report.system_health >= 0.0);
        assert!(report.system_health <= 1.0);
    }

    #[test]
    fn test_alert_generation() {
        let metrics = ProductionMetrics::new();
        let mut counters = PerformanceCounters::default();
        counters.memory_usage_bytes = 1000;
        
        let new_counters = PerformanceCounters {
            memory_usage_bytes: 1500,  // 50% increase
            ..counters.clone()
        };
        
        let alerts = metrics.update_performance(new_counters).unwrap();
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].severity, AlertSeverity::Warning);
    }

    #[test]
    fn test_system_health_calculation() {
        let metrics = ProductionMetrics::new();
        let counters = PerformanceCounters {
            error_rate: 0.005,  // 0.5% error rate
            memory_usage_bytes: 500_000_000,  // 500MB
            ..PerformanceCounters::default()
        };
        
        metrics.update_performance(counters).unwrap();
        let health = metrics.calculate_system_health();
        assert!(health > 0.95);  // Should be healthy
    }
}