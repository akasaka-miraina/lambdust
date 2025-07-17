//! SRFI 113: Sets and Bags
//!
//! This SRFI provides linear-update sets and bags (multisets).

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Set data structure
#[derive(Debug, Clone, PartialEq)]
pub struct Set {
    /// Internal storage using `HashSet` for uniqueness
    elements: HashSet<String>, // Using string representation for simplicity
    /// Value mapping for proper retrieval
    values: HashMap<String, Value>,
}

impl Default for Set {
    fn default() -> Self {
        Self::new()
    }
}

impl Set {
    /// Create a new empty set
    #[must_use] pub fn new() -> Self {
        Self {
            elements: HashSet::new(),
            values: HashMap::new(),
        }
    }

    /// Create a set from a vector of values
    #[must_use] pub fn from_values(values: Vec<Value>) -> Self {
        let mut set = Self::new();
        for value in values {
            set.insert(value);
        }
        set
    }

    /// Insert a value into the set
    pub fn insert(&mut self, value: Value) -> bool {
        let key = format!("{value}");
        let was_new = self.elements.insert(key.clone());
        if was_new {
            self.values.insert(key, value);
        }
        was_new
    }

    /// Check if the set contains a value
    #[must_use] pub fn contains(&self, value: &Value) -> bool {
        let key = format!("{value}");
        self.elements.contains(&key)
    }

    /// Remove a value from the set
    pub fn remove(&mut self, value: &Value) -> bool {
        let key = format!("{value}");
        if self.elements.remove(&key) {
            self.values.remove(&key);
            true
        } else {
            false
        }
    }

    /// Get the size of the set
    #[must_use] pub fn size(&self) -> usize {
        self.elements.len()
    }

    /// Check if the set is empty
    #[must_use] pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Convert to vector of values
    #[must_use] pub fn to_vector(&self) -> Vec<Value> {
        self.values.values().cloned().collect()
    }

    /// Union with another set
    #[must_use] pub fn union(&self, other: &Set) -> Set {
        let mut result = self.clone();
        for value in other.values.values() {
            result.insert(value.clone());
        }
        result
    }

    /// Intersection with another set
    #[must_use] pub fn intersection(&self, other: &Set) -> Set {
        let mut result = Set::new();
        for (key, value) in &self.values {
            if other.elements.contains(key) {
                result.insert(value.clone());
            }
        }
        result
    }

    /// Difference with another set
    #[must_use] pub fn difference(&self, other: &Set) -> Set {
        let mut result = Set::new();
        for (key, value) in &self.values {
            if !other.elements.contains(key) {
                result.insert(value.clone());
            }
        }
        result
    }
}

// Manual Send + Sync implementation for Set
unsafe impl Send for Set {}
unsafe impl Sync for Set {}

/// Bag (multiset) data structure
#[derive(Debug, Clone, PartialEq)]
pub struct Bag {
    /// Internal storage with counts
    counts: HashMap<String, usize>,
    /// Value mapping for proper retrieval
    values: HashMap<String, Value>,
}

impl Default for Bag {
    fn default() -> Self {
        Self::new()
    }
}

impl Bag {
    /// Create a new empty bag
    #[must_use] pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            values: HashMap::new(),
        }
    }

    /// Create a bag from a vector of values
    #[must_use] pub fn from_values(values: Vec<Value>) -> Self {
        let mut bag = Self::new();
        for value in values {
            bag.insert(value);
        }
        bag
    }

    /// Insert a value into the bag
    pub fn insert(&mut self, value: Value) {
        let key = format!("{value}");
        *self.counts.entry(key.clone()).or_insert(0) += 1;
        self.values.entry(key).or_insert(value);
    }

    /// Get the count of a value in the bag
    #[must_use] pub fn count(&self, value: &Value) -> usize {
        let key = format!("{value}");
        self.counts.get(&key).copied().unwrap_or(0)
    }

    /// Remove one instance of a value from the bag
    pub fn remove_one(&mut self, value: &Value) -> bool {
        let key = format!("{value}");
        if let Some(count) = self.counts.get_mut(&key) {
            if *count > 1 {
                *count -= 1;
            } else {
                self.counts.remove(&key);
                self.values.remove(&key);
            }
            true
        } else {
            false
        }
    }

    /// Get the total size of the bag
    #[must_use] pub fn size(&self) -> usize {
        self.counts.values().sum()
    }

    /// Check if the bag is empty
    #[must_use] pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }

    /// Convert to vector of values (with duplicates)
    #[must_use] pub fn to_vector(&self) -> Vec<Value> {
        let mut result = Vec::new();
        for (key, &count) in &self.counts {
            if let Some(value) = self.values.get(key) {
                for _ in 0..count {
                    result.push(value.clone());
                }
            }
        }
        result
    }
}

// Manual Send + Sync implementation for Bag
unsafe impl Send for Bag {}
unsafe impl Sync for Bag {}

