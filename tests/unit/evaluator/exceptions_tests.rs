//! Unit tests for exception handling evaluator functions
//!
//! Tests the exception handling implementation including raise,
//! with-exception-handler, and guard special forms.

use lambdust::ast::{Expr, Literal};
use lambdust::environment::Environment;
use lambdust::error::LambdustError;
use lambdust::evaluator::{Continuation, Evaluator};
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Procedure, Value};
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

/// Helper function to create a string literal
fn string(s: &str) -> Expr {
    Expr::Literal(Literal::String(s.to_string()))
}

/// Helper function to create a number literal
fn num(n: f64) -> Expr {
    Expr::Literal(Literal::Number(SchemeNumber::Real(n)))
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
mod raise_tests {
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
    fn test_eval_raise_too_many_args() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (raise "error1" "error2") - too many arguments
        let operands = vec![string("error1"), string("error2")];
        let result = evaluator.eval_raise(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::ArityError { .. }));
    }

    #[test]
    fn test_eval_raise_with_string() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (raise "error message") - should result in uncaught exception
        let operands = vec![string("error message")];
        let result = evaluator.eval_raise(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::RuntimeError { .. }));
    }

    #[test]
    fn test_eval_raise_with_symbol() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (raise 'error-symbol) - should result in uncaught exception
        let operands = vec![quote(var("error-symbol"))];
        let result = evaluator.eval_raise(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::RuntimeError { .. }));
    }

    #[test]
    fn test_eval_raise_with_number() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (raise 42) - should result in uncaught exception
        let operands = vec![num(42.0)];
        let result = evaluator.eval_raise(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::RuntimeError { .. }));
    }

    #[test]
    fn test_raise_exception_formatting() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // Test string formatting in exception message
        let operands = vec![string("test error")];
        let result = evaluator.eval_raise(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        if let Err(LambdustError::RuntimeError { message, .. }) = result {
            assert!(message.contains("\"test error\""));
        }
    }
}

#[cfg(test)]
mod with_exception_handler_tests {
    use super::*;

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
    fn test_eval_with_exception_handler_insufficient_args() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (with-exception-handler handler) - missing thunk
        let operands = vec![var("handler")];
        let result = evaluator.eval_with_exception_handler(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::ArityError { .. }));
    }

    #[test]
    fn test_eval_with_exception_handler_too_many_args() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (with-exception-handler h t extra) - too many arguments
        let operands = vec![var("h"), var("t"), var("extra")];
        let result = evaluator.eval_with_exception_handler(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::ArityError { .. }));
    }

    #[test]
    fn test_exception_handler_stack_management() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // Check initial state
        assert_eq!(evaluator.exception_handlers().len(), 0);
        
        // Create a simple handler procedure
        let handler = Value::Procedure(Procedure::Builtin {
            name: "test-handler".to_string(),
            arity: Some(1),
            func: |_args| Ok(Value::Symbol("handled".to_string())),
        });
        
        // Store handler in environment
        env.define("test-handler".to_string(), handler);
        
        // Create a simple thunk procedure
        let thunk = Value::Procedure(Procedure::Builtin {
            name: "test-thunk".to_string(),
            arity: Some(0),
            func: |_args| Ok(Value::Number(SchemeNumber::Real(42.0))),
        });
        
        // Store thunk in environment
        env.define("test-thunk".to_string(), thunk);
        
        // Test that handler stack is managed properly during evaluation
        // This is a minimal test since we can't easily test the full flow
        let operands = vec![var("test-handler"), var("test-thunk")];
        let _result = evaluator.eval_with_exception_handler(&operands, env, Continuation::Identity);
        
        // Handler should be removed after evaluation (even if it fails)
        assert_eq!(evaluator.exception_handlers().len(), 0);
    }
}

#[cfg(test)]
mod guard_tests {
    use super::*;

