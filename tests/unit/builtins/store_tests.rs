//! Unit tests for store management functions (store.rs)
//!
//! Tests the memory management built-in functions including memory usage,
//! statistics collection, garbage collection, and location predicates.

use lambdust::builtins::store::{
    register_store_functions, statistics_to_scheme_value
};
use lambdust::error::LambdustError;
use lambdust::evaluator::types::StoreStatisticsWrapper;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;
use std::collections::HashMap;

#[test]
fn test_store_functions_registration() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    // Check that all store functions are registered
    assert!(builtins.contains_key("memory-usage"));
    assert!(builtins.contains_key("memory-statistics"));
    assert!(builtins.contains_key("collect-garbage"));
    assert!(builtins.contains_key("set-memory-limit!"));
    assert!(builtins.contains_key("location?"));
    assert!(builtins.contains_key("location-equal?"));

    // Check that they are all procedures
    for (name, value) in &builtins {
        assert!(value.is_procedure(), "{} should be a procedure", name);
    }
}

#[test]
fn test_memory_usage_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let memory_usage_proc = builtins.get("memory-usage").unwrap();

    // Test memory-usage with no arguments (correct usage)
    if let Value::Procedure(proc) = memory_usage_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("memory-usage requires evaluator context"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_memory_usage_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let memory_usage_proc = builtins.get("memory-usage").unwrap();

    // Test memory-usage with arguments - should fail
    if let Value::Procedure(proc) = memory_usage_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Integer(42))];
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
fn test_memory_statistics_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let memory_stats_proc = builtins.get("memory-statistics").unwrap();

    // Test memory-statistics with no arguments (correct usage)
    if let Value::Procedure(proc) = memory_stats_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("memory-statistics requires evaluator context"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_memory_statistics_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let memory_stats_proc = builtins.get("memory-statistics").unwrap();

    // Test memory-statistics with arguments - should fail
    if let Value::Procedure(proc) = memory_stats_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::String("extra".to_string())];
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
fn test_collect_garbage_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let collect_gc_proc = builtins.get("collect-garbage").unwrap();

    // Test collect-garbage with no arguments (correct usage)
    if let Value::Procedure(proc) = collect_gc_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("collect-garbage requires evaluator context"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_collect_garbage_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let collect_gc_proc = builtins.get("collect-garbage").unwrap();

    // Test collect-garbage with arguments - should fail
    if let Value::Procedure(proc) = collect_gc_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Boolean(true)];
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
fn test_set_memory_limit_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let set_limit_proc = builtins.get("set-memory-limit!").unwrap();

    // Test set-memory-limit! with integer argument
    if let Value::Procedure(proc) = set_limit_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Integer(1024))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("set-memory-limit! requires evaluator context"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test set-memory-limit! with real number argument
    if let Value::Procedure(proc) = set_limit_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Real(2048.0))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("set-memory-limit! requires evaluator context"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_set_memory_limit_type_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let set_limit_proc = builtins.get("set-memory-limit!").unwrap();

    // Test set-memory-limit! with non-numeric types - should fail
    if let Value::Procedure(proc) = set_limit_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let invalid_args = vec![
                    Value::String("not-a-number".to_string()),
                    Value::Boolean(true),
                    Value::Symbol("symbol".to_string()),
                    Value::Nil,
                ];

                for arg in invalid_args {
                    let args = vec![arg];
                    let result = func(&args);
                    assert!(result.is_err());
                    if let Err(LambdustError::TypeError { message, .. }) = result {
                        assert!(message.contains("Memory limit must be an integer"));
                    }
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test set-memory-limit! with fractional real number - should fail
    if let Value::Procedure(proc) = set_limit_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Real(1024.5))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::TypeError { message, .. }) = result {
                    assert!(message.contains("Memory limit must be an integer"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_set_memory_limit_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let set_limit_proc = builtins.get("set-memory-limit!").unwrap();

    // Test set-memory-limit! with no arguments - should fail
    if let Value::Procedure(proc) = set_limit_proc {
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

    // Test set-memory-limit! with too many arguments - should fail
    if let Value::Procedure(proc) = set_limit_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Number(SchemeNumber::Integer(1024)),
                    Value::Number(SchemeNumber::Integer(2048)),
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
fn test_location_predicate_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let location_pred_proc = builtins.get("location?").unwrap();

    // Test location? with various value types
    if let Value::Procedure(proc) = location_pred_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let test_values = vec![
                    Value::Number(SchemeNumber::Integer(42)),
                    Value::String("test".to_string()),
                    Value::Boolean(true),
                    Value::Symbol("symbol".to_string()),
                    Value::Nil,
                ];

                for val in test_values {
                    let args = vec![val];
                    let result = func(&args);
                    assert!(result.is_ok());
                    // For now, always returns false since locations aren't implemented
                    assert_eq!(result.unwrap(), Value::Boolean(false));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_location_predicate_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let location_pred_proc = builtins.get("location?").unwrap();

    // Test location? with no arguments - should fail
    if let Value::Procedure(proc) = location_pred_proc {
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

    // Test location? with too many arguments - should fail
    if let Value::Procedure(proc) = location_pred_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::Number(SchemeNumber::Integer(2)),
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
fn test_location_equal_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let location_eq_proc = builtins.get("location-equal?").unwrap();

    // Test location-equal? with various value pairs
    if let Value::Procedure(proc) = location_eq_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let test_pairs = vec![
                    (Value::Number(SchemeNumber::Integer(42)), Value::Number(SchemeNumber::Integer(42))),
                    (Value::String("test".to_string()), Value::String("test".to_string())),
                    (Value::Boolean(true), Value::Boolean(false)),
                    (Value::Symbol("symbol".to_string()), Value::Symbol("symbol".to_string())),
                    (Value::Nil, Value::Nil),
                ];

                for (val1, val2) in test_pairs {
                    let args = vec![val1, val2];
                    let result = func(&args);
                    assert!(result.is_ok());
                    // For now, always returns false since locations aren't implemented
                    assert_eq!(result.unwrap(), Value::Boolean(false));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_location_equal_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    let location_eq_proc = builtins.get("location-equal?").unwrap();

    // Test location-equal? with no arguments - should fail
    if let Value::Procedure(proc) = location_eq_proc {
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

    // Test location-equal? with one argument - should fail
    if let Value::Procedure(proc) = location_eq_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { function, .. }) = result {
                    assert_eq!(function, "<unknown>");
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test location-equal? with too many arguments - should fail
    if let Value::Procedure(proc) = location_eq_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                let args = vec![
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::Number(SchemeNumber::Integer(2)),
                    Value::Number(SchemeNumber::Integer(3)),
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
fn test_statistics_to_scheme_value_conversion() {
    // Create a mock statistics wrapper
    let stats = StoreStatisticsWrapper::from_raii(
        lambdust::evaluator::raii_store::RaiiStoreStatistics::default()
    );
    
    // Convert to Scheme value
    let scheme_value = statistics_to_scheme_value(&stats);
    
    // Check that the result is a proper list
    assert!(scheme_value.is_list());
    
    // Convert to vector to check contents
    if let Some(vec) = scheme_value.to_vector() {
        assert_eq!(vec.len(), 3);
        
        // Check that each element is a pair
        for item in &vec {
            assert!(item.as_pair().is_some());
        }
        
        // Check specific keys exist
        let mut found_keys = std::collections::HashSet::new();
        for item in &vec {
            if let Some((key, _value)) = item.as_pair() {
                if let Some(key_str) = key.as_symbol() {
                    found_keys.insert(key_str.to_string());
                }
            }
        }
        
        assert!(found_keys.contains("total-allocations"));
        assert!(found_keys.contains("total-deallocations"));
        assert!(found_keys.contains("memory-usage"));
    } else {
        panic!("Expected list result from statistics_to_scheme_value");
    }
}

#[test]
fn test_store_functions_isolation() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    // Test that each store function works independently
    let functions = vec![
        "memory-usage",
        "memory-statistics", 
        "collect-garbage",
        "set-memory-limit!",
        "location?",
        "location-equal?",
    ];

    for func_name in functions {
        let proc = builtins.get(func_name).unwrap();
        assert!(proc.is_procedure(), "{} should be a procedure", func_name);
    }
}

#[test]
fn test_store_functions_edge_cases() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    // Test set-memory-limit! with edge case values
    let set_limit_proc = builtins.get("set-memory-limit!").unwrap();
    
    if let Value::Procedure(proc) = set_limit_proc {
        match proc {
            lambdust::value::Procedure::Builtin { func, .. } => {
                // Test with zero limit
                let args = vec![Value::Number(SchemeNumber::Integer(0))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("set-memory-limit! requires evaluator context"));
                }
                
                // Test with negative limit
                let args = vec![Value::Number(SchemeNumber::Integer(-1))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("set-memory-limit! requires evaluator context"));
                }
                
                // Test with very large limit
                let args = vec![Value::Number(SchemeNumber::Integer(i64::MAX))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::RuntimeError { message, .. }) = result {
                    assert!(message.contains("set-memory-limit! requires evaluator context"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_store_functions_comprehensive_error_handling() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_store_functions(&mut builtins);

    // Test that all functions properly handle various error conditions
    let functions_with_arity = vec![
        ("memory-usage", 0),
        ("memory-statistics", 0),
        ("collect-garbage", 0),
        ("set-memory-limit!", 1),
        ("location?", 1),
        ("location-equal?", 2),
    ];

    for (func_name, expected_arity) in functions_with_arity {
        let proc = builtins.get(func_name).unwrap();
        
        if let Value::Procedure(proc_value) = proc {
            match proc_value {
                lambdust::value::Procedure::Builtin { func, .. } => {
                    // Test with wrong number of arguments
                    let wrong_args = vec![Value::Number(SchemeNumber::Integer(42)); expected_arity + 1];
                    let result = func(&wrong_args);
                    assert!(result.is_err(), "{} should fail with wrong arity", func_name);
                }
                _ => panic!("Expected builtin procedure for {}", func_name),
            }
        }
    }
}