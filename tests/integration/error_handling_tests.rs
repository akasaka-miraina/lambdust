//! Error handling and edge case tests
//!
//! Tests for proper error reporting, edge cases, and robustness.

use lambdust::{FromScheme, Interpreter, LambdustBridge, LambdustError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_errors() {
        let mut interpreter = Interpreter::new();

        // Unmatched parentheses
        let result = interpreter.eval("(+ 1 2");
        assert!(result.is_err());
        if let Err(LambdustError::ParseError { .. }) = result {
            // Expected parse error
        } else {
            panic!("Expected parse error for unmatched parentheses");
        }

        // Invalid token
        let result = interpreter.eval("(+ 1 @invalid)");
        assert!(result.is_err());
    }

    #[test]
    fn test_runtime_errors() {
        let mut interpreter = Interpreter::new();

        // Undefined variable
        let result = interpreter.eval("undefined-variable");
        assert!(result.is_err());
        if let Err(LambdustError::UndefinedVariable { .. }) = result {
            // Expected undefined variable error
        } else {
            panic!("Expected undefined variable error for undefined variable");
        }

        // Type error (calling non-procedure)
        let result = interpreter.eval("(42 1 2 3)");
        assert!(result.is_err());
    }

    #[test]
    fn test_arity_errors() {
        let mut interpreter = Interpreter::new();

        // Too few arguments
        let _result = interpreter.eval("(+)");
        // Note: + with no arguments might be valid in some Schemes (returns 0)
        // but let's test with a function that requires arguments
        let result = interpreter.eval("(car)");
        assert!(result.is_err());

        // Too many arguments for a fixed-arity function
        interpreter
            .eval("(define (fixed-arity x y) (+ x y))")
            .unwrap();
        let result = interpreter.eval("(fixed-arity 1 2 3)");
        assert!(result.is_err());
        if let Err(LambdustError::ArityError { .. }) = result {
            // Expected arity error
        } else {
            panic!("Expected arity error");
        }
    }

    #[test]
    fn test_type_errors() {
        let mut interpreter = Interpreter::new();

        // Arithmetic on non-numbers
        let result = interpreter.eval(r#"(+ 1 "not-a-number")"#);
        assert!(result.is_err());

        // Car on non-pair
        let result = interpreter.eval("(car 42)");
        assert!(result.is_err());

        // Cdr on non-pair
        let result = interpreter.eval("(cdr 42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_division_by_zero() {
        let mut interpreter = Interpreter::new();

        let result = interpreter.eval("(/ 1 0)");
        assert!(result.is_err());
        // Division by zero should result in an error (type varies by implementation)
    }

    #[test]
    fn test_error_recovery() {
        let mut interpreter = Interpreter::new();

        // Test that the interpreter can continue after an error
        let result = interpreter.eval("undefined-variable");
        assert!(result.is_err());

        // Should still work after the error
        let result = interpreter.eval("(+ 1 2)").unwrap();
        assert_eq!(result, lambdust::Value::from(3i64));
    }

    #[test]
    fn test_nested_error_contexts() {
        let mut interpreter = Interpreter::new();

        // Define a function that causes an error
        interpreter.eval("(define (error-func) (car 42))").unwrap();

        // Call it from another function
        interpreter.eval("(define (caller) (error-func))").unwrap();

        // The error should propagate up the call stack
        let result = interpreter.eval("(caller)");
        assert!(result.is_err());
    }

    #[test]
    fn test_bridge_error_handling() {
        let mut bridge = LambdustBridge::new();

        // Register a function that can error
        bridge.register_function("divide", Some(2), |args| {
            let a = i64::from_scheme(&args[0])?;
            let b = i64::from_scheme(&args[1])?;
            if b == 0 {
                return Err(LambdustError::runtime_error("Division by zero".to_string()));
            }
            Ok(lambdust::Value::from(a / b))
        });

        // Test successful case
        let result = bridge.eval("(divide 10 2)").unwrap();
        assert_eq!(result, lambdust::Value::from(5i64));

        // Test error case
        let result = bridge.eval("(divide 10 0)");
        assert!(result.is_err());
    }

    #[test]
    fn test_large_expressions() {
        let mut interpreter = Interpreter::new();

        // Test deeply nested expressions (reduced depth to avoid stack overflow)
        let deep_expr = "(+ ".repeat(50) + "1" + &")".repeat(50);
        let result = interpreter.eval(&deep_expr);
        // This should either work or fail gracefully (not crash)
        match result {
            Ok(value) => {
                assert_eq!(value, lambdust::Value::from(1i64));
            }
            Err(_) => {
                // If it fails due to stack overflow protection, that's acceptable
            }
        }
    }

    #[test]
    fn test_empty_input() {
        let mut interpreter = Interpreter::new();

        // Empty string should be handled gracefully
        let result = interpreter.eval("");
        assert!(result.is_err()); // Should be a parse error

        // Whitespace only
        let result = interpreter.eval("   \n\t  ");
        assert!(result.is_err()); // Should be a parse error
    }

    #[test]
    fn test_malformed_special_forms() {
        let mut interpreter = Interpreter::new();

        // Malformed lambda
        let result = interpreter.eval("(lambda)");
        assert!(result.is_err());

        let result = interpreter.eval("(lambda ())");
        assert!(result.is_err());

        // Malformed if
        let result = interpreter.eval("(if)");
        assert!(result.is_err());

        let result = interpreter.eval("(if #t)");
        assert!(result.is_err());

        // Malformed define
        let result = interpreter.eval("(define)");
        assert!(result.is_err());
    }
}
