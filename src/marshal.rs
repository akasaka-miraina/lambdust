//! Type-safe marshalling between Rust and Scheme values
//!
//! This module provides safe conversion between Rust types and Scheme values,
//! isolating unsafe operations to ensure memory safety.

use crate::error::{LambdustError, Result};
use crate::lexer::SchemeNumber;
use crate::value::Value;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Error types for marshalling operations
#[derive(Debug, Clone)]
pub enum MarshalError {
    /// Type mismatch between expected and actual types
    TypeMismatch {
        /// Expected type name
        expected: String,
        /// Actual type name
        found: String,
    },
    /// Conversion failed for a specific reason
    ConversionFailed(String),
    /// Unsupported type for conversion
    UnsupportedType(String),
    /// Null pointer error
    NullPointer,
    /// Invalid UTF-8 string
    InvalidUtf8,
}

impl From<MarshalError> for LambdustError {
    fn from(error: MarshalError) -> Self {
        LambdustError::RuntimeError(format!("Marshal error: {error:?}"))
    }
}

/// Trait for types that can be converted to/from Scheme values
pub trait Marshallable: 'static {
    /// Convert from Scheme value to Rust type
    fn from_scheme(value: &Value) -> Result<Self>
    where
        Self: Sized;

    /// Convert from Rust type to Scheme value
    fn to_scheme(&self) -> Result<Value>;

    /// Get the corresponding ValueType for this Rust type
    fn value_type() -> crate::host::ValueType;
}

/// Type converter function signature
pub type TypeConverter = Box<dyn Fn(Box<dyn Any>) -> Result<Value>>;

/// Type-safe marshaller for Rust-Scheme value conversion
pub struct TypeSafeMarshaller {
    /// Registry of type converters
    type_registry: HashMap<TypeId, TypeConverter>,
}

impl std::fmt::Debug for TypeSafeMarshaller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeSafeMarshaller")
            .field(
                "type_registry",
                &format!("HashMap with {} entries", self.type_registry.len()),
            )
            .finish()
    }
}

impl TypeSafeMarshaller {
    /// Create a new marshaller with default converters
    pub fn new() -> Self {
        let mut marshaller = Self {
            type_registry: HashMap::new(),
        };

        // Register built-in type converters
        marshaller.register_builtin_converters();
        marshaller
    }

    /// Register built-in type converters
    fn register_builtin_converters(&mut self) {
        self.register_converter::<i64>(Box::new(Self::convert_i64));
        self.register_converter::<f64>(Box::new(Self::convert_f64));
        self.register_converter::<String>(Box::new(Self::convert_string));
        self.register_converter::<bool>(Box::new(Self::convert_bool));
    }

    /// Convert i64 to Scheme value
    fn convert_i64(any: Box<dyn Any>) -> Result<Value> {
        any.downcast::<i64>()
            .map(|value| Value::Number(SchemeNumber::Integer(*value)))
            .map_err(|_| LambdustError::TypeError("Expected i64".to_string()))
    }

    /// Convert f64 to Scheme value
    fn convert_f64(any: Box<dyn Any>) -> Result<Value> {
        any.downcast::<f64>()
            .map(|value| Value::Number(SchemeNumber::Real(*value)))
            .map_err(|_| LambdustError::TypeError("Expected f64".to_string()))
    }

    /// Convert String to Scheme value
    fn convert_string(any: Box<dyn Any>) -> Result<Value> {
        any.downcast::<String>()
            .map(|value| Value::String(*value))
            .map_err(|_| LambdustError::TypeError("Expected String".to_string()))
    }

    /// Convert bool to Scheme value
    fn convert_bool(any: Box<dyn Any>) -> Result<Value> {
        any.downcast::<bool>()
            .map(|value| Value::Boolean(*value))
            .map_err(|_| LambdustError::TypeError("Expected bool".to_string()))
    }

    /// Register a type converter
    pub fn register_converter<T: 'static>(&mut self, converter: TypeConverter) {
        self.type_registry.insert(TypeId::of::<T>(), converter);
    }

    /// Convert Rust type to Scheme value using registered converters
    pub fn rust_to_scheme<T: 'static>(&self, value: T) -> Result<Value> {
        let type_id = TypeId::of::<T>();
        if let Some(converter) = self.type_registry.get(&type_id) {
            converter(Box::new(value))
        } else {
            Err(MarshalError::UnsupportedType(format!("No converter for type {type_id:?}")).into())
        }
    }

    /// Convert Scheme value to Rust type
    pub fn scheme_to_rust<T: Marshallable>(&self, value: &Value) -> Result<T> {
        T::from_scheme(value)
    }
}

impl Default for TypeSafeMarshaller {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Marshallable for common types

impl Marshallable for i64 {
    fn from_scheme(value: &Value) -> Result<Self> {
        match value {
            Value::Number(SchemeNumber::Integer(n)) => Ok(*n),
            Value::Number(SchemeNumber::Real(n)) => Ok(*n as i64),
            Value::Number(SchemeNumber::Rational(num, den)) => Ok(num / den),
            Value::Number(SchemeNumber::Complex(real, _)) => Ok(*real as i64),
            _ => Err(MarshalError::TypeMismatch {
                expected: "Number".to_string(),
                found: format!("{value:?}"),
            }
            .into()),
        }
    }

