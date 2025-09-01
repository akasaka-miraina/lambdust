#![allow(missing_docs)]//! Performance Regression Detection for Phase 8
//!
//! Detects performance regressions by comparing current results
//! against historical baselines.

use std::collections::HashMap;
use crate::validation::benchmark_automation::BenchmarkReport;

/// Regression detector that compares against baselines
pub struct RegressionDetector {
    baselines: HashMap<String, Baseline>,
}

/// Performance baseline for a benchmark
#[derive(Debug, Clone)]
pub struct Baseline {
    pub benchmark_name: String,
    pub avg_time_ms: f64,
    pub memory_usage_bytes: Option<usize>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Detected performance regression
#[derive(Debug)]
pub struct PerformanceRegression {
    pub benchmark_name: String,
    pub regression_type: RegressionType,
    pub baseline_value: f64,
    pub current_value: f64,
    pub regression_percentage: f64,
}

#[derive(Debug)]
pub enum RegressionType {
    ExecutionTime,
    MemoryUsage,
}

impl RegressionDetector {
    /// Create a new regression detector
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
        }
    }
    
    /// Check for regressions in a benchmark report
    pub fn check_for_regressions(&self, report: &BenchmarkReport) -> Result<Vec<PerformanceRegression>, String> {
        let mut regressions = Vec::new();
        
        for (benchmark_name, result) in &report.results {
            if let Some(baseline) = self.baselines.get(benchmark_name) {
                // Check execution time regression
                let current_time_ms = result.execution_time.as_millis() as f64;
                if current_time_ms > baseline.avg_time_ms * 1.1 { // 10% threshold
                    let regression_pct = ((current_time_ms - baseline.avg_time_ms) / baseline.avg_time_ms) * 100.0;
                    regressions.push(PerformanceRegression {
                        benchmark_name: benchmark_name.clone(),
                        regression_type: RegressionType::ExecutionTime,
                        baseline_value: baseline.avg_time_ms,
                        current_value: current_time_ms,
                        regression_percentage: regression_pct,
                    });
                }
                
                // Check memory regression
                if let (Some(current_memory), Some(baseline_memory)) = 
                   (result.memory_usage_bytes, baseline.memory_usage_bytes) {
                    if current_memory as f64 > baseline_memory as f64 * 1.1 { // 10% threshold
                        let regression_pct = ((current_memory as f64 - baseline_memory as f64) / baseline_memory as f64) * 100.0;
                        regressions.push(PerformanceRegression {
                            benchmark_name: benchmark_name.clone(),
                            regression_type: RegressionType::MemoryUsage,
                            baseline_value: baseline_memory as f64,
                            current_value: current_memory as f64,
                            regression_percentage: regression_pct,
                        });
                    }
                }
            }
        }
        
        Ok(regressions)
    }
    
    /// Add a baseline for a benchmark
    pub fn add_baseline(&mut self, baseline: Baseline) {
        self.baselines.insert(baseline.benchmark_name.clone(), baseline);
    }
}