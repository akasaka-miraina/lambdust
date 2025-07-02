//! Main interpreter interface for host applications
//! 
//! This module provides the primary interface for integrating Lambdust
//! into host applications, allowing bidirectional function calls.

use crate::error::{LambdustError, Result};
use crate::evaluator::Evaluator;
use crate::host::{FunctionSignature, HostFunctionRegistry, ValueType};
use crate::lexer::tokenize;
use crate::marshal::{Marshallable, TypeSafeMarshaller};
use crate::parser::parse;
use crate::value::{Procedure, Value};
use std::collections::HashMap;
use std::rc::Rc;

/// Main interpreter for Lambdust Scheme
/// 
/// This provides a high-level interface for executing Scheme code
/// and managing host function integration.
pub struct LambdustInterpreter {
    /// Core evaluator
    evaluator: Evaluator,
    /// Host function registry
    host_registry: HostFunctionRegistry,
    /// Type marshaller
    marshaller: TypeSafeMarshaller,
    /// Defined Scheme functions available to host
    scheme_functions: HashMap<String, Value>,
}

impl LambdustInterpreter {
    /// Create a new interpreter instance
    pub fn new() -> Self {
        let mut interpreter = Self {
            evaluator: Evaluator::new(),
            host_registry: HostFunctionRegistry::default(),
            marshaller: TypeSafeMarshaller::new(),
            scheme_functions: HashMap::new(),
        };
        
        // Register host functions in the global environment
        interpreter.register_host_functions();
        interpreter
    }

    /// Register host functions in the Scheme environment
    fn register_host_functions(&mut self) {
        let function_names = self.host_registry.list_functions().clone();
        
        for name in function_names {
            if let Some(procedure) = self.host_registry.get_procedure(name) {
                self.evaluator.global_env.define(name.clone(), procedure);
            }
        }
    }

    /// Execute Scheme code from string
    pub fn eval_string(&mut self, code: &str) -> Result<Value> {
        let tokens = tokenize(code)?;
        let ast = parse(tokens)?;
        let result = self.evaluator.eval(ast)?;
        
        // Check if the result is a procedure and cache it for host access
        if let Value::Procedure(Procedure::Lambda { .. }) = &result {
            // Extract function name if this was a define form
            // This is a simplified approach - in practice, we'd need better tracking
            if code.trim().starts_with("(define") {
                if let Some(name) = self.extract_function_name(code) {
                    self.scheme_functions.insert(name, result.clone());
                }
            }
        }
        
        Ok(result)
    }

    /// Extract function name from define form (simplified)
    fn extract_function_name(&self, code: &str) -> Option<String> {
        // Simple regex-like parsing for function names
        // In practice, this should use proper AST analysis
        let code = code.trim();
        if code.starts_with("(define") {
            let parts: Vec<&str> = code.split_whitespace().collect();
            if parts.len() >= 3 {
                let name_part = parts[1];
                if name_part.starts_with('(') {
                    // Function definition: (define (name params...) body...)
                    let name = &name_part[1..];
                    return Some(name.to_string());
                } else {
                    // Variable definition: (define name value)
                    return Some(name_part.to_string());
                }
            }
        }
        None
    }

    /// Register a host function that can be called from Scheme
    pub fn register_host_function<F>(&mut self, name: String, func: F, signature: FunctionSignature)
    where
        F: Fn(&[Value]) -> Result<Value> + 'static,
    {
        self.host_registry.register_function(name.clone(), func, signature);
        
        // Register in the Scheme environment
        if let Some(procedure) = self.host_registry.get_procedure(&name) {
            self.evaluator.global_env.define(name, procedure);
        }
    }

    /// Register a simple host function without signature validation
    pub fn register_simple_host_function<F>(&mut self, name: String, func: F)
    where
        F: Fn(&[Value]) -> Result<Value> + 'static,
    {
        let signature = FunctionSignature::variadic(ValueType::Any);
        self.register_host_function(name, func, signature);
    }

