//! Phase 6-D: Tail Call Optimization Tests
//!
//! Tests the tail call optimization system implementation that provides
//! stack-safe recursive function calls and enhanced performance for
//! functional programming patterns.

use lambdust::evaluator::{Evaluator, TailCallOptimizer, TailCallContext, TailCallAnalyzer, OptimizationLevel};
use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_tail_call_context_creation() {
    let context = TailCallContext::new();
    assert!(context.is_tail_position);
    assert!(context.optimization_enabled);
    assert_eq!(context.recursion_depth, 0);
    assert!(context.current_function.is_none());
}

#[test]
fn test_tail_call_context_function_entry() {
    let context = TailCallContext::new();
    let func_context = context.enter_function(Some("factorial".to_string()));
    
    assert!(func_context.is_tail_position);
    assert_eq!(func_context.current_function, Some("factorial".to_string()));
    assert_eq!(func_context.recursion_depth, 0);
    
    // Test recursive call detection
    let recursive_context = func_context.enter_function(Some("factorial".to_string()));
    assert_eq!(recursive_context.recursion_depth, 1);
}

#[test]
fn test_tail_call_context_non_tail_position() {
    let context = TailCallContext::new();
    let non_tail = context.non_tail();
    
    assert!(!non_tail.is_tail_position);
    assert!(non_tail.optimization_enabled);
}

#[test]
fn test_tail_call_context_self_recursive_detection() {
    let mut context = TailCallContext::new();
    context.current_function = Some("factorial".to_string());
    context.is_tail_position = true;
    context.optimization_enabled = true;
    
    assert!(context.is_self_recursive_tail_call("factorial"));
    assert!(!context.is_self_recursive_tail_call("other"));
    
    context.is_tail_position = false;
    assert!(!context.is_self_recursive_tail_call("factorial"));
}

#[test]
fn test_tail_call_analyzer_creation() {
    let analyzer = TailCallAnalyzer::new();
    let stats = analyzer.get_stats();
    
    assert_eq!(stats.tail_calls_detected, 0);
    assert_eq!(stats.tail_calls_optimized, 0);
    assert_eq!(stats.self_recursive_optimized, 0);
}

#[test]
fn test_tail_call_analyzer_function_registration() {
    let mut analyzer = TailCallAnalyzer::new();
    analyzer.register_function("factorial".to_string(), 1);
    
    // Registration should not change stats
    let stats = analyzer.get_stats();
    assert_eq!(stats.tail_calls_detected, 0);
}

