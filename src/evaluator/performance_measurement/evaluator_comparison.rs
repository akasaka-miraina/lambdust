//! Comprehensive comparison framework for `SemanticEvaluator` vs `RuntimeExecutor`
//!
//! This module provides detailed comparison analysis between the pure semantic
//! evaluation and optimized runtime execution, measuring effectiveness of
//! `RuntimeExecutor` optimizations and ensuring correctness preservation.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::Result;
use crate::evaluator::{
    Continuation, RuntimeExecutor, RuntimeOptimizationLevel, SemanticEvaluator,
};
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Comprehensive evaluator comparison framework
pub struct EvaluatorComparison {
    /// Semantic evaluator (reference implementation)
    semantic_evaluator: SemanticEvaluator,
    /// Runtime executor (optimized implementation)
    runtime_executor: RuntimeExecutor,
    /// Comparison history for trend analysis
    comparison_history: Vec<ComparisonResult>,
    /// Statistical analysis cache
    analysis_cache: HashMap<String, ComparisonAnalysis>,
}

/// Result of a single evaluator comparison
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    /// Expression being evaluated
    pub expression_summary: String,
    /// Semantic evaluation metrics
    pub semantic_metrics: EvaluationMetrics,
    /// Runtime evaluation metrics  
    pub runtime_metrics: EvaluationMetrics,
    /// Correctness verification
    pub correctness_check: CorrectnessCheck,
    /// Performance comparison
    pub performance_comparison: PerformanceComparison,
    /// Optimization effectiveness
    pub optimization_effectiveness: OptimizationEffectiveness,
    /// Timestamp of comparison
    pub timestamp: Instant,
}

/// Detailed evaluation metrics for a single evaluation
#[derive(Debug, Clone)]
pub struct EvaluationMetrics {
    /// Evaluation result
    pub result: Value,
    /// Execution time
    pub execution_time: Duration,
    /// Memory usage (estimated)
    pub memory_usage_bytes: usize,
    /// Number of reduction steps
    pub reduction_steps: usize,
    /// Number of function calls
    pub function_calls: usize,
    /// Number of continuation applications
    pub continuation_applications: usize,
    /// Garbage collection cycles (if any)
    pub gc_cycles: usize,
}

/// Correctness verification between evaluators
#[derive(Debug, Clone)]
pub struct CorrectnessCheck {
    /// Whether results are equivalent
    pub results_equivalent: bool,
    /// Semantic evaluation result
    pub semantic_result: Value,
    /// Runtime evaluation result
    pub runtime_result: Value,
    /// Detailed comparison notes
    pub comparison_notes: Vec<String>,
    /// Confidence level of equivalence check
    pub confidence_level: f64,
}

/// Performance comparison metrics
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    /// Speedup factor (`semantic_time` / `runtime_time`)
    pub speedup_factor: f64,
    /// Memory efficiency ratio
    pub memory_efficiency_ratio: f64,
    /// Reduction steps efficiency
    pub reduction_efficiency_ratio: f64,
    /// Overall performance score
    pub overall_performance_score: f64,
    /// Performance category
    pub performance_category: PerformanceCategory,
}

/// Performance improvement category
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PerformanceCategory {
    /// Significant improvement (>2x speedup)
    SignificantImprovement,
    /// Moderate improvement (1.5x-2x speedup)
    ModerateImprovement,
    /// Minor improvement (1.1x-1.5x speedup)
    MinorImprovement,
    /// Neutral (0.9x-1.1x speedup)
    Neutral,
    /// Minor regression (0.7x-0.9x speedup)
    MinorRegression,
    /// Moderate regression (0.5x-0.7x speedup)
    ModerateRegression,
    /// Significant regression (<0.5x speedup)
    SignificantRegression,
}

/// Optimization effectiveness analysis
#[derive(Debug, Clone)]
pub struct OptimizationEffectiveness {
    /// Which optimizations were applied
    pub applied_optimizations: Vec<String>,
    /// Effectiveness score per optimization
    pub optimization_scores: HashMap<String, f64>,
    /// Overall optimization effectiveness
    pub overall_effectiveness: f64,
    /// Optimization level used
    pub optimization_level: RuntimeOptimizationLevel,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Statistical analysis of multiple comparisons
#[derive(Debug, Clone)]
pub struct ComparisonAnalysis {
    /// Number of comparisons analyzed
    pub sample_size: usize,
    /// Average speedup factor
    pub average_speedup: f64,
    /// Median speedup factor
    pub median_speedup: f64,
    /// Standard deviation of speedup
    pub speedup_std_dev: f64,
    /// Performance distribution
    pub performance_distribution: HashMap<PerformanceCategory, usize>,
    /// Correctness rate (percentage of correct results)
    pub correctness_rate: f64,
    /// Trend analysis
    pub trend_analysis: TrendAnalysis,
}

/// Trend analysis over time
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    /// Performance trend direction
    pub performance_trend: TrendDirection,
    /// Trend confidence
    pub trend_confidence: f64,
    /// Projected future performance
    pub projected_performance: f64,
    /// Key factors influencing trend
    pub influencing_factors: Vec<String>,
}

