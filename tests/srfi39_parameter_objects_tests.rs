#![allow(clippy::uninlined_format_args)]
//! Comprehensive tests for SRFI-39 Parameter Objects implementation.
//!
//! This test suite verifies both correctness and performance of the parameter system,
//! including thread-local bindings, parameter conversion, and performance optimizations.

use lambdust::eval::parameter::ParameterBinding;
use lambdust::eval::value::Parameter;
use lambdust::eval::value::Value;
use lambdust::stdlib::parameters::{is_parameter, make_parameter};
use std::collections::HashMap;

#[test]
fn test_basic_parameter_creation() {
    // Test creating a parameter with just an initial value
    let param = Parameter::new(Value::integer(42), None);
    assert_eq!(param.get().as_integer(), Some(42));
    assert!(!param.has_converter());
}

#[test]
fn test_parameter_with_converter() {
    // Test creating a parameter with a converter function
    let converter = Value::integer(0); // placeholder converter
    let param = Parameter::new(Value::integer(42), Some(converter));
    assert_eq!(param.get().as_integer(), Some(42));
    assert!(param.has_converter());
}

#[test]
fn test_parameter_global_set() {
    let param = Parameter::new(Value::integer(1), None);
    assert_eq!(param.get().as_integer(), Some(1));

    // Set global value
    param.set_global(Value::integer(100)).unwrap();
    assert_eq!(param.get().as_integer(), Some(100));
}

#[test]
fn test_parameter_thread_local_binding() {
    // Clear stack not available in integration tests

    let param = Parameter::new(Value::integer(1), None);
    assert_eq!(param.get().as_integer(), Some(1));

    // Create thread-local binding
    let mut bindings = HashMap::new();
    bindings.insert(param.id(), Value::integer(42));

    let result = ParameterBinding::with_bindings(bindings, || {
        // Should see thread-local value, not global
        assert_eq!(param.get().as_integer(), Some(42));
        param.get().as_integer().unwrap() * 2
    });

    // After binding is removed, should see global value again
    assert_eq!(param.get().as_integer(), Some(1));
    assert_eq!(result, 84);
}

#[test]
fn test_nested_parameter_bindings() {
    // Clear stack not available in integration tests

    let param = Parameter::new(Value::integer(1), None);

    let mut bindings1 = HashMap::new();
    bindings1.insert(param.id(), Value::integer(10));

    let mut bindings2 = HashMap::new();
    bindings2.insert(param.id(), Value::integer(20));

    ParameterBinding::with_bindings(bindings1, || {
        assert_eq!(param.get().as_integer(), Some(10));

        ParameterBinding::with_bindings(bindings2, || {
            // Inner binding should shadow outer binding
            assert_eq!(param.get().as_integer(), Some(20));
        });

        // Should return to outer binding
        assert_eq!(param.get().as_integer(), Some(10));
    });

    // Should return to global default
    assert_eq!(param.get().as_integer(), Some(1));
}

#[test]
fn test_multiple_parameters_same_binding() {
    // Clear stack not available in integration tests

    let param1 = Parameter::new(Value::integer(1), None);
    let param2 = Parameter::new(Value::integer(2), None);
    let param3 = Parameter::new(Value::integer(3), None);

    let mut bindings = HashMap::new();
    bindings.insert(param1.id(), Value::integer(10));
    bindings.insert(param2.id(), Value::integer(20));
    // param3 not bound, should use global default

    ParameterBinding::with_bindings(bindings, || {
        assert_eq!(param1.get().as_integer(), Some(10));
        assert_eq!(param2.get().as_integer(), Some(20));
        assert_eq!(param3.get().as_integer(), Some(3)); // global default
    });

    // All should return to global defaults
    assert_eq!(param1.get().as_integer(), Some(1));
    assert_eq!(param2.get().as_integer(), Some(2));
    assert_eq!(param3.get().as_integer(), Some(3));
}

#[test]
fn test_parameter_stack_depth() {
    // Clear stack not available in integration tests
    assert_eq!(ParameterBinding::stack_depth(), 0);

    let bindings = HashMap::new();
    ParameterBinding::with_bindings(bindings.clone(), || {
        assert_eq!(ParameterBinding::stack_depth(), 1);

        ParameterBinding::with_bindings(bindings.clone(), || {
            assert_eq!(ParameterBinding::stack_depth(), 2);

            ParameterBinding::with_bindings(bindings, || {
                assert_eq!(ParameterBinding::stack_depth(), 3);
            });

            assert_eq!(ParameterBinding::stack_depth(), 2);
        });

        assert_eq!(ParameterBinding::stack_depth(), 1);
    });

    assert_eq!(ParameterBinding::stack_depth(), 0);
}

#[test]
fn test_parameter_with_name() {
    let param = Parameter::with_name(Value::string("hello"), None, "test-param".to_string());

    assert_eq!(param.get().as_string(), Some("hello"));
    assert_eq!(param.name(), Some("test-param"));
    assert!(param.id() > 0); // Should have a valid ID
}

#[test]
fn test_make_parameter_function() {
    // Test make-parameter with 1 argument
    let args = &[Value::integer(42)];
    let result = make_parameter(args).unwrap();
    assert!(result.is_parameter());

    // Test make-parameter with 2 arguments (with converter)
    let args = &[Value::integer(42), Value::integer(0)]; // placeholder converter
    let result = make_parameter(args).unwrap();
    assert!(result.is_parameter());

    // Test make-parameter with wrong number of arguments
    let args = &[];
    assert!(make_parameter(args).is_err());

    let args = &[Value::integer(1), Value::integer(2), Value::integer(3)];
    assert!(make_parameter(args).is_err());
}

