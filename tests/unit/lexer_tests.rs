//! Unit tests for the lexer module
//!
//! These tests verify the basic tokenization functionality of the Scheme lexer.

use lambdust::lexer::{tokenize, Token, SchemeNumber};

#[test]
fn test_basic_tokens() {
    let tokens = tokenize("()").unwrap();
    assert_eq!(tokens, vec![Token::LeftParen, Token::RightParen]);
}

#[test]
#[allow(clippy::approx_constant)]
fn test_numbers() {
    let tokens = tokenize("42 3.14 1/2").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Number(SchemeNumber::Integer(42)),
            Token::Number(SchemeNumber::Real(3.14)),
            Token::Number(SchemeNumber::Rational(1, 2)),
        ]
    );
}

#[test]
fn test_strings() {
    let tokens = tokenize("\"hello world\"").unwrap();
    assert_eq!(tokens, vec![Token::String("hello world".to_string())]);
}

#[test]
fn test_symbols() {
    let tokens = tokenize("+ define lambda").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Symbol("+".to_string()),
            Token::Symbol("define".to_string()),
            Token::Symbol("lambda".to_string()),
        ]
    );
}

#[test]
fn test_booleans() {
    let tokens = tokenize("#t #f").unwrap();
    assert_eq!(tokens, vec![Token::Boolean(true), Token::Boolean(false)]);
}

#[test]
fn test_quote_tokens() {
    let tokens = tokenize("'x `(,y ,@z)").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Quote,
            Token::Symbol("x".to_string()),
            Token::Quasiquote,
            Token::LeftParen,
            Token::Unquote,
            Token::Symbol("y".to_string()),
            Token::UnquoteSplicing,
            Token::Symbol("z".to_string()),
            Token::RightParen,
        ]
    );
}

#[test]
fn test_comments() {
    let tokens = tokenize("; This is a comment\n(+ 1 2)").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::LeftParen,
            Token::Symbol("+".to_string()),
            Token::Number(SchemeNumber::Integer(1)),
            Token::Number(SchemeNumber::Integer(2)),
            Token::RightParen,
        ]
    );
}