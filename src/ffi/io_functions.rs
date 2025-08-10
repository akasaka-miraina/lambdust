//! I/O FFI functions for printing and output operations.
//!
//! This module provides FFI functions for basic I/O operations including
//! print and println functions for outputting values to standard output.

#![allow(missing_docs)]

use super::*;
use crate::eval::Value;

pub struct PrintFunction;

impl FfiFunction for PrintFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "print".to_string(),
            arity: AritySpec::AtLeast(0),
            parameter_types: vec!["any".to_string()],
            return_type: "unspecified".to_string(),
            documentation: Some("Prints values to standard output.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{arg}");
        }
        Ok(Value::Unspecified)
    }
}

pub struct PrintlnFunction;

impl FfiFunction for PrintlnFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "println".to_string(),
            arity: AritySpec::AtLeast(0),
            parameter_types: vec!["any".to_string()],
            return_type: "unspecified".to_string(),
            documentation: Some("Prints values to standard output followed by a newline.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{arg}");
        }
        println!();
        Ok(Value::Unspecified)
    }
}