/// SRFI 113 implementation
pub struct Srfi113;

impl super::SrfiModule for Srfi113 {
    fn srfi_id(&self) -> u32 {
        113
    }

    fn name(&self) -> &'static str {
        "Sets and Bags"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec![]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // set constructor
        exports.insert(
            "set".to_string(),
            make_builtin_procedure("set", None, |args| {
                let set = Set::from_values(args.to_vec());
                Ok(Value::External(crate::bridge::ExternalObject {
                    id: 0, // Will be assigned by the system
                    type_name: "set".to_string(),
                    data: Arc::new(set),
                }))
            }),
        );

        // set? predicate
        exports.insert(
            "set?".to_string(),
            make_builtin_procedure("set?", Some(1), |args| {
                check_arity(args, 1)?;

                let is_set = match &args[0] {
                    Value::External(obj) => obj.type_name == "set",
                    _ => false,
                };
                Ok(Value::Boolean(is_set))
            }),
        );

        // set-contains? predicate
        exports.insert(
            "set-contains?".to_string(),
            make_builtin_procedure("set-contains?", Some(2), |args| {
                check_arity(args, 2)?;

                if let Value::External(obj) = &args[0] {
                    if obj.type_name == "set" {
                        if let Some(set) = obj.data.downcast_ref::<Set>() {
                            return Ok(Value::Boolean(set.contains(&args[1])));
                        }
                    }
                }
                Err(LambdustError::type_error("Expected set".to_string()))
            }),
        );

        // set-size procedure
        exports.insert(
            "set-size".to_string(),
            make_builtin_procedure("set-size", Some(1), |args| {
                check_arity(args, 1)?;

                if let Value::External(obj) = &args[0] {
                    if obj.type_name == "set" {
                        if let Some(set) = obj.data.downcast_ref::<Set>() {
                            return Ok(Value::from(set.size() as i64));
                        }
                    }
                }
                Err(LambdustError::type_error("Expected set".to_string()))
            }),
        );

        // set-empty? predicate
        exports.insert(
            "set-empty?".to_string(),
            make_builtin_procedure("set-empty?", Some(1), |args| {
                check_arity(args, 1)?;

                if let Value::External(obj) = &args[0] {
                    if obj.type_name == "set" {
                        if let Some(set) = obj.data.downcast_ref::<Set>() {
                            return Ok(Value::Boolean(set.is_empty()));
                        }
                    }
                }
                Err(LambdustError::type_error("Expected set".to_string()))
            }),
        );

        // set->list converter
        exports.insert(
            "set->list".to_string(),
            make_builtin_procedure("set->list", Some(1), |args| {
                check_arity(args, 1)?;

                if let Value::External(obj) = &args[0] {
                    if obj.type_name == "set" {
                        if let Some(set) = obj.data.downcast_ref::<Set>() {
                            return Ok(Value::from_vector(set.to_vector()));
                        }
                    }
                }
                Err(LambdustError::type_error("Expected set".to_string()))
            }),
        );

        // list->set converter
        exports.insert(
            "list->set".to_string(),
            make_builtin_procedure("list->set", Some(1), |args| {
                check_arity(args, 1)?;

                if let Some(vec) = args[0].to_vector() {
                    let set = Set::from_values(vec);
                    Ok(Value::External(crate::bridge::ExternalObject {
                        id: 0,
                        type_name: "set".to_string(),
                        data: Arc::new(set),
                    }))
                } else {
                    Err(LambdustError::type_error("Expected list".to_string()))
                }
            }),
        );

        // bag constructor
        exports.insert(
            "bag".to_string(),
            make_builtin_procedure("bag", None, |args| {
                let bag = Bag::from_values(args.to_vec());
                Ok(Value::External(crate::bridge::ExternalObject {
                    id: 0,
                    type_name: "bag".to_string(),
                    data: Arc::new(bag),
                }))
            }),
        );

        // bag? predicate
        exports.insert(
            "bag?".to_string(),
            make_builtin_procedure("bag?", Some(1), |args| {
                check_arity(args, 1)?;

                let is_bag = match &args[0] {
                    Value::External(obj) => obj.type_name == "bag",
                    _ => false,
                };
                Ok(Value::Boolean(is_bag))
            }),
        );

        // bag-count procedure
        exports.insert(
            "bag-count".to_string(),
            make_builtin_procedure("bag-count", Some(2), |args| {
                check_arity(args, 2)?;

                if let Value::External(obj) = &args[0] {
                    if obj.type_name == "bag" {
                        if let Some(bag) = obj.data.downcast_ref::<Bag>() {
                            return Ok(Value::from(bag.count(&args[1]) as i64));
                        }
                    }
                }
                Err(LambdustError::type_error("Expected bag".to_string()))
            }),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 113 has no parts, return all exports
        Ok(self.exports())
    }
}

