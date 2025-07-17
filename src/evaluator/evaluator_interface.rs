//! Unified evaluator interface for transparent semantic/runtime evaluation switching
//!
//! This module provides a unified interface that allows transparent switching
//! between semantic evaluation (`SemanticEvaluator`) and optimized runtime execution
//! (`RuntimeExecutor`), with automatic correctness verification.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{
    Continuation, CorrectnessProof, CorrectnessProperty, EvaluationContext, EvaluationModeSelector,
    PerformanceRequirements, SelectionCriteria,
    SemanticCorrectnessProver, SemanticEvaluator, ExecutionContext, Evaluator,
};
use crate::executor::runtime::RuntimeExecutor;
use crate::executor::RuntimeOptimizationLevel;
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

/// Evaluation mode selection
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluationMode {
    /// Pure semantic evaluation (reference implementation)
    Semantic,
    /// Optimized runtime execution
    Runtime(RuntimeOptimizationLevel),
    /// Automatic selection based on expression analysis
    Auto,
    /// Verification mode: run both and verify equivalence
    Verification,
}

/// Evaluation strategy configuration
#[derive(Debug, Clone)]
pub struct EvaluationConfig {
    /// Primary evaluation mode
    pub mode: EvaluationMode,
    /// Enable automatic correctness verification
    pub verify_correctness: bool,
    /// Enable performance monitoring
    pub monitor_performance: bool,
    /// Fallback to semantic evaluation on optimization failure
    pub fallback_to_semantic: bool,
    /// Maximum verification time in milliseconds
    pub verification_timeout_ms: u64,
    /// Advanced verification system configuration
    #[cfg(feature = "development")]
    pub verification_config: crate::evaluator::VerificationConfig,
}

/// Evaluation result with metadata
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// Evaluated value
    pub value: Value,
    /// Evaluation mode used
    pub mode_used: EvaluationMode,
    /// Evaluation time in microseconds
    pub evaluation_time_us: u64,
    /// Correctness verification result (if enabled)
    pub correctness_proof: Option<CorrectnessProof>,
    /// Advanced verification result
    #[cfg(feature = "development")]
    pub verification_result: Option<crate::evaluator::SystemVerificationResult>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Whether fallback was used
    pub fallback_used: bool,
}

/// Performance metrics for evaluation
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Total evaluation time (microseconds)
    pub total_time_us: u64,
    /// Semantic evaluation time (microseconds)
    pub semantic_time_us: u64,
    /// Runtime evaluation time (microseconds)
    pub runtime_time_us: u64,
    /// Verification time (microseconds)
    pub verification_time_us: u64,
    /// Number of reduction steps
    pub reduction_steps: usize,
    /// Memory usage (bytes)
    pub memory_usage_bytes: usize,
}

/// Verification result for dual evaluation
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether semantic and runtime results match
    pub results_match: bool,
    /// Semantic evaluation result
    pub semantic_result: Value,
    /// Runtime evaluation result
    pub runtime_result: Value,
    /// Correctness proof
    pub correctness_proof: CorrectnessProof,
    /// Performance comparison
    pub performance_comparison: PerformanceComparison,
}

/// Performance comparison between semantic and runtime evaluation
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    /// Runtime speedup factor (`runtime_time` / `semantic_time`)
    pub speedup_factor: f64,
    /// Memory efficiency (`semantic_memory` / `runtime_memory`)
    pub memory_efficiency: f64,
    /// Optimization effectiveness score (0.0 to 1.0)
    pub optimization_score: f64,
}

/// Unified evaluator interface
pub struct EvaluatorInterface {
    /// Semantic evaluator (reference implementation)
    semantic_evaluator: SemanticEvaluator,
    /// Runtime executor (optimized implementation)
    runtime_executor: RuntimeExecutor,
    /// Evaluator for `ExecutionContext` generation (Phase 11 integration)
    evaluator: Evaluator,
    /// Correctness prover for verification
    correctness_prover: SemanticCorrectnessProver,
    /// Intelligent mode selector
    mode_selector: EvaluationModeSelector,
    /// Advanced verification system
    #[cfg(feature = "development")]
    verification_system: crate::evaluator::VerificationSystem,
    /// Current evaluation configuration
    config: EvaluationConfig,
    /// Performance history
    performance_history: Vec<PerformanceMetrics>,
    /// Verification cache
    verification_cache: HashMap<String, VerificationResult>,
}

