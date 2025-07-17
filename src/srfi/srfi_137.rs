//! SRFI 137: Minimal Unique Types
//!
//! This SRFI provides a minimal mechanism for creating disjoint types at runtime.
//! The key procedure `make-type` takes any Scheme object and returns 5 procedures
//! that collectively manage a completely new disjoint type.

use crate::builtins::utils::check_arity;
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Unique type instance for SRFI 137
#[derive(Debug, Clone, PartialEq)]
pub struct UniqueTypeInstance {
    /// Unique type identifier
    pub type_id: usize,
    /// Instance payload (any Scheme value) - boxed to avoid infinite recursion
    pub payload: Box<Value>,
    /// Subtype chain for efficient predicate checking
    pub subtype_chain: Vec<usize>,
}

/// Generate unique type ID
fn generate_type_id() -> usize {
    static NEXT_TYPE_ID: AtomicUsize = AtomicUsize::new(1);
    NEXT_TYPE_ID.fetch_add(1, Ordering::SeqCst)
}

/// Check if an object is an instance of a specific type
fn is_instance_of_type(obj: &Value, type_id: usize) -> bool {
    match obj {
        Value::UniqueTypeInstance(instance) => {
            // Check if instance type matches or is subtype
            instance.type_id == type_id || instance.subtype_chain.contains(&type_id)
        }
        _ => false,
    }
}

/// Get instance payload if object is of correct type
fn get_instance_payload(obj: &Value, type_id: usize) -> Result<Value> {
    match obj {
        Value::UniqueTypeInstance(instance) => {
            if instance.type_id == type_id || instance.subtype_chain.contains(&type_id) {
                Ok(*instance.payload.clone())
            } else {
                Err(LambdustError::type_error(
                    "Object is not an instance of the expected type".to_string(),
                ))
            }
        }
        _ => Err(LambdustError::type_error(
            "Expected unique type instance".to_string(),
        )),
    }
}

/// Create the 5 procedures returned by make-type
fn create_type_procedures(type_id: usize, type_payload: Value, subtype_chain: Option<Vec<usize>>) -> Result<Value> {
    let default_chain = vec![type_id];
    let chain = subtype_chain.unwrap_or(default_chain);
    
    let procedures = vec![
        // 1. Type accessor
        Value::Procedure(Procedure::HostFunction {
            name: "type-accessor".to_string(),
            arity: Some(0),
            func: {
                let payload = type_payload.clone();
                Rc::new(move |args| {
                    check_arity(args, 0)?;
                    Ok(payload.clone())
                })
            },
        }),
        // 2. Constructor
        Value::Procedure(Procedure::HostFunction {
            name: "constructor".to_string(),
            arity: Some(1),
            func: {
                let chain = chain.clone();
                Rc::new(move |args| {
                    check_arity(args, 1)?;
                    Ok(Value::UniqueTypeInstance(UniqueTypeInstance {
                        type_id,
                        payload: Box::new(args[0].clone()),
                        subtype_chain: chain.clone(),
                    }))
                })
            },
        }),
        // 3. Predicate
        Value::Procedure(Procedure::HostFunction {
            name: "predicate".to_string(),
            arity: Some(1),
            func: Rc::new(move |args| {
                check_arity(args, 1)?;
                Ok(Value::Boolean(is_instance_of_type(&args[0], type_id)))
            }),
        }),
        // 4. Instance accessor
        Value::Procedure(Procedure::HostFunction {
            name: "instance-accessor".to_string(),
            arity: Some(1),
            func: Rc::new(move |args| {
                check_arity(args, 1)?;
                get_instance_payload(&args[0], type_id)
            }),
        }),
        // 5. Subtype maker
        Value::Procedure(Procedure::HostFunction {
            name: "subtype-maker".to_string(),
            arity: Some(1),
            func: Rc::new(move |args| {
                check_arity(args, 1)?;
                // Create a simple subtype with the current type as parent
                let new_type_id = generate_type_id();
                let new_chain = vec![new_type_id, type_id];
                create_type_procedures(new_type_id, args[0].clone(), Some(new_chain))
            }),
        }),
    ];

    Ok(Value::Vector(procedures))
}

/// Main make-type procedure
fn make_type_proc(args: &[Value]) -> Result<Value> {
    check_arity(args, 1)?;
    let type_payload = args[0].clone();
    let type_id = generate_type_id();

    // Return the 5 procedures
    create_type_procedures(type_id, type_payload, None)
}

/// SRFI 137 implementation
pub struct Srfi137;

impl super::SrfiModule for Srfi137 {
    fn srfi_id(&self) -> u32 {
        137
    }

    fn name(&self) -> &'static str {
        "Minimal Unique Types"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["types"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Main procedure
        exports.insert(
            "make-type".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "make-type".to_string(),
                arity: Some(1),
                func: make_type_proc,
            }),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 137 exports all functions for the "types" part
        Ok(self.exports())
    }
}
