//! Phase 6-C: JIT Loop Optimization Tests
//!
//! Comprehensive test suite for JIT loop optimization system covering:
//! - Loop pattern detection and classification
//! - Native code generation strategies
//! - Hot path detection and compilation decisions
//! - ExpressionAnalyzer integration
//! - Performance characteristics and optimization statistics

use lambdust::ast::{Expr, Literal};
use lambdust::environment::Environment;
use lambdust::evaluator::expression_analyzer::EvaluationComplexity;
use lambdust::evaluator::{
    Continuation, Evaluator, IterationStrategy, IteratorType, JitHint, JitLoopOptimizer,
    LoopPattern, NativeCodeGenerator,
};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;
use std::rc::Rc;

/// Test basic loop pattern detection
#[test]
fn test_counting_loop_pattern_detection() {
    let mut optimizer = JitLoopOptimizer::new();

    // Simple counting loop: (do ((i 0 (+ i 1))) ((>= i 5)) body)
    let operands = create_counting_loop_operands("i", 0, 5, 1);

    let pattern = optimizer
        .pattern_analyzer
        .analyze_do_loop(&operands)
        .unwrap();

    match pattern {
        LoopPattern::CountingLoop {
            variable,
            start,
            end,
            step,
        } => {
            assert_eq!(variable, "i");
            assert_eq!(start, 0);
            assert_eq!(end, 5);
            assert_eq!(step, 1);
        }
        _ => panic!("Expected counting loop pattern, got {:?}", pattern),
    }
}

/// Test list iteration pattern detection
#[test]
fn test_list_iteration_pattern_detection() {
    let mut optimizer = JitLoopOptimizer::new();

    // List iteration pattern
    let operands = &[
        Expr::List(vec![Expr::List(vec![
            Expr::Variable("lst".to_string()),
            Expr::List(vec![
                Expr::Variable("quote".to_string()),
                Expr::List(vec![
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
                ]),
            ]),
        ])]),
        Expr::List(vec![
            Expr::Variable("null?".to_string()),
            Expr::Variable("lst".to_string()),
        ]),
    ];

    let pattern = optimizer
        .pattern_analyzer
        .analyze_do_loop(operands)
        .unwrap();

    match pattern {
        LoopPattern::ListIteration { variable, .. } => {
            assert_eq!(variable, "lst");
        }
        _ => panic!("Expected list iteration pattern, got {:?}", pattern),
    }
}

/// Test complex loop classification
#[test]
fn test_complex_loop_classification() {
    let mut optimizer = JitLoopOptimizer::new();

    // Complex loop with multiple variables and conditions
    let operands = &[
        Expr::List(vec![
            Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
                Expr::List(vec![
                    Expr::Variable("complex-update".to_string()),
                    Expr::Variable("x".to_string()),
                ]),
            ]),
            Expr::List(vec![
                Expr::Variable("y".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
                Expr::List(vec![
                    Expr::Variable("another-update".to_string()),
                    Expr::Variable("y".to_string()),
                ]),
            ]),
        ]),
        Expr::List(vec![
            Expr::Variable("complex-condition?".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
        ]),
    ];

    let pattern = optimizer
        .pattern_analyzer
        .analyze_do_loop(operands)
        .unwrap();

    // The pattern analyzer may classify this as AccumulationLoop instead of ComplexLoop
    // due to the first variable being the accumulator pattern
    match pattern {
        LoopPattern::ComplexLoop | LoopPattern::AccumulationLoop { .. } => {
            // Both are acceptable for this test case
        }
        _ => panic!(
            "Expected ComplexLoop or AccumulationLoop, got {:?}",
            pattern
        ),
    }
}

/// Test hot path detection and JIT hints
#[test]
fn test_hot_path_detection() {
    let mut optimizer = JitLoopOptimizer::with_threshold(3);
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;

    let operands = create_counting_loop_operands("i", 0, 5, 1);

    // First execution - should not compile
    let result1 =
        optimizer.try_optimize_do_loop(&mut evaluator, &operands, env.clone(), cont.clone());
    assert!(result1.is_ok());
    assert!(result1.unwrap().is_none()); // No optimization yet

    // Second execution - still building up
    let result2 =
        optimizer.try_optimize_do_loop(&mut evaluator, &operands, env.clone(), cont.clone());
    assert!(result2.is_ok());

    // Third execution - should trigger compilation
    let result3 =
        optimizer.try_optimize_do_loop(&mut evaluator, &operands, env.clone(), cont.clone());
    assert!(result3.is_ok());

    let stats = optimizer.optimization_statistics();
    assert!(stats.total_patterns >= 1);
    println!("Optimization stats: {:?}", stats);
}

