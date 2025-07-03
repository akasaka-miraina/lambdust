//! Pair data structures for cons cells

use super::Value;
use std::cell::RefCell;
use std::rc::Rc;

/// Pair data structure for cons cells
/// 
/// This uses Rc<RefCell<>> to allow efficient sharing across execution contexts
/// while maintaining mutability for operations like set-car! and set-cdr!
#[derive(Debug, Clone)]
pub struct PairData {
    /// The car (first element) of the pair
    pub car: Value,
    /// The cdr (rest element) of the pair  
    pub cdr: Value,
}

impl PairData {
    /// Create a new pair
    pub fn new(car: Value, cdr: Value) -> Self {
        PairData { car, cdr }
    }
}

impl Value {
    /// Create a pair (cons cell)
    pub fn cons(car: Value, cdr: Value) -> Value {
        Value::Pair(Rc::new(RefCell::new(PairData::new(car, cdr))))
    }

    /// Check if this value is a pair
    pub fn is_pair(&self) -> bool {
        matches!(self, Value::Pair(_))
    }

    /// Get the pair components if this is a pair
    /// Note: This returns cloned values to avoid borrow checker issues
    pub fn as_pair(&self) -> Option<(Value, Value)> {
        match self {
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                Some((pair.car.clone(), pair.cdr.clone()))
            }
            _ => None,
        }
    }

    /// Get the car (first element) of a pair
    /// Returns a cloned value to avoid borrow checker issues
    pub fn car(&self) -> Option<Value> {
        match self {
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                Some(pair.car.clone())
            }
            _ => None,
        }
    }

    /// Get the cdr (rest) of a pair
    /// Returns a cloned value to avoid borrow checker issues
    pub fn cdr(&self) -> Option<Value> {
        match self {
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                Some(pair.cdr.clone())
            }
            _ => None,
        }
    }

    /// Set the car (first element) of a pair (for set-car!)
    /// This enables mutating operations on pairs while maintaining safety
    pub fn set_car(&self, new_car: Value) -> Result<(), String> {
        match self {
            Value::Pair(pair_ref) => {
                let mut pair = pair_ref.borrow_mut();
                pair.car = new_car;
                Ok(())
            }
            _ => Err("set-car! can only be applied to pairs".to_string()),
        }
    }

    /// Set the cdr (rest) of a pair (for set-cdr!)
    /// This enables mutating operations on pairs while maintaining safety
    pub fn set_cdr(&self, new_cdr: Value) -> Result<(), String> {
        match self {
            Value::Pair(pair_ref) => {
                let mut pair = pair_ref.borrow_mut();
                pair.cdr = new_cdr;
                Ok(())
            }
            _ => Err("set-cdr! can only be applied to pairs".to_string()),
        }
    }
}