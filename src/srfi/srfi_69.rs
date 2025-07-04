//! SRFI 69: Basic Hash Tables implementation
//!
//! This module implements the SRFI 69 Basic Hash Tables, providing
//! comprehensive hash table (dictionary) functionality for R7RS Scheme.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Hash table implementation for SRFI 69
#[derive(Debug, Clone)]
pub struct HashTable {
    /// Internal storage using Rust HashMap
    table: HashMap<HashKey, Value>,
    /// Equality predicate for keys (evaluator integration ready)
    equality_predicate: Option<Value>,
    /// Hash function for keys (evaluator integration ready)
    hash_function: Option<Value>,
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

impl Default for HashTable {
    fn default() -> Self {
        Self {
            table: HashMap::new(),
            equality_predicate: None,
            hash_function: None,
        }
    }
}

impl HashTable {
    /// Create a new hash table
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new hash table with custom equality and hash functions
    pub fn with_functions(equality: Option<Value>, hash: Option<Value>) -> Self {
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

    /// Iterate over all key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&HashKey, &Value)> {
        self.table.iter()
    }

    /// Insert a key-value pair (for merge operations)
    pub fn insert_raw(&mut self, key: HashKey, value: Value) {
        self.table.insert(key, value);
    }

    /// Get all key-value pairs as an association list
    pub fn to_alist(&self) -> Value {
        let pairs: Vec<Value> = self
            .table
            .iter()
            .map(|(k, v)| {
                let key = k.to_value();
                let pair_data = crate::value::PairData::new(key, v.clone());
                Value::Pair(Rc::new(RefCell::new(pair_data)))
            })
            .collect();
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
    builtins.insert(
        "hash-table-ref/default".to_string(),
        hash_table_ref_default_function(),
    );
    builtins.insert("hash-table-set!".to_string(), hash_table_set_function());
    builtins.insert(
        "hash-table-delete!".to_string(),
        hash_table_delete_function(),
    );
    builtins.insert(
        "hash-table-exists?".to_string(),
        hash_table_exists_function(),
    );

    // Hash table information
    builtins.insert("hash-table-size".to_string(), hash_table_size_function());
    builtins.insert("hash-table-keys".to_string(), hash_table_keys_function());
    builtins.insert(
        "hash-table-values".to_string(),
        hash_table_values_function(),
    );
    builtins.insert(
        "hash-table->alist".to_string(),
        hash_table_to_alist_function(),
    );
    builtins.insert(
        "alist->hash-table".to_string(),
        alist_to_hash_table_function(),
    );

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
        // Store the actual procedure value for equality predicate
        Some(args[0].clone())
    };

    let hash_func = if args.len() < 2 {
        None
    } else {
        // Store the actual procedure value for hash function
        Some(args[1].clone())
    };

    let hash_table = HashTable::with_functions(equality, hash_func);
    Ok(Value::HashTable(Rc::new(RefCell::new(hash_table))))
}

/// Create hash-table? function
fn hash_table_predicate_function() -> Value {
    make_builtin_procedure("hash-table?", Some(1), |args| {
        check_arity(args, 1)?;
        Ok(Value::Boolean(matches!(&args[0], Value::HashTable(_))))
    })
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
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a hash table".to_string(),
            ));
        }
    };

    let key = &args[1];
    let default = if args.len() == 3 {
        Some(&args[2])
    } else {
        None
    };

    let ht = hash_table.borrow();
    match ht.get(key)? {
        Some(value) => Ok(value),
        None => {
            if let Some(def) = default {
                Ok(def.clone())
            } else {
                Err(LambdustError::runtime_error(
                    "Key not found in hash table".to_string(),
                ))
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
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a hash table".to_string(),
            ));
        }
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
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a hash table".to_string(),
            ));
        }
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
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a hash table".to_string(),
            ));
        }
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
        _ => {
            return Err(LambdustError::type_error(
                "Argument must be a hash table".to_string(),
            ));
        }
    };

    let ht = hash_table.borrow();
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
        ht.size() as i64,
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

/// Hash-table-keys - get all keys as a list
pub fn hash_table_keys(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let hash_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => {
            return Err(LambdustError::type_error(
                "Argument must be a hash table".to_string(),
            ));
        }
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
        _ => {
            return Err(LambdustError::type_error(
                "Argument must be a hash table".to_string(),
            ));
        }
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
        _ => {
            return Err(LambdustError::type_error(
                "Argument must be a hash table".to_string(),
            ));
        }
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
        return Err(LambdustError::type_error(
            "First argument must be an association list".to_string(),
        ));
    }

    let mut hash_table = HashTable::new();

    let list_vec = alist
        .to_vector()
        .ok_or_else(|| LambdustError::type_error("First argument must be a proper list"))?;

    for item in list_vec {
        match item {
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                let key = pair.car.clone();
                let value = pair.cdr.clone();
                hash_table.set(key, value)?;
            }
            _ => {
                return Err(LambdustError::type_error(
                    "Association list must contain pairs".to_string(),
                ));
            }
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
        _ => {
            return Err(LambdustError::type_error(
                "Argument must be a hash table".to_string(),
            ));
        }
    };

    let ht = hash_table.borrow();
    let copy = ht.clone();
    Ok(Value::HashTable(Rc::new(RefCell::new(copy))))
}

