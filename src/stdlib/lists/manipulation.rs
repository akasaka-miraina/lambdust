//! List manipulation functions (append, reverse, set-car!, etc.)

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use crate::stdlib::lists::common::{is_proper_list, copy_list};
use std::sync::Arc;

/// Binds list manipulation operations.
pub fn bind_list_manipulation(env: &Arc<ThreadSafeEnvironment>) {
    // append
    env.define("append".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "append".to_string(),
        arity_min: 0,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_append),
        effects: vec![Effect::Pure],
    })));
    
    // reverse
    env.define("reverse".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "reverse".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_reverse),
        effects: vec![Effect::Pure],
    })));
    
    // set-car!
    env.define("set-car!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "set-car!".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_set_car),
        effects: vec![Effect::State],
    })));
    
    // set-cdr!
    env.define("set-cdr!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "set-cdr!".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_set_cdr),
        effects: vec![Effect::State],
    })));
    
    // list-set!
    env.define("list-set!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list-set!".to_string(),
        arity_min: 3,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_list_set),
        effects: vec![Effect::State],
    })));
    
    // list-copy
    env.define("list-copy".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list-copy".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_list_copy),
        effects: vec![Effect::Pure],
    })));
}

/// append procedure
fn primitive_append(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Nil);
    }
    
    if args.len() == 1 {
        return Ok(args[0].clone());
    }
    
    // All but the last argument must be proper lists
    for arg in &args[..args.len() - 1] {
        if !is_proper_list(arg) {
            return Err(Box::new(DiagnosticError::runtime_error(
                "append arguments (except the last) must be proper lists".to_string(),
                None,
            )));
        }
    }
    
    let mut result = args[args.len() - 1].clone();
    
    for arg in args[..args.len() - 1].iter().rev() {
        if let Some(list) = arg.as_list() {
            for item in list.into_iter().rev() {
                result = Value::pair(item, result);
            }
        }
    }
    
    Ok(result)
}

/// reverse procedure
fn primitive_reverse(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("reverse expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    if let Some(list) = args[0].as_list() {
        let mut reversed = list;
        reversed.reverse();
        Ok(Value::list(reversed))
    } else {
        Err(Box::new(DiagnosticError::runtime_error(
            "reverse requires a proper list".to_string(),
            None,
        )))
    }
}

/// set-car! procedure (mutation)
fn primitive_set_car(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("set-car! expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    match &args[0] {
        Value::MutablePair(car_ref, _) => {
            if let Ok(mut car) = car_ref.write() {
                *car = args[1].clone();
                Ok(Value::Unspecified)
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "set-car! failed to acquire write lock".to_string(),
                    None,
                )))
            }
        }
        Value::Pair(_, _) => {
            Err(Box::new(DiagnosticError::runtime_error(
                "set-car! requires a mutable pair (immutable pair given)".to_string(),
                None,
            )))
        }
        _ => {
            Err(Box::new(DiagnosticError::runtime_error(
                "set-car! requires a pair".to_string(),
                None,
            )))
        }
    }
}

/// set-cdr! procedure (mutation)
fn primitive_set_cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("set-cdr! expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    match &args[0] {
        Value::MutablePair(_, cdr_ref) => {
            if let Ok(mut cdr) = cdr_ref.write() {
                *cdr = args[1].clone();
                Ok(Value::Unspecified)
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "set-cdr! failed to acquire write lock".to_string(),
                    None,
                )))
            }
        }
        Value::Pair(_, _) => {
            Err(Box::new(DiagnosticError::runtime_error(
                "set-cdr! requires a mutable pair (immutable pair given)".to_string(),
                None,
            )))
        }
        _ => {
            Err(Box::new(DiagnosticError::runtime_error(
                "set-cdr! requires a pair".to_string(),
                None,
            )))
        }
    }
}

/// list-set! procedure (mutation)
fn primitive_list_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("list-set! expects 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let index = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "list-set! index must be a non-negative integer".to_string(),
            None,
        )
    })?;
    
    if index < 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "list-set! index must be non-negative".to_string(),
            None,
        )));
    }
    
    set_list_element(&args[0], index, args[2].clone())
}

/// list-copy procedure
fn primitive_list_copy(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("list-copy expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    copy_list(&args[0])
}

/// Helper function to set list element at index
fn set_list_element(list: &Value, index: i64, value: Value) -> Result<Value> {
    let mut current = list;
    let mut remaining = index;
    
    while remaining > 0 {
        match current {
            Value::MutablePair(_, cdr_ref) => {
                if let Ok(cdr) = cdr_ref.read() {
                    // We need to navigate without holding the lock
                    // This is a simplified approach - in practice we'd need better handling
                    return Err(Box::new(DiagnosticError::runtime_error(
                        "list-set! complex mutable pair traversal not fully implemented".to_string(),
                        None,
                    )));
                } else {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        "list-set! failed to acquire read lock".to_string(),
                        None,
                    )));
                }
            }
            Value::Pair(_, cdr) => {
                current = cdr;
                remaining -= 1;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "list-set! index out of bounds".to_string(),
                    None,
                )));
            }
        }
    }
    
    // Set the car of the current element
    match current {
        Value::MutablePair(car_ref, _) => {
            if let Ok(mut car) = car_ref.write() {
                *car = value;
                Ok(Value::Unspecified)
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "list-set! failed to acquire write lock".to_string(),
                    None,
                )))
            }
        }
        Value::Pair(_, _) => {
            Err(Box::new(DiagnosticError::runtime_error(
                "list-set! requires a mutable list (immutable pair found)".to_string(),
                None,
            )))
        }
        _ => {
            Err(Box::new(DiagnosticError::runtime_error(
                "list-set! index out of bounds".to_string(),
                None,
            )))
        }
    }
}