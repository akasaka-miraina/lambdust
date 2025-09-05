//! SRFI-111: Box (Mutable Cells) Implementation
//!
//! This module provides a SRFI-111 compliant implementation of mutable cells.
//! Boxes are containers that hold a single value and allow mutation of that value.
//!
//! SRFI-111 Specification:
//! - `(box value)` - Creates a new box containing value
//! - `(box? obj)` - Returns #t if obj is a box, #f otherwise
//! - `(unbox box)` - Returns the current value stored in box
//! - `(set-box! box value)` - Changes the value stored in box to value

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use std::boxed::Box as StdBox;
use std::sync::{Arc, RwLock};

/// Thread-safe mutable box for SRFI-111 compliance
#[derive(Debug, Clone)]
pub struct Box {
    /// The contained value (thread-safe)
    value: Arc<RwLock<Value>>,
    /// Optional name for debugging
    name: Option<String>,
}

impl Box {
    /// Creates a new box containing the given value
    pub fn new(value: Value) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
            name: None,
        }
    }

    /// Creates a named box for debugging
    pub fn with_name(value: Value, name: impl Into<String>) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
            name: Some(name.into()),
        }
    }

    /// Gets the current value from the box
    pub fn get(&self) -> Result<Value> {
        self.value
            .try_read()
            .map(|guard| guard.clone())
            .map_err(|_| {
                StdBox::new(DiagnosticError::runtime_error(
                    "Failed to read from box (lock poisoned)".to_string(),
                    None,
                ))
            })
    }

    /// Sets the value in the box
    pub fn set(&self, new_value: Value) -> Result<()> {
        self.value
            .write()
            .map(|mut guard| *guard = new_value)
            .map_err(|_| {
                StdBox::new(DiagnosticError::runtime_error(
                    "Failed to write to box (lock poisoned)".to_string(),
                    None,
                ))
            })
    }

    /// Gets the name of the box (for debugging)
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl PartialEq for Box {
    fn eq(&self, other: &Self) -> bool {
        // Two boxes are equal if they point to the same location
        Arc::ptr_eq(&self.value, &other.value)
    }
}

impl Eq for Box {}

/// Creates SRFI-111 box bindings for the standard library.
pub fn create_box_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // box constructor
    env.define(
        "box".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "box".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_box),
            effects: vec![Effect::Pure],
        })),
    );

    // box? predicate
    env.define(
        "box?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "box?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_box_p),
            effects: vec![Effect::Pure],
        })),
    );

    // unbox accessor
    env.define(
        "unbox".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "unbox".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_unbox),
            effects: vec![Effect::Pure],
        })),
    );

    // set-box! mutator
    env.define(
        "set-box!".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "set-box!".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(primitive_set_box),
            effects: vec![Effect::Mutation],
        })),
    );
}

// ============= PRIMITIVE IMPLEMENTATIONS =============

/// `box` primitive - creates a new box containing the given value
fn primitive_box(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(StdBox::new(DiagnosticError::runtime_error(
            format!("box expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    let box_instance = Box::new(args[0].clone());
    Ok(Value::Box(Arc::new(box_instance)))
}

/// `box?` primitive - type predicate for boxes
fn primitive_box_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(StdBox::new(DiagnosticError::runtime_error(
            format!("box? expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    let is_box = matches!(&args[0], Value::Box(_));
    Ok(Value::boolean(is_box))
}

/// `unbox` primitive - gets the value from a box
fn primitive_unbox(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(StdBox::new(DiagnosticError::runtime_error(
            format!("unbox expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::Box(box_ref) => box_ref.get(),
        _ => Err(StdBox::new(DiagnosticError::runtime_error(
            "unbox: argument must be a box".to_string(),
            None,
        ))),
    }
}

/// `set-box!` primitive - sets the value in a box
fn primitive_set_box(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(StdBox::new(DiagnosticError::runtime_error(
            format!("set-box! expects 2 arguments, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::Box(box_ref) => {
            box_ref.set(args[1].clone())?;
            Ok(Value::Unspecified)
        }
        _ => Err(StdBox::new(DiagnosticError::runtime_error(
            "set-box!: first argument must be a box".to_string(),
            None,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_creation() {
        let value = Value::integer(42);
        let box_instance = Box::new(value.clone());
        let retrieved = box_instance.get().unwrap();
        assert_eq!(retrieved, value);
    }

    #[test]
    fn test_box_mutation() {
        let initial_value = Value::integer(42);
        let new_value = Value::string("hello");

        let box_instance = Box::new(initial_value);
        box_instance.set(new_value.clone()).unwrap();

        let retrieved = box_instance.get().unwrap();
        assert_eq!(retrieved, new_value);
    }

    #[test]
    fn test_box_equality() {
        let value = Value::integer(42);
        let box1 = Box::new(value.clone());
        let box2 = box1.clone();
        let box3 = Box::new(value);

        assert_eq!(box1, box2); // Same box
        assert_ne!(box1, box3); // Different boxes with same content
    }

    #[test]
    fn test_box_primitive() {
        let args = vec![Value::integer(42)];
        let result = primitive_box(&args).unwrap();

        assert!(matches!(result, Value::Box(_)));

        // Test unbox
        let unbox_args = vec![result];
        let unboxed = primitive_unbox(&unbox_args).unwrap();
        assert_eq!(unboxed, Value::integer(42));
    }

    #[test]
    fn test_box_predicate() {
        let box_val = primitive_box(&[Value::integer(42)]).unwrap();
        let non_box = Value::string("not a box");

        assert_eq!(primitive_box_p(&[box_val]).unwrap(), Value::boolean(true));
        assert_eq!(primitive_box_p(&[non_box]).unwrap(), Value::boolean(false));
    }

    #[test]
    fn test_set_box() {
        let box_val = primitive_box(&[Value::integer(42)]).unwrap();
        let new_value = Value::string("hello");

        // Set new value
        let result = primitive_set_box(&[box_val.clone(), new_value.clone()]).unwrap();
        assert_eq!(result, Value::Unspecified);

        // Verify value changed
        let retrieved = primitive_unbox(&[box_val]).unwrap();
        assert_eq!(retrieved, new_value);
    }

    #[test]
    fn test_error_handling() {
        // Wrong number of arguments
        assert!(primitive_box(&[]).is_err());
        assert!(primitive_box(&[Value::integer(1), Value::integer(2)]).is_err());

        // Wrong type for unbox
        assert!(primitive_unbox(&[Value::integer(42)]).is_err());

        // Wrong type for set-box!
        assert!(primitive_set_box(&[Value::integer(42), Value::string("test")]).is_err());
    }

    #[test]
    fn test_named_box() {
        let box_instance = Box::with_name(Value::integer(42), "test-box");
        assert_eq!(box_instance.name(), Some("test-box"));

        let unnamed = Box::new(Value::integer(42));
        assert_eq!(unnamed.name(), None);
    }
}
