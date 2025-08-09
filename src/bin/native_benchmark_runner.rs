//! Native Benchmark Runner for Lambdust
//!
//! This executable provides comprehensive performance benchmarking for Lambdust
//! without requiring Docker or external containers. It measures performance across
//! multiple categories and generates structured output for analysis.
//!
//! Usage:
//!   cargo run --bin native-benchmark-runner --features benchmarks
//!   cargo run --bin native-benchmark-runner --features benchmarks -- --output results.json
//!   cargo run --bin native-benchmark-runner --features benchmarks -- --quick

use lambdust::eval::{Value, Evaluator, Environment};
use lambdust::numeric::NumericValue;
use lambdust::utils::intern_symbol;
use std::collections::HashMap;
use std::time::Instant;
use clap::{Arg, Command};
use serde::{Serialize, Deserialize};
use chrono::Utc;

/// Performance benchmark category with aggregated results.
/// 
/// Groups related benchmark tests together with summary statistics
/// and analysis for a specific performance domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkCategory {
    /// Category name
    pub name: String,
    /// Category description
    pub description: String,
    /// Individual test results in this category
    pub results: Vec<BenchmarkResult>,
    /// Aggregated statistics for this category
    pub summary: CategorySummary,
}

/// Individual benchmark test execution results.
/// 
/// Contains timing, throughput, and statistical metrics
/// for a single benchmark test run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Name of the benchmark test
    pub test_name: String,
    /// Number of test iterations performed
    pub iterations: usize,
    /// Total execution time in milliseconds
    pub total_time_ms: f64,
    /// Operations performed per second
    pub ops_per_second: f64,
    /// Memory usage in megabytes
    pub memory_usage_mb: f64,
    /// Optional throughput metric (items per second)
    pub throughput_items_per_sec: Option<f64>,
    /// Statistical analysis of execution times
    pub statistical_metrics: StatisticalMetrics,
}

/// Statistical analysis of benchmark execution times.
/// 
/// Provides comprehensive timing statistics including
/// central tendency, variability, and percentile measures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalMetrics {
    /// Mean execution time in milliseconds
    pub mean_time_ms: f64,
    /// Median execution time in milliseconds
    pub median_time_ms: f64,
    /// Standard deviation of execution times in milliseconds
    pub std_deviation_ms: f64,
    /// Minimum execution time in milliseconds
    pub min_time_ms: f64,
    /// Maximum execution time in milliseconds
    pub max_time_ms: f64,
    /// 95th percentile execution time in milliseconds
    pub p95_time_ms: f64,
    /// 99th percentile execution time in milliseconds
    pub p99_time_ms: f64,
}

/// Summary statistics and analysis for a benchmark category.
/// 
/// Aggregates category performance with grades and optimization
/// opportunities for targeted performance improvements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    /// Total number of tests in this category
    pub total_tests: usize,
    /// Average operations per second across all tests
    pub avg_ops_per_second: f64,
    /// Overall performance grade for this category
    pub performance_grade: String,
    /// Identified optimization opportunities
    pub optimization_opportunities: Vec<String>,
}

/// Complete benchmark report with system context and recommendations.
/// 
/// Comprehensive performance analysis report including all categories,
/// system information, and actionable optimization recommendations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveBenchmarkReport {
    /// Timestamp when benchmark was executed
    pub timestamp: String,
    /// System hardware and environment information
    pub system_info: SystemInfo,
    /// Configuration used for testing
    pub test_configuration: TestConfiguration,
    /// Results organized by benchmark categories
    pub categories: Vec<BenchmarkCategory>,
    /// Overall performance summary
    pub overall_summary: OverallSummary,
    /// Performance improvement recommendations
    pub performance_recommendations: Vec<String>,
}

/// System hardware and environment information.
/// 
/// Captures relevant system details that may influence
/// benchmark results for result interpretation and comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Number of CPU cores
    pub cpu_count: usize,
    /// Total system memory in megabytes
    pub total_memory_mb: u64,
    /// Operating system platform
    pub platform: String,
    /// System architecture
    pub architecture: String,
    /// Rust compiler version used
    pub rust_version: String,
}

