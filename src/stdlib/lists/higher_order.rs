//! Higher-order functions for list processing (map, filter, fold, etc.)

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use crate::stdlib::lists::common::{MAP_ARITY_ERROR, FOR_EACH_ARITY_ERROR, is_proper_list};
use std::sync::Arc;

/// Binds higher-order functions.
pub fn bind_higher_order_functions(env: &Arc<ThreadSafeEnvironment>) {
    // map
    env.define("map".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "map".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_map),
        effects: vec![Effect::Pure],
    })));
    
    // for-each
    env.define("for-each".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "for-each".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_for_each),
        effects: vec![Effect::IO], // Could have side effects
    })));
    
    // filter
    env.define("filter".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "filter".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_filter),
        effects: vec![Effect::Pure],
    })));
    
    // fold-left
    env.define("fold-left".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "fold-left".to_string(),
        arity_min: 3,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_fold_left),
        effects: vec![Effect::Pure],
    })));
    
    // fold-right
    env.define("fold-right".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "fold-right".to_string(),
        arity_min: 3,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_fold_right),
        effects: vec![Effect::Pure],
    })));
    
    // any (exists)
    env.define("any".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "any".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_any),
        effects: vec![Effect::Pure],
    })));
    
    // every (for-all)
    env.define("every".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "every".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_every),
        effects: vec![Effect::Pure],
    })));
}

/// map procedure - Enhanced R7RS implementation supporting multiple lists
fn primitive_map(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            MAP_ARITY_ERROR.to_string(),
            None,
        )));
    }
    
    let procedure = &args[0];
    let lists = &args[1..];
    
    // Verify procedure is callable
    if !procedure.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "map first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    // Convert all arguments to proper lists and find minimum length
    let mut list_vectors = Vec::new();
    let mut min_length = usize::MAX;
    
    for (i, list_arg) in lists.iter().enumerate() {
        if let Some(list_values) = list_arg.as_list() {
            min_length = min_length.min(list_values.len());
            list_vectors.push(list_values);
        } else {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("map argument {} must be a list", i + 2),
                None,
            )));
        }
    }
    
    // If any list is empty, return empty list
    if min_length == 0 || min_length == usize::MAX {
        return Ok(Value::Nil);
    }
    
    // Apply procedure to each position across all lists
    let mut results = Vec::new();
    
    for i in 0..min_length {
        let mut proc_args = Vec::new();
        for list in &list_vectors {
            proc_args.push(list[i].clone());
        }
        
        // Apply the procedure - for now we can only handle primitive procedures
        match procedure {
            Value::Primitive(prim) => {
                let result = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                    PrimitiveImpl::Native(func) => func(&proc_args)?,
                    PrimitiveImpl::EvaluatorIntegrated(_) => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "map with EvaluatorIntegrated functions requires evaluator access (not yet implemented)".to_string(),
                            None,
                        )));
                    }
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "map with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
                results.push(result);
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "map with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(Value::list(results))
}

/// for-each procedure - Enhanced R7RS implementation supporting multiple lists
fn primitive_for_each(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            FOR_EACH_ARITY_ERROR.to_string(),
            None,
        )));
    }
    
    let procedure = &args[0];
    let lists = &args[1..];
    
    // Verify procedure is callable
    if !procedure.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "for-each first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    // Convert all arguments to proper lists and find minimum length
    let mut list_vectors = Vec::new();
    let mut min_length = usize::MAX;
    
    for (i, list_arg) in lists.iter().enumerate() {
        if let Some(list_values) = list_arg.as_list() {
            min_length = min_length.min(list_values.len());
            list_vectors.push(list_values);
        } else {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("for-each argument {} must be a list", i + 2),
                None,
            )));
        }
    }
    
    // If any list is empty, return unspecified immediately
    if min_length == 0 || min_length == usize::MAX {
        return Ok(Value::Unspecified);
    }
    
    // Apply procedure to each position across all lists for side effects
    for i in 0..min_length {
        let mut proc_args = Vec::new();
        for list in &list_vectors {
            proc_args.push(list[i].clone());
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
                    PrimitiveImpl::EvaluatorIntegrated(_) => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "for-each with EvaluatorIntegrated functions requires evaluator access (not yet implemented)".to_string(),
                            None,
                        )));
                    }
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "for-each with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "for-each with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(Value::Unspecified)
}

