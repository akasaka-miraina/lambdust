//! Regression Testing Benchmarks
//!
//! Automated performance regression detection and monitoring system.
//! Tracks performance changes over time and provides alerts for significant
//! performance degradations or improvements.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lambdust::{LambdustRuntime, MultithreadedLambdust};
use lambdust::runtime::{BootstrapIntegrationConfig, BootstrapMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime as TokioRuntime;

/// Performance regression testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionTestConfig {
    pub baseline_threshold: f64,      // Percentage change to trigger alert (e.g., 0.15 = 15%)
    pub improvement_threshold: f64,   // Percentage improvement to note (e.g., 0.10 = 10%)
    pub sample_size: usize,          // Number of samples for statistical significance
    pub warmup_iterations: usize,    // Warmup iterations before measurement
    pub history_retention_days: u32, // Days to retain historical data
    pub enabled_categories: Vec<RegressionCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionCategory {
    Migration,      // Migration impact benchmarks
    CoreOperations, // Core operation performance
    SystemLevel,    // System-level performance
    Memory,         // Memory usage and GC performance
    Concurrency,    // Thread safety and concurrent performance
}

/// Historical performance data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDataPoint {
    pub timestamp: u64,
    pub benchmark_name: String,
    pub category: RegressionCategory,
    pub bootstrap_mode: String,
    pub thread_count: usize,
    pub data_size: usize,
    pub execution_time: Duration,
    pub memory_usage: Option<usize>,
    pub throughput: Option<f64>,
    pub git_commit: Option<String>,
    pub build_info: Option<String>,
}

