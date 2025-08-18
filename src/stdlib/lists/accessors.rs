//! List accessor functions (list-ref, length, list-tail, etc.)

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use std::sync::Arc;

/// Binds list accessors.
pub fn bind_list_accessors(env: &Arc<ThreadSafeEnvironment>) {
    // Common car/cdr combinations
    let combinations = vec![
        ("caar", make_car_cdr_combination("caar")),
        ("cadr", make_car_cdr_combination("cadr")),
        ("cdar", make_car_cdr_combination("cdar")),
        ("cddr", make_car_cdr_combination("cddr")),
        // Add more as needed
    ];
    
    for (name, func) in combinations {
        env.define(name.to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
            name: name.to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(func),
            effects: vec![Effect::Pure],
        })));
    }
    
    // list-ref
    env.define("list-ref".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list-ref".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_list_ref),
        effects: vec![Effect::Pure],
    })));
    
    // list-tail
    env.define("list-tail".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list-tail".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_list_tail),
        effects: vec![Effect::Pure],
    })));
    
    // length
    env.define("length".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "length".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_length),
        effects: vec![Effect::Pure],
    })));
}

/// Creates a car/cdr combination function
fn make_car_cdr_combination(name: &str) -> fn(&[Value]) -> Result<Value> {
    match name {
        "caar" => |args| {
            let result = primitive_car(args)?;
            primitive_car(&[result])
        },
        "cadr" => |args| {
            let result = primitive_cdr(args)?;
            primitive_car(&[result])
        },
        "cdar" => |args| {
            let result = primitive_car(args)?;
            primitive_cdr(&[result])
        },
        "cddr" => |args| {
            let result = primitive_cdr(args)?;
            primitive_cdr(&[result])
        },
        _ => {
            fn unknown_combination(_args: &[Value]) -> Result<Value> {
                Err(Box::new(DiagnosticError::runtime_error(
                    "Unknown car/cdr combination".to_string(),
                    None,
                )))
            }
            unknown_combination
        },
    }
}

// Helper functions that need to be imported from basic.rs
fn primitive_car(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("car expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    match &args[0] {
        Value::Pair(car, _) => Ok((**car).clone()),
        Value::MutablePair(car_ref, _) => {
            if let Ok(car) = car_ref.read() {
                Ok(car.clone())
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "car failed to acquire read lock".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "car requires a pair".to_string(),
            None,
        ))),
    }
}

fn primitive_cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("cdr expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    match &args[0] {
        Value::Pair(_, cdr) => Ok((**cdr).clone()),
        Value::MutablePair(_, cdr_ref) => {
            if let Ok(cdr) = cdr_ref.read() {
                Ok(cdr.clone())
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "cdr failed to acquire read lock".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "cdr requires a pair".to_string(),
            None,
        ))),
    }
}

/// list-ref procedure
fn primitive_list_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("list-ref expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let index = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "list-ref index must be a non-negative integer".to_string(),
            None,
        )
    })?;
    
    if index < 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "list-ref index must be non-negative".to_string(),
            None,
        )));
    }
    
    let mut current = &args[0];
    let mut i = 0;
    
    while i < index {
        match current {
            Value::Pair(_, cdr) => {
                current = cdr;
                i += 1;
            }
            Value::MutablePair(_, _) => {
                // For mutable pairs, use as_list for simplicity
                if let Some(list_values) = args[0].as_list() {
                    if index as usize >= list_values.len() {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "list-ref index out of bounds".to_string(),
                            None,
                        )));
                    }
                    return Ok(list_values[index as usize].clone());
                } else {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        "list-ref requires a proper list".to_string(),
                        None,
                    )));
                }
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "list-ref index out of bounds".to_string(),
                    None,
                )));
            }
        }
    }
    
    match current {
        Value::Pair(car, _) => Ok((**car).clone()),
        Value::MutablePair(car_ref, _) => {
            if let Ok(car) = car_ref.read() {
                Ok(car.clone())
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "list-ref failed to acquire read lock".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "list-ref index out of bounds".to_string(),
            None,
        ))),
    }
}

/// list-tail procedure
fn primitive_list_tail(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("list-tail expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let k = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "list-tail k must be a non-negative integer".to_string(),
            None,
        )
    })?;
    
    if k < 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "list-tail k must be non-negative".to_string(),
            None,
        )));
    }
    
    let mut current = args[0].clone();
    let mut i = 0;
    
    while i < k {
        match current {
            Value::Pair(_, cdr) => {
                current = cdr.as_ref().clone();
                i += 1;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "list-tail k out of bounds".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(current)
}

/// length procedure
fn primitive_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("length expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let mut current = &args[0];
    let mut length = 0;
    
    loop {
        match current {
            Value::Nil => break,
            Value::Pair(_, cdr) => {
                current = cdr;
                length += 1;
            }
            Value::MutablePair(_, _) => {
                // For mutable pairs, use as_list for simplicity
                if let Some(list) = args[0].as_list() {
                    return Ok(Value::integer(list.len() as i64));
                } else {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        "length requires a proper list".to_string(),
                        None,
                    )));
                }
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "length requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(Value::integer(length))
}