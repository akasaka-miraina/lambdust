//! Unit tests for I/O operations (io.rs)
//!
//! Tests the I/O built-in functions including display, write, newline,
//! write-char, and placeholder implementations for read operations.

use lambdust::builtins::io::register_io_functions;
use lambdust::error::LambdustError;
use lambdust::value::Value;
use std::collections::HashMap;

#[test]
fn test_io_functions_registration() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    // Check that all I/O functions are registered
    assert!(builtins.contains_key("display"));
    assert!(builtins.contains_key("newline"));
    assert!(builtins.contains_key("read"));
    assert!(builtins.contains_key("write"));
    assert!(builtins.contains_key("read-char"));
    assert!(builtins.contains_key("peek-char"));
    assert!(builtins.contains_key("write-char"));
    
    // Check that they are all procedures
    for (name, value) in &builtins {
        assert!(value.is_procedure(), "{} should be a procedure", name);
    }
}

#[test]
fn test_display_function() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let display_proc = builtins.get("display").unwrap();
    
    // Test display with string
    if let Value::Procedure(proc) = display_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::String("Hello, World!".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test display with number
    if let Value::Procedure(proc) = display_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(lambdust::lexer::SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test display with boolean
    if let Value::Procedure(proc) = display_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Boolean(true)];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test display with symbol
    if let Value::Procedure(proc) = display_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Symbol("test-symbol".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_display_arity_errors() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let display_proc = builtins.get("display").unwrap();
    
    // Test display with no arguments - should fail
    if let Value::Procedure(proc) = display_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { function, .. }) = result {
                    assert_eq!(function, "<unknown>");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test display with too many arguments - should fail
    if let Value::Procedure(proc) = display_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::String("Hello".to_string()),
                    Value::String("World".to_string()),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { function, .. }) = result {
                    assert_eq!(function, "<unknown>");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_newline_function() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let newline_proc = builtins.get("newline").unwrap();
    
    // Test newline with no arguments (correct usage)
    if let Value::Procedure(proc) = newline_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_newline_arity_errors() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let newline_proc = builtins.get("newline").unwrap();
    
    // Test newline with arguments - should fail
    if let Value::Procedure(proc) = newline_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::String("unexpected".to_string())];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { function, .. }) = result {
                    assert_eq!(function, "<unknown>");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_write_function() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let write_proc = builtins.get("write").unwrap();
    
    // Test write with string
    if let Value::Procedure(proc) = write_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::String("Hello, World!".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test write with number
    if let Value::Procedure(proc) = write_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(lambdust::lexer::SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test write with complex structure (list)
    if let Value::Procedure(proc) = write_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let list_value = Value::cons(
                    Value::Number(lambdust::lexer::SchemeNumber::Integer(1)),
                    Value::cons(
                        Value::Number(lambdust::lexer::SchemeNumber::Integer(2)),
                        Value::Nil
                    )
                );
                let args = vec![list_value];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_write_arity_errors() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let write_proc = builtins.get("write").unwrap();
    
    // Test write with no arguments - should fail
    if let Value::Procedure(proc) = write_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { function, .. }) = result {
                    assert_eq!(function, "<unknown>");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test write with too many arguments - should fail
    if let Value::Procedure(proc) = write_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::String("Hello".to_string()),
                    Value::String("World".to_string()),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { function, .. }) = result {
                    assert_eq!(function, "<unknown>");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_write_char_function() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let write_char_proc = builtins.get("write-char").unwrap();
    
    // Test write-char with character
    if let Value::Procedure(proc) = write_char_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Character('A')];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test write-char with various characters
    if let Value::Procedure(proc) = write_char_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let test_chars = vec!['x', '1', ' ', '\n', '\t'];
                for ch in test_chars {
                    let args = vec![Value::Character(ch)];
                    let result = func(&args);
                    assert!(result.is_ok());
                    assert_eq!(result.unwrap(), Value::Undefined);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_write_char_type_errors() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let write_char_proc = builtins.get("write-char").unwrap();
    
    // Test write-char with non-character types - should fail
    if let Value::Procedure(proc) = write_char_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let invalid_args = vec![
                    Value::String("not a char".to_string()),
                    Value::Number(lambdust::lexer::SchemeNumber::Integer(65)),
                    Value::Boolean(true),
                    Value::Symbol("symbol".to_string()),
                ];
                
                for arg in invalid_args {
                    let args = vec![arg];
                    let result = func(&args);
                    assert!(result.is_err());
                    if let Err(LambdustError::TypeError { message, .. }) = result {
                        assert!(message.contains("write-char"));
                        assert!(message.contains("expected character"));
                    }
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_write_char_arity_errors() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let write_char_proc = builtins.get("write-char").unwrap();
    
    // Test write-char with no arguments - should fail
    if let Value::Procedure(proc) = write_char_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { function, .. }) = result {
                    assert_eq!(function, "<unknown>");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test write-char with too many arguments - should fail
    if let Value::Procedure(proc) = write_char_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Character('A'),
                    Value::Character('B'),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { function, .. }) = result {
                    assert_eq!(function, "<unknown>");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_read_placeholder_implementation() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let read_proc = builtins.get("read").unwrap();
    
    // Test read returns "not yet implemented" error
    if let Value::Procedure(proc) = read_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("read"));
                    assert!(message.contains("not yet implemented"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_read_char_placeholder_implementation() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let read_char_proc = builtins.get("read-char").unwrap();
    
    // Test read-char returns "not yet implemented" error
    if let Value::Procedure(proc) = read_char_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("read-char"));
                    assert!(message.contains("not yet implemented"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_peek_char_placeholder_implementation() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let peek_char_proc = builtins.get("peek-char").unwrap();
    
    // Test peek-char returns "not yet implemented" error
    if let Value::Procedure(proc) = peek_char_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("peek-char"));
                    assert!(message.contains("not yet implemented"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_io_functions_isolation() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    // Test that each I/O function works independently
    let display_proc = builtins.get("display").unwrap();
    let newline_proc = builtins.get("newline").unwrap();
    let write_proc = builtins.get("write").unwrap();
    let write_char_proc = builtins.get("write-char").unwrap();
    
    // Test display
    if let Value::Procedure(proc) = display_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::String("test".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test newline
    if let Value::Procedure(proc) = newline_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_ok());
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test write
    if let Value::Procedure(proc) = write_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(lambdust::lexer::SchemeNumber::Integer(123))];
                let result = func(&args);
                assert!(result.is_ok());
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test write-char
    if let Value::Procedure(proc) = write_char_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Character('!')];
                let result = func(&args);
                assert!(result.is_ok());
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_io_functions_with_complex_data() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let display_proc = builtins.get("display").unwrap();
    let write_proc = builtins.get("write").unwrap();
    
    // Test display with complex nested structure
    if let Value::Procedure(proc) = display_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let nested_list = Value::cons(
                    Value::String("nested".to_string()),
                    Value::cons(
                        Value::Vector(vec![
                            Value::Number(lambdust::lexer::SchemeNumber::Integer(1)),
                            Value::Number(lambdust::lexer::SchemeNumber::Integer(2)),
                        ]),
                        Value::Nil
                    )
                );
                let args = vec![nested_list];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test write with complex nested structure
    if let Value::Procedure(proc) = write_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let nested_structure = Value::cons(
                    Value::Symbol("quote".to_string()),
                    Value::cons(
                        Value::cons(
                            Value::Number(lambdust::lexer::SchemeNumber::Integer(1)),
                            Value::cons(
                                Value::Number(lambdust::lexer::SchemeNumber::Integer(2)),
                                Value::Nil
                            )
                        ),
                        Value::Nil
                    )
                );
                let args = vec![nested_structure];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_io_functions_edge_cases() {
    let mut builtins = HashMap::new();
    register_io_functions(&mut builtins);
    
    let display_proc = builtins.get("display").unwrap();
    let write_proc = builtins.get("write").unwrap();
    
    // Test display with empty string
    if let Value::Procedure(proc) = display_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::String("".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test display with Nil
    if let Value::Procedure(proc) = display_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Nil];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test display with Undefined
    if let Value::Procedure(proc) = display_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Undefined];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test write with zero
    if let Value::Procedure(proc) = write_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(lambdust::lexer::SchemeNumber::Integer(0))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
    
    // Test write with negative number
    if let Value::Procedure(proc) = write_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(lambdust::lexer::SchemeNumber::Integer(-42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}