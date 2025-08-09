//! List processing functions for the Lambdust standard library.
//!
//! This module implements R7RS-compliant list operations including
//! list construction, manipulation, higher-order functions, and
//! list predicates.

#![allow(dead_code)]

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use std::sync::Arc;

/// Creates list operation bindings for the standard library.
pub fn create_list_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Basic list operations
    bind_basic_list_operations(env);
    
    // List predicates
    bind_list_predicates(env);
    
    // List accessors
    bind_list_accessors(env);
    
    // List manipulation
    bind_list_manipulation(env);
    
    // Higher-order functions
    bind_higher_order_functions(env);
    
    // List utilities
    bind_list_utilities(env);
    
    // SRFI-1 extensions
    // TODO: Implement SRFI-1 extensions
    // bind_srfi1_extensions(env);
}

/// Binds basic list construction and deconstruction operations.
fn bind_basic_list_operations(env: &Arc<ThreadSafeEnvironment>) {
    // cons
    env.define("cons".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "cons".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_cons),
        effects: vec![Effect::Pure],
    })));
    
    // car
    env.define("car".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "car".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_car),
        effects: vec![Effect::Pure],
    })));
    
    // cdr
    env.define("cdr".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "cdr".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_cdr),
        effects: vec![Effect::Pure],
    })));
    
    // list
    env.define("list".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list".to_string(),
        arity_min: 0,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_list),
        effects: vec![Effect::Pure],
    })));
    
    // list*
    env.define("list*".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list*".to_string(),
        arity_min: 1,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_list_star),
        effects: vec![Effect::Pure],
    })));
    
    // make-list
    env.define("make-list".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-list".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_make_list),
        effects: vec![Effect::Pure],
    })));
}

/// Binds list predicates.
fn bind_list_predicates(env: &Arc<ThreadSafeEnvironment>) {
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

/// Binds list accessor functions.
fn bind_list_accessors(env: &Arc<ThreadSafeEnvironment>) {
    // Combinations of car/cdr
    let combinations = [
        ("caar", "car", "car"),
        ("cadr", "cdr", "car"),
        ("cdar", "car", "cdr"),
        ("cddr", "cdr", "cdr"),
        ("caaar", "caar", "car"),
        ("caadr", "cadr", "car"),
        ("cadar", "cdar", "car"),
        ("caddr", "cddr", "car"),
        ("cdaar", "caar", "cdr"),
        ("cdadr", "cadr", "cdr"),
        ("cddar", "cdar", "cdr"),
        ("cdddr", "cddr", "cdr"),
        ("caaaar", "caaar", "car"),
        ("caaadr", "caadr", "car"),
        ("caadar", "cadar", "car"),
        ("caaddr", "caddr", "car"),
        ("cadaar", "cdaar", "car"),
        ("cadadr", "cdadr", "car"),
        ("caddar", "cddar", "car"),
        ("cadddr", "cdddr", "car"),
        ("cdaaar", "caaar", "cdr"),
        ("cdaadr", "caadr", "cdr"),
        ("cdadar", "cadar", "cdr"),
        ("cdaddr", "caddr", "cdr"),
        ("cddaar", "cdaar", "cdr"),
        ("cddadr", "cdadr", "cdr"),
        ("cdddar", "cddar", "cdr"),
        ("cddddr", "cdddr", "cdr"),
    ];
    
    for (name, _, _) in &combinations {
        env.define(name.to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
            name: name.to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(make_car_cdr_combination(name)),
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
}

/// Binds list manipulation functions.
fn bind_list_manipulation(env: &Arc<ThreadSafeEnvironment>) {
    // length
    env.define("length".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "length".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_length),
        effects: vec![Effect::Pure],
    })));
    
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
    
    // set-car! (mutation)
    env.define("set-car!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "set-car!".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_set_car),
        effects: vec![Effect::State], // Mutation effect
    })));
    
    // set-cdr! (mutation)
    env.define("set-cdr!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "set-cdr!".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_set_cdr),
        effects: vec![Effect::State], // Mutation effect
    })));
    
    // list-set! (mutation)
    env.define("list-set!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list-set!".to_string(),
        arity_min: 3,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_list_set),
        effects: vec![Effect::State], // Mutation effect
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

/// Binds higher-order list functions.
fn bind_higher_order_functions(env: &Arc<ThreadSafeEnvironment>) {
    // map
    env.define("map".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "map".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_map),
        effects: vec![Effect::Pure], // May call user functions with effects
    })));
    
    // for-each
    env.define("for-each".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "for-each".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_for_each),
        effects: vec![Effect::State], // For side effects
    })));
    
    // filter
    env.define("filter".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "filter".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_filter),
        effects: vec![Effect::Pure],
    })));
    
    // fold-left (reduce)
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
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_any),
        effects: vec![Effect::Pure],
    })));
    
    // every (for-all)
    env.define("every".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "every".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_every),
        effects: vec![Effect::Pure],
    })));
}

