//! SRFI 13: String Libraries - Constructors and basic operations
//!
//! This module implements basic string operations and predicates.

use crate::builtins::utils::{check_arity, expect_string, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register constructor and basic operation functions
pub fn register_functions(builtins: &mut HashMap<String, Value>) {
    // String constructors and accessors
    builtins.insert("string-null?".to_string(), string_null_function());
    builtins.insert("string-every".to_string(), string_every_function());
    builtins.insert("string-any".to_string(), string_any_function());
}

/// Create string-null? function
fn string_null_function() -> Value {
    make_builtin_procedure("string-null?", Some(1), |args| {
        check_arity(args, 1)?;
        let s = expect_string(&args[0], "string-null?")?;
        Ok(Value::Boolean(s.is_empty()))
    })
}

/// Create string-every function
fn string_every_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-every".to_string(),
        arity: None, // Variable arity
        func: string_every,
    })
}

/// String-every - test if predicate is true for every character
pub fn string_every(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let _pred = &args[0];
    let string = &args[1];

    match string {
        Value::String(_s) => {
            // This would need evaluator integration to call the predicate
            Err(LambdustError::runtime_error(
                "string-every requires evaluator integration for predicate calls".to_string(),
            ))
        }
        _ => Err(LambdustError::type_error(
            "Second argument must be a string".to_string(),
        )),
    }
}

/// Create string-any function
fn string_any_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-any".to_string(),
        arity: None, // Variable arity
        func: string_any,
    })
}

/// String-any - test if predicate is true for any character
pub fn string_any(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let _pred = &args[0];
    let string = &args[1];

    match string {
        Value::String(_s) => {
            // This would need evaluator integration to call the predicate
            Err(LambdustError::runtime_error(
                "string-any requires evaluator integration for predicate calls".to_string(),
            ))
        }
        _ => Err(LambdustError::type_error(
            "Second argument must be a string".to_string(),
        )),
    }
}
