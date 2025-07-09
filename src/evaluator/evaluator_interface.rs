//! Unified evaluator interface for transparent semantic/runtime evaluation switching
//!
//! This module provides a unified interface that allows transparent switching
//! between semantic evaluation (SemanticEvaluator) and optimized runtime execution
//! (RuntimeExecutor), with automatic correctness verification.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{
    Continuation, RuntimeExecutor, RuntimeOptimizationLevel, SemanticEvaluator,
    SemanticCorrectnessProver, CorrectnessProperty, CorrectnessProof,
    EvaluationModeSelector, SelectionCriteria, PerformanceRequirements, EvaluationContext,
    VerificationSystem, VerificationConfig, SystemVerificationResult,
};
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
    pub verification_config: VerificationConfig,
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
    pub verification_result: Option<SystemVerificationResult>,
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
    /// Runtime speedup factor (runtime_time / semantic_time)
    pub speedup_factor: f64,
    /// Memory efficiency (semantic_memory / runtime_memory)
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
    /// Correctness prover for verification
    correctness_prover: SemanticCorrectnessProver,
    /// Intelligent mode selector
    mode_selector: EvaluationModeSelector,
    /// Advanced verification system
    verification_system: VerificationSystem,
    /// Current evaluation configuration
    config: EvaluationConfig,
    /// Performance history
    performance_history: Vec<PerformanceMetrics>,
    /// Verification cache
    verification_cache: HashMap<String, VerificationResult>,
}

