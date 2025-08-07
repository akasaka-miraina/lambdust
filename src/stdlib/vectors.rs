//! Vector operations for the Lambdust standard library.
//!
//! This module implements R7RS-compliant vector operations including
//! vector creation, manipulation, and conversion functions.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use std::sync::Arc;

/// Creates vector operation bindings for the standard library.
pub fn create_vector_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Vector creation
    bind_vector_creation(env);
    
    // Vector predicates
    bind_vector_predicates(env);
    
    // Vector accessors and mutators
    bind_vector_accessors(env);
    
    // Vector manipulation
    bind_vector_manipulation(env);
    
    // Vector iteration
    bind_vector_iteration(env);
    
    // Vector conversion
    bind_vector_conversion(env);
}

/// Binds vector creation operations.
fn bind_vector_creation(env: &Arc<ThreadSafeEnvironment>) {
    // vector
    env.define("vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector".to_string(),
        arity_min: 0,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_vector),
        effects: vec![Effect::Pure],
    })));
    
    // make-vector
    env.define("make-vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-vector".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_make_vector),
        effects: vec![Effect::Pure],
    })));
    
    // vector-copy
    env.define("vector-copy".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector-copy".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_vector_copy),
        effects: vec![Effect::Pure],
    })));
}

/// Binds vector predicates.
fn bind_vector_predicates(env: &Arc<ThreadSafeEnvironment>) {
    // vector?
    env.define("vector?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_vector_p),
        effects: vec![Effect::Pure],
    })));
}

/// Binds vector accessors and mutators.
fn bind_vector_accessors(env: &Arc<ThreadSafeEnvironment>) {
    // vector-length
    env.define("vector-length".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector-length".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_vector_length),
        effects: vec![Effect::Pure],
    })));
    
    // vector-ref
    env.define("vector-ref".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector-ref".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_vector_ref),
        effects: vec![Effect::Pure],
    })));
    
    // vector-set!
    env.define("vector-set!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector-set!".to_string(),
        arity_min: 3,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_vector_set),
        effects: vec![Effect::State], // Mutation effect
    })));
}

/// Binds vector manipulation operations.
fn bind_vector_manipulation(env: &Arc<ThreadSafeEnvironment>) {
    // vector-fill!
    env.define("vector-fill!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector-fill!".to_string(),
        arity_min: 2,
        arity_max: Some(4),
        implementation: PrimitiveImpl::RustFn(primitive_vector_fill),
        effects: vec![Effect::State], // Mutation effect
    })));
    
    // vector-copy!
    env.define("vector-copy!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector-copy!".to_string(),
        arity_min: 3,
        arity_max: Some(5),
        implementation: PrimitiveImpl::RustFn(primitive_vector_copy_mut),
        effects: vec![Effect::State], // Mutation effect
    })));
    
    // vector-append
    env.define("vector-append".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector-append".to_string(),
        arity_min: 0,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_vector_append),
        effects: vec![Effect::Pure],
    })));
}

/// Binds vector iteration operations.
fn bind_vector_iteration(env: &Arc<ThreadSafeEnvironment>) {
    // vector-map
    env.define("vector-map".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector-map".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_vector_map),
        effects: vec![Effect::Pure],
    })));
    
    // vector-for-each
    env.define("vector-for-each".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector-for-each".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_vector_for_each),
        effects: vec![Effect::Pure], // May call user functions with effects
    })));
}

/// Binds vector conversion operations.
fn bind_vector_conversion(env: &Arc<ThreadSafeEnvironment>) {
    // vector->list
    env.define("vector->list".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector->list".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_vector_to_list),
        effects: vec![Effect::Pure],
    })));
    
    // list->vector
    env.define("list->vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list->vector".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_list_to_vector),
        effects: vec![Effect::Pure],
    })));
}

// ============= VECTOR CREATION IMPLEMENTATIONS =============

/// vector procedure
fn primitive_vector(args: &[Value]) -> Result<Value> {
    Ok(Value::vector(args.to_vec()))
}

/// make-vector procedure
fn primitive_make_vector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(DiagnosticError::runtime_error(
            format!("make-vector expects 1 or 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let length = args[0].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "make-vector first argument must be a non-negative integer".to_string(),
            None,
        )
    })?;
    
    if length < 0 {
        return Err(DiagnosticError::runtime_error(
            "make-vector length must be non-negative".to_string(),
            None,
        ));
    }
    
    let fill = if args.len() == 2 {
        args[1].clone())
    } else {
        Value::Unspecified
    };
    
    let elements = vec![fill; length as usize];
    Ok(Value::vector(elements))
}

