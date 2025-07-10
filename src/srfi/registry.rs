//! SRFI registry for managing available SRFI implementations

use super::{SrfiImport, SrfiModule};
use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;

/// Central registry for all SRFI implementations
pub struct SrfiRegistry {
    /// Registered SRFI modules by ID
    modules: HashMap<u32, Box<dyn SrfiModule>>,
}

impl std::fmt::Debug for SrfiRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SrfiRegistry")
            .field("modules", &self.modules.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl SrfiRegistry {
    /// Create a new SRFI registry
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Create a registry with standard SRFIs pre-registered
    pub fn with_standard_srfis() -> Self {
        let mut registry = Self::new();

        // Register standard SRFIs that are available
        registry.register(Box::new(super::srfi_1::Srfi1)); // List Library
        registry.register(Box::new(super::srfi_9::Srfi9)); // Define-record-type
        registry.register(Box::new(super::srfi_13::Srfi13)); // String Libraries
        registry.register(Box::new(super::srfi_45::Srfi45)); // Lazy evaluation
        registry.register(Box::new(super::srfi_46::Srfi46)); // Syntax-rules extensions
        registry.register(Box::new(super::srfi_69::Srfi69)); // Basic Hash Tables
        registry.register(Box::new(super::srfi_97::Srfi97)); // SRFI Libraries
        registry.register(Box::new(super::srfi_111::Srfi111)); // Boxes
        registry.register(Box::new(super::srfi_113::Srfi113)); // Sets and Bags
        registry.register(Box::new(super::srfi_125::Srfi125)); // Intermediate Hash Tables
        registry.register(Box::new(super::srfi_128::Srfi128)); // Comparators
        registry.register(Box::new(super::srfi_130::Srfi130)); // Cursor-based String Library
        registry.register(Box::new(super::srfi_132::Srfi132)); // Sort Libraries
        registry.register(Box::new(super::srfi_133::Srfi133)); // Vector Libraries
        registry.register(Box::new(super::srfi_134::Srfi134)); // Immutable Deques
        registry.register(Box::new(super::srfi_135::Srfi135)); // Immutable Texts
        registry.register(Box::new(super::srfi_136::Srfi136)); // Extensible Record Types
        registry.register(Box::new(super::srfi_137::Srfi137)); // Minimal Unique Types
        registry.register(Box::new(super::srfi_138::Srfi138)); // Compiling Scheme to Machine Code
        registry.register(Box::new(super::srfi_139::Srfi139)); // Syntax Parameters
        registry.register(Box::new(super::srfi_140::Srfi140)); // Immutable Strings
        registry.register(Box::new(super::srfi_141::Srfi141)); // Integer Division

        registry
    }

    /// Register a SRFI module
    pub fn register(&mut self, module: Box<dyn SrfiModule>) {
        let id = module.srfi_id();
        self.modules.insert(id, module);
    }

    /// Check if a SRFI is available
    pub fn has_srfi(&self, id: u32) -> bool {
        self.modules.contains_key(&id)
    }

    /// Get available SRFI IDs
    pub fn available_srfis(&self) -> Vec<u32> {
        let mut ids: Vec<u32> = self.modules.keys().copied().collect();
        ids.sort();
        ids
    }

    /// Get SRFI information
    pub fn get_srfi_info(&self, id: u32) -> Option<(u32, &str, Vec<&str>)> {
        self.modules
            .get(&id)
            .map(|module| (module.srfi_id(), module.name(), module.parts()))
    }

    /// Import functions from a SRFI
    pub fn import_srfi(&self, import: &SrfiImport) -> Result<HashMap<String, Value>> {
        let module = self.modules.get(&import.id).ok_or_else(|| {
            LambdustError::runtime_error(format!("SRFI {} is not available", import.id))
        })?;

        if import.imports_all() {
            Ok(module.exports())
        } else {
            let parts: Vec<&str> = import.parts.iter().map(|s| s.as_str()).collect();
            module.exports_for_parts(&parts)
        }
    }

    /// Import multiple SRFIs at once
    pub fn import_multiple(&self, imports: &[SrfiImport]) -> Result<HashMap<String, Value>> {
        let mut exports = HashMap::new();

        for import in imports {
            let srfi_exports = self.import_srfi(import)?;

            // Check for conflicts
            for (name, value) in srfi_exports {
                if let Some(existing) = exports.get(&name) {
                    // Check if they're the same implementation
                    if !values_equivalent(&value, existing) {
                        return Err(LambdustError::runtime_error(format!(
                            "Conflicting exports for '{}' from SRFI {}",
                            name, import.id
                        )));
                    }
                }
                exports.insert(name, value);
            }
        }

        Ok(exports)
    }

    /// Get exports for a SRFI (used by evaluator)
    pub fn get_exports(&self, id: u32) -> Result<HashMap<String, Value>> {
        let module = self
            .modules
            .get(&id)
            .ok_or_else(|| LambdustError::runtime_error(format!("SRFI {} is not available", id)))?;
        Ok(module.exports())
    }

    /// Get exports for specific parts of a SRFI (used by evaluator)
    pub fn get_exports_for_parts(&self, id: u32, parts: &[&str]) -> Result<HashMap<String, Value>> {
        let module = self
            .modules
            .get(&id)
            .ok_or_else(|| LambdustError::runtime_error(format!("SRFI {} is not available", id)))?;
        module.exports_for_parts(parts)
    }
}

impl Default for SrfiRegistry {
    fn default() -> Self {
        Self::with_standard_srfis()
    }
}

/// Check if two values are equivalent for conflict detection
fn values_equivalent(a: &Value, b: &Value) -> bool {
    // For now, just check if they're the same type and name for procedures
    match (a, b) {
        (Value::Procedure(proc_a), Value::Procedure(proc_b)) => {
            use crate::value::Procedure;
            match (proc_a, proc_b) {
                (
                    Procedure::Builtin { name: name_a, .. },
                    Procedure::Builtin { name: name_b, .. },
                ) => name_a == name_b,
                _ => false,
            }
        }
        _ => a == b,
    }
}
