//! Rounding and conversion operations for Lambdust
//!
//! This module implements R7RS-compliant rounding, conversion, and related operations.

use crate::diagnostics::Result;
use crate::eval::value::Value;

/// Implements max primitive
pub fn primitive_max(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(crate::diagnostics::Error::runtime_error(
            "max requires at least one argument",
            None,
        )
        .into());
    }

    let mut max_val = args[0].clone();
    for arg in &args[1..] {
        if numeric_greater_than(arg, &max_val)? {
            max_val = arg.clone();
        }
    }
    Ok(max_val)
}

/// Implements min primitive
pub fn primitive_min(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(crate::diagnostics::Error::runtime_error(
            "min requires at least one argument",
            None,
        )
        .into());
    }

    let mut min_val = args[0].clone();
    for arg in &args[1..] {
        if numeric_less_than(arg, &min_val)? {
            min_val = arg.clone();
        }
    }
    Ok(min_val)
}

/// Implements floor primitive
pub fn primitive_floor(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "floor expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(*x)))
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => Ok(Value::Literal(
            crate::ast::Literal::ExactInteger(x.floor() as i64),
        )),
        _ => Err(crate::diagnostics::Error::type_error(
            "floor requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements ceiling primitive
pub fn primitive_ceiling(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "ceiling expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(*x)))
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => Ok(Value::Literal(
            crate::ast::Literal::ExactInteger(x.ceil() as i64),
        )),
        _ => Err(crate::diagnostics::Error::type_error(
            "ceiling requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements truncate primitive
pub fn primitive_truncate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "truncate expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(*x)))
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => Ok(Value::Literal(
            crate::ast::Literal::ExactInteger(x.trunc() as i64),
        )),
        _ => Err(crate::diagnostics::Error::type_error(
            "truncate requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements round primitive
pub fn primitive_round(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "round expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(*x)))
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => Ok(Value::Literal(
            crate::ast::Literal::ExactInteger(x.round() as i64),
        )),
        _ => Err(crate::diagnostics::Error::type_error(
            "round requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements exact->inexact conversion
pub fn primitive_exact_to_inexact(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "exact->inexact expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(*x as f64)))
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(*x)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "exact->inexact requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements inexact->exact conversion
pub fn primitive_inexact_to_exact(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "inexact->exact expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(*x)))
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            if x.is_finite() && x.fract() == 0.0 {
                Ok(Value::Literal(crate::ast::Literal::ExactInteger(*x as i64)))
            } else {
                Err(crate::diagnostics::Error::runtime_error(
                    "Cannot convert non-integer real to exact",
                    None,
                )
                .into())
            }
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "inexact->exact requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

// Helper functions for min/max comparisons
fn numeric_greater_than(a: &Value, b: &Value) -> Result<bool> {
    match (a, b) {
        (
            Value::Literal(crate::ast::Literal::ExactInteger(x)),
            Value::Literal(crate::ast::Literal::ExactInteger(y)),
        ) => Ok(x > y),
        (
            Value::Literal(crate::ast::Literal::InexactReal(x)),
            Value::Literal(crate::ast::Literal::InexactReal(y)),
        ) => Ok(x > y),
        (
            Value::Literal(crate::ast::Literal::ExactInteger(x)),
            Value::Literal(crate::ast::Literal::InexactReal(y)),
        ) => Ok((*x as f64) > *y),
        (
            Value::Literal(crate::ast::Literal::InexactReal(x)),
            Value::Literal(crate::ast::Literal::ExactInteger(y)),
        ) => Ok(*x > (*y as f64)),
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid arguments to numeric comparison",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

fn numeric_less_than(a: &Value, b: &Value) -> Result<bool> {
    match (a, b) {
        (
            Value::Literal(crate::ast::Literal::ExactInteger(x)),
            Value::Literal(crate::ast::Literal::ExactInteger(y)),
        ) => Ok(x < y),
        (
            Value::Literal(crate::ast::Literal::InexactReal(x)),
            Value::Literal(crate::ast::Literal::InexactReal(y)),
        ) => Ok(x < y),
        (
            Value::Literal(crate::ast::Literal::ExactInteger(x)),
            Value::Literal(crate::ast::Literal::InexactReal(y)),
        ) => Ok((*x as f64) < *y),
        (
            Value::Literal(crate::ast::Literal::InexactReal(x)),
            Value::Literal(crate::ast::Literal::ExactInteger(y)),
        ) => Ok(*x < (*y as f64)),
        _ => Err(crate::diagnostics::Error::type_error(
            "Invalid arguments to numeric comparison",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}