/// filter procedure
fn primitive_filter(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("filter expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let predicate = &args[0];
    let list = &args[1];
    
    // Verify predicate is callable
    if !predicate.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "filter first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    if let Some(list_values) = list.as_list() {
        let mut filtered = Vec::new();
        
        for item in list_values {
            // Apply predicate to each item
            let keep = match predicate {
                Value::Primitive(prim) => {
                    let result = match &prim.implementation {
                        PrimitiveImpl::RustFn(func) => func(&[item.clone()])?,
                        PrimitiveImpl::Native(func) => func(&[item.clone()])?,
                        PrimitiveImpl::EvaluatorIntegrated(_) => {
                            return Err(Box::new(DiagnosticError::runtime_error(
                                "filter with EvaluatorIntegrated functions requires evaluator access (not yet implemented)".to_string(),
                                None,
                            )));
                        }
                        PrimitiveImpl::ForeignFn { .. } => {
                            return Err(Box::new(DiagnosticError::runtime_error(
                                "filter with foreign functions not yet implemented".to_string(),
                                None,
                            )));
                        }
                    };
                    !result.is_falsy() // In Scheme, only #f is false
                },
                _ => {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        "filter with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                        None,
                    )));
                }
            };
            
            if keep {
                filtered.push(item);
            }
        }
        
        Ok(Value::list(filtered))
    } else {
        Err(Box::new(DiagnosticError::runtime_error(
            "filter second argument must be a list".to_string(),
            None,
        )))
    }
}

/// fold-left procedure (accumulate from left)
fn primitive_fold_left(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "fold-left expects at least 3 arguments".to_string(),
            None,
        )));
    }
    
    let procedure = &args[0];
    let mut acc = args[1].clone();
    let lists = &args[2..];
    
    // Verify procedure is callable
    if !procedure.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "fold-left first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    // Convert all list arguments to proper lists and find minimum length
    let mut list_vectors = Vec::new();
    let mut min_length = usize::MAX;
    
    for (i, list_arg) in lists.iter().enumerate() {
        if let Some(list_values) = list_arg.as_list() {
            min_length = min_length.min(list_values.len());
            list_vectors.push(list_values);
        } else {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("fold-left argument {} must be a list", i + 3),
                None,
            )));
        }
    }
    
    // If any list is empty, return the initial accumulator
    if min_length == 0 || min_length == usize::MAX {
        return Ok(acc);
    }
    
    // Fold from left to right
    for i in 0..min_length {
        let mut proc_args = vec![acc.clone()];
        for list in &list_vectors {
            proc_args.push(list[i].clone());
        }
        
        // Apply the procedure
        acc = match procedure {
            Value::Primitive(prim) => {
                match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                    PrimitiveImpl::Native(func) => func(&proc_args)?,
                    PrimitiveImpl::EvaluatorIntegrated(_) => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "fold-left with EvaluatorIntegrated functions requires evaluator access (not yet implemented)".to_string(),
                            None,
                        )));
                    }
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "fold-left with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                }
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "fold-left with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                    None,
                )));
            }
        };
    }
    
    Ok(acc)
}

/// fold-right procedure (accumulate from right)
fn primitive_fold_right(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "fold-right expects at least 3 arguments".to_string(),
            None,
        )));
    }
    
    let procedure = &args[0];
    let acc = args[1].clone();
    let lists = &args[2..];
    
    // Verify procedure is callable
    if !procedure.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "fold-right first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    // Convert all list arguments to proper lists and find minimum length
    let mut list_vectors = Vec::new();
    let mut min_length = usize::MAX;
    
    for (i, list_arg) in lists.iter().enumerate() {
        if let Some(list_values) = list_arg.as_list() {
            min_length = min_length.min(list_values.len());
            list_vectors.push(list_values);
        } else {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("fold-right argument {} must be a list", i + 3),
                None,
            )));
        }
    }
    
    // If any list is empty, return the initial accumulator
    if min_length == 0 || min_length == usize::MAX {
        return Ok(acc);
    }
    
    // Fold from right to left (recursive approach)
    fold_right_recursive(procedure, &list_vectors, min_length - 1, acc)
}

