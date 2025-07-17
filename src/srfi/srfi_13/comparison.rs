//! SRFI 13: String Libraries - Comparison and hashing operations
//!
//! This module implements string comparison and hashing functions.

use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register comparison and hashing functions
pub fn register_functions(builtins: &mut HashMap<String, Value>) {
    // String comparison
    builtins.insert("string-compare".to_string(), string_compare_function());
    builtins.insert(
        "string-compare-ci".to_string(),
        string_compare_ci_function(),
    );
    builtins.insert("string-hash".to_string(), string_hash_function());
    builtins.insert("string-hash-ci".to_string(), string_hash_ci_function());
}

/// Create string-compare function
fn string_compare_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-compare".to_string(),
        arity: None, // Variable arity
        func: string_compare,
    })
}

/// String-compare - compare strings and call appropriate procedure
pub fn string_compare(args: &[Value]) -> Result<Value> {
    if args.len() < 5 {
        return Err(LambdustError::arity_error(5, args.len()));
    }

    let Value::String(s1) = &args[0] else {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        };

    let Value::String(s2) = &args[1] else {
            return Err(LambdustError::type_error(
                "Second argument must be a string".to_string(),
            ));
        };

    // Compare strings
    match s1.cmp(s2) {
        std::cmp::Ordering::Less => Err(LambdustError::runtime_error(
            "string-compare requires evaluator integration for procedure calls".to_string(),
        )),
        std::cmp::Ordering::Equal => Err(LambdustError::runtime_error(
            "string-compare requires evaluator integration for procedure calls".to_string(),
        )),
        std::cmp::Ordering::Greater => Err(LambdustError::runtime_error(
            "string-compare requires evaluator integration for procedure calls".to_string(),
        )),
    }
}

/// Create string-compare-ci function
fn string_compare_ci_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-compare-ci".to_string(),
        arity: None, // Variable arity
        func: string_compare_ci,
    })
}

/// String-compare-ci - case-insensitive string comparison
pub fn string_compare_ci(args: &[Value]) -> Result<Value> {
    if args.len() < 5 {
        return Err(LambdustError::arity_error(5, args.len()));
    }

    let Value::String(s1_str) = &args[0] else {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        };
    let s1 = s1_str.to_lowercase();

    let Value::String(s2_str) = &args[1] else {
            return Err(LambdustError::type_error(
                "Second argument must be a string".to_string(),
            ));
        };
    let s2 = s2_str.to_lowercase();

    // Compare strings case-insensitively
    match s1.cmp(&s2) {
        std::cmp::Ordering::Less => Err(LambdustError::runtime_error(
            "string-compare-ci requires evaluator integration for procedure calls".to_string(),
        )),
        std::cmp::Ordering::Equal => Err(LambdustError::runtime_error(
            "string-compare-ci requires evaluator integration for procedure calls".to_string(),
        )),
        std::cmp::Ordering::Greater => Err(LambdustError::runtime_error(
            "string-compare-ci requires evaluator integration for procedure calls".to_string(),
        )),
    }
}

/// Create string-hash function
fn string_hash_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-hash".to_string(),
        arity: None, // 1 or 2 args
        func: string_hash,
    })
}

/// String-hash - compute hash of string
pub fn string_hash(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let Value::String(string) = &args[0] else {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        };

    let bound = if args.len() == 2 {
        match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as u32,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as u32,
            _ => {
                return Err(LambdustError::type_error(
                    "Second argument must be an integer".to_string(),
                ));
            }
        }
    } else {
        u32::MAX
    };

    // Simple hash implementation
    let mut hash: u32 = 0;
    for c in string.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(c as u32);
    }

    let result = if bound == u32::MAX {
        hash
    } else {
        hash % bound
    };

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
        i64::from(result),
    )))
}

/// Create string-hash-ci function
fn string_hash_ci_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-hash-ci".to_string(),
        arity: None, // 1 or 2 args
        func: string_hash_ci,
    })
}

/// String-hash-ci - case-insensitive hash of string
pub fn string_hash_ci(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let Value::String(string_str) = &args[0] else {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        };
    let string = string_str.to_lowercase();

    let bound = if args.len() == 2 {
        match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as u32,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as u32,
            _ => {
                return Err(LambdustError::type_error(
                    "Second argument must be an integer".to_string(),
                ));
            }
        }
    } else {
        u32::MAX
    };

    // Simple hash implementation
    let mut hash: u32 = 0;
    for c in string.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(c as u32);
    }

    let result = if bound == u32::MAX {
        hash
    } else {
        hash % bound
    };

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
        i64::from(result),
    )))
}
