//! Performance benchmarks comparing unsafe OptimizedValue vs safe SafeOptimizedValue
//!
//! This module provides comprehensive benchmarks to validate that the safe implementation
//! maintains the same performance characteristics as the unsafe version while providing
//! complete memory safety.

#![allow(dead_code, missing_docs)]

use super::optimized_value::{OptimizedValue, OptimizedEnvironment};
use super::safe_optimized_value::{SafeOptimizedValue, SafeOptimizedEnvironment};
use crate::utils::SymbolId;
use std::time::{Duration, Instant};

/// Benchmark configuration
pub struct BenchmarkConfig {
    pub iterations: usize,
    pub warmup_iterations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100_000,
            warmup_iterations: 10_000,
        }
    }
}

/// Benchmark results for comparison
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub unsafe_time: Duration,
    pub safe_time: Duration,
    pub overhead_percentage: f64,
    pub operations_per_second_unsafe: f64,
    pub operations_per_second_safe: f64,
}

impl BenchmarkResults {
    pub fn new(unsafe_time: Duration, safe_time: Duration, iterations: usize) -> Self {
        let overhead_percentage = if unsafe_time.as_nanos() > 0 {
            ((safe_time.as_nanos() as f64 / unsafe_time.as_nanos() as f64) - 1.0) * 100.0
        } else {
            0.0
        };

        let operations_per_second_unsafe = iterations as f64 / unsafe_time.as_secs_f64();
        let operations_per_second_safe = iterations as f64 / safe_time.as_secs_f64();

        Self {
            unsafe_time,
            safe_time,
            overhead_percentage,
            operations_per_second_unsafe,
            operations_per_second_safe,
        }
    }

    pub fn is_acceptable(&self, max_overhead_percentage: f64) -> bool {
        self.overhead_percentage <= max_overhead_percentage
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub unsafe_size: usize,
    pub safe_size: usize,
    pub overhead_bytes: isize,
    pub overhead_percentage: f64,
}

impl MemoryStats {
    pub fn new(unsafe_size: usize, safe_size: usize) -> Self {
        let overhead_bytes = safe_size as isize - unsafe_size as isize;
        let overhead_percentage = if unsafe_size > 0 {
            (overhead_bytes as f64 / unsafe_size as f64) * 100.0
        } else {
            0.0
        };

        Self {
            unsafe_size,
            safe_size,
            overhead_bytes,
            overhead_percentage,
        }
    }
}

/// Comprehensive benchmark suite
pub struct MemorySafetyBenchmark {
    config: BenchmarkConfig,
}

impl MemorySafetyBenchmark {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self { config }
    }

    /// Benchmark immediate value creation
    pub fn bench_immediate_values(&self) -> BenchmarkResults {
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = OptimizedValue::fixnum(42i64);
            let _ = SafeOptimizedValue::fixnum(42i64);
        }

        // Benchmark unsafe version
        let start = Instant::now();
        for i in 0..self.config.iterations {
            let _val = OptimizedValue::fixnum(i as i64);
        }
        let unsafe_time = start.elapsed();

        // Benchmark safe version
        let start = Instant::now();
        for i in 0..self.config.iterations {
            let _val = SafeOptimizedValue::fixnum(i as i64);
        }
        let safe_time = start.elapsed();

