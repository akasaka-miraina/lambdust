//! Unit tests for marshal module
//!
//! Tests type-safe marshalling between Rust and Scheme values

use std::ffi::CString;

use lambdust::error::LambdustError;
use lambdust::lexer::SchemeNumber;
use lambdust::marshal::{
    c_int_to_scheme, c_string_to_scheme, free_c_string, scheme_string_to_c, scheme_to_c_int,
    Marshallable, TypeSafeMarshaller,
};
use lambdust::value::Value;

#[test]
fn test_marshallable_basic_types() {
    // Test i64
    let num: i64 = 42;
    let scheme_val = num.to_scheme().unwrap();
    assert_eq!(scheme_val, Value::Number(SchemeNumber::Integer(42)));
    let back: i64 = i64::from_scheme(&scheme_val).unwrap();
    assert_eq!(back, 42);

    // Test String
    let text = "hello".to_string();
    let scheme_val = text.to_scheme().unwrap();
    assert_eq!(scheme_val, Value::String("hello".to_string()));
    let back: String = String::from_scheme(&scheme_val).unwrap();
    assert_eq!(back, "hello");

    // Test bool
    let flag = true;
    let scheme_val = flag.to_scheme().unwrap();
    assert_eq!(scheme_val, Value::Boolean(true));
    let back: bool = bool::from_scheme(&scheme_val).unwrap();
    assert!(back);
}

#[test]
fn test_marshallable_vec() {
    let vec_data = vec![1i64, 2, 3];
    let scheme_val = vec_data.to_scheme().unwrap();

    // Should create a proper list
    let back: Vec<i64> = Vec::from_scheme(&scheme_val).unwrap();
    assert_eq!(back, vec![1, 2, 3]);
}

#[test]
fn test_type_safe_marshaller() {
    let marshaller = TypeSafeMarshaller::new();

    // Test with registered types
    let value = marshaller.rust_to_scheme(42i64).unwrap();
    assert_eq!(value, Value::Number(SchemeNumber::Integer(42)));

    let string_val = marshaller.rust_to_scheme("test".to_string()).unwrap();
    assert_eq!(string_val, Value::String("test".to_string()));
}

#[test]
fn test_c_conversions() {
    // Test C int conversion
    let c_val = c_int_to_scheme(100).unwrap();
    assert_eq!(c_val, Value::Number(SchemeNumber::Integer(100)));

    let back = scheme_to_c_int(&c_val).unwrap();
    assert_eq!(back, 100);
}

#[test]
fn test_c_string_conversions() {
    // Test valid C string conversion
    let test_str = "hello world";
    let c_string = CString::new(test_str).unwrap();
    let c_ptr = c_string.as_ptr();

    let scheme_val = unsafe { c_string_to_scheme(c_ptr) }.unwrap();
    assert_eq!(scheme_val, Value::String("hello world".to_string()));

    // Test scheme string to C conversion
    let scheme_string = Value::String("test string".to_string());
    let c_ptr = scheme_string_to_c(&scheme_string).unwrap();

    // Convert back to verify
    let reconstructed = unsafe { CString::from_raw(c_ptr) };
    assert_eq!(reconstructed.to_str().unwrap(), "test string");
}

#[test]
fn test_c_string_null_pointer() {
    // Test null pointer handling
    let result = unsafe { c_string_to_scheme(std::ptr::null()) };
    assert!(result.is_err());

    match result.unwrap_err() {
        LambdustError::RuntimeError { message, .. } => {
            assert!(message.contains("Marshal error"));
        }
        _ => panic!("Expected marshal error for null pointer"),
    }
}

#[test]
fn test_c_string_with_null_bytes() {
    // Test string containing null bytes
    let scheme_val = Value::String("hello\0world".to_string());
    let result = scheme_string_to_c(&scheme_val);

    assert!(result.is_err());
    match result.unwrap_err() {
        LambdustError::RuntimeError { message, .. } => {
            assert!(message.contains("String contains null bytes"));
        }
        _ => panic!("Expected error for string with null bytes"),
    }
}

#[test]
fn test_c_string_type_mismatch() {
    // Test type mismatch in scheme_string_to_c
    let non_string = Value::Number(SchemeNumber::Integer(42));
    let result = scheme_string_to_c(&non_string);

    assert!(result.is_err());
    match result.unwrap_err() {
        LambdustError::RuntimeError { message, .. } => {
            assert!(message.contains("TypeMismatch"));
        }
        _ => panic!("Expected type mismatch error"),
    }
}

#[test]
fn test_scheme_to_c_int_type_errors() {
    // Test various non-numeric types
    let test_cases = vec![
        Value::String("not a number".to_string()),
        Value::Boolean(true),
        Value::Nil,
        Value::Symbol("symbol".to_string()),
    ];

    for val in test_cases {
        let result = scheme_to_c_int(&val);
        assert!(result.is_err(), "Expected error for value: {:?}", val);
    }
}

#[test]
fn test_free_c_string_safety() {
    // Test that free_c_string handles null pointers safely
    unsafe {
        free_c_string(std::ptr::null_mut());
    }
    // Should not panic or cause issues

    // Test proper deallocation cycle
    let scheme_val = Value::String("test".to_string());
    let c_ptr = scheme_string_to_c(&scheme_val).unwrap();

    // This should safely deallocate
    unsafe {
        free_c_string(c_ptr);
    }
}
