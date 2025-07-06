//! Unit tests for SRFI 128: Comparators

use lambdust::srfi::SrfiModule;
use lambdust::srfi::srfi_128::{Srfi128, default_number_comparator, default_string_comparator};
use lambdust::value::{Procedure, Value};
use std::rc::Rc;

#[test]
fn test_number_comparator() {
    let comp = default_number_comparator();
    let num1 = Value::from(5i64);
    let num2 = Value::from(10i64);
    let num3 = Value::from(5i64);

    // Type test
    assert!(comp.test_type(&num1));
    assert!(!comp.test_type(&Value::String("hello".to_string())));

    // Equality
    assert!(comp.equal(&num1, &num3));
    assert!(!comp.equal(&num1, &num2));

    // Comparison
    assert_eq!(comp.compare(&num1, &num2).unwrap(), -1);
    assert_eq!(comp.compare(&num2, &num1).unwrap(), 1);
    assert_eq!(comp.compare(&num1, &num3).unwrap(), 0);

    // Hash
    assert!(comp.hash(&num1).is_ok());
    assert!(comp.has_hash());
    assert!(comp.has_comparison());
}

#[test]
fn test_string_comparator() {
    let comp = default_string_comparator();
    let str1 = Value::String("apple".to_string());
    let str2 = Value::String("banana".to_string());
    let str3 = Value::String("apple".to_string());

    // Type test
    assert!(comp.test_type(&str1));
    assert!(!comp.test_type(&Value::from(42i64)));

    // Equality
    assert!(comp.equal(&str1, &str3));
    assert!(!comp.equal(&str1, &str2));

    // Comparison
    assert_eq!(comp.compare(&str1, &str2).unwrap(), -1);
    assert_eq!(comp.compare(&str2, &str1).unwrap(), 1);
    assert_eq!(comp.compare(&str1, &str3).unwrap(), 0);

    // Hash
    assert!(comp.hash(&str1).is_ok());
    assert!(comp.has_hash());
    assert!(comp.has_comparison());
}

#[test]
fn test_comparator_predicates() {
    let srfi = Srfi128;
    let exports = srfi.exports();

    // Test comparator? predicate
    let comparator_pred = exports.get("comparator?").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = comparator_pred {
        // Test with comparator
        let comp = Value::Comparator(Rc::new(default_number_comparator()));
        let result = func(&[comp]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test with non-comparator
        let result = func(&[Value::from(42i64)]).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }
}

#[test]
fn test_comparator_ordered_predicate() {
    let srfi = Srfi128;
    let exports = srfi.exports();

    let ordered_pred = exports.get("comparator-ordered?").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = ordered_pred {
        // Test with ordered comparator
        let comp = Value::Comparator(Rc::new(default_number_comparator()));
        let result = func(&[comp]).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }
}

#[test]
fn test_comparator_hashable_predicate() {
    let srfi = Srfi128;
    let exports = srfi.exports();

    let hashable_pred = exports.get("comparator-hashable?").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = hashable_pred {
        // Test with hashable comparator
        let comp = Value::Comparator(Rc::new(default_string_comparator()));
        let result = func(&[comp]).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }
}

#[test]
fn test_equality_comparison() {
    let srfi = Srfi128;
    let exports = srfi.exports();

    let eq_proc = exports.get("=?").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = eq_proc {
        let comp = Value::Comparator(Rc::new(default_number_comparator()));
        let num1 = Value::from(5i64);
        let num2 = Value::from(5i64);
        let num3 = Value::from(10i64);

        // Test equal values
        let result = func(&[comp.clone(), num1.clone(), num2]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test unequal values
        let result = func(&[comp, num1, num3]).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }
}

#[test]
fn test_less_than_comparison() {
    let srfi = Srfi128;
    let exports = srfi.exports();

    let lt_proc = exports.get("<?").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = lt_proc {
        let comp = Value::Comparator(Rc::new(default_number_comparator()));
        let num1 = Value::from(1i64);
        let num2 = Value::from(5i64);
        let num3 = Value::from(10i64);

        // Test increasing sequence
        let result = func(&[comp.clone(), num1.clone(), num2.clone(), num3]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test non-increasing sequence
        let result = func(&[comp, num2, num1]).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }
}

#[test]
fn test_standard_comparators() {
    let srfi = Srfi128;
    let exports = srfi.exports();

    // Check that all standard comparators are available
    assert!(exports.contains_key("default-comparator"));
    assert!(exports.contains_key("boolean-comparator"));
    assert!(exports.contains_key("real-comparator"));
    assert!(exports.contains_key("string-comparator"));
    assert!(exports.contains_key("symbol-comparator"));

    // Verify they are comparator values
    if let Value::Comparator(_) = exports.get("real-comparator").unwrap() {
        // Good
    } else {
        panic!("real-comparator should be a Comparator value");
    }
}

#[test]
fn test_make_comparator() {
    let srfi = Srfi128;
    let exports = srfi.exports();

    let make_proc = exports.get("make-comparator").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = make_proc {
        // Create a custom comparator (minimal test with dummy args)
        let dummy_proc = Value::Procedure(Procedure::Builtin {
            name: "dummy".to_string(),
            func: |_| Ok(Value::Boolean(true)),
            arity: Some(1),
        });

        let result = func(&[dummy_proc.clone(), dummy_proc]).unwrap();
        assert!(matches!(result, Value::Comparator(_)));
    }
}

#[test]
fn test_srfi_128_metadata() {
    let srfi = Srfi128;
    assert_eq!(srfi.srfi_id(), 128);
    assert_eq!(srfi.name(), "Comparators");
    assert!(srfi.parts().is_empty());
}

#[test]
fn test_exports_for_parts() {
    let srfi = Srfi128;
    let all_exports = srfi.exports();
    let parts_exports = srfi.exports_for_parts(&[]).unwrap();

    // Should return all exports since SRFI 128 has no parts
    assert_eq!(all_exports.len(), parts_exports.len());
}
