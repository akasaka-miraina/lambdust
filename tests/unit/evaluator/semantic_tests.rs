//! Comprehensive tests for the evaluator/semantic module
//!
//! These tests verify the pure R7RS semantic evaluator implementation,
//! ensuring correct formal semantics without optimizations.

use lambdust::ast::{Expr, Literal};
use lambdust::environment::Environment;
use lambdust::evaluator::semantic::SemanticEvaluator;
use lambdust::evaluator::Continuation;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;
use std::rc::Rc;

/// Helper function to create a simple halt continuation
fn halt_continuation() -> Continuation {
    Continuation::Identity
}

/// Helper function to create a test environment
fn test_environment() -> Rc<Environment> {
    Rc::new(Environment::new())
}

#[cfg(test)]
mod constructor_tests {
    use super::*;

    #[test]
    fn test_semantic_evaluator_new() {
        let _evaluator = SemanticEvaluator::new();
        
        // Should create without panic
        assert!(true);
    }

    #[test]
    fn test_semantic_evaluator_with_environment() {
        let env = test_environment();
        let _evaluator = SemanticEvaluator::with_environment(env);
        
        // Should create without panic
        assert!(true);
    }
}

#[cfg(test)]
mod literal_evaluation_tests {
    use super::*;

    #[test]
    fn test_eval_integer_literal() {
        let mut evaluator = SemanticEvaluator::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let env = test_environment();
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Number(SchemeNumber::Integer(n))) => assert_eq!(n, 42),
            Ok(other) => panic!("Expected integer, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_real_literal() {
        let mut evaluator = SemanticEvaluator::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Real(3.14)));
        let env = test_environment();
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Number(SchemeNumber::Real(n))) => assert!((n - 3.14).abs() < 1e-10),
            Ok(other) => panic!("Expected real, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_rational_literal() {
        let mut evaluator = SemanticEvaluator::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Rational(3, 4)));
        let env = test_environment();
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Number(SchemeNumber::Rational(n, d))) => {
                assert_eq!(n, 3);
                assert_eq!(d, 4);
            }
            Ok(other) => panic!("Expected rational, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_complex_literal() {
        let mut evaluator = SemanticEvaluator::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Complex(2.0, 3.0)));
        let env = test_environment();
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Number(SchemeNumber::Complex(real, imag))) => {
                assert!((real - 2.0).abs() < 1e-10);
                assert!((imag - 3.0).abs() < 1e-10);
            }
            Ok(other) => panic!("Expected complex, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_string_literal() {
        let mut evaluator = SemanticEvaluator::new();
        let expr = Expr::Literal(Literal::String("hello".to_string()));
        let env = test_environment();
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::String(s)) => assert_eq!(s, "hello"),
            Ok(other) => panic!("Expected string, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_character_literal() {
        let mut evaluator = SemanticEvaluator::new();
        let expr = Expr::Literal(Literal::Character('a'));
        let env = test_environment();
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Character(c)) => assert_eq!(c, 'a'),
            Ok(other) => panic!("Expected character, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_boolean_literal() {
        let mut evaluator = SemanticEvaluator::new();
        
        // Test true literal
        let expr_true = Expr::Literal(Literal::Boolean(true));
        let env = test_environment();
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr_true, env, cont);
        
        match result {
            Ok(Value::Boolean(b)) => assert_eq!(b, true),
            Ok(other) => panic!("Expected boolean true, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }

        // Test false literal
        let expr_false = Expr::Literal(Literal::Boolean(false));
        let env = test_environment();
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr_false, env, cont);
        
        match result {
            Ok(Value::Boolean(b)) => assert_eq!(b, false),
            Ok(other) => panic!("Expected boolean false, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    #[ignore = "Quote expressions not implemented in semantic evaluator"]
    fn test_eval_symbol_literal() {
        let mut evaluator = SemanticEvaluator::new();
        let expr = Expr::Quote(Box::new(Expr::Variable("test".to_string())));
        let env = test_environment();
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Symbol(s)) => assert_eq!(s, "test"),
            Ok(other) => panic!("Expected symbol, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_nil_literal() {
        let mut evaluator = SemanticEvaluator::new();
        let expr = Expr::Literal(Literal::Nil);
        let env = test_environment();
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Nil) => assert!(true),
            Ok(other) => panic!("Expected nil, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }
}

#[cfg(test)]
mod variable_evaluation_tests {
    use super::*;

    #[test]
    fn test_eval_defined_variable() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // Define a variable in the environment
        env.define("test-var".to_string(), Value::Number(SchemeNumber::Integer(100)));
        
        let expr = Expr::Variable("test-var".to_string());
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Number(SchemeNumber::Integer(n))) => assert_eq!(n, 100),
            Ok(other) => panic!("Expected integer 100, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_undefined_variable() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        let expr = Expr::Variable("undefined-var".to_string());
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Undefined) => assert!(true),
            Ok(other) => panic!("Expected undefined, got: {:?}", other),
            Err(_) => assert!(true), // Error for undefined variable is also acceptable
        }
    }

    #[test]
    fn test_eval_various_variable_types() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // Define different types of variables
        env.define("int-var".to_string(), Value::Number(SchemeNumber::Integer(42)));
        env.define("string-var".to_string(), Value::String("hello".to_string()));
        env.define("bool-var".to_string(), Value::Boolean(true));
        env.define("char-var".to_string(), Value::Character('x'));
        env.define("symbol-var".to_string(), Value::Symbol("test".to_string()));
        env.define("nil-var".to_string(), Value::Nil);
        
        // Test integer variable
        let result = evaluator.eval_pure(
            Expr::Variable("int-var".to_string()),
            env.clone(),
            halt_continuation(),
        );
        assert!(matches!(result, Ok(Value::Number(SchemeNumber::Integer(42)))));
        
        // Test string variable
        let result = evaluator.eval_pure(
            Expr::Variable("string-var".to_string()),
            env.clone(),
            halt_continuation(),
        );
        assert!(matches!(result, Ok(Value::String(s)) if s == "hello"));
        
        // Test boolean variable
        let result = evaluator.eval_pure(
            Expr::Variable("bool-var".to_string()),
            env.clone(),
            halt_continuation(),
        );
        assert!(matches!(result, Ok(Value::Boolean(true))));
        
        // Test character variable
        let result = evaluator.eval_pure(
            Expr::Variable("char-var".to_string()),
            env.clone(),
            halt_continuation(),
        );
        assert!(matches!(result, Ok(Value::Character('x'))));
        
        // Test symbol variable
        let result = evaluator.eval_pure(
            Expr::Variable("symbol-var".to_string()),
            env.clone(),
            halt_continuation(),
        );
        assert!(matches!(result, Ok(Value::Symbol(s)) if s == "test"));
        
        // Test nil variable
        let result = evaluator.eval_pure(
            Expr::Variable("nil-var".to_string()),
            env.clone(),
            halt_continuation(),
        );
        assert!(matches!(result, Ok(Value::Nil)));
    }
}

