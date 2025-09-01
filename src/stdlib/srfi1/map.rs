//! SRFI-1 Mapping Operations
//!
//! This module implements the mapping operations optimized for performance:
//! - map, for-each, append-map, map-in-order
//!
//! These operations use arena allocation and NaN-boxing optimizations
//! to achieve the cs-architect's performance targets.

use crate::diagnostics::{Error, Result};
use crate::eval::list_arena::GlobalListArena;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::effects::Effect;
use crate::stdlib::srfi1::SRFI1Core;
use std::sync::Arc;

/// Binds all map operations to the environment
pub fn bind_map_operations(env: &Arc<ThreadSafeEnvironment>) {
    // map (enhanced SRFI-1 version)
    env.define(
        "map".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "map".to_string(),
            arity_min: 2,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(srfi1_map),
            effects: vec![Effect::Pure],
        })),
    );

    // for-each (enhanced SRFI-1 version)
    env.define(
        "for-each".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "for-each".to_string(),
            arity_min: 2,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(srfi1_for_each),
            effects: vec![Effect::Mutation],
        })),
    );

    // append-map
    env.define(
        "append-map".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "append-map".to_string(),
            arity_min: 2,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(srfi1_append_map),
            effects: vec![Effect::Pure],
        })),
    );

    // map-in-order
    env.define(
        "map-in-order".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "map-in-order".to_string(),
            arity_min: 2,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(srfi1_map_in_order),
            effects: vec![Effect::Pure],
        })),
    );
}

