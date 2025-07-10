//! Comprehensive unit tests for Procedure types
//!
//! Tests the Procedure value type that represents callable functions,
//! including lambda expressions, built-in functions, and continuations.

use lambdust::ast::{Expr, Literal};
use lambdust::environment::Environment;
use lambdust::error::{LambdustError, Result};
use lambdust::evaluator::Continuation as EvaluatorContinuation;
use lambdust::host::HostFunc;
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Continuation, Procedure, StackFrame, Value};
use std::rc::Rc;

/// Helper to create test environment
fn create_test_environment() -> Rc<Environment> {
    let env = Rc::new(Environment::new());
    env.define("x".to_string(), Value::Number(SchemeNumber::Integer(10)));
    env.define("y".to_string(), Value::Number(SchemeNumber::Integer(20)));
    env
}

/// Helper to create test expressions
fn create_test_expressions() -> Vec<Expr> {
    vec![
        Expr::Variable("x".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
        ]),
    ]
}

/// Test builtin function for procedure tests
fn test_builtin_add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Number(SchemeNumber::Integer(0)));
    }
    
    let mut sum = 0;
    for arg in args {
        if let Value::Number(SchemeNumber::Integer(n)) = arg {
            sum += n;
        } else {
            return Err(LambdustError::type_error("Expected integer".to_string()));
        }
    }
    Ok(Value::Number(SchemeNumber::Integer(sum)))
}

/// Test builtin function with specific arity
fn test_builtin_square(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    if let Value::Number(SchemeNumber::Integer(n)) = &args[0] {
        Ok(Value::Number(SchemeNumber::Integer(n * n)))
    } else {
        Err(LambdustError::type_error("Expected integer".to_string()))
    }
}

#[test]
fn test_lambda_procedure_creation() {
    let env = create_test_environment();
    let params = vec!["a".to_string(), "b".to_string()];
    let body = vec![
        Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Variable("a".to_string()),
            Expr::Variable("b".to_string()),
        ])
    ];
    
    let lambda_proc = Procedure::Lambda {
        params: params.clone(),
        variadic: false,
        body: body.clone(),
        closure: env.clone(),
    };
    
    // Verify lambda structure
    if let Procedure::Lambda { params: p, variadic: v, body: b, closure: c } = lambda_proc {
        assert_eq!(p, params);
        assert!(!v);
        assert_eq!(b, body);
        assert!(Rc::ptr_eq(&c, &env));
    } else {
        panic!("Expected Lambda procedure");
    }
}

#[test]
fn test_lambda_procedure_variadic() {
    let env = create_test_environment();
    let params = vec!["first".to_string()];
    let body = vec![Expr::Variable("first".to_string())];
    
    let variadic_proc = Procedure::Lambda {
        params: params.clone(),
        variadic: true,
        body: body.clone(),
        closure: env.clone(),
    };
    
    // Verify variadic lambda
    if let Procedure::Lambda { params: p, variadic: v, .. } = variadic_proc {
        assert_eq!(p, params);
        assert!(v);
    } else {
        panic!("Expected variadic Lambda procedure");
    }
}

#[test]
fn test_lambda_procedure_no_params() {
    let env = create_test_environment();
    let body = vec![Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))];
    
    let no_param_proc = Procedure::Lambda {
        params: vec![],
        variadic: false,
        body: body.clone(),
        closure: env.clone(),
    };
    
    // Verify no-parameter lambda
    if let Procedure::Lambda { params: p, .. } = no_param_proc {
        assert!(p.is_empty());
    } else {
        panic!("Expected no-parameter Lambda procedure");
    }
}

#[test]
fn test_builtin_procedure_creation() {
    let builtin_proc = Procedure::Builtin {
        name: "+".to_string(),
        arity: None, // Variadic
        func: test_builtin_add,
    };
    
    // Verify builtin structure
    if let Procedure::Builtin { name, arity, .. } = builtin_proc {
        assert_eq!(name, "+");
        assert!(arity.is_none());
    } else {
        panic!("Expected Builtin procedure");
    }
}

