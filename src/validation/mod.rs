//! Continuous Validation Pipeline for Phase 8 Optimization
//!
//! This module provides comprehensive validation infrastructure to ensure
//! that Phase 8 optimizations maintain correctness while delivering
//! performance improvements.

pub mod performance_validation;
pub mod correctness_validation;
pub mod regression_detection;
pub mod benchmark_automation;

pub use performance_validation::{PerformanceValidator, PerformanceThreshold, ValidationResult};
pub use correctness_validation::{CorrectnessValidator, TestSuite, ValidationError};
pub use regression_detection::{RegressionDetector, PerformanceRegression, Baseline};
pub use benchmark_automation::{BenchmarkRunner, BenchmarkConfig, BenchmarkReport};

use std::time::Duration;
use crate::feature::optimization_features::{OptimizationFeature, global_optimization_flags};

/// Main validation pipeline that orchestrates all validation activities
pub struct ValidationPipeline {
    performance_validator: PerformanceValidator,
    correctness_validator: CorrectnessValidator,
    regression_detector: RegressionDetector,
    benchmark_runner: BenchmarkRunner,
}

impl ValidationPipeline {
    /// Create a new validation pipeline with default configuration
    pub fn new() -> Self {
        Self {
            performance_validator: PerformanceValidator::new(),
            correctness_validator: CorrectnessValidator::new(),
            regression_detector: RegressionDetector::new(),
            benchmark_runner: BenchmarkRunner::new(),
        }
    }
    
    /// Run the complete validation pipeline
    pub fn run_full_validation(&mut self) -> ValidationPipelineResult {
        let mut results = ValidationPipelineResult::new();
        
        println!("🚀 Starting Phase 8 Optimization Validation Pipeline");
        
        // Step 1: Correctness validation
        println!("📋 Step 1: Running correctness validation...");
        let correctness_result = self.correctness_validator.validate_all();
        results.correctness_passed = correctness_result.is_ok();
        if let Err(e) = correctness_result {
            results.errors.push(format!("Correctness validation failed: {:?}", e));
        }
        
        // Step 2: Performance benchmarks
        println!("⏱️  Step 2: Running performance benchmarks...");
        match self.benchmark_runner.run_all_benchmarks() {
            Ok(report) => {
                results.benchmark_report = Some(report);
            }
            Err(e) => {
                results.errors.push(format!("Benchmark execution failed: {:?}", e));
            }
        }
        
        // Step 3: Performance validation against thresholds
        println!("📊 Step 3: Validating performance thresholds...");
        if let Some(ref report) = results.benchmark_report {
            let perf_result = self.performance_validator.validate_report(report);
            results.performance_passed = perf_result.is_ok();
            if let Err(e) = perf_result {
                results.errors.push(format!("Performance validation failed: {:?}", e));
            }
        }
        
        // Step 4: Regression detection
        println!("🔍 Step 4: Checking for performance regressions...");
        if let Some(ref report) = results.benchmark_report {
            match self.regression_detector.check_for_regressions(report) {
                Ok(regressions) => {
                    results.regressions = regressions;
                }
                Err(e) => {
                    results.errors.push(format!("Regression detection failed: {:?}", e));
                }
            }
        }
        
        // Step 5: Feature flag analysis
        println!("🏁 Step 5: Analyzing feature flag performance...");
        results.feature_analysis = self.analyze_feature_performance();
        
        println!("✅ Validation pipeline completed");
        results
    }
    
    /// Analyze performance of enabled optimization features
    fn analyze_feature_performance(&self) -> FeatureAnalysis {
        let flags = global_optimization_flags();
        let all_stats = flags.get_all_stats();
        
        let mut analysis = FeatureAnalysis {
            total_features: OptimizationFeature::all().len(),
            enabled_features: 0,
            well_performing_features: 0,
            problematic_features: Vec::new(),
            top_performers: Vec::new(),
        };
        
        for feature in OptimizationFeature::all() {
            if flags.is_enabled(feature) {
                analysis.enabled_features += 1;
                
                if let Some(stats) = all_stats.get(&feature) {
                    let success_rate = stats.success_rate();
                    let avg_time = stats.avg_time_micros();
                    
                    if success_rate >= 0.95 && stats.usage_count >= 100 {
                        analysis.well_performing_features += 1;
                        
                        if avg_time < 100.0 { // Under 100μs average
                            analysis.top_performers.push((feature, stats.clone()));
                        }
                    } else if success_rate < 0.8 || (stats.fallback_count > 0 && stats.usage_count > 50) {
                        analysis.problematic_features.push((feature, stats.clone()));
                    }
                }
            }
        }
        
        analysis
    }
    
