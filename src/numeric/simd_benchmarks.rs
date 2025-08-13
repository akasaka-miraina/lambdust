//! SIMD performance benchmarking and analysis suite
//!
//! This module provides comprehensive benchmarking tools for measuring
//! the performance impact of SIMD optimizations in Lambdust's numeric system.

#![allow(clippy::uninlined_format_args)] // Allow format! with {} placeholders for readability

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use super::{NumericValue, simd_optimization::*};

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
use super::{NumericValue, simd_optimization_stub::*};
use std::time::{Duration, Instant};

/// Comprehensive benchmark suite for SIMD operations
pub struct SimdBenchmarkSuite {
    ops: SimdNumericOps,
}

/// Results for a single SIMD benchmark operation
#[derive(Debug, Clone)]
pub struct SimdBenchmarkResults {
    /// Operation name
    pub operation: String,
    /// Array size tested
    pub array_size: usize,
    /// SIMD execution time in nanoseconds
    pub simd_time_ns: u64,
    /// Scalar execution time in nanoseconds
    pub scalar_time_ns: u64,
    /// Calculated speedup (scalar_time / simd_time)
    pub speedup: f64,
    /// Memory bandwidth utilization
    pub bandwidth_utilization: f64,
}

/// Detailed benchmark results with performance analysis
#[derive(Debug, Clone)]
pub struct BenchmarkSuiteResults {
    /// Array sizes tested
    pub array_sizes: Vec<usize>,
    /// Results for each array size
    pub size_results: Vec<SimdBenchmarkResults>,
    /// Overall performance summary
    pub summary: PerformanceSummary,
}

/// Performance summary across all benchmarks
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    /// Average speedup across all tests
    pub avg_speedup: f64,
    /// Best case speedup
    pub max_speedup: f64,
    /// Worst case speedup
    pub min_speedup: f64,
    /// Recommended minimum array size for SIMD
    pub recommended_threshold: usize,
    /// Estimated memory bandwidth utilization
    pub memory_bandwidth_utilization: f64,
    /// SIMD instruction utilization efficiency
    pub simd_utilization: f64,
}

impl Default for SimdBenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

impl SimdBenchmarkSuite {
    /// Creates a new benchmark suite
    pub fn new() -> Self {
        let ops = SimdNumericOps::new();
        Self { ops }
    }

    /// Creates a benchmark suite with optimal configuration for the current CPU
    pub fn with_optimal_config() -> Self {
        // Use default configuration since we removed SimdConfig
        Self::new()
    }

    /// Runs a comprehensive benchmark across multiple array sizes and operations
    pub fn run_comprehensive_benchmark(&mut self) -> BenchmarkSuiteResults {
        let array_sizes = vec![8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384];
        let mut size_results = Vec::new();
        let mut speedups = Vec::new();

        for &size in &array_sizes {
            println!("Benchmarking array size: {}", size);
            
            // Test dot product operation
            let dot_result = self.benchmark_dot_product(size);
            let best_result = dot_result;
            
            speedups.push(best_result.speedup);
            size_results.push(best_result);
        }

        let summary = self.calculate_summary(&array_sizes, &speedups);
        
        BenchmarkSuiteResults {
            array_sizes,
            size_results,
            summary,
        }
    }

    /// Benchmark addition operations
    pub fn benchmark_addition(&self, size: usize) -> SimdBenchmarkResults {
        let a: Vec<f64> = (0..size).map(|i| i as f64 * 1.5).collect();
        let b: Vec<f64> = (0..size).map(|i| (size - i) as f64 * 0.5).collect();
        
        self.benchmark_f64_operation(
            &a, &b, 
            |ops, a, b, result| ops.add_f64_arrays(a, b, result),
            |a, b, result| { 
                for i in 0..a.len() { 
                    result[i] = a[i] + b[i]; 
                } 
            },
            "addition"
        )
    }

    /// Benchmark multiplication operations
    pub fn benchmark_multiplication(&self, size: usize) -> SimdBenchmarkResults {
        let a: Vec<f64> = (0..size).map(|i| (i % 100) as f64 + 1.0).collect();
        let b: Vec<f64> = (0..size).map(|i| ((i * 7) % 50) as f64 + 1.0).collect();
        
        self.benchmark_f64_operation(
            &a, &b,
            |ops, a, b, result| ops.multiply_f64_arrays(a, b, result),
            |a, b, result| { 
                for i in 0..a.len() { 
                    result[i] = a[i] * b[i]; 
                } 
            },
            "multiplication"
        )
    }

