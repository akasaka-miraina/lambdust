//! SRFI-1 Filtering Operations
//!
//! This module implements list filtering and partitioning operations:
//! - filter, remove, partition
//!
//! Optimized for performance using arena allocation and efficient traversal.

use crate::diagnostics::{Error, Result};
use crate::effects::Effect;
use crate::eval::list_arena::GlobalListArena;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::stdlib::srfi1::SRFI1Core;
use std::sync::Arc;

/// Binds all filtering operations to the environment
pub fn bind_filtering_operations(env: &Arc<ThreadSafeEnvironment>) {
    // filter
    env.define(
        "filter".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "filter".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(srfi1_filter),
            effects: vec![Effect::Pure],
        })),
    );

    // remove (opposite of filter)
    env.define(
        "remove".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "remove".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(srfi1_remove),
            effects: vec![Effect::Pure],
        })),
    );

    // partition
    env.define(
        "partition".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "partition".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(srfi1_partition),
            effects: vec![Effect::Pure],
        })),
    );
}

/// filter - Select elements that satisfy predicate
pub fn srfi1_filter(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 2, Some(2), "filter")?;

    let predicate = &args[0];
    let list = &args[1];

    if !predicate.is_procedure() {
        return Err(Box::new(Error::runtime_error(
            "filter first argument must be a procedure".to_string(),
            None,
        )));
    }

    SRFI1Core::ensure_proper_list(list, "filter")?;

    let elements = SRFI1Core::list_to_vec(list)?;
    if elements.is_empty() {
        return Ok(Value::Nil);
    }

    // Create arena for optimized allocation
    let mut arena = GlobalListArena::create_construction_arena(elements.len());
    let mut filtered_elements = Vec::new();

    // Apply predicate to each element
    for element in elements {
        let keep = match predicate {
            Value::Primitive(prim) => {
                let pred_result = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&[element.clone()])?,
                    PrimitiveImpl::Native(func) => func(&[element.clone()])?,
                    _ => {
                        return Err(Box::new(Error::runtime_error(
                            "filter with complex procedures not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
                !pred_result.is_falsy()
            }
            _ => {
                return Err(Box::new(Error::runtime_error(
                    "filter with user-defined procedures not yet implemented".to_string(),
                    None,
                )));
            }
        };

        if keep {
            filtered_elements.push(element);
        }
    }

    Ok(SRFI1Core::vec_to_list(filtered_elements))
}

/// remove - Remove elements that satisfy predicate (opposite of filter)
pub fn srfi1_remove(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 2, Some(2), "remove")?;

    let predicate = &args[0];
    let list = &args[1];

    if !predicate.is_procedure() {
        return Err(Box::new(Error::runtime_error(
            "remove first argument must be a procedure".to_string(),
            None,
        )));
    }

    SRFI1Core::ensure_proper_list(list, "remove")?;

    let elements = SRFI1Core::list_to_vec(list)?;
    if elements.is_empty() {
        return Ok(Value::Nil);
    }

    let mut removed_elements = Vec::new();

    // Apply predicate to each element (keep those that don't satisfy predicate)
    for element in elements {
        let remove = match predicate {
            Value::Primitive(prim) => {
                let pred_result = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&[element.clone()])?,
                    PrimitiveImpl::Native(func) => func(&[element.clone()])?,
                    _ => {
                        return Err(Box::new(Error::runtime_error(
                            "remove with complex procedures not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
                !pred_result.is_falsy()
            }
            _ => {
                return Err(Box::new(Error::runtime_error(
                    "remove with user-defined procedures not yet implemented".to_string(),
                    None,
                )));
            }
        };

        if !remove {
            // Keep elements that don't satisfy predicate
            removed_elements.push(element);
        }
    }

    Ok(SRFI1Core::vec_to_list(removed_elements))
}

/// partition - Split list into two lists based on predicate
/// Returns (values satisfying-list not-satisfying-list)
pub fn srfi1_partition(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 2, Some(2), "partition")?;

    let predicate = &args[0];
    let list = &args[1];

    if !predicate.is_procedure() {
        return Err(Box::new(Error::runtime_error(
            "partition first argument must be a procedure".to_string(),
            None,
        )));
    }

    SRFI1Core::ensure_proper_list(list, "partition")?;

    let elements = SRFI1Core::list_to_vec(list)?;
    if elements.is_empty() {
        return Ok(Value::pair(Value::Nil, Value::Nil));
    }

    let mut satisfying = Vec::new();
    let mut not_satisfying = Vec::new();

    // Partition elements based on predicate
    for element in elements {
        let satisfies = match predicate {
            Value::Primitive(prim) => {
                let pred_result = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&[element.clone()])?,
                    PrimitiveImpl::Native(func) => func(&[element.clone()])?,
                    _ => {
                        return Err(Box::new(Error::runtime_error(
                            "partition with complex procedures not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
                !pred_result.is_falsy()
            }
            _ => {
                return Err(Box::new(Error::runtime_error(
                    "partition with user-defined procedures not yet implemented".to_string(),
                    None,
                )));
            }
        };

        if satisfies {
            satisfying.push(element);
        } else {
            not_satisfying.push(element);
        }
    }

    let satisfying_list = SRFI1Core::vec_to_list(satisfying);
    let not_satisfying_list = SRFI1Core::vec_to_list(not_satisfying);

    Ok(Value::pair(satisfying_list, not_satisfying_list))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    fn is_positive() -> Value {
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "positive?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| match &args[0] {
                Value::Literal(Literal::ExactInteger(n)) => Ok(Value::boolean(*n > 0)),
                Value::Literal(Literal::InexactReal(n)) => Ok(Value::boolean(*n > 0.0)),
                #[allow(deprecated)]
                Value::Literal(Literal::Number(n)) => Ok(Value::boolean(*n > 0.0)),
                _ => Ok(Value::boolean(false)),
            }),
            effects: vec![Effect::Pure],
        }))
    }

    #[test]
    fn test_filtering_binding() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        bind_filtering_operations(&env);

        assert!(env.lookup("filter").is_some());
        assert!(env.lookup("remove").is_some());
        assert!(env.lookup("partition").is_some());
    }

    #[test]
    fn test_filter_empty_list() {
        let predicate = is_positive();
        let result = srfi1_filter(&[predicate, Value::Nil]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_filter_basic() {
        let predicate = is_positive();
        let list = Value::list(vec![
            Value::integer(-1),
            Value::integer(2),
            Value::integer(-3),
            Value::integer(4),
        ]);

        let result = srfi1_filter(&[predicate, list]).unwrap();
        let result_vec = SRFI1Core::list_to_vec(&result).unwrap();

        assert_eq!(result_vec, vec![Value::integer(2), Value::integer(4)]);
    }

    #[test]
    fn test_remove_basic() {
        let predicate = is_positive();
        let list = Value::list(vec![
            Value::integer(-1),
            Value::integer(2),
            Value::integer(-3),
            Value::integer(4),
        ]);

        let result = srfi1_remove(&[predicate, list]).unwrap();
        let result_vec = SRFI1Core::list_to_vec(&result).unwrap();

        assert_eq!(result_vec, vec![Value::integer(-1), Value::integer(-3)]);
    }

    #[test]
    fn test_partition_basic() {
        let predicate = is_positive();
        let list = Value::list(vec![
            Value::integer(-1),
            Value::integer(2),
            Value::integer(-3),
            Value::integer(4),
        ]);

        let result = srfi1_partition(&[predicate, list]).unwrap();

        match result {
            Value::Pair(satisfying, not_satisfying) => {
                let sat_vec = SRFI1Core::list_to_vec(&satisfying).unwrap();
                let not_sat_vec = SRFI1Core::list_to_vec(&not_satisfying).unwrap();

                assert_eq!(sat_vec, vec![Value::integer(2), Value::integer(4)]);

                assert_eq!(not_sat_vec, vec![Value::integer(-1), Value::integer(-3)]);
            }
            _ => panic!("partition should return a pair"),
        }
    }

    #[test]
    fn test_partition_empty() {
        let predicate = is_positive();
        let result = srfi1_partition(&[predicate, Value::Nil]).unwrap();

        match result {
            Value::Pair(satisfying, not_satisfying) => {
                assert_eq!(*satisfying.as_ref(), Value::Nil);
                assert_eq!(*not_satisfying.as_ref(), Value::Nil);
            }
            _ => panic!("partition should return a pair"),
        }
    }
}
