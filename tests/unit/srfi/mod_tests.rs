//! Unit tests for SRFI module functionality
//!
//! These tests were extracted from src/srfi/mod.rs

use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;
use lambdust::srfi::parse_srfi_import;

#[test]
fn test_parse_srfi_import_full() {
    let expr = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);

    let import = parse_srfi_import(&expr).unwrap();
    assert_eq!(import.id, 1);
    assert!(import.imports_all());
}

#[test]
fn test_parse_srfi_import_with_parts() {
    let expr = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Variable("lists".to_string()),
    ]);

    let import = parse_srfi_import(&expr).unwrap();
    assert_eq!(import.id, 1);
    assert_eq!(import.parts, vec!["lists"]);
    assert!(!import.imports_all());
}
