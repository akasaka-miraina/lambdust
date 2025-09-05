//! Performance Validation for Phase 4.1 Optimizations
//!
//! This module provides validation tests for the performance improvements
//! achieved through the staged optimization integration.

use crate::eval::Value;

#[cfg(feature = "trait-optimization")]
use crate::eval::OptimizationHint;

#[cfg(feature = "trait-optimization")]
use crate::eval::{OptimizationFramework, SimpleNanBoxedValue, ValueOptimizationExt};

#[cfg(feature = "nan-boxing-optimization")]
use crate::eval::{EnhancedNanBoxedValue, NanBoxingValueOptimizer, ValueNanBoxingExt};

#[cfg(feature = "arena-optimization")]
use crate::eval::{ArenaValueOptimizer, ValueArenaExt};

#[cfg(feature = "simd-optimization")]
use crate::eval::{SimdValueOperations, ValueSimdExt};

use std::time::{Duration, Instant};

/// Performance validation results
#[derive(Debug, Clone)]
pub struct PerformanceResults {
    /// Baseline performance (no optimizations)
    pub baseline_time: Duration,
    /// Stage 1 performance (trait optimization)
    pub stage1_time: Option<Duration>,
    /// Stage 2 performance (trait + NaN boxing)
    pub stage2_time: Option<Duration>,
    /// Stage 3 performance (stages 1-2 + arena)
    pub stage3_time: Option<Duration>,
    /// Stage 4 performance (stages 1-3 + SIMD)
    pub stage4_time: Option<Duration>,
    /// Memory usage baseline
    pub baseline_memory: usize,
    /// Memory usage with all optimizations
    pub optimized_memory: Option<usize>,
}

impl PerformanceResults {
    /// Calculate performance improvement ratio
    pub fn improvement_ratio(&self) -> f64 {
        let best_time = self.best_optimized_time();
        self.baseline_time.as_nanos() as f64 / best_time.as_nanos() as f64
    }

    /// Calculate memory savings ratio
    pub fn memory_savings_ratio(&self) -> f64 {
        if let Some(optimized) = self.optimized_memory {
            (self.baseline_memory - optimized) as f64 / self.baseline_memory as f64
        } else {
            0.0
        }
    }

    /// Get the best optimized time achieved
    pub fn best_optimized_time(&self) -> Duration {
        [
            self.stage1_time,
            self.stage2_time,
            self.stage3_time,
            self.stage4_time,
        ]
        .iter()
        .filter_map(|&t| t)
        .min()
        .unwrap_or(self.baseline_time)
    }
}

/// Performance validator for optimization stages
pub struct PerformanceValidator {
    test_data: Vec<Value>,
    iterations: usize,
}

impl PerformanceValidator {
    /// Create new performance validator
    pub fn new(iterations: usize) -> Self {
        // Generate test data
        let test_data = (0..1000)
            .map(|i| match i % 6 {
                0 => Value::number(i as f64),
                1 => Value::integer(i as i64),
                2 => Value::boolean(i % 2 == 0),
                3 => Value::string(format!("test_{}", i)),
                4 => Value::vector(vec![Value::number(i as f64), Value::integer(i as i64)]),
                _ => Value::pair(Value::integer(i as i64), Value::Nil),
            })
            .collect();

        Self {
            test_data,
            iterations,
        }
    }

    /// Run comprehensive performance validation
    pub fn validate_all_stages(&self) -> PerformanceResults {
        let baseline_time = self.measure_baseline_performance();
        let baseline_memory = self.measure_baseline_memory();

        let stage1_time = self.measure_stage1_performance();
        let stage2_time = self.measure_stage2_performance();
        let stage3_time = self.measure_stage3_performance();
        let stage4_time = self.measure_stage4_performance();

        let optimized_memory = self.measure_optimized_memory();

        PerformanceResults {
            baseline_time,
            stage1_time,
            stage2_time,
            stage3_time,
            stage4_time,
            baseline_memory,
            optimized_memory,
        }
    }

    /// Measure baseline performance without optimizations
    fn measure_baseline_performance(&self) -> Duration {
        let start = Instant::now();

        for _ in 0..self.iterations {
            for value in &self.test_data {
                // Basic operations without optimization
                let _ = value.clone();
                let _ = value.is_number();
                let _ = value.is_boolean();
                let _ = std::mem::size_of_val(value);
            }
        }

        start.elapsed()
    }