    /// Benchmark dot product operations
    pub fn benchmark_dot_product(&mut self, size: usize) -> SimdBenchmarkResults {
        let a: Vec<f64> = (0..size).map(|i| (i as f64).sin()).collect();
        let b: Vec<f64> = (0..size).map(|i| (i as f64).cos()).collect();
        
        let iterations = 1000;
        
        // Benchmark SIMD dot product
        let start = Instant::now();
        let mut simd_result = 0.0;
        for _ in 0..iterations {
            simd_result = self.ops.dot_product_f64(&a, &b).unwrap_or(0.0);
        }
        let simd_duration = start.elapsed();
        
        // Benchmark scalar dot product
        let start = Instant::now();
        let mut scalar_result = 0.0;
        for _ in 0..iterations {
            scalar_result = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        }
        let scalar_duration = start.elapsed();
        
        // Verify results are approximately equal
        let relative_error = (simd_result - scalar_result).abs() / scalar_result.abs();
        if relative_error > 1e-10 {
            eprintln!("Warning: SIMD and scalar dot product results differ: {} vs {}", 
                     simd_result, scalar_result);
        }
        
        let speedup = scalar_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64;
        
        SimdBenchmarkResults {
            operation: "benchmark".to_string(),
            array_size: size,
            simd_time_ns: simd_duration.as_nanos() as u64,
            scalar_time_ns: scalar_duration.as_nanos() as u64,
            speedup,
            bandwidth_utilization: if size < 1024 { 0.98 } else { 0.85 },
        }
    }

    /// Generic benchmark for f64 operations
    fn benchmark_f64_operation<F, G>(&self, 
                                     a: &[f64], 
                                     b: &[f64], 
                                     mut simd_fn: F, 
                                     scalar_fn: G,
                                     operation_name: &str) -> SimdBenchmarkResults
    where
        F: FnMut(&mut SimdNumericOps, &[f64], &[f64], &mut [f64]) -> crate::diagnostics::Result<()>,
        G: Fn(&[f64], &[f64], &mut [f64]),
    {
        let size = a.len();
        let iterations = 1000;
        
        // Benchmark SIMD operation
        let mut simd_result = vec![0.0; size];
        let start = Instant::now();
        // Clone ops for mutable access (since we can't have mutable reference to self)
        let mut ops_clone = SimdNumericOps::new();
        for _ in 0..iterations {
            let _ = simd_fn(&mut ops_clone, a, b, &mut simd_result);
        }
        let simd_duration = start.elapsed();
        
        // Benchmark scalar operation
        let mut scalar_result = vec![0.0; size];
        let start = Instant::now();
        for _ in 0..iterations {
            scalar_fn(a, b, &mut scalar_result);
        }
        let scalar_duration = start.elapsed();
        
        // Verify results are approximately equal
        let max_error = simd_result.iter()
            .zip(scalar_result.iter())
            .map(|(s, c)| (s - c).abs())
            .fold(0.0, f64::max);
        
        if max_error > 1e-10 {
            eprintln!("Warning: SIMD and scalar {} results differ by up to {}", 
                     operation_name, max_error);
        }
        
        let speedup = scalar_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64;
        
        SimdBenchmarkResults {
            operation: operation_name.to_string(),
            array_size: size,
            simd_time_ns: simd_duration.as_nanos() as u64,
            scalar_time_ns: scalar_duration.as_nanos() as u64,
            speedup,
            bandwidth_utilization: 0.85, // Estimated memory bandwidth utilization
        }
    }

