//! List utilities (member, assoc, sort, etc.)

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use crate::stdlib::lists::common::{values_equal, values_eq, values_eqv};
use std::sync::Arc;

/// Binds list utilities.
pub fn bind_list_utilities(env: &Arc<ThreadSafeEnvironment>) {
    // member
    env.define("member".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "member".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_member),
        effects: vec![Effect::Pure],
    })));
    
    // memq
    env.define("memq".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "memq".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_memq),
        effects: vec![Effect::Pure],
    })));
    
    // memv
    env.define("memv".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "memv".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_memv),
        effects: vec![Effect::Pure],
    })));
    
    // assoc
    env.define("assoc".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "assoc".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_assoc),
        effects: vec![Effect::Pure],
    })));
    
    // assq
    env.define("assq".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "assq".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_assq),
        effects: vec![Effect::Pure],
    })));
    
    // assv
    env.define("assv".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "assv".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_assv),
        effects: vec![Effect::Pure],
    })));
    
    // sort
    env.define("sort".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "sort".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_sort),
        effects: vec![Effect::Pure],
    })));
}

/// member procedure
fn primitive_member(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("member expects 2 or 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let obj = &args[0];
    let mut current = &args[1];
    
    // For now, we'll use default equality (equal?)
    // TODO: Handle custom comparison function when provided
    
    loop {
        match current {
            Value::Nil => return Ok(Value::boolean(false)),
            Value::Pair(car, cdr) => {
                if values_equal(obj, car) {
                    return Ok(current.clone());
                }
                current = cdr;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "member requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
}

/// memq procedure (eq? comparison)
fn primitive_memq(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("memq expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let obj = &args[0];
    let mut current = &args[1];
    
    loop {
        match current {
            Value::Nil => return Ok(Value::boolean(false)),
            Value::Pair(car, cdr) => {
                if values_eq(obj, car) {
                    return Ok(current.clone());
                }
                current = cdr;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "memq requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
}

/// memv procedure (eqv? comparison)
fn primitive_memv(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("memv expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let obj = &args[0];
    let mut current = &args[1];
    
    loop {
        match current {
            Value::Nil => return Ok(Value::boolean(false)),
            Value::Pair(car, cdr) => {
                if values_eqv(obj, car) {
                    return Ok(current.clone());
                }
                current = cdr;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "memv requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
}

/// assoc procedure
fn primitive_assoc(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("assoc expects 2 or 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let obj = &args[0];
    let mut current = &args[1];
    
    loop {
        match current {
            Value::Nil => return Ok(Value::boolean(false)),
            Value::Pair(car, cdr) => {
                match car.as_ref() {
                    Value::Pair(key, _) => {
                        if values_equal(obj, key) {
                            return Ok((**car).clone());
                        }
                    }
                    _ => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "assoc requires a list of pairs".to_string(),
                            None,
                        )));
                    }
                }
                current = cdr;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "assoc requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
}

/// assq procedure (eq? comparison)
fn primitive_assq(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("assq expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let obj = &args[0];
    let mut current = &args[1];
    
    loop {
        match current {
            Value::Nil => return Ok(Value::boolean(false)),
            Value::Pair(car, cdr) => {
                match car.as_ref() {
                    Value::Pair(key, _) => {
                        if values_eq(obj, key) {
                            return Ok((**car).clone());
                        }
                    }
                    _ => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "assq requires a list of pairs".to_string(),
                            None,
                        )));
                    }
                }
                current = cdr;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "assq requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
}

/// assv procedure (eqv? comparison)
fn primitive_assv(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("assv expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let obj = &args[0];
    let mut current = &args[1];
    
    loop {
        match current {
            Value::Nil => return Ok(Value::boolean(false)),
            Value::Pair(car, cdr) => {
                match car.as_ref() {
                    Value::Pair(key, _) => {
                        if values_eqv(obj, key) {
                            return Ok((**car).clone());
                        }
                    }
                    _ => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "assv requires a list of pairs".to_string(),
                            None,
                        )));
                    }
                }
                current = cdr;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "assv requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
}

/// sort procedure
fn primitive_sort(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("sort expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let _less_than = &args[0]; // Comparator function
    let list = &args[1];
    
    // TODO: Implement proper sorting with custom comparator
    // For now, this is a placeholder implementation
    
    if let Some(mut list_values) = list.as_list() {
        // Simple sort that only works with numbers for now
        list_values.sort_by(|a, b| {
            match (a.as_integer(), b.as_integer()) {
                (Some(a_int), Some(b_int)) => a_int.cmp(&b_int),
                _ => std::cmp::Ordering::Equal, // Can't compare non-numbers yet
            }
        });
        
        Ok(Value::list(list_values))
    } else {
        Err(Box::new(DiagnosticError::runtime_error(
            "sort second argument must be a list".to_string(),
            None,
        )))
    }
}