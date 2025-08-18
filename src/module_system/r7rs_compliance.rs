//! R7RS library system compliance implementation.
//!
//! This module ensures full compliance with the R7RS library system specification,
//! including proper handling of:
//! - Library syntax and semantics
//! - Export specifications (rename, only, except)
//! - Import specifications with all modifiers
//! - Standard library integration
//! - Proper scoping and hygiene

use super::{Module, ModuleId, ModuleError, runtime_integration, ModuleNamespace};
use crate::ast::{Expr, Spanned};
use crate::diagnostics::{Error, Result, Span};
use crate::eval::Value;
use std::collections::{HashMap, HashSet};

/// R7RS-compliant library system implementation.
#[derive(Debug)]
pub struct R7RSLibrarySystem {
    /// Standard library registry
    standard_libraries: HashMap<ModuleId, StandardLibraryInfo>,
    /// Export spec resolver
    export_resolver: ExportSpecResolver,
    /// Import spec resolver
    import_resolver: runtime_integration::ImportSpecResolver,
    /// Compliance checking configuration
    compliance_config: ComplianceConfig,
}

/// Information about a standard library.
#[derive(Debug, Clone)]
pub struct StandardLibraryInfo {
    /// Library identifier
    pub id: ModuleId,
    /// Library description
    pub description: String,
    /// Required exports (for compliance checking)
    pub required_exports: HashSet<String>,
    /// Optional exports
    pub optional_exports: HashSet<String>,
    /// Dependencies on other standard libraries
    pub dependencies: Vec<ModuleId>,
    /// R7RS version introduced
    pub introduced_in: R7RSVersion,
}

/// R7RS version enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum R7RSVersion {
    /// R7RS-small (the current standard)
    Small,
    /// R7RS-large (future standard)
    Large,
}

/// Configuration for R7RS compliance checking.
#[derive(Debug, Clone)]
pub struct ComplianceConfig {
    /// Whether to enforce strict R7RS compliance
    pub strict_compliance: bool,
    /// Target R7RS version
    pub target_version: R7RSVersion,
    /// Whether to allow Lambdust extensions
    pub allow_extensions: bool,
    /// Whether to check export completeness
    pub check_export_completeness: bool,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            strict_compliance: true,
            target_version: R7RSVersion::Small,
            allow_extensions: true,
            check_export_completeness: true,
        }
    }
}

/// Export specification resolver for R7RS compliance.
#[derive(Debug)]
pub struct ExportSpecResolver;

impl ExportSpecResolver {
    /// Resolves an export specification according to R7RS rules.
    pub fn resolve_export_spec(
        export_expr: &Spanned<Expr>,
        library_env: &HashMap<String, Value>,
    ) -> Result<Vec<ExportBinding>> {
        match &export_expr.inner {
            Expr::Identifier(name) => {
                // Simple export: just the identifier
                if let Some(value) = library_env.get(name) {
                    Ok(vec![ExportBinding {
                        exported_name: name.clone(),
                        internal_name: name.clone(),
                        value: value.clone(),
                        export_type: ExportType::Direct,
                    }])
                } else {
                    Err(Box::new(Error::from(ModuleError::ExportError(
                        format!("Cannot export undefined binding: {name}")
                    ))))
                }
            }
            Expr::List(elements) if !elements.is_empty() => {
                // Compound export specification
                Self::resolve_compound_export_spec(elements, library_env)
            }
            _ => Err(Box::new(Error::from(ModuleError::ExportError(
                "Invalid export specification format".to_string()
            ))))
        }
    }

    /// Resolves compound export specifications (rename, etc.).
    fn resolve_compound_export_spec(
        elements: &[Spanned<Expr>],
        library_env: &HashMap<String, Value>,
    ) -> Result<Vec<ExportBinding>> {
        if elements.is_empty() {
            return Ok(Vec::new());
        }

        match &elements[0].inner {
            Expr::Identifier(keyword) => {
                match keyword.as_str() {
                    "rename" => Self::resolve_rename_export(&elements[1..], library_env),
                    _ => {
                        // Treat as list of simple exports
                        let mut bindings = Vec::new();
                        for element in elements {
                            if let Expr::Identifier(name) = &element.inner {
                                if let Some(value) = library_env.get(name) {
                                    bindings.push(ExportBinding {
                                        exported_name: name.clone(),
                                        internal_name: name.clone(),
                                        value: value.clone(),
                                        export_type: ExportType::Direct,
                                    });
                                }
                            }
                        }
                        Ok(bindings)
                    }
                }
            }
            _ => Err(Box::new(Error::from(ModuleError::ExportError(
                "Invalid compound export specification".to_string()
            ))))
        }
    }

