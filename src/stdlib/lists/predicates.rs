//! List predicates and type checking functions.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use crate::stdlib::lists::common::is_proper_list;
use std::sync::Arc;

/// Binds list predicates.
pub fn bind_list_predicates(env: &Arc<ThreadSafeEnvironment>) {
    // pair?
    env.define("pair?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "pair?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_pair_p),
        effects: vec![Effect::Pure],
    })));
    
    // null?
    env.define("null?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "null?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_null_p),
        effects: vec![Effect::Pure],
    })));
    
    // list?
    env.define("list?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_list_p),
        effects: vec![Effect::Pure],
    })));
}

/// pair? predicate
fn primitive_pair_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("pair? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::boolean(args[0].is_pair()))
}

/// null? predicate
fn primitive_null_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("null? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::boolean(args[0].is_nil()))
}

/// list? predicate
fn primitive_list_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("list? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::boolean(is_proper_list(&args[0])))
}