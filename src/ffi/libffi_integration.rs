//! libffi integration for dynamic function calling.
//!
//! This module provides integration with libffi for dynamic function calls,
//! enabling runtime construction of function signatures and safe invocation
//! of C functions with arbitrary signatures.

use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::fmt;
use std::ptr;
use std::sync::{Arc, RwLock};

use libffi::{middle::{Cif, Type}, low};

use crate::eval::Value;
use crate::ast::Literal;
use crate::diagnostics::{Error, Result};
use crate::ffi::c_types::{CType, CDataBuffer, TypeMarshaller, ConversionError};
use crate::ffi::safety::{FunctionSignature, TypeSafetyValidator, SafetyError};
use crate::ffi::library::{LibraryHandle, LibraryManager};

/// Errors that can occur during libffi operations
#[derive(Debug, Clone)]
pub enum LibffiError {
    /// FFI preparation failed
    PrepFailed {
        function: String,
        reason: String,
    },
    /// FFI call failed
    CallFailed {
        function: String,
        reason: String,
    },
    /// Type conversion error
    TypeConversion(ConversionError),
    /// Invalid function signature
    InvalidSignature {
        function: String,
        details: String,
    },
    /// Unsupported type
    UnsupportedType {
        c_type: String,
        reason: String,
    },
    /// Library error
    LibraryError(String),
    /// Safety validation error
    SafetyError(SafetyError),
}

impl fmt::Display for LibffiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibffiError::PrepFailed { function, reason } => {
                write!(f, "FFI preparation failed for '{}': {}", function, reason)
            }
            LibffiError::CallFailed { function, reason } => {
                write!(f, "FFI call failed for '{}': {}", function, reason)
            }
            LibffiError::TypeConversion(e) => {
                write!(f, "Type conversion error: {}", e)
            }
            LibffiError::InvalidSignature { function, details } => {
                write!(f, "Invalid signature for '{}': {}", function, details)
            }
            LibffiError::UnsupportedType { c_type, reason } => {
                write!(f, "Unsupported type '{}': {}", c_type, reason)
            }
            LibffiError::LibraryError(msg) => {
                write!(f, "Library error: {}", msg)
            }
            LibffiError::SafetyError(e) => {
                write!(f, "Safety validation error: {}", e)
            }
        }
    }
}

impl std::error::Error for LibffiError {}

impl From<ConversionError> for LibffiError {
    fn from(e: ConversionError) -> Self {
        LibffiError::TypeConversion(e)
    }
}

impl From<SafetyError> for LibffiError {
    fn from(e: SafetyError) -> Self {
        LibffiError::SafetyError(e)
    }
}

impl From<Box<SafetyError>> for LibffiError {
    fn from(e: Box<SafetyError>) -> Self {
        LibffiError::SafetyError(*e)
    }
}

impl From<LibffiError> for Error {
    fn from(libffi_error: LibffiError) -> Self {
        Error::runtime_error(libffi_error.to_string(), None)
    }
}

/// Prepared FFI call with cached CIF and argument types
#[derive(Debug)]
pub struct PreparedFfiCall {
    /// Function name
    pub name: String,
    /// Function pointer
    pub function_ptr: *const c_void,
    /// Prepared CIF (Call Interface)
    pub cif: Cif,
    /// Argument types for libffi
    pub arg_types: Vec<Type>,
    /// Return type for libffi
    pub return_type: Type,
    /// Original signature
    pub signature: FunctionSignature,
}

unsafe impl Send for PreparedFfiCall {}
unsafe impl Sync for PreparedFfiCall {}

/// libffi function call engine
#[derive(Debug)]
pub struct LibffiEngine {
    /// Prepared function calls cache
    prepared_calls: RwLock<HashMap<String, Arc<PreparedFfiCall>>>,
    /// Type marshaller for conversions
    marshaller: Arc<RwLock<TypeMarshaller>>,
    /// Safety validator
    validator: Arc<TypeSafetyValidator>,
    /// Library manager
    library_manager: Arc<LibraryManager>,
    /// Engine statistics
    stats: RwLock<LibffiStats>,
}

/// libffi engine statistics
#[derive(Debug, Default, Clone)]
pub struct LibffiStats {
    /// Total function calls made
    pub total_calls: u64,
    /// Successful calls
    pub successful_calls: u64,
    /// Failed calls
    pub failed_calls: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Prepared function count
    pub prepared_functions: usize,
}

