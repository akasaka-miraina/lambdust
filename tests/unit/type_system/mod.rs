//! Unit tests for type system

use lambdust::type_system::{TypeChecker, TypeInference};
use lambdust::value::Value;

#[test]
fn test_type_checking() {
    let checker = TypeChecker::new();
    
    // Test number type
    let num_type = checker.infer_type(&Value::Number(42.0));
    assert!(num_type.is_ok());
    
    // Test string type
    let str_type = checker.infer_type(&Value::String("hello".to_string()));
    assert!(str_type.is_ok());
}