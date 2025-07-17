//! Miscellaneous functions for Scheme

use crate::error::LambdustError;
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register all miscellaneous functions
pub fn register_misc_functions(builtins: &mut HashMap<String, Value>) {
    // Multiple values functions
    builtins.insert("values".to_string(), values_function());
    // call-with-values is handled as a special form in the evaluator

    // Record operations (SRFI 9)
    builtins.insert("make-record".to_string(), record_make());
    builtins.insert("record-of-type?".to_string(), record_predicate());
    builtins.insert("record-field".to_string(), record_field_get());
    builtins.insert("record-set-field!".to_string(), record_field_set());
}

// Multiple values functions

/// Implements the `values` function for creating multiple values
fn values_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "values".to_string(),
        arity: None, // Variadic - can take any number of arguments
        func: |args| Ok(Value::Values(args.to_vec())),
    })
}

// Record operations (SRFI 9)

/// Creates a record instance
fn record_make() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "make-record".to_string(),
        arity: None, // Variadic
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            // First argument should be the record type (as a symbol/string)
            let type_name = match &args[0] {
                Value::Symbol(s) => s.clone(),
                Value::String(s) => s.clone(),
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "make-record: expected type name as symbol or string, got {}",
                        args[0]
                    )));
                }
            };

            // Remaining arguments are field values
            let field_values = args[1..].to_vec();

            // Create a basic record type and record
            let record_type = crate::value::RecordType {
                name: type_name,
                field_names: (0..field_values.len())
                    .map(|i| format!("field{i}"))
                    .collect(),
                constructor_name: "make-record".to_string(),
                predicate_name: "record?".to_string(),
            };

            let record = crate::value::Record {
                record_type,
                fields: field_values,
            };

            Ok(Value::Record(record))
        },
    })
}

/// Checks if a value is a record of a specific type
fn record_predicate() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "record-of-type?".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            let type_name = match &args[1] {
                Value::Symbol(s) => s.clone(),
                Value::String(s) => s.clone(),
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "record-of-type?: expected type name as symbol or string, got {}",
                        args[1]
                    )));
                }
            };

            Ok(Value::Boolean(args[0].is_record_of_type(&type_name)))
        },
    })
}

/// Gets a field value from a record
fn record_field_get() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "record-field".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            let Some(record) = args[0].as_record() else {
                    return Err(LambdustError::type_error(format!(
                        "record-field: expected record, got {}",
                        args[0]
                    )));
                };

            let index = match args[1].as_number() {
                Some(crate::lexer::SchemeNumber::Integer(i)) if *i >= 0 => *i as usize,
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "record-field: expected non-negative integer index, got {}",
                        args[1]
                    )));
                }
            };

            if index >= record.fields.len() {
                return Err(LambdustError::runtime_error(format!(
                    "record-field: index {} out of bounds for record with {} fields",
                    index,
                    record.fields.len()
                )));
            }

            Ok(record.fields[index].clone())
        },
    })
}

/// Sets a field value in a record (returns new record)
fn record_field_set() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "record-set-field!".to_string(),
        arity: Some(3),
        func: |args| {
            if args.len() != 3 {
                return Err(LambdustError::arity_error(3, args.len()));
            }

            let Some(record) = args[0].as_record() else {
                    return Err(LambdustError::type_error(format!(
                        "record-set-field!: expected record, got {}",
                        args[0]
                    )));
                };

            let index = match args[1].as_number() {
                Some(crate::lexer::SchemeNumber::Integer(i)) if *i >= 0 => *i as usize,
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "record-set-field!: expected non-negative integer index, got {}",
                        args[1]
                    )));
                }
            };

            if index >= record.fields.len() {
                return Err(LambdustError::runtime_error(format!(
                    "record-set-field!: index {} out of bounds for record with {} fields",
                    index,
                    record.fields.len()
                )));
            }

            // Create new record with updated field
            let mut new_fields = record.fields.clone();
            new_fields[index] = args[2].clone();

            let new_record = crate::value::Record {
                record_type: record.record_type.clone(),
                fields: new_fields,
            };

            Ok(Value::Record(new_record))
        },
    })
}
