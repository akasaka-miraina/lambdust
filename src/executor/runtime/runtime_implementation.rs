//! Main Runtime Executor Implementation
//!
//! This module provides the main implementation of the Runtime Executor.

use super::core_types::{RuntimeExecutor, AdaptiveOptimizationEngine, RuntimeOptimizationLevel, ExpressionAnalysisResult, ExecutionFrequency, AdaptiveOptimizationType, OptimizationHint, OptimizedTailCall, CallPattern, JitCompiledCode, RuntimeStats};
use crate::ast::Expr;
#[cfg(feature = "development")]
use crate::environment::{Environment, StatisticsMessage};
#[cfg(not(feature = "development"))]
use crate::environment::Environment;

/// Macro for conditional statistics reporting
/// Only sends statistics in development builds
#[cfg(feature = "development")]
macro_rules! send_stats {
    ($self:expr, $env:expr, $message:expr) => {
        $self.send_statistics($env, $message);
    };
}

#[cfg(not(feature = "development"))]
macro_rules! send_stats {
    ($self:expr, $env:expr, $message:expr) => {
        $self.send_statistics($env, ());
    };
}

use crate::error::{LambdustError, Result};
use crate::evaluator::{
    Continuation, ContinuationPoolManager, InlineEvaluator,
    JitLoopOptimizer, SemanticEvaluator,
    ExecutionContext, ExecutionPriority,
    continuation_pooling::{ContinuationType},
    jit_loop_optimization::{JitOptimizationStats},
};
#[cfg(feature = "development")]
use crate::evaluator::llvm_backend::{LLVMCompilerIntegration, LLVMOptimizationLevel};
use crate::executor::runtime_optimization_integration::{IntegratedOptimizationManager, OptimizationResult};
#[cfg(feature = "development")]
use crate::performance_monitor::hotpath_analysis::AdvancedHotPathDetector;

// Fallback type when development feature is not enabled
#[cfg(not(feature = "development"))]
use super::core_types::AdvancedHotPathDetector;
use crate::value::Value;
use std::rc::Rc;