    fn to_scheme(&self) -> Result<Value> {
        Ok(Value::Number(SchemeNumber::Integer(*self)))
    }

    fn value_type() -> crate::host::ValueType {
        crate::host::ValueType::Number
    }
}

impl Marshallable for f64 {
    fn from_scheme(value: &Value) -> Result<Self> {
        match value {
            Value::Number(SchemeNumber::Real(n)) => Ok(*n),
            Value::Number(SchemeNumber::Integer(n)) => Ok(*n as f64),
            Value::Number(SchemeNumber::Rational(num, den)) => Ok(*num as f64 / *den as f64),
            Value::Number(SchemeNumber::Complex(real, _)) => Ok(*real),
            _ => Err(MarshalError::TypeMismatch {
                expected: "Number".to_string(),
                found: format!("{value:?}"),
            }
            .into()),
        }
    }

    fn to_scheme(&self) -> Result<Value> {
        Ok(Value::Number(SchemeNumber::Real(*self)))
    }

    fn value_type() -> crate::host::ValueType {
        crate::host::ValueType::Number
    }
}

impl Marshallable for String {
    fn from_scheme(value: &Value) -> Result<Self> {
        match value {
            Value::String(s) => Ok(s.clone()),
            Value::Symbol(s) => Ok(s.clone()),
            _ => Err(MarshalError::TypeMismatch {
                expected: "String or Symbol".to_string(),
                found: format!("{value:?}"),
            }
            .into()),
        }
    }

    fn to_scheme(&self) -> Result<Value> {
        Ok(Value::String(self.clone()))
    }

    fn value_type() -> crate::host::ValueType {
        crate::host::ValueType::String
    }
}

impl Marshallable for bool {
    fn from_scheme(value: &Value) -> Result<Self> {
        match value {
            Value::Boolean(b) => Ok(*b),
            _ => Err(MarshalError::TypeMismatch {
                expected: "Boolean".to_string(),
                found: format!("{value:?}"),
            }
            .into()),
        }
    }

    fn to_scheme(&self) -> Result<Value> {
        Ok(Value::Boolean(*self))
    }

    fn value_type() -> crate::host::ValueType {
        crate::host::ValueType::Boolean
    }
}

impl<T: Marshallable> Marshallable for Vec<T> {
    fn from_scheme(value: &Value) -> Result<Self> {
        if let Some(vec) = value.to_vector() {
            let mut result = Vec::new();
            for item in vec {
                result.push(T::from_scheme(&item)?);
            }
            Ok(result)
        } else {
            Err(MarshalError::TypeMismatch {
                expected: "List".to_string(),
                found: format!("{value:?}"),
            }
            .into())
        }
    }

    fn to_scheme(&self) -> Result<Value> {
        let mut result = Value::Nil;
        for item in self.iter().rev() {
            result = Value::cons(item.to_scheme()?, result);
        }
        Ok(result)
    }

    fn value_type() -> crate::host::ValueType {
        crate::host::ValueType::List
    }
}

/// C-compatible marshalling functions for FFI
///
/// These functions provide safe C FFI interface while isolating unsafe operations
/// Convert C string to Scheme string
///
/// # Safety
/// The input pointer must be a valid null-terminated C string
pub unsafe fn c_string_to_scheme(c_str: *const c_char) -> Result<Value> {
    if c_str.is_null() {
        return Err(MarshalError::NullPointer.into());
    }

    let c_str = unsafe { CStr::from_ptr(c_str) };
    let rust_str = c_str.to_str().map_err(|_| MarshalError::InvalidUtf8)?;

    Ok(Value::String(rust_str.to_string()))
}

/// Convert Scheme string to C string
///
/// Returns a newly allocated C string that must be freed by the caller
pub fn scheme_string_to_c(value: &Value) -> Result<*mut c_char> {
    match value {
        Value::String(s) | Value::Symbol(s) => {
            let c_string = CString::new(s.as_str()).map_err(|_| {
                MarshalError::ConversionFailed("String contains null bytes".to_string())
            })?;
            Ok(c_string.into_raw())
        }
        _ => Err(MarshalError::TypeMismatch {
            expected: "String or Symbol".to_string(),
            found: format!("{value:?}"),
        }
        .into()),
    }
}

/// Convert C integer to Scheme number
pub fn c_int_to_scheme(value: c_int) -> Result<Value> {
    Ok(Value::Number(SchemeNumber::Integer(value as i64)))
}

/// Convert Scheme number to C integer
pub fn scheme_to_c_int(value: &Value) -> Result<c_int> {
    match value {
        Value::Number(SchemeNumber::Integer(n)) => Ok(*n as c_int),
        Value::Number(SchemeNumber::Real(n)) => Ok(*n as c_int),
        Value::Number(SchemeNumber::Rational(num, den)) => Ok((num / den) as c_int),
        Value::Number(SchemeNumber::Complex(real, _)) => Ok(*real as c_int),
        _ => Err(MarshalError::TypeMismatch {
            expected: "Number".to_string(),
            found: format!("{value:?}"),
        }
        .into()),
    }
}

/// Free C string allocated by scheme_string_to_c
///
/// # Safety
/// The pointer must have been allocated by scheme_string_to_c
pub unsafe fn free_c_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = unsafe { CString::from_raw(ptr) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(back, true);
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
        use std::ffi::CString;

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
}
