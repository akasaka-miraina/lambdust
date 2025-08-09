//! High-level Scheme API for FFI operations.
//!
//! This module provides convenient Scheme functions for interacting with
//! native libraries, including automatic wrapper generation and safe
//! function calling interfaces.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use crate::eval::{Value, Environment};
use crate::eval::value::{PrimitiveProcedure, PrimitiveImpl};
use crate::effects::Effect;
use crate::ast::Literal;
use crate::diagnostics::{Error, Result};
use crate::ffi::c_types::CType;
use crate::ffi::safety::{FunctionSignature, TypeConstraint};
#[cfg(feature = "ffi")]
use crate::ffi::libffi_integration::{FfiInterface, LibffiError};

#[cfg(not(feature = "ffi"))]
#[derive(Debug)]
pub struct FfiInterface;

#[cfg(not(feature = "ffi"))]
impl Default for FfiInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl FfiInterface {
    pub fn new() -> Self { Self }
    
    pub fn load_function(
        &self,
        _library_name: &str,
        _function_name: &str,
        _signature: crate::ffi::safety::FunctionSignature,
    ) -> std::result::Result<(), LibffiError> {
        Err(LibffiError)
    }
    
    pub fn call(
        &self,
        _function_name: &str,
        _args: &[crate::eval::Value],
    ) -> std::result::Result<crate::eval::Value, LibffiError> {
        Err(LibffiError)
    }
}

#[cfg(not(feature = "ffi"))]
#[derive(Debug, Clone)]
pub struct LibffiError;

#[cfg(not(feature = "ffi"))]
impl std::fmt::Display for LibffiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FFI not available")
    }
}

#[cfg(not(feature = "ffi"))]
impl std::error::Error for LibffiError {}
use crate::ffi::library::LibraryManager;

/// Errors that can occur in the Scheme FFI API
#[derive(Debug, Clone)]
pub enum SchemeApiError {
    /// Invalid FFI operation
    InvalidOperation {
        operation: String,
        reason: String,
    },
    /// Library definition error
    LibraryDefinitionError {
        library: String,
        error: String,
    },
    /// Function definition error
    FunctionDefinitionError {
        function: String,
        error: String,
    },
    /// Type definition error
    TypeDefinitionError {
        type_name: String,
        error: String,
    },
    /// Wrapper generation error
    WrapperGenerationError {
        target: String,
        error: String,
    },
    /// FFI call error
    CallError(LibffiError),
}

impl fmt::Display for SchemeApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemeApiError::InvalidOperation { operation, reason } => {
                write!(f, "Invalid FFI operation '{operation}': {reason}")
            }
            SchemeApiError::LibraryDefinitionError { library, error } => {
                write!(f, "Library definition error for '{library}': {error}")
            }
            SchemeApiError::FunctionDefinitionError { function, error } => {
                write!(f, "Function definition error for '{function}': {error}")
            }
            SchemeApiError::TypeDefinitionError { type_name, error } => {
                write!(f, "Type definition error for '{type_name}': {error}")
            }
            SchemeApiError::WrapperGenerationError { target, error } => {
                write!(f, "Wrapper generation error for '{target}': {error}")
            }
            SchemeApiError::CallError(e) => {
                write!(f, "FFI call error: {e}")
            }
        }
    }
}

impl std::error::Error for SchemeApiError {}

impl From<LibffiError> for SchemeApiError {
    fn from(e: LibffiError) -> Self {
        SchemeApiError::CallError(e)
    }
}

impl From<SchemeApiError> for Error {
    fn from(api_error: SchemeApiError) -> Self {
        Error::runtime_error(api_error.to_string(), None)
    }
}

/// Library definition for the Scheme FFI API
#[derive(Debug, Clone)]
pub struct LibraryDefinition {
    /// Library name
    pub name: String,
    /// Library path (optional, will search if not provided)
    pub path: Option<String>,
    /// Functions defined in this library
    pub functions: HashMap<String, FunctionDefinition>,
    /// Types defined in this library
    pub types: HashMap<String, TypeDefinition>,
    /// Library metadata
    pub metadata: LibraryMetadata,
}