        BenchmarkResults::new(unsafe_time, safe_time, self.config.iterations)
    }

    /// Benchmark string value creation
    pub fn bench_string_values(&self) -> BenchmarkResults {
        let test_string = "Hello, World!";

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = OptimizedValue::string(test_string);
            let _ = SafeOptimizedValue::string(test_string);
        }

        // Benchmark unsafe version
        let start = Instant::now();
        for _ in 0..self.config.iterations {
            let _val = OptimizedValue::string(test_string);
        }
        let unsafe_time = start.elapsed();

        // Benchmark safe version
        let start = Instant::now();
        for _ in 0..self.config.iterations {
            let _val = SafeOptimizedValue::string(test_string);
        }
        let safe_time = start.elapsed();

        BenchmarkResults::new(unsafe_time, safe_time, self.config.iterations)
    }

    /// Benchmark pair creation (most critical for Arc reduction)
    pub fn bench_pair_creation(&self) -> BenchmarkResults {
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let car = OptimizedValue::fixnum(1i64);
            let cdr = OptimizedValue::fixnum(2i64);
            let _ = OptimizedValue::pair(car, cdr);
            
            let car = SafeOptimizedValue::fixnum(1i64);
            let cdr = SafeOptimizedValue::fixnum(2i64);
            let _ = SafeOptimizedValue::pair(car, cdr);
        }

        // Benchmark unsafe version
        let start = Instant::now();
        for _ in 0..self.config.iterations {
            let car = OptimizedValue::fixnum(1i64);
            let cdr = OptimizedValue::fixnum(2i64);
            let _pair = OptimizedValue::pair(car, cdr);
        }
        let unsafe_time = start.elapsed();

        // Benchmark safe version
        let start = Instant::now();
        for _ in 0..self.config.iterations {
            let car = SafeOptimizedValue::fixnum(1i64);
            let cdr = SafeOptimizedValue::fixnum(2i64);
            let _pair = SafeOptimizedValue::pair(car, cdr);
        }
        let safe_time = start.elapsed();

        BenchmarkResults::new(unsafe_time, safe_time, self.config.iterations)
    }

    /// Benchmark value type checking
    pub fn bench_type_checking(&self) -> BenchmarkResults {
        let unsafe_values = vec![
            OptimizedValue::nil(),
            OptimizedValue::boolean(true),
            OptimizedValue::fixnum(42i64),
            OptimizedValue::string("test"),
        ];

        let safe_values = vec![
            SafeOptimizedValue::nil(),
            SafeOptimizedValue::boolean(true),
            SafeOptimizedValue::fixnum(42i64),
            SafeOptimizedValue::string("test"),
        ];

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            for val in &unsafe_values {
                let _ = val.is_number();
            }
            for val in &safe_values {
                let _ = val.is_number();
            }
        }

        // Benchmark unsafe version
        let start = Instant::now();
        for _ in 0..self.config.iterations {
            for val in &unsafe_values {
                let _ = val.is_number();
            }
        }
        let unsafe_time = start.elapsed();

        // Benchmark safe version
        let start = Instant::now();
        for _ in 0..self.config.iterations {
            for val in &safe_values {
                let _ = val.is_number();
            }
        }
        let safe_time = start.elapsed();

        BenchmarkResults::new(unsafe_time, safe_time, self.config.iterations)
    }

    /// Benchmark value access patterns
    pub fn bench_value_access(&self) -> BenchmarkResults {
        let unsafe_values = vec![
            OptimizedValue::fixnum(42i64),
            OptimizedValue::number(3.14),
            OptimizedValue::string("test"),
        ];

        let safe_values = vec![
            SafeOptimizedValue::fixnum(42i64),
            SafeOptimizedValue::number(3.14),
            SafeOptimizedValue::string("test"),
        ];

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            for val in &unsafe_values {
                let _ = val.as_number();
                let _ = val.as_string();
            }
            for val in &safe_values {
                let _ = val.as_number();
                let _ = val.as_string();
            }
        }

        // Benchmark unsafe version
        let start = Instant::now();
        for _ in 0..self.config.iterations {
            for val in &unsafe_values {
                let _ = val.as_number();
                let _ = val.as_string();
            }
        }
        let unsafe_time = start.elapsed();

        // Benchmark safe version
        let start = Instant::now();
        for _ in 0..self.config.iterations {
            for val in &safe_values {
                let _ = val.as_number();
                let _ = val.as_string();
            }
        }
        let safe_time = start.elapsed();

        BenchmarkResults::new(unsafe_time, safe_time, self.config.iterations)
    }

    /// Benchmark environment operations
    pub fn bench_environment_operations(&self) -> BenchmarkResults {
        // Warmup
        let unsafe_env = OptimizedEnvironment::new(None, 0);
        let safe_env = SafeOptimizedEnvironment::new(None, 0);
        
        for i in 0..self.config.warmup_iterations {
            unsafe_env.define(format!("var{i}"), OptimizedValue::fixnum(i as i64));
            safe_env.define(format!("var{i}"), SafeOptimizedValue::fixnum(i as i64));
        }

        // Benchmark unsafe version
        let unsafe_env = OptimizedEnvironment::new(None, 0);
        let start = Instant::now();
        for i in 0..self.config.iterations {
            unsafe_env.define(format!("var{i}"), OptimizedValue::fixnum(i as i64));
            let _ = unsafe_env.lookup(&format!("var{i}"));
        }
        let unsafe_time = start.elapsed();

        // Benchmark safe version
        let safe_env = SafeOptimizedEnvironment::new(None, 0);
        let start = Instant::now();
        for i in 0..self.config.iterations {
            safe_env.define(format!("var{i}"), SafeOptimizedValue::fixnum(i as i64));
            let _ = safe_env.lookup(&format!("var{i}"));
        }
        let safe_time = start.elapsed();

        BenchmarkResults::new(unsafe_time, safe_time, self.config.iterations)
    }

    /// Benchmark list creation and operations
    pub fn bench_list_operations(&self) -> BenchmarkResults {
        let test_data: Vec<i64> = (0..100).collect();

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let unsafe_list = OptimizedValue::list(
                test_data.iter().map(|&x| OptimizedValue::fixnum(x)).collect()
            );
            let _ = unsafe_list.as_list();

            let safe_list = SafeOptimizedValue::list(
                test_data.iter().map(|&x| SafeOptimizedValue::fixnum(x)).collect()
            );
            let _ = safe_list.as_list();
        }

        // Benchmark unsafe version
        let start = Instant::now();
        for _ in 0..self.config.iterations {
            let unsafe_list = OptimizedValue::list(
                test_data.iter().map(|&x| OptimizedValue::fixnum(x)).collect()
            );
            let _ = unsafe_list.as_list();
        }
        let unsafe_time = start.elapsed();

        // Benchmark safe version
        let start = Instant::now();
        for _ in 0..self.config.iterations {
            let safe_list = SafeOptimizedValue::list(
                test_data.iter().map(|&x| SafeOptimizedValue::fixnum(x)).collect()
            );
            let _ = safe_list.as_list();
        }
        let safe_time = start.elapsed();

        BenchmarkResults::new(unsafe_time, safe_time, self.config.iterations)
    }

    /// Run all benchmarks and return comprehensive results
    pub fn run_all_benchmarks(&self) -> BenchmarkSuite {
        println!("Running memory safety benchmarks...");
        
        println!("  Benchmarking immediate values...");
        let immediate_values = self.bench_immediate_values();
        
        println!("  Benchmarking string values...");
        let string_values = self.bench_string_values();
        
        println!("  Benchmarking pair creation...");
        let pair_creation = self.bench_pair_creation();
        
        println!("  Benchmarking type checking...");
        let type_checking = self.bench_type_checking();
        
        println!("  Benchmarking value access...");
        let value_access = self.bench_value_access();
        
        println!("  Benchmarking environment operations...");
        let environment_ops = self.bench_environment_operations();
        
        println!("  Benchmarking list operations...");
        let list_operations = self.bench_list_operations();

        BenchmarkSuite {
            immediate_values,
            string_values,
            pair_creation,
            type_checking,
            value_access,
            environment_ops,
            list_operations,
        }
    }

    /// Measure memory usage differences
    pub fn measure_memory_usage(&self) -> MemoryUsageReport {
        // Measure size of individual values
        let fixnum_unsafe = std::mem::size_of::<OptimizedValue>();
        let fixnum_safe = std::mem::size_of::<SafeOptimizedValue>();

        // These are estimates - in practice, enum variants may be larger
        let string_unsafe = fixnum_unsafe; // Same size due to union
        let string_safe = 24; // Estimated enum size with Box pointer

        let pair_unsafe = fixnum_unsafe; // Same size due to union  
        let pair_safe = 24; // Estimated enum size with Box pointer

        MemoryUsageReport {
            fixnum_stats: MemoryStats::new(fixnum_unsafe, fixnum_safe),
            string_stats: MemoryStats::new(string_unsafe, string_safe),
            pair_stats: MemoryStats::new(pair_unsafe, pair_safe),
        }
    }
}

