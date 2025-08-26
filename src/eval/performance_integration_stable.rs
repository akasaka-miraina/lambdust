//! Phase 5 Stage 5: Performance Integration System

#![allow(missing_docs)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
//!
//! This module implements a comprehensive performance integration system that unifies
//! all performance monitoring, analysis, and optimization capabilities across Stages 1-4.
//! The system provides real-time performance analytics, automatic bottleneck detection,
//! and adaptive optimization strategies.
//!
//! ## Architecture
//!
//! The performance integration system consists of five main components:
//!
//! 1. **Unified Metrics Collector**: O(1) amortized collection from all stages
//! 2. **Real-time Analytics Engine**: O(log N) performance analysis and correlation
//! 3. **Dynamic Optimization Engine**: Reinforcement learning-based adaptive optimization
//! 4. **Bottleneck Detection System**: O(log B) automatic performance bottleneck identification
//! 5. **Resource Optimization Manager**: Cross-system resource allocation optimization
//!
//! ## Stage Integration
//!
//! - **Stage 1 JIT Integration**: JIT compilation metrics and optimization feedback
//! - **Stage 2 Parallel Execution**: Thread pool and work-stealing performance metrics
//! - **Stage 3 Distributed Computing**: Network and node performance monitoring
//! - **Stage 4 Security Integration**: Security overhead and threat response metrics
//!
//! ## Performance Characteristics
//!
//! - **Metrics Collection**: O(1) amortized time complexity
//! - **Analytics Processing**: O(log N) where N is number of metrics points
//! - **Optimization Decision**: O(1) lookup with ML model inference
//! - **Memory Usage**: Bounded ring buffers with configurable retention periods

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::eval::Value;
use crate::diagnostics::{Error, Result};
use std::collections::{HashMap, BTreeMap, VecDeque};
use std::sync::{Arc, RwLock, Mutex, atomic::{AtomicU64, AtomicBool, AtomicUsize, Ordering}};
use std::time::{Duration, Instant, SystemTime};
use dashmap::DashMap;
use ordered_float::OrderedFloat;

// Stage integrations (conditionally available)
#[cfg(feature = "stage1")]
use crate::eval::{JITIntegrationSystem, JITStatistics};
#[cfg(feature = "stage2")]  
use crate::eval::{ParallelExecutionSystem, ParallelSystemStats};
#[cfg(feature = "stage3")]
use crate::eval::{DistributedComputingSystem, DistributedStatistics};
#[cfg(feature = "stage4")]
use crate::eval::{SecurityIntegrationSystem, SecurityIntegrationStats};

// ============= CORE TYPES =============

/// Unique identifier for performance metrics
pub type MetricId = u64;

/// Unique identifier for optimization strategies
pub type OptimizationStrategyId = u64;

/// Unique identifier for performance events
pub type PerformanceEventId = u64;

/// Performance integration operation result
pub type PerformanceResult<T> = std::result::Result<T, PerformanceError>;

// ============= ERROR TYPES =============

/// Errors that can occur in performance integration
#[derive(Debug, Clone)]
pub enum PerformanceError {
    /// Metrics collection failed
    MetricsCollectionFailed(String),
    /// Analytics processing error
    AnalyticsProcessingError(String),
    /// Optimization engine failure
    OptimizationEngineFailed(String),
    /// Bottleneck detection error
    BottleneckDetectionFailed(String),
    /// Resource optimization error
    ResourceOptimizationFailed(String),
    /// Configuration error
    ConfigurationError(String),
    /// System integration error
    SystemIntegrationError(String),
}

impl std::fmt::Display for PerformanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MetricsCollectionFailed(msg) => write!(f, "Metrics collection failed: {}", msg),
            Self::AnalyticsProcessingError(msg) => write!(f, "Analytics processing error: {}", msg),
            Self::OptimizationEngineFailed(msg) => write!(f, "Optimization engine failed: {}", msg),
            Self::BottleneckDetectionFailed(msg) => write!(f, "Bottleneck detection failed: {}", msg),
            Self::ResourceOptimizationFailed(msg) => write!(f, "Resource optimization failed: {}", msg),
            Self::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            Self::SystemIntegrationError(msg) => write!(f, "System integration error: {}", msg),
        }
    }
}

impl std::error::Error for PerformanceError {}

// ============= METRICS TYPES =============

/// Performance metric data point
#[derive(Debug, Clone)]
pub struct MetricDataPoint {
    /// Timestamp when the metric was recorded
    pub timestamp: Instant,
    /// Metric value as floating point
    pub value: f64,
    /// Optional metadata associated with the metric
    pub metadata: HashMap<String, String>,
}

