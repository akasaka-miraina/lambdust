//! SRFI 13: String Libraries - String modification operations
//!
//! This module implements string modification, padding, and trimming functions.

// Note: Not using utils functions here as we implement type checking manually
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register modification, padding, and trimming functions
pub fn register_functions(builtins: &mut HashMap<String, Value>) {
    // String modification
    builtins.insert("string-take".to_string(), string_take_function());
    builtins.insert("string-drop".to_string(), string_drop_function());
    builtins.insert("string-take-right".to_string(), string_take_right_function());
    builtins.insert("string-drop-right".to_string(), string_drop_right_function());

    // String padding
    builtins.insert("string-pad".to_string(), string_pad_function());
    builtins.insert("string-pad-right".to_string(), string_pad_right_function());

    // String trimming
    builtins.insert("string-trim".to_string(), string_trim_function());
    builtins.insert("string-trim-right".to_string(), string_trim_right_function());
    builtins.insert("string-trim-both".to_string(), string_trim_both_function());

    // String replacement
    builtins.insert("string-replace".to_string(), string_replace_function());

    // String filtering
    builtins.insert("string-filter".to_string(), string_filter_function());
    builtins.insert("string-delete".to_string(), string_delete_function());
}

/// Create string-take function
fn string_take_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-take".to_string(),
        arity: Some(2),
        func: string_take,
    })
}

/// String-take - take first n characters
pub fn string_take(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(LambdustError::type_error("First argument must be a string".to_string())),
    };

    let n = match &args[1] {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i,
        Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as i64,
        _ => return Err(LambdustError::type_error("Second argument must be an integer".to_string())),
    };

    if n < 0 {
        return Err(LambdustError::runtime_error("Cannot take negative number of characters".to_string()));
    }

    let chars: Vec<char> = string.chars().collect();
    let take_count = std::cmp::min(n as usize, chars.len());
    let result: String = chars.into_iter().take(take_count).collect();

    Ok(Value::String(result))
}

/// Create string-drop function
fn string_drop_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-drop".to_string(),
        arity: Some(2),
        func: string_drop,
    })
}

/// String-drop - drop first n characters
pub fn string_drop(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(LambdustError::type_error("First argument must be a string".to_string())),
    };

    let n = match &args[1] {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i,
        Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as i64,
        _ => return Err(LambdustError::type_error("Second argument must be an integer".to_string())),
    };

    if n < 0 {
        return Err(LambdustError::runtime_error("Cannot drop negative number of characters".to_string()));
    }

    let chars: Vec<char> = string.chars().collect();
    let drop_count = std::cmp::min(n as usize, chars.len());
    let result: String = chars.into_iter().skip(drop_count).collect();

    Ok(Value::String(result))
}

/// Create string-take-right function
fn string_take_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-take-right".to_string(),
        arity: Some(2),
        func: string_take_right,
    })
}

/// String-take-right - take last n characters
pub fn string_take_right(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(LambdustError::type_error("First argument must be a string".to_string())),
    };

    let n = match &args[1] {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i,
        Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as i64,
        _ => return Err(LambdustError::type_error("Second argument must be an integer".to_string())),
    };

    if n < 0 {
        return Err(LambdustError::runtime_error("Cannot take negative number of characters".to_string()));
    }

    let chars: Vec<char> = string.chars().collect();
    let len = chars.len();
    let take_count = std::cmp::min(n as usize, len);
    let start = len.saturating_sub(take_count);
    let result: String = chars.into_iter().skip(start).collect();

    Ok(Value::String(result))
}

/// Create string-drop-right function
fn string_drop_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-drop-right".to_string(),
        arity: Some(2),
        func: string_drop_right,
    })
}

/// String-drop-right - drop last n characters
pub fn string_drop_right(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(LambdustError::type_error("First argument must be a string".to_string())),
    };

    let n = match &args[1] {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i,
        Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as i64,
        _ => return Err(LambdustError::type_error("Second argument must be an integer".to_string())),
    };

    if n < 0 {
        return Err(LambdustError::runtime_error("Cannot drop negative number of characters".to_string()));
    }

    let chars: Vec<char> = string.chars().collect();
    let len = chars.len();
    let drop_count = std::cmp::min(n as usize, len);
    let take_count = len.saturating_sub(drop_count);
    let result: String = chars.into_iter().take(take_count).collect();

    Ok(Value::String(result))
}

// Placeholder functions for padding and trimming operations that need full implementation

/// Create string-pad function (placeholder)
fn string_pad_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-pad".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error("string-pad not yet implemented".to_string())),
    })
}

/// Create string-pad-right function (placeholder)
fn string_pad_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-pad-right".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error("string-pad-right not yet implemented".to_string())),
    })
}

/// Create string-trim function (placeholder)
fn string_trim_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-trim".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error("string-trim requires evaluator integration".to_string())),
    })
}

/// Create string-trim-right function (placeholder)
fn string_trim_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-trim-right".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error("string-trim-right requires evaluator integration".to_string())),
    })
}

/// Create string-trim-both function (placeholder)
fn string_trim_both_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-trim-both".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error("string-trim-both requires evaluator integration".to_string())),
    })
}

/// Create string-replace function (placeholder)
fn string_replace_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-replace".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error("string-replace not yet implemented".to_string())),
    })
}

/// Create string-filter function (placeholder)
fn string_filter_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-filter".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error("string-filter requires evaluator integration".to_string())),
    })
}

/// Create string-delete function (placeholder)
fn string_delete_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-delete".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error("string-delete requires evaluator integration".to_string())),
    })
}