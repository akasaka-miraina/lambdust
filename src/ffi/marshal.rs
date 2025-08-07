//! Value marshaling between Rust and Lambdust types.
//!
//! This module provides type-safe conversion functions between
//! Rust native types and Lambdust runtime values.

use crate::eval::Value;
use crate::ast::Literal;
use super::FfiError;
use std::collections::HashMap;

/// Trait for converting Rust types to Lambdust values.
pub trait ToLambdust {
    /// Convert this Rust value to a Lambdust value.
    fn to_lambdust(self) -> Value;
}

/// Trait for converting Lambdust values to Rust types.
pub trait FromLambdust: Sized {
    /// Convert a Lambdust value to this Rust type.
    fn from_lambdust(value: &Value) -> Result<Self, FfiError>;
    
    /// Get the expected type name for error reporting.
    fn expected_type() -> &'static str;
}

// ============= IMPLEMENTATIONS FOR BASIC TYPES =============

impl ToLambdust for Value {
    fn to_lambdust(self) -> Value {
        self
    }
}

impl ToLambdust for () {
    fn to_lambdust(self) -> Value {
        Value::Unspecified
    }
}

impl FromLambdust for Value {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        Ok(value.clone())
    }
    
    fn expected_type() -> &'static str {
        "any"
    }
}

impl FromLambdust for () {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        match value {
            Value::Unspecified => Ok(()),
            _ => Err(FfiError::RuntimeError {
                function: "type_conversion".to_string(),
                message: format!("Expected unspecified, got {value}"),
            }),
        }
    }
    
    fn expected_type() -> &'static str {
        "unspecified"
    }
}

// Numbers
impl ToLambdust for f64 {
    fn to_lambdust(self) -> Value {
        Value::number(self)
    }
}

impl FromLambdust for f64 {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        value.as_number().ok_or_else(|| FfiError::RuntimeError {
            function: "type_conversion".to_string(),
            message: format!("Expected number, got {value}"),
        })
    }
    
    fn expected_type() -> &'static str {
        "number"
    }
}

impl ToLambdust for f32 {
    fn to_lambdust(self) -> Value {
        Value::number(self as f64)
    }
}

impl FromLambdust for f32 {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        let num = value.as_number().ok_or_else(|| FfiError::RuntimeError {
            function: "type_conversion".to_string(),
            message: format!("Expected number, got {value}"),
        })?;
        Ok(num as f32)
    }
    
    fn expected_type() -> &'static str {
        "number"
    }
}

impl ToLambdust for i64 {
    fn to_lambdust(self) -> Value {
        Value::integer(self)
    }
}

impl FromLambdust for i64 {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        value.as_integer().or_else(|| {
            // Try to convert from f64 if it's a whole number
            value.as_number().and_then(|n| {
                if n.fract() == 0.0 && n.is_finite() {
                    Some(n as i64)
                } else {
                    None
                }
            })
        }).ok_or_else(|| FfiError::RuntimeError {
            function: "type_conversion".to_string(),
            message: format!("Expected integer, got {value}"),
        })
    }
    
    fn expected_type() -> &'static str {
        "integer"
    }
}

impl ToLambdust for i32 {
    fn to_lambdust(self) -> Value {
        Value::integer(self as i64)
    }
}

impl FromLambdust for i32 {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        let int = i64::from_lambdust(value)?;
        if int >= i32::MIN as i64 && int <= i32::MAX as i64 {
            Ok(int as i32)
        } else {
            Err(FfiError::RuntimeError {
                function: "type_conversion".to_string(),
                message: format!("Integer {int} out of range for i32"),
            })
        }
    }
    
    fn expected_type() -> &'static str {
        "i32"
    }
}

impl ToLambdust for usize {
    fn to_lambdust(self) -> Value {
        Value::integer(self as i64)
    }
}

impl FromLambdust for usize {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        let int = i64::from_lambdust(value)?;
        if int >= 0 {
            Ok(int as usize)
        } else {
            Err(FfiError::RuntimeError {
                function: "type_conversion".to_string(),
                message: format!("Negative integer {int} cannot be converted to usize"),
            })
        }
    }
    
    fn expected_type() -> &'static str {
        "non-negative integer"
    }
}

// Booleans
impl ToLambdust for bool {
    fn to_lambdust(self) -> Value {
        Value::boolean(self)
    }
}

impl FromLambdust for bool {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        match value {
            Value::Literal(Literal::Boolean(b)) => Ok(*b),
            _ => Ok(value.is_truthy()), // Scheme semantics: everything except #f is truthy
        }
    }
    
    fn expected_type() -> &'static str {
        "boolean"
    }
}

// Strings
impl ToLambdust for String {
    fn to_lambdust(self) -> Value {
        Value::string(self)
    }
}

impl ToLambdust for &str {
    fn to_lambdust(self) -> Value {
        Value::string(self)
    }
}

