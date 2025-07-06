//! SRFI 69: Basic Hash Tables - Core operations
//!
//! This module implements basic hash table operations: creation, access, modification.

use super::types::{HashKey, HashTable};
use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register core hash table functions
pub fn register_functions(builtins: &mut HashMap<String, Value>) {
    // Hash table creation and basic operations
    builtins.insert("make-hash-table".to_string(), make_hash_table_function());
    builtins.insert("hash-table?".to_string(), hash_table_predicate_function());
    builtins.insert("hash-table-ref".to_string(), hash_table_ref_function());
    builtins.insert("hash-table-ref/default".to_string(), hash_table_ref_default_function());
    builtins.insert("hash-table-set!".to_string(), hash_table_set_function());
    builtins.insert("hash-table-delete!".to_string(), hash_table_delete_function());
}

/// Create make-hash-table function
fn make_hash_table_function() -> Value {
    make_builtin_procedure("make-hash-table", None, |args| {
        make_hash_table(args)
    })
}

/// Make-hash-table - create new hash table
pub fn make_hash_table(args: &[Value]) -> Result<Value> {
    // For now, ignore optional equality and hash function arguments
    // In the future, these will be supported with evaluator integration
    if args.len() > 2 {
        return Err(LambdustError::arity_error(0, args.len()));
    }

    let hash_table = if args.is_empty() {
        HashTable::new()
    } else {
        // Arguments provided but not yet supported
        HashTable::new()
    };

    Ok(Value::HashTable(std::rc::Rc::new(std::cell::RefCell::new(hash_table))))
}

/// Create hash-table? predicate function
fn hash_table_predicate_function() -> Value {
    make_builtin_procedure("hash-table?", Some(1), |args| {
        check_arity(args, 1)?;
        Ok(Value::Boolean(matches!(args[0], Value::HashTable(_))))
    })
}

/// Create hash-table-ref function
fn hash_table_ref_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-ref".to_string(),
        arity: Some(2),
        func: hash_table_ref,
    })
}

/// Hash-table-ref - get value from hash table
pub fn hash_table_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let hash_table_ref = match &args[0] {
        Value::HashTable(ht) => ht.borrow(),
        _ => return Err(LambdustError::type_error("First argument must be a hash table".to_string())),
    };

    let key = HashKey::from_value(&args[1])?;

    match hash_table_ref.get(&key) {
        Some(value) => Ok(value.clone()),
        None => Err(LambdustError::runtime_error(format!(
            "Key not found in hash table: {:?}",
            key
        ))),
    }
}

/// Create hash-table-ref/default function
fn hash_table_ref_default_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-ref/default".to_string(),
        arity: Some(3),
        func: hash_table_ref_default,
    })
}

/// Hash-table-ref/default - get value with default
pub fn hash_table_ref_default(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }

    match hash_table_ref(&args[0..2]) {
        Ok(value) => Ok(value),
        Err(_) => Ok(args[2].clone()), // Return default value if key not found
    }
}

/// Create hash-table-set! function
fn hash_table_set_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-set!".to_string(),
        arity: Some(3),
        func: hash_table_set,
    })
}

/// Hash-table-set! - set key-value pair
pub fn hash_table_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }

    let mut hash_table_ref = match &args[0] {
        Value::HashTable(ht) => ht.borrow_mut(),
        _ => return Err(LambdustError::type_error("First argument must be a hash table".to_string())),
    };

    let key = HashKey::from_value(&args[1])?;
    let value = args[2].clone();

    hash_table_ref.set(key, value);
    Ok(Value::Nil)
}

/// Create hash-table-delete! function
fn hash_table_delete_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-delete!".to_string(),
        arity: Some(2),
        func: hash_table_delete,
    })
}

/// Hash-table-delete! - delete key from hash table
pub fn hash_table_delete(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let mut hash_table_ref = match &args[0] {
        Value::HashTable(ht) => ht.borrow_mut(),
        _ => return Err(LambdustError::type_error("First argument must be a hash table".to_string())),
    };

    let key = HashKey::from_value(&args[1])?;

    match hash_table_ref.remove(&key) {
        Some(_) => Ok(Value::Boolean(true)),
        None => Ok(Value::Boolean(false)), // Return false instead of error for missing keys
    }
}