//! Shared utility functions for builtin implementations
//!
//! This module provides common patterns and utilities used across
//! all builtin function implementations to reduce code duplication
//! and ensure consistent behavior.

use crate::error::LambdustError;
use crate::lexer::SchemeNumber;
use crate::value::{Procedure, Value};

/// Check function arity with exact count
pub fn check_arity(args: &[Value], expected: usize) -> Result<(), LambdustError> {
    if args.len() != expected {
        return Err(LambdustError::arity_error(expected, args.len()));
    }
    Ok(())
}

/// Check function arity within a range
pub fn check_arity_range(
    args: &[Value],
    min: usize,
    max: Option<usize>,
) -> Result<(), LambdustError> {
    if args.len() < min {
        return Err(LambdustError::arity_error(min, args.len()));
    }
    if let Some(max) = max {
        if args.len() > max {
            return Err(LambdustError::arity_error(max, args.len()));
        }
    }
    Ok(())
}

/// Extract a number from a Value with error handling
pub fn expect_number<'a>(
    value: &'a Value,
    func_name: &str,
) -> Result<&'a SchemeNumber, LambdustError> {
    value.as_number().ok_or_else(|| {
        LambdustError::type_error(format!("{}: expected number, got {}", func_name, value))
    })
}

/// Extract a string from a Value with error handling
pub fn expect_string<'a>(value: &'a Value, func_name: &str) -> Result<&'a str, LambdustError> {
    value.as_string().ok_or_else(|| {
        LambdustError::type_error(format!("{}: expected string, got {}", func_name, value))
    })
}

/// Extract two strings from first two arguments with error handling
pub fn expect_two_strings<'a>(
    args: &'a [Value],
    func_name: &str,
) -> Result<(&'a str, &'a str), LambdustError> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let s1 = expect_string(&args[0], func_name)?;
    let s2 = expect_string(&args[1], func_name)?;
    Ok((s1, s2))
}

/// Extract a symbol from a Value with error handling
pub fn expect_symbol<'a>(value: &'a Value, func_name: &str) -> Result<&'a str, LambdustError> {
    value.as_symbol().ok_or_else(|| {
        LambdustError::type_error(format!("{}: expected symbol, got {}", func_name, value))
    })
}

/// Extract a character from a Value with error handling
pub fn expect_character(value: &Value, func_name: &str) -> Result<char, LambdustError> {
    value.as_character().ok_or_else(|| {
        LambdustError::type_error(format!("{}: expected character, got {}", func_name, value))
    })
}

/// Extract a non-negative integer index from a Value
pub fn expect_integer_index(value: &Value, func_name: &str) -> Result<usize, LambdustError> {
    match value.as_number() {
        Some(SchemeNumber::Integer(i)) if *i >= 0 => Ok(*i as usize),
        Some(SchemeNumber::Real(f)) if f.fract() == 0.0 && *f >= 0.0 => Ok(*f as usize),
        _ => Err(LambdustError::type_error(format!(
            "{}: expected non-negative integer, got {}",
            func_name, value
        ))),
    }
}

/// Extract an integer from a Value with error handling
pub fn expect_integer(value: &Value, func_name: &str) -> Result<i64, LambdustError> {
    match value.as_number() {
        Some(SchemeNumber::Integer(i)) => Ok(*i),
        Some(SchemeNumber::Real(f)) if f.fract() == 0.0 => Ok(*f as i64),
        _ => Err(LambdustError::type_error(format!(
            "{}: expected integer, got {}",
            func_name, value
        ))),
    }
}

/// Check bounds for index access operations
pub fn check_bounds(
    index: usize,
    length: usize,
    func_name: &str,
    collection_type: &str,
) -> Result<(), LambdustError> {
    if index >= length {
        return Err(LambdustError::runtime_error(format!(
            "{}: index {} out of bounds for {} of length {}",
            func_name, index, collection_type, length
        )));
    }
    Ok(())
}

/// Create a builtin procedure with consistent structure
pub fn make_builtin_procedure(
    name: &str,
    arity: Option<usize>,
    func: fn(&[Value]) -> Result<Value, LambdustError>,
) -> Value {
    Value::Procedure(Procedure::Builtin {
        name: name.to_string(),
        arity,
        func,
    })
}

/// Macro to create predicate functions with less boilerplate
#[macro_export]
macro_rules! make_predicate {
    ($name:expr, $predicate:expr) => {
        $crate::builtins::utils::make_builtin_procedure($name, Some(1), |args| {
            $crate::builtins::utils::check_arity(args, 1)?;
            Ok($crate::builtins::utils::make_boolean($predicate(&args[0])))
        })
    };
}

/// Macro to create comparison functions for any type with an extractor function
#[macro_export]
macro_rules! make_comparison {
    ($name:expr, $op:tt, $extractor:expr) => {
        $crate::builtins::utils::make_builtin_procedure($name, None, |args| {
            $crate::builtins::utils::check_arity_range(args, 2, None)?;

            for pair in args.windows(2) {
                let current = $extractor(&pair[0], $name)?;
                let next = $extractor(&pair[1], $name)?;

                if !(current $op next) {
                    return Ok($crate::builtins::utils::make_boolean(false));
                }
            }
            Ok($crate::builtins::utils::make_boolean(true))
        })
    };
}

