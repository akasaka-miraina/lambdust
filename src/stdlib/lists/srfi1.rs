//! SRFI-1 List Extensions
//!
//! This module provides SRFI-1 compliant list processing functions
//! that extend the basic R7RS list operations.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use crate::stdlib::lists::common::{values_equal, copy_list, is_proper_list};
use std::sync::Arc;

/// Binds SRFI-1 extensions.
pub fn bind_srfi1_extensions(env: &Arc<ThreadSafeEnvironment>) {
    // take
    env.define("take".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "take".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(srfi1_take),
        effects: vec![Effect::Pure],
    })));
    
    // drop
    env.define("drop".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "drop".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(srfi1_drop),
        effects: vec![Effect::Pure],
    })));
    
    // take-right
    env.define("take-right".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "take-right".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(srfi1_take_right),
        effects: vec![Effect::Pure],
    })));
    
    // drop-right
    env.define("drop-right".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "drop-right".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(srfi1_drop_right),
        effects: vec![Effect::Pure],
    })));
    
    // take-while
    env.define("take-while".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "take-while".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(srfi1_take_while),
        effects: vec![Effect::Pure],
    })));
    
    // drop-while
    env.define("drop-while".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "drop-while".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(srfi1_drop_while),
        effects: vec![Effect::Pure],
    })));
    
    // split-at
    env.define("split-at".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "split-at".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(srfi1_split_at),
        effects: vec![Effect::Pure],
    })));
    
    // last
    env.define("last".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "last".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(srfi1_last),
        effects: vec![Effect::Pure],
    })));
    
    // last-pair
    env.define("last-pair".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "last-pair".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(srfi1_last_pair),
        effects: vec![Effect::Pure],
    })));
}

/// take - Take the first n elements of a list
fn srfi1_take(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("take expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let list_arg = &args[0];
    let n = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "take n must be a non-negative integer".to_string(),
            None,
        )
    })?;
    
    if n < 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "take n must be non-negative".to_string(),
            None,
        )));
    }
    
    let mut current = list_arg;
    let mut result = Vec::new();
    let mut count = 0;
    
    while count < n {
        match current {
            Value::Nil => break,
            Value::Pair(car, cdr) => {
                result.push((**car).clone());
                current = cdr;
                count += 1;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "take requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
    
    if count < n {
        return Err(Box::new(DiagnosticError::runtime_error(
            "take: list too short".to_string(),
            None,
        )));
    }
    
    Ok(Value::list(result))
}

/// drop - Drop the first n elements of a list
fn srfi1_drop(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("drop expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let mut list_arg = args[0].clone();
    let n = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "drop n must be a non-negative integer".to_string(),
            None,
        )
    })?;
    
    if n < 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "drop n must be non-negative".to_string(),
            None,
        )));
    }
    
    let mut count = 0;
    while count < n {
        match list_arg {
            Value::Nil => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "drop: list too short".to_string(),
                    None,
                )));
            }
            Value::Pair(_, cdr) => {
                list_arg = cdr.as_ref().clone();
                count += 1;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "drop requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(list_arg)
}

/// take-right - Take the last n elements of a list
fn srfi1_take_right(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("take-right expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let list_arg = &args[0];
    let n = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "take-right n must be a non-negative integer".to_string(),
            None,
        )
    })?;
    
    if n < 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "take-right n must be non-negative".to_string(),
            None,
        )));
    }
    
    let list = list_arg.as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "take-right requires a proper list".to_string(),
            None,
        )
    })?;
    
    let list_len = list.len();
    if (n as usize) > list_len {
        return Err(Box::new(DiagnosticError::runtime_error(
            "take-right: n larger than list length".to_string(),
            None,
        )));
    }
    
    let start_idx = list_len - (n as usize);
    let result = list[start_idx..].to_vec();
    Ok(Value::list(result))
}

