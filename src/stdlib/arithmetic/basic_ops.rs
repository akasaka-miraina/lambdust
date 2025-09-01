//! Basic arithmetic operations (+, -, *, /) for Lambdust
//! 
//! This module implements the core arithmetic operations required by R7RS,
//! handling the numeric tower with proper type coercion and exactness preservation.

use crate::diagnostics::Result;
use crate::eval::value::Value;

/// Implements the + (addition) primitive
pub fn primitive_add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Literal(crate::ast::Literal::ExactInteger(0)));
    }
    
    let mut result = args[0].clone();
    for arg in &args[1..] {
        result = add_two_values(result, arg.clone())?;
    }
    Ok(result)
}

/// Implements the - (subtraction) primitive
pub fn primitive_subtract(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(crate::diagnostics::Error::runtime_error(
            "- requires at least one argument",
            None,
        ).into());
    }
    
    if args.len() == 1 {
        // Unary minus
        return negate_value(args[0].clone());
    }
    
    let mut result = args[0].clone();
    for arg in &args[1..] {
        result = subtract_two_values(result, arg.clone())?;
    }
    Ok(result)
}

/// Implements the * (multiplication) primitive
pub fn primitive_multiply(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Literal(crate::ast::Literal::ExactInteger(1)));
    }
    
    let mut result = args[0].clone();
    for arg in &args[1..] {
        result = multiply_two_values(result, arg.clone())?;
    }
    Ok(result)
}

/// Implements the / (division) primitive
pub fn primitive_divide(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(crate::diagnostics::Error::runtime_error(
            "/ requires at least one argument",
            None,
        ).into());
    }
    
    if args.len() == 1 {
        // Reciprocal: 1/x
        return reciprocal_value(args[0].clone());
    }
    
    let mut result = args[0].clone();
    for arg in &args[1..] {
        result = divide_two_values(result, arg.clone())?;
    }
    Ok(result)
}

// Helper functions for two-value operations
fn add_two_values(a: Value, b: Value) -> Result<Value> {
    match (&a, &b) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(x + y)))
        }
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x + y)))
        }
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(*x as f64 + y)))
        }
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x + *y as f64)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid arguments to +",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

fn subtract_two_values(a: Value, b: Value) -> Result<Value> {
    match (&a, &b) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(x - y)))
        }
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x - y)))
        }
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(*x as f64 - y)))
        }
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x - *y as f64)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid arguments to -",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

fn multiply_two_values(a: Value, b: Value) -> Result<Value> {
    match (&a, &b) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(x * y)))
        }
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x * y)))
        }
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(*x as f64 * y)))
        }
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x * *y as f64)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid arguments to *",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

fn divide_two_values(a: Value, b: Value) -> Result<Value> {
    match (&a, &b) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            if *y == 0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero",
                    None,
                ).into());
            }
            // Always return real for division to maintain R7RS semantics
            Ok(Value::Literal(crate::ast::Literal::InexactReal(*x as f64 / *y as f64)))
        }
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => {
            if *y == 0.0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero",
                    None,
                ).into());
            }
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x / y)))
        }
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => {
            if *y == 0.0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero",
                    None,
                ).into());
            }
            Ok(Value::Literal(crate::ast::Literal::InexactReal(*x as f64 / y)))
        }
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            if *y == 0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero",
                    None,
                ).into());
            }
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x / *y as f64)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid arguments to /",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

fn negate_value(value: Value) -> Result<Value> {
    match value {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(-x)))
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(-x)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid argument to unary -",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

fn reciprocal_value(value: Value) -> Result<Value> {
    match value {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            if x == 0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero",
                    None,
                ).into());
            }
            Ok(Value::Literal(crate::ast::Literal::InexactReal(1.0 / x as f64)))
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            if x == 0.0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero", 
                    None,
                ).into());
            }
            Ok(Value::Literal(crate::ast::Literal::InexactReal(1.0 / x)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid argument to reciprocal",
            crate::diagnostics::Span::default(),
        ).into())
    }
}