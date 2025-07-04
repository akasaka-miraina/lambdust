//! Unit tests for SRFI 97 (SRFI Libraries)
//!
//! These tests were extracted from src/srfi/srfi_97.rs

use lambdust::lexer::SchemeNumber;
use lambdust::srfi::SrfiModule;
use lambdust::srfi::srfi_97::Srfi97;
use lambdust::value::{Procedure, Value};

#[test]
fn test_srfi_97_info() {
    let srfi97 = Srfi97;
    assert_eq!(srfi97.srfi_id(), 97);
    assert_eq!(srfi97.name(), "SRFI Libraries");
    assert!(srfi97.parts().contains(&"inquiry"));
    assert!(srfi97.parts().contains(&"available"));
}

#[test]
fn test_srfi_97_exports() {
    let srfi97 = Srfi97;
    let exports = srfi97.exports();

    assert!(exports.contains_key("srfi-available?"));
    assert!(exports.contains_key("srfi-supported-ids"));
    assert!(exports.contains_key("srfi-name"));
    assert!(exports.contains_key("srfi-parts"));
}

#[test]
fn test_srfi_available_function() {
    let srfi97 = Srfi97;
    let exports = srfi97.exports();

    let func = exports.get("srfi-available?").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
        // Test with supported SRFI
        let result = func(&[Value::Number(SchemeNumber::Integer(9))]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test with unsupported SRFI
        let result = func(&[Value::Number(SchemeNumber::Integer(999))]).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }
}

#[test]
fn test_srfi_supported_ids_function() {
    let srfi97 = Srfi97;
    let exports = srfi97.exports();

    let func = exports.get("srfi-supported-ids").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
        let result = func(&[]).unwrap();
        assert!(matches!(result, Value::Vector(_)));
    }
}

#[test]
fn test_srfi_name_function() {
    let srfi97 = Srfi97;
    let exports = srfi97.exports();

    let func = exports.get("srfi-name").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
        let result = func(&[Value::Number(SchemeNumber::Integer(9))]).unwrap();
        assert_eq!(result, Value::String("Defining Record Types".to_string()));
    }
}