/// Binds list utility functions.
fn bind_list_utilities(env: &Arc<ThreadSafeEnvironment>) {
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

// ============= BASIC LIST OPERATION IMPLEMENTATIONS =============

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

// ============= LIST PREDICATE IMPLEMENTATIONS =============

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

// ============= LIST ACCESSOR IMPLEMENTATIONS =============

/// Creates a car/cdr combination function
fn make_car_cdr_combination(name: &str) -> fn(&[Value]) -> Result<Value> {
    // This is a simplified implementation
    // In practice, you'd generate these dynamically
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
        // Add more combinations as needed
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

// ============= LIST MANIPULATION IMPLEMENTATIONS =============

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
    
    // Note: This would require mutable pair support in a full implementation
    Err(Box::new(DiagnosticError::runtime_error(
        "set-car! requires mutable pair support (not yet implemented)".to_string(),
        None,
    )))
}

/// set-cdr! procedure (mutation)
fn primitive_set_cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("set-cdr! expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // Note: This would require mutable pair support in a full implementation
    Err(Box::new(DiagnosticError::runtime_error(
        "set-cdr! requires mutable pair support (not yet implemented)".to_string(),
        None,
    )))
}

/// list-set! procedure (mutation)
fn primitive_list_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("list-set! expects 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // Note: This would require mutable pair support in a full implementation
    Err(Box::new(DiagnosticError::runtime_error(
        "list-set! requires mutable pair support (not yet implemented)".to_string(),
        None,
    )))
}

/// list-copy procedure
fn primitive_list_copy(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("list-copy expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    // For now, we'll do a shallow copy by reconstructing the list
    copy_list(&args[0])
}

// ============= HIGHER-ORDER FUNCTION IMPLEMENTATIONS =============

/// map procedure - Enhanced R7RS implementation supporting multiple lists
fn primitive_map(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "map requires at least 2 arguments".to_string(),
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
            "for-each requires at least 2 arguments".to_string(),
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
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "for-each with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                }
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "for-each with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                    None,
                )));
            }
        }
    }
    
    // for-each returns unspecified
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
    let list_arg = &args[1];
    
    // Verify predicate is callable
    if !predicate.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "filter first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    // Convert to proper list
    let list = list_arg.as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "filter requires a proper list".to_string(),
            None,
        )
    })?;
    
    let mut results = Vec::new();
    
    // Apply predicate to each element
    for element in list {
        let proc_args = vec![element.clone()];
        
        // Apply the predicate - for now we can only handle primitive procedures
        let keep = match predicate {
            Value::Primitive(prim) => {
                let result = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                    PrimitiveImpl::Native(func) => func(&proc_args)?,
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "filter with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
                
                // Check if result is truthy
                result.is_truthy()
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "filter with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                    None,
                )));
            }
        };
        
        if keep {
            results.push(element);
        }
    }
    
    Ok(Value::list(results))
}

/// fold-left procedure
fn primitive_fold_left(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "fold-left requires at least 3 arguments".to_string(),
            None,
        )));
    }
    
    let procedure = &args[0];
    let mut accumulator = args[1].clone();
    let lists = &args[2..];
    
    // Verify procedure is callable
    if !procedure.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "fold-left first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    // Convert all list arguments to proper lists and find minimum length
    let mut list_data = Vec::new();
    let mut min_length = usize::MAX;
    
    for (i, list_arg) in lists.iter().enumerate() {
        let list = list_arg.as_list().ok_or_else(|| {
            DiagnosticError::runtime_error(
                format!("fold-left argument {} must be a proper list", i + 3),
                None,
            )
        })?;
        min_length = min_length.min(list.len());
        list_data.push(list);
    }
    
    // If any list is empty, return accumulator immediately
    if min_length == 0 || min_length == usize::MAX {
        return Ok(accumulator);
    }
    
    // Apply procedure to accumulator and each position across all lists
    for i in 0..min_length {
        let mut proc_args = vec![accumulator];
        for list in &list_data {
            proc_args.push(list[i].clone());
        }
        
        // Apply the procedure - for now we can only handle primitive procedures
        match procedure {
            Value::Primitive(prim) => {
                accumulator = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                    PrimitiveImpl::Native(func) => func(&proc_args)?,
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "fold-left with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "fold-left with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(accumulator)
}

/// fold-right procedure
fn primitive_fold_right(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "fold-right requires at least 3 arguments".to_string(),
            None,
        )));
    }
    
    let procedure = &args[0];
    let mut accumulator = args[1].clone();
    let lists = &args[2..];
    
    // Verify procedure is callable
    if !procedure.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "fold-right first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    // Convert all list arguments to proper lists and find minimum length
    let mut list_data = Vec::new();
    let mut min_length = usize::MAX;
    
    for (i, list_arg) in lists.iter().enumerate() {
        let list = list_arg.as_list().ok_or_else(|| {
            DiagnosticError::runtime_error(
                format!("fold-right argument {} must be a proper list", i + 3),
                None,
            )
        })?;
        min_length = min_length.min(list.len());
        list_data.push(list);
    }
    
    // If any list is empty, return accumulator immediately
    if min_length == 0 || min_length == usize::MAX {
        return Ok(accumulator);
    }
    
    // Apply procedure from right to left (reverse order)
    for i in (0..min_length).rev() {
        let mut proc_args = Vec::new();
        for list in &list_data {
            proc_args.push(list[i].clone());
        }
        proc_args.push(accumulator);
        
        // Apply the procedure - for now we can only handle primitive procedures
        match procedure {
            Value::Primitive(prim) => {
                accumulator = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                    PrimitiveImpl::Native(func) => func(&proc_args)?,
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "fold-right with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "fold-right with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(accumulator)
}

/// any procedure
fn primitive_any(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "any requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    // Note: This is a simplified implementation
    // A full implementation would need to handle procedure calls with the evaluator
    Err(Box::new(DiagnosticError::runtime_error(
        "any requires evaluator integration (not yet implemented)".to_string(),
        None,
    )))
}

/// every procedure
fn primitive_every(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "every requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    // Note: This is a simplified implementation
    // A full implementation would need to handle procedure calls with the evaluator
    Err(Box::new(DiagnosticError::runtime_error(
        "every requires evaluator integration (not yet implemented)".to_string(),
        None,
    )))
}

// ============= LIST UTILITY IMPLEMENTATIONS =============

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
    
    // Note: This is a simplified implementation
    // A full implementation would need to handle procedure calls with the evaluator
    Err(Box::new(DiagnosticError::runtime_error(
        "sort requires evaluator integration (not yet implemented)".to_string(),
        None,
    )))
}

// ============= HELPER FUNCTIONS =============

/// Checks if a value is a proper list.
fn is_proper_list(value: &Value) -> bool {
    let mut current = value;
    
    loop {
        match current {
            Value::Nil => return true,
            Value::Pair(_, cdr) => {
                current = cdr;
            }
            _ => return false,
        }
    }
}

/// Copies a list (shallow copy).
fn copy_list(value: &Value) -> Result<Value> {
    match value {
        Value::Nil => Ok(Value::Nil),
        Value::Pair(car, cdr) => {
            let copied_cdr = copy_list(cdr)?;
            Ok(Value::pair((**car).clone(), copied_cdr))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "copy-list requires a list".to_string(),
            None,
        ))),
    }
}