/// vector-copy procedure
fn primitive_vector_copy(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(DiagnosticError::runtime_error(
            format!("vector-copy expects 1 to 3 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let vector = extract_vector(&args[0], "vector-copy")?;
    let length = vector.len();
    
    let start = if args.len() > 1 {
        let start_idx = args[1].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "vector-copy start index must be an integer".to_string(),
                None,
            )
        })? as usize;
        
        if start_idx > length {
            return Err(DiagnosticError::runtime_error(
                "vector-copy start index out of bounds".to_string(),
                None,
            ));
        }
        start_idx
    } else {
        0
    };
    
    let end = if args.len() > 2 {
        let end_idx = args[2].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "vector-copy end index must be an integer".to_string(),
                None,
            )
        })? as usize;
        
        if end_idx > length || end_idx < start {
            return Err(DiagnosticError::runtime_error(
                "vector-copy end index out of bounds".to_string(),
                None,
            ));
        }
        end_idx
    } else {
        length
    };
    
    let result = vector[start..end].to_vec();
    Ok(Value::vector(result))
}

// ============= VECTOR PREDICATE IMPLEMENTATIONS =============

/// vector? predicate
fn primitive_vector_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("vector? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    Ok(Value::boolean(args[0].is_vector()))
}

// ============= VECTOR ACCESSOR IMPLEMENTATIONS =============

/// vector-length procedure
fn primitive_vector_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("vector-length expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let vector = extract_vector(&args[0], "vector-length")?;
    Ok(Value::integer(vector.len() as i64))
}

/// vector-ref procedure
fn primitive_vector_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("vector-ref expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let vector = extract_vector(&args[0], "vector-ref")?;
    let index = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "vector-ref index must be an integer".to_string(),
            None,
        )
    })? as usize;
    
    if index >= vector.len() {
        return Err(DiagnosticError::runtime_error(
            "vector-ref index out of bounds".to_string(),
            None,
        ));
    }
    
    Ok(vector[index].clone())
}

/// vector-set! procedure (mutation)
fn primitive_vector_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(DiagnosticError::runtime_error(
            format!("vector-set! expects 3 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let vector_value = &args[0];
    let index = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "vector-set! index must be an integer".to_string(),
            None,
        )
    })? as usize;
    let new_value = &args[2];
    
    match vector_value {
        Value::Vector(vector_ref) => {
            let mut vector = vector_ref.write().unwrap();
            
            if index >= vector.len() {
                return Err(DiagnosticError::runtime_error(
                    "vector-set! index out of bounds".to_string(),
                    None,
                ));
            }
            
            vector[index] = new_value.clone());
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "vector-set! requires a vector".to_string(),
            None,
        )),
    }
}

// ============= VECTOR MANIPULATION IMPLEMENTATIONS =============

