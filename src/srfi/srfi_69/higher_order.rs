//! SRFI 69: Basic Hash Tables - Higher-order functions
//!
//! This module implements higher-order functions: walk, fold, merge.

// Removed unused HashTable import
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register higher-order functions
pub fn register_functions(builtins: &mut HashMap<String, Value>) {
    // Higher-order functions (placeholders for evaluator integration)
    builtins.insert("hash-table-walk".to_string(), hash_table_walk_function());
    builtins.insert("hash-table-fold".to_string(), hash_table_fold_function());
    builtins.insert("hash-table-merge!".to_string(), hash_table_merge_function());
}

/// Create hash-table-walk function (placeholder)
fn hash_table_walk_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-walk".to_string(),
        arity: Some(2),
        func: hash_table_walk,
    })
}

/// Hash-table-walk - iterate over hash table entries (placeholder)
pub fn hash_table_walk(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let _hash_table_ref = match &args[0] {
        Value::HashTable(ht) => ht.borrow(),
        _ => return Err(LambdustError::type_error("First argument must be a hash table".to_string())),
    };

    let _procedure = &args[1];

    // This needs evaluator integration to call the procedure
    Err(LambdustError::runtime_error(
        "hash-table-walk requires evaluator integration for procedure calls".to_string(),
    ))
}

/// Create hash-table-fold function (placeholder)
fn hash_table_fold_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-fold".to_string(),
        arity: Some(3),
        func: hash_table_fold,
    })
}

/// Hash-table-fold - fold over hash table entries (placeholder)
pub fn hash_table_fold(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }

    let _hash_table_ref = match &args[0] {
        Value::HashTable(ht) => ht.borrow(),
        _ => return Err(LambdustError::type_error("First argument must be a hash table".to_string())),
    };

    let _procedure = &args[1];
    let _initial = &args[2];

    // This needs evaluator integration to call the procedure
    Err(LambdustError::runtime_error(
        "hash-table-fold requires evaluator integration for procedure calls".to_string(),
    ))
}

/// Create hash-table-merge! function (placeholder)
fn hash_table_merge_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-merge!".to_string(),
        arity: None, // Variable arity
        func: hash_table_merge,
    })
}

/// Hash-table-merge! - merge hash tables
pub fn hash_table_merge(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    // Get mutable reference to destination hash table
    let mut hash_table1_ref = match &args[0] {
        Value::HashTable(ht) => ht.borrow_mut(),
        _ => return Err(LambdustError::type_error("First argument must be a hash table".to_string())),
    };

    // Get source hash tables
    for source_arg in &args[1..] {
        let hash_table2_ref = match source_arg {
            Value::HashTable(ht) => ht.borrow(),
            _ => return Err(LambdustError::type_error("All arguments must be hash tables".to_string())),
        };

        // Simple merge: copy all entries from source to destination
        // In case of key conflicts, the source value overwrites the destination
        for (key, value) in hash_table2_ref.table.iter() {
            hash_table1_ref.table.insert(key.clone(), value.clone());
        }
    }

    Ok(Value::Undefined)
}