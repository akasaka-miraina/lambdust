//! Comprehensive test suite for SRFI-8 receive implementation
//!
//! This test suite verifies the correct implementation of SRFI-8 `receive` syntax
//! for binding multiple values in Lambdust. Tests cover all formal parameter
//! patterns, error conditions, and integration with the existing multi-value system.

use lambdust::ast::Formals;
use lambdust::eval::value::{MultipleValues, ThreadSafeEnvironment, Value};
use lambdust::stdlib::srfi8_receive::{
    bind_multiple_values, create_srfi8_bindings, enhanced_primitive_values, expand_receive,
    primitive_multiple_values_p, primitive_values_length, validate_receive_arity,
};
use lambdust::utils::intern_symbol;
use std::sync::Arc;

/// Test enhanced values procedure with proper multiple values support.
#[test]
fn test_enhanced_values_procedure() {
    // Test values with no arguments -> unspecified
    let result = enhanced_primitive_values(&[]).unwrap();
    assert_eq!(result, Value::Unspecified);

    // Test values with single argument -> return that argument
    let single_arg = Value::integer(42);
    let result = enhanced_primitive_values(&[single_arg.clone()]).unwrap();
    assert_eq!(result, single_arg);

    // Test values with multiple arguments -> MultipleValues object
    let args = vec![
        Value::integer(1),
        Value::string("hello".to_string()),
        Value::boolean(true),
    ];
    let result = enhanced_primitive_values(&args).unwrap();

    assert!(matches!(result, Value::MultipleValues(_)));
    if let Value::MultipleValues(mv) = result {
        assert_eq!(mv.len(), 3);
        assert_eq!(*mv.get(0).unwrap(), Value::integer(1));
        assert_eq!(*mv.get(1).unwrap(), Value::string("hello".to_string()));
        assert_eq!(*mv.get(2).unwrap(), Value::boolean(true));
    }
}

