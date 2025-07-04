//! Control flow functions for Scheme

use crate::error::LambdustError;
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register all control flow functions
pub fn register_control_flow_functions(builtins: &mut HashMap<String, Value>) {
    // Note: call/cc and call-with-current-continuation are handled directly
    // in the formal evaluator as special forms, not as builtin functions
    
    builtins.insert("raise".to_string(), raise_function());
    builtins.insert(
        "with-exception-handler".to_string(),
        with_exception_handler_function(),
    );
    builtins.insert("dynamic-wind".to_string(), dynamic_wind_function());
}


/// Implements the `raise` function for raising exceptions
fn raise_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "raise".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            // Convert the raised value to a runtime error
            let message = format!("Uncaught exception: {}", args[0]);
            Err(LambdustError::RuntimeError {
                message,
                context: Box::new(crate::error::ErrorContext::unknown()),
            })
        },
    })
}

/// Implements the `with-exception-handler` function for exception handling
fn with_exception_handler_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "with-exception-handler".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            // First argument should be the exception handler (procedure)
            let _handler = match &args[0] {
                Value::Procedure(_) => &args[0],
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "with-exception-handler: expected procedure as handler, got {}",
                        args[0]
                    )));
                }
            };

            // Second argument should be the thunk (procedure of no arguments)
            let _thunk = match &args[1] {
                Value::Procedure(_) => &args[1],
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "with-exception-handler: expected procedure as thunk, got {}",
                        args[1]
                    )));
                }
            };

            // For now, return a placeholder implementation
            // A complete implementation would require evaluator integration
            Err(LambdustError::RuntimeError {
                message: "with-exception-handler: requires evaluator integration - not yet fully implemented".to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            })
        },
    })
}

/// Implements the `dynamic-wind` function for unwinding and rewinding
fn dynamic_wind_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "dynamic-wind".to_string(),
        arity: Some(3),
        func: |args| {
            if args.len() != 3 {
                return Err(LambdustError::arity_error(3, args.len()));
            }

            // First argument: before thunk (procedure of no arguments)
            let _before = match &args[0] {
                Value::Procedure(_) => &args[0],
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "dynamic-wind: expected procedure as before thunk, got {}",
                        args[0]
                    )));
                }
            };

            // Second argument: during thunk (procedure of no arguments)
            let _during = match &args[1] {
                Value::Procedure(_) => &args[1],
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "dynamic-wind: expected procedure as during thunk, got {}",
                        args[1]
                    )));
                }
            };

            // Third argument: after thunk (procedure of no arguments)
            let _after = match &args[2] {
                Value::Procedure(_) => &args[2],
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "dynamic-wind: expected procedure as after thunk, got {}",
                        args[2]
                    )));
                }
            };

            // For now, return a placeholder implementation
            // A complete implementation would require evaluator integration
            Err(LambdustError::RuntimeError {
                message: "dynamic-wind: requires evaluator integration - not yet fully implemented"
                    .to_string(),
                context: Box::new(crate::error::ErrorContext::unknown()),
            })
        },
    })
}
