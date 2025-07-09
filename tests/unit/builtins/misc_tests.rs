//! Unit tests for miscellaneous functions (misc.rs)
//!
//! Tests the miscellaneous built-in functions including multiple values
//! and record operations (SRFI 9).

use lambdust::builtins::misc::register_misc_functions;
use lambdust::error::LambdustError;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;
use std::collections::HashMap;

#[test]
fn test_misc_functions_registration() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    // Check that all misc functions are registered
    assert!(builtins.contains_key("values"));
    assert!(builtins.contains_key("make-record"));
    assert!(builtins.contains_key("record-of-type?"));
    assert!(builtins.contains_key("record-field"));
    assert!(builtins.contains_key("record-set-field!"));

    // Check that they are all procedures
    for (name, value) in &builtins {
        assert!(value.is_procedure(), "{} should be a procedure", name);
    }
}

#[test]
fn test_values_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let values_proc = builtins.get("values").unwrap();

    // Test values with no arguments
    if let Value::Procedure(proc) = values_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_ok());
                if let Value::Values(vals) = result.unwrap() {
                    assert_eq!(vals.len(), 0);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test values with one argument
    if let Value::Procedure(proc) = values_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_ok());
                if let Value::Values(vals) = result.unwrap() {
                    assert_eq!(vals.len(), 1);
                    assert_eq!(vals[0], Value::Number(SchemeNumber::Integer(42)));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test values with multiple arguments
    if let Value::Procedure(proc) = values_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::String("hello".to_string()),
                    Value::Boolean(true),
                ];
                let result = func(&args);
                assert!(result.is_ok());
                if let Value::Values(vals) = result.unwrap() {
                    assert_eq!(vals.len(), 3);
                    assert_eq!(vals[0], Value::Number(SchemeNumber::Integer(1)));
                    assert_eq!(vals[1], Value::String("hello".to_string()));
                    assert_eq!(vals[2], Value::Boolean(true));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_make_record_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let make_record_proc = builtins.get("make-record").unwrap();

    // Test make-record with symbol type name
    if let Value::Procedure(proc) = make_record_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Symbol("person".to_string()),
                    Value::String("John".to_string()),
                    Value::Number(SchemeNumber::Integer(30)),
                ];
                let result = func(&args);
                assert!(result.is_ok());
                
                let record_value = result.unwrap();
                assert!(record_value.is_record_of_type("person"));
                
                if let Some(record) = record_value.as_record() {
                    assert_eq!(record.record_type.name, "person");
                    assert_eq!(record.fields.len(), 2);
                    assert_eq!(record.fields[0], Value::String("John".to_string()));
                    assert_eq!(record.fields[1], Value::Number(SchemeNumber::Integer(30)));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test make-record with string type name
    if let Value::Procedure(proc) = make_record_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::String("book".to_string()),
                    Value::String("1984".to_string()),
                    Value::String("Orwell".to_string()),
                ];
                let result = func(&args);
                assert!(result.is_ok());
                
                let record_value = result.unwrap();
                assert!(record_value.is_record_of_type("book"));
                
                if let Some(record) = record_value.as_record() {
                    assert_eq!(record.record_type.name, "book");
                    assert_eq!(record.fields.len(), 2);
                    assert_eq!(record.fields[0], Value::String("1984".to_string()));
                    assert_eq!(record.fields[1], Value::String("Orwell".to_string()));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_make_record_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let make_record_proc = builtins.get("make-record").unwrap();

    // Test make-record with no arguments
    if let Value::Procedure(proc) = make_record_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test make-record with only one argument
    if let Value::Procedure(proc) = make_record_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Symbol("person".to_string())];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_make_record_type_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let make_record_proc = builtins.get("make-record").unwrap();

    // Test make-record with non-symbol/string type name
    if let Value::Procedure(proc) = make_record_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Number(SchemeNumber::Integer(42)),
                    Value::String("value".to_string()),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::TypeError { message, .. }) = result {
                    assert!(message.contains("expected type name as symbol or string"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_record_of_type_predicate() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let make_record_proc = builtins.get("make-record").unwrap();
    let record_of_type_proc = builtins.get("record-of-type?").unwrap();

    // Create a record first
    let record = if let Value::Procedure(proc) = make_record_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Symbol("person".to_string()),
                    Value::String("John".to_string()),
                ];
                func(&args).unwrap()
            }
            _ => panic!("Expected builtin procedure"),
        }
    } else {
        panic!("Expected procedure");
    };

    // Test record-of-type? with correct type
    if let Value::Procedure(proc) = record_of_type_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![record.clone(), Value::Symbol("person".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test record-of-type? with wrong type
    if let Value::Procedure(proc) = record_of_type_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![record.clone(), Value::Symbol("book".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test record-of-type? with non-record
    if let Value::Procedure(proc) = record_of_type_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Number(SchemeNumber::Integer(42)),
                    Value::Symbol("person".to_string()),
                ];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_record_of_type_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let record_of_type_proc = builtins.get("record-of-type?").unwrap();

    // Test record-of-type? with no arguments
    if let Value::Procedure(proc) = record_of_type_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test record-of-type? with one argument
    if let Value::Procedure(proc) = record_of_type_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test record-of-type? with too many arguments
    if let Value::Procedure(proc) = record_of_type_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::Symbol("person".to_string()),
                    Value::Number(SchemeNumber::Integer(3)),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_record_of_type_type_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let record_of_type_proc = builtins.get("record-of-type?").unwrap();

    // Test record-of-type? with non-symbol/string type name
    if let Value::Procedure(proc) = record_of_type_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Number(SchemeNumber::Integer(42)),
                    Value::Number(SchemeNumber::Integer(123)),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::TypeError { message, .. }) = result {
                    assert!(message.contains("expected type name as symbol or string"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_record_field_get() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let make_record_proc = builtins.get("make-record").unwrap();
    let record_field_proc = builtins.get("record-field").unwrap();

    // Create a record first
    let record = if let Value::Procedure(proc) = make_record_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Symbol("person".to_string()),
                    Value::String("John".to_string()),
                    Value::Number(SchemeNumber::Integer(30)),
                ];
                func(&args).unwrap()
            }
            _ => panic!("Expected builtin procedure"),
        }
    } else {
        panic!("Expected procedure");
    };

    // Test record-field with valid indices
    if let Value::Procedure(proc) = record_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                // Get first field
                let args = vec![record.clone(), Value::Number(SchemeNumber::Integer(0))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::String("John".to_string()));

                // Get second field
                let args = vec![record.clone(), Value::Number(SchemeNumber::Integer(1))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Integer(30)));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_record_field_get_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let make_record_proc = builtins.get("make-record").unwrap();
    let record_field_proc = builtins.get("record-field").unwrap();

    // Create a record first
    let record = if let Value::Procedure(proc) = make_record_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Symbol("person".to_string()),
                    Value::String("John".to_string()),
                ];
                func(&args).unwrap()
            }
            _ => panic!("Expected builtin procedure"),
        }
    } else {
        panic!("Expected procedure");
    };

    // Test record-field with non-record
    if let Value::Procedure(proc) = record_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Number(SchemeNumber::Integer(42)),
                    Value::Number(SchemeNumber::Integer(0)),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::TypeError { message, .. }) = result {
                    assert!(message.contains("expected record"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test record-field with invalid index type
    if let Value::Procedure(proc) = record_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![record.clone(), Value::String("not-an-index".to_string())];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::TypeError { message, .. }) = result {
                    assert!(message.contains("expected non-negative integer index"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test record-field with negative index
    if let Value::Procedure(proc) = record_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![record.clone(), Value::Number(SchemeNumber::Integer(-1))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::TypeError { message, .. }) = result {
                    assert!(message.contains("expected non-negative integer index"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test record-field with out-of-bounds index
    if let Value::Procedure(proc) = record_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![record.clone(), Value::Number(SchemeNumber::Integer(99))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("index 99 out of bounds"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_record_field_set() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let make_record_proc = builtins.get("make-record").unwrap();
    let record_field_proc = builtins.get("record-field").unwrap();
    let record_set_field_proc = builtins.get("record-set-field!").unwrap();

    // Create a record first
    let record = if let Value::Procedure(proc) = make_record_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Symbol("person".to_string()),
                    Value::String("John".to_string()),
                    Value::Number(SchemeNumber::Integer(30)),
                ];
                func(&args).unwrap()
            }
            _ => panic!("Expected builtin procedure"),
        }
    } else {
        panic!("Expected procedure");
    };

    // Test record-set-field! with valid arguments
    let updated_record = if let Value::Procedure(proc) = record_set_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    record.clone(),
                    Value::Number(SchemeNumber::Integer(0)),
                    Value::String("Jane".to_string()),
                ];
                let result = func(&args);
                assert!(result.is_ok());
                result.unwrap()
            }
            _ => panic!("Expected builtin procedure"),
        }
    } else {
        panic!("Expected procedure");
    };

    // Verify the field was updated
    if let Value::Procedure(proc) = record_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![updated_record.clone(), Value::Number(SchemeNumber::Integer(0))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::String("Jane".to_string()));

                // Verify other fields remain unchanged
                let args = vec![updated_record.clone(), Value::Number(SchemeNumber::Integer(1))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Integer(30)));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Verify original record is unchanged (immutable)
    if let Value::Procedure(proc) = record_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![record.clone(), Value::Number(SchemeNumber::Integer(0))];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::String("John".to_string()));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_record_field_set_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let make_record_proc = builtins.get("make-record").unwrap();
    let record_set_field_proc = builtins.get("record-set-field!").unwrap();

    // Create a record first
    let record = if let Value::Procedure(proc) = make_record_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Symbol("person".to_string()),
                    Value::String("John".to_string()),
                ];
                func(&args).unwrap()
            }
            _ => panic!("Expected builtin procedure"),
        }
    } else {
        panic!("Expected procedure");
    };

    // Test record-set-field! with non-record
    if let Value::Procedure(proc) = record_set_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Number(SchemeNumber::Integer(42)),
                    Value::Number(SchemeNumber::Integer(0)),
                    Value::String("value".to_string()),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::TypeError { message, .. }) = result {
                    assert!(message.contains("expected record"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test record-set-field! with invalid index type
    if let Value::Procedure(proc) = record_set_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    record.clone(),
                    Value::String("not-an-index".to_string()),
                    Value::String("value".to_string()),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::TypeError { message, .. }) = result {
                    assert!(message.contains("expected non-negative integer index"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test record-set-field! with out-of-bounds index
    if let Value::Procedure(proc) = record_set_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    record.clone(),
                    Value::Number(SchemeNumber::Integer(99)),
                    Value::String("value".to_string()),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("index 99 out of bounds"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_record_field_set_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let record_set_field_proc = builtins.get("record-set-field!").unwrap();

    // Test record-set-field! with insufficient arguments
    if let Value::Procedure(proc) = record_set_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::Number(SchemeNumber::Integer(0)),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 3);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test record-set-field! with too many arguments
    if let Value::Procedure(proc) = record_set_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::Number(SchemeNumber::Integer(0)),
                    Value::String("value".to_string()),
                    Value::String("extra".to_string()),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 3);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_misc_functions_isolation() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    // Test that each misc function works independently
    let functions = vec![
        "values",
        "make-record",
        "record-of-type?",
        "record-field",
        "record-set-field!",
    ];

    for func_name in functions {
        let proc = builtins.get(func_name).unwrap();
        assert!(proc.is_procedure(), "{} should be a procedure", func_name);
    }
}

#[test]
fn test_values_function_edge_cases() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let values_proc = builtins.get("values").unwrap();

    // Test values with many arguments
    if let Value::Procedure(proc) = values_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args: Vec<Value> = (0..100)
                    .map(|i| Value::Number(SchemeNumber::Integer(i)))
                    .collect();
                let result = func(&args);
                assert!(result.is_ok());
                if let Value::Values(vals) = result.unwrap() {
                    assert_eq!(vals.len(), 100);
                    for (i, val) in vals.iter().enumerate() {
                        assert_eq!(*val, Value::Number(SchemeNumber::Integer(i as i64)));
                    }
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test values with mixed types
    if let Value::Procedure(proc) = values_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Nil,
                    Value::Undefined,
                    Value::Vector(vec![]),
                    Value::Character('\0'),
                ];
                let result = func(&args);
                assert!(result.is_ok());
                if let Value::Values(vals) = result.unwrap() {
                    assert_eq!(vals.len(), 4);
                    assert_eq!(vals[0], Value::Nil);
                    assert_eq!(vals[1], Value::Undefined);
                    assert_eq!(vals[2], Value::Vector(vec![]));
                    assert_eq!(vals[3], Value::Character('\0'));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_record_operations_comprehensive() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_misc_functions(&mut builtins);

    let make_record_proc = builtins.get("make-record").unwrap();
    let record_of_type_proc = builtins.get("record-of-type?").unwrap();
    let record_field_proc = builtins.get("record-field").unwrap();
    let _record_set_field_proc = builtins.get("record-set-field!").unwrap();

    // Test that record creation with insufficient arguments fails
    if let Value::Procedure(proc) = make_record_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Symbol("empty".to_string())];
                // This should fail due to minimum arity
                let result = func(&args);
                assert!(result.is_err());
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Create a record with many fields
    let many_fields: Vec<Value> = (0..10)
        .map(|i| Value::Number(SchemeNumber::Integer(i)))
        .collect();
    let mut args = vec![Value::Symbol("many-fields".to_string())];
    args.extend(many_fields);

    let large_record = if let Value::Procedure(proc) = make_record_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let result = func(&args);
                assert!(result.is_ok());
                result.unwrap()
            }
            _ => panic!("Expected builtin procedure"),
        }
    } else {
        panic!("Expected procedure");
    };

    // Test accessing all fields
    if let Value::Procedure(proc) = record_field_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                for i in 0..10 {
                    let args = vec![
                        large_record.clone(),
                        Value::Number(SchemeNumber::Integer(i)),
                    ];
                    let result = func(&args);
                    assert!(result.is_ok());
                    assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Integer(i)));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test type checking works correctly
    if let Value::Procedure(proc) = record_of_type_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![large_record.clone(), Value::Symbol("many-fields".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(true));

                let args = vec![large_record.clone(), Value::Symbol("different-type".to_string())];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Boolean(false));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}