impl FromLambdust for String {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        value.as_string().map(|s| s.to_string()).ok_or_else(|| FfiError::RuntimeError {
            function: "type_conversion".to_string(),
            message: format!("Expected string, got {value}"),
        })
    }
    
    fn expected_type() -> &'static str {
        "string"
    }
}

// Characters
impl ToLambdust for char {
    fn to_lambdust(self) -> Value {
        Value::Literal(Literal::Character(self))
    }
}

impl FromLambdust for char {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        match value {
            Value::Literal(Literal::Character(c)) => Ok(*c),
            _ => Err(FfiError::RuntimeError {
                function: "type_conversion".to_string(),
                message: format!("Expected character, got {value}"),
            }),
        }
    }
    
    fn expected_type() -> &'static str {
        "character"
    }
}

// Lists (vectors in Rust)
impl<T: FromLambdust> FromLambdust for Vec<T> {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        match value.as_list() {
            Some(list_values) => {
                let mut result = Vec::with_capacity(list_values.len());
                for (i, val) in list_values.iter().enumerate() {
                    match T::from_lambdust(val) {
                        Ok(item) => result.push(item),
                        Err(_) => return Err(FfiError::TypeMismatch {
                            function: "type_conversion".to_string(),
                            parameter: i,
                            expected: T::expected_type().to_string(),
                            actual: format!("{val}"),
                        }),
                    }
                }
                Ok(result)
            }
            None => match value {
                Value::Vector(vec_ref) => {
                    let vec = vec_ref.read().unwrap();
                    let mut result = Vec::with_capacity(vec.len());
                    for (i, val) in vec.iter().enumerate() {
                        match T::from_lambdust(val) {
                            Ok(item) => result.push(item),
                            Err(_) => return Err(FfiError::TypeMismatch {
                                function: "type_conversion".to_string(),
                                parameter: i,
                                expected: T::expected_type().to_string(),
                                actual: format!("{val}"),
                            }),
                        }
                    }
                    Ok(result)
                }
                _ => Err(FfiError::RuntimeError {
                    function: "type_conversion".to_string(),
                    message: format!("Expected list or vector, got {value}"),
                }),
            }
        }
    }
    
    fn expected_type() -> &'static str {
        "list"
    }
}

impl<T: ToLambdust> ToLambdust for Vec<T> {
    fn to_lambdust(self) -> Value {
        let values: Vec<Value> = self.into_iter().map(|item| item.to_lambdust()).collect();
        Value::list(values)
    }
}

// Options
impl<T: ToLambdust> ToLambdust for Option<T> {
    fn to_lambdust(self) -> Value {
        match self {
            Some(value) => value.to_lambdust(),
            None => Value::Nil,
        }
    }
}

impl<T: FromLambdust> FromLambdust for Option<T> {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        match value {
            Value::Nil => Ok(None),
            _ => T::from_lambdust(value).map(Some),
        }
    }
    
    fn expected_type() -> &'static str {
        "optional value"
    }
}

// Results
impl<T: ToLambdust, E: ToLambdust> ToLambdust for Result<T, E> {
    fn to_lambdust(self) -> Value {
        match self {
            Ok(value) => {
                // Return a tagged value: (ok value)
                Value::list(vec![
                    Value::symbol(crate::utils::intern_symbol("ok")),
                    value.to_lambdust(),
                ])
            }
            Err(error) => {
                // Return a tagged value: (error error-value)
                Value::list(vec![
                    Value::symbol(crate::utils::intern_symbol("error")),
                    error.to_lambdust(),
                ])
            }
        }
    }
}

// Tuples
impl<A: ToLambdust, B: ToLambdust> ToLambdust for (A, B) {
    fn to_lambdust(self) -> Value {
        Value::pair(self.0.to_lambdust(), self.1.to_lambdust())
    }
}

impl<A: FromLambdust, B: FromLambdust> FromLambdust for (A, B) {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        match value {
            Value::Pair(car, cdr) => {
                let a = A::from_lambdust(car)?;
                let b = B::from_lambdust(cdr)?;
                Ok((a, b))
            }
            _ => Err(FfiError::RuntimeError {
                function: "type_conversion".to_string(),
                message: format!("Expected pair, got {value}"),
            }),
        }
    }
    
    fn expected_type() -> &'static str {
        "pair"
    }
}

// Hash maps (for keyword arguments or association lists)
impl<V: ToLambdust> ToLambdust for HashMap<String, V> {
    fn to_lambdust(self) -> Value {
        let pairs: Vec<Value> = self.into_iter()
            .map(|(k, v)| Value::pair(
                Value::string(k),
                v.to_lambdust()
            ))
            .collect();
        Value::list(pairs)
    }
}

