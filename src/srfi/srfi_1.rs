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
    builtins.insert(
        "delete-duplicates".to_string(),
        delete_duplicates_function(),
    );

    // Higher-order functions implemented in higher_order module
    // These are now properly implemented for builtin functions
    // Lambda function support requires future evaluator integration

    // Note: fold, fold-right, filter are now handled as special forms in the evaluator
    // for full lambda integration support
    // For SRFI import compatibility, we also register placeholder builtin versions
    builtins.insert("fold".to_string(), fold_placeholder_function());
    builtins.insert("fold-right".to_string(), fold_right_placeholder_function());
    builtins.insert("map".to_string(), map_placeholder_function());
    builtins.insert("filter".to_string(), filter_placeholder_function());

    builtins.insert("find".to_string(), find_function());
    builtins.insert("any".to_string(), any_function());
    builtins.insert("every".to_string(), every_function());
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
        return Err(LambdustError::type_error(
            "First argument must be a list".to_string(),
        ));
    }

    let n_val = match n {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i,
        Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as i64,
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be an integer".to_string(),
            ));
        }
    };

    if n_val < 0 {
        return Err(LambdustError::runtime_error(
            "Cannot take negative number of elements".to_string(),
        ));
    }

    let list_vec = list
        .to_vector()
        .ok_or_else(|| LambdustError::type_error("First argument must be a list"))?;
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
        return Err(LambdustError::type_error(
            "First argument must be a list".to_string(),
        ));
    }

    let n_val = match n {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i,
        Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as i64,
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be an integer".to_string(),
            ));
        }
    };

    if n_val < 0 {
        return Err(LambdustError::runtime_error(
            "Cannot drop negative number of elements".to_string(),
        ));
    }

    let list_vec = list
        .to_vector()
        .ok_or_else(|| LambdustError::type_error("First argument must be a list"))?;
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
            return Err(LambdustError::type_error(
                "All arguments must be lists".to_string(),
            ));
        }
        let vec = arg
            .to_vector()
            .ok_or_else(|| LambdustError::type_error("All arguments must be lists"))?;
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
        return Err(LambdustError::type_error(
            "First argument must be a list".to_string(),
        ));
    }

    let list_vec = list
        .to_vector()
        .ok_or_else(|| LambdustError::type_error("First argument must be a list"))?;
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

/// Create find function (placeholder implementation)
fn find_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "find".to_string(),
        arity: Some(2),
        func: find_placeholder,
    })
}

/// Create any function (placeholder implementation)
fn any_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "any".to_string(),
        arity: Some(2),
        func: any_placeholder,
    })
}

/// Create every function (placeholder implementation)
fn every_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "every".to_string(),
        arity: Some(2),
        func: every_placeholder,
    })
}

/// Placeholder implementation for find
/// TODO: Implement proper evaluator integration for lambda support
fn find_placeholder(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    Err(LambdustError::runtime_error(
        "find: lambda functions require evaluator integration (not yet implemented)".to_string(),
    ))
}

/// Placeholder implementation for any
/// TODO: Implement proper evaluator integration for lambda support
fn any_placeholder(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    Err(LambdustError::runtime_error(
        "any: lambda functions require evaluator integration (not yet implemented)".to_string(),
    ))
}

/// Placeholder implementation for every
/// TODO: Implement proper evaluator integration for lambda support
fn every_placeholder(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    Err(LambdustError::runtime_error(
        "every: lambda functions require evaluator integration (not yet implemented)".to_string(),
    ))
}

/// Create fold placeholder function
fn fold_placeholder_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "fold".to_string(),
        arity: Some(3),
        func: fold_placeholder,
    })
}

/// Create fold-right placeholder function  
fn fold_right_placeholder_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "fold-right".to_string(),
        arity: Some(3),
        func: fold_right_placeholder,
    })
}

/// Create map placeholder function
fn map_placeholder_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "map".to_string(),
        arity: None, // Variable arity
        func: map_placeholder,
    })
}

/// Create filter placeholder function
fn filter_placeholder_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "filter".to_string(),
        arity: Some(2),
        func: filter_placeholder,
    })
}

/// Placeholder implementation for fold
/// TODO: Implement proper evaluator integration for lambda support
fn fold_placeholder(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }

    Err(LambdustError::runtime_error(
        "fold: lambda functions require evaluator integration (not yet implemented)".to_string(),
    ))
}

/// Placeholder implementation for fold-right
/// TODO: Implement proper evaluator integration for lambda support
fn fold_right_placeholder(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }

    Err(LambdustError::runtime_error(
        "fold-right: lambda functions require evaluator integration (not yet implemented)"
            .to_string(),
    ))
}

/// Placeholder implementation for map
/// TODO: Implement proper evaluator integration for lambda support
fn map_placeholder(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    Err(LambdustError::runtime_error(
        "map: lambda functions require evaluator integration (not yet implemented)".to_string(),
    ))
}

/// Placeholder implementation for filter
/// TODO: Implement proper evaluator integration for lambda support
fn filter_placeholder(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    Err(LambdustError::runtime_error(
        "filter: lambda functions require evaluator integration (not yet implemented)".to_string(),
    ))
}

/// SRFI 1 module implementation
pub struct Srfi1;

impl crate::srfi::SrfiModule for Srfi1 {
    fn srfi_id(&self) -> u32 {
        1
    }

    fn name(&self) -> &'static str {
        "List Library"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["all"] // SRFI 1 doesn't have separate parts
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        register_srfi_1_functions(&mut exports);
        exports
    }

    fn exports_for_parts(&self, parts: &[&str]) -> Result<HashMap<String, Value>> {
        if parts.contains(&"all") {
            return Ok(self.exports());
        }

        let all_exports = self.exports();
        let mut filtered = HashMap::new();

        for part in parts {
            match *part {
                "fold" => {
                    // Fold operations
                    if let Some(value) = all_exports.get("fold") {
                        filtered.insert("fold".to_string(), value.clone());
                    }
                    if let Some(value) = all_exports.get("fold-right") {
                        filtered.insert("fold-right".to_string(), value.clone());
                    }
                }
                "map" => {
                    // Map operation
                    if let Some(value) = all_exports.get("map") {
                        filtered.insert("map".to_string(), value.clone());
                    }
                }
                "filter" => {
                    // Filter operation
                    if let Some(value) = all_exports.get("filter") {
                        filtered.insert("filter".to_string(), value.clone());
                    }
                }
                "list-lib" => {
                    // List manipulation functions
                    for name in &["take", "drop", "concatenate", "delete-duplicates"] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "higher-order" => {
                    // Higher-order functions
                    for name in &[
                        "fold",
                        "fold-right",
                        "map",
                        "filter",
                        "find",
                        "any",
                        "every",
                    ] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                // Individual function names
                name if all_exports.contains_key(name) => {
                    if let Some(value) = all_exports.get(name) {
                        filtered.insert(name.to_string(), value.clone());
                    }
                }
                _ => {
                    return Err(LambdustError::runtime_error(format!(
                        "SRFI 1: unknown part '{}'",
                        part
                    )));
                }
            }
        }

        Ok(filtered)
    }
}
