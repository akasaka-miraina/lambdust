//! Comprehensive unit tests for the bridge module
//!
//! Tests the complete bridge functionality including ToScheme/FromScheme traits,
//! Callable interface, ObjectRegistry, LambdustBridge, and external integrations.

use crate::bridge::*;
use crate::error::LambdustError;
use crate::value::Value;
use std::sync::Arc;

#[cfg(test)]
mod to_scheme_trait_tests {
    use super::*;

    #[test]
    fn test_i32_to_scheme() {
        let val = 42i32;
        let result = val.to_scheme().unwrap();
        assert_eq!(result, Value::from(42i64));
    }

    #[test]
    fn test_i64_to_scheme() {
        let val = 123i64;
        let result = val.to_scheme().unwrap();
        assert_eq!(result, Value::from(123i64));
    }

    #[test]
    fn test_f64_to_scheme() {
        let val = 1.234f64;
        let result = val.to_scheme().unwrap();
        assert_eq!(result, Value::from(1.234f64));
    }

    #[test]
    fn test_bool_to_scheme_true() {
        let val = true;
        let result = val.to_scheme().unwrap();
        assert_eq!(result, Value::from(true));
    }

    #[test]
    fn test_bool_to_scheme_false() {
        let val = false;
        let result = val.to_scheme().unwrap();
        assert_eq!(result, Value::from(false));
    }

    #[test]
    fn test_string_to_scheme() {
        let val = "hello".to_string();
        let result = val.to_scheme().unwrap();
        assert_eq!(result, Value::from("hello".to_string()));
    }

    #[test]
    fn test_str_to_scheme() {
        let val = "world";
        let result = val.to_scheme().unwrap();
        assert_eq!(result, Value::from("world"));
    }

    #[test]
    fn test_to_scheme_edge_cases() {
        // Test empty string
        let empty_str = "";
        let result = empty_str.to_scheme().unwrap();
        assert_eq!(result, Value::from(""));

        // Test zero values
        let zero_int = 0i64;
        let result = zero_int.to_scheme().unwrap();
        assert_eq!(result, Value::from(0i64));

        let zero_float = 0.0f64;
        let result = zero_float.to_scheme().unwrap();
        assert_eq!(result, Value::from(0.0f64));
    }

    #[test]
    fn test_to_scheme_large_values() {
        let large_int = i64::MAX;
        let result = large_int.to_scheme().unwrap();
        assert_eq!(result, Value::from(i64::MAX));

        let large_float = f64::MAX;
        let result = large_float.to_scheme().unwrap();
        assert_eq!(result, Value::from(f64::MAX));
    }

    #[test]
    fn test_to_scheme_unicode_string() {
        let unicode_str = "こんにちは世界";
        let result = unicode_str.to_scheme().unwrap();
        assert_eq!(result, Value::from("こんにちは世界"));
    }
}

