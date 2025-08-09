#![allow(ambiguous_glob_reexports)]
#![allow(unused_variables)]
#![allow(dead_code)]
//! Foreign Function Interface (FFI) support for Rust interoperability.
//!
//! This module provides seamless integration with Rust code,
//! allowing Lambdust programs to call Rust functions through the `primitive` special form.
//!
//! # Architecture
//! 
//! The FFI system consists of several key components:
//! - **Function Registry**: Type-safe registration of Rust functions
//! - **Value Marshaling**: Conversion between Rust and Lambdust types
//! - **Error Handling**: Proper error propagation and type checking
//! - **FFI Bridge**: Main interface for calling registered functions
//!
//! # Usage
//!
//! ```lambdust
//! ;; Call a registered Rust function
//! (primitive 'string-length "hello")
//! (primitive 'add-numbers 42 58)
//! (primitive 'list-map (lambda (x) (* x 2)) '(1 2 3 4))
//! ```

#![allow(missing_docs)]

use crate::eval::Value;
use crate::diagnostics::Error;
use std::fmt;

pub mod marshal;
pub mod registry;
pub mod builtins;
pub mod examples;
pub mod library;
pub mod c_types;
pub mod callback;
pub mod memory;
pub mod safety;
#[cfg(feature = "ffi")]
pub mod libffi_integration;
pub mod scheme_api;
pub mod profiling;

// Individual structure modules
pub mod ffi_signature;
pub mod registered_function;
pub mod ffi_stats;
pub mod ffi_registry;
pub mod ffi_bridge;

pub use marshal::*;
pub use registry::*;
pub use builtins::*;
pub use library::*;
pub use c_types::*;
pub use callback::*;
pub use memory::*;
pub use safety::*;
#[cfg(feature = "ffi")]
pub use libffi_integration::*;
pub use scheme_api::*;
#[allow(ambiguous_glob_reexports)]
pub use profiling::*;

// Re-export individual structures
pub use ffi_signature::*;
pub use registered_function::*;
pub use ffi_stats::*;
pub use ffi_registry::*;
pub use ffi_bridge::*;

/// Errors that can occur during FFI operations.
#[derive(Debug, Clone)]
pub enum FfiError {
    /// Function not found in registry
    FunctionNotFound(String),
    /// Wrong number of arguments
    ArityMismatch {
        function: String,
        expected: AritySpec,
        actual: usize,
    },
    /// Type conversion error
    TypeMismatch {
        function: String,
        parameter: usize,
        expected: String,
        actual: String,
    },
    /// Runtime error in FFI function
    RuntimeError {
        function: String,
        message: String,
    },
    /// Invalid function signature
    InvalidSignature(String),
}

impl fmt::Display for FfiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FfiError::FunctionNotFound(name) => {
                write!(f, "FFI function not found: {name}")
            }
            FfiError::ArityMismatch { function, expected, actual } => {
                write!(f, "FFI function '{function}' expects {expected} arguments, got {actual}")
            }
            FfiError::TypeMismatch { function, parameter, expected, actual } => {
                write!(f, "FFI function '{function}' parameter {parameter}: expected {expected}, got {actual}")
            }
            FfiError::RuntimeError { function, message } => {
                write!(f, "FFI function '{function}' runtime error: {message}")
            }
            FfiError::InvalidSignature(msg) => {
                write!(f, "Invalid FFI function signature: {msg}")
            }
        }
    }
}

impl std::error::Error for FfiError {}

impl From<FfiError> for Error {
    fn from(ffi_error: FfiError) -> Self {
        Error::runtime_error(ffi_error.to_string(), None)
    }
}

/// Specification for function arity (argument count).
#[derive(Debug, Clone, PartialEq)]
pub enum AritySpec {
    /// Exact number of arguments
    Exact(usize),
    /// Minimum number of arguments (variadic)
    AtLeast(usize),
    /// Range of acceptable argument counts
    Range(usize, usize),
}

impl fmt::Display for AritySpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AritySpec::Exact(n) => write!(f, "exactly {n}"),
            AritySpec::AtLeast(n) => write!(f, "at least {n}"),
            AritySpec::Range(min, max) => write!(f, "between {min} and {max}"),
        }
    }
}

impl AritySpec {
    /// Checks if the given argument count satisfies this arity specification.
    pub fn check(&self, arg_count: usize) -> bool {
        match self {
            AritySpec::Exact(n) => arg_count == *n,
            AritySpec::AtLeast(n) => arg_count >= *n,
            AritySpec::Range(min, max) => arg_count >= *min && arg_count <= *max,
        }
    }
}

// FfiSignature moved to ffi_signature.rs

/// Trait for FFI function implementations.
///
/// This trait provides a type-safe way to implement FFI functions
/// with automatic argument validation and error handling.
pub trait FfiFunction: Send + Sync {
    /// Get the function signature
    fn signature(&self) -> &FfiSignature;
    
    /// Call the function with the given arguments
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError>;
    
    /// Validate arguments before calling (optional optimization)
    fn validate_args(&self, args: &[Value]) -> std::result::Result<(), FfiError> {
        let sig = self.signature();
        
        // Check arity
        if !sig.arity.check(args.len()) {
            return Err(FfiError::ArityMismatch {
                function: sig.name.clone(),
                expected: sig.arity.clone(),
                actual: args.len(),
            });
        }
        
        // Type checking can be implemented by specific functions
        Ok(())
    }
}

// RegisteredFunction moved to registered_function.rs

// FfiRegistry moved to ffi_registry.rs

// FfiStats moved to ffi_stats.rs

// FfiRegistry implementations moved to ffi_registry.rs

// FfiBridge moved to ffi_bridge.rs

// FfiBridge implementations moved to ffi_bridge.rs