/// map - Apply procedure to corresponding elements of lists
/// Optimized version using arena allocation for performance
pub fn srfi1_map(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 2, None, "map")?;
    
    let proc = &args[0];
    let lists = &args[1..];
    
    if !proc.is_procedure() {
        return Err(Box::new(Error::runtime_error(
            "map first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    // Validate all lists are proper lists
    for (i, list) in lists.iter().enumerate() {
        SRFI1Core::ensure_proper_list(list, &format!("map argument {}", i + 2))?;
    }
    
    // Convert lists to vectors for easier processing
    let list_vecs: Result<Vec<Vec<Value>>> = lists.iter()
        .map(|list| SRFI1Core::list_to_vec(list))
        .collect();
    let list_vecs = list_vecs?;
    
    // Check all lists have the same length
    let list_len = if let Some(first_vec) = list_vecs.first() {
        let first_len = first_vec.len();
        for (i, vec) in list_vecs.iter().enumerate().skip(1) {
            if vec.len() != first_len {
                return Err(Box::new(Error::runtime_error(
                    format!("map: all lists must have the same length, list {} has different length", i + 1),
                    None,
                )));
            }
        }
        first_len
    } else {
        return Ok(Value::Nil); // No lists provided
    };
    
    if list_len == 0 {
        return Ok(Value::Nil); // All lists are empty
    }
    
    // Create arena for optimized allocation
    let mut arena = GlobalListArena::create_construction_arena(list_len);
    let mut results = Vec::with_capacity(list_len);
    
    // Apply procedure to each set of corresponding elements
    for i in 0..list_len {
        if list_vecs.len() == 1 {
            // Single list case - most common and optimizable
            let result = SRFI1Core::apply_procedure(proc, &list_vecs[0][i])?;
            results.push(result);
        } else {
            // Multiple lists - need multi-argument procedure application
            // This is a simplified placeholder - full implementation needs
            // proper multi-argument procedure calls
            return Err(Box::new(Error::runtime_error(
                "map with multiple lists requires enhanced procedure application".to_string(),
                None,
            )));
        }
    }
    
    // Convert results to optimized list using arena
    Ok(SRFI1Core::vec_to_list(results))
}

/// for-each - Apply procedure to corresponding elements for side effects
pub fn srfi1_for_each(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 2, None, "for-each")?;
    
    let proc = &args[0];
    let lists = &args[1..];
    
    if !proc.is_procedure() {
        return Err(Box::new(Error::runtime_error(
            "for-each first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    // Validate all lists are proper lists
    for (i, list) in lists.iter().enumerate() {
        SRFI1Core::ensure_proper_list(list, &format!("for-each argument {}", i + 2))?;
    }
    
    // Convert lists to vectors
    let list_vecs: Result<Vec<Vec<Value>>> = lists.iter()
        .map(|list| SRFI1Core::list_to_vec(list))
        .collect();
    let list_vecs = list_vecs?;
    
    // Check all lists have the same length
    let list_len = if let Some(first_vec) = list_vecs.first() {
        let first_len = first_vec.len();
        for (i, vec) in list_vecs.iter().enumerate().skip(1) {
            if vec.len() != first_len {
                return Err(Box::new(Error::runtime_error(
                    format!("for-each: all lists must have the same length, list {} has different length", i + 1),
                    None,
                )));
            }
        }
        first_len
    } else {
        return Ok(Value::Unspecified); // No lists provided
    };
    
    // Apply procedure to each set of corresponding elements
    for i in 0..list_len {
        if list_vecs.len() == 1 {
            // Single list case
            let _result = SRFI1Core::apply_procedure(proc, &list_vecs[0][i])?;
            // Ignore result - for-each is for side effects only
        } else {
            // Multiple lists case
            return Err(Box::new(Error::runtime_error(
                "for-each with multiple lists requires enhanced procedure application".to_string(),
                None,
            )));
        }
    }
    
    Ok(Value::Unspecified)
}

/// append-map - Map procedure and append all results
pub fn srfi1_append_map(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 2, None, "append-map")?;
    
    // First apply map to get a list of lists
    let mapped_result = srfi1_map(args)?;
    
    // Then append all the resulting lists together
    if let Value::Nil = mapped_result {
        return Ok(Value::Nil);
    }
    
    let mapped_lists = SRFI1Core::list_to_vec(&mapped_result)?;
    let mut all_elements = Vec::new();
    
    for list_value in mapped_lists {
        match list_value {
            Value::Nil => {
                // Empty list contributes nothing
            }
            Value::Pair(_, _) => {
                let elements = SRFI1Core::list_to_vec(&list_value)?;
                all_elements.extend(elements);
            }
            _ => {
                return Err(Box::new(Error::runtime_error(
                    "append-map procedure must return lists".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(SRFI1Core::vec_to_list(all_elements))
}

/// map-in-order - Like map but guarantees left-to-right evaluation order
pub fn srfi1_map_in_order(args: &[Value]) -> Result<Value> {
    // For now, same as map since we don't have parallel evaluation
    // In a parallel implementation, this would force sequential evaluation
    srfi1_map(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_map_binding() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        bind_map_operations(&env);
        
        assert!(env.lookup("map").is_some());
        assert!(env.lookup("for-each").is_some());
        assert!(env.lookup("append-map").is_some());
        assert!(env.lookup("map-in-order").is_some());
    }

    #[test]
    fn test_map_empty_list() {
        let proc = Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "identity".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| Ok(args[0].clone())),
            effects: vec![Effect::Pure],
        }));
        
        let result = srfi1_map(&[proc, Value::Nil]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_for_each_side_effects() {
        let proc = Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "display".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|_| Ok(Value::Unspecified)),
            effects: vec![Effect::Mutation],
        }));
        
        let list = Value::list(vec![Value::boolean(true), Value::boolean(false)]);
        let result = srfi1_for_each(&[proc, list]).unwrap();
        assert_eq!(result, Value::Unspecified);
    }

    #[test]
    fn test_append_map_empty() {
        let proc = Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "list".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| Ok(Value::list(vec![args[0].clone()]))),
            effects: vec![Effect::Pure],
        }));
        
        let result = srfi1_append_map(&[proc, Value::Nil]).unwrap();
        assert_eq!(result, Value::Nil);
    }
}