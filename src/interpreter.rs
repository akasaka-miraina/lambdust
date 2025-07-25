//! Main interpreter interface for host applications
//!
//! This module provides the primary interface for integrating Lambdust
//! into host applications, allowing bidirectional function calls.

use crate::error::{LambdustError, Result};
use crate::evaluator::Evaluator;
use crate::host::{FunctionSignature, HostFunctionRegistry, ValueType};
use crate::lexer::tokenize;
use crate::marshal::Marshallable;
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
    /// Defined Scheme functions available to host
    scheme_functions: HashMap<String, Value>,
}

impl LambdustInterpreter {
    /// Create a new interpreter instance
    pub fn new() -> Self {
        let mut interpreter = Self {
            evaluator: Evaluator::new(),
            host_registry: HostFunctionRegistry::default(),
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
        let result = self.evaluator.eval_string(code)?;

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

    /// Extract function name from define form using AST
    fn extract_function_name(&self, code: &str) -> Option<String> {
        let ast = self.parse_define_form(code)?;
        self.extract_name_from_define(&ast)
    }

    /// Parse code and check if it's a define form
    fn parse_define_form(&self, code: &str) -> Option<Vec<crate::ast::Expr>> {
        let tokens = tokenize(code).ok()?;
        if let Ok(crate::ast::Expr::List(exprs)) = parse(tokens) {
            if self.is_define_form(&exprs) {
                return Some(exprs);
            }
        }
        None
    }

    /// Check if expressions form a define statement
    fn is_define_form(&self, exprs: &[crate::ast::Expr]) -> bool {
        if exprs.len() < 2 {
            return false;
        }

        matches!(&exprs[0], crate::ast::Expr::Variable(op) if op == "define")
    }

    /// Extract name from define form AST
    fn extract_name_from_define(&self, exprs: &[crate::ast::Expr]) -> Option<String> {
        match &exprs[1] {
            // (define var value)
            crate::ast::Expr::Variable(name) => Some(name.clone()),
            // (define (name params...) body...)
            crate::ast::Expr::List(def_exprs) if !def_exprs.is_empty() => {
                if let crate::ast::Expr::Variable(name) = &def_exprs[0] {
                    Some(name.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Register a host function that can be called from Scheme
    pub fn register_host_function<F>(&mut self, name: String, func: F, signature: FunctionSignature)
    where
        F: Fn(&[Value]) -> Result<Value> + 'static,
    {
        self.host_registry
            .register_function(name.clone(), func, signature);

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
            Some(value) => {
                if let Value::Procedure(_) = value {
                    self.apply_procedure(value, args.to_vec())
                } else {
                    Err(LambdustError::type_error(format!(
                        "{name} is not a procedure"
                    )))
                }
            }
            None => Err(LambdustError::undefined_variable(name.to_string())),
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
        // Use the evaluator's public interface for calling procedures
        match &procedure {
            Value::Procedure(_) => self.evaluator.call_procedure(procedure, args),
            _ => Err(LambdustError::type_error("Not a procedure".to_string())),
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
        self.scheme_functions.contains_key(name)
            || self
                .evaluator
                .global_env
                .get(name)
                .map(|v| v.is_procedure())
                .unwrap_or(false)
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
        self.eval_string(code)
            .map_err(|e| LambdustError::runtime_error(format!("Error in {context}: {e}")))
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
