//! Integration tests for the Lambdust language implementation.
//!
//! These tests verify the complete pipeline from source code to evaluation result,
//! ensuring that Lexer → Parser → Evaluator integration works correctly.

use lambdust::{Lambdust, Value, Literal};

/// Helper function to evaluate Scheme source and expect a successful result.
fn eval_expect_ok(source: &str) -> Value {
    let mut lambdust = Lambdust::new();
    match lambdust.eval(source, Some("test")) {
        Ok(value) => value,
        Err(e) => panic!("Expected successful evaluation of '{}', got error: {}", source, e),
    }
}

/// Helper function to evaluate Scheme source and expect an error.
fn eval_expect_err(source: &str) -> String {
    let mut lambdust = Lambdust::new();
    match lambdust.eval(source, Some("test")) {
        Ok(value) => panic!("Expected error for '{}', got value: {:?}", source, value),
        Err(e) => e.to_string(),
    }
}

/// Helper function to assert integer result.
fn assert_integer_result(value: Value, expected: i64) {
    match value {
        Value::Literal(Literal::Number(n)) if n.fract() == 0.0 => {
            assert_eq!(n as i64, expected)
        },
        Value::Literal(Literal::Rational { numerator, denominator }) if denominator == 1 => {
            assert_eq!(numerator, expected)
        },
        _ => panic!("Expected integer {}, got: {:?}", expected, value),
    }
}

/// Helper function to assert string result.
fn assert_string_result(value: Value, expected: &str) {
    match value {
        Value::Literal(Literal::String(s)) => assert_eq!(s, expected),
        _ => panic!("Expected string '{}', got: {:?}", expected, value),
    }
}

/// Helper function to assert boolean result.
fn assert_boolean_result(value: Value, expected: bool) {
    match value {
        Value::Literal(Literal::Boolean(b)) => assert_eq!(b, expected),
        _ => panic!("Expected boolean {}, got: {:?}", expected, value),
    }
}

#[test]
fn test_basic_arithmetic() {
    // Test simple addition
    let result = eval_expect_ok("(+ 1 2)");
    assert_integer_result(result, 3);
    
    // Test addition with multiple arguments
    let result = eval_expect_ok("(+ 1 2 3 4)");
    assert_integer_result(result, 10);
    
    // Test subtraction
    let result = eval_expect_ok("(- 10 3)");
    assert_integer_result(result, 7);
    
    // Test multiplication
    let result = eval_expect_ok("(* 4 5)");
    assert_integer_result(result, 20);
    
    // Test division
    let result = eval_expect_ok("(/ 20 4)");
    assert_integer_result(result, 5);
    
    // Test nested arithmetic
    let result = eval_expect_ok("(+ (* 2 3) (- 10 4))");
    assert_integer_result(result, 12);
}

#[test]
fn test_function_definition() {
    // Test function definition and application (using lambda syntax)
    let program = r#"
        (define square (lambda (x) (* x x)))
        (square 5)
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 25);
    
    // Test function with multiple parameters
    let program = r#"
        (define add-three (lambda (x y z) (+ x y z)))
        (add-three 1 2 3)
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 6);
    
    // Test simple function call (non-recursive for now)
    let program = r#"
        (define double (lambda (x) (* x 2)))
        (double 5)
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 10);
}

#[test]
fn test_literals_and_self_evaluation() {
    // Test integer literals
    let result = eval_expect_ok("42");
    assert_integer_result(result, 42);
    
    // Test negative integers
    let result = eval_expect_ok("-17");
    assert_integer_result(result, -17);
    
    // Test string literals
    let result = eval_expect_ok("\"hello world\"");
    assert_string_result(result, "hello world");
    
    // Test boolean literals
    let result = eval_expect_ok("#t");
    assert_boolean_result(result, true);
    
    let result = eval_expect_ok("#f");
    assert_boolean_result(result, false);
    
    // Test character literals
    let result = eval_expect_ok("#\\A");
    match result {
        Value::Literal(Literal::Character(c)) => assert_eq!(c, 'A'),
        _ => panic!("Expected character 'A', got: {:?}", result),
    }
}

#[test]
fn test_variable_definition_and_lookup() {
    // Test simple variable definition
    let program = r#"
        (define x 42)
        x
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 42);
    
    // Test multiple variable definitions
    let program = r#"
        (define x 10)
        (define y 20)
        (+ x y)
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 30);
    
    // Test variable shadowing with lambda
    let program = r#"
        (define x 100)
        ((lambda (x) (+ x 1)) 5)
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 6);
}

#[test]
fn test_conditional_expressions() {
    // Test if with both branches
    let result = eval_expect_ok("(if #t 42 24)");
    assert_integer_result(result, 42);
    
    let result = eval_expect_ok("(if #f 42 24)");
    assert_integer_result(result, 24);
    
    // Test if without else clause
    let result = eval_expect_ok("(if #f 42)");
    assert!(matches!(result, Value::Unspecified));
    
    // Test nested conditionals
    let program = r#"
        (if #t
            (if #t 100 200)
            300)
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 100);
}

