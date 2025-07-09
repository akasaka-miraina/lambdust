//! Tests for SRFI 140: Immutable Strings

use lambdust::environment::Environment;
use lambdust::error::Result;
use lambdust::evaluator::eval_with_formal_semantics;
use lambdust::lexer::tokenize;
use lambdust::parser::parse;
use lambdust::srfi::{SrfiImport, SrfiRegistry};
use lambdust::value::Value;
use std::rc::Rc;

fn eval_str_with_srfi_140(input: &str) -> Result<Value> {
    let tokens = tokenize(input)?;
    let ast = parse(tokens)?;
    let env = Rc::new(Environment::with_builtins());

    // Import SRFI 140
    let registry = SrfiRegistry::with_standard_srfis();
    let import = SrfiImport::new(140);
    let exports = registry.import_srfi(&import)?;

    for (name, value) in exports {
        env.define(name, value);
    }

    eval_with_formal_semantics(ast, env)
}

#[test]
fn test_string_predicates() {
    // Test string? predicate - should work for both istrings and mstrings
    let result = eval_str_with_srfi_140("(string? \"hello\")").unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = eval_str_with_srfi_140("(string? 42)").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_istring_predicate() {
    // Test istring? predicate
    let result = eval_str_with_srfi_140("(istring? (string #\\h #\\i))").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_mstring_predicate() {
    // Test mstring? predicate with make-string
    let result = eval_str_with_srfi_140("(mstring? (make-string 5 #\\a))").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_string_construction() {
    // Test string construction - should return immutable string
    let result = eval_str_with_srfi_140("(string #\\h #\\e #\\l #\\l #\\o)").unwrap();
    if let Value::IString(istr) = result {
        assert_eq!(istr.to_string(), "hello");
    } else {
        panic!("Expected IString");
    }
}

#[test]
fn test_make_string() {
    // Test make-string - should return mutable string
    let result = eval_str_with_srfi_140("(make-string 3 #\\a)").unwrap();
    if let Value::String(s) = result {
        assert_eq!(s, "aaa");
    } else {
        panic!("Expected String");
    }
}

#[test]
fn test_string_length() {
    // Test string-length with istring
    let result = eval_str_with_srfi_140("(string-length (string #\\h #\\i))").unwrap();
    assert_eq!(
        result,
        Value::Number(lambdust::lexer::SchemeNumber::Integer(2))
    );

    // Test string-length with mstring
    let result = eval_str_with_srfi_140("(string-length (make-string 5))").unwrap();
    assert_eq!(
        result,
        Value::Number(lambdust::lexer::SchemeNumber::Integer(5))
    );
}

#[test]
fn test_string_ref() {
    // Test string-ref with istring
    let result =
        eval_str_with_srfi_140("(string-ref (string #\\h #\\e #\\l #\\l #\\o) 1)").unwrap();
    assert_eq!(result, Value::Character('e'));

    // Test string-ref with mstring
    let result = eval_str_with_srfi_140("(string-ref (make-string 3 #\\x) 1)").unwrap();
    assert_eq!(result, Value::Character('x'));
}

#[test]
fn test_string_copy() {
    // Test string-copy - should return mutable string
    let result = eval_str_with_srfi_140("(string-copy (string #\\h #\\i))").unwrap();
    if let Value::String(s) = result {
        assert_eq!(s, "hi");
    } else {
        panic!("Expected String (mutable)");
    }
}

#[test]
fn test_substring() {
    // Test substring - should return immutable string
    let result =
        eval_str_with_srfi_140("(substring (string #\\h #\\e #\\l #\\l #\\o) 1 4)").unwrap();
    if let Value::IString(istr) = result {
        assert_eq!(istr.to_string(), "ell");
    } else {
        panic!("Expected IString");
    }
}

#[test]
fn test_string_append() {
    // Test string-append - should return immutable string
    let result = eval_str_with_srfi_140(
        "(string-append (string #\\h #\\i) (string #\\  #\\t #\\h #\\e #\\r #\\e))",
    )
    .unwrap();
    if let Value::IString(istr) = result {
        assert_eq!(istr.to_string(), "hi there");
    } else {
        panic!("Expected IString");
    }
}

#[test]
fn test_string_append_empty() {
    // Test string-append with no arguments
    let result = eval_str_with_srfi_140("(string-append)").unwrap();
    if let Value::IString(istr) = result {
        assert_eq!(istr.to_string(), "");
    } else {
        panic!("Expected empty IString");
    }
}

#[test]
fn test_substring_bounds() {
    // Test substring with out of bounds - should error
    let result = eval_str_with_srfi_140("(substring (string #\\h #\\i) 0 5)");
    assert!(result.is_err());
}

#[test]
fn test_string_ref_bounds() {
    // Test string-ref with out of bounds - should error
    let result = eval_str_with_srfi_140("(string-ref (string #\\h #\\i) 5)");
    assert!(result.is_err());
}

#[test]
fn test_srfi_140_exports() {
    let registry = SrfiRegistry::with_standard_srfis();
    let import = SrfiImport::new(140);
    let exports = registry.import_srfi(&import).unwrap();

    // Check that all required exports exist
    assert!(exports.contains_key("string?"));
    assert!(exports.contains_key("istring?"));
    assert!(exports.contains_key("mstring?"));
    assert!(exports.contains_key("string-length"));
    assert!(exports.contains_key("string-ref"));
    assert!(exports.contains_key("string"));
    assert!(exports.contains_key("make-string"));
    assert!(exports.contains_key("string-copy"));
    assert!(exports.contains_key("substring"));
    assert!(exports.contains_key("string-append"));
}
