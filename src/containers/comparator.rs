//! Comparator implementations for container data structures.
//!
//! This module provides flexible comparator abstractions that support
//! both built-in and custom comparison functions, as well as hash functions
//! for use with hash-based data structures.

use crate::eval::value::Value;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Type alias for comparison functions
type CompareFn = Arc<dyn Fn(&Value, &Value) -> Ordering + Send + Sync>;

/// Type alias for hash functions  
type HashFn = Arc<dyn Fn(&Value) -> u64 + Send + Sync>;

/// Type alias for equality functions
type EqualityFn = Arc<dyn Fn(&Value, &Value) -> bool + Send + Sync>;

/// A comparator that can compare and hash values
#[derive(Clone)]
pub struct Comparator {
    /// Comparison function
    compare_fn: CompareFn,
    /// Hash function
    hash_fn: HashFn,
    /// Name for debugging
    name: String,
}

impl Comparator {
    /// Creates a new comparator with custom comparison and hash functions
    pub fn new<C, H>(name: impl Into<String>, compare_fn: C, hash_fn: H) -> Self
    where
        C: Fn(&Value, &Value) -> Ordering + Send + Sync + 'static,
        H: Fn(&Value) -> u64 + Send + Sync + 'static,
    {
        Self {
            compare_fn: Arc::new(compare_fn),
            hash_fn: Arc::new(hash_fn),
            name: name.into(),
        }
    }
    
    /// Creates the default comparator using built-in value comparison and hashing
    pub fn with_default() -> Self {
        Self::new(
            "default",
            super::utils::compare_values,
            super::utils::hash_value,
        )
    }
    
    /// Creates a numeric comparator for comparing numbers
    pub fn numeric() -> Self {
        Self::new(
            "numeric",
            |a, b| {
                match (a.as_number(), b.as_number()) {
                    (Some(n1), Some(n2)) => n1.partial_cmp(&n2).unwrap_or(Ordering::Equal),
                    _ => Ordering::Equal, // Non-numbers are considered equal
                }
            },
            |v| {
                if let Some(n) = v.as_number() {
                    n.to_bits()
                } else {
                    0
                }
            },
        )
    }
    
    /// Creates a string comparator for comparing strings
    pub fn string() -> Self {
        Self::new(
            "string",
            |a, b| {
                match (a.as_string(), b.as_string()) {
                    (Some(s1), Some(s2)) => s1.cmp(s2),
                    _ => Ordering::Equal, // Non-strings are considered equal
                }
            },
            |v| {
                if let Some(s) = v.as_string() {
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    s.hash(&mut hasher);
                    hasher.finish()
                } else {
                    0
                }
            },
        )
    }
    
    /// Creates a symbol comparator for comparing symbols
    pub fn symbol() -> Self {
        Self::new(
            "symbol",
            |a, b| {
                match (a.as_symbol(), b.as_symbol()) {
                    (Some(sym1), Some(sym2)) => sym1.cmp(&sym2),
                    _ => Ordering::Equal, // Non-symbols are considered equal
                }
            },
            |v| {
                if let Some(sym) = v.as_symbol() {
                    sym.id() as u64
                } else {
                    0
                }
            },
        )
    }
    
    /// Compares two values using this comparator
    pub fn compare(&self, a: &Value, b: &Value) -> Ordering {
        (self.compare_fn)(a, b)
    }
    
    /// Hashes a value using this comparator
    pub fn hash(&self, value: &Value) -> u64 {
        (self.hash_fn)(value)
    }
    
    /// Gets the name of this comparator
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Creates an equality predicate from this comparator
    pub fn equality_predicate(&self) -> impl Fn(&Value, &Value) -> bool + '_ {
        move |a, b| self.compare(a, b) == Ordering::Equal
    }
}

impl std::fmt::Debug for Comparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Comparator({})", self.name)
    }
}

/// A hash-only comparator for cases where ordering is not needed
#[derive(Clone)]
pub struct HashComparator {
    /// Hash function
    hash_fn: HashFn,
    /// Equality function
    eq_fn: EqualityFn,
    /// Name for debugging
    name: String,
}

