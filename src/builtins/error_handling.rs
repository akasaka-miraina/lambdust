//! Error handling functions for Scheme

use crate::error::LambdustError;
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register all error handling functions
pub fn register_error_functions(builtins: &mut HashMap<String, Value>) {
    builtins.insert("error".to_string(), error_function());
}

/// Implements the `error` function for raising errors
fn error_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "error".to_string(),
        arity: None, // Variadic - can take message and optional irritants
        func: |args| {
            if args.is_empty() {
                return Err(LambdustError::RuntimeError {
                    message: "error: expected at least one argument".to_string(),
                    location: crate::error::SourceSpan::unknown(),
                    stack_trace: Vec::new(),
                });
            }

            // First argument should be the error message
            let message = match &args[0] {
                Value::String(s) => s.clone(),
                Value::Symbol(s) => s.clone(),
                other => format!("{}", other),
            };

            // Additional arguments are irritants (values that provide context)
            let mut full_message = message;
            if args.len() > 1 {
                full_message.push_str(": ");
                for (i, irritant) in args[1..].iter().enumerate() {
                    if i > 0 {
                        full_message.push_str(", ");
                    }
                    full_message.push_str(&format!("{}", irritant));
                }
            }

            Err(LambdustError::RuntimeError {
                message: full_message,
                location: crate::error::SourceSpan::unknown(),
                stack_trace: Vec::new(),
            })
        },
    })
}