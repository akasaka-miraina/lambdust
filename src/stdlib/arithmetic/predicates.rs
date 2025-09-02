//! Numeric predicate functions for Lambdust
//!
//! This module implements R7RS-compliant numeric predicates and type checking functions.

use crate::diagnostics::Result;
use crate::eval::value::Value;

/// Implements zero? predicate
pub fn primitive_zero_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "zero? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => Ok(Value::boolean(*x == 0)),
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            Ok(Value::boolean(x.abs() < f64::EPSILON))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "zero? requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements positive? predicate
pub fn primitive_positive_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "positive? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => Ok(Value::boolean(*x > 0)),
        Value::Literal(crate::ast::Literal::InexactReal(x)) => Ok(Value::boolean(*x > 0.0)),
        _ => Err(crate::diagnostics::Error::type_error(
            "positive? requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements negative? predicate
pub fn primitive_negative_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "negative? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => Ok(Value::boolean(*x < 0)),
        Value::Literal(crate::ast::Literal::InexactReal(x)) => Ok(Value::boolean(*x < 0.0)),
        _ => Err(crate::diagnostics::Error::type_error(
            "negative? requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements odd? predicate
pub fn primitive_odd_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "odd? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => Ok(Value::boolean(x % 2 != 0)),
        _ => Err(crate::diagnostics::Error::type_error(
            "odd? requires an integer",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements even? predicate
pub fn primitive_even_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "even? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => Ok(Value::boolean(x % 2 == 0)),
        _ => Err(crate::diagnostics::Error::type_error(
            "even? requires an integer",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements number? predicate
pub fn primitive_number_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "number? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(_))
        | Value::Literal(crate::ast::Literal::InexactReal(_)) => Ok(Value::boolean(true)),
        _ => Ok(Value::boolean(false)),
    }
}

/// Implements integer? predicate
pub fn primitive_integer_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "integer? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(_)) => Ok(Value::boolean(true)),
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            Ok(Value::boolean(x.fract() == 0.0 && x.is_finite()))
        }
        _ => Ok(Value::boolean(false)),
    }
}

/// Implements rational? predicate
pub fn primitive_rational_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "rational? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(_)) => Ok(Value::boolean(true)),
        Value::Literal(crate::ast::Literal::InexactReal(x)) => Ok(Value::boolean(x.is_finite())),
        _ => Ok(Value::boolean(false)),
    }
}

/// Implements real? predicate
pub fn primitive_real_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "real? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(_))
        | Value::Literal(crate::ast::Literal::InexactReal(_)) => Ok(Value::boolean(true)),
        _ => Ok(Value::boolean(false)),
    }
}

/// Implements complex? predicate
pub fn primitive_complex_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "complex? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(_))
        | Value::Literal(crate::ast::Literal::InexactReal(_)) => Ok(Value::boolean(true)),
        _ => Ok(Value::boolean(false)),
    }
}

/// Implements exact? predicate
pub fn primitive_exact_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "exact? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(_)) => Ok(Value::boolean(true)),
        Value::Literal(crate::ast::Literal::InexactReal(_)) => Ok(Value::boolean(false)),
        _ => Err(crate::diagnostics::Error::type_error(
            "exact? requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements inexact? predicate
pub fn primitive_inexact_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "inexact? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(_)) => Ok(Value::boolean(false)),
        Value::Literal(crate::ast::Literal::InexactReal(_)) => Ok(Value::boolean(true)),
        _ => Err(crate::diagnostics::Error::type_error(
            "inexact? requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements exact-integer? predicate
pub fn primitive_exact_integer_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "exact-integer? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(_)) => Ok(Value::boolean(true)),
        _ => Ok(Value::boolean(false)),
    }
}

/// Implements finite? predicate
pub fn primitive_finite_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "finite? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(_)) => Ok(Value::boolean(true)),
        Value::Literal(crate::ast::Literal::InexactReal(x)) => Ok(Value::boolean(x.is_finite())),
        _ => Err(crate::diagnostics::Error::type_error(
            "finite? requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements infinite? predicate
pub fn primitive_infinite_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "infinite? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(_)) => Ok(Value::boolean(false)),
        Value::Literal(crate::ast::Literal::InexactReal(x)) => Ok(Value::boolean(x.is_infinite())),
        _ => Err(crate::diagnostics::Error::type_error(
            "infinite? requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements nan? predicate
pub fn primitive_nan_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "nan? expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(_)) => Ok(Value::boolean(false)),
        Value::Literal(crate::ast::Literal::InexactReal(x)) => Ok(Value::boolean(x.is_nan())),
        _ => Err(crate::diagnostics::Error::type_error(
            "nan? requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}
