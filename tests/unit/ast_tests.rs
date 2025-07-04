//! Tests for AST (Abstract Syntax Tree) module

use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;

#[test]
fn test_literal_display() {
    assert_eq!(format!("{}", Literal::Boolean(true)), "#t");
    assert_eq!(format!("{}", Literal::Boolean(false)), "#f");
    assert_eq!(
        format!("{}", Literal::Number(SchemeNumber::Integer(42))),
        "42"
    );
    assert_eq!(
        format!("{}", Literal::String("hello".to_string())),
        "\"hello\""
    );
    assert_eq!(format!("{}", Literal::Character('a')), "#\\a");
    assert_eq!(format!("{}", Literal::Character(' ')), "#\\space");
    assert_eq!(format!("{}", Literal::Nil), "()");
}

#[test]
fn test_expr_display() {
    let expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);
    assert_eq!(format!("{}", expr), "(+ 1 2)");
}

#[test]
fn test_special_form_detection() {
    let define_expr = Expr::List(vec![
        Expr::Variable("define".to_string()),
        Expr::Variable("x".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
    ]);
    assert!(define_expr.is_special_form());

    let lambda_expr = Expr::List(vec![
        Expr::Variable("lambda".to_string()),
        Expr::List(vec![Expr::Variable("x".to_string())]),
        Expr::Variable("x".to_string()),
    ]);
    assert!(lambda_expr.is_special_form());

    let call_expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);
    assert!(!call_expr.is_special_form());
}

#[test]
fn test_operator_operands() {
    let expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);

    assert_eq!(expr.get_operator(), Some("+"));
    assert_eq!(expr.get_operands().unwrap().len(), 2);
}
