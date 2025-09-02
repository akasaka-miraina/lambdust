//! SRFI-1 Miscellaneous List Operations
//!
//! This module implements basic list operations:
//! - length, append, reverse
//! - take, drop, split-at
//! - last, butlast

use crate::ast::Literal;
use crate::diagnostics::{Error, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};

use crate::stdlib::srfi1::SRFI1Core;
use std::sync::Arc;

/// Binds all miscellaneous operations to the environment
pub fn bind_miscellaneous_operations(env: &Arc<ThreadSafeEnvironment>) {
    // length (already exists in core, ensure SRFI-1 semantics)
    env.define(
        "length".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "length".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(srfi1_length),
            effects: vec![Effect::Pure],
        })),
    );

    // append (already exists in core, ensure SRFI-1 semantics)
    env.define(
        "append".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "append".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(srfi1_append),
            effects: vec![Effect::Pure],
        })),
    );

    // reverse (already exists in core, ensure SRFI-1 semantics)
    env.define(
        "reverse".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "reverse".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(srfi1_reverse),
            effects: vec![Effect::Pure],
        })),
    );

    // take (from existing srfi1.rs)
    env.define(
        "take".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "take".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(srfi1_take),
            effects: vec![Effect::Pure],
        })),
    );

    // drop (from existing srfi1.rs)
    env.define(
        "drop".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "drop".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(srfi1_drop),
            effects: vec![Effect::Pure],
        })),
    );

    // split-at
    env.define(
        "split-at".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "split-at".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(srfi1_split_at),
            effects: vec![Effect::Pure],
        })),
    );

    // last
    env.define(
        "last".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "last".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(srfi1_last),
            effects: vec![Effect::Pure],
        })),
    );

    // butlast (all but last element)
    env.define(
        "butlast".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "butlast".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(srfi1_butlast),
            effects: vec![Effect::Pure],
        })),
    );
}

/// length - Get the length of a proper list
pub fn srfi1_length(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 1, Some(1), "length")?;

    let list = &args[0];
    let len = SRFI1Core::fast_length(list)?;
    Ok(Value::number(len as i64 as f64))
}

/// append - Concatenate lists
pub fn srfi1_append(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Nil);
    }

    if args.len() == 1 {
        return Ok(args[0].clone());
    }

    let mut all_elements = Vec::new();

    // Process all but the last argument as proper lists
    for (i, list) in args[..args.len() - 1].iter().enumerate() {
        SRFI1Core::ensure_proper_list(list, &format!("append argument {}", i + 1))?;
        let elements = SRFI1Core::list_to_vec(list)?;
        all_elements.extend(elements);
    }

    // The last argument can be any value (becomes the final cdr)
    let last_arg = &args[args.len() - 1];

    if all_elements.is_empty() {
        Ok(last_arg.clone())
    } else {
        // Build list with last_arg as the final cdr
        let mut result = last_arg.clone();
        for element in all_elements.into_iter().rev() {
            result = Value::pair(element, result);
        }
        Ok(result)
    }
}

/// reverse - Reverse a proper list
pub fn srfi1_reverse(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 1, Some(1), "reverse")?;

    let list = &args[0];
    SRFI1Core::ensure_proper_list(list, "reverse")?;

    let mut elements = SRFI1Core::list_to_vec(list)?;
    elements.reverse();

    Ok(SRFI1Core::vec_to_list(elements))
}

/// take - Take the first n elements of a list
pub fn srfi1_take(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 2, Some(2), "take")?;

    let list = &args[0];
    let n = args[1].as_integer().ok_or_else(|| {
        Error::runtime_error("take n must be a non-negative integer".to_string(), None)
    })?;

    if n < 0 {
        return Err(Box::new(Error::runtime_error(
            "take n must be non-negative".to_string(),
            None,
        )));
    }

    SRFI1Core::ensure_proper_list(list, "take")?;
    let elements = SRFI1Core::list_to_vec(list)?;

    if (n as usize) > elements.len() {
        return Err(Box::new(Error::runtime_error(
            "take: list too short".to_string(),
            None,
        )));
    }

    let taken = elements[..(n as usize)].to_vec();
    Ok(SRFI1Core::vec_to_list(taken))
}

/// drop - Drop the first n elements of a list
pub fn srfi1_drop(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 2, Some(2), "drop")?;

    let list = &args[0];
    let n = args[1].as_integer().ok_or_else(|| {
        Error::runtime_error("drop n must be a non-negative integer".to_string(), None)
    })?;

    if n < 0 {
        return Err(Box::new(Error::runtime_error(
            "drop n must be non-negative".to_string(),
            None,
        )));
    }

    SRFI1Core::ensure_proper_list(list, "drop")?;
    let elements = SRFI1Core::list_to_vec(list)?;

    if (n as usize) > elements.len() {
        return Err(Box::new(Error::runtime_error(
            "drop: list too short".to_string(),
            None,
        )));
    }

    let dropped = elements[(n as usize)..].to_vec();
    Ok(SRFI1Core::vec_to_list(dropped))
}

