//! Phase 6-A: Trampoline evaluator tests for stack overflow prevention
//!
//! Tests the trampoline evaluator implementation that resolves the critical
//! stack overflow issue in iterative constructs like do-loops.

use lambdust::evaluator::types::Evaluator;
use lambdust::evaluator::{Continuation, TrampolineEvaluation};
use lambdust::environment::Environment;
use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;
use std::rc::Rc;

#[test]
fn test_trampoline_prevents_stack_overflow() {
    let mut evaluator = Evaluator::new();
    let env = evaluator.global_env.clone();
    
    // Create a simple do-loop that would cause stack overflow in regular CPS evaluator
    // (do ((i 0 (+ i 1))) ((>= i 5) i))
    let do_expr = Expr::List(vec![
        Expr::Variable("do".to_string()),
        // Variable bindings: ((i 0 (+ i 1)))
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
        // Test clause: ((>= i 5) i)
        Expr::List(vec![
            Expr::List(vec![
                Expr::Variable(">=".to_string()),
                Expr::Variable("i".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
            ]),
            Expr::Variable("i".to_string()),
        ]),
    ]);
    
    // This should complete without stack overflow using trampoline evaluation
    let result = evaluator.eval_trampoline(do_expr, env, Continuation::Identity);
    
    // Should succeed without stack overflow
    assert!(result.is_ok(), "Trampoline evaluation should prevent stack overflow");
}

#[test]
fn test_trampoline_basic_expressions() {
    let mut evaluator = Evaluator::new();
    let env = evaluator.global_env.clone();
    
    // Test constant evaluation
    let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let result = evaluator.eval_trampoline(expr, env.clone(), Continuation::Identity).unwrap();
    assert_eq!(result, Value::from(42i64));
    
    // Test string constant
    let expr = Expr::Literal(Literal::String("hello".to_string()));
    let result = evaluator.eval_trampoline(expr, env.clone(), Continuation::Identity).unwrap();
    assert_eq!(result, Value::String("hello".to_string()));
    
    // Test boolean constant  
    let expr = Expr::Literal(Literal::Boolean(true));
    let result = evaluator.eval_trampoline(expr, env, Continuation::Identity).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_trampoline_variable_lookup() {
    let mut evaluator = Evaluator::new();
    
    // Create environment with variable
    let mut test_env = Environment::new();
    test_env.define("x".to_string(), Value::from(100i64));
    let env = Rc::new(test_env.extend());
    
    // Test variable lookup
    let expr = Expr::Variable("x".to_string());
    let result = evaluator.eval_trampoline(expr, env, Continuation::Identity).unwrap();
    assert_eq!(result, Value::from(100i64));
}

#[test]
fn test_trampoline_empty_list() {
    let mut evaluator = Evaluator::new();
    let env = evaluator.global_env.clone();
    
    // Test empty list
    let expr = Expr::List(vec![]);
    let result = evaluator.eval_trampoline(expr, env, Continuation::Identity).unwrap();
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_trampoline_bounded_iterations() {
    let mut evaluator = Evaluator::new();
    let env = evaluator.global_env.clone();
    
    // Test that trampoline respects iteration bounds
    // This should complete within reasonable time
    let start_time = std::time::Instant::now();
    
    let do_expr = Expr::List(vec![
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
            Expr::List(vec![
                Expr::Variable(">=".to_string()),
                Expr::Variable("i".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
            ]),
            Expr::Variable("i".to_string()),
        ]),
    ]);
    
    let result = evaluator.eval_trampoline(do_expr, env, Continuation::Identity);
    let elapsed = start_time.elapsed();
    
    // Should complete in reasonable time (< 1 second)
    assert!(elapsed.as_secs() < 1, "Trampoline evaluation should be efficient");
    assert!(result.is_ok(), "Should complete successfully");
}

#[test]
fn test_trampoline_do_loop_termination() {
    let mut evaluator = Evaluator::new();
    let env = evaluator.global_env.clone();
    
    // Test different termination conditions
    
    // Test 1: Simple counting loop
    let do_expr1 = Expr::List(vec![
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
            Expr::List(vec![
                Expr::Variable(">=".to_string()),
                Expr::Variable("i".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
            ]),
            Expr::Variable("i".to_string()),
        ]),
    ]);
    
    let result1 = evaluator.eval_trampoline(do_expr1, env.clone(), Continuation::Identity);
    assert!(result1.is_ok(), "First do-loop should terminate correctly");
    
    // Test 2: Different variable name
    let do_expr2 = Expr::List(vec![
        Expr::Variable("do".to_string()),
        Expr::List(vec![
            Expr::List(vec![
                Expr::Variable("counter".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
                Expr::List(vec![
                    Expr::Variable("+".to_string()),
                    Expr::Variable("counter".to_string()),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                ]),
            ]),
        ]),
        Expr::List(vec![
            Expr::List(vec![
                Expr::Variable(">=".to_string()),
                Expr::Variable("counter".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            ]),
            Expr::Variable("counter".to_string()),
        ]),
    ]);
    
    let result2 = evaluator.eval_trampoline(do_expr2, env, Continuation::Identity);
    assert!(result2.is_ok(), "Second do-loop should terminate correctly");
}

#[test]
fn test_trampoline_iteration_count_limit() {
    let mut evaluator = Evaluator::new();
    let env = evaluator.global_env.clone();
    
    // Create a potentially infinite loop to test iteration limits
    let infinite_loop = Expr::List(vec![
        Expr::Variable("do".to_string()),
        Expr::List(vec![
            Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                Expr::Variable("x".to_string()), // x stays the same (no increment)
            ]),
        ]),
        Expr::List(vec![
            Expr::Literal(Literal::Boolean(false)), // Never terminate
            Expr::Variable("x".to_string()),
        ]),
    ]);
    
    let result = evaluator.eval_trampoline(infinite_loop, env, Continuation::Identity);
    
    // Should fail with iteration limit error
    assert!(result.is_err(), "Should detect infinite loop and fail");
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(
            error_msg.contains("maximum iterations"),
            "Should fail with iteration limit error, got: {}",
            error_msg
        );
    }
}

#[test]
fn test_trampoline_memory_efficiency() {
    let mut evaluator = Evaluator::new();
    let env = evaluator.global_env.clone();
    
    // Test that trampoline doesn't use excessive memory
    let initial_memory = evaluator.memory_usage();
    
    // Run a loop that would use stack in regular CPS
    let do_expr = Expr::List(vec![
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
            Expr::List(vec![
                Expr::Variable(">=".to_string()),
                Expr::Variable("i".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
            ]),
            Expr::Variable("i".to_string()),
        ]),
    ]);
    
    let result = evaluator.eval_trampoline(do_expr, env, Continuation::Identity);
    let final_memory = evaluator.memory_usage();
    
    assert!(result.is_ok(), "Loop should complete successfully");
    
    // Memory usage should be bounded (not grow linearly with iteration count)
    let memory_growth = final_memory.saturating_sub(initial_memory);
    
    // Allow for some memory growth but not excessive (< 10KB for a simple loop)
    assert!(
        memory_growth < 10240,
        "Memory growth should be bounded, but grew by {} bytes",
        memory_growth
    );
}