//! SRFI built-in functions
//!
//! This module provides built-in functions for SRFI access and management,
//! implementing SRFI 97 functions for library inquiry.

use crate::error::{LambdustError, Result};
use crate::srfi::SrfiRegistry;
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register SRFI functions into the builtins map
pub fn register_srfi_functions(builtins: &mut HashMap<String, Value>) {
    builtins.insert("srfi-available?".to_string(), srfi_available_function());
    builtins.insert(
        "srfi-supported-ids".to_string(),
        srfi_supported_ids_function(),
    );
    builtins.insert("srfi-name".to_string(), srfi_name_function());
    builtins.insert("srfi-parts".to_string(), srfi_parts_function());
}

/// Implementation of srfi-available? function
pub fn srfi_available_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "srfi-available?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match &args[0] {
                Value::Number(n) => {
                    let id = extract_integer_from_number(n)?;
                    let registry = SrfiRegistry::with_standard_srfis();
                    Ok(Value::Boolean(registry.has_srfi(id)))
                }
                _ => Err(LambdustError::type_error(
                    "srfi-available? expects a number".to_string(),
                )),
            }
        },
    })
}

/// Implementation of srfi-supported-ids function
pub fn srfi_supported_ids_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "srfi-supported-ids".to_string(),
        arity: Some(0),
        func: |args| {
            if !args.is_empty() {
                return Err(LambdustError::arity_error(0, args.len()));
            }

            let registry = SrfiRegistry::with_standard_srfis();
            let ids = registry.available_srfis();

            let id_values: Vec<Value> = ids
                .into_iter()
                .map(|id| Value::Number(crate::lexer::SchemeNumber::Integer(id as i64)))
                .collect();

            Ok(Value::Vector(id_values))
        },
    })
}

/// Implementation of srfi-name function
pub fn srfi_name_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "srfi-name".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match &args[0] {
                Value::Number(n) => {
                    let id = extract_integer_from_number(n)?;
                    let registry = SrfiRegistry::with_standard_srfis();

                    if let Some((_, name, _)) = registry.get_srfi_info(id) {
                        Ok(Value::String(name.to_string()))
                    } else {
                        Err(LambdustError::runtime_error(format!(
                            "Unknown SRFI: {}",
                            id
                        )))
                    }
                }
                _ => Err(LambdustError::type_error(
                    "srfi-name expects a number".to_string(),
                )),
            }
        },
    })
}

/// Implementation of srfi-parts function
pub fn srfi_parts_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "srfi-parts".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match &args[0] {
                Value::Number(n) => {
                    let id = extract_integer_from_number(n)?;
                    let registry = SrfiRegistry::with_standard_srfis();

                    if let Some((_, _, parts)) = registry.get_srfi_info(id) {
                        let part_values: Vec<Value> = parts
                            .into_iter()
                            .map(|s| Value::String(s.to_string()))
                            .collect();
                        Ok(Value::Vector(part_values))
                    } else {
                        Err(LambdustError::runtime_error(format!(
                            "Unknown SRFI: {}",
                            id
                        )))
                    }
                }
                _ => Err(LambdustError::type_error(
                    "srfi-parts expects a number".to_string(),
                )),
            }
        },
    })
}

/// Helper function to extract integer from SchemeNumber
pub fn extract_integer_from_number(n: &crate::lexer::SchemeNumber) -> Result<u32> {
    match n {
        crate::lexer::SchemeNumber::Integer(i) if *i >= 0 => Ok(*i as u32),
        crate::lexer::SchemeNumber::Real(f) if *f >= 0.0 && f.fract() == 0.0 => Ok(*f as u32),
        crate::lexer::SchemeNumber::Rational(num, den) if *num >= 0 && *den > 0 => {
            let result = *num / *den;
            if result >= 0 {
                Ok(result as u32)
            } else {
                Err(LambdustError::type_error(
                    "SRFI ID must be non-negative".to_string(),
                ))
            }
        }
        crate::lexer::SchemeNumber::Complex(real, _) if *real >= 0.0 && real.fract() == 0.0 => {
            Ok(*real as u32)
        }
        _ => Err(LambdustError::type_error(
            "SRFI ID must be a non-negative integer".to_string(),
        )),
    }
}