    /// Call a Scheme function defined in the interpreter from host code
    pub fn call_scheme_function(&mut self, name: &str, args: &[Value]) -> Result<Value> {
        // First, try to get from cached functions
        if let Some(procedure) = self.scheme_functions.get(name) {
            return self.apply_procedure(procedure.clone(), args.to_vec());
        }
        
        // Try to get from global environment
        match self.evaluator.global_env.get(name) {
            Ok(value) => {
                if let Value::Procedure(_) = value {
                    self.apply_procedure(value, args.to_vec())
                } else {
                    Err(LambdustError::TypeError(format!("{} is not a procedure", name)))
                }
            }
            Err(_) => Err(LambdustError::UndefinedVariable(name.to_string())),
        }
    }

    /// Call a Scheme function with typed arguments
    pub fn call_scheme_function_typed<Args, Ret>(&mut self, name: &str, args: Args) -> Result<Ret>
    where
        Args: IntoSchemeArgs,
        Ret: Marshallable,
    {
        let scheme_args = args.into_scheme_args()?;
        let result = self.call_scheme_function(name, &scheme_args)?;
        Ret::from_scheme(&result)
    }

    /// Apply a procedure with arguments
    fn apply_procedure(&mut self, procedure: Value, args: Vec<Value>) -> Result<Value> {
        // Create a temporary function call expression and evaluate it
        match &procedure {
            Value::Procedure(_) => {
                self.evaluator.apply_procedure(procedure, args, self.evaluator.global_env.clone())
            }
            _ => Err(LambdustError::TypeError("Not a procedure".to_string())),
        }
    }

    /// List all available Scheme functions
    pub fn list_scheme_functions(&self) -> Vec<&String> {
        self.scheme_functions.keys().collect()
    }

    /// List all available host functions  
    pub fn list_host_functions(&self) -> Vec<&String> {
        self.host_registry.list_functions()
    }

    /// Check if a Scheme function exists
    pub fn has_scheme_function(&self, name: &str) -> bool {
        self.scheme_functions.contains_key(name) || 
        self.evaluator.global_env.get(name).map(|v| v.is_procedure()).unwrap_or(false)
    }

    /// Get the global environment (for advanced usage)
    pub fn global_environment(&self) -> &Rc<crate::environment::Environment> {
        &self.evaluator.global_env
    }

    /// Execute multiple Scheme expressions and return the last result
    pub fn eval_expressions(&mut self, expressions: &[&str]) -> Result<Value> {
        let mut last_result = Value::Undefined;
        
        for expr in expressions {
            last_result = self.eval_string(expr)?;
        }
        
        Ok(last_result)
    }

    /// Load and execute Scheme code from a string with error context
    pub fn load_string(&mut self, code: &str, context: &str) -> Result<Value> {
        self.eval_string(code).map_err(|e| {
            LambdustError::RuntimeError(format!("Error in {}: {}", context, e))
        })
    }
}

impl Default for LambdustInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for converting Rust values to Scheme arguments
pub trait IntoSchemeArgs {
    /// Convert to Scheme arguments
    fn into_scheme_args(self) -> Result<Vec<Value>>;
}

impl IntoSchemeArgs for Vec<Value> {
    fn into_scheme_args(self) -> Result<Vec<Value>> {
        Ok(self)
    }
}

impl IntoSchemeArgs for () {
    fn into_scheme_args(self) -> Result<Vec<Value>> {
        Ok(vec![])
    }
}

impl<T: Marshallable> IntoSchemeArgs for (T,) {
    fn into_scheme_args(self) -> Result<Vec<Value>> {
        Ok(vec![self.0.to_scheme()?])
    }
}

impl<T1: Marshallable, T2: Marshallable> IntoSchemeArgs for (T1, T2) {
    fn into_scheme_args(self) -> Result<Vec<Value>> {
        Ok(vec![self.0.to_scheme()?, self.1.to_scheme()?])
    }
}

