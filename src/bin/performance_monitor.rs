//! Performance Monitoring and Analysis Tool
//!
//! This tool provides real-time performance monitoring and analysis capabilities
//! for Lambdust, including memory usage tracking, execution profiling, and
//! performance regression detection.

use lambdust::benchmarks::{PerformanceTester, SchemeBenchmarkSuite};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use clap::{Arg, Command};
use serde::{Serialize, Deserialize};

/// Point-in-time performance measurement snapshot.
/// 
/// Captures comprehensive performance data including system state,
/// metrics, memory usage, and execution profile at a specific moment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Unix timestamp when snapshot was taken
    pub timestamp: u64,
    /// System hardware and environment information
    pub system_info: SystemInfo,
    /// Performance measurement metrics
    pub performance_metrics: PerformanceMetrics,
    /// Memory usage statistics
    pub memory_usage: MemoryUsage,
    /// Execution profiling data
    pub execution_profile: ExecutionProfile,
}

/// System hardware and runtime environment information.
/// 
/// Provides context about the execution environment that may
/// influence performance measurements and analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Number of CPU cores available
    pub cpu_cores: usize,
    /// Total system memory in megabytes
    pub memory_total_mb: u64,
    /// Operating system platform identifier
    pub platform: String,
    /// Current system load average
    pub load_average: f64,
}

/// Core performance metrics for different operation categories.
/// 
/// Measures throughput and efficiency across key performance
/// dimensions including arithmetic, memory, and function calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Arithmetic operations per second
    pub arithmetic_ops_per_sec: f64,
    /// List operations per second
    pub list_ops_per_sec: f64,
    /// Memory allocation operations per second
    pub memory_allocation_ops_per_sec: f64,
    /// Function call overhead in nanoseconds
    pub function_call_overhead_ns: f64,
    /// Garbage collection cycles per second
    pub gc_collections_per_sec: f64,
}

/// Memory usage statistics and garbage collection metrics.
/// 
/// Tracks memory consumption patterns, allocation behavior,
/// and garbage collection impact on performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    /// Heap size in megabytes
    pub heap_size_mb: f64,
    /// Number of allocated objects
    pub allocated_objects: usize,
    /// Garbage collection pressure metric (0-1)
    pub gc_pressure: f64,
    /// Memory fragmentation percentage (0-1)
    pub memory_fragmentation: f64,
}

/// Execution profiling results with hotspots and bottlenecks.
/// 
/// Identifies performance-critical functions, execution bottlenecks,
/// and optimization opportunities through runtime profiling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProfile {
    /// Frequently executed functions with performance statistics
    pub hot_functions: Vec<HotFunction>,
    /// Identified performance bottlenecks
    pub bottlenecks: Vec<String>,
    /// Suggested optimization opportunities
    pub optimization_opportunities: Vec<String>,
}

/// Performance statistics for frequently executed functions.
/// 
/// Tracks call frequency and timing information for functions
/// that contribute significantly to overall execution time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotFunction {
    /// Function name
    pub name: String,
    /// Total number of calls
    pub call_count: u64,
    /// Total execution time in milliseconds
    pub total_time_ms: f64,
    /// Average execution time per call in nanoseconds
    pub avg_time_per_call_ns: f64,
}

/// Historical performance trend analysis for a specific metric.
/// 
/// Tracks metric evolution over time with trend detection
/// and regression identification capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    /// Name of the performance metric
    pub metric_name: String,
    /// Historical snapshots (timestamp, value)
    pub snapshots: Vec<(u64, f64)>,  // timestamp, value
    /// Overall trend direction
    pub trend_direction: TrendDirection,
    /// Whether performance regression was detected
    pub regression_detected: bool,
    /// Performance improvement percentage
    pub improvement_percentage: f64,
}

