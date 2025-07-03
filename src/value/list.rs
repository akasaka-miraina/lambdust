//! List operations and utilities

use super::{PairData, Value};
use std::cell::RefCell;
use std::rc::Rc;

impl Value {
    /// Check if this value is nil (empty list)
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    /// Check if this value is a list (proper list ending in nil)
    pub fn is_list(&self) -> bool {
        match self {
            Value::Nil => true,
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                pair.cdr.is_list()
            }
            _ => false,
        }
    }

    /// Convert this value to a vector if it's a list
    pub fn to_vector(&self) -> Option<Vec<Value>> {
        let mut result = Vec::new();
        let mut current = self.clone();

        loop {
            match current {
                Value::Nil => return Some(result),
                Value::Pair(pair_ref) => {
                    let pair = pair_ref.borrow();
                    result.push(pair.car.clone());
                    current = pair.cdr.clone();
                }
                _ => return None, // Not a proper list
            }
        }
    }

    /// Create a list from a vector of values
    pub fn from_vector(values: Vec<Value>) -> Value {
        values.into_iter().rev().fold(Value::Nil, |acc, val| {
            Value::Pair(Rc::new(RefCell::new(PairData::new(val, acc))))
        })
    }

    /// Get the length of a list
    pub fn list_length(&self) -> Option<usize> {
        let mut length = 0;
        let mut current = self.clone();

        loop {
            match current {
                Value::Nil => return Some(length),
                Value::Pair(pair_ref) => {
                    length += 1;
                    let pair = pair_ref.borrow();
                    current = pair.cdr.clone();
                }
                _ => return None, // Not a proper list
            }
        }
    }
}