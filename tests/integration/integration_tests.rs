//! Integration tests for Lambdust Scheme interpreter
//!
//! These tests verify the complete interpreter functionality from an external perspective,
//! testing the public API and ensuring all components work together correctly.

use lambdust::{Interpreter, Value};

#[cfg(test)]
mod interpreter_tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(+ 1 2 3)").unwrap();
        assert_eq!(result, Value::from(6i64));
    }

    #[test]
    fn test_define_and_use_variable() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("(define x 42)").unwrap();
        let result = interpreter.eval("x").unwrap();
        assert_eq!(result, Value::from(42i64));
    }

    #[test]
    fn test_lambda_and_function_call() {
        let mut interpreter = Interpreter::new();
        interpreter
            .eval("(define square (lambda (x) (* x x)))")
            .unwrap();
        let result = interpreter.eval("(square 5)").unwrap();
        assert_eq!(result, Value::from(25i64));
    }

    #[test]
    fn test_recursive_function() {
        let mut interpreter = Interpreter::new();
        interpreter
            .eval("(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))")
            .unwrap();
        let result = interpreter.eval("(factorial 5)").unwrap();
        assert_eq!(result, Value::from(120i64));
    }

    #[test]
    fn test_list_operations() {
        let mut interpreter = Interpreter::new();

        let result = interpreter.eval("(length '(1 2 3 4))").unwrap();
        assert_eq!(result, Value::from(4i64));

        let result = interpreter.eval("(car '(1 2 3))").unwrap();
        assert_eq!(result, Value::from(1i64));

        let result = interpreter.eval("(cdr '(1 2 3))").unwrap();
        // cdr returns (2 3) as a list
        assert!(matches!(result, Value::Pair(_)));
    }

    #[test]
    fn test_conditional_expressions() {
        let mut interpreter = Interpreter::new();

        let result = interpreter.eval("(if #t 42 0)").unwrap();
        assert_eq!(result, Value::from(42i64));

        let result = interpreter.eval("(if #f 42 0)").unwrap();
        assert_eq!(result, Value::from(0i64));
    }

    #[test]
    fn test_multiple_expressions() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (define x 10)
            (define y 20)
            (+ x y)
        "#;
        let result = interpreter.eval(code).unwrap();
        assert_eq!(result, Value::from(30i64));
    }
}