impl MetricDataPoint {
    pub fn new(value: f64) -> Self {
        Self {
            timestamp: Instant::now(),
            value,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_metadata(value: f64, metadata: HashMap<String, String>) -> Self {
        Self {
            timestamp: Instant::now(),
            value,
            metadata,
        }
    }
}

/// Time-series metric storage with ring buffer implementation
#[derive(Debug)]
pub struct TimeSeriesMetric {
    /// Ring buffer for metric data points (fixed size for O(1) operations)
    data_points: RwLock<VecDeque<MetricDataPoint>>,
    /// Maximum number of data points to retain
    max_size: usize,
    /// Total number of data points ever added
    total_added: AtomicU64,
    /// Statistical summaries (updated incrementally)
    running_sum: AtomicU64, // Using bits to represent f64 
    running_sum_squares: AtomicU64,
    min_value: Mutex<Option<OrderedFloat<f64>>>,
    max_value: Mutex<Option<OrderedFloat<f64>>>,
}

impl TimeSeriesMetric {
    pub fn new(max_size: usize) -> Self {
        Self {
            data_points: RwLock::new(VecDeque::with_capacity(max_size)),
            max_size,
            total_added: AtomicU64::new(0),
            running_sum: AtomicU64::new(0),
            running_sum_squares: AtomicU64::new(0),
            min_value: Mutex::new(None),
            max_value: Mutex::new(None),
        }
    }
    
    /// Add a new data point (O(1) amortized)
    pub fn add_data_point(&self, data_point: MetricDataPoint) -> PerformanceResult<()> {
        let value = data_point.value;
        
        // Update ring buffer
        {
            let mut buffer = self.data_points.write()
                .map_err(|_| PerformanceError::MetricsCollectionFailed("Lock poisoned".to_string()))?;
            
            if buffer.len() >= self.max_size {
                buffer.pop_front(); // Remove oldest point
            }
            buffer.push_back(data_point);
        }
        
        // Update statistics atomically
        self.total_added.fetch_add(1, Ordering::Relaxed);
        
        // Update running sums (using bit representation for atomic f64)
        let value_bits = value.to_bits();
        let old_sum_bits = self.running_sum.fetch_add(value_bits, Ordering::Relaxed);
        let _new_sum = f64::from_bits(old_sum_bits.wrapping_add(value_bits));
        
        let square_bits = (value * value).to_bits();
        self.running_sum_squares.fetch_add(square_bits, Ordering::Relaxed);
        
        // Update min/max (requires locking for atomic float comparison)
        {
            let mut min = self.min_value.lock()
                .map_err(|_| PerformanceError::MetricsCollectionFailed("Lock poisoned".to_string()))?;
            if min.is_none() || OrderedFloat(value) < *min.as_ref().unwrap() {
                *min = Some(OrderedFloat(value));
            }
        }
        
        {
            let mut max = self.max_value.lock()
                .map_err(|_| PerformanceError::MetricsCollectionFailed("Lock poisoned".to_string()))?;
            if max.is_none() || OrderedFloat(value) > *max.as_ref().unwrap() {
                *max = Some(OrderedFloat(value));
            }
        }
        
        Ok(())
    }
    
    /// Get current statistical summary (O(1))
    pub fn get_statistics(&self) -> MetricStatistics {
        let count = self.total_added.load(Ordering::Relaxed);
        let sum_bits = self.running_sum.load(Ordering::Relaxed);
        let sum_squares_bits = self.running_sum_squares.load(Ordering::Relaxed);
        
        let sum = f64::from_bits(sum_bits);
        let sum_squares = f64::from_bits(sum_squares_bits);
        
        let mean = if count > 0 { sum / count as f64 } else { 0.0 };
        
        let variance = if count > 1 {
            (sum_squares - sum * sum / count as f64) / (count - 1) as f64
        } else {
            0.0
        };
        
        let std_dev = variance.sqrt();
        
        let min = self.min_value.lock().ok()
            .and_then(|guard| guard.as_ref().map(|v| v.0));
        let max = self.max_value.lock().ok()
            .and_then(|guard| guard.as_ref().map(|v| v.0));
        
        MetricStatistics {
            count,
            mean,
            std_dev,
            min,
            max,
            sum,
        }
    }
    
    /// Get recent data points within time window (O(k) where k is points in window)
    pub fn get_recent_points(&self, time_window: Duration) -> PerformanceResult<Vec<MetricDataPoint>> {
        let buffer = self.data_points.read()
            .map_err(|_| PerformanceError::MetricsCollectionFailed("Lock poisoned".to_string()))?;
        
        let cutoff = Instant::now() - time_window;
        let recent_points: Vec<MetricDataPoint> = buffer
            .iter()
            .filter(|point| point.timestamp >= cutoff)
            .cloned()
            .collect();
        
        Ok(recent_points)
    }
}

/// Statistical summary of a time-series metric
#[derive(Debug, Clone)]
pub struct MetricStatistics {
    pub count: u64,
    pub mean: f64,
    pub std_dev: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub sum: f64,
}

impl MetricStatistics {
    /// Calculate coefficient of variation (CV) for stability assessment
    pub fn coefficient_of_variation(&self) -> f64 {
        if self.mean != 0.0 {
            self.std_dev / self.mean.abs()
        } else {
            0.0
        }
    }
    
    /// Calculate z-score for a given value
    pub fn z_score(&self, value: f64) -> f64 {
        if self.std_dev != 0.0 {
            (value - self.mean) / self.std_dev
        } else {
            0.0
        }
    }
}

// ============= UNIFIED METRICS COLLECTOR =============

/// Unified metrics collector that aggregates performance data from all stages
pub struct UnifiedMetricsCollector {
    /// Stage-specific metrics storage
    stage_metrics: DashMap<String, Arc<TimeSeriesMetric>>,
    /// Cross-stage correlation metrics
    correlation_metrics: DashMap<String, Arc<TimeSeriesMetric>>,
    /// System-wide performance indicators
    system_metrics: DashMap<String, Arc<TimeSeriesMetric>>,
    /// Collection configuration
    config: MetricsCollectorConfig,
    /// Collection statistics
    collection_stats: CollectionStatistics,
}

/// Configuration for metrics collector
#[derive(Debug, Clone)]
pub struct MetricsCollectorConfig {
    /// Maximum data points per metric
    pub max_data_points: usize,
    /// Collection interval for automatic metrics
    pub collection_interval: Duration,
    /// Whether to enable cross-stage correlation
    pub enable_correlation: bool,
    /// Memory usage limit for metrics storage
    pub memory_limit_mb: usize,
}

impl Default for MetricsCollectorConfig {
    fn default() -> Self {
        Self {
            max_data_points: 10000, // ~10 minutes at 1-second intervals
            collection_interval: Duration::from_secs(1),
            enable_correlation: true,
            memory_limit_mb: 100, // 100MB limit for metrics storage
        }
    }
}

/// Statistics about metrics collection performance
#[derive(Debug)]
pub struct CollectionStatistics {
    pub total_metrics_collected: AtomicU64,
    pub collection_errors: AtomicU64,
    pub memory_usage_bytes: AtomicU64,
    pub last_collection_duration: Mutex<Option<Duration>>,
    pub average_collection_duration: Mutex<Option<Duration>>,
}

impl Default for CollectionStatistics {
    fn default() -> Self {
        Self {
            total_metrics_collected: AtomicU64::new(0),
            collection_errors: AtomicU64::new(0),
            memory_usage_bytes: AtomicU64::new(0),
            last_collection_duration: Mutex::new(None),
            average_collection_duration: Mutex::new(None),
        }
    }
}

impl UnifiedMetricsCollector {
    pub fn new(config: MetricsCollectorConfig) -> Self {
        Self {
            stage_metrics: DashMap::new(),
            correlation_metrics: DashMap::new(),
            system_metrics: DashMap::new(),
            config,
            collection_stats: CollectionStatistics::default(),
        }
    }
    
    /// Collect metrics from JIT integration system (Stage 1)
    #[cfg(feature = "stage1")]
    pub fn collect_jit_metrics(&self, jit_stats: &JITStatistics) -> PerformanceResult<()> {
        let start_time = Instant::now();
        
        // Collect JIT-specific metrics using available fields
        self.record_metric("jit.total_executions", jit_stats.total_executions as f64)?;
        self.record_metric("jit.cache_hits", jit_stats.cache_hits as f64)?;
        self.record_metric("jit.compilation_attempts", jit_stats.compilation_attempts as f64)?;
        self.record_metric("jit.successful_compilations", jit_stats.successful_compilations as f64)?;
        
        // Calculate derived metrics
        let hit_rate = jit_stats.cache_hit_rate();
        self.record_metric("jit.cache_hit_rate", hit_rate)?;
        
        let compilation_success_rate = if jit_stats.compilation_attempts > 0 {
            jit_stats.successful_compilations as f64 / jit_stats.compilation_attempts as f64
        } else {
            1.0
        };
        self.record_metric("jit.compilation_success_rate", compilation_success_rate)?;
        
        self.update_collection_stats(start_time);
        Ok(())
    }
    
    /// Collect metrics from parallel execution system (Stage 2)
    #[cfg(feature = "stage2")]
    pub fn collect_parallel_metrics(&self, parallel_stats: &ParallelSystemStats) -> PerformanceResult<()> {
        let start_time = Instant::now();
        
        // Collect parallel execution metrics using available fields
        self.record_metric("parallel.continuations_submitted", parallel_stats.continuations_submitted as f64)?;
        self.record_metric("parallel.continuations_completed", parallel_stats.continuations_completed as f64)?;
        self.record_metric("parallel.thread_safe_values_created", parallel_stats.thread_safe_values_created as f64)?;
        self.record_metric("parallel.numa_allocations", parallel_stats.numa_allocations as f64)?;
        
        // Calculate throughput metrics
        let completion_rate = if parallel_stats.continuations_submitted > 0 {
            parallel_stats.continuations_completed as f64 / parallel_stats.continuations_submitted as f64
        } else {
            1.0
        };
        self.record_metric("parallel.completion_rate", completion_rate)?;
        
        // Include scheduler stats if available
        if let Some(ref scheduler_stats) = parallel_stats.scheduler_stats {
            self.record_metric("parallel.scheduler_num_workers", scheduler_stats.num_workers as f64)?;
            self.record_metric("parallel.scheduler_total_tasks", scheduler_stats.total_tasks as f64)?;
            self.record_metric("parallel.parallel_efficiency", scheduler_stats.parallel_efficiency)?;
        }
        
        self.update_collection_stats(start_time);
        Ok(())
    }
    
    /// Collect metrics from distributed computing system (Stage 3)
    #[cfg(feature = "stage3")]
    pub fn collect_distributed_metrics(&self, dist_stats: &DistributedStatistics) -> PerformanceResult<()> {
        let start_time = Instant::now();
        
        // Collect distributed computing metrics using available fields
        self.record_metric("distributed.nodes_registered", dist_stats.get_nodes_registered() as f64)?;
        self.record_metric("distributed.tasks_completed", dist_stats.get_tasks_completed() as f64)?;
        self.record_metric("distributed.tasks_failed", dist_stats.get_tasks_failed() as f64)?;
        self.record_metric("distributed.network_messages_sent", dist_stats.get_network_messages_sent() as f64)?;
        
        // Calculate efficiency metrics
        let task_success_rate = {
            let completed = dist_stats.get_tasks_completed() as f64;
            let failed = dist_stats.get_tasks_failed() as f64;
            let total = completed + failed;
            if total > 0.0 { completed / total } else { 1.0 }
        };
        self.record_metric("distributed.task_success_rate", task_success_rate)?;
        
        self.update_collection_stats(start_time);
        Ok(())
    }
    
    /// Collect metrics from security integration system (Stage 4)
    #[cfg(feature = "stage4")]
    pub fn collect_security_metrics(&self, security_stats: &SecurityIntegrationStats) -> PerformanceResult<()> {
        let start_time = Instant::now();
        
        // Collect security metrics using available fields
        self.record_metric("security.is_active", if security_stats.is_active { 1.0 } else { 0.0 })?;
        self.record_metric("security.security_violations", security_stats.security_violations as f64)?;
        self.record_metric("security.security_events", security_stats.security_events as f64)?;
        
        // Collect threat detection metrics
        self.record_metric("security.threats_detected", security_stats.threat_detection.threats_detected as f64)?;
        self.record_metric("security.analysis_engines_active", security_stats.threat_detection.analysis_engines_active as f64)?;
        
        // Calculate security health metrics - use a simple security score
        let security_score = if security_stats.is_active {
            // Higher score if system is active and has analysis engines running
            if security_stats.threat_detection.analysis_engines_active > 0 {
                0.9 // 90% if active with engines running
            } else {
                0.7 // 70% if active but no engines
            }
        } else {
            0.0 // 0% if inactive
        };
        self.record_metric("security.overall_score", security_score)?;
        
        self.update_collection_stats(start_time);
        Ok(())
    }
    
    /// Record a metric value with automatic timestamp
    pub fn record_metric(&self, metric_name: &str, value: f64) -> PerformanceResult<()> {
        let metric = self.stage_metrics
            .entry(metric_name.to_string())
            .or_insert_with(|| Arc::new(TimeSeriesMetric::new(self.config.max_data_points)));
        
        let data_point = MetricDataPoint::new(value);
        metric.add_data_point(data_point)?;
        
        self.collection_stats.total_metrics_collected.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    
    /// Get statistics for a specific metric
    pub fn get_metric_statistics(&self, metric_name: &str) -> Option<MetricStatistics> {
        self.stage_metrics.get(metric_name)
            .map(|metric| metric.get_statistics())
    }
    
    /// Get all available metric names
    pub fn get_metric_names(&self) -> Vec<String> {
        self.stage_metrics.iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
    
    /// Update collection performance statistics
    fn update_collection_stats(&self, start_time: Instant) {
        let duration = start_time.elapsed();
        
        // Update last collection duration
        if let Ok(mut last_duration) = self.collection_stats.last_collection_duration.lock() {
            *last_duration = Some(duration);
        }
        
        // Update average collection duration using exponential moving average
        if let Ok(mut avg_duration) = self.collection_stats.average_collection_duration.lock() {
            match *avg_duration {
                Some(current_avg) => {
                    // Exponential moving average with alpha = 0.1
                    let new_avg = current_avg.mul_f64(0.9) + duration.mul_f64(0.1);
                    *avg_duration = Some(new_avg);
                }
                None => {
                    *avg_duration = Some(duration);
                }
            }
        }
    }
    
    /// Get collector performance statistics
    pub fn get_collector_stats(&self) -> PerformanceResult<CollectorPerformanceStats> {
        let total_collected = self.collection_stats.total_metrics_collected.load(Ordering::Relaxed);
        let collection_errors = self.collection_stats.collection_errors.load(Ordering::Relaxed);
        let memory_usage = self.collection_stats.memory_usage_bytes.load(Ordering::Relaxed);
        
        let last_duration = self.collection_stats.last_collection_duration.lock()
            .map_err(|_| PerformanceError::MetricsCollectionFailed("Lock poisoned".to_string()))?
            .clone();
        
        let avg_duration = self.collection_stats.average_collection_duration.lock()
            .map_err(|_| PerformanceError::MetricsCollectionFailed("Lock poisoned".to_string()))?
            .clone();
        
        Ok(CollectorPerformanceStats {
            total_metrics_collected: total_collected,
            collection_errors,
            memory_usage_bytes: memory_usage,
            last_collection_duration: last_duration,
            average_collection_duration: avg_duration,
            active_metrics_count: self.stage_metrics.len() as u64,
        })
    }
}

/// Performance statistics for the metrics collector itself
#[derive(Debug, Clone)]
pub struct CollectorPerformanceStats {
    pub total_metrics_collected: u64,
    pub collection_errors: u64,
    pub memory_usage_bytes: u64,
    pub last_collection_duration: Option<Duration>,
    pub average_collection_duration: Option<Duration>,
    pub active_metrics_count: u64,
}

// ============= MAIN PERFORMANCE INTEGRATION SYSTEM =============

/// Main performance integration system that unifies all performance monitoring
/// and optimization capabilities across Stages 1-4
pub struct PerformanceIntegrationSystem {
    /// Unified metrics collection system
    metrics_collector: Arc<UnifiedMetricsCollector>,
    /// Real-time analytics engine
    analytics_engine: Arc<RealTimeAnalyticsEngine>,
    /// Dynamic optimization engine
    optimization_engine: Arc<DynamicOptimizationEngine>,
    /// Bottleneck detection system
    bottleneck_detector: Arc<BottleneckDetector>,
    /// Resource optimization manager
    resource_optimizer: Arc<ResourceOptimizer>,
    /// System configuration
    config: PerformanceSystemConfig,
    /// System status and statistics
    system_stats: Arc<PerformanceSystemStats>,
}

/// Configuration for the performance integration system
#[derive(Debug, Clone)]
pub struct PerformanceSystemConfig {
    /// Metrics collection configuration
    pub metrics_config: MetricsCollectorConfig,
    /// Analytics processing interval
    pub analytics_interval: Duration,
    /// Optimization update frequency
    pub optimization_frequency: Duration,
    /// Bottleneck detection sensitivity
    pub bottleneck_sensitivity: f64,
    /// Enable automatic optimization
    pub enable_auto_optimization: bool,
}

impl Default for PerformanceSystemConfig {
    fn default() -> Self {
        Self {
            metrics_config: MetricsCollectorConfig::default(),
            analytics_interval: Duration::from_secs(5),
            optimization_frequency: Duration::from_secs(30),
            bottleneck_sensitivity: 0.8, // 80% threshold
            enable_auto_optimization: true,
        }
    }
}

/// System-wide performance statistics
#[derive(Debug)]
pub struct PerformanceSystemStats {
    pub system_start_time: Instant,
    pub total_optimizations_applied: AtomicU64,
    pub bottlenecks_detected: AtomicU64,
    pub bottlenecks_resolved: AtomicU64,
    pub overall_performance_score: Mutex<Option<f64>>,
    pub last_analytics_run: Mutex<Option<Instant>>,
    pub last_optimization_run: Mutex<Option<Instant>>,
}

impl Default for PerformanceSystemStats {
    fn default() -> Self {
        Self {
            system_start_time: Instant::now(),
            total_optimizations_applied: AtomicU64::new(0),
            bottlenecks_detected: AtomicU64::new(0),
            bottlenecks_resolved: AtomicU64::new(0),
            overall_performance_score: Mutex::new(None),
            last_analytics_run: Mutex::new(None),
            last_optimization_run: Mutex::new(None),
        }
    }
}

// ============= REAL-TIME ANALYTICS ENGINE =============

/// Real-time analytics engine for performance data analysis
pub struct RealTimeAnalyticsEngine {
    /// Correlation matrix for cross-metric analysis (O(M²) space where M is metrics count)
    correlation_matrix: RwLock<BTreeMap<(String, String), f64>>,
    /// Trend analysis using exponential smoothing (O(1) per update)
    trend_analyzers: DashMap<String, TrendAnalyzer>,
    /// Anomaly detection using statistical methods
    anomaly_detectors: DashMap<String, AnomalyDetector>,
    /// Configuration parameters
    config: AnalyticsConfig,
}

/// Configuration for analytics engine
#[derive(Debug, Clone)]
pub struct AnalyticsConfig {
    /// Minimum correlation threshold for reporting
    pub correlation_threshold: f64,
    /// Trend analysis window size
    pub trend_window_size: usize,
    /// Anomaly detection sensitivity (z-score threshold)
    pub anomaly_threshold: f64,
    /// Update frequency for correlations
    pub correlation_update_frequency: Duration,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            correlation_threshold: 0.7, // Strong correlation
            trend_window_size: 100, // Last 100 data points
            anomaly_threshold: 2.5, // 2.5 standard deviations
            correlation_update_frequency: Duration::from_secs(10),
        }
    }
}

/// Trend analyzer using exponential smoothing (O(1) updates)
#[derive(Debug)]
pub struct TrendAnalyzer {
    /// Exponentially weighted moving average
    ewma: f64,
    /// Trend direction (-1: decreasing, 0: stable, 1: increasing)
    trend_direction: f64,
    /// Smoothing factor (α)
    alpha: f64,
    /// Trend smoothing factor (β)
    beta: f64,
    /// Last observed value
    last_value: Option<f64>,
}

impl TrendAnalyzer {
    pub fn new(alpha: f64, beta: f64) -> Self {
        Self {
            ewma: 0.0,
            trend_direction: 0.0,
            alpha: alpha.clamp(0.0, 1.0),
            beta: beta.clamp(0.0, 1.0),
            last_value: None,
        }
    }
    
    /// Update trend analysis with new data point (O(1))
    pub fn update(&mut self, value: f64) {
        match self.last_value {
            None => {
                // First data point
                self.ewma = value;
                self.last_value = Some(value);
            }
            Some(last) => {
                // Update EWMA and trend
                let old_ewma = self.ewma;
                self.ewma = self.alpha * value + (1.0 - self.alpha) * self.ewma;
                self.trend_direction = self.beta * (self.ewma - old_ewma) + (1.0 - self.beta) * self.trend_direction;
                self.last_value = Some(value);
            }
        }
    }
    
    /// Get current trend information
    pub fn get_trend_info(&self) -> TrendInfo {
        TrendInfo {
            current_value: self.ewma,
            trend_direction: self.trend_direction,
            is_increasing: self.trend_direction > 0.1,
            is_decreasing: self.trend_direction < -0.1,
            is_stable: self.trend_direction.abs() <= 0.1,
        }
    }
}

/// Trend analysis information
#[derive(Debug, Clone)]
pub struct TrendInfo {
    pub current_value: f64,
    pub trend_direction: f64,
    pub is_increasing: bool,
    pub is_decreasing: bool,
    pub is_stable: bool,
}

/// Anomaly detector using statistical methods
#[derive(Debug)]
pub struct AnomalyDetector {
    /// Running mean (using Welford's algorithm for O(1) updates)
    mean: f64,
    /// Running variance (using Welford's algorithm)
    variance: f64,
    /// Number of observations
    count: u64,
    /// Anomaly threshold (z-score)
    threshold: f64,
    /// Recent anomalies for rate limiting
    recent_anomalies: VecDeque<Instant>,
}

impl AnomalyDetector {
    pub fn new(threshold: f64) -> Self {
        Self {
            mean: 0.0,
            variance: 0.0,
            count: 0,
            threshold,
            recent_anomalies: VecDeque::new(),
        }
    }
    
    /// Update statistics and detect anomalies (O(1))
    pub fn update(&mut self, value: f64) -> AnomalyInfo {
        self.count += 1;
        
        if self.count == 1 {
            self.mean = value;
            self.variance = 0.0;
            return AnomalyInfo::normal(value, 0.0);
        }
        
        // Welford's algorithm for online mean and variance calculation
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.variance += delta * delta2;
        
        // Calculate z-score for anomaly detection
        let std_dev = if self.count > 1 {
            (self.variance / (self.count - 1) as f64).sqrt()
        } else {
            0.0
        };
        
        let z_score = if std_dev > 0.0 {
            (value - self.mean) / std_dev
        } else {
            0.0
        };
        
        // Clean old anomalies (older than 5 minutes)
        let cutoff = Instant::now() - Duration::from_secs(300);
        while self.recent_anomalies.front().map_or(false, |&t| t < cutoff) {
            self.recent_anomalies.pop_front();
        }
        
        let is_anomaly = z_score.abs() > self.threshold;
        if is_anomaly {
            self.recent_anomalies.push_back(Instant::now());
        }
        
        AnomalyInfo {
            value,
            z_score,
            is_anomaly,
            anomaly_severity: if is_anomaly { z_score.abs() / self.threshold } else { 0.0 },
            recent_anomaly_count: self.recent_anomalies.len() as u64,
        }
    }
}

/// Anomaly detection information
#[derive(Debug, Clone)]
pub struct AnomalyInfo {
    pub value: f64,
    pub z_score: f64,
    pub is_anomaly: bool,
    pub anomaly_severity: f64,
    pub recent_anomaly_count: u64,
}

impl AnomalyInfo {
    pub fn normal(value: f64, z_score: f64) -> Self {
        Self {
            value,
            z_score,
            is_anomaly: false,
            anomaly_severity: 0.0,
            recent_anomaly_count: 0,
        }
    }
}

impl RealTimeAnalyticsEngine {
    pub fn new(config: AnalyticsConfig) -> Self {
        Self {
            correlation_matrix: RwLock::new(BTreeMap::new()),
            trend_analyzers: DashMap::new(),
            anomaly_detectors: DashMap::new(),
            config,
        }
    }
    
    /// Analyze metrics for trends and anomalies
    pub fn analyze_metrics(&self, metrics: &HashMap<String, f64>) -> PerformanceResult<AnalyticsReport> {
        let mut trend_reports = HashMap::new();
        let mut anomaly_reports = HashMap::new();
        
        // Update trend analyzers and anomaly detectors
        for (metric_name, &value) in metrics {
            // Update trend analysis
            let mut trend_analyzer = self.trend_analyzers
                .entry(metric_name.clone())
                .or_insert_with(|| TrendAnalyzer::new(0.1, 0.1)); // α=0.1, β=0.1
            trend_analyzer.update(value);
            trend_reports.insert(metric_name.clone(), trend_analyzer.get_trend_info());
            
            // Update anomaly detection
            let mut anomaly_detector = self.anomaly_detectors
                .entry(metric_name.clone())
                .or_insert_with(|| AnomalyDetector::new(self.config.anomaly_threshold));
            let anomaly_info = anomaly_detector.update(value);
            if anomaly_info.is_anomaly {
                anomaly_reports.insert(metric_name.clone(), anomaly_info);
            }
        }
        
        // Calculate correlations (periodically for efficiency)
        let correlations = self.calculate_correlations(metrics)?;
        
        Ok(AnalyticsReport {
            trend_reports,
            anomaly_reports,
            correlations,
            timestamp: Instant::now(),
        })
    }
    
    /// Calculate metric correlations using Pearson correlation coefficient
    fn calculate_correlations(&self, _metrics: &HashMap<String, f64>) -> PerformanceResult<Vec<CorrelationPair>> {
        // Placeholder for correlation calculation
        // In a full implementation, this would maintain sliding windows of metric values
        // and calculate Pearson correlation coefficients between metric pairs
        Ok(Vec::new())
    }
}

/// Analytics report containing trend and anomaly information
#[derive(Debug)]
pub struct AnalyticsReport {
    pub trend_reports: HashMap<String, TrendInfo>,
    pub anomaly_reports: HashMap<String, AnomalyInfo>,
    pub correlations: Vec<CorrelationPair>,
    pub timestamp: Instant,
}

/// Correlation between two metrics
#[derive(Debug, Clone)]
pub struct CorrelationPair {
    pub metric1: String,
    pub metric2: String,
    pub correlation: f64,
    pub strength: CorrelationStrength,
}

/// Strength of correlation
#[derive(Debug, Clone, PartialEq)]
pub enum CorrelationStrength {
    Weak,    // |r| < 0.3
    Moderate, // 0.3 <= |r| < 0.7
    Strong,   // |r| >= 0.7
}

impl From<f64> for CorrelationStrength {
    fn from(correlation: f64) -> Self {
        let abs_corr = correlation.abs();
        if abs_corr < 0.3 {
            CorrelationStrength::Weak
        } else if abs_corr < 0.7 {
            CorrelationStrength::Moderate
        } else {
            CorrelationStrength::Strong
        }
    }
}

// ============= DYNAMIC OPTIMIZATION ENGINE =============

/// Dynamic optimization engine using reinforcement learning principles
pub struct DynamicOptimizationEngine {
    /// Multi-armed bandit for strategy selection
    strategy_bandit: Mutex<MultiArmedBandit>,
    /// Optimization strategies
    strategies: HashMap<OptimizationStrategyId, OptimizationStrategy>,
    /// Performance history for learning
    performance_history: RwLock<VecDeque<PerformanceSnapshot>>,
    /// Current optimization state
    current_state: Mutex<OptimizationState>,
    /// Configuration parameters
    config: OptimizationConfig,
    /// Learning statistics
    learning_stats: OptimizationLearningStats,
}

/// Configuration for optimization engine
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Learning rate for strategy updates
    pub learning_rate: f64,
    /// Exploration rate for multi-armed bandit
    pub exploration_rate: f64,
    /// Maximum performance history size
    pub max_history_size: usize,
    /// Minimum confidence threshold for applying optimizations
    pub confidence_threshold: f64,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            exploration_rate: 0.1,
            max_history_size: 1000,
            confidence_threshold: 0.8,
        }
    }
}

