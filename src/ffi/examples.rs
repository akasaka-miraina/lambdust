//! Examples demonstrating FFI functionality.
//!
//! This module contains examples showing how to use the FFI system
//! to call Rust functions from Lambdust code.

use super::*;
use crate::eval::Value;

/// Example demonstrating basic FFI usage.
pub fn demo_basic_ffi() {
    println!("=== Basic FFI Demo ===");
    
    // Create an FFI bridge with built-in functions
    let bridge = FfiBridge::with_builtins();
    
    // List available functions
    println!("Available FFI functions:");
    for function in bridge.list_functions() {
        if let Some(info) = bridge.get_function_info(&function) {
            println!("  - {}: {} -> {}", 
                     info.name, 
                     info.parameter_types.join(", "), 
                     info.return_type);
        }
    }
    
    println!();
}

/// Example demonstrating arithmetic operations.
pub fn demo_arithmetic() {
    println!("=== Arithmetic Operations Demo ===");
    
    let bridge = FfiBridge::with_builtins();
    
    // Addition
    let args = vec![Value::number(2.0), Value::number(3.0)];
    if let Ok(result) = bridge.call_rust_function("add", &args) {
        println!("(primitive 'add 2 3) => {result}");
    }
    
    // Subtraction
    let args = vec![Value::number(10.0), Value::number(4.0)];
    if let Ok(result) = bridge.call_rust_function("subtract", &args) {
        println!("(primitive 'subtract 10 4) => {result}");
    }
    
    // Multiplication
    let args = vec![Value::number(6.0), Value::number(7.0)];
    if let Ok(result) = bridge.call_rust_function("multiply", &args) {
        println!("(primitive 'multiply 6 7) => {result}");
    }
    
    // Division
    let args = vec![Value::number(15.0), Value::number(3.0)];
    if let Ok(result) = bridge.call_rust_function("divide", &args) {
        println!("(primitive 'divide 15 3) => {result}");
    }
    
    println!();
}

/// Example demonstrating string operations.
pub fn demo_strings() {
    println!("=== String Operations Demo ===");
    
    let bridge = FfiBridge::with_builtins();
    
    // String length
    let args = vec![Value::string("Hello, World!")];
    if let Ok(result) = bridge.call_rust_function("string-length", &args) {
        println!("(primitive 'string-length \"Hello, World!\") => {result}");
    }
    
    // String concatenation
    let args = vec![
        Value::string("Hello"),
        Value::string(", "),
        Value::string("FFI"),
        Value::string(" "),
        Value::string("World!")
    ];
    if let Ok(result) = bridge.call_rust_function("string-concat", &args) {
        println!("(primitive 'string-concat \"Hello\" \", \" \"FFI\" \" \" \"World!\") => {result}");
    }
    
    // String case conversion
    let args = vec![Value::string("lambdust")];
    if let Ok(result) = bridge.call_rust_function("string-upper", &args) {
        println!("(primitive 'string-upper \"lambdust\") => {result}");
    }
    
    let args = vec![Value::string("RUST")];
    if let Ok(result) = bridge.call_rust_function("string-lower", &args) {
        println!("(primitive 'string-lower \"RUST\") => {result}");
    }
    
    println!();
}

/// Example demonstrating type checking.
pub fn demo_type_checking() {
    println!("=== Type Checking Demo ===");
    
    let bridge = FfiBridge::with_builtins();
    
    let test_values = vec![
        ("42.0", Value::number(42.0)),
        ("\"hello\"", Value::string("hello")),
        ("'(1 2 3)", Value::list(vec![
            Value::number(1.0), 
            Value::number(2.0), 
            Value::number(3.0)
        ])),
        ("#t", Value::boolean(true)),
        ("#f", Value::boolean(false)),
    ];
    
    let type_predicates = vec!["number?", "string?", "list?", "boolean?"];
    
    for (value_desc, value) in &test_values {
        for predicate in &type_predicates {
            let args = vec![value.clone()];
            if let Ok(result) = bridge.call_rust_function(predicate, &args) {
                println!("(primitive '{predicate} {value_desc}) => {result}");
            }
        }
        println!();
    }
}

/// Example demonstrating list operations.
pub fn demo_lists() {
    println!("=== List Operations Demo ===");
    
    let bridge = FfiBridge::with_builtins();
    
    // Create a test list
    let test_list = Value::list(vec![
        Value::string("apple"),
        Value::string("banana"), 
        Value::string("cherry"),
        Value::string("date")
    ]);
    
    // List length
    let args = vec![test_list.clone()];
    if let Ok(result) = bridge.call_rust_function("list-length", &args) {
        println!("(primitive 'list-length '(\"apple\" \"banana\" \"cherry\" \"date\")) => {result}");
    }
    
    println!();
}

/// Example demonstrating error handling.
pub fn demo_error_handling() {
    println!("=== Error Handling Demo ===");
    
    let bridge = FfiBridge::with_builtins();
    
    // Function not found
    println!("Attempting to call non-existent function:");
    let args = vec![Value::number(42.0)];
    match bridge.call_rust_function("nonexistent-function", &args) {
        Ok(_) => println!("  Unexpected success!"),
        Err(e) => println!("  Error: {e}"),
    }
    
    // Wrong number of arguments
    println!("Attempting to call 'add' with wrong number of arguments:");
    let args = vec![Value::number(42.0)]; // add expects 2 arguments
    match bridge.call_rust_function("add", &args) {
        Ok(_) => println!("  Unexpected success!"),
        Err(e) => println!("  Error: {e}"),
    }
    
    // Type mismatch
    println!("Attempting to call 'add' with wrong argument types:");
    let args = vec![Value::string("hello"), Value::number(42.0)];
    match bridge.call_rust_function("add", &args) {
        Ok(_) => println!("  Unexpected success!"),
        Err(e) => println!("  Error: {e}"),
    }
    
    println!();
}

/// Example showing FFI statistics.
pub fn demo_statistics() {
    println!("=== FFI Statistics Demo ===");
    
    let bridge = FfiBridge::with_builtins();
    
    println!("Initial statistics:");
    let stats = bridge.stats();
    println!("  Registered functions: {}", stats.registered_functions);
    println!("  Total calls: {}", stats.total_calls);
    println!("  Successful calls: {}", stats.successful_calls);
    println!("  Failed calls: {}", stats.failed_calls);
    
    // Make some calls
    let args = vec![Value::number(1.0), Value::number(2.0)];
    let _ = bridge.call_rust_function("add", &args);
    let _ = bridge.call_rust_function("multiply", &args);
    let _ = bridge.call_rust_function("nonexistent", &args); // This will fail
    
    println!("Statistics after some calls:");
    let stats = bridge.stats();
    println!("  Total calls: {}", stats.total_calls);
    println!("  Successful calls: {}", stats.successful_calls);
    println!("  Failed calls: {}", stats.failed_calls);
    
    println!();
}

/// Run all FFI demonstrations.
pub fn run_all_demos() {
    println!("Lambdust FFI System Demonstration");
    println!("=================================");
    println!();
    
    demo_basic_ffi();
    demo_arithmetic();
    demo_strings();
    demo_type_checking();
    demo_lists();
    demo_error_handling();
    demo_statistics();
    
    println!("FFI demonstrations completed!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_functions_dont_panic() {
        // These tests just ensure the demo functions don't panic
        demo_basic_ffi();
        demo_arithmetic();
        demo_strings();
        demo_type_checking();
        demo_lists();
        demo_error_handling();
        demo_statistics();
    }
    
    #[test]
    fn test_run_all_demos() {
        run_all_demos();
    }
}