impl HashComparator {
    /// Creates a new hash comparator
    pub fn new<H, E>(name: impl Into<String>, hash_fn: H, eq_fn: E) -> Self
    where
        H: Fn(&Value) -> u64 + Send + Sync + 'static,
        E: Fn(&Value, &Value) -> bool + Send + Sync + 'static,
    {
        Self {
            hash_fn: Arc::new(hash_fn),
            eq_fn: Arc::new(eq_fn),
            name: name.into(),
        }
    }
    
    /// Creates the default hash comparator
    pub fn with_default() -> Self {
        Self::new(
            "default-hash",
            super::utils::hash_value,
            |a, b| a == b,
        )
    }
    
    /// Creates a case-insensitive string hash comparator
    pub fn string_ci() -> Self {
        Self::new(
            "string-ci",
            |v| {
                if let Some(s) = v.as_string() {
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    s.to_lowercase().hash(&mut hasher);
                    hasher.finish()
                } else {
                    super::utils::hash_value(v)
                }
            },
            |a, b| {
                match (a.as_string(), b.as_string()) {
                    (Some(s1), Some(s2)) => s1.to_lowercase() == s2.to_lowercase(),
                    _ => a == b,
                }
            },
        )
    }
    
    /// Hashes a value using this comparator
    pub fn hash(&self, value: &Value) -> u64 {
        (self.hash_fn)(value)
    }
    
    /// Tests equality of two values using this comparator
    pub fn eq(&self, a: &Value, b: &Value) -> bool {
        (self.eq_fn)(a, b)
    }
    
    /// Gets the name of this comparator
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl std::fmt::Debug for HashComparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashComparator({})", self.name)
    }
}

/// Trait for types that can provide a comparator
pub trait HasComparator {
    /// Returns the comparator used by this type
    fn comparator(&self) -> &Comparator;
}

/// Trait for types that can provide a hash comparator
pub trait HasHashComparator {
    /// Returns the hash comparator used by this type
    fn hash_comparator(&self) -> &HashComparator;
}

/// A wrapper that makes any type comparable using a specific comparator
#[derive(Clone)]
pub struct Comparable<T> {
    value: T,
    comparator: Comparator,
}

impl<T> Comparable<T> {
    /// Creates a new comparable value
    pub fn new(value: T, comparator: Comparator) -> Self {
        Self { value, comparator }
    }
    
    /// Gets the wrapped value
    pub fn value(&self) -> &T {
        &self.value
    }
    
    /// Gets the comparator
    pub fn comparator(&self) -> &Comparator {
        &self.comparator
    }
    
    /// Unwraps the value
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> std::fmt::Debug for Comparable<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Comparable")
            .field("value", &self.value)
            .field("comparator", &self.comparator.name)
            .finish()
    }
}

impl Comparable<Value> {
    /// Compares two comparable values
    pub fn compare(&self, other: &Self) -> Ordering {
        self.comparator.compare(&self.value, &other.value)
    }
    
    /// Hashes this comparable value
    pub fn hash_value(&self) -> u64 {
        self.comparator.hash(&self.value)
    }
}

impl PartialEq for Comparable<Value> {
    fn eq(&self, other: &Self) -> bool {
        self.compare(other) == Ordering::Equal
    }
}

impl Eq for Comparable<Value> {}

impl PartialOrd for Comparable<Value> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl Ord for Comparable<Value> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
}

impl Hash for Comparable<Value> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_value().hash(state);
    }
}

/// A builder for creating custom comparators
pub struct ComparatorBuilder {
    name: String,
    compare_fn: Option<CompareFn>,
    hash_fn: Option<HashFn>,
}