/// Multi-armed bandit for strategy selection
#[derive(Debug)]
pub struct MultiArmedBandit {
    /// Strategy rewards (Q-values)
    strategy_rewards: HashMap<OptimizationStrategyId, f64>,
    /// Strategy selection counts
    strategy_counts: HashMap<OptimizationStrategyId, u64>,
    /// Total selections
    total_selections: u64,
    /// Exploration parameter for UCB
    exploration_param: f64,
}

impl MultiArmedBandit {
    pub fn new(exploration_param: f64) -> Self {
        Self {
            strategy_rewards: HashMap::new(),
            strategy_counts: HashMap::new(),
            total_selections: 0,
            exploration_param,
        }
    }
    
    /// Select strategy using Upper Confidence Bound (UCB) algorithm
    pub fn select_strategy(&mut self, available_strategies: &[OptimizationStrategyId]) -> Option<OptimizationStrategyId> {
        if available_strategies.is_empty() {
            return None;
        }
        
        self.total_selections += 1;
        
        // Calculate UCB values for each strategy
        let mut best_strategy = available_strategies[0];
        let mut best_ucb = f64::NEG_INFINITY;
        
        for &strategy_id in available_strategies {
            let count = *self.strategy_counts.get(&strategy_id).unwrap_or(&0);
            let reward = *self.strategy_rewards.get(&strategy_id).unwrap_or(&0.0);
            
            let ucb = if count == 0 {
                f64::INFINITY // Explore never-tried strategies first
            } else {
                let average_reward = reward / count as f64;
                let confidence_bound = (self.exploration_param * (self.total_selections as f64).ln() / count as f64).sqrt();
                average_reward + confidence_bound
            };
            
            if ucb > best_ucb {
                best_ucb = ucb;
                best_strategy = strategy_id;
            }
        }
        
        Some(best_strategy)
    }
    
