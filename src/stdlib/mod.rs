//! Standard library implementation for Lambdust.
//!
//! This module provides R7RS-compatible standard library functions
//! plus Lambdust-specific extensions including effect system support.
//!
//! The standard library is organized into modules:
//! - `arithmetic`: Number operations and mathematical functions
//! - `strings`: String manipulation and conversion
//! - `lists`: List processing and higher-order functions
//! - `vectors`: Vector operations and conversions
//! - `characters`: Character operations and predicates
//! - `io`: Input/output operations
//! - `control`: Control flow procedures
//! - `types`: Type operations and predicates
//! - `effects`: Effect system integration

#![allow(missing_docs)]

/// Arithmetic operations and mathematical functions.
pub mod arithmetic;
/// Bytevector operations and utilities.
pub mod bytevector;
/// Character operations and predicates.
pub mod characters;
/// Character set operations and SRFI-14 implementation.
pub mod charset;
/// Concurrency primitives and parallel operations.
pub mod concurrency;
/// Control flow procedures (if, cond, case, etc.).
pub mod control;
/// Effect system integration and monadic operations.
pub mod effects;
/// Exception handling and error operations.
pub mod exceptions;
/// Basic input/output operations.
pub mod io;
/// List processing and higher-order functions.
pub mod lists;
/// Parameter objects (SRFI-39) implementation.
pub mod parameters;
/// Simple record types implementation.
pub mod records_simple;
/// Enhanced SRFI-23 error handling.
pub mod srfi23_enhanced;
/// SRFI-9 record types macro system.
pub mod srfi9_macro;
/// String manipulation and conversion operations.
pub mod strings;
/// System interface and process operations.
pub mod system;
/// Type operations, predicates, and reflection.
pub mod types;
/// Vector operations and conversions.
pub mod vectors;

// Advanced I/O system modules (R7RS-large)
/// Advanced I/O operations and utilities.
pub mod advanced_io;
/// Asynchronous I/O operations.
pub mod async_io;
/// Network I/O and socket operations.
pub mod network_io;
/// Streaming I/O and lazy sequences.
pub mod streaming_io;
/// Platform-specific I/O operations.
pub mod platform_io;
/// Secure I/O operations and sandboxing.
pub mod security_io;
/// I/O system integration and coordination.
pub mod io_integration;

// SRFI-135 Text processing modules
/// Text processing and manipulation (SRFI-135).
pub mod text;
/// Regular expression support for text processing.
pub mod text_regex;
/// Advanced text algorithms and utilities.
pub mod text_algorithms;
/// SRFI-135 specific text implementations.
pub mod text_srfi135;
/// Performance-optimized text operations.
pub mod text_performance;

/// Text processing test suite.
#[cfg(test)]
pub mod text_tests;

// Individual structure modules
/// Standard library core implementation and bindings.
pub mod standard_library;

pub use standard_library::*;

use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use std::sync::Arc;

/// Built-in procedure types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinProcedure {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    
    // Comparison
    Equal,
    LessThan,
    GreaterThan,
    
    // List operations
    Cons,
    Car,
    Cdr,
    List,
    
    // I/O
    Display,
    Newline,
    
    // System functions
    Features,
    CurrentSecond,
    CurrentJiffy,
    JiffiesPerSecond,
    CommandLine,
    GetEnvironmentVariable,
    GetEnvironmentVariables,
    Exit,
    EmergencyExit,
}

/// Binds core Scheme procedures that don't fit in specific categories.
fn bind_core_procedures(env: &Arc<ThreadSafeEnvironment>) {
    // eq? - identity equality
    env.define("eq?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "eq?".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_eq),
        effects: vec![Effect::Pure],
    })));
    
    // eqv? - operational equivalence
    env.define("eqv?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "eqv?".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_eqv),
        effects: vec![Effect::Pure],
    })));
    
    // equal? - structural equality
    env.define("equal?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "equal?".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_equal),
        effects: vec![Effect::Pure],
    })));
    
    // not - logical negation
    env.define("not".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "not".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_not),
        effects: vec![Effect::Pure],
    })));
    
    // boolean? - boolean predicate
    env.define("boolean?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "boolean?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_boolean_p),
        effects: vec![Effect::Pure],
    })));
    
    // boolean=? - boolean equality
    env.define("boolean=?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "boolean=?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_boolean_equal),
        effects: vec![Effect::Pure],
    })));
    
    // symbol? - symbol predicate
    env.define("symbol?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "symbol?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_symbol_p),
        effects: vec![Effect::Pure],
    })));
    
    // symbol->string - convert symbol to string
    env.define("symbol->string".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "symbol->string".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_symbol_to_string),
        effects: vec![Effect::Pure],
    })));
    
    // string->symbol - convert string to symbol
    env.define("string->symbol".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "string->symbol".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_string_to_symbol),
        effects: vec![Effect::Pure],
    })));
    
    // procedure? - procedure predicate
    env.define("procedure?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "procedure?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_procedure_p),
        effects: vec![Effect::Pure],
    })));
}