/// split-at - Split a list at the nth position
pub fn srfi1_split_at(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 2, Some(2), "split-at")?;

    let first_part = srfi1_take(args)?;
    let second_part = srfi1_drop(args)?;

    Ok(Value::pair(first_part, second_part))
}

/// last - Return the last element of a non-empty list
pub fn srfi1_last(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 1, Some(1), "last")?;

    let list = &args[0];
    SRFI1Core::ensure_proper_list(list, "last")?;

    let elements = SRFI1Core::list_to_vec(list)?;
    if elements.is_empty() {
        return Err(Box::new(Error::runtime_error(
            "last: empty list".to_string(),
            None,
        )));
    }

    Ok(elements[elements.len() - 1].clone())
}

/// butlast - Return all but the last element of a list
pub fn srfi1_butlast(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 1, Some(1), "butlast")?;

    let list = &args[0];
    SRFI1Core::ensure_proper_list(list, "butlast")?;

    let mut elements = SRFI1Core::list_to_vec(list)?;
    if !elements.is_empty() {
        elements.pop(); // Remove last element
    }

    Ok(SRFI1Core::vec_to_list(elements))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_misc_binding() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        bind_miscellaneous_operations(&env);

        assert!(env.lookup("length").is_some());
        assert!(env.lookup("append").is_some());
        assert!(env.lookup("reverse").is_some());
        assert!(env.lookup("take").is_some());
        assert!(env.lookup("drop").is_some());
        assert!(env.lookup("split-at").is_some());
        assert!(env.lookup("last").is_some());
        assert!(env.lookup("butlast").is_some());
    }

    #[test]
    fn test_length() {
        let list = Value::list(vec![
            Value::boolean(true),
            Value::boolean(false),
            Value::Nil,
        ]);
        let result = srfi1_length(&[list]).unwrap();
        assert_eq!(result, Value::integer(3));

        let empty = Value::Nil;
        let result = srfi1_length(&[empty]).unwrap();
        assert_eq!(result, Value::integer(0));
    }

    #[test]
    fn test_append() {
        let list1 = Value::list(vec![Value::boolean(true)]);
        let list2 = Value::list(vec![Value::boolean(false)]);
        let result = srfi1_append(&[list1, list2]).unwrap();

        let result_vec = SRFI1Core::list_to_vec(&result).unwrap();
        assert_eq!(
            result_vec,
            vec![Value::boolean(true), Value::boolean(false)]
        );
    }

    #[test]
    fn test_reverse() {
        let list = Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]);

        let result = srfi1_reverse(&[list]).unwrap();
        let result_vec = SRFI1Core::list_to_vec(&result).unwrap();

        assert_eq!(
            result_vec,
            vec![Value::integer(3), Value::integer(2), Value::integer(1)]
        );
    }

    #[test]
    fn test_take_drop() {
        let list = Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
            Value::integer(4),
        ]);

        let taken = srfi1_take(&[list.clone(), Value::integer(2)]).unwrap();
        let taken_vec = SRFI1Core::list_to_vec(&taken).unwrap();
        assert_eq!(taken_vec, vec![Value::integer(1), Value::integer(2)]);

        let dropped = srfi1_drop(&[list, Value::integer(2)]).unwrap();
        let dropped_vec = SRFI1Core::list_to_vec(&dropped).unwrap();
        assert_eq!(dropped_vec, vec![Value::integer(3), Value::integer(4)]);
    }

    #[test]
    fn test_last_butlast() {
        let list = Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]);

        let last = srfi1_last(&[list.clone()]).unwrap();
        assert_eq!(last, Value::integer(3));

        let butlast = srfi1_butlast(&[list]).unwrap();
        let butlast_vec = SRFI1Core::list_to_vec(&butlast).unwrap();
        assert_eq!(butlast_vec, vec![Value::integer(1), Value::integer(2)]);
    }

    #[test]
    fn test_split_at() {
        let list = Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
            Value::integer(4),
        ]);

        let result = srfi1_split_at(&[list, Value::integer(2)]).unwrap();

        match result {
            Value::Pair(first, second) => {
                let first_vec = SRFI1Core::list_to_vec(&first).unwrap();
                let second_vec = SRFI1Core::list_to_vec(&second).unwrap();

                assert_eq!(first_vec, vec![Value::integer(1), Value::integer(2)]);

                assert_eq!(second_vec, vec![Value::integer(3), Value::integer(4)]);
            }
            _ => panic!("split-at should return a pair"),
        }
    }
}
