//! R7RS Exception Handling Tests
//!
//! Tests for R7RS-small exception handling including:
//! - Error raising and handling (raise, error)
//! - Exception guard forms (guard)
//! - Exception object predicates and accessors
//! - Error message and irritant handling
//! - Control flow during exception handling
//!
//! This module tests exception handling operations
//! required by R7RS-small.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value, utils::intern_symbol};
use std::sync::Arc;

/// Run all exception handling tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS Exception Handling tests...");
    
    if suite.skip_if_unimplemented("exception handling") {
        println!("⚠ Skipping exception handling tests (not implemented)");
        return Ok(());
    }
    
    test_error_raising(suite)?;
    test_guard_forms(suite)?;
    test_exception_objects(suite)?;
    test_error_procedures(suite)?;
    test_exception_control_flow(suite)?;
    test_nested_exception_handling(suite)?;
    test_exception_edge_cases(suite)?;
    
    println!("✓ Exception handling tests passed");
    Ok(())
}

/// Test error raising with raise
fn test_error_raising(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Basic raise - should be caught by guard
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else 'caught))
          (raise 'an-error))
    "#, Value::symbol(intern_symbol("caught")))?;  // 'caught
    
    // raise with different types of objects
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else condition))
          (raise 42))
    "#, Value::Literal(Literal::integer(42)))?;
    
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else condition))
          (raise "error message"))
    "#, Value::Literal(Literal::String("error message".to_string())))?;
    
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else condition))
          (raise '(error-type details)))
    "#, Value::Pair(
        Arc::new(Value::symbol(intern_symbol("error-type"))),  // 'error-type
        Arc::new(Value::Pair(
            Arc::new(Value::symbol(intern_symbol("details"))),  // 'details
            Arc::new(Value::Nil)
        ))
    ))?;
    
    Ok(())
}