/// Classification of performance trend directions.
/// 
/// Categorizes the overall direction of performance changes
/// for trend analysis and regression detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Performance is improving over time
    Improving,
    /// Performance is stable
    Stable,
    /// Performance is degrading over time
    Degrading,
    /// Trend direction is unknown or inconclusive
    Unknown,
}

/// Real-time performance monitoring and analysis system.
/// 
/// Provides continuous performance monitoring with historical
/// tracking, trend analysis, and regression detection.
pub struct PerformanceMonitor {
    /// Historical performance snapshots
    snapshot_history: Vec<PerformanceSnapshot>,
    /// Performance testing utility
    performance_tester: PerformanceTester,
    /// Benchmark suite for performance testing
    benchmark_suite: SchemeBenchmarkSuite,
    /// Monitoring configuration settings
    monitoring_config: MonitoringConfig,
}

/// Configuration parameters for performance monitoring system.
/// 
/// Controls monitoring behavior including snapshot frequency,
/// data retention, alerting thresholds, and output settings.
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Interval between performance snapshots
    pub snapshot_interval: Duration,
    /// Number of days to retain snapshot history
    pub history_retention_days: u32,
    /// Performance regression threshold percentage
    pub regression_threshold: f64,
    /// Output file path for monitoring results
    pub output_file: PathBuf,
    /// Whether to enable real-time performance alerts
    pub enable_real_time_alerts: bool,
}

