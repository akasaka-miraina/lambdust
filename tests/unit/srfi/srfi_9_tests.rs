//! Unit tests for SRFI 9 (Defining Record Types)
//!
//! These tests were extracted from src/srfi/srfi_9.rs

use lambdust::lexer::SchemeNumber;
use lambdust::srfi::srfi_9::Srfi9;
use lambdust::srfi::SrfiModule;
use lambdust::value::{Procedure, Value};

#[test]
fn test_srfi_9_info() {
    let srfi9 = Srfi9;
    assert_eq!(srfi9.srfi_id(), 9);
    assert_eq!(srfi9.name(), "Defining Record Types");
    assert!(srfi9.parts().contains(&"records"));
    assert!(srfi9.parts().contains(&"types"));
}

#[test]
fn test_srfi_9_exports() {
    let srfi9 = Srfi9;
    let exports = srfi9.exports();

    assert!(exports.contains_key("make-record"));
    assert!(exports.contains_key("record-of-type?"));
    assert!(exports.contains_key("record-field"));
    assert!(exports.contains_key("record-set-field!"));
}

#[test]
fn test_record_operations() {
    let srfi9 = Srfi9;
    let exports = srfi9.exports();

    // Test make-record
    let make_record = exports.get("make-record").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = make_record {
        let args = vec![
            Value::Symbol("person".to_string()),
            Value::String("Alice".to_string()),
            Value::Number(SchemeNumber::Integer(30)),
        ];
        let result = func(&args).unwrap();
        assert!(matches!(result, Value::Record(_)));
    }
}