/// Function definition for the Scheme FFI API
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    /// C function name (if different)
    pub c_name: Option<String>,
    /// Function signature
    pub signature: FunctionSignature,
    /// Scheme wrapper code (generated)
    pub wrapper_code: Option<String>,
    /// Documentation
    pub documentation: Option<String>,
}

/// Type definition for the Scheme FFI API
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// Type name
    pub name: String,
    /// C type
    pub c_type: CType,
    /// Constructor function (if applicable)
    pub constructor: Option<String>,
    /// Destructor function (if applicable)
    pub destructor: Option<String>,
    /// Methods associated with this type
    pub methods: HashMap<String, FunctionDefinition>,
}

/// Library metadata
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct LibraryMetadata {
    /// Library version
    pub version: Option<String>,
    /// Library description
    pub description: Option<String>,
    /// Library author
    pub author: Option<String>,
    /// License information
    pub license: Option<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
}


/// Scheme FFI API manager
#[derive(Debug)]
pub struct SchemeFfiApi {
    /// FFI interface
    ffi_interface: Arc<FfiInterface>,
    /// Library manager
    library_manager: Arc<LibraryManager>,
    /// Registered libraries
    libraries: RwLock<HashMap<String, LibraryDefinition>>,
    /// Generated wrappers cache
    wrapper_cache: RwLock<HashMap<String, String>>,
    /// API configuration
    config: RwLock<ApiConfig>,
}

/// API configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Auto-generate wrappers
    pub auto_generate_wrappers: bool,
    /// Enable safety checks
    pub enable_safety_checks: bool,
    /// Default calling convention
    pub default_calling_convention: String,
    /// Wrapper prefix
    pub wrapper_prefix: String,
    /// Enable documentation generation
    pub generate_documentation: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            auto_generate_wrappers: true,
            enable_safety_checks: true,
            default_calling_convention: "C".to_string(),
            wrapper_prefix: "ffi-".to_string(),
            generate_documentation: true,
        }
    }
}

impl Default for SchemeFfiApi {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemeFfiApi {
    /// Create a new Scheme FFI API
    pub fn new() -> Self {
        Self {
            ffi_interface: Arc::new(FfiInterface::new()),
            library_manager: Arc::new(LibraryManager::new()),
            libraries: RwLock::new(HashMap::new()),
            wrapper_cache: RwLock::new(HashMap::new()),
            config: RwLock::new(ApiConfig::default()),
        }
    }

    /// Configure the API
    pub fn configure(&self, config: ApiConfig) {
        let mut current_config = self.config.write().unwrap();
        *current_config = config;
    }

    /// Define a library
    pub fn define_library(
        &self,
        definition: LibraryDefinition,
    ) -> std::result::Result<(), SchemeApiError> {
        // Validate the library definition
        self.validate_library_definition(&definition)?;

        // Register functions with the FFI interface
        for (func_name, func_def) in &definition.functions {
            let lib_name = &definition.name;
            let full_name = format!("{lib_name}::{func_name}");
            self.ffi_interface
                .load_function(&definition.name, &full_name, func_def.signature.clone())
                .map_err(SchemeApiError::from)?;
        }

        // Generate wrappers if enabled
        if self.config.read().unwrap().auto_generate_wrappers {
            self.generate_library_wrappers(&definition)?;
        }

        // Store the library definition
        {
            let mut libraries = self.libraries.write().unwrap();
            libraries.insert(definition.name.clone(), definition);
        }

        Ok(())
    }

