//! Unit tests for special forms evaluator functions
//!
//! Tests the evaluation of special forms including lambda, if, define,
//! begin, and, or, cond, case, when, unless, etc.

use lambdust::ast::{Expr, Literal};
use lambdust::environment::Environment;
use lambdust::error::LambdustError;
use lambdust::evaluator::{Continuation, Evaluator};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;
use std::rc::Rc;

/// Helper function to create a test evaluator
fn create_test_evaluator() -> Evaluator {
    Evaluator::new()
}

/// Helper function to create test environment
fn create_test_env() -> Rc<Environment> {
    Rc::new(Environment::new())
}

/// Helper function to create a variable expression
fn var(name: &str) -> Expr {
    Expr::Variable(name.to_string())
}

/// Helper function to create a number literal
fn num(n: f64) -> Expr {
    Expr::Literal(Literal::Number(SchemeNumber::Real(n)))
}

/// Helper function to create a boolean literal
fn bool_lit(b: bool) -> Expr {
    Expr::Literal(Literal::Boolean(b))
}

/// Helper function to create a list expression
fn list(exprs: Vec<Expr>) -> Expr {
    Expr::List(exprs)
}

/// Helper function to create a quoted expression
fn quote(expr: Expr) -> Expr {
    Expr::Quote(Box::new(expr))
}

#[cfg(test)]
mod lambda_tests {
    use super::*;

    #[test]
    fn test_eval_lambda_arity_error() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (lambda) - no arguments
        let operands = vec![];
        let result = evaluator.eval_lambda(&operands, env, Continuation::Identity);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LambdustError::SyntaxError { .. }
        ));
    }

    #[test]
    fn test_eval_lambda_basic() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (lambda (x) x)
        let params = list(vec![var("x")]);
        let body = var("x");
        let operands = vec![params, body];
        let result = evaluator.eval_lambda(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        match result.unwrap() {
            Value::Procedure(_) => {}
            other => panic!("Expected Procedure, got {:?}", other),
        }
    }
}

#[cfg(test)]
mod if_tests {
    use super::*;

    #[test]
    fn test_eval_if_arity_error() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (if) - no arguments
        let operands = vec![];
        let result = evaluator.eval_if(&operands, env, Continuation::Identity);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LambdustError::ArityError { .. }
        ));
    }

    #[test]
    fn test_eval_if_basic() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (if #t 42 0)
        let operands = vec![bool_lit(true), num(42.0), num(0.0)];
        let result = evaluator.eval_if(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Real(42.0)));
    }

    #[test]
    fn test_eval_if_false_condition() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (if #f 42 0)
        let operands = vec![bool_lit(false), num(42.0), num(0.0)];
        let result = evaluator.eval_if(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Real(0.0)));
    }

    #[test]
    fn test_eval_if_no_else() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (if #f 42)
        let operands = vec![bool_lit(false), num(42.0)];
        let result = evaluator.eval_if(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Undefined);
    }
}

#[cfg(test)]
mod define_tests {
    use super::*;

    #[test]
    fn test_eval_define_arity_error() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (define) - no arguments
        let operands = vec![];
        let result = evaluator.eval_define(&operands, env, Continuation::Identity);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LambdustError::ArityError { .. }
        ));
    }

    #[test]
    fn test_eval_define_variable() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (define x 42)
        let operands = vec![var("x"), num(42.0)];
        let result = evaluator.eval_define(&operands, env.clone(), Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Undefined);

        // Check that the variable was defined
        let lookup_result = env.get("x");
        assert!(lookup_result.is_some());
        assert_eq!(
            lookup_result.unwrap(),
            Value::Number(SchemeNumber::Real(42.0))
        );
    }

    #[test]
    fn test_eval_define_function() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (define (square x) (* x x))
        let func_sig = list(vec![var("square"), var("x")]);
        let body = list(vec![var("*"), var("x"), var("x")]);
        let operands = vec![func_sig, body];
        let result = evaluator.eval_define(&operands, env.clone(), Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Undefined);

        // Check that the function was defined
        let lookup_result = env.get("square");
        assert!(lookup_result.is_some());
        match lookup_result.unwrap() {
            Value::Procedure(_) => {}
            other => panic!("Expected Procedure, got {:?}", other),
        }
    }
}

