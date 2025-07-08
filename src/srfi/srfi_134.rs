//! SRFI 134: Immutable Deques
//!
//! This SRFI provides immutable double-ended queues (ideques) with efficient
//! O(1) amortized operations for adding and removing elements from both ends.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// Immutable deque implementation using a two-list structure
/// for amortized O(1) operations on both ends
#[derive(Debug, Clone, PartialEq)]
pub struct Ideque {
    /// Front list (elements in normal order)
    front: Vec<Value>,
    /// Rear list (elements in reverse order)
    rear: Vec<Value>,
    /// Cached length for O(1) length queries
    length: usize,
}

impl Ideque {
    /// Create a new empty ideque
    pub fn new() -> Self {
        Self {
            front: Vec::new(),
            rear: Vec::new(),
            length: 0,
        }
    }

    /// Create ideque from elements
    pub fn from_elements(elements: Vec<Value>) -> Self {
        let length = elements.len();
        Self {
            front: elements,
            rear: Vec::new(),
            length,
        }
    }

    /// Check if ideque is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Get length of ideque
    pub fn len(&self) -> usize {
        self.length
    }

    /// Get front element without removing it
    pub fn front(&self) -> Result<Value> {
        if self.is_empty() {
            return Err(LambdustError::runtime_error(
                "Cannot get front of empty ideque".to_string(),
            ));
        }

        if !self.front.is_empty() {
            Ok(self.front[0].clone())
        } else {
            // Front is empty, so the element must be at the rear end
            Ok(self.rear[self.rear.len() - 1].clone())
        }
    }

    /// Get back element without removing it
    pub fn back(&self) -> Result<Value> {
        if self.is_empty() {
            return Err(LambdustError::runtime_error(
                "Cannot get back of empty ideque".to_string(),
            ));
        }

        if !self.rear.is_empty() {
            Ok(self.rear[0].clone())
        } else {
            // Rear is empty, so the element must be at the front end
            Ok(self.front[self.front.len() - 1].clone())
        }
    }

    /// Add element to front
    pub fn add_front(&self, element: Value) -> Self {
        let mut new_front = Vec::with_capacity(self.front.len() + 1);
        new_front.push(element);
        new_front.extend_from_slice(&self.front);

        Self {
            front: new_front,
            rear: self.rear.clone(),
            length: self.length + 1,
        }
    }

    /// Add element to back
    pub fn add_back(&self, element: Value) -> Self {
        let mut new_rear = Vec::with_capacity(self.rear.len() + 1);
        new_rear.push(element);
        new_rear.extend_from_slice(&self.rear);

        Self {
            front: self.front.clone(),
            rear: new_rear,
            length: self.length + 1,
        }
    }

    /// Remove element from front
    pub fn remove_front(&self) -> Result<Self> {
        if self.is_empty() {
            return Err(LambdustError::runtime_error(
                "Cannot remove from empty ideque".to_string(),
            ));
        }

        if !self.front.is_empty() {
            // Remove from front list
            let new_front = self.front[1..].to_vec();
            Ok(Self {
                front: new_front,
                rear: self.rear.clone(),
                length: self.length - 1,
            })
        } else {
            // Front is empty, remove from rear (which means last element)
            let new_rear = self.rear[..self.rear.len() - 1].to_vec();
            Ok(Self {
                front: self.front.clone(),
                rear: new_rear,
                length: self.length - 1,
            })
        }
    }

    /// Remove element from back
    pub fn remove_back(&self) -> Result<Self> {
        if self.is_empty() {
            return Err(LambdustError::runtime_error(
                "Cannot remove from empty ideque".to_string(),
            ));
        }

        if !self.rear.is_empty() {
            // Remove from rear list
            let new_rear = self.rear[1..].to_vec();
            Ok(Self {
                front: self.front.clone(),
                rear: new_rear,
                length: self.length - 1,
            })
        } else {
            // Rear is empty, remove from front (which means last element)
            let new_front = self.front[..self.front.len() - 1].to_vec();
            Ok(Self {
                front: new_front,
                rear: self.rear.clone(),
                length: self.length - 1,
            })
        }
    }

    /// Convert to list for iteration
    pub fn to_list(&self) -> Vec<Value> {
        let mut result = Vec::with_capacity(self.length);
        result.extend_from_slice(&self.front);
        result.extend(self.rear.iter().rev().cloned());
        result
    }
}

impl Default for Ideque {
    fn default() -> Self {
        Self::new()
    }
}

/// SRFI 134 implementation
pub struct Srfi134;

impl super::SrfiModule for Srfi134 {
    fn srfi_id(&self) -> u32 {
        134
    }

    fn name(&self) -> &'static str {
        "Immutable Deques"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec![]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Constructor procedures
        exports.insert(
            "ideque".to_string(),
            make_builtin_procedure("ideque", None, |args| {
                let ideque = Ideque::from_elements(args.to_vec());
                Ok(Value::Ideque(Rc::new(ideque)))
            }),
        );