    /// Define a single function
    pub fn define_function(
        &self,
        library_name: &str,
        function_def: FunctionDefinition,
    ) -> std::result::Result<(), SchemeApiError> {
        // Load the function
        self.ffi_interface
            .load_function(library_name, &function_def.name, function_def.signature.clone())
            .map_err(SchemeApiError::from)?;

        // Generate wrapper if enabled
        if self.config.read().unwrap().auto_generate_wrappers {
            let wrapper = self.generate_function_wrapper(&function_def)?;
            let mut cache = self.wrapper_cache.write().unwrap();
            cache.insert(function_def.name.clone(), wrapper);
        }

        // Add to library definition
        {
            let mut libraries = self.libraries.write().unwrap();
            if let Some(lib_def) = libraries.get_mut(library_name) {
                lib_def.functions.insert(function_def.name.clone(), function_def);
            } else {
                return Err(SchemeApiError::LibraryDefinitionError {
                    library: library_name.to_string(),
                    error: "Library not found".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Call a library function
    pub fn call_function(
        &self,
        library_name: &str,
        function_name: &str,
        args: &[Value],
    ) -> std::result::Result<Value, SchemeApiError> {
        let full_name = format!("{library_name}::{function_name}");
        self.ffi_interface
            .call(&full_name, args)
            .map_err(SchemeApiError::from)
    }

    /// Generate wrappers for a library
    fn generate_library_wrappers(
        &self,
        library_def: &LibraryDefinition,
    ) -> std::result::Result<(), SchemeApiError> {
        let mut cache = self.wrapper_cache.write().unwrap();

        for (func_name, func_def) in &library_def.functions {
            let wrapper = self.generate_function_wrapper(func_def)?;
            cache.insert(func_name.clone(), wrapper);
        }

        Ok(())
    }

    /// Generate a Scheme wrapper for a function
    fn generate_function_wrapper(
        &self,
        func_def: &FunctionDefinition,
    ) -> std::result::Result<String, SchemeApiError> {
        let config = self.config.read().unwrap();
        let prefix = &config.wrapper_prefix;
        let func_name = &func_def.name;
        let wrapper_name = format!("{prefix}{func_name}");

        let mut wrapper = String::new();

        // Add documentation if available
        if config.generate_documentation {
            if let Some(doc) = &func_def.documentation {
                wrapper.push_str(&format!(";;; {doc}\n"));
            }
            let params = &func_def.signature.parameters;
            wrapper.push_str(&format!(";;; Parameters: {params:?}\n"));
            let ret_type = &func_def.signature.return_type;
            wrapper.push_str(&format!(";;; Returns: {ret_type:?}\n"));
        }

        // Generate function definition
        wrapper.push_str(&format!("(define ({wrapper_name}"));

        // Add parameters
        for (i, _param_type) in func_def.signature.parameters.iter().enumerate() {
            wrapper.push_str(&format!(" param{i}"));
        }
        wrapper.push_str(")\n");

        // Add safety checks if enabled
        if config.enable_safety_checks {
            wrapper.push_str("  ;; Safety checks\n");
            for (i, (_param_type, constraint)) in func_def.signature.parameters
                .iter()
                .zip(func_def.signature.constraints.iter())
                .enumerate()
            {
                match constraint {
                    TypeConstraint::NonNull(param_idx) if *param_idx == i => {
                        wrapper.push_str(&format!("  (when (null? param{i})\n"));
                        wrapper.push_str(&format!("    (error \"Parameter {i} cannot be null\"))\n"));
                    }
                    TypeConstraint::Bounds { parameter, min, max } if *parameter == i => {
                        wrapper.push_str(&format!("  (when (or (< param{i} {min}) (> param{i} {max}))\n"));
                        wrapper.push_str(&format!("    (error \"Parameter {i} out of bounds [{min}, {max}]\"))\n"));
                    }
                    _ => {}
                }
            }
        }

        // Generate the actual FFI call
        wrapper.push_str(&format!("  (ffi-call '{}' '{}'", 
            func_def.c_name.as_ref().unwrap_or(&func_def.name),
            func_def.name
        ));

        for i in 0..func_def.signature.parameters.len() {
            wrapper.push_str(&format!(" param{i}"));
        }
        wrapper.push_str("))\n");

        Ok(wrapper)
    }

    /// Validate a library definition
    fn validate_library_definition(
        &self,
        library_def: &LibraryDefinition,
    ) -> std::result::Result<(), SchemeApiError> {
        // Check if library name is valid
        if library_def.name.is_empty() {
            return Err(SchemeApiError::LibraryDefinitionError {
                library: library_def.name.clone(),
                error: "Library name cannot be empty".to_string(),
            });
        }

        // Validate each function
        for (func_name, func_def) in &library_def.functions {
            if func_name != &func_def.name {
                return Err(SchemeApiError::FunctionDefinitionError {
                    function: func_name.clone(),
                    error: "Function name mismatch".to_string(),
                });
            }

            // Validate function signature
            if func_def.signature.parameters.len() > 32 {
                return Err(SchemeApiError::FunctionDefinitionError {
                    function: func_name.clone(),
                    error: "Too many parameters (max 32)".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Get library definition
    pub fn get_library(&self, name: &str) -> Option<LibraryDefinition> {
        let libraries = self.libraries.read().unwrap();
        libraries.get(name).cloned()
    }

    /// List all libraries
    pub fn list_libraries(&self) -> Vec<String> {
        let libraries = self.libraries.read().unwrap();
        libraries.keys().cloned().collect()
    }

    /// Get generated wrapper
    pub fn get_wrapper(&self, function_name: &str) -> Option<String> {
        let cache = self.wrapper_cache.read().unwrap();
        cache.get(function_name).cloned()
    }

    /// Export library as Scheme module
    pub fn export_as_scheme_module(
        &self,
        library_name: &str,
    ) -> std::result::Result<String, SchemeApiError> {
        let library = self.get_library(library_name)
            .ok_or_else(|| SchemeApiError::LibraryDefinitionError {
                library: library_name.to_string(),
                error: "Library not found".to_string(),
            })?;

        let mut module = String::new();

        // Module header
        module.push_str(&format!(";;; Generated FFI module for {library_name}\n"));
        if let Some(description) = &library.metadata.description {
            module.push_str(&format!(";;; {description}\n"));
        }
        module.push('\n');

        // Module declaration
        module.push_str(&format!("(define-library (ffi {library_name})\n"));
        
        // Exports
        module.push_str("  (export\n");
        for func_name in library.functions.keys() {
            let config = self.config.read().unwrap();
            let prefix = &config.wrapper_prefix;
            let wrapper_name = format!("{prefix}{func_name}");
            module.push_str(&format!("    {wrapper_name}\n"));
        }
        module.push_str("  )\n");

        // Imports
        module.push_str("  (import (scheme base)\n");
        module.push_str("          (lambdust ffi))\n");

        // Begin
        module.push_str("  (begin\n");

        // Function definitions
        for func_name in library.functions.keys() {
            if let Some(wrapper) = self.get_wrapper(func_name) {
                // Indent the wrapper code
                let indented_wrapper = wrapper
                    .lines()
                    .map(|line| format!("    {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                module.push_str(&indented_wrapper);
                module.push_str("\n\n");
            }
        }

        module.push_str("  ))\n");

        Ok(module)
    }

    /// Generate C header from library definition
    pub fn generate_c_header(
        &self,
        library_name: &str,
    ) -> std::result::Result<String, SchemeApiError> {
        let library = self.get_library(library_name)
            .ok_or_else(|| SchemeApiError::LibraryDefinitionError {
                library: library_name.to_string(),
                error: "Library not found".to_string(),
            })?;

        let mut header = String::new();

        // Header guard
        let lib_upper = library_name.to_uppercase();
        let guard = format!("{lib_upper}_H");
        header.push_str(&format!("#ifndef {guard}\n"));
        header.push_str(&format!("#define {guard}\n\n"));

        // Includes
        header.push_str("#include <stdint.h>\n");
        header.push_str("#include <stddef.h>\n\n");

        // Type definitions
        for (type_name, type_def) in &library.types {
            match &type_def.c_type {
                CType::Struct { name, fields, .. } => {
                    header.push_str(&format!("typedef struct {name} {{\n"));
                    for field in fields {
                        let field_type = &field.c_type;
                        let field_name = &field.name;
                        header.push_str(&format!("    {field_type} {field_name};\n"));
                    }
                    header.push_str(&format!("}} {type_name};\n\n"));
                }
                _ => {
                    let c_type = &type_def.c_type;
                    header.push_str(&format!("typedef {c_type} {type_name};\n\n"));
                }
            }
        }

        // Function declarations
        for (func_name, func_def) in &library.functions {
            let ret_type = &func_def.signature.return_type;
            header.push_str(&format!("{ret_type} {func_name}("));
            
            if func_def.signature.parameters.is_empty() {
                header.push_str("void");
            } else {
                for (i, param_type) in func_def.signature.parameters.iter().enumerate() {
                    if i > 0 {
                        header.push_str(", ");
                    }
                    header.push_str(&format!("{param_type} param{i}"));
                }
            }
            
            header.push_str(");\n");
        }

        // Header guard end
        header.push_str(&format!("\n#endif /* {guard} */\n"));

        Ok(header)
    }
}

/// Built-in FFI functions for Scheme
pub struct FfiBuiltins;

impl FfiBuiltins {
    /// Register built-in FFI functions
    pub fn register_builtins(env: &mut Environment) {
        // (ffi-load-library name [path])
        env.define("ffi-load-library".to_string(), Value::Primitive(Arc::new(
            PrimitiveProcedure {
                name: "ffi-load-library".to_string(),
                arity_min: 1,
                arity_max: Some(2),
                implementation: PrimitiveImpl::RustFn(ffi_load_library),
                effects: vec![Effect::IO],
            }
        )));

        // (ffi-define-function library-name func-name signature)
        env.define("ffi-define-function".to_string(), Value::Primitive(Arc::new(
            PrimitiveProcedure {
                name: "ffi-define-function".to_string(),
                arity_min: 3,
                arity_max: Some(3),
                implementation: PrimitiveImpl::RustFn(ffi_define_function),
                effects: vec![Effect::State],
            }
        )));

        // (ffi-call library-name func-name . args)
        env.define("ffi-call".to_string(), Value::Primitive(Arc::new(
            PrimitiveProcedure {
                name: "ffi-call".to_string(),
                arity_min: 2,
                arity_max: None, // Variadic
                implementation: PrimitiveImpl::RustFn(ffi_call),
                effects: vec![Effect::IO],
            }
        )));

        // (ffi-define-struct name . fields)
        env.define("ffi-define-struct".to_string(), Value::Primitive(Arc::new(
            PrimitiveProcedure {
                name: "ffi-define-struct".to_string(),
                arity_min: 1,
                arity_max: None,
                implementation: PrimitiveImpl::RustFn(ffi_define_struct),
                effects: vec![Effect::State],
            }
        )));

        // (ffi-sizeof type)
        env.define("ffi-sizeof".to_string(), Value::Primitive(Arc::new(
            PrimitiveProcedure {
                name: "ffi-sizeof".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::RustFn(ffi_sizeof),
                effects: vec![Effect::Pure],
            }
        )));

        // (ffi-null? ptr)
        env.define("ffi-null?".to_string(), Value::Primitive(Arc::new(
            PrimitiveProcedure {
                name: "ffi-null?".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::RustFn(ffi_null_p),
                effects: vec![Effect::Pure],
            }
        )));
    }
}

// ============= PRIMITIVE FUNCTION IMPLEMENTATIONS =============

/// Implementation of (ffi-load-library name [path])
fn ffi_load_library(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error("ffi-load-library expects 1 or 2 arguments".to_string(), None)));
    }

    let _library_name = match &args[0] {
        Value::Literal(Literal::String(s)) => s.clone(),
        _ => return Err(Box::new(Error::runtime_error("Library name must be a string".to_string(), None))),
    };

    // Load the library (simplified implementation)
    // In practice, this would use the global library manager
    Ok(Value::Literal(Literal::Boolean(true)))
}

/// Implementation of (ffi-define-function library-name func-name signature)
fn ffi_define_function(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::runtime_error("ffi-define-function expects 3 arguments".to_string(), None)));
    }

    // Implementation would parse the signature and register the function
    Ok(Value::Literal(Literal::Boolean(true)))
}

/// Implementation of (ffi-call library-name func-name . args)
fn ffi_call(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error("ffi-call expects at least 2 arguments".to_string(), None)));
    }

    let _library_name = match &args[0] {
        Value::Literal(Literal::String(s)) => s.clone(),
        _ => return Err(Box::new(Error::runtime_error("Library name must be a string".to_string(), None))),
    };

    let _function_name = match &args[1] {
        Value::Literal(Literal::String(s)) => s.clone(),
        _ => return Err(Box::new(Error::runtime_error("Function name must be a string".to_string(), None))),
    };

    let _func_args = &args[2..];

    // Implementation would call the actual FFI function
    // For now, return a placeholder
    Ok(Value::Literal(Literal::Number(0.0)))
}

/// Implementation of (ffi-define-struct name . fields)
fn ffi_define_struct(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::runtime_error("ffi-define-struct expects at least 1 argument".to_string(), None)));
    }

