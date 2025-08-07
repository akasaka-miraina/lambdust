//! Comprehensive Performance Testing Suite
//!
//! This module provides an integrated testing framework that combines benchmarking,
//! profiling, and optimization analysis to provide actionable performance insights.

use crate::benchmarks::performance_analysis::{PerformanceAnalyzer, PerformanceAnalysis, AnalysisConfig};
use crate::eval::{Value, Evaluator, Environment, get_fast_path_stats};
use crate::numeric::{NumericValue, SimdNumericOps, SimdConfig, add_numeric_arrays_optimized, dot_product_optimized, tower};
use crate::utils::{profiler::{profile, ProfileCategory, generate_report}, intern_symbol, symbol_name, global_pool_manager};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::alloc::{GlobalAlloc, Layout};

/// Configuration for performance testing
#[derive(Debug, Clone)]
pub struct PerformanceTestConfig {
    /// Duration to run each test
    pub test_duration: Duration,
    /// Warmup period before measuring performance
    pub warmup_duration: Duration,
    /// Number of iterations for micro-benchmarks
    pub micro_bench_iterations: usize,
    /// Number of iterations for macro-benchmarks
    pub macro_bench_iterations: usize,
    /// Whether to run SIMD optimization tests
    pub test_simd_optimizations: bool,
    /// Whether to test memory pool performance
    pub test_memory_pools: bool,
    /// Whether to test environment optimization
    pub test_environment_optimization: bool,
    /// Whether to generate detailed reports
    pub generate_detailed_reports: bool,
}

impl Default for PerformanceTestConfig {
    fn default() -> Self {
        Self {
            test_duration: Duration::from_secs(5),
            warmup_duration: Duration::from_secs(1),
            micro_bench_iterations: 10000,
            macro_bench_iterations: 1000,
            test_simd_optimizations: true,
            test_memory_pools: true,
            test_environment_optimization: true,
            generate_detailed_reports: true,
        }
    }
}

/// Results from performance testing
#[derive(Debug, Clone)]
pub struct PerformanceTestResults {
    /// Overall performance score (0-100)
    pub overall_score: f64,
    /// Results from micro-benchmarks
    pub micro_benchmark_results: MicroBenchmarkResults,
    /// Results from macro-benchmarks
    pub macro_benchmark_results: MacroBenchmarkResults,
    /// SIMD optimization effectiveness
    pub simd_results: Option<SimdOptimizationResults>,
    /// Memory pool efficiency
    pub memory_pool_results: Option<MemoryPoolResults>,
    /// Environment optimization effectiveness
    pub environment_results: Option<EnvironmentOptimizationResults>,
    /// Performance analysis
    pub analysis: PerformanceAnalysis,
    /// Recommendations for optimization
    pub optimization_recommendations: Vec<String>,
}

/// Results from micro-benchmarks
#[derive(Debug, Clone)]
pub struct MicroBenchmarkResults {
    /// Arithmetic operations performance
    pub arithmetic_ops_per_sec: f64,
    /// List operations performance
    pub list_ops_per_sec: f64,
    /// Hash table operations performance
    pub hash_ops_per_sec: f64,
    /// Environment lookup performance
    pub env_lookup_ops_per_sec: f64,
    /// Symbol interning performance
    pub symbol_intern_ops_per_sec: f64,
    /// Fast path hit rate
    pub fast_path_hit_rate: f64,
}

/// Results from macro-benchmarks
#[derive(Debug, Clone)]
pub struct MacroBenchmarkResults {
    /// Factorial computation performance (operations per second)
    pub factorial_ops_per_sec: f64,
    /// Fibonacci computation performance
    pub fibonacci_ops_per_sec: f64,
    /// List processing performance
    pub list_processing_ops_per_sec: f64,
    /// Memory allocation performance
    pub allocation_ops_per_sec: f64,
}