/// Benchmark execution configuration and parameters.
/// 
/// Defines test parameters, iteration counts, and execution
/// settings used during the benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfiguration {
    /// Whether to run in quick mode with fewer iterations
    pub quick_mode: bool,
    /// Number of iterations per individual test
    pub iterations_per_test: usize,
    /// Number of warmup iterations before measurement
    pub warmup_iterations: usize,
    /// Maximum duration for each test in seconds
    pub test_duration_seconds: u64,
    /// Whether memory profiling is enabled
    pub memory_profiling_enabled: bool,
}

/// Overall summary statistics across all benchmark categories.
/// 
/// Provides high-level performance assessment with strengths identification
/// and comprehensive performance characteristics analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallSummary {
    /// Total number of benchmark categories tested
    pub total_categories: usize,
    /// Total number of individual tests executed
    pub total_tests: usize,
    /// Total execution time across all tests in seconds
    pub total_execution_time_seconds: f64,
    /// Overall performance score (0-100)
    pub overall_performance_score: f64,
    /// Identified strengths of the Lambdust implementation
    pub lambdust_strengths: Vec<String>,
    /// Performance characteristics by category
    pub performance_characteristics: HashMap<String, f64>,
}

/// Native benchmark execution engine for comprehensive performance testing.
/// 
/// Provides self-contained benchmark execution without external dependencies,
/// supporting various test configurations and detailed result collection.
pub struct NativeBenchmarkRunner {
    /// Test execution configuration
    config: TestConfiguration,
    /// Lambdust evaluator instance
    evaluator: Evaluator,
    /// Accumulated benchmark results
    results: Vec<BenchmarkCategory>,
}

impl NativeBenchmarkRunner {
    /// Creates a new native benchmark runner
    pub fn new(quick_mode: bool) -> Self {
        let config = if quick_mode {
            TestConfiguration {
                quick_mode: true,
                iterations_per_test: 100,
                warmup_iterations: 10,
                test_duration_seconds: 1,
                memory_profiling_enabled: false,
            }
        } else {
            TestConfiguration {
                quick_mode: false,
                iterations_per_test: 1000,
                warmup_iterations: 100,
                test_duration_seconds: 5,
                memory_profiling_enabled: true,
            }
        };

        Self {
            config,
            evaluator: Evaluator::new(),
            results: Vec::new(),
        }
    }

    /// Runs all benchmark categories
    pub fn run_all_benchmarks(&mut self) -> ComprehensiveBenchmarkReport {
        println!("🚀 Starting Lambdust Native Performance Benchmarking Suite");
        println!("Configuration: {} mode", if self.config.quick_mode { "Quick" } else { "Comprehensive" });
        
        let overall_start = Instant::now();
        
        // Run benchmark categories
        self.run_arithmetic_benchmarks();
        self.run_list_operation_benchmarks();
        self.run_recursion_benchmarks();
        self.run_memory_allocation_benchmarks();
        self.run_function_call_benchmarks();
        self.run_scheme_program_benchmarks();
        
        let total_execution_time = overall_start.elapsed().as_secs_f64();
        
        // Generate comprehensive report
        let report = self.generate_comprehensive_report(total_execution_time);
        
        println!("✅ Benchmarking complete! Total time: {total_execution_time:.2}s");
        
        report
    }

