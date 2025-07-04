//! Tests for Environment module

use lambdust::environment::Environment;
use lambdust::error::LambdustError;
use lambdust::value::Value;
use std::rc::Rc;

#[test]
fn test_environment_define_get() {
    let env = Environment::new();
    env.define("x".to_string(), Value::from(42i64));

    assert_eq!(env.get("x").unwrap(), Value::from(42i64));
    assert!(env.get("y").is_err());
}

#[test]
fn test_environment_set() {
    let env = Environment::new();
    env.define("x".to_string(), Value::from(42i64));

    env.set("x", Value::from(100i64)).unwrap();
    assert_eq!(env.get("x").unwrap(), Value::from(100i64));

    assert!(env.set("y", Value::from(200i64)).is_err());
}

#[test]
fn test_environment_scoping() {
    let parent = Environment::new();
    parent.define("x".to_string(), Value::from(42i64));

    let child = Environment::with_parent(Rc::new(parent.clone()));
    child.define("y".to_string(), Value::from(100i64));

    // Child can access parent's variables
    assert_eq!(child.get("x").unwrap(), Value::from(42i64));
    assert_eq!(child.get("y").unwrap(), Value::from(100i64));

    // Parent cannot access child's variables
    assert!(parent.get("y").is_err());
}

#[test]
fn test_environment_shadowing() {
    let parent = Environment::new();
    parent.define("x".to_string(), Value::from(42i64));

    let child = Environment::with_parent(Rc::new(parent.clone()));
    child.define("x".to_string(), Value::from(100i64));

    // Child's binding shadows parent's
    assert_eq!(child.get("x").unwrap(), Value::from(100i64));
    assert_eq!(parent.get("x").unwrap(), Value::from(42i64));
}

#[test]
fn test_bind_parameters_fixed() {
    let env = Environment::new();
    let params = vec!["x".to_string(), "y".to_string()];
    let args = vec![Value::from(1i64), Value::from(2i64)];

    let new_env = env.bind_parameters(&params, &args, false).unwrap();

    assert_eq!(new_env.get("x").unwrap(), Value::from(1i64));
    assert_eq!(new_env.get("y").unwrap(), Value::from(2i64));
}

#[test]
fn test_bind_parameters_variadic() {
    let env = Environment::new();
    let params = vec!["x".to_string(), "rest".to_string()];
    let args = vec![Value::from(1i64), Value::from(2i64), Value::from(3i64)];

    let new_env = env.bind_parameters(&params, &args, true).unwrap();

    assert_eq!(new_env.get("x").unwrap(), Value::from(1i64));
    let rest = new_env.get("rest").unwrap();
    assert!(rest.is_list());
    assert_eq!(rest.list_length(), Some(2));
}

#[test]
fn test_bind_parameters_arity_error() {
    let env = Environment::new();
    let params = vec!["x".to_string(), "y".to_string()];
    let args = vec![Value::from(1i64)];

    let result = env.bind_parameters(&params, &args, false);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LambdustError::ArityError { .. }
    ));
}

#[test]
fn test_builtin_environment() {
    let env = Environment::with_builtins();

    assert!(env.contains("car"));
    assert!(env.contains("cdr"));
    assert!(env.contains("cons"));
    assert!(env.contains("null?"));
    assert!(env.contains("pair?"));
}
