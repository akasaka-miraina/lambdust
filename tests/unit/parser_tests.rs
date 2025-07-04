//! Unit tests for the parser module
//!
//! These tests verify the parsing functionality for converting tokens into AST expressions.

use lambdust::ast::{Expr, Literal};
use lambdust::lexer::{SchemeNumber, tokenize};
use lambdust::parser::{parse, parse_multiple};

#[test]
fn test_parse_atoms() {
    let tokens = tokenize("42 #t \"hello\" x").unwrap();
    let expressions = parse_multiple(tokens).unwrap();

    assert_eq!(expressions.len(), 4);
    assert_eq!(
        expressions[0],
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))
    );
    assert_eq!(expressions[1], Expr::Literal(Literal::Boolean(true)));
    assert_eq!(
        expressions[2],
        Expr::Literal(Literal::String("hello".to_string()))
    );
    assert_eq!(expressions[3], Expr::Variable("x".to_string()));
}

#[test]
fn test_parse_simple_list() {
    let tokens = tokenize("(+ 1 2)").unwrap();
    let expr = parse(tokens).unwrap();

    match expr {
        Expr::List(exprs) => {
            assert_eq!(exprs.len(), 3);
            assert_eq!(exprs[0], Expr::Variable("+".to_string()));
            assert_eq!(
                exprs[1],
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1)))
            );
            assert_eq!(
                exprs[2],
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2)))
            );
        }
        _ => panic!("Expected list expression"),
    }
}

#[test]
fn test_parse_nested_list() {
    let tokens = tokenize("(+ (* 2 3) 4)").unwrap();
    let expr = parse(tokens).unwrap();

    match expr {
        Expr::List(exprs) => {
            assert_eq!(exprs.len(), 3);
            assert_eq!(exprs[0], Expr::Variable("+".to_string()));

            match &exprs[1] {
                Expr::List(inner) => {
                    assert_eq!(inner.len(), 3);
                    assert_eq!(inner[0], Expr::Variable("*".to_string()));
                }
                _ => panic!("Expected nested list"),
            }
        }
        _ => panic!("Expected list expression"),
    }
}

#[test]
fn test_parse_quote() {
    let tokens = tokenize("'x").unwrap();
    let expr = parse(tokens).unwrap();

    match expr {
        Expr::Quote(inner) => {
            assert_eq!(*inner, Expr::Variable("x".to_string()));
        }
        _ => panic!("Expected quote expression"),
    }
}

#[test]
fn test_parse_dotted_list() {
    let tokens = tokenize("(a b . c)").unwrap();
    let expr = parse(tokens).unwrap();

    match expr {
        Expr::DottedList(exprs, tail) => {
            assert_eq!(exprs.len(), 2);
            assert_eq!(exprs[0], Expr::Variable("a".to_string()));
            assert_eq!(exprs[1], Expr::Variable("b".to_string()));
            assert_eq!(*tail, Expr::Variable("c".to_string()));
        }
        _ => panic!("Expected dotted list expression"),
    }
}

#[test]
fn test_parse_empty_list() {
    let tokens = tokenize("()").unwrap();
    let expr = parse(tokens).unwrap();

    assert_eq!(expr, Expr::List(vec![]));
}

#[test]
fn test_parse_quasiquote() {
    let tokens = tokenize("`(,x ,@y)").unwrap();
    let expr = parse(tokens).unwrap();

    match expr {
        Expr::Quasiquote(inner) => match inner.as_ref() {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 2);
                assert!(matches!(exprs[0], Expr::Unquote(_)));
                assert!(matches!(exprs[1], Expr::UnquoteSplicing(_)));
            }
            _ => panic!("Expected list inside quasiquote"),
        },
        _ => panic!("Expected quasiquote expression"),
    }
}

#[test]
fn test_parse_error_unterminated_list() {
    let tokens = tokenize("(+ 1 2").unwrap();
    let result = parse(tokens);
    assert!(result.is_err());
}

#[test]
fn test_parse_error_unexpected_rparen() {
    let tokens = tokenize(")").unwrap();
    let result = parse(tokens);
    assert!(result.is_err());
}