    /// Measure Stage 1 (trait optimization) performance
    fn measure_stage1_performance(&self) -> Option<Duration> {
        #[cfg(feature = "trait-optimization")]
        {
            let framework = OptimizationFramework::new();
            let start = Instant::now();

            for _ in 0..self.iterations {
                for value in &self.test_data {
                    if value.can_optimize() {
                        #[cfg(feature = "trait-optimization")]
                        let _optimized =
                            value.to_optimized(crate::eval::OptimizationHint::Computational);
                    }
                }
            }

            Some(start.elapsed())
        }
        #[cfg(not(feature = "trait-optimization"))]
        None
    }

    /// Measure Stage 2 (trait + NaN boxing) performance
    fn measure_stage2_performance(&self) -> Option<Duration> {
        #[cfg(all(feature = "trait-optimization", feature = "nan-boxing-optimization"))]
        {
            let optimizer = NanBoxingValueOptimizer::new();
            let start = Instant::now();

            for _ in 0..self.iterations {
                for value in &self.test_data {
                    if value.can_nan_box() {
                        #[cfg(feature = "nan-boxing-optimization")]
                        let _optimized = value
                            .to_nan_boxed_optimized(crate::eval::OptimizationHint::Computational);
                    }
                }
            }

            Some(start.elapsed())
        }
        #[cfg(not(all(feature = "trait-optimization", feature = "nan-boxing-optimization")))]
        None
    }

    /// Measure Stage 3 (stages 1-2 + arena) performance
    fn measure_stage3_performance(&self) -> Option<Duration> {
        #[cfg(all(
            feature = "trait-optimization",
            feature = "nan-boxing-optimization",
            feature = "arena-optimization"
        ))]
        {
            let arena_optimizer = ArenaValueOptimizer::new();
            let start = Instant::now();

            for _ in 0..self.iterations {
                for value in &self.test_data {
                    if value.can_arena_optimize() {
                        #[cfg(feature = "arena-optimization")]
                        let _optimized =
                            value.to_arena_optimized(crate::eval::OptimizationHint::Computational);
                    }
                }
            }

            Some(start.elapsed())
        }
        #[cfg(not(all(
            feature = "trait-optimization",
            feature = "nan-boxing-optimization",
            feature = "arena-optimization"
        )))]
        None
    }

    /// Measure Stage 4 (all stages + SIMD) performance
    fn measure_stage4_performance(&self) -> Option<Duration> {
        #[cfg(feature = "simd-optimization")]
        {
            let simd_ops = SimdValueOperations::new();
            let start = Instant::now();

            // Create numerical test data for SIMD
            let numbers: Vec<Value> = (0..100).map(|i| Value::number(i as f64)).collect();
            let numbers2: Vec<Value> = (0..100).map(|i| Value::number((i * 2) as f64)).collect();

            for _ in 0..self.iterations {
                // SIMD vector operations
                if let Some(_result) = simd_ops.vector_add(&numbers, &numbers2) {
                    // SIMD addition performed
                }

                let _scaled = simd_ops.scalar_multiply(&numbers, 2.0);

                // Apply SIMD optimization to other values
                for value in &self.test_data {
                    if value.can_simd_optimize() {
                        #[cfg(feature = "simd-optimization")]
                        let _optimized =
                            value.to_simd_optimized(crate::eval::OptimizationHint::Computational);
                    }
                }
            }

            Some(start.elapsed())
        }
        #[cfg(not(feature = "simd-optimization"))]
        None
    }

    /// Measure baseline memory usage
    fn measure_baseline_memory(&self) -> usize {
        self.test_data
            .iter()
            .map(|v| std::mem::size_of_val(v))
            .sum()
    }

    /// Measure optimized memory usage
    fn measure_optimized_memory(&self) -> Option<usize> {
        #[cfg(feature = "nan-boxing-optimization")]
        {
            let mut total = 0;
            for value in &self.test_data {
                if value.can_nan_box() {
                    // NaN boxed values are smaller
                    total += std::mem::size_of::<EnhancedNanBoxedValue>();
                } else {
                    total += std::mem::size_of_val(value);
                }
            }
            Some(total)
        }
        #[cfg(not(feature = "nan-boxing-optimization"))]
        None
    }

    /// Generate performance report
    pub fn generate_report(&self, results: &PerformanceResults) -> String {
        let mut report = String::new();

        report.push_str("=== Phase 4.1 Performance Validation Report ===\n\n");

        report.push_str(&format!(
            "Baseline Performance: {:?}\n",
            results.baseline_time
        ));
        report.push_str(&format!(
            "Test Data Size: {} values\n",
            self.test_data.len()
        ));
        report.push_str(&format!("Iterations: {}\n\n", self.iterations));

        // Stage performance
        if let Some(stage1) = results.stage1_time {
            let improvement = results.baseline_time.as_nanos() as f64 / stage1.as_nanos() as f64;
            report.push_str(&format!(
                "Stage 1 (Trait): {:?} ({:.2}x improvement)\n",
                stage1, improvement
            ));
        }

        if let Some(stage2) = results.stage2_time {
            let improvement = results.baseline_time.as_nanos() as f64 / stage2.as_nanos() as f64;
            report.push_str(&format!(
                "Stage 2 (+NaN Boxing): {:?} ({:.2}x improvement)\n",
                stage2, improvement
            ));
        }

        if let Some(stage3) = results.stage3_time {
            let improvement = results.baseline_time.as_nanos() as f64 / stage3.as_nanos() as f64;
            report.push_str(&format!(
                "Stage 3 (+Arena): {:?} ({:.2}x improvement)\n",
                stage3, improvement
            ));
        }

        if let Some(stage4) = results.stage4_time {
            let improvement = results.baseline_time.as_nanos() as f64 / stage4.as_nanos() as f64;
            report.push_str(&format!(
                "Stage 4 (+SIMD): {:?} ({:.2}x improvement)\n",
                stage4, improvement
            ));
        }

        // Overall results
        report.push_str(&format!(
            "\nOverall Performance Improvement: {:.2}x\n",
            results.improvement_ratio()
        ));

        // Memory results
        report.push_str(&format!(
            "Baseline Memory: {} bytes\n",
            results.baseline_memory
        ));
        if let Some(optimized) = results.optimized_memory {
            report.push_str(&format!("Optimized Memory: {} bytes\n", optimized));
            report.push_str(&format!(
                "Memory Savings: {:.1}%\n",
                results.memory_savings_ratio() * 100.0
            ));
        }

        // Validation conclusion
        let overall_improvement = results.improvement_ratio();
        let memory_savings = results.memory_savings_ratio();

        report.push_str("\n=== Validation Results ===\n");

        if overall_improvement > 1.5 {
            report.push_str("✅ PERFORMANCE: Significant improvement achieved\n");
        } else if overall_improvement > 1.2 {
            report.push_str("✅ PERFORMANCE: Moderate improvement achieved\n");
        } else {
            report.push_str("⚠️  PERFORMANCE: Limited improvement observed\n");
        }

        if memory_savings > 0.3 {
            report.push_str("✅ MEMORY: Significant memory savings achieved\n");
        } else if memory_savings > 0.1 {
            report.push_str("✅ MEMORY: Moderate memory savings achieved\n");
        } else {
            report.push_str("⚠️  MEMORY: Limited memory savings observed\n");
        }

        report.push_str(&format!(
            "\nPhase 4.1 Integration: {}\n",
            if overall_improvement > 1.2 || memory_savings > 0.2 {
                "SUCCESS - Measurable performance benefits achieved"
            } else {
                "NEEDS_REVIEW - Optimization impact less than expected"
            }
        ));

        report
    }
}