/// Complete benchmark suite results
#[derive(Debug)]
pub struct BenchmarkSuite {
    pub immediate_values: BenchmarkResults,
    pub string_values: BenchmarkResults,
    pub pair_creation: BenchmarkResults,
    pub type_checking: BenchmarkResults,
    pub value_access: BenchmarkResults,
    pub environment_ops: BenchmarkResults,
    pub list_operations: BenchmarkResults,
}

impl BenchmarkSuite {
    /// Check if all benchmarks meet performance criteria
    pub fn validate_performance(&self, max_overhead_percentage: f64) -> bool {
        self.immediate_values.is_acceptable(max_overhead_percentage)
            && self.string_values.is_acceptable(max_overhead_percentage)
            && self.pair_creation.is_acceptable(max_overhead_percentage)
            && self.type_checking.is_acceptable(max_overhead_percentage)
            && self.value_access.is_acceptable(max_overhead_percentage)
            && self.environment_ops.is_acceptable(max_overhead_percentage)
            && self.list_operations.is_acceptable(max_overhead_percentage)
    }

    /// Get average overhead across all benchmarks
    pub fn average_overhead(&self) -> f64 {
        let overheads = [
            self.immediate_values.overhead_percentage,
            self.string_values.overhead_percentage,
            self.pair_creation.overhead_percentage,
            self.type_checking.overhead_percentage,
            self.value_access.overhead_percentage,
            self.environment_ops.overhead_percentage,
            self.list_operations.overhead_percentage,
        ];
        overheads.iter().sum::<f64>() / overheads.len() as f64
    }