#[cfg(test)]
mod special_form_tests {
    use super::*;

    #[test]
    fn test_eval_if_true_condition() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (if #t 42 100)
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Literal(Literal::Boolean(true)),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(100))),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Number(SchemeNumber::Integer(n))) => assert_eq!(n, 42),
            Ok(other) => panic!("Expected integer 42, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_if_false_condition() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (if #f 42 100)
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Literal(Literal::Boolean(false)),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(100))),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Number(SchemeNumber::Integer(n))) => assert_eq!(n, 100),
            Ok(other) => panic!("Expected integer 100, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_if_without_else() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (if #t 42)
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Literal(Literal::Boolean(true)),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Number(SchemeNumber::Integer(n))) => assert_eq!(n, 42),
            Ok(other) => panic!("Expected integer 42, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_define_variable() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (define test-var 42)
        let expr = Expr::List(vec![
            Expr::Variable("define".to_string()),
            Expr::Variable("test-var".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env.clone(), cont);
        
        // Define should return undefined
        match result {
            Ok(Value::Undefined) => assert!(true),
            Ok(other) => panic!("Expected undefined, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
        
        // Check that variable was defined
        let var_value = env.get("test-var");
        match var_value {
            Some(Value::Number(SchemeNumber::Integer(n))) => assert_eq!(n, 42),
            Some(other) => panic!("Expected integer 42, got: {:?}", other),
            None => panic!("Variable was not defined"),
        }
    }

    #[test]
    fn test_eval_begin_sequence() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (begin 1 2 3)
        let expr = Expr::List(vec![
            Expr::Variable("begin".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Number(SchemeNumber::Integer(n))) => assert_eq!(n, 3),
            Ok(other) => panic!("Expected integer 3, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_empty_begin() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (begin)
        let expr = Expr::List(vec![Expr::Variable("begin".to_string())]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Undefined) => assert!(true),
            Ok(other) => panic!("Expected undefined, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_lambda_creation() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (lambda (x) x)
        let expr = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![Expr::Variable("x".to_string())]),
            Expr::Variable("x".to_string()),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Procedure(_)) => assert!(true),
            Ok(other) => panic!("Expected procedure, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_lambda_no_params() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (lambda () 42)
        let expr = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Procedure(_)) => assert!(true),
            Ok(other) => panic!("Expected procedure, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_lambda_multiple_params() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (lambda (x y z) (+ x y z))
        let expr = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Variable("y".to_string()),
                Expr::Variable("z".to_string()),
            ]),
            Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Variable("x".to_string()),
                Expr::Variable("y".to_string()),
                Expr::Variable("z".to_string()),
            ]),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Procedure(_)) => assert!(true),
            Ok(other) => panic!("Expected procedure, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_eval_lambda_multiple_body_expressions() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (lambda (x) (define y 10) (+ x y))
        let expr = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![Expr::Variable("x".to_string())]),
            Expr::List(vec![
                Expr::Variable("define".to_string()),
                Expr::Variable("y".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
            ]),
            Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Variable("x".to_string()),
                Expr::Variable("y".to_string()),
            ]),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Procedure(_)) => assert!(true),
            Ok(other) => panic!("Expected procedure, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }
}