    /// Resolves "rename" export specifications.
    fn resolve_rename_export(
        elements: &[Spanned<Expr>],
        library_env: &HashMap<String, Value>,
    ) -> Result<Vec<ExportBinding>> {
        let mut bindings = Vec::new();

        for element in elements {
            if let Expr::List(rename_spec) = &element.inner {
                if rename_spec.len() == 2 {
                    if let (Expr::Identifier(internal_name), Expr::Identifier(exported_name)) = 
                        (&rename_spec[0].inner, &rename_spec[1].inner) {
                        
                        if let Some(value) = library_env.get(internal_name) {
                            bindings.push(ExportBinding {
                                exported_name: exported_name.clone(),
                                internal_name: internal_name.clone(),
                                value: value.clone(),
                                export_type: ExportType::Renamed,
                            });
                        } else {
                            return Err(Box::new(Error::from(ModuleError::ExportError(
                                format!("Cannot export undefined binding: {internal_name}")
                            ))));
                        }
                        continue;
                    }
                }
            }
            
            return Err(Box::new(Error::from(ModuleError::ExportError(
                "Invalid rename export specification".to_string()
            ))));
        }

        Ok(bindings)
    }
}

/// Represents an export binding from a library.
#[derive(Debug, Clone)]
pub struct ExportBinding {
    /// The name used for export
    pub exported_name: String,
    /// The internal name in the library
    pub internal_name: String,
    /// The bound value
    pub value: Value,
    /// Type of export
    pub export_type: ExportType,
}

/// Types of export bindings.
#[derive(Debug, Clone, PartialEq)]
pub enum ExportType {
    /// Direct export (same name)
    Direct,
    /// Renamed export
    Renamed,
}

impl R7RSLibrarySystem {
    /// Creates a new R7RS library system.
    pub fn new() -> Self {
        Self::with_config(ComplianceConfig::default())
    }

    /// Creates a library system with custom compliance configuration.
    pub fn with_config(config: ComplianceConfig) -> Self {
        let mut system = Self {
            standard_libraries: HashMap::new(),
            export_resolver: ExportSpecResolver,
            import_resolver: runtime_integration::ImportSpecResolver,
            compliance_config: config,
        };

        system.initialize_standard_libraries();
        system
    }

    /// Validates a library for R7RS compliance.
    pub fn validate_library_compliance(&self, module: &Module) -> Result<ComplianceReport> {
        let mut report = ComplianceReport {
            library_id: module.id.clone(),
            is_compliant: true,
            violations: Vec::new(),
            warnings: Vec::new(),
            standard_library_info: None,
        };

        // Check if this is a standard library
        if let Some(std_info) = self.standard_libraries.get(&module.id) {
            report.standard_library_info = Some(std_info.clone());
            self.validate_standard_library_compliance(module, std_info, &mut report)?;
        }

        // Validate library name format
        self.validate_library_name(&module.id, &mut report);

        // Validate dependencies
        self.validate_dependencies(&module.dependencies, &mut report);

        // Check if any violations make the library non-compliant
        report.is_compliant = report.violations.is_empty() || !self.compliance_config.strict_compliance;

        Ok(report)
    }