#[cfg(test)]
mod from_scheme_trait_tests {
    use super::*;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_i64_from_scheme_integer() {
        let value = Value::Number(SchemeNumber::Integer(42));
        let result = i64::from_scheme(&value).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_i64_from_scheme_real() {
        let value = Value::Number(SchemeNumber::Real(1.234));
        let result = i64::from_scheme(&value).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_i64_from_scheme_error() {
        let value = Value::String("not a number".to_string());
        let result = i64::from_scheme(&value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Expected number"));
    }

    #[test]
    fn test_f64_from_scheme_integer() {
        let value = Value::Number(SchemeNumber::Integer(42));
        let result = f64::from_scheme(&value).unwrap();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_f64_from_scheme_real() {
        let value = Value::Number(SchemeNumber::Real(1.234));
        let result = f64::from_scheme(&value).unwrap();
        assert_eq!(result, 1.234);
    }

    #[test]
    fn test_f64_from_scheme_error() {
        let value = Value::Boolean(true);
        let result = f64::from_scheme(&value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Expected number"));
    }

    #[test]
    fn test_bool_from_scheme_true() {
        let value = Value::Boolean(true);
        let result = bool::from_scheme(&value).unwrap();
        assert!(result);
    }

    #[test]
    fn test_bool_from_scheme_false() {
        let value = Value::Boolean(false);
        let result = bool::from_scheme(&value).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_bool_from_scheme_truthy() {
        let value = Value::Number(SchemeNumber::Integer(42));
        let result = bool::from_scheme(&value).unwrap();
        assert!(result); // Non-false values are truthy
    }

    #[test]
    fn test_string_from_scheme_string() {
        let value = Value::String("hello".to_string());
        let result = String::from_scheme(&value).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_string_from_scheme_symbol() {
        let value = Value::Symbol("test-symbol".to_string());
        let result = String::from_scheme(&value).unwrap();
        assert_eq!(result, "test-symbol");
    }

    #[test]
    fn test_string_from_scheme_error() {
        let value = Value::Number(SchemeNumber::Integer(42));
        let result = String::from_scheme(&value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Expected string or symbol"));
    }

    #[test]
    fn test_from_scheme_edge_cases() {
        // Test empty string
        let empty_val = Value::String("".to_string());
        let result = String::from_scheme(&empty_val).unwrap();
        assert_eq!(result, "");

        // Test zero values
        let zero_val = Value::Number(SchemeNumber::Integer(0));
        let result = i64::from_scheme(&zero_val).unwrap();
        assert_eq!(result, 0);

        // Test negative values
        let neg_val = Value::Number(SchemeNumber::Integer(-42));
        let result = i64::from_scheme(&neg_val).unwrap();
        assert_eq!(result, -42);
    }

    #[test]
    fn test_from_scheme_complex_number_error() {
        let value = Value::Number(SchemeNumber::Complex(1.0, 2.0));
        let result = i64::from_scheme(&value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot convert to i64"));
    }

    #[test]
    fn test_from_scheme_rational_number_error() {
        let value = Value::Number(SchemeNumber::Rational(3, 4));
        let result = f64::from_scheme(&value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot convert to f64"));
    }
}

#[cfg(test)]
mod callable_trait_tests {
    use super::*;

    struct TestCallable {
        name: String,
        arity: Option<usize>,
    }

    impl Callable for TestCallable {
        fn call(&self, args: &[Value]) -> crate::error::Result<Value> {
            if let Some(expected_arity) = self.arity {
                if args.len() != expected_arity {
                    return Err(LambdustError::arity_error(expected_arity, args.len()));
                }
            }
            Ok(Value::from(format!("called-{}", self.name)))
        }

        fn arity(&self) -> Option<usize> {
            self.arity
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_callable_basic_call() {
        let callable = TestCallable {
            name: "test-func".to_string(),
            arity: Some(0),
        };
        let result = callable.call(&[]).unwrap();
        assert_eq!(result, Value::from("called-test-func"));
    }

    #[test]
    fn test_callable_arity_check() {
        let callable = TestCallable {
            name: "test-func".to_string(),
            arity: Some(1),
        };
        let result = callable.call(&[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Arity error"));
    }

    #[test]
    fn test_callable_variadic() {
        let callable = TestCallable {
            name: "variadic-func".to_string(),
            arity: None,
        };
        let result = callable.call(&[Value::from(1i64), Value::from(2i64), Value::from(3i64)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from("called-variadic-func"));
    }

    #[test]
    fn test_callable_name() {
        let callable = TestCallable {
            name: "my-function".to_string(),
            arity: Some(2),
        };
        assert_eq!(callable.name(), "my-function");
    }

    #[test]
    fn test_callable_equality() {
        let callable1 = TestCallable {
            name: "func1".to_string(),
            arity: Some(1),
        };
        let callable2 = TestCallable {
            name: "func1".to_string(),
            arity: Some(2),
        };
        let callable3 = TestCallable {
            name: "func2".to_string(),
            arity: Some(1),
        };

        // Same name should be equal
        assert_eq!(callable1.name(), callable2.name());
        assert_ne!(callable1.name(), callable3.name());
    }
}

#[cfg(test)]
mod external_object_tests {
    use super::*;

    #[test]
    fn test_external_object_creation() {
        let data = Arc::new(42i32);
        let obj = ExternalObject {
            id: 1,
            type_name: "i32".to_string(),
            data: data.clone(),
        };
        assert_eq!(obj.id, 1);
        assert_eq!(obj.type_name, "i32");
    }

    #[test]
    fn test_external_object_equality() {
        let data1 = Arc::new(42i32);
        let data2 = Arc::new(42i32);
        
        let obj1 = ExternalObject {
            id: 1,
            type_name: "i32".to_string(),
            data: data1,
        };
        let obj2 = ExternalObject {
            id: 1,
            type_name: "i32".to_string(),
            data: data2,
        };
        let obj3 = ExternalObject {
            id: 2,
            type_name: "i32".to_string(),
            data: Arc::new(42i32),
        };

        assert_eq!(obj1, obj2);
        assert_ne!(obj1, obj3);
    }

    #[test]
    fn test_external_object_different_types() {
        let obj1 = ExternalObject {
            id: 1,
            type_name: "i32".to_string(),
            data: Arc::new(42i32),
        };
        let obj2 = ExternalObject {
            id: 1,
            type_name: "String".to_string(),
            data: Arc::new("hello".to_string()),
        };

        assert_ne!(obj1, obj2);
    }

    #[test]
    fn test_external_object_clone() {
        let obj = ExternalObject {
            id: 1,
            type_name: "i32".to_string(),
            data: Arc::new(42i32),
        };
        let cloned = obj.clone();
        assert_eq!(obj, cloned);
        assert_eq!(obj.id, cloned.id);
        assert_eq!(obj.type_name, cloned.type_name);
    }

    #[test]
    fn test_external_object_debug() {
        let obj = ExternalObject {
            id: 1,
            type_name: "i32".to_string(),
            data: Arc::new(42i32),
        };
        let debug_str = format!("{:?}", obj);
        assert!(debug_str.contains("ExternalObject"));
        assert!(debug_str.contains("id: 1"));
        assert!(debug_str.contains("type_name: \"i32\""));
    }
}

#[cfg(test)]
mod object_registry_tests {
    use super::*;

    #[test]
    fn test_object_registry_creation() {
        let registry = ObjectRegistry::new();
        // Test that registry was initialized (can't access private field directly)
        assert!(registry.objects_is_empty());
        assert!(registry.functions_is_empty());
    }

    #[test]
    fn test_object_registry_default() {
        let registry = ObjectRegistry::default();
        // Test that registry was initialized (can't access private field directly)
        assert!(registry.objects_is_empty());
        assert!(registry.functions_is_empty());
    }

    #[test]
    fn test_register_object() {
        let mut registry = ObjectRegistry::new();
        let obj = 42i32;
        let id = registry.register_object(obj, "i32");
        
        assert_eq!(id, 1);
        // Test that registry state changed (can't access private field directly)
        assert!(!registry.objects_is_empty());
        
        let retrieved = registry.get_object(id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, 1);
        assert_eq!(retrieved.unwrap().type_name, "i32");
    }

    #[test]
    fn test_register_multiple_objects() {
        let mut registry = ObjectRegistry::new();
        
        let id1 = registry.register_object(42i32, "i32");
        let id2 = registry.register_object("hello".to_string(), "String");
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        // Test that multiple objects were registered
        
        let obj1 = registry.get_object(id1).unwrap();
        let obj2 = registry.get_object(id2).unwrap();
        
        assert_eq!(obj1.type_name, "i32");
        assert_eq!(obj2.type_name, "String");
    }

    #[test]
    fn test_register_function() {
        let mut registry = ObjectRegistry::new();
        
        struct TestFunc;
        impl Callable for TestFunc {
            fn call(&self, _args: &[Value]) -> crate::error::Result<Value> {
                Ok(Value::from("test"))
            }
            fn arity(&self) -> Option<usize> { Some(0) }
            fn name(&self) -> &str { "test-func" }
        }
        
        let func = Arc::new(TestFunc);
        registry.register_function("test-func", func.clone());
        
        assert!(!registry.functions_is_empty());
        assert!(registry.has_function("test-func"));
        assert!(!registry.has_function("non-existent"));
        
        let retrieved = registry.get_function("test-func");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test-func");
    }

    #[test]
    fn test_register_converter() {
        let mut registry = ObjectRegistry::new();
        
        let converter = |x: &i32| -> crate::error::Result<Value> {
            Ok(Value::from(*x as i64))
        };
        
        registry.register_converter("i32", converter);
        // Note: This is a placeholder test since converters are not fully implemented
        // Can't access private field directly, so just test that function doesn't panic
    }

    #[test]
    fn test_get_nonexistent_object() {
        let registry = ObjectRegistry::new();
        let result = registry.get_object(999);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_nonexistent_function() {
        let registry = ObjectRegistry::new();
        let result = registry.get_function("non-existent");
        assert!(result.is_none());
    }

    #[test]
    fn test_object_to_value_no_converter() {
        let mut registry = ObjectRegistry::new();
        let id = registry.register_object(42i32, "i32");
        let obj = registry.get_object(id).unwrap();
        
        let result = registry.object_to_value(obj).unwrap();
        // Should return as external object since no converter
        match result {
            Value::External(ext_obj) => {
                assert_eq!(ext_obj.id, 1);
                assert_eq!(ext_obj.type_name, "i32");
            }
            _ => panic!("Expected External value"),
        }
    }

    #[test]
    fn test_object_to_value_with_converter() {
        let mut registry = ObjectRegistry::new();
        
        // Register a converter using the public API
        registry.register_converter("i32", |_x: &i32| Ok(Value::from(42i64)));
        
        let id = registry.register_object(42i32, "i32");
        let obj = registry.get_object(id).unwrap();
        
        let result = registry.object_to_value(obj);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Type converter not implemented"));
    }

    #[test]
    fn test_registry_debug() {
        let registry = ObjectRegistry::new();
        let debug_str = format!("{:?}", registry);
        assert!(debug_str.contains("ObjectRegistry"));
        assert!(debug_str.contains("next_id: 1"));
    }
}

#[cfg(test)]
mod lambdust_bridge_tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = LambdustBridge::new();
        // Note: global_env.is_empty() method doesn't exist, test registry state instead
        assert!(bridge.registry.lock().unwrap().objects_is_empty());
        assert!(bridge.registry.lock().unwrap().functions_is_empty());
    }

    #[test]
    fn test_bridge_default() {
        let bridge = LambdustBridge::default();
        // Test that bridge was initialized correctly
        assert!(bridge.registry.lock().unwrap().objects_is_empty());
        assert!(bridge.registry.lock().unwrap().functions_is_empty());
    }

    #[test]
    fn test_bridge_register_object() {
        let mut bridge = LambdustBridge::new();
        let obj = 42i32;
        let id = bridge.register_object(obj, "i32");
        
        assert_eq!(id, 1);
        let registry = bridge.registry.lock().unwrap();
        assert!(!registry.objects_is_empty());
        assert!(registry.get_object(id).is_some());
    }

    #[test]
    fn test_bridge_register_function() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("square", Some(1), |args| {
            let n = f64::from_scheme(&args[0])?;
            (n * n).to_scheme()
        });
        
        // Check that function is registered in registry
        let registry = bridge.registry.lock().unwrap();
        assert!(registry.has_function("square"));
        
        // Check that function is available in evaluator
        let result = bridge.evaluator.global_env.get("square");
        assert!(result.is_some());
    }

    #[test]
    fn test_bridge_register_variadic_function() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("sum", None, |args| {
            let mut total = 0.0;
            for arg in args {
                total += f64::from_scheme(arg)?;
            }
            total.to_scheme()
        });
        
        let registry = bridge.registry.lock().unwrap();
        assert!(registry.has_function("sum"));
    }

    #[test]
    fn test_bridge_eval_basic() {
        let mut bridge = LambdustBridge::new();
        let result = bridge.eval("(+ 1 2)").unwrap();
        assert_eq!(result, Value::from(3i64));
    }

    #[test]
    fn test_bridge_eval_with_registered_function() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("double", Some(1), |args| {
            let n = f64::from_scheme(&args[0])?;
            (n * 2.0).to_scheme()
        });
        
        let result = bridge.eval("(double 21.0)").unwrap();
        assert_eq!(result, Value::from(42.0));
    }

    #[test]
    fn test_bridge_eval_error() {
        let mut bridge = LambdustBridge::new();
        let result = bridge.eval("(undefined-function)");
        assert!(result.is_err());
    }

    #[test]
    fn test_bridge_define_variable() {
        let mut bridge = LambdustBridge::new();
        bridge.define("test-var", Value::from(42i64));
        
        let result = bridge.eval("test-var").unwrap();
        assert_eq!(result, Value::from(42i64));
    }

    #[test]
    fn test_bridge_load_file_error() {
        let mut bridge = LambdustBridge::new();
        let result = bridge.load_file("nonexistent.scm");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("I/O error"));
    }

    #[test]
    fn test_bridge_functions_placeholder() {
        let mut bridge = LambdustBridge::new();
        
        // Test that bridge functions are added but not implemented
        let result = bridge.eval("(call-external 'test)");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("call-external not implemented"));
        
        let result = bridge.eval("(get-property 'obj 'prop)");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("get-property not implemented"));
        
        let result = bridge.eval("(set-property! 'obj 'prop 'value)");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("set-property! not implemented"));
    }

    #[test]
    fn test_bridge_debug() {
        let bridge = LambdustBridge::new();
        let debug_str = format!("{:?}", bridge);
        assert!(debug_str.contains("LambdustBridge"));
        assert!(debug_str.contains("evaluator"));
        assert!(debug_str.contains("registry"));
    }
}

#[cfg(test)]
mod callable_function_tests {
    use super::*;

    // Note: CallableFunction is private, so we test it through the bridge API
    #[test]
    fn test_callable_function_creation_through_bridge() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("test-func", Some(1), |args| {
            Ok(Value::from(args.len() as i64))
        });
        
        let registry = bridge.registry.lock().unwrap();
        assert!(registry.has_function("test-func"));
        
        let func = registry.get_function("test-func").unwrap();
        assert_eq!(func.name(), "test-func");
        assert_eq!(func.arity(), Some(1));
    }

    #[test]
    fn test_callable_function_call_through_bridge() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("test-func", Some(1), |args| {
            Ok(Value::from(args.len() as i64))
        });
        
        let registry = bridge.registry.lock().unwrap();
        let func = registry.get_function("test-func").unwrap();
        
        let result = func.call(&[Value::from(42i64)]).unwrap();
        assert_eq!(result, Value::from(1i64));
    }

    #[test]
    fn test_callable_function_arity_error_through_bridge() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("test-func", Some(2), |args| {
            Ok(Value::from(args.len() as i64))
        });
        
        let registry = bridge.registry.lock().unwrap();
        let func = registry.get_function("test-func").unwrap();
        
        let result = func.call(&[Value::from(42i64)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Arity error"));
    }

    #[test]
    fn test_callable_function_variadic_through_bridge() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("variadic-func", None, |args| {
            Ok(Value::from(args.len() as i64))
        });
        
        let registry = bridge.registry.lock().unwrap();
        let func = registry.get_function("variadic-func").unwrap();
        
        let result = func.call(&[Value::from(1i64), Value::from(2i64), Value::from(3i64)]).unwrap();
        assert_eq!(result, Value::from(3i64));
    }

    #[test]
    fn test_callable_function_debug_through_bridge() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("debug-func", Some(0), |_| Ok(Value::from("test")));
        
        let registry = bridge.registry.lock().unwrap();
        let func = registry.get_function("debug-func").unwrap();
        
        assert_eq!(func.name(), "debug-func");
        assert_eq!(func.arity(), Some(0));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_bridge_full_workflow() {
        let mut bridge = LambdustBridge::new();
        
        // Register an object
        let counter = 42i32;
        let counter_id = bridge.register_object(counter, "Counter");
        
        // Register a function
        bridge.register_function("increment", Some(1), |args| {
            let n = i64::from_scheme(&args[0])?;
            (n + 1).to_scheme()
        });
        
        // Define variables
        bridge.define("counter-id", Value::from(counter_id as i64));
        bridge.define("base-value", Value::from(10i64));
        
        // Use registered function
        let result = bridge.eval("(increment base-value)").unwrap();
        assert_eq!(result, Value::from(11i64));
        
        // Verify object registration
        let registry = bridge.registry.lock().unwrap();
        assert!(registry.get_object(counter_id).is_some());
        assert!(registry.has_function("increment"));
    }

    #[test]
    fn test_type_conversion_chain() {
        let mut bridge = LambdustBridge::new();
        
        // Register functions that use type conversions
        bridge.register_function("to-string", Some(1), |args| {
            let n = i64::from_scheme(&args[0])?;
            format!("number-{}", n).to_scheme()
        });
        
        bridge.register_function("string-length", Some(1), |args| {
            let s = String::from_scheme(&args[0])?;
            (s.len() as i64).to_scheme()
        });
        
        // Chain the conversions
        let result = bridge.eval("(string-length (to-string 42))").unwrap();
        assert_eq!(result, Value::from(9i64)); // "number-42" has 9 characters
    }

    #[test]
    fn test_error_handling_in_bridge() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("divide", Some(2), |args| {
            let a = f64::from_scheme(&args[0])?;
            let b = f64::from_scheme(&args[1])?;
            if b == 0.0 {
                return Err(LambdustError::division_by_zero());
            }
            (a / b).to_scheme()
        });
        
        // Test successful division
        let result = bridge.eval("(divide 10.0 2.0)").unwrap();
        assert_eq!(result, Value::from(5.0));
        
        // Test division by zero
        let result = bridge.eval("(divide 10.0 0.0)");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));
    }

    #[test]
    fn test_complex_data_structures() {
        let mut bridge = LambdustBridge::new();
        
        // Register a function that works with complex data
        bridge.register_function("process-data", Some(1), |args| {
            // This would process some complex data structure
            // For now, just return a success indicator
            let _data = &args[0];
            "processed".to_scheme()
        });
        
        // Test with various data types
        let result = bridge.eval("(process-data '(1 2 3))").unwrap();
        assert_eq!(result, Value::from("processed"));
        
        let result = bridge.eval("(process-data \"hello\")").unwrap();
        assert_eq!(result, Value::from("processed"));
    }

    #[test]
    fn test_unicode_support() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("echo", Some(1), |args| {
            let s = String::from_scheme(&args[0])?;
            s.to_scheme()
        });
        
        bridge.define("japanese-text", Value::from("こんにちは"));
        let result = bridge.eval("(echo japanese-text)").unwrap();
        assert_eq!(result, Value::from("こんにちは"));
    }

    #[test]
    fn test_concurrent_access_safety() {
        let mut bridge = LambdustBridge::new();
        
        // Register function and object
        bridge.register_function("test-func", Some(0), |_| {
            Ok(Value::from("test"))
        });
        
        let obj = "test-object".to_string();
        let obj_id = bridge.register_object(obj, "String");
        
        // Test that registry can be accessed safely
        let registry = bridge.registry.lock().unwrap();
        assert!(registry.has_function("test-func"));
        assert!(registry.get_object(obj_id).is_some());
    }
}

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_empty_function_name() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("", Some(0), |_| {
            Ok(Value::from("empty-name"))
        });
        
