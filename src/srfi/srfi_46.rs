//! SRFI 46: Basic Syntax-rules Extensions
//!
//! This SRFI provides enhanced syntax-rules with better ellipsis handling.

use super::SrfiModule;
use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;

/// SRFI 46 implementation
pub struct Srfi46;

impl SrfiModule for Srfi46 {
    fn srfi_id(&self) -> u32 {
        46
    }

    fn name(&self) -> &'static str {
        "Basic Syntax-rules Extensions"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["syntax", "ellipsis"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        // SRFI 46 doesn't export runtime functions, but macro-related utilities
        // These would be handled by the macro system rather than runtime functions

        // For completeness, we could export some macro-related utilities
        // but the main functionality is in the macro expander

        HashMap::new()
    }

    fn exports_for_parts(&self, parts: &[&str]) -> Result<HashMap<String, Value>> {
        let filtered = HashMap::new();

        for part in parts {
            match *part {
                "syntax" => {
                    // Syntax-related exports (none for now)
                }
                "ellipsis" => {
                    // Ellipsis-related exports (none for now)
                }
                _ => {
                    return Err(LambdustError::runtime_error(format!(
                        "Unknown SRFI 46 part: {}",
                        part
                    )));
                }
            }
        }

        Ok(filtered)
    }
}
