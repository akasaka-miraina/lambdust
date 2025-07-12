//! SRFI 69: Basic Hash Tables - Query operations
//!
//! This module implements query operations: exists, size, keys, values.

use super::types::HashKey;
// Removed unused imports
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register query functions
pub fn register_functions(builtins: &mut HashMap<String, Value>) {
    // Query operations
    builtins.insert(
        "hash-table-exists?".to_string(),
        hash_table_exists_function(),
    );
    builtins.insert("hash-table-size".to_string(), hash_table_size_function());
    builtins.insert("hash-table-keys".to_string(), hash_table_keys_function());
    builtins.insert(
        "hash-table-values".to_string(),
        hash_table_values_function(),
    );
}

/// Create hash-table-exists? function
fn hash_table_exists_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-exists?".to_string(),
        arity: Some(2),
        func: hash_table_exists,
    })
}

/// Hash-table-exists? - check if key exists
pub fn hash_table_exists(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let hash_table_ref = match &args[0] {
        Value::HashTable(ht) => ht.borrow(),
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a hash table".to_string(),
            ));
        }
    };

    let key = HashKey::from_value(&args[1])?;
    Ok(Value::Boolean(hash_table_ref.contains_key(&key)))
}

/// Create hash-table-size function
fn hash_table_size_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-size".to_string(),
        arity: Some(1),
        func: hash_table_size,
    })
}

/// Hash-table-size - get number of key-value pairs
pub fn hash_table_size(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let hash_table_ref = match &args[0] {
        Value::HashTable(ht) => ht.borrow(),
        _ => {
            return Err(LambdustError::type_error(
                "Argument must be a hash table".to_string(),
            ));
        }
    };

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
        hash_table_ref.size() as i64,
    )))
}

/// Create hash-table-keys function
fn hash_table_keys_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-keys".to_string(),
        arity: Some(1),
        func: hash_table_keys,
    })
}

/// Hash-table-keys - get list of all keys
pub fn hash_table_keys(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let hash_table_ref = match &args[0] {
        Value::HashTable(ht) => ht.borrow(),
        _ => {
            return Err(LambdustError::type_error(
                "Argument must be a hash table".to_string(),
            ));
        }
    };

    let keys: Vec<Value> = hash_table_ref.keys().iter().map(super::types::HashKey::to_value).collect();
    let mut result = Value::Nil;
    for key in keys.into_iter().rev() {
        result = Value::cons(key, result);
    }
    Ok(result)
}

/// Create hash-table-values function
fn hash_table_values_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-values".to_string(),
        arity: Some(1),
        func: hash_table_values,
    })
}

/// Hash-table-values - get list of all values
pub fn hash_table_values(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let hash_table_ref = match &args[0] {
        Value::HashTable(ht) => ht.borrow(),
        _ => {
            return Err(LambdustError::type_error(
                "Argument must be a hash table".to_string(),
            ));
        }
    };

    let values = hash_table_ref.values();
    let mut result = Value::Nil;
    for value in values.into_iter().rev() {
        result = Value::cons(value, result);
    }
    Ok(result)
}