// Placeholder functions for operations that need evaluator integration

/// Create hash-table-walk function
fn hash_table_walk_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-walk".to_string(),
        arity: Some(2),
        func: hash_table_walk,
    })
}

/// Create hash-table-fold function
fn hash_table_fold_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-fold".to_string(),
        arity: Some(3),
        func: hash_table_fold,
    })
}

/// Create hash-table-merge! function
fn hash_table_merge_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "hash-table-merge!".to_string(),
        arity: None, // Variable arity, at least 1
        func: hash_table_merge,
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
            _ => {
                return Err(LambdustError::type_error(
                    "Second argument must be an integer".to_string(),
                ));
            }
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

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
        result as i64,
    )))
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
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let bound = if args.len() == 2 {
        match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as u32,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as u32,
            _ => {
                return Err(LambdustError::type_error(
                    "Second argument must be an integer".to_string(),
                ));
            }
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

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
        result as i64,
    )))
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
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let bound = if args.len() == 2 {
        match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as u32,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as u32,
            _ => {
                return Err(LambdustError::type_error(
                    "Second argument must be an integer".to_string(),
                ));
            }
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

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
        result as i64,
    )))
}

/// SRFI 69 module implementation
pub struct Srfi69;

impl crate::srfi::SrfiModule for Srfi69 {
    fn srfi_id(&self) -> u32 {
        69
    }

    fn name(&self) -> &'static str {
        "Basic Hash Tables"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["all"] // SRFI 69 doesn't have separate parts
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        register_srfi_69_functions(&mut exports);
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
                "constructors" => {
                    // Hash table constructors
                    for name in &["make-hash-table"] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "accessors" => {
                    // Hash table accessors
                    for name in &["hash-table-ref", "hash-table-set!", "hash-table-delete!", 
                                 "hash-table-exists?", "hash-table-size"] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "predicates" => {
                    // Hash table predicates
                    for name in &["hash-table?", "hash-table-exists?"] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "conversion" => {
                    // Conversion functions
                    for name in &["hash-table->alist", "alist->hash-table", "hash-table-copy",
                                 "hash-table-keys", "hash-table-values"] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "hash-functions" => {
                    // Hash functions
                    for name in &["hash", "string-hash", "string-ci-hash"] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "higher-order" => {
                    // Higher-order functions (placeholder for future implementation)
                    for name in &["hash-table-walk", "hash-table-fold", "hash-table-merge!"] {
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
                        "SRFI 69: unknown part '{}'", part
                    )));
                }
            }
        }

        Ok(filtered)
    }
}

/// Hash-table-walk - apply procedure to all key-value pairs (builtin version)
/// Note: This is a placeholder. Full functionality is available as a special form.
pub fn hash_table_walk(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let hash_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a hash table".to_string(),
            ));
        }
    };

    let proc = &args[1];

    // Basic implementation for builtin procedures only
    if let Value::Procedure(crate::value::Procedure::Builtin { func, .. }) = proc {
        let ht = hash_table.borrow();
        for (key, value) in ht.iter() {
            let key_value = key.to_value();
            let call_args = vec![key_value, value.clone()];
            func(&call_args)?;
        }
        Ok(Value::Undefined)
    } else {
        Err(LambdustError::runtime_error(
            "hash-table-walk: lambda procedures require evaluator integration (use as special form)".to_string(),
        ))
    }
}

/// Hash-table-fold - fold over all key-value pairs (builtin version)
/// Note: This is a placeholder. Full functionality is available as a special form.
pub fn hash_table_fold(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }

    let hash_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a hash table".to_string(),
            ));
        }
    };

    let proc = &args[1];
    let mut accumulator = args[2].clone();

    // Basic implementation for builtin procedures only
    if let Value::Procedure(crate::value::Procedure::Builtin { func, .. }) = proc {
        let ht = hash_table.borrow();
        for (key, value) in ht.iter() {
            let key_value = key.to_value();
            let call_args = vec![key_value, value.clone(), accumulator];
            accumulator = func(&call_args)?;
        }
        Ok(accumulator)
    } else {
        Err(LambdustError::runtime_error(
            "hash-table-fold: lambda procedures require evaluator integration (use as special form)".to_string(),
        ))
    }
}

/// Hash-table-merge! - merge multiple hash tables
pub fn hash_table_merge(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let target_table = match &args[0] {
        Value::HashTable(ht) => ht,
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a hash table".to_string(),
            ));
        }
    };

    // Merge all source tables into the target
    for source_arg in &args[1..] {
        let source_table = match source_arg {
            Value::HashTable(ht) => ht,
            _ => {
                return Err(LambdustError::type_error(
                    "All arguments must be hash tables".to_string(),
                ));
            }
        };

        let source = source_table.borrow();
        let mut target = target_table.borrow_mut();

        // Copy all entries from source to target
        for (key, value) in source.iter() {
            target.insert_raw(key.clone(), value.clone());
        }
    }

    Ok(Value::Undefined)
}