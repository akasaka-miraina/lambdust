//! SRFI-180: JSON support for Lambdust Scheme
//!
//! This module provides comprehensive JSON support according to SRFI-180 specification,
//! including streaming parsers, generators, and RFC 8259 compliant JSON processing.
//!
//! ## Core Procedures
//!
//! ### Basic I/O
//! - `json-read`: Read JSON from input port
//! - `json-write`: Write JSON to output port
//!
//! ### Streaming/Generator Support
//! - `json-generator`: Create generator for streaming JSON parsing
//! - `json-fold`: Fold over JSON elements with streaming
//! - `json-accumulator`: Incremental JSON construction
//!
//! ### Format Support
//! - `json-lines-read`: Read JSON Lines format
//! - `json-sequence-read`: Read JSON Text Sequences (RFC 7464)
//!
//! ### Error Handling
//! - `json-error?`: Check if value is JSON error
//! - `json-error-reason`: Get error description
//!
//! ## Value Mappings
//!
//! JSON → Scheme value mappings according to SRFI-180:
//! - `null` → `'null`
//! - `true` → `#t`
//! - `false` → `#f`
//! - Numbers → Numbers (exact/inexact preserved)
//! - Strings → Strings
//! - Arrays → Vectors
//! - Objects → Association lists (alist format)

pub mod error;
pub mod formats;
pub mod generator;
pub mod lexer;
pub mod parser;
pub mod serializer;
pub mod value_conversion;

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use std::sync::Arc;

pub use error::{JsonError, JsonErrorKind, JsonOptions};
pub use formats::{parse_json_lines, parse_json_sequences, serialize_json_lines, serialize_json_sequences};
pub use generator::JsonGenerator;
pub use parser::JsonParser;
pub use serializer::JsonSerializer;
pub use value_conversion::{json_to_scheme, scheme_to_json, JsonValue};

// Re-export core functionality
// Internal module references for primitive implementations

/// Initialize SRFI-180 JSON procedures in the environment.
///
/// This function registers all JSON-related procedures defined in SRFI-180:
/// - Core I/O: json-read, json-write  
/// - Streaming: json-generator, json-fold, json-accumulator
/// - Formats: json-lines-read, json-sequence-read
/// - Error handling: json-error?, json-error-reason
pub fn init_srfi180_json(env: &Arc<ThreadSafeEnvironment>) {
    // Core I/O procedures
    env.define(
        "json-read".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "json-read".to_string(),
            arity_min: 0,
            arity_max: Some(1), // Optional port argument
            implementation: PrimitiveImpl::RustFn(primitive_json_read),
            effects: vec![Effect::IO],
        })),
    );

    env.define(
        "json-write".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "json-write".to_string(),
            arity_min: 1,
            arity_max: Some(2), // value + optional port
            implementation: PrimitiveImpl::RustFn(primitive_json_write),
            effects: vec![Effect::IO],
        })),
    );

    // Streaming procedures
    env.define(
        "json-generator".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "json-generator".to_string(),
            arity_min: 0,
            arity_max: Some(2), // Optional port and options
            implementation: PrimitiveImpl::RustFn(primitive_json_generator),
            effects: vec![Effect::IO],
        })),
    );

    env.define(
        "json-fold".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "json-fold".to_string(),
            arity_min: 3,
            arity_max: Some(4), // proc, seed, generator, optional options
            implementation: PrimitiveImpl::RustFn(primitive_json_fold),
            effects: vec![Effect::IO, Effect::Pure],
        })),
    );

    env.define(
        "json-accumulator".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "json-accumulator".to_string(),
            arity_min: 0,
            arity_max: Some(1), // Optional options
            implementation: PrimitiveImpl::RustFn(primitive_json_accumulator),
            effects: vec![Effect::Pure],
        })),
    );

    // Format-specific procedures
    env.define(
        "json-lines-read".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "json-lines-read".to_string(),
            arity_min: 0,
            arity_max: Some(1), // Optional port
            implementation: PrimitiveImpl::RustFn(primitive_json_lines_read),
            effects: vec![Effect::IO],
        })),
    );

    env.define(
        "json-sequence-read".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "json-sequence-read".to_string(),
            arity_min: 0,
            arity_max: Some(1), // Optional port
            implementation: PrimitiveImpl::RustFn(primitive_json_sequence_read),
            effects: vec![Effect::IO],
        })),
    );

    // Error handling procedures
    env.define(
        "json-error?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "json-error?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_json_error_p),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "json-error-reason".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "json-error-reason".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_json_error_reason),
            effects: vec![Effect::Pure],
        })),
    );
}

// Primitive implementations

