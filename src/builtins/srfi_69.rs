//! SRFI 69: Basic Hash Tables implementation
//!
//! This module implements the SRFI 69 Basic Hash Tables, providing
//! comprehensive hash table (dictionary) functionality for R7RS Scheme.

use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Hash table implementation for SRFI 69
#[derive(Debug, Clone)]
pub struct HashTable {
    /// Internal storage using Rust HashMap
    table: HashMap<HashKey, Value>,
    /// Equality predicate for keys (placeholder for future evaluator integration)
    #[allow(dead_code)]
    equality_predicate: Option<String>,
    /// Hash function for keys (placeholder for future evaluator integration)  
    #[allow(dead_code)]
    hash_function: Option<String>,
}

/// Hash key wrapper to enable using Scheme values as hash keys
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HashKey {
    /// Number key
    Number(String), // Store as string for consistent hashing
    /// String key
    String(String),
    /// Symbol key
    Symbol(String),
    /// Character key
    Character(char),
    /// Boolean key
    Boolean(bool),
    /// Complex key (for other types, using string representation)
    Complex(String),
}

impl HashKey {
    /// Convert a Scheme value to a hash key
    pub fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Number(n) => Ok(HashKey::Number(n.to_string())),
            Value::String(s) => Ok(HashKey::String(s.clone())),
            Value::Symbol(s) => Ok(HashKey::Symbol(s.clone())),
            Value::Character(c) => Ok(HashKey::Character(*c)),
            Value::Boolean(b) => Ok(HashKey::Boolean(*b)),
            _ => Ok(HashKey::Complex(format!("{:?}", value))),
        }
    }

    /// Convert hash key back to a Scheme value
    pub fn to_value(&self) -> Value {
        match self {
            HashKey::Number(s) => {
                // Try to parse back to number
                if let Ok(i) = s.parse::<i64>() {
                    Value::Number(crate::lexer::SchemeNumber::Integer(i))
                } else if let Ok(f) = s.parse::<f64>() {
                    Value::Number(crate::lexer::SchemeNumber::Real(f))
                } else {
                    Value::String(s.clone())
                }
            }
            HashKey::String(s) => Value::String(s.clone()),
            HashKey::Symbol(s) => Value::Symbol(s.clone()),
            HashKey::Character(c) => Value::Character(*c),
            HashKey::Boolean(b) => Value::Boolean(*b),
            HashKey::Complex(s) => Value::String(s.clone()),
        }
    }
}

impl HashTable {
    /// Create a new hash table
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            equality_predicate: None,
            hash_function: None,
        }
    }

    /// Create a new hash table with custom equality and hash functions
    pub fn with_functions(equality: Option<String>, hash: Option<String>) -> Self {
        Self {
            table: HashMap::new(),
            equality_predicate: equality,
            hash_function: hash,
        }
    }

    /// Get the number of key-value pairs in the hash table
    pub fn size(&self) -> usize {
        self.table.len()
    }

    /// Check if the hash table is empty
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Get a value by key
    pub fn get(&self, key: &Value) -> Result<Option<Value>> {
        let hash_key = HashKey::from_value(key)?;
        Ok(self.table.get(&hash_key).cloned())
    }

    /// Set a key-value pair
    pub fn set(&mut self, key: Value, value: Value) -> Result<()> {
        let hash_key = HashKey::from_value(&key)?;
        self.table.insert(hash_key, value);
        Ok(())
    }

    /// Remove a key-value pair
    pub fn remove(&mut self, key: &Value) -> Result<Option<Value>> {
        let hash_key = HashKey::from_value(key)?;
        Ok(self.table.remove(&hash_key))
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &Value) -> Result<bool> {
        let hash_key = HashKey::from_value(key)?;
        Ok(self.table.contains_key(&hash_key))
    }

    /// Get all keys as a list
    pub fn keys(&self) -> Value {
        let keys: Vec<Value> = self.table.keys().map(|k| k.to_value()).collect();
        Value::from_vector(keys)
    }

    /// Get all values as a list
    pub fn values(&self) -> Value {
        let values: Vec<Value> = self.table.values().cloned().collect();
        Value::from_vector(values)
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.table.clear();
    }

    /// Get all key-value pairs as an association list
    pub fn to_alist(&self) -> Value {
        let pairs: Vec<Value> = self.table.iter().map(|(k, v)| {
            let key = k.to_value();
            let pair_data = crate::value::PairData::new(key, v.clone());
            Value::Pair(Rc::new(RefCell::new(pair_data)))
        }).collect();
        Value::from_vector(pairs)
    }
}

