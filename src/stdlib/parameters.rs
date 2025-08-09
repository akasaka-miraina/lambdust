//! SRFI-39 Parameter objects standard library functions.
//!
//! This module provides the standard library functions for working with parameters:
//! - `make-parameter`: Create a new parameter object
//! - Parameter objects are callable as procedures to get their current value

use crate::eval::{Value, Parameter};
use crate::diagnostics::Result;
use std::collections::HashMap;

/// Creates a new parameter with the given initial value and optional converter.
///
/// Scheme signature: `(make-parameter init [converter]) -> parameter`
///
/// # Arguments
/// * `args` - Vector containing initial value and optional converter function
///
/// # Returns
/// A new parameter object
///
/// # Examples
/// ```scheme
/// (define my-param (make-parameter 42))
/// (my-param) ; => 42
///
/// (define validated-param (make-parameter 0 
///   (lambda (x) 
///     (if (number? x) x (error "Must be a number")))))
/// ```
pub fn make_parameter(args: &[Value]) -> Result<Value> {
    match args.len() {
        1 => {
            let initial_value = args[0].clone();
            let param = Parameter::new(initial_value, None);
            Ok(Value::parameter(param))
        }
        2 => {
            let initial_value = args[0].clone();
            let converter = args[1].clone();
            let param = Parameter::new(initial_value, Some(converter));
            Ok(Value::parameter(param))
        }
        _ => Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("make-parameter expects 1 or 2 arguments, got {}", args.len()),
            None,
        ))),
    }
}

/// Checks if a value is a parameter object.
///
/// Scheme signature: `(parameter? obj) -> boolean`
pub fn is_parameter(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("parameter? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::boolean(args[0].is_parameter()))
}

/// Calls a parameter object to get its current value.
///
/// This is used internally when a parameter is applied as a procedure.
/// Parameters are callable with 0 or 1 arguments:
/// - 0 arguments: returns current value
/// - 1 argument: sets the global default (if no converter error)
pub fn call_parameter(parameter: &Parameter, args: &[Value]) -> Result<Value> {
    match args.len() {
        0 => {
            // Get current value
            Ok(parameter.get())
        }
        1 => {
            // Set global default value
            let new_value = args[0].clone();
            parameter.set_global(new_value)?;
            Ok(Value::Unspecified)
        }
        _ => Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("parameter expects 0 or 1 arguments, got {}", args.len()),
            None,
        ))),
    }
}

/// Utility function to convert parameter bindings from AST to runtime format.
///
/// This is used by the evaluator when processing `parameterize` forms.
pub fn process_parameter_bindings(
    bindings: &[crate::ast::ParameterBinding],
    mut evaluate_expr: impl FnMut(&crate::ast::Expr) -> Result<Value>,
) -> Result<HashMap<u64, Value>> {
    let mut runtime_bindings = HashMap::new();
    
    for binding in bindings {
        // Evaluate the parameter expression
        let param_value = evaluate_expr(&binding.parameter.inner)?;
        
        // Ensure it's actually a parameter
        if let Value::Parameter(param) = param_value {
            // Evaluate the value expression
            let value = evaluate_expr(&binding.value.inner)?;
            
            // Apply converter if present
            let processed_value = param.apply_converter(value)?;
            
            // Add to runtime bindings
            runtime_bindings.insert(param.id, processed_value);
        } else {
            return Err(Box::new(crate::diagnostics::Error::runtime_error(
                "Expected parameter object in parameterize binding".to_string(),
                None,
            )));
        }
    }
    
    Ok(runtime_bindings)
}

/// Installs parameter-related functions into the global environment.
pub fn install_parameter_functions(env: &crate::eval::ThreadSafeEnvironment) {
    use crate::eval::{PrimitiveProcedure, PrimitiveImpl};
    use std::sync::Arc;
    
    // make-parameter
    let make_param_proc = PrimitiveProcedure {
        name: "make-parameter".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(make_parameter),
        effects: vec![],
    };
    env.define("make-parameter".to_string(), Value::Primitive(Arc::new(make_param_proc)));
    
    // parameter?
    let is_param_proc = PrimitiveProcedure {
        name: "parameter?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(is_parameter),
        effects: vec![],
    };
    env.define("parameter?".to_string(), Value::Primitive(Arc::new(is_param_proc)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{Value, Parameter};
    
    #[test]
    fn test_make_parameter_basic() {
        let args = vec![Value::integer(42)];
        let result = make_parameter(&args).unwrap();
        
        assert!(result.is_parameter());
        if let Value::Parameter(param) = result {
            assert_eq!(param.get().as_integer(), Some(42));
        }
    }
    
    #[test]
    fn test_make_parameter_with_converter() {
        let args = vec![
            Value::integer(42),
            Value::integer(0), // placeholder converter
        ];
        let result = make_parameter(&args).unwrap();
        
        assert!(result.is_parameter());
        if let Value::Parameter(param) = result {
            assert!(param.has_converter());
        }
    }
    
    #[test]
    fn test_is_parameter() {
        let param = Parameter::new(Value::integer(42), None);
        let param_value = Value::parameter(param);
        
        let args = vec![param_value];
        let result = is_parameter(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::integer(42)];
        let result = is_parameter(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_call_parameter() {
        let param = Parameter::new(Value::integer(42), None);
        
        // Get current value (0 args)
        let result = call_parameter(&param, &[]).unwrap();
        assert_eq!(result.as_integer(), Some(42));
        
        // Set new value (1 arg)
        let result = call_parameter(&param, &[Value::integer(100)]).unwrap();
        assert_eq!(result, Value::Unspecified);
        
        // Verify new value
        let result = call_parameter(&param, &[]).unwrap();
        assert_eq!(result.as_integer(), Some(100));
    }
    
    #[test]
    fn test_call_parameter_arity_error() {
        let param = Parameter::new(Value::integer(42), None);
        
        // Too many arguments
        let result = call_parameter(&param, &[Value::integer(1), Value::integer(2)]);
        assert!(result.is_err());
    }
}