/// vector-fill! procedure (mutation)
fn primitive_vector_fill(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(DiagnosticError::runtime_error(
            format!("vector-fill! expects 2 to 4 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let vector_value = &args[0];
    let fill_value = &args[1];
    
    match vector_value {
        Value::Vector(vector_ref) => {
            let mut vector = vector_ref.write().unwrap();
            let length = vector.len();
            
            let start = if args.len() > 2 {
                let start_idx = args[2].as_integer().ok_or_else(|| {
                    DiagnosticError::runtime_error(
                        "vector-fill! start index must be an integer".to_string(),
                        None,
                    )
                })? as usize;
                
                if start_idx > length {
                    return Err(DiagnosticError::runtime_error(
                        "vector-fill! start index out of bounds".to_string(),
                        None,
                    ));
                }
                start_idx
            } else {
                0
            };
            
            let end = if args.len() > 3 {
                let end_idx = args[3].as_integer().ok_or_else(|| {
                    DiagnosticError::runtime_error(
                        "vector-fill! end index must be an integer".to_string(),
                        None,
                    )
                })? as usize;
                
                if end_idx > length || end_idx < start {
                    return Err(DiagnosticError::runtime_error(
                        "vector-fill! end index out of bounds".to_string(),
                        None,
                    ));
                }
                end_idx
            } else {
                length
            };
            
            for i in start..end {
                vector[i] = fill_value.clone());
            }
            
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "vector-fill! requires a vector".to_string(),
            None,
        )),
    }
}

/// vector-copy! procedure (mutation)
fn primitive_vector_copy_mut(args: &[Value]) -> Result<Value> {
    if args.len() < 3 || args.len() > 5 {
        return Err(DiagnosticError::runtime_error(
            format!("vector-copy! expects 3 to 5 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let to_vector = &args[0];
    let at = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "vector-copy! at index must be an integer".to_string(),
            None,
        )
    })? as usize;
    let from_vector_value = &args[2];
    
    let from_vector = extract_vector(from_vector_value, "vector-copy!")?;
    let from_length = from_vector.len();
    
    let start = if args.len() > 3 {
        let start_idx = args[3].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "vector-copy! start index must be an integer".to_string(),
                None,
            )
        })? as usize;
        
        if start_idx > from_length {
            return Err(DiagnosticError::runtime_error(
                "vector-copy! start index out of bounds".to_string(),
                None,
            ));
        }
        start_idx
    } else {
        0
    };
    
    let end = if args.len() > 4 {
        let end_idx = args[4].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "vector-copy! end index must be an integer".to_string(),
                None,
            )
        })? as usize;
        
        if end_idx > from_length || end_idx < start {
            return Err(DiagnosticError::runtime_error(
                "vector-copy! end index out of bounds".to_string(),
                None,
            ));
        }
        end_idx
    } else {
        from_length
    };
    
    match to_vector {
        Value::Vector(to_vector_ref) => {
            let mut to_vec = to_vector_ref.write().unwrap();
            let copy_length = end - start;
            
            if at + copy_length > to_vec.len() {
                return Err(DiagnosticError::runtime_error(
                    "vector-copy! destination range out of bounds".to_string(),
                    None,
                ));
            }
            
            for (i, j) in (start..end).enumerate() {
                to_vec[at + i] = from_vector[j].clone());
            }
            
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "vector-copy! requires a vector destination".to_string(),
            None,
        )),
    }
}

/// vector-append procedure
fn primitive_vector_append(args: &[Value]) -> Result<Value> {
    let mut result = Vec::new();
    
    for arg in args {
        let vector = extract_vector(arg, "vector-append")?;
        result.extend_from_slice(&vector);
    }
    
    Ok(Value::vector(result))
}

// ============= VECTOR CONVERSION IMPLEMENTATIONS =============

/// vector->list procedure
fn primitive_vector_to_list(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(DiagnosticError::runtime_error(
            format!("vector->list expects 1 to 3 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let vector = extract_vector(&args[0], "vector->list")?;
    let length = vector.len();
    
    let start = if args.len() > 1 {
        let start_idx = args[1].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "vector->list start index must be an integer".to_string(),
                None,
            )
        })? as usize;
        
        if start_idx > length {
            return Err(DiagnosticError::runtime_error(
                "vector->list start index out of bounds".to_string(),
                None,
            ));
        }
        start_idx
    } else {
        0
    };
    
    let end = if args.len() > 2 {
        let end_idx = args[2].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "vector->list end index must be an integer".to_string(),
                None,
            )
        })? as usize;
        
        if end_idx > length || end_idx < start {
            return Err(DiagnosticError::runtime_error(
                "vector->list end index out of bounds".to_string(),
                None,
            ));
        }
        end_idx
    } else {
        length
    };
    
    let slice = vector[start..end].to_vec();
    Ok(Value::list(slice))
}

/// list->vector procedure
fn primitive_list_to_vector(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("list->vector expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let list = args[0].as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "list->vector requires a proper list".to_string(),
            None,
        )
    })?;
    
    Ok(Value::vector(list))
}

// ============= HELPER FUNCTIONS =============

/// Extracts a vector from a Value (borrowing the contents).
fn extract_vector(value: &Value, operation: &str) -> Result<Vec<Value>> {
    match value {
        Value::Vector(vector_ref) => Ok(vector_ref.read().unwrap().clone()),
        _ => Err(DiagnosticError::runtime_error(
            format!("{operation} requires a vector"),
            None,
        )),
    }
}

// ============= VECTOR ITERATION IMPLEMENTATIONS =============

