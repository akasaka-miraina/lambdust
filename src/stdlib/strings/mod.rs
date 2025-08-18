//! String manipulation functions for the Lambdust standard library.
//!
//! This module implements R7RS-compliant string operations and SRFI-13
//! String Library including comprehensive string processing utilities.
//! 
//! The module is organized into the following sub-modules:
//! - `common`: Common utilities and helper functions
//! - `basic`: Basic string operations (make-string, string, string-length, etc.)
//! - `predicates`: String predicates (string?, string-null?, etc.)

use crate::eval::value::ThreadSafeEnvironment;
use std::sync::Arc;

// Sub-modules containing different categories of string operations
pub mod common;
pub mod basic;
pub mod predicates;

// Re-export commonly used functions
pub use common::{
    CharacterSet,
    validate_string_bounds,
    extract_string,
    extract_string_owned, 
    extract_character,
    extract_char_at_index,
    substring_by_chars,
};

pub use basic::{
    primitive_make_string,
    primitive_string,
    primitive_string_length,
    primitive_string_copy,
};

pub use predicates::{
    primitive_string_null_p,
};

/// Creates string operation bindings for the standard library.
/// 
/// This is the main entry point for setting up all string-related
/// functions in the environment.
pub fn create_string_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // String creation and basic operations
    basic::bind_string_creation_operations(env);
    
    // String predicates
    predicates::bind_string_predicates(env);
    
    // For now, bind the remaining operations from the original module
    // TODO: Split these into additional sub-modules
    bind_remaining_string_operations(env);
}

/// Temporary function to bind operations not yet moved to sub-modules
fn bind_remaining_string_operations(env: &Arc<ThreadSafeEnvironment>) {
    // This function will be removed as we move more operations to sub-modules
    // For now, we'll create simple stubs or use the original functions
    
    // String comparison operations
    use crate::effects::Effect;
    use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl};
    
    // string=?
    env.define("string=?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "string=?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(|args| {
            if args.len() < 2 {
                return Err(Box::new(crate::diagnostics::Error::runtime_error(
                    "string=? requires at least 2 arguments".to_string(),
                    None,
                )));
            }
            
            let first = args[0].as_string().ok_or_else(|| {
                crate::diagnostics::Error::runtime_error(
                    "string=? requires string arguments".to_string(),
                    None,
                )
            })?;
            
            for arg in &args[1..] {
                let s = arg.as_string().ok_or_else(|| {
                    crate::diagnostics::Error::runtime_error(
                        "string=? requires string arguments".to_string(),
                        None,
                    )
                })?;
                if first != s {
                    return Ok(Value::boolean(false));
                }
            }
            
            Ok(Value::boolean(true))
        }),
        effects: vec![Effect::Pure],
    })));
    
    // string-ref (simplified implementation)
    env.define("string-ref".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "string-ref".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(|args| {
            if args.len() != 2 {
                return Err(Box::new(crate::diagnostics::Error::runtime_error(
                    format!("string-ref expects 2 arguments, got {}", args.len()),
                    None,
                )));
            }
            
            let s = args[0].as_string().ok_or_else(|| {
                crate::diagnostics::Error::runtime_error(
                    "string-ref requires string argument".to_string(),
                    None,
                )
            })?;
            
            let index = args[1].as_integer().ok_or_else(|| {
                crate::diagnostics::Error::runtime_error(
                    "string-ref index must be an integer".to_string(),
                    None,
                )
            })? as usize;
            
            if let Some(ch) = extract_char_at_index(s, index) {
                Ok(Value::Literal(crate::ast::Literal::Character(ch)))
            } else {
                Err(Box::new(crate::diagnostics::Error::runtime_error(
                    format!("string-ref index {} out of bounds", index),
                    None,
                )))
            }
        }),
        effects: vec![Effect::Pure],
    })));
}