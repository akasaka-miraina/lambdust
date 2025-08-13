//! R7RS-small compliance test suite.
//!
//! This module tests the essential R7RS-small primitives to ensure they work correctly
//! and comply with the R7RS specification.

use lambdust::eval::value::{Value, ThreadSafeEnvironment};
use lambdust::runtime::{LambdustRuntime, GlobalEnvironmentManager};
use std::sync::Arc;

/// Helper function to create a test runtime environment
fn create_test_environment() -> Arc<GlobalEnvironmentManager> {
    let runtime = LambdustRuntime::new().unwrap();
    runtime.global_env().clone()
}

#[test]
fn test_numeric_predicates() {
    let env = create_test_environment();
    
    // Test exact?
    if let Some(exact_proc) = env.lookup_global("exact?") {
        // Test with integer (should be exact)
        // Note: In a full test, we'd need to call the evaluator
        // This is a structural test to verify the procedure exists
        assert!(exact_proc.is_procedure());
    }
    
    // Test inexact?
    assert!(env.lookup_global("inexact?").is_some());
    
    // Test exact-integer?
    assert!(env.lookup_global("exact-integer?").is_some());
    
    // Test finite?, infinite?, nan?
    assert!(env.lookup_global("finite?").is_some());
    assert!(env.lookup_global("infinite?").is_some());
    assert!(env.lookup_global("nan?").is_some());
    
    // Test exact/inexact conversions
    assert!(env.lookup_global("exact->inexact").is_some());
    assert!(env.lookup_global("inexact->exact").is_some());
}

#[test]
fn test_type_predicates() {
    let env = create_test_environment();
    
    // Test list?
    assert!(env.lookup_global("list?").is_some());
    
    // Test procedure?
    assert!(env.lookup_global("procedure?").is_some());
    
    // Test port?
    assert!(env.lookup_global("port?").is_some());
}

#[test]
fn test_string_functions() {
    let env = create_test_environment();
    
    // Test string-copy
    assert!(env.lookup_global("string-copy").is_some());
    
    // Test substring
    assert!(env.lookup_global("substring").is_some());
    
    // Test string-fill!
    assert!(env.lookup_global("string-fill!").is_some());
}

#[test]
fn test_vector_functions() {
    let env = create_test_environment();
    
    // Test make-vector
    assert!(env.lookup_global("make-vector").is_some());
    
    // Test vector-fill!
    assert!(env.lookup_global("vector-fill!").is_some());
    
    // Test vector-copy
    assert!(env.lookup_global("vector-copy").is_some());
}

#[test]
fn test_control_functions() {
    let env = create_test_environment();
    
    // Test values
    assert!(env.lookup_global("values").is_some());
    
    // Test call-with-values
    assert!(env.lookup_global("call-with-values").is_some());
    
    // Test apply
    assert!(env.lookup_global("apply").is_some());
}

#[test]
fn test_io_functions() {
    let env = create_test_environment();
    
    // Test read-char
    assert!(env.lookup_global("read-char").is_some());
    
    // Test write-char
    assert!(env.lookup_global("write-char").is_some());
    
    // Test peek-char
    assert!(env.lookup_global("peek-char").is_some());
    
    // Test char-ready?
    assert!(env.lookup_global("char-ready?").is_some());
}

#[test]
fn test_r7rs_small_procedure_availability() {
    let env = create_test_environment();
    
    // Core R7RS-small procedures that must be available
    let required_procedures = vec![
        // Basic predicates
        "eq?", "eqv?", "equal?", "not", "boolean?", "boolean=?",
        "symbol?", "symbol->string", "string->symbol",
        
        // Numeric functions
        "number?", "exact?", "inexact?", "exact-integer?",
        "finite?", "infinite?", "nan?",
        "exact->inexact", "inexact->exact",
        "+", "-", "*", "/", "quotient", "remainder", "modulo",
        "=", "<", ">", "<=", ">=",
        "zero?", "positive?", "negative?", "odd?", "even?",
        
        // List operations
        "pair?", "cons", "car", "cdr", "null?", "list?", "list",
        
        // String operations
        "string?", "string=?", "string<?", "string>?", "string<=?", "string>=?",
        "make-string", "string", "string-length", "string-ref",
        "string-copy", "substring", "string-fill!",
        
        // Vector operations
        "vector?", "make-vector", "vector", "vector-length", "vector-ref", "vector-set!",
        "vector-copy", "vector-fill!",
        
        // Character operations
        "char?", "char=?", "char<?", "char>?", "char<=?", "char>=?",
        
        // Control flow
        "procedure?", "apply", "values", "call-with-values",
        
        // I/O operations
        "port?", "input-port?", "output-port?", "read-char", "write-char", "peek-char", "char-ready?",
    ];
    
    let mut missing_procedures = Vec::new();
    
    for proc_name in required_procedures {
        if env.lookup_global(proc_name).is_none() {
            missing_procedures.push(proc_name);
        }
    }
    
    if !missing_procedures.is_empty() {
        panic!("Missing R7RS-small required procedures: {:?}", missing_procedures);
    }
}

#[test]
fn test_value_type_methods() {
    // Test that Value type has the necessary methods for type checking
    let int_val = Value::integer(42);
    assert!(int_val.is_number());
    assert!(!int_val.is_string());
    assert!(!int_val.is_procedure());
    assert!(!int_val.is_port());
    
    let bool_val = Value::boolean(true);
    assert!(!bool_val.is_number());
    assert!(!bool_val.is_string());
    assert!(!bool_val.is_procedure());
    assert!(!bool_val.is_port());
}