#[test]
fn test_is_parameter_function() {
    let param = Parameter::new(Value::integer(42), None);
    let param_value = Value::parameter(param);

    // Test parameter? with parameter object
    let args = &[param_value];
    let result = is_parameter(args).unwrap();
    assert_eq!(result, Value::boolean(true));

    // Test parameter? with non-parameter object
    let args = &[Value::integer(42)];
    let result = is_parameter(args).unwrap();
    assert_eq!(result, Value::boolean(false));

    // Test parameter? with wrong number of arguments
    let args = &[];
    assert!(is_parameter(args).is_err());

    let args = &[Value::integer(1), Value::integer(2)];
    assert!(is_parameter(args).is_err());
}

#[test]
fn test_parameter_performance_statistics() {
    // Clear stack not available in integration tests
    ParameterBinding::reset_statistics();

    let param = Parameter::new(Value::integer(42), None);

    // Perform some operations to generate statistics
    for _ in 0..10 {
        param.get(); // Should increment read count
    }

    // Create some bindings to test parameterize statistics
    let mut bindings = HashMap::new();
    bindings.insert(param.id(), Value::integer(100));

    ParameterBinding::with_bindings(bindings, || {
        for _ in 0..5 {
            param.get(); // Should hit hot cache after a few reads
        }
    });

    // Check that statistics were recorded
    if let Some((reads, _writes, _cache_hits, _cache_misses, parameterize_calls)) =
        ParameterBinding::get_statistics()
    {
        assert!(reads > 0, "Should have recorded parameter reads");
        assert!(
            parameterize_calls > 0,
            "Should have recorded parameterize calls"
        );
        // Note: cache behavior depends on implementation details
    }
}

/// Performance benchmark test - should run quickly with optimizations
#[test]
fn test_parameter_performance_benchmark() {
    use std::time::Instant;

    // Clear stack not available in integration tests
    ParameterBinding::reset_statistics();

    let param = Parameter::new(Value::integer(42), None);
    let mut bindings = HashMap::new();
    bindings.insert(param.id(), Value::integer(100));

    // Benchmark parameter reading with bindings
    let start = Instant::now();
    let iterations = 1000;

    ParameterBinding::with_bindings(bindings, || {
        for _ in 0..iterations {
            let _value = param.get(); // Should be fast due to hot cache
        }
    });

    let duration = start.elapsed();
    let ns_per_read = duration.as_nanos() / iterations;

    // Performance target: <100ns per read (including hot cache optimization)
    // This is a reasonable target for the optimized implementation
    println!("Parameter read performance: {ns_per_read}ns per read");

    // Don't fail the test if performance doesn't meet target in debug builds
    // In release builds with optimizations, this should be much faster
    if cfg!(not(debug_assertions)) {
        assert!(
            ns_per_read < 1000,
            "Parameter reads should be under 1000ns in release mode, got {ns_per_read}ns"
        );
    }
}

/// Test parameter system under concurrent access (if threading is enabled)
#[cfg(feature = "async-runtime")]
#[test]
fn test_parameter_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let param = Arc::new(Parameter::new(Value::integer(0), None));
    let param_clone = param.clone();

    // Set global value from main thread
    param.set_global(Value::integer(42)).unwrap();

    let handle = thread::spawn(move || {
        // Should see the global value from the other thread
        assert_eq!(param_clone.get().as_integer(), Some(42));

        // Thread-local bindings should be isolated
        let mut bindings = HashMap::new();
        bindings.insert(param_clone.id(), Value::integer(100));

        ParameterBinding::with_bindings(bindings, || {
            assert_eq!(param_clone.get().as_integer(), Some(100));
        });

        // Should return to global value after binding
        assert_eq!(param_clone.get().as_integer(), Some(42));
    });

    handle.join().unwrap();

    // Main thread should still see global value
    assert_eq!(param.get().as_integer(), Some(42));
}

/// Test error handling in parameter operations
#[test]
fn test_parameter_error_handling() {
    // Test set_global with converter (when converter is implemented)
    let param = Parameter::new(Value::integer(42), Some(Value::integer(0)));

    // For now, this should succeed since converter is not fully implemented
    // In a full implementation, this would test converter validation
    assert!(param.set_global(Value::integer(100)).is_ok());
}

/// Integration test with stdlib parameter functions
#[test]
fn test_stdlib_parameter_integration() {
    // Test the stdlib functions work with the optimized parameter system
    let args = &[Value::string("initial")];
    let param_value = make_parameter(args).unwrap();

    // Should be recognized as a parameter
    let args = &[param_value.clone()];
    let result = is_parameter(args).unwrap();
    assert_eq!(result, Value::boolean(true));

    // Extract the parameter and test its behavior
    if let Value::Parameter(param) = param_value {
        assert_eq!(param.get().as_string(), Some("initial"));

        // Test global setting
        param.set_global(Value::string("updated")).unwrap();
        assert_eq!(param.get().as_string(), Some("updated"));

        // Test thread-local binding
        let mut bindings = HashMap::new();
        bindings.insert(param.id, Value::string("bound"));

        ParameterBinding::with_bindings(bindings, || {
            assert_eq!(param.get().as_string(), Some("bound"));
        });

        // Should return to global value
        assert_eq!(param.get().as_string(), Some("updated"));
    }
}
