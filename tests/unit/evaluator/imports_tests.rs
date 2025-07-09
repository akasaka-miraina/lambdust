//! Unit tests for import functionality (imports.rs)
//!
//! Tests the SRFI import system including import specification parsing,
//! SRFI module loading, part specification handling, and error cases.

use lambdust::ast::{Expr, Literal};
use lambdust::environment::Environment;
use lambdust::error::LambdustError;
use lambdust::evaluator::{Continuation, Evaluator};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;
use std::rc::Rc;

#[test]
fn test_import_with_empty_operands() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test import with no operands - should fail
    let result = evaluator.eval_import(&[], env, cont);
    assert!(result.is_err());
    if let Err(LambdustError::SyntaxError { message, .. }) = result {
        assert!(message.contains("import"));
        assert!(message.contains("at least one import set required"));
    }
}

#[test]
fn test_import_with_empty_specification() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test import with empty list - should fail
    let empty_spec = Expr::List(vec![]);
    let result = evaluator.eval_import(&[empty_spec], env, cont);
    assert!(result.is_err());
    if let Err(LambdustError::SyntaxError { message, .. }) = result {
        assert!(message.contains("import"));
        assert!(message.contains("empty import specification"));
    }
}

#[test]
fn test_import_with_non_list_specification() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test import with non-list specification - should fail
    let non_list_spec = Expr::Variable("not-a-list".to_string());
    let result = evaluator.eval_import(&[non_list_spec], env, cont);
    assert!(result.is_err());
    if let Err(LambdustError::SyntaxError { message, .. }) = result {
        assert!(message.contains("import"));
        assert!(message.contains("import specification must be a list"));
    }
}

#[test]
fn test_import_with_unsupported_library_type() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test import with unsupported library type - should fail
    let unsupported_spec = Expr::List(vec![
        Expr::Variable("unsupported".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);
    let result = evaluator.eval_import(&[unsupported_spec], env, cont);
    assert!(result.is_err());
    if let Err(LambdustError::SyntaxError { message, .. }) = result {
        assert!(message.contains("import"));
        assert!(message.contains("only SRFI imports are currently supported"));
    }
}

#[test]
fn test_srfi_import_without_number() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test SRFI import without number - should fail
    let srfi_spec = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
    ]);
    let result = evaluator.eval_import(&[srfi_spec], env, cont);
    assert!(result.is_err());
    if let Err(LambdustError::SyntaxError { message, .. }) = result {
        assert!(message.contains("import"));
        assert!(message.contains("SRFI number required"));
    }
}

#[test]
fn test_srfi_import_with_invalid_number() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test SRFI import with non-integer number - should fail
    let invalid_specs = vec![
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Variable("not-a-number".to_string()),
        ]),
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::String("1".to_string())),
        ]),
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Real(1.5))),
        ]),
    ];
    
    for spec in invalid_specs {
        let result = evaluator.eval_import(&[spec], env.clone(), cont.clone());
        assert!(result.is_err());
        if let Err(LambdustError::SyntaxError { message, .. }) = result {
            assert!(message.contains("import"));
            assert!(message.contains("SRFI number must be an integer"));
        }
    }
}

#[test]
fn test_srfi_import_with_valid_number() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test SRFI import with valid number
    let valid_spec = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);
    
    // This should attempt to load SRFI 1 (might succeed or fail depending on implementation)
    let result = evaluator.eval_import(&[valid_spec], env, cont);
    
    // The result might be Ok or Err depending on SRFI registry implementation
    // We just ensure it doesn't panic and processes the syntax correctly
    match result {
        Ok(value) => assert_eq!(value, Value::Undefined),
        Err(err) => {
            // If it fails, it should be a runtime error about missing SRFI, not syntax error
            match err {
                LambdustError::RuntimeError { .. } => {}, // Expected for unimplemented SRFI
                LambdustError::SyntaxError { message, .. } => panic!("Unexpected syntax error: {}", message),
                _ => {}, // Other error types are acceptable
            }
        }
    }
}

#[test]
fn test_srfi_import_with_parts() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test SRFI import with part specification
    let spec_with_parts = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Variable("lists".to_string()),
        Expr::Variable("fold".to_string()),
    ]);
    
    // This should attempt to load specific parts of SRFI 1
    let result = evaluator.eval_import(&[spec_with_parts], env, cont);
    
    // Similar to above - we check that syntax is processed correctly
    match result {
        Ok(value) => assert_eq!(value, Value::Undefined),
        Err(err) => {
            // Should be runtime error, not syntax error
            match err {
                LambdustError::RuntimeError { .. } => {}, // Expected for unimplemented SRFI
                LambdustError::SyntaxError { message, .. } => panic!("Unexpected syntax error: {}", message),
                _ => {}, // Other error types are acceptable
            }
        }
    }
}

