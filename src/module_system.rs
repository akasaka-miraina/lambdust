//! Module system for Scheme import/export functionality
//!
//! This module implements the `(import ...)` special form and module management.

use crate::error::{LambdustError, Result};
use crate::srfi::{parse_srfi_import, SrfiRegistry};
use crate::value::Value;
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
}

impl ModuleSystem {
    /// Create a new module system
    pub fn new() -> Self {
        Self {
            srfi_registry: SrfiRegistry::with_standard_srfis(),
            imported_bindings: HashMap::new(),
        }
    }
    
    /// Parse import specifications from S-expressions
    pub fn parse_import_specs(&self, exprs: &[crate::ast::Expr]) -> Result<Vec<ImportSpec>> {
        let mut specs = Vec::new();
        
        for expr in exprs {
            if let crate::ast::Expr::List(elements) = expr {
                if elements.is_empty() {
                    return Err(LambdustError::syntax_error(
                        "Empty import specification".to_string()
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
                            return Err(LambdustError::syntax_error(
                                format!("Unknown import type: {}", name)
                            ));
                        }
                    }
                } else {
                    return Err(LambdustError::syntax_error(
                        "Import specification must start with a symbol".to_string()
                    ));
                }
            } else {
                return Err(LambdustError::syntax_error(
                    "Import specification must be a list".to_string()
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
                    "Library name parts must be symbols".to_string()
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
                                return Err(LambdustError::runtime_error(
                                    format!("Import conflict for binding '{}'", name)
                                ));
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
                                return Err(LambdustError::runtime_error(
                                    format!("Import conflict for binding '{}'", name)
                                ));
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
        let parts_str: Vec<&str> = library.parts.iter().map(|s| s.as_str()).collect();
        match parts_str.as_slice() {
            ["scheme", "base"] => {
                // Core Scheme functions (already available in builtins)
                Ok(HashMap::new())
            }
            ["scheme", "write"] => {
                // Write-related functions 
                Ok(HashMap::new())
            }
            _ => {
                Err(LambdustError::runtime_error(
                    format!("Unknown library: ({})", library.parts.join(" "))
                ))
            }
        }
    }
    
    /// Get all imported bindings
    pub fn imported_bindings(&self) -> &HashMap<String, Value> {
        &self.imported_bindings
    }
    
    /// Check if a binding is imported
    pub fn has_binding(&self, name: &str) -> bool {
        self.imported_bindings.contains_key(name)
    }
    
    /// Get an imported binding
    pub fn get_binding(&self, name: &str) -> Option<&Value> {
        self.imported_bindings.get(name)
    }
    
    /// List available SRFIs
    pub fn available_srfis(&self) -> Vec<u32> {
        self.srfi_registry.available_srfis()
    }
    
    /// Get SRFI information
    pub fn srfi_info(&self, id: u32) -> Option<(u32, &str, Vec<&str>)> {
        self.srfi_registry.get_srfi_info(id)
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
                (Procedure::Builtin { name: name_a, .. }, Procedure::Builtin { name: name_b, .. }) => {
                    name_a == name_b
                }
                _ => false,
            }
        }
        _ => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::lexer::SchemeNumber;
    
    #[test]
    fn test_parse_srfi_import_spec() {
        let module_system = ModuleSystem::new();
        
        let expr = Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(9))),
        ]);
        
        let specs = module_system.parse_import_specs(&[expr]).unwrap();
        assert_eq!(specs.len(), 1);
        
        if let ImportSpec::Srfi(srfi_import) = &specs[0] {
            assert_eq!(srfi_import.id, 9);
        } else {
            panic!("Expected SRFI import spec");
        }
    }
    
    #[test]
    fn test_parse_library_import_spec() {
        let module_system = ModuleSystem::new();
        
        let expr = Expr::List(vec![
            Expr::Variable("scheme".to_string()),
            Expr::Variable("base".to_string()),
        ]);
        
        let specs = module_system.parse_import_specs(&[expr]).unwrap();
        assert_eq!(specs.len(), 1);
        
        if let ImportSpec::Library(library_import) = &specs[0] {
            assert_eq!(library_import.parts, vec!["scheme", "base"]);
        } else {
            panic!("Expected library import spec");
        }
    }
    
    #[test]
    fn test_available_srfis() {
        let module_system = ModuleSystem::new();
        let available = module_system.available_srfis();
        
        assert!(available.contains(&9));
        assert!(available.contains(&45));
        assert!(available.contains(&46));
    }
}