/// Equality comparison functions (placeholders)
fn values_equal(a: &Value, b: &Value) -> bool {
    a == b // Using derived PartialEq for now
}

fn values_eq(a: &Value, b: &Value) -> bool {
    // eq? is stricter than equal? - reference equality for mutable objects
    a == b // Simplified for now
}

fn values_eqv(a: &Value, b: &Value) -> bool {
    // eqv? is between eq? and equal?
    a == b // Simplified for now
}

// ============= SRFI-1 MANIPULATION IMPLEMENTATION =============

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
    
    let list = list_arg.as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "take-while requires a proper list".to_string(),
            None,
        )
    })?;
    
    let mut result = Vec::new();
    
    for element in list {
        let proc_args = vec![element.clone()];
        
        let should_take = match predicate {
            Value::Primitive(prim) => {
                let pred_result = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                    PrimitiveImpl::Native(func) => func(&proc_args)?,
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "take-while with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
                pred_result.is_truthy()
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "take-while with user-defined procedures requires evaluator integration".to_string(),
                    None,
                )));
            }
        };
        
        if should_take {
            result.push(element);
        } else {
            break;
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
    let mut current = &args[1];
    
    if !predicate.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "drop-while first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    loop {
        match current {
            Value::Nil => return Ok(Value::Nil),
            Value::Pair(car, cdr) => {
                let proc_args = vec![(**car).clone()];
                
                let should_drop = match predicate {
                    Value::Primitive(prim) => {
                        let pred_result = match &prim.implementation {
                            PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                        PrimitiveImpl::Native(func) => func(&proc_args)?,
                            PrimitiveImpl::ForeignFn { .. } => {
                                return Err(Box::new(DiagnosticError::runtime_error(
                                    "drop-while with foreign functions not yet implemented".to_string(),
                                    None,
                                )));
                            }
                        };
                        pred_result.is_truthy()
                    },
                    _ => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "drop-while with user-defined procedures requires evaluator integration".to_string(),
                            None,
                        )));
                    }
                };
                
                if should_drop {
                    current = cdr;
                } else {
                    return Ok(current.clone());
                }
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "drop-while requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
}

/// split-at - Split a list at position n
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
    
    let list = list_arg.as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "split-at requires a proper list".to_string(),
            None,
        )
    })?;
    
    let n_usize = n as usize;
    if n_usize > list.len() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "split-at: index out of bounds".to_string(),
            None,
        )));
    }
    
    let (prefix, suffix) = list.split_at(n_usize);
    Ok(Value::pair(
        Value::list(prefix.to_vec()),
        Value::list(suffix.to_vec()),
    ))
}

/// last - Get the last element of a list
fn srfi1_last(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("last expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let list = args[0].as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "last requires a proper list".to_string(),
            None,
        )
    })?;
    
    if list.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "last: empty list".to_string(),
            None,
        )));
    }
    
    Ok(list[list.len() - 1].clone())
}

/// last-pair - Get the last pair of a list
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
                let last_pair = current.clone();
                if cdr.is_nil() {
                    return Ok(last_pair);
                }
                current = cdr;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "last-pair requires a list".to_string(),
                    None,
                )));
            }
        }
    }
}

// ============= SRFI-1 FOLDING IMPLEMENTATION =============

