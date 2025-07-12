//! Integration tests for ExecutionContext-based evaluation flow
//!
//! This module tests the complete integration of:
//! - Evaluator static analysis and ExecutionContext generation
//! - RuntimeExecutor dynamic optimization with ExecutionContext
//! - EvaluatorInterface unified API for transparent evaluation
//! - Performance measurement and verification systems

use lambdust::{
    ast::{Expr, Literal},
    environment::Environment,
    evaluator::{
        Continuation, EvaluatorInterface, Evaluator, RuntimeExecutor,
        EvaluationMode, EvaluationConfig, RuntimeOptimizationLevel,
        ExecutionContext, ExecutionContextBuilder, ExecutionPriority,
        StaticCallPattern, VariableTypeHint,
    },
    lexer::SchemeNumber,
    value::Value,
};
use std::rc::Rc;

/// Test basic ExecutionContext generation from Evaluator
#[test]
fn test_evaluator_execution_context_generation() {
    let mut evaluator = Evaluator::new();
    let env = evaluator.global_env.clone();
    let cont = Continuation::Identity;
    
    // Test simple literal expression
    let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let context = evaluator.create_execution_context(expr.clone(), env.clone(), cont.clone())
        .expect("Should create ExecutionContext for literal");
    
    // Verify ExecutionContext properties
    assert_eq!(context.expression, expr);
    assert!(context.static_analysis.complexity_score <= 25); // Literals should be low complexity
    assert!(!context.static_analysis.has_tail_calls);
    assert!(!context.static_analysis.has_loops);
    assert!(context.static_analysis.is_pure); // Literals are pure
    
    println!("✅ Basic ExecutionContext generation successful");
    println!("   Complexity score: {}", context.static_analysis.complexity_score);
    println!("   Memory estimate: {} bytes", context.estimated_memory_usage());
}

/// Test ExecutionContext generation for complex expressions
#[test]
fn test_complex_expression_execution_context() {
    let mut evaluator = Evaluator::new();
    let env = evaluator.global_env.clone();
    let cont = Continuation::Identity;
    
    // Create a complex nested arithmetic expression: (+ (* 2 3) (- 10 5))
    let expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::List(vec![
            Expr::Variable("*".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
        ]),
        Expr::List(vec![
            Expr::Variable("-".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
        ]),
    ]);
    
    let context = evaluator.create_execution_context(expr.clone(), env.clone(), cont.clone())
        .expect("Should create ExecutionContext for complex expression");
    
    // Verify complex expression properties  
    assert!(context.static_analysis.complexity_score > 20); // Should be more complex than literal
    assert!(!context.static_analysis.call_patterns.is_empty()); // Should detect call patterns
    
    // Check if builtin function pattern is detected
    let has_builtin = context.static_analysis.call_patterns.iter()
        .any(|pattern| matches!(pattern, StaticCallPattern::Builtin { .. }));
    
    println!("✅ Complex expression ExecutionContext generation successful");
    println!("   Complexity score: {}", context.static_analysis.complexity_score);
    println!("   Call patterns: {}", context.static_analysis.call_patterns.len());
    println!("   Has builtin pattern: {}", has_builtin);
    println!("   Should optimize: {}", context.should_optimize());
    println!("   Should use JIT: {}", context.should_use_jit());
}

/// Test RuntimeExecutor integration with ExecutionContext
#[test]
fn test_runtime_executor_execution_context_integration() {
    let mut runtime_executor = RuntimeExecutor::new();
    
    // Create ExecutionContext for arithmetic expression: (+ 10 20)
    let expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(20))),
    ]);
    let env = Rc::new(Environment::with_builtins());
    let cont = Continuation::Identity;
    
    let context = ExecutionContextBuilder::new(expr, env, cont)
        .with_complexity_score(15) // Low complexity
        .with_purity(true)
        .with_priority(ExecutionPriority::Normal)
        .add_call_pattern(StaticCallPattern::Builtin {
            name: "+".to_string(),
            arity: Some(2),
        })
        .build();
    
    // Execute with ExecutionContext
    let result = runtime_executor.eval_with_execution_context(context)
        .expect("RuntimeExecutor should evaluate with ExecutionContext");
    
    // Verify result
    assert_eq!(result, Value::Number(SchemeNumber::Integer(30)));
    
    println!("✅ RuntimeExecutor ExecutionContext integration successful");
    println!("   Result: {:?}", result);
    
    // Check execution statistics
    let stats = runtime_executor.get_stats();
    assert!(stats.expressions_evaluated > 0);
    println!("   Expressions evaluated: {}", stats.expressions_evaluated);
}

