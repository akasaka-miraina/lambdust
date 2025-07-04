//! Host function registration and management
//!
//! This module provides functionality for registering host functions
//! that can be called from Scheme code safely.

use crate::error::{LambdustError, Result};
use crate::marshal::Marshallable;
use crate::value::{Procedure, Value};
use std::collections::HashMap;
use std::rc::Rc;

/// Signature for host functions callable from Scheme
pub type HostFunction = Box<dyn Fn(&[Value]) -> Result<Value>>;

/// Type information for function parameters and return values
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    /// Any value type
    Any,
    /// Boolean type
    Boolean,
    /// Numeric type
    Number,
    /// String type
    String,
    /// Symbol type
    Symbol,
    /// List type
    List,
    /// Pair type
    Pair,
    /// Nil type
    Nil,
    /// Procedure type
    Procedure,
}

impl ValueType {
    /// Check if a value matches this type
    pub fn matches(&self, value: &Value) -> bool {
        match (self, value) {
            (ValueType::Any, _) => true,
            (ValueType::Boolean, Value::Boolean(_)) => true,
            (ValueType::Number, Value::Number(_)) => true,
            (ValueType::String, Value::String(_)) => true,
            (ValueType::Symbol, Value::Symbol(_)) => true,
            (ValueType::List, value) if value.is_list() => true,
            (ValueType::Pair, Value::Pair(_)) => true,
            (ValueType::Nil, Value::Nil) => true,
            (ValueType::Procedure, Value::Procedure(_)) => true,
            _ => false,
        }
    }

    /// Get type name as string
    pub fn name(&self) -> &'static str {
        match self {
            ValueType::Any => "Any",
            ValueType::Boolean => "Boolean",
            ValueType::Number => "Number",
            ValueType::String => "String",
            ValueType::Symbol => "Symbol",
            ValueType::List => "List",
            ValueType::Pair => "Pair",
            ValueType::Nil => "Nil",
            ValueType::Procedure => "Procedure",
        }
    }
}

/// Function signature descriptor
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// Parameter types (None means variadic)
    pub parameters: Option<Vec<ValueType>>,
    /// Return type
    pub return_type: ValueType,
    /// Whether the function is variadic
    pub variadic: bool,
}

impl FunctionSignature {
    /// Create a new function signature
    pub fn new(parameters: Vec<ValueType>, return_type: ValueType) -> Self {
        Self {
            parameters: Some(parameters),
            return_type,
            variadic: false,
        }
    }

    /// Create a variadic function signature
    pub fn variadic(return_type: ValueType) -> Self {
        Self {
            parameters: None,
            return_type,
            variadic: true,
        }
    }

    /// Validate arguments against this signature
    pub fn validate_args(&self, args: &[Value]) -> Result<()> {
        if let Some(ref params) = self.parameters {
            if !self.variadic && args.len() != params.len() {
                return Err(LambdustError::arity_error(params.len(), args.len()));
            }

            if self.variadic && args.len() < params.len() {
                return Err(LambdustError::arity_error(params.len(), args.len()));
            }

            // Check required parameters
            let check_count = if self.variadic {
                params.len()
            } else {
                args.len()
            };

            for (i, (param_type, arg)) in params[..check_count].iter().zip(args.iter()).enumerate()
            {
                if !param_type.matches(arg) {
                    return Err(LambdustError::type_error(format!(
                        "Parameter {}: expected {}, got {:?}",
                        i + 1,
                        param_type.name(),
                        arg
                    )));
                }
            }
        }
        Ok(())
    }

    /// Validate return value against this signature
    pub fn validate_return(&self, value: &Value) -> Result<()> {
        if !self.return_type.matches(value) {
            return Err(LambdustError::type_error(format!(
                "Return value type mismatch: expected {}, got {:?}",
                self.return_type.name(),
                value
            )));
        }
        Ok(())
    }
}

/// Host function type alias to reduce complexity
pub type HostFunc = std::rc::Rc<dyn Fn(&[Value]) -> Result<Value>>;

/// Host function registry
pub struct HostFunctionRegistry {
    /// Registered functions
    functions: HashMap<String, (HostFunc, FunctionSignature)>,
}

impl HostFunctionRegistry {
    /// Create a new host function registry
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Register a host function with signature validation
    pub fn register_function<F>(&mut self, name: String, func: F, signature: FunctionSignature)
    where
        F: Fn(&[Value]) -> Result<Value> + 'static,
    {
        let rc_func = std::rc::Rc::new(func);
        self.functions.insert(name, (rc_func, signature));
    }

    /// Register a simple host function without signature validation
    pub fn register_simple_function<F>(&mut self, name: String, func: F)
    where
        F: Fn(&[Value]) -> Result<Value> + 'static,
    {
        let signature = FunctionSignature::variadic(ValueType::Any);
        self.register_function(name, func, signature);
    }

    /// Register a typed host function with automatic marshalling
    pub fn register_typed_function<F, Args, Ret>(&mut self, name: String, func: F)
    where
        F: Fn(Args) -> Ret + 'static,
        Args: FromSchemeArgs,
        Ret: Marshallable,
    {
        let wrapper = move |args: &[Value]| -> Result<Value> {
            let typed_args = Args::from_scheme_args(args)?;
            let result = func(typed_args);
            result.to_scheme()
        };

        // Create signature based on types
        let signature = FunctionSignature::new(Args::parameter_types(), Ret::value_type());

        self.register_function(name, wrapper, signature);
    }