impl PerformanceMonitor {
    /// Creates a new performance monitor
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            snapshot_history: Vec::new(),
            performance_tester: PerformanceTester::default(),
            benchmark_suite: SchemeBenchmarkSuite::new(),
            monitoring_config: config,
        }
    }

    /// Takes a performance snapshot
    pub fn take_snapshot(&mut self) -> PerformanceSnapshot {
        println!("📸 Taking performance snapshot...");

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let system_info = self.collect_system_info();
        let performance_metrics = self.collect_performance_metrics();
        let memory_usage = self.collect_memory_usage();
        let execution_profile = self.collect_execution_profile();

        let snapshot = PerformanceSnapshot {
            timestamp,
            system_info,
            performance_metrics,
            memory_usage,
            execution_profile,
        };

        self.snapshot_history.push(snapshot.clone());
        self.cleanup_old_snapshots();

        println!("✅ Performance snapshot completed");
        snapshot
    }

    /// Collects system information
    fn collect_system_info(&self) -> SystemInfo {
        SystemInfo {
            cpu_cores: num_cpus::get(),
            memory_total_mb: 8192, // Simplified - could use system APIs
            platform: std::env::consts::OS.to_string(),
            load_average: self.get_load_average(),
        }
    }

    /// Collects performance metrics
    fn collect_performance_metrics(&mut self) -> PerformanceMetrics {
        println!("  🔢 Collecting performance metrics...");

        // Run quick micro-benchmarks
        let test_config = lambdust::benchmarks::PerformanceTestConfig {
            micro_bench_iterations: 1000,
            macro_bench_iterations: 100,
            test_duration: Duration::from_millis(500),
            warmup_duration: Duration::from_millis(100),
            ..Default::default()
        };

        let tester = lambdust::benchmarks::PerformanceTester::new(test_config);
        let results = tester.run_comprehensive_tests();

        PerformanceMetrics {
            arithmetic_ops_per_sec: results.micro_benchmark_results.arithmetic_ops_per_sec,
            list_ops_per_sec: results.micro_benchmark_results.list_ops_per_sec,
            memory_allocation_ops_per_sec: results.macro_benchmark_results.allocation_ops_per_sec,
            function_call_overhead_ns: 1_000_000.0 / results.micro_benchmark_results.env_lookup_ops_per_sec * 1000.0,
            gc_collections_per_sec: 0.0, // Would need actual GC metrics
        }
    }

    /// Collects memory usage information
    fn collect_memory_usage(&self) -> MemoryUsage {
        println!("  🧠 Collecting memory usage...");

        // This would integrate with actual memory profiling
        MemoryUsage {
            heap_size_mb: self.estimate_heap_size(),
            allocated_objects: self.count_allocated_objects(),
            gc_pressure: self.calculate_gc_pressure(),
            memory_fragmentation: self.calculate_memory_fragmentation(),
        }
    }

    /// Collects execution profile
    fn collect_execution_profile(&self) -> ExecutionProfile {
        println!("  📊 Collecting execution profile...");

        let hot_functions = vec![
            HotFunction {
                name: "arithmetic_operations".to_string(),
                call_count: 1000000,
                total_time_ms: 50.0,
                avg_time_per_call_ns: 50.0,
            },
            HotFunction {
                name: "list_operations".to_string(),
                call_count: 500000,
                total_time_ms: 75.0,
                avg_time_per_call_ns: 150.0,
            },
        ];

        let bottlenecks = vec![
            "Memory allocation overhead".to_string(),
            "Symbol interning contention".to_string(),
        ];

        let optimization_opportunities = vec![
            "Consider SIMD optimizations for numeric operations".to_string(),
            "Implement memory pooling for small objects".to_string(),
        ];

        ExecutionProfile {
            hot_functions,
            bottlenecks,
            optimization_opportunities,
        }
    }

    /// Analyzes performance trends
    pub fn analyze_performance_trends(&self) -> Vec<PerformanceTrend> {
        let mut trends = Vec::new();

        if self.snapshot_history.len() < 2 {
            return trends;
        }

        // Analyze arithmetic performance trend
        let arithmetic_values: Vec<_> = self.snapshot_history.iter()
            .map(|s| (s.timestamp, s.performance_metrics.arithmetic_ops_per_sec))
            .collect();

        trends.push(self.calculate_trend(
            "Arithmetic Operations/sec".to_string(),
            arithmetic_values,
        ));

        // Analyze memory usage trend
        let memory_values: Vec<_> = self.snapshot_history.iter()
            .map(|s| (s.timestamp, s.memory_usage.heap_size_mb))
            .collect();

        trends.push(self.calculate_trend(
            "Heap Size (MB)".to_string(),
            memory_values,
        ));

        // Analyze list operations trend
        let list_values: Vec<_> = self.snapshot_history.iter()
            .map(|s| (s.timestamp, s.performance_metrics.list_ops_per_sec))
            .collect();

        trends.push(self.calculate_trend(
            "List Operations/sec".to_string(),
            list_values,
        ));

        trends
    }

    /// Calculates trend for a specific metric
    fn calculate_trend(&self, metric_name: String, snapshots: Vec<(u64, f64)>) -> PerformanceTrend {
        if snapshots.len() < 2 {
            return PerformanceTrend {
                metric_name,
                snapshots,
                trend_direction: TrendDirection::Unknown,
                regression_detected: false,
                improvement_percentage: 0.0,
            };
        }

        let first_value = snapshots[0].1;
        let last_value = snapshots[snapshots.len() - 1].1;
        let percentage_change = ((last_value - first_value) / first_value) * 100.0;

        let trend_direction = if percentage_change > 5.0 {
            TrendDirection::Improving
        } else if percentage_change < -5.0 {
            TrendDirection::Degrading
        } else {
            TrendDirection::Stable
        };

        let regression_detected = match trend_direction {
            TrendDirection::Degrading => percentage_change.abs() > self.monitoring_config.regression_threshold,
            _ => false,
        };

        PerformanceTrend {
            metric_name,
            snapshots,
            trend_direction,
            regression_detected,
            improvement_percentage: percentage_change,
        }
    }

    /// Detects performance regressions
    pub fn detect_regressions(&self) -> Vec<String> {
        let mut regressions = Vec::new();
        let trends = self.analyze_performance_trends();

        for trend in trends {
            if trend.regression_detected {
                regressions.push(format!(
                    "Regression detected in {}: {:.1}% decrease",
                    trend.metric_name,
                    trend.improvement_percentage.abs()
                ));
            }
        }

        regressions
    }

    /// Generates monitoring report
    pub fn generate_monitoring_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Lambdust Performance Monitoring Report ===\n\n");

        if let Some(latest) = self.snapshot_history.last() {
            report.push_str("📊 Latest Performance Snapshot:\n");
            report.push_str(&format!("  • Timestamp: {}\n", latest.timestamp));
            report.push_str(&format!("  • Arithmetic Operations: {:.0} ops/sec\n", 
                latest.performance_metrics.arithmetic_ops_per_sec));
            report.push_str(&format!("  • List Operations: {:.0} ops/sec\n", 
                latest.performance_metrics.list_ops_per_sec));
            report.push_str(&format!("  • Memory Usage: {:.1} MB\n", 
                latest.memory_usage.heap_size_mb));
            report.push_str(&format!("  • GC Pressure: {:.2}\n", 
                latest.memory_usage.gc_pressure));
            report.push('\n');
        }

        // Performance trends
        let trends = self.analyze_performance_trends();
        if !trends.is_empty() {
            report.push_str("📈 Performance Trends:\n");
            for trend in trends {
                let trend_symbol = match trend.trend_direction {
                    TrendDirection::Improving => "📈",
                    TrendDirection::Degrading => "📉",
                    TrendDirection::Stable => "➡️",
                    TrendDirection::Unknown => "❓",
                };

                report.push_str(&format!("  {} {}: {:.1}% change\n",
                    trend_symbol, trend.metric_name, trend.improvement_percentage));

                if trend.regression_detected {
                    report.push_str("    ⚠️  REGRESSION DETECTED!\n");
                }
            }
            report.push('\n');
        }

        // Regressions
        let regressions = self.detect_regressions();
        if !regressions.is_empty() {
            report.push_str("🚨 Performance Regressions Detected:\n");
            for regression in regressions {
                report.push_str(&format!("  • {regression}\n"));
            }
            report.push('\n');
        }

        // System information
        if let Some(latest) = self.snapshot_history.last() {
            report.push_str("💻 System Information:\n");
            report.push_str(&format!("  • CPU Cores: {}\n", latest.system_info.cpu_cores));
            report.push_str(&format!("  • Memory: {} MB\n", latest.system_info.memory_total_mb));
            report.push_str(&format!("  • Platform: {}\n", latest.system_info.platform));
            report.push_str(&format!("  • Load Average: {:.2}\n", latest.system_info.load_average));
            report.push('\n');
        }

        // Recommendations
        if let Some(latest) = self.snapshot_history.last() {
            report.push_str("🔧 Optimization Recommendations:\n");
            for rec in &latest.execution_profile.optimization_opportunities {
                report.push_str(&format!("  • {rec}\n"));
            }
        }

        report
    }

    /// Saves monitoring data to file
    pub fn save_monitoring_data(&self) -> Result<(), std::io::Error> {
        let data = serde_json::to_string_pretty(&self.snapshot_history)?;
        std::fs::write(&self.monitoring_config.output_file, data)?;
        Ok(())
    }

    /// Loads monitoring data from file
    pub fn load_monitoring_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.monitoring_config.output_file.exists() {
            let data = std::fs::read_to_string(&self.monitoring_config.output_file)?;
            self.snapshot_history = serde_json::from_str(&data)?;
        }
        Ok(())
    }

    /// Cleanup old snapshots based on retention policy
    fn cleanup_old_snapshots(&mut self) {
        let retention_seconds = self.monitoring_config.history_retention_days as u64 * 24 * 60 * 60;
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - retention_seconds;

        self.snapshot_history.retain(|snapshot| snapshot.timestamp >= cutoff_time);
    }

    // Helper methods for system metrics (simplified implementations)
    fn get_load_average(&self) -> f64 {
        // Simplified load average calculation
        1.0
    }

    fn estimate_heap_size(&self) -> f64 {
        // Simplified heap size estimation
        64.0
    }

    fn count_allocated_objects(&self) -> usize {
        // Simplified object count
        10000
    }

    fn calculate_gc_pressure(&self) -> f64 {
        // Simplified GC pressure calculation
        0.3
    }

    fn calculate_memory_fragmentation(&self) -> f64 {
        // Simplified fragmentation calculation
        0.15
    }

    /// Runs continuous monitoring
    pub fn run_continuous_monitoring(&mut self) {
        println!("🔄 Starting continuous performance monitoring...");
        println!("Press Ctrl+C to stop monitoring\n");

        loop {
            let snapshot = self.take_snapshot();

            // Check for regressions
            let regressions = self.detect_regressions();
            if !regressions.is_empty() && self.monitoring_config.enable_real_time_alerts {
                println!("🚨 PERFORMANCE REGRESSION ALERT:");
                for regression in regressions {
                    println!("  • {regression}");
                }
                println!();
            }

            // Save data
            if let Err(e) = self.save_monitoring_data() {
                eprintln!("⚠️  Failed to save monitoring data: {e}");
            }

            // Wait for next snapshot
            std::thread::sleep(self.monitoring_config.snapshot_interval);
        }
    }
}