/// Performance regression analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    pub benchmark_name: String,
    pub current_performance: PerformanceMetrics,
    pub baseline_performance: PerformanceMetrics,
    pub performance_change: PerformanceChange,
    pub statistical_significance: f64,
    pub recommendation: RegressionRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub mean_time: Duration,
    pub median_time: Duration,
    pub std_deviation: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceChange {
    pub percentage_change: f64,
    pub absolute_change: Duration,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Improvement,
    Degradation,
    NoSignificantChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionRecommendation {
    NoAction,
    Investigate,
    Alert,
    CriticalAlert,
}

impl Default for RegressionTestConfig {
    fn default() -> Self {
        Self {
            baseline_threshold: 0.15,
            improvement_threshold: 0.10,
            sample_size: 100,
            warmup_iterations: 10,
            history_retention_days: 30,
            enabled_categories: vec![
                RegressionCategory::Migration,
                RegressionCategory::CoreOperations,
                RegressionCategory::SystemLevel,
                RegressionCategory::Memory,
                RegressionCategory::Concurrency,
            ],
        }
    }
}

/// Performance history manager
pub struct PerformanceHistoryManager {
    config: RegressionTestConfig,
    history_file: String,
    data_points: Vec<PerformanceDataPoint>,
}

impl PerformanceHistoryManager {
    pub fn new(config: RegressionTestConfig, history_file: &str) -> Self {
        let mut manager = Self {
            config,
            history_file: history_file.to_string(),
            data_points: Vec::new(),
        };
        
        manager.load_history();
        manager
    }
    
    pub fn add_data_point(&mut self, data_point: PerformanceDataPoint) {
        self.data_points.push(data_point);
        self.cleanup_old_data();
        self.save_history();
    }
    
    pub fn analyze_regression(&self, benchmark_name: &str, current_metrics: PerformanceMetrics) -> Option<RegressionAnalysis> {
        let baseline_metrics = self.get_baseline_metrics(benchmark_name)?;
        
        let current_mean = current_metrics.mean_time.as_nanos() as f64;
        let baseline_mean = baseline_metrics.mean_time.as_nanos() as f64;
        
        let percentage_change = (current_mean - baseline_mean) / baseline_mean;
        let absolute_change = current_metrics.mean_time - baseline_metrics.mean_time;
        
        let change_type = if percentage_change > self.config.baseline_threshold {
            ChangeType::Degradation
        } else if percentage_change < -self.config.improvement_threshold {
            ChangeType::Improvement
        } else {
            ChangeType::NoSignificantChange
        };
        
        let recommendation = match change_type {
            ChangeType::Degradation if percentage_change > 0.5 => RegressionRecommendation::CriticalAlert,
            ChangeType::Degradation if percentage_change > 0.25 => RegressionRecommendation::Alert,
            ChangeType::Degradation => RegressionRecommendation::Investigate,
            _ => RegressionRecommendation::NoAction,
        };
        
        let statistical_significance = self.calculate_statistical_significance(&current_metrics, &baseline_metrics);
        
        Some(RegressionAnalysis {
            benchmark_name: benchmark_name.to_string(),
            current_performance: current_metrics,
            baseline_performance: baseline_metrics,
            performance_change: PerformanceChange {
                percentage_change,
                absolute_change,
                change_type,
            },
            statistical_significance,
            recommendation,
        })
    }
    
    fn get_baseline_metrics(&self, benchmark_name: &str) -> Option<PerformanceMetrics> {
        let recent_data: Vec<_> = self.data_points
            .iter()
            .filter(|dp| dp.benchmark_name == benchmark_name)
            .collect();
        
        if recent_data.len() < 10 {
            return None;
        }
        
        let times: Vec<Duration> = recent_data.iter()
            .map(|dp| dp.execution_time)
            .collect();
        
        Some(calculate_performance_metrics(&times))
    }
    
    fn calculate_statistical_significance(&self, current: &PerformanceMetrics, baseline: &PerformanceMetrics) -> f64 {
        // Simplified statistical significance calculation
        // In a real implementation, this would use proper statistical tests like t-test
        let pooled_variance = ((current.std_deviation.as_nanos() as f64).powi(2) + 
                              (baseline.std_deviation.as_nanos() as f64).powi(2)) / 2.0;
        let standard_error = (pooled_variance / (current.sample_count as f64)).sqrt();
        
        if standard_error == 0.0 {
            1.0
        } else {
            let t_statistic = ((current.mean_time.as_nanos() as f64) - (baseline.mean_time.as_nanos() as f64)).abs() / standard_error;
            // Simplified p-value approximation
            (t_statistic / (t_statistic + 1.0)).min(1.0)
        }
    }
    
    fn load_history(&mut self) {
        if let Ok(data) = fs::read_to_string(&self.history_file) {
            if let Ok(data_points) = serde_json::from_str(&data) {
                self.data_points = data_points;
            }
        }
    }
    
    fn save_history(&self) {
        if let Ok(data) = serde_json::to_string_pretty(&self.data_points) {
            let _ = fs::write(&self.history_file, data);
        }
    }
    
    fn cleanup_old_data(&mut self) {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - (self.config.history_retention_days as u64 * 24 * 60 * 60);
        
        self.data_points.retain(|dp| dp.timestamp > cutoff_time);
    }
}

// ============================================================================
// REGRESSION TEST BENCHMARKS
// ============================================================================

fn bench_migration_impact_regression(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    let config = RegressionTestConfig::default();
    let mut history_manager = PerformanceHistoryManager::new(config.clone(), "migration_regression_history.json");
    
    let mut group = c.benchmark_group("migration_impact_regression");
    group.measurement_time(Duration::from_secs(10));

    let regression_tests = vec![
        ("arithmetic_migration", "(fold + 0 (range 1 1000))", RegressionCategory::Migration),
        ("list_migration", "(map (lambda (x) (* x x)) (range 1 500))", RegressionCategory::Migration),
        ("string_migration", r#"(fold string-append "" (map (lambda (x) (string-append "item-" (number->string x))) (range 1 100)))"#, RegressionCategory::Migration),
    ];

    for (test_name, expression, category) in regression_tests {
        let benchmark_id = BenchmarkId::from_parameter(test_name);
        
        group.bench_with_input(benchmark_id, &expression, |b, &expr| {
            b.to_async(&rt).iter_custom(|iters| async move {
                let mut measurements = Vec::new();
                
                for _ in 0..iters {
                    let config = BootstrapIntegrationConfig {
                        mode: BootstrapMode::Minimal,
                        verbose: false,
                        ..Default::default()
                    };
                    
                    let runtime = LambdustRuntime::with_bootstrap_config(Some(1), config).unwrap();
                    let lambdust = MultithreadedLambdust::with_runtime(runtime);
                    
                    let start = Instant::now();
                    let _result = lambdust.eval(expr, Some("regression-test")).await.unwrap();
                    let elapsed = start.elapsed();
                    measurements.push(elapsed);
                    
                    let _ = lambdust.shutdown().await;
                }
                
                // Record performance data point
                let current_metrics = calculate_performance_metrics(&measurements);
                let data_point = PerformanceDataPoint {
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    benchmark_name: test_name.to_string(),
                    category: category.clone(),
                    bootstrap_mode: "Minimal".to_string(),
                    thread_count: 1,
                    data_size: 1000,
                    execution_time: current_metrics.mean_time,
                    memory_usage: None,
                    throughput: None,
                    git_commit: get_git_commit_hash(),
                    build_info: Some(format!("rustc-{}", env!("RUSTC_VERSION"))),
                };
                
                // Analyze for regression
                if let Some(analysis) = history_manager.analyze_regression(test_name, current_metrics.clone()) {
                    match analysis.recommendation {
                        RegressionRecommendation::Alert | RegressionRecommendation::CriticalAlert => {
                            eprintln!("⚠️ Performance regression detected in {}: {:.2}% change", 
                                test_name, analysis.performance_change.percentage_change * 100.0);
                        }
                        RegressionRecommendation::Investigate => {
                            println!("📊 Performance change in {}: {:.2}% change", 
                                test_name, analysis.performance_change.percentage_change * 100.0);
                        }
                        _ => {}
                    }
                }
                
                // Return the sum of all measurements for Criterion
                measurements.iter().sum()
            });
        });
    }
    
    group.finish();
}

fn bench_core_operations_regression(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    let config = RegressionTestConfig::default();
    let mut history_manager = PerformanceHistoryManager::new(config, "core_operations_regression_history.json");
    
    let mut group = c.benchmark_group("core_operations_regression");
    group.measurement_time(Duration::from_secs(10));

    let core_operation_tests = vec![
        ("list_operations_perf", "(fold append '() (map (lambda (x) (list x)) (range 1 500)))", 500),
        ("string_operations_perf", r#"(map string-upcase (map (lambda (x) (string-append "test-" (number->string x))) (range 1 200)))"#, 200),
        ("vector_operations_perf", "(vector-fold + 0 (list->vector (range 1 1000)))", 1000),
        ("arithmetic_operations_perf", "(fold * 1 (range 1 50))", 50),
    ];

    for (test_name, expression, data_size) in core_operation_tests {
        let benchmark_id = BenchmarkId::from_parameter(test_name);
        
        group.throughput(Throughput::Elements(data_size as u64));
        group.bench_with_input(benchmark_id, &expression, |b, &expr| {
            b.to_async(&rt).iter_custom(|iters| async move {
                let mut measurements = Vec::new();
                
                // Warmup
                for _ in 0..config.warmup_iterations {
                    let config = BootstrapIntegrationConfig {
                        mode: BootstrapMode::Minimal,
                        verbose: false,
                        ..Default::default()
                    };
                    
                    let runtime = LambdustRuntime::with_bootstrap_config(Some(1), config).unwrap();
                    let lambdust = MultithreadedLambdust::with_runtime(runtime);
                    let _ = lambdust.eval(expr, Some("warmup")).await;
                    let _ = lambdust.shutdown().await;
                }
                
                // Actual measurements
                for _ in 0..iters.min(config.sample_size as u64) {
                    let config = BootstrapIntegrationConfig {
                        mode: BootstrapMode::Minimal,
                        verbose: false,
                        ..Default::default()
                    };
                    
                    let runtime = LambdustRuntime::with_bootstrap_config(Some(1), config).unwrap();
                    let lambdust = MultithreadedLambdust::with_runtime(runtime);
                    
                    let start = Instant::now();
                    let _result = lambdust.eval(expr, Some("regression-test")).await.unwrap();
                    let elapsed = start.elapsed();
                    measurements.push(elapsed);
                    
                    let _ = lambdust.shutdown().await;
                }
                
                // Performance analysis
                let current_metrics = calculate_performance_metrics(&measurements);
                let data_point = PerformanceDataPoint {
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    benchmark_name: test_name.to_string(),
                    category: RegressionCategory::CoreOperations,
                    bootstrap_mode: "Minimal".to_string(),
                    thread_count: 1,
                    data_size,
                    execution_time: current_metrics.mean_time,
                    memory_usage: None,
                    throughput: Some(data_size as f64 / current_metrics.mean_time.as_secs_f64()),
                    git_commit: get_git_commit_hash(),
                    build_info: Some(format!("rustc-{}", env!("RUSTC_VERSION"))),
                };
                
                history_manager.add_data_point(data_point);
                
                measurements.iter().sum()
            });
        });
    }
    
    group.finish();
}

fn bench_system_level_regression(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    let config = RegressionTestConfig::default();
    let mut history_manager = PerformanceHistoryManager::new(config, "system_level_regression_history.json");
    
    let mut group = c.benchmark_group("system_level_regression");
    group.measurement_time(Duration::from_secs(15));

    let system_tests = vec![
        ("startup_time", "", 0), // Special case for startup measurement
        ("memory_allocation", "(map (lambda (x) (list x (* x 2) (* x 3))) (range 1 500))", 500),
        ("concurrent_access", "(map (lambda (x) (fold + 0 (range 1 x))) (range 1 50))", 50),
    ];

    for (test_name, expression, data_size) in system_tests {
        let benchmark_id = BenchmarkId::from_parameter(test_name);
        
        if test_name == "startup_time" {
            group.bench_function(benchmark_id, |b| {
                b.to_async(&rt).iter_custom(|iters| async move {
                    let mut measurements = Vec::new();
                    
                    for _ in 0..iters {
                        let config = BootstrapIntegrationConfig {
                            mode: BootstrapMode::Minimal,
                            verbose: false,
                            ..Default::default()
                        };
                        
                        let start = Instant::now();
                        let runtime = LambdustRuntime::with_bootstrap_config(Some(1), config).unwrap();
                        let _lambdust = MultithreadedLambdust::with_runtime(runtime);
                        let elapsed = start.elapsed();
                        measurements.push(elapsed);
                        
                        let _ = _lambdust.shutdown().await;
                    }
                    
                    let current_metrics = calculate_performance_metrics(&measurements);
                    let data_point = PerformanceDataPoint {
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        benchmark_name: test_name.to_string(),
                        category: RegressionCategory::SystemLevel,
                        bootstrap_mode: "Minimal".to_string(),
                        thread_count: 1,
                        data_size: 0,
                        execution_time: current_metrics.mean_time,
                        memory_usage: None,
                        throughput: None,
                        git_commit: get_git_commit_hash(),
                        build_info: Some(format!("rustc-{}", env!("RUSTC_VERSION"))),
                    };
                    
                    history_manager.add_data_point(data_point);
                    
                    measurements.iter().sum()
                });
            });
        } else {
            group.bench_with_input(benchmark_id, &expression, |b, &expr| {
                b.to_async(&rt).iter_custom(|iters| async move {
                    let mut measurements = Vec::new();
                    
                    for _ in 0..iters {
                        let config = BootstrapIntegrationConfig {
                            mode: BootstrapMode::Minimal,
                            verbose: false,
                            ..Default::default()
                        };
                        
                        let runtime = LambdustRuntime::with_bootstrap_config(Some(2), config).unwrap();
                        let lambdust = MultithreadedLambdust::with_runtime(runtime);
                        
                        let start = Instant::now();
                        let _result = lambdust.eval(expr, Some("system-regression-test")).await.unwrap();
                        let elapsed = start.elapsed();
                        measurements.push(elapsed);
                        
                        let _ = lambdust.shutdown().await;
                    }
                    
                    let current_metrics = calculate_performance_metrics(&measurements);
                    let data_point = PerformanceDataPoint {
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        benchmark_name: test_name.to_string(),
                        category: RegressionCategory::SystemLevel,
                        bootstrap_mode: "Minimal".to_string(),
                        thread_count: 2,
                        data_size,
                        execution_time: current_metrics.mean_time,
                        memory_usage: None,
                        throughput: if data_size > 0 { Some(data_size as f64 / current_metrics.mean_time.as_secs_f64()) } else { None },
                        git_commit: get_git_commit_hash(),
                        build_info: Some(format!("rustc-{}", env!("RUSTC_VERSION"))),
                    };
                    
                    history_manager.add_data_point(data_point);
                    
                    measurements.iter().sum()
                });
            });
        }
    }
    
    group.finish();
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn calculate_performance_metrics(measurements: &[Duration]) -> PerformanceMetrics {
    if measurements.is_empty() {
        return PerformanceMetrics {
            mean_time: Duration::ZERO,
            median_time: Duration::ZERO,
            std_deviation: Duration::ZERO,
            min_time: Duration::ZERO,
            max_time: Duration::ZERO,
            sample_count: 0,
        };
    }
    
    let mut sorted_measurements = measurements.to_vec();
    sorted_measurements.sort();
    
    let mean_nanos = measurements.iter().map(|d| d.as_nanos()).sum::<u128>() / measurements.len() as u128;
    let mean_time = Duration::from_nanos(mean_nanos as u64);
    
    let median_time = sorted_measurements[measurements.len() / 2];
    let min_time = sorted_measurements[0];
    let max_time = sorted_measurements[measurements.len() - 1];
    
    // Calculate standard deviation
    let variance = measurements.iter()
        .map(|d| {
            let diff = d.as_nanos() as i128 - mean_nanos as i128;
            (diff * diff) as u128
        })
        .sum::<u128>() / measurements.len() as u128;
    let std_deviation = Duration::from_nanos((variance as f64).sqrt() as u64);
    
    PerformanceMetrics {
        mean_time,
        median_time,
        std_deviation,
        min_time,
        max_time,
        sample_count: measurements.len(),
    }
}

fn get_git_commit_hash() -> Option<String> {
    // In a real implementation, this would use git commands or environment variables
    // For now, return a placeholder
    std::env::var("GIT_COMMIT").ok().or_else(|| Some("unknown".to_string()))
}

// ============================================================================
// PERFORMANCE REPORT GENERATION
// ============================================================================

pub fn generate_performance_report(history_managers: &[&PerformanceHistoryManager]) -> String {
    let mut report = String::new();
    report.push_str("# Performance Regression Report\n\n");
    report.push_str(&format!("Generated at: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    let mut total_alerts = 0;
    let mut total_investigations = 0;
    
    for manager in history_managers {
        let mut benchmark_names = std::collections::HashSet::new();
        for dp in &manager.data_points {
            benchmark_names.insert(&dp.benchmark_name);
        }
        
        for benchmark_name in benchmark_names {
            if let Some(recent_dp) = manager.data_points.iter()
                .filter(|dp| dp.benchmark_name == benchmark_name)
                .max_by_key(|dp| dp.timestamp) {
                
                let recent_measurements = vec![recent_dp.execution_time]; // Simplified
                let metrics = calculate_performance_metrics(&recent_measurements);
                
                if let Some(analysis) = manager.analyze_regression(benchmark_name, metrics) {
                    match analysis.recommendation {
                        RegressionRecommendation::CriticalAlert => {
                            report.push_str(&format!("🚨 **CRITICAL ALERT**: {} - {:.2}% performance degradation\n", 
                                benchmark_name, analysis.performance_change.percentage_change * 100.0));
                            total_alerts += 1;
                        }
                        RegressionRecommendation::Alert => {
                            report.push_str(&format!("⚠️ **ALERT**: {} - {:.2}% performance degradation\n", 
                                benchmark_name, analysis.performance_change.percentage_change * 100.0));
                            total_alerts += 1;
                        }
                        RegressionRecommendation::Investigate => {
                            report.push_str(&format!("📊 **INVESTIGATE**: {} - {:.2}% performance change\n", 
                                benchmark_name, analysis.performance_change.percentage_change * 100.0));
                            total_investigations += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    report.push_str(&format!("\n## Summary\n"));
    report.push_str(&format!("- Total alerts: {}\n", total_alerts));
    report.push_str(&format!("- Items requiring investigation: {}\n", total_investigations));
    
    if total_alerts == 0 && total_investigations == 0 {
        report.push_str("✅ No significant performance regressions detected.\n");
    }
    
    report
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    regression_benches,
    bench_migration_impact_regression,
    bench_core_operations_regression,
    bench_system_level_regression
);

criterion_main!(regression_benches);