impl EvaluatorInterface {
    /// Create new evaluator interface with default configuration
    pub fn new() -> Self {
        Self {
            semantic_evaluator: SemanticEvaluator::new(),
            runtime_executor: RuntimeExecutor::new(),
            correctness_prover: SemanticCorrectnessProver::new(),
            mode_selector: EvaluationModeSelector::new(),
            verification_system: VerificationSystem::new(),
            config: EvaluationConfig::default(),
            performance_history: Vec::new(),
            verification_cache: HashMap::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: EvaluationConfig) -> Self {
        Self {
            semantic_evaluator: SemanticEvaluator::new(),
            runtime_executor: RuntimeExecutor::new(),
            correctness_prover: SemanticCorrectnessProver::new(),
            mode_selector: EvaluationModeSelector::new(),
            verification_system: VerificationSystem::with_config(config.verification_config.clone()),
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
        let _start_time = Instant::now();
        
        match &self.config.mode {
            EvaluationMode::Semantic => {
                self.eval_semantic(expr, env, cont)
            }
            EvaluationMode::Runtime(level) => {
                self.eval_runtime(expr, env, cont, level.clone())
            }
            EvaluationMode::Auto => {
                self.eval_auto(expr, env, cont)
            }
            EvaluationMode::Verification => {
                self.eval_verification(expr, env, cont)
            }
        }
    }

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

        let performance_metrics = PerformanceMetrics {
            total_time_us: evaluation_time,
            semantic_time_us: evaluation_time,
            runtime_time_us: 0,
            verification_time_us: 0,
            reduction_steps: 0, // TODO: track from SemanticEvaluator
            memory_usage_bytes: 0, // TODO: implement memory tracking
        };

        Ok(EvaluationResult {
            value,
            mode_used: EvaluationMode::Semantic,
            evaluation_time_us: evaluation_time,
            correctness_proof,
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
        self.runtime_executor.set_optimization_level(level.clone());
        let runtime_result = self.runtime_executor.eval_optimized(
            expr.clone(),
            env.clone(),
            cont.clone(),
        );

        match runtime_result {
            Ok(value) => {
                let evaluation_time = start_time.elapsed().as_micros() as u64;
                
                let correctness_proof = if self.config.verify_correctness {
                    Some(self.verify_correctness(&expr, &value)?)
                } else {
                    None
                };

                // Perform advanced verification against SemanticEvaluator
                let verification_result = if self.config.verification_config.verify_semantic_equivalence {
                    Some(self.verification_system.verify_execution(
                        &expr,
                        &env,
                        &cont,
                        &value,
                        level.clone(),
                    )?)
                } else {
                    None
                };

                let performance_metrics = PerformanceMetrics {
                    total_time_us: evaluation_time,
                    semantic_time_us: 0,
                    runtime_time_us: evaluation_time,
                    verification_time_us: verification_result.as_ref()
                        .map(|vr| vr.verification_time.as_micros() as u64)
                        .unwrap_or(0),
                    reduction_steps: 0, // TODO: track from RuntimeExecutor
                    memory_usage_bytes: 0, // TODO: implement memory tracking
                };

                Ok(EvaluationResult {
                    value,
                    mode_used: EvaluationMode::Runtime(level),
                    evaluation_time_us: evaluation_time,
                    correctness_proof,
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
        let semantic_result = self.semantic_evaluator.eval_pure(
            expr.clone(),
            env.clone(),
            cont.clone(),
        )?;
        let semantic_time = semantic_start.elapsed().as_micros() as u64;

        // Run runtime evaluation
        let runtime_start = Instant::now();
        self.runtime_executor.set_optimization_level(RuntimeOptimizationLevel::Conservative);
        let runtime_result = self.runtime_executor.eval_optimized(
            expr.clone(),
            env.clone(),
            cont.clone(),
        )?;
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
        let verification_result = self.verification_system.verify_execution(
            &expr,
            &env,
            &cont,
            &runtime_result,
            RuntimeOptimizationLevel::Conservative,
        )?;

        let performance_metrics = PerformanceMetrics {
            total_time_us: total_time,
            semantic_time_us: semantic_time,
            runtime_time_us: runtime_time,
            verification_time_us: verification_time + verification_result.verification_time.as_micros() as u64,
            reduction_steps: 0, // TODO: aggregate from both evaluators
            memory_usage_bytes: 0, // TODO: implement memory tracking
        };

        Ok(EvaluationResult {
            value: semantic_result, // Use semantic result as reference
            mode_used: EvaluationMode::Verification,
            evaluation_time_us: total_time,
            correctness_proof: Some(correctness_proof),
            verification_result: Some(verification_result),
            performance_metrics,
            fallback_used: false,
        })
    }

    /// Verify correctness of evaluation result
    fn verify_correctness(&mut self, expr: &Expr, value: &Value) -> Result<CorrectnessProof> {
        let property = CorrectnessProperty::ReferentialTransparency(
            expr.clone(),
            value.clone(),
        );
        
        self.correctness_prover.prove_property(property)
    }

    /// Check if two values are equivalent
    fn values_equivalent(&self, v1: &Value, v2: &Value) -> Result<bool> {
        // Simple structural equivalence check
        // TODO: implement deeper semantic equivalence
        Ok(format!("{:?}", v1) == format!("{:?}", v2))
    }

    /// Analyze expression complexity for auto mode
    fn analyze_expression_complexity(&self, expr: &Expr) -> usize {
        match expr {
            Expr::Literal(_) | Expr::Variable(_) => 1,
            Expr::List(exprs) => {
                1 + exprs.iter().map(|e| self.analyze_expression_complexity(e)).sum::<usize>()
            }
            Expr::Quote(expr) => 1 + self.analyze_expression_complexity(expr),
            Expr::Vector(exprs) => {
                1 + exprs.iter().map(|e| self.analyze_expression_complexity(e)).sum::<usize>()
            }
            _ => 5, // Default complexity for other expressions
        }
    }

    /// Update configuration
    pub fn set_config(&mut self, config: EvaluationConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &EvaluationConfig {
        &self.config
    }

    /// Get performance history
    pub fn get_performance_history(&self) -> &[PerformanceMetrics] {
        &self.performance_history
    }

    /// Clear performance history
    pub fn clear_performance_history(&mut self) {
        self.performance_history.clear();
    }

    /// Get verification cache statistics
    pub fn get_verification_cache_stats(&self) -> (usize, usize) {
        let total_entries = self.verification_cache.len();
        let successful_verifications = self.verification_cache.values()
            .filter(|v| v.results_match)
            .count();
        (total_entries, successful_verifications)
    }


    /// Get mode selector statistics
    pub fn get_mode_selector_stats(&self) -> Vec<String> {
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
            stats.push(format!("{:?}: {:?}", expr_type, recommendations));
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
    pub fn get_verification_statistics(&self) -> &crate::evaluator::VerificationStatistics {
        self.verification_system.get_statistics()
    }

    /// Get verification system configuration
    pub fn get_verification_config(&self) -> &VerificationConfig {
        self.verification_system.get_config()
    }

    /// Update verification system configuration
    pub fn set_verification_config(&mut self, config: VerificationConfig) {
        self.verification_system.set_config(config.clone());
        self.config.verification_config = config;
    }

    /// Reset verification system statistics
    pub fn reset_verification_statistics(&mut self) {
        self.verification_system.reset_statistics();
    }

    /// Clear verification system cache
    pub fn clear_verification_cache(&mut self) {
        self.verification_system.clear_cache();
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
            verification_config: VerificationConfig::default(),
        }
    }
}

impl Default for EvaluatorInterface {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_evaluator_interface_creation() {
        let interface = EvaluatorInterface::new();
        assert_eq!(interface.config.mode, EvaluationMode::Auto);
        assert!(interface.config.verify_correctness);
        assert!(interface.config.monitor_performance);
        assert!(interface.config.fallback_to_semantic);
    }

    #[test]
    fn test_semantic_evaluation() {
        let mut interface = EvaluatorInterface::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        
        interface.config.mode = EvaluationMode::Semantic;
        interface.config.verify_correctness = false; // Disable for simple test
        
        let result = interface.eval(expr, env, Continuation::Identity).unwrap();
        
        assert_eq!(result.mode_used, EvaluationMode::Semantic);
        assert!(!result.fallback_used);
        assert!(result.evaluation_time_us > 0);
    }

    #[test]
    fn test_runtime_evaluation_with_fallback() {
        let mut interface = EvaluatorInterface::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        
        interface.config.mode = EvaluationMode::Runtime(RuntimeOptimizationLevel::Conservative);
        interface.config.fallback_to_semantic = true;
        interface.config.verify_correctness = false; // Disable for simple test
        
        let result = interface.eval(expr, env, Continuation::Identity).unwrap();
        
        // Should succeed either with runtime or fallback to semantic
        assert!(result.evaluation_time_us > 0);
    }

    #[test]
    fn test_auto_mode_selection() {
        let mut interface = EvaluatorInterface::new();
        interface.config.verify_correctness = false; // Disable for simple test
        
        // Simple expression should use semantic evaluation
        let simple_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        
        let result = interface.eval(simple_expr, env, Continuation::Identity).unwrap();
        assert_eq!(result.mode_used, EvaluationMode::Semantic);
    }

    #[test]
    fn test_expression_complexity_analysis() {
        let interface = EvaluatorInterface::new();
        
        // Simple literal
        let simple = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        assert_eq!(interface.analyze_expression_complexity(&simple), 1);
        
        // Complex nested expression
        let complex = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::List(vec![
                Expr::Variable("*".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
            ]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(4))),
        ]);
        assert!(interface.analyze_expression_complexity(&complex) > 5);
    }

    #[test]
    fn test_configuration_management() {
        let mut interface = EvaluatorInterface::new();
        
        let custom_config = EvaluationConfig {
            mode: EvaluationMode::Semantic,
            verify_correctness: false,
            monitor_performance: false,
            fallback_to_semantic: false,
            verification_timeout_ms: 1000,
            verification_config: VerificationConfig::default(),
        };
        
        interface.set_config(custom_config.clone());
        assert_eq!(interface.get_config().mode, EvaluationMode::Semantic);
        assert!(!interface.get_config().verify_correctness);
    }

    #[test]
    fn test_performance_tracking() {
        let mut interface = EvaluatorInterface::new();
        assert_eq!(interface.get_performance_history().len(), 0);
        
        interface.clear_performance_history();
        assert_eq!(interface.get_performance_history().len(), 0);
    }

    #[test]
    fn test_verification_cache_management() {
        let mut interface = EvaluatorInterface::new();
        let (total, successful) = interface.get_verification_cache_stats();
        assert_eq!(total, 0);
        assert_eq!(successful, 0);
        
        interface.clear_verification_cache();
        let (total, successful) = interface.get_verification_cache_stats();
        assert_eq!(total, 0);
        assert_eq!(successful, 0);
    }
}