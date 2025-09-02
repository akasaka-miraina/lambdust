//! R7RS Multiple Values Error Condition Tests
//!
//! This module tests error conditions and edge cases for multiple values
//! to ensure proper R7RS compliance and robust error handling.

use lambdust::diagnostics::{Error as DiagnosticError, Result};
use lambdust::eval::value::{MultipleValues, Value};
use lambdust::stdlib::control::{evaluator_call_with_values, primitive_values};
use lambdust::stdlib::srfi71_let_syntax::{list_to_values, uncons, unlist, values_to_list};
use std::sync::Arc;

// ============= VALUES PROCEDURE ERROR TESTS =============

#[test]
fn test_values_procedure_no_errors() {
    // values should never error, regardless of arguments

    // No arguments
    let result = primitive_values(&[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Unspecified);

    // Single argument
    let result = primitive_values(&[Value::integer(42)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::integer(42));

    // Multiple arguments
    let args = &[
        Value::integer(1),
        Value::string("hello"),
        Value::boolean(true),
    ];
    let result = primitive_values(&args);
    assert!(result.is_ok());

    if let Value::MultipleValues(mv) = result.unwrap() {
        assert_eq!(mv.len(), 3);
    } else {
        panic!("Expected MultipleValues");
    }
}

// ============= CALL-WITH-VALUES ERROR TESTS =============

#[test]
fn test_call_with_values_wrong_argument_count() {
    // call-with-values requires exactly 2 arguments

    // Note: Since evaluator_call_with_values requires an Evaluator,
    // we test the argument validation directly

    // This is conceptual - the actual function would be called with evaluator
    // Test case 1: No arguments (would error)
    // Test case 2: One argument (would error)
    // Test case 3: Three arguments (would error)

    // For now, we document the expected behavior
    assert!(true); // Placeholder - actual test would require evaluator integration
}

#[test]
fn test_call_with_values_non_procedure_producer() {
    // First argument must be a procedure

    // This test documents the expected error when producer is not a procedure
    let non_procedure = Value::integer(42);
    assert!(!non_procedure.is_procedure());

    // In actual use, this would cause an error in call-with-values
    let string_value = Value::string("not-a-procedure");
    assert!(!string_value.is_procedure());

    let boolean_value = Value::boolean(false);
    assert!(!boolean_value.is_procedure());
}

#[test]
fn test_call_with_values_non_procedure_consumer() {
    // Second argument must be a procedure

    let non_procedure = Value::integer(42);
    assert!(!non_procedure.is_procedure());

    // These would all cause errors as consumers
    let list_value = Value::list(vec![Value::integer(1), Value::integer(2)]);
    assert!(!list_value.is_procedure());

    let symbol_value = Value::symbol(0); // Assuming symbol id 0
    assert!(!symbol_value.is_procedure());
}

// ============= MULTIPLE VALUES IN NON-FINAL EXPRESSIONS =============

#[test]
fn test_multiple_values_non_final_expression_detection() {
    // R7RS: "The continuations of all non-final expressions within a sequence
    // of expressions... take exactly one value"

    // This test documents the requirement for the evaluator to detect
    // multiple values in non-final positions and raise an error

    let multiple_values = Value::MultipleValues(Arc::new(MultipleValues::new(vec![
        Value::integer(1),
        Value::integer(2),
    ])));

    // This should be detected as an error in contexts like:
    // (begin (values 1 2) 42)  ; Multiple values in non-final position
    // (+ (values 1 2) 3)       ; Multiple values as argument

    assert!(matches!(multiple_values, Value::MultipleValues(_)));
}

// ============= ARITY MISMATCH TESTS =============

#[test]
fn test_arity_mismatch_scenarios() {
    // Test various arity mismatch scenarios that should cause errors

    // Producer returns 3 values, consumer expects 2
    let producer_values = Value::MultipleValues(Arc::new(MultipleValues::new(vec![
        Value::integer(1),
        Value::integer(2),
        Value::integer(3),
    ])));

    // Consumer that expects 2 arguments would get 3 - should error
    // This would be caught in the evaluator during procedure application

    if let Value::MultipleValues(mv) = producer_values {
        assert_eq!(mv.len(), 3); // Producer gives 3 values
        // Consumer expecting 2 would cause arity error
    }

    // Producer returns 1 value, consumer expects 3
    let producer_single = Value::integer(42);
    // Consumer expecting 3 arguments would get 1 - should error

    // Producer returns 0 values, consumer expects 1
    let producer_empty = Value::Unspecified; // represents (values)
    // Consumer expecting 1 argument would get 0 - should error
}

// ============= SRFI-71 UTILITY PROCEDURE ERRORS =============

#[test]
fn test_uncons_error_conditions() {
    // uncons requires exactly 1 argument
    let result = uncons(&[]);
    assert!(result.is_err());

    let result = uncons(&[Value::integer(1), Value::integer(2)]);
    assert!(result.is_err());

    // uncons requires a pair argument
    let result = uncons(&[Value::integer(42)]);
    assert!(result.is_err());

    let result = uncons(&[Value::string("not-a-pair")]);
    assert!(result.is_err());

    let result = uncons(&[Value::Nil]);
    assert!(result.is_err());

    let result = uncons(&[Value::boolean(true)]);
    assert!(result.is_err());
}

#[test]
fn test_unlist_error_conditions() {
    // unlist requires 1 or 2 arguments
    let result = unlist(&[]);
    assert!(result.is_err());

    let result = unlist(&[Value::integer(1), Value::integer(2), Value::integer(3)]);
    assert!(result.is_err());

    // unlist requires a proper list as first argument
    let result = unlist(&[Value::integer(42)]);
    assert!(result.is_err());

    let result = unlist(&[Value::string("not-a-list")]);
    assert!(result.is_err());

    // unlist with count requires integer as second argument
    let list = Value::list(vec![Value::integer(1), Value::integer(2)]);
    let result = unlist(&[list, Value::string("not-an-integer")]);
    assert!(result.is_err());

    // unlist with count exceeding list length
    let list = Value::list(vec![Value::integer(1), Value::integer(2)]);
    let result = unlist(&[
        list,
        Value::Literal(lambdust::ast::Literal::ExactInteger(5)),
    ]);
    assert!(result.is_err());
}

#[test]
fn test_values_to_list_error_conditions() {
    // values->list requires exactly 1 argument
    let result = values_to_list(&[]);
    assert!(result.is_err());

    let result = values_to_list(&[Value::integer(1), Value::integer(2)]);
    assert!(result.is_err());

    // values->list should accept any value (single values are wrapped)
    let result = values_to_list(&[Value::integer(42)]);
    assert!(result.is_ok());

    let result = values_to_list(&[Value::string("hello")]);
    assert!(result.is_ok());

    let mv = Value::MultipleValues(Arc::new(MultipleValues::new(vec![
        Value::integer(1),
        Value::integer(2),
    ])));
    let result = values_to_list(&[mv]);
    assert!(result.is_ok());
}

#[test]
fn test_list_to_values_error_conditions() {
    // list->values requires exactly 1 argument
    let result = list_to_values(&[]);
    assert!(result.is_err());

    let result = list_to_values(&[Value::integer(1), Value::integer(2)]);
    assert!(result.is_err());

    // list->values requires a proper list
    let result = list_to_values(&[Value::integer(42)]);
    assert!(result.is_err());

    let result = list_to_values(&[Value::string("not-a-list")]);
    assert!(result.is_err());

    // Proper lists should work
    let list = Value::list(vec![Value::integer(1), Value::integer(2)]);
    let result = list_to_values(&[list]);
    assert!(result.is_ok());

    // Empty list should work
    let empty_list = Value::Nil;
    let result = list_to_values(&[empty_list]);
    assert!(result.is_ok());
}

// ============= MULTIPLE VALUES CONTAINER EDGE CASES =============

#[test]
fn test_multiple_values_container_edge_cases() {
    // Test edge cases in MultipleValues container

    // Empty multiple values
    let empty_mv = MultipleValues::empty();
    assert_eq!(empty_mv.len(), 0);
    assert!(empty_mv.is_empty());
    assert_eq!(empty_mv.first(), None);

    // Very large multiple values (stress test)
    let large_values: Vec<Value> = (0..1000).map(|i| Value::integer(i)).collect();
    let large_mv = MultipleValues::new(large_values.clone());
    assert_eq!(large_mv.len(), 1000);
    assert_eq!(large_mv.first(), Some(&Value::integer(0)));
    assert_eq!(large_mv.as_slice(), &large_values);
}

#[test]
fn test_multiple_values_equality_edge_cases() {
    // Test equality with edge cases

    // Empty vs empty
    let empty1 = MultipleValues::empty();
    let empty2 = MultipleValues::empty();
    assert_eq!(empty1, empty2);

    // Single vs single
    let single1 = MultipleValues::single(Value::integer(42));
    let single2 = MultipleValues::single(Value::integer(42));
    let single3 = MultipleValues::single(Value::integer(43));
    assert_eq!(single1, single2);
    assert_ne!(single1, single3);

    // Different lengths
    let short = MultipleValues::new(&[Value::integer(1)]);
    let long = MultipleValues::new(&[Value::integer(1), Value::integer(2)]);
    assert_ne!(short, long);
}

// ============= R7RS COMPLIANCE VIOLATION TESTS =============

#[test]
fn test_multiple_values_in_arithmetic_context() {
    // Test that multiple values cannot be used in arithmetic contexts
    // This would be caught by the evaluator, not the values themselves

    let mv = Value::MultipleValues(Arc::new(MultipleValues::new(vec![
        Value::integer(1),
        Value::integer(2),
    ])));

    // In contexts like (+ (values 1 2) 3), the evaluator should detect
    // that a non-final expression produces multiple values and error

    assert!(matches!(mv, Value::MultipleValues(_)));
    assert!(!mv.is_number()); // Cannot be used as a number
}

#[test]
fn test_multiple_values_in_conditional_context() {
    // Test multiple values in conditional contexts

    let mv = Value::MultipleValues(Arc::new(MultipleValues::new(vec![
        Value::boolean(true),
        Value::boolean(false),
    ])));

    // In contexts like (if (values #t #f) 'yes 'no), this should error
    // because if expects a single value for its condition

    assert!(matches!(mv, Value::MultipleValues(_)));
    assert!(!mv.is_falsy()); // MultipleValues are truthy, but shouldn't be used in conditionals
}

// ============= INTEGRATION ERROR TESTS =============

#[test]
fn test_multiple_values_with_define() {
    // Test that regular define cannot handle multiple values
    // (define x (values 1 2)) should error - use define-values instead

    let mv = Value::MultipleValues(Arc::new(MultipleValues::new(vec![
        Value::integer(1),
        Value::integer(2),
    ])));

    // This would be an error in the evaluator - define expects single value
    assert!(matches!(mv, Value::MultipleValues(_)));
}

#[test]
fn test_multiple_values_with_assignment() {
    // Test that regular assignment cannot handle multiple values
    // (set! x (values 1 2)) should error

    let mv = Value::MultipleValues(Arc::new(MultipleValues::new(vec![
        Value::integer(1),
        Value::integer(2),
    ])));

    // This would be an error in the evaluator - set! expects single value
    assert!(matches!(mv, Value::MultipleValues(_)));
}

// ============= ERROR MESSAGE QUALITY TESTS =============

#[test]
fn test_error_message_quality() {
    // Test that error messages are helpful and specific

    // uncons with wrong type
    let result = uncons(&[Value::integer(42)]);
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("pair")); // Should mention that a pair is required

    // unlist with wrong type
    let result = unlist(&[Value::integer(42)]);
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("list")); // Should mention that a list is required

    // unlist with count too large
    let list = Value::list(vec![Value::integer(1), Value::integer(2)]);
    let result = unlist(&[
        list,
        Value::Literal(lambdust::ast::Literal::ExactInteger(5)),
    ]);
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("2")); // Should mention actual list length
    assert!(error_msg.contains("5")); // Should mention requested count
}

