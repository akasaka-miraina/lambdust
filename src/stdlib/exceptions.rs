//! Exception handling for the Lambdust standard library.
//!
//! This module implements R7RS-small compliant exception handling including:
//! - Error objects and predicates (error?, error-object?, read-error?, file-error?)
//! - Exception raising (raise, raise-continuable, error)
//! - Error object accessors (error-object-message, error-object-irritants)
//! - Complete error type hierarchy (general, read, file errors)
//! - Exception handling infrastructure
//! - Integration with the guard syntax form
//! - Helper functions for creating and raising typed errors
//!
//! ## R7RS-small Compliance
//! 
//! All required R7RS-small exception procedures are implemented:
//! - `error` - Creates and raises an error object with message and irritants
//! - `raise` - Raises a non-continuable exception 
//! - `raise-continuable` - Raises a continuable exception
//! - `error-object?` - Tests if argument is an error object
//! - `error-object-message` - Gets error message from error object
//! - `error-object-irritants` - Gets error irritants from error object  
//! - `read-error?` - Tests if error is a read error
//! - `file-error?` - Tests if error is a file error
//!
//! ## Error Types
//!
//! The module supports three types of errors as per R7RS:
//! - **General errors** - Created by `error` procedure
//! - **Read errors** - Parsing and syntax errors
//! - **File errors** - I/O and filesystem errors
//!
//! ## Usage Examples
//!
//! ```rust
//! use lambdust::stdlib::exceptions::*;
//! use lambdust::eval::value::Value;
//!
//! // Create error objects
//! let general_error = create_error_object("Something went wrong".to_string(), vec![]);
//! let read_error = create_read_error_object("Invalid syntax".to_string(), vec![]);
//! let file_error = create_file_error_object("File not found".to_string(), vec![Value::string("test.txt")]);
//!
//! // Raise errors (returns Result<Value> with exception)
//! let _ = raise_read_error("Parse error".to_string(), vec![]);
//! let _ = raise_file_error("I/O error".to_string(), vec![]);
//! ```

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect; 
use std::sync::Arc;
use std::fmt;

/// R7RS Exception object representation
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionObject {
    /// The type of exception (error, read-error, file-error, etc.)
    pub exception_type: String,
    /// Exception payload/value
    pub value: Value,
    /// Optional message (for error objects)
    pub message: Option<String>,
    /// Optional irritants (for error objects)
    pub irritants: Vec<Value>,
    /// Whether this exception can be continued from
    pub continuable: bool,
}

/// R7RS error types for proper categorization
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    /// General error (created by `error` procedure)
    General,
    /// Read error (parsing/syntax errors)
    ReadError,
    /// File error (I/O and filesystem errors) 
    FileError,
}

/// R7RS Error object (subtype of exception object)
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorObject {
    /// Error message
    pub message: String,
    /// Error irritants (additional objects that caused the error)
    pub irritants: Vec<Value>,
    /// Type of error (general, read, file)
    pub error_type: ErrorType,
}

impl ExceptionObject {
    /// Creates a new general exception
    pub fn new(exception_type: String, value: Value, continuable: bool) -> Self {
        Self {
            exception_type,
            value,
            message: None,
            irritants: Vec::new(),
            continuable,
        }
    }
    
    /// Creates a new error exception
    pub fn error(message: String, irritants: Vec<Value>) -> Self {
        Self {
            exception_type: "error".to_string(),
            value: Value::ErrorObject(Arc::new(ErrorObject::new(
                message.clone(),
                irritants.clone(),
            ))),
            message: Some(message),
            irritants,
            continuable: false,
        }
    }
    
    /// Creates a read error exception
    pub fn read_error(message: String, irritants: Vec<Value>) -> Self {
        Self {
            exception_type: "read-error".to_string(),
            value: Value::ErrorObject(Arc::new(ErrorObject::read_error(
                message.clone(),
                irritants.clone(),
            ))),
            message: Some(message),
            irritants,
            continuable: false,
        }
    }
    