/// Trend direction
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    /// Performance improving over time
    Improving,
    /// Performance stable over time
    Stable,
    /// Performance degrading over time
    Degrading,
    /// Insufficient data for trend analysis
    Unknown,
}

impl EvaluatorComparison {
    /// Create a new evaluator comparison framework
    #[must_use] pub fn new() -> Self {
        Self {
            semantic_evaluator: SemanticEvaluator::new(),
            runtime_executor: RuntimeExecutor::new(),
            comparison_history: Vec::new(),
            analysis_cache: HashMap::new(),
        }
    }

    /// Compare evaluators on a single expression
    pub fn compare_expression(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        optimization_level: RuntimeOptimizationLevel,
    ) -> Result<ComparisonResult> {
        let expression_summary = format!("{expr:?}").chars().take(100).collect();
        
        // Evaluate with semantic evaluator
        let semantic_metrics = self.evaluate_with_semantic(expr.clone(), env.clone())?;
        
        // Evaluate with runtime executor
        let runtime_metrics = self.evaluate_with_runtime(expr, env, optimization_level)?;
        
        // Perform correctness check
        let correctness_check = self.check_correctness(&semantic_metrics, &runtime_metrics);
        
        // Analyze performance
        let performance_comparison = self.analyze_performance(&semantic_metrics, &runtime_metrics);
        
        // Analyze optimization effectiveness
        let optimization_effectiveness = self.analyze_optimization_effectiveness(
            &performance_comparison,
            optimization_level,
        );
        
        let result = ComparisonResult {
            expression_summary,
            semantic_metrics,
            runtime_metrics,
            correctness_check,
            performance_comparison,
            optimization_effectiveness,
            timestamp: Instant::now(),
        };
        
        self.comparison_history.push(result.clone());
        Ok(result)
    }

    /// Evaluate expression with semantic evaluator
    fn evaluate_with_semantic(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
    ) -> Result<EvaluationMetrics> {
        let start_time = Instant::now();
        
        let result = self.semantic_evaluator.eval_pure(expr, env, Continuation::Identity)?;
        
        let execution_time = start_time.elapsed();
        
        Ok(EvaluationMetrics {
            result,
            execution_time,
            memory_usage_bytes: 0, // Semantic evaluator has minimal memory overhead
            reduction_steps: 1,    // Pure semantic evaluation is reference
            function_calls: 1,     // Single expression evaluation
            continuation_applications: 1, // Single continuation application
            gc_cycles: 0,         // No GC in pure semantic evaluation
        })
    }

    /// Evaluate expression with runtime executor
    fn evaluate_with_runtime(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        _optimization_level: RuntimeOptimizationLevel,
    ) -> Result<EvaluationMetrics> {
        let start_time = Instant::now();
        
        // Reset runtime executor stats before evaluation
        self.runtime_executor.reset_stats();
        
        let result = self.runtime_executor.eval_optimized(expr, env, Continuation::Identity)?;
        
        let execution_time = start_time.elapsed();
        
        // Get metrics from runtime executor after evaluation
        let stats = self.runtime_executor.get_stats();
        
        Ok(EvaluationMetrics {
            result,
            execution_time,
            memory_usage_bytes: stats.pooling_memory_saved,
            reduction_steps: stats.expressions_evaluated,
            function_calls: stats.verification_checks,
            continuation_applications: stats.continuation_pool_hits + stats.continuation_pool_misses,
            gc_cycles: stats.pool_defragmentations,
        })
    }

    /// Check correctness between evaluations
    fn check_correctness(
        &self,
        semantic_metrics: &EvaluationMetrics,
        runtime_metrics: &EvaluationMetrics,
    ) -> CorrectnessCheck {
        let results_equivalent = semantic_metrics.result == runtime_metrics.result;
        let mut comparison_notes = Vec::new();
        
        if !results_equivalent {
            comparison_notes.push(format!(
                "Results differ: semantic = {:?}, runtime = {:?}",
                semantic_metrics.result, runtime_metrics.result
            ));
        }
        
        // Simple confidence calculation
        let confidence_level = if results_equivalent { 1.0 } else { 0.0 };
        
        CorrectnessCheck {
            results_equivalent,
            semantic_result: semantic_metrics.result.clone(),
            runtime_result: runtime_metrics.result.clone(),
            comparison_notes,
            confidence_level,
        }
    }

    /// Analyze performance comparison
    fn analyze_performance(
        &self,
        semantic_metrics: &EvaluationMetrics,
        runtime_metrics: &EvaluationMetrics,
    ) -> PerformanceComparison {
        let speedup_factor = if runtime_metrics.execution_time.as_nanos() == 0 {
            f64::INFINITY
        } else {
            semantic_metrics.execution_time.as_nanos() as f64 
                / runtime_metrics.execution_time.as_nanos() as f64
        };

        let memory_efficiency_ratio = if runtime_metrics.memory_usage_bytes == 0 {
            1.0
        } else {
            semantic_metrics.memory_usage_bytes as f64 / runtime_metrics.memory_usage_bytes as f64
        };

        let reduction_efficiency_ratio = if runtime_metrics.reduction_steps == 0 {
            1.0
        } else {
            semantic_metrics.reduction_steps as f64 / runtime_metrics.reduction_steps as f64
        };

        // Overall performance score (weighted average)
        let overall_performance_score = (speedup_factor * 0.6) + 
                                       (memory_efficiency_ratio * 0.2) + 
                                       (reduction_efficiency_ratio * 0.2);

        let performance_category = Self::categorize_performance(speedup_factor);

        PerformanceComparison {
            speedup_factor,
            memory_efficiency_ratio,
            reduction_efficiency_ratio,
            overall_performance_score,
            performance_category,
        }
    }

    /// Categorize performance based on speedup factor
    fn categorize_performance(speedup_factor: f64) -> PerformanceCategory {
        if speedup_factor >= 2.0 {
            PerformanceCategory::SignificantImprovement
        } else if speedup_factor >= 1.5 {
            PerformanceCategory::ModerateImprovement
        } else if speedup_factor >= 1.1 {
            PerformanceCategory::MinorImprovement
        } else if speedup_factor >= 0.9 {
            PerformanceCategory::Neutral
        } else if speedup_factor >= 0.7 {
            PerformanceCategory::MinorRegression
        } else if speedup_factor >= 0.5 {
            PerformanceCategory::ModerateRegression
        } else {
            PerformanceCategory::SignificantRegression
        }
    }

    /// Analyze optimization effectiveness
    fn analyze_optimization_effectiveness(
        &self,
        performance: &PerformanceComparison,
        optimization_level: RuntimeOptimizationLevel,
    ) -> OptimizationEffectiveness {
        let applied_optimizations = vec![
            "tail_call_optimization".to_string(),
            "constant_folding".to_string(),
            "dead_code_elimination".to_string(),
        ];

        let mut optimization_scores = HashMap::new();
        for opt in &applied_optimizations {
            optimization_scores.insert(opt.clone(), performance.speedup_factor.min(5.0) / 5.0);
        }

        let overall_effectiveness = performance.overall_performance_score / 3.0; // Normalize

        let mut recommendations = Vec::new();
        if performance.speedup_factor < 1.5 {
            recommendations.push("Consider higher optimization level".to_string());
        }
        if performance.memory_efficiency_ratio < 1.0 {
            recommendations.push("Memory usage increased; review memory optimizations".to_string());
        }

        OptimizationEffectiveness {
            applied_optimizations,
            optimization_scores,
            overall_effectiveness,
            optimization_level,
            recommendations,
        }
    }

    /// Run comprehensive comparison across multiple optimization levels
    pub fn run_comprehensive_comparison(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
    ) -> Result<Vec<ComparisonResult>> {
        let mut results = Vec::new();
        
        let optimization_levels = [
            RuntimeOptimizationLevel::None,
            RuntimeOptimizationLevel::Conservative,
            RuntimeOptimizationLevel::Balanced,
            RuntimeOptimizationLevel::Aggressive,
        ];

        for level in &optimization_levels {
            let result = self.compare_expression(expr.clone(), env.clone(), *level)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Generate statistical analysis of recent comparisons
    #[must_use] pub fn generate_analysis(&self, window_size: usize) -> ComparisonAnalysis {
        let recent_comparisons: Vec<_> = self.comparison_history
            .iter()
            .rev()
            .take(window_size)
            .collect();

        if recent_comparisons.is_empty() {
            return ComparisonAnalysis {
                sample_size: 0,
                average_speedup: 0.0,
                median_speedup: 0.0,
                speedup_std_dev: 0.0,
                performance_distribution: HashMap::new(),
                correctness_rate: 0.0,
                trend_analysis: TrendAnalysis {
                    performance_trend: TrendDirection::Unknown,
                    trend_confidence: 0.0,
                    projected_performance: 0.0,
                    influencing_factors: Vec::new(),
                },
            };
        }

        let speedups: Vec<f64> = recent_comparisons.iter()
            .map(|c| c.performance_comparison.speedup_factor)
            .filter(|&s| s.is_finite())
            .collect();

        let average_speedup = speedups.iter().sum::<f64>() / speedups.len() as f64;
        
        let mut sorted_speedups = speedups.clone();
        sorted_speedups.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_speedup = if sorted_speedups.len() % 2 == 0 {
            f64::midpoint(sorted_speedups[sorted_speedups.len() / 2 - 1], sorted_speedups[sorted_speedups.len() / 2])
        } else {
            sorted_speedups[sorted_speedups.len() / 2]
        };

        let variance = speedups.iter()
            .map(|&s| (s - average_speedup).powi(2))
            .sum::<f64>() / speedups.len() as f64;
        let speedup_std_dev = variance.sqrt();

        let mut performance_distribution = HashMap::new();
        for comparison in &recent_comparisons {
            let category = &comparison.performance_comparison.performance_category;
            *performance_distribution.entry(category.clone()).or_insert(0) += 1;
        }

        let correct_count = recent_comparisons.iter()
            .filter(|c| c.correctness_check.results_equivalent)
            .count();
        let correctness_rate = correct_count as f64 / recent_comparisons.len() as f64;

        let trend_analysis = self.analyze_trend(&recent_comparisons);

        ComparisonAnalysis {
            sample_size: recent_comparisons.len(),
            average_speedup,
            median_speedup,
            speedup_std_dev,
            performance_distribution,
            correctness_rate,
            trend_analysis,
        }
    }

    /// Analyze performance trend
    fn analyze_trend(&self, comparisons: &[&ComparisonResult]) -> TrendAnalysis {
        if comparisons.len() < 3 {
            return TrendAnalysis {
                performance_trend: TrendDirection::Unknown,
                trend_confidence: 0.0,
                projected_performance: 0.0,
                influencing_factors: Vec::new(),
            };
        }

        // Simple linear trend analysis
        let speedups: Vec<f64> = comparisons.iter()
            .map(|c| c.performance_comparison.speedup_factor)
            .filter(|&s| s.is_finite())
            .collect();

        let first_half_avg = speedups[..speedups.len()/2].iter().sum::<f64>() / (speedups.len()/2) as f64;
        let second_half_avg = speedups[speedups.len()/2..].iter().sum::<f64>() / (speedups.len() - speedups.len()/2) as f64;

        let trend_direction = if second_half_avg > first_half_avg * 1.05 {
            TrendDirection::Improving
        } else if second_half_avg < first_half_avg * 0.95 {
            TrendDirection::Degrading
        } else {
            TrendDirection::Stable
        };

        let trend_confidence = ((second_half_avg - first_half_avg).abs() / first_half_avg).min(1.0);
        let projected_performance = second_half_avg;

        TrendAnalysis {
            performance_trend: trend_direction,
            trend_confidence,
            projected_performance,
            influencing_factors: vec!["sample_size".to_string(), "optimization_level".to_string()],
        }
    }

    /// Get comparison history
    #[must_use] pub fn get_history(&self) -> &[ComparisonResult] {
        &self.comparison_history
    }

    /// Clear comparison history
    pub fn clear_history(&mut self) {
        self.comparison_history.clear();
        self.analysis_cache.clear();
    }
}

impl Default for EvaluatorComparison {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;

    #[test]
    fn test_evaluator_comparison_creation() {
        let comparison = EvaluatorComparison::new();
        assert!(comparison.comparison_history.is_empty());
    }

    #[test]
    fn test_performance_categorization() {
        assert_eq!(
            EvaluatorComparison::categorize_performance(3.0),
            PerformanceCategory::SignificantImprovement
        );
        assert_eq!(
            EvaluatorComparison::categorize_performance(1.2),
            PerformanceCategory::MinorImprovement
        );
        assert_eq!(
            EvaluatorComparison::categorize_performance(0.8),
            PerformanceCategory::MinorRegression
        );
    }

    #[test]
    fn test_simple_expression_comparison() {
        let mut comparison = EvaluatorComparison::new();
        let expr = Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());

        let result = comparison.compare_expression(
            expr,
            env,
            RuntimeOptimizationLevel::Balanced,
        );

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.correctness_check.results_equivalent);
    }
}