    /// Get a registered function as a Scheme procedure
    pub fn get_procedure(&self, name: &str) -> Option<Value> {
        if let Some((func, signature)) = self.functions.get(name) {
            let name_clone = name.to_string();
            let signature_clone = signature.clone();
            let func_clone = func.clone();

            // Create a wrapper that validates arguments and return value
            let wrapper = move |args: &[Value]| -> Result<Value> {
                signature_clone.validate_args(args)?;
                let result = func_clone(args)?;
                signature_clone.validate_return(&result)?;
                Ok(result)
            };

            Some(Value::Procedure(Procedure::HostFunction {
                name: name_clone,
                func: Rc::new(wrapper),
                arity: signature.parameters.as_ref().map(|p| p.len()),
            }))
        } else {
            None
        }
    }

    /// List all registered function names
    pub fn list_functions(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }

    /// Get function signature
    pub fn get_signature(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name).map(|(_, sig)| sig)
    }

    /// Register built-in utility functions
    pub fn register_builtins(&mut self) {
        // Debug print function
        self.register_function(
            "host-print".to_string(),
            |args: &[Value]| -> Result<Value> {
                for arg in args {
                    print!("{arg} ");
                }
                println!();
                Ok(Value::Nil)
            },
            FunctionSignature::variadic(ValueType::Nil),
        );

        // String concatenation
        self.register_function(
            "host-string-append".to_string(),
            |args: &[Value]| -> Result<Value> {
                let mut result = String::new();
                for arg in args {
                    match arg {
                        Value::String(s) => result.push_str(s),
                        Value::Symbol(s) => result.push_str(s),
                        _ => {
                            return Err(LambdustError::type_error(
                                "host-string-append: all arguments must be strings".to_string(),
                            ));
                        }
                    }
                }
                Ok(Value::String(result))
            },
            FunctionSignature::new(vec![ValueType::String], ValueType::String),
        );

        // Length function
        self.register_function(
            "host-length".to_string(),
            |args: &[Value]| -> Result<Value> {
                if args.len() != 1 {
                    return Err(LambdustError::arity_error(1, args.len()));
                }

                match &args[0] {
                    Value::String(s) => Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
                        s.len() as i64,
                    ))),
                    value if value.is_list() => {
                        if let Some(vec) = value.to_vector() {
                            Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
                                vec.len() as i64,
                            )))
                        } else {
                            Err(LambdustError::type_error("Invalid list".to_string()))
                        }
                    }
                    _ => Err(LambdustError::type_error(
                        "host-length: argument must be a string or list".to_string(),
                    )),
                }
            },
            FunctionSignature::new(vec![ValueType::Any], ValueType::Number),
        );
    }
}

impl Default for HostFunctionRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register_builtins();
        registry
    }
}

/// Trait for converting Scheme arguments to typed parameters
pub trait FromSchemeArgs {
    /// Convert from Scheme arguments
    fn from_scheme_args(args: &[Value]) -> Result<Self>
    where
        Self: Sized;

    /// Get parameter types for signature
    fn parameter_types() -> Vec<ValueType>;
}

// Implement FromSchemeArgs for common tuple types
impl FromSchemeArgs for () {
    fn from_scheme_args(args: &[Value]) -> Result<Self> {
        if args.is_empty() {
            Ok(())
        } else {
            Err(LambdustError::arity_error(0, args.len()))
        }
    }

    fn parameter_types() -> Vec<ValueType> {
        vec![]
    }
}

impl<T: Marshallable> FromSchemeArgs for (T,) {
    fn from_scheme_args(args: &[Value]) -> Result<Self> {
        if args.len() != 1 {
            return Err(LambdustError::arity_error(1, args.len()));
        }
        Ok((T::from_scheme(&args[0])?,))
    }

    fn parameter_types() -> Vec<ValueType> {
        vec![T::value_type()]
    }
}

impl<T1: Marshallable, T2: Marshallable> FromSchemeArgs for (T1, T2) {
    fn from_scheme_args(args: &[Value]) -> Result<Self> {
        if args.len() != 2 {
            return Err(LambdustError::arity_error(2, args.len()));
        }
        Ok((T1::from_scheme(&args[0])?, T2::from_scheme(&args[1])?))
    }

    fn parameter_types() -> Vec<ValueType> {
        vec![T1::value_type(), T2::value_type()]
    }
}

impl<T1: Marshallable, T2: Marshallable, T3: Marshallable> FromSchemeArgs for (T1, T2, T3) {
    fn from_scheme_args(args: &[Value]) -> Result<Self> {
        if args.len() != 3 {
            return Err(LambdustError::arity_error(3, args.len()));
        }
        Ok((
            T1::from_scheme(&args[0])?,
            T2::from_scheme(&args[1])?,
            T3::from_scheme(&args[2])?,
        ))
    }

    fn parameter_types() -> Vec<ValueType> {
        vec![T1::value_type(), T2::value_type(), T3::value_type()]
    }
}