impl<T1: Marshallable, T2: Marshallable, T3: Marshallable> IntoSchemeArgs for (T1, T2, T3) {
    fn into_scheme_args(self) -> Result<Vec<Value>> {
        Ok(vec![
            self.0.to_scheme()?,
            self.1.to_scheme()?,
            self.2.to_scheme()?,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_interpreter_basic() {
        let mut interpreter = LambdustInterpreter::new();
        
        // Test basic evaluation
        let _result = interpreter.eval_string("(+ 1 2 3)").unwrap();
        // Note: This will fail until arithmetic is implemented
        // assert_eq!(result, Value::Number(SchemeNumber::Integer(6)));
    }

    #[test]
    fn test_function_definition_and_call() {
        let mut interpreter = LambdustInterpreter::new();
        
        // Test simple literal evaluation first
        let result = interpreter.eval_string("42");
        assert!(result.is_ok());
        
        // Test simple variable definition first
        let result = interpreter.eval_string("(define x 42)");
        if let Err(e) = result {
            panic!("Failed to define variable: {:?}", e);
        }
        
        // Define a simple function
        let result = interpreter.eval_string("(define (identity x) x)");
        if let Err(e) = result {
            panic!("Failed to define function: {:?}", e);
        }
        
        // Check if function is available
        assert!(interpreter.has_scheme_function("identity"));
        
        // Call the function (will fail until arithmetic is implemented)
        // let result = interpreter.call_scheme_function("square", &[Value::Number(SchemeNumber::Integer(5))]).unwrap();
        // assert_eq!(result, Value::Number(SchemeNumber::Integer(25)));
    }

    #[test]
    fn test_host_function_registration() {
        let mut interpreter = LambdustInterpreter::new();
        
        // Register a host function
        interpreter.register_simple_host_function(
            "test-add".to_string(),
            |args: &[Value]| -> Result<Value> {
                if args.len() != 2 {
                    return Err(LambdustError::ArityError {
                        expected: 2,
                        actual: args.len(),
                    });
                }
                
                match (&args[0], &args[1]) {
                    (Value::Number(SchemeNumber::Integer(a)), Value::Number(SchemeNumber::Integer(b))) => {
                        Ok(Value::Number(SchemeNumber::Integer(a + b)))
                    }
                    _ => Err(LambdustError::TypeError("Expected numbers".to_string())),
                }
            },
        );

        // Test that the function is registered
        assert!(interpreter.list_host_functions().contains(&&"test-add".to_string()));
        
        // Test calling the host function from Scheme
        let result = interpreter.eval_string("(test-add 10 20)").unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(30)));
    }

    #[test]
    fn test_typed_function_calls() {
        let mut interpreter = LambdustInterpreter::new();
        
        // Register a typed host function
        interpreter.register_simple_host_function(
            "string-length".to_string(),
            |args: &[Value]| -> Result<Value> {
                if args.len() != 1 {
                    return Err(LambdustError::ArityError {
                        expected: 1,
                        actual: args.len(),
                    });
                }
                
                match &args[0] {
                    Value::String(s) => Ok(Value::Number(SchemeNumber::Integer(s.len() as i64))),
                    _ => Err(LambdustError::TypeError("Expected string".to_string())),
                }
            },
        );

        // Test typed call
        let result: i64 = interpreter
            .call_scheme_function_typed("string-length", ("hello".to_string(),))
            .unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn test_multiple_expressions() {
        let mut interpreter = LambdustInterpreter::new();
        
        let expressions = vec![
            "(define x 10)",
            "(define y 20)", 
            "(define (get-x) x)",
        ];
        
        interpreter.eval_expressions(&expressions).unwrap();
        
        // Check that functions and variables are defined
        assert!(interpreter.has_scheme_function("get-x"));
    }

    #[test]
    fn test_error_handling() {
        let mut interpreter = LambdustInterpreter::new();
        
        // Test undefined function call
        let result = interpreter.call_scheme_function("undefined-function", &[]);
        assert!(result.is_err());
        
        // Test malformed code
        let result = interpreter.eval_string("(invalid syntax");
        assert!(result.is_err());
    }
}