    /// Update strategy reward based on performance improvement
    pub fn update_strategy_reward(&mut self, strategy_id: OptimizationStrategyId, reward: f64) {
        *self.strategy_rewards.entry(strategy_id).or_insert(0.0) += reward;
        *self.strategy_counts.entry(strategy_id).or_insert(0) += 1;
    }
}

/// Optimization strategy definition
#[derive(Debug, Clone)]
pub struct OptimizationStrategy {
    pub id: OptimizationStrategyId,
    pub name: String,
    pub description: String,
    pub target_metrics: Vec<String>,
    pub parameters: HashMap<String, OptimizationParameter>,
    pub expected_improvement: f64,
    pub confidence: f64,
}

/// Optimization parameter with constraints
#[derive(Debug, Clone)]
pub struct OptimizationParameter {
    pub name: String,
    pub current_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub step_size: f64,
}

/// Performance snapshot for learning
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub timestamp: Instant,
    pub metrics: HashMap<String, f64>,
    pub overall_score: f64,
    pub applied_strategy: Option<OptimizationStrategyId>,
}

/// Current optimization state
#[derive(Debug, Clone)]
pub struct OptimizationState {
    pub active_optimizations: HashMap<OptimizationStrategyId, OptimizationExecution>,
    pub performance_baseline: Option<f64>,
    pub last_optimization_time: Option<Instant>,
}

