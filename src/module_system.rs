//! Module system for Scheme import/export functionality
//!
//! This module implements the `(import ...)` special form and module management.

use crate::error::{LambdustError, Result};
use crate::srfi::{parse_srfi_import, SrfiRegistry};
use crate::value::Value;
use crate::macros::Macro;
use std::collections::HashMap;

/// Module import specification
#[derive(Debug, Clone)]
pub enum ImportSpec {
    /// SRFI import: (srfi 1), (srfi 1 lists)
    Srfi(crate::srfi::SrfiImport),
    /// Library import: (scheme base), (scheme write)
    Library(LibraryImport),
}

/// Library import specification
#[derive(Debug, Clone)]
pub struct LibraryImport {
    /// Library name parts
    pub parts: Vec<String>,
}

/// Module system for handling imports and exports
pub struct ModuleSystem {
    /// SRFI registry
    srfi_registry: SrfiRegistry,
    /// Currently imported bindings
    imported_bindings: HashMap<String, Value>,
    /// Currently imported macros
    imported_macros: HashMap<String, Macro>,
    /// Exported bindings for this module
    exported_bindings: HashMap<String, Value>,
    /// Exported macros for this module
    exported_macros: HashMap<String, Macro>,
}

impl ModuleSystem {
    /// Create a new module system
    #[must_use] pub fn new() -> Self {
        Self {
            srfi_registry: SrfiRegistry::with_standard_srfis(),
            imported_bindings: HashMap::new(),
            imported_macros: HashMap::new(),
            exported_bindings: HashMap::new(),
            exported_macros: HashMap::new(),
        }
    }

    /// Parse import specifications from S-expressions
    pub fn parse_import_specs(&self, exprs: &[crate::ast::Expr]) -> Result<Vec<ImportSpec>> {
        let mut specs = Vec::new();

        for expr in exprs {
            if let crate::ast::Expr::List(elements) = expr {
                if elements.is_empty() {
                    return Err(LambdustError::syntax_error(
                        "Empty import specification".to_string(),
                    ));
                }

                // Check first element to determine import type
                if let crate::ast::Expr::Variable(name) = &elements[0] {
                    match name.as_str() {
                        "srfi" => {
                            let srfi_import = parse_srfi_import(expr)?;
                            specs.push(ImportSpec::Srfi(srfi_import));
                        }
                        "scheme" => {
                            let library_import = self.parse_library_import(elements)?;
                            specs.push(ImportSpec::Library(library_import));
                        }
                        _ => {
                            return Err(LambdustError::syntax_error(format!(
                                "Unknown import type: {name}"
                            )));
                        }
                    }
                } else {
                    return Err(LambdustError::syntax_error(
                        "Import specification must start with a symbol".to_string(),
                    ));
                }
            } else {
                return Err(LambdustError::syntax_error(
                    "Import specification must be a list".to_string(),
                ));
            }
        }

        Ok(specs)
    }

    /// Parse library import from elements
    fn parse_library_import(&self, elements: &[crate::ast::Expr]) -> Result<LibraryImport> {
        let parts = elements[0..]
            .iter()
            .map(|expr| match expr {
                crate::ast::Expr::Variable(name) => Ok(name.clone()),
                _ => Err(LambdustError::syntax_error(
                    "Library name parts must be symbols".to_string(),
                )),
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(LibraryImport { parts })
    }

    /// Execute import specifications
    pub fn execute_imports(&mut self, specs: &[ImportSpec]) -> Result<()> {
        for spec in specs {
            match spec {
                ImportSpec::Srfi(srfi_import) => {
                    let exports = self.srfi_registry.import_srfi(srfi_import)?;

                    // Check for conflicts
                    for (name, value) in exports {
                        if let Some(existing) = self.imported_bindings.get(&name) {
                            if !values_equivalent(&value, existing) {
                                return Err(LambdustError::runtime_error(format!(
                                    "Import conflict for binding '{name}'"
                                )));
                            }
                        }
                        self.imported_bindings.insert(name, value);
                    }
                }
                ImportSpec::Library(library_import) => {
                    // Handle library imports (scheme base, scheme write, etc.)
                    let exports = self.import_library(library_import)?;

                    for (name, value) in exports {
                        if let Some(existing) = self.imported_bindings.get(&name) {
                            if !values_equivalent(&value, existing) {
                                return Err(LambdustError::runtime_error(format!(
                                    "Import conflict for binding '{name}'"
                                )));
                            }
                        }
                        self.imported_bindings.insert(name, value);
                    }
                }
            }
        }

        Ok(())
    }

    /// Import from standard library
    fn import_library(&self, library: &LibraryImport) -> Result<HashMap<String, Value>> {
        let parts_str: Vec<&str> = library.parts.iter().map(std::string::String::as_str).collect();
        match parts_str.as_slice() {
            ["scheme", "base"] => {
                // Core Scheme functions (already available in builtins)
                Ok(HashMap::new())
            }
            ["scheme", "write"] => {
                // Write-related functions
                Ok(HashMap::new())
            }
            _ => Err(LambdustError::runtime_error(format!(
                "Unknown library: ({})",
                library.parts.join(" ")
            ))),
        }
    }

    /// Get all imported bindings
    #[must_use] pub fn imported_bindings(&self) -> &HashMap<String, Value> {
        &self.imported_bindings
    }

    /// Check if a binding is imported
    #[must_use] pub fn has_binding(&self, name: &str) -> bool {
        self.imported_bindings.contains_key(name)
    }

    /// Get an imported binding
    #[must_use] pub fn get_binding(&self, name: &str) -> Option<&Value> {
        self.imported_bindings.get(name)
    }

    /// List available SRFIs
    #[must_use] pub fn available_srfis(&self) -> Vec<u32> {
        self.srfi_registry.available_srfis()
    }

    /// Get SRFI information
    #[must_use] pub fn srfi_info(&self, id: u32) -> Option<(u32, &str, Vec<&str>)> {
        self.srfi_registry.get_srfi_info(id)
    }

    /// Export a macro for use by other modules
    pub fn export_macro(&mut self, name: String, macro_def: Macro) -> crate::error::Result<()> {
        if self.exported_macros.contains_key(&name) {
            return Err(LambdustError::runtime_error(format!(
                "Macro '{name}' is already exported"
            )));
        }
        self.exported_macros.insert(name, macro_def);
        Ok(())
    }

    /// Import a macro from another module
    pub fn import_macro(&mut self, name: String, macro_def: Macro) -> crate::error::Result<()> {
        if let Some(existing) = self.imported_macros.get(&name) {
            if !macros_equivalent(&macro_def, existing) {
                return Err(LambdustError::runtime_error(format!(
                    "Import conflict for macro '{name}'"
                )));
            }
        }
        self.imported_macros.insert(name, macro_def);
        Ok(())
    }

    /// Get all exported macros
    #[must_use] pub fn get_exported_macros(&self) -> &HashMap<String, Macro> {
        &self.exported_macros
    }

    /// Get all imported macros
    #[must_use] pub fn get_imported_macros(&self) -> &HashMap<String, Macro> {
        &self.imported_macros
    }

    /// Check if a macro is available (imported or exported)
    #[must_use] pub fn has_macro(&self, name: &str) -> bool {
        self.imported_macros.contains_key(name) || self.exported_macros.contains_key(name)
    }

    /// Get a macro by name
    #[must_use] pub fn get_macro(&self, name: &str) -> Option<&Macro> {
        self.imported_macros.get(name)
            .or_else(|| self.exported_macros.get(name))
    }

    /// Export a binding for use by other modules
    pub fn export_binding(&mut self, name: String, value: Value) -> crate::error::Result<()> {
        if self.exported_bindings.contains_key(&name) {
            return Err(LambdustError::runtime_error(format!(
                "Binding '{name}' is already exported"
            )));
        }
        self.exported_bindings.insert(name, value);
        Ok(())
    }

    /// Get all exported bindings
    #[must_use] pub fn get_exported_bindings(&self) -> &HashMap<String, Value> {
        &self.exported_bindings
    }
}

impl Default for ModuleSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two values are equivalent for conflict detection
fn values_equivalent(a: &Value, b: &Value) -> bool {
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

/// Check if two macros are equivalent for conflict detection
fn macros_equivalent(a: &Macro, b: &Macro) -> bool {
    match (a, b) {
        (Macro::SyntaxRules { name: name_a, .. }, Macro::SyntaxRules { name: name_b, .. }) => {
            name_a == name_b
        }
        (Macro::SyntaxCase { name: name_a, .. }, Macro::SyntaxCase { name: name_b, .. }) => {
            name_a == name_b
        }
        (Macro::Builtin { name: name_a, .. }, Macro::Builtin { name: name_b, .. }) => {
            name_a == name_b
        }
        _ => false,
    }
}