    /// Validates standard library compliance.
    fn validate_standard_library_compliance(
        &self,
        module: &Module,
        std_info: &StandardLibraryInfo,
        report: &mut ComplianceReport,
    ) -> Result<()> {
        if self.compliance_config.check_export_completeness {
            // Check that all required exports are present
            for required_export in &std_info.required_exports {
                if !module.exports.contains_key(required_export) {
                    report.violations.push(ComplianceViolation::MissingRequiredExport {
                        export_name: required_export.clone(),
                    });
                }
            }

            // Check for unexpected exports (if strict compliance)
            if self.compliance_config.strict_compliance {
                for export_name in module.exports.keys() {
                    if !std_info.required_exports.contains(export_name) 
                        && !std_info.optional_exports.contains(export_name) {
                        report.warnings.push(ComplianceWarning::UnexpectedExport {
                            export_name: export_name.clone(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Validates library name format according to R7RS rules.
    fn validate_library_name(&self, module_id: &ModuleId, report: &mut ComplianceReport) {
        // R7RS library names must be lists of identifiers and exact integers
        for component in &module_id.components {
            // Check if component is a valid identifier or integer
            if component.is_empty() {
                report.violations.push(ComplianceViolation::InvalidLibraryName {
                    reason: "Empty component in library name".to_string(),
                });
            }
            
            // Additional validation could check for reserved words, etc.
        }

        // Validate namespace-specific rules
        match module_id.namespace {
            ModuleNamespace::R7RS => {
                if module_id.components.is_empty() || module_id.components[0] != "scheme" {
                    report.violations.push(ComplianceViolation::InvalidLibraryName {
                        reason: "R7RS libraries must start with 'scheme'".to_string(),
                    });
                }
            }
            ModuleNamespace::SRFI => {
                if module_id.components.is_empty() || module_id.components[0] != "srfi" {
                    report.violations.push(ComplianceViolation::InvalidLibraryName {
                        reason: "SRFI libraries must start with 'srfi'".to_string(),
                    });
                }
            }
            _ => {} // No specific requirements for other namespaces
        }
    }

    /// Validates library dependencies.
    fn validate_dependencies(&self, dependencies: &[ModuleId], report: &mut ComplianceReport) {
        for dep_id in dependencies {
            // Check if dependency is a known standard library
            if dep_id.namespace == ModuleNamespace::R7RS
                && !self.standard_libraries.contains_key(dep_id) {
                report.warnings.push(ComplianceWarning::UnknownStandardLibrary {
                    library_id: dep_id.clone(),
                });
            }
        }
    }

    /// Initializes the registry of standard libraries.
    fn initialize_standard_libraries(&mut self) {
        // R7RS-small standard libraries
        let standard_libs: [(&str, &str, Vec<&str>, Vec<&str>); 15] = [
            ("base", "Core procedures and syntax", vec!["define", "lambda", "if", "quote", "set!", "cons", "car", "cdr", "list", "null?", "pair?", "eq?", "equal?", "+", "-", "*", "/"], vec![]),
            ("case-lambda", "Case-lambda syntax", vec!["case-lambda"], vec![]),
            ("char", "Character procedures", vec!["char?", "char=?", "char<?", "char>?", "char<=?", "char>=?", "char-alphabetic?", "char-numeric?", "char-whitespace?", "char-upper-case?", "char-lower-case?", "char-upcase", "char-downcase", "char-foldcase"], vec![]),
            ("complex", "Complex number procedures", vec!["complex?", "real?", "rational?", "integer?", "exact?", "inexact?", "exact-integer?", "finite?", "infinite?", "nan?", "make-rectangular", "make-polar", "real-part", "imag-part", "magnitude", "angle"], vec![]),
            ("cxr", "Composed car/cdr procedures", vec!["caar", "cadr", "cdar", "cddr", "caaar", "caadr", "cadar", "caddr", "cdaar", "cdadr", "cddar", "cdddr"], vec![]),
            ("eval", "Evaluation procedures", vec!["eval", "environment"], vec![]),
            ("file", "File I/O procedures", vec!["call-with-input-file", "call-with-output-file", "with-input-from-file", "with-output-to-file", "open-input-file", "open-output-file", "close-input-port", "close-output-port", "read-char", "peek-char", "read-line", "eof-object?", "eof-object", "char-ready?", "write-char", "newline", "write"], vec![]),
            ("inexact", "Inexact number procedures", vec!["inexact", "exact", "floor", "ceiling", "truncate", "round", "exp", "log", "sin", "cos", "tan", "asin", "acos", "atan", "sqrt", "expt"], vec![]),
            ("lazy", "Lazy evaluation", vec!["delay", "force", "promise?", "make-promise"], vec![]),
            ("load", "Dynamic loading", vec!["load"], vec![]),
            ("process-context", "Process context", vec!["command-line", "exit", "emergency-exit", "get-environment-variable", "get-environment-variables"], vec![]),
            ("read", "Reading procedures", vec!["read"], vec![]),
            ("repl", "REPL interaction", vec!["interaction-environment"], vec![]),
            ("time", "Time procedures", vec!["current-second", "current-jiffy", "jiffies-per-second"], vec![]),
            ("write", "Writing procedures", vec!["write", "write-shared", "write-simple", "display"], vec![]),
        ];

        for (name, description, required, optional) in &standard_libs {
            let module_id = ModuleId {
                components: vec!["scheme".to_string(), name.to_string()],
                namespace: ModuleNamespace::R7RS,
            };

            let info = StandardLibraryInfo {
                id: module_id.clone(),
                description: description.to_string(),
                required_exports: required.iter().map(|s| s.to_string()).collect::<HashSet<_>>(),
                optional_exports: optional.iter().map(|s| s.to_string()).collect::<HashSet<_>>(),
                dependencies: Vec::new(), // Most standard libraries are independent
                introduced_in: R7RSVersion::Small,
            };

            self.standard_libraries.insert(module_id, info);
        }

        // Add dependencies for some libraries
        if let Some(char_lib) = self.standard_libraries.get_mut(&ModuleId {
            components: vec!["scheme".to_string(), "char".to_string()],
            namespace: ModuleNamespace::R7RS,
        }) {
            char_lib.dependencies.push(ModuleId {
                components: vec!["scheme".to_string(), "base".to_string()],
                namespace: ModuleNamespace::R7RS,
            });
        }
    }

    /// Gets information about a standard library.
    pub fn get_standard_library_info(&self, module_id: &ModuleId) -> Option<&StandardLibraryInfo> {
        self.standard_libraries.get(module_id)
    }

    /// Lists all standard libraries.
    pub fn list_standard_libraries(&self) -> Vec<&StandardLibraryInfo> {
        self.standard_libraries.values().collect()
    }

    /// Processes an export specification according to R7RS rules.
    pub fn process_export_spec(
        &self,
        export_expr: &Spanned<Expr>,
        library_env: &HashMap<String, Value>,
    ) -> Result<Vec<ExportBinding>> {
        ExportSpecResolver::resolve_export_spec(export_expr, library_env)
    }

    /// Processes an import specification according to R7RS rules.
    pub fn process_import_spec(&self, import_expr: &Spanned<Expr>) -> Result<runtime_integration::ImportResolution> {
        runtime_integration::ImportSpecResolver::resolve_import_spec(import_expr)
    }
}

/// Report on R7RS compliance for a library.
#[derive(Debug, Clone)]
pub struct ComplianceReport {
    /// The library being checked
    pub library_id: ModuleId,
    /// Whether the library is compliant
    pub is_compliant: bool,
    /// List of compliance violations
    pub violations: Vec<ComplianceViolation>,
    /// List of warnings (non-critical issues)
    pub warnings: Vec<ComplianceWarning>,
    /// Standard library information (if applicable)
    pub standard_library_info: Option<StandardLibraryInfo>,
}

/// Types of compliance violations.
#[derive(Debug, Clone)]
pub enum ComplianceViolation {
    /// Invalid library name format
    InvalidLibraryName { 
        /// Reason for invalid library name
        reason: String 
    },
    /// Missing required export for standard library
    MissingRequiredExport { 
        /// Name of the missing required export
        export_name: String 
    },
    /// Invalid export specification
    InvalidExportSpec { 
        /// Reason for invalid export specification
        reason: String 
    },
    /// Invalid import specification
    InvalidImportSpec { 
        /// Reason for invalid import specification
        reason: String 
    },
    /// Circular dependency detected
    CircularDependency { 
        /// Modules involved in the circular dependency
        cycle: Vec<ModuleId> 
    },
}

/// Types of compliance warnings.
#[derive(Debug, Clone)]
pub enum ComplianceWarning {
    /// Unexpected export in standard library
    UnexpectedExport { 
        /// Name of the unexpected export
        export_name: String 
    },
    /// Unknown standard library dependency
    UnknownStandardLibrary { 
        /// ID of the unknown standard library
        library_id: ModuleId 
    },
    /// Use of implementation-specific features
    ImplementationSpecific { 
        /// Name of the implementation-specific feature
        feature: String 
    },
}

impl std::fmt::Display for ComplianceViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplianceViolation::InvalidLibraryName { reason } => {
                write!(f, "Invalid library name: {reason}")
            }
            ComplianceViolation::MissingRequiredExport { export_name } => {
                write!(f, "Missing required export: {export_name}")
            }
            ComplianceViolation::InvalidExportSpec { reason } => {
                write!(f, "Invalid export specification: {reason}")
            }
            ComplianceViolation::InvalidImportSpec { reason } => {
                write!(f, "Invalid import specification: {reason}")
            }
            ComplianceViolation::CircularDependency { cycle } => {
                let cycle_str = cycle.iter()
                    .map(super::format_module_id)
                    .collect::<Vec<_>>()
                    .join(" -> ");
                write!(f, "Circular dependency: {cycle_str}")
            }
        }
    }
}

impl std::fmt::Display for ComplianceWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplianceWarning::UnexpectedExport { export_name } => {
                write!(f, "Unexpected export: {export_name}")
            }
            ComplianceWarning::UnknownStandardLibrary { library_id } => {
                write!(f, "Unknown standard library: {}", super::format_module_id(library_id))
            }
            ComplianceWarning::ImplementationSpecific { feature } => {
                write!(f, "Implementation-specific feature: {feature}")
            }
        }
    }
}

impl ComplianceReport {
    /// Gets a summary of the compliance status.
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str(&format!("Library: {}\n", super::format_module_id(&self.library_id)));
        summary.push_str(&format!("Compliant: {}\n", if self.is_compliant { "Yes" } else { "No" }));
        
        if !self.violations.is_empty() {
            summary.push_str(&format!("Violations ({}):\n", self.violations.len()));
            for violation in &self.violations {
                summary.push_str(&format!("  • {violation}\n"));
            }
        }
        
        if !self.warnings.is_empty() {
            summary.push_str(&format!("Warnings ({}):\n", self.warnings.len()));
            for warning in &self.warnings {
                summary.push_str(&format!("  • {warning}\n"));
            }
        }
        
        if let Some(std_info) = &self.standard_library_info {
            summary.push_str(&format!("Standard Library: {}\n", std_info.description));
            summary.push_str(&format!("Required exports: {}\n", std_info.required_exports.len()));
        }
        
        summary
    }
}

impl Default for R7RSLibrarySystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;
    use crate::module_system::ModuleMetadata;