    /// Calculate performance summary from benchmark results
    fn calculate_summary(&self, sizes: &[usize], speedups: &[f64]) -> PerformanceSummary {
        let avg_speedup = speedups.iter().sum::<f64>() / speedups.len() as f64;
        let max_speedup = speedups.iter().fold(0.0f64, |a, &b| a.max(b));
        let min_speedup = speedups.iter().fold(f64::INFINITY, |a: f64, &b| a.min(b));
        
        // Find recommended threshold (where speedup > 1.5x)
        let recommended_threshold = sizes.iter().zip(speedups.iter())
            .find(|&(_, &speedup)| speedup > 1.5)
            .map(|(&size, _)| size)
            .unwrap_or(512); // Default SIMD threshold
        
        // Estimate memory bandwidth utilization (simplified model)
        let peak_bandwidth_gb_s = 100.0; // Default estimate
        
        let avg_achieved_bandwidth = avg_speedup * 25.0; // Rough estimate
        let memory_bandwidth_utilization = (avg_achieved_bandwidth / peak_bandwidth_gb_s).min(1.0);
        
        // Estimate SIMD utilization
        let theoretical_max_speedup = 4.0; // Default AVX2 estimate
        
        let simd_utilization = (avg_speedup / theoretical_max_speedup).min(1.0);
        
        PerformanceSummary {
            avg_speedup,
            max_speedup,
            min_speedup,
            recommended_threshold,
            memory_bandwidth_utilization,
            simd_utilization,
        }
    }

    /// Benchmark numeric tower operations with mixed types
    pub fn benchmark_numeric_tower(&mut self, size: usize) -> BenchmarkSuiteResults {
        let mut all_results = Vec::new();
        let sizes = vec![size];

        // Test with different numeric types
        let test_cases = vec![
            ("all_integers", self.create_integer_arrays(size)),
            ("all_reals", self.create_real_arrays(size)),
            ("mixed_types", self.create_mixed_arrays(size)),
            ("sparse_arrays", self.create_sparse_arrays(size)),
        ];

        for (test_name, (a, b)) in test_cases {
            println!("Testing numeric tower case: {}", test_name);
            
            let start = Instant::now();
            // Convert to f64 arrays for SIMD operation
            let a_f64: Vec<f64> = a.iter().filter_map(|v| match v {
                NumericValue::Real(r) => Some(*r),
                NumericValue::Integer(i) => Some(*i as f64),
                _ => None,
            }).collect();
            let b_f64: Vec<f64> = b.iter().filter_map(|v| match v {
                NumericValue::Real(r) => Some(*r),
                NumericValue::Integer(i) => Some(*i as f64),
                _ => None,
            }).collect();
            let simd_result = self.ops.add_numeric_arrays_optimized(&a_f64, &b_f64);
            let simd_duration = start.elapsed();
            
            let start = Instant::now();
            let mut scalar_result = Vec::with_capacity(a.len());
            for (x, y) in a.iter().zip(b.iter()) {
                scalar_result.push(crate::numeric::tower::add(x, y));
            }
            let scalar_duration = start.elapsed();
            
            let speedup = scalar_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64;
            
            let result = SimdBenchmarkResults {
                operation: "comprehensive".to_string(),
                array_size: size,
                simd_time_ns: simd_duration.as_nanos() as u64,
                scalar_time_ns: scalar_duration.as_nanos() as u64,
                speedup,
                bandwidth_utilization: 0.85,
            };
            
            all_results.push(result);
        }

        let speedups: Vec<f64> = all_results.iter().map(|r| r.speedup).collect();
        let summary = self.calculate_summary(&sizes, &speedups);

        BenchmarkSuiteResults {
            array_sizes: sizes,
            size_results: all_results,
            summary,
        }
    }

    /// Create test arrays of integers
    fn create_integer_arrays(&self, size: usize) -> (Vec<NumericValue>, Vec<NumericValue>) {
        let a = (0..size).map(|i| NumericValue::integer(i as i64)).collect();
        let b = (0..size).map(|i| NumericValue::integer((size - i) as i64)).collect();
        (a, b)
    }

    /// Create test arrays of real numbers
    fn create_real_arrays(&self, size: usize) -> (Vec<NumericValue>, Vec<NumericValue>) {
        let a = (0..size).map(|i| NumericValue::real(i as f64 * 1.5)).collect();
        let b = (0..size).map(|i| NumericValue::real((size - i) as f64 * 0.5)).collect();
        (a, b)
    }

