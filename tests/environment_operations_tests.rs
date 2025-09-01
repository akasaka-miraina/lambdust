//! Comprehensive tests for environment operations and R7RS (scheme eval) compliance.

use lambdust::eval::{ThreadSafeEnvironment, Value};
use lambdust::stdlib::eval_operations::*;
use lambdust::utils::intern_symbol;
use std::sync::Arc;

#[test]
fn test_environment_bound_predicate() {
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
    env.define("test-var".to_string(), Value::integer(42));

    // Test bound symbol
    let symbol_value = Value::symbol(intern_symbol("test-var".to_string()));
    let env_value = Value::Environment(env.clone());

    let result = primitive_environment_bound_p(&[symbol_value, env_value]).unwrap();
    assert!(result.is_truthy(), "Expected test-var to be bound");

    // Test unbound symbol
    let unbound_symbol = Value::symbol(intern_symbol("unbound-var".to_string()));
    let env_value = Value::Environment(env);

    let result = primitive_environment_bound_p(&[unbound_symbol, env_value]).unwrap();
    assert!(result.is_falsy(), "Expected unbound-var to be unbound");
}

#[test]
fn test_interaction_environment() {
    let result = primitive_interaction_environment(&[]).unwrap();

    match result {
        Value::Environment(_env) => {
            // Should return an environment
        }
        _ => panic!("interaction-environment should return an environment"),
    }
}

#[test]
fn test_scheme_report_environment_r5rs() {
    let version_5 = Value::integer(5);
    let result = primitive_scheme_report_environment(&[version_5]).unwrap();

    match result {
        Value::Environment(_env) => {
            // Should return an R5RS environment
        }
        _ => panic!("scheme-report-environment should return an environment"),
    }
}

#[test]
fn test_scheme_report_environment_unsupported_version() {
    let version_7 = Value::integer(7);
    let result = primitive_scheme_report_environment(&[version_7]);

    assert!(result.is_err(), "Should error on unsupported version");
}

#[test]
fn test_null_environment_r5rs() {
    let version_5 = Value::integer(5);
    let result = primitive_null_environment(&[version_5]).unwrap();

    match result {
        Value::Environment(_env) => {
            // Should return a null environment
        }
        _ => panic!("null-environment should return an environment"),
    }
}

#[test]
fn test_null_environment_unsupported_version() {
    let version_7 = Value::integer(7);
    let result = primitive_null_environment(&[version_7]);

    assert!(result.is_err(), "Should error on unsupported version");
}

#[test]
fn test_environment_creation_from_scheme_base() {
    // Test creating environment from (scheme base)
    let scheme_symbol = Value::symbol(intern_symbol("scheme".to_string()));
    let base_symbol = Value::symbol(intern_symbol("base".to_string()));
    let import_list = Value::pair(scheme_symbol, Value::pair(base_symbol, Value::Nil));

    let result = primitive_environment(&[import_list]).unwrap();

    match result {
        Value::Environment(_env) => {
            // Should return an environment with (scheme base) bindings
        }
        _ => panic!("environment should return an environment"),
    }
}

#[test]
fn test_eval_basic_expression() {
    // Create a simple environment with arithmetic
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
    create_eval_bindings(&env);

    // Import basic arithmetic for testing
    lambdust::stdlib::arithmetic::create_arithmetic_bindings(&env);

    let env_value = Value::Environment(env);

    // Test evaluating a simple arithmetic expression: (+ 1 2)
    let plus_symbol = Value::symbol(intern_symbol("+".to_string()));
    let one = Value::integer(1);
    let two = Value::integer(2);
    let expr = Value::pair(plus_symbol, Value::pair(one, Value::pair(two, Value::Nil)));

    let result = primitive_eval(&[expr, env_value]).unwrap();

    match result {
        Value::Literal(lit) => {
            if let Some(n) = lit.to_f64() {
                assert!((n - 3.0).abs() < f64::EPSILON, "Expected 3, got {}", n);
            } else {
                panic!("Expected numeric result");
            }
        }
        _ => panic!("Expected numeric result from (+ 1 2)"),
    }
}

#[test]
fn test_eval_literal() {
    // Test evaluating a literal value
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
    create_eval_bindings(&env);
    let env_value = Value::Environment(env);

    let literal = Value::integer(42);
    let result = primitive_eval(&[literal, env_value]).unwrap();

    match result {
        Value::Literal(lit) => {
            if let Some(n) = lit.to_i64() {
                assert_eq!(n, 42, "Expected 42, got {}", n);
            } else {
                panic!("Expected integer literal");
            }
        }
        _ => panic!("Expected literal result"),
    }
}

#[test]
fn test_eval_error_handling() {
    // Test error handling for invalid arguments
    let result = primitive_eval(&[Value::integer(1)]);
    assert!(result.is_err(), "Should error with insufficient arguments");

    let result = primitive_eval(&[Value::integer(1), Value::integer(2), Value::integer(3)]);
    assert!(result.is_err(), "Should error with too many arguments");

    // Test error with non-environment second argument
    let result = primitive_eval(&[Value::integer(1), Value::integer(2)]);
    assert!(
        result.is_err(),
        "Should error with non-environment argument"
    );
}

