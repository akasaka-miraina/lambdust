//! Unit tests for bridge functionality
//!
//! This module contains tests for the bridge API that enables
//! integration between Lambdust and external applications.

use lambdust::bridge::{FromScheme, LambdustBridge, ToScheme};
use lambdust::value::Value;

#[test]
fn test_bridge_creation() {
    let bridge = LambdustBridge::new();
    assert!(bridge.registry.lock().unwrap().objects_is_empty());
    assert!(bridge.registry.lock().unwrap().functions_is_empty());
}

#[test]
fn test_register_function() {
    let mut bridge = LambdustBridge::new();

    bridge.register_function("add", Some(2), |args| {
        let a = i64::from_scheme(&args[0])?;
        let b = i64::from_scheme(&args[1])?;
        (a + b).to_scheme()
    });

    // For now, just test that the function is registered
    assert!(bridge.registry.lock().unwrap().has_function("add"));
}

#[test]
fn test_type_conversion() {
    assert_eq!(42i64.to_scheme().unwrap(), Value::from(42i64));
    assert_eq!(
        std::f64::consts::PI.to_scheme().unwrap(),
        Value::from(std::f64::consts::PI)
    );
    assert_eq!(true.to_scheme().unwrap(), Value::from(true));
    assert_eq!("hello".to_scheme().unwrap(), Value::from("hello"));

    let value = Value::from(42i64);
    assert_eq!(i64::from_scheme(&value).unwrap(), 42i64);

    let value = Value::from(true);
    assert!(bool::from_scheme(&value).unwrap());
}

#[test]
fn test_define_variable() {
    let mut bridge = LambdustBridge::new();
    bridge.define("my-var", Value::from(100i64));

    let result = bridge.eval("my-var").unwrap();
    assert_eq!(result, Value::from(100i64));
}
