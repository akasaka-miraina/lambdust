//! SRFI 69: Basic Hash Tables - Conversion operations
//!
//! This module implements conversion operations: alist conversion, copying.

use super::types::{HashKey, HashTable};
// Removed unused imports
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register conversion functions
pub fn register_functions(builtins: &mut HashMap<String, Value>) {
    // Conversion operations
    builtins.insert("hash-table->alist".to_string(), hash_table_to_alist_function());
    builtins.insert("alist->hash-table".to_string(), alist_to_hash_table_function());
    builtins.insert("hash-table-copy".to_string(), hash_table_copy_function());
}

/// Create hash-table->alist function
fn hash_table_to_alist_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table->alist".to_string(),
        arity: Some(1),
        func: hash_table_to_alist,
    })
}

/// Hash-table->alist - convert hash table to association list
pub fn hash_table_to_alist(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let hash_table_ref = match &args[0] {
        Value::HashTable(ht) => ht.borrow(),
        _ => return Err(LambdustError::type_error("Argument must be a hash table".to_string())),
    };

    let pairs: Vec<Value> = hash_table_ref
        .table
        .iter()
        .map(|(k, v)| Value::cons(k.to_value(), v.clone()))
        .collect();

    let mut result = Value::Nil;
    for pair in pairs.into_iter().rev() {
        result = Value::cons(pair, result);
    }
    Ok(result)
}

/// Create alist->hash-table function
fn alist_to_hash_table_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "alist->hash-table".to_string(),
        arity: None, // 1-3 args
        func: alist_to_hash_table,
    })
}

/// Alist->hash-table - convert association list to hash table
pub fn alist_to_hash_table(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let alist = &args[0];
    if !alist.is_list() {
        return Err(LambdustError::type_error("First argument must be a list".to_string()));
    }

    let list_vec = alist
        .to_vector()
        .ok_or_else(|| LambdustError::type_error("First argument must be a proper list"))?;

    let mut hash_table = HashTable::new();

    for item in list_vec {
        if !item.is_pair() {
            return Err(LambdustError::type_error("All list elements must be pairs".to_string()));
        }

        let (key_val, value_val) = item.as_pair()
            .ok_or_else(|| LambdustError::type_error("Invalid pair in association list"))?;

        let key = HashKey::from_value(&key_val)?;
        hash_table.set(key, value_val);
    }

    Ok(Value::HashTable(std::rc::Rc::new(std::cell::RefCell::new(hash_table))))
}

/// Create hash-table-copy function
fn hash_table_copy_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-copy".to_string(),
        arity: Some(1),
        func: hash_table_copy,
    })
}

/// Hash-table-copy - create copy of hash table
pub fn hash_table_copy(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let hash_table_ref = match &args[0] {
        Value::HashTable(ht) => ht.borrow(),
        _ => return Err(LambdustError::type_error("Argument must be a hash table".to_string())),
    };

    let copied_table = hash_table_ref.copy();
    Ok(Value::HashTable(std::rc::Rc::new(std::cell::RefCell::new(copied_table))))
}