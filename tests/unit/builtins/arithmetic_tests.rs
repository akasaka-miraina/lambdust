//! Unit tests for arithmetic builtin functions
//!
//! Tests all arithmetic operations for correctness, edge cases, and error handling.

use lambdust::builtins::arithmetic::register_arithmetic_functions;
use lambdust::error::LambdustError;
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Procedure, Value};
use std::collections::HashMap;

/// Helper function to get a builtin function by name
fn get_builtin(name: &str) -> Value {
    let mut builtins = HashMap::new();
    register_arithmetic_functions(&mut builtins);
    builtins.get(name).unwrap().clone()
}

/// Helper function to call a builtin function
fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value, LambdustError> {
    let func = get_builtin(name);
    if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
        func(&args)
    } else {
        panic!("Expected builtin function for {}", name);
    }
}

/// Helper to create integer value
fn int(n: i64) -> Value {
    Value::Number(SchemeNumber::Integer(n))
}

/// Helper to create real value
fn real(n: f64) -> Value {
    Value::Number(SchemeNumber::Real(n))
}

/// Helper to check if two floating point numbers are approximately equal
fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < f64::EPSILON
}

#[cfg(test)]
mod basic_arithmetic_tests {
    use super::*;

    #[test]
    fn test_addition() {
        // Basic addition
        assert_eq!(call_builtin("+", vec![int(1), int(2)]).unwrap(), int(3));
        assert_eq!(call_builtin("+", vec![int(1), int(2), int(3)]).unwrap(), int(6));
        
        // Empty addition (identity)
        assert_eq!(call_builtin("+", vec![]).unwrap(), int(0));
        
        // Single argument
        assert_eq!(call_builtin("+", vec![int(5)]).unwrap(), int(5));
        
        // Mixed integers and reals
        assert_eq!(call_builtin("+", vec![int(1), real(2.5)]).unwrap(), real(3.5));
        
        // Negative numbers
        assert_eq!(call_builtin("+", vec![int(-1), int(2)]).unwrap(), int(1));
        assert_eq!(call_builtin("+", vec![int(-5), int(-3)]).unwrap(), int(-8));
    }

    #[test]
    fn test_subtraction() {
        // Basic subtraction
        assert_eq!(call_builtin("-", vec![int(5), int(3)]).unwrap(), int(2));
        assert_eq!(call_builtin("-", vec![int(10), int(3), int(2)]).unwrap(), int(5));
        
        // Unary minus
        assert_eq!(call_builtin("-", vec![int(5)]).unwrap(), int(-5));
        assert_eq!(call_builtin("-", vec![int(-3)]).unwrap(), int(3));
        assert_eq!(call_builtin("-", vec![real(2.5)]).unwrap(), real(-2.5));
        
        // Mixed types
        assert_eq!(call_builtin("-", vec![real(5.5), int(2)]).unwrap(), real(3.5));
        
        // Negative results
        assert_eq!(call_builtin("-", vec![int(3), int(5)]).unwrap(), int(-2));
        
        // Arity error
        assert!(call_builtin("-", vec![]).is_err());
    }

    #[test]
    fn test_multiplication() {
        // Basic multiplication
        assert_eq!(call_builtin("*", vec![int(3), int(4)]).unwrap(), int(12));
        assert_eq!(call_builtin("*", vec![int(2), int(3), int(4)]).unwrap(), int(24));
        
        // Empty multiplication (identity)
        assert_eq!(call_builtin("*", vec![]).unwrap(), int(1));
        
        // Single argument
        assert_eq!(call_builtin("*", vec![int(7)]).unwrap(), int(7));
        
        // Zero multiplication
        assert_eq!(call_builtin("*", vec![int(5), int(0)]).unwrap(), int(0));
        assert_eq!(call_builtin("*", vec![int(0), int(100)]).unwrap(), int(0));
        
        // Negative numbers
        assert_eq!(call_builtin("*", vec![int(-3), int(4)]).unwrap(), int(-12));
        assert_eq!(call_builtin("*", vec![int(-2), int(-5)]).unwrap(), int(10));
        
        // Mixed types
        assert_eq!(call_builtin("*", vec![int(2), real(3.5)]).unwrap(), real(7.0));
    }

