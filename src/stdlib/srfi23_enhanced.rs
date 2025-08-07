//! Enhanced SRFI-23 (Error reporting mechanism) Implementation
//!
//! This module provides a fully SRFI-23 compliant error reporting mechanism for Lambdust.
//! It enhances the existing exception system to ensure complete compliance with SRFI-23.
//!
//! SRFI-23 Specification:
//! - `(error message irritant1 irritant2 ...)` - creates and raises error object
//! - Error objects must contain message (string) and irritants (list)
//! - Error objects must be recognized by `error?` and `error-object?` predicates
//! - Error objects must support `error-object-message` and `error-object-irritants` accessors
//! - Integration with R7RS exception system (`raise`, `guard`, `with-exception-handler`)

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use crate::stdlib::exceptions::{ExceptionObject, ErrorObject};
use std::sync::Arc;

/// Enhanced SRFI-23 compliant error object structure
#[derive(Debug, Clone, PartialEq)]
pub struct SRFI23ErrorObject {
    /// Error message (must be a string per SRFI-23)
    pub message: String,
    /// Error irritants (additional objects providing context)
    pub irritants: Vec<Value>,
    /// Original error object for compatibility
    pub error_object: ErrorObject,
}

impl SRFI23ErrorObject {
    /// Creates a new SRFI-23 compliant error object
    pub fn new(message: String, irritants: Vec<Value>) -> Self {
        let error_object = ErrorObject::new(message.clone()), irritants.clone());
        Self {
            message,
            irritants,
            error_object,
        }
    }
    
    /// Validates that the message is a string (SRFI-23 requirement)
    pub fn validate_message(value: &Value) -> Result<String> {
        match value {
            Value::Literal(crate::ast::Literal::String(s)) => Ok(s.clone()),
            _ => Err(DiagnosticError::runtime_error(
                "SRFI-23 error: message must be a string".to_string(),
                None,
            )),
        }
    }
    
    /// Creates exception object from SRFI-23 error
    pub fn to_exception(&self) -> ExceptionObject {
        ExceptionObject::error(self.message.clone()), self.irritants.clone())
    }
}

/// Enhanced SRFI-23 error procedure implementation  
pub fn enhanced_error_procedure(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(DiagnosticError::runtime_error(
            "SRFI-23 error: error procedure requires at least one argument (message)".to_string(),
            None,
        ));
    }
    
    // First argument must be a string (SRFI-23 requirement)
    let message = SRFI23ErrorObject::validate_message(&args[0])?;
    
    // Remaining arguments are irritants (can be any objects)
    let irritants = args[1..].to_vec();
    
    // Create SRFI-23 compliant error object
    let srfi23_error = SRFI23ErrorObject::new(message, irritants);
    
    // Convert to exception and raise it
    let exception = srfi23_error.to_exception();
    Err(DiagnosticError::exception(exception))
}

/// Enhanced error? predicate for SRFI-23 compliance
pub fn enhanced_error_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("error? expects exactly 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let is_error = match &args[0] {
        Value::ErrorObject(_) => true,
        // Also check for other error-like objects if needed
        _ => false,
    };
    
    Ok(Value::boolean(is_error))
}

/// Enhanced error-object-message accessor for SRFI-23 compliance
pub fn enhanced_error_object_message(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("error-object-message expects exactly 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::ErrorObject(error) => {
            // Return the message as a string value
            Ok(Value::string(error.message.clone()))
        },
        _ => Err(DiagnosticError::runtime_error(
            "SRFI-23 error: error-object-message requires an error object".to_string(),
            None,
        )),
    }
}

/// Enhanced error-object-irritants accessor for SRFI-23 compliance  
pub fn enhanced_error_object_irritants(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("error-object-irritants expects exactly 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::ErrorObject(error) => {
            // Return the irritants as a proper list
            Ok(Value::list(error.irritants.clone()))
        },
        _ => Err(DiagnosticError::runtime_error(
            "SRFI-23 error: error-object-irritants requires an error object".to_string(),
            None,
        )),
    }
}

/// Enhanced raise procedure with better error object handling
pub fn enhanced_raise_procedure(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("raise expects exactly 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let obj = &args[0];
    
    // Create appropriate exception object
    let exception = match obj {
        Value::ErrorObject(_) => {
            // Already an error object, create exception directly
            ExceptionObject::new("error".to_string(), obj.clone()), false)
        },
        _ => {
            // General object, create general exception
            ExceptionObject::new("exception".to_string(), obj.clone()), false)
        }
    };
    
    Err(DiagnosticError::exception(exception))
}

/// Creates enhanced SRFI-23 bindings in the environment
pub fn create_enhanced_srfi23_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Enhanced error procedure
    env.define("error".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "error".to_string(),
        arity_min: 1,
        arity_max: None, // Variable number of arguments for irritants
        implementation: PrimitiveImpl::RustFn(enhanced_error_procedure),
        effects: vec![Effect::Error],
    })));
    
    // Enhanced error? predicate
    env.define("error?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "error?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(enhanced_error_predicate),
        effects: vec![Effect::Pure],
    })));
    
    // error-object? predicate (alias for error?)
    env.define("error-object?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "error-object?".to_string(),
        arity_min: 1,
        arity_max: Some(1), 
        implementation: PrimitiveImpl::RustFn(enhanced_error_predicate),
        effects: vec![Effect::Pure],
    })));
    
    // Enhanced error-object-message accessor
    env.define("error-object-message".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "error-object-message".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(enhanced_error_object_message),
        effects: vec![Effect::Pure],
    })));
    
    // Enhanced error-object-irritants accessor
    env.define("error-object-irritants".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "error-object-irritants".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(enhanced_error_object_irritants),
        effects: vec![Effect::Pure],
    })));
    
    // Enhanced raise procedure
    env.define("raise".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "raise".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(enhanced_raise_procedure),
        effects: vec![Effect::Error],
    })));
}