/// Register SRFI 69 functions into the builtins map
pub fn register_srfi_69_functions(builtins: &mut HashMap<String, Value>) {
    // Hash table constructors
    builtins.insert("make-hash-table".to_string(), make_hash_table_function());
    builtins.insert("hash-table?".to_string(), hash_table_predicate_function());
    
    // Hash table access
    builtins.insert("hash-table-ref".to_string(), hash_table_ref_function());
    builtins.insert("hash-table-ref/default".to_string(), hash_table_ref_default_function());
    builtins.insert("hash-table-set!".to_string(), hash_table_set_function());
    builtins.insert("hash-table-delete!".to_string(), hash_table_delete_function());
    builtins.insert("hash-table-exists?".to_string(), hash_table_exists_function());
    
    // Hash table information
    builtins.insert("hash-table-size".to_string(), hash_table_size_function());
    builtins.insert("hash-table-keys".to_string(), hash_table_keys_function());
    builtins.insert("hash-table-values".to_string(), hash_table_values_function());
    builtins.insert("hash-table->alist".to_string(), hash_table_to_alist_function());
    builtins.insert("alist->hash-table".to_string(), alist_to_hash_table_function());
    
    // Hash table operations
    builtins.insert("hash-table-walk".to_string(), hash_table_walk_function());
    builtins.insert("hash-table-fold".to_string(), hash_table_fold_function());
    builtins.insert("hash-table-copy".to_string(), hash_table_copy_function());
    builtins.insert("hash-table-merge!".to_string(), hash_table_merge_function());
    
    // Utilities
    builtins.insert("hash".to_string(), hash_function());
    builtins.insert("string-hash".to_string(), string_hash_function());
    builtins.insert("string-ci-hash".to_string(), string_ci_hash_function());
}

/// Create make-hash-table function
fn make_hash_table_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "make-hash-table".to_string(),
        arity: None, // 0-2 args
        func: make_hash_table,
    })
}

/// Make-hash-table - create a new hash table
pub fn make_hash_table(args: &[Value]) -> Result<Value> {
    if args.len() > 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let equality = if args.is_empty() {
        None
    } else {
        // For now, we'll store the function name as a string
        // In a full implementation, this would be a procedure reference
        Some("equal?".to_string())
    };

    let hash_func = if args.len() < 2 {
        None
    } else {
        Some("hash".to_string())
    };

    let hash_table = HashTable::with_functions(equality, hash_func);
    Ok(Value::HashTable(Rc::new(RefCell::new(hash_table))))
}

/// Create hash-table? function
fn hash_table_predicate_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table?".to_string(),
        arity: Some(1),
        func: hash_table_predicate,
    })
}

/// Hash-table? - test if value is a hash table
pub fn hash_table_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    Ok(Value::Boolean(matches!(&args[0], Value::HashTable(_))))
}

/// Create hash-table-ref function
fn hash_table_ref_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-ref".to_string(),
        arity: None, // 2-3 args
        func: hash_table_ref,
    })
}