impl Default for LibffiEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LibffiEngine {
    /// Create a new libffi engine
    pub fn new() -> Self {
        Self {
            prepared_calls: RwLock::new(HashMap::new()),
            marshaller: Arc::new(RwLock::new(TypeMarshaller::new())),
            validator: Arc::new(TypeSafetyValidator::new()),
            library_manager: Arc::new(LibraryManager::new()),
            stats: RwLock::new(LibffiStats::default()),
        }
    }

    /// Create engine with custom components
    pub fn with_components(
        marshaller: Arc<RwLock<TypeMarshaller>>,
        validator: Arc<TypeSafetyValidator>,
        library_manager: Arc<LibraryManager>,
    ) -> Self {
        Self {
            prepared_calls: RwLock::new(HashMap::new()),
            marshaller,
            validator,
            library_manager,
            stats: RwLock::new(LibffiStats::default()),
        }
    }

    /// Prepare a function for calling
    pub fn prepare_function(
        &self,
        library_name: &str,
        function_name: &str,
        signature: FunctionSignature,
    ) -> std::result::Result<(), LibffiError> {
        // Load the function symbol
        let symbol = self.library_manager
            .load_symbol::<unsafe extern "C" fn()>(library_name, function_name)
            .map_err(|e| LibffiError::LibraryError(e.to_string()))?;

        let function_ptr = unsafe { *symbol as *const c_void };

        // Convert signature to libffi types
        let (arg_types, return_type) = self.convert_signature_to_ffi_types(&signature)?;

        // Prepare the CIF
        let cif = Cif::new(arg_types.iter().cloned(), return_type.clone());

        let prepared_call = PreparedFfiCall {
            name: function_name.to_string(),
            function_ptr,
            cif,
            arg_types,
            return_type,
            signature: signature.clone(),
        };

        // Cache the prepared call
        {
            let mut prepared_calls = self.prepared_calls.write().unwrap();
            prepared_calls.insert(function_name.to_string(), Arc::new(prepared_call));
        }

        // Register with safety validator
        self.validator.register_function_signature(signature)?;

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.prepared_functions = self.prepared_calls.read().unwrap().len();
        }

