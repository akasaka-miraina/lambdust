//! SRFI-13 String Transform Operations
//!
//! This module provides SRFI-13 compliant string transformation functions
//! for the Lambdust standard library.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::stdlib::strings::common::extract_string;
use std::sync::Arc;

/// Binds SRFI-13 string transform operations to the environment
pub fn bind_transform_operations(env: &Arc<ThreadSafeEnvironment>) {
    // string-upcase
    env.define(
        "string-upcase".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-upcase".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_string_upcase),
            effects: vec![Effect::Pure],
        })),
    );

    // string-downcase
    env.define(
        "string-downcase".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-downcase".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_string_downcase),
            effects: vec![Effect::Pure],
        })),
    );

    // string-foldcase
    env.define(
        "string-foldcase".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-foldcase".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_string_foldcase),
            effects: vec![Effect::Pure],
        })),
    );

    // string-titlecase (SRFI-13 specific)
    env.define(
        "string-titlecase".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-titlecase".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_string_titlecase),
            effects: vec![Effect::Pure],
        })),
    );
}

/// string-upcase - Convert string to uppercase
fn primitive_string_upcase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-upcase expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    let s = extract_string(&args[0], "string-upcase")?;
    let result = s.to_uppercase();
    Ok(Value::string(result))
}

/// string-downcase - Convert string to lowercase
fn primitive_string_downcase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-downcase expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    let s = extract_string(&args[0], "string-downcase")?;
    let result = s.to_lowercase();
    Ok(Value::string(result))
}

/// string-foldcase - Convert string to folded case for case-insensitive comparison
fn primitive_string_foldcase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-foldcase expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    let s = extract_string(&args[0], "string-foldcase")?;
    // For Unicode-aware case folding, we use to_lowercase as a reasonable approximation
    // A full implementation would use proper Unicode case folding rules
    let result = s.to_lowercase();
    Ok(Value::string(result))
}

/// string-titlecase - Convert string to title case
fn primitive_string_titlecase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-titlecase expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    let s = extract_string(&args[0], "string-titlecase")?;

    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in s.chars() {
        if ch.is_alphabetic() {
            if capitalize_next {
                result.extend(ch.to_uppercase());
                capitalize_next = false;
            } else {
                result.extend(ch.to_lowercase());
            }
        } else {
            result.push(ch);
            if ch.is_whitespace() {
                capitalize_next = true;
            }
        }
    }

    Ok(Value::string(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_upcase() {
        let args = vec![Value::string("Hello World!")];
        let result = primitive_string_upcase(&args).unwrap();
        assert_eq!(result.as_string(), Some("HELLO WORLD!"));

        // Test empty string
        let args = vec![Value::string("")];
        let result = primitive_string_upcase(&args).unwrap();
        assert_eq!(result.as_string(), Some(""));

        // Test Unicode
        let args = vec![Value::string("café")];
        let result = primitive_string_upcase(&args).unwrap();
        assert_eq!(result.as_string(), Some("CAFÉ"));
    }

    #[test]
    fn test_string_downcase() {
        let args = vec![Value::string("HELLO WORLD!")];
        let result = primitive_string_downcase(&args).unwrap();
        assert_eq!(result.as_string(), Some("hello world!"));

        // Test mixed case
        let args = vec![Value::string("HeLLo WoRLd!")];
        let result = primitive_string_downcase(&args).unwrap();
        assert_eq!(result.as_string(), Some("hello world!"));

        // Test Unicode
        let args = vec![Value::string("CAFÉ")];
        let result = primitive_string_downcase(&args).unwrap();
        assert_eq!(result.as_string(), Some("café"));
    }

    #[test]
    fn test_string_foldcase() {
        let args = vec![Value::string("HeLLo WoRLd!")];
        let result = primitive_string_foldcase(&args).unwrap();
        assert_eq!(result.as_string(), Some("hello world!"));

        // Test German ß character (special case folding)
        let args = vec![Value::string("Straße")];
        let result = primitive_string_foldcase(&args).unwrap();
        assert_eq!(result.as_string(), Some("straße"));
    }

    #[test]
    fn test_string_titlecase() {
        let args = vec![Value::string("hello world")];
        let result = primitive_string_titlecase(&args).unwrap();
        assert_eq!(result.as_string(), Some("Hello World"));

        // Test multiple words
        let args = vec![Value::string("the quick brown fox")];
        let result = primitive_string_titlecase(&args).unwrap();
        assert_eq!(result.as_string(), Some("The Quick Brown Fox"));

        // Test with punctuation
        let args = vec![Value::string("hello, world!")];
        let result = primitive_string_titlecase(&args).unwrap();
        assert_eq!(result.as_string(), Some("Hello, World!"));

        // Test already titlecased
        let args = vec![Value::string("Already Title Case")];
        let result = primitive_string_titlecase(&args).unwrap();
        assert_eq!(result.as_string(), Some("Already Title Case"));
    }

    #[test]
    fn test_transform_arity_errors() {
        // Test wrong number of arguments
        let result = primitive_string_upcase(&[]);
        assert!(result.is_err());

        let result = primitive_string_upcase(&[Value::string("a"), Value::string("b")]);
        assert!(result.is_err());

        let result = primitive_string_downcase(&[]);
        assert!(result.is_err());

        let result = primitive_string_foldcase(&[]);
        assert!(result.is_err());

        let result = primitive_string_titlecase(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_non_string_args() {
        // Test with non-string arguments
        let result = primitive_string_upcase(&[Value::integer(42)]);
        assert!(result.is_err());

        let result = primitive_string_downcase(&[Value::boolean(true)]);
        assert!(result.is_err());
    }
}