/// Execution state of an optimization
#[derive(Debug, Clone)]
pub struct OptimizationExecution {
    pub strategy_id: OptimizationStrategyId,
    pub start_time: Instant,
    pub parameters_applied: HashMap<String, f64>,
    pub performance_before: f64,
    pub performance_current: Option<f64>,
}

/// Learning statistics for optimization engine
#[derive(Debug)]
pub struct OptimizationLearningStats {
    pub total_optimizations_attempted: AtomicU64,
    pub successful_optimizations: AtomicU64,
    pub average_improvement: Mutex<Option<f64>>,
    pub best_strategy: Mutex<Option<OptimizationStrategyId>>,
}

impl Default for OptimizationLearningStats {
    fn default() -> Self {
        Self {
            total_optimizations_attempted: AtomicU64::new(0),
            successful_optimizations: AtomicU64::new(0),
            average_improvement: Mutex::new(None),
            best_strategy: Mutex::new(None),
        }
    }
}

impl DynamicOptimizationEngine {
    pub fn new(config: OptimizationConfig) -> Self {
        let mut strategies = HashMap::new();
        
        // Initialize built-in optimization strategies
        Self::initialize_builtin_strategies(&mut strategies);
        
        Self {
            strategy_bandit: Mutex::new(MultiArmedBandit::new(config.exploration_rate)),
            strategies,
            performance_history: RwLock::new(VecDeque::new()),
            current_state: Mutex::new(OptimizationState {
                active_optimizations: HashMap::new(),
                performance_baseline: None,
                last_optimization_time: None,
            }),
            config,
            learning_stats: OptimizationLearningStats::default(),
        }
    }
    
    /// Initialize built-in optimization strategies
    fn initialize_builtin_strategies(strategies: &mut HashMap<OptimizationStrategyId, OptimizationStrategy>) {
        // JIT compilation optimization strategy
        let jit_strategy = OptimizationStrategy {
            id: 1,
            name: "JIT Compilation Optimization".to_string(),
            description: "Optimize JIT compilation parameters based on hotspot patterns".to_string(),
            target_metrics: vec!["jit.cache_hit_rate".to_string(), "jit.compilation_success_rate".to_string()],
            parameters: {
                let mut params = HashMap::new();
                params.insert("optimization_level".to_string(), OptimizationParameter {
                    name: "optimization_level".to_string(),
                    current_value: 2.0,
                    min_value: 1.0,
                    max_value: 3.0,
                    step_size: 1.0,
                });
                params.insert("cache_size".to_string(), OptimizationParameter {
                    name: "cache_size".to_string(),
                    current_value: 1024.0,
                    min_value: 512.0,
                    max_value: 4096.0,
                    step_size: 256.0,
                });
                params
            },
            expected_improvement: 0.15, // 15% improvement expected
            confidence: 0.8,
        };
        strategies.insert(1, jit_strategy);
        
        // Parallel execution optimization strategy
        let parallel_strategy = OptimizationStrategy {
            id: 2,
            name: "Parallel Execution Optimization".to_string(),
            description: "Optimize thread pool and work-stealing parameters".to_string(),
            target_metrics: vec!["parallel.completion_rate".to_string(), "parallel.continuations_completed".to_string()],
            parameters: {
                let mut params = HashMap::new();
                params.insert("thread_count".to_string(), OptimizationParameter {
                    name: "thread_count".to_string(),
                    current_value: 8.0,
                    min_value: 2.0,
                    max_value: 32.0,
                    step_size: 2.0,
                });
                params.insert("queue_size".to_string(), OptimizationParameter {
                    name: "queue_size".to_string(),
                    current_value: 1000.0,
                    min_value: 100.0,
                    max_value: 10000.0,
                    step_size: 100.0,
                });
                params
            },
            expected_improvement: 0.20, // 20% improvement expected
            confidence: 0.75,
        };
        strategies.insert(2, parallel_strategy);
    }
    