/// Helper function for fold-right recursive implementation
fn fold_right_recursive(
    procedure: &Value,
    list_vectors: &[Vec<Value>],
    index: usize,
    acc: Value
) -> Result<Value> {
    // Base case: if we've processed all elements
    if index == usize::MAX {
        return Ok(acc);
    }
    
    // Get elements at current index from all lists
    let mut proc_args = Vec::new();
    for list in list_vectors {
        proc_args.push(list[index].clone());
    }
    
    // Recursively process the rest of the list first (right-to-left)
    let next_acc = if index > 0 {
        fold_right_recursive(procedure, list_vectors, index - 1, acc)?
    } else {
        acc
    };
    
    // Add the accumulated value as the last argument
    proc_args.push(next_acc);
    
    // Apply the procedure
    match procedure {
        Value::Primitive(prim) => {
            match &prim.implementation {
                PrimitiveImpl::RustFn(func) => func(&proc_args),
                PrimitiveImpl::Native(func) => func(&proc_args),
                PrimitiveImpl::EvaluatorIntegrated(_) => {
                    Err(Box::new(DiagnosticError::runtime_error(
                        "fold-right with EvaluatorIntegrated functions requires evaluator access (not yet implemented)".to_string(),
                        None,
                    )))
                }
                PrimitiveImpl::ForeignFn { .. } => {
                    Err(Box::new(DiagnosticError::runtime_error(
                        "fold-right with foreign functions not yet implemented".to_string(),
                        None,
                    )))
                }
            }
        },
        _ => {
            Err(Box::new(DiagnosticError::runtime_error(
                "fold-right with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                None,
            )))
        }
    }
}

/// any procedure - returns #t if any element satisfies the predicate
fn primitive_any(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("any expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let predicate = &args[0];
    let list = &args[1];
    
    if !predicate.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "any first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    if let Some(list_values) = list.as_list() {
        for item in list_values {
            let result = match predicate {
                Value::Primitive(prim) => {
                    match &prim.implementation {
                        PrimitiveImpl::RustFn(func) => func(&[item])?,
                        PrimitiveImpl::Native(func) => func(&[item])?,
                        _ => return Err(Box::new(DiagnosticError::runtime_error(
                            "any with complex procedures not yet implemented".to_string(),
                            None,
                        )))
                    }
                },
                _ => return Err(Box::new(DiagnosticError::runtime_error(
                    "any with user-defined procedures not yet implemented".to_string(),
                    None,
                )))
            };
            
            if !result.is_falsy() {
                return Ok(Value::boolean(true));
            }
        }
        Ok(Value::boolean(false))
    } else {
        Err(Box::new(DiagnosticError::runtime_error(
            "any second argument must be a list".to_string(),
            None,
        )))
    }
}

/// every procedure - returns #t if all elements satisfy the predicate
fn primitive_every(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("every expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let predicate = &args[0];
    let list = &args[1];
    
    if !predicate.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "every first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    if let Some(list_values) = list.as_list() {
        for item in list_values {
            let result = match predicate {
                Value::Primitive(prim) => {
                    match &prim.implementation {
                        PrimitiveImpl::RustFn(func) => func(&[item])?,
                        PrimitiveImpl::Native(func) => func(&[item])?,
                        _ => return Err(Box::new(DiagnosticError::runtime_error(
                            "every with complex procedures not yet implemented".to_string(),
                            None,
                        )))
                    }
                },
                _ => return Err(Box::new(DiagnosticError::runtime_error(
                    "every with user-defined procedures not yet implemented".to_string(),
                    None,
                )))
            };
            
            if result.is_falsy() {
                return Ok(Value::boolean(false));
            }
        }
        Ok(Value::boolean(true))
    } else {
        Err(Box::new(DiagnosticError::runtime_error(
            "every second argument must be a list".to_string(),
            None,
        )))
    }
}