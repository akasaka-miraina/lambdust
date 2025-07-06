//! SRFI 69: Basic Hash Tables - Data types and structures
//!
//! This module defines the core data structures for hash tables.

use crate::error::Result;
use crate::value::Value;
use std::collections::HashMap;

/// Hash table implementation for SRFI 69
#[derive(Debug, Clone)]
pub struct HashTable {
    /// Internal storage using Rust HashMap
    pub table: HashMap<HashKey, Value>,
    /// Equality predicate for keys (evaluator integration ready)
    #[allow(dead_code)]
    pub equality_predicate: Option<Value>,
    /// Hash function for keys (evaluator integration ready)
    #[allow(dead_code)]
    pub hash_function: Option<Value>,
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

    /// Convert hash key back to Scheme value
    pub fn to_value(&self) -> Value {
        match self {
            HashKey::Number(n) => {
                // Try to parse back to number
                if let Ok(i) = n.parse::<i64>() {
                    Value::Number(crate::lexer::SchemeNumber::Integer(i))
                } else if let Ok(f) = n.parse::<f64>() {
                    Value::Number(crate::lexer::SchemeNumber::Real(f))
                } else {
                    Value::String(n.clone())
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
        Self::new()
    }
}

impl HashTable {
    /// Create a new empty hash table
    pub fn new() -> Self {
        HashTable {
            table: HashMap::new(),
            equality_predicate: None,
            hash_function: None,
        }
    }

    /// Create a new hash table with specified predicates
    pub fn with_predicates(equality_predicate: Option<Value>, hash_function: Option<Value>) -> Self {
        HashTable {
            table: HashMap::new(),
            equality_predicate,
            hash_function,
        }
    }

    /// Get value by key
    pub fn get(&self, key: &HashKey) -> Option<&Value> {
        self.table.get(key)
    }

    /// Set key-value pair
    pub fn set(&mut self, key: HashKey, value: Value) {
        self.table.insert(key, value);
    }

    /// Remove key
    pub fn remove(&mut self, key: &HashKey) -> Option<Value> {
        self.table.remove(key)
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &HashKey) -> bool {
        self.table.contains_key(key)
    }

    /// Get size
    pub fn size(&self) -> usize {
        self.table.len()
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<HashKey> {
        self.table.keys().cloned().collect()
    }

    /// Get all values
    pub fn values(&self) -> Vec<Value> {
        self.table.values().cloned().collect()
    }

    /// Clear the hash table
    pub fn clear(&mut self) {
        self.table.clear();
    }

    /// Create a copy of the hash table
    pub fn copy(&self) -> Self {
        HashTable {
            table: self.table.clone(),
            equality_predicate: self.equality_predicate.clone(),
            hash_function: self.hash_function.clone(),
        }
    }
}