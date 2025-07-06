//! SRFI 13: String Libraries implementation
//!
//! This module implements the SRFI 13 String Libraries, providing
//! comprehensive string processing functions for R7RS Scheme.
//! 
//! The implementation is divided into functional modules:
//! - constructors: Basic string operations and predicates
//! - comparison: String comparison and hashing functions  
//! - search: String searching, prefix/suffix operations
//! - modification: String modification, padding, trimming
//! - joining: String joining, splitting, and tokenization

pub mod constructors;
pub mod comparison;
pub mod search;
pub mod modification;
pub mod joining;

use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;

/// Register all SRFI 13 functions into the builtins map
pub fn register_srfi_13_functions(builtins: &mut HashMap<String, Value>) {
    constructors::register_functions(builtins);
    comparison::register_functions(builtins);
    search::register_functions(builtins);
    modification::register_functions(builtins);
    joining::register_functions(builtins);
}

/// SRFI 13 module implementation
pub struct Srfi13;

impl crate::srfi::SrfiModule for Srfi13 {
    fn srfi_id(&self) -> u32 {
        13
    }

    fn name(&self) -> &'static str {
        "String Libraries"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["all"] // SRFI 13 doesn't have separate parts
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        register_srfi_13_functions(&mut exports);
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
                "predicates" => {
                    // String predicate functions
                    for name in &[
                        "string-null?",
                        "string-prefix?",
                        "string-suffix?",
                        "string-prefix-ci?",
                        "string-suffix-ci?",
                    ] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "search" => {
                    // String search functions
                    for name in &["string-contains", "string-contains-ci"] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "manipulation" => {
                    // String manipulation functions
                    for name in &[
                        "string-take",
                        "string-drop",
                        "string-take-right",
                        "string-drop-right",
                        "string-concatenate",
                    ] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "hash" => {
                    // Hash functions
                    for name in &["string-hash", "string-hash-ci"] {
                        if let Some(value) = all_exports.get(*name) {
                            filtered.insert(name.to_string(), value.clone());
                        }
                    }
                }
                "higher-order" => {
                    // Higher-order functions (placeholder for future implementation)
                    for name in &["string-every", "string-any"] {
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
                        "SRFI 13: unknown part '{}'",
                        part
                    )));
                }
            }
        }

        Ok(filtered)
    }
}