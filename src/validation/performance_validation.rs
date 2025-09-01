#![allow(missing_docs)]//! Performance Validation for Phase 8 Optimizations
//!
//! This module ensures that optimization features meet performance thresholds
//! and deliver the expected improvements.

use std::collections::HashMap;
use std::time::Duration;
use crate::validation::benchmark_automation::BenchmarkReport;

/// Performance validator that checks benchmarks against defined thresholds
pub struct PerformanceValidator {
    thresholds: HashMap<String, PerformanceThreshold>,
}

/// Performance threshold configuration for a specific benchmark
#[derive(Debug, Clone)]
pub struct PerformanceThreshold {
    /// Maximum acceptable execution time
    pub max_time_ms: f64,
    
    /// Minimum required improvement over baseline (as ratio, e.g., 1.3 = 30% improvement)
    pub min_improvement_ratio: f64,
    
    /// Maximum acceptable memory usage in bytes
    pub max_memory_bytes: usize,
    
    /// Minimum memory reduction ratio (e.g., 0.7 = 30% reduction)
    pub min_memory_reduction_ratio: f64,
    
    /// Whether this threshold is critical (failure blocks deployment)
    pub is_critical: bool,
}

/// Result of performance validation
#[derive(Debug)]
pub struct ValidationResult {
    pub passed: bool,
    pub violations: Vec<ThresholdViolation>,
    pub summary: String,
}

/// A specific threshold violation
#[derive(Debug)]
pub struct ThresholdViolation {
    pub benchmark_name: String,
    pub threshold_type: ThresholdType,
    pub expected: f64,
    pub actual: f64,
    pub is_critical: bool,
}

#[derive(Debug)]
pub enum ThresholdType {
    MaxTime,
    MinImprovement,
    MaxMemory,
    MinMemoryReduction,
}

impl PerformanceValidator {
    /// Create a new performance validator with Phase 8 optimization thresholds
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        
        // Value creation benchmarks - expect significant memory improvements
        thresholds.insert("value_creation".to_string(), PerformanceThreshold {
            max_time_ms: 100.0,
            min_improvement_ratio: 1.4, // 40% improvement expected
            max_memory_bytes: 1_000_000, // 1MB max
            min_memory_reduction_ratio: 0.6, // 40% memory reduction
            is_critical: true,
        });
        
        // String interning - expect time improvements
        thresholds.insert("string_interning".to_string(), PerformanceThreshold {
            max_time_ms: 50.0,
            min_improvement_ratio: 2.0, // 100% improvement (2x faster)
            max_memory_bytes: 500_000,
            min_memory_reduction_ratio: 0.8, // 20% memory reduction
            is_critical: true,
        });
        
        // Parsing benchmarks - expect moderate improvements
        thresholds.insert("parsing".to_string(), PerformanceThreshold {
            max_time_ms: 200.0,
            min_improvement_ratio: 1.2, // 20% improvement
            max_memory_bytes: 2_000_000,
            min_memory_reduction_ratio: 0.9, // 10% memory reduction
            is_critical: false,
        });
        
        // NaN boxing - expect major memory improvements
        thresholds.insert("nan_boxing".to_string(), PerformanceThreshold {
            max_time_ms: 10.0,
            min_improvement_ratio: 1.5, // 50% improvement
            max_memory_bytes: 100_000,
            min_memory_reduction_ratio: 0.4, // 60% memory reduction
            is_critical: true,
        });
        
        // Arena allocation - expect memory efficiency
        thresholds.insert("arena_allocation".to_string(), PerformanceThreshold {
            max_time_ms: 150.0,
            min_improvement_ratio: 1.3, // 30% improvement
            max_memory_bytes: 5_000_000,
            min_memory_reduction_ratio: 0.7, // 30% memory reduction
            is_critical: false,
        });
        
