//! High-performance container library for R7RS-large support.
//!
//! This module provides efficient implementations of various data structures
//! required by R7RS-large specifications, including SRFI implementations.
//!
//! ## Features
//!
//! - **Hash Tables (SRFI-125)**: Hybrid open addressing and chaining with
//!   dynamic resizing and custom comparators
//! - **Ideques (SRFI-134)**: Persistent double-ended queues using finger trees
//! - **Priority Queues**: Binary heap and Fibonacci heap implementations
//! - **Ordered Sets**: Red-black tree based sets with efficient set operations
//! - **List Queues (SRFI-117)**: FIFO queues with O(1) operations
//! - **Random Access Lists (SRFI-101)**: Skew binary lists with O(log n) indexing
//!
//! ## Thread Safety
//!
//! All data structures come in both single-threaded and thread-safe variants.
//! Thread-safe versions use Arc and RwLock for shared ownership and synchronization.
//!
//! ## Integration
//!
//! All containers integrate seamlessly with the existing Value system and
//! maintain compatibility with Scheme's pair-based lists.

use crate::eval::value::Value;

pub mod hash_table;
pub mod ideque;
pub mod priority_queue;
pub mod ordered_set;
pub mod list_queue;
pub mod random_access_list;
pub mod comparator;
pub mod benchmarks;

// Re-export main types for convenience
pub use hash_table::{HashTable, ThreadSafeHashTable};
pub use ideque::{Ideque, PersistentIdeque};
pub use priority_queue::{PriorityQueue, ThreadSafePriorityQueue};
pub use ordered_set::{OrderedSet, ThreadSafeOrderedSet};
pub use list_queue::{ListQueue, ThreadSafeListQueue};
pub use random_access_list::{RandomAccessList, PersistentRandomAccessList, ThreadSafeRandomAccessList};
pub use comparator::{Comparator, HashComparator};
pub use benchmarks::{ContainerBenchmarks, BenchmarkResult, run_quick_benchmark};

/// Common traits for all container types
pub trait Container {
    /// Returns the number of elements in the container
    fn len(&self) -> usize;
    
    /// Returns true if the container is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Clears all elements from the container
    fn clear(&mut self);
}

/// Trait for containers that support iteration
pub trait Iterable<T> {
    /// Iterator type that yields items of type T
    type Iterator: Iterator<Item = T>;
    
    /// Returns an iterator over the container's elements
    fn iter(&self) -> Self::Iterator;
}

/// Trait for persistent (immutable) containers
pub trait Persistent<T>: Clone {
    /// Returns a new container with the given element inserted
    fn insert(&self, element: T) -> Self;
    
    /// Returns a new container with the given element removed
    fn remove(&self, element: &T) -> Self;
}

/// Load factor constants for hash-based containers
pub mod load_factors {
    /// Default load factor for hash tables (0.75)
    pub const DEFAULT_LOAD_FACTOR: f64 = 0.75;
    
    /// Maximum load factor before resize (0.9)
    pub const MAX_LOAD_FACTOR: f64 = 0.9;
    
    /// Minimum load factor before shrink (0.25)
    pub const MIN_LOAD_FACTOR: f64 = 0.25;
}

/// Capacity constants for container initialization
pub mod capacities {
    /// Default initial capacity for hash tables
    pub const DEFAULT_HASH_TABLE_CAPACITY: usize = 16;
    
    /// Default initial capacity for priority queues
    pub const DEFAULT_PRIORITY_QUEUE_CAPACITY: usize = 16;
    
    /// Default initial capacity for list queues
    pub const DEFAULT_LIST_QUEUE_CAPACITY: usize = 32;
}

/// Error types for container operations
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerError {
    /// Index out of bounds
    IndexOutOfBounds {
        /// The invalid index that was accessed
        index: usize,
        /// The actual length of the container
        length: usize,
    },
    
    /// Empty container operation
    EmptyContainer {
        /// The operation that was attempted on an empty container
        operation: String,
    },
    
    /// Key not found in associative container
    KeyNotFound {
        /// The key that was not found
        key: String,
    },
    
    /// Invalid comparator
    InvalidComparator {
        /// Description of what makes the comparator invalid
        message: String,
    },
    
    /// Capacity exceeded
    CapacityExceeded {
        /// The requested capacity that was too large
        requested: usize,
        /// The maximum allowed capacity
        maximum: usize,
    },
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerError::IndexOutOfBounds { index, length } => {
                write!(f, "Index {index} out of bounds for container of length {length}")
            }
            ContainerError::EmptyContainer { operation } => {
                write!(f, "Cannot perform '{operation}' on empty container")
            }
            ContainerError::KeyNotFound { key } => {
                write!(f, "Key '{key}' not found")
            }
            ContainerError::InvalidComparator { message } => {
                write!(f, "Invalid comparator: {message}")
            }
            ContainerError::CapacityExceeded { requested, maximum } => {
                write!(f, "Requested capacity {requested} exceeds maximum {maximum}")
            }
        }
    }
}

