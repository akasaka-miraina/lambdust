//! SRFI 111: Boxes
//!
//! This SRFI provides boxes: objects with a single mutable state
//! that can be updated by calling box-set! and observed by calling unbox.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
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
    #[must_use] pub fn new(value: Value) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
        }
    }

    /// Get the value from the box
    #[must_use] pub fn unbox(&self) -> Value {
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

