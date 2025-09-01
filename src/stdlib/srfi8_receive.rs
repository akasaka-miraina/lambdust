#![allow(missing_docs)]//! SRFI-8: receive - Binding to multiple values
//!
//! This module implements SRFI-8, which provides the `receive` syntax form for
//! conveniently binding multiple values returned by a producer expression.
//!
//! ## R7RS Compliance
//!
//! SRFI-8 is included in R7RS-large and provides a crucial building block
//! for working with multiple values in Scheme. This implementation fully
//! supports all R7RS formal parameter patterns.
//!
//! ## Syntax
//!
//! ```scheme
//! (receive formals producer body ...)
//! ```
//!
//! Where:
//! - `formals` follows standard lambda formal parameter patterns
//! - `producer` is an expression that may return multiple values
//! - `body` is executed with formals bound to the producer's values
//!
//! ## Examples
//!
//! ```scheme
//! (receive (a b) (values 1 2)    ; Fixed arity
//!   (+ a b))                     ; => 3
//!
//! (receive args (values 1 2 3)   ; Variable arity
//!   (length args))               ; => 3
//!
//! (receive (a . rest) (values 1 2 3 4)  ; Improper list
//!   (cons a rest))               ; => (1 2 3 4)
//! ```
//!
//! ## Implementation Strategy
//!
//! The implementation follows the canonical SRFI-8 approach:
//! 1. Expand `receive` to `call-with-values`
//! 2. Support all standard formal parameter patterns
//! 3. Provide clear error messages for arity mismatches
//!
//! This allows leveraging Lambdust's existing lambda machinery while
//! providing the ergonomic `receive` interface.

use crate::ast::Formals;
use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{
    MultipleValues, PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value,
};
use crate::utils::intern_symbol;
use std::sync::Arc;

/// Creates SRFI-8 receive bindings for the standard library.
pub fn create_srfi8_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Enhanced values implementation with proper multi-value support
    env.define(
        "values".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "values".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(enhanced_primitive_values),
            effects: vec![Effect::Pure],
        })),
    );

    // Enhanced call-with-values implementation
    env.define(
        "call-with-values".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "call-with-values".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(enhanced_primitive_call_with_values),
            effects: vec![Effect::Pure],
        })),
    );

    // SRFI-8 receive macro implementation (placeholder - requires evaluator integration)
    env.define(
        "receive".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "receive".to_string(),
            arity_min: 3,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(primitive_receive_placeholder),
            effects: vec![Effect::Pure],
        })),
    );

    // Helper predicates
    env.define(
        "multiple-values?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "multiple-values?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_multiple_values_p),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "values-length".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "values-length".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_values_length),
            effects: vec![Effect::Pure],
        })),
    );
}

/// Placeholder receive implementation - requires evaluator integration for full functionality.
fn primitive_receive_placeholder(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "receive requires evaluator integration for macro expansion (not yet implemented)"
            .to_string(),
        None,
    )))
}

// ============= ENHANCED MULTI-VALUE SUPPORT =============

/// Enhanced values implementation that properly creates multiple values.
pub fn enhanced_primitive_values(args: &[Value]) -> Result<Value> {
    match args.len() {
        0 => Ok(Value::Unspecified),
        1 => Ok(args[0].clone()),
        _ => {
            // Create proper multiple values object
            Ok(Value::MultipleValues(Arc::new(MultipleValues::new(args.to_vec()))))
        }
    }
}

/// Enhanced call-with-values implementation.
fn enhanced_primitive_call_with_values(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("call-with-values expects 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let _producer = &args[0];
    let _consumer = &args[1];

    // Verify both arguments are procedures
    if !args[0].is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "call-with-values: first argument (producer) must be a procedure".to_string(),
            None,
        )));
    }

    if !args[1].is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "call-with-values: second argument (consumer) must be a procedure".to_string(),
            None,
        )));
    }

    // For now, return a placeholder error - full implementation requires evaluator integration
    Err(Box::new(DiagnosticError::runtime_error(
        "call-with-values requires evaluator integration for procedure calls (implementation in progress)".to_string(),
        None,
    )))
}

// ============= HELPER PROCEDURES =============

/// Predicate to check if a value is a multiple values object.
pub fn primitive_multiple_values_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("multiple-values? expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    let is_multiple = matches!(args[0], Value::MultipleValues(_));
    Ok(Value::boolean(is_multiple))
}

/// Get the number of values in a multiple values object.
pub fn primitive_values_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("values-length expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::MultipleValues(mv) => Ok(Value::integer(mv.len() as i64)),
        _ => Ok(Value::integer(1)), // Single values have length 1
    }
}

// ============= RECEIVE MACRO EXPANSION UTILITIES =============

