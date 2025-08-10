//! List FFI functions for data structure operations.
//!
//! This module provides FFI functions for list operations including
//! length calculation, mapping, filtering, and other data structure manipulations.

#![allow(missing_docs)]

use super::*;
use crate::eval::Value;

pub struct ListLengthFunction;

impl FfiFunction for ListLengthFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "list-length".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["list".to_string()],
            return_type: "number".to_string(),
            documentation: Some("Returns the length of a list.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let list = Vec::<Value>::from_lambdust(&args[0])?;
        Ok((list.len() as i64).to_lambdust())
    }
}

pub struct ListMapFunction;

impl FfiFunction for ListMapFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "list-map".to_string(),
            arity: AritySpec::Exact(2),
            parameter_types: vec!["procedure".to_string(), "list".to_string()],
            return_type: "list".to_string(),
            documentation: Some("Maps a function over a list.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        // For now, return the list unchanged
        // In a real implementation, this would apply the function to each element
        let list = Vec::<Value>::from_lambdust(&args[1])?;
        Ok(list.to_lambdust())
    }
}

pub struct ListFilterFunction;

impl FfiFunction for ListFilterFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "list-filter".to_string(),
            arity: AritySpec::Exact(2),
            parameter_types: vec!["procedure".to_string(), "list".to_string()],
            return_type: "list".to_string(),
            documentation: Some("Filters a list using a predicate function.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        // For now, return the list unchanged
        // In a real implementation, this would filter elements based on the predicate
        let list = Vec::<Value>::from_lambdust(&args[1])?;
        Ok(list.to_lambdust())
    }
}