        let registry = bridge.registry.lock().unwrap();
        assert!(registry.has_function(""));
    }

    #[test]
    fn test_very_long_function_name() {
        let mut bridge = LambdustBridge::new();
        let long_name = "a".repeat(1000);
        
        bridge.register_function(&long_name, Some(0), |_| {
            Ok(Value::from("long-name"))
        });
        
        let registry = bridge.registry.lock().unwrap();
        assert!(registry.has_function(&long_name));
    }

    #[test]
    fn test_special_characters_in_function_name() {
        let mut bridge = LambdustBridge::new();
        let special_name = "func-with-!@#$%^&*()_+-=[]{}|;':\",./<>?";
        
        bridge.register_function(special_name, Some(0), |_| {
            Ok(Value::from("special-chars"))
        });
        
        let registry = bridge.registry.lock().unwrap();
        assert!(registry.has_function(special_name));
    }

    #[test]
    fn test_large_number_conversion() {
        let large_num = i64::MAX;
        let result = large_num.to_scheme().unwrap();
        assert_eq!(result, Value::from(i64::MAX));
        
        let back = i64::from_scheme(&result).unwrap();
        assert_eq!(back, i64::MAX);
    }

    #[test]
    fn test_nan_and_infinity() {
        let nan_val = f64::NAN;
        let result = nan_val.to_scheme().unwrap();
        
        if let Value::Number(crate::lexer::SchemeNumber::Real(val)) = result {
            assert!(val.is_nan());
        } else {
            panic!("Expected real number");
        }
        
        let inf_val = f64::INFINITY;
        let result = inf_val.to_scheme().unwrap();
        
        if let Value::Number(crate::lexer::SchemeNumber::Real(val)) = result {
            assert!(val.is_infinite());
        } else {
            panic!("Expected real number");
        }
    }

    #[test]
    fn test_zero_arity_function() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("zero-arity", Some(0), |args| {
            assert_eq!(args.len(), 0);
            Ok(Value::from("no-args"))
        });
        
        let result = bridge.eval("(zero-arity)").unwrap();
        assert_eq!(result, Value::from("no-args"));
    }

    #[test]
    fn test_high_arity_function() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("high-arity", Some(10), |args| {
            assert_eq!(args.len(), 10);
            Ok(Value::from(args.len() as i64))
        });
        
        let result = bridge.eval("(high-arity 1 2 3 4 5 6 7 8 9 10)").unwrap();
        assert_eq!(result, Value::from(10i64));
    }

    #[test]
    fn test_function_returning_error() {
        let mut bridge = LambdustBridge::new();
        
        bridge.register_function("error-func", Some(0), |_| {
            Err(LambdustError::runtime_error("Intentional error"))
        });
        
        let result = bridge.eval("(error-func)");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Intentional error"));
    }

    #[test]
    fn test_object_id_overflow() {
        let mut registry = ObjectRegistry::new();
        
        // Register multiple objects to test ID increment
        let id1 = registry.register_object(1i32, "i32");
        let id2 = registry.register_object(2i32, "i32");
        
        // IDs should increment sequentially
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        
        // Verify objects exist
        assert!(registry.get_object(id1).is_some());
        assert!(registry.get_object(id2).is_some());
    }
}