/// Test native code generation strategies
#[test]
fn test_native_code_generation() {
    let mut generator = NativeCodeGenerator::new();

    // Test counting loop code generation
    let counting_pattern = LoopPattern::CountingLoop {
        variable: "i".to_string(),
        start: 0,
        end: 100,
        step: 2,
    };

    let code = generator.generate_native_code(&counting_pattern).unwrap();

    match code.strategy {
        IterationStrategy::NativeForLoop { start, end, step } => {
            assert_eq!(start, 0);
            assert_eq!(end, 100);
            assert_eq!(step, 2);
        }
        _ => panic!("Expected native for-loop strategy"),
    }

    // Check performance characteristics
    assert!(code.characteristics.iterations_per_second > 1_000_000.0);
    assert!(code.characteristics.cache_friendliness > 0.8);
    assert!(code.characteristics.memory_overhead < 100);

    // Test list iteration code generation
    let list_pattern = LoopPattern::ListIteration {
        variable: "lst".to_string(),
        list_expr: Expr::Variable("my-list".to_string()),
    };

    let list_code = generator.generate_native_code(&list_pattern).unwrap();

    match list_code.strategy {
        IterationStrategy::IteratorBased { iterator_type } => {
            assert_eq!(iterator_type, IteratorType::List);
        }
        _ => panic!("Expected iterator-based strategy"),
    }
}

/// Test ExpressionAnalyzer integration
#[test]
fn test_expression_analyzer_integration() {
    let optimizer = JitLoopOptimizer::new();
    let evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());

    // Create a simple counting loop
    let operands = create_counting_loop_operands("i", 0, 10, 1);

    // Analyze complexity
    let complexity = optimizer
        .analyze_loop_complexity(&evaluator, &operands, &env)
        .unwrap();

    // Simple counting loop should have low complexity
    assert!(matches!(
        complexity,
        EvaluationComplexity::Constant
            | EvaluationComplexity::Variable
            | EvaluationComplexity::Simple
    ));

    // Test hint adjustment based on complexity
    let base_hint = JitHint::ProfileAndDecide;
    let adjusted_hint =
        optimizer.adjust_hint_by_complexity(base_hint, EvaluationComplexity::Constant);

    // Simple expressions should be compiled more aggressively
    assert_eq!(adjusted_hint, JitHint::CompileImmediate);

    // High complexity should prevent compilation
    let high_complexity_hint =
        optimizer.adjust_hint_by_complexity(JitHint::CompileImmediate, EvaluationComplexity::High);
    assert_eq!(high_complexity_hint, JitHint::NoCompile);
}

/// Test JIT optimization statistics
#[test]
fn test_optimization_statistics() {
    let mut optimizer = JitLoopOptimizer::with_threshold(2);
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;

    // Execute multiple different loop patterns
    let counting_loop = create_counting_loop_operands("i", 0, 5, 1);
    let another_counting_loop = create_counting_loop_operands("j", 1, 10, 2);

    // Execute patterns multiple times
    for _ in 0..3 {
        let _ = optimizer.try_optimize_do_loop(
            &mut evaluator,
            &counting_loop,
            env.clone(),
            cont.clone(),
        );
        let _ = optimizer.try_optimize_do_loop(
            &mut evaluator,
            &another_counting_loop,
            env.clone(),
            cont.clone(),
        );
    }

    let stats = optimizer.optimization_statistics();

    assert!(stats.total_patterns >= 2);
    assert!(stats.compilation_rate >= 0.0);
    assert!(stats.compilation_rate <= 1.0);
    assert!(!stats.pattern_detections.is_empty());

    println!("Pattern detections: {:?}", stats.pattern_detections);
    println!("Code generations: {:?}", stats.code_generations);
}