#[test]
fn test_builtin_procedure_fixed_arity() {
    let builtin_proc = Procedure::Builtin {
        name: "square".to_string(),
        arity: Some(1),
        func: test_builtin_square,
    };
    
    // Verify fixed arity builtin
    if let Procedure::Builtin { name, arity, .. } = builtin_proc {
        assert_eq!(name, "square");
        assert_eq!(arity, Some(1));
    } else {
        panic!("Expected fixed-arity Builtin procedure");
    }
}

#[test]
fn test_host_function_procedure() {
    let host_func: HostFunc = Rc::new(|args: &[Value]| {
        Ok(Value::Number(SchemeNumber::Integer(args.len() as i64)))
    });
    
    let host_proc = Procedure::HostFunction {
        name: "count-args".to_string(),
        arity: None,
        func: host_func,
    };
    
    // Verify host function structure
    if let Procedure::HostFunction { name, arity, .. } = host_proc {
        assert_eq!(name, "count-args");
        assert!(arity.is_none());
    } else {
        panic!("Expected HostFunction procedure");
    }
}

#[test]
fn test_continuation_procedure() {
    let env = create_test_environment();
    let continuation = Continuation {
        stack: vec![StackFrame {
            expr: Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            env: env.clone(),
        }],
        env,
    };
    let cont_proc = Procedure::Continuation {
        continuation: Box::new(continuation),
    };
    
    // Verify continuation procedure
    assert!(matches!(cont_proc, Procedure::Continuation { .. }));
}

#[test]
fn test_captured_continuation_procedure() {
    let eval_continuation = EvaluatorContinuation::Identity;
    let captured_proc = Procedure::CapturedContinuation {
        continuation: Box::new(eval_continuation),
    };
    
    // Verify captured continuation procedure
    assert!(matches!(captured_proc, Procedure::CapturedContinuation { .. }));
}

#[test]
fn test_reusable_continuation_procedure() {
    let env = create_test_environment();
    let eval_continuation = EvaluatorContinuation::Identity;
    
    let reusable_proc = Procedure::ReusableContinuation {
        continuation: Box::new(eval_continuation),
        capture_env: env.clone(),
        reuse_id: 12345,
        is_escaping: true,
    };
    
    // Verify reusable continuation structure
    if let Procedure::ReusableContinuation { 
        capture_env, reuse_id, is_escaping, .. 
    } = reusable_proc {
        assert!(Rc::ptr_eq(&capture_env, &env));
        assert_eq!(reuse_id, 12345);
        assert!(is_escaping);
    } else {
        panic!("Expected ReusableContinuation procedure");
    }
}

#[test]
fn test_procedure_debug_formatting() {
    let env = create_test_environment();
    
    // Test debug formatting for different procedure types
    let lambda_proc = Procedure::Lambda {
        params: vec!["x".to_string()],
        variadic: false,
        body: vec![Expr::Variable("x".to_string())],
        closure: env.clone(),
    };
    
    let builtin_proc = Procedure::Builtin {
        name: "test".to_string(),
        arity: Some(1),
        func: test_builtin_square,
    };
    
    let host_func: HostFunc = Rc::new(|_| Ok(Value::Nil));
    let host_proc = Procedure::HostFunction {
        name: "host-test".to_string(),
        arity: Some(2),
        func: host_func,
    };
    
    // Should format without panicking
    let lambda_debug = format!("{:?}", lambda_proc);
    let builtin_debug = format!("{:?}", builtin_proc);
    let host_debug = format!("{:?}", host_proc);
    
    assert!(lambda_debug.contains("Lambda"));
    assert!(builtin_debug.contains("Builtin"));
    assert!(host_debug.contains("HostFunction"));
}

