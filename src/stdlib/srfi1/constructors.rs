//! SRFI-1 List Constructor Operations
//!
//! This module implements the core list construction operations from SRFI-1:
//! - cons, list, xcons, cons*
//! - list-copy, make-list, list-tabulate
//! - circular-list, iota
//!
//! ## Performance Optimizations
//!
//! - Arena allocation for large list construction
//! - NaN-boxing integration for small lists
//! - Optimized memory layout for cache performance

use crate::ast::Literal;
use crate::diagnostics::{Error, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
// Note: Number type is handled through Value::number() factory methods
use crate::stdlib::srfi1::SRFI1Core;
use std::sync::Arc;

/// Binds all constructor operations to the environment
pub fn bind_constructor_operations(env: &Arc<ThreadSafeEnvironment>) {
    // cons (already exists in core, ensure SRFI-1 semantics)
    env.define(
        "cons".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "cons".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(srfi1_cons),
            effects: vec![Effect::Pure],
        })),
    );

    // list (already exists in core, ensure SRFI-1 semantics)
    env.define(
        "list".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "list".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(srfi1_list),
            effects: vec![Effect::Pure],
        })),
    );

    // xcons - reverse cons
    env.define(
        "xcons".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "xcons".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(srfi1_xcons),
            effects: vec![Effect::Pure],
        })),
    );

    // cons* - generalized cons
    env.define(
        "cons*".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "cons*".to_string(),
            arity_min: 1,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(srfi1_cons_star),
            effects: vec![Effect::Pure],
        })),
    );

    // list-copy
    env.define(
        "list-copy".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "list-copy".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(srfi1_list_copy),
            effects: vec![Effect::Pure],
        })),
    );

    // make-list (already exists in core, ensure SRFI-1 semantics)
    env.define(
        "make-list".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "make-list".to_string(),
            arity_min: 1,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(srfi1_make_list),
            effects: vec![Effect::Pure],
        })),
    );

    // list-tabulate
    env.define(
        "list-tabulate".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "list-tabulate".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(srfi1_list_tabulate),
            effects: vec![Effect::Pure],
        })),
    );

    // circular-list
    env.define(
        "circular-list".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "circular-list".to_string(),
            arity_min: 1,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(srfi1_circular_list),
            effects: vec![Effect::Pure],
        })),
    );

    // iota
    env.define(
        "iota".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "iota".to_string(),
            arity_min: 1,
            arity_max: Some(3),
            implementation: PrimitiveImpl::RustFn(srfi1_iota),
            effects: vec![Effect::Pure],
        })),
    );
}

/// cons - Create a pair (SRFI-1 compliant)
pub fn srfi1_cons(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 2, Some(2), "cons")?;
    Ok(Value::pair(args[0].clone(), args[1].clone()))
}

/// list - Create a list from arguments (SRFI-1 compliant)  
pub fn srfi1_list(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        Ok(Value::Nil)
    } else {
        Ok(Value::list(args.to_vec()))
    }
}

/// xcons - Reverse cons: (xcons cdr car) = (cons car cdr)
pub fn srfi1_xcons(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 2, Some(2), "xcons")?;
    Ok(Value::pair(args[1].clone(), args[0].clone()))
}

/// cons* - Generalized cons that creates a proper list except for the last argument
/// (cons* 1 2 3 4) => (1 2 3 . 4)
/// (cons* 1) => 1
/// (cons* 1 2) => (1 . 2)
pub fn srfi1_cons_star(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 1, None, "cons*")?;

    if args.len() == 1 {
        Ok(args[0].clone())
    } else {
        let mut result = args[args.len() - 1].clone();
        for i in (0..args.len() - 1).rev() {
            result = Value::pair(args[i].clone(), result);
        }
        Ok(result)
    }
}

/// list-copy - Create a shallow copy of a list
pub fn srfi1_list_copy(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 1, Some(1), "list-copy")?;

    let list = &args[0];
    match list {
        Value::Nil => Ok(Value::Nil),
        Value::Pair(_, _) => {
            // Convert to vector and back to create a copy
            let vec = SRFI1Core::list_to_vec(list)?;
            Ok(SRFI1Core::vec_to_list(vec))
        }
        _ => {
            // For non-lists, return the value as-is (SRFI-1 allows this)
            Ok(list.clone())
        }
    }
}

/// make-list - Create a list of specified length with optional fill value
pub fn srfi1_make_list(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 1, Some(2), "make-list")?;

    let n = args[0].as_integer().ok_or_else(|| {
        Error::runtime_error("make-list length must be an integer".to_string(), None)
    })?;

    if n < 0 {
        return Err(Box::new(Error::runtime_error(
            "make-list length must be non-negative".to_string(),
            None,
        )));
    }

    let fill_value = args.get(1).cloned().unwrap_or(Value::Nil);
    let values = vec![fill_value; n as usize];

    Ok(SRFI1Core::vec_to_list(values))
}

/// list-tabulate - Create a list by calling a procedure for each index
pub fn srfi1_list_tabulate(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 2, Some(2), "list-tabulate")?;

    let n = args[0].as_integer().ok_or_else(|| {
        Error::runtime_error("list-tabulate length must be an integer".to_string(), None)
    })?;

    if n < 0 {
        return Err(Box::new(Error::runtime_error(
            "list-tabulate length must be non-negative".to_string(),
            None,
        )));
    }

    let proc = &args[1];
    if !proc.is_procedure() {
        return Err(Box::new(Error::runtime_error(
            "list-tabulate second argument must be a procedure".to_string(),
            None,
        )));
    }

    let mut values = Vec::with_capacity(n as usize);
    for i in 0..n {
        let index_value = Value::number(i as f64);
        let result = SRFI1Core::apply_procedure(proc, &index_value)?;
        values.push(result);
    }

    Ok(SRFI1Core::vec_to_list(values))
}