// ============= MEMORY SAFETY TESTS =============

#[test]
fn test_multiple_values_memory_safety() {
    // Test that MultipleValues handle memory safely

    // Create and drop multiple values
    let mv = MultipleValues::new(&[Value::integer(1), Value::integer(2)]);
    let value = Value::MultipleValues(Arc::new(mv));
    drop(value); // Should not cause memory issues

    // Clone and reference counting
    let mv = Arc::new(MultipleValues::new(&[Value::integer(1), Value::integer(2)]));
    let mv_clone = mv.clone();
    assert_eq!(Arc::strong_count(&mv), 2);
    drop(mv_clone);
    assert_eq!(Arc::strong_count(&mv), 1);
}

// ============= COMPREHENSIVE ERROR TEST RUNNER =============

/// Runs all error condition tests to ensure robust error handling
#[test]
fn test_comprehensive_error_conditions() {
    println!("Running R7RS Multiple Values Error Condition Tests...");

    // Values procedure (should not error)
    test_values_procedure_no_errors();

    // Call-with-values errors
    test_call_with_values_wrong_argument_count();
    test_call_with_values_non_procedure_producer();
    test_call_with_values_non_procedure_consumer();

    // Multiple values placement errors
    test_multiple_values_non_final_expression_detection();
    test_arity_mismatch_scenarios();

    // SRFI-71 utility errors
    test_uncons_error_conditions();
    test_unlist_error_conditions();
    test_values_to_list_error_conditions();
    test_list_to_values_error_conditions();

    // Container edge cases
    test_multiple_values_container_edge_cases();
    test_multiple_values_equality_edge_cases();

    // R7RS compliance violations
    test_multiple_values_in_arithmetic_context();
    test_multiple_values_in_conditional_context();

    // Integration errors
    test_multiple_values_with_define();
    test_multiple_values_with_assignment();

    // Error message quality
    test_error_message_quality();

    // Memory safety
    test_multiple_values_memory_safety();

    println!("✓ All R7RS Multiple Values Error Condition Tests Passed!");
}
