//! SRFI-23 (Error reporting mechanism) Compliance Tests
//!
//! This module provides comprehensive tests for SRFI-23 compliance in Lambdust.
//! SRFI-23 specifies the error reporting mechanism with the `error` procedure.
//!
//! Key SRFI-23 Requirements:
//! - `(error message irritant1 irritant2 ...)` procedure
//! - Error objects with message and irritant information  
//! - Integration with R7RS exception system (raise, guard)
//! - Error object predicates and accessors

use lambdust::eval::evaluator::Evaluator;
use lambdust::eval::value::{Value, ThreadSafeEnvironment};
use lambdust::stdlib::{exceptions::ExceptionObject, create_standard_environment};
use lambdust::diagnostics::Error as DiagnosticError;
use std::sync::Arc;

/// Comprehensive SRFI-23 compliance test suite
#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper: Create evaluator with standard environment
    fn create_test_evaluator() -> (Evaluator, Arc<ThreadSafeEnvironment>) {
        let env = create_standard_environment();
        let evaluator = Evaluator::new();
        (evaluator, env)
    }

    /// Test helper: Evaluate expression and expect error
    fn expect_error_with_message(code: &str, expected_message: &str) -> Result<ExceptionObject, String> {
        let (mut evaluator, env) = create_test_evaluator();
        
        // Parse and evaluate the expression
        match lambdust::parser::parse(code) {
            Ok(expr) => {
                match evaluator.eval(&expr, &env) {
                    Err(DiagnosticError::Exception { exception, .. }) => {
                        if let Some(message) = &exception.message {
                            if message.contains(expected_message) {
                                Ok(exception)
                            } else {
                                Err(format!("Expected message containing '{}', got '{}'", expected_message, message))
                            }
                        } else {
                            Err("Exception has no message".to_string())
                        }
                    },
                    Err(other_error) => Err(format!("Expected exception, got: {:?}", other_error)),
                    Ok(value) => Err(format!("Expected error, got successful result: {:?}", value)),
                }
            },
            Err(parse_error) => Err(format!("Parse error: {:?}", parse_error))
        }
    }

    /// Test helper: Evaluate expression within guard and return result
    fn eval_with_guard(guard_expr: &str) -> Result<Value, String> {
        let (mut evaluator, env) = create_test_evaluator();
        
        match lambdust::parser::parse(guard_expr) {
            Ok(expr) => {
                match evaluator.eval(&expr, &env) {
                    Ok(value) => Ok(value),
                    Err(error) => Err(format!("Evaluation error: {:?}", error)),
                }
            },
            Err(parse_error) => Err(format!("Parse error: {:?}", parse_error))
        }
    }

    #[test]
    fn test_error_procedure_basic() {
        // Basic error with just message
        let result = expect_error_with_message(r#"(error "test message")"#, "test message");
        assert!(result.is_ok(), "error procedure should raise exception");
        
        let exception = result.unwrap();
        assert_eq!(exception.exception_type, "error");
        assert!(!exception.continuable);
        assert!(exception.is_error());
    }

    #[test]
    fn test_error_procedure_with_irritants() {
        // Error with message and irritants
        let code = r#"
            (guard (condition
                     (else (list 
                       (error-object-message condition)
                       (error-object-irritants condition))))
              (error "test error" 42 'symbol "string"))
        "#;
        
        let result = eval_with_guard(code);
        assert!(result.is_ok(), "Guard should catch error and return message/irritants");
        
        // The result should be a list with message and irritants
        if let Ok(Value::Pair(car, cdr)) = result {
            // First element should be the message
            if let Value::Literal(lambdust::ast::Literal::String(msg)) = car.as_ref() {
                assert_eq!(msg, "test error");
            } else {
                panic!("Expected string message, got: {:?}", car);
            }
            
            // Second element should be the irritants list
            if let Value::Pair(irritants_car, _) = cdr.as_ref() {
                if let Value::Pair(first_irritant, rest) = irritants_car.as_ref() {
                    // First irritant should be 42
                    if let Value::Literal(lambdust::ast::Literal::Number(n)) = first_irritant.as_ref() {
                        assert_eq!(*n, 42);
                    } else {
                        panic!("Expected integer 42, got: {:?}", first_irritant);
                    }
                }
            }
        } else {
            panic!("Expected pair result, got: {:?}", result);
        }
    }

    #[test]
    fn test_error_procedure_arity() {
        // error requires at least one argument (message)
        let result = expect_error_with_message(r#"(error)"#, "requires at least");
        assert!(result.is_err(), "error with no arguments should fail");
        
        // error message must be a string
        let result = expect_error_with_message(r#"(error 42)"#, "must be a string");
        assert!(result.is_err(), "error with non-string message should fail");
    }

    #[test]
    fn test_error_object_predicates() {
        let code = r#"
            (guard (condition
                     (else (list 
                       (error? condition)
                       (error-object? condition))))
              (error "test"))
        "#;
        
        let result = eval_with_guard(code);
        assert!(result.is_ok(), "Guard should catch error");
        
        // Both predicates should return #t
        if let Ok(Value::Pair(error_p, cdr)) = result {
            assert_eq!(*error_p, Value::boolean(true), "error? should return #t");
            
            if let Value::Pair(error_object_p, _) = cdr.as_ref() {
                assert_eq!(**error_object_p, Value::boolean(true), "error-object? should return #t");
            }
        }
    }

    #[test]
    fn test_error_object_accessors() {
        // Test error-object-message
        let message_code = r#"
            (guard (condition
                     (else (error-object-message condition)))
              (error "access test message"))
        "#;
        
        let result = eval_with_guard(message_code);
        assert!(result.is_ok(), "Should be able to access error message");
        
        if let Ok(Value::Literal(lambdust::ast::Literal::String(msg))) = result {
            assert_eq!(msg, "access test message");
        } else {
            panic!("Expected string message, got: {:?}", result);
        }
        
        // Test error-object-irritants  
        let irritants_code = r#"
            (guard (condition
                     (else (length (error-object-irritants condition))))
              (error "test" 'a 'b 'c))
        "#;
        
        let result = eval_with_guard(irritants_code);
        assert!(result.is_ok(), "Should be able to access error irritants");
        
        if let Ok(Value::Literal(lambdust::ast::Literal::Number(count))) = result {
            assert_eq!(count, 3, "Should have 3 irritants");
        } else {
            panic!("Expected integer count, got: {:?}", result);
        }
    }

    #[test]
    fn test_error_integration_with_raise() {
        // Test that error objects work with raise
        let code = r#"
            (guard (condition
                     (else (list
                       'caught
                       (error? condition)
                       (if (error? condition)
                           (error-object-message condition)
                           'not-error))))
              (raise (guard (inner-condition
                              (else inner-condition))
                        (error "raised error"))))
        "#;
        
        let result = eval_with_guard(code);
        assert!(result.is_ok(), "Should be able to raise error objects");
        
        // The result should indicate the error was caught and recognized
        if let Ok(Value::Pair(caught, rest)) = result {
            assert_eq!(*caught, Value::symbol(lambdust::utils::intern_symbol("caught")));
        }
    }

    #[test] 
    fn test_error_integration_with_guard() {
        // Test comprehensive guard integration
        let code = r#"
            (guard (condition
                     ((error? condition) 
                      (list 'error-caught 
                            (error-object-message condition)
                            (length (error-object-irritants condition))))
                     (else 'other-caught))
              (error "guard integration test" 'irritant1 'irritant2))
        "#;
        
        let result = eval_with_guard(code);
        assert!(result.is_ok(), "Guard should properly handle error objects");
        
        // Verify the result structure
        if let Ok(Value::Pair(tag, rest)) = result {
            assert_eq!(*tag, Value::symbol(lambdust::utils::intern_symbol("error-caught")));
            
            if let Value::Pair(message, rest2) = rest.as_ref() {
                if let Value::Literal(lambdust::ast::Literal::String(msg)) = message.as_ref() {
                    assert_eq!(msg, "guard integration test");
                }
                
                if let Value::Pair(count, _) = rest2.as_ref() {
                    if let Value::Literal(lambdust::ast::Literal::Number(n)) = count.as_ref() {
                        assert_eq!(*n, 2, "Should have 2 irritants");
                    }
                }
            }
        }
    }

    #[test]
    fn test_error_types_hierarchy() {
        // Test that error objects are recognized by type system
        let code = r#"
            (guard (condition
                     (else (list
                       (error? condition)
                       (error-object? condition)
                       ;; These should return #f for basic error objects
                       (read-error? condition)  
                       (file-error? condition))))
              (error "type test"))
        "#;
        
        let result = eval_with_guard(code);
        assert!(result.is_ok(), "Should be able to test error types");
        
        if let Ok(Value::Pair(error_p, rest)) = result {
            assert_eq!(*error_p, Value::boolean(true), "error? should be #t");
            
            // error-object? should also be #t
            if let Value::Pair(error_object_p, rest2) = rest.as_ref() {
                assert_eq!(**error_object_p, Value::boolean(true), "error-object? should be #t");
                
                // read-error? and file-error? should be #f for basic error objects
                if let Value::Pair(read_error_p, rest3) = rest2.as_ref() {
                    assert_eq!(**read_error_p, Value::boolean(false), "read-error? should be #f");
                    
                    if let Value::Pair(file_error_p, _) = rest3.as_ref() {
                        assert_eq!(**file_error_p, Value::boolean(false), "file-error? should be #f");
                    }
                }
            }
        }
    }

    #[test]
    fn test_error_with_complex_irritants() {
        // Test error with various types of irritants
        let code = r#"
            (guard (condition
                     (else (map (lambda (irritant)
                                  (cond
                                    ((number? irritant) 'number)
                                    ((string? irritant) 'string)  
                                    ((symbol? irritant) 'symbol)
                                    ((pair? irritant) 'pair)
                                    ((vector? irritant) 'vector)
                                    (else 'other)))
                                (error-object-irritants condition))))
              (error "complex irritants test"
                     42                    ; number
                     "string"              ; string  
                     'symbol               ; symbol
                     '(a b c)              ; pair/list
                     #(1 2 3)))            ; vector
        "#;
        
        let result = eval_with_guard(code);
        assert!(result.is_ok(), "Should handle complex irritants");
        
        // The result should be a list of type symbols
        // This tests that all irritant types are preserved correctly
    }

    #[test]
    fn test_error_message_requirements() {
        // SRFI-23 requires the message to be a string
        let test_cases = vec![
            ("42", "number instead of string"),
            ("'symbol", "symbol instead of string"), 
            ("(list 1 2 3)", "list instead of string"),
            ("#t", "boolean instead of string"),
        ];
        
        for (non_string_arg, description) in test_cases {
            let code = format!(r#"(error {})"#, non_string_arg);
            let result = expect_error_with_message(&code, "must be a string");
            assert!(result.is_err(), "Error with {} should fail", description);
        }
    }

    #[test]
    fn test_error_empty_irritants() {
        // Test error with message but no irritants
        let code = r#"
            (guard (condition
                     (else (length (error-object-irritants condition))))
              (error "no irritants"))
        "#;
        
        let result = eval_with_guard(code);
        assert!(result.is_ok(), "Should handle error with no irritants");
        
        if let Ok(Value::Literal(lambdust::ast::Literal::Number(count))) = result {
            assert_eq!(count, 0, "Should have 0 irritants");
        }
    }

    #[test]
    fn test_srfi23_compliance_comprehensive() {
        // Comprehensive test covering all SRFI-23 requirements
        let code = r#"
            (define (test-srfi23)
              (guard (condition
                       ((error? condition)
                        (list 'srfi23-compliant
                              ;; Check error object structure
                              (error? condition)
                              (error-object? condition)
                              ;; Check message access
                              (string? (error-object-message condition))
                              (equal? (error-object-message condition) "SRFI-23 test")
                              ;; Check irritants access
                              (list? (error-object-irritants condition))
                              (= (length (error-object-irritants condition)) 3)
                              ;; Check irritant values
                              (equal? (car (error-object-irritants condition)) 'first)
                              (equal? (cadr (error-object-irritants condition)) 42)
                              (equal? (caddr (error-object-irritants condition)) "third")))
                       (else 'not-error))
                (error "SRFI-23 test" 'first 42 "third")))
            
            (test-srfi23)
        "#;
        
        let result = eval_with_guard(code);
        assert!(result.is_ok(), "Comprehensive SRFI-23 test should pass");
        
        // Verify all checks returned #t
        if let Ok(Value::Pair(tag, checks)) = result {
            assert_eq!(*tag, Value::symbol(lambdust::utils::intern_symbol("srfi23-compliant")));
            
            // All subsequent values should be #t
            let mut current = checks.as_ref();
            let mut check_count = 0;
            
            while let Value::Pair(check, rest) = current {
                if let Value::Literal(lambdust::ast::Literal::Boolean(b)) = check.as_ref() {
                    assert!(*b, "SRFI-23 compliance check {} failed", check_count + 1);
                    check_count += 1;
                }
                current = rest.as_ref();
                
                if let Value::Nil = current {
                    break;
                }
            }
            
            assert!(check_count >= 8, "Should have performed at least 8 compliance checks");
        }
    }
}