//! String FFI functions for text manipulation operations.
//!
//! This module provides FFI functions for string operations including
//! length calculation, concatenation, case conversion, and other text processing.

#![allow(missing_docs)]

use super::*;
use crate::eval::Value;

pub struct StringLengthFunction;

impl FfiFunction for StringLengthFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "string-length".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["string".to_string()],
            return_type: "number".to_string(),
            documentation: Some("Returns the length of a string.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let s = String::from_lambdust(&args[0])?;
        Ok((s.chars().count() as i64).to_lambdust())
    }
}

pub struct StringConcatFunction;

impl FfiFunction for StringConcatFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "string-concat".to_string(),
            arity: AritySpec::AtLeast(0),
            parameter_types: vec!["string".to_string()],
            return_type: "string".to_string(),
            documentation: Some("Concatenates multiple strings together.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let mut result = String::new();
        for arg in args {
            if let Some(s) = arg.as_string() {
                result.push_str(s);
            }
        }
        Ok(result.to_lambdust())
    }
}

pub struct StringUpperFunction;

impl FfiFunction for StringUpperFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "string-upper".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["string".to_string()],
            return_type: "string".to_string(),
            documentation: Some("Converts a string to uppercase.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let s = String::from_lambdust(&args[0])?;
        Ok(s.to_uppercase().to_lambdust())
    }
}

pub struct StringLowerFunction;

impl FfiFunction for StringLowerFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "string-lower".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["string".to_string()],
            return_type: "string".to_string(),
            documentation: Some("Converts a string to lowercase.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let s = String::from_lambdust(&args[0])?;
        Ok(s.to_lowercase().to_lambdust())
    }
}