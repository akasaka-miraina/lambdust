//! Higher-order functions implementation
//!
//! This module provides higher-order functions like map, for-each, filter, fold, etc.
//! These functions require evaluator integration to call user-defined procedures.

use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register higher-order functions into the builtins map
pub fn register_higher_order_functions(builtins: &mut HashMap<String, Value>) {
    // Core higher-order functions
    builtins.insert("map".to_string(), create_map_function());
    builtins.insert("for-each".to_string(), create_for_each_function());
    builtins.insert("apply".to_string(), create_apply_function());

    // Additional higher-order functions
    builtins.insert("filter".to_string(), create_filter_function());
    builtins.insert("fold".to_string(), create_fold_function());
    builtins.insert("fold-right".to_string(), create_fold_right_function());
}

/// Create map function
///
/// (map proc list1 list2 ...)
fn create_map_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "map".to_string(),
        arity: None, // Variadic: at least 2 arguments
        func: map_implementation,
    })
}

/// Create for-each function
///
/// (for-each proc list1 list2 ...)
fn create_for_each_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "for-each".to_string(),
        arity: None, // Variadic: at least 2 arguments
        func: for_each_implementation,
    })
}

/// Create apply function
///
/// (apply proc args)
/// (apply proc arg1 arg2 ... args)
fn create_apply_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "apply".to_string(),
        arity: None, // Variadic: at least 2 arguments
        func: apply_implementation,
    })
}

/// Create filter function
///
/// (filter pred list)
fn create_filter_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "filter".to_string(),
        arity: Some(2),
        func: filter_implementation,
    })
}

/// Create fold function
///
/// (fold kons knil list1 list2 ...)
fn create_fold_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "fold".to_string(),
        arity: None, // Variadic: at least 3 arguments
        func: fold_implementation,
    })
}

/// Create fold-right function
///
/// (fold-right kons knil list1 list2 ...)
fn create_fold_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "fold-right".to_string(),
        arity: None, // Variadic: at least 3 arguments
        func: fold_right_implementation,
    })
}

/// Internal implementation of map
/// This is a simplified version that works with builtin functions only
pub fn map_implementation(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let proc = &args[0];
    let lists = &args[1..];

    // Validate that the first argument is a procedure
    match proc {
        Value::Procedure(Procedure::Builtin { .. }) => {} // Valid builtin procedure
        Value::Procedure(Procedure::Lambda { .. }) => {
            return Err(LambdustError::runtime_error(
                "map: lambda functions require evaluator integration (not yet implemented)"
                    .to_string(),
            ));
        }
        _ => {
            return Err(LambdustError::type_error(
                "map: first argument must be a procedure".to_string(),
            ));
        }
    }

    // Validate that all arguments (except the first) are lists
    for (i, list) in lists.iter().enumerate() {
        if !list.is_list() {
            return Err(LambdustError::type_error(format!(
                "map: argument {} is not a list",
                i + 2
            )));
        }
    }

    // Convert lists to vectors for easier processing
    let list_vectors: Result<Vec<Vec<Value>>> = lists
        .iter()
        .map(|list| {
            list.to_vector().ok_or_else(|| {
                LambdustError::type_error("map: argument is not a proper list".to_string())
            })
        })
        .collect();
    let list_vectors = list_vectors?;

    // Find the minimum length (map stops at shortest list)
    let min_length = list_vectors.iter().map(|v| v.len()).min().unwrap_or(0);

    let mut results = Vec::new();

    for i in 0..min_length {
        // Collect arguments for this iteration
        let call_args: Vec<Value> = list_vectors.iter().map(|v| v[i].clone()).collect();

        // Call the procedure
        match proc {
            Value::Procedure(Procedure::Builtin { func, .. }) => {
                let result = func(&call_args)?;
                results.push(result);
            }
            Value::Procedure(Procedure::Lambda { .. }) => {
                // For lambda functions, we need evaluator integration
                return Err(LambdustError::runtime_error(
                    "map: lambda functions require evaluator integration (not yet implemented)"
                        .to_string(),
                ));
            }
            _ => {
                return Err(LambdustError::type_error(
                    "map: first argument must be a procedure".to_string(),
                ));
            }
        }
    }

    Ok(Value::from_vector(results))
}