/// Test multiple-values? predicate.
#[test]
fn test_multiple_values_predicate() {
    // Test with single values
    let single_cases = vec![
        Value::integer(42),
        Value::string("test".to_string()),
        Value::boolean(false),
        Value::Nil,
        Value::Unspecified,
    ];

    for case in single_cases {
        let result = primitive_multiple_values_p(&[case]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    // Test with MultipleValues object
    let mv = Value::MultipleValues(Arc::new(MultipleValues::new(vec![
        Value::integer(1),
        Value::integer(2),
        Value::integer(3),
    ])));
    let result = primitive_multiple_values_p(&[mv]).unwrap();
    assert_eq!(result, Value::boolean(true));

    // Test error cases
    assert!(primitive_multiple_values_p(&[]).is_err());
    assert!(primitive_multiple_values_p(&[Value::integer(1), Value::integer(2)]).is_err());
}

/// Test values-length procedure.
#[test]
fn test_values_length() {
    // Test with single values (length 1)
    let single_cases = vec![
        Value::integer(42),
        Value::string("test".to_string()),
        Value::boolean(true),
    ];

    for case in single_cases {
        let result = primitive_values_length(&[case]).unwrap();
        assert_eq!(result, Value::integer(1));
    }

    // Test with MultipleValues objects
    // Test empty MultipleValues
    let mv = Value::MultipleValues(Arc::new(MultipleValues::new(vec![])));
    let result = primitive_values_length(&[mv]).unwrap();
    assert_eq!(result, Value::integer(0));

    // Test single value MultipleValues
    let mv = Value::MultipleValues(Arc::new(MultipleValues::new(vec![Value::integer(1)])));
    let result = primitive_values_length(&[mv]).unwrap();
    assert_eq!(result, Value::integer(1));

    // Test two values MultipleValues
    let mv = Value::MultipleValues(Arc::new(MultipleValues::new(vec![Value::integer(1), Value::integer(2)])));
    let result = primitive_values_length(&[mv]).unwrap();
    assert_eq!(result, Value::integer(2));

    // Test three values MultipleValues  
    let mv = Value::MultipleValues(Arc::new(MultipleValues::new(vec![
        Value::integer(1),
        Value::string("test".to_string()),
        Value::boolean(true),
    ])));
    let result = primitive_values_length(&[mv]).unwrap();
    assert_eq!(result, Value::integer(3));

    // Test error cases
    assert!(primitive_values_length(&[]).is_err());
    assert!(primitive_values_length(&[Value::integer(1), Value::integer(2)]).is_err());
}

/// Test arity validation for different formal parameter patterns.
#[test]
fn test_receive_arity_validation() {
    // Test Fixed formals
    let fixed_formals = Formals::Fixed(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

    // Correct arity
    assert!(validate_receive_arity(&fixed_formals, 3).is_ok());

    // Incorrect arity
    assert!(validate_receive_arity(&fixed_formals, 2).is_err());
    assert!(validate_receive_arity(&fixed_formals, 4).is_err());
    assert!(validate_receive_arity(&fixed_formals, 0).is_err());

    // Test Variable formals (accepts any arity)
    let var_formals = Formals::Variable("args".to_string());

    for i in 0..10 {
        assert!(validate_receive_arity(&var_formals, i).is_ok());
    }

    // Test Mixed formals
    let mixed_formals = Formals::Mixed {
        fixed: vec!["a".to_string(), "b".to_string()],
        rest: "rest".to_string(),
    };

    // At least as many values as fixed parameters
    assert!(validate_receive_arity(&mixed_formals, 2).is_ok());
    assert!(validate_receive_arity(&mixed_formals, 3).is_ok());
    assert!(validate_receive_arity(&mixed_formals, 10).is_ok());

    // Too few values
    assert!(validate_receive_arity(&mixed_formals, 1).is_err());
    assert!(validate_receive_arity(&mixed_formals, 0).is_err());

    // Test unsupported formals (should return error)
    let keyword_formals = Formals::Keyword {
        fixed: vec![],
        rest: None,
        keywords: vec![],
    };
    assert!(validate_receive_arity(&keyword_formals, 0).is_err());
}

/// Test binding multiple values to different formal parameter patterns.
#[test]
fn test_bind_multiple_values() {
    let env = Arc::new(ThreadSafeEnvironment::default());

    // Test Fixed formals
    let fixed_formals = Formals::Fixed(vec![
        "first".to_string(),
        "second".to_string(),
        "third".to_string(),
    ]);
    let values = vec![
        Value::integer(1),
        Value::string("hello".to_string()),
        Value::boolean(true),
    ];

    assert!(bind_multiple_values(&env, &fixed_formals, &values).is_ok());

    // Verify bindings
    assert_eq!(env.lookup("first"), Some(Value::integer(1)));
    assert_eq!(
        env.lookup("second"),
        Some(Value::string("hello".to_string()))
    );
    assert_eq!(env.lookup("third"), Some(Value::boolean(true)));

    // Test Variable formals
    let env2 = Arc::new(ThreadSafeEnvironment::default());
    let var_formals = Formals::Variable("all_args".to_string());
    let values2 = vec![Value::integer(10), Value::integer(20), Value::integer(30)];

    assert!(bind_multiple_values(&env2, &var_formals, &values2).is_ok());

    // Verify binding - should be a list of all values
    if let Some(bound_value) = env2.lookup("all_args") {
        if let Some(list_values) = bound_value.as_list() {
            assert_eq!(list_values.len(), 3);
            assert_eq!(list_values[0], Value::integer(10));
            assert_eq!(list_values[1], Value::integer(20));
            assert_eq!(list_values[2], Value::integer(30));
        } else {
            panic!("Expected list value for variable formals");
        }
    } else {
        panic!("Variable formal binding not found");
    }

    // Test Mixed formals
    let env3 = Arc::new(ThreadSafeEnvironment::default());
    let mixed_formals = Formals::Mixed {
        fixed: vec!["x".to_string(), "y".to_string()],
        rest: "remaining".to_string(),
    };
    let values3 = vec![
        Value::integer(100),
        Value::integer(200),
        Value::integer(300),
        Value::integer(400),
        Value::integer(500),
    ];

    assert!(bind_multiple_values(&env3, &mixed_formals, &values3).is_ok());

    // Verify fixed bindings
    assert_eq!(env3.lookup("x"), Some(Value::integer(100)));
    assert_eq!(env3.lookup("y"), Some(Value::integer(200)));

    // Verify rest binding
    if let Some(rest_value) = env3.lookup("remaining") {
        if let Some(list_values) = rest_value.as_list() {
            assert_eq!(list_values.len(), 3);
            assert_eq!(list_values[0], Value::integer(300));
            assert_eq!(list_values[1], Value::integer(400));
            assert_eq!(list_values[2], Value::integer(500));
        } else {
            panic!("Expected list value for rest parameter");
        }
    } else {
        panic!("Rest parameter binding not found");
    }
}

/// Test expand_receive macro expansion.
#[test]
fn test_expand_receive() {
    // Test basic fixed formals expansion
    let formals = Formals::Fixed(vec!["a".to_string(), "b".to_string()]);
    let producer = Value::list(vec![
        Value::symbol(intern_symbol("values")),
        Value::integer(1),
        Value::integer(2),
    ]);
    let body = vec![Value::list(vec![
        Value::symbol(intern_symbol("+")),
        Value::symbol(intern_symbol("a")),
        Value::symbol(intern_symbol("b")),
    ])];

    let result = expand_receive(&formals, &producer, &body);
    assert!(result.is_ok());

    if let Ok(expanded) = result {
        // Should be a call-with-values form
        if let Some(list_values) = expanded.as_list() {
            assert_eq!(list_values.len(), 3); // call-with-values + producer-thunk + consumer-lambda
            assert_eq!(
                list_values[0],
                Value::symbol(intern_symbol("call-with-values"))
            );

            // Verify producer thunk structure
            if let Some(producer_list) = list_values[1].as_list() {
                assert_eq!(producer_list[0], Value::symbol(intern_symbol("lambda")));
                assert_eq!(producer_list[1], Value::Nil); // No parameters for thunk
            }

            // Verify consumer lambda structure
            if let Some(consumer_list) = list_values[2].as_list() {
                assert_eq!(consumer_list[0], Value::symbol(intern_symbol("lambda")));
                // Consumer should have formals matching our input
            }
        } else {
            panic!("Expected expanded form to be a list");
        }
    }

    // Test variable formals
    let var_formals = Formals::Variable("args".to_string());
    let result2 = expand_receive(&var_formals, &producer, &body);
    assert!(result2.is_ok());

    // Test mixed formals
    let mixed_formals = Formals::Mixed {
        fixed: &["first".to_string()],
        rest: "rest".to_string(),
    };
    let result3 = expand_receive(&mixed_formals, &producer, &body);
    assert!(result3.is_ok());

    // Test error case - empty body
    let result_error = expand_receive(&formals, &producer, &[]);
    assert!(result_error.is_err());
}

/// Test MultipleValues struct functionality.
#[test]
fn test_multiple_values_struct() {
    // Test creation and basic operations
    let values = vec![
        Value::integer(1),
        Value::string("test".to_string()),
        Value::boolean(false),
    ];
    let mv = MultipleValues::new(values.clone());

    assert_eq!(mv.len(), 3);
    assert!(!mv.is_empty());
    assert_eq!(mv.get(0), Some(&Value::integer(1)));
    assert_eq!(mv.get(1), Some(&Value::string("test".to_string())));
    assert_eq!(mv.get(2), Some(&Value::boolean(false)));
    assert_eq!(mv.get(3), None);

    // Test single value creation
    let single_mv = MultipleValues::single(Value::integer(42));
    assert_eq!(single_mv.len(), 1);
    assert_eq!(single_mv.get(0), Some(&Value::integer(42)));

    // Test empty MultipleValues
    let empty_mv = MultipleValues::new(&[]);
    assert_eq!(empty_mv.len(), 0);
    assert!(empty_mv.is_empty());
    assert_eq!(empty_mv.get(0), None);

    // Test iteration
    let iter_values: Vec<&Value> = mv.iter().collect();
    assert_eq!(iter_values.len(), 3);
    assert_eq!(iter_values[0], &Value::integer(1));
    assert_eq!(iter_values[1], &Value::string("test".to_string()));
    assert_eq!(iter_values[2], &Value::boolean(false));

    // Test into_vec
    let mv_clone = MultipleValues::new(values.clone());
    let extracted_values = mv_clone.into_vec();
    assert_eq!(extracted_values, values);

    // Test as_slice
    let slice = mv.as_slice();
    assert_eq!(slice.len(), 3);
    assert_eq!(slice[0], Value::integer(1));
}

/// Test SRFI-8 environment binding integration.
#[test]
fn test_srfi8_environment_integration() {
    let env = Arc::new(ThreadSafeEnvironment::default());
    create_srfi8_bindings(&env);

    // Verify that SRFI-8 procedures are bound
    assert!(env.lookup("values").is_some());
    assert!(env.lookup("call-with-values").is_some());
    assert!(env.lookup("receive").is_some());
    assert!(env.lookup("multiple-values?").is_some());
    assert!(env.lookup("values-length").is_some());

    // Verify values procedure works with enhanced implementation
    if let Some(values_proc) = env.lookup("values") {
        // This would require evaluator integration to fully test
        assert!(values_proc.is_procedure());
    }

    // Verify helper predicates work
    if let Some(mv_pred) = env.lookup("multiple-values?") {
        assert!(mv_pred.is_procedure());
    }

    if let Some(len_proc) = env.lookup("values-length") {
        assert!(len_proc.is_procedure());
    }
}

/// Test edge cases and error conditions.
#[test]
fn test_edge_cases_and_errors() {
    // Test arity validation with edge cases
    let empty_fixed = Formals::Fixed(&[]);
    assert!(validate_receive_arity(&empty_fixed, 0).is_ok());
    assert!(validate_receive_arity(&empty_fixed, 1).is_err());

    let empty_mixed = Formals::Mixed {
        fixed: &[],
        rest: "all".to_string(),
    };
    assert!(validate_receive_arity(&empty_mixed, 0).is_ok());
    assert!(validate_receive_arity(&empty_mixed, 5).is_ok());

    // Test MultipleValues equality
    let mv1 = MultipleValues::new(&[Value::integer(1), Value::integer(2)]);
    let mv2 = MultipleValues::new(&[Value::integer(1), Value::integer(2)]);
    let mv3 = MultipleValues::new(&[Value::integer(1), Value::integer(3)]);

    assert_eq!(mv1, mv2);
    assert_ne!(mv1, mv3);

    // Test binding with mismatched arity
    let env = Arc::new(ThreadSafeEnvironment::default());
    let formals = Formals::Fixed(vec!["a".to_string(), "b".to_string()]);
    let wrong_values = &[Value::integer(1)]; // Only one value for two parameters

    assert!(bind_multiple_values(&env, &formals, &wrong_values).is_err());

    // Test expand_receive error cases
    let formals = Formals::Fixed(&["a".to_string()]);
    let producer = Value::integer(42); // Simple value, not a producer expression
    let empty_body: Vec<Value> = &[];

    assert!(expand_receive(&formals, &producer, &empty_body).is_err());
}

/// Performance test for MultipleValues operations.
#[test]
fn test_multiple_values_performance() {
    // Create a large MultipleValues object
    let large_values: Vec<Value> = (0..1000).map(|i| Value::integer(i)).collect();

    let mv = MultipleValues::new(large_values.clone());

    // Test that operations are reasonably fast
    assert_eq!(mv.len(), 1000);
    assert_eq!(mv.get(500), Some(&Value::integer(500)));
    assert_eq!(mv.get(1000), None);

    // Test iteration performance
    let mut count = 0;
    for (i, value) in mv.iter().enumerate() {
        assert_eq!(value, &Value::integer(i as i64));
        count += 1;
    }
    assert_eq!(count, 1000);

    // Test slicing performance
    let slice = mv.as_slice();
    assert_eq!(slice.len(), 1000);
    assert_eq!(slice[999], Value::integer(999));
}

/// Test R7RS compliance scenarios.
#[test]
fn test_r7rs_compliance() {
    // Test that values with 0 arguments returns unspecified
    let result = enhanced_primitive_values(&[]).unwrap();
    assert_eq!(result, Value::Unspecified);

    // Test that values with 1 argument returns that argument unchanged
    let single_arg = Value::string("hello".to_string());
    let result = enhanced_primitive_values(&[single_arg.clone()]).unwrap();
    assert_eq!(result, single_arg);

    // Test that receive properly handles different formal patterns
    // This verifies that our implementation follows R7RS lambda formal semantics

    // Fixed arity: (receive (a b) (values 1 2) body...)
    let fixed_formals = Formals::Fixed(vec!["a".to_string(), "b".to_string()]);
    assert!(validate_receive_arity(&fixed_formals, 2).is_ok());
    assert!(validate_receive_arity(&fixed_formals, 1).is_err());
    assert!(validate_receive_arity(&fixed_formals, 3).is_err());

    // Variable arity: (receive args (values 1 2 3) body...)
    let var_formals = Formals::Variable("args".to_string());
    for i in 0..10 {
        assert!(validate_receive_arity(&var_formals, i).is_ok());
    }

    // Improper list: (receive (a b . rest) (values 1 2 3 4) body...)
    let mixed_formals = Formals::Mixed {
        fixed: &["a".to_string(), "b".to_string()],
        rest: "rest".to_string(),
    };
    assert!(validate_receive_arity(&mixed_formals, 2).is_ok()); // Minimum
    assert!(validate_receive_arity(&mixed_formals, 5).is_ok()); // Extra goes to rest
    assert!(validate_receive_arity(&mixed_formals, 1).is_err()); // Too few
}