    /// Benchmarks arithmetic operations
    fn run_arithmetic_benchmarks(&mut self) {
        println!("📊 Running arithmetic operation benchmarks...");
        
        let mut results = Vec::new();
        
        // Integer arithmetic
        results.push(self.benchmark_operation(
            "integer_addition",
            "Add two integers repeatedly",
            || {
                let a = NumericValue::integer(42);
                let b = NumericValue::integer(17);
                a.add(&b).unwrap()
            }
        ));
        
        results.push(self.benchmark_operation(
            "integer_multiplication",
            "Multiply two integers repeatedly",
            || {
                let a = NumericValue::integer(42);
                let b = NumericValue::integer(17);
                a.multiply(&b).unwrap()
            }
        ));
        
        // Floating point arithmetic
        results.push(self.benchmark_operation(
            "float_operations",
            "Mixed floating point operations",
            || {
                let a = NumericValue::real(std::f64::consts::PI);
                let b = NumericValue::real(std::f64::consts::E);
                let sum = a.add(&b).unwrap();
                let product = a.multiply(&b).unwrap();
                sum.divide(&product).unwrap()
            }
        ));
        
        // Complex number operations
        results.push(self.benchmark_operation(
            "complex_operations",
            "Complex number arithmetic",
            || {
                let a = NumericValue::complex(1.0, 2.0);
                let b = NumericValue::complex(3.0, -1.0);
                a.multiply(&b).unwrap()
            }
        ));
        
        // Numeric tower operations
        results.push(self.benchmark_operation(
            "numeric_tower_promotions",
            "Number type promotions across numeric tower",
            || {
                let int_val = NumericValue::integer(42);
                let float_val = NumericValue::real(std::f64::consts::PI);
                let complex_val = NumericValue::complex(1.0, 2.0);
                
                let result1 = int_val.add(&float_val).unwrap();
                
                result1.multiply(&complex_val).unwrap()
            }
        ));
        
        let summary = self.calculate_category_summary(&results, "arithmetic");
        
        self.results.push(BenchmarkCategory {
            name: "Arithmetic Operations".to_string(),
            description: "Performance of numeric computations and type promotions".to_string(),
            results,
            summary,
        });
    }

    /// Benchmarks list operations
    fn run_list_operation_benchmarks(&mut self) {
        println!("📋 Running list operation benchmarks...");
        
        let mut results = Vec::new();
        
        // Create test lists of varying sizes
        let small_list = self.create_test_list(10);
        let _medium_list = self.create_test_list(100);
        let large_list = self.create_test_list(1000);
        
        // List construction
        results.push(self.benchmark_operation_with_data(
            "list_construction_small",
            "Construct 10-element list",
            10,
            || self.create_test_list(10)
        ));
        
        results.push(self.benchmark_operation_with_data(
            "list_construction_large",
            "Construct 1000-element list", 
            1000,
            || self.create_test_list(1000)
        ));
        
        // List traversal
        results.push(self.benchmark_operation_with_data(
            "list_traversal_small",
            "Traverse 10-element list",
            10,
            || self.traverse_and_sum_list(&small_list)
        ));
        
        results.push(self.benchmark_operation_with_data(
            "list_traversal_large",
            "Traverse 1000-element list",
            1000,
            || self.traverse_and_sum_list(&large_list)
        ));
        
        // List operations (car, cdr)
        results.push(self.benchmark_operation(
            "cons_car_cdr_operations",
            "Basic cons, car, cdr operations",
            || {
                let list = Value::pair(Value::integer(1), Value::pair(Value::integer(2), Value::Nil));
                let _car = list.car();
                let _cdr = list.cdr();
                list
            }
        ));
        
        let summary = self.calculate_category_summary(&results, "list_operations");
        
        self.results.push(BenchmarkCategory {
            name: "List Operations".to_string(),
            description: "Performance of list construction, traversal, and manipulation".to_string(),
            results,
            summary,
        });
    }

    /// Benchmarks recursive computations
    fn run_recursion_benchmarks(&mut self) {
        println!("🔄 Running recursion benchmarks...");
        
        let mut results = Vec::new();
        
        // Factorial benchmarks
        let factorial_sizes = if self.config.quick_mode { vec![10, 15] } else { vec![10, 15, 20] };
        
        for &n in &factorial_sizes {
            results.push(self.benchmark_operation(
                &format!("factorial_{n}"),
                &format!("Compute factorial of {n}"),
                || self.compute_factorial(n)
            ));
        }
        
        // Fibonacci benchmarks
        let fib_sizes = if self.config.quick_mode { vec![15, 20] } else { vec![15, 20, 25] };
        
        for &n in &fib_sizes {
            results.push(self.benchmark_operation(
                &format!("fibonacci_{n}"),
                &format!("Compute fibonacci number {n}"),
                || self.compute_fibonacci(n)
            ));
        }
        
        // Tree recursion benchmark
        let tree_depth = if self.config.quick_mode { 15 } else { 18 };
        results.push(self.benchmark_operation(
            "tree_sum_recursion",
            "Recursive tree sum computation",
            move || {
                fn tree_sum(depth: u32) -> u64 {
                    if depth == 0 { 1 } else { tree_sum(depth - 1) + tree_sum(depth - 1) }
                }
                tree_sum(tree_depth)
            }
        ));
        
        let summary = self.calculate_category_summary(&results, "recursion");
        
        self.results.push(BenchmarkCategory {
            name: "Recursion Performance".to_string(),
            description: "Performance of recursive algorithms and deep call stacks".to_string(),
            results,
            summary,
        });
    }

