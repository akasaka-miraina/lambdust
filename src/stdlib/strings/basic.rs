//! Basic string operations (make-string, string, string-length, string-copy)

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::stdlib::strings::common::{
    bind_primitive, extract_character, extract_string, validate_string_bounds,
};
use std::sync::Arc;

/// Binds basic string creation and manipulation operations.
pub fn bind_string_creation_operations(env: &Arc<ThreadSafeEnvironment>) {
    bind_primitive!(
        env,
        "make-string",
        1,
        Some(2),
        primitive_make_string,
        vec![Effect::Pure]
    );
    bind_primitive!(env, "string", 0, None, primitive_string, vec![Effect::Pure]);
    bind_primitive!(
        env,
        "string-length",
        1,
        Some(1),
        primitive_string_length,
        vec![Effect::Pure]
    );
    bind_primitive!(
        env,
        "string-copy",
        1,
        Some(3),
        primitive_string_copy,
        vec![Effect::Pure]
    );
}

/// make-string procedure
pub fn primitive_make_string(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("make-string expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let length = args[0].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "make-string first argument must be a non-negative integer".to_string(),
            None,
        )
    })?;

    if length < 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "make-string length must be non-negative".to_string(),
            None,
        )));
    }

    let fill_char = if args.len() == 2 {
        extract_character(&args[1], "make-string")?
    } else {
        ' ' // Default fill character (space)
    };

    // R7RS-small specifies that make-string creates mutable strings
    Ok(Value::mutable_string_filled(length as usize, fill_char))
}

/// string constructor from characters
pub fn primitive_string(args: &[Value]) -> Result<Value> {
    let mut result = String::new();

    for arg in args {
        let ch = extract_character(arg, "string")?;
        result.push(ch);
    }

    Ok(Value::string(result))
}

/// string-length procedure
pub fn primitive_string_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-length expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    let length = args[0].string_length().ok_or_else(|| {
        DiagnosticError::runtime_error("string-length requires a string argument".to_string(), None)
    })?;

    Ok(Value::integer(length as i64))
}

/// string-copy procedure
pub fn primitive_string_copy(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-copy expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }

    let s = extract_string(&args[0], "string-copy")?;
    let chars: Vec<char> = s.chars().collect();
    let length = chars.len();

    let start = if args.len() > 1 {
        args[1].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "string-copy start index must be an integer".to_string(),
                None,
            )
        })? as usize
    } else {
        0
    };

    let end = if args.len() > 2 {
        Some(args[2].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "string-copy end index must be an integer".to_string(),
                None,
            )
        })? as usize)
    } else {
        None
    };

    let actual_end = validate_string_bounds(s, start, end, "string-copy")?;

    // Extract substring using character indices
    let result_chars: Vec<char> = chars
        .into_iter()
        .skip(start)
        .take(actual_end - start)
        .collect();
    let result: String = result_chars.into_iter().collect();

    Ok(Value::string(result))
}
