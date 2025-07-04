//! Unit tests for import special form in evaluator

use lambdust::ast::{Expr, Literal};
use lambdust::evaluator::{Continuation, Evaluator};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_import_srfi_1() {
    let mut evaluator = Evaluator::new();
    
    // Test (import (srfi 1))
    let import_expr = Expr::List(vec![
        Expr::Variable("import".to_string()),
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        ]),
    ]);
    
    let result = evaluator.eval(
        import_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    
    // Test that SRFI 1 functions are now available
    // Create a list first using quote
    let list_expr = Expr::List(vec![
        Expr::Variable("quote".to_string()),
        Expr::List(vec![
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
        ]),
    ]);
    
    // Evaluate the list
    let _list_result = evaluator.eval(
        list_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    ).unwrap();
    
    // Now test take function: (take list 2)
    let take_expr = Expr::List(vec![
        Expr::Variable("take".to_string()),
        Expr::List(vec![
            Expr::Variable("quote".to_string()),
            Expr::List(vec![
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
            ]),
        ]),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);
    
    let result = evaluator.eval(
        take_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    
    // Check that result is a list with first 2 elements
    let result_val = result.unwrap();
    match result_val {
        Value::Vector(elements) => {
            assert_eq!(elements.len(), 2);
            assert_eq!(elements[0], Value::Number(SchemeNumber::Integer(1)));
            assert_eq!(elements[1], Value::Number(SchemeNumber::Integer(2)));
        }
        Value::Pair(_) | Value::Nil => {
            // Take function returns a list, which could be represented as pairs
            if let Some(vec) = result_val.to_vector() {
                assert_eq!(vec.len(), 2);
                assert_eq!(vec[0], Value::Number(SchemeNumber::Integer(1)));
                assert_eq!(vec[1], Value::Number(SchemeNumber::Integer(2)));
            } else {
                panic!("Could not convert result to vector");
            }
        }
        _ => panic!("Expected vector or list result, got: {:?}", result_val),
    }
}

#[test]
fn test_import_srfi_13() {
    let mut evaluator = Evaluator::new();
    
    // Test (import (srfi 13))
    let import_expr = Expr::List(vec![
        Expr::Variable("import".to_string()),
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(13))),
        ]),
    ]);
    
    let result = evaluator.eval(
        import_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    
    // Test that SRFI 13 functions are now available
    let string_null_expr = Expr::List(vec![
        Expr::Variable("string-null?".to_string()),
        Expr::Literal(Literal::String("".to_string())),
    ]);
    
    let result = evaluator.eval(
        string_null_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_import_srfi_69() {
    let mut evaluator = Evaluator::new();
    
    // Test (import (srfi 69))
    let import_expr = Expr::List(vec![
        Expr::Variable("import".to_string()),
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(69))),
        ]),
    ]);
    
    let result = evaluator.eval(
        import_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    
    // Test that SRFI 69 functions are now available
    let make_hash_table_expr = Expr::List(vec![
        Expr::Variable("make-hash-table".to_string()),
    ]);
    
    let result = evaluator.eval(
        make_hash_table_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    
    // Check that result is a hash table
    match result.unwrap() {
        Value::HashTable(_) => (), // Success
        _ => panic!("Expected hash table result"),
    }
}

#[test]
fn test_import_multiple_srfis() {
    let mut evaluator = Evaluator::new();
    
    // Test (import (srfi 1) (srfi 13))
    let import_expr = Expr::List(vec![
        Expr::Variable("import".to_string()),
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        ]),
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(13))),
        ]),
    ]);
    
    let result = evaluator.eval(
        import_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    
    // Test that both SRFI 1 and SRFI 13 functions are available
    // Test SRFI 1 function
    let take_expr = Expr::List(vec![
        Expr::Variable("take".to_string()),
        Expr::List(vec![
            Expr::Variable("quote".to_string()),
            Expr::List(vec![
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            ]),
        ]),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);
    
    let result = evaluator.eval(
        take_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    
    // Test SRFI 13 function
    let string_null_expr = Expr::List(vec![
        Expr::Variable("string-null?".to_string()),
        Expr::Literal(Literal::String("test".to_string())),
    ]);
    
    let result = evaluator.eval(
        string_null_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
}

#[test]
fn test_import_error_cases() {
    let mut evaluator = Evaluator::new();
    
    // Test import with no arguments
    let import_expr = Expr::List(vec![
        Expr::Variable("import".to_string()),
    ]);
    
    let result = evaluator.eval(
        import_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_err());
    
    // Test import with invalid SRFI ID
    let import_expr = Expr::List(vec![
        Expr::Variable("import".to_string()),
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(999))),
        ]),
    ]);
    
    let result = evaluator.eval(
        import_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_err());
}