#[test]
fn test_tail_call_analysis_simple_function_call() {
    let mut analyzer = TailCallAnalyzer::new();
    let context = TailCallContext::new();
    
    // Create a simple function call: (factorial 5)
    let expr = Expr::List(vec![
        Expr::Variable("factorial".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ]);
    
    let result = analyzer.analyze_tail_calls(&expr, &context).unwrap();
    
    // Should detect tail call since we're in tail position
    assert!(result.optimizations.iter().any(|opt| matches!(opt, lambdust::evaluator::OptimizationHint::TailCall)));
    
    let stats = analyzer.get_stats();
    assert_eq!(stats.tail_calls_detected, 1);
    assert_eq!(stats.tail_calls_optimized, 1);
}

#[test]
fn test_tail_call_analysis_self_recursive_call() {
    let mut analyzer = TailCallAnalyzer::new();
    let mut context = TailCallContext::new();
    context.current_function = Some("factorial".to_string());
    context.is_tail_position = true;
    
    // Create self-recursive call: (factorial (- n 1))
    let expr = Expr::List(vec![
        Expr::Variable("factorial".to_string()),
        Expr::List(vec![
            Expr::Variable("-".to_string()),
            Expr::Variable("n".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        ]),
    ]);
    
    let result = analyzer.analyze_tail_calls(&expr, &context).unwrap();
    
    // Should detect self-recursive tail call
    assert!(result.optimizations.iter().any(|opt| matches!(opt, lambdust::evaluator::OptimizationHint::TailCall)));
    
    let stats = analyzer.get_stats();
    assert_eq!(stats.tail_calls_detected, 1);
    assert_eq!(stats.self_recursive_optimized, 1);
}

#[test]
fn test_tail_call_analysis_if_expression() {
    let mut analyzer = TailCallAnalyzer::new();
    let context = TailCallContext::new();
    
    // Create if expression with tail calls in both branches
    // (if (= n 0) 1 (factorial (- n 1)))
    let expr = Expr::List(vec![
        Expr::Variable("if".to_string()),
        Expr::List(vec![
            Expr::Variable("=".to_string()),
            Expr::Variable("n".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
        ]),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::List(vec![
            Expr::Variable("factorial".to_string()),
            Expr::List(vec![
                Expr::Variable("-".to_string()),
                Expr::Variable("n".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            ]),
        ]),
    ]);
    
    let result = analyzer.analyze_tail_calls(&expr, &context).unwrap();
    
    // Should detect tail call in else branch
    assert!(result.optimizations.iter().any(|opt| matches!(opt, lambdust::evaluator::OptimizationHint::TailCall)));
    
    let stats = analyzer.get_stats();
    assert_eq!(stats.tail_calls_detected, 1);
}

#[test]
fn test_tail_call_analysis_begin_expression() {
    let mut analyzer = TailCallAnalyzer::new();
    let context = TailCallContext::new();
    
    // Create begin expression: (begin (display n) (factorial (- n 1)))
    let expr = Expr::List(vec![
        Expr::Variable("begin".to_string()),
        Expr::List(vec![
            Expr::Variable("display".to_string()),
            Expr::Variable("n".to_string()),
        ]),
        Expr::List(vec![
            Expr::Variable("factorial".to_string()),
            Expr::List(vec![
                Expr::Variable("-".to_string()),
                Expr::Variable("n".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            ]),
        ]),
    ]);
    
    let result = analyzer.analyze_tail_calls(&expr, &context).unwrap();
    
    // Should detect tail call in last expression
    assert!(result.optimizations.iter().any(|opt| matches!(opt, lambdust::evaluator::OptimizationHint::TailCall)));
    
    let stats = analyzer.get_stats();
    assert_eq!(stats.tail_calls_detected, 1);
}

#[test]
fn test_tail_call_analysis_lambda_expression() {
    let mut analyzer = TailCallAnalyzer::new();
    let context = TailCallContext::new();
    
    // Create lambda with tail call: (lambda (n) (factorial n))
    let expr = Expr::List(vec![
        Expr::Variable("lambda".to_string()),
        Expr::List(vec![Expr::Variable("n".to_string())]),
        Expr::List(vec![
            Expr::Variable("factorial".to_string()),
            Expr::Variable("n".to_string()),
        ]),
    ]);
    
    let result = analyzer.analyze_tail_calls(&expr, &context).unwrap();
    
    // Should detect tail call in lambda body
    assert!(result.optimizations.iter().any(|opt| matches!(opt, lambdust::evaluator::OptimizationHint::TailCall)));
}

#[test]
fn test_tail_call_optimizer_creation() {
    let optimizer = TailCallOptimizer::new();
    let stats = optimizer.get_stats();
    
    assert_eq!(stats.optimizations_applied, 0);
    assert_eq!(stats.self_recursive_optimizations, 0);
    assert_eq!(stats.stack_frames_saved, 0);
    assert_eq!(stats.cache_hits, 0);
    assert_eq!(stats.cache_misses, 0);
}

#[test]
fn test_tail_call_optimization_cache() {
    let mut optimizer = TailCallOptimizer::new();
    let context = TailCallContext::new();
    let mut evaluator = Evaluator::new();
    
    // Create the same function call twice
    let expr = Expr::List(vec![
        Expr::Variable("factorial".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ]);
    
    // First call should be a cache miss
    let result1 = optimizer.optimize_tail_call(&expr, &context, &mut evaluator).unwrap();
    assert!(result1.is_some());
    
    let stats_after_first = optimizer.get_stats();
    assert_eq!(stats_after_first.cache_misses, 1);
    assert_eq!(stats_after_first.cache_hits, 0);
    
    // Second call should be a cache hit
    let result2 = optimizer.optimize_tail_call(&expr, &context, &mut evaluator).unwrap();
    assert!(result2.is_some());
    
    let stats_after_second = optimizer.get_stats();
    assert_eq!(stats_after_second.cache_misses, 1);
    assert_eq!(stats_after_second.cache_hits, 1);
}

#[test]
fn test_optimization_level_determination() {
    let mut optimizer = TailCallOptimizer::new();
    let context = TailCallContext::new();
    let mut evaluator = Evaluator::new();
    
    // Test basic tail call
    let expr = Expr::List(vec![
        Expr::Variable("other".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ]);
    
    let optimization = optimizer.optimize_tail_call(&expr, &context, &mut evaluator).unwrap();
    if let Some(opt) = optimization {
        assert_eq!(opt.optimization_level, OptimizationLevel::Basic);
        assert!(!opt.is_self_recursive);
    }
}

#[test]
fn test_self_recursive_optimization_level() {
    let mut optimizer = TailCallOptimizer::new();
    let mut context = TailCallContext::new();
    context.current_function = Some("factorial".to_string());
    context.is_tail_position = true;
    context.recursion_depth = 5;
    
    let mut evaluator = Evaluator::new();
    
    // Test self-recursive tail call
    let expr = Expr::List(vec![
        Expr::Variable("factorial".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ]);
    
    let optimization = optimizer.optimize_tail_call(&expr, &context, &mut evaluator).unwrap();
    if let Some(opt) = optimization {
        assert_eq!(opt.optimization_level, OptimizationLevel::Advanced);
        assert!(opt.is_self_recursive);
    }
}

#[test]
fn test_deep_recursion_optimization_level() {
    let mut optimizer = TailCallOptimizer::new();
    let mut context = TailCallContext::new();
    context.current_function = Some("factorial".to_string());
    context.is_tail_position = true;
    context.recursion_depth = 15; // Deep recursion
    
    let mut evaluator = Evaluator::new();
    
    // Test deep recursive tail call
    let expr = Expr::List(vec![
        Expr::Variable("factorial".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ]);
    
    let optimization = optimizer.optimize_tail_call(&expr, &context, &mut evaluator).unwrap();
    if let Some(opt) = optimization {
        assert_eq!(opt.optimization_level, OptimizationLevel::Full);
        assert!(opt.is_self_recursive);
    }
}

#[test]
fn test_non_tail_position_no_optimization() {
    let mut optimizer = TailCallOptimizer::new();
    let context = TailCallContext::new().non_tail();
    let mut evaluator = Evaluator::new();
    
    // Function call in non-tail position
    let expr = Expr::List(vec![
        Expr::Variable("factorial".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ]);
    
    let optimization = optimizer.optimize_tail_call(&expr, &context, &mut evaluator).unwrap();
    assert!(optimization.is_none());
}

#[test]
fn test_basic_optimization_application() {
    let mut optimizer = TailCallOptimizer::new();
    let evaluator = Evaluator::new();
    let env = evaluator.global_env.clone();
    
    // Create a basic optimization
    let optimization = lambdust::evaluator::OptimizedTailCall {
        function_name: "test".to_string(),
        arg_strategy: lambdust::evaluator::ArgEvaluationStrategy::Direct,
        is_self_recursive: false,
        optimization_level: OptimizationLevel::Basic,
    };
    
    // Define a simple test function in environment
    env.define("test".to_string(), Value::Procedure(lambdust::value::Procedure::Builtin {
        name: "test".to_string(),
        func: |args| {
            if args.len() == 1 {
                Ok(args[0].clone())
            } else {
                Err(lambdust::error::LambdustError::runtime_error("test requires 1 argument".to_string()))
            }
        },
        arity: Some(1),
    }));
    
    let args = vec![Value::Number(SchemeNumber::Integer(42))];
    let mut evaluator_mut = Evaluator::new();
    let result = optimizer.apply_optimization(&optimization, &args, &mut evaluator_mut, env);
    
    // Basic optimization should work for builtin functions
    assert!(result.is_ok());
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(SchemeNumber::Integer(42)));
    }
    
    let stats = optimizer.get_stats();
    assert_eq!(stats.stack_frames_saved, 1);
}

#[test]
fn test_optimization_statistics_tracking() {
    let mut optimizer = TailCallOptimizer::new();
    let context = TailCallContext::new();
    let mut evaluator = Evaluator::new();
    
    // Perform multiple optimizations
    for i in 0..3 {
        let expr = Expr::List(vec![
            Expr::Variable(format!("func{}", i)),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(i as i64))),
        ]);
        
        let _optimization = optimizer.optimize_tail_call(&expr, &context, &mut evaluator);
    }
    
    let stats = optimizer.get_stats();
    assert_eq!(stats.optimizations_applied, 3);
    assert_eq!(stats.cache_misses, 3); // Each function is different
    
    // Test stats reset
    optimizer.reset_stats();
    let reset_stats = optimizer.get_stats();
    assert_eq!(reset_stats.optimizations_applied, 0);
    assert_eq!(reset_stats.cache_misses, 0);
}

#[test]
fn test_cache_clearing() {
    let mut optimizer = TailCallOptimizer::new();
    let context = TailCallContext::new();
    let mut evaluator = Evaluator::new();
    
    let expr = Expr::List(vec![
        Expr::Variable("factorial".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ]);
    
    // Create cache entry
    let _optimization1 = optimizer.optimize_tail_call(&expr, &context, &mut evaluator).unwrap();
    assert_eq!(optimizer.get_stats().cache_misses, 1);
    
    // Use cache
    let _optimization2 = optimizer.optimize_tail_call(&expr, &context, &mut evaluator).unwrap();
    assert_eq!(optimizer.get_stats().cache_hits, 1);
    
    // Clear cache
    optimizer.clear_cache();
    
    // Should be cache miss again
    let _optimization3 = optimizer.optimize_tail_call(&expr, &context, &mut evaluator).unwrap();
    assert_eq!(optimizer.get_stats().cache_misses, 2);
}

#[test]
fn test_complex_tail_call_analysis() {
    let mut analyzer = TailCallAnalyzer::new();
    analyzer.register_function("factorial".to_string(), 1);
    
    let mut context = TailCallContext::new();
    context.current_function = Some("factorial".to_string());
    context.is_tail_position = true;
    
    // Complex factorial implementation with nested if
    // (if (= n 0) 
    //     1 
    //     (if (= n 1) 
    //         1 
    //         (factorial (- n 1))))
    let expr = Expr::List(vec![
        Expr::Variable("if".to_string()),
        Expr::List(vec![
            Expr::Variable("=".to_string()),
            Expr::Variable("n".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
        ]),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::List(vec![
                Expr::Variable("=".to_string()),
                Expr::Variable("n".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            ]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::List(vec![
                Expr::Variable("factorial".to_string()),
                Expr::List(vec![
                    Expr::Variable("-".to_string()),
                    Expr::Variable("n".to_string()),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                ]),
            ]),
        ]),
    ]);
    
    let result = analyzer.analyze_tail_calls(&expr, &context).unwrap();
    
    // Should detect self-recursive tail call in nested structure
    assert!(result.optimizations.iter().any(|opt| matches!(opt, lambdust::evaluator::OptimizationHint::TailCall)));
    
    let stats = analyzer.get_stats();
    assert!(stats.self_recursive_optimized > 0);
}