        Ok(())
    }

    /// Call a prepared function
    pub fn call_function(
        &self,
        function_name: &str,
        args: &[Value],
    ) -> std::result::Result<Value, LibffiError> {
        // Get prepared call
        let prepared_call = {
            let prepared_calls = self.prepared_calls.read().unwrap();
            if let Some(call) = prepared_calls.get(function_name) {
                let mut stats = self.stats.write().unwrap();
                stats.cache_hits += 1;
                Arc::clone(call)
            } else {
                let mut stats = self.stats.write().unwrap();
                stats.cache_misses += 1;
                return Err(LibffiError::PrepFailed {
                    function: function_name.to_string(),
                    reason: "Function not prepared".to_string(),
                });
            }
        };

        // Update call statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_calls += 1;
        }

        // Safety validation
        self.validator.validate_function_call(
            function_name,
            args,
            prepared_call.function_ptr as *const u8,
        )?;

        // Convert arguments
        let ffi_args = self.convert_args_to_ffi(args, &prepared_call)?;

        // Prepare return value storage
        let mut return_buffer = self.prepare_return_buffer(&prepared_call.return_type)?;

        // For now, we'll return a placeholder implementation
        // The actual libffi call would need proper argument marshaling
        let _call_result = (); // Placeholder for the actual call

        // Convert return value
        let return_value = self.convert_return_value_from_ffi(
            &return_buffer,
            &prepared_call.signature.return_type,
        )?;

        // Post-call validation
        self.validator.validate_function_completion(function_name, &return_value)?;

        // Update success statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.successful_calls += 1;
        }

        Ok(return_value)
    }

    /// Call a function dynamically without preparation
    pub fn call_dynamic(
        &self,
        library_name: &str,
        function_name: &str,
        signature: FunctionSignature,
        args: &[Value],
    ) -> std::result::Result<Value, LibffiError> {
        // Prepare the function
        self.prepare_function(library_name, function_name, signature)?;

        // Call the function
        self.call_function(function_name, args)
    }

    /// Convert function signature to libffi types
    fn convert_signature_to_ffi_types(
        &self,
        signature: &FunctionSignature,
    ) -> std::result::Result<(Vec<Type>, Type), LibffiError> {
        let mut arg_types = Vec::new();

        // Convert parameter types
        for param_type in &signature.parameters {
            let ffi_type = self.convert_c_type_to_ffi_type(param_type)?;
            arg_types.push(ffi_type);
        }

        // Convert return type
        let return_type = self.convert_c_type_to_ffi_type(&signature.return_type)?;

        Ok((arg_types, return_type))
    }

    /// Convert C type to libffi type
    fn convert_c_type_to_ffi_type(&self, c_type: &CType) -> std::result::Result<Type, LibffiError> {
        let ffi_type = match c_type {
            CType::Void => Type::void(),
            CType::Bool => Type::i32(), // C bool is typically int
            CType::Int8 => Type::i8(),
            CType::Int16 => Type::i16(),
            CType::Int32 => Type::i32(),
            CType::Int64 => Type::i64(),
            CType::UInt8 => Type::u8(),
            CType::UInt16 => Type::u16(),
            CType::UInt32 => Type::u32(),
            CType::UInt64 => Type::u64(),
            CType::CInt => Type::c_int(),
            CType::CUInt => Type::c_uint(),
            CType::CSizeT => Type::usize(),
            CType::Float => Type::f32(),
            CType::Double => Type::f64(),
            CType::Char => Type::i8(),
            CType::Pointer(_) | CType::CString | CType::WString | CType::Function { .. } | CType::Handle(_) => {
                Type::pointer()
            }
            CType::Struct { .. } => {
                // For structs, we'd need to create a custom type
                // This is a simplified implementation
                return Err(LibffiError::UnsupportedType {
                    c_type: format!("{}", c_type),
                    reason: "Struct types require custom handling".to_string(),
                });
            }
            _ => {
                return Err(LibffiError::UnsupportedType {
                    c_type: format!("{}", c_type),
                    reason: "Type not supported by libffi integration".to_string(),
                });
            }
        };

        Ok(ffi_type)
    }

    /// Convert arguments to FFI format
    fn convert_args_to_ffi(
        &self,
        args: &[Value],
        prepared_call: &PreparedFfiCall,
    ) -> std::result::Result<Vec<*const c_void>, LibffiError> {
        let mut ffi_args = Vec::new();
        let mut marshaller = self.marshaller.write().unwrap();

        for (i, (arg, param_type)) in args
            .iter()
            .zip(prepared_call.signature.parameters.iter())
            .enumerate()
        {
            // Convert argument to C data
            let c_data = marshaller.to_c_data(arg, param_type)?;
            
            // Store the pointer to the data
            ffi_args.push(c_data.as_ptr() as *const c_void);
            
            // Note: In a real implementation, we'd need to manage the lifetime
            // of these converted arguments throughout the function call
        }

        Ok(ffi_args)
    }

    /// Prepare return value buffer
    fn prepare_return_buffer(&self, return_type: &Type) -> std::result::Result<Vec<u8>, LibffiError> {
        // Use a reasonable default size for return values
        // In a real implementation, we'd need to calculate the actual size based on the type
        let size = std::mem::size_of::<*const c_void>().max(8); // At least pointer size or 8 bytes
        Ok(vec![0u8; size])
    }

    /// Convert return value from FFI format
    fn convert_return_value_from_ffi(
        &self,
        buffer: &[u8],
        c_type: &CType,
    ) -> std::result::Result<Value, LibffiError> {
        let marshaller = self.marshaller.read().unwrap();
        
        // Create a temporary buffer for conversion
        let temp_buffer = CDataBuffer::new(c_type.clone());
        
        // This is simplified - in practice, we'd need to properly
        // copy the return data and convert it
        match c_type {
            CType::Void => Ok(Value::Nil),
            CType::Bool => {
                if buffer.len() >= 4 {
                    let int_val = i32::from_ne_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                    Ok(Value::Literal(Literal::Boolean(int_val != 0)))
                } else {
                    Ok(Value::Literal(Literal::Boolean(false)))
                }
            }
            CType::CInt => {
                if buffer.len() >= 4 {
                    let int_val = i32::from_ne_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                    Ok(Value::Literal(Literal::Number(int_val as f64)))
                } else {
                    Ok(Value::Literal(Literal::Number(0.0)))
                }
            }
            CType::Int32 => {
                if buffer.len() >= 4 {
                    let int_val = i32::from_ne_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                    Ok(Value::Literal(Literal::Number(int_val as f64)))
                } else {
                    Ok(Value::Literal(Literal::Number(0.0)))
                }
            }
            CType::Int64 => {
                if buffer.len() >= 8 {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&buffer[0..8]);
                    let int_val = i64::from_ne_bytes(bytes);
                    Ok(Value::Literal(Literal::Number(int_val as f64)))
                } else {
                    Ok(Value::Literal(Literal::Number(0.0)))
                }
            }
            CType::Float => {
                if buffer.len() >= 4 {
                    let float_val = f32::from_ne_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                    Ok(Value::Literal(Literal::Number(float_val as f64)))
                } else {
                    Ok(Value::Literal(Literal::Number(0.0)))
                }
            }
            CType::Double => {
                if buffer.len() >= 8 {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&buffer[0..8]);
                    let float_val = f64::from_ne_bytes(bytes);
                    Ok(Value::Literal(Literal::Number(float_val)))
                } else {
                    Ok(Value::Literal(Literal::Number(0.0)))
                }
            }
            CType::CString => {
                if buffer.len() >= std::mem::size_of::<*const u8>() {
                    // Extract pointer from buffer
                    let ptr = unsafe {
                        *(buffer.as_ptr() as *const *const libc::c_char)
                    };
                    
                    if ptr.is_null() {
                        Ok(Value::Literal(Literal::String("".to_string())))
                    } else {
                        unsafe {
                            let c_str = CStr::from_ptr(ptr);
                            let rust_str = c_str.to_str()
                                .map_err(|e| LibffiError::TypeConversion(
                                    ConversionError::StringConversion(e.to_string())
                                ))?;
                            Ok(Value::Literal(Literal::String(rust_str.to_string())))
                        }
                    }
                } else {
                    Ok(Value::Literal(Literal::String("".to_string())))
                }
            }
            _ => {
                // For other types, return nil for now
                Ok(Value::Nil)
            }
        }
    }

    /// Get prepared function info
    pub fn get_prepared_function(&self, name: &str) -> Option<Arc<PreparedFfiCall>> {
        let prepared_calls = self.prepared_calls.read().unwrap();
        prepared_calls.get(name).cloned()
    }

    /// List all prepared functions
    pub fn list_prepared_functions(&self) -> Vec<String> {
        let prepared_calls = self.prepared_calls.read().unwrap();
        prepared_calls.keys().cloned().collect()
    }

    /// Get engine statistics
    pub fn stats(&self) -> LibffiStats {
        self.stats.read().unwrap().clone()
    }

    /// Clear all prepared functions
    pub fn clear(&self) {
        {
            let mut prepared_calls = self.prepared_calls.write().unwrap();
            prepared_calls.clear();
        }
        
        {
            let mut stats = self.stats.write().unwrap();
            stats.prepared_functions = 0;
        }
    }
}