impl ComparatorBuilder {
    /// Creates a new comparator builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            compare_fn: None,
            hash_fn: None,
        }
    }
    
    /// Sets the comparison function
    pub fn compare<F>(mut self, f: F) -> Self
    where
        F: Fn(&Value, &Value) -> Ordering + Send + Sync + 'static,
    {
        self.compare_fn = Some(Arc::new(f));
        self
    }
    
    /// Sets the hash function
    pub fn hash<F>(mut self, f: F) -> Self
    where
        F: Fn(&Value) -> u64 + Send + Sync + 'static,
    {
        self.hash_fn = Some(Arc::new(f));
        self
    }
    
    /// Builds the comparator
    pub fn build(self) -> Comparator {
        let compare_fn = self.compare_fn.unwrap_or_else(|| {
            Arc::new(super::utils::compare_values)
        });
        
        let hash_fn = self.hash_fn.unwrap_or_else(|| {
            Arc::new(super::utils::hash_value)
        });
        
        Comparator {
            compare_fn,
            hash_fn,
            name: self.name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_comparator() {
        let comp = Comparator::with_default();
        let v1 = Value::number(1.0);
        let v2 = Value::number(2.0);
        
        assert_eq!(comp.compare(&v1, &v2), Ordering::Less);
        assert_eq!(comp.compare(&v2, &v1), Ordering::Greater);
        assert_eq!(comp.compare(&v1, &v1), Ordering::Equal);
    }
    
    #[test]
    fn test_numeric_comparator() {
        let comp = Comparator::numeric();
        let v1 = Value::number(std::f64::consts::PI as f64);
        let v2 = Value::number(2.71 as f64);
        let v3 = Value::string("not a number");
        
        assert_eq!(comp.compare(&v1, &v2), Ordering::Greater);
        assert_eq!(comp.compare(&v1, &v3), Ordering::Equal); // Non-numbers are equal
    }
    
    #[test]
    fn test_string_comparator() {
        let comp = Comparator::string();
        let v1 = Value::string("apple");
        let v2 = Value::string("banana");
        let v3 = Value::number(42.0);
        
        assert_eq!(comp.compare(&v1, &v2), Ordering::Less);
        assert_eq!(comp.compare(&v1, &v3), Ordering::Equal); // Non-strings are equal
    }
    
    #[test]
    fn test_hash_comparator() {
        let comp = HashComparator::with_default();
        let v1 = Value::string("hello");
        let v2 = Value::string("hello");
        let v3 = Value::string("world");
        
        assert!(comp.eq(&v1, &v2));
        assert!(!comp.eq(&v1, &v3));
        assert_eq!(comp.hash(&v1), comp.hash(&v2));
    }
    
    #[test]
    fn test_string_ci_comparator() {
        let comp = HashComparator::string_ci();
        let v1 = Value::string("Hello");
        let v2 = Value::string("HELLO");
        let v3 = Value::string("world");
        
        assert!(comp.eq(&v1, &v2));
        assert!(!comp.eq(&v1, &v3));
        assert_eq!(comp.hash(&v1), comp.hash(&v2));
    }
    
    #[test]
    fn test_comparable_wrapper() {
        let comp = Comparator::numeric();
        let c1 = Comparable::new(Value::number(1.0), comp.clone());
        let c2 = Comparable::new(Value::number(2.0), comp.clone());
        
        assert!(c1 < c2);
        assert_eq!(c1.compare(&c2), Ordering::Less);
    }
    
    #[test]
    fn test_comparator_builder() {
        let comp = ComparatorBuilder::new("custom")
            .compare(|a, b| {
                // Reverse numeric comparison
                match (a.as_number(), b.as_number()) {
                    (Some(n1), Some(n2)) => n2.partial_cmp(&n1).unwrap_or(Ordering::Equal),
                    _ => Ordering::Equal,
                }
            })
            .hash(|v| v.as_number().map(|n| n.to_bits()).unwrap_or(0))
            .build();
        
        let v1 = Value::number(1.0);
        let v2 = Value::number(2.0);
        
        // Should be reversed
        assert_eq!(comp.compare(&v1, &v2), Ordering::Greater);
        assert_eq!(comp.name(), "custom");
    }
}