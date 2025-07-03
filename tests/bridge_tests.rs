//! Bridge API integration tests
//! 
//! Tests for the host function integration and external object marshalling.
//! 
//! Note: Some tests are currently disabled due to bridge implementation issues.

use lambdust::{LambdustBridge, Value, FromScheme, ToScheme};

#[cfg(test)]
mod bridge_api_tests {
    use super::*;

    #[test]
    fn test_bridge_creation_and_basic_evaluation() {
        let mut bridge = LambdustBridge::new();
        let result = bridge.eval("(+ 1 2 3)").unwrap();
        assert_eq!(result, Value::from(6i64));
    }

    #[test]
    fn test_host_function_registration() {
        let mut bridge = LambdustBridge::new();
        
        // Register a simple host function
        bridge.register_function("double", Some(1), |args| {
            let n = i64::from_scheme(&args[0])?;
            (n * 2).to_scheme()
        });
        
        // Test the host function
        let result = bridge.eval("(double 21)").unwrap();
        assert_eq!(result, Value::from(42i64));
    }

    #[test]
    fn test_variable_definition() {
        let mut bridge = LambdustBridge::new();
        
        // Define a variable
        bridge.define("pi", Value::from(std::f64::consts::PI));
        
        // Use the variable
        let result = bridge.eval("(* pi 2)").unwrap();
        // Note: Exact floating point comparison might need adjustment
        if let Value::Number(n) = result {
            // Check if the result is approximately 6.28318
            let float_val = match n {
                lambdust::lexer::SchemeNumber::Real(f) => f,
                lambdust::lexer::SchemeNumber::Integer(i) => i as f64,
                lambdust::lexer::SchemeNumber::Rational(num, den) => num as f64 / den as f64,
                lambdust::lexer::SchemeNumber::Complex(real, _) => real, // Use real part
            };
            assert!((float_val - std::f64::consts::TAU).abs() < 0.00001);
        } else {
            panic!("Expected numeric result");
        }
    }

    #[test]
    fn test_complex_host_function() {
        let mut bridge = LambdustBridge::new();
        
        // Register a function that works with strings
        bridge.register_function("string-length-squared", Some(1), |args| {
            let s = String::from_scheme(&args[0])?;
            let len = s.len() as i64;
            (len * len).to_scheme()
        });
        
        let result = bridge.eval(r#"(string-length-squared "hello")"#).unwrap();
        assert_eq!(result, Value::from(25i64)); // "hello" has 5 characters, 5^2 = 25
    }

    #[test]
    fn test_host_function_with_multiple_arguments() {
        let mut bridge = LambdustBridge::new();
        
        // Register a function that takes multiple arguments
        bridge.register_function("add-three", Some(3), |args| {
            let a = i64::from_scheme(&args[0])?;
            let b = i64::from_scheme(&args[1])?;
            let c = i64::from_scheme(&args[2])?;
            (a + b + c).to_scheme()
        });
        
        let result = bridge.eval("(add-three 1 2 3)").unwrap();
        assert_eq!(result, Value::from(6i64));
    }

    #[test]
    fn test_host_function_error_handling() {
        let mut bridge = LambdustBridge::new();
        
        // Register a function that expects a number
        bridge.register_function("must-be-number", Some(1), |args| {
            let _n = i64::from_scheme(&args[0])?; // This will fail if not a number
            Ok(Value::Boolean(true))
        });
        
        // Test with correct type
        let result = bridge.eval("(must-be-number 42)").unwrap();
        assert_eq!(result, Value::Boolean(true));
        
        // Test with incorrect type (should error)
        let result = bridge.eval(r#"(must-be-number "not-a-number")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_mixed_scheme_and_host_operations() {
        let mut bridge = LambdustBridge::new();
        
        // Register a factorial function in the host
        bridge.register_function("host-factorial", Some(1), |args| {
            let n = i64::from_scheme(&args[0])?;
            let mut result = 1i64;
            for i in 1..=n {
                result *= i;
            }
            result.to_scheme()
        });
        
        // Define a Scheme function that uses the host function
        bridge.eval("(define (combined-factorial n) (* 2 (host-factorial n)))").unwrap();
        
        let result = bridge.eval("(combined-factorial 5)").unwrap();
        assert_eq!(result, Value::from(240i64)); // 2 * 5! = 2 * 120 = 240
    }

    #[test]
    fn test_scheme_calling_host_calling_scheme() {
        let mut bridge = LambdustBridge::new();
        
        // Define a Scheme helper function
        bridge.eval("(define (add-one x) (+ x 1))").unwrap();
        
        // Register a host function that calls back to Scheme
        // Note: This would require a more sophisticated bridge implementation
        // For now, we test that the basic infrastructure works
        
        bridge.register_function("process-and-increment", Some(1), |args| {
            let n = i64::from_scheme(&args[0])?;
            // In a full implementation, this might call back to Scheme
            (n + 1).to_scheme()
        });
        
        let result = bridge.eval("(process-and-increment 41)").unwrap();
        assert_eq!(result, Value::from(42i64));
    }
}