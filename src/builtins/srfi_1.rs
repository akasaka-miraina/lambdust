//! SRFI 1: List Library implementation
//!
//! This module implements the SRFI 1 List Library, providing
//! comprehensive list processing functions for R7RS Scheme.

use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register SRFI 1 functions into the builtins map
pub fn register_srfi_1_functions(builtins: &mut HashMap<String, Value>) {
    // Simple functions that don't need evaluator integration
    builtins.insert("take".to_string(), take_function());
    builtins.insert("drop".to_string(), drop_function());
    builtins.insert("concatenate".to_string(), concatenate_function());
    builtins.insert("delete-duplicates".to_string(), delete_duplicates_function());
    
    // Higher-order functions implemented in higher_order module
    // These are now properly implemented for builtin functions
    // Lambda function support requires future evaluator integration
}

/// Create take function
fn take_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "take".to_string(),
        arity: Some(2),
        func: take,
    })
}

/// Create drop function
fn drop_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "drop".to_string(),
        arity: Some(2),
        func: drop,
    })
}

/// Create concatenate function
fn concatenate_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "concatenate".to_string(),
        arity: None, // Variadic
        func: concatenate,
    })
}

/// Create delete-duplicates function
fn delete_duplicates_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "delete-duplicates".to_string(),
        arity: None, // 1 or 2 args
        func: delete_duplicates,
    })
}

// Higher-order functions are now implemented in the higher_order module
// to avoid duplication and ensure consistent behavior

// Higher-order functions (fold, filter, map, for-each) are now implemented
// in the higher_order module to provide consistent behavior and avoid duplication

/// Take - returns first n elements of list
///
/// (take list n)
pub fn take(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let list = &args[0];
    let n = &args[1];

    if !list.is_list() {
        return Err(LambdustError::type_error("First argument must be a list".to_string()));
    }

    let n_val = match n {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i,
        Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as i64,
        _ => return Err(LambdustError::type_error("Second argument must be an integer".to_string())),
    };

    if n_val < 0 {
        return Err(LambdustError::runtime_error("Cannot take negative number of elements".to_string()));
    }

    let list_vec = list.to_vector().ok_or_else(|| 
        LambdustError::type_error("First argument must be a list"))?;
    let take_count = std::cmp::min(n_val as usize, list_vec.len());
    
    let result: Vec<Value> = list_vec.into_iter().take(take_count).collect();
    Ok(Value::from_vector(result))
}

/// Drop - returns list with first n elements removed
///
/// (drop list n)
pub fn drop(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let list = &args[0];
    let n = &args[1];

    if !list.is_list() {
        return Err(LambdustError::type_error("First argument must be a list".to_string()));
    }

    let n_val = match n {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i,
        Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as i64,
        _ => return Err(LambdustError::type_error("Second argument must be an integer".to_string())),
    };

    if n_val < 0 {
        return Err(LambdustError::runtime_error("Cannot drop negative number of elements".to_string()));
    }

    let list_vec = list.to_vector().ok_or_else(|| 
        LambdustError::type_error("First argument must be a list"))?;
    let drop_count = std::cmp::min(n_val as usize, list_vec.len());
    
    let result: Vec<Value> = list_vec.into_iter().skip(drop_count).collect();
    Ok(Value::from_vector(result))
}

// Additional higher-order functions (find, any, every) will be implemented
// in the higher_order module in future updates

/// List utilities that don't require procedure calls
/// Concatenate lists
///
/// (concatenate lists)
pub fn concatenate(args: &[Value]) -> Result<Value> {
    let mut result = Vec::new();
    
    for arg in args {
        if !arg.is_list() {
            return Err(LambdustError::type_error("All arguments must be lists".to_string()));
        }
        let vec = arg.to_vector().ok_or_else(|| 
            LambdustError::type_error("All arguments must be lists"))?;
        result.extend(vec);
    }
    
    Ok(Value::from_vector(result))
}

/// Remove duplicates from list
///
/// (delete-duplicates list [=])
pub fn delete_duplicates(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let list = &args[0];
    if !list.is_list() {
        return Err(LambdustError::type_error("First argument must be a list".to_string()));
    }

    let list_vec = list.to_vector().ok_or_else(|| 
        LambdustError::type_error("First argument must be a list"))?;
    let mut result = Vec::new();
    
    // Simple implementation using equal? semantics
    for item in list_vec {
        let mut found = false;
        for existing in &result {
            if values_equal(&item, existing)? {
                found = true;
                break;
            }
        }
        if !found {
            result.push(item);
        }
    }
    
    Ok(Value::from_vector(result))
}

/// Helper function to check value equality (simplified)
fn values_equal(a: &Value, b: &Value) -> Result<bool> {
    match (a, b) {
        (Value::Number(n1), Value::Number(n2)) => Ok(n1 == n2),
        (Value::String(s1), Value::String(s2)) => Ok(s1 == s2),
        (Value::Symbol(s1), Value::Symbol(s2)) => Ok(s1 == s2),
        (Value::Boolean(b1), Value::Boolean(b2)) => Ok(b1 == b2),
        (Value::Character(c1), Value::Character(c2)) => Ok(c1 == c2),
        (Value::Nil, Value::Nil) => Ok(true),
        (Value::Pair(pair1), Value::Pair(pair2)) => {
            let p1 = pair1.borrow();
            let p2 = pair2.borrow();
            Ok(values_equal(&p1.car, &p2.car)? && values_equal(&p1.cdr, &p2.cdr)?)
        }
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_take() {
        let list = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3)),
            Value::Number(SchemeNumber::Integer(4)),
        ]);
        let n = Value::Number(SchemeNumber::Integer(2));
        
        let result = take(&[list, n]).unwrap();
        let expected = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
        ]);
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_drop() {
        let list = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3)),
            Value::Number(SchemeNumber::Integer(4)),
        ]);
        let n = Value::Number(SchemeNumber::Integer(2));
        
        let result = drop(&[list, n]).unwrap();
        let expected = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(3)),
            Value::Number(SchemeNumber::Integer(4)),
        ]);
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_concatenate() {
        let list1 = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
        ]);
        let list2 = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(3)),
            Value::Number(SchemeNumber::Integer(4)),
        ]);
        
        let result = concatenate(&[list1, list2]).unwrap();
        let expected = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3)),
            Value::Number(SchemeNumber::Integer(4)),
        ]);
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_delete_duplicates() {
        let list = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(3)),
            Value::Number(SchemeNumber::Integer(2)),
        ]);
        
        let result = delete_duplicates(&[list]).unwrap();
        let expected = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3)),
        ]);
        
        assert_eq!(result, expected);
    }
}