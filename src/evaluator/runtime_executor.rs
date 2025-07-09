//! Runtime Executor for optimized evaluation
//!
//! This module implements the runtime executor that applies performance optimizations
//! while maintaining correctness through reference to the semantic evaluator.
//!
//! The RuntimeExecutor integrates all optimization systems:
//! - JIT loop optimization
//! - Continuation pooling
//! - Tail call optimization
//! - Inline evaluation
//! - Expression analysis and optimization hints

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{
    Continuation, ContinuationPoolManager, ExpressionAnalyzer, InlineEvaluator, 
    JitLoopOptimizer, SemanticEvaluator, TailCallOptimizer,
    IntegratedOptimizationManager, OptimizationResult,
};
use crate::value::Value;
use std::rc::Rc;

/// Runtime optimization level
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuntimeOptimizationLevel {
    /// No optimizations
    None,
    /// Conservative optimizations only
    Conservative,
    /// Balanced optimization approach
    Balanced,
    /// Aggressive optimizations
    Aggressive,
}

/// Placeholder analysis result for initial implementation
#[derive(Debug, Clone)]
pub struct PlaceholderAnalysis {
    // Placeholder fields
}

impl PlaceholderAnalysis {
    fn new() -> Self {
        Self {}
    }
    
    fn is_tail_call_candidate(&self) -> bool {
        false // Conservative placeholder
    }
    
    fn is_hot_path(&self) -> bool {
        false // Conservative placeholder
    }
    
    fn is_loop_candidate(&self) -> bool {
        false // Conservative placeholder
    }
}

/// Placeholder for optimized tail call
#[derive(Debug, Clone)]
pub struct PlaceholderOptimizedTailCall {
    // Placeholder
}

/// Placeholder for generated code
#[derive(Debug, Clone)]
pub struct PlaceholderGeneratedCode {
    // Placeholder
}

/// Runtime executor with integrated optimization systems
pub struct RuntimeExecutor {
    /// Reference semantic evaluator for correctness verification
    semantic_evaluator: SemanticEvaluator,
    
    /// Expression analyzer for optimization hints
    expression_analyzer: ExpressionAnalyzer,
    
    /// JIT loop optimizer
    jit_optimizer: JitLoopOptimizer,
    
    /// Tail call optimizer
    tail_call_optimizer: TailCallOptimizer,
    
    /// Inline evaluator for hot path optimization
    inline_evaluator: InlineEvaluator,
    
    /// Continuation pooling manager
    continuation_pooler: ContinuationPoolManager,
    
    /// Integrated optimization manager
    integrated_optimizer: IntegratedOptimizationManager,
    
    /// Current optimization level
    optimization_level: RuntimeOptimizationLevel,
    
    /// Whether to verify against semantic evaluator
    verification_enabled: bool,
    
    /// Runtime statistics
    stats: RuntimeStats,
    
    /// Recursion depth tracking
    recursion_depth: usize,
    max_recursion_depth: usize,
}

/// Runtime execution statistics
#[derive(Debug, Default, Clone)]
pub struct RuntimeStats {
    /// Total expressions evaluated
    pub expressions_evaluated: usize,
    
    /// Optimizations applied
    pub optimizations_applied: usize,
    
    /// JIT compilations performed
    pub jit_compilations: usize,
    
    /// Tail call optimizations
    pub tail_calls_optimized: usize,
    
    /// Inline evaluations
    pub inline_evaluations: usize,
    
    /// Continuation pool hits
    pub continuation_pool_hits: usize,
    
    /// Verification checks performed
    pub verification_checks: usize,
    
    /// Verification mismatches found
    pub verification_mismatches: usize,
}

impl RuntimeExecutor {
    /// Create a new runtime executor with default optimization level
    pub fn new() -> Self {
        Self {
            semantic_evaluator: SemanticEvaluator::new(),
            expression_analyzer: ExpressionAnalyzer::new(),
            jit_optimizer: JitLoopOptimizer::new(),
            tail_call_optimizer: TailCallOptimizer::new(),
            inline_evaluator: InlineEvaluator::new(),
            continuation_pooler: ContinuationPoolManager::new(),
            integrated_optimizer: IntegratedOptimizationManager::new(),
            optimization_level: RuntimeOptimizationLevel::Balanced,
            verification_enabled: cfg!(debug_assertions),
            stats: RuntimeStats::default(),
            recursion_depth: 0,
            max_recursion_depth: 1000,
        }
    }
    
