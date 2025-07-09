//! Unit tests for SRFI 45 (Primitives for Expressing Iterative Lazy Algorithms)
//!
//! These tests were extracted from src/srfi/srfi_45.rs

use lambdust::lexer::SchemeNumber;
use lambdust::srfi::srfi_45::Srfi45;
use lambdust::srfi::SrfiModule;
use lambdust::value::{Procedure, Value};

#[test]
fn test_srfi_45_info() {
    let srfi45 = Srfi45;
    assert_eq!(srfi45.srfi_id(), 45);
    assert_eq!(
        srfi45.name(),
        "Primitives for Expressing Iterative Lazy Algorithms"
    );
    assert!(srfi45.parts().contains(&"lazy"));
    assert!(srfi45.parts().contains(&"promises"));
}

#[test]
fn test_srfi_45_exports() {
    let srfi45 = Srfi45;
    let exports = srfi45.exports();

    assert!(exports.contains_key("delay"));
    assert!(exports.contains_key("lazy"));
    assert!(exports.contains_key("force"));
    assert!(exports.contains_key("promise?"));
}

#[test]
fn test_promise_predicate() {
    let srfi45 = Srfi45;
    let exports = srfi45.exports();

    let promise_pred = exports.get("promise?").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = promise_pred {
        // Test with non-promise
        let result = func(&[Value::Number(SchemeNumber::Integer(42))]).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }
}