/// Hash-table-ref - get value by key
pub fn hash_table_ref(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let hash_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => return Err(LambdustError::type_error("First argument must be a hash table".to_string())),
    };

    let key = &args[1];
    let default = if args.len() == 3 { Some(&args[2]) } else { None };

    let ht = hash_table.borrow();
    match ht.get(key)? {
        Some(value) => Ok(value),
        None => {
            if let Some(def) = default {
                Ok(def.clone())
            } else {
                Err(LambdustError::runtime_error("Key not found in hash table".to_string()))
            }
        }
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

/// Hash-table-ref/default - get value by key with default
pub fn hash_table_ref_default(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }

    // This is the same as hash-table-ref with 3 arguments
    hash_table_ref(args)
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

    let hash_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => return Err(LambdustError::type_error("First argument must be a hash table".to_string())),
    };

    let key = args[1].clone();
    let value = args[2].clone();

    let mut ht = hash_table.borrow_mut();
    ht.set(key, value)?;
    Ok(Value::Undefined)
}

/// Create hash-table-delete! function
fn hash_table_delete_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-delete!".to_string(),
        arity: Some(2),
        func: hash_table_delete,
    })
}

/// Hash-table-delete! - remove key-value pair
pub fn hash_table_delete(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let hash_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => return Err(LambdustError::type_error("First argument must be a hash table".to_string())),
    };

    let key = &args[1];

    let mut ht = hash_table.borrow_mut();
    match ht.remove(key)? {
        Some(_) => Ok(Value::Boolean(true)),
        None => Ok(Value::Boolean(false)),
    }
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

    let hash_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => return Err(LambdustError::type_error("First argument must be a hash table".to_string())),
    };

    let key = &args[1];

    let ht = hash_table.borrow();
    Ok(Value::Boolean(ht.contains_key(key)?))
}

/// Create hash-table-size function
fn hash_table_size_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-size".to_string(),
        arity: Some(1),
        func: hash_table_size,
    })
}

/// Hash-table-size - get number of entries
pub fn hash_table_size(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let hash_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => return Err(LambdustError::type_error("Argument must be a hash table".to_string())),
    };

    let ht = hash_table.borrow();
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(ht.size() as i64)))
}

/// Create hash-table-keys function
fn hash_table_keys_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-keys".to_string(),
        arity: Some(1),
        func: hash_table_keys,
    })
}

/// Hash-table-keys - get all keys as a list
pub fn hash_table_keys(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let hash_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => return Err(LambdustError::type_error("Argument must be a hash table".to_string())),
    };

    let ht = hash_table.borrow();
    Ok(ht.keys())
}

/// Create hash-table-values function
fn hash_table_values_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-values".to_string(),
        arity: Some(1),
        func: hash_table_values,
    })
}

/// Hash-table-values - get all values as a list
pub fn hash_table_values(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let hash_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => return Err(LambdustError::type_error("Argument must be a hash table".to_string())),
    };

    let ht = hash_table.borrow();
    Ok(ht.values())
}

/// Create hash-table->alist function
fn hash_table_to_alist_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table->alist".to_string(),
        arity: Some(1),
        func: hash_table_to_alist,
    })
}

/// Hash-table->alist - convert to association list
pub fn hash_table_to_alist(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let hash_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => return Err(LambdustError::type_error("Argument must be a hash table".to_string())),
    };

    let ht = hash_table.borrow();
    Ok(ht.to_alist())
}

/// Create alist->hash-table function
fn alist_to_hash_table_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "alist->hash-table".to_string(),
        arity: None, // 1-3 args
        func: alist_to_hash_table,
    })
}

/// Alist->hash-table - create hash table from association list
pub fn alist_to_hash_table(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let alist = &args[0];
    if !alist.is_list() {
        return Err(LambdustError::type_error("First argument must be an association list".to_string()));
    }

    let mut hash_table = HashTable::new();
    
    let list_vec = alist.to_vector().ok_or_else(|| 
        LambdustError::type_error("First argument must be a proper list"))?;
    
    for item in list_vec {
        match item {
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                let key = pair.car.clone();
                let value = pair.cdr.clone();
                hash_table.set(key, value)?;
            }
            _ => return Err(LambdustError::type_error("Association list must contain pairs".to_string())),
        }
    }

    Ok(Value::HashTable(Rc::new(RefCell::new(hash_table))))
}

