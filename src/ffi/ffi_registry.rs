use super::{FfiError, FfiFunction, RegisteredFunction, FfiStats, FfiSignature};
use super::builtins::*;
use crate::eval::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// FFI function registry for managing registered Rust functions.
#[derive(Debug, Default)]
pub struct FfiRegistry {
    /// Map of function names to implementations
    functions: RwLock<HashMap<String, RegisteredFunction>>,
    /// Statistics
    stats: RwLock<FfiStats>,
}

impl FfiRegistry {
    /// Creates a new FFI registry.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Creates a new FFI registry with built-in functions.
    pub fn with_builtins() -> Self {
        let registry = Self::new();
        registry.register_builtins();
        registry
    }
    
    /// Registers a new FFI function.
    pub fn register<F>(&self, function: F) -> std::result::Result<(), FfiError>
    where
        F: FfiFunction + 'static,
    {
        let name = function.signature().name.clone();
        let registered_fn = RegisteredFunction {
            function: Arc::new(function),
            registered_at: std::time::SystemTime::now(),
        };
        
        {
            let mut functions = self.functions.write().unwrap();
            functions.insert(name, registered_fn);
        }
        
        {
            let mut stats = self.stats.write().unwrap();
            stats.registered_functions = self.functions.read().unwrap().len();
        }
        
        Ok(())
    }
    
    /// Calls a registered FFI function.
    pub fn call(&self, name: &str, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let function = {
            let functions = self.functions.read().unwrap();
            functions.get(name).cloned()
        };
        
        let function = function.ok_or_else(|| FfiError::FunctionNotFound(name.to_string()))?;
        
        // Update call statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_calls += 1;
        }
        
        // Validate arguments
        function.function.validate_args(args)?;
        
        // Call the function
        let result = function.function.call(args);
        
        // Update statistics based on result
        {
            let mut stats = self.stats.write().unwrap();
            match &result {
                Ok(_) => stats.successful_calls += 1,
                Err(_) => stats.failed_calls += 1,
            }
        }
        
        result
    }
    
    /// Gets information about a registered function.
    pub fn get_function_info(&self, name: &str) -> Option<FfiSignature> {
        let functions = self.functions.read().unwrap();
        functions.get(name).map(|f| f.function.signature().clone())
    }
    
    /// Lists all registered function names.
    pub fn list_functions(&self) -> Vec<String> {
        let functions = self.functions.read().unwrap();
        functions.keys().cloned().collect()
    }
    
    /// Gets FFI usage statistics.
    pub fn stats(&self) -> FfiStats {
        self.stats.read().unwrap().clone()
    }
    
    /// Clears all registered functions.
    pub fn clear(&self) {
        {
            let mut functions = self.functions.write().unwrap();
            functions.clear();
        }
        {
            let mut stats = self.stats.write().unwrap();
            stats.registered_functions = 0;
        }
    }
    
    /// Registers built-in FFI functions.
    fn register_builtins(&self) {
        // Register basic arithmetic functions
        self.register(AddFunction).ok();
        self.register(SubtractFunction).ok();
        self.register(MultiplyFunction).ok();
        self.register(DivideFunction).ok();
        
        // Register string functions
        self.register(StringLengthFunction).ok();
        self.register(StringConcatFunction).ok();
        self.register(StringUpperFunction).ok();
        self.register(StringLowerFunction).ok();
        
        // Register list functions
        self.register(ListLengthFunction).ok();
        self.register(ListMapFunction).ok();
        self.register(ListFilterFunction).ok();
        
        // Register type checking functions
        self.register(IsNumberFunction).ok();
        self.register(IsStringFunction).ok();
        self.register(IsListFunction).ok();
        self.register(IsBooleanFunction).ok();
        
        // Register I/O functions
        self.register(PrintFunction).ok();
        self.register(PrintlnFunction).ok();
    }
}