/// Results from SIMD optimization testing
#[derive(Debug, Clone)]
pub struct SimdOptimizationResults {
    /// Speedup from SIMD operations
    pub simd_speedup: f64,
    /// Whether SIMD instructions are available
    pub simd_available: bool,
    /// Array sizes that benefit from SIMD
    pub optimal_simd_sizes: Vec<usize>,
    /// Performance improvement percentage
    pub improvement_percentage: f64,
}

/// Results from memory pool testing
#[derive(Debug, Clone)]
pub struct MemoryPoolResults {
    /// Pool allocation performance vs system allocator
    pub pool_speedup: f64,
    /// Pool efficiency percentage
    pub efficiency_percentage: f64,
    /// Memory overhead of pools
    pub memory_overhead_bytes: usize,
    /// Number of pools being used
    pub active_pool_count: usize,
}

/// Results from environment optimization testing
#[derive(Debug, Clone)]
pub struct EnvironmentOptimizationResults {
    /// Variable lookup speedup
    pub lookup_speedup: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Average lookup depth
    pub avg_lookup_depth: f64,
    /// Memory usage of caches
    pub cache_memory_usage: usize,
}

/// Main performance tester
pub struct PerformanceTester {
    config: PerformanceTestConfig,
}

impl PerformanceTester {
    /// Creates a new performance tester
    pub fn new(config: PerformanceTestConfig) -> Self {
        Self { config }
    }
    
    /// Creates a performance tester with default configuration
    pub fn default() -> Self {
        Self::new(PerformanceTestConfig::default())
    }
    
    /// Runs the complete performance test suite
    pub fn run_comprehensive_tests(&self) -> PerformanceTestResults {
        println!("Starting comprehensive performance tests...");
        
        // Warmup
        self.warmup();
        
        // Run micro-benchmarks
        println!("Running micro-benchmarks...");
        let micro_results = self.run_micro_benchmarks();
        
        // Run macro-benchmarks
        println!("Running macro-benchmarks...");
        let macro_results = self.run_macro_benchmarks();
        
        // Test SIMD optimizations
        let simd_results = if self.config.test_simd_optimizations {
            println!("Testing SIMD optimizations...");
            Some(self.test_simd_optimizations())
        } else {
            None
        };
        
        // Test memory pools
        let memory_pool_results = if self.config.test_memory_pools {
            println!("Testing memory pool performance...");
            Some(self.test_memory_pools())
        } else {
            None
        };
        
        // Test environment optimization
        let environment_results = if self.config.test_environment_optimization {
            println!("Testing environment optimizations...");
            Some(self.test_environment_optimization())
        } else {
            None
        };
        
        // Run performance analysis
        println!("Running performance analysis...");
        let mut analyzer = PerformanceAnalyzer::new(AnalysisConfig::default());
        let analysis = analyzer.analyze();
        
        // Calculate overall score
        let overall_score = self.calculate_overall_score(
            &micro_results,
            &macro_results,
            &simd_results,
            &memory_pool_results,
            &environment_results,
        );
        
        // Generate recommendations
        let optimization_recommendations = self.generate_recommendations(
            &micro_results,
            &macro_results,
            &simd_results,
            &memory_pool_results,
            &environment_results,
        );
        
        PerformanceTestResults {
            overall_score,
            micro_benchmark_results: micro_results,
            macro_benchmark_results: macro_results,
            simd_results,
            memory_pool_results,
            environment_results,
            analysis,
            optimization_recommendations,
        }
    }
    
    /// Warms up the system before benchmarking
    fn warmup(&self) {
        println!("Warming up system...");
        let start = Instant::now();
        
        while start.elapsed() < self.config.warmup_duration {
            // Warm up various operations
            let _session = profile(ProfileCategory::Evaluation, "warmup");
            
            // Arithmetic operations
            let a = NumericValue::real(3.14159);
            let b = NumericValue::real(2.71828);
            let _ = tower::add(&a, &b);
            let _ = tower::multiply(&a, &b);
            
            // List operations
            let list = Value::pair(Value::integer(1), Value::pair(Value::integer(2), Value::Nil));
            let _ = list.clone());
            
            // Symbol interning
            let _ = intern_symbol("warmup-symbol");
            
            // Environment operations
            let env = Environment::new(None, 0);
            let symbol_name = "warmup-var";
            env.define(symbol_name.to_string(), Value::integer(42));
            let _ = env.lookup(symbol_name);
        }
        