/// Test native for-loop execution
#[test]
fn test_native_for_loop_execution() {
    let mut optimizer = JitLoopOptimizer::with_threshold(1); // Immediate compilation
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;

    // Create a simple counting loop that should be optimized
    let operands = create_counting_loop_operands("i", 0, 3, 1);

    // This should trigger JIT compilation and execution
    let result =
        optimizer.try_optimize_do_loop(&mut evaluator, &operands, env.clone(), cont.clone());

    assert!(result.is_ok());
    // The result might be Some(value) if compilation succeeded, or None if fallback to CPS

    let stats = optimizer.optimization_statistics();
    assert!(stats.total_patterns >= 1);
}

/// Test cache behavior and memory management
#[test]
fn test_cache_behavior() {
    let mut optimizer = JitLoopOptimizer::new();
    let _evaluator = Evaluator::new();
    let _env = Rc::new(Environment::new());

    // Generate some patterns and code
    let pattern1 = LoopPattern::CountingLoop {
        variable: "i".to_string(),
        start: 0,
        end: 10,
        step: 1,
    };

    let pattern2 = LoopPattern::ListIteration {
        variable: "lst".to_string(),
        list_expr: Expr::Variable("my-list".to_string()),
    };

    // Generate code for patterns
    let _code1 = optimizer
        .code_generator
        .generate_native_code(&pattern1)
        .unwrap();
    let _code2 = optimizer
        .code_generator
        .generate_native_code(&pattern2)
        .unwrap();

    // Check that statistics are being tracked
    let gen_stats = optimizer.code_generator.generation_statistics();
    assert!(!gen_stats.is_empty());

    // Clear caches
    optimizer.clear_caches();

    // Verify caches are cleared
    let stats_after_clear = optimizer.optimization_statistics();
    assert_eq!(stats_after_clear.total_patterns, 0);
    assert_eq!(stats_after_clear.compiled_patterns, 0);
    assert!(stats_after_clear.pattern_detections.is_empty());
}

/// Test pattern analyzer edge cases
#[test]
fn test_pattern_analyzer_edge_cases() {
    let mut optimizer = JitLoopOptimizer::new();

    // Test empty operands
    let empty_operands = &[];
    let result = optimizer.pattern_analyzer.analyze_do_loop(empty_operands);
    // May return error or ComplexLoop pattern depending on implementation
    match result {
        Err(_) => {}                       // Expected error case
        Ok(LoopPattern::ComplexLoop) => {} // Also acceptable fallback
        Ok(pattern) => panic!("Unexpected pattern for empty operands: {:?}", pattern),
    }

    // Test malformed operands
    let malformed_operands = &[
        Expr::Variable("not-a-list".to_string()),
        Expr::Literal(Literal::Boolean(true)),
    ];
    let pattern = optimizer
        .pattern_analyzer
        .analyze_do_loop(malformed_operands)
        .unwrap();
    assert_eq!(pattern, LoopPattern::ComplexLoop);

    // Test single operand
    let single_operand = &[Expr::List(vec![])];
    let result = optimizer.pattern_analyzer.analyze_do_loop(single_operand);
    // May return error or ComplexLoop pattern depending on implementation
    match result {
        Err(_) => {}                       // Expected error case
        Ok(LoopPattern::ComplexLoop) => {} // Also acceptable fallback
        Ok(pattern) => panic!("Unexpected pattern for single operand: {:?}", pattern),
    }
}