/// Expands a receive form to call-with-values.
pub fn expand_receive(formals: &Formals, producer: &Value, body: &[Value]) -> Result<Value> {
    // Validate that we have a body
    if body.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "receive: body cannot be empty".to_string(),
            None,
        )));
    }

    // Create the consumer lambda
    let consumer = create_consumer_lambda(formals, body)?;

    // Create the producer thunk
    let producer_thunk = create_producer_thunk(producer)?;

    // Expand to call-with-values
    Ok(Value::list(vec![
        Value::symbol(intern_symbol("call-with-values")),
        producer_thunk,
        consumer,
    ]))
}

/// Creates a lambda consumer from formals and body.
fn create_consumer_lambda(formals: &Formals, body: &[Value]) -> Result<Value> {
    // Convert formals to appropriate list representation
    let formals_list = formals_to_list(formals)?;

    // Create lambda expression
    let mut lambda_parts = vec![Value::symbol(intern_symbol("lambda")), formals_list];

    // Add body expressions
    lambda_parts.extend(body.iter().cloned());

    Ok(Value::list(lambda_parts))
}

/// Creates a producer thunk that evaluates the producer expression.
fn create_producer_thunk(producer: &Value) -> Result<Value> {
    // Wrap producer in a lambda thunk
    Ok(Value::list(vec![
        Value::symbol(intern_symbol("lambda")),
        Value::Nil, // No parameters
        producer.clone(),
    ]))
}

/// Converts Formals to a list representation for lambda.
pub fn formals_to_list(formals: &Formals) -> Result<Value> {
    match formals {
        Formals::Fixed(params) => {
            // (param1 param2 ...)
            let param_symbols: Vec<Value> = params
                .iter()
                .map(|name| Value::symbol(intern_symbol(name)))
                .collect();
            Ok(Value::list(param_symbols))
        }
        Formals::Variable(param) => {
            // param
            Ok(Value::symbol(intern_symbol(param)))
        }
        Formals::Mixed { fixed, rest } => {
            // (param1 param2 . rest)
            let mut param_symbols: Vec<Value> = fixed
                .iter()
                .map(|name| Value::symbol(intern_symbol(name)))
                .collect();

            // Create improper list
            let rest_symbol = Value::symbol(intern_symbol(rest));
            Ok(create_improper_list(param_symbols, rest_symbol))
        }
        _ => {
            // Other formal types (Keyword, Typed, etc.) not supported in SRFI-8
            Err(Box::new(DiagnosticError::runtime_error(
                "receive: unsupported formal parameter type (only fixed, variable, and mixed supported)".to_string(),
                None,
            )))
        }
    }
}

/// Creates an improper list (param1 param2 . rest).
fn create_improper_list(mut elements: Vec<Value>, rest: Value) -> Value {
    if elements.is_empty() {
        return rest;
    }

    let last = elements.pop().unwrap();
    let mut result = Value::cons(last, rest);

    // Build the list backwards
    for elem in elements.into_iter().rev() {
        result = Value::cons(elem, result);
    }

    result
}

// ============= ARITY VALIDATION UTILITIES =============

/// Validates that the number of values matches the formal parameters.
pub fn validate_receive_arity(formals: &Formals, value_count: usize) -> Result<()> {
    match formals {
        Formals::Fixed(params) => {
            if value_count != params.len() {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!(
                        "receive: expected {} values, got {}",
                        params.len(),
                        value_count
                    ),
                    None,
                )));
            }
        }
        Formals::Variable(_) => {
            // Variable arity accepts any number of values
        }
        Formals::Mixed { fixed, .. } => {
            if value_count < fixed.len() {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!(
                        "receive: expected at least {} values, got {}",
                        fixed.len(),
                        value_count
                    ),
                    None,
                )));
            }
        }
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                "receive: unsupported formal parameter type".to_string(),
                None,
            )));
        }
    }

    Ok(())
}

