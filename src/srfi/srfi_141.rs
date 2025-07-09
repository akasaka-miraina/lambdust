//! SRFI 141: Integer Division
//!
//! This SRFI provides a complete set of integer division operators for R7RS Scheme.
//! It defines six different division families, each with quotient, remainder, and
//! combined operations:
//!
//! 1. Floor division family: floor-quotient, floor-remainder, floor/
//! 2. Ceiling division family: ceiling-quotient, ceiling-remainder, ceiling/
//! 3. Truncate division family: truncate-quotient, truncate-remainder, truncate/
//! 4. Round division family: round-quotient, round-remainder, round/
//! 5. Euclidean division family: euclidean-quotient, euclidean-remainder, euclidean/
//! 6. Balanced division family: balanced-quotient, balanced-remainder, balanced/

use crate::srfi::SrfiModule;
use crate::value::Value;
use std::collections::HashMap;

/// SRFI 141: Integer Division implementation
pub struct Srfi141;

impl SrfiModule for Srfi141 {
    fn srfi_id(&self) -> u32 {
        141
    }

    fn name(&self) -> &'static str {
        "Integer Division"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec![
            "floor",
            "ceiling",
            "truncate",
            "round",
            "euclidean",
            "balanced",
        ]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Floor division family
        exports.extend(self.floor_division_exports());

        // Ceiling division family
        exports.extend(self.ceiling_division_exports());

        // Truncate division family
        exports.extend(self.truncate_division_exports());

        // Round division family
        exports.extend(self.round_division_exports());

        // Euclidean division family
        exports.extend(self.euclidean_division_exports());

        // Balanced division family
        exports.extend(self.balanced_division_exports());

        exports
    }

    fn exports_for_parts(&self, parts: &[&str]) -> crate::error::Result<HashMap<String, Value>> {
        let mut exports = HashMap::new();

        for part in parts {
            match *part {
                "floor" => exports.extend(self.floor_division_exports()),
                "ceiling" => exports.extend(self.ceiling_division_exports()),
                "truncate" => exports.extend(self.truncate_division_exports()),
                "round" => exports.extend(self.round_division_exports()),
                "euclidean" => exports.extend(self.euclidean_division_exports()),
                "balanced" => exports.extend(self.balanced_division_exports()),
                _ => {
                    return Err(crate::error::LambdustError::runtime_error(format!(
                        "SRFI 141: Unknown part '{}'",
                        part
                    )));
                }
            }
        }

        Ok(exports)
    }
}

impl Default for Srfi141 {
    fn default() -> Self {
        Self::new()
    }
}

impl Srfi141 {
    /// Create a new SRFI 141 module instance
    pub fn new() -> Self {
        Self
    }

    /// Get floor division family exports
    fn floor_division_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Register the functions from arithmetic.rs
        crate::builtins::arithmetic::register_arithmetic_functions(&mut exports);

        // Extract only floor division functions
        let mut floor_exports = HashMap::new();
        if let Some(func) = exports.get("floor-quotient") {
            floor_exports.insert("floor-quotient".to_string(), func.clone());
        }
        if let Some(func) = exports.get("floor-remainder") {
            floor_exports.insert("floor-remainder".to_string(), func.clone());
        }
        if let Some(func) = exports.get("floor/") {
            floor_exports.insert("floor/".to_string(), func.clone());
        }