    #[test]
    fn test_division() {
        // Basic division
        assert_eq!(call_builtin("/", vec![int(12), int(3)]).unwrap(), int(4));
        assert_eq!(call_builtin("/", vec![int(15), int(3), int(2)]).unwrap(), real(2.5));
        
        // Division resulting in real
        assert_eq!(call_builtin("/", vec![int(5), int(2)]).unwrap(), real(2.5));
        
        // Reciprocal
        assert_eq!(call_builtin("/", vec![int(4)]).unwrap(), real(0.25));
        assert_eq!(call_builtin("/", vec![real(2.0)]).unwrap(), real(0.5));
        
        // Mixed types
        assert_eq!(call_builtin("/", vec![real(10.0), int(4)]).unwrap(), real(2.5));
        
        // Division by zero errors
        assert!(call_builtin("/", vec![int(1), int(0)]).is_err());
        assert!(call_builtin("/", vec![int(1), real(0.0)]).is_err());
        assert!(call_builtin("/", vec![int(0)]).is_err()); // Reciprocal of zero
        
        // Arity error
        assert!(call_builtin("/", vec![]).is_err());
    }
}

#[cfg(test)]
mod comparison_tests {
    use super::*;

    #[test]
    fn test_equality() {
        // Basic equality
        assert_eq!(call_builtin("=", vec![int(5), int(5)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("=", vec![int(5), int(6)]).unwrap(), Value::Boolean(false));
        
        // Multiple arguments
        assert_eq!(call_builtin("=", vec![int(3), int(3), int(3)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("=", vec![int(3), int(3), int(4)]).unwrap(), Value::Boolean(false));
        
        // Mixed types
        assert_eq!(call_builtin("=", vec![int(5), real(5.0)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("=", vec![real(2.5), real(2.5)]).unwrap(), Value::Boolean(true));
        
        // Floating point precision
        assert_eq!(call_builtin("=", vec![real(0.1 + 0.2), real(0.3)]).unwrap(), Value::Boolean(true));
        
        // Arity error
        assert!(call_builtin("=", vec![int(1)]).is_err());
        assert!(call_builtin("=", vec![]).is_err());
    }

    #[test]
    fn test_less_than() {
        // Basic comparisons
        assert_eq!(call_builtin("<", vec![int(3), int(5)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("<", vec![int(5), int(3)]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("<", vec![int(5), int(5)]).unwrap(), Value::Boolean(false));
        
        // Multiple arguments (transitive)
        assert_eq!(call_builtin("<", vec![int(1), int(2), int(3)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("<", vec![int(1), int(3), int(2)]).unwrap(), Value::Boolean(false));
        
        // Mixed types
        assert_eq!(call_builtin("<", vec![int(2), real(2.5)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("<", vec![real(2.5), int(3)]).unwrap(), Value::Boolean(true));
        
        // Negative numbers
        assert_eq!(call_builtin("<", vec![int(-5), int(-2)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("<", vec![int(-2), int(1)]).unwrap(), Value::Boolean(true));
        
        // Arity error
        assert!(call_builtin("<", vec![int(1)]).is_err());
        assert!(call_builtin("<", vec![]).is_err());
    }

    #[test]
    fn test_less_equal() {
        // Basic comparisons
        assert_eq!(call_builtin("<=", vec![int(3), int(5)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("<=", vec![int(5), int(5)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("<=", vec![int(5), int(3)]).unwrap(), Value::Boolean(false));
        
        // Multiple arguments
        assert_eq!(call_builtin("<=", vec![int(1), int(2), int(2)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("<=", vec![int(1), int(3), int(2)]).unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_greater_than() {
        // Basic comparisons
        assert_eq!(call_builtin(">", vec![int(5), int(3)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin(">", vec![int(3), int(5)]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin(">", vec![int(5), int(5)]).unwrap(), Value::Boolean(false));
        
        // Multiple arguments
        assert_eq!(call_builtin(">", vec![int(5), int(3), int(1)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin(">", vec![int(5), int(1), int(3)]).unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_greater_equal() {
        // Basic comparisons
        assert_eq!(call_builtin(">=", vec![int(5), int(3)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin(">=", vec![int(5), int(5)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin(">=", vec![int(3), int(5)]).unwrap(), Value::Boolean(false));
        
        // Multiple arguments
        assert_eq!(call_builtin(">=", vec![int(5), int(3), int(3)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin(">=", vec![int(5), int(6), int(3)]).unwrap(), Value::Boolean(false));
    }
}

#[cfg(test)]
mod extended_numeric_tests {
    use super::*;

    #[test]
    fn test_abs() {
        assert_eq!(call_builtin("abs", vec![int(5)]).unwrap(), int(5));
        assert_eq!(call_builtin("abs", vec![int(-5)]).unwrap(), int(5));
        assert_eq!(call_builtin("abs", vec![int(0)]).unwrap(), int(0));
        assert_eq!(call_builtin("abs", vec![real(-3.5)]).unwrap(), real(3.5));
        assert_eq!(call_builtin("abs", vec![real(2.7)]).unwrap(), real(2.7));
        
        // Arity errors
        assert!(call_builtin("abs", vec![]).is_err());
        assert!(call_builtin("abs", vec![int(1), int(2)]).is_err());
        
        // Type error
        assert!(call_builtin("abs", vec![Value::String("hello".to_string())]).is_err());
    }

    #[test]
    fn test_quotient() {
        assert_eq!(call_builtin("quotient", vec![int(15), int(4)]).unwrap(), int(3));
        assert_eq!(call_builtin("quotient", vec![int(-15), int(4)]).unwrap(), int(-3));
        assert_eq!(call_builtin("quotient", vec![int(15), int(-4)]).unwrap(), int(-3));
        assert_eq!(call_builtin("quotient", vec![int(-15), int(-4)]).unwrap(), int(3));
        
        // Division by zero
        assert!(call_builtin("quotient", vec![int(5), int(0)]).is_err());
        
        // Type errors
        assert!(call_builtin("quotient", vec![real(5.5), int(2)]).is_err());
        assert!(call_builtin("quotient", vec![int(5), real(2.5)]).is_err());
        
        // Arity errors
        assert!(call_builtin("quotient", vec![int(5)]).is_err());
        assert!(call_builtin("quotient", vec![]).is_err());
    }

    #[test]
    fn test_remainder() {
        assert_eq!(call_builtin("remainder", vec![int(15), int(4)]).unwrap(), int(3));
        assert_eq!(call_builtin("remainder", vec![int(-15), int(4)]).unwrap(), int(-3));
        assert_eq!(call_builtin("remainder", vec![int(15), int(-4)]).unwrap(), int(3));
        assert_eq!(call_builtin("remainder", vec![int(-15), int(-4)]).unwrap(), int(-3));
        
        // Division by zero
        assert!(call_builtin("remainder", vec![int(5), int(0)]).is_err());
        
        // Type errors
        assert!(call_builtin("remainder", vec![real(5.5), int(2)]).is_err());
        
        // Arity errors
        assert!(call_builtin("remainder", vec![int(5)]).is_err());
    }

    #[test]
    fn test_modulo() {
        assert_eq!(call_builtin("modulo", vec![int(15), int(4)]).unwrap(), int(3));
        assert_eq!(call_builtin("modulo", vec![int(-15), int(4)]).unwrap(), int(1)); // Result has sign of divisor
        assert_eq!(call_builtin("modulo", vec![int(15), int(-4)]).unwrap(), int(-1));
        assert_eq!(call_builtin("modulo", vec![int(-15), int(-4)]).unwrap(), int(-3));
        
        // Division by zero
        assert!(call_builtin("modulo", vec![int(5), int(0)]).is_err());
        
        // Type errors
        assert!(call_builtin("modulo", vec![real(5.5), int(2)]).is_err());
        
        // Arity errors
        assert!(call_builtin("modulo", vec![int(5)]).is_err());
    }

    #[test]
    fn test_gcd() {
        // Basic GCD
        assert_eq!(call_builtin("gcd", vec![int(12), int(8)]).unwrap(), int(4));
        assert_eq!(call_builtin("gcd", vec![int(15), int(25)]).unwrap(), int(5));
        assert_eq!(call_builtin("gcd", vec![int(7), int(13)]).unwrap(), int(1)); // Coprime
        
        // Multiple arguments
        assert_eq!(call_builtin("gcd", vec![int(12), int(8), int(16)]).unwrap(), int(4));
        
        // With zero
        assert_eq!(call_builtin("gcd", vec![int(5), int(0)]).unwrap(), int(5));
        assert_eq!(call_builtin("gcd", vec![int(0), int(7)]).unwrap(), int(7));
        
        // Empty case
        assert_eq!(call_builtin("gcd", vec![]).unwrap(), int(0));
        
        // Single argument
        assert_eq!(call_builtin("gcd", vec![int(15)]).unwrap(), int(15));
        
        // Negative numbers
        assert_eq!(call_builtin("gcd", vec![int(-12), int(8)]).unwrap(), int(4));
        
        // Type error
        assert!(call_builtin("gcd", vec![real(5.5)]).is_err());
    }

    #[test]
    fn test_lcm() {
        // Basic LCM
        assert_eq!(call_builtin("lcm", vec![int(4), int(6)]).unwrap(), int(12));
        assert_eq!(call_builtin("lcm", vec![int(3), int(5)]).unwrap(), int(15)); // Coprime
        
        // Multiple arguments
        assert_eq!(call_builtin("lcm", vec![int(2), int(3), int(4)]).unwrap(), int(12));
        
        // With zero
        assert_eq!(call_builtin("lcm", vec![int(5), int(0)]).unwrap(), int(0));
        assert_eq!(call_builtin("lcm", vec![int(0), int(7)]).unwrap(), int(0));
        
        // Empty case
        assert_eq!(call_builtin("lcm", vec![]).unwrap(), int(1));
        
        // Single argument
        assert_eq!(call_builtin("lcm", vec![int(15)]).unwrap(), int(15));
        
        // Negative numbers
        assert_eq!(call_builtin("lcm", vec![int(-4), int(6)]).unwrap(), int(12));
        
        // Type error
        assert!(call_builtin("lcm", vec![real(5.5)]).is_err());
    }

    #[test]
    fn test_floor() {
        assert_eq!(call_builtin("floor", vec![real(3.7)]).unwrap(), int(3));
        assert_eq!(call_builtin("floor", vec![real(-3.7)]).unwrap(), int(-4));
        assert_eq!(call_builtin("floor", vec![real(5.0)]).unwrap(), int(5));
        assert_eq!(call_builtin("floor", vec![int(7)]).unwrap(), int(7)); // Integer unchanged
        
        // Arity error
        assert!(call_builtin("floor", vec![]).is_err());
        assert!(call_builtin("floor", vec![int(1), int(2)]).is_err());
        
        // Type error
        assert!(call_builtin("floor", vec![Value::String("hello".to_string())]).is_err());
    }

    #[test]
    fn test_ceiling() {
        assert_eq!(call_builtin("ceiling", vec![real(3.2)]).unwrap(), int(4));
        assert_eq!(call_builtin("ceiling", vec![real(-3.2)]).unwrap(), int(-3));
        assert_eq!(call_builtin("ceiling", vec![real(5.0)]).unwrap(), int(5));
        assert_eq!(call_builtin("ceiling", vec![int(7)]).unwrap(), int(7)); // Integer unchanged
        
        // Arity error
        assert!(call_builtin("ceiling", vec![]).is_err());
        
        // Type error
        assert!(call_builtin("ceiling", vec![Value::String("hello".to_string())]).is_err());
    }

    #[test]
    fn test_truncate() {
        assert_eq!(call_builtin("truncate", vec![real(3.7)]).unwrap(), int(3));
        assert_eq!(call_builtin("truncate", vec![real(-3.7)]).unwrap(), int(-3));
        assert_eq!(call_builtin("truncate", vec![real(5.0)]).unwrap(), int(5));
        assert_eq!(call_builtin("truncate", vec![int(7)]).unwrap(), int(7)); // Integer unchanged
        
        // Arity error
        assert!(call_builtin("truncate", vec![]).is_err());
        
        // Type error
        assert!(call_builtin("truncate", vec![Value::String("hello".to_string())]).is_err());
    }

    #[test]
    fn test_round() {
        assert_eq!(call_builtin("round", vec![real(3.4)]).unwrap(), int(3));
        assert_eq!(call_builtin("round", vec![real(3.6)]).unwrap(), int(4));
        assert_eq!(call_builtin("round", vec![real(-3.4)]).unwrap(), int(-3));
        assert_eq!(call_builtin("round", vec![real(-3.6)]).unwrap(), int(-4));
        assert_eq!(call_builtin("round", vec![real(5.0)]).unwrap(), int(5));
        assert_eq!(call_builtin("round", vec![int(7)]).unwrap(), int(7)); // Integer unchanged
        
        // Arity error
        assert!(call_builtin("round", vec![]).is_err());
        
        // Type error
        assert!(call_builtin("round", vec![Value::String("hello".to_string())]).is_err());
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(call_builtin("sqrt", vec![int(16)]).unwrap(), int(4)); // Perfect square
        assert_eq!(call_builtin("sqrt", vec![int(0)]).unwrap(), int(0));
        
        // Non-perfect square
        let result = call_builtin("sqrt", vec![int(2)]).unwrap();
        if let Value::Number(SchemeNumber::Real(r)) = result {
            assert!(approx_eq(r, 2.0_f64.sqrt()));
        } else {
            panic!("Expected real result for sqrt(2)");
        }
        
        // Real numbers
        let result = call_builtin("sqrt", vec![real(6.25)]).unwrap();
        if let Value::Number(SchemeNumber::Real(r)) = result {
            assert!(approx_eq(r, 2.5));
        } else {
            panic!("Expected real result for sqrt(6.25)");
        }
        
        // Domain errors (negative numbers)
        assert!(call_builtin("sqrt", vec![int(-1)]).is_err());
        assert!(call_builtin("sqrt", vec![real(-2.5)]).is_err());
        
        // Arity error
        assert!(call_builtin("sqrt", vec![]).is_err());
        
        // Type error
        assert!(call_builtin("sqrt", vec![Value::String("hello".to_string())]).is_err());
    }

    #[test]
    fn test_expt() {
        // Integer powers
        assert_eq!(call_builtin("expt", vec![int(2), int(3)]).unwrap(), int(8));
        assert_eq!(call_builtin("expt", vec![int(5), int(0)]).unwrap(), int(1));
        assert_eq!(call_builtin("expt", vec![int(10), int(1)]).unwrap(), int(10));
        
        // Negative base
        assert_eq!(call_builtin("expt", vec![int(-2), int(3)]).unwrap(), int(-8));
        assert_eq!(call_builtin("expt", vec![int(-2), int(2)]).unwrap(), int(4));
        
        // Negative exponent (results in real)
        let result = call_builtin("expt", vec![int(2), int(-1)]).unwrap();
        if let Value::Number(SchemeNumber::Real(r)) = result {
            assert!(approx_eq(r, 0.5));
        } else {
            panic!("Expected real result for 2^(-1)");
        }
        
        // Real numbers
        let result = call_builtin("expt", vec![real(2.5), int(2)]).unwrap();
        if let Value::Number(SchemeNumber::Real(r)) = result {
            assert!(approx_eq(r, 6.25));
        } else {
            panic!("Expected real result for 2.5^2");
        }
        
        // Arity errors
        assert!(call_builtin("expt", vec![int(2)]).is_err());
        assert!(call_builtin("expt", vec![]).is_err());
        
        // Type errors
        assert!(call_builtin("expt", vec![Value::String("hello".to_string()), int(2)]).is_err());
        assert!(call_builtin("expt", vec![int(2), Value::String("world".to_string())]).is_err());
    }

    #[test]
    fn test_min() {
        // Basic min
        assert_eq!(call_builtin("min", vec![int(3), int(1), int(4)]).unwrap(), int(1));
        assert_eq!(call_builtin("min", vec![int(5)]).unwrap(), int(5)); // Single argument
        
        // Mixed types
        assert_eq!(call_builtin("min", vec![real(2.5), int(3)]).unwrap(), real(2.5));
        assert_eq!(call_builtin("min", vec![int(2), real(1.5)]).unwrap(), real(1.5));
        
        // Negative numbers
        assert_eq!(call_builtin("min", vec![int(-5), int(-1), int(-10)]).unwrap(), int(-10));
        
        // Arity error
        assert!(call_builtin("min", vec![]).is_err());
        
        // Type error
        assert!(call_builtin("min", vec![int(1), Value::String("hello".to_string())]).is_err());
    }

    #[test]
    fn test_max() {
        // Basic max
        assert_eq!(call_builtin("max", vec![int(3), int(1), int(4)]).unwrap(), int(4));
        assert_eq!(call_builtin("max", vec![int(5)]).unwrap(), int(5)); // Single argument
        
        // Mixed types
        assert_eq!(call_builtin("max", vec![real(2.5), int(2)]).unwrap(), real(2.5));
        assert_eq!(call_builtin("max", vec![int(3), real(3.5)]).unwrap(), real(3.5));
        
        // Negative numbers
        assert_eq!(call_builtin("max", vec![int(-5), int(-1), int(-10)]).unwrap(), int(-1));
        
        // Arity error
        assert!(call_builtin("max", vec![]).is_err());
        
        // Type error
        assert!(call_builtin("max", vec![int(1), Value::String("hello".to_string())]).is_err());
    }
}

#[cfg(test)]
mod predicate_tests {
    use super::*;

    #[test]
    fn test_odd_predicate() {
        assert_eq!(call_builtin("odd?", vec![int(3)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("odd?", vec![int(4)]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("odd?", vec![int(-3)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("odd?", vec![int(0)]).unwrap(), Value::Boolean(false));
        
        // Real numbers (should be false for non-integers)
        assert_eq!(call_builtin("odd?", vec![real(3.5)]).unwrap(), Value::Boolean(false));
        
        // Non-numbers
        assert_eq!(call_builtin("odd?", vec![Value::String("hello".to_string())]).unwrap(), Value::Boolean(false));
        
        // Arity error
        assert!(call_builtin("odd?", vec![]).is_err());
        assert!(call_builtin("odd?", vec![int(1), int(2)]).is_err());
    }

    #[test]
    fn test_even_predicate() {
        assert_eq!(call_builtin("even?", vec![int(4)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("even?", vec![int(3)]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("even?", vec![int(-4)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("even?", vec![int(0)]).unwrap(), Value::Boolean(true));
        
        // Real numbers (should be false for non-integers)
        assert_eq!(call_builtin("even?", vec![real(4.0)]).unwrap(), Value::Boolean(false));
        
        // Non-numbers
        assert_eq!(call_builtin("even?", vec![Value::String("hello".to_string())]).unwrap(), Value::Boolean(false));
        
        // Arity error
        assert!(call_builtin("even?", vec![]).is_err());
        assert!(call_builtin("even?", vec![int(1), int(2)]).is_err());
    }

    #[test]
    fn test_zero_predicate() {
        assert_eq!(call_builtin("zero?", vec![int(0)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("zero?", vec![int(1)]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("zero?", vec![int(-1)]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("zero?", vec![real(0.0)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("zero?", vec![real(0.1)]).unwrap(), Value::Boolean(false));
        
        // Non-numbers
        assert_eq!(call_builtin("zero?", vec![Value::String("0".to_string())]).unwrap(), Value::Boolean(false));
        
        // Arity error
        assert!(call_builtin("zero?", vec![]).is_err());
        assert!(call_builtin("zero?", vec![int(1), int(2)]).is_err());
    }

    #[test]
    fn test_positive_predicate() {
        assert_eq!(call_builtin("positive?", vec![int(5)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("positive?", vec![int(0)]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("positive?", vec![int(-5)]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("positive?", vec![real(std::f64::consts::PI)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("positive?", vec![real(-0.1)]).unwrap(), Value::Boolean(false));
        
        // Non-numbers
        assert_eq!(call_builtin("positive?", vec![Value::String("5".to_string())]).unwrap(), Value::Boolean(false));
        
        // Arity error
        assert!(call_builtin("positive?", vec![]).is_err());
        assert!(call_builtin("positive?", vec![int(1), int(2)]).is_err());
    }

    #[test]
    fn test_negative_predicate() {
        assert_eq!(call_builtin("negative?", vec![int(-5)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("negative?", vec![int(0)]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("negative?", vec![int(5)]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("negative?", vec![real(-std::f64::consts::PI)]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("negative?", vec![real(0.1)]).unwrap(), Value::Boolean(false));
        
        // Non-numbers
        assert_eq!(call_builtin("negative?", vec![Value::String("-5".to_string())]).unwrap(), Value::Boolean(false));
        
        // Arity error
        assert!(call_builtin("negative?", vec![]).is_err());
        assert!(call_builtin("negative?", vec![int(1), int(2)]).is_err());
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_type_errors() {
        // All arithmetic operations should reject non-numbers
        let non_number = Value::String("not-a-number".to_string());
        
        assert!(call_builtin("+", vec![non_number.clone()]).is_err());
        assert!(call_builtin("-", vec![non_number.clone()]).is_err());
        assert!(call_builtin("*", vec![non_number.clone()]).is_err());
        assert!(call_builtin("/", vec![non_number.clone()]).is_err());
        assert!(call_builtin("=", vec![int(1), non_number.clone()]).is_err());
        assert!(call_builtin("<", vec![int(1), non_number.clone()]).is_err());
    }

    #[test]
    fn test_large_numbers() {
        // Test with large integers
        let large = int(i64::MAX);
        let result = call_builtin("+", vec![large.clone(), int(0)]).unwrap();
        assert_eq!(result, large);
        
        // Test potential overflow scenarios gracefully handled
        let result = call_builtin("*", vec![int(i64::MAX), int(0)]).unwrap();
        assert_eq!(result, int(0));
    }

    #[test]
    fn test_floating_point_edge_cases() {
        // Test with very small numbers
        let tiny = real(f64::EPSILON);
        let result = call_builtin("+", vec![tiny, real(0.0)]).unwrap();
        if let Value::Number(SchemeNumber::Real(r)) = result {
            assert!(approx_eq(r, f64::EPSILON));
        }
        
        // Test infinity handling in sqrt
        let result = call_builtin("sqrt", vec![real(f64::INFINITY)]).unwrap();
        if let Value::Number(SchemeNumber::Real(r)) = result {
            assert!(r.is_infinite() && r.is_sign_positive());
        }
    }
}