/// fold - SRFI-1 version of fold (different argument order)
/// (fold kons knil clist1 clist2 ...)
fn srfi1_fold(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "fold requires at least 3 arguments".to_string(),
            None,
        )));
    }
    
    let procedure = &args[0];
    let mut accumulator = args[1].clone();
    let lists = &args[2..];
    
    if !procedure.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "fold first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    // Convert all list arguments to proper lists and find minimum length
    let mut list_data = Vec::new();
    let mut min_length = usize::MAX;
    
    for (i, list_arg) in lists.iter().enumerate() {
        let list = list_arg.as_list().ok_or_else(|| {
            DiagnosticError::runtime_error(
                format!("fold argument {} must be a proper list", i + 3),
                None,
            )
        })?;
        min_length = min_length.min(list.len());
        list_data.push(list);
    }
    
    if min_length == 0 || min_length == usize::MAX {
        return Ok(accumulator);
    }
    
    // Apply procedure to accumulator and each position across all lists
    for i in 0..min_length {
        let mut proc_args = vec![accumulator];
        for list in &list_data {
            proc_args.push(list[i].clone());
        }
        
        match procedure {
            Value::Primitive(prim) => {
                accumulator = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                    PrimitiveImpl::Native(func) => func(&proc_args)?,
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "fold with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "fold with user-defined procedures requires evaluator integration".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(accumulator)
}

/// fold-right - SRFI-1 version of fold-right
fn srfi1_fold_right(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "fold-right requires at least 3 arguments".to_string(),
            None,
        )));
    }
    
    let procedure = &args[0];
    let mut accumulator = args[1].clone();
    let lists = &args[2..];
    
    if !procedure.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "fold-right first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    // Convert all list arguments to proper lists and find minimum length
    let mut list_data = Vec::new();
    let mut min_length = usize::MAX;
    
    for (i, list_arg) in lists.iter().enumerate() {
        let list = list_arg.as_list().ok_or_else(|| {
            DiagnosticError::runtime_error(
                format!("fold-right argument {} must be a proper list", i + 3),
                None,
            )
        })?;
        min_length = min_length.min(list.len());
        list_data.push(list);
    }
    
    if min_length == 0 || min_length == usize::MAX {
        return Ok(accumulator);
    }
    
    // Apply procedure from right to left (reverse order)
    for i in (0..min_length).rev() {
        let mut proc_args = Vec::new();
        for list in &list_data {
            proc_args.push(list[i].clone());
        }
        proc_args.push(accumulator);
        
        match procedure {
            Value::Primitive(prim) => {
                accumulator = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                    PrimitiveImpl::Native(func) => func(&proc_args)?,
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "fold-right with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "fold-right with user-defined procedures requires evaluator integration".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(accumulator)
}

/// reduce - Reduce a list using a binary operation
fn srfi1_reduce(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("reduce expects 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let procedure = &args[0];
    let ridentity = &args[1];
    let list_arg = &args[2];
    
    if !procedure.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "reduce first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    let list = list_arg.as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "reduce requires a proper list".to_string(),
            None,
        )
    })?;
    
    if list.is_empty() {
        return Ok(ridentity.clone());
    }
    
    if list.len() == 1 {
        return Ok(list[0].clone());
    }
    
    let mut accumulator = list[0].clone();
    
    for element in &list[1..] {
        let proc_args = vec![accumulator, element.clone()];
        
        match procedure {
            Value::Primitive(prim) => {
                accumulator = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                    PrimitiveImpl::Native(func) => func(&proc_args)?,
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "reduce with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "reduce with user-defined procedures requires evaluator integration".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(accumulator)
}

/// reduce-right - Reduce a list from right to left
fn srfi1_reduce_right(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("reduce-right expects 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let procedure = &args[0];
    let ridentity = &args[1];
    let list_arg = &args[2];
    
    if !procedure.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "reduce-right first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    let list = list_arg.as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "reduce-right requires a proper list".to_string(),
            None,
        )
    })?;
    
    if list.is_empty() {
        return Ok(ridentity.clone());
    }
    
    if list.len() == 1 {
        return Ok(list[0].clone());
    }
    
    let mut accumulator = list[list.len() - 1].clone();
    
    for element in list[..list.len() - 1].iter().rev() {
        let proc_args = vec![element.clone(), accumulator];
        
        match procedure {
            Value::Primitive(prim) => {
                accumulator = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                    PrimitiveImpl::Native(func) => func(&proc_args)?,
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "reduce-right with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "reduce-right with user-defined procedures requires evaluator integration".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(accumulator)
}

/// unfold - Generate a list by repeatedly applying functions
fn srfi1_unfold(args: &[Value]) -> Result<Value> {
    if args.len() < 4 || args.len() > 5 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("unfold expects 4 or 5 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // For now, return not implemented
    Err(Box::new(DiagnosticError::runtime_error(
        "unfold requires evaluator integration (not yet implemented)".to_string(),
        None,
    )))
}

/// unfold-right - Generate a list in reverse by repeatedly applying functions
fn srfi1_unfold_right(args: &[Value]) -> Result<Value> {
    if args.len() < 4 || args.len() > 5 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("unfold-right expects 4 or 5 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // For now, return not implemented
    Err(Box::new(DiagnosticError::runtime_error(
        "unfold-right requires evaluator integration (not yet implemented)".to_string(),
        None,
    )))
}

// ============= SRFI-1 ASSOCIATION IMPLEMENTATION =============

/// alist-cons - Add a key-value pair to an association list
fn srfi1_alist_cons(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("alist-cons expects 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let key = &args[0];
    let datum = &args[1];
    let alist = &args[2];
    
    let pair = Value::pair(key.clone(), datum.clone());
    Ok(Value::pair(pair, alist.clone()))
}

/// alist-copy - Copy an association list
fn srfi1_alist_copy(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("alist-copy expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    copy_alist(&args[0])
}

/// alist-delete - Delete entries with a given key from an association list
fn srfi1_alist_delete(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("alist-delete expects 2 or 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let key = &args[0];
    let alist = &args[1];
    // TODO: Handle custom equality predicate when provided
    
    let mut result = Vec::new();
    let mut current = alist;
    
    loop {
        match current {
            Value::Nil => break,
            Value::Pair(car, cdr) => {
                match car.as_ref() {
                    Value::Pair(entry_key, entry_value) => {
                        if !values_equal(key, entry_key) {
                            let entry = Value::pair((**entry_key).clone(), (**entry_value).clone());
                            result.push(entry);
                        }
                    }
                    _ => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "alist-delete requires a list of pairs".to_string(),
                            None,
                        )));
                    }
                }
                current = cdr;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "alist-delete requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(Value::list(result))
}

// ============= SRFI-1 COMPARISON IMPLEMENTATION =============

/// list= - Test if lists are equal element-wise
fn srfi1_list_equal(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "list= requires at least 1 argument".to_string(),
            None,
        )));
    }
    
    if args.len() == 1 {
        return Ok(Value::boolean(true));
    }
    
    // For now, use the first argument as the equality predicate if it's a procedure
    // Otherwise, use default equality
    let (eq_pred, lists) = if args[0].is_procedure() {
        (&args[0], &args[1..])
    } else {
        // No custom equality predicate, use default
        return Ok(Value::boolean(lists_equal_default(&args[0], &args[1..])));
    };
    
    if lists.len() < 2 {
        return Ok(Value::boolean(true));
    }
    
    // Convert all to proper lists
    let mut list_data = Vec::new();
    for (i, list_arg) in lists.iter().enumerate() {
        let list = list_arg.as_list().ok_or_else(|| {
            DiagnosticError::runtime_error(
                format!("list= argument {} must be a proper list", i + 2),
                None,
            )
        })?;
        list_data.push(list);
    }
    
    // Check that all lists have the same length
    let first_len = list_data[0].len();
    for list in &list_data[1..] {
        if list.len() != first_len {
            return Ok(Value::boolean(false));
        }
    }
    
    // Compare elements pairwise using the equality predicate
    for i in 0..first_len {
        for j in 1..list_data.len() {
            let proc_args = vec![list_data[0][i].clone(), list_data[j][i].clone()];
            
            let are_equal = match eq_pred {
                Value::Primitive(prim) => {
                    let result = match &prim.implementation {
                        PrimitiveImpl::RustFn(func) => func(&proc_args)?,
                        PrimitiveImpl::Native(func) => func(&proc_args)?,
                        PrimitiveImpl::ForeignFn { .. } => {
                            return Err(Box::new(DiagnosticError::runtime_error(
                                "list= with foreign functions not yet implemented".to_string(),
                                None,
                            )));
                        }
                    };
                    result.is_truthy()
                },
                _ => {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        "list= with user-defined procedures requires evaluator integration".to_string(),
                        None,
                    )));
                }
            };
            
            if !are_equal {
                return Ok(Value::boolean(false));
            }
        }
    }
    
    Ok(Value::boolean(true))
}

// ============= SRFI-1 HELPER FUNCTIONS =============

/// Check if a value is a circular list (using Floyd's cycle detection)
fn is_circular_list(value: &Value) -> bool {
    let mut slow = value;
    let mut fast = value;
    
    loop {
        // Move fast pointer two steps
        match fast {
            Value::Pair(_, cdr1) => {
                match cdr1.as_ref() {
                    Value::Pair(_, cdr2) => {
                        fast = cdr2;
                    }
                    Value::Nil => return false,
                    _ => return false, // Not a proper list structure
                }
            }
            Value::Nil => return false,
            _ => return false,
        }
        
        // Move slow pointer one step
        match slow {
            Value::Pair(_, cdr) => {
                slow = cdr;
            }
            Value::Nil => return false,
            _ => return false,
        }
        
        // Check if they meet (cycle detected)
        if std::ptr::eq(slow, fast) {
            return true;
        }
    }
}

/// Check if a value is a dotted (improper) list
fn is_dotted_list(value: &Value) -> bool {
    let mut current = value;
    
    loop {
        match current {
            Value::Nil => return false, // Proper list
            Value::Pair(_, cdr) => {
                current = cdr;
            }
            _ => return true, // Improper list (dotted)
        }
    }
}

/// Copy an association list (deep copy of pairs)
fn copy_alist(alist: &Value) -> Result<Value> {
    let mut result = Vec::new();
    let mut current = alist;
    
    loop {
        match current {
            Value::Nil => break,
            Value::Pair(car, cdr) => {
                match car.as_ref() {
                    Value::Pair(key, value) => {
                        let copied_pair = Value::pair((**key).clone(), (**value).clone());
                        result.push(copied_pair);
                    }
                    _ => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "alist-copy requires a list of pairs".to_string(),
                            None,
                        )));
                    }
                }
                current = cdr;
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "alist-copy requires a proper list".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(Value::list(result))
}

