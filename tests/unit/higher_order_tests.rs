//! Unit tests for the higher-order functions module
//!
//! These tests verify the functionality of higher-order functions like map, apply, and fold.

use lambdust::builtins::higher_order::{
    apply_implementation, fold_implementation, map_implementation,
};
use lambdust::error::LambdustError;
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Procedure, Value};

#[test]
fn test_map_with_builtin_function() {
    // Create a simple test: (map + '(1 2 3) '(4 5 6))
    // This would be (+ 1 4), (+ 2 5), (+ 3 6) = (5 7 9)

    // Create the + function
    let plus_func = Value::Procedure(Procedure::Builtin {
        name: "+".to_string(),
        arity: None,
        func: |args| {
            let mut sum = 0i64;
            for arg in args {
                match arg {
                    Value::Number(SchemeNumber::Integer(n)) => sum += n,
                    _ => {
                        return Err(LambdustError::type_error(
                            "+ requires numbers".to_string(),
                        ))
                    }
                }
            }
            Ok(Value::Number(SchemeNumber::Integer(sum)))
        },
    });

    let list1 = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
    ]);

    let list2 = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(4)),
        Value::Number(SchemeNumber::Integer(5)),
        Value::Number(SchemeNumber::Integer(6)),
    ]);

    let result = map_implementation(&[plus_func, list1, list2]).unwrap();

    let expected = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(5)),
        Value::Number(SchemeNumber::Integer(7)),
        Value::Number(SchemeNumber::Integer(9)),
    ]);

    assert_eq!(result, expected);
}

#[test]
fn test_apply_with_builtin_function() {
    // Test (apply + '(1 2 3 4))
    let plus_func = Value::Procedure(Procedure::Builtin {
        name: "+".to_string(),
        arity: None,
        func: |args| {
            let mut sum = 0i64;
            for arg in args {
                match arg {
                    Value::Number(SchemeNumber::Integer(n)) => sum += n,
                    _ => {
                        return Err(LambdustError::type_error(
                            "+ requires numbers".to_string(),
                        ))
                    }
                }
            }
            Ok(Value::Number(SchemeNumber::Integer(sum)))
        },
    });

    let list = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
        Value::Number(SchemeNumber::Integer(4)),
    ]);

    let result = apply_implementation(&[plus_func, list]).unwrap();
    let expected = Value::Number(SchemeNumber::Integer(10));

    assert_eq!(result, expected);
}

#[test]
fn test_fold_with_builtin_function() {
    // Test (fold + 0 '(1 2 3 4))
    let plus_func = Value::Procedure(Procedure::Builtin {
        name: "+".to_string(),
        arity: None,
        func: |args| {
            let mut sum = 0i64;
            for arg in args {
                match arg {
                    Value::Number(SchemeNumber::Integer(n)) => sum += n,
                    _ => {
                        return Err(LambdustError::type_error(
                            "+ requires numbers".to_string(),
                        ))
                    }
                }
            }
            Ok(Value::Number(SchemeNumber::Integer(sum)))
        },
    });

    let init = Value::Number(SchemeNumber::Integer(0));
    let list = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
        Value::Number(SchemeNumber::Integer(4)),
    ]);

    let result = fold_implementation(&[plus_func, init, list]).unwrap();
    let expected = Value::Number(SchemeNumber::Integer(10));

    assert_eq!(result, expected);
}