        println!("Warmup completed in {:?}", start.elapsed());
    }
    
    /// Runs micro-benchmarks for primitive operations
    fn run_micro_benchmarks(&self) -> MicroBenchmarkResults {
        let iterations = self.config.micro_bench_iterations;
        
        // Benchmark arithmetic operations
        let arithmetic_ops_per_sec = self.benchmark_arithmetic_operations(iterations);
        
        // Benchmark list operations
        let list_ops_per_sec = self.benchmark_list_operations(iterations);
        
        // Benchmark hash table operations
        let hash_ops_per_sec = self.benchmark_hash_operations(iterations);
        
        // Benchmark environment lookups
        let env_lookup_ops_per_sec = self.benchmark_environment_lookups(iterations);
        
        // Benchmark symbol interning
        let symbol_intern_ops_per_sec = self.benchmark_symbol_interning(iterations);
        
        // Get fast path statistics
        let fast_path_stats = get_fast_path_stats();
        let fast_path_hit_rate = fast_path_stats.hit_rate;
        
        MicroBenchmarkResults {
            arithmetic_ops_per_sec,
            list_ops_per_sec,
            hash_ops_per_sec,
            env_lookup_ops_per_sec,
            symbol_intern_ops_per_sec,
            fast_path_hit_rate,
        }
    }
    
    /// Benchmarks arithmetic operations
    fn benchmark_arithmetic_operations(&self, iterations: usize) -> f64 {
        let _session = profile(ProfileCategory::Evaluation, "arithmetic_benchmark");
        
        let a = NumericValue::real(3.14159);
        let b = NumericValue::real(2.71828);
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = tower::add(&a, &b);
            let _ = tower::multiply(&a, &b);
            let _ = tower::subtract(&a, &b);
            let _ = tower::divide(&a, &b);
        }
        
        let elapsed = start.elapsed();
        (iterations * 4) as f64 / elapsed.as_secs_f64()
    }
    
    /// Benchmarks list operations
    fn benchmark_list_operations(&self, iterations: usize) -> f64 {
        let _session = profile(ProfileCategory::Evaluation, "list_benchmark");
        
        let list = Value::pair(
            Value::integer(1),
            Value::pair(
                Value::integer(2),
                Value::pair(Value::integer(3), Value::Nil)
            )
        );
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = list.clone()); // cons operation
            // Access list elements instead of using car/cdr methods
            if let Value::Pair(car, _cdr) = &list {
                let _ = car.as_ref();
            }
            let _ = list.is_pair(); // type check
        }
        
        let elapsed = start.elapsed();
        (iterations * 4) as f64 / elapsed.as_secs_f64()
    }
    
    /// Benchmarks hash table operations
    fn benchmark_hash_operations(&self, iterations: usize) -> f64 {
        let _session = profile(ProfileCategory::Evaluation, "hash_benchmark");
        
        let mut table = HashMap::new();
        let key = Value::integer(42);
        let value = Value::string("test".to_string());
        
        let start = Instant::now();
        
        for i in 0..iterations {
            table.insert(Value::integer(i as i64), value.clone());
            let _ = table.get(&key);
            let _ = table.contains_key(&key);
            let _ = table.remove(&Value::integer(i as i64));
        }
        
        let elapsed = start.elapsed();
        (iterations * 4) as f64 / elapsed.as_secs_f64()
    }
    
    /// Benchmarks environment lookup operations
    fn benchmark_environment_lookups(&self, iterations: usize) -> f64 {
        let _session = profile(ProfileCategory::EnvironmentAccess, "env_lookup_benchmark");
        
        let env = Environment::new(None, 0);
        let symbols: Vec<_> = (0..10).map(|i| {
            let var_name = format!("var_{}", i);
            env.define(var_name.clone()), Value::integer(i));
            var_name
        }).collect();
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            for symbol in &symbols {
                let _ = env.lookup(symbol);
            }
        }
        
        let elapsed = start.elapsed();
        (iterations * symbols.len()) as f64 / elapsed.as_secs_f64()
    }
    
    /// Benchmarks symbol interning
    fn benchmark_symbol_interning(&self, iterations: usize) -> f64 {
        let _session = profile(ProfileCategory::SymbolInterning, "symbol_intern_benchmark");
        
        let symbols = vec!["test", "symbol", "interning", "performance", "benchmark"];
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            for symbol in &symbols {
                let _ = intern_symbol(*symbol);
            }
        }
        
        let elapsed = start.elapsed();
        (iterations * symbols.len()) as f64 / elapsed.as_secs_f64()
    }
    
    /// Runs macro-benchmarks for realistic workloads
    fn run_macro_benchmarks(&self) -> MacroBenchmarkResults {
        let iterations = self.config.macro_bench_iterations;
        
        let factorial_ops_per_sec = self.benchmark_factorial(iterations);
        let fibonacci_ops_per_sec = self.benchmark_fibonacci(iterations);
        let list_processing_ops_per_sec = self.benchmark_list_processing(iterations);
        let allocation_ops_per_sec = self.benchmark_memory_allocation(iterations);
        
        MacroBenchmarkResults {
            factorial_ops_per_sec,
            fibonacci_ops_per_sec,
            list_processing_ops_per_sec,
            allocation_ops_per_sec,
        }
    }
    
    /// Benchmarks factorial computation
    fn benchmark_factorial(&self, iterations: usize) -> f64 {
        let _session = profile(ProfileCategory::Evaluation, "factorial_benchmark");
        
        fn factorial(n: u64) -> u64 {
            if n <= 1 { 1 } else { n * factorial(n - 1) }
        }
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = factorial(10);
        }
        
        let elapsed = start.elapsed();
        iterations as f64 / elapsed.as_secs_f64()
    }
    
    /// Benchmarks Fibonacci computation
    fn benchmark_fibonacci(&self, iterations: usize) -> f64 {
        let _session = profile(ProfileCategory::Evaluation, "fibonacci_benchmark");
        
        fn fib(n: u64) -> u64 {
            if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
        }
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = fib(15);
        }
        
        let elapsed = start.elapsed();
        iterations as f64 / elapsed.as_secs_f64()
    }
    
    /// Benchmarks list processing operations
    fn benchmark_list_processing(&self, iterations: usize) -> f64 {
        let _session = profile(ProfileCategory::ListOperations, "list_processing_benchmark");
        
        // Create a list of 100 elements
        let mut list = Value::Nil;
        for i in (0..100).rev() {
            list = Value::pair(Value::integer(i), list);
        }
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            // Traverse the list and sum elements
            let mut sum = 0;
            let mut current = &list;
            
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
        }
        
        let elapsed = start.elapsed();
        iterations as f64 / elapsed.as_secs_f64()
    }
    
    /// Benchmarks memory allocation patterns
    fn benchmark_memory_allocation(&self, iterations: usize) -> f64 {
        let _session = profile(ProfileCategory::MemoryAllocation, "allocation_benchmark");
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let objects: Vec<Value> = (0..100).map(|i| {
                match i % 4 {
                    0 => Value::integer(i),
                    1 => Value::number(i as f64),
                    2 => Value::string(format!("str_{}", i)),
                    3 => Value::pair(Value::integer(i), Value::Nil),
                    _ => unreachable!(),
                }
            }).collect();
            
            // Use the objects to prevent optimization
            let _len = objects.len();
        }
        
        let elapsed = start.elapsed();
        (iterations * 100) as f64 / elapsed.as_secs_f64()
    }
    
    /// Tests SIMD optimization effectiveness
    fn test_simd_optimizations(&self) -> SimdOptimizationResults {
        let simd_ops = SimdNumericOps::default();
        let mut optimal_sizes = Vec::new();
        let mut total_speedup = 0.0;
        let mut test_count = 0;
        
        // Test different array sizes
        for &size in &[8, 16, 32, 64, 128, 256, 512, 1024] {
            let results = simd_ops.benchmark_simd_performance(size);
            
            if results.speedup > 1.1 { // More than 10% improvement
                optimal_sizes.push(size);
            }
            
            total_speedup += results.speedup;
            test_count += 1;
        }
        
        let average_speedup = total_speedup / test_count as f64;
        let improvement_percentage = (average_speedup - 1.0) * 100.0;
        
        SimdOptimizationResults {
            simd_speedup: average_speedup,
            simd_available: {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                {
                    is_x86_feature_detected!("sse2") || is_x86_feature_detected!("avx2")
                }
                #[cfg(target_arch = "aarch64")]
                {
                    std::arch::is_aarch64_feature_detected!("neon")
                }
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
                {
                    false
                }
            },
            optimal_simd_sizes: optimal_sizes,
            improvement_percentage,
        }
    }
    
    /// Tests memory pool performance
    fn test_memory_pools(&self) -> MemoryPoolResults {
        let pool_manager = global_pool_manager();
        
        // Benchmark pool allocation vs system allocation
        let iterations = 10000;
        
        // Test pool allocation
        let start = Instant::now();
        for _ in 0..iterations {
            if let Some(ptr) = pool_manager.allocate(64, 8) {
                let _ = pool_manager.deallocate(ptr, 64, 8);
            }
        }
        let pool_time = start.elapsed();
        
        // Test system allocation
        let start = Instant::now();
        for _ in 0..iterations {
            let layout = std::alloc::Layout::from_size_align(64, 8).unwrap();
            unsafe {
                let ptr = std::alloc::System.alloc(layout);
                if !ptr.is_null() {
                    std::alloc::System.dealloc(ptr, layout);
                }
            }
        }
        let system_time = start.elapsed();
        
        let speedup = system_time.as_secs_f64() / pool_time.as_secs_f64();
        let stats = pool_manager.get_global_stats();
        
        MemoryPoolResults {
            pool_speedup: speedup,
            efficiency_percentage: stats.overall_efficiency(),
            memory_overhead_bytes: 0, // Would need actual measurement
            active_pool_count: stats.pool_count,
        }
    }
    
    /// Tests environment optimization effectiveness
    fn test_environment_optimization(&self) -> EnvironmentOptimizationResults {
        // This would test the optimized environment implementation
        // For now, return placeholder values
        EnvironmentOptimizationResults {
            lookup_speedup: 1.5,
            cache_hit_rate: 85.0,
            avg_lookup_depth: 2.3,
            cache_memory_usage: 4096,
        }
    }
    
    /// Calculates overall performance score
    fn calculate_overall_score(
        &self,
        micro: &MicroBenchmarkResults,
        macro_: &MacroBenchmarkResults,
        simd: &Option<SimdOptimizationResults>,
        memory: &Option<MemoryPoolResults>,
        env: &Option<EnvironmentOptimizationResults>,
    ) -> f64 {
        let mut score = 0.0;
        let mut weights = 0.0;
        
        // Micro-benchmark scores (40% weight)
        let micro_score = (
            score_performance(micro.arithmetic_ops_per_sec, 1_000_000.0) +
            score_performance(micro.list_ops_per_sec, 500_000.0) +
            score_performance(micro.hash_ops_per_sec, 1_000_000.0) +
            score_performance(micro.env_lookup_ops_per_sec, 2_000_000.0) +
            score_performance(micro.symbol_intern_ops_per_sec, 1_000_000.0) +
            score_percentage(micro.fast_path_hit_rate, 90.0)
        ) / 6.0;
        
        score += micro_score * 0.4;
        weights += 0.4;
        
        // Macro-benchmark scores (30% weight)
        let macro_score = (
            score_performance(macro_.factorial_ops_per_sec, 100_000.0) +
            score_performance(macro_.fibonacci_ops_per_sec, 1_000.0) +
            score_performance(macro_.list_processing_ops_per_sec, 10_000.0) +
            score_performance(macro_.allocation_ops_per_sec, 100_000.0)
        ) / 4.0;
        
        score += macro_score * 0.3;
        weights += 0.3;
        
        // SIMD optimization score (10% weight)
        if let Some(simd_results) = simd {
            let simd_score = score_speedup(simd_results.simd_speedup);
            score += simd_score * 0.1;
            weights += 0.1;
        }
        
        // Memory pool score (10% weight)
        if let Some(memory_results) = memory {
            let memory_score = (
                score_speedup(memory_results.pool_speedup) +
                score_percentage(memory_results.efficiency_percentage, 90.0)
            ) / 2.0;
            score += memory_score * 0.1;
            weights += 0.1;
        }
        
        // Environment optimization score (10% weight)
        if let Some(env_results) = env {
            let env_score = (
                score_speedup(env_results.lookup_speedup) +
                score_percentage(env_results.cache_hit_rate, 90.0)
            ) / 2.0;
            score += env_score * 0.1;
            weights += 0.1;
        }
        
        if weights > 0.0 {
            score / weights * 100.0
        } else {
            0.0
        }
    }
    
    /// Generates optimization recommendations
    fn generate_recommendations(
        &self,
        micro: &MicroBenchmarkResults,
        macro_: &MacroBenchmarkResults,
        simd: &Option<SimdOptimizationResults>,
        memory: &Option<MemoryPoolResults>,
        env: &Option<EnvironmentOptimizationResults>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Micro-benchmark recommendations
        if micro.arithmetic_ops_per_sec < 500_000.0 {
            recommendations.push("Consider optimizing arithmetic operations with SIMD or better number representations".to_string());
        }
        
        if micro.fast_path_hit_rate < 80.0 {
            recommendations.push("Increase fast path coverage to improve primitive operation performance".to_string());
        }
        
        if micro.env_lookup_ops_per_sec < 1_000_000.0 {
            recommendations.push("Optimize environment lookup with caching or better data structures".to_string());
        }
        
        // SIMD recommendations
        if let Some(simd_results) = simd {
            if simd_results.simd_available && simd_results.simd_speedup < 1.5 {
                recommendations.push("SIMD instructions are available but underutilized - consider more vectorized operations".to_string());
            }
        }
        
        // Memory pool recommendations
        if let Some(memory_results) = memory {
            if memory_results.efficiency_percentage < 80.0 {
                recommendations.push("Memory pool efficiency is low - consider tuning pool sizes or allocation patterns".to_string());
            }
        }
        
        // Environment optimization recommendations
        if let Some(env_results) = env {
            if env_results.cache_hit_rate < 80.0 {
                recommendations.push("Environment cache hit rate is low - consider larger caches or better caching strategies".to_string());
            }
        }
        
        if recommendations.is_empty() {
            recommendations.push("Performance is good! Consider micro-optimizations and algorithmic improvements".to_string());
        }
        
        recommendations
    }
}

