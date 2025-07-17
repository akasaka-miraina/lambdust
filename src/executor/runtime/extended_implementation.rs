//! Extended Runtime Executor Implementation
//!
//! このモジュールはRuntime Executorの拡張実装とデフォルト実装を
//! 提供します。

use super::core_types::{RuntimeExecutor, AdaptiveOptimizationType, ExpressionAnalysisResult, CallPattern, ExecutionFrequency, RuntimeStats};
use super::performance_reporting::RuntimePerformanceReport;
use crate::executor::runtime_optimization::{OptimizationRecommendation, RecommendationType, RecommendationPriority, ImplementationEffort};
use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::Result;
use crate::evaluator::{
    Continuation,
    ExecutionContext,
    continuation_pooling::{ContinuationType},
};
use crate::value::Value;
use std::rc::Rc;

impl RuntimeExecutor {
    /// Get continuation pooling statistics
    #[must_use] pub fn get_pooling_statistics(&self) -> (usize, usize, usize, f64) {
        self.continuation_pooler().global_statistics()
    }
    
    /// Get detailed pooling statistics by type
    #[must_use] pub fn get_detailed_pooling_statistics(&self) -> Vec<(ContinuationType, f64)> {
        self.continuation_pooler().all_statistics()
            .into_iter()
            .map(|(cont_type, stats)| (cont_type, stats.reuse_efficiency()))
            .collect()
    }
    
    /// Check if memory defragmentation is needed for continuation pools
    #[must_use] pub fn needs_pool_defragmentation(&self) -> bool {
        self.continuation_pooler().needs_defragmentation()
    }
    
    /// Perform continuation pool defragmentation
    pub fn defragment_pools(&mut self) {
        self.continuation_pooler_mut().defragment();
    }
    
    /// Clear all continuation pools
    pub fn clear_continuation_pools(&mut self) {
        self.continuation_pooler_mut().clear_all();
    }
    
    /// Optimize continuation pooling based on runtime statistics
    pub fn optimize_continuation_pooling(&mut self) {
        // Check if defragmentation is needed
        if self.needs_pool_defragmentation() {
            self.defragment_pools();
            // Pool defragmentation tracking no longer used (removed embedded statistics)
        }
        
        // Clear underused pools to free memory
        let detailed_stats = self.get_detailed_pooling_statistics();
        for (cont_type, efficiency) in detailed_stats {
            // Clear pools with very low efficiency
            if efficiency < 10.0 {
                self.continuation_pooler_mut().clear_type(cont_type);
            }
        }
        
        // Adaptive pool management
        self.adaptive_pool_management();
    }
    
    /// Use adaptive engine for dynamic optimization decisions
    pub fn apply_adaptive_optimization(&mut self, execution_context: &ExecutionContext) -> Result<Value> {
        // Get adaptive optimization recommendation
        let optimization_type = self.adaptive_engine_mut().get_optimization_recommendation(
            &execution_context.expression,
            &execution_context.static_analysis
        );
        
        // Apply the recommended optimization
        match optimization_type {
            AdaptiveOptimizationType::NoOptimization => {
                // Use standard evaluation path
                self.execute_with_semantic_evaluator(execution_context)
            }
            AdaptiveOptimizationType::JitCompilation => {
                // Use JIT compilation for hot paths
                self.execute_with_jit_optimization(execution_context)
            }
            AdaptiveOptimizationType::AdaptiveLoopUnrolling { factor } => {
                // Apply loop unrolling with specified factor
                self.execute_with_loop_unrolling(execution_context, factor)
            }
            AdaptiveOptimizationType::HotPathInlining => {
                // Use inline evaluation for hot paths
                self.execute_with_inline_evaluation(execution_context)
            }
            AdaptiveOptimizationType::TypeSpecialization => {
                // Apply type-specific optimizations
                self.execute_with_type_specialization(execution_context)
            }
            AdaptiveOptimizationType::None => {
                // No optimization, use basic evaluation
                self.execute_with_semantic_evaluator(execution_context)
            },
            AdaptiveOptimizationType::Inline => {
                // Inline optimization
                self.execute_with_inline_evaluation(execution_context)
            },
            AdaptiveOptimizationType::JitCompile => {
                // JIT compilation
                self.execute_with_jit_optimization(execution_context)
            },
            AdaptiveOptimizationType::TailCallOptimize => {
                // Tail call optimization
                self.execute_with_semantic_evaluator(execution_context) // TODO: Implement tail call optimization
            },
            AdaptiveOptimizationType::TypeSpecialize => {
                // Type specialization
                self.execute_with_type_specialization(execution_context)
            },
            AdaptiveOptimizationType::LoopUnroll { factor } => {
                // Loop unrolling with factor
                self.execute_with_loop_unrolling(execution_context, factor)
            },
            AdaptiveOptimizationType::ContinuationPooling |
            AdaptiveOptimizationType::MemoryLayoutOptimization |
            AdaptiveOptimizationType::ProfileGuidedOptimization |
            _ => {
                // Catch-all for any remaining patterns
                self.execute_with_semantic_evaluator(execution_context)
            }
        }
    }
    
    /// Execute using JIT optimization
    fn execute_with_jit_optimization(&mut self, execution_context: &ExecutionContext) -> Result<Value> {
        // For now, fallback to semantic evaluator with JIT hint
        // In a full implementation, this would use JIT-compiled code
        self.execute_with_semantic_evaluator(execution_context)
    }
    
    /// Execute with loop unrolling optimization
    fn execute_with_loop_unrolling(&mut self, execution_context: &ExecutionContext, factor: u32) -> Result<Value> {
        // Apply loop unrolling transformation
        let optimized_expr = self.apply_loop_unrolling(&execution_context.expression, factor)?;
        let optimized_context = ExecutionContext::new(optimized_expr, execution_context.environment.clone(), execution_context.continuation.clone());
        self.execute_with_semantic_evaluator(&optimized_context)
    }
    
    /// Execute with inline evaluation
    fn execute_with_inline_evaluation(&mut self, execution_context: &ExecutionContext) -> Result<Value> {
        // For now, fallback to semantic evaluator with inline hint
        // In a full implementation, this would inline simple expressions
        self.execute_with_semantic_evaluator(execution_context)
    }
    
    /// Execute with type specialization
    fn execute_with_type_specialization(&mut self, execution_context: &ExecutionContext) -> Result<Value> {
        // Apply type-specific optimizations based on detected types
        let specialized_expr = self.apply_type_specialization(&execution_context.expression)?;
        let optimized_context = ExecutionContext::new(specialized_expr, execution_context.environment.clone(), execution_context.continuation.clone());
        self.execute_with_semantic_evaluator(&optimized_context)
    }
    
    /// Apply loop unrolling transformation
    fn apply_loop_unrolling(&self, expr: &Expr, _factor: u32) -> Result<Expr> {
        // Simplified loop unrolling implementation
        // In a full implementation, this would detect and unroll loop constructs
        Ok(expr.clone())
    }
    
    /// Apply type specialization based on detected types
    fn apply_type_specialization(&self, expr: &Expr) -> Result<Expr> {
        // Simplified type specialization implementation
        // In a full implementation, this would specialize operations based on types
        Ok(expr.clone())
    }
    
    /// Execute with semantic evaluator (fallback method)
    fn execute_with_semantic_evaluator(&self, execution_context: &ExecutionContext) -> Result<Value> {
        // Use the semantic evaluator for correct evaluation
        self.semantic_evaluator().eval_pure_functional(
            execution_context.expression.clone(),
            execution_context.environment.clone(),
            execution_context.continuation.clone(),
            0 // Use 0 as starting recursion depth for this context
        )
    }

    /// Adaptive pool management based on usage patterns
    fn adaptive_pool_management(&mut self) {
        // Adaptive pool management has been simplified for decoupled statistics
        // In a full implementation, this would use environment-based performance metrics
        // For now, we use basic pool management based on available metrics
        
        // Simple adaptive management based on pooling efficiency
        let global_stats = self.get_pooling_statistics();
        let (pool_hits, pool_misses, _total_pools, efficiency) = global_stats;
        
        if pool_hits + pool_misses > 1000
            && efficiency < 50.0 {
                // Clear all pools and let them rebuild with current patterns
                self.clear_continuation_pools();
            }
    }

    /// Try JIT loop optimization using the integrated JIT loop optimizer
    pub fn try_jit_loop_optimization(
        &mut self,
        expr: &Expr,
        env: &Rc<Environment>,
        _cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        // Check if this is a do-loop expression that can be optimized
        if let Expr::List(operands) = expr {
            if !operands.is_empty() {
                if let Expr::Variable(name) = &operands[0] {
                    if name == "do" {
                        // Try JIT optimization for do-loop
                        if let Some(do_expr) = operands.get(1) {
                            if let Ok(Some(result)) = self.jit_optimizer_mut().try_optimize(
                                do_expr,
                                env.clone(),
                            ) {
                                return Ok(Some(result));
                            }
                        }
                    }
                }
            }
        }

        // Check for other loop patterns based on call patterns
        for pattern in &analysis.call_patterns {
            if let CallPattern::Loop { iteration_estimate } = pattern {
                if let Some(iterations) = iteration_estimate {
                    if *iterations > 10 {
                        // For high iteration loops, attempt optimization
                        // Note: Direct JIT stats access simplified for now
                        
                        // Try JIT optimization for high iteration patterns
                        if let Ok(Some(result)) = self.jit_optimizer_mut().try_optimize(
                            expr,
                            env.clone(),
                        ) {
                            return Ok(Some(result));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Try JIT compilation for general expressions
    pub fn try_jit_compilation(
        &mut self,
        expr: &Expr,
        env: &Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        // First try JIT loop optimization if applicable
        if analysis.contains_loops {
            if let Some(result) = self.try_jit_loop_optimization(expr, env, cont.clone(), analysis)? {
                return Ok(Some(result));
            }
        }

        // For non-loop expressions, we would implement general JIT compilation here
        // This is a placeholder for future JIT compilation of general expressions
        Ok(None)
    }

    /// Try LLVM native code generation for critical performance paths
    pub fn try_llvm_compilation(
        &mut self,
        #[cfg_attr(not(feature = "development"), allow(unused_variables))]
        expr: &Expr,
        _env: &Rc<Environment>,
        _cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {

        // Only attempt LLVM compilation for very hot paths with specific characteristics
        if analysis.execution_frequency != ExecutionFrequency::Critical {
            return Ok(None);
        }

#[cfg(feature = "development")]
        {
            // Create tail call context for LLVM compilation
            let tail_context = TailCallContext::new();

            // Attempt to compile the expression to LLVM IR
            let llvm_result = self.llvm_compiler_mut().compile_with_tail_calls(expr, &tail_context);

            match llvm_result {
                Ok(_llvm_ir) => {
                    // In a real implementation, we would:
                    // 1. Compile the LLVM IR to native code
                    // 2. Cache the compiled code
                    // 3. Execute the native code
                    // 
                    // For now, we return None to indicate that LLVM compilation
                    // is not yet fully implemented
                    return Ok(None);
                }
                Err(_) => {
                    // LLVM compilation failed, fall back to regular evaluation
                    return Ok(None);
                }
            }
        }
        
        // Fallback for non-development builds
        Ok(None)
    }
    
    /// Get comprehensive runtime performance report
    #[must_use] pub fn generate_performance_report(&self) -> RuntimePerformanceReport {
        RuntimePerformanceReport {
            runtime_stats: RuntimeStats::default(), // Basic placeholder stats
            pooling_stats: self.get_pooling_statistics(),
            detailed_pooling: self.get_detailed_pooling_statistics(),
            memory_usage: self.continuation_pooler().memory_usage_summary(),
            optimization_recommendations: self.generate_optimization_recommendations().into_iter().map(|rec| super::performance_reporting::OptimizationRecommendation {
                category: super::performance_reporting::RecommendationCategory::GeneralOptimization,
                priority: super::performance_reporting::RecommendationPriority::Medium,
                description: rec.description,
                estimated_benefit: format!("{:.1}x speedup", rec.expected_benefit),
            }).collect(),
        }
    }
    
    /// Generate optimization recommendations based on current performance
    fn generate_optimization_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        // Pool efficiency recommendations based on available metrics
        let (pool_hits, pool_misses, _total_pools, pool_efficiency) = self.get_pooling_statistics();
        if pool_efficiency < 30.0 && pool_hits + pool_misses > 100 {
            recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::ResourceAdjustment,
                priority: RecommendationPriority::Medium,
                description: "Low continuation pool efficiency detected. Consider adjusting pool sizes or clearing underused pools.".to_string(),
                expected_benefit: 0.15, // 15% improvement
                implementation_effort: ImplementationEffort::Low,
            });
        }
        
        // JIT optimization recommendations based on available data
        // Note: In decoupled statistics, specific hot path counts would come from environment
        let detailed_pooling = self.get_detailed_pooling_statistics();
        if detailed_pooling.len() > 10 {
            recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::AlgorithmChange,
                priority: RecommendationPriority::High,
                description: "Multiple continuation types detected. Consider enabling JIT compilation for frequently used patterns.".to_string(),
                expected_benefit: 0.30, // 30% improvement
                implementation_effort: ImplementationEffort::Medium,
            });
        }
        
        // General optimization recommendations
        if pool_hits + pool_misses > 1000 && pool_efficiency > 80.0 {
            recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::ConfigurationOptimization,
                priority: RecommendationPriority::Medium,
                description: "High pool usage with good efficiency. Consider increasing pool sizes for better performance.".to_string(),
                expected_benefit: 0.20, // 20% improvement
                implementation_effort: ImplementationEffort::Low,
            });
        }
        
        recommendations
    }
}