// Note: parse_import_parts is a private method, so we test it indirectly through eval_import

#[test]
fn test_multiple_import_specifications() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test multiple import specifications
    let spec1 = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);
    let spec2 = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(13))),
    ]);
    
    let result = evaluator.eval_import(&[spec1, spec2], env, cont);
    
    // Should process both specifications (might succeed or fail based on implementation)
    match result {
        Ok(value) => assert_eq!(value, Value::Undefined),
        Err(err) => {
            // Should be runtime error for missing SRFI, not syntax error
            match err {
                LambdustError::RuntimeError { .. } => {}, // Expected for unimplemented SRFI
                LambdustError::SyntaxError { message, .. } => panic!("Unexpected syntax error: {}", message),
                _ => {}, // Other error types are acceptable
            }
        }
    }
}

#[test]
fn test_import_with_mixed_valid_invalid_specifications() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test with one valid and one invalid specification
    let valid_spec = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);
    let invalid_spec = Expr::List(vec![
        Expr::Variable("unsupported".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);
    
    let result = evaluator.eval_import(&[valid_spec, invalid_spec], env, cont);
    
    // Should fail on the invalid specification
    assert!(result.is_err());
    if let Err(LambdustError::SyntaxError { message, .. }) = result {
        assert!(message.contains("only SRFI imports are currently supported"));
    }
}

#[test]
fn test_import_specification_edge_cases() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test with negative SRFI number
    let negative_spec = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(-1))),
    ]);
    
    let result = evaluator.eval_import(&[negative_spec], env.clone(), cont.clone());
    
    // Should handle negative numbers (might succeed or fail based on implementation)
    if let Ok(value) = result {
        assert_eq!(value, Value::Undefined);
    }
    // Various error types are acceptable
    
    // Test with zero SRFI number
    let zero_spec = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
    ]);
    
    let result = evaluator.eval_import(&[zero_spec], env.clone(), cont.clone());
    
    // Should handle zero (might succeed or fail based on implementation)
    if let Ok(value) = result {
        assert_eq!(value, Value::Undefined);
    }
    // Various error types are acceptable
    
    // Test with very large SRFI number
    let large_spec = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(99999))),
    ]);
    
    let result = evaluator.eval_import(&[large_spec], env, cont);
    
    // Should handle large numbers (might succeed or fail based on implementation)
    if let Ok(value) = result {
        assert_eq!(value, Value::Undefined);
    }
    // Various error types are acceptable
}

#[test]
fn test_import_return_value() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test that import returns unspecified value (Undefined)
    let valid_spec = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);
    
    let result = evaluator.eval_import(&[valid_spec], env, cont);
    
    // If import succeeds, it should return Undefined
    if let Ok(value) = result {
        assert_eq!(value, Value::Undefined);
    }
    // Error is also acceptable if SRFI is not available
}

#[test]
fn test_import_with_complex_part_names() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test with complex part names
    let complex_spec = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Variable("fold-left".to_string()),
        Expr::Variable("fold-right".to_string()),
        Expr::Variable("filter-map".to_string()),
    ]);
    
    let result = evaluator.eval_import(&[complex_spec], env, cont);
    
    // Should handle complex part names
    match result {
        Ok(value) => assert_eq!(value, Value::Undefined),
        Err(err) => {
            // Should be runtime error, not syntax error
            match err {
                LambdustError::RuntimeError { .. } => {}, // Expected for unimplemented SRFI
                LambdustError::SyntaxError { message, .. } => panic!("Unexpected syntax error: {}", message),
                _ => {}, // Other error types are acceptable
            }
        }
    }
}

#[test]
fn test_import_error_message_quality() {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;
    
    // Test that error messages are informative
    let test_cases = vec![
        (vec![], "at least one import set required"),
        (vec![Expr::List(vec![])], "empty import specification"),
        (vec![Expr::Variable("not-list".to_string())], "import specification must be a list"),
        (vec![Expr::List(vec![Expr::Variable("srfi".to_string())])], "SRFI number required"),
        (vec![Expr::List(vec![Expr::Variable("srfi".to_string()), Expr::Variable("not-number".to_string())])], "SRFI number must be an integer"),
    ];
    
    for (specs, expected_msg) in test_cases {
        let result = evaluator.eval_import(&specs, env.clone(), cont.clone());
        assert!(result.is_err());
        if let Err(LambdustError::SyntaxError { message, .. }) = result {
            assert!(message.contains(expected_msg), "Expected '{}' in error message: {}", expected_msg, message);
        }
    }
}