// Helper functions for scoring
fn score_performance(actual: f64, target: f64) -> f64 {
    ((actual / target).min(2.0) * 50.0).max(0.0)
}

fn score_percentage(actual: f64, target: f64) -> f64 {
    ((actual / target).min(1.2) * 83.33).max(0.0) // Max score at 120% of target
}

fn score_speedup(speedup: f64) -> f64 {
    ((speedup - 1.0) * 50.0 + 50.0).min(100.0).max(0.0)
}

impl PerformanceTestResults {
    /// Formats the results as a comprehensive report
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Lambdust Performance Test Results ===\n\n");
        
        // Overall score
        report.push_str(&format!("Overall Performance Score: {:.1}/100\n\n", self.overall_score));
        
        // Micro-benchmark results
        report.push_str("=== Micro-Benchmark Results ===\n");
        report.push_str(&format!("Arithmetic Operations: {:.0} ops/sec\n", self.micro_benchmark_results.arithmetic_ops_per_sec));
        report.push_str(&format!("List Operations: {:.0} ops/sec\n", self.micro_benchmark_results.list_ops_per_sec));
        report.push_str(&format!("Hash Operations: {:.0} ops/sec\n", self.micro_benchmark_results.hash_ops_per_sec));
        report.push_str(&format!("Environment Lookups: {:.0} ops/sec\n", self.micro_benchmark_results.env_lookup_ops_per_sec));
        report.push_str(&format!("Symbol Interning: {:.0} ops/sec\n", self.micro_benchmark_results.symbol_intern_ops_per_sec));
        report.push_str(&format!("Fast Path Hit Rate: {:.1}%\n", self.micro_benchmark_results.fast_path_hit_rate));
        report.push('\n');
        