// ============= CORE PROCEDURE IMPLEMENTATIONS =============

use crate::diagnostics::{Error as DiagnosticError, Result};

/// eq? procedure - identity equality
fn primitive_eq(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("eq? expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // For now, use PartialEq implementation
    // In a full implementation, this would check reference equality for mutable objects
    Ok(Value::boolean(args[0] == args[1]))
}

/// eqv? procedure - operational equivalence
fn primitive_eqv(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("eqv? expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // For now, use PartialEq implementation
    // In a full implementation, this would be more nuanced than eq? but less than equal?
    Ok(Value::boolean(args[0] == args[1]))
}

/// equal? procedure - structural equality
fn primitive_equal(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("equal? expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // For now, use PartialEq implementation
    // In a full implementation, this would recursively check structural equality
    Ok(Value::boolean(args[0] == args[1]))
}

/// not procedure - logical negation
fn primitive_not(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("not expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::boolean(args[0].is_falsy()))
}

/// boolean? predicate
fn primitive_boolean_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("boolean? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let is_boolean = matches!(args[0], Value::Literal(crate::ast::Literal::Boolean(_)));
    Ok(Value::boolean(is_boolean))
}

/// boolean=? procedure - R7RS boolean equality
fn primitive_boolean_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "boolean=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    // All arguments must be boolean
    for (i, arg) in args.iter().enumerate() {
        if !matches!(arg, Value::Literal(crate::ast::Literal::Boolean(_))) {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("boolean=? argument {} is not a boolean", i + 1),
                None,
            )));
        }
    }
    
    // Check if all booleans are equal
    if let Value::Literal(crate::ast::Literal::Boolean(first)) = &args[0] {
        for arg in &args[1..] {
            if let Value::Literal(crate::ast::Literal::Boolean(val)) = arg {
                if first != val {
                    return Ok(Value::boolean(false));
                }
            }
        }
        Ok(Value::boolean(true))
    } else {
        // This shouldn't happen given the check above, but be safe
        Ok(Value::boolean(false))
    }
}

/// symbol? predicate
fn primitive_symbol_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("symbol? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::boolean(args[0].is_symbol()))
}

/// procedure? predicate
fn primitive_procedure_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("procedure? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::boolean(args[0].is_procedure()))
}

/// symbol->string procedure
fn primitive_symbol_to_string(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("symbol->string expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    match &args[0] {
        Value::Symbol(symbol_id) => {
            // Get the symbol name from the global symbol table
            use crate::utils::symbol::symbol_name;
            if let Some(name) = symbol_name(*symbol_id) {
                Ok(Value::string(name))
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "Invalid symbol ID".to_string(),
                    None,
                )))
            }
        },
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "symbol->string requires a symbol argument".to_string(),
            None,
        ))),
    }
}

