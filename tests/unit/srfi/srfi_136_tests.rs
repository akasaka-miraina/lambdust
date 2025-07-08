//! Tests for SRFI 136: Extensible Record Types

use lambdust::environment::Environment;
use lambdust::error::Result;
use lambdust::evaluator::eval_with_formal_semantics;
use lambdust::lexer::tokenize;
use lambdust::parser::parse;
use lambdust::srfi::{SrfiImport, SrfiRegistry};
use lambdust::value::Value;
use std::rc::Rc;

fn eval_str_with_srfi_136(input: &str) -> Result<Value> {
    let tokens = tokenize(input)?;
    let ast = parse(tokens)?;
    let env = Rc::new(Environment::with_builtins());
    
    // Import SRFI 136
    let registry = SrfiRegistry::with_standard_srfis();
    let import = SrfiImport::new(136);
    let exports = registry.import_srfi(&import)?;
    
    for (name, value) in exports {
        env.define(name, value);
    }
    
    eval_with_formal_semantics(ast, env)
}

#[test]
fn test_basic_predicates() {
    // Test record? predicate
    let result = eval_str_with_srfi_136("(record? 42)").unwrap();
    assert_eq!(result, Value::Boolean(false));
    
    // Test record-type-descriptor? predicate
    let result = eval_str_with_srfi_136("(record-type-descriptor? #t)").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test] 
fn test_record_type_descriptor_creation() {
    // Create a simple record type descriptor and test it directly
    let result = eval_str_with_srfi_136(
        "(record-type-descriptor? (make-record-type-descriptor 'point '#(x y)))"
    ).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_record_type_name() {
    let result = eval_str_with_srfi_136(
        "(record-type-name (make-record-type-descriptor 'person '#(name age)))"
    ).unwrap();
    assert_eq!(result, Value::Symbol("person".to_string()));
}

#[test]
fn test_record_type_fields() {
    let result = eval_str_with_srfi_136(
        "(record-type-fields (make-record-type-descriptor 'car '#(make model year)))"
    ).unwrap();
    
    if let Value::Vector(fields) = result {
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0], Value::Symbol("make".to_string()));
        assert_eq!(fields[1], Value::Symbol("model".to_string()));
        assert_eq!(fields[2], Value::Symbol("year".to_string()));
    } else {
        panic!("Expected vector of field names");
    }
}

#[test]
fn test_record_type_parent() {
    // Test with no parent
    let result = eval_str_with_srfi_136(
        r#"
        (begin
          (define simple-rtd 
            (make-record-type-descriptor 'simple '#(field)))
          (record-type-parent simple-rtd))
        "#
    ).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_record_creation() {
    let result = eval_str_with_srfi_136(
        r#"
        (begin
          (define point-rtd 
            (make-record-type-descriptor 'point '#(x y)))
          (define p (make-record point-rtd '#(10 20)))
          (record? p))
        "#
    ).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_record_field_count_mismatch() {
    let result = eval_str_with_srfi_136(
        r#"
        (begin
          (define point-rtd 
            (make-record-type-descriptor 'point '#(x y)))
          (make-record point-rtd '#(10)))
        "#
    );
    assert!(result.is_err());
}

#[test]
fn test_invalid_arguments() {
    // Test invalid name type
    let result = eval_str_with_srfi_136(
        "(make-record-type-descriptor 42 '#(field))"
    );
    assert!(result.is_err());
    
    // Test invalid field specification
    let result = eval_str_with_srfi_136(
        "(make-record-type-descriptor 'test 42)"
    );
    assert!(result.is_err());
}

#[test]
fn test_inheritance_with_parent() {
    let result = eval_str_with_srfi_136(
        r#"
        (begin
          (define shape-rtd 
            (make-record-type-descriptor 'shape '#(color)))
          (define circle-rtd 
            (make-record-type-descriptor 'circle '#(radius) shape-rtd))
          (record-type-descriptor? circle-rtd))
        "#
    ).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_inheritance_field_count() {
    let result = eval_str_with_srfi_136(
        r#"
        (begin
          (define shape-rtd 
            (make-record-type-descriptor 'shape '#(color)))
          (define circle-rtd 
            (make-record-type-descriptor 'circle '#(radius) shape-rtd))
          (define c (make-record circle-rtd '#(red 5)))
          (record? c))
        "#
    ).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_record_type_parent_with_inheritance() {
    let result = eval_str_with_srfi_136(
        r#"
        (begin
          (define base-rtd 
            (make-record-type-descriptor 'base '#(id)))
          (define derived-rtd 
            (make-record-type-descriptor 'derived '#(value) base-rtd))
          (define parent (record-type-parent derived-rtd))
          (record-type-descriptor? parent))
        "#
    ).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_complex_inheritance() {
    let result = eval_str_with_srfi_136(
        r#"
        (begin
          ; Create a hierarchy: Vehicle -> Car -> SportsCar
          (define vehicle-rtd 
            (make-record-type-descriptor 'vehicle '#(year)))
          (define car-rtd 
            (make-record-type-descriptor 'car '#(doors) vehicle-rtd))
          (define sports-car-rtd 
            (make-record-type-descriptor 'sports-car '#(top-speed) car-rtd))
          
          ; Create a sports car instance (should have all fields: year, doors, top-speed)
          (define my-car (make-record sports-car-rtd '#(2024 2 200)))
          (record? my-car))
        "#
    ).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_srfi_136_exports() {
    let registry = SrfiRegistry::with_standard_srfis();
    let import = SrfiImport::new(136);
    let exports = registry.import_srfi(&import).unwrap();

    // Check that all required exports exist
    assert!(exports.contains_key("record?"));
    assert!(exports.contains_key("record-type-descriptor?"));
    assert!(exports.contains_key("record-type-descriptor"));
    assert!(exports.contains_key("record-type-name"));
    assert!(exports.contains_key("record-type-parent"));
    assert!(exports.contains_key("record-type-fields"));
    assert!(exports.contains_key("make-record-type-descriptor"));
    assert!(exports.contains_key("make-record"));
}