    /// Create runtime executor with custom optimization level
    pub fn with_optimization_level(level: RuntimeOptimizationLevel) -> Self {
        let mut executor = Self::new();
        executor.optimization_level = level;
        executor
    }
    
    /// Create runtime executor with custom environment
    pub fn with_environment(env: Rc<Environment>) -> Self {
        let mut executor = Self::new();
        executor.semantic_evaluator = SemanticEvaluator::with_environment(env);
        executor
    }
    
    /// Enable or disable verification against semantic evaluator
    pub fn set_verification_enabled(&mut self, enabled: bool) {
        self.verification_enabled = enabled;
    }
    
    /// Get current optimization level
    pub fn optimization_level(&self) -> RuntimeOptimizationLevel {
        self.optimization_level.clone()
    }
    
    /// Set optimization level
    pub fn set_optimization_level(&mut self, level: RuntimeOptimizationLevel) {
        self.optimization_level = level;
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
        self.recursion_depth += 1;
        self.stats.expressions_evaluated += 1;
        
        // Phase 1: Expression analysis for optimization hints
        // For now, skip analysis and use placeholder
        let analysis = PlaceholderAnalysis::new();
        
        // Phase 2: Apply optimizations based on analysis
        let result = match self.optimization_level.clone() {
            RuntimeOptimizationLevel::None => {
                // No optimizations - delegate to semantic evaluator
                self.semantic_evaluator.eval_pure(expr, env, cont)
            }
            
            RuntimeOptimizationLevel::Conservative => {
                // Conservative optimizations only
                self.eval_with_conservative_optimizations(expr, env, cont, &analysis)
            }
            
            RuntimeOptimizationLevel::Balanced => {
                // Balanced optimization approach
                self.eval_with_balanced_optimizations(expr, env, cont, &analysis)
            }
            
            RuntimeOptimizationLevel::Aggressive => {
                // Aggressive optimizations
                self.eval_with_aggressive_optimizations(expr, env, cont, &analysis)
            }
        };
        
        // Phase 3: Verification (if enabled) - currently disabled for Phase 2
        // if self.verification_enabled {
        //     self.verify_result(&expr, &env, &cont, &result)?;
        // }
        
        self.recursion_depth -= 1;
        result
    }
    
    /// Conservative optimizations: basic optimizations with high confidence
    fn eval_with_conservative_optimizations(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        _analysis: &PlaceholderAnalysis,
    ) -> Result<Value> {
        // Apply only safe optimizations using integrated optimization system
        
        // Step 1: Select conservative optimization strategies
        let strategies = match self.integrated_optimizer.select_optimization_strategy(&expr, &RuntimeOptimizationLevel::Conservative) {
            Ok(strategies) => strategies,
            Err(_) => {
                // Fallback to semantic evaluation if strategy selection fails
                return self.semantic_evaluator.eval_pure(expr, env, cont);
            }
        };
        
        // Step 2: Execute optimizations if strategies are available
        if !strategies.is_empty() {
            match self.integrated_optimizer.execute_optimization(expr.clone(), env.clone(), strategies) {
                Ok(optimization_result) => {
                    if optimization_result.success {
                        // Apply optimization result
                        self.stats.optimizations_applied += 1;
                        return self.apply_optimization_result(optimization_result, env, cont);
                    }
                }
                Err(_) => {
                    // Fallback on optimization failure
                }
            }
        }
        
        // Fallback to semantic evaluation
        self.semantic_evaluator.eval_pure(expr, env, cont)
    }
    
    /// Balanced optimizations: good balance of safety and performance
    fn eval_with_balanced_optimizations(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        _analysis: &PlaceholderAnalysis,
    ) -> Result<Value> {
        // Apply balanced optimizations using integrated optimization system
        
        // Step 1: Select balanced optimization strategies
        let strategies = match self.integrated_optimizer.select_optimization_strategy(&expr, &RuntimeOptimizationLevel::Balanced) {
            Ok(strategies) => strategies,
            Err(_) => {
                // Fallback to semantic evaluation if strategy selection fails
                return self.semantic_evaluator.eval_pure(expr, env, cont);
            }
        };
        
        // Step 2: Execute optimizations if strategies are available
        if !strategies.is_empty() {
            match self.integrated_optimizer.execute_optimization(expr.clone(), env.clone(), strategies) {
                Ok(optimization_result) => {
                    if optimization_result.success {
                        // Apply optimization result
                        self.stats.optimizations_applied += 1;
                        return self.apply_optimization_result(optimization_result, env, cont);
                    }
                }
                Err(_) => {
                    // Fallback on optimization failure
                }
            }
        }
        
        // Fallback to semantic evaluation
        self.semantic_evaluator.eval_pure(expr, env, cont)
    }
    
