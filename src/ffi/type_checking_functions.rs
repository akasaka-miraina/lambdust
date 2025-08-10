//! Type checking FFI functions for runtime type inspection.
//!
//! This module provides FFI functions for checking the types of Scheme values
//! at runtime, supporting predicates for numbers, strings, lists, booleans, and other types.

#![allow(missing_docs)]

use super::*;
use crate::eval::Value;

pub struct IsNumberFunction;

impl FfiFunction for IsNumberFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "number?".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["any".to_string()],
            return_type: "boolean".to_string(),
            documentation: Some("Checks if a value is a number.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        Ok(args[0].is_number().to_lambdust())
    }
}

pub struct IsStringFunction;

impl FfiFunction for IsStringFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "string?".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["any".to_string()],
            return_type: "boolean".to_string(),
            documentation: Some("Checks if a value is a string.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        Ok(args[0].is_string().to_lambdust())
    }
}

pub struct IsListFunction;

impl FfiFunction for IsListFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "list?".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["any".to_string()],
            return_type: "boolean".to_string(),
            documentation: Some("Checks if a value is a list.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        Ok(args[0].is_list().to_lambdust())
    }
}

pub struct IsBooleanFunction;

impl FfiFunction for IsBooleanFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "boolean?".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["any".to_string()],
            return_type: "boolean".to_string(),
            documentation: Some("Checks if a value is a boolean.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let is_bool = matches!(args[0], Value::Literal(crate::ast::Literal::Boolean(_)));
        Ok(is_bool.to_lambdust())
    }
}