#[test]
fn test_procedure_equality() {
    let env = create_test_environment();
    
    // Lambda procedures
    let lambda1 = Procedure::Lambda {
        params: vec!["x".to_string()],
        variadic: false,
        body: vec![Expr::Variable("x".to_string())],
        closure: env.clone(),
    };
    
    let lambda2 = Procedure::Lambda {
        params: vec!["x".to_string()],
        variadic: false,
        body: vec![Expr::Variable("x".to_string())],
        closure: env.clone(),
    };
    
    let lambda_different = Procedure::Lambda {
        params: vec!["y".to_string()],
        variadic: false,
        body: vec![Expr::Variable("y".to_string())],
        closure: env.clone(),
    };
    
    // Lambda equality (same structure and environment)
    assert_eq!(lambda1, lambda2);
    assert_ne!(lambda1, lambda_different);
    
    // Builtin procedures
    let builtin1 = Procedure::Builtin {
        name: "test".to_string(),
        arity: Some(1),
        func: test_builtin_square,
    };
    
    let builtin2 = Procedure::Builtin {
        name: "test".to_string(),
        arity: Some(1),
        func: test_builtin_add, // Different function but same name/arity
    };
    
    let builtin_different = Procedure::Builtin {
        name: "different".to_string(),
        arity: Some(1),
        func: test_builtin_square,
    };
    
    // Builtin equality (by name and arity)
    assert_eq!(builtin1, builtin2);
    assert_ne!(builtin1, builtin_different);
    
    // Host function procedures
    let host1 = Procedure::HostFunction {
        name: "host".to_string(),
        arity: Some(2),
        func: Rc::new(|_| Ok(Value::Nil)),
    };
    
    let host2 = Procedure::HostFunction {
        name: "host".to_string(),
        arity: Some(2),
        func: Rc::new(|_| Ok(Value::Boolean(true))), // Different function
    };
    
    // Host function equality (by name and arity)
    assert_eq!(host1, host2);
    
    // Continuation procedures are never equal
    let env1 = create_test_environment();
    let env2 = create_test_environment();
    let cont1 = Procedure::Continuation {
        continuation: Box::new(Continuation {
            stack: vec![],
            env: env1,
        }),
    };
    let cont2 = Procedure::Continuation {
        continuation: Box::new(Continuation {
            stack: vec![],
            env: env2,
        }),
    };
    assert_ne!(cont1, cont2);
    
    // Reusable continuations are equal if same reuse_id
    let reusable1 = Procedure::ReusableContinuation {
        continuation: Box::new(EvaluatorContinuation::Identity),
        capture_env: env.clone(),
        reuse_id: 123,
        is_escaping: true,
    };
    
    let reusable2 = Procedure::ReusableContinuation {
        continuation: Box::new(EvaluatorContinuation::Identity),
        capture_env: env.clone(),
        reuse_id: 123,
        is_escaping: false, // Different is_escaping but same reuse_id
    };
    
    let reusable_different = Procedure::ReusableContinuation {
        continuation: Box::new(EvaluatorContinuation::Identity),
        capture_env: env.clone(),
        reuse_id: 456,
        is_escaping: true,
    };
    
    assert_eq!(reusable1, reusable2);
    assert_ne!(reusable1, reusable_different);
}

#[test]
fn test_value_is_procedure() {
    let env = create_test_environment();
    let proc = Procedure::Lambda {
        params: vec!["x".to_string()],
        variadic: false,
        body: vec![Expr::Variable("x".to_string())],
        closure: env,
    };
    
    let proc_value = Value::Procedure(proc);
    let non_proc_value = Value::Number(SchemeNumber::Integer(42));
    
    assert!(proc_value.is_procedure());
    assert!(!non_proc_value.is_procedure());
}

#[test]
fn test_value_as_procedure() {
    let env = create_test_environment();
    let proc = Procedure::Lambda {
        params: vec!["x".to_string()],
        variadic: false,
        body: vec![Expr::Variable("x".to_string())],
        closure: env,
    };
    
    let proc_value = Value::Procedure(proc.clone());
    let non_proc_value = Value::String("not a procedure".to_string());
    
    // Should extract procedure successfully
    let extracted_proc = proc_value.as_procedure();
    assert!(extracted_proc.is_some());
    assert_eq!(extracted_proc.unwrap(), &proc);
    
    // Should return None for non-procedure
    assert!(non_proc_value.as_procedure().is_none());
}

