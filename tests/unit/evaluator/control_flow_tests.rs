//! Unit tests for control flow evaluator functions
//!
//! Tests the individual control flow constructs including do loops, call/cc,
//! promises, multiple values, dynamic-wind, and exception handling.

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

/// Helper function to create a string literal
fn string(s: &str) -> Expr {
    Expr::Literal(Literal::String(s.to_string()))
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
mod promise_tests {
    use super::*;

    #[test]
    fn test_eval_delay_basic() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (delay 42)
        let operands = vec![num(42.0)];
        let result = evaluator.eval_delay(&operands, env, Continuation::Identity);
        
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Promise(_) => {},
            other => panic!("Expected Promise, got {:?}", other),
        }
    }

    #[test]
    fn test_eval_delay_arity_error() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (delay) - no arguments
        let operands = vec![];
        let result = evaluator.eval_delay(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::ArityError { .. }));
    }

    #[test]
    fn test_eval_lazy_basic() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (lazy 42)
        let operands = vec![num(42.0)];
        let result = evaluator.eval_lazy(&operands, env, Continuation::Identity);
        
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Promise(_) => {},
            other => panic!("Expected Promise, got {:?}", other),
        }
    }

    #[test]
    fn test_eval_promise_predicate_with_promise() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // First create a promise
        let promise_operands = vec![num(42.0)];
        let promise_result = evaluator.eval_delay(&promise_operands, env.clone(), Continuation::Identity).unwrap();
        
        // Store the promise in environment
        env.define("test-promise".to_string(), promise_result);
        
        // (promise? test-promise)
        let operands = vec![var("test-promise")];
        let result = evaluator.eval_promise_predicate(&operands, env, Continuation::Identity);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_eval_promise_predicate_with_non_promise() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // Store a number in environment
        env.define("test-number".to_string(), Value::Number(SchemeNumber::Real(42.0)));
        
        // (promise? test-number)
        let operands = vec![var("test-number")];
        let result = evaluator.eval_promise_predicate(&operands, env, Continuation::Identity);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Boolean(false));
    }
}

#[cfg(test)]
mod do_loop_tests {
    use super::*;

    #[test]
    fn test_eval_do_insufficient_args() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (do) - no arguments
        let operands = vec![];
        let result = evaluator.eval_do(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::SyntaxError { .. }));
    }

    #[test]
    fn test_eval_do_minimal_args() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (do () (#t 42)) - minimal valid do loop
        let bindings = list(vec![]);
        let test_clause = list(vec![
            Expr::Literal(Literal::Boolean(true)),
            num(42.0)
        ]);
        let operands = vec![bindings, test_clause];
        
        let result = evaluator.eval_do(&operands, env, Continuation::Identity);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod call_cc_tests {
    use super::*;

    #[test]
    fn test_eval_call_cc_arity_error() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (call/cc) - no arguments
        let operands = vec![];
        let result = evaluator.eval_call_cc(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::ArityError { .. }));
    }

    #[test]
    fn test_eval_call_cc_too_many_args() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (call/cc f g) - too many arguments
        let operands = vec![var("f"), var("g")];
        let result = evaluator.eval_call_cc(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::ArityError { .. }));
    }
}

#[cfg(test)]
mod multi_values_tests {
    use super::*;

    #[test]
    fn test_eval_values_empty() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (values)
        let operands = vec![];
        let result = evaluator.eval_values(&operands, env, Continuation::Identity);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Values(vec![]));
    }

    #[test]
    fn test_eval_values_single() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (values 42)
        let operands = vec![num(42.0)];
        let result = evaluator.eval_values(&operands, env, Continuation::Identity);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Real(42.0)));
    }

    #[test]
    fn test_eval_call_with_values_arity_error() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (call-with-values) - no arguments
        let operands = vec![];
        let result = evaluator.eval_call_with_values(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::ArityError { .. }));
    }
}

#[cfg(test)]
mod exception_tests {
    use super::*;

    #[test]
    fn test_eval_raise_arity_error() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (raise) - no arguments
        let operands = vec![];
        let result = evaluator.eval_raise(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::ArityError { .. }));
    }

    #[test]
    fn test_eval_with_exception_handler_arity_error() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (with-exception-handler) - no arguments
        let operands = vec![];
        let result = evaluator.eval_with_exception_handler(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::ArityError { .. }));
    }

    #[test]
    fn test_eval_guard_insufficient_args() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (guard (e)) - missing body
        let operands = vec![list(vec![var("e")])];
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::SyntaxError { .. }));
    }

    #[test]
    fn test_raise_with_string() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (raise "error message") - should result in uncaught exception
        let operands = vec![string("error message")];
        let result = evaluator.eval_raise(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::RuntimeError { .. }));
    }

    #[test]
    fn test_raise_with_symbol() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (raise 'error-symbol) - should result in uncaught exception
        let operands = vec![quote(var("error-symbol"))];
        let result = evaluator.eval_raise(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::RuntimeError { .. }));
    }
}

#[cfg(test)]
mod dynamic_wind_tests {
    use super::*;

    #[test]
    fn test_eval_dynamic_wind_arity_error() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (dynamic-wind) - no arguments
        let operands = vec![];
        let result = evaluator.eval_dynamic_wind(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::ArityError { .. }));
    }

    #[test]
    fn test_eval_dynamic_wind_insufficient_args() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (dynamic-wind before) - insufficient arguments
        let operands = vec![var("before")];
        let result = evaluator.eval_dynamic_wind(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::ArityError { .. }));
    }
}

#[cfg(test)]
mod continuation_tests {
    use super::*;

    #[test]
    fn test_continuation_test_method() {
        // Test Do continuation test() method
        let test_expr = num(42.0);
        let do_cont = Continuation::Do {
            bindings: vec![],
            test: test_expr.clone(),
            result_exprs: vec![],
            body_exprs: vec![],
            env: create_test_env(),
            parent: Box::new(Continuation::Identity),
        };
        
        let extracted_test = do_cont.test();
        match extracted_test {
            Expr::Literal(Literal::Number(SchemeNumber::Real(n))) => assert_eq!(*n, 42.0),
            other => panic!("Expected number literal, got {:?}", other),
        }
    }

    #[test]
    #[should_panic(expected = "test() called on non-Do continuation")]
    fn test_continuation_test_method_panic() {
        let identity_cont = Continuation::Identity;
        let _ = identity_cont.test(); // Should panic
    }
}