/// Create hash-table-copy function
fn hash_table_copy_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-copy".to_string(),
        arity: Some(1),
        func: hash_table_copy,
    })
}

/// Hash-table-copy - create a copy of hash table
pub fn hash_table_copy(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let hash_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => return Err(LambdustError::type_error("Argument must be a hash table".to_string())),
    };

    let ht = hash_table.borrow();
    let copy = ht.clone();
    Ok(Value::HashTable(Rc::new(RefCell::new(copy))))
}

// Placeholder functions for operations that need evaluator integration

/// Create hash-table-walk function (placeholder)
fn hash_table_walk_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-walk".to_string(),
        arity: Some(2),
        func: |_args| Err(LambdustError::runtime_error(
            "hash-table-walk requires evaluator integration for procedure calls".to_string()
        )),
    })
}

/// Create hash-table-fold function (placeholder)
fn hash_table_fold_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-fold".to_string(),
        arity: Some(3),
        func: |_args| Err(LambdustError::runtime_error(
            "hash-table-fold requires evaluator integration for procedure calls".to_string()
        )),
    })
}

/// Create hash-table-merge! function (placeholder)
fn hash_table_merge_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-merge!".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error(
            "hash-table-merge! not yet implemented".to_string()
        )),
    })
}

/// Create hash function
fn hash_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash".to_string(),
        arity: None, // 1-2 args
        func: hash_value,
    })
}

/// Hash - compute hash of a value
pub fn hash_value(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let value = &args[0];
    let bound = if args.len() == 2 {
        match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as u32,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as u32,
            _ => return Err(LambdustError::type_error("Second argument must be an integer".to_string())),
        }
    } else {
        u32::MAX
    };

    // Simple hash implementation
    let hash_str = format!("{:?}", value);
    let mut hash: u32 = 0;
    for c in hash_str.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(c as u32);
    }

    let result = if bound != u32::MAX {
        hash % bound
    } else {
        hash
    };

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result as i64)))
}

/// Create string-hash function
fn string_hash_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-hash".to_string(),
        arity: None, // 1-2 args
        func: string_hash_impl,
    })
}

/// String-hash implementation
pub fn string_hash_impl(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(LambdustError::type_error("First argument must be a string".to_string())),
    };

    let bound = if args.len() == 2 {
        match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as u32,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as u32,
            _ => return Err(LambdustError::type_error("Second argument must be an integer".to_string())),
        }
    } else {
        u32::MAX
    };

    // Simple string hash implementation
    let mut hash: u32 = 0;
    for c in string.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(c as u32);
    }

    let result = if bound != u32::MAX {
        hash % bound
    } else {
        hash
    };

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result as i64)))
}

/// Create string-ci-hash function
fn string_ci_hash_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-ci-hash".to_string(),
        arity: None, // 1-2 args
        func: string_ci_hash_impl,
    })
}