/// Implementation of json-read procedure
fn primitive_json_read(args: &[Value]) -> Result<Value> {
    let port = if args.is_empty() {
        // Use current input port - for now, return error
        return Err(Box::new(DiagnosticError::runtime_error(
            "json-read: current input port not supported yet".to_string(),
            None,
        )));
    } else {
        &args[0]
    };
    
    // TODO: Extract string from port and parse JSON
    // For now, if it's a string, parse it directly
    match port {
        Value::Literal(crate::ast::literal::Literal::String(s)) => {
            match parser::parse_json(s) {
                Ok(json_value) => {
                    match json_to_scheme(&json_value) {
                        Ok(scheme_value) => Ok(scheme_value),
                        Err(json_error) => Err(Box::new(DiagnosticError::from(json_error))),
                    }
                }
                Err(json_error) => Err(Box::new(DiagnosticError::from(json_error))),
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "json-read: expected string or port".to_string(),
            None,
        ))),
    }
}

/// Implementation of json-write procedure
fn primitive_json_write(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "json-write: expected at least 1 argument".to_string(),
            None,
        )));
    }
    
    let value = &args[0];
    let _port = if args.len() > 1 { Some(&args[1]) } else { None };
    
    // Serialize value to JSON
    match serializer::serialize_to_json(value) {
        Ok(json_string) => {
            // TODO: Write to port - for now return the JSON string
            Ok(Value::string(json_string))
        }
        Err(json_error) => Err(Box::new(DiagnosticError::from(json_error))),
    }
}

/// Implementation of json-generator procedure
fn primitive_json_generator(args: &[Value]) -> Result<Value> {
    let _port = if args.is_empty() {
        None
    } else {
        Some(&args[0])
    };
    
    // TODO: Create generator from port
    // For now, return a basic generator that signals EOF
    Ok(Value::symbol_from_str("*eof-object*"))
}

/// Implementation of json-fold procedure
fn primitive_json_fold(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "json-fold: expected at least 3 arguments (proc seed generator)".to_string(),
            None,
        )));
    }
    
    let _proc = &args[0];
    let seed = &args[1];
    let _generator = &args[2];
    
    // TODO: Implement folding over generator values
    // For now, return the seed unchanged
    Ok(seed.clone())
}

/// Implementation of json-accumulator procedure
fn primitive_json_accumulator(args: &[Value]) -> Result<Value> {
    let _options = if args.is_empty() { None } else { Some(&args[0]) };
    
    // Create a new accumulator procedure
    // TODO: Implement proper accumulator closure
    let accumulator_proc = Arc::new(PrimitiveProcedure {
        name: "json-accumulator-closure".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(|args| {
            if args.is_empty() {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "json-accumulator: expected value to accumulate".to_string(),
                    None,
                )));
            }
            
            let value = &args[0];
            match serializer::serialize_to_json(value) {
                Ok(json_string) => Ok(Value::string(json_string)),
                Err(json_error) => Err(Box::new(DiagnosticError::from(json_error))),
            }
        }),
        effects: vec![Effect::Pure],
    });
    
    Ok(Value::Primitive(accumulator_proc))
}

/// Implementation of json-lines-read procedure
fn primitive_json_lines_read(args: &[Value]) -> Result<Value> {
    let port = if args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "json-lines-read: current input port not supported yet".to_string(),
            None,
        )));
    } else {
        &args[0]
    };
    
    match port {
        Value::Literal(crate::ast::literal::Literal::String(s)) => {
            match formats::parse_json_lines(s) {
                Ok(values) => Ok(Value::list(values)),
                Err(json_error) => Err(Box::new(DiagnosticError::from(json_error))),
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "json-lines-read: expected string or port".to_string(),
            None,
        ))),
    }
}

/// Implementation of json-sequence-read procedure
fn primitive_json_sequence_read(args: &[Value]) -> Result<Value> {
    let port = if args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "json-sequence-read: current input port not supported yet".to_string(),
            None,
        )));
    } else {
        &args[0]
    };
    
    match port {
        Value::Literal(crate::ast::literal::Literal::String(s)) => {
            match formats::parse_json_sequences(s) {
                Ok(values) => Ok(Value::list(values)),
                Err(json_error) => Err(Box::new(DiagnosticError::from(json_error))),
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "json-sequence-read: expected string or port".to_string(),
            None,
        ))),
    }
}

/// Implementation of json-error? predicate
fn primitive_json_error_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "json-error?: expected exactly 1 argument".to_string(),
            None,
        )));
    }
    
    // Check if the value is a JSON error
    // TODO: Implement proper JSON error type checking
    // For now, check if it's an error symbol or has error properties
    match &args[0] {
        Value::Symbol(_sym) => {
            // TODO: Check if the symbol represents a JSON error
            // For now, return false since we don't have error symbols yet
            Ok(Value::boolean(false))
        }
        _ => Ok(Value::boolean(false)),
    }
}

/// Implementation of json-error-reason procedure
fn primitive_json_error_reason(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "json-error-reason: expected exactly 1 argument".to_string(),
            None,
        )));
    }
    
    // Extract error reason from JSON error object
    match &args[0] {
        Value::Symbol(_sym) => {
            // TODO: Extract error reason from JSON error symbols
            // For now, return a generic error message
            Ok(Value::string("unknown error"))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "json-error-reason: argument is not a JSON error".to_string(),
            None,
        ))),
    }
}