    // Implementation would define a struct type
    Ok(Value::Literal(Literal::Boolean(true)))
}

/// Implementation of (ffi-sizeof type)
fn ffi_sizeof(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("ffi-sizeof expects 1 argument".to_string(), None)));
    }

    // Implementation would return the size of the type
    Ok(Value::Literal(Literal::Number(4.0))) // Placeholder
}

/// Implementation of (ffi-null? ptr)
fn ffi_null_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("ffi-null? expects 1 argument".to_string(), None)));
    }

    // Check if the argument represents a null pointer
    match &args[0] {
        Value::Nil => Ok(Value::Literal(Literal::Boolean(true))),
        _ => Ok(Value::Literal(Literal::Boolean(false))),
    }
}

lazy_static::lazy_static! {
    /// Global Scheme FFI API instance
    pub static ref GLOBAL_SCHEME_FFI_API: SchemeFfiApi = SchemeFfiApi::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme_ffi_api_creation() {
        let api = SchemeFfiApi::new();
        let libraries = api.list_libraries();
        assert!(libraries.is_empty());
    }

    #[test]
    fn test_library_definition() {
        let api = SchemeFfiApi::new();
        
        let lib_def = LibraryDefinition {
            name: "test_lib".to_string(),
            path: None,
            functions: HashMap::new(),
            types: HashMap::new(),
            metadata: LibraryMetadata::default(),
        };

        let result = api.define_library(lib_def);
        assert!(result.is_ok());
        
        let libraries = api.list_libraries();
        assert!(libraries.contains(&"test_lib".to_string()));
    }

    #[test]
    fn test_function_wrapper_generation() {
        let api = SchemeFfiApi::new();
        
        let func_def = FunctionDefinition {
            name: "test_func".to_string(),
            c_name: None,
            signature: FunctionSignature {
                name: "test_func".to_string(),
                parameters: vec![CType::CInt, CType::CString],
                return_type: CType::CInt,
                variadic: false,
                safe: true,
                constraints: vec![],
            },
            wrapper_code: None,
            documentation: Some("A test function".to_string()),
        };

        let wrapper = api.generate_function_wrapper(&func_def).unwrap();
        assert!(wrapper.contains("ffi-test_func"));
        assert!(wrapper.contains("A test function"));
        assert!(wrapper.contains("param0"));
        assert!(wrapper.contains("param1"));
    }

    #[test]
    fn test_c_header_generation() {
        let api = SchemeFfiApi::new();
        
        let mut functions = HashMap::new();
        functions.insert("test_func".to_string(), FunctionDefinition {
            name: "test_func".to_string(),
            c_name: None,
            signature: FunctionSignature {
                name: "test_func".to_string(),
                parameters: vec![CType::CInt],
                return_type: CType::CInt,
                variadic: false,
                safe: true,
                constraints: vec![],
            },
            wrapper_code: None,
            documentation: None,
        });

        let lib_def = LibraryDefinition {
            name: "test_lib".to_string(),
            path: None,
            functions,
            types: HashMap::new(),
            metadata: LibraryMetadata::default(),
        };

        api.define_library(lib_def).unwrap();
        
        let header = api.generate_c_header("test_lib").unwrap();
        assert!(header.contains("#ifndef TEST_LIB_H"));
        assert!(header.contains("int test_func(int param0);"));
    }

    #[test]
    fn test_scheme_module_export() {
        let api = SchemeFfiApi::new();
        
        let mut functions = HashMap::new();
        functions.insert("test_func".to_string(), FunctionDefinition {
            name: "test_func".to_string(),
            c_name: None,
            signature: FunctionSignature {
                name: "test_func".to_string(),
                parameters: vec![],
                return_type: CType::CInt,
                variadic: false,
                safe: true,
                constraints: vec![],
            },
            wrapper_code: None,
            documentation: None,
        });

        let lib_def = LibraryDefinition {
            name: "test_lib".to_string(),
            path: None,
            functions,
            types: HashMap::new(),
            metadata: LibraryMetadata {
                description: Some("A test library".to_string()),
                ..Default::default()
            },
        };

        api.define_library(lib_def).unwrap();
        
        let module = api.export_as_scheme_module("test_lib").unwrap();
        assert!(module.contains("(define-library (ffi test_lib)"));
        assert!(module.contains("A test library"));
        assert!(module.contains("ffi-test_func"));
    }
}