/// Test guard forms and exception handling
fn test_guard_forms(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Basic guard with specific condition tests
    suite.assert_eval_eq(r#"
        (guard (condition
                 ((eq? condition 'specific) 'matched-specific)
                 ((eq? condition 'other) 'matched-other)
                 (else 'matched-else))
          (raise 'specific))
    "#, Value::symbol(intern_symbol("matched-specific")))?;  // 'matched-specific
    
    suite.assert_eval_eq(r#"
        (guard (condition
                 ((eq? condition 'specific) 'matched-specific)
                 ((eq? condition 'other) 'matched-other)
                 (else 'matched-else))
          (raise 'other))
    "#, Value::symbol(intern_symbol("matched-other")))?;  // 'matched-other
    
    suite.assert_eval_eq(r#"
        (guard (condition
                 ((eq? condition 'specific) 'matched-specific)
                 ((eq? condition 'other) 'matched-other)
                 (else 'matched-else))
          (raise 'unknown))
    "#, Value::symbol(intern_symbol("matched-else")))?;  // 'matched-else
    
    // Guard with => clause
    suite.assert_eval_eq(r#"
        (guard (condition
                 ((assq 'a condition) => cdr)
                 (else 'no-match))
          (raise (list (cons 'a 42) (cons 'b 24))))
    "#, Value::Pair(
        Arc::new(Value::Literal(Literal::integer(42))),
        Arc::new(Value::Nil)
    ))?;  // '(42)
    
    // Guard with no matching clause (re-raises)
    suite.assert_eval_eq(r#"
        (guard (outer-condition
                 (else outer-condition))
          (guard (inner-condition
                   ((eq? inner-condition 'never) 'never-matches))
            (raise 'uncaught)))
    "#, Value::symbol(intern_symbol("uncaught")))?;  // 'uncaught (re-raised to outer guard)
    
    // Guard with normal completion (no exception)
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else 'error))
          (+ 1 2 3))
    "#, Value::Literal(Literal::integer(6)))?;  // Normal result
    
    Ok(())
}

/// Test exception object predicates and structure
fn test_exception_objects(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Test with error objects (if implemented)
    if !suite.skip_if_unimplemented("error objects") {
        // error? predicate
        suite.assert_eval_true(r#"
            (guard (condition
                     (else (error? condition)))
              (error "test error"))
        "#)?;
        
        suite.assert_eval_false("(error? 42)")?;
        suite.assert_eval_false("(error? 'symbol)")?;
        suite.assert_eval_false("(error? \"string\")")?;
        
        // error-object? predicate
        suite.assert_eval_true(r#"
            (guard (condition
                     (else (error-object? condition)))
              (error "test error"))
        "#)?;
        
        // error-object-message
        suite.assert_eval_eq(r#"
            (guard (condition
                     (else (error-object-message condition)))
              (error "test message"))
        "#, Value::Literal(Literal::String("test message".to_string())))?;
        
        // error-object-irritants
        let result = suite.eval(r#"
            (guard (condition
                     (else (error-object-irritants condition)))
              (error "test" 'irritant1 'irritant2))
        "#)?;
        // Should be a list containing 'irritant1 and 'irritant2
        suite.assert_eval_eq("(length (guard (condition (else (error-object-irritants condition))) (error \"test\" 'a 'b)))", 
                           Value::Literal(Literal::integer(2)))?;
    }
    
    // read-error? and file-error? (if implemented)
    if !suite.skip_if_unimplemented("specific error types") {
        // These would typically be thrown by read and file operations
        suite.assert_eval_false("(read-error? 'not-an-error)")?;
        suite.assert_eval_false("(file-error? 'not-an-error)")?;
    }
    
    Ok(())
}

/// Test error procedure
fn test_error_procedures(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // error procedure - basic usage
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else 'caught))
          (error "This is an error"))
    "#, Value::symbol(intern_symbol("caught")))?;  // 'caught
    
    // error with message and irritants
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else 'caught))
          (error "Error with irritants" 'value1 'value2))
    "#, Value::symbol(intern_symbol("caught")))?;  // 'caught
    
    // error in different contexts
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else condition))
          (if #t
              (error "conditional error")
              'never-reached))
    "#, Value::Literal(Literal::String("conditional error".to_string())))?;  // Error object or message
    
    // error in procedure
    suite.eval(r#"
        (define (failing-procedure x)
          (if (< x 0)
              (error "negative argument" x)
              (* x x)))
    "#)?;
    
    suite.assert_eval_eq("(failing-procedure 5)", Value::Literal(Literal::integer(25)))?;
    
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else 'caught-in-procedure))
          (failing-procedure -3))
    "#, Value::symbol(intern_symbol("caught")))?;  // 'caught-in-procedure
    
    Ok(())
}

/// Test exception control flow
fn test_exception_control_flow(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Exception propagation through call stack
    suite.eval(r#"
        (define (level3)
          (error "deep error"))
          
        (define (level2)
          (level3)
          'never-returned)
          
        (define (level1)
          (level2)
          'never-returned)
    "#)?;
    
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else 'caught-from-deep))
          (level1))
    "#, Value::symbol(intern_symbol("caught")))?;  // 'caught-from-deep
    
    // Exception in tail position
    suite.eval(r#"
        (define (tail-error)
          (error "tail error"))
    "#)?;
    
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else 'tail-caught))
          (tail-error))
    "#, Value::symbol(intern_symbol("tail-caught")))?;  // 'tail-caught
    
    // Exception vs normal return
    suite.eval(r#"
        (define (maybe-error flag)
          (if flag
              (error "conditional error")
              'normal-return))
    "#)?;
    
    suite.assert_eval_eq("(maybe-error #f)", Value::symbol(intern_symbol("normal-return")))?;  // 'normal-return
    
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else 'error-caught))
          (maybe-error #t))
    "#, Value::symbol(intern_symbol("error-caught")))?;  // 'error-caught
    
    // Exception during evaluation of guard test
    if !suite.skip_if_unimplemented("complex guard expressions") {
        suite.assert_eval_eq(r#"
            (guard (condition
                     ((error "error in test") 'never)
                     (else 'outer-else))
              (raise 'original-error))
        "#, Value::symbol(intern_symbol("caught")))?;  // Should catch the error from the test
    }
    
    Ok(())
}

/// Test nested exception handling
fn test_nested_exception_handling(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Nested guard forms
    suite.assert_eval_eq(r#"
        (guard (outer-condition
                 (else (list 'outer outer-condition)))
          (guard (inner-condition
                   ((eq? inner-condition 'inner-error) 'inner-caught)
                   (else 'inner-else))
            (raise 'inner-error)))
    "#, Value::symbol(intern_symbol("inner-caught")))?;  // 'inner-caught
    
    suite.assert_eval_eq(r#"
        (guard (outer-condition
                 (else (list 'outer outer-condition)))
          (guard (inner-condition
                   ((eq? inner-condition 'inner-error) 'inner-caught)
                   (else 'inner-else))
            (raise 'outer-error)))
    "#, Value::Pair(
        Arc::new(Value::symbol(intern_symbol("outer"))),  // 'outer
        Arc::new(Value::Pair(
            Arc::new(Value::symbol(intern_symbol("outer-error"))),  // 'outer-error
            Arc::new(Value::Nil)
        ))
    ))?;
    
    // Exception in guard handler
    suite.assert_eval_eq(r#"
        (guard (outer-condition
                 (else 'outer-caught))
          (guard (inner-condition
                   (else (error "error in handler")))
            (raise 'original)))
    "#, Value::symbol(intern_symbol("outer-caught")))?;  // 'outer-caught
    
    // Multiple exceptions
    suite.eval("(define error-count 0)")?;
    suite.eval(r#"
        (define (count-and-error)
          (set! error-count (+ error-count 1))
          (error "counted error"))
    "#)?;
    
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else 'first-caught))
          (count-and-error))
    "#, Value::symbol(intern_symbol("first-caught")))?;  // 'first-caught
    
    suite.assert_eval_eq("error-count", Value::Literal(Literal::integer(1)))?;
    
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else 'second-caught))
          (count-and-error))
    "#, Value::symbol(intern_symbol("second-caught")))?;  // 'second-caught
    
    suite.assert_eval_eq("error-count", Value::Literal(Literal::integer(2)))?;
    
    Ok(())
}

/// Test exception handling edge cases
fn test_exception_edge_cases(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Raising #f (valid in R7RS)
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else condition))
          (raise #f))
    "#, Value::Literal(Literal::Boolean(false)))?;
    
    // Raising complex objects
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else (car condition)))
          (raise '(error-type "message" (details here))))
    "#, Value::symbol(intern_symbol("error-type")))?;  // 'error-type
    
    // Exception during exception handling (implementation-dependent)
    if !suite.skip_if_unimplemented("exception during handling") {
        // This behavior is implementation-specific
        // Some implementations might terminate, others might have special handling
        
        suite.eval(r#"
            (define (problematic-handler condition)
              (/ 1 0))  ; Division by zero
        "#)?;
        
        // The behavior here depends on implementation
        // We just test that it doesn't crash the system
        let _result = suite.eval(r#"
            (guard (condition
                     (else 'final-catch))
              (guard (condition
                       (else (problematic-handler condition)))
                (raise 'original-error)))
        "#);
    }
    
    // Empty guard (no exception raised)
    suite.assert_eval_eq(r#"
        (guard (condition
                 (else 'never-used))
          42)
    "#, Value::Literal(Literal::integer(42)))?;
    
    // Type errors in guard expressions
    suite.assert_eval_error(r#"
        (guard condition  ; Missing parentheses
          (+ 1 2))
    "#)?;
    
    suite.assert_eval_error(r#"
        (guard (condition)  ; No clauses
          (+ 1 2))
    "#)?;
    
    // Arity errors
    suite.assert_eval_error("(raise)")?;           // raise needs an argument
    suite.assert_eval_error("(raise 'a 'b)")?;     // raise takes exactly one argument
    suite.assert_eval_error("(error)")?;           // error needs at least a message
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exception_handling_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("Exception handling tests should pass");
    }
    
    #[test]
    fn test_error_raising_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_error_raising(&mut suite).expect("Error raising tests should pass");
    }
    
    #[test]
    fn test_guard_forms_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_guard_forms(&mut suite).expect("Guard form tests should pass");
    }
    
    #[test]
    fn test_error_procedures_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_error_procedures(&mut suite).expect("Error procedure tests should pass");
    }
    
    #[test]
    fn test_nested_exception_handling_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_nested_exception_handling(&mut suite).expect("Nested exception handling tests should pass");
    }
}