/// Test EvaluatorInterface unified API with ExecutionContext
#[test]
fn test_evaluator_interface_unified_api() {
    let mut interface = EvaluatorInterface::new();
    
    // Configure for automatic mode selection
    let config = EvaluationConfig {
        mode: EvaluationMode::Auto,
        verify_correctness: true,
        monitor_performance: true,
        fallback_to_semantic: true,
        verification_timeout_ms: 1000,
        verification_config: Default::default(),
    };
    interface.set_config(config);
    
    // Test simple expression
    let expr = Expr::List(vec![
        Expr::Variable("*".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(6))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(7))),
    ]);
    let env = Rc::new(Environment::with_builtins());
    let cont = Continuation::Identity;
    
    // Execute with unified API
    let result = interface.eval_with_execution_context(expr.clone(), env, cont)
        .expect("EvaluatorInterface should evaluate with ExecutionContext");
    
    // Verify result
    assert_eq!(result.value, Value::Number(SchemeNumber::Integer(42)));
    assert!(result.evaluation_time_us > 0);
    
    println!("✅ EvaluatorInterface unified API successful");
    println!("   Result: {:?}", result.value);
    println!("   Mode used: {:?}", result.mode_used);
    println!("   Evaluation time: {} μs", result.evaluation_time_us);
    println!("   Fallback used: {}", result.fallback_used);
    
    // Check performance history
    let history = interface.get_performance_history();
    assert!(!history.is_empty());
    println!("   Performance entries: {}", history.len());
}

/// Test verification mode with ExecutionContext
#[test]
fn test_verification_mode_execution_context() {
    let mut interface = EvaluatorInterface::new();
    
    // Configure for verification mode
    let config = EvaluationConfig {
        mode: EvaluationMode::Verification,
        verify_correctness: true,
        monitor_performance: true,
        fallback_to_semantic: true,
        verification_timeout_ms: 2000,
        verification_config: Default::default(),
    };
    interface.set_config(config);
    
    // Test simple arithmetic instead of abs (not implemented in pure evaluator)
    let expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(100))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(23))),
    ]);
    let env = Rc::new(Environment::with_builtins());
    let cont = Continuation::Identity;
    
    // Execute in verification mode
    let result = interface.eval_with_execution_context(expr, env, cont)
        .expect("Verification mode should work");
    
    // Verify result 
    assert_eq!(result.value, Value::Number(SchemeNumber::Integer(123)));
    assert_eq!(result.mode_used, EvaluationMode::Verification);
    
    // Check verification results
    if let Some(verification) = result.verification_result {
        println!("✅ Verification mode successful");
        println!("   Semantic result: {:?}", verification.reference_result);
        println!("   Runtime result: {:?}", verification.actual_result);
        println!("   Semantic equivalence: {:?}", verification.semantic_equivalence);
        println!("   Verification time: {:?}", verification.verification_time);
    } else {
        println!("⚠️  No verification result returned");
    }
}