impl EvaluatorInterface {
    // ========================================
    // PUBLIC INTERFACE - Evaluator Interface
    // ========================================

    /// Create new evaluator interface with default configuration
    #[must_use] pub fn new() -> Self {
        Self {
            semantic_evaluator: SemanticEvaluator::new(),
            runtime_executor: RuntimeExecutor::new(),
            evaluator: Evaluator::new(),
            correctness_prover: SemanticCorrectnessProver::new(),
            mode_selector: EvaluationModeSelector::new(),
            #[cfg(feature = "development")]
            verification_system: crate::evaluator::VerificationSystem::new(),
            config: EvaluationConfig::default(),
            performance_history: Vec::new(),
            verification_cache: HashMap::new(),
        }
    }

    /// Create with custom configuration
    #[must_use] pub fn with_config(config: EvaluationConfig) -> Self {
        Self {
            semantic_evaluator: SemanticEvaluator::new(),
            runtime_executor: RuntimeExecutor::new(),
            evaluator: Evaluator::new(),
            correctness_prover: SemanticCorrectnessProver::new(),
            mode_selector: EvaluationModeSelector::new(),
            #[cfg(feature = "development")]
            verification_system: crate::evaluator::VerificationSystem::with_config(
                config.verification_config.clone(),
            ),
            config,
            performance_history: Vec::new(),
            verification_cache: HashMap::new(),
        }
    }

    /// Evaluate expression using configured mode
    pub fn eval(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<EvaluationResult> {
        match &self.config.mode {
            EvaluationMode::Semantic => self.eval_semantic(expr, env, cont),
            EvaluationMode::Runtime(level) => self.eval_runtime(expr, env, cont, *level),
            EvaluationMode::Auto => self.eval_auto(expr, env, cont),
            EvaluationMode::Verification => self.eval_verification(expr, env, cont),
        }
    }

    /// ExecutionContext-based unified evaluation (Phase 11 integration)
    /// This is the core integration method that combines static analysis with dynamic optimization
    pub fn eval_with_execution_context(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<EvaluationResult> {
        let start_time = Instant::now();
        
        // Phase 1: Generate ExecutionContext using Evaluator's static analysis
        // Share the same environment reference to maintain consistency
        let execution_context = self.evaluator.create_execution_context(
            expr.clone(),
            env.clone(), // This is Rc<Environment> clone, which shares the same environment
        );
        
        let context_generation_time = start_time.elapsed().as_micros() as u64;
        
        // Phase 2: Select evaluation strategy based on ExecutionContext
        let selected_mode = self.select_evaluation_mode_from_context(&execution_context);
        
        // Phase 3: Execute with selected strategy
        let execution_start = Instant::now();
        let result = match selected_mode {
            EvaluationMode::Semantic => {
                // Use semantic evaluator as fallback or when explicitly requested
                self.eval_semantic(expr, env, cont)
            }
            EvaluationMode::Runtime(level) => {
                // Extract needed information before consuming ExecutionContext
                let complexity_score = execution_context.static_analysis.complexity_score;
                let memory_usage = execution_context.estimated_memory_usage();
                
                // Try RuntimeExecutor with ExecutionContext (shared reference)
                match self.runtime_executor.eval_with_execution_context(execution_context) {
                    Ok(value) => {
                        let execution_time = execution_start.elapsed().as_micros() as u64;
                        
                        // Verify result if enabled
                        let correctness_proof = if self.config.verify_correctness {
                            Some(self.verify_correctness(&expr, &value)?)
                        } else {
                            None
                        };
                        
                        let performance_metrics = PerformanceMetrics {
                            total_time_us: context_generation_time + execution_time,
                            semantic_time_us: 0,
                            runtime_time_us: execution_time,
                            verification_time_us: 0,
                            reduction_steps: complexity_score as usize,
                            memory_usage_bytes: memory_usage,
                        };
                        
                        Ok(EvaluationResult {
                            value,
                            mode_used: EvaluationMode::Runtime(level),
                            evaluation_time_us: execution_time,
                            correctness_proof,
                            #[cfg(feature = "development")]
                            #[cfg(feature = "development")]
            verification_result: None,
                            performance_metrics,
                            fallback_used: false,
                        })
                    }
                    Err(_) if self.config.fallback_to_semantic => {
                        // Fallback to semantic evaluation on optimization failure
                        let mut semantic_result = self.eval_semantic(expr, env, cont)?;
                        semantic_result.fallback_used = true;
                        Ok(semantic_result)
                    }
                    Err(e) => Err(e),
                }
            }
            EvaluationMode::Auto => {
                // Auto mode already handled in select_evaluation_mode_from_context
                unreachable!("Auto mode should be resolved to specific mode")
            }
            EvaluationMode::Verification => {
                // Run both evaluations and verify equivalence
                #[cfg(feature = "development")]
                {
                    self.eval_verification_mode_with_context(execution_context, expr, env, cont)
                }
                #[cfg(not(feature = "development"))]
                {
                    // In production, fall back to semantic evaluation
                    self.eval_semantic(expr, env, cont)
                }
            }
        }?;
        
        // Update performance history
        if self.config.monitor_performance {
            self.performance_history.push(result.performance_metrics.clone());
            
            // Keep only recent history (last 1000 evaluations)
            if self.performance_history.len() > 1000 {
                self.performance_history.drain(0..100);
            }
        }
        
        Ok(result)
    }

    /// Update configuration
    pub fn set_config(&mut self, config: EvaluationConfig) {
        self.config = config;
    }

    /// Get current configuration
    #[must_use] pub fn get_config(&self) -> &EvaluationConfig {
        &self.config
    }

    /// Get performance history
    #[must_use] pub fn get_performance_history(&self) -> &[PerformanceMetrics] {
        &self.performance_history
    }

    /// Clear performance history
    pub fn clear_performance_history(&mut self) {
        self.performance_history.clear();
    }

    /// Get verification cache statistics
    #[must_use] pub fn get_verification_cache_stats(&self) -> (usize, usize) {
        let total_entries = self.verification_cache.len();
        let successful_verifications = self
            .verification_cache
            .values()
            .filter(|v| v.results_match)
            .count();
        (total_entries, successful_verifications)
    }

    /// Get mode selector statistics
    #[must_use] pub fn get_mode_selector_stats(&self) -> Vec<String> {
        use crate::evaluator::ExpressionType;

        let expression_types = vec![
            ExpressionType::Literal,
            ExpressionType::Variable,
            ExpressionType::SimpleArithmetic,
            ExpressionType::ComplexArithmetic,
            ExpressionType::FunctionCall,
            ExpressionType::Lambda,
            ExpressionType::ConditionalExpression,
            ExpressionType::ListProcessing,
            ExpressionType::RecursiveFunction,
            ExpressionType::ComplexNested,
        ];

        let mut stats = Vec::new();
        for expr_type in expression_types {
            let recommendations = self.mode_selector.get_recommendations(&expr_type);
            stats.push(format!("{expr_type:?}: {recommendations:?}"));
        }

        stats
    }

    /// Reset mode selector learning data
    pub fn reset_mode_selector(&mut self) {
        self.mode_selector.clear_history();
    }

    /// Get intelligent mode selection recommendation
    pub fn get_mode_recommendation(&mut self, expr: &Expr) -> EvaluationMode {
        let criteria = SelectionCriteria {
            expression: expr.clone(),
            expected_type: None,
            performance_requirements: PerformanceRequirements::default(),
            context: EvaluationContext::default(),
        };

        self.mode_selector.select_mode(&criteria)
    }

    /// Get verification system statistics
    #[cfg(feature = "development")]
    #[must_use] pub fn get_verification_statistics(&self) -> &crate::evaluator::VerificationStatistics {
        self.verification_system.get_statistics()
    }

    /// Get verification system configuration
    #[cfg(feature = "development")]
    #[must_use] pub fn get_verification_config(&self) -> &crate::evaluator::VerificationConfig {
        self.verification_system.get_config()
    }

    /// Update verification system configuration
    #[cfg(feature = "development")]
    pub fn set_verification_config(&mut self, config: crate::evaluator::VerificationConfig) {
        self.verification_system.set_config(config.clone());
        self.config.verification_config = config;
    }

    /// Reset verification system statistics
    #[cfg(feature = "development")]
    pub fn reset_verification_statistics(&mut self) {
        self.verification_system.reset_statistics();
    }

    /// Clear verification system cache
    #[cfg(feature = "development")]
    pub fn clear_verification_cache(&mut self) {
        self.verification_system.clear_cache();
    }

    /// Helper function to create a callable custom predicate that uses the evaluator
    /// This bridges Scheme procedures to Rust custom predicate functions
    /// Note: This is a simplified implementation for demonstration purposes
    pub fn create_custom_predicate_fn(
        &self,
        _procedure: Value,
        _environment: Environment,
    ) -> crate::value::CustomPredicateFn {
        use std::sync::Arc;
        
        // For now, return a placeholder predicate that always returns false
        // In a full implementation, this would need to properly handle
        // calling Scheme procedures with the evaluator context
        Arc::new(move |_value: &Value| -> bool {
            // Placeholder implementation - always returns false
            false
        })
    }
    
    // Removed duplicate set_config/get_config methods - see line 281-287
    
    /// Evaluate with continuation (compatibility method)
    pub fn eval_with_continuation(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Delegate to the main eval method and extract the value
        self.eval(expr, env, cont).map(|result| result.value)
    }
    
    // Removed duplicate methods - see line 291-305

    // ========================================
    // PRIVATE HELPER METHODS
    // ========================================

    /// Evaluate using semantic evaluator
    fn eval_semantic(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<EvaluationResult> {
        let start_time = Instant::now();

        let value = self.semantic_evaluator.eval_pure(expr.clone(), env, cont)?;

        let evaluation_time = start_time.elapsed().as_micros() as u64;

        let correctness_proof = if self.config.verify_correctness {
            Some(self.verify_correctness(&expr, &value)?)
        } else {
            None
        };

        // Get estimated metrics from semantic evaluator (pure evaluation)
        let performance_metrics = PerformanceMetrics {
            total_time_us: evaluation_time,
            semantic_time_us: evaluation_time,
            runtime_time_us: 0,
            verification_time_us: 0,
            reduction_steps: 1, // Pure semantic evaluation is reference implementation
            memory_usage_bytes: 0, // Pure semantic evaluation has minimal overhead
        };

        Ok(EvaluationResult {
            value,
            mode_used: EvaluationMode::Semantic,
            evaluation_time_us: evaluation_time,
            correctness_proof,
            #[cfg(feature = "development")]
            verification_result: None, // Semantic evaluation is the reference
            performance_metrics,
            fallback_used: false,
        })
    }

    /// Evaluate using runtime executor
    fn eval_runtime(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        level: RuntimeOptimizationLevel,
    ) -> Result<EvaluationResult> {
        let start_time = Instant::now();

        // Set optimization level and attempt runtime evaluation
        self.runtime_executor.set_optimization_level(level);
        let runtime_result =
            self.runtime_executor
                .eval_optimized(expr.clone(), env.clone(), cont.clone());

        match runtime_result {
            Ok(value) => {
                let evaluation_time = start_time.elapsed().as_micros() as u64;

                let correctness_proof = if self.config.verify_correctness {
                    Some(self.verify_correctness(&expr, &value)?)
                } else {
                    None
                };

                // Perform advanced verification against SemanticEvaluator
                #[cfg(feature = "development")]
                let verification_result =
                    if self.config.verification_config.verify_semantic_equivalence {
                        Some(self.verification_system.verify_execution(
                            &expr,
                            &env,
                            &cont,
                            &value,
                            level,
                        )?)
                    } else {
                        None
                    };

                // Get detailed metrics from runtime executor
                let runtime_stats = self.runtime_executor.get_stats();
                
                let performance_metrics = PerformanceMetrics {
                    total_time_us: evaluation_time,
                    semantic_time_us: 0,
                    runtime_time_us: evaluation_time,
                    verification_time_us: {
                        #[cfg(feature = "development")]
                        {
                            verification_result
                                .as_ref()
                                .map_or(0, |vr| vr.verification_time.as_micros() as u64)
                        }
                        #[cfg(not(feature = "development"))]
                        {
                            0
                        }
                    },
                    reduction_steps: runtime_stats.expressions_evaluated,
                    memory_usage_bytes: runtime_stats.pooling_memory_saved,
                };

                Ok(EvaluationResult {
                    value,
                    mode_used: EvaluationMode::Runtime(level),
                    evaluation_time_us: evaluation_time,
                    correctness_proof,
                    #[cfg(feature = "development")]
                    verification_result,
                    performance_metrics,
                    fallback_used: false,
                })
            }
            Err(_) if self.config.fallback_to_semantic => {
                // Fallback to semantic evaluation
                let mut result = self.eval_semantic(expr, env, cont)?;
                result.fallback_used = true;
                Ok(result)
            }
            Err(e) => Err(e),
        }
    }

    /// Automatic evaluation mode selection
    fn eval_auto(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<EvaluationResult> {
        // Use intelligent mode selector
        let criteria = SelectionCriteria {
            expression: expr.clone(),
            expected_type: None,
            performance_requirements: PerformanceRequirements::default(),
            context: EvaluationContext::default(),
        };

        let selected_mode = self.mode_selector.select_mode(&criteria);

        // Execute with selected mode
        match selected_mode {
            EvaluationMode::Semantic => self.eval_semantic(expr, env, cont),
            EvaluationMode::Runtime(level) => self.eval_runtime(expr, env, cont, level),
            EvaluationMode::Verification => self.eval_verification(expr, env, cont),
            EvaluationMode::Auto => {
                // Fallback to simple heuristic if auto mode returns auto
                let complexity = self.analyze_expression_complexity(&expr);
                if complexity > 10 {
                    self.eval_runtime(expr, env, cont, RuntimeOptimizationLevel::Conservative)
                } else {
                    self.eval_semantic(expr, env, cont)
                }
            }
        }
    }

    /// Verification mode: run both evaluators and verify equivalence
    fn eval_verification(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<EvaluationResult> {
        let start_time = Instant::now();

        // Run semantic evaluation
        let semantic_start = Instant::now();
        let semantic_result =
            self.semantic_evaluator
                .eval_pure(expr.clone(), env.clone(), cont.clone())?;
        let semantic_time = semantic_start.elapsed().as_micros() as u64;

        // Run runtime evaluation
        let runtime_start = Instant::now();
        self.runtime_executor
            .set_optimization_level(RuntimeOptimizationLevel::Conservative);
        let runtime_result =
            self.runtime_executor
                .eval_optimized(expr.clone(), env.clone(), cont.clone())?;
        let runtime_time = runtime_start.elapsed().as_micros() as u64;

        // Verify equivalence
        let verification_start = Instant::now();
        let results_match = self.values_equivalent(&semantic_result, &runtime_result)?;
        let verification_time = verification_start.elapsed().as_micros() as u64;

        if !results_match {
            return Err(LambdustError::runtime_error(
                "Semantic and runtime evaluation results do not match".to_string(),
            ));
        }

        let total_time = start_time.elapsed().as_micros() as u64;

        let correctness_proof = self.verify_correctness(&expr, &semantic_result)?;

        // Perform comprehensive verification
        let _verification_result = VerificationResult {
            results_match,
            semantic_result: semantic_result.clone(),
            runtime_result: runtime_result.clone(),
            correctness_proof: correctness_proof.clone(),
            performance_comparison: PerformanceComparison {
                speedup_factor: if semantic_time > 0 { semantic_time as f64 / runtime_time as f64 } else { 1.0 },
                memory_efficiency: 1.0, // Placeholder
                optimization_score: if semantic_time > runtime_time { 0.8 } else { 0.5 },
            },
        };

        // Get aggregated metrics from both evaluators
        let runtime_stats = self.runtime_executor.get_stats();
        
        let performance_metrics = PerformanceMetrics {
            total_time_us: total_time,
            semantic_time_us: semantic_time,
            runtime_time_us: runtime_time,
            verification_time_us: verification_time
                + verification_time,
            reduction_steps: 1 + runtime_stats.expressions_evaluated, // Semantic + Runtime
            memory_usage_bytes: runtime_stats.pooling_memory_saved,
        };

        Ok(EvaluationResult {
            value: semantic_result, // Use semantic result as reference
            mode_used: EvaluationMode::Verification,
            evaluation_time_us: total_time,
            correctness_proof: Some(correctness_proof),
            #[cfg(feature = "development")]
            verification_result: Some(self.convert_to_system_verification_result(_verification_result)),
            performance_metrics,
            fallback_used: false,
        })
    }

    /// Verify correctness of evaluation result
    fn verify_correctness(&mut self, expr: &Expr, value: &Value) -> Result<CorrectnessProof> {
        let property = CorrectnessProperty::ReferentialTransparency(expr.clone(), value.clone());

        self.correctness_prover.prove_property(property)
    }

    /// Check if two values are equivalent
    fn values_equivalent(&self, v1: &Value, v2: &Value) -> Result<bool> {
        // Simple structural equivalence check
        Ok(format!("{v1:?}") == format!("{v2:?}"))
    }

    /// Analyze expression complexity for auto mode
    fn analyze_expression_complexity(&self, expr: &Expr) -> usize {
        match expr {
            Expr::Literal(_) | Expr::Variable(_) => 1,
            Expr::List(exprs) => {
                1 + exprs
                    .iter()
                    .map(|e| self.analyze_expression_complexity(e))
                    .sum::<usize>()
            }
            Expr::Quote(expr) => 1 + self.analyze_expression_complexity(expr),
            Expr::Vector(exprs) => {
                1 + exprs
                    .iter()
                    .map(|e| self.analyze_expression_complexity(e))
                    .sum::<usize>()
            }
            _ => 5, // Default complexity for other expressions
        }
    }
    
    /// Select evaluation mode based on `ExecutionContext` analysis
    fn select_evaluation_mode_from_context(&self, context: &ExecutionContext) -> EvaluationMode {
        match &self.config.mode {
            EvaluationMode::Auto => {
                // Intelligent mode selection based on ExecutionContext
                if context.should_use_jit() {
                    EvaluationMode::Runtime(RuntimeOptimizationLevel::Aggressive)
                } else if context.should_optimize() {
                    EvaluationMode::Runtime(RuntimeOptimizationLevel::Balanced)
                } else if context.static_analysis.complexity_score < 30 {
                    EvaluationMode::Runtime(RuntimeOptimizationLevel::Conservative)
                } else {
                    EvaluationMode::Semantic
                }
            }
            EvaluationMode::Verification => EvaluationMode::Verification,
            other => other.clone(), // Use configured mode directly
        }
    }
    
    /// Run verification mode with ExecutionContext
    #[cfg(feature = "development")]
    fn eval_verification_mode_with_context(
        &mut self,
        execution_context: ExecutionContext,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<EvaluationResult> {
        let start_time = Instant::now();
        
        // Run semantic evaluation
        let semantic_start = Instant::now();
        let semantic_result = self.semantic_evaluator.eval_pure(
            expr.clone(), 
            env.clone(), 
            cont.clone()
        )?;
        let semantic_time = semantic_start.elapsed().as_micros() as u64;
        
        // Run runtime evaluation with ExecutionContext
        let runtime_start = Instant::now();
        let runtime_result = self.runtime_executor.eval_with_execution_context(execution_context.clone());
        let runtime_time = runtime_start.elapsed().as_micros() as u64;
        
        let total_time = start_time.elapsed().as_micros() as u64;
        
        match runtime_result {
            Ok(runtime_value) => {
                // Verify equivalence
                let verification_start = Instant::now();
                let values_equivalent = self.values_equivalent(&semantic_result, &runtime_value)?;
                let verification_time = verification_start.elapsed().as_micros() as u64;
                
                let verification_result = Some(VerificationResult {
                    results_match: values_equivalent,
                    semantic_result: semantic_result.clone(),
                    runtime_result: runtime_value.clone(),
                    correctness_proof: CorrectnessProof {
                        property: CorrectnessProperty::ReferentialTransparency(
                            expr.clone(), 
                            runtime_value.clone()
                        ),
                        proven: values_equivalent,
                        #[cfg(feature = "development")]
                        proof_term: None,
                        counterexample: None,
                        verification_time_ms: verification_time / 1000,
                    },
                    performance_comparison: PerformanceComparison {
                        speedup_factor: if semantic_time > 0 { semantic_time as f64 / runtime_time as f64 } else { 1.0 },
                        memory_efficiency: 1.0, // Placeholder
                        optimization_score: if semantic_time > runtime_time { 0.8 } else { 0.5 },
                    },
                });
                
                let performance_metrics = PerformanceMetrics {
                    total_time_us: total_time,
                    semantic_time_us: semantic_time,
                    runtime_time_us: runtime_time,
                    verification_time_us: verification_time,
                    reduction_steps: execution_context.static_analysis.complexity_score as usize,
                    memory_usage_bytes: execution_context.estimated_memory_usage(),
                };
                
                Ok(EvaluationResult {
                    value: if values_equivalent { runtime_value } else { semantic_result },
                    mode_used: EvaluationMode::Verification,
                    evaluation_time_us: total_time,
                    correctness_proof: None,
                    verification_result: verification_result.map(|vr| self.convert_to_system_verification_result(vr)),
                    performance_metrics,
                    fallback_used: !values_equivalent,
                })
            }
            Err(_) => {
                // Runtime evaluation failed, return semantic result
                let performance_metrics = PerformanceMetrics {
                    total_time_us: total_time,
                    semantic_time_us: semantic_time,
                    runtime_time_us: 0,
                    verification_time_us: 0,
                    reduction_steps: execution_context.static_analysis.complexity_score as usize,
                    memory_usage_bytes: execution_context.estimated_memory_usage(),
                };
                
                Ok(EvaluationResult {
                    value: semantic_result,
                    mode_used: EvaluationMode::Semantic,
                    evaluation_time_us: total_time,
                    correctness_proof: None,
                    #[cfg(feature = "development")]
            verification_result: None,
                    performance_metrics,
                    fallback_used: true,
                })
            }
        }
    }

    /// Convert evaluator_interface::VerificationResult to SystemVerificationResult
    #[cfg(feature = "development")]
    fn convert_to_system_verification_result(&self, verification_result: VerificationResult) -> crate::evaluator::SystemVerificationResult {
        use crate::prover::verification_system::{VerificationStatus, VerificationAnalysis};
        use std::time::Duration;

        crate::evaluator::SystemVerificationResult {
            status: if verification_result.results_match { VerificationStatus::Passed } else { VerificationStatus::Failed("Results don't match".to_string()) },
            reference_result: Some(verification_result.semantic_result),
            actual_result: Some(verification_result.runtime_result),
            semantic_equivalence: Some(verification_result.results_match),
            correctness_proof: Some(verification_result.correctness_proof),
            theorem_proof: None,
            verification_time: Duration::from_micros(10), // Placeholder
            analysis: VerificationAnalysis {
                value_type_match: verification_result.results_match,
                structural_match: verification_result.results_match,
                numerical_precision_match: Some(verification_result.results_match),
                string_content_match: Some(verification_result.results_match),
                list_structure_match: Some(verification_result.results_match),
                discrepancies: if verification_result.results_match { Vec::new() } else { vec!["Results don't match".to_string()] },
                confidence_level: if verification_result.results_match { 1.0 } else { 0.0 },
            },
        }
    }
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            mode: EvaluationMode::Auto,
            verify_correctness: true,
            monitor_performance: true,
            fallback_to_semantic: true,
            verification_timeout_ms: 5000,
            #[cfg(feature = "development")]
            verification_config: crate::evaluator::VerificationConfig::default(),
        }
    }
}

impl Default for EvaluatorInterface {
    fn default() -> Self {
        Self::new()
    }
}