impl std::error::Error for ContainerError {}

/// Result type for container operations
pub type ContainerResult<T> = Result<T, ContainerError>;

/// Utility functions for container operations
pub mod utils {
    use super::*;
    
    /// Calculates the next power of 2 greater than or equal to n
    pub fn next_power_of_two(n: usize) -> usize {
        if n == 0 {
            return 1;
        }
        let mut power = 1;
        while power < n {
            power <<= 1;
        }
        power
    }
    
    /// Calculates a hash value for a Scheme Value
    pub fn hash_value(value: &Value) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Compares two Scheme Values for ordering
    pub fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        
        // Type-based ordering first
        let type_order_a = value_type_order(a);
        let type_order_b = value_type_order(b);
        
        match type_order_a.cmp(&type_order_b) {
            Ordering::Equal => {
                // Same type, compare values
                match (a, b) {
                    (Value::Literal(lit_a), Value::Literal(lit_b)) => {
                        compare_literals(lit_a, lit_b)
                    }
                    (Value::Symbol(sym_a), Value::Symbol(sym_b)) => {
                        sym_a.cmp(sym_b)
                    }
                    (Value::Keyword(k_a), Value::Keyword(k_b)) => {
                        k_a.cmp(k_b)
                    }
                    (Value::Nil, Value::Nil) => Ordering::Equal,
                    (Value::Unspecified, Value::Unspecified) => Ordering::Equal,
                    // For other types, use pointer comparison as fallback
                    _ => Ordering::Equal,
                }
            }
            other => other,
        }
    }
    
    fn value_type_order(value: &Value) -> u8 {
        match value {
            Value::Literal(_) => 0,
            Value::Symbol(_) => 1,
            Value::Keyword(_) => 2,
            Value::Nil => 3,
            Value::Unspecified => 4,
            Value::Pair(_, _) => 5,
            Value::Vector(_) => 6,
            Value::Hashtable(_) => 7,
            Value::Procedure(_) => 8,
            Value::CaseLambda(_) => 9,
            Value::Primitive(_) => 10,
            Value::Continuation(_) => 11,
            Value::Syntax(_) => 12,
            Value::Port(_) => 13,
            Value::Promise(_) => 14,
            Value::Type(_) => 15,
            Value::Foreign(_) => 16,
            Value::ErrorObject(_) => 17,
            Value::CharSet(_) => 18,
            Value::Parameter(_) => 19,
            Value::Record(_) => 20,
            Value::AdvancedHashTable(_) => 21,
            Value::Ideque(_) => 22,
            Value::PriorityQueue(_) => 23,
            Value::OrderedSet(_) => 24,
            Value::ListQueue(_) => 25,
            Value::RandomAccessList(_) => 26,
            // Concurrency types
            Value::Future(_) => 27,
            Value::Channel(_) => 28,
            Value::Mutex(_) => 29,
            Value::Semaphore(_) => 30,
            Value::AtomicCounter(_) => 31,
            Value::DistributedNode(_) => 32,
            Value::Opaque(_) => 33,
        }
    }
    
    fn compare_literals(a: &crate::ast::Literal, b: &crate::ast::Literal) -> std::cmp::Ordering {
        use crate::ast::Literal;
        use std::cmp::Ordering;
        
        match (a, b) {
            (Literal::Number(n_a), Literal::Number(n_b)) => {
                n_a.partial_cmp(n_b).unwrap_or(Ordering::Equal)
            }
            (Literal::String(s_a), Literal::String(s_b)) => s_a.cmp(s_b),
            (Literal::Character(c_a), Literal::Character(c_b)) => c_a.cmp(c_b),
            (Literal::Boolean(b_a), Literal::Boolean(b_b)) => b_a.cmp(b_b),
            (Literal::Bytevector(bv_a), Literal::Bytevector(bv_b)) => bv_a.cmp(bv_b),
            (Literal::Rational { numerator: n_a, denominator: d_a }, Literal::Rational { numerator: n_b, denominator: d_b }) => {
                let val_a = *n_a as f64 / *d_a as f64;
                let val_b = *n_b as f64 / *d_b as f64;
                val_a.partial_cmp(&val_b).unwrap_or(Ordering::Equal)
            }
            (Literal::Complex { real: r_a, imaginary: i_a }, Literal::Complex { real: r_b, imaginary: i_b }) => {
                match r_a.partial_cmp(r_b) {
                    Some(Ordering::Equal) => i_a.partial_cmp(i_b).unwrap_or(Ordering::Equal),
                    Some(ord) => ord,
                    None => Ordering::Equal,
                }
            }
            (Literal::Nil, Literal::Nil) => Ordering::Equal,
            (Literal::Unspecified, Literal::Unspecified) => Ordering::Equal,
            // Different literal types - order by discriminant
            _ => {
                let a_disc = std::mem::discriminant(a);
                let b_disc = std::mem::discriminant(b);
                if a_disc == b_disc {
                    Ordering::Equal
                } else {
                    // Fallback to string comparison for different types
                    format!("{a_disc:?}").cmp(&format!("{b_disc:?}"))
                }
            },
        }
    }
    
    /// Creates a list of Values from a vector
    pub fn values_to_list(values: Vec<Value>) -> Value {
        values.into_iter().rev().fold(Value::Nil, |acc, val| {
            Value::pair(val, acc)
        })
    }
    
    /// Converts a Value list to a vector if it's a proper list
    pub fn list_to_values(list: &Value) -> Option<Vec<Value>> {
        list.as_list()
    }
    
    /// Estimates memory usage of a Value (rough approximation)
    pub fn estimate_value_memory(value: &Value) -> usize {
        match value {
            Value::Literal(lit) => estimate_literal_memory(lit),
            Value::Symbol(_) => std::mem::size_of::<u64>(),
            Value::Keyword(k) => std::mem::size_of::<String>() + k.len(),
            Value::Nil | Value::Unspecified => 0,
            Value::Pair(a, b) => {
                std::mem::size_of::<Value>() * 2 + 
                estimate_value_memory(a) + 
                estimate_value_memory(b)
            }
            Value::Vector(vec) => {
                if let Ok(vec_ref) = vec.read() {
                    std::mem::size_of::<Vec<Value>>() + 
                    vec_ref.len() * std::mem::size_of::<Value>() +
                    vec_ref.iter().map(estimate_value_memory).sum::<usize>()
                } else {
                    std::mem::size_of::<Vec<Value>>()
                }
            }
            // For other types, return a base estimate
            _ => std::mem::size_of::<Value>() + 64, // Base size + overhead
        }
    }
    
    fn estimate_literal_memory(lit: &crate::ast::Literal) -> usize {
        use crate::ast::Literal;
        match lit {
            Literal::Number(_) => std::mem::size_of::<f64>(),
            Literal::String(s) => std::mem::size_of::<String>() + s.len(),
            Literal::Character(_) => std::mem::size_of::<char>(),
            Literal::Boolean(_) => std::mem::size_of::<bool>(),
            Literal::Bytevector(bv) => std::mem::size_of::<Vec<u8>>() + bv.len(),
            Literal::Rational { .. } => std::mem::size_of::<i64>() * 2,
            Literal::Complex { .. } => std::mem::size_of::<f64>() * 2,
            Literal::Nil => 0,
            Literal::Unspecified => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils::*;
    
    #[test]
    fn test_next_power_of_two() {
        assert_eq!(next_power_of_two(0), 1);
        assert_eq!(next_power_of_two(1), 1);
        assert_eq!(next_power_of_two(2), 2);
        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(15), 16);
        assert_eq!(next_power_of_two(16), 16);
        assert_eq!(next_power_of_two(17), 32);
    }
    
    #[test]
    fn test_hash_value() {
        let v1 = Value::number(42.0);
        let v2 = Value::number(42.0);
        let v3 = Value::string("hello");
        
        assert_eq!(hash_value(&v1), hash_value(&v2));
        assert_ne!(hash_value(&v1), hash_value(&v3));
    }
    
    #[test]
    fn test_compare_values() {
        use std::cmp::Ordering;
        
        let n1 = Value::number(1.0);
        let n2 = Value::number(2.0);
        let s1 = Value::string("a");
        let s2 = Value::string("b");
        
        assert_eq!(compare_values(&n1, &n2), Ordering::Less);
        assert_eq!(compare_values(&n2, &n1), Ordering::Greater);
        assert_eq!(compare_values(&s1, &s2), Ordering::Less);
        
        // Different types should be ordered by type
        assert_eq!(compare_values(&n1, &s1), Ordering::Less);
    }
}