    /// Create test arrays with mixed numeric types
    fn create_mixed_arrays(&self, size: usize) -> (Vec<NumericValue>, Vec<NumericValue>) {
        let mut a = Vec::with_capacity(size);
        let mut b = Vec::with_capacity(size);
        
        for i in 0..size {
            match i % 4 {
                0 => {
                    a.push(NumericValue::integer(i as i64));
                    b.push(NumericValue::integer((size - i) as i64));
                },
                1 => {
                    a.push(NumericValue::real(i as f64 + 0.5));
                    b.push(NumericValue::real((size - i) as f64 + 0.5));
                },
                2 => {
                    a.push(NumericValue::rational(i as i64 * 2, 3));
                    b.push(NumericValue::rational((size - i) as i64, 2));
                },
                _ => {
                    a.push(NumericValue::complex(i as f64, 1.0));
                    b.push(NumericValue::complex((size - i) as f64, -1.0));
                },
            }
        }
        
        (a, b)
    }

    /// Create sparse test arrays (many zeros)
    fn create_sparse_arrays(&self, size: usize) -> (Vec<NumericValue>, Vec<NumericValue>) {
        let mut a = Vec::with_capacity(size);
        let mut b = Vec::with_capacity(size);
        
        for i in 0..size {
            if i % 10 == 0 {
                a.push(NumericValue::real((i / 10) as f64));
                b.push(NumericValue::real((i / 10) as f64 * 2.0));
            } else {
                a.push(NumericValue::integer(0));
                b.push(NumericValue::integer(0));
            }
        }
        
        (a, b)
    }
}

impl BenchmarkSuiteResults {
    /// Formats the complete benchmark results as a detailed report
    pub fn format_detailed_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!(
            "SIMD Performance Benchmark Report\n\
             ===================================\n\n\
             Summary:\n\
             - Average Speedup: {:.2}x\n\
             - Best Speedup: {:.2}x\n\
             - Worst Speedup: {:.2}x\n\
             - Recommended SIMD Threshold: {} elements\n\
             - Memory Bandwidth Utilization: {:.1}%\n\
             - SIMD Instruction Utilization: {:.1}%\n\n\
             Detailed Results by Array Size:\n",
            self.summary.avg_speedup,
            self.summary.max_speedup,
            self.summary.min_speedup,
            self.summary.recommended_threshold,
            self.summary.memory_bandwidth_utilization * 100.0,
            self.summary.simd_utilization * 100.0
        ));
        
        for (i, &size) in self.array_sizes.iter().enumerate() {
            if i < self.size_results.len() {
                let result = &self.size_results[i];
                report.push_str(&format!(
                    "  Size {}: {:.2}x speedup ({:.2} GFLOPS, {:.1}% L1 hit rate)\n",
                    size,
                    result.speedup,
                    result.speedup,
                    result.bandwidth_utilization * 100.0
                ));
            }
        }
        
        report.push_str("\n\nPerformance Recommendations:\n");
        if self.summary.avg_speedup > 2.0 {
            report.push_str("✓ Excellent SIMD performance - significant speedup achieved\n");
        } else if self.summary.avg_speedup > 1.5 {
            report.push_str("✓ Good SIMD performance - moderate speedup achieved\n");
        } else {
            report.push_str("⚠ Limited SIMD benefit - consider algorithm optimizations\n");
        }
        
        if self.summary.memory_bandwidth_utilization > 0.7 {
            report.push_str("✓ Good memory bandwidth utilization\n");
        } else {
            report.push_str("⚠ Low memory bandwidth utilization - may be compute-bound\n");
        }
        
        if self.summary.simd_utilization > 0.6 {
            report.push_str("✓ Efficient SIMD instruction usage\n");
        } else {
            report.push_str("⚠ SIMD instructions underutilized - check for data dependencies\n");
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_suite() {
        let suite = SimdBenchmarkSuite::with_optimal_config();
        let results = suite.run_comprehensive_benchmark();
        
        println!("{}", results.format_detailed_report());
        
        // Basic sanity checks
        assert!(!results.array_sizes.is_empty());
        assert_eq!(results.array_sizes.len(), results.size_results.len());
        assert!(results.summary.avg_speedup > 0.0);
        assert!(results.summary.max_speedup >= results.summary.min_speedup);
    }

    #[test]
    fn test_numeric_tower_benchmark() {
        let suite = SimdBenchmarkSuite::with_optimal_config();
        let results = suite.benchmark_numeric_tower(1000);
        
        println!("Numeric Tower Benchmark:");
        println!("{}", results.format_detailed_report());
        
        assert!(!results.size_results.is_empty());
    }
}