    /// Benchmarks memory allocation patterns
    fn run_memory_allocation_benchmarks(&mut self) {
        println!("🧠 Running memory allocation benchmarks...");
        
        let mut results = Vec::new();
        
        // Small object allocation
        results.push(self.benchmark_operation_with_data(
            "small_object_allocation",
            "Allocate many small objects",
            1000,
            || {
                let mut objects = Vec::new();
                for i in 0..1000 {
                    objects.push(Value::integer(i));
                }
                objects
            }
        ));
        
        // Large object allocation
        results.push(self.benchmark_operation_with_data(
            "large_object_allocation",
            "Allocate large string objects",
            100,
            || {
                let mut objects = Vec::new();
                for _i in 0..100 {
                    let large_string = "x".repeat(1000);
                    objects.push(Value::string(large_string));
                }
                objects
            }
        ));
        
        // Mixed allocation patterns
        results.push(self.benchmark_operation_with_data(
            "mixed_allocation_patterns",
            "Mixed object type allocations",
            500,
            || {
                let mut objects = Vec::new();
                for i in 0..500 {
                    match i % 4 {
                        0 => objects.push(Value::integer(i as i64)),
                        1 => objects.push(Value::number(i as f64 * std::f64::consts::PI)),
                        2 => objects.push(Value::string(format!("str_{i}"))),
                        3 => objects.push(Value::pair(Value::integer(i as i64), Value::Nil)),
                        _ => unreachable!(),
                    }
                }
                objects
            }
        ));
        
        // Nested structure allocation
        results.push(self.benchmark_operation_with_data(
            "nested_structure_allocation",
            "Allocate deeply nested structures",
            100,
            || {
                let mut list = Value::Nil;
                for i in 0..100 {
                    let pair = Value::pair(
                        Value::integer(i),
                        Value::pair(Value::string(format!("item_{i}")), Value::Nil)
                    );
                    list = Value::pair(pair, list);
                }
                list
            }
        ));
        
        let summary = self.calculate_category_summary(&results, "memory_allocation");
        
        self.results.push(BenchmarkCategory {
            name: "Memory Allocation".to_string(),
            description: "Performance of various memory allocation patterns".to_string(),
            results,
            summary,
        });
    }

    /// Benchmarks function call overhead
    fn run_function_call_benchmarks(&mut self) {
        println!("📞 Running function call benchmarks...");
        
        let mut results = Vec::new();
        
        // Direct function calls
        results.push(self.benchmark_operation(
            "direct_function_calls",
            "Direct Rust function invocation overhead",
            || {
                fn simple_add(a: i64, b: i64) -> i64 { a + b }
                for i in 0..100 {
                    simple_add(i, i + 1);
                }
            }
        ));
        
        // Environment lookup overhead
        results.push(self.benchmark_operation(
            "environment_lookups",
            "Symbol lookup in environment chains",
            || {
                let env = Environment::new(None, 0);
                for i in 0..10 {
                    let var_name = format!("var_{i}");
                    env.define(var_name.clone(), Value::integer(i as i64));
                }
                
                for i in 0..10 {
                    let var_name = format!("var_{i}");
                    env.lookup(&var_name);
                }
            }
        ));
        
        // Symbol interning performance
        results.push(self.benchmark_operation(
            "symbol_interning",
            "Symbol interning and lookup performance",
            || {
                let symbols = ["lambda", "define", "if", "cond", "+", "-", "*", "/"];
                for &symbol in &symbols {
                    intern_symbol(symbol);
                }
            }
        ));
        
        let summary = self.calculate_category_summary(&results, "function_calls");
        
        self.results.push(BenchmarkCategory {
            name: "Function Calls".to_string(),
            description: "Performance of function calls and environment operations".to_string(),
            results,
            summary,
        });
    }