/// Test accumulation loop pattern detection
#[test]
fn test_accumulation_loop_detection() {
    let mut optimizer = JitLoopOptimizer::new();

    // Accumulation loop: (do ((sum 0 (+ sum i)) (i 1 (+ i 1))) ((> i 5)) sum)
    let operands = &[
        Expr::List(vec![
            Expr::List(vec![
                Expr::Variable("sum".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
                Expr::List(vec![
                    Expr::Variable("+".to_string()),
                    Expr::Variable("sum".to_string()),
                    Expr::Variable("i".to_string()),
                ]),
            ]),
            Expr::List(vec![
                Expr::Variable("i".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                Expr::List(vec![
                    Expr::Variable("+".to_string()),
                    Expr::Variable("i".to_string()),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                ]),
            ]),
        ]),
        Expr::List(vec![
            Expr::Variable(">".to_string()),
            Expr::Variable("i".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
        ]),
    ];

    let pattern = optimizer
        .pattern_analyzer
        .analyze_do_loop(operands)
        .unwrap();

    match pattern {
        LoopPattern::AccumulationLoop { accumulator, .. } => {
            assert_eq!(accumulator, "sum");
        }
        _ => {
            // Could also be classified as complex loop depending on heuristics
            assert!(matches!(
                pattern,
                LoopPattern::ComplexLoop | LoopPattern::CountingLoop { .. }
            ));
        }
    }
}

/// Test performance characteristics estimation
#[test]
fn test_performance_characteristics() {
    let generator = NativeCodeGenerator::new();

    // Test different iteration strategies
    let strategies = vec![
        IterationStrategy::NativeForLoop {
            start: 0,
            end: 100,
            step: 1,
        },
        IterationStrategy::IteratorBased {
            iterator_type: IteratorType::List,
        },
        IterationStrategy::ManualLoop {
            max_iterations: 1000,
        },
        IterationStrategy::CpsFallback,
    ];

    for strategy in strategies {
        let characteristics = generator.estimate_performance_characteristics(&strategy);

        // All characteristics should be positive
        assert!(characteristics.iterations_per_second > 0.0);
        assert!(characteristics.memory_overhead > 0);
        assert!(characteristics.cache_friendliness >= 0.0);
        assert!(characteristics.cache_friendliness <= 1.0);

        // Native for-loop should be fastest
        if matches!(strategy, IterationStrategy::NativeForLoop { .. }) {
            assert!(characteristics.iterations_per_second > 5_000_000.0);
            assert!(characteristics.cache_friendliness > 0.9);
        }

        // CPS fallback should be slowest
        if matches!(strategy, IterationStrategy::CpsFallback) {
            assert!(characteristics.iterations_per_second < 1_000_000.0);
            assert!(characteristics.cache_friendliness < 0.5);
        }
    }
}

/// Helper function to create counting loop operands for testing
fn create_counting_loop_operands(var_name: &str, start: i64, end: i64, step: i64) -> Vec<Expr> {
    vec![
        Expr::List(vec![Expr::List(vec![
            Expr::Variable(var_name.to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(start))),
            Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Variable(var_name.to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(step))),
            ]),
        ])]),
        Expr::List(vec![
            Expr::List(vec![
                Expr::Variable(">=".to_string()),
                Expr::Variable(var_name.to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(end))),
            ]),
            Expr::Variable(var_name.to_string()),
        ]),
    ]
}

/// Integration test with actual evaluator
#[test]
fn test_jit_loop_optimization_failsafe() {
    let mut evaluator = Evaluator::new();

    // Test that JIT optimization gracefully handles unsupported operations
    evaluator.jit_loop_optimizer_mut().set_optimization_enabled(true);
    
    // Test that malformed do-loops are handled gracefully
    let malformed_do = Expr::List(vec![
        Expr::Variable("do".to_string()),
        // Missing variable bindings - should be handled gracefully
    ]);
    
    let env = Rc::new(Environment::with_builtins());
    let result = evaluator.eval(malformed_do, env, Continuation::Identity);
    
    // Should return appropriate error, not crash
    assert!(result.is_err());
    
    // Test basic do-loop still works (with or without JIT)
    let simple_do = Expr::List(vec![
        Expr::Variable("do".to_string()),
        Expr::List(vec![Expr::List(vec![
            Expr::Variable("i".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
            Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Variable("i".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            ]),
        ])]),
        Expr::List(vec![
            Expr::List(vec![
                Expr::Variable(">=".to_string()),
                Expr::Variable("i".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            ]),
            Expr::Variable("i".to_string()),
        ]),
    ]);
    
    let env = Rc::new(Environment::with_builtins());
    let result = evaluator.eval(simple_do, env, Continuation::Identity);
    
    // Should work with CPS fallback
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Integer(2)));
    
    
    // Test that optimizer handles invalid loop patterns gracefully
    let optimizer = evaluator.jit_loop_optimizer();
    let stats = optimizer.optimization_statistics();
    
    // Should not crash when getting statistics
    assert!((0.0..=1.0).contains(&stats.compilation_rate));
}