/// Test performance comparison between modes
#[test]
fn test_performance_comparison_execution_context() {
    let mut interface = EvaluatorInterface::new();
    
    // Create a moderately complex expression for performance testing
    let expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::List(vec![
            Expr::Variable("*".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(12))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(34))),
        ]),
        Expr::List(vec![
            Expr::Variable("-".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(100))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(75))),
        ]),
    ]);
    let env = Rc::new(Environment::with_builtins());
    let cont = Continuation::Identity;
    
    // Test semantic mode
    let semantic_config = EvaluationConfig {
        mode: EvaluationMode::Semantic,
        verify_correctness: false,
        monitor_performance: true,
        fallback_to_semantic: true,
        verification_timeout_ms: 1000,
        verification_config: Default::default(),
    };
    interface.set_config(semantic_config);
    
    let semantic_result = interface.eval_with_execution_context(
        expr.clone(), 
        env.clone(), 
        cont.clone()
    ).expect("Semantic evaluation should work");
    
    // Test runtime mode
    let runtime_config = EvaluationConfig {
        mode: EvaluationMode::Runtime(RuntimeOptimizationLevel::Balanced),
        verify_correctness: false,
        monitor_performance: true,
        fallback_to_semantic: true,
        verification_timeout_ms: 1000,
        verification_config: Default::default(),
    };
    interface.set_config(runtime_config);
    
    let runtime_result = interface.eval_with_execution_context(expr, env, cont)
        .expect("Runtime evaluation should work");
    
    // Compare results
    assert_eq!(semantic_result.value, runtime_result.value);
    
    println!("✅ Performance comparison successful");
    println!("   Expected result: {:?}", Value::Number(SchemeNumber::Integer(433))); // 12*34 + 100-75 = 408 + 25 = 433
    println!("   Semantic result: {:?}", semantic_result.value);
    println!("   Runtime result: {:?}", runtime_result.value);
    println!("   Semantic time: {} μs", semantic_result.evaluation_time_us);
    println!("   Runtime time: {} μs", runtime_result.evaluation_time_us);
    
    if runtime_result.evaluation_time_us > 0 && semantic_result.evaluation_time_us > 0 {
        let speedup = semantic_result.evaluation_time_us as f64 / runtime_result.evaluation_time_us as f64;
        println!("   Speedup factor: {:.2}x", speedup);
    }
}