        floor_exports
    }

    /// Get ceiling division family exports
    fn ceiling_division_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Register the functions from arithmetic.rs
        crate::builtins::arithmetic::register_arithmetic_functions(&mut exports);

        // Extract only ceiling division functions
        let mut ceiling_exports = HashMap::new();
        if let Some(func) = exports.get("ceiling-quotient") {
            ceiling_exports.insert("ceiling-quotient".to_string(), func.clone());
        }
        if let Some(func) = exports.get("ceiling-remainder") {
            ceiling_exports.insert("ceiling-remainder".to_string(), func.clone());
        }
        if let Some(func) = exports.get("ceiling/") {
            ceiling_exports.insert("ceiling/".to_string(), func.clone());
        }

        ceiling_exports
    }

    /// Get truncate division family exports
    fn truncate_division_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Register the functions from arithmetic.rs
        crate::builtins::arithmetic::register_arithmetic_functions(&mut exports);

        // Extract only truncate division functions
        let mut truncate_exports = HashMap::new();
        if let Some(func) = exports.get("truncate-quotient") {
            truncate_exports.insert("truncate-quotient".to_string(), func.clone());
        }
        if let Some(func) = exports.get("truncate-remainder") {
            truncate_exports.insert("truncate-remainder".to_string(), func.clone());
        }
        if let Some(func) = exports.get("truncate/") {
            truncate_exports.insert("truncate/".to_string(), func.clone());
        }

        truncate_exports
    }

    /// Get round division family exports
    fn round_division_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Register the functions from arithmetic.rs
        crate::builtins::arithmetic::register_arithmetic_functions(&mut exports);

        // Extract only round division functions
        let mut round_exports = HashMap::new();
        if let Some(func) = exports.get("round-quotient") {
            round_exports.insert("round-quotient".to_string(), func.clone());
        }
        if let Some(func) = exports.get("round-remainder") {
            round_exports.insert("round-remainder".to_string(), func.clone());
        }
        if let Some(func) = exports.get("round/") {
            round_exports.insert("round/".to_string(), func.clone());
        }

        round_exports
    }

    /// Get euclidean division family exports
    fn euclidean_division_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Register the functions from arithmetic.rs
        crate::builtins::arithmetic::register_arithmetic_functions(&mut exports);

        // Extract only euclidean division functions
        let mut euclidean_exports = HashMap::new();
        if let Some(func) = exports.get("euclidean-quotient") {
            euclidean_exports.insert("euclidean-quotient".to_string(), func.clone());
        }
        if let Some(func) = exports.get("euclidean-remainder") {
            euclidean_exports.insert("euclidean-remainder".to_string(), func.clone());
        }
        if let Some(func) = exports.get("euclidean/") {
            euclidean_exports.insert("euclidean/".to_string(), func.clone());
        }

        euclidean_exports
    }

    /// Get balanced division family exports
    fn balanced_division_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Register the functions from arithmetic.rs
        crate::builtins::arithmetic::register_arithmetic_functions(&mut exports);

        // Extract only balanced division functions
        let mut balanced_exports = HashMap::new();
        if let Some(func) = exports.get("balanced-quotient") {
            balanced_exports.insert("balanced-quotient".to_string(), func.clone());
        }
        if let Some(func) = exports.get("balanced-remainder") {
            balanced_exports.insert("balanced-remainder".to_string(), func.clone());
        }
        if let Some(func) = exports.get("balanced/") {
            balanced_exports.insert("balanced/".to_string(), func.clone());
        }

        balanced_exports
    }
}

/// Default instance for SRFI 141
pub static SRFI_141: Srfi141 = Srfi141;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srfi_141_metadata() {
        let srfi = Srfi141::new();

        assert_eq!(srfi.srfi_id(), 141);
        assert_eq!(srfi.name(), "Integer Division");

        let parts = srfi.parts();
        assert_eq!(parts.len(), 6);
        assert!(parts.contains(&"floor"));
        assert!(parts.contains(&"ceiling"));
        assert!(parts.contains(&"truncate"));
        assert!(parts.contains(&"round"));
        assert!(parts.contains(&"euclidean"));
        assert!(parts.contains(&"balanced"));
    }

    #[test]
    fn test_srfi_141_exports() {
        let srfi = Srfi141::new();
        let exports = srfi.exports();

        // Should have 18 functions total (6 families × 3 functions each)
        assert_eq!(exports.len(), 18);

        // Check floor division family
        assert!(exports.contains_key("floor-quotient"));
        assert!(exports.contains_key("floor-remainder"));
        assert!(exports.contains_key("floor/"));

        // Check ceiling division family
        assert!(exports.contains_key("ceiling-quotient"));
        assert!(exports.contains_key("ceiling-remainder"));
        assert!(exports.contains_key("ceiling/"));

        // Check truncate division family
        assert!(exports.contains_key("truncate-quotient"));
        assert!(exports.contains_key("truncate-remainder"));
        assert!(exports.contains_key("truncate/"));

        // Check round division family
        assert!(exports.contains_key("round-quotient"));
        assert!(exports.contains_key("round-remainder"));
        assert!(exports.contains_key("round/"));

        // Check euclidean division family
        assert!(exports.contains_key("euclidean-quotient"));
        assert!(exports.contains_key("euclidean-remainder"));
        assert!(exports.contains_key("euclidean/"));

        // Check balanced division family
        assert!(exports.contains_key("balanced-quotient"));
        assert!(exports.contains_key("balanced-remainder"));
        assert!(exports.contains_key("balanced/"));
    }

    #[test]
    fn test_srfi_141_partial_exports() {
        let srfi = Srfi141::new();

        // Test floor division family export
        let floor_exports = srfi.exports_for_parts(&["floor"]).unwrap();
        assert_eq!(floor_exports.len(), 3);
        assert!(floor_exports.contains_key("floor-quotient"));
        assert!(floor_exports.contains_key("floor-remainder"));
        assert!(floor_exports.contains_key("floor/"));

        // Test ceiling and truncate families
        let partial_exports = srfi.exports_for_parts(&["ceiling", "truncate"]).unwrap();
        assert_eq!(partial_exports.len(), 6);
        assert!(partial_exports.contains_key("ceiling-quotient"));
        assert!(partial_exports.contains_key("truncate-quotient"));

        // Test invalid part
        let result = srfi.exports_for_parts(&["invalid"]);
        assert!(result.is_err());
    }
}