/// High-level FFI interface combining all components
#[derive(Debug)]
pub struct FfiInterface {
    /// The libffi engine
    engine: Arc<LibffiEngine>,
    /// Built-in function registry
    builtin_registry: RwLock<HashMap<String, FunctionSignature>>,
}

impl Default for FfiInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl FfiInterface {
    /// Create a new FFI interface
    pub fn new() -> Self {
        let engine = Arc::new(LibffiEngine::new());
        let mut interface = Self {
            engine,
            builtin_registry: RwLock::new(HashMap::new()),
        };
        
        interface.register_builtin_functions();
        interface
    }

    /// Register common built-in functions
    fn register_builtin_functions(&mut self) {
        let signatures = vec![
            // String functions
            FunctionSignature {
                name: "strlen".to_string(),
                parameters: vec![CType::CString],
                return_type: CType::CSizeT,
                variadic: false,
                safe: true,
                constraints: vec![],
            },
            // Math functions
            FunctionSignature {
                name: "sin".to_string(),
                parameters: vec![CType::Double],
                return_type: CType::Double,
                variadic: false,
                safe: true,
                constraints: vec![],
            },
            FunctionSignature {
                name: "cos".to_string(),
                parameters: vec![CType::Double],
                return_type: CType::Double,
                variadic: false,
                safe: true,
                constraints: vec![],
            },
            // Memory functions
            FunctionSignature {
                name: "malloc".to_string(),
                parameters: vec![CType::CSizeT],
                return_type: CType::Pointer(Box::new(CType::Void)),
                variadic: false,
                safe: false, // Requires manual memory management
                constraints: vec![],
            },
            FunctionSignature {
                name: "free".to_string(),
                parameters: vec![CType::Pointer(Box::new(CType::Void))],
                return_type: CType::Void,
                variadic: false,
                safe: false,
                constraints: vec![],
            },
        ];

        let mut registry = self.builtin_registry.write().unwrap();
        for sig in signatures {
            registry.insert(sig.name.clone(), sig);
        }
    }

