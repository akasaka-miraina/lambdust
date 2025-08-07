//! Tests for the FFI system functionality.

use lambdust::ffi::*;
use lambdust::eval::Value;

#[test]
fn test_basic_ffi_function_registration() {
    let registry = FfiRegistry::new();
    
    // Register a basic function
    let result = registry.register(AddFunction);
    assert!(result.is_ok());
    
    // Check that the function is listed
    let functions = registry.list_functions();
    assert!(functions.contains(&"add".to_string()));
}

#[test]
fn test_ffi_bridge_with_builtins() {
    let bridge = FfiBridge::with_builtins();
    
    // List available functions
    let functions = bridge.list_functions();
    assert!(functions.contains(&"add".to_string()));
    assert!(functions.contains(&"string-length".to_string()));
    assert!(functions.contains(&"number?".to_string()));
}

#[test]
fn test_ffi_arithmetic_operations() {
    let bridge = FfiBridge::with_builtins();
    
    // Test addition
    let args = vec![Value::number(2.0), Value::number(3.0)];
    let result = bridge.call_rust_function("add", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.as_number().unwrap(), 5.0);
    
    // Test subtraction
    let args = vec![Value::number(10.0), Value::number(4.0)];
    let result = bridge.call_rust_function("subtract", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.as_number().unwrap(), 6.0);
    
    // Test multiplication
    let args = vec![Value::number(3.0), Value::number(4.0)];
    let result = bridge.call_rust_function("multiply", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.as_number().unwrap(), 12.0);
    
    // Test division
    let args = vec![Value::number(15.0), Value::number(3.0)];
    let result = bridge.call_rust_function("divide", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.as_number().unwrap(), 5.0);
}

#[test]
fn test_ffi_string_operations() {
    let bridge = FfiBridge::with_builtins();
    
    // Test string length
    let args = vec![Value::string("hello")];
    let result = bridge.call_rust_function("string-length", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.as_integer().unwrap(), 5);
    
    // Test string concatenation
    let args = vec![Value::string("hello"), Value::string(" "), Value::string("world")];
    let result = bridge.call_rust_function("string-concat", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.as_string().unwrap(), "hello world");
    
    // Test string uppercase
    let args = vec![Value::string("hello")];
    let result = bridge.call_rust_function("string-upper", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.as_string().unwrap(), "HELLO");
    
    // Test string lowercase
    let args = vec![Value::string("WORLD")];
    let result = bridge.call_rust_function("string-lower", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.as_string().unwrap(), "world");
}

#[test]
fn test_ffi_type_checking() {
    let bridge = FfiBridge::with_builtins();
    
    // Test number?
    let args = vec![Value::number(42.0)];
    let result = bridge.call_rust_function("number?", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, Value::boolean(true));
    
    let args = vec![Value::string("hello")];
    let result = bridge.call_rust_function("number?", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, Value::boolean(false));
    
    // Test string?
    let args = vec![Value::string("hello")];
    let result = bridge.call_rust_function("string?", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, Value::boolean(true));
    
    let args = vec![Value::number(42.0)];
    let result = bridge.call_rust_function("string?", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, Value::boolean(false));
}

#[test]
fn test_ffi_list_operations() {
    let bridge = FfiBridge::with_builtins();
    
    // Test list length
    let list = vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)];
    let args = vec![Value::list(list)];
    let result = bridge.call_rust_function("list-length", &args);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.as_integer().unwrap(), 3);
}

#[test]
fn test_ffi_error_handling() {
    let bridge = FfiBridge::with_builtins();
    
    // Test function not found
    let args = vec![Value::number(1.0)];
    let result = bridge.call_rust_function("nonexistent-function", &args);
    assert!(result.is_err());
    
    // Test wrong number of arguments
    let args = vec![Value::number(1.0)]; // add needs 2 arguments
    let result = bridge.call_rust_function("add", &args);
    assert!(result.is_err());
    
    // Test type mismatch (passing string to arithmetic function)
    let args = vec![Value::string("hello"), Value::number(2.0)];
    let result = bridge.call_rust_function("add", &args);
    assert!(result.is_err());
}

#[test]
fn test_ffi_statistics() {
    let bridge = FfiBridge::with_builtins();
    
    // Initial stats
    let stats = bridge.stats();
    let initial_calls = stats.total_calls;
    
    // Make some calls
    let args = vec![Value::number(2.0), Value::number(3.0)];
    let _ = bridge.call_rust_function("add", &args);
    let _ = bridge.call_rust_function("add", &args);
    
    // Check updated stats
    let stats = bridge.stats();
    assert_eq!(stats.total_calls, initial_calls + 2);
    assert_eq!(stats.successful_calls, initial_calls + 2);
    assert!(stats.registered_functions > 0);
}

#[test]
fn test_ffi_value_marshaling() {
    // Test ToLambdust trait
    assert_eq!(42_i64.to_lambdust().as_integer().unwrap(), 42);
    assert_eq!(3.05_f64.to_lambdust().as_number().unwrap(), 3.05);
    assert_eq!("hello".to_lambdust().as_string().unwrap(), "hello");
    assert_eq!(true.to_lambdust(), Value::boolean(true));
    assert_eq!(false.to_lambdust(), Value::boolean(false));
    
    // Test FromLambdust trait
    let val = Value::number(42.0);
    assert_eq!(f64::from_lambdust(&val).unwrap(), 42.0);
    
    let val = Value::string("hello");
    assert_eq!(String::from_lambdust(&val).unwrap(), "hello");
    
    let val = Value::boolean(true);
    assert!(bool::from_lambdust(&val).unwrap());
}

#[test]
fn test_ffi_function_info() {
    let bridge = FfiBridge::with_builtins();
    
    // Get function info
    let info = bridge.get_function_info("add");
    assert!(info.is_some());
    
    let sig = info.unwrap();
    assert_eq!(sig.name, "add");
    assert_eq!(sig.arity, AritySpec::Exact(2));
    assert_eq!(sig.parameter_types.len(), 2);
    assert_eq!(sig.return_type, "number");
}