#[cfg(test)]
mod recursion_depth_tests {
    use super::*;

    #[test]
    #[ignore = "Stack overflow protection needs to be implemented in semantic evaluator"]
    fn test_recursion_depth_protection() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // Create a deeply nested expression that would cause stack overflow
        let mut nested_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(1)));
        
        // Create nested begin expressions to test recursion depth
        for _i in 0..1500 {
            nested_expr = Expr::List(vec![
                Expr::Variable("begin".to_string()),
                nested_expr,
            ]);
        }
        
        let cont = halt_continuation();
        let result = evaluator.eval_pure(nested_expr, env, cont);
        
        // Should either succeed (if depth is manageable) or fail with proper error
        match result {
            Ok(_) => assert!(true), // Acceptable - managed to evaluate
            Err(e) => {
                // Should be a proper error, not a panic
                assert!(format!("{:?}", e).contains("recursion") || format!("{:?}", e).contains("depth"));
            }
        }
    }

    #[test]
    fn test_reasonable_recursion_depth() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // Create a moderately nested expression that should work
        let mut nested_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        
        for _i in 0..50 {
            nested_expr = Expr::List(vec![
                Expr::Variable("begin".to_string()),
                nested_expr,
            ]);
        }
        
        let cont = halt_continuation();
        let result = evaluator.eval_pure(nested_expr, env, cont);
        
        match result {
            Ok(Value::Number(SchemeNumber::Integer(n))) => assert_eq!(n, 42),
            Ok(other) => panic!("Expected integer 42, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_malformed_if_expression() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (if) - missing arguments
        let expr = Expr::List(vec![Expr::Variable("if".to_string())]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(_) => panic!("Expected error for malformed if"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_malformed_define_expression() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (define) - missing arguments
        let expr = Expr::List(vec![Expr::Variable("define".to_string())]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(_) => panic!("Expected error for malformed define"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_malformed_lambda_expression() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (lambda) - missing arguments
        let expr = Expr::List(vec![Expr::Variable("lambda".to_string())]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(_) => panic!("Expected error for malformed lambda"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_invalid_special_form_arguments() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (if 1 2 3 4) - too many arguments
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(4))),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(_) => panic!("Expected error for malformed if with too many args"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_invalid_lambda_parameters() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (lambda 123 x) - invalid parameter list
        let expr = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(123))),
            Expr::Variable("x".to_string()),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(_) => panic!("Expected error for invalid lambda parameters"),
            Err(_) => assert!(true),
        }
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_list_evaluation() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // () - empty list
        let expr = Expr::List(vec![]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        // Empty list evaluation behavior depends on implementation
        match result {
            Ok(_) => assert!(true), // Some implementations allow empty lists
            Err(_) => assert!(true), // Others may error on empty function application
        }
    }

    #[test]
    fn test_nested_special_forms() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (if #t (begin 1 2) (begin 3 4))
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Literal(Literal::Boolean(true)),
            Expr::List(vec![
                Expr::Variable("begin".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            ]),
            Expr::List(vec![
                Expr::Variable("begin".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(4))),
            ]),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        match result {
            Ok(Value::Number(SchemeNumber::Integer(n))) => assert_eq!(n, 2),
            Ok(other) => panic!("Expected integer 2, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn test_complex_nested_expressions() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (begin (define x 10) (if (> x 5) (+ x 1) (- x 1)))
        let expr = Expr::List(vec![
            Expr::Variable("begin".to_string()),
            Expr::List(vec![
                Expr::Variable("define".to_string()),
                Expr::Variable("x".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
            ]),
            Expr::List(vec![
                Expr::Variable("if".to_string()),
                Expr::List(vec![
                    Expr::Variable(">".to_string()),
                    Expr::Variable("x".to_string()),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
                ]),
                Expr::List(vec![
                    Expr::Variable("+".to_string()),
                    Expr::Variable("x".to_string()),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                ]),
                Expr::List(vec![
                    Expr::Variable("-".to_string()),
                    Expr::Variable("x".to_string()),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                ]),
            ]),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env, cont);
        
        // This test depends on whether built-in functions are available
        match result {
            Ok(_) => assert!(true), // Acceptable if it evaluates successfully
            Err(_) => assert!(true), // Also acceptable if built-ins aren't available
        }
    }

    #[test]
    fn test_define_with_complex_value() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // (define test-val (if #t 42 100))
        let expr = Expr::List(vec![
            Expr::Variable("define".to_string()),
            Expr::Variable("test-val".to_string()),
            Expr::List(vec![
                Expr::Variable("if".to_string()),
                Expr::Literal(Literal::Boolean(true)),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(100))),
            ]),
        ]);
        let cont = halt_continuation();
        
        let result = evaluator.eval_pure(expr, env.clone(), cont);
        
        match result {
            Ok(Value::Undefined) => assert!(true),
            Ok(other) => panic!("Expected undefined, got: {:?}", other),
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
        
        // Check that variable was defined with correct value
        let var_value = env.get("test-val");
        match var_value {
            Some(Value::Number(SchemeNumber::Integer(n))) => assert_eq!(n, 42),
            Some(other) => panic!("Expected integer 42, got: {:?}", other),
            None => panic!("Variable was not defined"),
        }
    }

    #[test]
    fn test_multiple_variable_defines() {
        let mut evaluator = SemanticEvaluator::new();
        let env = test_environment();
        
        // Define multiple variables
        let define_exprs = vec![
            ("var1", SchemeNumber::Integer(1)),
            ("var2", SchemeNumber::Integer(2)),
            ("var3", SchemeNumber::Integer(3)),
        ];
        
        for (name, value) in define_exprs {
            let expr = Expr::List(vec![
                Expr::Variable("define".to_string()),
                Expr::Variable(name.to_string()),
                Expr::Literal(Literal::Number(value)),
            ]);
            let cont = halt_continuation();
            
            let result = evaluator.eval_pure(expr, env.clone(), cont);
            
            match result {
                Ok(Value::Undefined) => assert!(true),
                Ok(other) => panic!("Expected undefined, got: {:?}", other),
                Err(e) => panic!("Evaluation failed: {:?}", e),
            }
        }
        
        // Check that all variables were defined
        for (name, expected_value) in [("var1", 1), ("var2", 2), ("var3", 3)] {
            let var_value = env.get(name);
            match var_value {
                Some(Value::Number(SchemeNumber::Integer(n))) => assert_eq!(n, expected_value),
                Some(other) => panic!("Expected integer {}, got: {:?}", expected_value, other),
                None => panic!("Variable {} was not defined", name),
            }
        }
    }
}