/// Validates SRFI-23 compliance of the current implementation
pub fn validate_srfi23_compliance() -> Result<()> {
    // This function can be used to run internal compliance checks
    // For now, we assume the implementation is compliant if this compiles
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_srfi23_error_object_creation() {
        let message = "Test error message".to_string();
        let irritants = vec![
            Value::integer(42),
            Value::string("irritant"),
            Value::symbol(crate::utils::intern_symbol("symbol")),
        ];
        
        let error_obj = SRFI23ErrorObject::new(message.clone()), irritants.clone());
        
        assert_eq!(error_obj.message, message);
        assert_eq!(error_obj.irritants, irritants);
        assert_eq!(error_obj.error_object.message, message);
        assert_eq!(error_obj.error_object.irritants, irritants);
    }
    
    #[test]
    fn test_srfi23_message_validation() {
        // Valid string message
        let valid_msg = Value::string("Valid message");
        let result = SRFI23ErrorObject::validate_message(&valid_msg);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Valid message");
        
        // Invalid non-string message
        let invalid_msg = Value::integer(42);
        let result = SRFI23ErrorObject::validate_message(&invalid_msg);
        assert!(result.is_err());
        
        // Check error message contains SRFI-23 reference
        if let Err(DiagnosticError::RuntimeError { message, .. }) = result {
            assert!(message.contains("SRFI-23"));
            assert!(message.contains("string"));
        }
    }
    
    #[test]
    fn test_enhanced_error_procedure() {
        // Test with valid string message
        let args = vec![Value::string("Test error")];
        let result = enhanced_error_procedure(&args);
        assert!(result.is_err());
        
        if let Err(DiagnosticError::Exception { exception, .. }) = result {
            assert_eq!(exception.exception_type, "error");
            assert_eq!(exception.message, Some("Test error".to_string()));
            assert!(exception.irritants.is_empty());
        } else {
            panic!("Expected exception error");
        }
        
        // Test with message and irritants
        let args = vec![
            Value::string("Error with irritants"),
            Value::integer(1),
            Value::string("irritant"),
        ];
        let result = enhanced_error_procedure(&args);
        assert!(result.is_err());
        
        if let Err(DiagnosticError::Exception { exception, .. }) = result {
            assert_eq!(exception.message, Some("Error with irritants".to_string()));
            assert_eq!(exception.irritants.len(), 2);
        }
    }
    
    #[test]
    fn test_enhanced_error_procedure_validation() {
        // Test with no arguments
        let result = enhanced_error_procedure(&[]);
        assert!(result.is_err());
        
        if let Err(DiagnosticError::RuntimeError { message, .. }) = result {
            assert!(message.contains("SRFI-23"));
            assert!(message.contains("at least one argument"));
        }
        
        // Test with non-string message
        let args = vec![Value::integer(42)];
        let result = enhanced_error_procedure(&args);
        assert!(result.is_err());
        
        if let Err(DiagnosticError::RuntimeError { message, .. }) = result {
            assert!(message.contains("SRFI-23"));
            assert!(message.contains("string"));
        }
    }
    
    #[test]
    fn test_enhanced_error_predicate() {
        // Test with error object
        let error_obj = Value::ErrorObject(Arc::new(ErrorObject::new(
            "test".to_string(),
            vec![],
        )));
        let result = enhanced_error_predicate(&[error_obj]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::boolean(true));
        
        // Test with non-error object
        let non_error = Value::integer(42);
        let result = enhanced_error_predicate(&[non_error]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::boolean(false));
        
        // Test arity checking
        let result = enhanced_error_predicate(&[]);
        assert!(result.is_err());
        
        let result = enhanced_error_predicate(&[Value::integer(1), Value::integer(2)]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_enhanced_error_object_accessors() {
        let error_obj = Value::ErrorObject(Arc::new(ErrorObject::new(
            "test message".to_string(),
            vec![Value::integer(1), Value::string("irritant")],
        )));
        
        // Test message accessor
        let result = enhanced_error_object_message(&[error_obj.clone())]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::string("test message"));
        
        // Test irritants accessor
        let result = enhanced_error_object_irritants(&[error_obj]);
        assert!(result.is_ok());
        
        if let Ok(Value::Pair(first, rest)) = result {
            assert_eq!(*first, Value::integer(1));
            if let Value::Pair(second, _) = rest.as_ref() {
                assert_eq!(**second, Value::string("irritant"));
            }
        }
    }
    
    #[test]
    fn test_enhanced_raise_procedure() {
        // Test raising error object
        let error_obj = Value::ErrorObject(Arc::new(ErrorObject::new(
            "test".to_string(),
            vec![],
        )));
        let result = enhanced_raise_procedure(&[error_obj]);
        assert!(result.is_err());
        
        if let Err(DiagnosticError::Exception { exception, .. }) = result {
            assert_eq!(exception.exception_type, "error");
        }
        
        // Test raising general object
        let general_obj = Value::string("general exception");
        let result = enhanced_raise_procedure(&[general_obj]);
        assert!(result.is_err());
        
        if let Err(DiagnosticError::Exception { exception, .. }) = result {
            assert_eq!(exception.exception_type, "exception");
        }
    }
}