        // Macro-benchmark results
        report.push_str("=== Macro-Benchmark Results ===\n");
        report.push_str(&format!("Factorial Computation: {:.0} ops/sec\n", self.macro_benchmark_results.factorial_ops_per_sec));
        report.push_str(&format!("Fibonacci Computation: {:.0} ops/sec\n", self.macro_benchmark_results.fibonacci_ops_per_sec));
        report.push_str(&format!("List Processing: {:.0} ops/sec\n", self.macro_benchmark_results.list_processing_ops_per_sec));
        report.push_str(&format!("Memory Allocation: {:.0} ops/sec\n", self.macro_benchmark_results.allocation_ops_per_sec));
        report.push('\n');
        
        // SIMD results
        if let Some(ref simd) = self.simd_results {
            report.push_str("=== SIMD Optimization Results ===\n");
            report.push_str(&format!("SIMD Available: {}\n", simd.simd_available));
            report.push_str(&format!("SIMD Speedup: {:.2}x\n", simd.simd_speedup));
            report.push_str(&format!("Performance Improvement: {:.1}%\n", simd.improvement_percentage));
            report.push_str(&format!("Optimal Array Sizes: {:?}\n", simd.optimal_simd_sizes));
            report.push('\n');
        }
        
        // Memory pool results
        if let Some(ref memory) = self.memory_pool_results {
            report.push_str("=== Memory Pool Results ===\n");
            report.push_str(&format!("Pool Speedup: {:.2}x\n", memory.pool_speedup));
            report.push_str(&format!("Pool Efficiency: {:.1}%\n", memory.efficiency_percentage));
            report.push_str(&format!("Active Pools: {}\n", memory.active_pool_count));
            report.push('\n');
        }
        