/// String-ci-hash implementation
pub fn string_ci_hash_impl(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s.to_lowercase(),
        _ => return Err(LambdustError::type_error("First argument must be a string".to_string())),
    };

    let bound = if args.len() == 2 {
        match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as u32,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as u32,
            _ => return Err(LambdustError::type_error("Second argument must be an integer".to_string())),
        }
    } else {
        u32::MAX
    };

    // Hash the lowercase version
    let mut hash: u32 = 0;
    for c in string.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(c as u32);
    }

    let result = if bound != u32::MAX {
        hash % bound
    } else {
        hash
    };

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result as i64)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_hash_table_creation() {
        let result = make_hash_table(&[]).unwrap();
        assert!(matches!(result, Value::HashTable(_)));
    }

    #[test]
    fn test_hash_table_predicate() {
        let ht = make_hash_table(&[]).unwrap();
        let result = hash_table_predicate(&[ht]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = hash_table_predicate(&[Value::String("not a hash table".to_string())]).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_hash_table_set_and_ref() {
        let ht = make_hash_table(&[]).unwrap();
        let key = Value::String("test-key".to_string());
        let value = Value::Number(SchemeNumber::Integer(42));

        // Set value
        let result = hash_table_set(&[ht.clone(), key.clone(), value.clone()]);
        assert!(result.is_ok());

        // Get value
        let result = hash_table_ref(&[ht, key]).unwrap();
        assert_eq!(result, value);
    }

    #[test]
    fn test_hash_table_size() {
        let ht = make_hash_table(&[]).unwrap();
        
        // Initially empty
        let result = hash_table_size(&[ht.clone()]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(0)));

        // Add one item
        let key = Value::String("test".to_string());
        let value = Value::Number(SchemeNumber::Integer(123));
        hash_table_set(&[ht.clone(), key, value]).unwrap();

        let result = hash_table_size(&[ht]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));
    }

    #[test]
    fn test_hash_table_exists() {
        let ht = make_hash_table(&[]).unwrap();
        let key = Value::String("test-key".to_string());
        let value = Value::Number(SchemeNumber::Integer(42));

        // Key doesn't exist initially
        let result = hash_table_exists(&[ht.clone(), key.clone()]).unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Set value
        hash_table_set(&[ht.clone(), key.clone(), value]).unwrap();

        // Key now exists
        let result = hash_table_exists(&[ht, key]).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_hash_table_delete() {
        let ht = make_hash_table(&[]).unwrap();
        let key = Value::String("test-key".to_string());
        let value = Value::Number(SchemeNumber::Integer(42));

        // Set value
        hash_table_set(&[ht.clone(), key.clone(), value]).unwrap();

        // Delete value
        let result = hash_table_delete(&[ht.clone(), key.clone()]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Key no longer exists
        let result = hash_table_exists(&[ht, key]).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_hash_table_keys_and_values() {
        let ht = make_hash_table(&[]).unwrap();
        let key1 = Value::String("key1".to_string());
        let key2 = Value::String("key2".to_string());
        let value1 = Value::Number(SchemeNumber::Integer(1));
        let value2 = Value::Number(SchemeNumber::Integer(2));

        // Add items
        hash_table_set(&[ht.clone(), key1, value1]).unwrap();
        hash_table_set(&[ht.clone(), key2, value2]).unwrap();

        // Get keys
        let keys = hash_table_keys(&[ht.clone()]).unwrap();
        assert!(keys.is_list());

        // Get values
        let values = hash_table_values(&[ht]).unwrap();
        assert!(values.is_list());
    }

    #[test]
    fn test_hash_table_copy() {
        let ht = make_hash_table(&[]).unwrap();
        let key = Value::String("test-key".to_string());
        let value = Value::Number(SchemeNumber::Integer(42));

        // Set value in original
        hash_table_set(&[ht.clone(), key.clone(), value.clone()]).unwrap();

        // Copy hash table
        let copy = hash_table_copy(&[ht]).unwrap();

        // Value should exist in copy
        let result = hash_table_ref(&[copy, key]).unwrap();
        assert_eq!(result, value);
    }

    #[test]
    fn test_hash_value() {
        let value = Value::String("test".to_string());
        let result = hash_value(&[value]).unwrap();
        assert!(matches!(result, Value::Number(_)));

        let value = Value::String("test".to_string());
        let bound = Value::Number(SchemeNumber::Integer(1000));
        let result = hash_value(&[value, bound]).unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_string_hash() {
        let string = Value::String("hello".to_string());
        let result = string_hash_impl(&[string]).unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_string_ci_hash() {
        let string1 = Value::String("Hello".to_string());
        let string2 = Value::String("HELLO".to_string());
        
        let hash1 = string_ci_hash_impl(&[string1]).unwrap();
        let hash2 = string_ci_hash_impl(&[string2]).unwrap();
        
        // Case-insensitive hashes should be equal
        assert_eq!(hash1, hash2);
    }
}