    #[test]
    fn test_r7rs_library_system_creation() {
        let system = R7RSLibrarySystem::new();
        assert!(!system.standard_libraries.is_empty());
    }

    #[test]
    fn test_standard_library_info() {
        let system = R7RSLibrarySystem::new();
        let base_lib_id = ModuleId {
            components: vec!["scheme".to_string(), "base".to_string()],
            namespace: ModuleNamespace::R7RS,
        };
        
        let info = system.get_standard_library_info(&base_lib_id);
        assert!(info.is_some());
        
        let info = info.unwrap();
        assert_eq!(info.description, "Core procedures and syntax");
        assert!(info.required_exports.contains("define"));
        assert!(info.required_exports.contains("lambda"));
    }

    #[test]
    fn test_simple_export_resolution() {
        let mut library_env = HashMap::new();
        library_env.insert("test-export".to_string(), Value::Integer(42));
        
        let export_expr = Spanned::new(
            Expr::Identifier("test-export".to_string()),
            Span::new(0, 11)
        );
        
        let result = ExportSpecResolver::resolve_export_spec(&export_expr, &library_env);
        assert!(result.is_ok());
        
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].exported_name, "test-export");
        assert_eq!(bindings[0].export_type, ExportType::Direct);
    }

    #[test]
    fn test_rename_export_resolution() {
        let mut library_env = HashMap::new();
        library_env.insert("internal-name".to_string(), Value::Integer(42));
        
        // (rename (internal-name external-name))
        let rename_spec = Spanned::new(
            Expr::List(vec![
                Spanned::new(Expr::Identifier("internal-name".to_string()), Span::new(0, 13)),
                Spanned::new(Expr::Identifier("external-name".to_string()), Span::new(14, 27)),
            ]),
            Span::new(0, 28)
        );
        
        let elements = vec![rename_spec];
        let result = ExportSpecResolver::resolve_rename_export(&elements, &library_env);
        assert!(result.is_ok());
        
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].internal_name, "internal-name");
        assert_eq!(bindings[0].exported_name, "external-name");
        assert_eq!(bindings[0].export_type, ExportType::Renamed);
    }

    #[test]
    fn test_library_name_validation() {
        let system = R7RSLibrarySystem::new();
        
        // Valid R7RS library name
        let valid_module = Module {
            id: ModuleId {
                components: vec!["scheme".to_string(), "base".to_string()],
                namespace: ModuleNamespace::R7RS,
            },
            exports: HashMap::new(),
            dependencies: Vec::new(),
            source: None,
            metadata: ModuleMetadata::default(),
        };
        
        let report = system.validate_library_compliance(&valid_module).unwrap();
        assert!(report.violations.iter().all(|v| !matches!(v, ComplianceViolation::InvalidLibraryName { .. })));
        
        // Invalid R7RS library name (doesn't start with 'scheme')
        let invalid_module = Module {
            id: ModuleId {
                components: vec!["invalid".to_string(), "base".to_string()],
                namespace: ModuleNamespace::R7RS,
            },
            exports: HashMap::new(),
            dependencies: Vec::new(),
            source: None,
            metadata: ModuleMetadata::default(),
        };
        
        let report = system.validate_library_compliance(&invalid_module).unwrap();
        assert!(report.violations.iter().any(|v| matches!(v, ComplianceViolation::InvalidLibraryName { .. })));
    }

    #[test]
    fn test_compliance_config() {
        let config = ComplianceConfig {
            strict_compliance: false,
            target_version: R7RSVersion::Small,
            allow_extensions: true,
            check_export_completeness: false,
        };
        
        let system = R7RSLibrarySystem::with_config(config);
        assert!(!system.compliance_config.strict_compliance);
        assert!(system.compliance_config.allow_extensions);
    }
}