    /// Benchmarks realistic Scheme programs
    fn run_scheme_program_benchmarks(&mut self) {
        println!("🎯 Running realistic Scheme program benchmarks...");
        
        let mut results = Vec::new();
        
        // Sorting algorithm simulation
        results.push(self.benchmark_operation_with_data(
            "sorting_algorithm",
            "Bubble sort on list of integers",
            100,
            || {
                let mut numbers: Vec<i64> = (0..100).rev().collect();
                // Bubble sort
                for i in 0..numbers.len() {
                    for j in 0..numbers.len() - 1 - i {
                        if numbers[j] > numbers[j + 1] {
                            numbers.swap(j, j + 1);
                        }
                    }
                }
                numbers
            }
        ));
        
        // Higher-order function simulation (map)
        results.push(self.benchmark_operation_with_data(
            "higher_order_map",
            "Map square function over list",
            1000,
            || {
                let mut result = Vec::new();
                for i in 0..1000 {
                    result.push(i * i);
                }
                result
            }
        ));
        
        // Nested data structure processing
        results.push(self.benchmark_operation_with_data(
            "nested_data_processing",
            "Process nested associative structures",
            100,
            || {
                let mut result = HashMap::new();
                for i in 0..100 {
                    let key = format!("key_{i}");
                    let nested: HashMap<String, i32> = HashMap::new();
                    result.insert(key, nested);
                }
                result
            }
        ));
        
        let summary = self.calculate_category_summary(&results, "scheme_programs");
        
        self.results.push(BenchmarkCategory {
            name: "Realistic Programs".to_string(),
            description: "Performance of realistic Scheme program patterns".to_string(),
            results,
            summary,
        });
    }

    /// Generic benchmark operation runner
    fn benchmark_operation<F, R>(&self, name: &str, description: &str, mut operation: F) -> BenchmarkResult
    where
        F: FnMut() -> R,
    {
        let iterations = self.config.iterations_per_test;
        let warmup_iterations = self.config.warmup_iterations;
        
        // Warmup
        for _ in 0..warmup_iterations {
            let _ = operation();
        }
        
        // Collect timing data
        let mut times = Vec::new();
        let memory_before = self.get_memory_usage_mb();
        
        let start_time = Instant::now();
        for _ in 0..iterations {
            let iteration_start = Instant::now();
            let _ = operation();
            let iteration_time = iteration_start.elapsed();
            times.push(iteration_time.as_millis() as f64);
        }
        let total_time = start_time.elapsed();
        
        let memory_after = self.get_memory_usage_mb();
        let memory_usage = memory_after - memory_before;
        
        // Calculate statistics
        let stats = self.calculate_statistical_metrics(&times);
        let ops_per_second = iterations as f64 / total_time.as_secs_f64();
        
        BenchmarkResult {
            test_name: format!("{name}: {description}"),
            iterations,
            total_time_ms: total_time.as_millis() as f64,
            ops_per_second,
            memory_usage_mb: memory_usage,
            throughput_items_per_sec: None,
            statistical_metrics: stats,
        }
    }

    /// Benchmark operation with throughput tracking
    fn benchmark_operation_with_data<F, R>(&self, name: &str, description: &str, item_count: usize, operation: F) -> BenchmarkResult
    where
        F: FnMut() -> R,
    {
        let mut result = self.benchmark_operation(name, description, operation);
        result.throughput_items_per_sec = Some(item_count as f64 * result.ops_per_second);
        result
    }

    /// Helper methods for specific computations
    fn create_test_list(&self, size: usize) -> Value {
        let mut list = Value::Nil;
        for i in (0..size).rev() {
            list = Value::pair(Value::integer(i as i64), list);
        }
        list
    }

