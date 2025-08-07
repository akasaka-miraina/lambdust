use super::{FfiRegistry, FfiSignature, FfiStats, FfiFunction, FfiError};
use crate::eval::Value;
use crate::diagnostics::{Error, Result};
use std::sync::Arc;

/// FFI bridge for calling Rust functions from Lambdust.
///
/// This is the main interface used by the evaluator to handle
/// `primitive` special form calls to FFI functions.
#[derive(Debug)]
pub struct FfiBridge {
    /// The function registry
    registry: Arc<FfiRegistry>,
}

impl FfiBridge {
    /// Creates a new FFI bridge with an empty registry.
    pub fn new() -> Self {
        Self {
            registry: Arc::new(FfiRegistry::new()),
        }
    }
    
    /// Creates a new FFI bridge with built-in functions.
    pub fn with_builtins() -> Self {
        Self {
            registry: Arc::new(FfiRegistry::with_builtins()),
        }
    }
    
    /// Creates an FFI bridge with a custom registry.
    pub fn with_registry(registry: Arc<FfiRegistry>) -> Self {
        Self { registry }
    }
    
    /// Gets a reference to the registry.
    pub fn registry(&self) -> &Arc<FfiRegistry> {
        &self.registry
    }
    
    /// Calls a Rust function with the given arguments.
    ///
    /// This is the main entry point used by the evaluator when
    /// processing `primitive` special forms.
    pub fn call_rust_function(&self, name: &str, args: &[Value]) -> Result<Value> {
        self.registry.call(name, args)
            .map_err(|ffi_err| Error::runtime_error(
                ffi_err.to_string(),
                None, // Span will be provided by the evaluator
            ))
    }
    
    /// Registers a new FFI function.
    pub fn register<F>(&self, function: F) -> Result<()>
    where
        F: FfiFunction + 'static,
    {
        self.registry.register(function)
            .map_err(|ffi_err| Error::runtime_error(
                format!("Failed to register FFI function: {ffi_err}"),
                None,
            ))
    }
    
    /// Gets information about a registered function.
    pub fn get_function_info(&self, name: &str) -> Option<FfiSignature> {
        self.registry.get_function_info(name)
    }
    
    /// Lists all registered function names.
    pub fn list_functions(&self) -> Vec<String> {
        self.registry.list_functions()
    }
    
    /// Gets FFI usage statistics.
    pub fn stats(&self) -> FfiStats {
        self.registry.stats()
    }
}

impl Default for FfiBridge {
    fn default() -> Self {
        Self::with_builtins()
    }
}

impl Clone for FfiBridge {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
        }
    }
}