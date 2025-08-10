//! Arithmetic FFI functions for mathematical operations.
//!
//! This module provides FFI functions for basic arithmetic operations including
//! addition, subtraction, multiplication, and division with proper error handling.

#![allow(missing_docs)]

use super::*;
use crate::eval::Value;

pub struct AddFunction;

impl FfiFunction for AddFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "add".to_string(),
            arity: AritySpec::Exact(2),
            parameter_types: vec!["number".to_string(), "number".to_string()],
            return_type: "number".to_string(),
            documentation: Some("Adds two numbers together.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let a = f64::from_lambdust(&args[0])?;
        let b = f64::from_lambdust(&args[1])?;
        Ok((a + b).to_lambdust())
    }
}

pub struct SubtractFunction;

impl FfiFunction for SubtractFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "subtract".to_string(),
            arity: AritySpec::Exact(2),
            parameter_types: vec!["number".to_string(), "number".to_string()],
            return_type: "number".to_string(),
            documentation: Some("Subtracts the second number from the first.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let a = f64::from_lambdust(&args[0])?;
        let b = f64::from_lambdust(&args[1])?;
        Ok((a - b).to_lambdust())
    }
}

pub struct MultiplyFunction;

impl FfiFunction for MultiplyFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "multiply".to_string(),
            arity: AritySpec::Exact(2),
            parameter_types: vec!["number".to_string(), "number".to_string()],
            return_type: "number".to_string(),
            documentation: Some("Multiplies two numbers.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let a = f64::from_lambdust(&args[0])?;
        let b = f64::from_lambdust(&args[1])?;
        Ok((a * b).to_lambdust())
    }
}

pub struct DivideFunction;

impl FfiFunction for DivideFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "divide".to_string(),
            arity: AritySpec::Exact(2),
            parameter_types: vec!["number".to_string(), "number".to_string()],
            return_type: "number".to_string(),
            documentation: Some("Divides the first number by the second.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let a = f64::from_lambdust(&args[0])?;
        let b = f64::from_lambdust(&args[1])?;
        if b == 0.0 {
            Ok(f64::INFINITY.to_lambdust())
        } else {
            Ok((a / b).to_lambdust())
        }
    }
}