// ============= INTEGRATION TESTS =============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_validator_creation() {
        let validator = PerformanceValidator::new(10);
        assert_eq!(validator.iterations, 10);
        assert_eq!(validator.test_data.len(), 1000);
    }

    #[test]
    fn test_baseline_performance_measurement() {
        let validator = PerformanceValidator::new(5);
        let baseline = validator.measure_baseline_performance();
        assert!(baseline > Duration::from_nanos(1));
    }

    #[test]
    fn test_performance_results() {
        let results = PerformanceResults {
            baseline_time: Duration::from_millis(100),
            stage1_time: Some(Duration::from_millis(80)),
            stage2_time: Some(Duration::from_millis(60)),
            stage3_time: Some(Duration::from_millis(50)),
            stage4_time: Some(Duration::from_millis(40)),
            baseline_memory: 1000,
            optimized_memory: Some(600),
        };

        assert_eq!(results.improvement_ratio(), 2.5); // 100/40
        assert_eq!(results.memory_savings_ratio(), 0.4); // (1000-600)/1000
        assert_eq!(results.best_optimized_time(), Duration::from_millis(40));
    }

    #[test]
    fn test_report_generation() {
        let validator = PerformanceValidator::new(1);
        let results = validator.validate_all_stages();
        let report = validator.generate_report(&results);

        assert!(report.contains("Phase 4.1 Performance Validation Report"));
        assert!(report.contains("Baseline Performance"));
        assert!(report.contains("Validation Results"));
    }
}