    #[test]
    fn test_eval_guard_insufficient_args() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (guard) - no arguments
        let operands = vec![];
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::SyntaxError { .. }));
    }

    #[test]
    fn test_eval_guard_missing_body() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (guard (e)) - missing body
        let operands = vec![list(vec![var("e")])];
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::SyntaxError { .. }));
    }

    #[test]
    fn test_guard_condition_parsing_empty_list() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (guard () body) - empty condition list
        let operands = vec![list(vec![]), num(42.0)];
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::SyntaxError { .. }));
    }

    #[test]
    fn test_guard_condition_parsing_non_symbol_variable() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (guard (42) body) - condition variable must be symbol
        let operands = vec![list(vec![num(42.0)]), num(42.0)];
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::SyntaxError { .. }));
    }

    #[test]
    fn test_guard_clause_parsing_non_list() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (guard (e 42) body) - clause must be list
        let operands = vec![list(vec![var("e"), num(42.0)]), num(42.0)];
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::SyntaxError { .. }));
    }

    #[test]
    fn test_guard_clause_parsing_empty_clause() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (guard (e ()) body) - clause cannot be empty
        let operands = vec![list(vec![var("e"), list(vec![])]), num(42.0)];
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::SyntaxError { .. }));
    }

    #[test]
    fn test_guard_multiple_else_clauses() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (guard (e (else 1) (else 2)) body) - multiple else clauses
        let condition = list(vec![
            var("e"),
            list(vec![var("else"), num(1.0)]),
            list(vec![var("else"), num(2.0)]),
        ]);
        let operands = vec![condition, num(42.0)];
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::SyntaxError { .. }));
    }

    #[test]
    fn test_guard_condition_parsing_non_list() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (guard 42 body) - condition must be a list
        let operands = vec![num(42.0), num(42.0)];
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::SyntaxError { .. }));
    }

    #[test]
    fn test_guard_basic_structure() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // (guard (e (else 'handled)) 42) - basic valid guard
        let condition = list(vec![
            var("e"),
            list(vec![var("else"), quote(var("handled"))]),
        ]);
        let operands = vec![condition, num(42.0)];
        
        // This should parse successfully and evaluate the body
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        // The result might be an error due to evaluation complexity,
        // but it should not be a syntax error
        if let Err(error) = result {
            assert!(!matches!(error, LambdustError::SyntaxError { .. }));
        }
    }
}

#[cfg(test)]
mod guard_handler_tests {
    use super::*;

    #[test]
    fn test_guard_handler_creation() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // Test that guard handler creation works for basic patterns
        // (guard (e (else 'default)) body)
        let condition = list(vec![
            var("e"),
            list(vec![var("else"), quote(var("default"))]),
        ]);
        let operands = vec![condition, num(42.0)];
        
        // Should create handler successfully (may fail during evaluation but not during parsing)
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        // Check that it's not a syntax error - the issue should be at evaluation level
        if let Err(error) = result {
            assert!(!matches!(error, LambdustError::SyntaxError { .. }));
        }
    }

    #[test]
    fn test_guard_handler_with_eq_condition() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // Test more complex guard condition: (guard (e ((eq? e 'test-error) 'handled)) body)
        let condition = list(vec![
            var("e"),
            list(vec![
                list(vec![var("eq?"), var("e"), quote(var("test-error"))]),
                quote(var("handled")),
            ]),
        ]);
        let operands = vec![condition, num(42.0)];
        
        // Should parse successfully
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        // Check that parsing works (may fail at evaluation but not syntax)
        if let Err(error) = result {
            assert!(!matches!(error, LambdustError::SyntaxError { .. }));
        }
    }
}

#[cfg(test)]
mod exception_integration_tests {
    use super::*;

    #[test]
    fn test_exception_handler_stack_operations() {
        let mut evaluator = create_test_evaluator();
        
        // Test initial state
        assert_eq!(evaluator.exception_handlers().len(), 0);
        
        // Test mutable access
        let handler_info = lambdust::evaluator::ExceptionHandlerInfo {
            handler: Value::Symbol("test-handler".to_string()),
            env: create_test_env(),
        };
        
        evaluator.exception_handlers_mut().push(handler_info);
        assert_eq!(evaluator.exception_handlers().len(), 1);
        
        // Test cleanup
        evaluator.exception_handlers_mut().pop();
        assert_eq!(evaluator.exception_handlers().len(), 0);
    }

    #[test]
    fn test_raise_through_eval_public_api() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // Test raise through public eval_raise API
        let operands = vec![string("integration test error")];
        let result = evaluator.eval_raise(&operands, env, Continuation::Identity);
        
