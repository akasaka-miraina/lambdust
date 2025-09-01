//! Numeric comparison operations for Lambdust
//! 
//! This module implements R7RS-compliant numeric comparison operations.

use crate::diagnostics::Result;
use crate::eval::value::Value;

/// Implements the = (numeric equality) primitive
pub fn primitive_numeric_equal(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::boolean(true));
    }
    
    let first = &args[0];
    for arg in &args[1..] {
        if !numeric_equal_two(first, arg)? {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

/// Implements the < (less than) primitive
pub fn primitive_less_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "< requires at least two arguments",
            None,
        ).into());
    }
    
    for i in 0..args.len() - 1 {
        if !numeric_less_than(&args[i], &args[i + 1])? {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

/// Implements the > (greater than) primitive
pub fn primitive_greater_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "> requires at least two arguments",
            None,
        ).into());
    }
    
    for i in 0..args.len() - 1 {
        if !numeric_greater_than(&args[i], &args[i + 1])? {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

/// Implements the <= (less than or equal) primitive
pub fn primitive_less_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "<= requires at least two arguments",
            None,
        ).into());
    }
    
    for i in 0..args.len() - 1 {
        if !numeric_less_equal(&args[i], &args[i + 1])? {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

/// Implements the >= (greater than or equal) primitive
pub fn primitive_greater_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            ">= requires at least two arguments",
            None,
        ).into());
    }
    
    for i in 0..args.len() - 1 {
        if !numeric_greater_equal(&args[i], &args[i + 1])? {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

// Helper functions for two-value comparisons
fn numeric_equal_two(a: &Value, b: &Value) -> Result<bool> {
    match (a, b) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => Ok(x == y),
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => Ok((x - y).abs() < f64::EPSILON),
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => Ok((*x as f64 - y).abs() < f64::EPSILON),
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => Ok((x - *y as f64).abs() < f64::EPSILON),
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid arguments to numeric comparison",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

fn numeric_less_than(a: &Value, b: &Value) -> Result<bool> {
    match (a, b) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => Ok(x < y),
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => Ok(x < y),
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => Ok((*x as f64) < *y),
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => Ok(*x < (*y as f64)),
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid arguments to <",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

fn numeric_greater_than(a: &Value, b: &Value) -> Result<bool> {
    match (a, b) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => Ok(x > y),
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => Ok(x > y),
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => Ok((*x as f64) > *y),
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => Ok(*x > (*y as f64)),
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid arguments to >",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

fn numeric_less_equal(a: &Value, b: &Value) -> Result<bool> {
    match (a, b) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => Ok(x <= y),
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => Ok(x <= y),
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => Ok((*x as f64) <= *y),
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => Ok(*x <= (*y as f64)),
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid arguments to <=",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

fn numeric_greater_equal(a: &Value, b: &Value) -> Result<bool> {
    match (a, b) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => Ok(x >= y),
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => Ok(x >= y),
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::InexactReal(y))) => Ok((*x as f64) >= *y),
        (Value::Literal(crate::ast::Literal::InexactReal(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => Ok(*x >= (*y as f64)),
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid arguments to >=",
            crate::diagnostics::Span::default(),
        ).into())
    }
}