#[cfg(test)]
mod begin_tests {
    use super::*;

    #[test]
    fn test_eval_begin_empty() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (begin)
        let operands = vec![];
        let result = evaluator.eval_begin(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Undefined);
    }

    #[test]
    fn test_eval_begin_single() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (begin 42)
        let operands = vec![num(42.0)];
        let result = evaluator.eval_begin(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Real(42.0)));
    }
}

#[cfg(test)]
mod boolean_logic_tests {
    use super::*;

    #[test]
    fn test_eval_and_empty() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (and)
        let operands = vec![];
        let result = evaluator.eval_and(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_eval_and_single_true() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (and #t)
        let operands = vec![bool_lit(true)];
        let result = evaluator.eval_and(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_eval_and_single_false() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (and #f)
        let operands = vec![bool_lit(false)];
        let result = evaluator.eval_and(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_eval_or_empty() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (or)
        let operands = vec![];
        let result = evaluator.eval_or(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_eval_or_single_true() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (or #t)
        let operands = vec![bool_lit(true)];
        let result = evaluator.eval_or(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_eval_or_single_false() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (or #f)
        let operands = vec![bool_lit(false)];
        let result = evaluator.eval_or(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Boolean(false));
    }
}

#[cfg(test)]
mod cond_tests {
    use super::*;

    #[test]
    fn test_eval_cond_empty() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (cond)
        let operands = vec![];
        let result = evaluator.eval_cond(&operands, env, Continuation::Identity);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LambdustError::SyntaxError { .. }
        ));
    }

    #[test]
    fn test_eval_cond_basic() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (cond (#t 42))
        let clause = list(vec![bool_lit(true), num(42.0)]);
        let operands = vec![clause];
        let result = evaluator.eval_cond(&operands, env, Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Real(42.0)));
    }
}

#[cfg(test)]
mod macro_tests {
    // Note: case, when, unless are implemented as macros, not special forms
    // These tests verify that the macro system handles them correctly

    #[test]
    fn test_macro_expansion_placeholder() {
        // Placeholder test for macro functionality
        // TODO: Add comprehensive macro expansion tests when macro system is ready
        // This test currently serves as documentation for future macro testing needs
    }
}

#[cfg(test)]
mod set_tests {
    use super::*;

    #[test]
    fn test_eval_set_arity_error() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (set!) - no arguments
        let operands = vec![];
        let result = evaluator.eval_set(&operands, env, Continuation::Identity);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LambdustError::ArityError { .. }
        ));
    }

    #[test]
    fn test_eval_set_undefined_variable() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // (set! x 42) - x is not defined
        let operands = vec![var("x"), num(42.0)];
        let result = evaluator.eval_set(&operands, env, Continuation::Identity);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LambdustError::UndefinedVariable { .. }
        ));
    }

    #[test]
    fn test_eval_set_basic() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // First define a variable
        env.define("x".to_string(), Value::Number(SchemeNumber::Real(0.0)));

        // (set! x 42)
        let operands = vec![var("x"), num(42.0)];
        let result = evaluator.eval_set(&operands, env.clone(), Continuation::Identity);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Undefined);

        // Check that the variable was updated
        let lookup_result = env.get("x");
        assert!(lookup_result.is_some());
        assert_eq!(
            lookup_result.unwrap(),
            Value::Number(SchemeNumber::Real(42.0))
        );
    }
}

#[cfg(test)]
mod quote_tests {
    use super::*;

    #[test]
    fn test_quote_construct_basic() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // Test quote construct by evaluating a complete expression
        // 'symbol
        let quote_expr = quote(var("symbol"));

        let result = evaluator.eval(quote_expr, env, Continuation::Identity);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Symbol("symbol".to_string()));
    }

    #[test]
    fn test_quote_list() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();

        // Test quote construct with list
        // '(a b c)
        let quote_expr = quote(list(vec![var("a"), var("b"), var("c")]));

        let result = evaluator.eval(quote_expr, env, Continuation::Identity);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Pair(_) => {}
            other => panic!("Expected Pair (list), got {:?}", other),
        }
    }
}