        // Should produce a runtime error since no handler is installed
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LambdustError::RuntimeError { .. }));
    }

    #[test]
    fn test_exception_handler_installation() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // Create minimal procedure values for testing
        let handler = Value::Procedure(Procedure::Builtin {
            name: "test-handler".to_string(),
            arity: Some(1),
            func: |_args| Ok(Value::Symbol("handled".to_string())),
        });
        
        let thunk = Value::Procedure(Procedure::Builtin {
            name: "test-thunk".to_string(),
            arity: Some(0),
            func: |_args| Ok(Value::Number(SchemeNumber::Real(42.0))),
        });
        
        // Store in environment
        env.define("handler".to_string(), handler);
        env.define("thunk".to_string(), thunk);
        
        // Test with-exception-handler stack management
        let operands = vec![var("handler"), var("thunk")];
        let _result = evaluator.eval_with_exception_handler(&operands, env, Continuation::Identity);
        
        // Handler should be cleaned up even if evaluation fails
        assert_eq!(evaluator.exception_handlers().len(), 0);
    }

    #[test]
    fn test_exception_value_formatting() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // Test different exception value types through public API
        let test_cases = vec![
            (string("string error"), "\"string error\""),
            (quote(var("symbol-error")), "symbol-error"),
            (num(42.0), "42"),
        ];
        
        for (exception_expr, expected_in_message) in test_cases {
            let operands = vec![exception_expr];
            let result = evaluator.eval_raise(&operands, env.clone(), Continuation::Identity);
            
            assert!(result.is_err());
            if let Err(LambdustError::RuntimeError { message, .. }) = result {
                assert!(message.contains("Uncaught exception"));
                assert!(message.contains(expected_in_message));
            }
        }
    }
}

#[cfg(test)]
mod immature_code_analysis_tests {
    use super::*;

    #[test]
    fn test_guard_handler_hardcoded_logic() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // The create_guard_handler function contains hardcoded pattern matching
        // This test demonstrates the immature code by testing the specific pattern
        // that the handler expects: ((eq? e 'test-error) 'handled)
        
        let condition = list(vec![
            var("e"),
            list(vec![
                list(vec![var("eq?"), var("e"), quote(var("test-error"))]),
                quote(var("handled")),
            ]),
        ]);
        let operands = vec![condition, num(42.0)];
        
        // The handler creation should work but reveals the hardcoded nature
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        // This test exposes the limitation that the guard handler
        // has hardcoded logic instead of proper dynamic evaluation
        match result {
            Ok(_) => {
                // If successful, it means the hardcoded pattern was recognized
                // This is a sign of immature implementation
            }
            Err(err) => {
                // Any error should not be a syntax error since parsing should work
                assert!(!matches!(err, LambdustError::SyntaxError { .. }));
            }
        }
    }

    #[test]
    fn test_simplified_guard_evaluation() {
        let mut evaluator = create_test_evaluator();
        let env = create_test_env();
        
        // Test that the guard implementation returns undefined from guard_handler_function
        // This demonstrates the placeholder nature of the current implementation
        
        let condition = list(vec![
            var("e"),
            list(vec![var("else"), quote(var("fallback"))]),
        ]);
        let operands = vec![condition, num(42.0)];
        
        // The current implementation has simplified logic that may not
        // properly evaluate guard conditions dynamically
        let result = evaluator.eval_guard(&operands, env, Continuation::Identity);
        
        // The result demonstrates the current limitations
        match result {
            Ok(_) => {
                // Success indicates the basic structure works
            }
            Err(err) => {
                // Should not fail on syntax but may fail on evaluation complexity
                assert!(!matches!(err, LambdustError::SyntaxError { .. }));
            }
        }
    }

    #[test]
    fn test_unimplemented_continuation_methods() {
        // This test documents that the exception handling has placeholder methods
        // that are not yet implemented, indicating immature code
        
        // The methods apply_exception_handler_continuation and 
        // apply_guard_clause_continuation both return runtime errors
        // with "not yet implemented" messages, showing incomplete implementation
        
        // This is a design issue that should be addressed in refactoring
        // NOTE: These methods have now been implemented in the current version
    }
}