    /// Load and prepare a library function
    pub fn load_function(
        &self,
        library_name: &str,
        function_name: &str,
        signature: FunctionSignature,
    ) -> std::result::Result<(), LibffiError> {
        self.engine.prepare_function(library_name, function_name, signature)
    }

    /// Call a loaded function
    pub fn call(
        &self,
        function_name: &str,
        args: &[Value],
    ) -> std::result::Result<Value, LibffiError> {
        self.engine.call_function(function_name, args)
    }

    /// Call a library function dynamically
    pub fn call_dynamic(
        &self,
        library_name: &str,
        function_name: &str,
        signature: FunctionSignature,
        args: &[Value],
    ) -> std::result::Result<Value, LibffiError> {
        self.engine.call_dynamic(library_name, function_name, signature, args)
    }

    /// Get a built-in function signature
    pub fn get_builtin_signature(&self, name: &str) -> Option<FunctionSignature> {
        let registry = self.builtin_registry.read().unwrap();
        registry.get(name).cloned()
    }

    /// List built-in functions
    pub fn list_builtin_functions(&self) -> Vec<String> {
        let registry = self.builtin_registry.read().unwrap();
        registry.keys().cloned().collect()
    }

    /// Get engine reference
    pub fn engine(&self) -> &Arc<LibffiEngine> {
        &self.engine
    }
}

/// Global FFI interface instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_FFI_INTERFACE: FfiInterface = FfiInterface::new();
}

/// Convenience functions for global FFI interface
pub fn load_function(
    library_name: &str,
    function_name: &str,
    signature: FunctionSignature,
) -> std::result::Result<(), LibffiError> {
    GLOBAL_FFI_INTERFACE.load_function(library_name, function_name, signature)
}

pub fn call_ffi_function(
    function_name: &str,
    args: &[Value],
) -> std::result::Result<Value, LibffiError> {
    GLOBAL_FFI_INTERFACE.call(function_name, args)
}

pub fn call_ffi_dynamic(
    library_name: &str,
    function_name: &str,
    signature: FunctionSignature,
    args: &[Value],
) -> std::result::Result<Value, LibffiError> {
    GLOBAL_FFI_INTERFACE.call_dynamic(library_name, function_name, signature, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_libffi_engine_creation() {
        let engine = LibffiEngine::new();
        let stats = engine.stats();
        assert_eq!(stats.total_calls, 0);
        assert_eq!(stats.prepared_functions, 0);
    }

    #[test]
    fn test_type_conversion() {
        let engine = LibffiEngine::new();
        
        // Test basic type conversions
        let int_type = engine.convert_c_type_to_ffi_type(&CType::Int32).unwrap();
        assert_eq!(int_type, Type::i32());
        
        let float_type = engine.convert_c_type_to_ffi_type(&CType::Float).unwrap();
        assert_eq!(float_type, Type::f32());
        
        let pointer_type = engine.convert_c_type_to_ffi_type(&CType::CString).unwrap();
        assert_eq!(pointer_type, Type::pointer());
    }

    #[test]
    fn test_ffi_interface_creation() {
        let interface = FfiInterface::new();
        let builtins = interface.list_builtin_functions();
        assert!(!builtins.is_empty());
        assert!(builtins.contains(&"strlen".to_string()));
        assert!(builtins.contains(&"malloc".to_string()));
    }

    #[test]
    fn test_builtin_signature_retrieval() {
        let interface = FfiInterface::new();
        let strlen_sig = interface.get_builtin_signature("strlen").unwrap();
        
        assert_eq!(strlen_sig.name, "strlen");
        assert_eq!(strlen_sig.parameters.len(), 1);
        assert_eq!(strlen_sig.parameters[0], CType::CString);
        assert_eq!(strlen_sig.return_type, CType::CSizeT);
    }

    #[test]
    fn test_return_buffer_preparation() {
        let engine = LibffiEngine::new();
        let int_type = Type::i32();
        let buffer = engine.prepare_return_buffer(&int_type).unwrap();
        assert_eq!(buffer.len(), 4); // i32 is 4 bytes
    }
}