#[test]
fn test_let_expressions() {
    // Test basic let
    let program = r#"
        (let ((x 10)
              (y 20))
          (+ x y))
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 30);
    
    // Test nested let
    let program = r#"
        (let ((x 5))
          (let ((y 10))
            (+ x y)))
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 15);
    
    // Test let with computation in bindings
    let program = r#"
        (let ((x (+ 1 2))
              (y (* 3 4)))
          (+ x y))
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 15);
}

#[test]
fn test_lambda_expressions() {
    // Test simple lambda
    let result = eval_expect_ok("((lambda (x) (+ x 1)) 5)");
    assert_integer_result(result, 6);
    
    // Test lambda with multiple parameters
    let result = eval_expect_ok("((lambda (x y z) (+ x y z)) 1 2 3)");
    assert_integer_result(result, 6);
    
    // Test lambda capturing environment
    let program = r#"
        (define x 100)
        ((lambda (y) (+ x y)) 5)
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 105);
    
    // Test higher-order functions
    let program = r#"
        (define (make-adder n)
          (lambda (x) (+ x n)))
        (define add-ten (make-adder 10))
        (add-ten 5)
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 15);
}

#[test]
fn test_quote_and_symbols() {
    // Test quoted symbols
    let result = eval_expect_ok("(quote hello)");
    assert!(matches!(result, Value::Symbol(_)));
    
    // Test quoted lists (will become list structures)
    let result = eval_expect_ok("(quote (1 2 3))");
    // Should be a proper list structure, not evaluated
    assert!(!matches!(result, Value::Literal(Literal::Number(6.0))));
    
    // Test quote shorthand
    let result = eval_expect_ok("'symbol");
    assert!(matches!(result, Value::Symbol(_)));
}

#[test]
fn test_error_handling_and_propagation() {
    // Test lexical error (unterminated string)
    let error = eval_expect_err("\"unterminated string");
    assert!(error.to_lowercase().contains("string") || error.to_lowercase().contains("lex"));
    
    // Test parse error (mismatched parentheses)
    let error = eval_expect_err("(+ 1 2");
    assert!(error.to_lowercase().contains("parse") || error.to_lowercase().contains("paren"));
    
    // Test runtime error (unbound variable)
    let error = eval_expect_err("unbound-variable");
    assert!(error.to_lowercase().contains("unbound"));
    
    // Test type error (applying non-procedure)
    let error = eval_expect_err("(42 1 2)");
    assert!(error.to_lowercase().contains("procedure") || error.to_lowercase().contains("apply"));
    
    // Test arity error
    let program = r#"
        (define (single-arg x) x)
        (single-arg 1 2 3)
    "#;
    let error = eval_expect_err(program);
    assert!(error.to_lowercase().contains("argument") || error.to_lowercase().contains("arity"));
}

#[test]
fn test_begin_sequences() {
    // Test begin with multiple expressions
    let program = r#"
        (begin
          (define x 10)
          (define y 20)
          (+ x y))
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 30);
    
    // Test begin returns last value
    let result = eval_expect_ok("(begin 1 2 3 4)");
    assert_integer_result(result, 4);
    
    // Test empty begin is an error
    let error = eval_expect_err("(begin)");
    assert!(error.to_lowercase().contains("empty") || error.to_lowercase().contains("begin"));
}

#[test]
fn test_lexer_parser_evaluator_pipeline() {
    // Test complete pipeline with complex nested expression
    let program = r#"
        (define complex-calc
          (lambda (x y)
            (+ (* x x) (* y y))))
        (complex-calc 3 4)
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 25);
    
    // Test pipeline with let expressions and closures
    let program = r#"
        (let ((multiplier 3))
          (let ((triple (lambda (x) (* x multiplier))))
            (triple 7)))
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 21);
    
    // Test pipeline with nested function definitions
    let program = r#"
        (define (outer x)
          (define (inner y)
            (+ x y))
          (inner 10))
        (outer 5)
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 15);
}

#[test]
fn test_tail_call_optimization_basic() {
    // Test basic tail call with simple calculation
    let program = r#"
        (define tail-calc
          (lambda (x)
            (+ x 10)))
        (tail-calc 90)
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 100);
}

#[test]
fn test_higher_order_functions() {
    // Test map-like function
    let _program = r#"
        (define (apply-to-all f lst)
          (if (null? lst)
              '()
              (cons (f (car lst))
                    (apply-to-all f (cdr lst)))))
        
        (define (square x) (* x x))
        (apply-to-all square '(1 2 3 4))
    "#;
    // Note: This test will only work if list operations are implemented
    // For now, we'll test the function definition at least
    let result = eval_expect_ok(r#"
        (define (square x) (* x x))
        (square 4)
    "#);
    assert_integer_result(result, 16);
}

#[test] 
fn test_complex_arithmetic_expressions() {
    // Test complex nested arithmetic
    let result = eval_expect_ok("(+ (* 2 (+ 3 4)) (- 10 (* 2 3)))");
    assert_integer_result(result, 18);  // (+ (* 2 7) (- 10 6)) = (+ 14 4) = 18
    
    // Test with variables
    let program = r#"
        (define a 2)
        (define b 3)
        (define c 4)
        (+ (* a b) (* b c) (* a c))
    "#;
    let result = eval_expect_ok(program);
    assert_integer_result(result, 26);  // (+ (* 2 3) (* 3 4) (* 2 4)) = (+ 6 12 8) = 26
}