//! SRFI-1 Predicate Operations
//!
//! This module implements list predicates (already exist in core, ensuring compatibility):
//! - pair?, null?, list?, proper-list?, circular-list?, dotted-list?

use crate::diagnostics::Result;
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::stdlib::srfi1::SRFI1Core;
use std::sync::Arc;

/// Binds predicate operations (most already exist in core)
pub fn bind_predicate_operations(env: &Arc<ThreadSafeEnvironment>) {
    // These are typically already bound in core, but we ensure SRFI-1 compatibility

    // proper-list? - SRFI-1 specific predicate
    env.define(
        "proper-list?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "proper-list?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(srfi1_proper_list_p),
            effects: vec![Effect::Pure],
        })),
    );

    // circular-list? - SRFI-1 specific predicate
    env.define(
        "circular-list?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "circular-list?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(srfi1_circular_list_p),
            effects: vec![Effect::Pure],
        })),
    );

    // dotted-list? - SRFI-1 specific predicate
    env.define(
        "dotted-list?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "dotted-list?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(srfi1_dotted_list_p),
            effects: vec![Effect::Pure],
        })),
    );
}

/// proper-list? - Test if value is a proper list (finite, null-terminated)
pub fn srfi1_proper_list_p(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 1, Some(1), "proper-list?")?;
    Ok(Value::boolean(SRFI1Core::is_proper_list(&args[0])))
}

/// circular-list? - Test if value is a circular list
pub fn srfi1_circular_list_p(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 1, Some(1), "circular-list?")?;

    // For now, return false since we don't have circular list support yet
    // TODO: Implement proper circular list detection with Floyd's cycle detection
    Ok(Value::boolean(false))
}

/// dotted-list? - Test if value is a dotted (improper) list
pub fn srfi1_dotted_list_p(args: &[Value]) -> Result<Value> {
    SRFI1Core::validate_arity(args, 1, Some(1), "dotted-list?")?;

    let value = &args[0];

    // A dotted list is a pair chain that doesn't end with null
    let is_dotted = match value {
        Value::Nil => false, // Empty list is proper, not dotted
        Value::Pair(_, _) => !SRFI1Core::is_proper_list(value),
        _ => false, // Non-pairs can't be dotted lists
    };

    Ok(Value::boolean(is_dotted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proper_list_predicate() {
        let proper = Value::list(vec![Value::boolean(true), Value::boolean(false)]);
        let result = srfi1_proper_list_p(&[proper]).unwrap();
        assert_eq!(result, Value::boolean(true));

        let nil = Value::Nil;
        let result = srfi1_proper_list_p(&[nil]).unwrap();
        assert_eq!(result, Value::boolean(true));

        let not_list = Value::boolean(true);
        let result = srfi1_proper_list_p(&[not_list]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_circular_list_predicate() {
        let list = Value::list(vec![Value::boolean(true)]);
        let result = srfi1_circular_list_p(&[list]).unwrap();
        // Currently always false since circular lists not implemented
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_dotted_list_predicate() {
        let proper = Value::list(vec![Value::boolean(true)]);
        let result = srfi1_dotted_list_p(&[proper]).unwrap();
        assert_eq!(result, Value::boolean(false));

        let dotted = Value::pair(Value::boolean(true), Value::boolean(false)); // (true . false)
        let result = srfi1_dotted_list_p(&[dotted]).unwrap();
        assert_eq!(result, Value::boolean(true));
    }
}