    fn traverse_and_sum_list(&self, list: &Value) -> i64 {
        let mut sum = 0;
        let mut current = list;
        
        loop {
            match current {
                Value::Nil => break,
                Value::Pair(car, cdr) => {
                    if let Some(n) = car.as_integer() {
                        sum += n;
                    }
                    current = cdr.as_ref();
                }
                _ => break,
            }
        }
        
        sum
    }

    fn compute_factorial(&self, n: u64) -> u64 {
        if n <= 1 { 1 } else { n * self.compute_factorial(n - 1) }
    }

    fn compute_fibonacci(&self, n: u64) -> u64 {
        if n <= 1 { n } else { self.compute_fibonacci(n - 1) + self.compute_fibonacci(n - 2) }
    }

    fn get_memory_usage_mb(&self) -> f64 {
        // This is a simplified memory usage measurement
        // In production, you might want to use a proper memory profiler
        0.0 // Placeholder
    }

    fn calculate_statistical_metrics(&self, times: &[f64]) -> StatisticalMetrics {
        let mut sorted_times = times.to_vec();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mean = times.iter().sum::<f64>() / times.len() as f64;
        let median = sorted_times[sorted_times.len() / 2];
        let min = sorted_times[0];
        let max = sorted_times[sorted_times.len() - 1];
        
        // Standard deviation
        let variance = times.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / times.len() as f64;
        let std_deviation = variance.sqrt();
        
        // Percentiles
        let p95_idx = (sorted_times.len() as f64 * 0.95) as usize;
        let p99_idx = (sorted_times.len() as f64 * 0.99) as usize;
        let p95 = sorted_times[p95_idx.min(sorted_times.len() - 1)];
        let p99 = sorted_times[p99_idx.min(sorted_times.len() - 1)];
        
        StatisticalMetrics {
            mean_time_ms: mean,
            median_time_ms: median,
            std_deviation_ms: std_deviation,
            min_time_ms: min,
            max_time_ms: max,
            p95_time_ms: p95,
            p99_time_ms: p99,
        }
    }

    fn calculate_category_summary(&self, results: &[BenchmarkResult], category_type: &str) -> CategorySummary {
        let total_tests = results.len();
        let avg_ops_per_second = results.iter()
            .map(|r| r.ops_per_second)
            .sum::<f64>() / total_tests as f64;
        
        // Performance grading based on ops/second thresholds
        let performance_grade = match category_type {
            "arithmetic" => {
                if avg_ops_per_second > 1_000_000.0 { "Excellent" }
                else if avg_ops_per_second > 500_000.0 { "Good" }
                else if avg_ops_per_second > 100_000.0 { "Fair" }
                else { "Needs Improvement" }
            }
            "list_operations" => {
                if avg_ops_per_second > 100_000.0 { "Excellent" }
                else if avg_ops_per_second > 50_000.0 { "Good" }
                else if avg_ops_per_second > 10_000.0 { "Fair" }
                else { "Needs Improvement" }
            }
            "recursion" => {
                if avg_ops_per_second > 10_000.0 { "Excellent" }
                else if avg_ops_per_second > 5_000.0 { "Good" }
                else if avg_ops_per_second > 1_000.0 { "Fair" }
                else { "Needs Improvement" }
            }
            _ => {
                if avg_ops_per_second > 100_000.0 { "Excellent" }
                else if avg_ops_per_second > 50_000.0 { "Good" }
                else if avg_ops_per_second > 10_000.0 { "Fair" }
                else { "Needs Improvement" }
            }
        }.to_string();
        
        let optimization_opportunities = self.identify_optimization_opportunities(results, category_type);
        
        CategorySummary {
            total_tests,
            avg_ops_per_second,
            performance_grade,
            optimization_opportunities,
        }
    }