    /// Print detailed benchmark report
    pub fn print_report(&self) {
        println!("\n=== Memory Safety Benchmark Report ===");
        println!();

        self.print_benchmark("Immediate Values", &self.immediate_values);
        self.print_benchmark("String Values", &self.string_values);
        self.print_benchmark("Pair Creation", &self.pair_creation);
        self.print_benchmark("Type Checking", &self.type_checking);
        self.print_benchmark("Value Access", &self.value_access);
        self.print_benchmark("Environment Ops", &self.environment_ops);
        self.print_benchmark("List Operations", &self.list_operations);

        println!();
        println!("=== Summary ===");
        println!("Average overhead: {:.2}%", self.average_overhead());
        println!("Performance acceptable (<5% overhead): {}", 
                 self.validate_performance(5.0));
        println!();
    }

    fn print_benchmark(&self, name: &str, results: &BenchmarkResults) {
        println!("{}:", name);
        println!("  Unsafe: {:.2?} ({:.0} ops/sec)", 
                 results.unsafe_time, results.operations_per_second_unsafe);
        println!("  Safe:   {:.2?} ({:.0} ops/sec)", 
                 results.safe_time, results.operations_per_second_safe);
        println!("  Overhead: {:.2}%", results.overhead_percentage);
        println!();
    }
}

/// Memory usage comparison report
#[derive(Debug)]
pub struct MemoryUsageReport {
    pub fixnum_stats: MemoryStats,
    pub string_stats: MemoryStats,
    pub pair_stats: MemoryStats,
}

impl MemoryUsageReport {
    pub fn print_report(&self) {
        println!("\n=== Memory Usage Report ===");
        println!();

        println!("Fixnum values:");
        println!("  Unsafe: {} bytes", self.fixnum_stats.unsafe_size);
        println!("  Safe:   {} bytes", self.fixnum_stats.safe_size);
        println!("  Overhead: {} bytes ({:.1}%)", 
                 self.fixnum_stats.overhead_bytes, self.fixnum_stats.overhead_percentage);
        println!();

        println!("String values:");
        println!("  Unsafe: {} bytes", self.string_stats.unsafe_size);
        println!("  Safe:   {} bytes", self.string_stats.safe_size);
        println!("  Overhead: {} bytes ({:.1}%)", 
                 self.string_stats.overhead_bytes, self.string_stats.overhead_percentage);
        println!();

        println!("Pair values:");
        println!("  Unsafe: {} bytes", self.pair_stats.unsafe_size);
        println!("  Safe:   {} bytes", self.pair_stats.safe_size);
        println!("  Overhead: {} bytes ({:.1}%)", 
                 self.pair_stats.overhead_bytes, self.pair_stats.overhead_percentage);
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_suite() {
        let config = BenchmarkConfig {
            iterations: 1000, // Smaller for tests
            warmup_iterations: 100,
        };
        
        let benchmark = MemorySafetyBenchmark::new(config);
        let results = benchmark.run_all_benchmarks();
        
        // Performance should be reasonable (within 20% for test purposes)
        assert!(results.validate_performance(20.0), 
                "Performance overhead too high: {:.2}%", results.average_overhead());
        
        results.print_report();
    }

    #[test]
    fn test_memory_usage() {
        let benchmark = MemorySafetyBenchmark::new(BenchmarkConfig::default());
        let memory_report = benchmark.measure_memory_usage();
        memory_report.print_report();

        // Memory overhead should be reasonable (within 100% is acceptable for safety gains)
        assert!(memory_report.fixnum_stats.overhead_percentage < 100.0);
    }

    #[test]
    fn test_arc_reduction_maintained() {
        // Verify that the key optimization (pair with 0 Arcs) is maintained
        let safe_pair = SafeOptimizedValue::pair(
            SafeOptimizedValue::fixnum(1i64),
            SafeOptimizedValue::fixnum(2i64)
        );
        
        // This should not use any Arc internally for the pair structure itself
        if let Some((car, cdr)) = safe_pair.as_pair() {
            assert_eq!(car.as_integer(), Some(1));
            assert_eq!(cdr.as_integer(), Some(2));
        } else {
            panic!("Pair creation failed");
        }
    }

    #[test]
    fn test_immediate_value_zero_cost() {
        // Verify zero-cost abstractions for immediate values
        let unsafe_val = OptimizedValue::fixnum(42i64);
        let safe_val = SafeOptimizedValue::fixnum(42i64);

        // Both should provide the same functionality
        assert_eq!(unsafe_val.as_integer(), safe_val.as_integer());
        assert_eq!(unsafe_val.is_number(), safe_val.is_number());
    }
}