/// string->symbol procedure
fn primitive_string_to_symbol(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string->symbol expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    match &args[0] {
        Value::Literal(crate::ast::Literal::String(s)) => {
            // Intern the string as a symbol
            use crate::utils::symbol::intern_symbol;
            let symbol_id = intern_symbol(s.clone());
            Ok(Value::symbol(symbol_id))
        },
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "string->symbol requires a string argument".to_string(),
            None,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::symbol::intern_symbol;

    #[test]
    fn test_boolean_predicate() {
        // Test boolean? with boolean values
        let args = vec![Value::boolean(true)];
        let result = primitive_boolean_p(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::boolean(false)];
        let result = primitive_boolean_p(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        // Test boolean? with non-boolean values
        let args = vec![Value::integer(42)];
        let result = primitive_boolean_p(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        let args = vec![Value::string("hello")];
        let result = primitive_boolean_p(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_boolean_equal() {
        // Test boolean=? with equal booleans
        let args = vec![Value::boolean(true), Value::boolean(true)];
        let result = primitive_boolean_equal(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::boolean(false), Value::boolean(false)];
        let result = primitive_boolean_equal(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        // Test boolean=? with unequal booleans
        let args = vec![Value::boolean(true), Value::boolean(false)];
        let result = primitive_boolean_equal(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test boolean=? with multiple equal booleans
        let args = vec![Value::boolean(true), Value::boolean(true), Value::boolean(true)];
        let result = primitive_boolean_equal(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        // Test boolean=? with multiple mixed booleans
        let args = vec![Value::boolean(true), Value::boolean(true), Value::boolean(false)];
        let result = primitive_boolean_equal(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_boolean_equal_errors() {
        // Test boolean=? with non-boolean arguments
        let args = vec![Value::integer(42), Value::boolean(true)];
        let result = primitive_boolean_equal(&args);
        assert!(result.is_err());
        
        let args = vec![Value::boolean(true), Value::string("hello")];
        let result = primitive_boolean_equal(&args);
        assert!(result.is_err());
        
        // Test boolean=? with too few arguments
        let args = vec![Value::boolean(true)];
        let result = primitive_boolean_equal(&args);
        assert!(result.is_err());
        
        let result = primitive_boolean_equal(&[]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_symbol_predicate() {
        // Test symbol? with symbol values
        let symbol_id = intern_symbol("test");
        let args = vec![Value::symbol(symbol_id)];
        let result = primitive_symbol_p(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        // Test symbol? with non-symbol values
        let args = vec![Value::string("test")];
        let result = primitive_symbol_p(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        let args = vec![Value::integer(42)];
        let result = primitive_symbol_p(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_symbol_to_string() {
        // Test symbol->string conversion
        let symbol_id = intern_symbol("hello");
        let args = vec![Value::symbol(symbol_id)];
        let result = primitive_symbol_to_string(&args).unwrap();
        assert_eq!(result, Value::string("hello"));
        
        // Test another symbol
        let symbol_id = intern_symbol("world");
        let args = vec![Value::symbol(symbol_id)];
        let result = primitive_symbol_to_string(&args).unwrap();
        assert_eq!(result, Value::string("world"));
    }
    
    #[test]
    fn test_string_to_symbol() {
        // Test string->symbol conversion
        let args = vec![Value::string("hello")];
        let result = primitive_string_to_symbol(&args).unwrap();
        
        // Verify it's a symbol by converting back
        if let Value::Symbol(_) = result {
            let back_to_string = primitive_symbol_to_string(&[result]).unwrap();
            assert_eq!(back_to_string, Value::string("hello"));
        } else {
            panic!("Expected symbol result");
        }
    }
    
    #[test]
    fn test_symbol_roundtrip() {
        // Test that string->symbol and symbol->string are inverses
        let original = "test-symbol";
        let string_val = Value::string(original);
        
        // Convert to symbol
        let symbol_val = primitive_string_to_symbol(&[string_val]).unwrap();
        
        // Convert back to string
        let result_val = primitive_symbol_to_string(&[symbol_val]).unwrap();
        
        assert_eq!(result_val, Value::string(original));
    }
    
    #[test]
    fn test_symbol_string_errors() {
        // Test symbol->string with non-symbol
        let args = vec![Value::string("not-a-symbol")];
        let result = primitive_symbol_to_string(&args);
        assert!(result.is_err());
        
        // Test string->symbol with non-string
        let args = vec![Value::integer(42)];
        let result = primitive_string_to_symbol(&args);
        assert!(result.is_err());
        
        // Test wrong number of arguments
        let result = primitive_symbol_to_string(&[]);
        assert!(result.is_err());
        
        let result = primitive_string_to_symbol(&[]);
        assert!(result.is_err());
        
        let args = vec![Value::string("a"), Value::string("b")];
        let result = primitive_string_to_symbol(&args);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_core_predicates() {
        // Test other core predicates from mod.rs
        
        // Test not
        let args = vec![Value::boolean(false)];
        let result = primitive_not(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::boolean(true)];
        let result = primitive_not(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        let args = vec![Value::integer(0)]; // Everything except #f is truthy
        let result = primitive_not(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test procedure?
        let proc = Arc::new(PrimitiveProcedure {
            name: "test".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(|_| Ok(Value::Unspecified)),
            effects: vec![Effect::Pure],
        });
        let args = vec![Value::Primitive(proc)];
        let result = primitive_procedure_p(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::integer(42)];
        let result = primitive_procedure_p(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
}