    /// Creates a file error exception
    pub fn file_error(message: String, irritants: Vec<Value>) -> Self {
        Self {
            exception_type: "file-error".to_string(),
            value: Value::ErrorObject(Arc::new(ErrorObject::file_error(
                message.clone(),
                irritants.clone(),
            ))),
            message: Some(message),
            irritants,
            continuable: false,
        }
    }
    
    /// Checks if this is an error object
    pub fn is_error(&self) -> bool {
        matches!(self.value, Value::ErrorObject(_))
    }
    
    /// Checks if this is a specific error type
    pub fn is_error_type(&self, error_type: &str) -> bool {
        self.exception_type == error_type
    }
}

impl ErrorObject {
    /// Creates a new general error object
    pub fn new(message: String, irritants: Vec<Value>) -> Self {
        Self { 
            message, 
            irritants,
            error_type: ErrorType::General,
        }
    }
    
    /// Creates a new read error object
    pub fn read_error(message: String, irritants: Vec<Value>) -> Self {
        Self {
            message,
            irritants,
            error_type: ErrorType::ReadError,
        }
    }
    
    /// Creates a new file error object
    pub fn file_error(message: String, irritants: Vec<Value>) -> Self {
        Self {
            message,
            irritants,
            error_type: ErrorType::FileError,
        }
    }
    
    /// Checks if this is a read error
    pub fn is_read_error(&self) -> bool {
        matches!(self.error_type, ErrorType::ReadError)
    }
    
    /// Checks if this is a file error
    pub fn is_file_error(&self) -> bool {
        matches!(self.error_type, ErrorType::FileError)
    }
}

impl fmt::Display for ExceptionObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = &self.message {
            write!(f, "{}: {}", self.exception_type, msg)
        } else {
            write!(f, "{}: {}", self.exception_type, self.value)
        }
    }
}

impl fmt::Display for ErrorObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Creates exception handling bindings for the standard library
pub fn create_exception_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Exception raising procedures
    bind_exception_raising(env);
    
    // Exception predicates
    bind_exception_predicates(env);
    
    // Error object accessors
    bind_error_object_accessors(env);
    
    // Exception handling procedures
    bind_exception_handling(env);
}

/// Binds exception raising procedures
fn bind_exception_raising(env: &Arc<ThreadSafeEnvironment>) {
    // raise - raises a non-continuable exception
    env.define("raise".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "raise".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_raise),
        effects: vec![Effect::Error],
    })));
    
    // raise-continuable - raises a continuable exception
    env.define("raise-continuable".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "raise-continuable".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_raise_continuable),
        effects: vec![Effect::Error],
    })));
    
    // error - creates and raises an error object
    env.define("error".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "error".to_string(),
        arity_min: 1,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_error),
        effects: vec![Effect::Error],
    })));
}

/// Binds exception predicates
fn bind_exception_predicates(env: &Arc<ThreadSafeEnvironment>) {
    // error? - tests if object is an error object
    env.define("error?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "error?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_error_p),
        effects: vec![Effect::Pure],
    })));
    
    // error-object? - tests if object is an error object (alias for error?)
    env.define("error-object?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "error-object?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_error_p),
        effects: vec![Effect::Pure],
    })));
    
    // read-error? - tests if object is a read error
    env.define("read-error?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "read-error?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_read_error_p),
        effects: vec![Effect::Pure],
    })));
    
    // file-error? - tests if object is a file error
    env.define("file-error?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "file-error?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_file_error_p),
        effects: vec![Effect::Pure],
    })));
}

/// Binds error object accessors
fn bind_error_object_accessors(env: &Arc<ThreadSafeEnvironment>) {
    // error-object-message - gets the message from an error object
    env.define("error-object-message".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "error-object-message".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_error_object_message),
        effects: vec![Effect::Pure],
    })));
    
    // error-object-irritants - gets the irritants from an error object
    env.define("error-object-irritants".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "error-object-irritants".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_error_object_irritants),
        effects: vec![Effect::Pure],
    })));
}