        // Predicate procedures
        exports.insert(
            "ideque?".to_string(),
            make_builtin_procedure("ideque?", Some(1), |args| {
                check_arity(args, 1)?;
                Ok(Value::Boolean(matches!(args[0], Value::Ideque(_))))
            }),
        );

        exports.insert(
            "ideque-empty?".to_string(),
            make_builtin_procedure("ideque-empty?", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::Ideque(ideque) = &args[0] {
                    Ok(Value::Boolean(ideque.is_empty()))
                } else {
                    Err(LambdustError::type_error("Expected ideque".to_string()))
                }
            }),
        );

        // Queue operations
        exports.insert(
            "ideque-front".to_string(),
            make_builtin_procedure("ideque-front", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::Ideque(ideque) = &args[0] {
                    ideque.front()
                } else {
                    Err(LambdustError::type_error("Expected ideque".to_string()))
                }
            }),
        );

        exports.insert(
            "ideque-back".to_string(),
            make_builtin_procedure("ideque-back", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::Ideque(ideque) = &args[0] {
                    ideque.back()
                } else {
                    Err(LambdustError::type_error("Expected ideque".to_string()))
                }
            }),
        );

        exports.insert(
            "ideque-add-front".to_string(),
            make_builtin_procedure("ideque-add-front", Some(2), |args| {
                check_arity(args, 2)?;
                if let Value::Ideque(ideque) = &args[0] {
                    let new_ideque = ideque.add_front(args[1].clone());
                    Ok(Value::Ideque(Rc::new(new_ideque)))
                } else {
                    Err(LambdustError::type_error("Expected ideque".to_string()))
                }
            }),
        );

        exports.insert(
            "ideque-add-back".to_string(),
            make_builtin_procedure("ideque-add-back", Some(2), |args| {
                check_arity(args, 2)?;
                if let Value::Ideque(ideque) = &args[0] {
                    let new_ideque = ideque.add_back(args[1].clone());
                    Ok(Value::Ideque(Rc::new(new_ideque)))
                } else {
                    Err(LambdustError::type_error("Expected ideque".to_string()))
                }
            }),
        );

        exports.insert(
            "ideque-remove-front".to_string(),
            make_builtin_procedure("ideque-remove-front", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::Ideque(ideque) = &args[0] {
                    let new_ideque = ideque.remove_front()?;
                    Ok(Value::Ideque(Rc::new(new_ideque)))
                } else {
                    Err(LambdustError::type_error("Expected ideque".to_string()))
                }
            }),
        );

        exports.insert(
            "ideque-remove-back".to_string(),
            make_builtin_procedure("ideque-remove-back", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::Ideque(ideque) = &args[0] {
                    let new_ideque = ideque.remove_back()?;
                    Ok(Value::Ideque(Rc::new(new_ideque)))
                } else {
                    Err(LambdustError::type_error("Expected ideque".to_string()))
                }
            }),
        );

        // Length procedure
        exports.insert(
            "ideque-length".to_string(),
            make_builtin_procedure("ideque-length", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::Ideque(ideque) = &args[0] {
                    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
                        ideque.len() as i64,
                    )))
                } else {
                    Err(LambdustError::type_error("Expected ideque".to_string()))
                }
            }),
        );

        // Conversion procedures
        exports.insert(
            "ideque->list".to_string(),
            make_builtin_procedure("ideque->list", Some(1), |args| {
                check_arity(args, 1)?;
                if let Value::Ideque(ideque) = &args[0] {
                    let elements = ideque.to_list();
                    let mut result = Value::Nil;
                    for element in elements.into_iter().rev() {
                        let pair_data = crate::value::PairData {
                            car: element,
                            cdr: result,
                        };
                        result = Value::Pair(std::rc::Rc::new(std::cell::RefCell::new(pair_data)));
                    }
                    Ok(result)
                } else {
                    Err(LambdustError::type_error("Expected ideque".to_string()))
                }
            }),
        );

        exports.insert(
            "list->ideque".to_string(),
            make_builtin_procedure("list->ideque", Some(1), |args| {
                check_arity(args, 1)?;
                let mut elements = Vec::new();
                let mut current = args[0].clone();

                // Convert list to vector
                loop {
                    match current {
                        Value::Nil => break,
                        Value::Pair(pair_ref) => {
                            let pair = pair_ref.borrow();
                            elements.push(pair.car.clone());
                            current = pair.cdr.clone();
                        }
                        _ => {
                            return Err(LambdustError::type_error(
                                "Expected proper list".to_string(),
                            ));
                        }
                    }
                }

                let ideque = Ideque::from_elements(elements);
                Ok(Value::Ideque(Rc::new(ideque)))
            }),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 134 has no parts
        Ok(self.exports())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srfi::SrfiModule;

    #[test]
    fn test_ideque_creation() {
        let empty_ideque = Ideque::new();
        assert!(empty_ideque.is_empty());
        assert_eq!(empty_ideque.len(), 0);

        let elements = vec![Value::from(1i64), Value::from(2i64), Value::from(3i64)];
        let ideque = Ideque::from_elements(elements);
        assert!(!ideque.is_empty());
        assert_eq!(ideque.len(), 3);
    }

    #[test]
    fn test_ideque_front_back_operations() {
        let mut ideque = Ideque::new();
        
        // Test empty ideque errors
        assert!(ideque.front().is_err());
        assert!(ideque.back().is_err());
        
        // Add elements
        ideque = ideque.add_front(Value::from(2i64));
        ideque = ideque.add_front(Value::from(1i64));
        ideque = ideque.add_back(Value::from(3i64));
        
        // Test front/back access
        assert_eq!(ideque.front().unwrap(), Value::from(1i64));
        assert_eq!(ideque.back().unwrap(), Value::from(3i64));
        assert_eq!(ideque.len(), 3);
        
        // Test removal
        ideque = ideque.remove_front().unwrap();
        assert_eq!(ideque.front().unwrap(), Value::from(2i64));
        assert_eq!(ideque.len(), 2);
        
        ideque = ideque.remove_back().unwrap();
        assert_eq!(ideque.back().unwrap(), Value::from(2i64));
        assert_eq!(ideque.len(), 1);
    }

    #[test]
    fn test_ideque_to_list() {
        let elements = vec![Value::from(1i64), Value::from(2i64), Value::from(3i64)];
        let ideque = Ideque::from_elements(elements.clone());
        let list = ideque.to_list();
        
        assert_eq!(list.len(), 3);
        assert_eq!(list[0], Value::from(1i64));
        assert_eq!(list[1], Value::from(2i64));
        assert_eq!(list[2], Value::from(3i64));
    }

    #[test]
    fn test_srfi_134_exports() {
        let srfi = Srfi134;
        let exports = srfi.exports();

        // Check all required exports exist
        assert!(exports.contains_key("ideque"));
        assert!(exports.contains_key("ideque?"));
        assert!(exports.contains_key("ideque-empty?"));
        assert!(exports.contains_key("ideque-front"));
        assert!(exports.contains_key("ideque-back"));
        assert!(exports.contains_key("ideque-add-front"));
        assert!(exports.contains_key("ideque-add-back"));
        assert!(exports.contains_key("ideque-remove-front"));
        assert!(exports.contains_key("ideque-remove-back"));
        assert!(exports.contains_key("ideque-length"));
        assert!(exports.contains_key("ideque->list"));
        assert!(exports.contains_key("list->ideque"));
    }

    #[test]
    fn test_ideque_procedures() {
        use crate::value::Procedure;
        
        let srfi = Srfi134;
        let exports = srfi.exports();

        // Test ideque constructor
        let ideque_proc = exports.get("ideque").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = ideque_proc {
            let result = func(&[Value::from(1i64), Value::from(2i64)]).unwrap();
            assert!(matches!(result, Value::Ideque(_)));
        }

        // Test ideque? predicate
        let ideque_pred = exports.get("ideque?").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = ideque_pred {
            let empty_ideque = Value::Ideque(std::rc::Rc::new(Ideque::new()));
            let result = func(&[empty_ideque]).unwrap();
            assert_eq!(result, Value::Boolean(true));
            
            let not_ideque = Value::from(42i64);
            let result = func(&[not_ideque]).unwrap();
            assert_eq!(result, Value::Boolean(false));
        }
    }

    #[test]
    fn test_list_ideque_conversion() {
        use crate::value::PairData;
        use std::rc::Rc;
        use std::cell::RefCell;
        
        let srfi = Srfi134;
        let exports = srfi.exports();

        // Create a simple list: (1 2 3)
        let pair3 = Value::Pair(Rc::new(RefCell::new(PairData {
            car: Value::from(3i64),
            cdr: Value::Nil,
        })));
        let pair2 = Value::Pair(Rc::new(RefCell::new(PairData {
            car: Value::from(2i64),
            cdr: pair3,
        })));
        let list = Value::Pair(Rc::new(RefCell::new(PairData {
            car: Value::from(1i64),
            cdr: pair2,
        })));

        // Test list->ideque conversion
        let list_to_ideque = exports.get("list->ideque").unwrap();
        if let Value::Procedure(crate::value::Procedure::Builtin { func, .. }) = list_to_ideque {
            let result = func(&[list]).unwrap();
            if let Value::Ideque(ideque) = result {
                assert_eq!(ideque.len(), 3);
                assert_eq!(ideque.front().unwrap(), Value::from(1i64));
                assert_eq!(ideque.back().unwrap(), Value::from(3i64));
            } else {
                panic!("Expected ideque result");
            }
        }
    }
}