    /// Aggressive optimizations: maximum performance optimizations
    fn eval_with_aggressive_optimizations(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        _analysis: &PlaceholderAnalysis,
    ) -> Result<Value> {
        // Apply aggressive optimizations using integrated optimization system
        
        // Step 1: Select aggressive optimization strategies
        let strategies = match self.integrated_optimizer.select_optimization_strategy(&expr, &RuntimeOptimizationLevel::Aggressive) {
            Ok(strategies) => strategies,
            Err(_) => {
                // Fallback to semantic evaluation if strategy selection fails
                return self.semantic_evaluator.eval_pure(expr, env, cont);
            }
        };
        
        // Step 2: Execute optimizations if strategies are available
        if !strategies.is_empty() {
            match self.integrated_optimizer.execute_optimization(expr.clone(), env.clone(), strategies) {
                Ok(optimization_result) => {
                    if optimization_result.success {
                        // Apply optimization result
                        self.stats.optimizations_applied += 1;
                        return self.apply_optimization_result(optimization_result, env, cont);
                    }
                }
                Err(_) => {
                    // Fallback on optimization failure
                }
            }
        }
        
        // Fallback to semantic evaluation
        self.semantic_evaluator.eval_pure(expr, env, cont)
    }
    
    /// Apply optimization result from IntegratedOptimizationManager
    fn apply_optimization_result(
        &mut self,
        optimization_result: OptimizationResult,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Update statistics based on the optimization result
        if optimization_result.success {
            self.stats.optimizations_applied += 1;
            
            // Determine optimization type from strategy name and apply accordingly
            match optimization_result.applied_strategy.as_str() {
                s if s.contains("tail_call") => {
                    self.stats.tail_calls_optimized += 1;
                }
                s if s.contains("jit") || s.contains("loop") => {
                    self.stats.jit_compilations += 1;
                }
                s if s.contains("inline") => {
                    self.stats.inline_evaluations += 1;
                }
                s if s.contains("continuation") || s.contains("pool") => {
                    self.stats.continuation_pool_hits += 1;
                }
                _ => {
                    // General optimization
                }
            }
            
            // Recursively evaluate the optimized expression
            self.eval_optimized(optimization_result.optimized_expression, env, cont)
        } else {
            // Optimization failed, use the original expression with semantic evaluator
            if let Some(error_msg) = optimization_result.error_message {
                eprintln!("Optimization failed: {}", error_msg);
            }
            
            // For now, we'll fallback to the optimized expression even if optimization "failed"
            // This is because the integrated optimization system may still produce a valid expression
            self.semantic_evaluator.eval_pure(optimization_result.optimized_expression, env, cont)
        }
    }
    
    /// Apply optimized tail call result (placeholder for future implementation)
    #[allow(dead_code)]
    fn apply_optimized_tail_call(
        &mut self,
        _optimized: PlaceholderOptimizedTailCall,
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // For Phase 2: fallback to semantic evaluation
        self.semantic_evaluator.eval_pure(
            crate::ast::Expr::Literal(crate::ast::Literal::Nil),
            Rc::new(crate::environment::Environment::new()),
            cont,
        )
    }
    
    /// Apply JIT compilation result (placeholder for future implementation)
    #[allow(dead_code)]
    fn apply_jit_result(
        &mut self,
        _jit_result: PlaceholderGeneratedCode,
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // For Phase 2: fallback to semantic evaluation
        self.semantic_evaluator.eval_pure(
            crate::ast::Expr::Literal(crate::ast::Literal::Nil),
            Rc::new(crate::environment::Environment::new()),
            cont,
        )
    }
    