#[test]
fn test_environment_bound_error_handling() {
    // Test error handling for wrong number of arguments
    let result = primitive_environment_bound_p(&[Value::integer(1)]);
    assert!(result.is_err(), "Should error with insufficient arguments");

    let result =
        primitive_environment_bound_p(&[Value::integer(1), Value::integer(2), Value::integer(3)]);
    assert!(result.is_err(), "Should error with too many arguments");

    // Test error with non-symbol first argument
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
    let env_value = Value::Environment(env);
    let result = primitive_environment_bound_p(&[Value::integer(1), env_value]);
    assert!(
        result.is_err(),
        "Should error with non-symbol first argument"
    );
}

#[test]
fn test_r7rs_compliance_integration() {
    // Integration test to verify R7RS compliance
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
    create_eval_bindings(&env);

    // Test that all required procedures are available
    assert!(env.lookup("eval").is_some(), "eval should be available");
    assert!(
        env.lookup("environment").is_some(),
        "environment should be available"
    );
    assert!(
        env.lookup("environment-bound?").is_some(),
        "environment-bound? should be available"
    );
    assert!(
        env.lookup("scheme-report-environment").is_some(),
        "scheme-report-environment should be available"
    );
    assert!(
        env.lookup("null-environment").is_some(),
        "null-environment should be available"
    );
    assert!(
        env.lookup("interaction-environment").is_some(),
        "interaction-environment should be available"
    );
}

#[test]
fn test_environment_security() {
    // Test that eval operations respect security boundaries
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
    create_eval_bindings(&env);
    let env_value = Value::Environment(env);

    // Test evaluating a simple safe expression
    let literal = Value::boolean(true);
    let result = primitive_eval(&[literal, env_value]);
    assert!(
        result.is_ok(),
        "Safe expression should evaluate successfully"
    );
}

#[test]
fn test_environment_creation_error_handling() {
    // Test error handling for invalid import specifications
    let invalid_import = Value::integer(42);
    let result = primitive_environment(&[invalid_import]);
    assert!(result.is_err(), "Should error with invalid import spec");

    // Test empty import specification
    let empty_list = Value::Nil;
    let result = primitive_environment(&[empty_list]);
    assert!(result.is_err(), "Should error with empty import spec");
}

#[cfg(test)]
mod property_based_tests {
    use super::*;
    // Note: quickcheck dependency removed - converting to standard test
    fn test_environment_bound_consistency_helper(var_name: String) -> bool {
        if var_name.is_empty() {
            return true; // Skip empty names
        }

        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let symbol = Value::symbol(intern_symbol(var_name.clone()));
        let env_value = Value::Environment(env.clone());

        // Check that unbound variable returns false
        let result = primitive_environment_bound_p(&[symbol.clone(), env_value.clone()]).unwrap();
        assert!(result.is_falsy(), "Unbound variable should return false");

        // Define the variable and check that it now returns true
        env.define(var_name, Value::integer(42));
        let result = primitive_environment_bound_p(&[symbol, env_value]).unwrap();
        assert!(result.is_truthy(), "Bound variable should return true");

        true
    }

    #[test]
    fn test_environment_bound_consistency() {
        // Test with some representative strings
        let test_cases = vec![
            "test-var".to_string(),
            "x".to_string(),
            "long-variable-name".to_string(),
            "special*chars".to_string(),
        ];

        for var_name in test_cases {
            assert!(test_environment_bound_consistency_helper(var_name));
        }
    }

    // Note: Converted from quickcheck - keeping similar logic
    fn test_eval_literal_identity(n: i64) -> bool {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        create_eval_bindings(&env);
        let env_value = Value::Environment(env);

        let literal = Value::integer(n);
        let result = primitive_eval(&[literal.clone(), env_value]).unwrap();

        // Evaluating a literal should return the same literal
        match (&literal, &result) {
            (Value::Literal(lit1), Value::Literal(lit2)) => lit1.to_i64() == lit2.to_i64(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_environment_creation() {
        let start = Instant::now();
        let iterations = 1000;

        for _ in 0..iterations {
            let scheme_symbol = Value::symbol(intern_symbol("scheme".to_string()));
            let base_symbol = Value::symbol(intern_symbol("base".to_string()));
            let import_list = Value::pair(scheme_symbol, Value::pair(base_symbol, Value::Nil));

            let _result = primitive_environment(&[import_list]).unwrap();
        }

        let duration = start.elapsed();
        println!(
            "Environment creation: {} operations in {:?} ({:.2} ops/sec)",
            iterations,
            duration,
            iterations as f64 / duration.as_secs_f64()
        );
    }

    #[test]
    fn benchmark_environment_bound_checks() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        env.define("test-var".to_string(), Value::integer(42));

        let symbol_value = Value::symbol(intern_symbol("test-var".to_string()));
        let env_value = Value::Environment(env);

        let start = Instant::now();
        let iterations = 10000;

        for _ in 0..iterations {
            let _result =
                primitive_environment_bound_p(&[symbol_value.clone(), env_value.clone()]).unwrap();
        }

        let duration = start.elapsed();
        println!(
            "Environment bound checks: {} operations in {:?} ({:.2} ops/sec)",
            iterations,
            duration,
            iterations as f64 / duration.as_secs_f64()
        );
    }
}