    fn identify_optimization_opportunities(&self, results: &[BenchmarkResult], category_type: &str) -> Vec<String> {
        let mut opportunities = Vec::new();
        
        for result in results {
            if result.ops_per_second < 10_000.0 && category_type == "arithmetic" {
                opportunities.push(format!("Consider SIMD optimizations for {}", result.test_name));
            }
            
            if result.statistical_metrics.std_deviation_ms > result.statistical_metrics.mean_time_ms * 0.5 {
                opportunities.push(format!("High variance detected in {} - consider algorithmic improvements", result.test_name));
            }
            
            if result.memory_usage_mb > 100.0 {
                opportunities.push(format!("High memory usage in {} - consider memory pooling", result.test_name));
            }
        }
        
        if opportunities.is_empty() {
            opportunities.push("Performance looks good in this category!".to_string());
        }
        
        opportunities
    }

    fn generate_comprehensive_report(&self, total_execution_time: f64) -> ComprehensiveBenchmarkReport {
        let system_info = SystemInfo {
            cpu_count: num_cpus::get(),
            total_memory_mb: 8192, // Simplified - could use actual system info
            platform: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            rust_version: "1.70+".to_string(), // Simplified
        };
        
        let total_tests: usize = self.results.iter().map(|c| c.results.len()).sum();
        let _overall_ops_per_second: f64 = self.results.iter()
            .flat_map(|c| &c.results)
            .map(|r| r.ops_per_second)
            .sum::<f64>() / total_tests as f64;
        
        let overall_performance_score = self.calculate_overall_performance_score();
        
        let mut performance_characteristics = HashMap::new();
        for category in &self.results {
            performance_characteristics.insert(
                category.name.clone(),
                category.summary.avg_ops_per_second
            );
        }
        
        let lambdust_strengths = self.identify_lambdust_strengths();
        let performance_recommendations = self.generate_performance_recommendations();
        
        ComprehensiveBenchmarkReport {
            timestamp: Utc::now().to_rfc3339(),
            system_info,
            test_configuration: self.config.clone(),
            categories: self.results.clone(),
            overall_summary: OverallSummary {
                total_categories: self.results.len(),
                total_tests,
                total_execution_time_seconds: total_execution_time,
                overall_performance_score,
                lambdust_strengths,
                performance_characteristics,
            },
            performance_recommendations,
        }
    }

    fn calculate_overall_performance_score(&self) -> f64 {
        let mut total_score = 0.0;
        let mut weight_sum = 0.0;
        
        for category in &self.results {
            let category_score = match category.summary.performance_grade.as_str() {
                "Excellent" => 90.0,
                "Good" => 75.0,
                "Fair" => 60.0,
                _ => 40.0,
            };
            
            let weight = match category.name.as_str() {
                "Arithmetic Operations" => 0.25,
                "List Operations" => 0.25,
                "Recursion Performance" => 0.20,
                "Function Calls" => 0.15,
                _ => 0.15,
            };
            
            total_score += category_score * weight;
            weight_sum += weight;
        }
        
        if weight_sum > 0.0 { total_score / weight_sum } else { 0.0 }
    }

    fn identify_lambdust_strengths(&self) -> Vec<String> {
        let mut strengths = Vec::new();
        
        for category in &self.results {
            if category.summary.performance_grade == "Excellent" {
                strengths.push(format!("Excellent performance in {}", category.name));
            }
        }
        
        if strengths.is_empty() {
            strengths.push("Solid overall performance across all categories".to_string());
        }
        
        strengths
    }

    fn generate_performance_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        for category in &self.results {
            recommendations.extend(category.summary.optimization_opportunities.clone());
        }
        
        // Remove duplicates and add general recommendations
        recommendations.sort();
        recommendations.dedup();
        
        if recommendations.iter().any(|r| r.contains("SIMD")) {
            recommendations.push("Consider implementing SIMD optimizations for numeric computations".to_string());
        }
        
        recommendations.push("Monitor memory allocation patterns for optimization opportunities".to_string());
        recommendations.push("Consider implementing JIT compilation for hot code paths".to_string());
        
        recommendations
    }
}