/// vector-map procedure - R7RS required
fn primitive_vector_map(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(DiagnosticError::runtime_error(
            "vector-map requires at least 2 arguments".to_string(),
            None,
        ));
    }
    
    let procedure = &args[0];
    let vectors = &args[1..];
    
    // Verify procedure is callable
    if !procedure.is_procedure() {
        return Err(DiagnosticError::runtime_error(
            "vector-map first argument must be a procedure".to_string(),
            None,
        ));
    }
    
    // Convert all arguments to vectors and find minimum length
    let mut vector_data = Vec::new();
    let mut min_length = usize::MAX;
    
    for (_i, vector_arg) in vectors.iter().enumerate() {
        let vector = extract_vector(vector_arg, "vector-map")?;
        min_length = min_length.min(vector.len());
        vector_data.push(vector);
    }
    
    // If any vector is empty, return empty vector
    if min_length == 0 || min_length == usize::MAX {
        return Ok(Value::vector(Vec::new()));
    }
    
    // Apply procedure to each position across all vectors
    let mut results = Vec::new();
    
    for i in 0..min_length {
        let mut proc_args = Vec::new();
        for vector in &vector_data {
            proc_args.push(vector[i].clone());
        }
        
        // Apply the procedure - for now we can only handle primitive procedures
        match procedure {
            Value::Primitive(prim) => {
                let result = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                    PrimitiveImpl::Native(func) => func(&proc_args)?,
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(DiagnosticError::runtime_error(
                            "vector-map with foreign functions not yet implemented".to_string(),
                            None,
                        ));
                    }
                };
                results.push(result);
            },
            _ => {
                return Err(DiagnosticError::runtime_error(
                    "vector-map with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                    None,
                ));
            }
        }
    }
    
    Ok(Value::vector(results))
}