/// Binds exception handling procedures
fn bind_exception_handling(env: &Arc<ThreadSafeEnvironment>) {
    // with-exception-handler - installs an exception handler
    env.define("with-exception-handler".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "with-exception-handler".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_with_exception_handler),
        effects: vec![Effect::Error],
    })));
}

// ============= PUBLIC ERROR CREATION HELPERS =============

/// Creates a general error object and returns it as a Value
pub fn create_error_object(message: String, irritants: Vec<Value>) -> Value {
    Value::ErrorObject(Arc::new(ErrorObject::new(message, irritants)))
}

/// Creates a read error object and returns it as a Value
pub fn create_read_error_object(message: String, irritants: Vec<Value>) -> Value {
    Value::ErrorObject(Arc::new(ErrorObject::read_error(message, irritants)))
}

/// Creates a file error object and returns it as a Value
pub fn create_file_error_object(message: String, irritants: Vec<Value>) -> Value {
    Value::ErrorObject(Arc::new(ErrorObject::file_error(message, irritants)))
}

/// Creates and raises a read error exception
pub fn raise_read_error(message: String, irritants: Vec<Value>) -> Result<Value> {
    let exception = ExceptionObject::read_error(message, irritants);
    Err(Box::new(DiagnosticError::exception(exception)))
}

/// Creates and raises a file error exception  
pub fn raise_file_error(message: String, irritants: Vec<Value>) -> Result<Value> {
    let exception = ExceptionObject::file_error(message, irritants);
    Err(Box::new(DiagnosticError::exception(exception)))
}

// ============= EXCEPTION RAISING IMPLEMENTATIONS =============

/// raise procedure - raises a non-continuable exception
pub fn primitive_raise(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("raise expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let obj = &args[0];
    
    // Create an exception object from the raised value
    let exception = if let Value::ErrorObject(_) = obj {
        // Already an error object, wrap as exception
        ExceptionObject::new("error".to_string(), obj.clone(), false)
    } else {
        // General exception
        ExceptionObject::new("exception".to_string(), obj.clone(), false)
    };
    
    // Convert to a DiagnosticError that carries the exception information
    Err(Box::new(DiagnosticError::exception(exception)))
}

/// raise-continuable procedure - raises a continuable exception
fn primitive_raise_continuable(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("raise-continuable expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let obj = &args[0];
    
    // Create a continuable exception object
    let exception = if let Value::ErrorObject(_) = obj {
        // Already an error object, wrap as continuable exception
        ExceptionObject::new("error".to_string(), obj.clone(), true)
    } else {
        // General continuable exception
        ExceptionObject::new("exception".to_string(), obj.clone(), true)
    };
    
    // Convert to a DiagnosticError that carries the exception information
    Err(Box::new(DiagnosticError::exception(exception)))
}

/// error procedure - creates and raises an error object
fn primitive_error(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "error requires at least a message argument".to_string(),
            None,
        )));
    }
    
    // First argument must be a string (the message)
    let message = match &args[0] {
        Value::Literal(crate::ast::Literal::String(s)) => s.clone(),
        _ => return Err(Box::new(DiagnosticError::runtime_error(
            "error message must be a string".to_string(),
            None,
        ))),
    };
    
    // Remaining arguments are irritants
    let irritants = args[1..].to_vec();
    
    // Create error object and raise it
    let exception = ExceptionObject::error(message, irritants);
    
    Err(Box::new(DiagnosticError::exception(exception)))
}

// ============= EXCEPTION PREDICATE IMPLEMENTATIONS =============

/// error? and error-object? predicate
fn primitive_error_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("error? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let is_error = matches!(args[0], Value::ErrorObject(_));
    Ok(Value::boolean(is_error))
}

/// read-error? predicate
fn primitive_read_error_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("read-error? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let is_read_error = match &args[0] {
        Value::ErrorObject(error) => error.is_read_error(),
        _ => false,
    };
    
    Ok(Value::boolean(is_read_error))
}

/// file-error? predicate
fn primitive_file_error_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("file-error? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let is_file_error = match &args[0] {
        Value::ErrorObject(error) => error.is_file_error(),
        _ => false,
    };
    
    Ok(Value::boolean(is_file_error))
}

