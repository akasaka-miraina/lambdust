//! Unit tests for error handling functions (error_handling.rs)
//!
//! Tests the error built-in function including message formatting,
//! irritant handling, and various error scenarios.

use lambdust::builtins::error_handling::register_error_functions;
use lambdust::error::LambdustError;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;
use std::collections::HashMap;

#[test]
fn test_error_functions_registration() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    // Check that error function is registered
    assert!(builtins.contains_key("error"));

    // Check that it's a procedure
    let error_func = builtins.get("error").unwrap();
    assert!(error_func.is_procedure());
}

#[test]
fn test_error_function_with_string_message() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with string message
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::String("Test error message".to_string())];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(message, "Test error message");
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_symbol_message() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with symbol message
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Symbol("error-symbol".to_string())];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(message, "error-symbol");
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_other_value_types() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with number message
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(message, "42");
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test error with boolean message
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Boolean(true)];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(message, "#t");
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test error with nil message
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Nil];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(message, "()");
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_single_irritant() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with message and single irritant
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::String("Division by zero".to_string()),
                    Value::Number(SchemeNumber::Integer(0)),
                ];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(message, "Division by zero: 0");
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_multiple_irritants() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with message and multiple irritants
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::String("Invalid arguments".to_string()),
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::String("foo".to_string()),
                    Value::Boolean(false),
                ];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(message, "Invalid arguments: 1, \"foo\", #f");
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_no_arguments() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with no arguments - should fail
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("error: expected at least one argument"));
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_complex_irritants() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with complex data structures as irritants
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let list_irritant = Value::cons(
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::cons(Value::Number(SchemeNumber::Integer(2)), Value::Nil),
                );
                let vector_irritant = Value::Vector(vec![
                    Value::String("a".to_string()),
                    Value::String("b".to_string()),
                ]);

                let args = vec![
                    Value::String("Complex error".to_string()),
                    list_irritant,
                    vector_irritant,
                ];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.starts_with("Complex error: "));
                    assert!(message.contains("(1 2)"));
                    assert!(message.contains("#(\"a\" \"b\")"));
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_message_formatting() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test that irritants are properly separated by commas
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::String("Test".to_string()),
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::Number(SchemeNumber::Integer(2)),
                    Value::Number(SchemeNumber::Integer(3)),
                    Value::Number(SchemeNumber::Integer(4)),
                    Value::Number(SchemeNumber::Integer(5)),
                ];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(message, "Test: 1, 2, 3, 4, 5");
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_empty_string_message() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with empty string message
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::String("".to_string())];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(message, "");
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_whitespace_message() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with whitespace message
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::String("   \n\t  ".to_string())];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(message, "   \n\t  ");
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_unicode_message() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with unicode message
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::String("エラー: 失敗しました".to_string())];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(message, "エラー: 失敗しました");
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_special_characters() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with special characters in message
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::String(
                    "Error: \"quoted\", 'single', (parentheses), [brackets], {braces}".to_string(),
                )];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(
                        message,
                        "Error: \"quoted\", 'single', (parentheses), [brackets], {braces}"
                    );
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_large_message() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with very large message
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let large_message = "x".repeat(10000);
                let args = vec![Value::String(large_message.clone())];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert_eq!(message, large_message);
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_many_irritants() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with many irritants
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let mut args = vec![Value::String("Many irritants".to_string())];
                for i in 0..100 {
                    args.push(Value::Number(SchemeNumber::Integer(i)));
                }
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.starts_with("Many irritants: "));
                    assert!(message.contains("0, 1, 2"));
                    assert!(message.contains("97, 98, 99"));
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_variadic_nature() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test that error function is variadic (can accept any number of arguments >= 1)
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                // Should accept different numbers of arguments
                let test_cases = vec![
                    vec![Value::String("One arg".to_string())],
                    vec![
                        Value::String("Two args".to_string()),
                        Value::Number(SchemeNumber::Integer(1)),
                    ],
                    vec![
                        Value::String("Three args".to_string()),
                        Value::Number(SchemeNumber::Integer(1)),
                        Value::Number(SchemeNumber::Integer(2)),
                    ],
                ];

                for args in test_cases {
                    let result = func(&args);
                    assert!(result.is_err());
                    if let Err(LambdustError::RuntimeError { .. }) = result {
                        // Expected
                    } else {
                        panic!("Expected RuntimeError");
                    }
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_context_information() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test that error includes context information
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::String("Context test".to_string())];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, context }) = result {
                    assert_eq!(message, "Context test");
                    // Context should be present (even if it's unknown)
                    // For builtin functions, we typically have unknown context
                    assert!(context.stack_trace.is_empty() && context.location.start.line == 0);
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_error_function_with_undefined_and_nil_irritants() {
    let mut builtins = HashMap::new();
    register_error_functions(&mut builtins);

    let error_proc = builtins.get("error").unwrap();

    // Test error with undefined and nil irritants
    if let Value::Procedure(proc) = error_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::String("Special values".to_string()),
                    Value::Undefined,
                    Value::Nil,
                ];
                let result = func(&args);

                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.starts_with("Special values: "));
                    assert!(message.contains("#<undefined>"));
                    assert!(message.contains("()"));
                } else {
                    panic!("Expected RuntimeError");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}