/// vector-for-each procedure - R7RS required
fn primitive_vector_for_each(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(DiagnosticError::runtime_error(
            "vector-for-each requires at least 2 arguments".to_string(),
            None,
        ));
    }
    
    let procedure = &args[0];
    let vectors = &args[1..];
    
    // Verify procedure is callable
    if !procedure.is_procedure() {
        return Err(DiagnosticError::runtime_error(
            "vector-for-each first argument must be a procedure".to_string(),
            None,
        ));
    }
    
    // Convert all arguments to vectors and find minimum length
    let mut vector_data = Vec::new();
    let mut min_length = usize::MAX;
    
    for (_i, vector_arg) in vectors.iter().enumerate() {
        let vector = extract_vector(vector_arg, "vector-for-each")?;
        min_length = min_length.min(vector.len());
        vector_data.push(vector);
    }
    
    // If any vector is empty, return unspecified immediately
    if min_length == 0 || min_length == usize::MAX {
        return Ok(Value::Unspecified);
    }
    
    // Apply procedure to each position across all vectors for side effects
    for i in 0..min_length {
        let mut proc_args = Vec::new();
        for vector in &vector_data {
            proc_args.push(vector[i].clone());
        }
        
        // Apply the procedure - for now we can only handle primitive procedures
        match procedure {
            Value::Primitive(prim) => {
                match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => {
                        // Call the function but ignore the result (for-each is for side effects)
                        func(&proc_args)?;
                    },
                    PrimitiveImpl::Native(func) => {
                        // Call the function but ignore the result (for-each is for side effects)
                        func(&proc_args)?;
                    },
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(DiagnosticError::runtime_error(
                            "vector-for-each with foreign functions not yet implemented".to_string(),
                            None,
                        ));
                    }
                }
            },
            _ => {
                return Err(DiagnosticError::runtime_error(
                    "vector-for-each with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                    None,
                ));
            }
        }
    }
    
    // vector-for-each returns unspecified
    Ok(Value::Unspecified)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_vector_creation() {
        let args = vec![Value::integer(1), Value::integer(2), Value::integer(3)];
        let vector = primitive_vector(&args).unwrap();
        
        assert!(vector.is_vector());
        
        let length = primitive_vector_length(&[vector]).unwrap();
        assert_eq!(length, Value::integer(3));
    }
    
    #[test]
    fn test_make_vector() {
        let args = vec![Value::integer(5), Value::string("hello")];
        let vector = primitive_make_vector(&args).unwrap();
        
        let length = primitive_vector_length(&[vector.clone())]).unwrap();
        assert_eq!(length, Value::integer(5));
        
        // Check that all elements are "hello"
        let first_element = primitive_vector_ref(&[vector, Value::integer(0)]).unwrap();
        assert_eq!(first_element, Value::string("hello"));
    }
    
    #[test]
    fn test_vector_ref_and_set() {
        let vector = Value::vector(vec![
            Value::string("a"),
            Value::string("b"),
            Value::string("c"),
        ]);
        
        let element = primitive_vector_ref(&[vector.clone()), Value::integer(1)]).unwrap();
        assert_eq!(element, Value::string("b"));
        
        // Test vector-set!
        let result = primitive_vector_set(&[
            vector.clone()),
            Value::integer(1),
            Value::string("modified"),
        ]).unwrap();
        assert_eq!(result, Value::Unspecified);
        
        let modified_element = primitive_vector_ref(&[vector, Value::integer(1)]).unwrap();
        assert_eq!(modified_element, Value::string("modified"));
    }
    
    #[test]
    fn test_vector_append() {
        let vec1 = Value::vector(vec![Value::integer(1), Value::integer(2)]);
        let vec2 = Value::vector(vec![Value::integer(3), Value::integer(4)]);
        
        let result = primitive_vector_append(&[vec1, vec2]).unwrap();
        
        let length = primitive_vector_length(&[result.clone())]).unwrap();
        assert_eq!(length, Value::integer(4));
        
        let first = primitive_vector_ref(&[result.clone()), Value::integer(0)]).unwrap();
        assert_eq!(first, Value::integer(1));
        
        let last = primitive_vector_ref(&[result, Value::integer(3)]).unwrap();
        assert_eq!(last, Value::integer(4));
    }
    
    #[test]
    fn test_vector_copy() {
        let original = Value::vector(vec![
            Value::string("a"),
            Value::string("b"),
            Value::string("c"),
            Value::string("d"),
        ]);
        
        // Copy a slice
        let args = vec![original, Value::integer(1), Value::integer(3)];
        let copy = primitive_vector_copy(&args).unwrap();
        
        let length = primitive_vector_length(&[copy.clone())]).unwrap();
        assert_eq!(length, Value::integer(2));
        
        let first = primitive_vector_ref(&[copy.clone()), Value::integer(0)]).unwrap();
        assert_eq!(first, Value::string("b"));
        
        let second = primitive_vector_ref(&[copy, Value::integer(1)]).unwrap();
        assert_eq!(second, Value::string("c"));
    }
    
    #[test]
    fn test_vector_list_conversion() {
        let list = Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]);
        
        let vector = primitive_list_to_vector(&[list]).unwrap();
        
        let length = primitive_vector_length(&[vector.clone())]).unwrap();
        assert_eq!(length, Value::integer(3));
        
        let back_to_list = primitive_vector_to_list(&[vector]).unwrap();
        let expected = Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]);
        assert_eq!(back_to_list, expected);
    }
    
    #[test]
    fn test_vector_fill() {
        let vector = Value::vector(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
            Value::integer(4),
        ]);
        
        // Fill the middle portion
        let result = primitive_vector_fill(&[
            vector.clone()),
            Value::string("filled"),
            Value::integer(1),
            Value::integer(3),
        ]).unwrap();
        assert_eq!(result, Value::Unspecified);
        
        // Check that positions 1 and 2 are filled
        let elem1 = primitive_vector_ref(&[vector.clone()), Value::integer(1)]).unwrap();
        assert_eq!(elem1, Value::string("filled"));
        
        let elem2 = primitive_vector_ref(&[vector.clone()), Value::integer(2)]).unwrap();
        assert_eq!(elem2, Value::string("filled"));
        
        // Check that positions 0 and 3 are unchanged
        let elem0 = primitive_vector_ref(&[vector.clone()), Value::integer(0)]).unwrap();
        assert_eq!(elem0, Value::integer(1));
        
        let elem3 = primitive_vector_ref(&[vector, Value::integer(3)]).unwrap();
        assert_eq!(elem3, Value::integer(4));
    }
    
    #[test]
    fn test_vector_map_single_vector() {
        // Test vector-map with a simple procedure that doubles numbers
        let double_proc = Arc::new(PrimitiveProcedure {
            name: "double".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| {
                if let Some(n) = args[0].as_number() {
                    Ok(Value::number(n * 2.0))
                } else {
                    Ok(Value::Unspecified)
                }
            }),
            effects: vec![Effect::Pure],
        });
        
        let vector = Value::vector(vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)]);
        let args = vec![Value::Primitive(double_proc), vector];
        let result = primitive_vector_map(&args).unwrap();
        
        let expected = Value::vector(vec![Value::number(2.0), Value::number(4.0), Value::number(6.0)]);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_vector_map_multiple_vectors() {
        // Test vector-map with multiple vectors
        let add_proc = Arc::new(PrimitiveProcedure {
            name: "+".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(|args| {
                let sum = args.iter()
                    .filter_map(|v| v.as_number())
                    .fold(0.0, |acc, n| acc + n);
                Ok(Value::number(sum))
            }),
            effects: vec![Effect::Pure],
        });
        
        let vector1 = Value::vector(vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)]);
        let vector2 = Value::vector(vec![Value::number(4.0), Value::number(5.0), Value::number(6.0)]);
        let args = vec![Value::Primitive(add_proc), vector1, vector2];
        let result = primitive_vector_map(&args).unwrap();
        
        let expected = Value::vector(vec![Value::number(5.0), Value::number(7.0), Value::number(9.0)]);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_vector_map_different_lengths() {
        // Test vector-map with vectors of different lengths - should use shortest
        let add_proc = Arc::new(PrimitiveProcedure {
            name: "+".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(|args| {
                let sum = args.iter()
                    .filter_map(|v| v.as_number())
                    .fold(0.0, |acc, n| acc + n);
                Ok(Value::number(sum))
            }),
            effects: vec![Effect::Pure],
        });
        
        let vector1 = Value::vector(vec![Value::number(1.0), Value::number(2.0)]);
        let vector2 = Value::vector(vec![Value::number(4.0), Value::number(5.0), Value::number(6.0)]);
        let args = vec![Value::Primitive(add_proc), vector1, vector2];
        let result = primitive_vector_map(&args).unwrap();
        
        let expected = Value::vector(vec![Value::number(5.0), Value::number(7.0)]);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_vector_map_empty_vector() {
        let double_proc = Arc::new(PrimitiveProcedure {
            name: "double".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| {
                if let Some(n) = args[0].as_number() {
                    Ok(Value::number(n * 2.0))
                } else {
                    Ok(Value::Unspecified)
                }
            }),
            effects: vec![Effect::Pure],
        });
        
        let empty_vector = Value::vector(Vec::new());
        let args = vec![Value::Primitive(double_proc), empty_vector];
        let result = primitive_vector_map(&args).unwrap();
        
        assert_eq!(result, Value::vector(Vec::new()));
    }
    
    #[test]
    fn test_vector_for_each_basic() {
        // Test vector-for-each with a simple side-effect procedure
        let identity_proc = Arc::new(PrimitiveProcedure {
            name: "identity".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| Ok(args[0].clone())),
            effects: vec![Effect::Pure],
        });
        
        let vector = Value::vector(vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)]);
        let args = vec![Value::Primitive(identity_proc), vector];
        let result = primitive_vector_for_each(&args).unwrap();
        
        // vector-for-each should return unspecified
        assert_eq!(result, Value::Unspecified);
    }
    
    #[test]
    fn test_vector_for_each_multiple_vectors() {
        let add_proc = Arc::new(PrimitiveProcedure {
            name: "+".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(|args| {
                let sum = args.iter()
                    .filter_map(|v| v.as_number())
                    .fold(0.0, |acc, n| acc + n);
                Ok(Value::number(sum))
            }),
            effects: vec![Effect::Pure],
        });
        
        let vector1 = Value::vector(vec![Value::number(1.0), Value::number(2.0)]);
        let vector2 = Value::vector(vec![Value::number(4.0), Value::number(5.0)]);
        let args = vec![Value::Primitive(add_proc), vector1, vector2];
        let result = primitive_vector_for_each(&args).unwrap();
        
        assert_eq!(result, Value::Unspecified);
    }
    
    #[test]
    fn test_vector_map_for_each_errors() {
        // Test errors for both vector-map and vector-for-each
        
        // Non-procedure first argument
        let args = vec![Value::integer(42), Value::vector(vec![Value::integer(1)])];
        assert!(primitive_vector_map(&args).is_err());
        assert!(primitive_vector_for_each(&args).is_err());
        
        // Non-vector argument
        let proc = Arc::new(PrimitiveProcedure {
            name: "test".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| Ok(args[0].clone())),
            effects: vec![Effect::Pure],
        });
        let args = vec![Value::Primitive(proc.clone()), Value::integer(42)];
        assert!(primitive_vector_map(&args).is_err());
        assert!(primitive_vector_for_each(&args).is_err());
        
        // Too few arguments
        assert!(primitive_vector_map(&[]).is_err());
        assert!(primitive_vector_for_each(&[]).is_err());
        
        let args = vec![Value::Primitive(proc)];
        assert!(primitive_vector_map(&args).is_err());
        assert!(primitive_vector_for_each(&args).is_err());
    }
}