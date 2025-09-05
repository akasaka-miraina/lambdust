//! Common utilities and constants for list operations.

use crate::eval::value::Value;
use std::sync::Arc;

// Static error messages for better performance (avoid string allocations)
pub const MAP_ARITY_ERROR: &str = "map requires at least 2 arguments";
pub const MAP_PROCEDURE_ERROR: &str = "map: first argument must be a procedure";
pub const FOR_EACH_ARITY_ERROR: &str = "for-each requires at least 2 arguments";
pub const FOR_EACH_PROCEDURE_ERROR: &str = "for-each: first argument must be a procedure";

/// Efficient type descriptor for error messages (avoids repeated pattern matching)
pub fn get_value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Literal(_) => "literal",
        Value::Symbol(_) => "symbol",
        Value::Keyword(_) => "keyword",
        Value::Pair(_, _) => "improper list",
        Value::MutablePair(_, _) => "mutable pair",
        Value::Vector(_) => "vector",
        Value::Nil => "empty list",
        Value::Unspecified => "unspecified",
        Value::Primitive(_) => "primitive procedure",
        Value::Procedure(_) => "procedure",
        Value::Continuation(_) => "continuation",
        _ => "unknown value",
    }
}

/// Check if a value is a proper list
pub fn is_proper_list(value: &Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Pair(_, cdr) => is_proper_list(cdr),
        _ => false,
    }
}

/// Copy a list structure
pub fn copy_list(value: &Value) -> crate::diagnostics::Result<Value> {
    match value {
        Value::Nil => Ok(Value::Nil),
        Value::Pair(car, cdr) => Ok(Value::Pair(
            Box::new(car.as_ref().clone()),
            Box::new(copy_list(cdr)?),
        )),
        _ => Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!(
                "copy_list: expected list, found {}",
                get_value_type_name(value)
            ),
            None,
        ))),
    }
}

/// Check if two values are equal using R7RS equal?
pub fn values_equal(a: &Value, b: &Value) -> bool {
    // TODO: Implement proper equal? semantics
    values_eqv(a, b)
}

/// Check if two values are eq?
pub fn values_eq(a: &Value, b: &Value) -> bool {
    std::ptr::eq(a, b)
}

/// Check if two values are eqv?
pub fn values_eqv(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Literal(l1), Value::Literal(l2)) => l1 == l2,
        (Value::Symbol(s1), Value::Symbol(s2)) => s1 == s2,
        _ => false,
    }
}

/// Check if a list is circular
pub fn is_circular_list(value: &Value) -> bool {
    let mut slow = value;
    let mut fast = value;

    loop {
        match fast {
            Value::Nil => return false,
            Value::Pair(_, cdr1) => {
                fast = cdr1;
                match fast {
                    Value::Nil => return false,
                    Value::Pair(_, cdr2) => {
                        fast = cdr2;
                        if std::ptr::eq(slow, fast) {
                            return true;
                        }
                    }
                    _ => return false,
                }
            }
            _ => return false,
        }

        match slow {
            Value::Pair(_, cdr) => slow = cdr,
            _ => return false,
        }
    }
}

/// Check if a list is dotted (improper)
pub fn is_dotted_list(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Pair(_, cdr) => match cdr.as_ref() {
            Value::Nil => false,
            Value::Pair(_, _) => is_dotted_list(cdr),
            _ => true,
        },
        _ => true,
    }
}
