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
        LambdustError::runtime_error(format!("Marshal error: {error:?}"))
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
            .map_err(|_| LambdustError::type_error("Expected i64".to_string()))
    }

    /// Convert f64 to Scheme value
    fn convert_f64(any: Box<dyn Any>) -> Result<Value> {
        any.downcast::<f64>()
            .map(|value| Value::Number(SchemeNumber::Real(*value)))
            .map_err(|_| LambdustError::type_error("Expected f64".to_string()))
    }

    /// Convert String to Scheme value
    fn convert_string(any: Box<dyn Any>) -> Result<Value> {
        any.downcast::<String>()
            .map(|value| Value::String(*value))
            .map_err(|_| LambdustError::type_error("Expected String".to_string()))
    }

    /// Convert bool to Scheme value
    fn convert_bool(any: Box<dyn Any>) -> Result<Value> {
        any.downcast::<bool>()
            .map(|value| Value::Boolean(*value))
            .map_err(|_| LambdustError::type_error("Expected bool".to_string()))
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