// ============= ERROR OBJECT ACCESSOR IMPLEMENTATIONS =============

/// error-object-message accessor
fn primitive_error_object_message(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("error-object-message expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    match &args[0] {
        Value::ErrorObject(error) => {
            Ok(Value::string(error.message.clone()))
        },
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "error-object-message requires an error object".to_string(),
            None,
        ))),
    }
}

/// error-object-irritants accessor
fn primitive_error_object_irritants(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("error-object-irritants expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    match &args[0] {
        Value::ErrorObject(error) => {
            Ok(Value::list(error.irritants.clone()))
        },
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "error-object-irritants requires an error object".to_string(),
            None,
        ))),
    }
}

// ============= EXCEPTION HANDLING IMPLEMENTATIONS =============

/// with-exception-handler procedure
fn primitive_with_exception_handler(_args: &[Value]) -> Result<Value> {
    // This requires deeper integration with the evaluator to properly
    // set up exception handling contexts
    Err(Box::new(DiagnosticError::runtime_error(
        "with-exception-handler requires evaluator integration (implemented via guard syntax)".to_string(),
        None,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_exception_object_creation() {
        let irritants = vec![Value::integer(42), Value::string("test")];
        let exception = ExceptionObject::error("Test error".to_string(), irritants.clone());
        
        assert_eq!(exception.exception_type, "error");
        assert!(exception.is_error());
        assert_eq!(exception.message, Some("Test error".to_string()));
        assert_eq!(exception.irritants, irritants);
        assert!(!exception.continuable);
    }
    
    #[test]
    fn test_error_object_creation() {
        let irritants = vec![Value::integer(1), Value::integer(2)];
        let error = ErrorObject::new("Test message".to_string(), irritants.clone());
        
        assert_eq!(error.message, "Test message");
        assert_eq!(error.irritants, irritants);
    }
    
    #[test]
    fn test_error_predicate() {
        let error_obj = Value::ErrorObject(Arc::new(ErrorObject::new(
            "test".to_string(), 
            vec![]
        )));
        let not_error = Value::integer(42);
        
        let result = primitive_error_p(&[error_obj]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_error_p(&[not_error]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_error_object_message() {
        let error_obj = Value::ErrorObject(Arc::new(ErrorObject::new(
            "test message".to_string(), 
            vec![]
        )));
        
        let result = primitive_error_object_message(&[error_obj]).unwrap();
        assert_eq!(result, Value::string("test message"));
    }
    
    #[test]
    fn test_error_object_irritants() {
        let irritants = vec![Value::integer(1), Value::string("test")];
        let error_obj = Value::ErrorObject(Arc::new(ErrorObject::new(
            "test".to_string(), 
            irritants.clone()
        )));
        
        let result = primitive_error_object_irritants(&[error_obj]).unwrap();
        assert_eq!(result, Value::list(irritants));
    }
    
    #[test]
    fn test_raise() {
        let args = vec![Value::string("test exception")];
        let result = primitive_raise(&args);
        assert!(result.is_err());
        
        // Should be a DiagnosticError containing exception information
        if let Err(boxed_err) = result {
            if let DiagnosticError::Exception { exception, .. } = boxed_err.as_ref() {
                assert_eq!(exception.exception_type, "exception");
                assert!(!exception.continuable);
            } else {
                panic!("Expected exception error");
            }
        } else {
            panic!("Expected error result");
        }
    }
    
    #[test]
    fn test_raise_continuable() {
        let args = vec![Value::integer(42)];
        let result = primitive_raise_continuable(&args);
        assert!(result.is_err());
        
        // Should be a DiagnosticError containing continuable exception information
        if let Err(boxed_err) = result {
            if let DiagnosticError::Exception { exception, .. } = boxed_err.as_ref() {
                assert_eq!(exception.exception_type, "exception");
                assert!(exception.continuable);
            } else {
                panic!("Expected exception error");
            }
        } else {
            panic!("Expected error result");
        }
    }
    
    #[test]
    fn test_error_procedure() {
        let args = vec![Value::string("Error message"), Value::integer(42)];
        let result = primitive_error(&args);
        assert!(result.is_err());
        
        // Should be a DiagnosticError containing error exception
        if let Err(boxed_err) = result {
            if let DiagnosticError::Exception { exception, .. } = boxed_err.as_ref() {
                assert_eq!(exception.exception_type, "error");
                assert!(exception.is_error());
                assert_eq!(exception.message, Some("Error message".to_string()));
                assert_eq!(exception.irritants, vec![Value::integer(42)]);
            } else {
                panic!("Expected exception error");
            }
        } else {
            panic!("Expected error result");
        }
    }
    
    #[test]
    fn test_error_arity_errors() {
        // error? with wrong arity
        let result = primitive_error_p(&[]);
        assert!(result.is_err());
        
        let result = primitive_error_p(&[Value::integer(1), Value::integer(2)]);
        assert!(result.is_err());
        
        // raise with wrong arity
        let result = primitive_raise(&[]);
        assert!(result.is_err());
        
        let result = primitive_raise(&[Value::integer(1), Value::integer(2)]);
        assert!(result.is_err());
        
        // error with no arguments
        let result = primitive_error(&[]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_error_types() {
        // Test ErrorType enum
        assert_eq!(ErrorType::General, ErrorType::General);
        assert_ne!(ErrorType::General, ErrorType::ReadError);
        assert_ne!(ErrorType::General, ErrorType::FileError);
        assert_ne!(ErrorType::ReadError, ErrorType::FileError);
    }
    
    #[test]
    fn test_error_object_types() {
        // General error
        let general_error = ErrorObject::new("general".to_string(), vec![]);
        assert!(!general_error.is_read_error());
        assert!(!general_error.is_file_error());
        assert_eq!(general_error.error_type, ErrorType::General);
        
        // Read error
        let read_error = ErrorObject::read_error("read error".to_string(), vec![]);
        assert!(read_error.is_read_error());
        assert!(!read_error.is_file_error());
        assert_eq!(read_error.error_type, ErrorType::ReadError);
        
        // File error
        let file_error = ErrorObject::file_error("file error".to_string(), vec![]);
        assert!(!file_error.is_read_error());
        assert!(file_error.is_file_error());
        assert_eq!(file_error.error_type, ErrorType::FileError);
    }
    
    #[test]
    fn test_exception_object_error_types() {
        let irritants = vec![Value::integer(42)];
        
        // General error exception
        let general_exc = ExceptionObject::error("general error".to_string(), irritants.clone());
        assert_eq!(general_exc.exception_type, "error");
        assert!(general_exc.is_error());
        
        // Read error exception
        let read_exc = ExceptionObject::read_error("read error".to_string(), irritants.clone());
        assert_eq!(read_exc.exception_type, "read-error");
        assert!(read_exc.is_error());
        
        // File error exception
        let file_exc = ExceptionObject::file_error("file error".to_string(), irritants.clone());
        assert_eq!(file_exc.exception_type, "file-error");
        assert!(file_exc.is_error());
    }
    
    #[test]
    fn test_read_error_predicate() {
        // Test with read error object
        let read_error_obj = create_read_error_object("read error".to_string(), vec![]);
        let result = primitive_read_error_p(&[read_error_obj]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        // Test with general error object
        let general_error_obj = create_error_object("general error".to_string(), vec![]);
        let result = primitive_read_error_p(&[general_error_obj]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test with file error object
        let file_error_obj = create_file_error_object("file error".to_string(), vec![]);
        let result = primitive_read_error_p(&[file_error_obj]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test with non-error object
        let non_error = Value::integer(42);
        let result = primitive_read_error_p(&[non_error]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_file_error_predicate() {
        // Test with file error object
        let file_error_obj = create_file_error_object("file error".to_string(), vec![]);
        let result = primitive_file_error_p(&[file_error_obj]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        // Test with general error object
        let general_error_obj = create_error_object("general error".to_string(), vec![]);
        let result = primitive_file_error_p(&[general_error_obj]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test with read error object
        let read_error_obj = create_read_error_object("read error".to_string(), vec![]);
        let result = primitive_file_error_p(&[read_error_obj]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test with non-error object
        let non_error = Value::string("not an error");
        let result = primitive_file_error_p(&[non_error]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_helper_functions() {
        // Test create_error_object
        let general = create_error_object("test".to_string(), vec![Value::integer(1)]);
        match general {
            Value::ErrorObject(err) => {
                assert_eq!(err.message, "test");
                assert_eq!(err.irritants, vec![Value::integer(1)]);
                assert_eq!(err.error_type, ErrorType::General);
            },
            _ => panic!("Expected ErrorObject"),
        }
        
        // Test create_read_error_object
        let read_err = create_read_error_object("read test".to_string(), vec![]);
        match read_err {
            Value::ErrorObject(err) => {
                assert_eq!(err.message, "read test");
                assert_eq!(err.error_type, ErrorType::ReadError);
                assert!(err.is_read_error());
            },
            _ => panic!("Expected ErrorObject"),
        }
        
        // Test create_file_error_object
        let file_err = create_file_error_object("file test".to_string(), vec![]);
        match file_err {
            Value::ErrorObject(err) => {
                assert_eq!(err.message, "file test");
                assert_eq!(err.error_type, ErrorType::FileError);
                assert!(err.is_file_error());
            },
            _ => panic!("Expected ErrorObject"),
        }
    }
    
    #[test]
    fn test_raise_helper_functions() {
        // Test raise_read_error
        let result = raise_read_error("read error".to_string(), vec![Value::integer(42)]);
        assert!(result.is_err());
        
        if let Err(boxed_err) = result {
            if let DiagnosticError::Exception { exception, .. } = boxed_err.as_ref() {
                assert_eq!(exception.exception_type, "read-error");
                assert!(exception.is_error());
            } else {
                panic!("Expected exception error");
            }
        } else {
            panic!("Expected error result");
        }
        
        // Test raise_file_error
        let result = raise_file_error("file error".to_string(), vec![Value::string("test.txt")]);
        assert!(result.is_err());
        
        if let Err(boxed_err) = result {
            if let DiagnosticError::Exception { exception, .. } = boxed_err.as_ref() {
                assert_eq!(exception.exception_type, "file-error");
                assert!(exception.is_error());
            } else {
                panic!("Expected exception error");
            }
        } else {
            panic!("Expected error result");
        }
    }
    
    #[test]
    fn test_error_object_message_with_types() {
        // Test with general error
        let general_error = create_error_object("general message".to_string(), vec![]);
        let result = primitive_error_object_message(&[general_error]).unwrap();
        assert_eq!(result, Value::string("general message"));
        
        // Test with read error
        let read_error = create_read_error_object("read message".to_string(), vec![]);
        let result = primitive_error_object_message(&[read_error]).unwrap();
        assert_eq!(result, Value::string("read message"));
        
        // Test with file error
        let file_error = create_file_error_object("file message".to_string(), vec![]);
        let result = primitive_error_object_message(&[file_error]).unwrap();
        assert_eq!(result, Value::string("file message"));
    }
    
    #[test]
    fn test_error_object_irritants_with_types() {
        let irritants = vec![Value::integer(1), Value::string("test"), Value::boolean(true)];
        
        // Test with general error
        let general_error = create_error_object("message".to_string(), irritants.clone());
        let result = primitive_error_object_irritants(&[general_error]).unwrap();
        assert_eq!(result, Value::list(irritants.clone()));
        
        // Test with read error
        let read_error = create_read_error_object("message".to_string(), irritants.clone());
        let result = primitive_error_object_irritants(&[read_error]).unwrap();
        assert_eq!(result, Value::list(irritants.clone()));
        
        // Test with file error
        let file_error = create_file_error_object("message".to_string(), irritants.clone());
        let result = primitive_error_object_irritants(&[file_error]).unwrap();
        assert_eq!(result, Value::list(irritants.clone()));
    }
}