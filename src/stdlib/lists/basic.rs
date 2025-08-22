//! Basic list construction and deconstruction operations.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use std::sync::Arc;

/// Binds basic list construction and deconstruction operations.
pub fn bind_basic_list_operations(env: &Arc<ThreadSafeEnvironment>) {
    // cons - only define if not already present (preserves bootstrap primitives)
    if env.lookup("cons").is_none() {
        env.define(
            "cons".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "cons".to_string(),
                arity_min: 2,
                arity_max: Some(2),
                implementation: PrimitiveImpl::RustFn(primitive_cons),
                effects: vec![Effect::Pure],
            })),
        );
    }

    // car - only define if not already present (preserves bootstrap primitives)
    if env.lookup("car").is_none() {
        env.define(
            "car".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "car".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::RustFn(primitive_car),
                effects: vec![Effect::Pure],
            })),
        );
    }

    // cdr - only define if not already present (preserves bootstrap primitives)
    if env.lookup("cdr").is_none() {
        env.define(
            "cdr".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "cdr".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::RustFn(primitive_cdr),
                effects: vec![Effect::Pure],
            })),
        );
    }

    // list
    env.define(
        "list".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "list".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(primitive_list),
            effects: vec![Effect::Pure],
        })),
    );

    // list*
    env.define(
        "list*".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "list*".to_string(),
            arity_min: 1,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(primitive_list_star),
            effects: vec![Effect::Pure],
        })),
    );

    // make-list
    env.define(
        "make-list".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "make-list".to_string(),
            arity_min: 1,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(primitive_make_list),
            effects: vec![Effect::Pure],
        })),
    );
}

/// cons procedure
fn primitive_cons(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("cons expects 2 arguments, got {}", args.len()),
            None,
        )));
    }

    Ok(Value::pair(args[0].clone(), args[1].clone()))
}

/// car procedure
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
            if let Ok(car) = car_ref.try_borrow() {
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

/// cdr procedure
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
            if let Ok(cdr) = cdr_ref.try_borrow() {
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

/// list constructor
fn primitive_list(args: &[Value]) -> Result<Value> {
    Ok(Value::list(args.to_vec()))
}

/// list* procedure (improper list constructor)
fn primitive_list_star(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "list* requires at least 1 argument".to_string(),
            None,
        )));
    }

    if args.len() == 1 {
        return Ok(args[0].clone());
    }

    let mut result = args[args.len() - 1].clone();

    for arg in args[..args.len() - 1].iter().rev() {
        result = Value::pair(arg.clone(), result);
    }

    Ok(result)
}

/// make-list procedure
fn primitive_make_list(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("make-list expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let length = args[0].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "make-list first argument must be a non-negative integer".to_string(),
            None,
        )
    })?;

    if length < 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "make-list length must be non-negative".to_string(),
            None,
        )));
    }

    let fill = if args.len() == 2 {
        args[1].clone()
    } else {
        Value::Unspecified
    };

    let elements = vec![fill; length as usize];
    Ok(Value::list(elements))
}
