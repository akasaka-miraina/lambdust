//! SRFI 111: Boxes
//!
//! This SRFI provides boxes: objects with a single mutable state
//! that can be updated by calling box-set! and observed by calling unbox.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
#[cfg(test)]
use crate::value::Procedure;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Box data structure - mutable container for a single value
#[derive(Debug, Clone)]
pub struct Box {
    value: Rc<RefCell<Value>>,
}

impl Box {
    /// Create a new box with the given value
    pub fn new(value: Value) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
        }
    }

    /// Get the value from the box
    pub fn unbox(&self) -> Value {
        self.value.borrow().clone()
    }

    /// Set the value in the box
    pub fn set(&self, new_value: Value) {
        *self.value.borrow_mut() = new_value;
    }
}

impl PartialEq for Box {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }
}

/// SRFI 111 implementation
pub struct Srfi111;

impl super::SrfiModule for Srfi111 {
    fn srfi_id(&self) -> u32 {
        111
    }

    fn name(&self) -> &'static str {
        "Boxes"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec![]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // box constructor
        exports.insert(
            "box".to_string(),
            make_builtin_procedure("box", Some(1), |args| {
                check_arity(args, 1)?;
                Ok(Value::Box(Box::new(args[0].clone())))
            }),
        );

        // unbox procedure
        exports.insert(
            "unbox".to_string(),
            make_builtin_procedure("unbox", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::Box(box_val) = &args[0] {
                    Ok(box_val.unbox())
                } else {
                    Err(LambdustError::type_error("Expected box".to_string()))
                }
            }),
        );

        // set-box! procedure
        exports.insert(
            "set-box!".to_string(),
            make_builtin_procedure("set-box!", Some(2), |args| {
                check_arity(args, 2)?;
                if let Value::Box(box_val) = &args[0] {
                    box_val.set(args[1].clone());
                    Ok(Value::Undefined)
                } else {
                    Err(LambdustError::type_error("Expected box".to_string()))
                }
            }),
        );

        // box? predicate
        exports.insert(
            "box?".to_string(),
            make_builtin_procedure("box?", Some(1), |args| {
                check_arity(args, 1)?;
                Ok(Value::Boolean(matches!(args[0], Value::Box(_))))
            }),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 111 has no parts, return all exports
        Ok(self.exports())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srfi::SrfiModule;

    #[test]
    fn test_box_creation() {
        let srfi = Srfi111;
        let exports = srfi.exports();

        // Test box constructor
        let box_proc = exports.get("box").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = box_proc {
            let result = func(&[Value::from(42i64)]).unwrap();
            assert!(matches!(result, Value::Box(_)));
        }
    }

    #[test]
    fn test_box_operations() {
        let srfi = Srfi111;
        let exports = srfi.exports();

        // Create a box
        let box_proc = exports.get("box").unwrap();
        let unbox_proc = exports.get("unbox").unwrap();
        let set_box_proc = exports.get("set-box!").unwrap();
        let box_pred = exports.get("box?").unwrap();

        if let (
            Value::Procedure(Procedure::Builtin { func: box_func, .. }),
            Value::Procedure(Procedure::Builtin {
                func: unbox_func, ..
            }),
            Value::Procedure(Procedure::Builtin { func: set_func, .. }),
            Value::Procedure(Procedure::Builtin {
                func: pred_func, ..
            }),
        ) = (box_proc, unbox_proc, set_box_proc, box_pred)
        {
            // Create box with initial value
            let box_val = box_func(&[Value::from(42i64)]).unwrap();

            // Test predicate
            let is_box = pred_func(&[box_val.clone()]).unwrap();
            assert_eq!(is_box, Value::Boolean(true));

            let not_box = pred_func(&[Value::from(42i64)]).unwrap();
            assert_eq!(not_box, Value::Boolean(false));

            // Test unbox
            let unboxed = unbox_func(&[box_val.clone()]).unwrap();
            assert_eq!(unboxed, Value::from(42i64));

            // Test set-box!
            let _result = set_func(&[box_val.clone(), Value::from(100i64)]).unwrap();

            // Test that value changed
            let new_unboxed = unbox_func(&[box_val]).unwrap();
            assert_eq!(new_unboxed, Value::from(100i64));
        }
    }

    #[test]
    fn test_box_errors() {
        let srfi = Srfi111;
        let exports = srfi.exports();

        let unbox_proc = exports.get("unbox").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = unbox_proc {
            // Test unbox with wrong type
            let result = func(&[Value::from(42i64)]);
            assert!(result.is_err());

            // Test unbox with wrong arity
            let result = func(&[]);
            assert!(result.is_err());
        }
    }
}