    /// Run a quick validation (subset of full validation)
    pub fn run_quick_validation(&mut self) -> ValidationPipelineResult {
        let mut results = ValidationPipelineResult::new();
        
        println!("🚀 Starting Quick Validation");
        
        // Quick correctness check
        let correctness_result = self.correctness_validator.validate_critical();
        results.correctness_passed = correctness_result.is_ok();
        
        // Quick performance check
        match self.benchmark_runner.run_quick_benchmarks() {
            Ok(report) => {
                let perf_result = self.performance_validator.validate_report(&report);
                results.performance_passed = perf_result.is_ok();
                results.benchmark_report = Some(report);
            }
            Err(e) => {
                results.errors.push(format!("Quick benchmark failed: {:?}", e));
            }
        }
        
        results
    }
}

/// Results of the validation pipeline execution
#[derive(Debug)]
#[allow(missing_docs)]
pub struct ValidationPipelineResult {
    pub correctness_passed: bool,
    pub performance_passed: bool,
    pub benchmark_report: Option<BenchmarkReport>,
    pub regressions: Vec<PerformanceRegression>,
    pub feature_analysis: FeatureAnalysis,
    pub errors: Vec<String>,
    pub execution_time: Duration,
}

impl ValidationPipelineResult {
    fn new() -> Self {
        Self {
            correctness_passed: false,
            performance_passed: false,
            benchmark_report: None,
            regressions: Vec::new(),
            feature_analysis: FeatureAnalysis::default(),
            errors: Vec::new(),
            execution_time: Duration::from_secs(0),
        }
    }
    
    /// Check if validation passed overall
    pub fn is_successful(&self) -> bool {
        self.correctness_passed && 
        self.performance_passed && 
        self.regressions.is_empty() && 
        self.errors.is_empty()
    }
    
    /// Get a summary report
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str(&format!("=== Phase 8 Validation Summary ===\n"));
        summary.push_str(&format!("Overall Status: {}\n", 
            if self.is_successful() { "✅ PASSED" } else { "❌ FAILED" }
        ));
        
        summary.push_str(&format!("Correctness: {}\n", 
            if self.correctness_passed { "✅ PASSED" } else { "❌ FAILED" }
        ));
        
        summary.push_str(&format!("Performance: {}\n", 
            if self.performance_passed { "✅ PASSED" } else { "❌ FAILED" }
        ));
        
        if !self.regressions.is_empty() {
            summary.push_str(&format!("Regressions: {} detected\n", self.regressions.len()));
        }
        
        if !self.errors.is_empty() {
            summary.push_str(&format!("Errors: {} encountered\n", self.errors.len()));
        }
        
        // Feature analysis summary
        let fa = &self.feature_analysis;
        summary.push_str(&format!("Features: {}/{} enabled, {} performing well\n", 
            fa.enabled_features, fa.total_features, fa.well_performing_features
        ));
        
        if !fa.problematic_features.is_empty() {
            summary.push_str(&format!("Problematic features: {:?}\n", 
                fa.problematic_features.iter()
                    .map(|(f, _)| f.name())
                    .collect::<Vec<_>>()
            ));
        }
        
        summary
    }
}

/// Analysis of optimization feature performance
#[derive(Debug, Default)]
#[allow(missing_docs)]
pub struct FeatureAnalysis {
    pub total_features: usize,
    pub enabled_features: usize,
    pub well_performing_features: usize,
    pub problematic_features: Vec<(OptimizationFeature, crate::feature::optimization_features::FeatureStatsSnapshot)>,
    pub top_performers: Vec<(OptimizationFeature, crate::feature::optimization_features::FeatureStatsSnapshot)>,
}

/// Global validation pipeline instance
static GLOBAL_VALIDATION_PIPELINE: std::sync::OnceLock<std::sync::Mutex<ValidationPipeline>> = std::sync::OnceLock::new();

/// Get the global validation pipeline
pub fn global_validation_pipeline() -> &'static std::sync::Mutex<ValidationPipeline> {
    GLOBAL_VALIDATION_PIPELINE.get_or_init(|| std::sync::Mutex::new(ValidationPipeline::new()))
}

/// Run validation and return results
pub fn run_validation() -> ValidationPipelineResult {
    let pipeline = global_validation_pipeline();
    let mut pipeline = pipeline.lock().unwrap();
    pipeline.run_full_validation()
}

/// Run quick validation and return results  
pub fn run_quick_validation() -> ValidationPipelineResult {
    let pipeline = global_validation_pipeline();
    let mut pipeline = pipeline.lock().unwrap();
    pipeline.run_quick_validation()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_pipeline_creation() {
        let pipeline = ValidationPipeline::new();
        // Should create without panic
    }
    
    #[test]
    fn test_validation_result_summary() {
        let mut result = ValidationPipelineResult::new();
        result.correctness_passed = true;
        result.performance_passed = true;
        
        let summary = result.summary();
        assert!(summary.contains("PASSED"));
    }
    
    #[test]
    fn test_feature_analysis_default() {
        let analysis = FeatureAnalysis::default();
        assert_eq!(analysis.enabled_features, 0);
        assert_eq!(analysis.well_performing_features, 0);
    }
}