        Self { thresholds }
    }
    
    /// Validate a benchmark report against performance thresholds
    pub fn validate_report(&self, report: &BenchmarkReport) -> Result<ValidationResult, String> {
        let mut violations = Vec::new();
        let mut critical_failures = 0;
        let mut total_checks = 0;
        
        for (benchmark_name, benchmark_result) in &report.results {
            if let Some(threshold) = self.thresholds.get(benchmark_name) {
                total_checks += 1;
                
                // Check execution time
                let time_ms = benchmark_result.execution_time.as_millis() as f64;
                if time_ms > threshold.max_time_ms {
                    let violation = ThresholdViolation {
                        benchmark_name: benchmark_name.clone(),
                        threshold_type: ThresholdType::MaxTime,
                        expected: threshold.max_time_ms,
                        actual: time_ms,
                        is_critical: threshold.is_critical,
                    };
                    
                    if threshold.is_critical {
                        critical_failures += 1;
                    }
                    violations.push(violation);
                }
                
                // Check improvement ratio
                if let Some(baseline_time) = benchmark_result.baseline_time {
                    let improvement_ratio = baseline_time.as_millis() as f64 / time_ms;
                    if improvement_ratio < threshold.min_improvement_ratio {
                        let violation = ThresholdViolation {
                            benchmark_name: benchmark_name.clone(),
                            threshold_type: ThresholdType::MinImprovement,
                            expected: threshold.min_improvement_ratio,
                            actual: improvement_ratio,
                            is_critical: threshold.is_critical,
                        };
                        
                        if threshold.is_critical {
                            critical_failures += 1;
                        }
                        violations.push(violation);
                    }
                }
                
                // Check memory usage
                if let Some(memory_bytes) = benchmark_result.memory_usage_bytes {
                    if memory_bytes > threshold.max_memory_bytes {
                        let violation = ThresholdViolation {
                            benchmark_name: benchmark_name.clone(),
                            threshold_type: ThresholdType::MaxMemory,
                            expected: threshold.max_memory_bytes as f64,
                            actual: memory_bytes as f64,
                            is_critical: threshold.is_critical,
                        };
                        
                        if threshold.is_critical {
                            critical_failures += 1;
                        }
                        violations.push(violation);
                    }
                    
                    // Check memory reduction
                    if let Some(baseline_memory) = benchmark_result.baseline_memory_bytes {
                        let reduction_ratio = memory_bytes as f64 / baseline_memory as f64;
                        if reduction_ratio > threshold.min_memory_reduction_ratio {
                            let violation = ThresholdViolation {
                                benchmark_name: benchmark_name.clone(),
                                threshold_type: ThresholdType::MinMemoryReduction,
                                expected: threshold.min_memory_reduction_ratio,
                                actual: reduction_ratio,
                                is_critical: threshold.is_critical,
                            };
                            
                            if threshold.is_critical {
                                critical_failures += 1;
                            }
                            violations.push(violation);
                        }
                    }
                }
            }
        }
        
        let passed = critical_failures == 0;
        let summary = format!(
            "Performance validation: {} critical failures, {} total violations out of {} checks",
            critical_failures, violations.len(), total_checks
        );
        
        Ok(ValidationResult {
            passed,
            violations,
            summary,
        })
    }
    
    /// Add or update a performance threshold
    pub fn set_threshold(&mut self, benchmark_name: String, threshold: PerformanceThreshold) {
        self.thresholds.insert(benchmark_name, threshold);
    }
    
    /// Get all configured thresholds
    pub fn get_thresholds(&self) -> &HashMap<String, PerformanceThreshold> {
        &self.thresholds
    }
}

impl ValidationResult {
    /// Get critical violations only
    pub fn critical_violations(&self) -> Vec<&ThresholdViolation> {
        self.violations.iter()
            .filter(|v| v.is_critical)
            .collect()
    }
    
    /// Get non-critical violations
    pub fn warning_violations(&self) -> Vec<&ThresholdViolation> {
        self.violations.iter()
            .filter(|v| !v.is_critical)
            .collect()
    }
    
    /// Generate detailed report
    pub fn detailed_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("Performance Validation Report\n"));
        report.push_str(&format!("Status: {}\n", if self.passed { "PASSED" } else { "FAILED" }));
        report.push_str(&format!("Summary: {}\n\n", self.summary));
        
        if !self.violations.is_empty() {
            report.push_str("Violations:\n");
            for violation in &self.violations {
                let severity = if violation.is_critical { "CRITICAL" } else { "WARNING" };
                let threshold_desc = match violation.threshold_type {
                    ThresholdType::MaxTime => "Max execution time",
                    ThresholdType::MinImprovement => "Min improvement ratio",
                    ThresholdType::MaxMemory => "Max memory usage",
                    ThresholdType::MinMemoryReduction => "Min memory reduction",
                };
                
                report.push_str(&format!(
                    "  [{}] {}: {} - Expected: {:.2}, Actual: {:.2}\n",
                    severity, violation.benchmark_name, threshold_desc, 
                    violation.expected, violation.actual
                ));
            }
        } else {
            report.push_str("No violations detected.\n");
        }
        
        report
    }
}

impl std::fmt::Display for ThresholdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThresholdType::MaxTime => write!(f, "max_time"),
            ThresholdType::MinImprovement => write!(f, "min_improvement"),
            ThresholdType::MaxMemory => write!(f, "max_memory"),
            ThresholdType::MinMemoryReduction => write!(f, "min_memory_reduction"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_performance_validator_creation() {
        let validator = PerformanceValidator::new();
        assert!(!validator.thresholds.is_empty());
    }
    
    #[test]
    fn test_threshold_violation_detection() {
        let validator = PerformanceValidator::new();
        
        // Create a mock benchmark report that violates time threshold
        let mut results = HashMap::new();
        results.insert("value_creation".to_string(), crate::validation::benchmark_automation::BenchmarkResult {
            execution_time: Duration::from_millis(200), // Exceeds 100ms threshold
            memory_usage_bytes: Some(500_000),
            baseline_time: Some(Duration::from_millis(300)),
            baseline_memory_bytes: Some(800_000),
            iterations: 1000,
        });
        
        let report = BenchmarkReport { results };
        let result = validator.validate_report(&report).unwrap();
        
        assert!(!result.passed); // Should fail due to time violation
        assert_eq!(result.violations.len(), 1);
        assert!(result.violations[0].is_critical);
    }
    
    #[test] 
    fn test_improvement_ratio_calculation() {
        let validator = PerformanceValidator::new();
        
        let mut results = HashMap::new();
        results.insert("value_creation".to_string(), crate::validation::benchmark_automation::BenchmarkResult {
            execution_time: Duration::from_millis(50),
            memory_usage_bytes: Some(400_000),
            baseline_time: Some(Duration::from_millis(60)), // Only 20% improvement (1.2x), needs 40% (1.4x)
            baseline_memory_bytes: Some(800_000),
            iterations: 1000,
        });
        
        let report = BenchmarkReport { results };
        let result = validator.validate_report(&report).unwrap();
        
        // Should fail due to insufficient improvement
        assert!(!result.passed);
        let improvement_violations: Vec<_> = result.violations.iter()
            .filter(|v| matches!(v.threshold_type, ThresholdType::MinImprovement))
            .collect();
        assert_eq!(improvement_violations.len(), 1);
    }
}