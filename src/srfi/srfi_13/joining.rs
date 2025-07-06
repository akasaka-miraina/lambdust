//! SRFI 13: String Libraries - String joining and splitting operations
//!
//! This module implements string joining, splitting, and tokenization functions.

use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register joining, splitting, and tokenization functions
pub fn register_functions(builtins: &mut HashMap<String, Value>) {
    // String joining and splitting
    builtins.insert(
        "string-concatenate".to_string(),
        string_concatenate_function(),
    );
    builtins.insert(
        "string-concatenate-reverse".to_string(),
        string_concatenate_reverse_function(),
    );
    builtins.insert("string-join".to_string(), string_join_function());

    // String tokenization
    builtins.insert("string-tokenize".to_string(), string_tokenize_function());
}

/// Create string-concatenate function
fn string_concatenate_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-concatenate".to_string(),
        arity: Some(1),
        func: string_concatenate,
    })
}

/// String-concatenate - concatenate list of strings
pub fn string_concatenate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let list = &args[0];
    if !list.is_list() {
        return Err(LambdustError::type_error(
            "Argument must be a list".to_string(),
        ));
    }

    let list_vec = list
        .to_vector()
        .ok_or_else(|| LambdustError::type_error("Argument must be a proper list"))?;

    let mut result = String::new();
    for item in list_vec {
        match item {
            Value::String(s) => result.push_str(&s),
            _ => {
                return Err(LambdustError::type_error(
                    "All list elements must be strings".to_string(),
                ));
            }
        }
    }

    Ok(Value::String(result))
}

/// Create string-concatenate-reverse function (placeholder)
fn string_concatenate_reverse_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-concatenate-reverse".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-concatenate-reverse not yet implemented".to_string(),
            ))
        },
    })
}

/// Create string-join function (placeholder)
fn string_join_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-join".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-join not yet implemented".to_string(),
            ))
        },
    })
}

/// Create string-tokenize function (placeholder)
fn string_tokenize_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-tokenize".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-tokenize requires evaluator integration".to_string(),
            ))
        },
    })
}
