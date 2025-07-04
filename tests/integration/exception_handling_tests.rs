//! Exception handling system tests
//!
//! Tests for R7RS exception handling features including raise, with-exception-handler, and guard.

use lambdust::{Interpreter, Value};

#[cfg(test)]
mod exception_tests {
    use super::*;

    #[test]
    fn test_raise_basic() {
        let mut interpreter = Interpreter::new();

        // Test raise with string
        let result = interpreter.eval("(raise \"This is an error\")");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(
                e.to_string()
                    .contains("Uncaught exception: \"This is an error\"")
            );
        }

        // Test raise with symbol
        let result = interpreter.eval("(raise 'error-symbol)");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Uncaught exception: error-symbol"));
        }
    }

    #[test]
    fn test_with_exception_handler_syntax() {
        let mut interpreter = Interpreter::new();

        // Test basic syntax parsing
        let result =
            interpreter.eval("(with-exception-handler (lambda (obj) 'handled) (lambda () 42))");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from(42i64));
    }

    #[test]
    fn test_guard_syntax_basic() {
        let mut interpreter = Interpreter::new();

        // Test guard with no exception (should return body result)
        let result =
            interpreter.eval("(guard (e ((string? e) 'string-error) (else 'other-error)) 42)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from(42i64));

        // Test guard with multiple clauses
        let result = interpreter.eval(
            r#"
            (guard (e 
                ((number? e) 'number-error)
                ((string? e) 'string-error)
                ((symbol? e) 'symbol-error)
                (else 'unknown-error))
                100)
        "#,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from(100i64));
    }

    #[test]
    fn test_guard_else_clause() {
        let mut interpreter = Interpreter::new();

        // Test guard with else clause
        let result = interpreter.eval(
            r#"
            (guard (e (else 'always-handled))
                (+ 1 2 3))
        "#,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from(6i64));
    }

    #[test]
    fn test_guard_syntax_errors() {
        let mut interpreter = Interpreter::new();

        // Test guard with empty specification
        let result = interpreter.eval("(guard () 42)");
        assert!(result.is_err());

        // Test guard with non-identifier condition variable
        let result = interpreter.eval("(guard (123 (else 'error)) 42)");
        assert!(result.is_err());

        // Test guard with too few arguments
        let result = interpreter.eval("(guard (e))");
        assert!(result.is_err());
    }

    #[test]
    fn test_exception_handling_arity_errors() {
        let mut interpreter = Interpreter::new();

        // Test raise with wrong arity
        let result = interpreter.eval("(raise)");
        assert!(result.is_err());

        let result = interpreter.eval("(raise 'a 'b)");
        assert!(result.is_err());

        // Test with-exception-handler with wrong arity
        let result = interpreter.eval("(with-exception-handler)");
        assert!(result.is_err());

        let result = interpreter.eval("(with-exception-handler (lambda (x) x))");
        assert!(result.is_err());

        let result =
            interpreter.eval("(with-exception-handler (lambda (x) x) (lambda () 1) extra)");
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_guard_expressions() {
        let mut interpreter = Interpreter::new();

        // Test guard with complex conditions
        let result = interpreter.eval(
            r#"
            (guard (e 
                ((and (string? e) (string=? e "test")) 'test-string)
                ((and (number? e) (> e 10)) 'big-number)
                (else 'other))
                (string-append "hello" " world"))
        "#,
        );
        assert!(result.is_ok());
        // Should return the result of string-append
        assert_eq!(result.unwrap(), Value::String("hello world".to_string()));
    }

    #[test]
    fn test_nested_exception_structures() {
        let mut interpreter = Interpreter::new();

        // Test nested guard expressions
        let result = interpreter.eval(
            r#"
            (guard (outer-e 
                (else 'outer-handled))
                (guard (inner-e 
                    (else 'inner-handled))
                    42))
        "#,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from(42i64));
    }
}

#[cfg(test)]
mod exception_integration_tests {
    use super::*;

    #[test]
    fn test_exception_with_user_defined_functions() {
        let mut interpreter = Interpreter::new();

        // Define a function that might raise an exception
        interpreter
            .eval("(define (safe-divide a b) (if (= b 0) (raise 'division-by-zero) (/ a b)))")
            .unwrap();

        // Test normal case
        let result = interpreter.eval("(safe-divide 10 2)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from(5i64));

        // Test error case
        let result = interpreter.eval("(safe-divide 10 0)");
        assert!(result.is_err());
    }

    #[test]
    fn test_exception_with_control_structures() {
        let mut interpreter = Interpreter::new();

        // Test exception handling with if
        let result = interpreter.eval(
            r#"
            (if #t
                (guard (e (else 'handled)) 42)
                (raise 'never-reached))
        "#,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from(42i64));

        // Test exception handling with begin
        let result = interpreter.eval(
            r#"
            (begin
                (define x 10)
                (guard (e (else 'handled)) (+ x 5)))
        "#,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from(15i64));
    }

    #[test]
    fn test_exception_with_lambda() {
        let mut interpreter = Interpreter::new();

        // Test exception handling with lambda
        let result = interpreter.eval(
            r#"
            ((lambda (x)
                (guard (e (else 'handled))
                    (* x x)))
             7)
        "#,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from(49i64));
    }
}