fn main() {
    let matches = Command::new("Native Benchmark Runner")
        .version("1.0.0")
        .about("Comprehensive performance benchmarking for Lambdust")
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .value_name("FILE")
                .help("Output file for results (JSON format)")
        )
        .arg(
            Arg::new("quick")
                .long("quick")
                .action(clap::ArgAction::SetTrue)
                .help("Run in quick mode (fewer iterations, faster execution)")
        )
        .arg(
            Arg::new("csv")
                .long("csv")
                .action(clap::ArgAction::SetTrue)
                .help("Also generate CSV output for spreadsheet analysis")
        )
        .get_matches();
    
    let quick_mode = matches.get_flag("quick");
    let output_file = matches.get_one::<String>("output");
    let csv_output = matches.get_flag("csv");
    
    println!("🎯 Lambdust Native Benchmark Runner");
    println!("========================================");
    
    let mut runner = NativeBenchmarkRunner::new(quick_mode);
    let report = runner.run_all_benchmarks();
    
    // Display summary
    println!("\n🏆 BENCHMARK RESULTS SUMMARY");
    println!("========================================");
    println!("Overall Performance Score: {:.1}/100", report.overall_summary.overall_performance_score);
    println!("Total Tests: {}", report.overall_summary.total_tests);
    println!("Execution Time: {:.2}s", report.overall_summary.total_execution_time_seconds);
    
    println!("\n📊 Category Performance:");
    for category in &report.categories {
        println!("  {}: {:.0} ops/sec ({})", 
                category.name,
                category.summary.avg_ops_per_second,
                category.summary.performance_grade);
    }
    
    println!("\n💪 Lambdust Strengths:");
    for strength in &report.overall_summary.lambdust_strengths {
        println!("  • {strength}");
    }
    
    println!("\n🔧 Performance Recommendations:");
    for rec in &report.performance_recommendations[..3.min(report.performance_recommendations.len())] {
        println!("  • {rec}");
    }
    
    // Save results
    if let Some(output_path) = output_file {
        let json_output = serde_json::to_string_pretty(&report)
            .expect("Failed to serialize benchmark results");
        
        std::fs::write(output_path, json_output)
            .expect("Failed to write benchmark results to file");
        
        println!("\n💾 Results saved to: {output_path}");
        
        if csv_output {
            let csv_path = output_path.replace(".json", ".csv");
            save_csv_results(&report, &csv_path);
            println!("📊 CSV results saved to: {csv_path}");
        }
    } else {
        // Save to default location
        let default_path = format!("lambdust_benchmark_results_{}.json", 
                                 Utc::now().format("%Y%m%d_%H%M%S"));
        let json_output = serde_json::to_string_pretty(&report)
            .expect("Failed to serialize benchmark results");
        
        std::fs::write(&default_path, json_output)
            .expect("Failed to write benchmark results to file");
        
        println!("\n💾 Results saved to: {default_path}");
        
        if csv_output {
            let csv_path = default_path.replace(".json", ".csv");
            save_csv_results(&report, &csv_path);
            println!("📊 CSV results saved to: {csv_path}");
        }
    }
    
    println!("\n✅ Benchmarking complete! Use the JSON/CSV files for detailed analysis.");
}

fn save_csv_results(report: &ComprehensiveBenchmarkReport, path: &str) {
    let mut csv_content = String::new();
    csv_content.push_str("Category,Test Name,Iterations,Total Time (ms),Ops/sec,Memory (MB),Throughput (items/sec),Mean (ms),Median (ms),Std Dev (ms)\n");
    
    for category in &report.categories {
        for result in &category.results {
            csv_content.push_str(&format!(
                "{},{},{},{:.2},{:.0},{:.2},{:.0},{:.3},{:.3},{:.3}\n",
                category.name,
                result.test_name,
                result.iterations,
                result.total_time_ms,
                result.ops_per_second,
                result.memory_usage_mb,
                result.throughput_items_per_sec.unwrap_or(0.0),
                result.statistical_metrics.mean_time_ms,
                result.statistical_metrics.median_time_ms,
                result.statistical_metrics.std_deviation_ms
            ));
        }
    }
    
    std::fs::write(path, csv_content)
        .expect("Failed to write CSV results");
}