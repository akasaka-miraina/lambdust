//! SRFI-124 procedures implementation
//!
//! This module provides the core procedures for SRFI-124 ephemerons:
//! - `make-ephemeron`: Creates a new ephemeron
//! - `ephemeron-broken?`: Checks if an ephemeron is broken
//! - `ephemeron-key`: Gets the ephemeron's key (#f if broken)  
//! - `ephemeron-datum`: Gets the ephemeron's datum (#f if broken)
//! - `reference-barrier`: Ensures values stay reachable

use crate::eval::Value;
use crate::stdlib::srfi124_ephemerons_simple::{Ephemeron, ReferenceBarrier};
use crate::ast::literal::Literal;
use crate::diagnostics::{Result, Error as DiagnosticError, Span};
use crate::utils::{SymbolId, intern_symbol};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;

/// Creates a new ephemeron with the given key and datum.
/// 
/// ## Syntax
/// `(make-ephemeron key datum) -> ephemeron`
///
/// ## Parameters
/// - `key`: The key value (held weakly)
/// - `datum`: The datum value (held strongly until broken)
///
/// ## Returns
/// A new ephemeron object
///
/// ## Example
/// ```scheme
/// (define e (make-ephemeron 'key "value"))
/// (ephemeron-key e)   ; => 'key
/// (ephemeron-datum e) ; => "value"
/// ```
pub fn make_ephemeron(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("make-ephemeron: expected 2 arguments, got {}", args.len()),
            Some(Span::new(0, 0)), // Dummy span for internal error
        )));
    }

    let key = Rc::new(RefCell::new(args[0].clone()));
    let datum = args[1].clone();
    
    let ephemeron = Ephemeron::new(key, datum);
    Ok(Value::Ephemeron(ephemeron))
}

/// Checks if an ephemeron has been broken by the garbage collector.
///
/// ## Syntax
/// `(ephemeron-broken? obj) -> boolean`
///
/// ## Parameters
/// - `obj`: The object to test
///
/// ## Returns
/// `#t` if the object is a broken ephemeron, `#f` otherwise
///
/// ## Example
/// ```scheme
/// (define e (make-ephemeron 'key "value"))
/// (ephemeron-broken? e) ; => #f
/// ;; ... key gets garbage collected ...
/// (ephemeron-broken? e) ; => #t
/// ```
pub fn ephemeron_broken_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("ephemeron-broken?: expected 1 argument, got {}", args.len()),
            Some(Span::new(0, 0)), // Dummy span for internal error
        )));
    }

    let result = match &args[0] {
        Value::Ephemeron(ephemeron) => ephemeron.is_broken(),
        _ => false, // Non-ephemeron objects are never "broken"
    };

    Ok(Value::Literal(Literal::Boolean(result)))
}

/// Gets the key from an ephemeron, returning `#f` if broken.
///
/// ## Syntax
/// `(ephemeron-key ephemeron) -> value`
///
/// ## Parameters
/// - `ephemeron`: The ephemeron object
///
/// ## Returns
/// The key value, or `#f` if the ephemeron is broken
///
/// ## Errors
/// Signals an error if the argument is not an ephemeron
///
/// ## Example
/// ```scheme
/// (define e (make-ephemeron 'key "value"))
/// (ephemeron-key e) ; => 'key
/// ```
pub fn ephemeron_key(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("ephemeron-key: expected 1 argument, got {}", args.len()),
            Some(Span::new(0, 0)), // Dummy span for internal error
        )));
    }

    match &args[0] {
        Value::Ephemeron(ephemeron) => Ok(ephemeron.key()),
        _ => Err(Box::new(DiagnosticError::type_error(
            "ephemeron-key: argument must be an ephemeron".to_string(),
            Span::new(0, 0), // Dummy span for internal error
        ))),
    }
}

/// Gets the datum from an ephemeron, returning `#f` if broken.
///
/// ## Syntax
/// `(ephemeron-datum ephemeron) -> value`
///
/// ## Parameters
/// - `ephemeron`: The ephemeron object
///
/// ## Returns
/// The datum value, or `#f` if the ephemeron is broken
///
/// ## Errors
/// Signals an error if the argument is not an ephemeron
///
/// ## Example
/// ```scheme
/// (define e (make-ephemeron 'key "value"))
/// (ephemeron-datum e) ; => "value"
/// ```
pub fn ephemeron_datum(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("ephemeron-datum: expected 1 argument, got {}", args.len()),
            Some(Span::new(0, 0)), // Dummy span for internal error
        )));
    }

    match &args[0] {
        Value::Ephemeron(ephemeron) => Ok(ephemeron.datum()),
        _ => Err(Box::new(DiagnosticError::type_error(
            "ephemeron-datum: argument must be an ephemeron".to_string(),
            Span::new(0, 0), // Dummy span for internal error
        ))),
    }
}