/// Internal implementation of for-each
pub fn for_each_implementation(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let proc = &args[0];
    let lists = &args[1..];

    // Validate that the first argument is a procedure
    match proc {
        Value::Procedure(Procedure::Builtin { .. }) => {} // Valid builtin procedure
        Value::Procedure(Procedure::Lambda { .. }) => {
            return Err(LambdustError::runtime_error(
                "for-each: lambda functions require evaluator integration (not yet implemented)"
                    .to_string(),
            ));
        }
        _ => {
            return Err(LambdustError::type_error(
                "for-each: first argument must be a procedure".to_string(),
            ));
        }
    }

    // Validate that all arguments (except the first) are lists
    for (i, list) in lists.iter().enumerate() {
        if !list.is_list() {
            return Err(LambdustError::type_error(format!(
                "for-each: argument {} is not a list",
                i + 2
            )));
        }
    }

    // Convert lists to vectors for easier processing
    let list_vectors: Result<Vec<Vec<Value>>> = lists
        .iter()
        .map(|list| {
            list.to_vector().ok_or_else(|| {
                LambdustError::type_error("for-each: argument is not a proper list".to_string())
            })
        })
        .collect();
    let list_vectors = list_vectors?;

    // Find the minimum length
    let min_length = list_vectors.iter().map(|v| v.len()).min().unwrap_or(0);

    for i in 0..min_length {
        // Collect arguments for this iteration
        let call_args: Vec<Value> = list_vectors.iter().map(|v| v[i].clone()).collect();

        // Call the procedure (but ignore result)
        match proc {
            Value::Procedure(Procedure::Builtin { func, .. }) => {
                let _ = func(&call_args)?; // Ignore result for for-each
            }
            Value::Procedure(Procedure::Lambda { .. }) => {
                // For lambda functions, we need evaluator integration
                return Err(LambdustError::runtime_error(
                    "for-each: lambda functions require evaluator integration (not yet implemented)".to_string()
                ));
            }
            _ => {
                return Err(LambdustError::type_error(
                    "for-each: first argument must be a procedure".to_string(),
                ));
            }
        }
    }

    Ok(Value::Undefined) // for-each returns unspecified value
}

/// Internal implementation of apply
pub fn apply_implementation(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let proc = &args[0];

    // Validate that the first argument is a procedure
    match proc {
        Value::Procedure(Procedure::Builtin { .. }) => {} // Valid builtin procedure
        Value::Procedure(Procedure::Lambda { .. }) => {
            return Err(LambdustError::runtime_error(
                "apply: lambda functions require evaluator integration (not yet implemented)"
                    .to_string(),
            ));
        }
        _ => {
            return Err(LambdustError::type_error(
                "apply: first argument must be a procedure".to_string(),
            ));
        }
    }

    let mut call_args = Vec::new();

    // Handle different forms of apply:
    // (apply proc args) - args is a list
    // (apply proc arg1 arg2 ... args) - last argument is a list
    if args.len() == 2 {
        // Simple form: (apply proc args)
        let arg_list = &args[1];
        if !arg_list.is_list() {
            return Err(LambdustError::type_error(
                "apply: second argument must be a list".to_string(),
            ));
        }
        call_args = arg_list.to_vector().ok_or_else(|| {
            LambdustError::type_error("apply: second argument must be a proper list".to_string())
        })?;
    } else {
        // Extended form: (apply proc arg1 arg2 ... args)
        call_args.extend_from_slice(&args[1..args.len() - 1]);
        let last_arg = &args[args.len() - 1];
        if !last_arg.is_list() {
            return Err(LambdustError::type_error(
                "apply: last argument must be a list".to_string(),
            ));
        }
        let last_list = last_arg.to_vector().ok_or_else(|| {
            LambdustError::type_error("apply: last argument must be a proper list".to_string())
        })?;
        call_args.extend(last_list);
    }

    // Call the procedure
    match proc {
        Value::Procedure(Procedure::Builtin { func, .. }) => func(&call_args),
        Value::Procedure(Procedure::Lambda { .. }) => {
            // For lambda functions, we need evaluator integration
            Err(LambdustError::runtime_error(
                "apply: lambda functions require evaluator integration (not yet implemented)"
                    .to_string(),
            ))
        }
        _ => Err(LambdustError::type_error(
            "apply: first argument must be a procedure".to_string(),
        )),
    }
}

/// Internal implementation of filter
pub fn filter_implementation(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let predicate = &args[0];
    let list = &args[1];

    // Validate that the first argument is a procedure
    match predicate {
        Value::Procedure(Procedure::Builtin { .. }) => {} // Valid builtin procedure
        Value::Procedure(Procedure::Lambda { .. }) => {
            return Err(LambdustError::runtime_error(
                "filter: lambda functions require evaluator integration (not yet implemented)"
                    .to_string(),
            ));
        }
        _ => {
            return Err(LambdustError::type_error(
                "filter: first argument must be a procedure".to_string(),
            ));
        }
    }

    if !list.is_list() {
        return Err(LambdustError::type_error(
            "filter: second argument must be a list".to_string(),
        ));
    }

    let list_vec = list.to_vector().ok_or_else(|| {
        LambdustError::type_error("filter: second argument must be a proper list".to_string())
    })?;

    let mut results = Vec::new();

    for item in list_vec {
        // Apply predicate to item
        let keep = match predicate {
            Value::Procedure(Procedure::Builtin { func, .. }) => {
                let result = func(&[item.clone()])?;
                result.is_truthy()
            }
            Value::Procedure(Procedure::Lambda { .. }) => {
                // For lambda functions, we need evaluator integration
                return Err(LambdustError::runtime_error(
                    "filter: lambda predicates require evaluator integration (not yet implemented)"
                        .to_string(),
                ));
            }
            _ => {
                return Err(LambdustError::type_error(
                    "filter: first argument must be a procedure".to_string(),
                ));
            }
        };

        if keep {
            results.push(item);
        }
    }

    Ok(Value::from_vector(results))
}

