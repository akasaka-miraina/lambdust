//! Common utilities and helper functions for arithmetic operations.

use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use std::sync::Arc;

/// Helper function to bind a pure arithmetic primitive.
pub fn bind_pure_arithmetic_primitive(
    env: &Arc<ThreadSafeEnvironment>,
    name: &str,
    arity_min: usize,
    arity_max: Option<usize>,
    implementation: fn(&[crate::eval::value::Value]) -> crate::diagnostics::Result<crate::eval::value::Value>,
) {
    env.define(name.to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: name.to_string(),
        arity_min,
        arity_max,
        implementation: PrimitiveImpl::RustFn(implementation),
        effects: vec![Effect::Pure],
    })));
}

/// Extract a numeric value from a Value, returning None if not a number
pub fn extract_number(value: &Value) -> Option<f64> {
    match value {
        Value::Literal(literal) => match literal {
            crate::ast::Literal::Integer(i) => Some(*i as f64),
            crate::ast::Literal::Float(f) => Some(*f),
            crate::ast::Literal::Rational(num, den) => Some(*num as f64 / *den as f64),
            _ => None,
        },
        _ => None,
    }
}

/// Extract an integer value from a Value, returning None if not an integer
pub fn extract_integer(value: &Value) -> Option<i64> {
    match value {
        Value::Literal(literal) => match literal {
            crate::ast::Literal::Integer(i) => Some(*i),
            _ => None,
        },
        _ => None,
    }
}

/// Check if a value represents zero
pub fn is_zero(value: &Value) -> bool {
    match extract_number(value) {
        Some(n) => n == 0.0,
        None => false,
    }
}

/// Check if a value represents a positive number
pub fn is_positive(value: &Value) -> bool {
    match extract_number(value) {
        Some(n) => n > 0.0,
        None => false,
    }
}

/// Check if a value represents a negative number
pub fn is_negative(value: &Value) -> bool {
    match extract_number(value) {
        Some(n) => n < 0.0,
        None => false,
    }
}