//! Function registry utilities and helper types.
//!
//! This module provides additional utilities for the FFI registry,
//! including helper macros and registration patterns.

use super::*;
// use std::sync::OnceLock;

/// Macro for defining FFI functions with documentation.
#[macro_export]
macro_rules! define_ffi_function {
    (
        $(#[doc = $doc:literal])*
        pub struct $name:ident;
        
        fn $fn_name:ident($($arg:ident: $arg_type:ty),*) -> $ret_type:ty {
            $($body:tt)*
        }
    ) => {
        $(#[doc = $doc])*
        pub struct $name;
        
        impl $crate::ffi::FfiFunction for $name {
            fn signature(&self) -> &$crate::ffi::FfiSignature {
                static SIGNATURE: std::sync::OnceLock<$crate::ffi::FfiSignature> = std::sync::OnceLock::new();
                SIGNATURE.get_or_init(|| {
                    let docs = vec![$($doc,)*].join("\n");
                    $crate::ffi::FfiSignature {
                        name: stringify!($fn_name).replace('_', "-").to_string(),
                        arity: $crate::ffi::AritySpec::Exact({
                            #[allow(unused)]
                            let args: &[&str] = &[$(stringify!($arg),)*];
                            args.len()
                        }),
                        parameter_types: vec![$(
                            <$arg_type as $crate::ffi::FromLambdust>::expected_type().to_string()
                        ),*],
                        return_type: stringify!($ret_type).to_string(),
                        documentation: if docs.is_empty() { None } else { Some(docs) },
                    }
                })
            }
            
            fn call(&self, args: &[$crate::eval::Value]) -> std::result::Result<$crate::eval::Value, $crate::ffi::FfiError> {
                let expected_count = {
                    #[allow(unused)]
                    let args: &[&str] = &[$(stringify!($arg),)*];
                    args.len()
                };
                
                if args.len() != expected_count {
                    return Err($crate::ffi::FfiError::ArityMismatch {
                        function: self.signature().name.clone(),
                        expected: $crate::ffi::AritySpec::Exact(expected_count),
                        actual: args.len(),
                    });
                }
                
                let mut arg_iter = args.iter();
                $(
                    let $arg = <$arg_type as $crate::ffi::FromLambdust>::from_lambdust(
                        arg_iter.next().unwrap()
                    ).map_err(|_| $crate::ffi::FfiError::TypeMismatch {
                        function: self.signature().name.clone(),
                        parameter: 0, // Simplified for now
                        expected: <$arg_type as $crate::ffi::FromLambdust>::expected_type().to_string(),
                        actual: format!("{:?}", arg_iter.next().unwrap()),
                    })?;
                )*
                
                let result: $ret_type = {
                    $($body)*
                };
                
                Ok($crate::ffi::ToLambdust::to_lambdust(result))
            }
        }
    };
}

/// Macro for defining variadic FFI functions.
#[macro_export]
macro_rules! define_variadic_ffi_function {
    (
        $(#[doc = $doc:literal])*
        pub struct $name:ident;
        
        fn $fn_name:ident($args:ident: &[Value]) -> $ret_type:ty {
            $($body:tt)*
        }
    ) => {
        $(#[doc = $doc])*
        pub struct $name;
        
        impl $crate::ffi::FfiFunction for $name {
            fn signature(&self) -> &$crate::ffi::FfiSignature {
                static SIGNATURE: std::sync::OnceLock<$crate::ffi::FfiSignature> = std::sync::OnceLock::new();
                SIGNATURE.get_or_init(|| {
                    let docs = vec![$($doc,)*].join("\n");
                    $crate::ffi::FfiSignature {
                        name: stringify!($fn_name).replace('_', "-").to_string(),
                        arity: $crate::ffi::AritySpec::AtLeast(0),
                        parameter_types: vec!["any".to_string()],
                        return_type: stringify!($ret_type).to_string(),
                        documentation: if docs.is_empty() { None } else { Some(docs) },
                    }
                })
            }
            
            fn call(&self, $args: &[$crate::eval::Value]) -> std::result::Result<$crate::eval::Value, $crate::ffi::FfiError> {
                let result: $ret_type = {
                    $($body)*
                };
                
                Ok($crate::ffi::ToLambdust::to_lambdust(result))
            }
        }
    };
}

/// Helper for registering multiple functions at once.
pub struct RegistrationBuilder {
    registry: std::sync::Arc<FfiRegistry>,
}

impl RegistrationBuilder {
    /// Create a new registration builder.
    pub fn new(registry: std::sync::Arc<FfiRegistry>) -> Self {
        Self { registry }
    }
    
    /// Register a function.
    pub fn register<F>(self, function: F) -> Self
    where
        F: FfiFunction + 'static,
    {
        if let Err(e) = self.registry.register(function) {
            eprintln!("Warning: Failed to register FFI function: {e}");
        }
        self
    }
    
    /// Finish registration and return the registry.
    pub fn build(self) -> std::sync::Arc<FfiRegistry> {
        self.registry
    }
}

/// Extension trait for easier registry building.
pub trait FfiRegistryExt {
    /// Create a registration builder for this registry.
    fn builder(self) -> RegistrationBuilder;
}

impl FfiRegistryExt for std::sync::Arc<FfiRegistry> {
    fn builder(self) -> RegistrationBuilder {
        RegistrationBuilder::new(self)
    }
}

/// Trait for modules that can register their FFI functions.
pub trait FfiModule {
    /// Register all functions from this module into the given registry.
    fn register_functions(registry: &FfiRegistry) -> std::result::Result<(), FfiError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Value;

    define_ffi_function! {
        /// Test function that adds two numbers.
        pub struct TestAddFunction;
        
        fn test_add(a: f64, b: f64) -> f64 {
            a + b
        }
    }

    #[test]
    fn test_function_definition() {
        let func = TestAddFunction;
        let sig = func.signature();
        
        assert_eq!(sig.name, "test-add");
        assert_eq!(sig.arity, AritySpec::Exact(2));
        
        let args = vec![Value::number(2.0), Value::number(3.0)];
        let result = func.call(&args).unwrap();
        
        assert_eq!(result.as_number().unwrap(), 5.0);
    }

    define_variadic_ffi_function! {
        /// Test variadic function that sums all arguments.
        pub struct TestSumFunction;
        
        fn test_sum(args: &[Value]) -> f64 {
            args.iter()
                .filter_map(|v| v.as_number())
                .sum()
        }
    }

    #[test]
    fn test_variadic_function() {
        let func = TestSumFunction;
        let sig = func.signature();
        
        assert_eq!(sig.name, "test-sum");
        assert_eq!(sig.arity, AritySpec::AtLeast(0));
        
        let args = vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)];
        let result = func.call(&args).unwrap();
        
        assert_eq!(result.as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_registration_builder() {
        let registry = std::sync::Arc::new(FfiRegistry::new());
        let registry = registry
            .builder()
            .register(TestAddFunction)
            .register(TestSumFunction)
            .build();
        
        let functions = registry.list_functions();
        assert!(functions.contains(&"test-add".to_string()));
        assert!(functions.contains(&"test-sum".to_string()));
    }
}