//! SRFI 146: Mappings
//!
//! This SRFI defines an interface for finite mappings from keys to values.
//! Mappings are similar to hash tables but with a more functional interface
//! and immutable semantics.

use crate::error::{LambdustError, Result};
use crate::value::{Value, Procedure};
use std::collections::HashMap;

/// SRFI 146 module implementation
pub struct Srfi146Module;

impl crate::srfi::SrfiModule for Srfi146Module {
    fn srfi_id(&self) -> u32 {
        146
    }

    fn name(&self) -> &'static str {
        "SRFI 146"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["mappings"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Core mapping operations
        exports.insert("mapping".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping".to_string(), arity: None, func: mapping }));
        exports.insert("mapping-unfold".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-unfold".to_string(), arity: Some(4), func: mapping_unfold }));
        exports.insert("alist->mapping".to_string(), Value::Procedure(Procedure::Builtin { name: "alist->mapping".to_string(), arity: Some(2), func: alist_to_mapping }));
        exports.insert("mapping->alist".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping->alist".to_string(), arity: Some(1), func: mapping_to_alist }));
        
        // Predicates
        exports.insert("mapping?".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping?".to_string(), arity: Some(1), func: mapping_p }));
        exports.insert("mapping-empty?".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-empty?".to_string(), arity: Some(1), func: mapping_empty_p }));
        exports.insert("mapping-contains?".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-contains?".to_string(), arity: Some(2), func: mapping_contains_p }));
        
        // Accessors
        exports.insert("mapping-ref".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-ref".to_string(), arity: None, func: mapping_ref }));
        exports.insert("mapping-ref/default".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-ref/default".to_string(), arity: Some(3), func: mapping_ref_default }));
        exports.insert("mapping-key-comparator".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-key-comparator".to_string(), arity: Some(1), func: mapping_key_comparator }));
        
        // Mutators
        exports.insert("mapping-adjoin".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-adjoin".to_string(), arity: None, func: mapping_adjoin }));
        exports.insert("mapping-set".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-set".to_string(), arity: None, func: mapping_set }));
        exports.insert("mapping-replace".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-replace".to_string(), arity: Some(3), func: mapping_replace }));
        exports.insert("mapping-delete".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-delete".to_string(), arity: None, func: mapping_delete }));
        exports.insert("mapping-delete-all".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-delete-all".to_string(), arity: None, func: mapping_delete_all }));
        exports.insert("mapping-intern".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-intern".to_string(), arity: Some(3), func: mapping_intern }));
        exports.insert("mapping-update".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-update".to_string(), arity: None, func: mapping_update }));
        exports.insert("mapping-update/default".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-update/default".to_string(), arity: Some(4), func: mapping_update_default }));
        
        // The whole mapping
        exports.insert("mapping-size".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-size".to_string(), arity: Some(1), func: mapping_size }));
        exports.insert("mapping-find".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-find".to_string(), arity: Some(3), func: mapping_find }));
        exports.insert("mapping-count".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-count".to_string(), arity: Some(2), func: mapping_count }));
        exports.insert("mapping-any".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-any".to_string(), arity: Some(2), func: mapping_any }));
        exports.insert("mapping-every".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-every".to_string(), arity: Some(2), func: mapping_every }));
        
        // Mapping and folding
        exports.insert("mapping-keys".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-keys".to_string(), arity: Some(1), func: mapping_keys }));
        exports.insert("mapping-values".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-values".to_string(), arity: Some(1), func: mapping_values }));
        exports.insert("mapping-entries".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-entries".to_string(), arity: Some(1), func: mapping_entries }));
        exports.insert("mapping-map".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-map".to_string(), arity: Some(3), func: mapping_map }));
        exports.insert("mapping-for-each".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-for-each".to_string(), arity: Some(2), func: mapping_for_each }));
        exports.insert("mapping-fold".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-fold".to_string(), arity: Some(3), func: mapping_fold }));
        exports.insert("mapping-filter".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-filter".to_string(), arity: Some(2), func: mapping_filter }));
        exports.insert("mapping-remove".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-remove".to_string(), arity: Some(2), func: mapping_remove }));
        exports.insert("mapping-partition".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-partition".to_string(), arity: Some(2), func: mapping_partition }));
        
        // Copying and conversion
        exports.insert("mapping-copy".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-copy".to_string(), arity: Some(1), func: mapping_copy }));
        
        // Submappings
        exports.insert("mapping=?".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping=?".to_string(), arity: None, func: mapping_equal }));
        exports.insert("mapping<?".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping<?".to_string(), arity: None, func: mapping_less }));
        exports.insert("mapping>?".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping>?".to_string(), arity: None, func: mapping_greater }));
        exports.insert("mapping<=?".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping<=?".to_string(), arity: None, func: mapping_less_equal }));
        exports.insert("mapping>=?".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping>=?".to_string(), arity: None, func: mapping_greater_equal }));
        
        // Set theory operations
        exports.insert("mapping-union".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-union".to_string(), arity: None, func: mapping_union }));
        exports.insert("mapping-intersection".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-intersection".to_string(), arity: None, func: mapping_intersection }));
        exports.insert("mapping-difference".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-difference".to_string(), arity: None, func: mapping_difference }));
        exports.insert("mapping-xor".to_string(), Value::Procedure(Procedure::Builtin { name: "mapping-xor".to_string(), arity: None, func: mapping_xor }));
        
        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // For now, return all exports regardless of parts
        Ok(self.exports())
    }
}

impl Srfi146Module {
    /// Creates a new SRFI-146 module instance
    pub fn new() -> Self {
        Self
    }
}

/// Thread-safe value representation for storing in Arc
#[derive(Clone, Debug)]
pub enum SafeValue {
    /// Undefined value
    Undefined,
    /// Boolean values
    Boolean(bool),
    /// String values
    String(String),
    /// Symbol values
    Symbol(String),
    /// Numeric values (simplified representation)
    Integer(i64),
    /// Real number values
    Real(f64),
    /// The empty list
    Nil,
    /// Simplified representation for complex types
    Other(String), // String representation for complex values
}

impl SafeValue {
    /// Convert a Value to SafeValue (thread-safe representation)
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Undefined => SafeValue::Undefined,
            Value::Boolean(b) => SafeValue::Boolean(*b),
            Value::String(s) => SafeValue::String(s.clone()),
            Value::Symbol(s) => SafeValue::Symbol(s.clone()),
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => SafeValue::Integer(*i),
            Value::Number(crate::lexer::SchemeNumber::Real(f)) => SafeValue::Real(*f),
            #[allow(deprecated)]
            Value::Integer(i) => SafeValue::Integer(*i),
            Value::Nil => SafeValue::Nil,
            // For complex types, store string representation
            _ => SafeValue::Other(format!("{:?}", value)),
        }
    }
    
    /// Convert SafeValue back to Value
    pub fn to_value(&self) -> Value {
        match self {
            SafeValue::Undefined => Value::Undefined,
            SafeValue::Boolean(b) => Value::Boolean(*b),
            SafeValue::String(s) => Value::String(s.clone()),
            SafeValue::Symbol(s) => Value::Symbol(s.clone()),
            SafeValue::Integer(i) => Value::Number(crate::lexer::SchemeNumber::Integer(*i)),
            SafeValue::Real(f) => Value::Number(crate::lexer::SchemeNumber::Real(*f)),
            SafeValue::Nil => Value::Nil,
            SafeValue::Other(s) => Value::String(s.clone()), // Convert complex types back to string
        }
    }
}

/// Internal mapping representation (thread-safe)
#[derive(Clone, Debug)]
pub struct Mapping {
    /// Key-value store (thread-safe)
    pub data: HashMap<String, SafeValue>,
    /// Comparator (simplified as string for now)
    pub comparator: String,
}

impl Mapping {
    /// Create a new empty mapping
    pub fn new(comparator: String) -> Self {
        Self {
            data: HashMap::new(),
            comparator,
        }
    }
    
    /// Create a mapping from an association list
    pub fn from_alist(alist: &Value, comparator: String) -> Result<Self> {
        let mut mapping = Self::new(comparator);
        let mut current = alist.clone();
        
        loop {
            match current {
                Value::Nil => break,
                Value::Pair(pair_ref) => {
                    let pair = pair_ref.borrow();
                    // Each element should be a pair (key . value)
                    if let Value::Pair(entry_ref) = &pair.car {
                        let entry = entry_ref.borrow();
                        let key = mapping_key_to_string(&entry.car)?;
                        mapping.data.insert(key, SafeValue::from_value(&entry.cdr));
                    } else {
                        return Err(LambdustError::type_error("expected pair in association list".to_string()));
                    }
                    current = pair.cdr.clone();
                }
                _ => return Err(LambdustError::type_error("expected proper list".to_string())),
            }
        }
        
        Ok(mapping)
    }
    
    /// Convert mapping to association list
    pub fn to_alist(&self) -> Value {
        let mut result = Value::Nil;
        for (key, safe_value) in &self.data {
            let value = safe_value.to_value();
            let key_value = Value::cons(Value::Symbol(key.clone()), value);
            result = Value::cons(key_value, result);
        }
        result
    }
}

/// Helper function to convert a mapping key to string representation
fn mapping_key_to_string(key: &Value) -> Result<String> {
    match key {
        Value::Symbol(s) => Ok(s.clone()),
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(format!("{:?}", n)),
        Value::Character(c) => Ok(c.to_string()),
        _ => Ok(format!("{:?}", key)),
    }
}

/// Create a new mapping
fn mapping(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    let comparator = mapping_key_to_string(&args[0])?;
    let mut mapping = Mapping::new(comparator);
    
    // Process key-value pairs
    if args.len() % 2 != 1 {
        return Err(LambdustError::runtime_error("odd number of arguments required".to_string()));
    }
    
    for chunk in args[1..].chunks(2) {
        if chunk.len() == 2 {
            let key = mapping_key_to_string(&chunk[0])?;
            mapping.data.insert(key, SafeValue::from_value(&chunk[1]));
        }
    }
    
    Ok(Value::External(crate::bridge::ExternalObject {
        id: 0, // Would be assigned by registry
        type_name: "Mapping".to_string(),
        data: std::sync::Arc::new(mapping),
    }))
}

/// Unfold a mapping from a generator procedure
fn mapping_unfold(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("mapping-unfold not yet implemented"))
}

/// Convert association list to mapping
fn alist_to_mapping(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let comparator = mapping_key_to_string(&args[0])?;
    let alist = &args[1];
    
    let mapping = Mapping::from_alist(alist, comparator)?;
    Ok(Value::External(crate::bridge::ExternalObject {
        id: 0, // Would be assigned by registry
        type_name: "Mapping".to_string(),
        data: std::sync::Arc::new(mapping),
    }))
}

/// Convert mapping to association list
fn mapping_to_alist(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let mapping = extract_mapping(&args[0])?;
    Ok(mapping.to_alist())
}

/// Check if value is a mapping
fn mapping_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let is_mapping = extract_mapping(&args[0]).is_ok();
    Ok(Value::Boolean(is_mapping))
}

/// Check if mapping is empty
fn mapping_empty_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let mapping = extract_mapping(&args[0])?;
    Ok(Value::Boolean(mapping.data.is_empty()))
}

/// Check if mapping contains key
fn mapping_contains_p(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let mapping = extract_mapping(&args[0])?;
    let key = mapping_key_to_string(&args[1])?;
    Ok(Value::Boolean(mapping.data.contains_key(&key)))
}

/// Get value from mapping
fn mapping_ref(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(LambdustError::runtime_error("expected 2 or 3 arguments".to_string()));
    }
    
    let mapping = extract_mapping(&args[0])?;
    let key = mapping_key_to_string(&args[1])?;
    
    match mapping.data.get(&key) {
        Some(safe_value) => Ok(safe_value.to_value()),
        None => {
            if args.len() == 3 {
                Ok(args[2].clone()) // Return default value
            } else {
                Err(LambdustError::runtime_error(format!("key not found: {}", key)))
            }
        }
    }
}

/// Get value from mapping with default
fn mapping_ref_default(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }
    
    let mapping = extract_mapping(&args[0])?;
    let key = mapping_key_to_string(&args[1])?;
    let default = &args[2];
    
    Ok(mapping.data.get(&key).map(|v| v.to_value()).unwrap_or_else(|| default.clone()))
}

/// Get key comparator from mapping
fn mapping_key_comparator(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let mapping = extract_mapping(&args[0])?;
    Ok(Value::Symbol(mapping.comparator.clone()))
}

/// Add key-value pairs to mapping (functional)
fn mapping_adjoin(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    let mut mapping = extract_mapping(&args[0])?.clone();
    
    if args.len() % 2 != 1 {
        return Err(LambdustError::runtime_error("odd number of arguments required".to_string()));
    }
    
    // Add new key-value pairs (don't overwrite existing)
    for chunk in args[1..].chunks(2) {
        if chunk.len() == 2 {
            let key = mapping_key_to_string(&chunk[0])?;
            if !mapping.data.contains_key(&key) {
                mapping.data.insert(key, SafeValue::from_value(&chunk[1]));
            }
        }
    }
    
    Ok(Value::External(crate::bridge::ExternalObject {
        id: 0, // Would be assigned by registry
        type_name: "Mapping".to_string(),
        data: std::sync::Arc::new(mapping),
    }))
}

/// Set key-value pairs in mapping (functional)
fn mapping_set(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    let mut mapping = extract_mapping(&args[0])?.clone();
    
    if args.len() % 2 != 1 {
        return Err(LambdustError::runtime_error("odd number of arguments required".to_string()));
    }
    
    // Set key-value pairs (overwrite existing)
    for chunk in args[1..].chunks(2) {
        if chunk.len() == 2 {
            let key = mapping_key_to_string(&chunk[0])?;
            mapping.data.insert(key, SafeValue::from_value(&chunk[1]));
        }
    }
    
    Ok(Value::External(crate::bridge::ExternalObject {
        id: 0, // Would be assigned by registry
        type_name: "Mapping".to_string(),
        data: std::sync::Arc::new(mapping),
    }))
}

/// Replace value in mapping
fn mapping_replace(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }
    
    let mut mapping = extract_mapping(&args[0])?.clone();
    let key = mapping_key_to_string(&args[1])?;
    let value = &args[2];
    
    if mapping.data.contains_key(&key) {
        mapping.data.insert(key, SafeValue::from_value(value));
    }
    
    Ok(Value::External(crate::bridge::ExternalObject {
        id: 0, // Would be assigned by registry
        type_name: "Mapping".to_string(),
        data: std::sync::Arc::new(mapping),
    }))
}

/// Delete keys from mapping
fn mapping_delete(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let mut mapping = extract_mapping(&args[0])?.clone();
    
    for key_val in &args[1..] {
        let key = mapping_key_to_string(key_val)?;
        mapping.data.remove(&key);
    }
    
    Ok(Value::External(crate::bridge::ExternalObject {
        id: 0, // Would be assigned by registry
        type_name: "Mapping".to_string(),
        data: std::sync::Arc::new(mapping),
    }))
}

/// Delete all keys from mapping
fn mapping_delete_all(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let mut mapping = extract_mapping(&args[0])?.clone();
    let keys_list = &args[1];
    
    let mut current = keys_list.clone();
    loop {
        match current {
            Value::Nil => break,
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                let key = mapping_key_to_string(&pair.car)?;
                mapping.data.remove(&key);
                current = pair.cdr.clone();
            }
            _ => return Err(LambdustError::type_error("expected list of keys".to_string())),
        }
    }
    
    Ok(Value::External(crate::bridge::ExternalObject {
        id: 0, // Would be assigned by registry
        type_name: "Mapping".to_string(),
        data: std::sync::Arc::new(mapping),
    }))
}

/// Intern a key in mapping with default value
fn mapping_intern(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }
    
    let mut mapping = extract_mapping(&args[0])?.clone();
    let key = mapping_key_to_string(&args[1])?;
    let default = &args[2];
    
    if !mapping.data.contains_key(&key) {
        mapping.data.insert(key.clone(), SafeValue::from_value(default));
    }
    
    let value = mapping.data.get(&key).unwrap().to_value();
    let new_mapping = Value::External(crate::bridge::ExternalObject {
        id: 0, // Would be assigned by registry
        type_name: "Mapping".to_string(),
        data: std::sync::Arc::new(mapping),
    });
    
    // Return both new mapping and value as multiple values
    Ok(Value::Values(vec![new_mapping, value]))
}

/// Update value in mapping using function
fn mapping_update(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("mapping-update not yet implemented"))
}

/// Update value in mapping using function with default
fn mapping_update_default(args: &[Value]) -> Result<Value> {
    if args.len() != 4 {
        return Err(LambdustError::arity_error(4, args.len()));
    }
    
    let mut mapping = extract_mapping(&args[0])?.clone();
    let key = mapping_key_to_string(&args[1])?;
    let _updater = &args[2]; // Would need to call this procedure
    let default = &args[3];
    
    // Simplified: just set to default for now
    let current_value = mapping.data.get(&key).map(|v| v.to_value()).unwrap_or_else(|| default.clone());
    mapping.data.insert(key, SafeValue::from_value(&current_value));
    
    Ok(Value::External(crate::bridge::ExternalObject {
        id: 0, // Would be assigned by registry
        type_name: "Mapping".to_string(),
        data: std::sync::Arc::new(mapping),
    }))
}

/// Get size of mapping
fn mapping_size(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let mapping = extract_mapping(&args[0])?;
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(mapping.data.len() as i64)))
}

/// Find key-value pair satisfying predicate
fn mapping_find(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("mapping-find not yet implemented"))
}

/// Count key-value pairs satisfying predicate
fn mapping_count(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("mapping-count not yet implemented"))
}

/// Test if any key-value pair satisfies predicate
fn mapping_any(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("mapping-any not yet implemented"))
}

/// Test if every key-value pair satisfies predicate
fn mapping_every(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("mapping-every not yet implemented"))
}

/// Get list of keys from mapping
fn mapping_keys(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let mapping = extract_mapping(&args[0])?;
    let keys: Vec<Value> = mapping.data.keys()
        .map(|k| Value::Symbol(k.clone()))
        .collect();
    Ok(Value::from_vector(keys))
}

/// Get list of values from mapping
fn mapping_values(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let mapping = extract_mapping(&args[0])?;
    let values: Vec<Value> = mapping.data.values().map(|v| v.to_value()).collect();
    Ok(Value::from_vector(values))
}

/// Get key-value pairs from mapping
fn mapping_entries(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let mapping = extract_mapping(&args[0])?;
    let keys: Vec<Value> = mapping.data.keys()
        .map(|k| Value::Symbol(k.clone()))
        .collect();
    let values: Vec<Value> = mapping.data.values().map(|v| v.to_value()).collect();
    
    Ok(Value::Values(vec![Value::from_vector(keys), Value::from_vector(values)]))
}

/// Map function over mapping
fn mapping_map(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("mapping-map not yet implemented"))
}

/// Iterate over mapping with side effects
fn mapping_for_each(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("mapping-for-each not yet implemented"))
}

/// Fold over mapping
fn mapping_fold(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("mapping-fold not yet implemented"))
}

/// Filter mapping by predicate
fn mapping_filter(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("mapping-filter not yet implemented"))
}

/// Remove entries from mapping by predicate
fn mapping_remove(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("mapping-remove not yet implemented"))
}

/// Partition mapping by predicate
fn mapping_partition(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("mapping-partition not yet implemented"))
}

/// Copy a mapping
fn mapping_copy(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let mapping = extract_mapping(&args[0])?.clone();
    Ok(Value::External(crate::bridge::ExternalObject {
        id: 0, // Would be assigned by registry
        type_name: "Mapping".to_string(),
        data: std::sync::Arc::new(mapping),
    }))
}

/// Test mapping equality
fn mapping_equal(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper comparison
    Err(LambdustError::runtime_error("mapping=? not yet implemented"))
}

/// Test mapping less than
fn mapping_less(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper comparison
    Err(LambdustError::runtime_error("mapping<? not yet implemented"))
}

/// Test mapping greater than
fn mapping_greater(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper comparison
    Err(LambdustError::runtime_error("mapping>? not yet implemented"))
}

/// Test mapping less than or equal
fn mapping_less_equal(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper comparison
    Err(LambdustError::runtime_error("mapping<=? not yet implemented"))
}

/// Test mapping greater than or equal
fn mapping_greater_equal(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper comparison
    Err(LambdustError::runtime_error("mapping>=? not yet implemented"))
}

/// Union of mappings
fn mapping_union(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper merging
    Err(LambdustError::runtime_error("mapping-union not yet implemented"))
}

/// Intersection of mappings
fn mapping_intersection(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper merging
    Err(LambdustError::runtime_error("mapping-intersection not yet implemented"))
}

/// Difference of mappings
fn mapping_difference(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper merging
    Err(LambdustError::runtime_error("mapping-difference not yet implemented"))
}

/// XOR of mappings
fn mapping_xor(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper merging
    Err(LambdustError::runtime_error("mapping-xor not yet implemented"))
}

/// Helper function to extract mapping from external object
fn extract_mapping(value: &Value) -> Result<&Mapping> {
    match value {
        Value::External(obj) => {
            obj.data.downcast_ref::<Mapping>()
                .ok_or_else(|| LambdustError::type_error("expected mapping".to_string()))
        }
        _ => Err(LambdustError::type_error("expected mapping".to_string())),
    }
}