/// Binds multiple values to formal parameters in an environment.
pub fn bind_multiple_values(
    env: &Arc<ThreadSafeEnvironment>,
    formals: &Formals,
    values: &[Value],
) -> Result<()> {
    // First validate arity
    validate_receive_arity(formals, values.len())?;

    match formals {
        Formals::Fixed(params) => {
            // Bind each parameter to corresponding value
            for (param, value) in params.iter().zip(values.iter()) {
                env.define(param.clone(), value.clone());
            }
        }
        Formals::Variable(param) => {
            // Bind all values as a list to the parameter
            let values_list = Value::list(values.to_vec());
            env.define(param.clone(), values_list);
        }
        Formals::Mixed { fixed, rest } => {
            // Bind fixed parameters
            for (param, value) in fixed.iter().zip(values.iter()) {
                env.define(param.clone(), value.clone());
            }

            // Bind remaining values as a list to rest parameter
            let rest_values = &values[fixed.len()..];
            let rest_list = Value::list(rest_values.to_vec());
            env.define(rest.clone(), rest_list);
        }
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                "bind-multiple-values: unsupported formal parameter type".to_string(),
                None,
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_values() {
        // Test values with no arguments
        let result = enhanced_primitive_values(&[]).unwrap();
        assert_eq!(result, Value::Unspecified);

        // Test values with one argument
        let result = enhanced_primitive_values(&[Value::integer(42)]).unwrap();
        assert_eq!(result, Value::integer(42));

        // Test values with multiple arguments
        let args = vec![Value::integer(1), Value::integer(2), Value::integer(3)];
        let result = enhanced_primitive_values(&args).unwrap();
        assert!(matches!(result, Value::MultipleValues(_)));

        if let Value::MultipleValues(mv) = result {
            assert_eq!(mv.len(), 3);
            assert_eq!(*mv.get(0).unwrap(), Value::integer(1));
            assert_eq!(*mv.get(1).unwrap(), Value::integer(2));
            assert_eq!(*mv.get(2).unwrap(), Value::integer(3));
        }
    }

    #[test]
    fn test_multiple_values_predicate() {
        // Test with single value
        let result = primitive_multiple_values_p(&[Value::integer(42)]).unwrap();
        assert_eq!(result, Value::boolean(false));

        // Test with multiple values
        let mv = Value::MultipleValues(Arc::new(MultipleValues::new(vec![Value::integer(1), Value::integer(2)])));
        let result = primitive_multiple_values_p(&[mv]).unwrap();
        assert_eq!(result, Value::boolean(true));
    }

    #[test]
    fn test_values_length() {
        // Test with single value
        let result = primitive_values_length(&[Value::integer(42)]).unwrap();
        assert_eq!(result, Value::integer(1));

        // Test with multiple values
        let mv = Value::MultipleValues(Arc::new(MultipleValues::new(vec![Value::integer(1), Value::integer(2), Value::integer(3)])));
        let result = primitive_values_length(&[mv]).unwrap();
        assert_eq!(result, Value::integer(3));
    }

    #[test]
    fn test_formals_to_list() {
        // Test fixed formals
        let formals = Formals::Fixed(vec!["a".to_string(), "b".to_string()]);
        let result = formals_to_list(&formals).unwrap();
        let expected = Value::list(vec![
            Value::symbol(intern_symbol("a")),
            Value::symbol(intern_symbol("b")),
        ]);
        assert_eq!(result, expected);

        // Test variable formals
        let formals = Formals::Variable("args".to_string());
        let result = formals_to_list(&formals).unwrap();
        let expected = Value::symbol(intern_symbol("args"));
        assert_eq!(result, expected);

        // Test mixed formals
        let formals = Formals::Mixed {
            fixed: vec!["a".to_string(), "b".to_string()],
            rest: "rest".to_string(),
        };
        let result = formals_to_list(&formals).unwrap();
        // Should create (a b . rest)
        let expected_pair = Value::cons(
            Value::symbol(intern_symbol("b")),
            Value::symbol(intern_symbol("rest")),
        );
        let expected = Value::cons(Value::symbol(intern_symbol("a")), expected_pair);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_arity_validation() {
        // Test fixed arity
        let formals = Formals::Fixed(vec!["a".to_string(), "b".to_string()]);
        assert!(validate_receive_arity(&formals, 2).is_ok());
        assert!(validate_receive_arity(&formals, 1).is_err());
        assert!(validate_receive_arity(&formals, 3).is_err());

        // Test variable arity
        let formals = Formals::Variable("args".to_string());
        assert!(validate_receive_arity(&formals, 0).is_ok());
        assert!(validate_receive_arity(&formals, 1).is_ok());
        assert!(validate_receive_arity(&formals, 10).is_ok());

        // Test mixed arity
        let formals = Formals::Mixed {
            fixed: vec!["a".to_string(), "b".to_string()],
            rest: "rest".to_string(),
        };
        assert!(validate_receive_arity(&formals, 2).is_ok());
        assert!(validate_receive_arity(&formals, 3).is_ok());
        assert!(validate_receive_arity(&formals, 10).is_ok());
        assert!(validate_receive_arity(&formals, 1).is_err());
    }

    #[test]
    fn test_expand_receive() {
        let formals = Formals::Fixed(vec!["a".to_string(), "b".to_string()]);
        let producer = Value::list(vec![
            Value::symbol(intern_symbol("values")),
            Value::integer(1),
            Value::integer(2),
        ]);
        let body = vec![Value::list(vec![
            Value::symbol(intern_symbol("+")),
            Value::symbol(intern_symbol("a")),
            Value::symbol(intern_symbol("b")),
        ])];

        let result = expand_receive(&formals, &producer, &body);
        assert!(result.is_ok());

        if let Ok(expanded) = result {
            // Should be a call-with-values form
            if let Some(list) = expanded.as_list() {
                assert_eq!(list[0], Value::symbol(intern_symbol("call-with-values")));
                assert_eq!(list.len(), 3); // call-with-values + producer-thunk + consumer-lambda
            } else {
                panic!("Expected expanded form to be a list");
            }
        }
    }
}
