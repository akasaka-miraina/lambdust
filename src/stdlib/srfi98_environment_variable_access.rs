//! SRFI-98: Environment Variable Access
//!
//! This library provides access to environment variables as defined by SRFI-98.
//! Environment variables are an important part of system integration, allowing
//! programs to access configuration data and system settings.
//!
//! ## Specification
//!
//! SRFI-98 defines three essential procedures:
//! - `(get-environment-variable name)` - Returns the value of the environment variable as a string or #f
//! - `(get-environment-variables)` - Returns an alist of all environment variables
//!
//! ## R7RS Integration
//!
//! These procedures are essential for practical Scheme applications that need to
//! interact with the system environment, making SRFI-98 important for real-world usage.
//!
//! ## Usage Examples
//!
//! ```scheme
//! ;; Get a specific environment variable
//! (get-environment-variable "HOME")     ; => "/home/user" or #f
//! (get-environment-variable "PATH")     ; => "/bin:/usr/bin:..." or #f
//!
//! ;; Get all environment variables
//! (get-environment-variables)          ; => (("HOME" . "/home/user") ("PATH" . "/bin:/usr/bin") ...)
//! ```

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{Environment, Generation};
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, Value};
use std::sync::Arc;

/// Install SRFI-98 environment variable access procedures into the environment.
///
/// This adds the two core procedures defined by SRFI-98:
/// - `get-environment-variable` - Get value of specific environment variable
/// - `get-environment-variables` - Get all environment variables as alist
pub fn install_srfi98_procedures(env: &mut Environment) {
    // get-environment-variable
    env.define(
        "get-environment-variable".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "get-environment-variable".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(get_environment_variable),
            effects: vec![Effect::IO],
        })),
    );

    // get-environment-variables
    env.define(
        "get-environment-variables".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "get-environment-variables".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(get_environment_variables),
            effects: vec![Effect::IO],
        })),
    );
}

/// `(get-environment-variable name)` - Get the value of an environment variable.
///
/// Returns the value of the environment variable named by the string `name`.
/// If no such environment variable exists, returns #f.
///
/// ## Arguments
/// - `name` - A string naming the environment variable to retrieve
///
/// ## Returns
/// The value of the environment variable as a string, or #f if not found
///
/// ## Examples
/// ```scheme
/// (get-environment-variable "HOME")     ; => "/home/user" or #f
/// (get-environment-variable "PATH")     ; => "/bin:/usr/bin:..." or #f
/// (get-environment-variable "NONEXISTENT") ; => #f
/// ```
pub fn get_environment_variable(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "get-environment-variable expects 1 argument, got {}",
                args.len()
            ),
            None,
        )));
    }

    let var_name =
        crate::stdlib::strings::common::extract_string(&args[0], "get-environment-variable")?;

    match std::env::var(&var_name) {
        Ok(value) => Ok(Value::string(value)),
        Err(_) => Ok(Value::boolean(false)),
    }
}

