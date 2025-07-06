//! SRFI 69: Basic Hash Tables - Hash functions
//!
//! This module implements the hash functions for different data types.

use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register hash functions
pub fn register_functions(builtins: &mut HashMap<String, Value>) {
    // Hash functions
    builtins.insert("hash".to_string(), hash_function());
    builtins.insert("string-hash".to_string(), string_hash_function());
    builtins.insert("string-ci-hash".to_string(), string_ci_hash_function());
}

/// Create hash function
fn hash_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash".to_string(),
        arity: None, // 1 or 2 args
        func: hash_value,
    })
}

/// Hash - generic hash function for any object
pub fn hash_value(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let object = &args[0];
    let bound = if args.len() == 2 {
        match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as u32,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as u32,
            _ => return Err(LambdustError::type_error("Second argument must be an integer".to_string())),
        }
    } else {
        u32::MAX
    };

    // Simple hash implementation for different types
    let hash_value = match object {
        Value::Number(n) => {
            let s = n.to_string();
            calculate_string_hash(&s)
        }
        Value::String(s) => calculate_string_hash(s),
        Value::Symbol(s) => calculate_string_hash(s),
        Value::Character(c) => *c as u32,
        Value::Boolean(true) => 1,
        Value::Boolean(false) => 0,
        _ => {
            // For complex objects, use their string representation
            let s = format!("{:?}", object);
            calculate_string_hash(&s)
        }
    };

    let result = if bound != u32::MAX {
        hash_value % bound
    } else {
        hash_value
    };

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result as i64)))
}

/// Create string-hash function
fn string_hash_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-hash".to_string(),
        arity: None, // 1 or 2 args
        func: string_hash_impl,
    })
}

/// String-hash - hash function for strings
pub fn string_hash_impl(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(LambdustError::type_error("First argument must be a string".to_string())),
    };

    let bound = if args.len() == 2 {
        match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as u32,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as u32,
            _ => return Err(LambdustError::type_error("Second argument must be an integer".to_string())),
        }
    } else {
        u32::MAX
    };

    let hash_value = calculate_string_hash(string);
    let result = if bound != u32::MAX {
        hash_value % bound
    } else {
        hash_value
    };

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result as i64)))
}

/// Create string-ci-hash function
fn string_ci_hash_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-ci-hash".to_string(),
        arity: None, // 1 or 2 args
        func: string_ci_hash_impl,
    })
}

/// String-ci-hash - case-insensitive hash function for strings
pub fn string_ci_hash_impl(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s.to_lowercase(),
        _ => return Err(LambdustError::type_error("First argument must be a string".to_string())),
    };

    let bound = if args.len() == 2 {
        match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as u32,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as u32,
            _ => return Err(LambdustError::type_error("Second argument must be an integer".to_string())),
        }
    } else {
        u32::MAX
    };

    let hash_value = calculate_string_hash(&string);
    let result = if bound != u32::MAX {
        hash_value % bound
    } else {
        hash_value
    };

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result as i64)))
}

/// Helper function to calculate string hash using a simple algorithm
fn calculate_string_hash(s: &str) -> u32 {
    let mut hash: u32 = 0;
    for c in s.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(c as u32);
    }
    hash
}