    /// Apply continuation with optimization (placeholder for future implementation)
    #[allow(dead_code)]
    fn apply_continuation_optimized(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        // For Phase 2: use semantic evaluator's continuation system
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
        // For Phase 2: fallback to semantic evaluator
        self.semantic_evaluator.eval_pure(
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
        // For Phase 2: fallback to semantic evaluator
        self.semantic_evaluator.eval_pure(
            crate::ast::Expr::Literal(crate::ast::Literal::Nil),
            Rc::new(crate::environment::Environment::new()),
            cont,
        )
    }
    
    /// Optimized builtin application (placeholder for future implementation)
    #[allow(dead_code)]
    fn apply_builtin_optimized(&self, name: &str, args: &[Value]) -> Result<Value> {
        // For Phase 2: use simple implementation
        match name {
            "+" => self.builtin_add_simple(args),
            "-" => self.builtin_subtract_simple(args),
            "*" => self.builtin_multiply_simple(args),
            _ => {
                // For other builtins, fallback to error for now
                Err(LambdustError::runtime_error(format!(
                    "Builtin '{}' not implemented in runtime executor yet",
                    name
                )))
            }
        }
    }
    
    /// Simple addition implementation
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
    
    /// Simple division implementation (placeholder)
    #[allow(dead_code)]
    fn builtin_divide_simple(&self, _args: &[Value]) -> Result<Value> {
        // For Phase 2: not implemented yet
        Err(LambdustError::runtime_error("Division not implemented in runtime executor yet"))
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
        // For Phase 2: verification is disabled
        Ok(())
    }
    
    /// Compare two values for equality (placeholder for future implementation)
    #[allow(dead_code)]
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        // For Phase 2: simple equality check for basic types
        match (a, b) {
            (Value::Number(crate::lexer::SchemeNumber::Integer(a)), Value::Number(crate::lexer::SchemeNumber::Integer(b))) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Character(a), Value::Character(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Undefined, Value::Undefined) => true,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            _ => false, // More complex comparison would be needed for pairs, vectors, etc.
        }
    }
    
    /// Check recursion depth
    fn check_recursion_depth(&self) -> Result<()> {
        if self.recursion_depth >= self.max_recursion_depth {
            return Err(LambdustError::stack_overflow());
        }
        Ok(())
    }
    
    /// Get current runtime statistics
    pub fn get_stats(&self) -> &RuntimeStats {
        &self.stats
    }
    
    /// Reset runtime statistics
    pub fn reset_stats(&mut self) {
        self.stats = RuntimeStats::default();
    }
}

