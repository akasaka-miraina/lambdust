//! SRFI 97 integration tests
//!
//! Tests for SRFI 97 functions which provide standardized access to SRFI libraries.

use lambdust::{Interpreter, Value};

#[cfg(test)]
mod srfi_97_integration_tests {
    use super::*;

    #[test]
    fn test_srfi_available_predicate() {
        let mut interpreter = Interpreter::new();

        // Test with supported SRFI
        let result = interpreter.eval("(srfi-available? 9)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(srfi-available? 45)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(srfi-available? 46)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(srfi-available? 97)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test with unsupported SRFI
        let result = interpreter.eval("(srfi-available? 999)").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_srfi_supported_ids() {
        let mut interpreter = Interpreter::new();

        let result = interpreter.eval("(srfi-supported-ids)").unwrap();
        assert!(matches!(result, Value::Vector(_)));

        if let Value::Vector(ids) = result {
            assert!(ids.len() >= 4); // Should have at least SRFIs 9, 45, 46, 97

            // Check that all IDs are numbers
            for id in &ids {
                assert!(matches!(id, Value::Number(_)));
            }
        }
    }

    #[test]
    fn test_srfi_name() {
        let mut interpreter = Interpreter::new();

        // Test SRFI 9
        let result = interpreter.eval("(srfi-name 9)").unwrap();
        assert_eq!(result, Value::String("Defining Record Types".to_string()));

        // Test SRFI 45
        let result = interpreter.eval("(srfi-name 45)").unwrap();
        assert_eq!(
            result,
            Value::String("Primitives for Expressing Iterative Lazy Algorithms".to_string())
        );

        // Test SRFI 46
        let result = interpreter.eval("(srfi-name 46)").unwrap();
        assert_eq!(
            result,
            Value::String("Basic Syntax-rules Extensions".to_string())
        );

        // Test SRFI 97
        let result = interpreter.eval("(srfi-name 97)").unwrap();
        assert_eq!(result, Value::String("SRFI Libraries".to_string()));

        // Test error case
        let result = interpreter.eval("(srfi-name 999)");
        assert!(result.is_err());
    }

    #[test]
    fn test_srfi_parts() {
        let mut interpreter = Interpreter::new();

        // Test SRFI 9 parts
        let result = interpreter.eval("(srfi-parts 9)").unwrap();
        assert!(matches!(result, Value::Vector(_)));

        if let Value::Vector(parts) = result {
            assert!(!parts.is_empty());
            // Should contain parts like "records", "types"
            assert!(
                parts
                    .iter()
                    .any(|p| matches!(p, Value::String(s) if s == "records"))
            );
        }

        // Test SRFI 97 parts
        let result = interpreter.eval("(srfi-parts 97)").unwrap();
        assert!(matches!(result, Value::Vector(_)));

        if let Value::Vector(parts) = result {
            assert!(!parts.is_empty());
            // Should contain parts like "inquiry", "available"
            assert!(
                parts
                    .iter()
                    .any(|p| matches!(p, Value::String(s) if s == "inquiry"))
            );
        }

        // Test error case
        let result = interpreter.eval("(srfi-parts 999)");
        assert!(result.is_err());
    }

    #[test]
    fn test_srfi_functions_arity() {
        let mut interpreter = Interpreter::new();

        // Test arity errors
        let result = interpreter.eval("(srfi-available?)");
        assert!(result.is_err());

        let result = interpreter.eval("(srfi-available? 9 10)");
        assert!(result.is_err());

        let result = interpreter.eval("(srfi-supported-ids 1)");
        assert!(result.is_err());

        let result = interpreter.eval("(srfi-name)");
        assert!(result.is_err());

        let result = interpreter.eval("(srfi-parts)");
        assert!(result.is_err());
    }

    #[test]
    fn test_srfi_functions_type_errors() {
        let mut interpreter = Interpreter::new();

        // Test type errors
        let result = interpreter.eval(r#"(srfi-available? "not-a-number")"#);
        assert!(result.is_err());

        let result = interpreter.eval("(srfi-available? #t)");
        assert!(result.is_err());

        let result = interpreter.eval(r#"(srfi-name "not-a-number")"#);
        assert!(result.is_err());

        let result = interpreter.eval("(srfi-parts '())");
        assert!(result.is_err());
    }

    #[test]
    fn test_srfi_functions_with_arithmetic() {
        let mut interpreter = Interpreter::new();

        // Test with arithmetic expressions
        let result = interpreter.eval("(srfi-available? (+ 8 1))").unwrap();
        assert_eq!(result, Value::Boolean(true)); // 8 + 1 = 9, which is supported

        let result = interpreter.eval("(srfi-name (* 9 1))").unwrap();
        assert_eq!(result, Value::String("Defining Record Types".to_string())); // 9 * 1 = 9

        // Test with floating point that converts to integer
        let result = interpreter.eval("(srfi-available? 9.0)").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_srfi_97_self_reference() {
        let mut interpreter = Interpreter::new();

        // SRFI 97 should be able to report information about itself
        let result = interpreter.eval("(srfi-available? 97)").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval("(srfi-name 97)").unwrap();
        assert_eq!(result, Value::String("SRFI Libraries".to_string()));

        let result = interpreter.eval("(srfi-parts 97)").unwrap();
        assert!(matches!(result, Value::Vector(_)));
    }
}