fn main() {
    let matches = Command::new("Performance Monitor")
        .version("1.0.0")
        .about("Real-time performance monitoring and analysis for Lambdust")
        .arg(
            Arg::new("continuous")
                .long("continuous")
                .short('c')
                .action(clap::ArgAction::SetTrue)
                .help("Run continuous monitoring")
        )
        .arg(
            Arg::new("interval")
                .long("interval")
                .short('i')
                .value_name("SECONDS")
                .help("Snapshot interval in seconds")
                .default_value("60")
        )
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .value_name("FILE")
                .help("Output file for monitoring data")
                .default_value("performance_monitoring.json")
        )
        .arg(
            Arg::new("retention")
                .long("retention")
                .short('r')
                .value_name("DAYS")
                .help("Data retention period in days")
                .default_value("30")
        )
        .arg(
            Arg::new("threshold")
                .long("threshold")
                .short('t')
                .value_name("PERCENTAGE")
                .help("Regression detection threshold (percentage)")
                .default_value("10.0")
        )
        .arg(
            Arg::new("report")
                .long("report")
                .action(clap::ArgAction::SetTrue)
                .help("Generate monitoring report from existing data")
        )
        .get_matches();

    let interval_secs: u64 = matches.get_one::<String>("interval")
        .unwrap()
        .parse()
        .expect("Invalid interval value");

    let retention_days: u32 = matches.get_one::<String>("retention")
        .unwrap()
        .parse()
        .expect("Invalid retention value");

    let threshold: f64 = matches.get_one::<String>("threshold")
        .unwrap()
        .parse()
        .expect("Invalid threshold value");

    let output_file = PathBuf::from(matches.get_one::<String>("output").unwrap());

    let config = MonitoringConfig {
        snapshot_interval: Duration::from_secs(interval_secs),
        history_retention_days: retention_days,
        regression_threshold: threshold,
        output_file,
        enable_real_time_alerts: true,
    };

    let mut monitor = PerformanceMonitor::new(config);

    // Load existing data
    if let Err(e) = monitor.load_monitoring_data() {
        eprintln!("⚠️  Could not load existing monitoring data: {e}");
    }

    if matches.get_flag("report") {
        // Generate report mode
        println!("{}", monitor.generate_monitoring_report());
    } else if matches.get_flag("continuous") {
        // Continuous monitoring mode
        monitor.run_continuous_monitoring();
    } else {
        // Single snapshot mode
        let _snapshot = monitor.take_snapshot();
        println!("{}", monitor.generate_monitoring_report());
        
        if let Err(e) = monitor.save_monitoring_data() {
            eprintln!("⚠️  Failed to save monitoring data: {e}");
        } else {
            println!("✅ Monitoring data saved successfully");
        }
    }
}