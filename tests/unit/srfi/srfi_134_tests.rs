//! SRFI 134: Immutable Deques - Unit Tests

use lambdust::srfi::srfi_134::{Ideque, Srfi134};
use lambdust::srfi::SrfiModule;
use lambdust::value::Value;
use lambdust::lexer::SchemeNumber;
use std::rc::Rc;

#[test]
fn test_ideque_creation() {
    let empty_ideque = Ideque::new();
    assert!(empty_ideque.is_empty());
    assert_eq!(empty_ideque.len(), 0);

    let elements = vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
    ];
    let ideque = Ideque::from_elements(elements);
    assert!(!ideque.is_empty());
    assert_eq!(ideque.len(), 3);
}

#[test]
fn test_ideque_front_back_operations() {
    let mut ideque = Ideque::new();
    
    // Add elements to front and back
    let val1 = Value::Number(SchemeNumber::Integer(1));
    let val2 = Value::Number(SchemeNumber::Integer(2));
    let val3 = Value::Number(SchemeNumber::Integer(3));
    
    ideque = ideque.add_front(val1.clone());
    ideque = ideque.add_back(val2.clone());
    ideque = ideque.add_front(val3.clone());
    
    assert_eq!(ideque.len(), 3);
    
    // Check front and back
    assert_eq!(ideque.front().unwrap(), val3);
    assert_eq!(ideque.back().unwrap(), val2);
    
    // Remove from front and back
    let ideque_after_remove_front = ideque.remove_front().unwrap();
    assert_eq!(ideque_after_remove_front.len(), 2);
    assert_eq!(ideque_after_remove_front.front().unwrap(), val1);
    
    let ideque_after_remove_back = ideque.remove_back().unwrap();
    assert_eq!(ideque_after_remove_back.len(), 2);
    assert_eq!(ideque_after_remove_back.back().unwrap(), val1);
}

#[test]
fn test_ideque_empty_operations() {
    let empty_ideque = Ideque::new();
    
    // Should error on empty operations
    assert!(empty_ideque.front().is_err());
    assert!(empty_ideque.back().is_err());
    assert!(empty_ideque.remove_front().is_err());
    assert!(empty_ideque.remove_back().is_err());
}

#[test]
fn test_ideque_to_list() {
    let elements = vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
    ];
    let ideque = Ideque::from_elements(elements.clone());
    let list = ideque.to_list();
    
    assert_eq!(list.len(), 3);
    assert_eq!(list, elements);
}

#[test]
fn test_ideque_equality() {
    let elements1 = vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
    ];
    let elements2 = vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
    ];
    let elements3 = vec![
        Value::Number(SchemeNumber::Integer(3)),
        Value::Number(SchemeNumber::Integer(4)),
    ];
    
    let ideque1 = Ideque::from_elements(elements1);
    let ideque2 = Ideque::from_elements(elements2);
    let ideque3 = Ideque::from_elements(elements3);
    
    assert_eq!(ideque1, ideque2);
    assert_ne!(ideque1, ideque3);
}

#[test]
fn test_srfi_134_module() {
    let srfi = Srfi134;
    
    // Test SRFI metadata
    assert_eq!(srfi.srfi_id(), 134);
    assert_eq!(srfi.name(), "Immutable Deques");
    assert_eq!(srfi.parts(), Vec::<&str>::new());
    
    // Test exports
    let exports = srfi.exports();
    
    // Check that all expected functions are exported
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
fn test_ideque_constructor_function() {
    let srfi = Srfi134;
    let exports = srfi.exports();
    
    if let Some(Value::Procedure(proc)) = exports.get("ideque") {
        if let lambdust::value::Procedure::Builtin { func, .. } = proc {
            // Test creating empty ideque
            let empty_args = vec![];
            if let Ok(Value::Ideque(ideque)) = func(&empty_args) {
                assert!(ideque.is_empty());
            } else {
                panic!("ideque constructor should return an ideque");
            }
            
            // Test creating ideque with elements
            let args = vec![
                Value::Number(SchemeNumber::Integer(1)),
                Value::Number(SchemeNumber::Integer(2)),
                Value::Number(SchemeNumber::Integer(3)),
            ];
            if let Ok(Value::Ideque(ideque)) = func(&args) {
                assert_eq!(ideque.len(), 3);
                assert_eq!(ideque.front().unwrap(), Value::Number(SchemeNumber::Integer(1)));
                assert_eq!(ideque.back().unwrap(), Value::Number(SchemeNumber::Integer(3)));
            } else {
                panic!("ideque constructor should return an ideque");
            }
        } else {
            panic!("ideque function should be a builtin procedure");
        }
    } else {
        panic!("ideque function should be a procedure");
    }
}

#[test]
fn test_ideque_predicate_function() {
    let srfi = Srfi134;
    let exports = srfi.exports();
    
    if let Some(Value::Procedure(proc)) = exports.get("ideque?") {
        if let lambdust::value::Procedure::Builtin { func, .. } = proc {
            // Test with ideque
            let ideque = Value::Ideque(Rc::new(Ideque::new()));
            let args = vec![ideque];
            if let Ok(Value::Boolean(result)) = func(&args) {
                assert!(result);
            } else {
                panic!("ideque? should return boolean true for ideque");
            }
            
            // Test with non-ideque
            let non_ideque = Value::Number(SchemeNumber::Integer(42));
            let args = vec![non_ideque];
            if let Ok(Value::Boolean(result)) = func(&args) {
                assert!(!result);
            } else {
                panic!("ideque? should return boolean false for non-ideque");
            }
        } else {
            panic!("ideque? function should be a builtin procedure");
        }
    } else {
        panic!("ideque? function should be a procedure");
    }
}

#[test]
fn test_complex_ideque_operations() {
    let mut ideque = Ideque::new();
    
    // Complex sequence of operations
    let vals = (1..=10).map(|i| Value::Number(SchemeNumber::Integer(i))).collect::<Vec<_>>();
    
    // Add alternating front and back
    for (i, val) in vals.iter().enumerate() {
        if i % 2 == 0 {
            ideque = ideque.add_front(val.clone());
        } else {
            ideque = ideque.add_back(val.clone());
        }
    }
    
    assert_eq!(ideque.len(), 10);
    
    // Remove from both ends
    ideque = ideque.remove_front().unwrap();
    ideque = ideque.remove_back().unwrap();
    
    assert_eq!(ideque.len(), 8);
    
    // Convert to list and verify structure is maintained
    let list = ideque.to_list();
    assert_eq!(list.len(), 8);
}