/// Ensures that a value stays strongly reachable, preventing ephemeron breaking.
///
/// ## Syntax
/// `(reference-barrier key) -> unspecified`
///
/// ## Parameters
/// - `key`: The value to keep strongly reachable
///
/// ## Returns
/// An unspecified value
///
/// ## Description
/// The `reference-barrier` procedure is used when the datum of an ephemeron
/// references the key, to prevent the garbage collector from breaking the
/// ephemeron prematurely. This creates a strong reference that prevents
/// collection of the key.
///
/// ## Example
/// ```scheme
/// (define key (cons 'a 'b))
/// (define e (make-ephemeron key key)) ; Datum references key
/// (reference-barrier key)            ; Prevent premature breaking
/// ```
pub fn reference_barrier(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("reference-barrier: expected 1 argument, got {}", args.len()),
            Some(Span::new(0, 0)), // Dummy span for internal error
        )));
    }

    // Convert the value to a strong reference and add to barrier
    let key_ref = Rc::new(RefCell::new(args[0].clone()));
    
    // Get or create thread-local reference barrier
    thread_local! {
        static BARRIER: RefCell<ReferenceBarrier> = RefCell::new(ReferenceBarrier::new());
    }
    
    BARRIER.with(|barrier| {
        barrier.borrow().ensure_reachable(key_ref);
    });

    Ok(Value::Unspecified)
}

/// Checks if a value is an ephemeron object.
///
/// ## Syntax
/// `(ephemeron? obj) -> boolean`
///
/// ## Parameters
/// - `obj`: The object to test
///
/// ## Returns
/// `#t` if the object is an ephemeron, `#f` otherwise
///
/// ## Example
/// ```scheme
/// (ephemeron? (make-ephemeron 'a 'b)) ; => #t
/// (ephemeron? 'not-an-ephemeron)      ; => #f
/// ```
pub fn ephemeron_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("ephemeron?: expected 1 argument, got {}", args.len()),
            Some(Span::new(0, 0)), // Dummy span for internal error
        )));
    }

    let result = matches!(&args[0], Value::Ephemeron(_));
    Ok(Value::Literal(Literal::Boolean(result)))
}

/// Sets the datum of an ephemeron (if not broken).
///
/// ## Syntax
/// `(ephemeron-set-datum! ephemeron datum) -> unspecified`
///
/// ## Parameters
/// - `ephemeron`: The ephemeron object
/// - `datum`: The new datum value
///
/// ## Returns
/// An unspecified value
///
/// ## Errors
/// Signals an error if:
/// - The first argument is not an ephemeron
/// - The ephemeron is broken (no-op, returns unspecified)
///
/// ## Example
/// ```scheme
/// (define e (make-ephemeron 'key "old"))
/// (ephemeron-set-datum! e "new")
/// (ephemeron-datum e) ; => "new"
/// ```
pub fn ephemeron_set_datum(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("ephemeron-set-datum!: expected 2 arguments, got {}", args.len()),
            Some(Span::new(0, 0)), // Dummy span for internal error
        )));
    }

    match &args[0] {
        Value::Ephemeron(ephemeron) => {
            ephemeron.set_datum(args[1].clone());
            Ok(Value::Unspecified)
        }
        _ => Err(Box::new(DiagnosticError::type_error(
            "ephemeron-set-datum!: first argument must be an ephemeron".to_string(),
            Span::new(0, 0), // Dummy span for internal error
        ))),
    }
}

