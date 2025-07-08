//! SRFI 137: Minimal Unique Types
//!
//! This SRFI provides a minimal mechanism for creating disjoint types at runtime.
//! The key procedure `make-type` takes any Scheme object and returns 5 procedures
//! that collectively manage a completely new disjoint type.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;
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
        make_builtin_procedure("type-accessor", Some(0), {
            let payload = type_payload.clone();
            move |args| {
                check_arity(args, 0)?;
                Ok(payload.clone())
            }
        }),
        // 2. Constructor
        make_builtin_procedure("constructor", Some(1), {
            let chain = chain.clone();
            move |args| {
                check_arity(args, 1)?;
                Ok(Value::UniqueTypeInstance(UniqueTypeInstance {
                    type_id,
                    payload: Box::new(args[0].clone()),
                    subtype_chain: chain.clone(),
                }))
            }
        }),
        // 3. Predicate
        make_builtin_procedure("predicate", Some(1), move |args| {
            check_arity(args, 1)?;
            Ok(Value::Boolean(is_instance_of_type(&args[0], type_id)))
        }),
        // 4. Instance accessor
        make_builtin_procedure("instance-accessor", Some(1), move |args| {
            check_arity(args, 1)?;
            get_instance_payload(&args[0], type_id)
        }),
        // 5. Subtype maker
        make_builtin_procedure("subtype-maker", Some(1), move |args| {
            check_arity(args, 1)?;
            // Create a simple subtype with the current type as parent
            let new_type_id = generate_type_id();
            let new_chain = vec![new_type_id, type_id];
            create_type_procedures(new_type_id, args[0].clone(), Some(new_chain))
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
            make_builtin_procedure("make-type", Some(1), make_type_proc),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 137 exports all functions for the "types" part
        Ok(self.exports())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srfi::SrfiModule;
    use crate::value::Procedure;

    #[test]
    fn test_make_type_basic() {
        let srfi = Srfi137;
        let exports = srfi.exports();

        // Test make-type function exists
        let make_type = exports.get("make-type").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = make_type {
            let type_payload = Value::Symbol("test-type".to_string());
            let result = func(&[type_payload]).unwrap();

            // Should return a vector of 5 procedures
            if let Value::Vector(procedures) = result {
                assert_eq!(procedures.len(), 5);
                for proc in &procedures {
                    assert!(matches!(proc, Value::Procedure(_)));
                }
            } else {
                panic!("Expected vector of procedures");
            }
        }
    }

    #[test]
    fn test_type_procedures() {
        let srfi = Srfi137;
        let exports = srfi.exports();

        let make_type = exports.get("make-type").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = make_type {
            let type_payload = Value::Symbol("point".to_string());
            let result = func(&[type_payload.clone()]).unwrap();

            if let Value::Vector(procedures) = result {
                // Test type accessor (procedure 0)
                if let Value::Procedure(Procedure::Builtin { func: type_accessor, .. }) = &procedures[0] {
                    let payload = type_accessor(&[]).unwrap();
                    assert_eq!(payload, type_payload);
                }

                // Test constructor and predicate (procedures 1 and 2)
                if let (
                    Value::Procedure(Procedure::Builtin { func: constructor, .. }),
                    Value::Procedure(Procedure::Builtin { func: predicate, .. })
                ) = (&procedures[1], &procedures[2]) {
                    let instance_payload = Value::Vector(vec![
                        Value::Number(crate::lexer::SchemeNumber::Integer(10)),
                        Value::Number(crate::lexer::SchemeNumber::Integer(20)),
                    ]);
                    
                    let instance = constructor(&[instance_payload.clone()]).unwrap();
                    
                    // Test predicate
                    let is_instance = predicate(&[instance.clone()]).unwrap();
                    assert_eq!(is_instance, Value::Boolean(true));
                    
                    let not_instance = predicate(&[Value::Number(crate::lexer::SchemeNumber::Integer(42))]).unwrap();
                    assert_eq!(not_instance, Value::Boolean(false));

                    // Test instance accessor (procedure 3)
                    if let Value::Procedure(Procedure::Builtin { func: instance_accessor, .. }) = &procedures[3] {
                        let retrieved_payload = instance_accessor(&[instance]).unwrap();
                        assert_eq!(retrieved_payload, instance_payload);
                    }
                }
            }
        }
    }

    #[test]
    fn test_type_disjointness() {
        let srfi = Srfi137;
        let exports = srfi.exports();

        let make_type = exports.get("make-type").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = make_type {
            // Create two different types
            let type1_result = func(&[Value::Symbol("type1".to_string())]).unwrap();
            let type2_result = func(&[Value::Symbol("type2".to_string())]).unwrap();

            if let (Value::Vector(procs1), Value::Vector(procs2)) = (type1_result, type2_result) {
                // Get constructors and predicates
                if let (
                    Value::Procedure(Procedure::Builtin { func: constructor1, .. }),
                    Value::Procedure(Procedure::Builtin { func: predicate1, .. }),
                    Value::Procedure(Procedure::Builtin { func: constructor2, .. }),
                    Value::Procedure(Procedure::Builtin { func: predicate2, .. })
                ) = (&procs1[1], &procs1[2], &procs2[1], &procs2[2]) {
                    // Create instances
                    let instance1 = constructor1(&[Value::Number(crate::lexer::SchemeNumber::Integer(1))]).unwrap();
                    let instance2 = constructor2(&[Value::Number(crate::lexer::SchemeNumber::Integer(2))]).unwrap();

                    // Test disjointness - each instance should only satisfy its own predicate
                    assert_eq!(predicate1(&[instance1.clone()]).unwrap(), Value::Boolean(true));
                    assert_eq!(predicate1(&[instance2.clone()]).unwrap(), Value::Boolean(false));
                    assert_eq!(predicate2(&[instance1]).unwrap(), Value::Boolean(false));
                    assert_eq!(predicate2(&[instance2]).unwrap(), Value::Boolean(true));
                }
            }
        }
    }

    #[test]
    fn test_subtype_creation() {
        let srfi = Srfi137;
        let exports = srfi.exports();

        let make_type = exports.get("make-type").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = make_type {
            // Create base type
            let base_result = func(&[Value::Symbol("shape".to_string())]).unwrap();

            if let Value::Vector(base_procs) = base_result {
                // Get subtype maker (procedure 4)
                if let Value::Procedure(Procedure::Builtin { func: subtype_maker, .. }) = &base_procs[4] {
                    // Create subtype
                    let subtype_result = subtype_maker(&[Value::Symbol("circle".to_string())]).unwrap();

                    if let Value::Vector(subtype_procs) = subtype_result {
                        assert_eq!(subtype_procs.len(), 5);
                        
                        // Test that subtype instances satisfy both predicates
                        if let (
                            Value::Procedure(Procedure::Builtin { func: base_constructor, .. }),
                            Value::Procedure(Procedure::Builtin { func: base_predicate, .. }),
                            Value::Procedure(Procedure::Builtin { func: subtype_constructor, .. }),
                            Value::Procedure(Procedure::Builtin { func: subtype_predicate, .. })
                        ) = (&base_procs[1], &base_procs[2], &subtype_procs[1], &subtype_procs[2]) {
                            let subtype_instance = subtype_constructor(&[Value::Number(crate::lexer::SchemeNumber::Integer(5))]).unwrap();
                            let base_instance = base_constructor(&[Value::Number(crate::lexer::SchemeNumber::Integer(10))]).unwrap();

                            // Subtype instance should satisfy both predicates
                            assert_eq!(subtype_predicate(&[subtype_instance.clone()]).unwrap(), Value::Boolean(true));
                            assert_eq!(base_predicate(&[subtype_instance]).unwrap(), Value::Boolean(true));

                            // Base instance should only satisfy base predicate
                            assert_eq!(base_predicate(&[base_instance.clone()]).unwrap(), Value::Boolean(true));
                            assert_eq!(subtype_predicate(&[base_instance]).unwrap(), Value::Boolean(false));
                        }
                    }
                }
            }
        }
    }
}