/// Check if multiple lists are equal using default equality
fn lists_equal_default(first: &Value, rest: &[Value]) -> bool {
    let first_list = match first.as_list() {
        Some(list) => list,
        None => return false,
    };
    
    for other in rest {
        let other_list = match other.as_list() {
            Some(list) => list,
            None => return false,
        };
        
        if first_list.len() != other_list.len() {
            return false;
        }
        
        for (a, b) in first_list.iter().zip(other_list.iter()) {
            if !values_equal(a, b) {
                return false;
            }
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_cons_car_cdr() {
        let args = vec![Value::integer(1), Value::integer(2)];
        let pair = primitive_cons(&args).unwrap();
        
        let car_result = primitive_car(&[pair.clone()]).unwrap();
        assert_eq!(car_result, Value::integer(1));
        
        let cdr_result = primitive_cdr(&[pair]).unwrap();
        assert_eq!(cdr_result, Value::integer(2));
    }
    
    #[test]
    fn test_list_construction() {
        let args = vec![Value::integer(1), Value::integer(2), Value::integer(3)];
        let list = primitive_list(&args).unwrap();
        
        // Test that it's a proper list
        assert!(is_proper_list(&list));
        
        // Test list length
        let length = primitive_length(&[list]).unwrap();
        assert_eq!(length, Value::integer(3));
    }
    
    #[test]
    fn test_list_predicates() {
        let pair = Value::pair(Value::integer(1), Value::integer(2));
        assert_eq!(primitive_pair_p(&[pair]).unwrap(), Value::boolean(true));
        
        let nil = Value::Nil;
        assert_eq!(primitive_null_p(&[nil]).unwrap(), Value::boolean(true));
        
        let proper_list = Value::list(vec![Value::integer(1), Value::integer(2)]);
        assert_eq!(primitive_list_p(&[proper_list]).unwrap(), Value::boolean(true));
    }
    
    #[test]
    fn test_list_ref() {
        let list = Value::list(vec![
            Value::string("a"),
            Value::string("b"),
            Value::string("c"),
        ]);
        
        let args = vec![list, Value::integer(1)];
        let result = primitive_list_ref(&args).unwrap();
        assert_eq!(result, Value::string("b"));
    }
    
    #[test]
    fn test_append() {
        let list1 = Value::list(vec![Value::integer(1), Value::integer(2)]);
        let list2 = Value::list(vec![Value::integer(3), Value::integer(4)]);
        
        let args = vec![list1, list2];
        let result = primitive_append(&args).unwrap();
        
        // Result should be (1 2 3 4)
        let expected = Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
            Value::integer(4),
        ]);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_reverse() {
        let list = Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]);
        
        let result = primitive_reverse(&[list]).unwrap();
        let expected = Value::list(vec![
            Value::integer(3),
            Value::integer(2),
            Value::integer(1),
        ]);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_member() {
        let list = Value::list(vec![
            Value::string("a"),
            Value::string("b"),
            Value::string("c"),
        ]);
        
        let args = vec![Value::string("b"), list];
        let result = primitive_member(&args).unwrap();
        
        // Should return the tail starting with "b"
        assert!(result.is_pair());
    }
    
    #[test]
    fn test_map_single_list() {
        // Test map with a simple procedure that doubles numbers
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
        
        let list = Value::list(vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)]);
        let args = vec![Value::Primitive(double_proc), list];
        let result = primitive_map(&args).unwrap();
        
        let expected = Value::list(vec![Value::number(2.0), Value::number(4.0), Value::number(6.0)]);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_map_multiple_lists() {
        // Test map with multiple lists
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
        
        let list1 = Value::list(vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)]);
        let list2 = Value::list(vec![Value::number(4.0), Value::number(5.0), Value::number(6.0)]);
        let args = vec![Value::Primitive(add_proc), list1, list2];
        let result = primitive_map(&args).unwrap();
        
        let expected = Value::list(vec![Value::number(5.0), Value::number(7.0), Value::number(9.0)]);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_map_different_lengths() {
        // Test map with lists of different lengths - should use shortest
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
        
        let list1 = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        let list2 = Value::list(vec![Value::number(4.0), Value::number(5.0), Value::number(6.0)]);
        let args = vec![Value::Primitive(add_proc), list1, list2];
        let result = primitive_map(&args).unwrap();
        
        let expected = Value::list(vec![Value::number(5.0), Value::number(7.0)]);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_map_empty_list() {
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
        
        let empty_list = Value::Nil;
        let args = vec![Value::Primitive(double_proc), empty_list];
        let result = primitive_map(&args).unwrap();
        
        assert_eq!(result, Value::Nil);
    }
    
    #[test]
    fn test_for_each_basic() {
        // Test for-each with a simple side-effect procedure
        let identity_proc = Arc::new(PrimitiveProcedure {
            name: "identity".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| Ok(args[0].clone())),
            effects: vec![Effect::Pure],
        });
        
        let list = Value::list(vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)]);
        let args = vec![Value::Primitive(identity_proc), list];
        let result = primitive_for_each(&args).unwrap();
        
        // for-each should return unspecified
        assert_eq!(result, Value::Unspecified);
    }
    
    #[test]
    fn test_for_each_multiple_lists() {
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
        
        let list1 = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        let list2 = Value::list(vec![Value::number(4.0), Value::number(5.0)]);
        let args = vec![Value::Primitive(add_proc), list1, list2];
        let result = primitive_for_each(&args).unwrap();
        
        assert_eq!(result, Value::Unspecified);
    }
    
    #[test]
    fn test_map_for_each_errors() {
        // Test errors for both map and for-each
        
        // Non-procedure first argument
        let args = vec![Value::integer(42), Value::list(vec![Value::integer(1)])];
        assert!(primitive_map(&args).is_err());
        assert!(primitive_for_each(&args).is_err());
        
        // Non-list argument
        let proc = Arc::new(PrimitiveProcedure {
            name: "test".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| Ok(args[0].clone())),
            effects: vec![Effect::Pure],
        });
        let args = vec![Value::Primitive(proc.clone()), Value::integer(42)];
        assert!(primitive_map(&args).is_err());
        assert!(primitive_for_each(&args).is_err());
        
        // Too few arguments
        assert!(primitive_map(&[]).is_err());
        assert!(primitive_for_each(&[]).is_err());
        
        let args = vec![Value::Primitive(proc)];
        assert!(primitive_map(&args).is_err());
        assert!(primitive_for_each(&args).is_err());
    }
    
    #[test]
    fn test_filter_basic() {
        // Test filter with a predicate that keeps even numbers
        let even_pred = Arc::new(PrimitiveProcedure {
            name: "even?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| {
                if let Some(n) = args[0].as_integer() {
                    Ok(Value::boolean(n % 2 == 0))
                } else {
                    Ok(Value::boolean(false))
                }
            }),
            effects: vec![Effect::Pure],
        });
        
        let list = Value::list(vec![
            Value::integer(1), Value::integer(2), Value::integer(3), 
            Value::integer(4), Value::integer(5), Value::integer(6)
        ]);
        let args = vec![Value::Primitive(even_pred), list];
        let result = primitive_filter(&args).unwrap();
        
        let expected = Value::list(vec![Value::integer(2), Value::integer(4), Value::integer(6)]);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_filter_empty_result() {
        // Test filter with a predicate that keeps nothing
        let false_pred = Arc::new(PrimitiveProcedure {
            name: "false".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|_args| Ok(Value::boolean(false))),
            effects: vec![Effect::Pure],
        });
        
        let list = Value::list(vec![Value::integer(1), Value::integer(2), Value::integer(3)]);
        let args = vec![Value::Primitive(false_pred), list];
        let result = primitive_filter(&args).unwrap();
        
        assert_eq!(result, Value::Nil);
    }
    
    #[test]
    fn test_filter_all_kept() {
        // Test filter with a predicate that keeps everything
        let true_pred = Arc::new(PrimitiveProcedure {
            name: "true".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|_args| Ok(Value::boolean(true))),
            effects: vec![Effect::Pure],
        });
        
        let list = Value::list(vec![Value::integer(1), Value::integer(2), Value::integer(3)]);
        let args = vec![Value::Primitive(true_pred), list.clone()];
        let result = primitive_filter(&args).unwrap();
        
        assert_eq!(result, list);
    }
    
    #[test]
    fn test_fold_left_basic() {
        // Test fold-left with addition
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
        
        let list = Value::list(vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)]);
        let args = vec![Value::Primitive(add_proc), Value::number(0.0), list];
        let result = primitive_fold_left(&args).unwrap();
        
        assert_eq!(result, Value::number(6.0));
    }
    
    #[test]
    fn test_fold_left_with_strings() {
        // Test fold-left with string concatenation
        let concat_proc = Arc::new(PrimitiveProcedure {
            name: "string-append".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(|args| {
                let mut result = String::new();
                for arg in args {
                    if let Some(s) = arg.as_string() {
                        result.push_str(s);
                    }
                }
                Ok(Value::string(result))
            }),
            effects: vec![Effect::Pure],
        });
        
        let list = Value::list(vec![Value::string("a"), Value::string("b"), Value::string("c")]);
        let args = vec![Value::Primitive(concat_proc), Value::string(""), list];
        let result = primitive_fold_left(&args).unwrap();
        
        assert_eq!(result, Value::string("abc"));
    }
    
    #[test]
    fn test_fold_right_basic() {
        // Test fold-right with subtraction to show order difference
        let sub_proc = Arc::new(PrimitiveProcedure {
            name: "-".to_string(),
            arity_min: 1,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(|args| {
                if args.is_empty() {
                    return Ok(Value::number(0.0));
                }
                
                let first = args[0].as_number().unwrap_or(0.0);
                if args.len() == 1 {
                    return Ok(Value::number(-first));
                }
                
                let result = args[1..].iter()
                    .filter_map(|v| v.as_number())
                    .fold(first, |acc, n| acc - n);
                Ok(Value::number(result))
            }),
            effects: vec![Effect::Pure],
        });
        
        let list = Value::list(vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)]);
        let args = vec![Value::Primitive(sub_proc), Value::number(0.0), list];
        let result = primitive_fold_right(&args).unwrap();
        
        // fold-right with - should compute (1 - (2 - (3 - 0))) = 1 - (2 - 3) = 1 - (-1) = 2
        assert_eq!(result, Value::number(2.0));
    }
    
    #[test]
    fn test_fold_empty_list() {
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
        
        let empty_list = Value::Nil;
        let initial = Value::number(42.0);
        
        let args = vec![Value::Primitive(add_proc.clone()), initial.clone(), empty_list.clone()];
        let result_left = primitive_fold_left(&args).unwrap();
        assert_eq!(result_left, initial);
        
        let args = vec![Value::Primitive(add_proc), initial.clone(), empty_list];
        let result_right = primitive_fold_right(&args).unwrap();
        assert_eq!(result_right, initial);
    }
    
    #[test]
    fn test_higher_order_errors() {
        // Test errors for filter and folds
        
        // Non-procedure first argument
        let args = vec![Value::integer(42), Value::list(vec![Value::integer(1)])];
        assert!(primitive_filter(&args).is_err());
        
        let args = vec![Value::integer(42), Value::integer(0), Value::list(vec![Value::integer(1)])];
        assert!(primitive_fold_left(&args).is_err());
        assert!(primitive_fold_right(&args).is_err());
        
        // Non-list argument
        let proc = Arc::new(PrimitiveProcedure {
            name: "test".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| Ok(args[0].clone())),
            effects: vec![Effect::Pure],
        });
        
        let args = vec![Value::Primitive(proc.clone()), Value::integer(42)];
        assert!(primitive_filter(&args).is_err());
        
        let args = vec![Value::Primitive(proc.clone()), Value::integer(0), Value::integer(42)];
        assert!(primitive_fold_left(&args).is_err());
        assert!(primitive_fold_right(&args).is_err());
        
        // Too few arguments
        assert!(primitive_filter(&[]).is_err());
        assert!(primitive_fold_left(&[]).is_err());
        assert!(primitive_fold_right(&[]).is_err());
        
        let args = vec![Value::Primitive(proc.clone())];
        assert!(primitive_filter(&args).is_err());
        
        let args = vec![Value::Primitive(proc.clone()), Value::integer(0)];
        assert!(primitive_fold_left(&args).is_err());
        assert!(primitive_fold_right(&args).is_err());
    }
    
    // ============= SRFI-1 SPECIFIC TESTS =============
    
    #[test]
    fn test_srfi1_take_drop() {
        let list = Value::list(vec![Value::integer(1), Value::integer(2), Value::integer(3), Value::integer(4), Value::integer(5)]);
        
        // Test take
        let args = vec![list.clone(), Value::integer(3)];
        let result = srfi1_take(&args).unwrap();
        let expected = Value::list(vec![Value::integer(1), Value::integer(2), Value::integer(3)]);
        assert_eq!(result, expected);
        
        // Test drop
        let args = vec![list.clone(), Value::integer(2)];
        let result = srfi1_drop(&args).unwrap();
        let expected = Value::list(vec![Value::integer(3), Value::integer(4), Value::integer(5)]);
        assert_eq!(result, expected);
        
        // Test take-right
        let args = vec![list.clone(), Value::integer(2)];
        let result = srfi1_take_right(&args).unwrap();
        let expected = Value::list(vec![Value::integer(4), Value::integer(5)]);
        assert_eq!(result, expected);
        
        // Test drop-right
        let args = vec![list, Value::integer(2)];
        let result = srfi1_drop_right(&args).unwrap();
        let expected = Value::list(vec![Value::integer(1), Value::integer(2), Value::integer(3)]);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_srfi1_split_at() {
        let list = Value::list(vec![Value::integer(1), Value::integer(2), Value::integer(3), Value::integer(4)]);
        let args = vec![list, Value::integer(2)];
        let result = srfi1_split_at(&args).unwrap();
        
        match result {
            Value::Pair(prefix, suffix) => {
                let expected_prefix = Value::list(vec![Value::integer(1), Value::integer(2)]);
                let expected_suffix = Value::list(vec![Value::integer(3), Value::integer(4)]);
                assert_eq!(prefix.as_ref().clone(), expected_prefix);
                assert_eq!(suffix.as_ref().clone(), expected_suffix);
            },
            _ => panic!("split-at should return a pair"),
        }
    }
    
    #[test]
    fn test_srfi1_last_last_pair() {
        let list = Value::list(vec![Value::integer(1), Value::integer(2), Value::integer(3)]);
        
        // Test last
        let result = srfi1_last(&[list.clone()]).unwrap();
        assert_eq!(result, Value::integer(3));
        
        // Test last-pair
        let result = srfi1_last_pair(&[list]).unwrap();
        // Should return the pair (3 . ())
        match result {
            Value::Pair(car, cdr) => {
                assert_eq!(car.as_ref().clone(), Value::integer(3));
                assert_eq!(cdr.as_ref().clone(), Value::Nil);
            },
            _ => panic!("last-pair should return a pair"),
        }
    }
    
    #[test]
    fn test_srfi1_fold_reduce() {
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
        
        let list = Value::list(vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)]);
        
        // Test fold (SRFI-1 version: kons knil clist)
        let args = vec![Value::Primitive(add_proc.clone()), Value::number(0.0), list.clone()];
        let result = srfi1_fold(&args).unwrap();
        assert_eq!(result, Value::number(6.0));
        
        // Test reduce  
        let args = vec![Value::Primitive(add_proc), Value::number(0.0), list];
        let result = srfi1_reduce(&args).unwrap();
        assert_eq!(result, Value::number(6.0));
    }
    
    #[test]
    fn test_srfi1_alist_operations() {
        // Create an association list: ((a . 1) (b . 2) (c . 3))
        let alist = Value::list(vec![
            Value::pair(Value::string("a"), Value::integer(1)),
            Value::pair(Value::string("b"), Value::integer(2)),
            Value::pair(Value::string("c"), Value::integer(3)),
        ]);
        
        // Test alist-cons
        let args = vec![Value::string("d"), Value::integer(4), alist.clone()];
        let result = srfi1_alist_cons(&args).unwrap();
        
        // Should add (d . 4) to the front
        match result {
            Value::Pair(car, cdr) => {
                match car.as_ref() {
                    Value::Pair(key, value) => {
                        assert_eq!(key.as_ref().clone(), Value::string("d"));
                        assert_eq!(value.as_ref().clone(), Value::integer(4));
                    },
                    _ => panic!("Expected pair in alist-cons result"),
                }
                assert_eq!(cdr.as_ref().clone(), alist);
            },
            _ => panic!("alist-cons should return a pair"),
        }
        
        // Test alist-copy
        let result = srfi1_alist_copy(&[alist.clone()]).unwrap();
        // Should be structurally equal but not the same object
        if let (Some(orig_list), Some(copied_list)) = (alist.as_list(), result.as_list()) {
            assert_eq!(orig_list.len(), copied_list.len());
            for (orig, copied) in orig_list.iter().zip(copied_list.iter()) {
                assert_eq!(orig, copied);
            }
        } else {
            panic!("Both should be proper lists");
        }
        
        // Test alist-delete
        let args = vec![Value::string("b"), alist];
        let result = srfi1_alist_delete(&args).unwrap();
        let expected = Value::list(vec![
            Value::pair(Value::string("a"), Value::integer(1)),
            Value::pair(Value::string("c"), Value::integer(3)),
        ]);
        assert_eq!(result, expected);
    }
}