/// `(get-environment-variables)` - Get all environment variables as an association list.
///
/// Returns an association list of all environment variables, where each element
/// is a pair (name . value) with both name and value being strings.
///
/// ## Arguments
/// None
///
/// ## Returns
/// An association list of environment variables
///
/// ## Examples
/// ```scheme
/// (get-environment-variables)  ; => (("HOME" . "/home/user") ("PATH" . "/bin:/usr/bin") ...)
/// ```
pub fn get_environment_variables(args: &[Value]) -> Result<Value> {
    if !args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "get-environment-variables expects 0 arguments, got {}",
                args.len()
            ),
            None,
        )));
    }

    let mut env_vars = Vec::new();

    for (key, value) in std::env::vars() {
        // Create a pair (name . value)
        let pair = Value::cons(Value::string(key), Value::string(value));
        env_vars.push(pair);
    }

    // Convert to proper list
    Ok(Value::list(env_vars))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::{Environment, Generation};
    use std::collections::HashMap;

    /// Test basic environment variable access
    #[test]
    fn test_environment_variable_access_basic() {
        let mut env = Environment::new(None, 0 as Generation);
        install_srfi98_procedures(&mut env);

        // Test that procedures are installed
        assert!(env.lookup("get-environment-variable").is_some());
        assert!(env.lookup("get-environment-variables").is_some());

        // Test get-environment-variables
        let result = get_environment_variables(&[]).unwrap();
        assert!(matches!(result, Value::Nil | Value::Pair(_, _)));
    }

    /// Test error conditions
    #[test]
    fn test_environment_variable_access_errors() {
        // Wrong number of arguments for get-environment-variable
        assert!(get_environment_variable(&[]).is_err());
        assert!(
            get_environment_variable(&[Value::string("PATH"), Value::string("extra")]).is_err()
        );

        // Wrong number of arguments for get-environment-variables
        assert!(get_environment_variables(&[Value::string("unexpected")]).is_err());

        // Wrong argument type for get-environment-variable
        assert!(get_environment_variable(&[Value::integer(42)]).is_err());
    }

    /// Test environment variable retrieval
    #[test]
    fn test_environment_variable_retrieval() {
        // Set a test environment variable
        unsafe {
            std::env::set_var("LAMBDUST_TEST_VAR", "test_value");
        }

        // Test getting existing variable
        let result = get_environment_variable(&[Value::string("LAMBDUST_TEST_VAR")]).unwrap();
        assert_eq!(result, Value::string("test_value"));

        // Test getting non-existing variable
        let result =
            get_environment_variable(&[Value::string("LAMBDUST_NONEXISTENT_VAR")]).unwrap();
        assert_eq!(result, Value::boolean(false));

        // Clean up
        unsafe {
            std::env::remove_var("LAMBDUST_TEST_VAR");
        }
    }

    /// Test get-environment-variables returns proper alist
    #[test]
    fn test_get_environment_variables_alist() {
        // Set test environment variables
        unsafe {
            std::env::set_var("LAMBDUST_TEST_A", "value_a");
            std::env::set_var("LAMBDUST_TEST_B", "value_b");
        }

        let result = get_environment_variables(&[]).unwrap();

        // Should be a list (or empty list)
        match result {
            Value::Nil => {
                // Empty environment is valid
            }
            Value::Pair(_, _) => {
                // Should contain our test variables somewhere in the list
                let mut found_a = false;
                let mut found_b = false;

                let mut current = &result;
                loop {
                    match current {
                        Value::Pair(car, cdr) => {
                            // Each element should be a pair (name . value)
                            if let Value::Pair(name, value) = car.as_ref() {
                                if let (
                                    Value::Literal(crate::ast::Literal::String(name_str)),
                                    Value::Literal(crate::ast::Literal::String(value_str)),
                                ) = (name.as_ref(), value.as_ref())
                                {
                                    if name_str.as_ref() == "LAMBDUST_TEST_A"
                                        && value_str.as_ref() == "value_a"
                                    {
                                        found_a = true;
                                    }
                                    if name_str.as_ref() == "LAMBDUST_TEST_B"
                                        && value_str.as_ref() == "value_b"
                                    {
                                        found_b = true;
                                    }
                                }
                            }
                            current = cdr.as_ref();
                        }
                        Value::Nil => break,
                        _ => panic!("Environment variables list should be a proper list"),
                    }
                }

                assert!(
                    found_a,
                    "Should find LAMBDUST_TEST_A in environment variables"
                );
                assert!(
                    found_b,
                    "Should find LAMBDUST_TEST_B in environment variables"
                );
            }
            _ => panic!("get-environment-variables should return a list"),
        }

        // Clean up
        unsafe {
            std::env::remove_var("LAMBDUST_TEST_A");
            std::env::remove_var("LAMBDUST_TEST_B");
        }
    }

    /// Test SRFI-98 environment integration
    #[test]
    fn test_srfi98_environment_integration() {
        let mut env = Environment::new(None, 0 as Generation);
        install_srfi98_procedures(&mut env);

        // Verify all procedures are defined
        assert!(env.lookup("get-environment-variable").is_some());
        assert!(env.lookup("get-environment-variables").is_some());

        // Verify they are callable primitives
        let get_var = env.lookup("get-environment-variable").unwrap();
        assert!(matches!(get_var, Value::Primitive(_)));

        let get_vars = env.lookup("get-environment-variables").unwrap();
        assert!(matches!(get_vars, Value::Primitive(_)));
    }

    /// Test specific environment variable scenarios
    #[test]
    fn test_common_environment_variables() {
        // Test PATH which should exist on most systems
        let result = get_environment_variable(&[Value::string("PATH")]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Literal(crate::ast::Literal::String(_)) => {
                // PATH exists and is a string - good
            }
            Value::Literal(crate::ast::Literal::Boolean(false)) => {
                // PATH doesn't exist - unusual but not an error
            }
            _ => panic!("get-environment-variable should return string or #f"),
        }

        // Test definitely non-existent variable
        let result =
            get_environment_variable(&[Value::string("DEFINITELY_DOES_NOT_EXIST_XYZ123")]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
}
