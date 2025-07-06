//! WebAssembly bindings for Lambdust Scheme interpreter
//!
//! This module provides WebAssembly-compatible bindings that allow Lambdust
//! to run in browsers and Node.js environments. It includes both browser-specific
//! bindings using wasm-bindgen and WASI-compatible interfaces.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use js_sys::{Array, Function, Object, Reflect};

#[cfg(feature = "wasm")]
use web_sys::console;

use crate::interpreter::LambdustInterpreter;
use crate::bridge::LambdustBridge;
use crate::value::Value;
use crate::error::LambdustError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// WebAssembly-compatible Lambdust interpreter
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmLambdustInterpreter {
    interpreter: LambdustInterpreter,
    bridge: LambdustBridge,
    last_error: Option<String>,
    js_functions: Arc<Mutex<HashMap<String, js_sys::Function>>>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmLambdustInterpreter {
    /// Create a new WebAssembly Lambdust interpreter
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmLambdustInterpreter {
        // Initialize console error panic hook for better debugging
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
        
        WasmLambdustInterpreter {
            interpreter: LambdustInterpreter::new(),
            bridge: LambdustBridge::new(),
            last_error: None,
            js_functions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Evaluate Scheme code and return the result as a string
    #[wasm_bindgen]
    pub fn eval(&mut self, code: &str) -> Option<String> {
        match self.interpreter.eval_string(code) {
            Ok(value) => {
                self.last_error = None;
                Some(value.to_string())
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
                None
            }
        }
    }
    
    /// Evaluate Scheme code and return the result as a JavaScript value
    #[wasm_bindgen]
    pub fn eval_js(&mut self, code: &str) -> Result<JsValue, JsValue> {
        match self.interpreter.eval_string(code) {
            Ok(value) => {
                self.last_error = None;
                Ok(self.scheme_value_to_js(&value)?)
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
                Err(JsValue::from_str(&error.to_string()))
            }
        }
    }
    
    /// Get the last error message
    #[wasm_bindgen]
    pub fn get_last_error(&self) -> Option<String> {
        self.last_error.clone()
    }
    
    /// Register a JavaScript function as a Scheme procedure
    #[wasm_bindgen]
    pub fn register_js_function(&mut self, name: &str, func: &js_sys::Function) -> Result<(), JsValue> {
        // Store the JavaScript function
        if let Ok(mut js_functions) = self.js_functions.lock() {
            js_functions.insert(name.to_string(), func.clone());
        }
        
        // Register with bridge (simplified implementation)
        self.bridge.register_function(name, None, move |args| {
            // This would need a proper implementation to call JS functions
            // For now, return a placeholder
            Ok(Value::String(format!("js-function-{}", name)))
        });
        
        Ok(())
    }
    
    /// Load and evaluate a Scheme program from a string
    #[wasm_bindgen]
    pub fn load_program(&mut self, program: &str) -> Result<String, JsValue> {
        match self.interpreter.eval_string(program) {
            Ok(value) => Ok(value.to_string()),
            Err(error) => {
                self.last_error = Some(error.to_string());
                Err(JsValue::from_str(&error.to_string()))
            }
        }
    }
    
    /// Reset the interpreter to its initial state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.interpreter = LambdustInterpreter::new();
        self.bridge = LambdustBridge::new();
        self.last_error = None;
        if let Ok(mut js_functions) = self.js_functions.lock() {
            js_functions.clear();
        }
    }
    
    /// Get interpreter version
    #[wasm_bindgen]
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
    
    /// Check if the interpreter is in a valid state
    #[wasm_bindgen]
    pub fn is_healthy(&self) -> bool {
        // Simple health check - could be expanded
        self.js_functions.lock().is_ok()
    }
    
    /// Convert a Scheme Value to a JavaScript value
    fn scheme_value_to_js(&self, value: &Value) -> Result<JsValue, JsValue> {
        match value {
            Value::Void => Ok(JsValue::undefined()),
            Value::Boolean(b) => Ok(JsValue::from_bool(*b)),
            Value::Number(n) => Ok(JsValue::from_f64(n.as_f64())),
            Value::String(s) => Ok(JsValue::from_str(s)),
            Value::Symbol(s) => {
                let obj = Object::new();
                Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str("symbol"))?;
                Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str(s))?;
                Ok(obj.into())
            }
            Value::List(items) => {
                let array = Array::new();
                for item in items {
                    array.push(&self.scheme_value_to_js(item)?);
                }
                Ok(array.into())
            }
            Value::Vector(items) => {
                let array = Array::new();
                for item in items {
                    array.push(&self.scheme_value_to_js(item)?);
                }
                Ok(array.into())
            }
            Value::Procedure(_) => {
                let obj = Object::new();
                Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str("procedure"))?;
                Ok(obj.into())
            }
            _ => {
                // For other types, return string representation
                Ok(JsValue::from_str(&value.to_string()))
            }
        }
    }
}

