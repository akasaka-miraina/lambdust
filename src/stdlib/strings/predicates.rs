//! String predicates (string?, string-null?, etc.)

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use crate::stdlib::strings::common::bind_primitive;
use std::sync::Arc;

/// Binds string predicates.
pub fn bind_string_predicates(env: &Arc<ThreadSafeEnvironment>) {
    bind_primitive!(env, "string?", 1, Some(1), primitive_string_p, vec![Effect::Pure]);
    bind_primitive!(env, "string-null?", 1, Some(1), primitive_string_null_p, vec![Effect::Pure]);
}

/// string? predicate
fn primitive_string_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::boolean(args[0].is_string()))
}

/// string-null? predicate
pub fn primitive_string_null_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-null? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    if let Some(s) = args[0].as_string() {
        Ok(Value::boolean(s.is_empty()))
    } else {
        Err(Box::new(DiagnosticError::runtime_error(
            "string-null? requires string argument".to_string(),
            None,
        )))
    }
}