/// Macro to create string comparison functions
#[macro_export]
macro_rules! make_string_comparison {
    ($name:expr, $op:tt) => {
        $crate::make_comparison!($name, $op, $crate::builtins::utils::expect_string)
    };
}

/// Macro to create character comparison functions
#[macro_export]
macro_rules! make_char_comparison {
    ($name:expr, $op:tt) => {
        $crate::make_comparison!($name, $op, $crate::builtins::utils::expect_character)
    };
}

/// Apply a numeric operation to two SchemeNumbers
pub fn apply_numeric_operation<F>(
    a: &SchemeNumber,
    b: &SchemeNumber,
    op_name: &str,
    operation: F,
) -> Result<SchemeNumber, LambdustError>
where
    F: Fn(f64, f64) -> f64,
{
    let (x, y) = match (a, b) {
        (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => (*x as f64, *y as f64),
        (SchemeNumber::Real(x), SchemeNumber::Real(y)) => (*x, *y),
        (SchemeNumber::Integer(x), SchemeNumber::Real(y)) => (*x as f64, *y),
        (SchemeNumber::Real(x), SchemeNumber::Integer(y)) => (*x, *y as f64),
        _ => {
            return Err(LambdustError::type_error(format!(
                "Cannot {} {} and {}",
                op_name, a, b
            )));
        }
    };

    let result = operation(x, y);

    // If both inputs were integers and result is a whole number, keep as integer
    if matches!((a, b), (SchemeNumber::Integer(_), SchemeNumber::Integer(_)))
        && result.fract() == 0.0
    {
        Ok(SchemeNumber::Integer(result as i64))
    } else {
        Ok(SchemeNumber::Real(result))
    }
}

/// Apply a numeric comparison operation to two SchemeNumbers
pub fn compare_numbers<F>(a: &SchemeNumber, b: &SchemeNumber, operation: F) -> bool
where
    F: Fn(f64, f64) -> bool,
{
    let (x, y) = match (a, b) {
        (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => (*x as f64, *y as f64),
        (SchemeNumber::Real(x), SchemeNumber::Real(y)) => (*x, *y),
        (SchemeNumber::Integer(x), SchemeNumber::Real(y)) => (*x as f64, *y),
        (SchemeNumber::Real(x), SchemeNumber::Integer(y)) => (*x, *y as f64),
        _ => return false,
    };

    operation(x, y)
}

/// Convert a list Value to a vector of Values with error handling
pub fn expect_list_to_vector(value: &Value, func_name: &str) -> Result<Vec<Value>, LambdustError> {
    if !value.is_list() {
        return Err(LambdustError::type_error(format!(
            "{}: expected list, got {}",
            func_name, value
        )));
    }

    value.to_vector().ok_or_else(|| {
        LambdustError::type_error(format!(
            "{}: expected proper list, got improper list",
            func_name
        ))
    })
}

/// Safe slice operation for strings with character boundary respect
pub fn safe_string_slice(s: &str, start: usize, end: Option<usize>) -> &str {
    let chars: Vec<char> = s.chars().collect();
    let start_idx = start.min(chars.len());
    let end_idx = end.unwrap_or(chars.len()).min(chars.len());

    if start_idx >= end_idx {
        return "";
    }

    let start_byte = chars.iter().take(start_idx).map(|c| c.len_utf8()).sum();
    let end_byte = chars.iter().take(end_idx).map(|c| c.len_utf8()).sum();

    &s[start_byte..end_byte]
}

/// Get character at index in string with bounds checking
pub fn string_char_at(s: &str, index: usize, func_name: &str) -> Result<char, LambdustError> {
    let chars: Vec<char> = s.chars().collect();
    if index >= chars.len() {
        return Err(LambdustError::runtime_error(format!(
            "{}: index {} out of bounds for string of length {}",
            func_name,
            index,
            chars.len()
        )));
    }
    Ok(chars[index])
}

/// Check if a value represents an exact number
pub fn is_exact_number(value: &Value) -> bool {
    match value {
        Value::Number(n) => {
            use crate::lexer::SchemeNumber;
            matches!(n, SchemeNumber::Integer(_) | SchemeNumber::Rational(_, _))
        }
        _ => false,
    }
}

/// Check if a value represents an inexact number  
pub fn is_inexact_number(value: &Value) -> bool {
    match value {
        Value::Number(n) => {
            use crate::lexer::SchemeNumber;
            matches!(n, SchemeNumber::Real(_) | SchemeNumber::Complex(_, _))
        }
        _ => false,
    }
}

/// Check if a value represents an integer
pub fn is_integer(value: &Value) -> bool {
    match value {
        Value::Number(n) => {
            use crate::lexer::SchemeNumber;
            matches!(n, SchemeNumber::Integer(_))
                || matches!(n, SchemeNumber::Real(r) if r.fract() == 0.0)
        }
        _ => false,
    }
}

/// Check if a value represents a rational number
pub fn is_rational(value: &Value) -> bool {
    match value {
        Value::Number(n) => {
            use crate::lexer::SchemeNumber;
            matches!(
                n,
                SchemeNumber::Integer(_) | SchemeNumber::Rational(_, _) | SchemeNumber::Real(_)
            )
        }
        _ => false,
    }
}

/// Check if a value represents a real number
pub fn is_real(value: &Value) -> bool {
    match value {
        Value::Number(n) => {
            use crate::lexer::SchemeNumber;
            match n {
                SchemeNumber::Integer(_) => true,
                SchemeNumber::Rational(_, _) => true,
                SchemeNumber::Real(_) => true,
                SchemeNumber::Complex(_, imag) => *imag == 0.0,
            }
        }
        _ => false,
    }
}

/// Check if a value represents a complex number
pub fn is_complex(value: &Value) -> bool {
    match value {
        Value::Number(_) => true, // All numbers are complex in Scheme
        _ => false,
    }
}

/// Check if a value is an EOF object
pub fn is_eof_object(value: &Value) -> bool {
    // For now, no EOF objects are implemented
    // This is a placeholder for future implementation
    matches!(value, Value::Symbol(s) if s == "#<eof>")
}

/// Check if a value is an odd integer
pub fn is_odd(value: &Value) -> bool {
    match value.as_number() {
        Some(SchemeNumber::Integer(n)) => n % 2 != 0,
        _ => false,
    }
}

/// Check if a value is an even integer
pub fn is_even(value: &Value) -> bool {
    match value.as_number() {
        Some(SchemeNumber::Integer(n)) => n % 2 == 0,
        _ => false,
    }
}

/// Check if a value is zero
pub fn is_zero(value: &Value) -> bool {
    match value.as_number() {
        Some(SchemeNumber::Integer(n)) => *n == 0,
        Some(SchemeNumber::Real(n)) => *n == 0.0,
        _ => false,
    }
}

/// Check if a value is positive
pub fn is_positive(value: &Value) -> bool {
    match value.as_number() {
        Some(SchemeNumber::Integer(n)) => *n > 0,
        Some(SchemeNumber::Real(n)) => *n > 0.0,
        _ => false,
    }
}

/// Check if a value is negative
pub fn is_negative(value: &Value) -> bool {
    match value.as_number() {
        Some(SchemeNumber::Integer(n)) => *n < 0,
        Some(SchemeNumber::Real(n)) => *n < 0.0,
        _ => false,
    }
}

/// Create an optimized boolean value using memory pool
/// This reduces allocation overhead for frequent boolean operations
pub fn make_boolean(value: bool) -> Value {
    Value::new_boolean(value)
}

/// Create an optimized integer value using memory pool
/// This reduces allocation overhead for small integers
pub fn make_integer(value: i64) -> Value {
    Value::new_integer(value)
}

/// Create an optimized nil value using memory pool  
/// This reduces allocation overhead for nil values
pub fn make_nil() -> Value {
    Value::new_nil()
}

/// Create an optimized symbol value using symbol interning
/// This reduces string allocation overhead for symbols
pub fn make_symbol(symbol: &str) -> Value {
    Value::new_symbol(symbol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_arity() {
        let args = vec![Value::from(1i64), Value::from(2i64)];
        assert!(check_arity(&args, 2).is_ok());
        assert!(check_arity(&args, 1).is_err());
        assert!(check_arity(&args, 3).is_err());
    }

    #[test]
    fn test_check_arity_range() {
        let args = vec![Value::from(1i64), Value::from(2i64)];
        assert!(check_arity_range(&args, 1, Some(3)).is_ok());
        assert!(check_arity_range(&args, 2, Some(2)).is_ok());
        assert!(check_arity_range(&args, 3, None).is_err());
        assert!(check_arity_range(&args, 1, Some(1)).is_err());
    }

    #[test]
    fn test_expect_number() {
        let value = Value::from(42i64);
        assert!(expect_number(&value, "test").is_ok());

        let value = Value::from("not a number");
        assert!(expect_number(&value, "test").is_err());
    }

    #[test]
    fn test_apply_numeric_operation() {
        let a = SchemeNumber::Integer(10);
        let b = SchemeNumber::Integer(5);

        let result = apply_numeric_operation(&a, &b, "add", |x, y| x + y).unwrap();
        assert_eq!(result, SchemeNumber::Integer(15));

        let result = apply_numeric_operation(&a, &b, "divide", |x, y| x / y).unwrap();
        assert_eq!(result, SchemeNumber::Integer(2)); // 10/5 = 2 (exact result)
    }

    #[test]
    fn test_compare_numbers() {
        let a = SchemeNumber::Integer(10);
        let b = SchemeNumber::Integer(5);

        assert!(compare_numbers(&a, &b, |x, y| x > y));
        assert!(!compare_numbers(&a, &b, |x, y| x < y));
        assert!(!compare_numbers(&a, &b, |x, y| x == y));
    }
}