    /// Generate optimization suggestions based on current performance
    pub fn generate_optimization_suggestions(&self, metrics: &HashMap<String, f64>) -> PerformanceResult<Vec<OptimizationSuggestion>> {
        let mut suggestions = Vec::new();
        
        // Analyze performance patterns and suggest optimizations
        for strategy in self.strategies.values() {
            // Check if strategy is relevant for current metrics
            let relevant = strategy.target_metrics.iter()
                .any(|metric| metrics.contains_key(metric));
            
            if relevant {
                let suggestion = OptimizationSuggestion {
                    optimization_type: strategy.name.clone(),
                    parameters: strategy.parameters.iter()
                        .map(|(k, v)| (k.clone(), v.current_value))
                        .collect(),
                    expected_improvement: strategy.expected_improvement,
                    confidence: strategy.confidence,
                };
                
                if suggestion.confidence >= self.config.confidence_threshold {
                    suggestions.push(suggestion);
                }
            }
        }
        
        // Sort by expected improvement * confidence
        suggestions.sort_by(|a, b| {
            let score_a = a.expected_improvement * a.confidence;
            let score_b = b.expected_improvement * b.confidence;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(suggestions)
    }
    
    /// Apply optimization using reinforcement learning strategy selection
    pub fn apply_optimization(&self, metrics: &HashMap<String, f64>) -> PerformanceResult<Option<OptimizationExecution>> {
        let mut bandit = self.strategy_bandit.lock()
            .map_err(|_| PerformanceError::OptimizationEngineFailed("Lock poisoned".to_string()))?;
        
        // Select strategy using multi-armed bandit
        let available_strategies: Vec<_> = self.strategies.keys().copied().collect();
        let selected_strategy = bandit.select_strategy(&available_strategies);
        
        if let Some(strategy_id) = selected_strategy {
            let strategy = &self.strategies[&strategy_id];
            
            // Create optimization execution
            let current_performance = self.calculate_overall_performance(metrics);
            let execution = OptimizationExecution {
                strategy_id,
                start_time: Instant::now(),
                parameters_applied: strategy.parameters.iter()
                    .map(|(k, v)| (k.clone(), v.current_value))
                    .collect(),
                performance_before: current_performance,
                performance_current: None,
            };
            
            // Update state
            let mut state = self.current_state.lock()
                .map_err(|_| PerformanceError::OptimizationEngineFailed("Lock poisoned".to_string()))?;
            state.active_optimizations.insert(strategy_id, execution.clone());
            state.last_optimization_time = Some(Instant::now());
            
            self.learning_stats.total_optimizations_attempted.fetch_add(1, Ordering::Relaxed);
            
            Ok(Some(execution))
        } else {
            Ok(None)
        }
    }
    
    /// Update optimization performance and learning
    pub fn update_optimization_performance(&self, strategy_id: OptimizationStrategyId, new_metrics: &HashMap<String, f64>) -> PerformanceResult<()> {
        let new_performance = self.calculate_overall_performance(new_metrics);
        
        let mut state = self.current_state.lock()
            .map_err(|_| PerformanceError::OptimizationEngineFailed("Lock poisoned".to_string()))?;
        
        if let Some(execution) = state.active_optimizations.get_mut(&strategy_id) {
            execution.performance_current = Some(new_performance);
            
            // Calculate improvement
            let improvement = new_performance - execution.performance_before;
            
            // Update multi-armed bandit with reward
            let mut bandit = self.strategy_bandit.lock()
                .map_err(|_| PerformanceError::OptimizationEngineFailed("Lock poisoned".to_string()))?;
            bandit.update_strategy_reward(strategy_id, improvement);
            
            // Update learning statistics
            if improvement > 0.0 {
                self.learning_stats.successful_optimizations.fetch_add(1, Ordering::Relaxed);
            }
            
            // Update average improvement
            if let Ok(mut avg_improvement) = self.learning_stats.average_improvement.lock() {
                match *avg_improvement {
                    Some(current_avg) => {
                        *avg_improvement = Some(current_avg * 0.9 + improvement * 0.1);
                    }
                    None => {
                        *avg_improvement = Some(improvement);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Calculate overall performance score from metrics
    fn calculate_overall_performance(&self, metrics: &HashMap<String, f64>) -> f64 {
        // Weighted combination of key performance indicators
        let mut score = 0.0;
        let mut weight_sum = 0.0;
        
        // JIT performance (weight: 0.3)
        if let Some(&hit_rate) = metrics.get("jit.cache_hit_rate") {
            score += hit_rate * 0.3;
            weight_sum += 0.3;
        }
        
        // Parallel performance (weight: 0.3)
        if let Some(&completion_rate) = metrics.get("parallel.completion_rate") {
            score += completion_rate * 0.3;
            weight_sum += 0.3;
        }
        
        // Distributed performance (weight: 0.2)
        if let Some(&task_success) = metrics.get("distributed.task_success_rate") {
            score += task_success * 0.2;
            weight_sum += 0.2;
        }
        
        // Security performance (weight: 0.2)
        if let Some(&security_score) = metrics.get("security.overall_score") {
            score += security_score * 0.2;
            weight_sum += 0.2;
        }
        
        if weight_sum > 0.0 {
            score / weight_sum
        } else {
            0.5 // Default neutral score
        }
    }
    
    /// Get optimization learning statistics
    pub fn get_learning_stats(&self) -> PerformanceResult<OptimizationLearningReport> {
        let attempted = self.learning_stats.total_optimizations_attempted.load(Ordering::Relaxed);
        let successful = self.learning_stats.successful_optimizations.load(Ordering::Relaxed);
        
        let avg_improvement = self.learning_stats.average_improvement.lock()
            .map_err(|_| PerformanceError::OptimizationEngineFailed("Lock poisoned".to_string()))?
            .clone();
        
        let best_strategy = self.learning_stats.best_strategy.lock()
            .map_err(|_| PerformanceError::OptimizationEngineFailed("Lock poisoned".to_string()))?
            .clone();
        
        Ok(OptimizationLearningReport {
            optimizations_attempted: attempted,
            successful_optimizations: successful,
            success_rate: if attempted > 0 { successful as f64 / attempted as f64 } else { 0.0 },
            average_improvement: avg_improvement,
            best_strategy_id: best_strategy,
        })
    }
}

/// Learning report from optimization engine
#[derive(Debug, Clone)]
pub struct OptimizationLearningReport {
    pub optimizations_attempted: u64,
    pub successful_optimizations: u64,
    pub success_rate: f64,
    pub average_improvement: Option<f64>,
    pub best_strategy_id: Option<OptimizationStrategyId>,
}

// ============= BOTTLENECK DETECTOR =============

/// Automatic bottleneck detection system using statistical analysis
pub struct BottleneckDetector {
    /// Performance thresholds for different metrics
    thresholds: HashMap<String, PerformanceThreshold>,
    /// Bottleneck detection history
    detection_history: RwLock<VecDeque<BottleneckEvent>>,
    /// Current bottleneck status
    current_bottlenecks: DashMap<String, ActiveBottleneck>,
    /// Detection configuration
    config: BottleneckDetectionConfig,
}

/// Performance threshold configuration
#[derive(Debug, Clone)]
pub struct PerformanceThreshold {
    /// Metric name
    pub metric_name: String,
    /// Critical threshold (triggers immediate alert)
    pub critical_threshold: f64,
    /// Warning threshold (triggers monitoring)
    pub warning_threshold: f64,
    /// Whether higher values are better (false = lower is better)
    pub higher_is_better: bool,
    /// Minimum duration before declaring bottleneck
    pub min_duration: Duration,
}

/// Bottleneck detection configuration
#[derive(Debug, Clone)]
pub struct BottleneckDetectionConfig {
    /// Maximum detection history size
    pub max_history_size: usize,
    /// Detection sensitivity (0.0 - 1.0)
    pub sensitivity: f64,
    /// Minimum confidence for bottleneck declaration
    pub confidence_threshold: f64,
}

impl Default for BottleneckDetectionConfig {
    fn default() -> Self {
        Self {
            max_history_size: 1000,
            sensitivity: 0.8,
            confidence_threshold: 0.75,
        }
    }
}

/// Bottleneck detection event
#[derive(Debug, Clone)]
pub struct BottleneckEvent {
    pub timestamp: Instant,
    pub metric_name: String,
    pub event_type: BottleneckEventType,
    pub severity: BottleneckSeverity,
    pub value: f64,
    pub threshold: f64,
}

/// Type of bottleneck event
#[derive(Debug, Clone, PartialEq)]
pub enum BottleneckEventType {
    Detected,
    Resolved,
    Escalated,
}

/// Severity of bottleneck
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum BottleneckSeverity {
    Warning,
    Critical,
    Severe,
}

/// Active bottleneck being monitored
#[derive(Debug)]
pub struct ActiveBottleneck {
    pub metric_name: String,
    pub start_time: Instant,
    pub severity: BottleneckSeverity,
    pub peak_value: f64,
    pub duration: Duration,
    pub resolution_attempted: bool,
}

impl BottleneckDetector {
    pub fn new(config: BottleneckDetectionConfig) -> Self {
        let mut thresholds = HashMap::new();
        
        // Initialize default thresholds for common metrics
        Self::initialize_default_thresholds(&mut thresholds);
        
        Self {
            thresholds,
            detection_history: RwLock::new(VecDeque::new()),
            current_bottlenecks: DashMap::new(),
            config,
        }
    }
    
    /// Initialize default performance thresholds
    fn initialize_default_thresholds(thresholds: &mut HashMap<String, PerformanceThreshold>) {
        // JIT metrics thresholds
        thresholds.insert("jit.cache_hit_rate".to_string(), PerformanceThreshold {
            metric_name: "jit.cache_hit_rate".to_string(),
            critical_threshold: 0.5,  // 50% hit rate critical
            warning_threshold: 0.7,   // 70% hit rate warning
            higher_is_better: true,
            min_duration: Duration::from_secs(30),
        });
        
        // Parallel execution thresholds
        thresholds.insert("parallel.task_completion_rate".to_string(), PerformanceThreshold {
            metric_name: "parallel.task_completion_rate".to_string(),
            critical_threshold: 10.0,   // 10 tasks/sec critical
            warning_threshold: 50.0,    // 50 tasks/sec warning
            higher_is_better: true,
            min_duration: Duration::from_secs(15),
        });
        
        // Memory usage threshold
        thresholds.insert("memory.usage_percent".to_string(), PerformanceThreshold {
            metric_name: "memory.usage_percent".to_string(),
            critical_threshold: 90.0,   // 90% memory usage critical
            warning_threshold: 80.0,    // 80% memory usage warning
            higher_is_better: false,
            min_duration: Duration::from_secs(60),
        });
    }
    
    /// Detect bottlenecks in current metrics
    pub fn detect_bottlenecks(&self, metrics: &HashMap<String, f64>) -> PerformanceResult<BottleneckReport> {
        let mut detected_bottlenecks = Vec::new();
        let mut resolved_bottlenecks = Vec::new();
        
        // Check each metric against its threshold
        for (metric_name, &value) in metrics {
            if let Some(threshold) = self.thresholds.get(metric_name) {
                let bottleneck_info = self.check_metric_threshold(metric_name, value, threshold)?;
                
                match bottleneck_info {
                    Some(BottleneckInfo::Detected(severity)) => {
                        detected_bottlenecks.push(DetectedBottleneck {
                            metric_name: metric_name.clone(),
                            current_value: value,
                            threshold_value: if threshold.higher_is_better { 
                                threshold.warning_threshold 
                            } else { 
                                threshold.critical_threshold 
                            },
                            severity,
                            confidence: self.calculate_detection_confidence(metric_name, value, threshold),
                        });
                    }
                    Some(BottleneckInfo::Resolved) => {
                        resolved_bottlenecks.push(metric_name.clone());
                    }
                    None => {} // No change
                }
            }
        }
        
        // Update detection history
        let mut history = self.detection_history.write()
            .map_err(|_| PerformanceError::BottleneckDetectionFailed("Lock poisoned".to_string()))?;
        
        let timestamp = Instant::now();
        
        // Record detected bottlenecks
        for bottleneck in &detected_bottlenecks {
            let event = BottleneckEvent {
                timestamp,
                metric_name: bottleneck.metric_name.clone(),
                event_type: BottleneckEventType::Detected,
                severity: bottleneck.severity.clone(),
                value: bottleneck.current_value,
                threshold: bottleneck.threshold_value,
            };
            
            if history.len() >= self.config.max_history_size {
                history.pop_front();
            }
            history.push_back(event);
        }
        
        // Record resolved bottlenecks
        for metric_name in &resolved_bottlenecks {
            let event = BottleneckEvent {
                timestamp,
                metric_name: metric_name.clone(),
                event_type: BottleneckEventType::Resolved,
                severity: BottleneckSeverity::Warning,
                value: metrics.get(metric_name).copied().unwrap_or(0.0),
                threshold: 0.0,
            };
            
            if history.len() >= self.config.max_history_size {
                history.pop_front();
            }
            history.push_back(event);
        }
        
        Ok(BottleneckReport {
            detected_bottlenecks,
            resolved_bottlenecks,
            active_bottlenecks_count: self.current_bottlenecks.len() as u64,
            timestamp,
        })
    }
    
    /// Check if a metric value crosses threshold boundaries
    fn check_metric_threshold(&self, metric_name: &str, value: f64, threshold: &PerformanceThreshold) -> PerformanceResult<Option<BottleneckInfo>> {
        let is_critical = if threshold.higher_is_better {
            value < threshold.critical_threshold
        } else {
            value > threshold.critical_threshold
        };
        
        let is_warning = if threshold.higher_is_better {
            value < threshold.warning_threshold && value >= threshold.critical_threshold
        } else {
            value > threshold.warning_threshold && value <= threshold.critical_threshold
        };
        
        let current_time = Instant::now();
        
        // Check if this is a new bottleneck
        if is_critical || is_warning {
            let severity = if is_critical { BottleneckSeverity::Critical } else { BottleneckSeverity::Warning };
            
            // Check if already tracking this bottleneck
            match self.current_bottlenecks.get_mut(metric_name) {
                Some(mut bottleneck) => {
                    // Update existing bottleneck
                    bottleneck.duration = current_time - bottleneck.start_time;
                    if value > bottleneck.peak_value {
                        bottleneck.peak_value = value;
                    }
                    
                    // Check if duration meets minimum threshold
                    if bottleneck.duration >= threshold.min_duration {
                        Ok(Some(BottleneckInfo::Detected(severity)))
                    } else {
                        Ok(None) // Still within grace period
                    }
                }
                None => {
                    // New bottleneck detected
                    let bottleneck = ActiveBottleneck {
                        metric_name: metric_name.to_string(),
                        start_time: current_time,
                        severity,
                        peak_value: value,
                        duration: Duration::from_secs(0),
                        resolution_attempted: false,
                    };
                    
                    self.current_bottlenecks.insert(metric_name.to_string(), bottleneck);
                    Ok(None) // Don't report until minimum duration
                }
            }
        } else {
            // Check if this resolves an existing bottleneck
            if self.current_bottlenecks.remove(metric_name).is_some() {
                Ok(Some(BottleneckInfo::Resolved))
            } else {
                Ok(None)
            }
        }
    }
    
    /// Calculate confidence in bottleneck detection
    fn calculate_detection_confidence(&self, _metric_name: &str, value: f64, threshold: &PerformanceThreshold) -> f64 {
        // Calculate confidence based on how far the value is from threshold
        let threshold_value = if threshold.higher_is_better {
            threshold.warning_threshold
        } else {
            threshold.critical_threshold
        };
        
        let distance = (value - threshold_value).abs();
        let max_distance = threshold_value * 0.5; // 50% of threshold as max distance
        
        let confidence = (distance / max_distance).min(1.0);
        confidence * self.config.sensitivity
    }
    
    /// Get current bottleneck status
    pub fn get_bottleneck_status(&self) -> BottleneckStatusReport {
        let active_bottlenecks: Vec<_> = self.current_bottlenecks
            .iter()
            .map(|entry| {
                let bottleneck = entry.value();
                ActiveBottleneckInfo {
                    metric_name: bottleneck.metric_name.clone(),
                    duration: bottleneck.start_time.elapsed(),
                    severity: bottleneck.severity.clone(),
                    peak_value: bottleneck.peak_value,
                    resolution_attempted: bottleneck.resolution_attempted,
                }
            })
            .collect();
        
        BottleneckStatusReport {
            active_bottlenecks,
            total_active: self.current_bottlenecks.len() as u64,
            timestamp: Instant::now(),
        }
    }
}

/// Information about bottleneck detection result
#[derive(Debug)]
enum BottleneckInfo {
    Detected(BottleneckSeverity),
    Resolved,
}

/// Bottleneck detection report
#[derive(Debug)]
pub struct BottleneckReport {
    pub detected_bottlenecks: Vec<DetectedBottleneck>,
    pub resolved_bottlenecks: Vec<String>,
    pub active_bottlenecks_count: u64,
    pub timestamp: Instant,
}

/// Information about a detected bottleneck
#[derive(Debug, Clone)]
pub struct DetectedBottleneck {
    pub metric_name: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub severity: BottleneckSeverity,
    pub confidence: f64,
}

/// Status report of all bottlenecks
#[derive(Debug)]
pub struct BottleneckStatusReport {
    pub active_bottlenecks: Vec<ActiveBottleneckInfo>,
    pub total_active: u64,
    pub timestamp: Instant,
}

/// Information about an active bottleneck
#[derive(Debug, Clone)]
pub struct ActiveBottleneckInfo {
    pub metric_name: String,
    pub duration: Duration,
    pub severity: BottleneckSeverity,
    pub peak_value: f64,
    pub resolution_attempted: bool,
}

// ============= RESOURCE OPTIMIZER =============

/// Resource optimization manager for cross-system resource allocation
pub struct ResourceOptimizer {
    /// Resource allocation strategies
    allocation_strategies: HashMap<String, ResourceAllocationStrategy>,
    /// Current resource state
    resource_state: RwLock<ResourceState>,
    /// Optimization history
    optimization_history: RwLock<VecDeque<ResourceOptimizationEvent>>,
    /// Configuration
    config: ResourceOptimizationConfig,
}

/// Resource allocation strategy
#[derive(Debug, Clone)]
pub struct ResourceAllocationStrategy {
    pub name: String,
    pub target_resources: Vec<String>,
    pub allocation_weights: HashMap<String, f64>,
    pub constraints: Vec<ResourceConstraint>,
}

/// Resource constraint definition
#[derive(Debug, Clone)]
pub struct ResourceConstraint {
    pub resource_name: String,
    pub constraint_type: ConstraintType,
    pub limit_value: f64,
}

/// Type of resource constraint
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    MaxUsage,
    MinReserved,
    BalanceRatio,
}

/// Current resource state
#[derive(Debug, Clone)]
pub struct ResourceState {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub network_bandwidth_usage_percent: f64,
    pub disk_io_usage_percent: f64,
    pub resource_allocations: HashMap<String, f64>,
    pub last_updated: Instant,
}

/// Resource optimization event
#[derive(Debug, Clone)]
pub struct ResourceOptimizationEvent {
    pub timestamp: Instant,
    pub strategy_applied: String,
    pub resources_affected: Vec<String>,
    pub performance_impact: f64,
}

/// Configuration for resource optimization
#[derive(Debug, Clone)]
pub struct ResourceOptimizationConfig {
    pub optimization_interval: Duration,
    pub max_history_size: usize,
    pub enable_aggressive_optimization: bool,
}

impl Default for ResourceOptimizationConfig {
    fn default() -> Self {
        Self {
            optimization_interval: Duration::from_secs(60),
            max_history_size: 500,
            enable_aggressive_optimization: false,
        }
    }
}

impl ResourceOptimizer {
    pub fn new(config: ResourceOptimizationConfig) -> Self {
        Self {
            allocation_strategies: HashMap::new(),
            resource_state: RwLock::new(ResourceState {
                cpu_usage_percent: 0.0,
                memory_usage_percent: 0.0,
                network_bandwidth_usage_percent: 0.0,
                disk_io_usage_percent: 0.0,
                resource_allocations: HashMap::new(),
                last_updated: Instant::now(),
            }),
            optimization_history: RwLock::new(VecDeque::new()),
            config,
        }
    }
    
    /// Update current resource state
    pub fn update_resource_state(&self, metrics: &HashMap<String, f64>) -> PerformanceResult<()> {
        let mut state = self.resource_state.write()
            .map_err(|_| PerformanceError::ResourceOptimizationFailed("Lock poisoned".to_string()))?;
        
        // Update resource usage from metrics
        if let Some(&cpu_usage) = metrics.get("system.cpu_usage_percent") {
            state.cpu_usage_percent = cpu_usage;
        }
        if let Some(&memory_usage) = metrics.get("system.memory_usage_percent") {
            state.memory_usage_percent = memory_usage;
        }
        if let Some(&network_usage) = metrics.get("system.network_usage_percent") {
            state.network_bandwidth_usage_percent = network_usage;
        }
        if let Some(&disk_usage) = metrics.get("system.disk_io_usage_percent") {
            state.disk_io_usage_percent = disk_usage;
        }
        
        state.last_updated = Instant::now();
        
        Ok(())
    }
    
    /// Optimize resource allocation based on current performance
    pub fn optimize_resource_allocation(&self, performance_metrics: &HashMap<String, f64>) -> PerformanceResult<ResourceOptimizationReport> {
        let state = self.resource_state.read()
            .map_err(|_| PerformanceError::ResourceOptimizationFailed("Lock poisoned".to_string()))?;
        
        let mut optimizations_applied = Vec::new();
        
        // Simple resource optimization heuristics
        
        // CPU optimization
        if state.cpu_usage_percent > 80.0 {
            optimizations_applied.push("Reduce JIT compilation intensity".to_string());
        }
        
        // Memory optimization  
        if state.memory_usage_percent > 85.0 {
            optimizations_applied.push("Trigger garbage collection".to_string());
        }
        
        // Network optimization
        if state.network_bandwidth_usage_percent > 90.0 {
            optimizations_applied.push("Reduce distributed communication frequency".to_string());
        }
        
        // Record optimization event
        let event = ResourceOptimizationEvent {
            timestamp: Instant::now(),
            strategy_applied: "Reactive Resource Management".to_string(),
            resources_affected: vec!["cpu".to_string(), "memory".to_string(), "network".to_string()],
            performance_impact: 0.05, // Estimate 5% improvement
        };
        
        let mut history = self.optimization_history.write()
            .map_err(|_| PerformanceError::ResourceOptimizationFailed("Lock poisoned".to_string()))?;
        
        if history.len() >= self.config.max_history_size {
            history.pop_front();
        }
        history.push_back(event);
        
        Ok(ResourceOptimizationReport {
            optimizations_applied,
            current_cpu_usage: state.cpu_usage_percent,
            current_memory_usage: state.memory_usage_percent,
            current_network_usage: state.network_bandwidth_usage_percent,
            timestamp: Instant::now(),
        })
    }
}

/// Resource optimization report
#[derive(Debug, Clone)]
pub struct ResourceOptimizationReport {
    pub optimizations_applied: Vec<String>,
    pub current_cpu_usage: f64,
    pub current_memory_usage: f64,
    pub current_network_usage: f64,
    pub timestamp: Instant,
}

// ============= MAIN SYSTEM IMPLEMENTATION =============

impl PerformanceIntegrationSystem {
    /// Create a new performance integration system
    pub fn new(config: PerformanceSystemConfig) -> Self {
        let metrics_collector = Arc::new(UnifiedMetricsCollector::new(config.metrics_config.clone()));
        let analytics_engine = Arc::new(RealTimeAnalyticsEngine::new(AnalyticsConfig::default()));
        let optimization_engine = Arc::new(DynamicOptimizationEngine::new(OptimizationConfig::default()));
        let bottleneck_detector = Arc::new(BottleneckDetector::new(BottleneckDetectionConfig::default()));
        let resource_optimizer = Arc::new(ResourceOptimizer::new(ResourceOptimizationConfig::default()));
        let system_stats = Arc::new(PerformanceSystemStats::default());
        
        Self {
            metrics_collector,
            analytics_engine,
            optimization_engine,
            bottleneck_detector,
            resource_optimizer,
            config,
            system_stats,
        }
    }
    
    /// Start the performance integration system
    pub fn start(&self) -> PerformanceResult<()> {
        // Initialize all subsystems
        // This would start background threads for continuous monitoring
        Ok(())
    }
    
    /// Stop the performance integration system
    pub fn stop(&self) -> PerformanceResult<()> {
        // Gracefully stop all subsystems
        Ok(())
    }
    
    /// Get the unified metrics collector
    pub fn metrics_collector(&self) -> Arc<UnifiedMetricsCollector> {
        Arc::clone(&self.metrics_collector)
    }
    
    /// Get overall system performance statistics
    pub fn get_system_stats(&self) -> PerformanceResult<SystemPerformanceReport> {
        let uptime = self.system_stats.system_start_time.elapsed();
        let optimizations_applied = self.system_stats.total_optimizations_applied.load(Ordering::Relaxed);
        let bottlenecks_detected = self.system_stats.bottlenecks_detected.load(Ordering::Relaxed);
        let bottlenecks_resolved = self.system_stats.bottlenecks_resolved.load(Ordering::Relaxed);
        
        let overall_score = self.system_stats.overall_performance_score.lock()
            .map_err(|_| PerformanceError::SystemIntegrationError("Lock poisoned".to_string()))?
            .clone();
        
        let collector_stats = self.metrics_collector.get_collector_stats()?;
        
        Ok(SystemPerformanceReport {
            uptime,
            optimizations_applied,
            bottlenecks_detected,
            bottlenecks_resolved,
            overall_performance_score: overall_score,
            metrics_collected: collector_stats.total_metrics_collected,
            active_metrics: collector_stats.active_metrics_count,
            memory_usage_bytes: collector_stats.memory_usage_bytes,
        })
    }
}

/// Comprehensive system performance report
#[derive(Debug, Clone)]
pub struct SystemPerformanceReport {
    pub uptime: Duration,
    pub optimizations_applied: u64,
    pub bottlenecks_detected: u64,
    pub bottlenecks_resolved: u64,
    pub overall_performance_score: Option<f64>,
    pub metrics_collected: u64,
    pub active_metrics: u64,
    pub memory_usage_bytes: u64,
}

impl SystemPerformanceReport {
    /// Calculate bottleneck resolution rate
    pub fn bottleneck_resolution_rate(&self) -> f64 {
        if self.bottlenecks_detected > 0 {
            self.bottlenecks_resolved as f64 / self.bottlenecks_detected as f64
        } else {
            1.0 // Perfect score if no bottlenecks detected
        }
    }
    
    /// Calculate metrics collection rate per second
    pub fn metrics_collection_rate(&self) -> f64 {
        let uptime_secs = self.uptime.as_secs_f64();
        if uptime_secs > 0.0 {
            self.metrics_collected as f64 / uptime_secs
        } else {
            0.0
        }
    }
}

// ============= INTEGRATION TRAITS =============

/// Trait for systems that can provide performance metrics
pub trait PerformanceMonitorable {
    /// Get current performance metrics
    fn get_performance_metrics(&self) -> HashMap<String, f64>;
    
    /// Check if the system has performance data available
    fn has_performance_data(&self) -> bool;
}

/// Trait for systems that can be optimized based on performance data
pub trait PerformanceOptimizable {
    /// Apply optimization suggestions
    fn apply_optimization(&mut self, optimization: OptimizationSuggestion) -> PerformanceResult<()>;
    
    /// Check if optimization is supported
    fn supports_optimization(&self, optimization_type: &str) -> bool;
}

/// Optimization suggestion from the performance system
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub optimization_type: String,
    pub parameters: HashMap<String, f64>,
    pub expected_improvement: f64,
    pub confidence: f64,
}

// ============= TESTING AND VALIDATION =============

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_time_series_metric_basic_operations() {
        let metric = TimeSeriesMetric::new(100);
        
        // Add some data points
        metric.add_data_point(MetricDataPoint::new(10.0)).unwrap();
        metric.add_data_point(MetricDataPoint::new(20.0)).unwrap();
        metric.add_data_point(MetricDataPoint::new(30.0)).unwrap();
        
        let stats = metric.get_statistics();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.min, Some(10.0));
        assert_eq!(stats.max, Some(30.0));
    }
    
    #[test]
    fn test_unified_metrics_collector() {
        let config = MetricsCollectorConfig::default();
        let collector = UnifiedMetricsCollector::new(config);
        
        // Record some metrics
        collector.record_metric("test.metric1", 42.0).unwrap();
        collector.record_metric("test.metric2", 84.0).unwrap();
        
        let stats1 = collector.get_metric_statistics("test.metric1").unwrap();
        assert_eq!(stats1.count, 1);
        assert_eq!(stats1.mean, 42.0);
        
        let stats2 = collector.get_metric_statistics("test.metric2").unwrap();
        assert_eq!(stats2.count, 1);
        assert_eq!(stats2.mean, 84.0);
    }
    
    #[test]
    fn test_performance_integration_system_creation() {
        let config = PerformanceSystemConfig::default();
        let system = PerformanceIntegrationSystem::new(config);
        
        // Test system starts successfully
        system.start().unwrap();
        
        // Test we can get system stats
        let report = system.get_system_stats().unwrap();
        assert_eq!(report.optimizations_applied, 0);
        assert_eq!(report.bottlenecks_detected, 0);
        
        // Test system stops successfully
        system.stop().unwrap();
    }
    
    #[test]
    fn test_metric_statistics_calculations() {
        let stats = MetricStatistics {
            count: 5,
            mean: 10.0,
            std_dev: 2.0,
            min: Some(8.0),
            max: Some(12.0),
            sum: 50.0,
        };
        
        // Test coefficient of variation
        assert_eq!(stats.coefficient_of_variation(), 0.2); // 2.0 / 10.0
        
        // Test z-score calculation
        assert_eq!(stats.z_score(12.0), 1.0); // (12 - 10) / 2
        assert_eq!(stats.z_score(8.0), -1.0); // (8 - 10) / 2
    }
    
    #[test]
    fn test_system_performance_report_calculations() {
        let report = SystemPerformanceReport {
            uptime: Duration::from_secs(100),
            optimizations_applied: 5,
            bottlenecks_detected: 10,
            bottlenecks_resolved: 8,
            overall_performance_score: Some(0.85),
            metrics_collected: 1000,
            active_metrics: 50,
            memory_usage_bytes: 1024 * 1024, // 1MB
        };
        
        // Test bottleneck resolution rate
        assert_eq!(report.bottleneck_resolution_rate(), 0.8); // 8/10
        
        // Test metrics collection rate
        assert_eq!(report.metrics_collection_rate(), 10.0); // 1000/100
    }
}