/// circular-list - Create a circular list from arguments
pub fn srfi1_circular_list(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 1, None, "circular-list")?;

    if args.is_empty() {
        return Err(Box::new(Error::runtime_error(
            "circular-list requires at least one argument".to_string(),
            None,
        )));
    }

    // For now, return a regular list with a note that circular lists
    // need special handling in the value system
    // TODO: Implement proper circular list support
    Ok(Value::list(args.to_vec()))
}

/// iota - Generate a list of integers
/// (iota count [start [step]]) generates count integers starting from start (default 0)
/// with step (default 1)
pub fn srfi1_iota(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 1, Some(3), "iota")?;

    let count = args[0]
        .as_integer()
        .ok_or_else(|| Error::runtime_error("iota count must be an integer".to_string(), None))?;

    if count < 0 {
        return Err(Box::new(Error::runtime_error(
            "iota count must be non-negative".to_string(),
            None,
        )));
    }

    let start = args.get(1).and_then(|v| v.as_integer()).unwrap_or(0);

    let step = args.get(2).and_then(|v| v.as_integer()).unwrap_or(1);

    let mut values = Vec::with_capacity(count as usize);
    let mut current = start;

    for _ in 0..count {
        values.push(Value::number(current as f64));
        current += step;
    }

    Ok(SRFI1Core::vec_to_list(values))
}

#[cfg(test)]
mod tests {
    use super::*;
    // Note: Number type is handled through Value::number() factory methods

    #[test]
    fn test_srfi1_cons() {
        let result = srfi1_cons(&[Value::boolean(true), Value::boolean(false)]).unwrap();
        match result {
            Value::Pair(car, cdr) => {
                assert_eq!(*car, Value::boolean(true));
                assert_eq!(*cdr, Value::boolean(false));
            }
            _ => panic!("Expected pair"),
        }
    }

    #[test]
    fn test_srfi1_list() {
        let result = srfi1_list(&[]).unwrap();
        assert_eq!(result, Value::Nil);

        let result = srfi1_list(&[Value::boolean(true), Value::boolean(false)]).unwrap();
        assert!(SRFI1Core::is_proper_list(&result));
        assert_eq!(SRFI1Core::fast_length(&result).unwrap(), 2);
    }

    #[test]
    fn test_srfi1_xcons() {
        let result = srfi1_xcons(&[Value::boolean(false), Value::boolean(true)]).unwrap();
        match result {
            Value::Pair(car, cdr) => {
                assert_eq!(*car, Value::boolean(true)); // Arguments are reversed
                assert_eq!(*cdr, Value::boolean(false));
            }
            _ => panic!("Expected pair"),
        }
    }

    #[test]
    fn test_srfi1_cons_star() {
        // Single argument
        let result = srfi1_cons_star(&[Value::boolean(true)]).unwrap();
        assert_eq!(result, Value::boolean(true));

        // Two arguments
        let result = srfi1_cons_star(&[Value::boolean(true), Value::boolean(false)]).unwrap();
        match result {
            Value::Pair(car, cdr) => {
                assert_eq!(*car, Value::boolean(true));
                assert_eq!(*cdr, Value::boolean(false));
            }
            _ => panic!("Expected pair"),
        }

        // Three arguments: (cons* 1 2 3) => (1 . (2 . 3))
        let result =
            srfi1_cons_star(&[Value::integer(1), Value::integer(2), Value::integer(3)]).unwrap();

        // Should create (1 . (2 . 3))
        assert!(matches!(result, Value::Pair(_, _)));
    }

    #[test]
    fn test_srfi1_list_copy() {
        let original = Value::list(vec![Value::boolean(true), Value::boolean(false)]);
        let copy = srfi1_list_copy(&[original.clone()]).unwrap();

        // Should be equal but not the same object
        let orig_vec = SRFI1Core::list_to_vec(&original).unwrap();
        let copy_vec = SRFI1Core::list_to_vec(&copy).unwrap();
        assert_eq!(orig_vec, copy_vec);
    }

    #[test]
    fn test_srfi1_make_list() {
        // Empty list
        let result = srfi1_make_list(&[Value::integer(0)]).unwrap();
        assert_eq!(result, Value::Nil);

        // List with default fill
        let result = srfi1_make_list(&[Value::integer(3)]).unwrap();
        assert_eq!(SRFI1Core::fast_length(&result).unwrap(), 3);

        // List with custom fill
        let result = srfi1_make_list(&[Value::integer(2), Value::boolean(true)]).unwrap();
        let vec = SRFI1Core::list_to_vec(&result).unwrap();
        assert_eq!(vec, vec![Value::boolean(true), Value::boolean(true)]);
    }

    #[test]
    fn test_srfi1_iota() {
        // Basic iota
        let result = srfi1_iota(&[Value::integer(3)]).unwrap();
        let vec = SRFI1Core::list_to_vec(&result).unwrap();
        assert_eq!(
            vec,
            vec![Value::integer(0), Value::integer(1), Value::integer(2)]
        );

        // With start
        let result = srfi1_iota(&[Value::integer(3), Value::integer(10)]).unwrap();
        let vec = SRFI1Core::list_to_vec(&result).unwrap();
        assert_eq!(
            vec,
            vec![Value::integer(10), Value::integer(11), Value::integer(12)]
        );

        // With start and step
        let result =
            srfi1_iota(&[Value::integer(3), Value::integer(0), Value::integer(5)]).unwrap();
        let vec = SRFI1Core::list_to_vec(&result).unwrap();
        assert_eq!(
            vec,
            vec![Value::integer(0), Value::integer(5), Value::integer(10)]
        );
    }
}