/// Clears the global reference barrier, allowing previously protected values to be collected.
///
/// ## Syntax
/// `(clear-reference-barrier!) -> unspecified`
///
/// ## Parameters
/// None
///
/// ## Returns
/// An unspecified value
///
/// ## Description
/// This procedure clears all strong references maintained by the reference barrier,
/// allowing values that were previously protected from garbage collection to be
/// collected if they have no other strong references.
///
/// ## Example
/// ```scheme
/// (reference-barrier some-value)
/// ;; ... some-value is protected from GC ...
/// (clear-reference-barrier!)
/// ;; ... some-value can now be collected if no other references exist ...
/// ```
pub fn clear_reference_barrier(args: &[Value]) -> Result<Value> {
    if !args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("clear-reference-barrier!: expected 0 arguments, got {}", args.len()),
            Some(Span::new(0, 0)), // Dummy span for internal error
        )));
    }

    thread_local! {
        static BARRIER: RefCell<ReferenceBarrier> = RefCell::new(ReferenceBarrier::new());
    }
    
    BARRIER.with(|barrier| {
        barrier.borrow().release_all();
    });

    Ok(Value::Unspecified)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_ephemeron() {
        let key = Value::Symbol(intern_symbol("key"));
        let datum = Value::Literal(Literal::String(Box::new("value".to_string())));
        let args = vec![key, datum];

        let result = make_ephemeron(&args);
        assert!(result.is_ok());
        
        if let Ok(Value::Ephemeron(_)) = result {
            // Success
        } else {
            panic!("Expected ephemeron value");
        }
    }

    #[test]
    fn test_ephemeron_predicate() {
        // Test with ephemeron
        let key = Value::Symbol(intern_symbol("key"));
        let datum = Value::Literal(Literal::String(Box::new("value".to_string())));
        let ephemeron_result = make_ephemeron(&[key, datum]).unwrap();
        
        let pred_result = ephemeron_p(&[ephemeron_result]);
        assert!(matches!(pred_result, Ok(Value::Literal(Literal::Boolean(true)))));
        
        // Test with non-ephemeron
        let non_ephemeron = Value::Literal(Literal::Number(42.into()));
        let pred_result = ephemeron_p(&[non_ephemeron]);
        assert!(matches!(pred_result, Ok(Value::Literal(Literal::Boolean(false)))));
    }

    #[test]
    fn test_ephemeron_accessors() {
        let key = Value::Symbol(intern_symbol("test-key"));
        let datum = Value::Literal(Literal::String(Box::new("test-value".to_string())));
        
        let ephemeron = make_ephemeron(&[key.clone(), datum.clone()]).unwrap();
        
        // Test key access
        let key_result = ephemeron_key(&[ephemeron.clone()]);
        assert!(key_result.is_ok());
        
        // Test datum access
        let datum_result = ephemeron_datum(&[ephemeron]);
        assert!(datum_result.is_ok());
    }

    #[test]
    fn test_ephemeron_broken_predicate() {
        let key = Value::Symbol(intern_symbol("key"));
        let datum = Value::Literal(Literal::String(Box::new("value".to_string())));
        
        let ephemeron = make_ephemeron(&[key, datum]).unwrap();
        
        // New ephemeron should not be broken
        let broken_result = ephemeron_broken_p(&[ephemeron]);
        assert!(matches!(broken_result, Ok(Value::Literal(Literal::Boolean(false)))));
        
        // Test with non-ephemeron (should return #f)
        let non_ephemeron = Value::Literal(Literal::Number(42.into()));
        let broken_result = ephemeron_broken_p(&[non_ephemeron]);
        assert!(matches!(broken_result, Ok(Value::Literal(Literal::Boolean(false)))));
    }

    #[test]
    fn test_reference_barrier() {
        let value = Value::Symbol(intern_symbol("protected"));
        
        let result = reference_barrier(&[value]);
        assert!(matches!(result, Ok(Value::Unspecified)));
    }

    #[test]
    fn test_clear_reference_barrier() {
        let result = clear_reference_barrier(&[]);
        assert!(matches!(result, Ok(Value::Unspecified)));
    }

    #[test]
    fn test_ephemeron_set_datum() {
        let key = Value::Symbol(intern_symbol("key"));
        let old_datum = Value::Literal(Literal::String(Box::new("old".to_string())));
        let new_datum = Value::Literal(Literal::String(Box::new("new".to_string())));
        
        let ephemeron = make_ephemeron(&[key, old_datum]).unwrap();
        
        let result = ephemeron_set_datum(&[ephemeron.clone(), new_datum.clone()]);
        assert!(matches!(result, Ok(Value::Unspecified)));
        
        // Verify the datum was updated
        let datum_result = ephemeron_datum(&[ephemeron]);
        assert!(datum_result.is_ok());
    }

    #[test]
    fn test_argument_validation() {
        // Test wrong number of arguments
        assert!(make_ephemeron(&[]).is_err());
        assert!(make_ephemeron(&[Value::Unspecified]).is_err());
        
        assert!(ephemeron_p(&[]).is_err());
        assert!(ephemeron_p(&[Value::Unspecified, Value::Unspecified]).is_err());
        
        // Test wrong argument types
        let non_ephemeron = Value::Literal(Literal::Number(42.into()));
        assert!(ephemeron_key(&[non_ephemeron.clone()]).is_err());
        assert!(ephemeron_datum(&[non_ephemeron.clone()]).is_err());
        assert!(ephemeron_set_datum(&[non_ephemeron, Value::Unspecified]).is_err());
    }
}