impl Default for RuntimeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::lexer::SchemeNumber;
    
    #[test]
    fn test_runtime_executor_creation() {
        let executor = RuntimeExecutor::new();
        assert_eq!(executor.optimization_level(), RuntimeOptimizationLevel::Balanced);
        assert_eq!(executor.get_stats().expressions_evaluated, 0);
    }
    
    #[test]
    fn test_optimization_level_setting() {
        let mut executor = RuntimeExecutor::new();
        executor.set_optimization_level(RuntimeOptimizationLevel::Aggressive);
        assert_eq!(executor.optimization_level(), RuntimeOptimizationLevel::Aggressive);
    }
    
    #[test]
    fn test_verification_toggle() {
        let mut executor = RuntimeExecutor::new();
        executor.set_verification_enabled(true);
        assert!(executor.verification_enabled);
        
        executor.set_verification_enabled(false);
        assert!(!executor.verification_enabled);
    }
    
    #[test]
    fn test_basic_arithmetic_simple() {
        let mut executor = RuntimeExecutor::new();
        
        // Test simple addition
        let args = vec![
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3)),
        ];
        
        let result = executor.builtin_add_simple(&args).unwrap();
        match result {
            Value::Number(SchemeNumber::Integer(5)) => {}
            _ => panic!("Expected simple addition to return 5, got {:?}", result),
        }
    }
    
    #[test]
    fn test_stats_tracking() {
        let mut executor = RuntimeExecutor::new();
        
        // Test that statistics are tracked
        let initial_stats = executor.get_stats().clone();
        assert_eq!(initial_stats.expressions_evaluated, 0);
        
        // Reset stats
        executor.reset_stats();
        let reset_stats = executor.get_stats().clone();
        assert_eq!(reset_stats.expressions_evaluated, 0);
    }
    
    #[test]
    fn test_values_equality() {
        let executor = RuntimeExecutor::new();
        
        // Test number equality
        let num1 = Value::Number(SchemeNumber::Integer(42));
        let num2 = Value::Number(SchemeNumber::Integer(42));
        let num3 = Value::Number(SchemeNumber::Integer(24));
        
        assert!(executor.values_equal(&num1, &num2));
        assert!(!executor.values_equal(&num1, &num3));
        
        // Test boolean equality
        let bool1 = Value::Boolean(true);
        let bool2 = Value::Boolean(true);
        let bool3 = Value::Boolean(false);
        
        assert!(executor.values_equal(&bool1, &bool2));
        assert!(!executor.values_equal(&bool1, &bool3));
    }
    
    #[test]
    fn test_placeholder_analysis() {
        let analysis = PlaceholderAnalysis::new();
        
        // Test that placeholder analysis returns conservative values
        assert!(!analysis.is_tail_call_candidate());
        assert!(!analysis.is_hot_path());
        assert!(!analysis.is_loop_candidate());
    }
    
    #[test]
    fn test_semantic_evaluator_integration() {
        use crate::environment::Environment;
        use crate::evaluator::Continuation;
        use std::rc::Rc;
        
        let mut runtime_executor = RuntimeExecutor::new();
        let env = Rc::new(Environment::new());
        
        // Test simple literal evaluation
        let literal_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let result = runtime_executor.eval_optimized(literal_expr, env.clone(), Continuation::Identity);
        
        assert!(result.is_ok());
        if let Ok(Value::Number(SchemeNumber::Integer(42))) = result {
            // Success
        } else {
            panic!("Expected literal evaluation to return 42, got {:?}", result);
        }
    }
    
    #[test]
    fn test_optimization_level_behavior() {
        use crate::environment::Environment;
        use crate::evaluator::Continuation;
        use std::rc::Rc;
        
        let mut runtime_executor = RuntimeExecutor::new();
        let env = Rc::new(Environment::new());
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(123)));
        
        // Test different optimization levels
        let optimization_levels = vec![
            RuntimeOptimizationLevel::None,
            RuntimeOptimizationLevel::Conservative,
            RuntimeOptimizationLevel::Balanced,
            RuntimeOptimizationLevel::Aggressive,
        ];
        
        for level in optimization_levels {
            runtime_executor.set_optimization_level(level.clone());
            let result = runtime_executor.eval_optimized(expr.clone(), env.clone(), Continuation::Identity);
            
            assert!(result.is_ok(), "Optimization level {:?} failed", level);
            if let Ok(Value::Number(SchemeNumber::Integer(123))) = result {
                // Success
            } else {
                panic!("Expected result 123 for optimization level {:?}, got {:?}", level, result);
            }
        }
    }
    
    #[test]
    fn test_runtime_executor_with_semantic_evaluator() {
        use crate::environment::Environment;
        use crate::evaluator::{Continuation, SemanticEvaluator};
        use std::rc::Rc;
        
        let mut runtime_executor = RuntimeExecutor::new();
        let mut semantic_evaluator = SemanticEvaluator::new();
        let env = Rc::new(Environment::new());
        
        // Test that both evaluators produce the same result for simple expressions
        let test_cases = vec![
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            Expr::Literal(Literal::Boolean(true)),
            Expr::Literal(Literal::String("hello".to_string())),
            Expr::Literal(Literal::Nil),
        ];
        
        for expr in test_cases {
            let runtime_result = runtime_executor.eval_optimized(expr.clone(), env.clone(), Continuation::Identity);
            let semantic_result = semantic_evaluator.eval_pure(expr.clone(), env.clone(), Continuation::Identity);
            
            assert!(runtime_result.is_ok(), "Runtime executor failed for {:?}", expr);
            assert!(semantic_result.is_ok(), "Semantic evaluator failed for {:?}", expr);
            
            // For basic literals, both should produce the same result
            match (runtime_result.unwrap(), semantic_result.unwrap()) {
                (Value::Number(a), Value::Number(b)) => assert_eq!(a, b),
                (Value::Boolean(a), Value::Boolean(b)) => assert_eq!(a, b),
                (Value::String(a), Value::String(b)) => assert_eq!(a, b),
                (Value::Nil, Value::Nil) => {},
                (a, b) => panic!("Results differ: runtime={:?}, semantic={:?}", a, b),
            }
        }
    }
}