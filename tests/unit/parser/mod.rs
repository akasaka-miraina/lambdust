//! Unit tests for parser

use lambdust::parser::{parse, ParseError};
use lambdust::ast::Expr;

#[test]
fn test_parse_number() {
    let result = parse("42").unwrap();
    assert!(matches!(result, Expr::Number(42.0)));
}

#[test]
fn test_parse_string() {
    let result = parse("\"hello\"").unwrap();
    assert!(matches!(result, Expr::String(s) if s == "hello"));
}

#[test]
fn test_parse_symbol() {
    let result = parse("hello").unwrap();
    assert!(matches!(result, Expr::Symbol(s) if s == "hello"));
}

#[test]
fn test_parse_list() {
    let result = parse("(+ 1 2)").unwrap();
    assert!(matches!(result, Expr::List(_)));
}

#[test]
fn test_parse_error() {
    let result = parse("(unclosed");
    assert!(result.is_err());
}