/// drop-right - Drop the last n elements of a list
fn srfi1_drop_right(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("drop-right expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let list_arg = &args[0];
    let n = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "drop-right n must be a non-negative integer".to_string(),
            None,
        )
    })?;
    
    if n < 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "drop-right n must be non-negative".to_string(),
            None,
        )));
    }
    
    let list = list_arg.as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "drop-right requires a proper list".to_string(),
            None,
        )
    })?;
    
    let list_len = list.len();
    if (n as usize) > list_len {
        return Ok(Value::Nil);
    }
    
    let end_idx = list_len - (n as usize);
    let result = list[..end_idx].to_vec();
    Ok(Value::list(result))
}

/// take-while - Take elements while predicate is true
fn srfi1_take_while(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("take-while expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let predicate = &args[0];
    let list_arg = &args[1];
    
    if !predicate.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "take-while first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    let mut result = Vec::new();
    let mut current = list_arg;
    
    loop {
        match current {
            Value::Nil => break,
            Value::Pair(car, cdr) => {
                // Apply predicate to current element
                let keep = match predicate {
                    Value::Primitive(prim) => {
                        let pred_result = match &prim.implementation {
                            PrimitiveImpl::RustFn(func) => func(&[(**car).clone()])?,
                            PrimitiveImpl::Native(func) => func(&[(**car).clone()])?,
                            _ => {
                                return Err(Box::new(DiagnosticError::runtime_error(
                                    "take-while with complex procedures not yet implemented".to_string(),
                                    None,
                                )));
                            }
                        };
                        !pred_result.is_falsy()
                    },
                    _ => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "take-while with user-defined procedures not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
                
                if !keep {
                    break;
                }
                
                result.push((**car).clone());
                current = cdr;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "take-while requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(Value::list(result))
}

/// drop-while - Drop elements while predicate is true
fn srfi1_drop_while(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("drop-while expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let predicate = &args[0];
    let mut list_arg = args[1].clone();
    
    if !predicate.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "drop-while first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    loop {
        match &list_arg {
            Value::Nil => break,
            Value::Pair(car, cdr) => {
                // Apply predicate to current element
                let keep_dropping = match predicate {
                    Value::Primitive(prim) => {
                        let pred_result = match &prim.implementation {
                            PrimitiveImpl::RustFn(func) => func(&[(**car).clone()])?,
                            PrimitiveImpl::Native(func) => func(&[(**car).clone()])?,
                            _ => {
                                return Err(Box::new(DiagnosticError::runtime_error(
                                    "drop-while with complex procedures not yet implemented".to_string(),
                                    None,
                                )));
                            }
                        };
                        !pred_result.is_falsy()
                    },
                    _ => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "drop-while with user-defined procedures not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
                
                if !keep_dropping {
                    break;
                }
                
                list_arg = cdr.as_ref().clone();
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "drop-while requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(list_arg)
}

/// split-at - Split a list at the nth position
fn srfi1_split_at(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("split-at expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let list_arg = &args[0];
    let n = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "split-at n must be a non-negative integer".to_string(),
            None,
        )
    })?;
    
    if n < 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "split-at n must be non-negative".to_string(),
            None,
        )));
    }
    
    let first_part = srfi1_take(&[list_arg.clone(), args[1].clone()])?;
    let second_part = srfi1_drop(&[list_arg.clone(), args[1].clone()])?;
    
    // Return as two values (we'll use a pair for now)
    Ok(Value::pair(first_part, second_part))
}

/// last - Return the last element of a non-empty list
fn srfi1_last(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("last expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let list_arg = &args[0];
    
    if let Some(list) = list_arg.as_list() {
        if list.is_empty() {
            return Err(Box::new(DiagnosticError::runtime_error(
                "last: empty list".to_string(),
                None,
            )));
        }
        Ok(list[list.len() - 1].clone())
    } else {
        Err(Box::new(DiagnosticError::runtime_error(
            "last requires a proper list".to_string(),
            None,
        )))
    }
}

/// last-pair - Return the last pair of a non-empty list
fn srfi1_last_pair(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("last-pair expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let mut current = &args[0];
    
    loop {
        match current {
            Value::Nil => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "last-pair: empty list".to_string(),
                    None,
                )));
            }
            Value::Pair(_, cdr) => {
                match cdr.as_ref() {
                    Value::Nil => return Ok(current.clone()),
                    _ => current = cdr,
                }
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "last-pair requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
}