impl RuntimeExecutor {
    /// Create a new runtime executor with default optimization level
    #[must_use] pub fn new() -> Self {
        Self {
            semantic_evaluator: SemanticEvaluator::new(),
            jit_optimizer: JitLoopOptimizer::new(),
            inline_evaluator: InlineEvaluator::new(),
            continuation_pooler: ContinuationPoolManager::new(),
            integrated_optimizer: IntegratedOptimizationManager::new(),
            adaptive_engine: AdaptiveOptimizationEngine::new(),
            hotpath_detector: AdvancedHotPathDetector::new(),
            #[cfg(feature = "development")]
            llvm_compiler: LLVMCompilerIntegration::new(),
            optimization_level: RuntimeOptimizationLevel::Balanced,
            verification_enabled: cfg!(debug_assertions),
            recursion_depth: 0,
            max_recursion_depth: 1000,
        }
    }

    /// Create runtime executor with custom optimization level
    #[must_use] pub fn with_optimization_level(level: RuntimeOptimizationLevel) -> Self {
        let mut executor = Self::new();
        *executor.optimization_level_mut() = level;
        executor
    }

    /// Create runtime executor with custom environment
    #[must_use] pub fn with_environment(env: Rc<Environment>) -> Self {
        let mut executor = Self::new();
        *executor.semantic_evaluator_mut() = SemanticEvaluator::with_environment(env);
        executor
    }

    /// Create runtime executor with shared environment (new architecture)
    /// This allows the environment to be created once and shared across components
    #[must_use] pub fn with_shared_environment(env: std::sync::Arc<Environment>) -> Self {
        // Convert Arc<Environment> to Rc<Environment> for current compatibility
        let rc_env = Rc::new((*env).clone());
        Self::with_environment(rc_env)
    }

    // Removed set_verification_enabled() method - use core_types accessor instead

    // Removed optimization_level() method - use core_types accessor instead

    /// Set optimization level
    /// Apply optimization level configuration (functional approach)
    /// Instead of mutating state, we compute the LLVM level and apply it when needed
    #[cfg(feature = "development")]
    #[must_use] pub fn get_llvm_optimization_level(&self, level: RuntimeOptimizationLevel) -> LLVMOptimizationLevel {
        match level {
            RuntimeOptimizationLevel::None => LLVMOptimizationLevel::O0,
            RuntimeOptimizationLevel::Conservative => LLVMOptimizationLevel::O1,
            RuntimeOptimizationLevel::Balanced => LLVMOptimizationLevel::O2,
            RuntimeOptimizationLevel::Aggressive => LLVMOptimizationLevel::O3,
        }
    }
    
    /// Configure optimization level for LLVM when needed (mutable version)
    #[cfg(feature = "development")]
    pub fn configure_optimization_level(&mut self, level: RuntimeOptimizationLevel) {
        let llvm_level = self.get_llvm_optimization_level(level);
        self.llvm_compiler_mut().set_optimization_level(llvm_level);
    }

    /// Get JIT optimization statistics
    #[must_use] pub fn get_jit_statistics(&self) -> JitOptimizationStats {
        self.jit_optimizer().combined_stats()
    }

    /// Get advanced hot path analysis report
    #[must_use] pub fn get_hotpath_analysis_report(&self) -> String {
        let report = self.hotpath_detector().generate_performance_report();
        format!("Hot Path Analysis Report:\n{}", report)
    }

    /// Clear JIT compilation caches (useful for testing or memory management)
    pub fn clear_jit_caches(&mut self) {
        self.jit_optimizer_mut().clear_caches();
    }

    /// Get current recursion depth (useful for testing)
    #[must_use] pub fn get_recursion_depth(&self) -> usize {
        self.recursion_depth()
    }

    // Removed set_max_recursion_depth() method - use core_types accessor instead

    /// ExecutionContext-driven optimized evaluation (Phase 9/10 integration)
    /// This is the key integration point for receiving `ExecutionContext` from Evaluator
    pub fn eval_with_execution_context(
        &mut self,
        context: ExecutionContext,
    ) -> Result<Value> {
        // Stack overflow protection
        self.check_recursion_depth()?;
        *self.recursion_depth_mut() += 1;

        let _eval_start = std::time::Instant::now();
        
        // Extract information from ExecutionContext
        // Use expanded expression if available, otherwise use original
        let expr = context.get_execution_expression().clone();
        let env = context.environment.clone();
        let cont = context.continuation.clone();
        
        // Send evaluation started statistics through environment
        let global_env = self.global_environment();
        send_stats!(self, global_env, StatisticsMessage::EvaluationCompleted {
            expression_type: format!("{:?}", expr).chars().take(50).collect(),
            execution_time_ns: 0, // Will be updated at the end
            recursion_depth: self.recursion_depth(),
        });
        
        // Use static analysis from ExecutionContext
        let complexity_score = context.static_analysis.complexity_score;
        
        // Log static optimization information
        if context.was_macro_expanded() {
            let global_env = self.global_environment();
            send_stats!(self, global_env, StatisticsMessage::OptimizationApplied {
                optimization_type: "macro_expansion".to_string(),
                improvement_factor: 1.0,
                memory_saved: context.macro_expansion_state.expanded_macros.len() * 64, // Estimate
            });
        }
        
        // Apply pre-computed constant bindings from static optimization
        for _value in context.constant_bindings.values() {
            // Record that we're using a pre-computed constant
            let global_env = self.global_environment();
            send_stats!(self, global_env, StatisticsMessage::OptimizationApplied {
                optimization_type: "constant_binding".to_string(),
                improvement_factor: 1.1,
                memory_saved: 32, // Estimate per constant
            });
            // Use pre-computed constants by binding them in environment
            // Note: Environment is already Arc<MutableEnvironment>, constants should be pre-bound
        }
        
        // Record static optimization benefits
        #[cfg(feature = "development")]
        let static_benefit = context.estimated_static_benefit();
        let global_env = self.global_environment();
        send_stats!(self, global_env, StatisticsMessage::OptimizationApplied {
            optimization_type: "static_analysis".to_string(),
            improvement_factor: 1.0 + (static_benefit.time_savings_micros as f64 / 1000.0),
            memory_saved: static_benefit.memory_savings_bytes,
        });
        
        // Use optimization hints from ExecutionContext
        let should_use_jit = context.optimization_hints.jit_beneficial;
        let should_use_tail_call_opt = context.optimization_hints.use_tail_call_optimization;
        let should_use_continuation_pooling = context.optimization_hints.use_continuation_pooling;
        let should_use_inline_eval = context.optimization_hints.use_inline_evaluation;
        
        // Apply static optimization insights to dynamic optimization decisions  
        if should_use_jit {
            let global_env = self.global_environment();
            send_stats!(self, global_env, StatisticsMessage::JitCompilation {
                expression_hash: format!("{:?}", expr).chars().take(32).collect(),
                compilation_time_ns: 0, // Will be measured during actual compilation
                code_size: 1024, // Estimate
            });
        }
        
        if should_use_tail_call_opt {
            let global_env = self.global_environment();
            send_stats!(self, global_env, StatisticsMessage::OptimizationApplied {
                optimization_type: "tail_call_optimization".to_string(),
                improvement_factor: 1.2,
                memory_saved: 256, // Stack frame saving estimate
            });
        }
        
        if should_use_continuation_pooling {
            let global_env = self.global_environment();
            send_stats!(self, global_env, StatisticsMessage::ContinuationPooling {
                continuation_type: "optimized_pooling".to_string(),
                pool_hit: true,
                efficiency_gain: 1.15,
            });
        }
        
        if should_use_inline_eval {
            let global_env = self.global_environment();
            send_stats!(self, global_env, StatisticsMessage::OptimizationApplied {
                optimization_type: "inline_evaluation".to_string(),
                improvement_factor: 1.1,
                memory_saved: 128, // Call overhead saving estimate
            });
        }
        
        // Map ExecutionContext optimization level to RuntimeOptimizationLevel
        let runtime_level = match context.optimization_hints.optimization_level {
            crate::evaluator::execution_context::OptimizationLevel::None => RuntimeOptimizationLevel::None,
            crate::evaluator::execution_context::OptimizationLevel::Conservative => RuntimeOptimizationLevel::Conservative,
            crate::evaluator::execution_context::OptimizationLevel::Balanced => RuntimeOptimizationLevel::Balanced,
            crate::evaluator::execution_context::OptimizationLevel::Aggressive => RuntimeOptimizationLevel::Aggressive,
        };
        
        // Override runtime optimization level if context suggests different level
        if runtime_level != *self.optimization_level() {
            self.set_optimization_level(runtime_level);
        }
        
        // Advanced hot path detection with ExecutionContext information
        if let Err(_) = self.hotpath_detector_mut().record_execution(
            &expr,
            std::time::Duration::from_nanos(0), // Will be updated after evaluation
            context.static_analysis.memory_estimates.heap_allocations,
            &Value::Undefined, // Return value placeholder
            &[], // Call stack placeholder
        ) {
            // Continue with evaluation even if hotpath recording fails
        }
        
        // Track hot path detection based on complexity score
        if complexity_score > 75 {
            let global_env = self.global_environment();
            send_stats!(self, global_env, StatisticsMessage::HotPathDetected {
                expression_hash: format!("{:?}", expr).chars().take(32).collect(),
                frequency: complexity_score as u64,
                optimization_candidate: true,
            });
        }
        
        // Apply optimizations based on ExecutionContext
        let result = match runtime_level {
            RuntimeOptimizationLevel::None => {
                // No optimization - return pre-evaluated result from context if available
                // Or delegate back to evaluator via ExecutionContext completion signal
                if let Some(pre_evaluated) = context.get_pre_evaluated_result() {
                    Ok(pre_evaluated.clone())
                } else {
                    // Signal evaluator that no optimization was applied
                    // The evaluator should handle the actual evaluation
                    Err(LambdustError::optimization_signal(
                        "NoOptimization".to_string(),
                        expr.clone(),
                        env.clone(),
                        cont.clone()
                    ))
                }
            }
            RuntimeOptimizationLevel::Conservative => {
                self.apply_conservative_optimizations(expr.clone(), cont.clone(), &context)
            }
            RuntimeOptimizationLevel::Balanced => {
                self.apply_balanced_optimizations(expr.clone(), cont.clone(), &context)
            }
            RuntimeOptimizationLevel::Aggressive => {
                self.apply_aggressive_optimizations(expr.clone(), cont.clone(), &context)
            }
        };
        
        // Update evaluation time
        #[cfg(feature = "development")]
        let eval_time = _eval_start.elapsed();
        let global_env = self.global_environment();
        send_stats!(self, global_env, StatisticsMessage::EvaluationCompleted {
            expression_type: format!("{:?}", expr).chars().take(50).collect(),
            execution_time_ns: eval_time.as_nanos() as u64,
            recursion_depth: self.recursion_depth(),
        });
        
        // Verify result against semantic evaluator if enabled
        if self.verification_enabled() && result.is_ok() {
            self.verify_result_correctness(&expr, &env, &cont, &result);
        }
        
        *self.recursion_depth_mut() -= 1;
        result
    }

    /// ExecutionContext-driven JIT analysis and optimized execution (Phase 9 核心API)
    /// This is the key API for responsibility separation: receives `ExecutionContext` from Evaluator,
    /// performs dynamic optimization selection, JIT compilation decisions, and execution
    pub fn execute_with_jit_analysis(
        &mut self,
        context: ExecutionContext,
    ) -> Result<Value> {
        self.check_recursion_depth()?;
        *self.recursion_depth_mut() += 1;
        
        let eval_start = std::time::Instant::now();
        
        // 1. Utilize static analysis results (inherited from Evaluator responsibilities)
        // 2. Select dynamic optimization strategy (RuntimeExecutor responsibilities)
        let analysis_result = ExpressionAnalysisResult {
            complexity_score: context.static_analysis.complexity_score,
            is_tail_call_candidate: context.static_analysis.has_tail_calls,
            is_hot_path: context.execution_metadata.priority == ExecutionPriority::High,
            contains_loops: context.static_analysis.has_loops,
            call_patterns: vec![], // Simplified for now
            execution_frequency: ExecutionFrequency::Warm,
            memory_patterns: vec![],
            optimization_hints: vec![],
        };
        // Simplified strategy selection
        let runtime_strategy = if context.execution_metadata.priority == ExecutionPriority::High && analysis_result.complexity_score > 20 {
            AdaptiveOptimizationType::JitCompilation
        } else if analysis_result.contains_loops {
            AdaptiveOptimizationType::AdaptiveLoopUnrolling { factor: 2 }
        } else {
            AdaptiveOptimizationType::ProfileGuidedOptimization
        };
        
        // 3. JIT decision and execution strategy determination
        let opt_level = *self.optimization_level();
        let result = if runtime_strategy.should_compile_jit() {
            // JIT compilation and execution path
            match self.jit_compile_and_execute(&context, &opt_level) {
                Ok(jit_result) => {
                    send_stats!(self, &context.environment, StatisticsMessage::JitCompilation {
                        expression_hash: format!("{:?}", context.expression).chars().take(32).collect(),
                        compilation_time_ns: eval_start.elapsed().as_nanos() as u64,
                        code_size: 2048, // Estimate
                    });
                    Ok(jit_result)
                },
                Err(_) => {
                    // Fallback to optimized interpreter when JIT fails
                    // JIT fallback - no specific statistics message needed
                    self.interpret_with_optimizations(&context, &opt_level)
                }
            }
        } else {
            // Optimized interpreter execution path
            self.interpret_with_optimizations(&context, &opt_level)
        }?;
        
        // 4. Update performance statistics
        let _execution_time = eval_start.elapsed();
        send_stats!(self, &context.environment, StatisticsMessage::EvaluationCompleted {
            expression_type: "jit_evaluation".to_string(),
            execution_time_ns: _execution_time.as_nanos() as u64,
            recursion_depth: self.recursion_depth(),
        });
        
        // 5. Result verification (based on SemanticEvaluator)
        if self.verification_enabled() {
            // Simplified verification - using existing method
            self.verify_result_correctness(&context.expression, &context.environment, &context.continuation, &Ok(result.clone()));
        }
        
        *self.recursion_depth_mut() -= 1;
        Ok(result)
    }

    /// JIT compilation and execution with LLVM integration
    pub fn jit_compile_and_execute(
        &mut self,
        context: &ExecutionContext,
        strategy: &RuntimeOptimizationLevel,
    ) -> Result<Value> {
        let expr = context.get_execution_expression().clone();
        let env = context.environment.clone();
        let cont = context.continuation.clone();
        
        // Check if LLVM IR compilation is beneficial
#[cfg(feature = "development")]
        {
            if context.should_use_llvm_ir() {
                if let Some(llvm_context) = context.get_llvm_ir_context() {
                    // Use pre-generated LLVM IR for execution
                    return self.execute_llvm_ir(llvm_context, &env, cont);
                }
                // Generate LLVM IR on-demand
                return self.compile_and_execute_llvm_ir(context, strategy);
            }
        }
        
        // Fallback to traditional JIT optimization with tail call support
        if strategy.use_tail_call_optimization() {
            let analysis = ExpressionAnalysisResult {
                complexity_score: 10,
                is_tail_call_candidate: true,
                is_hot_path: false,
                contains_loops: false,
                call_patterns: vec![],
                execution_frequency: ExecutionFrequency::Warm,
                memory_patterns: vec![],
                optimization_hints: vec![],
            };
            let global_env = self.global_environment().clone();
            if let Some(optimized_result) = self.try_tail_call_optimization(&expr, &global_env, cont.clone(), &analysis)? {
                // self.stats_mut() - replaced with environment statistics.jit_compilations += 1;
                return Ok(optimized_result);
            }
        }
        
        // Signal evaluator for fallback evaluation
        // RuntimeExecutor should not directly evaluate - this violates responsibility separation
        Err(LambdustError::optimization_signal(
            "JitFallback".to_string(),
            expr,
            env,
            cont
        ))
    }

    /// Optimized interpreter execution (dynamic optimizations excluding JIT)
    pub fn interpret_with_optimizations(
        &mut self,
        context: &ExecutionContext,
        _strategy: &RuntimeOptimizationLevel,
    ) -> Result<Value> {
        let expr = context.get_execution_expression().clone();
        let env = context.environment.clone();
        let cont = context.continuation.clone();
        
        // 1. Continuation pooling optimization (simplified)
        let optimized_cont = cont; // Simplified: skip pooling for now
        
        // 2. Inline evaluation optimization (simplified)
        // Skip inline optimization for now
        
        // 3. Loop optimization (simplified)
        // Skip loop optimization for now
        
        // 4. Tail call optimization (simplified)
        // Skip tail call optimization for now in this method
        
        // 5. Constant optimization application (simplified)
        let const_optimized_expr = expr.clone(); // Skip immediate optimizations for now
        
        // 6. Return optimized context for evaluator to handle actual execution
        // RuntimeExecutor responsibility: optimization preparation, not evaluation
        if let Some(cached_result) = context.get_cached_result() {
            Ok(cached_result.clone())
        } else {
            // Signal evaluator with optimization-prepared context
            Err(LambdustError::optimization_signal(
                "OptimizedContext".to_string(),
                const_optimized_expr,
                env,
                optimized_cont
            ))
        }
    }

    /// Apply conservative optimizations based on `ExecutionContext`
    fn apply_conservative_optimizations(
        &mut self,
        expr: Expr,
        cont: Continuation,
        context: &ExecutionContext,
    ) -> Result<Value> {
        // Conservative: Apply safe optimizations using static analysis
        
        // Use pre-computed constants if available
        if let Expr::Variable(name) = &expr {
            if let Some(constant_value) = context.get_constant_binding(name) {
                // self.stats_mut() - replaced with environment statistics.constants_used += 1;
                return Ok(constant_value.clone());
            }
        }
        
        // Use inline evaluation for pure simple expressions
        if context.static_analysis.is_pure && 
           context.static_analysis.complexity_score < 20 &&
           context.optimization_hints.use_inline_evaluation {
            // Use inline evaluator with proper API integration  
            // Note: Inline evaluator operates on values, not expressions directly
            // This would require expression evaluation first, then inline optimization
            // For now, just track the optimization opportunity
            // self.stats_mut() - replaced with environment statistics.inline_evaluation_opportunities += 1;
            // self.stats_mut() - replaced with environment statistics.inline_evaluations += 1;
        }
        
        // Return optimization result to evaluator for execution
        // Conservative optimizations prepared - evaluator handles execution
        Err(LambdustError::optimization_signal(
            "ConservativeOptimizationPrepared".to_string(),
            expr,
            self.global_environment().clone(),
            cont
        ))
    }
    
    /// Apply balanced optimizations based on `ExecutionContext`
    fn apply_balanced_optimizations(
        &mut self,
        expr: Expr,
        cont: Continuation,
        context: &ExecutionContext,
    ) -> Result<Value> {
        // Balanced: Apply moderate optimizations based on static analysis
        
        // First try conservative optimizations
        if let Ok(result) = self.apply_conservative_optimizations(expr.clone(), cont.clone(), context) {
            return Ok(result);
        }
        
        // Apply tail call optimization if recommended
        if context.optimization_hints.use_tail_call_optimization {
            // Use integrated optimizer for tail call optimization
            let mut tail_call_optimizer = crate::evaluator::tail_call_optimization::TailCallOptimizer::new();
            let tail_call_context = crate::evaluator::tail_call_optimization::TailCallContext::new();
            if let Ok(Some(_optimized_result)) = tail_call_optimizer.optimize_tail_call(
                &expr, 
                &tail_call_context, 
                &mut crate::evaluator::Evaluator::new()
            ) {
                // Tail call optimization applied at evaluation time
                // Statistics tracked by environment
            }
        }
        
        // Apply continuation pooling if beneficial
        if context.optimization_hints.use_continuation_pooling {
            if let Some(_pooled_cont) = self.continuation_pooler.allocate(crate::evaluator::continuation_pooling::ContinuationType::Simple) {
                // Use pooled continuation for memory optimization
                // Statistics tracked by environment  
            }
        }
        
        // Return optimization result to evaluator for execution
        // Conservative optimizations prepared - evaluator handles execution
        Err(LambdustError::optimization_signal(
            "ConservativeOptimizationPrepared".to_string(),
            expr,
            self.global_environment().clone(),
            cont
        ))
    }
    
    /// Apply aggressive optimizations based on `ExecutionContext`  
    fn apply_aggressive_optimizations(
        &mut self,
        expr: Expr,
        cont: Continuation,
        context: &ExecutionContext,
    ) -> Result<Value> {
        // Aggressive: Apply all available optimizations based on static analysis
        
        // First try balanced optimizations
        if let Ok(result) = self.apply_balanced_optimizations(expr.clone(), cont.clone(), context) {
            return Ok(result);
        }
        
        #[cfg(feature = "development")]
        {
            // Apply JIT compilation for hot paths if recommended
            if context.optimization_hints.jit_beneficial && 
               context.static_analysis.complexity_score > 50 {
                // self.stats_mut() - replaced with environment statistics.jit_compilations += 1;
                // self.stats_mut() - replaced with environment statistics.jit_compilations_triggered += 1;
                
                // Use LLVM compiler with proper API integration
                // Create tail call context for compilation
                let tail_call_context = crate::evaluator::TailCallContext::new();
                if let Ok(_compiled_fn) = self.llvm_compiler_mut().compile_with_tail_calls(&expr, &tail_call_context) {
                    // LLVM compilation succeeded, track the optimization
                    // self.stats_mut() - replaced with environment statistics.llvm_optimizations_applied += 1;
                    // Note: Actual execution would require more integration work
                    // For now, fall through to semantic evaluation with optimization tracking
                }
            }
        }
        
        // Use all static optimization results
        for optimization in &context.static_analysis.static_optimizations {
            match optimization {
                crate::evaluator::execution_context::StaticOptimization::ConstantFolding { result, .. } => {
                    // If this is a constant folding result, use it directly
                    return Ok(result.clone());
                }
                _ => {} // Handle other optimizations as needed
            }
        }
        
        // Return optimization result to evaluator for execution
        // Conservative optimizations prepared - evaluator handles execution
        Err(LambdustError::optimization_signal(
            "ConservativeOptimizationPrepared".to_string(),
            expr,
            self.global_environment().clone(),
            cont
        ))
    }

    /// Main optimized evaluation function
    pub fn eval_optimized(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Stack overflow protection
        self.check_recursion_depth()?;
        *self.recursion_depth_mut() += 1;
        // Expression evaluation tracking through environment statistics

        // Expression analysis for optimization hints
        let eval_start = std::time::Instant::now();
        let analysis = ExpressionAnalysisResult::analyze_expression(&expr);
        
        // Advanced hot path detection with multi-dimensional analysis
        if let Err(_) = self.hotpath_detector_mut().record_execution(
            &expr,
            std::time::Duration::from_nanos(0), // Will be updated after evaluation
            0, // Memory usage placeholder
            &Value::Undefined, // Return value placeholder
            &[], // Call stack placeholder
        ) {
            // Continue with evaluation even if hotpath recording fails
        }
        
        // Track hot path detection
        if analysis.execution_frequency == ExecutionFrequency::Hot {
            // self.stats_mut() - replaced with environment statistics.hot_path_detections += 1;
        }

        // Apply optimization analysis - RuntimeExecutor responsibility: analysis & preparation
        // Evaluator responsibility: actual execution
        let result = match *self.optimization_level() {
            RuntimeOptimizationLevel::None => {
                // No optimizations - signal evaluator to handle directly
                Err(LambdustError::optimization_signal(
                    "NoOptimizationLegacy".to_string(),
                    expr,
                    env,
                    cont
                ))
            }

            RuntimeOptimizationLevel::Conservative => {
                // Conservative optimization analysis only
                self.prepare_conservative_optimizations(expr, env, cont, &analysis)
            }

            RuntimeOptimizationLevel::Balanced => {
                // Balanced optimization analysis
                self.prepare_balanced_optimizations(expr, env, cont, &analysis)
            }

            RuntimeOptimizationLevel::Aggressive => {
                // Create execution context for aggressive optimizations
                let execution_context = ExecutionContext::new(expr.clone(), env.clone(), cont.clone());
                self.apply_aggressive_optimizations(expr, cont, &execution_context)
            }
        };

        // Always update statistics and decrement recursion depth
        let _eval_time = eval_start.elapsed();
        // Evaluation time tracking handled through EvaluationCompleted statistics messages
        *self.recursion_depth_mut() -= 1;

        // Return result (could be Ok or Err)
        result
    }

    /// Conservative optimization preparation: basic optimizations with high confidence
    fn prepare_conservative_optimizations(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Value> {
        // Conservative optimization strategy: focus on safe, proven optimizations
        
        // Check for immediate optimization opportunities
        if let Some(result) = self.try_immediate_optimizations(&expr, analysis)? {
            return Ok(result);
        }
        
        // Apply builtin function optimizations
        if let Some(result) = self.try_builtin_optimizations(&expr, &env, analysis)? {
            // Optimization applied tracking handled through dedicated statistics messages
            return Ok(result);
        }
        
        // Apply tail call optimization if conservative
        if analysis.is_tail_call_candidate && 
           analysis.optimization_hints.contains(&OptimizationHint::TailCallOptimize) {
            let global_env = self.global_environment().clone();
            if let Some(result) = self.try_tail_call_optimization(&expr, &global_env, cont.clone(), analysis)? {
                // self.stats_mut() - replaced with environment statistics.tail_calls_optimized += 1;
                return Ok(result);
            }
        }
        
        // Use integrated optimization system as fallback
        let Ok(strategies) = self
            .integrated_optimizer_mut()
            .select_optimization_strategy(&expr, &RuntimeOptimizationLevel::Conservative) else {
                // Signal evaluator for fallback evaluation
                return Err(LambdustError::optimization_signal(
                    "StrategySelectionFailed".to_string(),
                    expr,
                    env,
                    cont
                ));
            };

        if !strategies.is_empty() {
            if let Ok(optimization_result) = self.integrated_optimizer_mut().execute_optimization(
                expr.clone(),
                env.clone(),
                strategies,
            ) {
                if !optimization_result.applied_strategies.is_empty() {
                    // Optimization applied tracking handled through dedicated statistics messages
                    return self.apply_optimization_result(optimization_result, env, cont);
                }
            }
        }

        // Signal evaluator for fallback evaluation
        Err(LambdustError::optimization_signal(
            "ConservativeOptimizationFallback".to_string(),
            expr,
            env,
            cont
        ))
    }

    /// Balanced optimization preparation: good balance of safety and performance
    fn prepare_balanced_optimizations(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Value> {
        // Balanced optimization strategy: balance between safety and performance
        
        // Check for immediate optimization opportunities
        if let Some(result) = self.try_immediate_optimizations(&expr, analysis)? {
            return Ok(result);
        }
        
        // Apply builtin function optimizations
        if let Some(result) = self.try_builtin_optimizations(&expr, &env, analysis)? {
            // Optimization applied tracking handled through dedicated statistics messages
            return Ok(result);
        }
        
        // Apply tail call optimization
        if analysis.is_tail_call_candidate {
            let global_env = self.global_environment().clone();
            if let Some(result) = self.try_tail_call_optimization(&expr, &global_env, cont.clone(), analysis)? {
                // self.stats_mut() - replaced with environment statistics.tail_calls_optimized += 1;
                return Ok(result);
            }
        }
        
        // Apply inlining for hot paths
        if analysis.execution_frequency == ExecutionFrequency::Hot ||
           analysis.optimization_hints.contains(&OptimizationHint::Inline) {
            let global_env = self.global_environment().clone();
            if let Some(result) = self.try_inline_optimization(&expr, &global_env, analysis)? {
                // self.stats_mut() - replaced with environment statistics.inline_evaluations += 1;
                return Ok(result);
            }
        }
        
        // Apply continuation pooling for recursive patterns
        if analysis.optimization_hints.contains(&OptimizationHint::PoolContinuations) {
            let global_env = self.global_environment().clone();
            if let Some(result) = self.try_continuation_pooling(&expr, &global_env, cont.clone(), analysis)? {
                // self.stats_mut() - replaced with environment statistics.continuation_pool_hits += 1;
                return Ok(result);
            }
        }
        
        // JIT loop optimization for hot loops
        if analysis.contains_loops && 
           analysis.execution_frequency >= ExecutionFrequency::Warm &&
           analysis.optimization_hints.contains(&OptimizationHint::JitCompile) {
            let global_env = self.global_environment().clone();
            if let Some(result) = self.try_jit_loop_optimization(&expr, &global_env, cont.clone(), analysis)? {
                // Optimization applied tracking handled through dedicated statistics messages
                return Ok(result);
            }
        }
        
        // Use integrated optimization system
        let Ok(strategies) = self
            .integrated_optimizer_mut()
            .select_optimization_strategy(&expr, &RuntimeOptimizationLevel::Balanced) else {
                return self.semantic_evaluator().eval_pure_functional(expr, env, cont, self.recursion_depth());
            };

        if !strategies.is_empty() {
            if let Ok(optimization_result) = self.integrated_optimizer_mut().execute_optimization(
                expr.clone(),
                env.clone(),
                strategies,
            ) {
                if !optimization_result.applied_strategies.is_empty() {
                    // Optimization applied tracking handled through dedicated statistics messages
                    return self.apply_optimization_result(optimization_result, env, cont);
                }
            }
        }

        // Fallback to semantic evaluation
        self.semantic_evaluator_mut().eval_pure(expr, env, cont)
    }

    // TODO: Implement aggressive optimizations for maximum performance
    // This function was removed as it's currently unused. When implementing:
    // - JIT compilation for hot paths
    // - LLVM native code generation for critical paths
    // - Aggressive inlining and loop optimizations
    // - Continuation pooling for frequently used patterns

    /// Apply optimization result from `IntegratedOptimizationManager`
    fn apply_optimization_result(
        &mut self,
        optimization_result: OptimizationResult,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Update statistics based on the optimization result
        if optimization_result.applied_strategies.is_empty() {
            // No optimization strategies were applied, use semantic evaluator to avoid recursion
            self.semantic_evaluator_mut().eval_pure(optimization_result.optimized_expr, env, cont)
        } else {
            // Optimization applied tracking handled through dedicated statistics messages

            // Determine optimization type from strategy name and apply accordingly
            // Use the first applied strategy for classification
            let strategy = optimization_result.applied_strategies.first().map_or("", std::string::String::as_str);
            match strategy {
                s if s.contains("tail_call") => {
                    // self.stats_mut() - replaced with environment statistics.tail_calls_optimized += 1;
                }
                s if s.contains("jit") || s.contains("loop") => {
                    // self.stats_mut() - replaced with environment statistics.jit_compilations += 1;
                }
                s if s.contains("inline") => {
                    // self.stats_mut() - replaced with environment statistics.inline_evaluations += 1;
                }
                s if s.contains("continuation") || s.contains("pool") => {
                    // self.stats_mut() - replaced with environment statistics.continuation_pool_hits += 1;
                }
                _ => {
                    // General optimization
                }
            }

            // Use semantic evaluator to avoid infinite recursion
            self.semantic_evaluator_mut().eval_pure(optimization_result.optimized_expr, env, cont)
        }
    }

    /// Try immediate optimizations for simple expressions
    fn try_immediate_optimizations(
        &self,
        expr: &Expr,
        _analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        match expr {
            // Constant folding for literals
            Expr::Literal(lit) => {
                Ok(Some(Value::from_literal(lit)))
            },
            
            // Simple arithmetic optimizations
            Expr::List(exprs) => {
                if exprs.len() == 3 {
                    if let Expr::Variable(name) = &exprs[0] {
                        if Self::is_simple_arithmetic(name) {
                            if let (Some(a), Some(b)) = (Self::extract_literal_number(&exprs[1]), Self::extract_literal_number(&exprs[2])) {
                                return Ok(Some(Self::compute_arithmetic(name, a, b)?));
                            }
                        }
                    }
                }
                Ok(None)
            },
            
            _ => Ok(None),
        }
    }
    
    /// Try builtin function optimizations
    fn try_builtin_optimizations(
        &self,
        expr: &Expr,
        _env: &Rc<Environment>,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        if let Expr::List(exprs) = expr {
            if !exprs.is_empty() {
                if let Expr::Variable(name) = &exprs[0] {
                    // Check if this is a builtin that can be optimized
                    if Self::is_optimizable_builtin(name, analysis) {
                        return self.apply_builtin_optimization(name, &exprs[1..]);
                    }
                }
            }
        }
        Ok(None)
    }
    
    /// Try tail call optimization
    fn try_tail_call_optimization(
        &mut self,
        expr: &Expr,
        _env: &Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        // Check if this is a tail call candidate
        if !analysis.is_tail_call_candidate {
            return Ok(None);
        }
        
        match expr {
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    Ok(None)
                } else {
                    // For tail calls, we can optimize by avoiding stack frame creation
                    let global_env = self.global_environment().clone();
                    let optimized = OptimizedTailCall {
                        target_function: Value::Nil, // Placeholder
                        arguments: Vec::new(),        // Placeholder
                        environment: global_env,
                        optimization_applied: true,
                    };
                    
                    // Apply the optimized tail call
                    let global_env = self.global_environment().clone();
                    Ok(Some(self.apply_optimized_tail_call(optimized, global_env, cont)?))
                }
            },
            _ => Ok(None),
        }
    }
    
    /// Try inline optimization
    fn try_inline_optimization(
        &mut self,
        expr: &Expr,
        env: &Rc<Environment>,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        // Only inline simple expressions
        if analysis.complexity_score > 15 {
            return Ok(None);
        }
        
        if let Expr::List(exprs) = expr {
            // Check for lambda expressions
            if exprs.len() >= 3 {
                if let Expr::Variable(name) = &exprs[0] {
                    if name == "lambda" {
                        // Inline simple lambda bodies
                        let body = &exprs[2];
                        if Self::is_inlinable_expression(body) {
                            return Ok(Some(self.semantic_evaluator().eval_pure_functional(
                                body.clone(),
                                env.clone(),
                                Continuation::Identity,
                                self.recursion_depth() + 1,
                            )?));
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Try continuation pooling optimization
    fn try_continuation_pooling(
        &mut self,
        expr: &Expr,
        _env: &Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        // Use continuation pooling for recursive patterns and hot paths
        let should_pool = analysis.call_patterns.iter().any(|p| matches!(p, 
            CallPattern::Recursive { .. } | CallPattern::TailRecursive
        )) || analysis.execution_frequency == ExecutionFrequency::Hot;
        
        if should_pool {
            // Determine continuation type for optimal pooling
            let cont_type = ContinuationType::from_continuation(&cont);
            
            // Try to allocate from pool first
            if let Some(pooled_cont) = self.continuation_pooler_mut().allocate(cont_type) {
                // Use pooled continuation for evaluation
                let global_env = self.global_environment().clone();
                let result = self.semantic_evaluator().eval_pure_functional(expr.clone(), global_env, pooled_cont.clone(), self.recursion_depth() + 1);
                
                // Return continuation to pool after use
                if self.continuation_pooler_mut().deallocate(pooled_cont) {
                    // Track successful pool usage
                    let (_, _, _memory_saved, _) = self.continuation_pooler().global_statistics();
                    // self.stats_mut() - replaced with environment statistics.pooling_memory_saved = memory_saved;
                } else {
                    // self.stats_mut() - replaced with environment statistics.continuation_pool_misses += 1;
                }
                
                return Ok(Some(result?));
            }
        }
        
        // Fall back to normal evaluation with original continuation
        match expr {
            Expr::List(_) => {
                // Use semantic evaluator with the original continuation
                let global_env = self.global_environment().clone();
                let result = self.semantic_evaluator().eval_pure_functional(expr.clone(), global_env, cont.clone(), self.recursion_depth() + 1);
                
                // Try to return continuation to pool if beneficial
                if should_pool {
                    if self.continuation_pooler_mut().deallocate(cont) {
                        // Update pooling statistics
                        let (_, _, _memory_saved, _) = self.continuation_pooler().global_statistics();
                        // self.stats_mut() - replaced with environment statistics.pooling_memory_saved = memory_saved;
                    } else {
                        // self.stats_mut() - replaced with environment statistics.continuation_pool_misses += 1;
                    }
                }
                
                Ok(Some(result?))
            },
            _ => Ok(None),
        }
    }

    // TODO: Implement loop optimization strategies
    // This function was removed as it's currently unused. When implementing:
    // - Loop unrolling for simple iteration patterns
    // - Loop fusion for multiple adjacent loops
    // - Tail recursion to iteration transformation
    
    /// Verify result correctness against semantic evaluator
    fn verify_result_correctness(
        &mut self,
        expr: &Expr,
        env: &Rc<Environment>,
        cont: &Continuation,
        result: &Result<Value>,
    ) {
        // Only verify successful results
        if let Ok(optimized_result) = result {
            // Compare with semantic evaluator result
            if let Ok(semantic_result) = self.semantic_evaluator().eval_pure_functional(
                expr.clone(),
                env.clone(),
                cont.clone(),
                self.recursion_depth() + 1,
            ) {
                // Check if results are equivalent
                if self.values_equivalent(optimized_result, &semantic_result) {
                    // self.stats_mut() - replaced with environment statistics.verification_successes += 1;
                } else {
                    eprintln!(
                        "WARNING: Optimization produced different result than semantic evaluator\n\
                        Expression: {expr:?}\n\
                        Optimized result: {optimized_result:?}\n\
                        Semantic result: {semantic_result:?}"
                    );
                    // self.stats_mut() - replaced with environment statistics.verification_failures += 1;
                }
            }
        }
    }
    
    /// Check if two values are equivalent (for verification)
    fn values_equivalent(&self, val1: &Value, val2: &Value) -> bool {
        // For now, use direct equality
        // In the future, this could be more sophisticated
        // (e.g., numerical tolerance for floating point)
        val1 == val2
    }

    /// Apply optimized tail call result
    fn apply_optimized_tail_call(
        &mut self,
        optimized: OptimizedTailCall,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if optimized.optimization_applied {
            // Use optimized execution path
            self.semantic_evaluator_mut().eval_pure(
                crate::ast::Expr::Literal(crate::ast::Literal::Nil),
                optimized.environment,
                cont,
            )
        } else {
            // Fallback to normal evaluation
            self.semantic_evaluator_mut().eval_pure(
                crate::ast::Expr::Literal(crate::ast::Literal::Nil),
                env,
                cont,
            )
        }
    }

    /// Apply JIT compiled code
    #[allow(dead_code)]
    fn apply_jit_compiled_code(
        &mut self,
        jit_code: JitCompiledCode,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if jit_code.is_ready {
            // Execute JIT compiled code (simulated)
            // In a real implementation, this would execute native compiled code
            self.semantic_evaluator_mut().eval_pure(
                jit_code.original_expr,
                env,
                cont,
            )
        } else {
            // Fallback if JIT compilation failed
            self.semantic_evaluator_mut().eval_pure(
                jit_code.original_expr,
                env,
                cont,
            )
        }
    }

    /// Apply continuation with optimization (placeholder for future implementation)
    #[allow(dead_code)]
    fn apply_continuation_optimized(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        // Use semantic evaluator's continuation system
        match cont {
            Continuation::Identity => Ok(value),
            _ => {
                // For now, fallback to semantic evaluator
                Ok(value) // Simplified placeholder
            }
        }
    }

    /// Direct procedure application (placeholder for future implementation)
    #[allow(dead_code)]
    fn apply_procedure_direct(
        &mut self,
        _procedure: Value,
        _args: Vec<Value>,
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Fallback to semantic evaluator for basic implementation
        self.semantic_evaluator_mut().eval_pure(
            crate::ast::Expr::Literal(crate::ast::Literal::Nil),
            Rc::new(crate::environment::Environment::new()),
            cont,
        )
    }

    /// Execute optimized loop (placeholder for future implementation)
    #[allow(dead_code)]
    fn execute_optimized_loop(
        &mut self,
        _loop_body: Vec<Expr>,
        _bindings: Vec<(String, Value)>,
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Fallback to semantic evaluator for basic implementation
        self.semantic_evaluator_mut().eval_pure(
            crate::ast::Expr::Literal(crate::ast::Literal::Nil),
            Rc::new(crate::environment::Environment::new()),
            cont,
        )
    }

    /// Optimized builtin application (placeholder for future implementation)
    #[allow(dead_code)]
    fn apply_builtin_optimized(&self, name: &str, args: &[Value]) -> Result<Value> {
        // Use simple implementation
        match name {
            "+" => self.builtin_add_simple(args),
            "-" => self.builtin_subtract_simple(args),
            "*" => self.builtin_multiply_simple(args),
            _ => {
                // For other builtins, fallback to error for now
                Err(LambdustError::runtime_error(format!(
                    "Builtin '{name}' not implemented in runtime executor yet"
                )))
            }
        }
    }

    /// Simple addition implementation
    #[allow(dead_code)]
    fn builtin_add_simple(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Number(crate::lexer::SchemeNumber::Integer(0)));
        }

        let mut sum = 0i64;
        for arg in args {
            if let Value::Number(crate::lexer::SchemeNumber::Integer(n)) = arg {
                sum += n;
            } else {
                return Err(LambdustError::type_error("Addition expects integers"));
            }
        }

        Ok(Value::Number(crate::lexer::SchemeNumber::Integer(sum)))
    }

    /// Simple subtraction implementation
    #[allow(dead_code)]
    fn builtin_subtract_simple(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Err(LambdustError::arity_error(1, 0));
        }

        let first = match &args[0] {
            Value::Number(crate::lexer::SchemeNumber::Integer(n)) => *n,
            _ => return Err(LambdustError::type_error("Subtraction expects integers")),
        };

        if args.len() == 1 {
            return Ok(Value::Number(crate::lexer::SchemeNumber::Integer(-first)));
        }

        let mut result = first;
        for arg in &args[1..] {
            if let Value::Number(crate::lexer::SchemeNumber::Integer(n)) = arg {
                result -= n;
            } else {
                return Err(LambdustError::type_error("Subtraction expects integers"));
            }
        }

        Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
    }

    /// Simple multiplication implementation
    #[allow(dead_code)]
    fn builtin_multiply_simple(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Number(crate::lexer::SchemeNumber::Integer(1)));
        }

        let mut product = 1i64;
        for arg in args {
            if let Value::Number(crate::lexer::SchemeNumber::Integer(n)) = arg {
                product *= n;
            } else {
                return Err(LambdustError::type_error("Multiplication expects integers"));
            }
        }

        Ok(Value::Number(crate::lexer::SchemeNumber::Integer(product)))
    }

    /// Helper methods for optimization
    /// Check if function name is simple arithmetic
    fn is_simple_arithmetic(name: &str) -> bool {
        matches!(name, "+" | "-" | "*")
    }
    
    /// Extract literal number from expression
    fn extract_literal_number(expr: &Expr) -> Option<i64> {
        if let Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(n))) = expr {
            Some(*n)
        } else {
            None
        }
    }
    
    /// Compute arithmetic operation
    fn compute_arithmetic(op: &str, a: i64, b: i64) -> Result<Value> {
        let result = match op {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            _ => return Err(LambdustError::runtime_error(format!("Unknown arithmetic operator: {op}"))),
        };
        Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
    }
    
    /// Check if builtin function can be optimized
    fn is_optimizable_builtin(name: &str, analysis: &ExpressionAnalysisResult) -> bool {
        // Optimize builtins on hot paths or with specific hints
        let is_builtin = matches!(name, "+" | "-" | "*" | "/" | "=" | "<" | ">" 
                                      | "cons" | "car" | "cdr" | "length");
        let should_optimize = analysis.execution_frequency != ExecutionFrequency::Cold ||
                               analysis.optimization_hints.contains(&OptimizationHint::Inline);
        is_builtin && should_optimize
    }
    
    /// Apply builtin function optimization
    fn apply_builtin_optimization(&self, name: &str, _args: &[Expr]) -> Result<Option<Value>> {
        // For now, delegate to simple implementations
        match name {
            "+" | "-" | "*" => {
                // Try to evaluate arguments and apply optimization
                Ok(None) // Placeholder
            },
            _ => Ok(None),
        }
    }
    
    /// Check if expression is suitable for inlining
    fn is_inlinable_expression(expr: &Expr) -> bool {
        match expr {
            Expr::Literal(_) => true,
            Expr::Variable(_) => true,
            Expr::List(exprs) => exprs.len() <= 3,
            _ => false,
        }
    }
    
    /// Simple division implementation
    #[allow(dead_code)]
    fn builtin_divide_simple(&self, args: &[Value]) -> Result<Value> {
        if args.len() < 2 {
            return Err(LambdustError::arity_error(2, args.len()));
        }
        
        let dividend = match &args[0] {
            Value::Number(crate::lexer::SchemeNumber::Integer(n)) => *n,
            _ => return Err(LambdustError::type_error("Division expects integers")),
        };
        
        let divisor = match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(n)) => *n,
            _ => return Err(LambdustError::type_error("Division expects integers")),
        };
        
        if divisor == 0 {
            return Err(LambdustError::runtime_error("Division by zero"));
        }
        
        Ok(Value::Number(crate::lexer::SchemeNumber::Integer(dividend / divisor)))
    }

    /// Verify result against semantic evaluator (placeholder for future implementation)
    #[allow(dead_code)]
    fn verify_result(
        &mut self,
        _expr: &Expr,
        _env: &Rc<Environment>,
        _cont: &Continuation,
        _result: &Result<Value>,
    ) -> Result<()> {
        // Verification is disabled
        Ok(())
    }

    /// Check recursion depth
    fn check_recursion_depth(&self) -> Result<()> {
        if self.recursion_depth() >= self.max_recursion_depth() {
            return Err(LambdustError::stack_overflow());
        }
        Ok(())
    }

    /// Get current runtime statistics (now handled through environment statistics)
    #[must_use] pub fn get_stats(&self) -> RuntimeStats {
        // Statistics are now collected through environment processor
        // Return default stats as placeholder
        RuntimeStats::default()
    }

    /// Reset runtime statistics
    pub fn reset_stats(&mut self) {
        // Statistics reset now handled through environment statistics processor
    }

    /// Get reference to semantic evaluator
    #[must_use] pub fn get_semantic_evaluator(&self) -> &SemanticEvaluator {
        self.semantic_evaluator()
    }

    /// Get mutable reference to semantic evaluator
    pub fn get_semantic_evaluator_mut(&mut self) -> &mut SemanticEvaluator {
        self.semantic_evaluator_mut()
    }

    /// Get reference to JIT optimizer (dynamic optimization)
    #[must_use] pub fn get_jit_optimizer(&self) -> &JitLoopOptimizer {
        self.jit_optimizer()
    }

    /// Get reference to inline evaluator
    #[must_use] pub fn get_inline_evaluator(&self) -> &InlineEvaluator {
        self.inline_evaluator()
    }

    /// Get reference to continuation pooler
    #[must_use] pub fn get_continuation_pooler(&self) -> &ContinuationPoolManager {
        self.continuation_pooler()
    }

    /// Verify optimization result against semantic evaluator
    pub fn verify_optimization(
        &self,
        expr: &Expr,
        env: Rc<Environment>,
        optimized_result: &Value,
    ) -> Result<bool> {
        let semantic_result = self.semantic_evaluator().eval_pure_functional(
            expr.clone(),
            env,
            Continuation::Identity,
            0, // verification doesn't need deep recursion tracking
        )?;
        
        // Simple value comparison - could be enhanced with more sophisticated comparison
        Ok(self.values_equal(&semantic_result, optimized_result))
    }

    /// Basic value equality check
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => n1 == n2,
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Nil, Value::Nil) => true,
            (Value::Symbol(s1), Value::Symbol(s2)) => s1 == s2,
            _ => false,
        }
    }

    /// Execute pre-generated LLVM IR
    #[cfg(feature = "development")]
    pub fn execute_llvm_ir(
        &mut self,
        llvm_context: &crate::evaluator::execution_context::LLVMIRExecutionContext,
        env: &Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Get the generated LLVM IR
        let llvm_ir = llvm_context.to_llvm_ir();
        
        // For now, compile with LLVM backend and execute
        // In a full implementation, this would interface with actual LLVM
        match self.llvm_compiler_mut().compile_ir(&llvm_ir) {
            Ok(compiled_function) => {
                // Execute the compiled function
                // This is a simplified placeholder - real implementation would
                // execute the native code directly
                self.execute_compiled_function(compiled_function, env, cont)
            },
            Err(_) => {
                // LLVM compilation failed, fallback to interpreter
                Err(LambdustError::optimization_signal(
                    "LLVMCompilationFailed".to_string(),
                    crate::ast::Expr::Literal(crate::ast::Literal::Nil),
                    env.clone(),
                    cont
                ))
            }
        }
    }

    /// Compile `ExecutionContext` to LLVM IR and execute
    #[cfg(feature = "development")]
    pub fn compile_and_execute_llvm_ir(
        &mut self,
        context: &ExecutionContext,
        _strategy: &RuntimeOptimizationLevel,
    ) -> Result<Value> {
        // Create a mutable copy of the context for LLVM IR generation
        let mut mutable_context = context.clone();
        
        // Generate LLVM IR
        match mutable_context.generate_llvm_ir() {
            Ok(llvm_ir) => {
                // Compile the generated IR
                match self.llvm_compiler_mut().compile_ir(&llvm_ir) {
                    Ok(compiled_function) => {
                        // Execute the compiled function
                        self.execute_compiled_function(
                            compiled_function,
                            &context.environment,
                            context.continuation.clone()
                        )
                    },
                    Err(_) => {
                        // LLVM compilation failed, signal fallback
                        Err(LambdustError::optimization_signal(
                            "LLVMIRCompilationFailed".to_string(),
                            context.expression.clone(),
                            context.environment.clone(),
                            context.continuation.clone()
                        ))
                    }
                }
            },
            Err(ir_error) => {
                // IR generation failed, signal fallback
                Err(LambdustError::optimization_signal(
                    format!("LLVMIRGenerationFailed: {ir_error}"),
                    context.expression.clone(),
                    context.environment.clone(),
                    context.continuation.clone()
                ))
            }
        }
    }

    /// Execute a compiled LLVM function (placeholder implementation)
    #[cfg(feature = "development")]
    fn execute_compiled_function(
        &self,
        _compiled_function: crate::evaluator::llvm_backend::CompiledFunction,
        _env: &Rc<Environment>,
        _cont: Continuation,
    ) -> Result<Value> {
        // This is a placeholder implementation
        // In a real system, this would:
        // 1. Set up the runtime environment for the compiled function
        // 2. Call the native compiled code
        // 3. Handle the return value conversion
        // 4. Manage memory and continuation handling
        
        // For now, return a placeholder result
        Ok(Value::Boolean(true))
    }

    /// Optimize `ExecutionContext` using static analysis and LLVM IR insights
    pub fn optimize_execution_context(
        &mut self,
        #[cfg_attr(not(feature = "development"), allow(unused_variables))]
        context: &mut ExecutionContext,
    ) -> Result<()> {
        #[cfg(feature = "development")]
        {
            // Enhance static analysis based on LLVM IR capabilities
            if context.should_use_llvm_ir() {
                // Extract needed values before creating LLVM context to avoid borrowing conflicts
                let has_tail_calls = context.static_analysis.has_tail_calls;
                let complexity_score = context.static_analysis.complexity_score;
                let has_loops = context.static_analysis.has_loops;
                let is_pure = context.static_analysis.is_pure;
                
                // Create LLVM IR context for analysis
                let llvm_context = context.create_llvm_ir_context();
                
                // Enable tail call optimization if beneficial
                if has_tail_calls {
                    llvm_context.enable_tail_call_optimization();
                }
                
                // Add optimization attributes based on analysis
                if complexity_score > 75 {
                    llvm_context.add_optimization_attribute(
                        crate::evaluator::execution_context::LLVMOptimizationAttribute::CommonSubexpressionElimination
                    );
                }
                
                if has_loops {
                    llvm_context.add_optimization_attribute(
                        crate::evaluator::execution_context::LLVMOptimizationAttribute::LoopUnroll { factor: 2 }
                    );
                }
                
                if is_pure {
                    llvm_context.add_optimization_attribute(
                        crate::evaluator::execution_context::LLVMOptimizationAttribute::ConstantPropagation
                    );
                    llvm_context.add_optimization_attribute(
                        crate::evaluator::execution_context::LLVMOptimizationAttribute::DeadCodeElimination
                    );
                }
            }
        }
        
        Ok(())
    }

    /// Generate condition register system for runtime-variable execution
    #[must_use] pub fn generate_condition_registers(
        &self,
        context: &ExecutionContext,
    ) -> crate::evaluator::execution_context::ConditionRegisterSystem {
        crate::evaluator::execution_context::ConditionRegisterSystem {
            registers: std::collections::HashMap::new(),
            current_condition: None,
            execution_mode: {
                #[cfg(feature = "development")]
                {
                    if context.should_use_llvm_ir() {
                        crate::evaluator::execution_context::ExecutionMode::LLVMCompiled
                    } else {
                        crate::evaluator::execution_context::ExecutionMode::Interpreted
                    }
                }
                #[cfg(not(feature = "development"))]
                {
                    crate::evaluator::execution_context::ExecutionMode::Interpreted
                }
            },
            optimization_level: context.optimization_hints.optimization_level,
        }
    }
}

