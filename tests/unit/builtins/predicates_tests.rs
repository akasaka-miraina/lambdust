//! Unit tests for predicate functions (predicates.rs)
//!
//! Tests the built-in predicate functions including type predicates,
//! equality predicates, and logical operations.

use lambdust::builtins::predicates::register_predicate_functions;
use lambdust::error::LambdustError;
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Procedure, Value};
use std::collections::HashMap;

#[test]
fn test_predicate_functions_registration() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_predicate_functions(&mut builtins);

    // Check that all predicate functions are registered
    let predicates = vec![
        "number?", "string?", "symbol?", "boolean?", "procedure?", "char?",
        "vector?", "pair?", "null?", "list?", "eq?", "eqv?", "equal?",
        "not", "eof-object?", "exact?", "inexact?", "integer?", "rational?",
        "real?", "complex?",
    ];

    for pred_name in predicates {
        assert!(builtins.contains_key(pred_name), "Missing predicate: {}", pred_name);
        assert!(builtins.get(pred_name).unwrap().is_procedure(), "{} should be a procedure", pred_name);
    }
}

#[test]
fn test_type_predicates() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_predicate_functions(&mut builtins);

    // Test number?
    let number_pred = builtins.get("number?").unwrap();
    if let Value::Procedure(proc) = number_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::String("not a number".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test string?
    let string_pred = builtins.get("string?").unwrap();
    if let Value::Procedure(proc) = string_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::String("hello".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test symbol?
    let symbol_pred = builtins.get("symbol?").unwrap();
    if let Value::Procedure(proc) = symbol_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Symbol("test-symbol".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::String("not a symbol".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test boolean?
    let boolean_pred = builtins.get("boolean?").unwrap();
    if let Value::Procedure(proc) = boolean_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Boolean(true)];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Boolean(false)];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Number(SchemeNumber::Integer(1))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test char?
    let char_pred = builtins.get("char?").unwrap();
    if let Value::Procedure(proc) = char_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Character('a')];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::String("a".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test vector?
    let vector_pred = builtins.get("vector?").unwrap();
    if let Value::Procedure(proc) = vector_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Vector(vec![Value::Number(SchemeNumber::Integer(1))])];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Nil];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test pair?
    let pair_pred = builtins.get("pair?").unwrap();
    if let Value::Procedure(proc) = pair_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let pair = Value::cons(Value::Number(SchemeNumber::Integer(1)), Value::Nil);
                let args = vec![pair];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Nil];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test null?
    let null_pred = builtins.get("null?").unwrap();
    if let Value::Procedure(proc) = null_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Nil];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Number(SchemeNumber::Integer(0))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test list?
    let list_pred = builtins.get("list?").unwrap();
    if let Value::Procedure(proc) = list_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Nil];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let proper_list = Value::cons(
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::cons(Value::Number(SchemeNumber::Integer(2)), Value::Nil),
                );
                let args = vec![proper_list];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test procedure?
    let procedure_pred = builtins.get("procedure?").unwrap();
    if let Value::Procedure(proc) = procedure_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let builtin_proc = Value::Procedure(Procedure::Builtin {
                    name: "test".to_string(),
                    arity: Some(1),
                    func: |_args| Ok(Value::Number(SchemeNumber::Integer(1))),
                });
                let args = vec![builtin_proc];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_number_type_predicates() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_predicate_functions(&mut builtins);

    // Test integer?
    let integer_pred = builtins.get("integer?").unwrap();
    if let Value::Procedure(proc) = integer_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Number(SchemeNumber::Real(3.5))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));

                let args = vec![Value::String("not a number".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test real?
    let real_pred = builtins.get("real?").unwrap();
    if let Value::Procedure(proc) = real_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Real(3.5))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::String("not a number".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test rational?
    let rational_pred = builtins.get("rational?").unwrap();
    if let Value::Procedure(proc) = rational_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Number(SchemeNumber::Real(3.5))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::String("not a number".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test complex?
    let complex_pred = builtins.get("complex?").unwrap();
    if let Value::Procedure(proc) = complex_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Number(SchemeNumber::Real(3.5))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::String("not a number".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_exactness_predicates() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_predicate_functions(&mut builtins);

    // Test exact?
    let exact_pred = builtins.get("exact?").unwrap();
    if let Value::Procedure(proc) = exact_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Number(SchemeNumber::Real(3.5))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));

                let args = vec![Value::String("not a number".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test inexact?
    let inexact_pred = builtins.get("inexact?").unwrap();
    if let Value::Procedure(proc) = inexact_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Real(3.5))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));

                let args = vec![Value::String("not a number".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_equality_predicates() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_predicate_functions(&mut builtins);

    // Test eq?
    let eq_pred = builtins.get("eq?").unwrap();
    if let Value::Procedure(proc) = eq_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test eq? with symbols (should be eq)
                let args = vec![Value::Symbol("test".to_string()), Value::Symbol("test".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                // Test eq? with different symbols
                let args = vec![Value::Symbol("test".to_string()), Value::Symbol("other".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));

                // Test eq? with numbers (implementation dependent)
                let args = vec![Value::Number(SchemeNumber::Integer(42)), Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                // Result depends on implementation
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test eqv?
    let eqv_pred = builtins.get("eqv?").unwrap();
    if let Value::Procedure(proc) = eqv_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test eqv? with same numbers
                let args = vec![Value::Number(SchemeNumber::Integer(42)), Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                // Test eqv? with different numbers
                let args = vec![Value::Number(SchemeNumber::Integer(42)), Value::Number(SchemeNumber::Integer(43))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));

                // Test eqv? with characters
                let args = vec![Value::Character('a'), Value::Character('a')];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::Character('a'), Value::Character('b')];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test equal?
    let equal_pred = builtins.get("equal?").unwrap();
    if let Value::Procedure(proc) = equal_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test equal? with strings
                let args = vec![Value::String("hello".to_string()), Value::String("hello".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![Value::String("hello".to_string()), Value::String("world".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));

                // Test equal? with lists
                let list1 = Value::cons(
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::cons(Value::Number(SchemeNumber::Integer(2)), Value::Nil),
                );
                let list2 = Value::cons(
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::cons(Value::Number(SchemeNumber::Integer(2)), Value::Nil),
                );
                let args = vec![list1, list2];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_logical_not() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_predicate_functions(&mut builtins);

    let not_pred = builtins.get("not").unwrap();
    if let Value::Procedure(proc) = not_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test not with false
                let args = vec![Value::Boolean(false)];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                // Test not with true
                let args = vec![Value::Boolean(true)];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));

                // Test not with truthy values (everything except #f is truthy)
                let truthy_values = vec![
                    Value::Number(SchemeNumber::Integer(0)),
                    Value::String("".to_string()),
                    Value::Nil,
                    Value::Character('a'),
                    Value::Symbol("test".to_string()),
                ];

                for val in truthy_values {
                    let args = vec![val];
                    let result = func(&args);
                    assert!(result.is_ok());
                    assert_eq!(result.unwrap(), Value::Boolean(false));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_eof_object_predicate() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_predicate_functions(&mut builtins);

    let eof_pred = builtins.get("eof-object?").unwrap();
    if let Value::Procedure(proc) = eof_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test with various non-EOF values
                let test_values = vec![
                    Value::Number(SchemeNumber::Integer(42)),
                    Value::String("test".to_string()),
                    Value::Boolean(true),
                    Value::Nil,
                    Value::Character('a'),
                ];

                for val in test_values {
                    let args = vec![val];
                    let result = func(&args);
                    assert!(result.is_ok());
                    // For now, assuming no values are EOF objects
                    assert_eq!(result.unwrap(), Value::Boolean(false));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_predicate_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_predicate_functions(&mut builtins);

    // Test unary predicates with wrong arity
    let unary_predicates = vec![
        "number?", "string?", "symbol?", "boolean?", "procedure?", "char?",
        "vector?", "pair?", "null?", "list?", "not", "eof-object?", "exact?",
        "inexact?", "integer?", "rational?", "real?", "complex?",
    ];

    for pred_name in unary_predicates {
        let pred = builtins.get(pred_name).unwrap();
        if let Value::Procedure(proc) = pred {
            match proc {
                Procedure::Builtin { func, .. } => {
                    // Test with no arguments
                    let args = vec![];
                    let result = func(&args);
                    assert!(result.is_err(), "{} should fail with no arguments", pred_name);
                    if let Err(LambdustError::ArityError { expected, .. }) = result {
                        assert_eq!(expected, 1);
                    }

                    // Test with too many arguments
                    let args = vec![Value::Number(SchemeNumber::Integer(1)), Value::Number(SchemeNumber::Integer(2))];
                    let result = func(&args);
                    assert!(result.is_err(), "{} should fail with too many arguments", pred_name);
                    if let Err(LambdustError::ArityError { expected, .. }) = result {
                        assert_eq!(expected, 1);
                    }
                }
                _ => panic!("Expected builtin procedure"),
            }
        }
    }

    // Test binary predicates with wrong arity
    let binary_predicates = vec!["eq?", "eqv?", "equal?"];

    for pred_name in binary_predicates {
        let pred = builtins.get(pred_name).unwrap();
        if let Value::Procedure(proc) = pred {
            match proc {
                Procedure::Builtin { func, .. } => {
                    // Test with no arguments
                    let args = vec![];
                    let result = func(&args);
                    assert!(result.is_err(), "{} should fail with no arguments", pred_name);
                    if let Err(LambdustError::ArityError { expected, .. }) = result {
                        assert_eq!(expected, 2);
                    }

                    // Test with one argument
                    let args = vec![Value::Number(SchemeNumber::Integer(1))];
                    let result = func(&args);
                    assert!(result.is_err(), "{} should fail with one argument", pred_name);
                    if let Err(LambdustError::ArityError { expected, .. }) = result {
                        assert_eq!(expected, 2);
                    }

                    // Test with too many arguments
                    let args = vec![
                        Value::Number(SchemeNumber::Integer(1)),
                        Value::Number(SchemeNumber::Integer(2)),
                        Value::Number(SchemeNumber::Integer(3)),
                    ];
                    let result = func(&args);
                    assert!(result.is_err(), "{} should fail with too many arguments", pred_name);
                    if let Err(LambdustError::ArityError { expected, .. }) = result {
                        assert_eq!(expected, 2);
                    }
                }
                _ => panic!("Expected builtin procedure"),
            }
        }
    }
}

#[test]
fn test_predicate_edge_cases() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_predicate_functions(&mut builtins);

    // Test with undefined values
    let undefined_tests = vec![
        ("number?", Value::Undefined),
        ("string?", Value::Undefined),
        ("symbol?", Value::Undefined),
        ("boolean?", Value::Undefined),
        ("procedure?", Value::Undefined),
        ("char?", Value::Undefined),
        ("vector?", Value::Undefined),
        ("pair?", Value::Undefined),
        ("null?", Value::Undefined),
        ("list?", Value::Undefined),
    ];

    for (pred_name, test_val) in undefined_tests {
        let pred = builtins.get(pred_name).unwrap();
        if let Value::Procedure(proc) = pred {
            match proc {
                Procedure::Builtin { func, .. } => {
                    let args = vec![test_val];
                    let result = func(&args);
                    assert!(result.is_ok(), "{} should handle undefined values", pred_name);
                    assert_eq!(result.unwrap(), Value::Boolean(false));
                }
                _ => panic!("Expected builtin procedure"),
            }
        }
    }
}

#[test]
fn test_predicates_isolation() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_predicate_functions(&mut builtins);

    // Test that each predicate function works independently
    let predicates = vec![
        "number?", "string?", "symbol?", "boolean?", "procedure?", "char?",
        "vector?", "pair?", "null?", "list?", "eq?", "eqv?", "equal?",
        "not", "eof-object?", "exact?", "inexact?", "integer?", "rational?",
        "real?", "complex?",
    ];

    for pred_name in predicates {
        let proc = builtins.get(pred_name).unwrap();
        assert!(proc.is_procedure(), "{} should be a procedure", pred_name);
    }
}

#[test]
fn test_complex_equality_cases() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_predicate_functions(&mut builtins);

    // Test equal? with nested structures
    let equal_pred = builtins.get("equal?").unwrap();
    if let Value::Procedure(proc) = equal_pred {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test equal? with vectors
                let vec1 = Value::Vector(vec![
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::String("hello".to_string()),
                ]);
                let vec2 = Value::Vector(vec![
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::String("hello".to_string()),
                ]);
                let args = vec![vec1, vec2];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                // Test equal? with different vectors
                let vec3 = Value::Vector(vec![
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::String("world".to_string()),
                ]);
                let vec4 = Value::Vector(vec![
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::String("hello".to_string()),
                ]);
                let args = vec![vec3, vec4];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}