#[test]
fn test_procedure_clone_behavior() {
    let env = create_test_environment();
    let original_proc = Procedure::Lambda {
        params: vec!["x".to_string(), "y".to_string()],
        variadic: false,
        body: vec![
            Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Variable("x".to_string()),
                Expr::Variable("y".to_string()),
            ])
        ],
        closure: env.clone(),
    };
    
    let cloned_proc = original_proc.clone();
    
    // Should be equal
    assert_eq!(original_proc, cloned_proc);
    
    // Environment should be shared via Rc
    if let (
        Procedure::Lambda { closure: orig_env, .. },
        Procedure::Lambda { closure: clone_env, .. }
    ) = (&original_proc, &cloned_proc) {
        assert!(Rc::ptr_eq(orig_env, clone_env));
    }
}

#[test]
fn test_lambda_with_complex_body() {
    let env = create_test_environment();
    let complex_body = vec![
        Expr::List(vec![
            Expr::Variable("define".to_string()),
            Expr::Variable("temp".to_string()),
            Expr::List(vec![
                Expr::Variable("*".to_string()),
                Expr::Variable("x".to_string()),
                Expr::Variable("x".to_string()),
            ]),
        ]),
        Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Variable("temp".to_string()),
            Expr::Variable("y".to_string()),
        ]),
    ];
    
    let complex_proc = Procedure::Lambda {
        params: vec!["x".to_string(), "y".to_string()],
        variadic: false,
        body: complex_body.clone(),
        closure: env,
    };
    
    // Verify complex body structure
    if let Procedure::Lambda { body, .. } = complex_proc {
        assert_eq!(body.len(), 2);
        assert!(matches!(body[0], Expr::List(_)));
        assert!(matches!(body[1], Expr::List(_)));
    }
}

#[test]
fn test_procedure_memory_efficiency() {
    let env = create_test_environment();
    
    // Create multiple procedures sharing the same environment
    let procedures: Vec<Procedure> = (0..10).map(|i| {
        Procedure::Lambda {
            params: vec![format!("arg{}", i)],
            variadic: false,
            body: vec![Expr::Variable(format!("arg{}", i))],
            closure: env.clone(),
        }
    }).collect();
    
    // All procedures should share the same environment instance
    for proc in procedures {
        if let Procedure::Lambda { closure, .. } = proc {
            assert!(Rc::ptr_eq(&closure, &env));
        }
    }
}

#[test]
fn test_procedure_arity_variants() {
    // Test all arity variants
    let no_arity = Procedure::Builtin {
        name: "variadic".to_string(),
        arity: None,
        func: test_builtin_add,
    };
    
    let zero_arity = Procedure::Builtin {
        name: "nullary".to_string(),
        arity: Some(0),
        func: |_| Ok(Value::Nil),
    };
    
    let unary = Procedure::Builtin {
        name: "unary".to_string(),
        arity: Some(1),
        func: test_builtin_square,
    };
    
    let binary = Procedure::Builtin {
        name: "binary".to_string(),
        arity: Some(2),
        func: |_| Ok(Value::Nil),
    };
    
    // Verify arity settings
    if let Procedure::Builtin { arity, .. } = no_arity { assert!(arity.is_none()); }
    if let Procedure::Builtin { arity, .. } = zero_arity { assert_eq!(arity, Some(0)); }
    if let Procedure::Builtin { arity, .. } = unary { assert_eq!(arity, Some(1)); }
    if let Procedure::Builtin { arity, .. } = binary { assert_eq!(arity, Some(2)); }
}

#[test]
fn test_cross_procedure_type_equality() {
    let env = create_test_environment();
    
    let lambda_proc = Procedure::Lambda {
        params: vec!["x".to_string()],
        variadic: false,
        body: vec![Expr::Variable("x".to_string())],
        closure: env,
    };
    
    let builtin_proc = Procedure::Builtin {
        name: "test".to_string(),
        arity: Some(1),
        func: test_builtin_square,
    };
    
    let host_proc = Procedure::HostFunction {
        name: "host-test".to_string(),
        arity: Some(1),
        func: Rc::new(|_| Ok(Value::Nil)),
    };
    
    // Different procedure types should never be equal
    assert_ne!(lambda_proc, builtin_proc);
    assert_ne!(builtin_proc, host_proc);
    assert_ne!(lambda_proc, host_proc);
}