/// Internal implementation of fold (left fold)
pub fn fold_implementation(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }

    let kons = &args[0];
    let mut accumulator = args[1].clone();
    let lists = &args[2..];

    // Validate that the first argument is a procedure
    match kons {
        Value::Procedure(Procedure::Builtin { .. }) => {} // Valid builtin procedure
        Value::Procedure(Procedure::Lambda { .. }) => {
            return Err(LambdustError::runtime_error(
                "fold: lambda functions require evaluator integration (not yet implemented)"
                    .to_string(),
            ));
        }
        _ => {
            return Err(LambdustError::type_error(
                "fold: first argument must be a procedure".to_string(),
            ));
        }
    }

    // Validate that all arguments (after first two) are lists
    for (i, list) in lists.iter().enumerate() {
        if !list.is_list() {
            return Err(LambdustError::type_error(format!(
                "fold: argument {} is not a list",
                i + 3
            )));
        }
    }

    // Convert lists to vectors
    let list_vectors: Result<Vec<Vec<Value>>> = lists
        .iter()
        .map(|list| {
            list.to_vector().ok_or_else(|| {
                LambdustError::type_error("fold: argument is not a proper list".to_string())
            })
        })
        .collect();
    let list_vectors = list_vectors?;

    // Find the minimum length
    let min_length = list_vectors.iter().map(|v| v.len()).min().unwrap_or(0);

    for i in 0..min_length {
        // Prepare arguments: accumulator + elements from each list
        let mut call_args = vec![accumulator];
        for list_vec in &list_vectors {
            call_args.push(list_vec[i].clone());
        }

        // Call the kons function
        accumulator = match kons {
            Value::Procedure(Procedure::Builtin { func, .. }) => func(&call_args)?,
            Value::Procedure(Procedure::Lambda { .. }) => {
                // For lambda functions, we need evaluator integration
                return Err(LambdustError::runtime_error(
                    "fold: lambda functions require evaluator integration (not yet implemented)"
                        .to_string(),
                ));
            }
            _ => {
                return Err(LambdustError::type_error(
                    "fold: first argument must be a procedure".to_string(),
                ));
            }
        };
    }

    Ok(accumulator)
}

/// Internal implementation of fold-right
pub fn fold_right_implementation(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }

    let kons = &args[0];
    let mut accumulator = args[1].clone();
    let lists = &args[2..];

    // Validate that the first argument is a procedure
    match kons {
        Value::Procedure(Procedure::Builtin { .. }) => {} // Valid builtin procedure
        Value::Procedure(Procedure::Lambda { .. }) => {
            return Err(LambdustError::runtime_error(
                "fold-right: lambda functions require evaluator integration (not yet implemented)"
                    .to_string(),
            ));
        }
        _ => {
            return Err(LambdustError::type_error(
                "fold-right: first argument must be a procedure".to_string(),
            ));
        }
    }

    // Validate that all arguments (after first two) are lists
    for (i, list) in lists.iter().enumerate() {
        if !list.is_list() {
            return Err(LambdustError::type_error(format!(
                "fold-right: argument {} is not a list",
                i + 3
            )));
        }
    }

    // Convert lists to vectors
    let list_vectors: Result<Vec<Vec<Value>>> = lists
        .iter()
        .map(|list| {
            list.to_vector().ok_or_else(|| {
                LambdustError::type_error("fold-right: argument is not a proper list".to_string())
            })
        })
        .collect();
    let list_vectors = list_vectors?;

    // Find the minimum length
    let min_length = list_vectors.iter().map(|v| v.len()).min().unwrap_or(0);

    // Process from right to left
    for i in (0..min_length).rev() {
        // Prepare arguments: elements from each list + accumulator
        let mut call_args = Vec::new();
        for list_vec in &list_vectors {
            call_args.push(list_vec[i].clone());
        }
        call_args.push(accumulator);

        // Call the kons function
        accumulator = match kons {
            Value::Procedure(Procedure::Builtin { func, .. }) => func(&call_args)?,
            Value::Procedure(Procedure::Lambda { .. }) => {
                // For lambda functions, we need evaluator integration
                return Err(LambdustError::runtime_error(
                    "fold-right: lambda functions require evaluator integration (not yet implemented)".to_string()
                ));
            }
            _ => {
                return Err(LambdustError::type_error(
                    "fold-right: first argument must be a procedure".to_string(),
                ));
            }
        };
    }

    Ok(accumulator)
}
