//! SRFI 69: Basic Hash Tables implementation
//!
//! This module implements the SRFI 69 Basic Hash Tables, providing
//! comprehensive hash table (dictionary) functionality for R7RS Scheme.
//! 
//! The implementation is divided into functional modules:
//! - types: Core data structures (HashTable, HashKey)
//! - core: Basic operations (creation, access, modification)
//! - queries: Query operations (exists, size, keys, values)
//! - conversion: Conversion operations (alist conversion, copying)
//! - hash_functions: Hash functions for different data types
//! - higher_order: Higher-order functions (walk, fold, merge)

pub mod types;
pub mod core;
pub mod queries;
pub mod conversion;
pub mod hash_functions;
pub mod higher_order;

use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;

// Re-export main types for convenience
pub use types::{HashTable, HashKey};

/// Register all SRFI 69 functions into the builtins map
pub fn register_srfi_69_functions(builtins: &mut HashMap<String, Value>) {
    core::register_functions(builtins);
    queries::register_functions(builtins);
    conversion::register_functions(builtins);
    hash_functions::register_functions(builtins);
    higher_order::register_functions(builtins);
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
        vec!["all", "core", "queries", "conversion", "hash", "higher-order", "constructors", "accessors"]
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
                "core" => {
                    // Core hash table operations
                    for name in &[
                        "make-hash-table",
                        "hash-table?",
                        "hash-table-ref",
                        "hash-table-ref/default",
                        "hash-table-set!",
                        "hash-table-delete!",
                    ] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "queries" => {
                    // Query operations
                    for name in &[
                        "hash-table-exists?",
                        "hash-table-size",
                        "hash-table-keys",
                        "hash-table-values",
                    ] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "conversion" => {
                    // Conversion operations
                    for name in &[
                        "hash-table->alist",
                        "alist->hash-table",
                        "hash-table-copy",
                    ] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "hash" => {
                    // Hash functions
                    for name in &["hash", "string-hash", "string-ci-hash"] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "higher-order" => {
                    // Higher-order functions
                    for name in &[
                        "hash-table-walk",
                        "hash-table-fold",
                        "hash-table-merge!",
                    ] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "constructors" => {
                    // Constructor operations (alias for core)
                    for name in &[
                        "make-hash-table",
                        "hash-table?",
                    ] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "accessors" => {
                    // Accessor operations (alias for core + queries)
                    for name in &[
                        "hash-table-ref",
                        "hash-table-ref/default", 
                        "hash-table-set!",
                        "hash-table-exists?",
                        "hash-table-size",
                        "hash-table-keys",
                        "hash-table-values",
                    ] {
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
                        "SRFI 69: unknown part '{}'",
                        part
                    )));
                }
            }
        }

        Ok(filtered)
    }
}