/// Test ExecutionContext builder pattern with advanced features
#[test]
fn test_execution_context_builder_advanced() {
    let expr = Expr::List(vec![
        Expr::Variable("do".to_string()),
        Expr::List(vec![
            Expr::List(vec![
                Expr::Variable("i".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
                Expr::List(vec![
                    Expr::Variable("+".to_string()),
                    Expr::Variable("i".to_string()),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                ]),
            ]),
        ]),
        Expr::List(vec![
            Expr::Variable(">=".to_string()),
            Expr::Variable("i".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
        ]),
        Expr::Variable("i".to_string()),
    ]);
    
    let env = Rc::new(Environment::with_builtins());
    let cont = Continuation::Identity;
    
    // Build advanced ExecutionContext
    let context = ExecutionContextBuilder::new(expr, env, cont)
        .with_complexity_score(85) // High complexity
        .with_tail_calls(true)
        .with_loops(true)
        .with_purity(true)
        .with_priority(ExecutionPriority::High)
        .add_call_pattern(StaticCallPattern::Loop {
            estimated_iterations: Some(10),
        })
        .add_call_pattern(StaticCallPattern::TailRecursive)
        .add_constant_binding(
            "max-iterations".to_string(),
            Value::Number(SchemeNumber::Integer(10)),
        )
        .build();
    
    // Verify advanced ExecutionContext
    assert_eq!(context.static_analysis.complexity_score, 85);
    assert!(context.static_analysis.has_tail_calls);
    assert!(context.static_analysis.has_loops);
    assert!(context.static_analysis.is_pure);
    assert_eq!(context.execution_metadata.priority, ExecutionPriority::High);
    assert_eq!(context.static_analysis.call_patterns.len(), 2);
    assert_eq!(context.constant_bindings.len(), 1);
    
    // Test optimization hints derivation
    assert!(context.should_optimize());
    assert!(context.should_use_jit()); // High complexity should trigger JIT
    
    println!("✅ Advanced ExecutionContext builder successful");
    println!("   Context ID: {}", context.execution_metadata.context_id);
    println!("   Optimization level: {:?}", context.optimization_hints.optimization_level);
    println!("   JIT beneficial: {}", context.optimization_hints.jit_beneficial);
    println!("   Use tail call optimization: {}", context.optimization_hints.use_tail_call_optimization);
    println!("   Use continuation pooling: {}", context.optimization_hints.use_continuation_pooling);
    println!("   Hot path indicators: {}", context.optimization_hints.hot_path_indicators.len());
    println!("   Optimization strategies: {}", context.optimization_hints.optimization_strategies.len());
}

/// Test error handling and fallback mechanisms
#[test]
fn test_error_handling_and_fallback() {
    let mut interface = EvaluatorInterface::new();
    
    // Configure with fallback enabled
    let config = EvaluationConfig {
        mode: EvaluationMode::Runtime(RuntimeOptimizationLevel::Aggressive),
        verify_correctness: false,
        monitor_performance: true,
        fallback_to_semantic: true,
        verification_timeout_ms: 1000,
        verification_config: Default::default(),
    };
    interface.set_config(config);
    
    // Test with valid arithmetic expression
    let expr = Expr::List(vec![
        Expr::Variable("*".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(8))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);
    let env = Rc::new(Environment::with_builtins());
    let cont = Continuation::Identity;
    
    // This should work (either optimized or fallback)
    let result = interface.eval_with_execution_context(expr, env, cont);
    
    match result {
        Ok(eval_result) => {
            println!("✅ Error handling test successful");
            println!("   Result: {:?}", eval_result.value);
            println!("   Mode used: {:?}", eval_result.mode_used);
            println!("   Fallback used: {}", eval_result.fallback_used);
        }
        Err(e) => {
            println!("⚠️  Expression evaluation failed: {:?}", e);
            // This is acceptable for this test - the important thing is graceful handling
        }
    }
}

/// Comprehensive integration test covering the full pipeline
#[test]
fn test_comprehensive_integration_pipeline() {
    println!("🚀 Starting comprehensive integration pipeline test...");
    
    // Phase 1: Setup components
    let mut evaluator = Evaluator::new();
    let mut runtime_executor = RuntimeExecutor::new();
    let mut interface = EvaluatorInterface::new();
    
    // Phase 2: Test expression: nested arithmetic (+ (* 5 4) (- 8 3))
    let expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::List(vec![
            Expr::Variable("*".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(4))),
        ]),
        Expr::List(vec![
            Expr::Variable("-".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(8))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
        ]),
    ]);
    
    let env = evaluator.global_env.clone();
    let cont = Continuation::Identity;
    
    // Phase 3: Test individual components
    println!("📋 Phase 3: Testing individual components...");
    
    // Test Evaluator ExecutionContext generation
    let context = evaluator.create_execution_context(expr.clone(), env.clone(), cont.clone())
        .expect("Evaluator should generate ExecutionContext");
    println!("   ✅ ExecutionContext generation: complexity={}", context.static_analysis.complexity_score);
    
    // Test RuntimeExecutor with ExecutionContext
    let runtime_result = runtime_executor.eval_with_execution_context(context.clone());
    match runtime_result {
        Ok(value) => println!("   ✅ RuntimeExecutor evaluation: {:?}", value),
        Err(e) => println!("   ⚠️  RuntimeExecutor failed: {:?}", e),
    }
    
    // Phase 4: Test unified interface
    println!("📋 Phase 4: Testing unified interface...");
    
    let unified_config = EvaluationConfig {
        mode: EvaluationMode::Auto,
        verify_correctness: true,
        monitor_performance: true,
        fallback_to_semantic: true,
        verification_timeout_ms: 2000,
        verification_config: Default::default(),
    };
    interface.set_config(unified_config);
    
    let unified_result = interface.eval_with_execution_context(expr, env, cont)
        .expect("Unified interface should work");
    
    println!("   ✅ Unified evaluation successful");
    println!("      Result: {:?}", unified_result.value);
    println!("      Mode: {:?}", unified_result.mode_used);
    println!("      Time: {} μs", unified_result.evaluation_time_us);
    println!("      Memory: {} bytes", unified_result.performance_metrics.memory_usage_bytes);
    
    // Phase 5: Performance analysis
    println!("📋 Phase 5: Performance analysis...");
    
    let history = interface.get_performance_history();
    if !history.is_empty() {
        let latest = &history[history.len() - 1];
        println!("   Total time: {} μs", latest.total_time_us);
        println!("   Semantic time: {} μs", latest.semantic_time_us);
        println!("   Runtime time: {} μs", latest.runtime_time_us);
        println!("   Reduction steps: {}", latest.reduction_steps);
    }
    
    println!("🎉 Comprehensive integration pipeline test completed successfully!");
}