impl Default for RuntimeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic performance metrics
#[derive(Debug, Clone, Default)]
pub struct BasicPerformanceMetrics {
    /// Number of executions recorded
    pub execution_count: u64,
    /// Average execution time in nanoseconds
    pub average_time_ns: u64,
    /// Total accumulated execution time in nanoseconds
    pub total_time_ns: u64,
}

/// Timing details
#[derive(Debug, Clone, Default)]
pub struct TimingDetails {
    /// Minimum execution time in nanoseconds
    pub min_time_ns: u64,
    /// Maximum execution time in nanoseconds
    pub max_time_ns: u64,
    /// Standard deviation of execution times
    pub standard_deviation: f64,
}

/// Memory usage statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStatistics {
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
    /// Average memory usage in bytes
    pub average_memory_bytes: usize,
    /// Total number of allocations
    pub allocations: u64,
}

/// Optimization effectiveness
#[derive(Debug, Clone, Default)]
pub struct OptimizationEffectiveness {
    /// List of applied optimization techniques
    pub applied_optimizations: Vec<String>,
    /// Performance improvement factor (1.0 = no improvement)
    pub performance_improvement: f64,
    /// Memory usage reduction factor (1.0 = no reduction)
    pub memory_reduction: f64,
}

/// Comprehensive runtime performance report
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct ComprehensivePerformanceReport {
    /// Basic performance metrics
    pub basic_metrics: BasicPerformanceMetrics,
    
    /// Detailed timing information
    pub timing_details: TimingDetails,
    
    /// Memory usage statistics
    pub memory_statistics: MemoryStatistics,
    
    /// Optimization effectiveness
    pub optimization_effectiveness: OptimizationEffectiveness,
}