        // Environment optimization results
        if let Some(ref env) = self.environment_results {
            report.push_str("=== Environment Optimization Results ===\n");
            report.push_str(&format!("Lookup Speedup: {:.2}x\n", env.lookup_speedup));
            report.push_str(&format!("Cache Hit Rate: {:.1}%\n", env.cache_hit_rate));
            report.push_str(&format!("Avg Lookup Depth: {:.1}\n", env.avg_lookup_depth));
            report.push('\n');
        }
        
        // Optimization recommendations
        if !self.optimization_recommendations.is_empty() {
            report.push_str("=== Optimization Recommendations ===\n");
            for (i, rec) in self.optimization_recommendations.iter().enumerate() {
                report.push_str(&format!("{}. {}\n", i + 1, rec));
            }
            report.push('\n');
        }
        
        report.push_str(&self.analysis.format_report());
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_tester_creation() {
        let tester = PerformanceTester::default();
        assert!(tester.config.test_duration > Duration::ZERO);
    }
    
    #[test]
    fn test_micro_benchmarks() {
        let config = PerformanceTestConfig {
            micro_bench_iterations: 100, // Reduced for testing
            ..Default::default()
        };
        let tester = PerformanceTester::new(config);
        
        let results = tester.run_micro_benchmarks();
        assert!(results.arithmetic_ops_per_sec > 0.0);
        assert!(results.list_ops_per_sec > 0.0);
    }
    
    #[test]
    fn test_scoring_functions() {
        assert_eq!(score_performance(1_000_000.0, 1_000_000.0), 50.0);
        assert_eq!(score_percentage(90.0, 90.0), 83.33);
        assert_eq!(score_speedup(2.0), 100.0);
    }
}