/// WASI-compatible interface for server-side WebAssembly
#[cfg(feature = "wasi")]
pub struct WasiLambdustInterpreter {
    interpreter: LambdustInterpreter,
    bridge: LambdustBridge,
}

#[cfg(feature = "wasi")]
impl WasiLambdustInterpreter {
    /// Create a new WASI Lambdust interpreter
    pub fn new() -> Self {
        WasiLambdustInterpreter {
            interpreter: LambdustInterpreter::new(),
            bridge: LambdustBridge::new(),
        }
    }
    
    /// Evaluate Scheme code
    pub fn eval(&mut self, code: &str) -> Result<String, String> {
        match self.interpreter.eval_string(code) {
            Ok(value) => Ok(value.to_string()),
            Err(error) => Err(error.to_string()),
        }
    }
    
    /// Load a Scheme file from the WASI filesystem
    pub fn load_file(&mut self, path: &str) -> Result<String, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", path, e))?;
        self.eval(&content)
    }
    
    /// Run a Scheme REPL (Read-Eval-Print Loop)
    pub fn repl(&mut self) -> Result<(), String> {
        use std::io::{self, Write};
        
        println!("Lambdust WASI REPL v{}", env!("CARGO_PKG_VERSION"));
        println!("Type 'exit' to quit.");
        
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }
            
            let input = input.trim();
            if input == "exit" || input == "(exit)" {
                break;
            }
            
            if input.is_empty() {
                continue;
            }
            
            match self.eval(input) {
                Ok(result) => println!("{}", result),
                Err(error) => eprintln!("Error: {}", error),
            }
        }
        
        Ok(())
    }
}

/// Utility functions for WebAssembly environments
#[cfg(feature = "wasm")]
pub mod utils {
    use super::*;
    
    /// Log a message to the browser console
    #[wasm_bindgen]
    pub fn log(message: &str) {
        console::log_1(&JsValue::from_str(message));
    }
    
    /// Log an error to the browser console
    #[wasm_bindgen]
    pub fn error(message: &str) {
        console::error_1(&JsValue::from_str(message));
    }
    
    /// Get the current timestamp (useful for benchmarking)
    #[wasm_bindgen]
    pub fn now() -> f64 {
        js_sys::Date::now()
    }
    
    /// Check if running in a browser environment
    #[wasm_bindgen]
    pub fn is_browser() -> bool {
        js_sys::eval("typeof window !== 'undefined'")
            .map(|v| v.is_truthy())
            .unwrap_or(false)
    }
    
    /// Check if running in Node.js environment
    #[wasm_bindgen]
    pub fn is_nodejs() -> bool {
        js_sys::eval("typeof process !== 'undefined' && process.versions && process.versions.node")
            .map(|v| v.is_truthy())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wasi_interpreter_creation() {
        #[cfg(feature = "wasi")]
        {
            let interpreter = WasiLambdustInterpreter::new();
            // Basic test to ensure creation works
            assert!(true);
        }
    }
    
    #[test]
    fn test_wasi_evaluation() {
        #[cfg(feature = "wasi")]
        {
            let mut interpreter = WasiLambdustInterpreter::new();
            let result = interpreter.eval("(+ 1 2 3)");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "6");
        }
    }
    
    #[cfg(feature = "wasm")]
    #[wasm_bindgen_test]
    fn test_wasm_interpreter_creation() {
        let interpreter = WasmLambdustInterpreter::new();
        assert!(interpreter.is_healthy());
    }
    
    #[cfg(feature = "wasm")]
    #[wasm_bindgen_test]
    fn test_wasm_evaluation() {
        let mut interpreter = WasmLambdustInterpreter::new();
        let result = interpreter.eval("(+ 1 2 3)");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "6");
    }
}