impl<V: FromLambdust> FromLambdust for HashMap<String, V> {
    fn from_lambdust(value: &Value) -> Result<Self, FfiError> {
        let list = value.as_list().ok_or_else(|| FfiError::RuntimeError {
            function: "type_conversion".to_string(),
            message: format!("Expected association list, got {value}"),
        })?;
        
        let mut map = HashMap::new();
        for (i, pair_value) in list.iter().enumerate() {
            match pair_value {
                Value::Pair(car, cdr) => {
                    let key = String::from_lambdust(car)?;
                    let val = V::from_lambdust(cdr)?;
                    map.insert(key, val);
                }
                _ => return Err(FfiError::TypeMismatch {
                    function: "type_conversion".to_string(),
                    parameter: i,
                    expected: "pair".to_string(),
                    actual: format!("{pair_value}"),
                }),
            }
        }
        
        Ok(map)
    }
    
    fn expected_type() -> &'static str {
        "association list"
    }
}

// ============= HELPER FUNCTIONS =============

/// Converts a Rust value to a Lambdust value.
pub fn to_lambdust<T: ToLambdust>(value: T) -> Value {
    value.to_lambdust()
}

/// Converts a Lambdust value to a Rust value.
pub fn from_lambdust<T: FromLambdust>(value: &Value) -> std::result::Result<T, FfiError> {
    T::from_lambdust(value)
}

/// Validates that a Lambdust value can be converted to the given Rust type.
pub fn validate_type<T: FromLambdust>(value: &Value) -> Result<(), FfiError> {
    T::from_lambdust(value).map(|_| ())
}

/// Converts multiple Lambdust values to Rust values.
pub fn from_lambdust_args<T: FromLambdust>(args: &[Value]) -> Result<Vec<T>, FfiError> {
    args.iter()
        .map(|arg| T::from_lambdust(arg))
        .collect()
}

/// Macro for easy implementation of FFI functions with automatic marshaling.
#[macro_export]
macro_rules! ffi_function {
    (
        fn $name:ident($($arg:ident: $arg_type:ty),*) -> $ret_type:ty {
            $($body:tt)*
        }
    ) => {
        pub struct $name;
        
        impl $crate::ffi::FfiFunction for $name {
            fn signature(&self) -> &$crate::ffi::FfiSignature {
                use std::sync::OnceLock;
                static SIGNATURE: OnceLock<$crate::ffi::FfiSignature> = OnceLock::new();
                SIGNATURE.get_or_init(|| {
                    $crate::ffi::FfiSignature {
                        name: stringify!($name).to_string(),
                        arity: $crate::ffi::AritySpec::Exact([$($arg_type,)*].len()),
                        parameter_types: vec![$(
                            <$arg_type as $crate::ffi::FromLambdust>::expected_type().to_string()
                        ),*],
                        return_type: stringify!($ret_type).to_string(),
                        documentation: None,
                    }
                })
            }
            
            fn call(&self, args: &[$crate::eval::Value]) -> Result<$crate::eval::Value, $crate::ffi::FfiError> {
                if args.len() != [$($arg_type,)*].len() {
                    return Err($crate::ffi::FfiError::ArityMismatch {
                        function: stringify!($name).to_string(),
                        expected: $crate::ffi::AritySpec::Exact([$($arg_type,)*].len()),
                        actual: args.len(),
                    });
                }
                
                let mut arg_iter = args.iter();
                $(
                    let $arg = <$arg_type as $crate::ffi::FromLambdust>::from_lambdust(
                        arg_iter.next().unwrap()
                    )?;
                )*
                
                let result: $ret_type = {
                    $($body)*
                };
                
                Ok($crate::ffi::ToLambdust::to_lambdust(result))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_number_conversion() {
        let val = 42.5_f64.to_lambdust();
        assert_eq!(f64::from_lambdust(&val).unwrap(), 42.5);
        
        let int_val = 42_i64.to_lambdust();
        assert_eq!(i64::from_lambdust(&int_val).unwrap(), 42);
    }

    #[test]
    fn test_string_conversion() {
        let val = "hello".to_lambdust();
        assert_eq!(String::from_lambdust(&val).unwrap(), "hello");
    }

    #[test]
    fn test_bool_conversion() {
        let true_val = true.to_lambdust();
        let false_val = false.to_lambdust();
        
        assert!(bool::from_lambdust(&true_val).unwrap());
        assert!(!bool::from_lambdust(&false_val).unwrap());
    }

    #[test]
    fn test_list_conversion() {
        let numbers = vec![1_i64, 2, 3].to_lambdust();
        let converted: Vec<i64> = Vec::from_lambdust(&numbers).unwrap();
        assert_eq!(converted, vec![1, 2, 3]);
    }

    #[test]
    fn test_option_conversion() {
        let some_val = Some(42_i64).to_lambdust();
        let none_val: Option<i64> = None;
        let none_val = none_val.to_lambdust();
        
        assert_eq!(Option::<i64>::from_lambdust(&some_val).unwrap(), Some(42));
        assert_eq!(Option::<i64>::from_lambdust(&none_val).unwrap(), None);
    }

    #[test]
    fn test_tuple_conversion() {
        let tuple = (42_i64, "hello".to_string()).to_lambdust();
        let (a, b): (i64, String) = FromLambdust::from_lambdust(&tuple).unwrap();
        assert_eq!(a, 42);
        assert_eq!(b, "hello");
    }
}