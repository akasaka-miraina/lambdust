//! Comprehensive unit tests for the parser module
//!
//! These tests verify the complete parsing functionality for converting tokens into AST expressions,
//! including edge cases, error conditions, and complex nested structures.

use lambdust::ast::{Expr, Literal};
use lambdust::lexer::{tokenize, SchemeNumber};
use lambdust::parser::{parse, parse_multiple, Parser, LoopDetectionConfig};
use lambdust::error::LambdustError;

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

// Extended comprehensive tests for better coverage

#[cfg(test)]
mod literal_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_integers() {
        let tokens = tokenize("42 -17 0 999").unwrap();
        let expressions = parse_multiple(tokens).unwrap();

        assert_eq!(expressions.len(), 4);
        assert_eq!(
            expressions[0],
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))
        );
        assert_eq!(
            expressions[1],
            Expr::Literal(Literal::Number(SchemeNumber::Integer(-17)))
        );
        assert_eq!(
            expressions[2],
            Expr::Literal(Literal::Number(SchemeNumber::Integer(0)))
        );
        assert_eq!(
            expressions[3],
            Expr::Literal(Literal::Number(SchemeNumber::Integer(999)))
        );
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_parse_real_numbers() {
        let tokens = tokenize("3.14159 -2.5 0.0").unwrap();
        let expressions = parse_multiple(tokens).unwrap();

        assert_eq!(expressions.len(), 3);
        assert_eq!(
            expressions[0],
            Expr::Literal(Literal::Number(SchemeNumber::Real(3.14159)))
        );
        assert_eq!(
            expressions[1],
            Expr::Literal(Literal::Number(SchemeNumber::Real(-2.5)))
        );
        assert_eq!(
            expressions[2],
            Expr::Literal(Literal::Number(SchemeNumber::Real(0.0)))
        );
    }

    #[test]
    fn test_parse_rational_numbers() {
        let tokens = tokenize("1/2 -3/4 0/1").unwrap();
        let expressions = parse_multiple(tokens).unwrap();

        assert_eq!(expressions.len(), 3);
        assert_eq!(
            expressions[0],
            Expr::Literal(Literal::Number(SchemeNumber::Rational(1, 2)))
        );
        assert_eq!(
            expressions[1],
            Expr::Literal(Literal::Number(SchemeNumber::Rational(-3, 4)))
        );
        assert_eq!(
            expressions[2],
            Expr::Literal(Literal::Number(SchemeNumber::Rational(0, 1)))
        );
    }

    #[test]
    fn test_parse_complex_numbers() {
        let tokens = tokenize("3i -2i 0i").unwrap();
        let expressions = parse_multiple(tokens).unwrap();

        assert_eq!(expressions.len(), 3);
        assert_eq!(
            expressions[0],
            Expr::Literal(Literal::Number(SchemeNumber::Complex(0.0, 3.0)))
        );
        assert_eq!(
            expressions[1],
            Expr::Literal(Literal::Number(SchemeNumber::Complex(0.0, -2.0)))
        );
        assert_eq!(
            expressions[2],
            Expr::Literal(Literal::Number(SchemeNumber::Complex(0.0, 0.0)))
        );
    }

    #[test]
    fn test_parse_booleans() {
        let tokens = tokenize("#t #f #true #false").unwrap();
        let expressions = parse_multiple(tokens).unwrap();

        assert_eq!(expressions.len(), 4);
        assert_eq!(expressions[0], Expr::Literal(Literal::Boolean(true)));
        assert_eq!(expressions[1], Expr::Literal(Literal::Boolean(false)));
        assert_eq!(expressions[2], Expr::Literal(Literal::Boolean(true)));
        assert_eq!(expressions[3], Expr::Literal(Literal::Boolean(false)));
    }

    #[test]
    fn test_parse_strings() {
        let tokens = tokenize("\"hello\" \"world\" \"\" \"with\\nescapes\"").unwrap();
        let expressions = parse_multiple(tokens).unwrap();

        assert_eq!(expressions.len(), 4);
        assert_eq!(
            expressions[0],
            Expr::Literal(Literal::String("hello".to_string()))
        );
        assert_eq!(
            expressions[1],
            Expr::Literal(Literal::String("world".to_string()))
        );
        assert_eq!(
            expressions[2],
            Expr::Literal(Literal::String("".to_string()))
        );
        assert_eq!(
            expressions[3],
            Expr::Literal(Literal::String("with\nescapes".to_string()))
        );
    }

    #[test]
    fn test_parse_characters() {
        let tokens = tokenize("#\\a #\\5 #\\space #\\newline").unwrap();
        let expressions = parse_multiple(tokens).unwrap();

        assert_eq!(expressions.len(), 4);
        assert_eq!(expressions[0], Expr::Literal(Literal::Character('a')));
        assert_eq!(expressions[1], Expr::Literal(Literal::Character('5')));
        assert_eq!(expressions[2], Expr::Literal(Literal::Character(' ')));
        assert_eq!(expressions[3], Expr::Literal(Literal::Character('\n')));
    }

    #[test]
    fn test_parse_symbols() {
        let tokens = tokenize("hello world + - * / lambda define").unwrap();
        let expressions = parse_multiple(tokens).unwrap();

        assert_eq!(expressions.len(), 8);
        assert_eq!(expressions[0], Expr::Variable("hello".to_string()));
        assert_eq!(expressions[1], Expr::Variable("world".to_string()));
        assert_eq!(expressions[2], Expr::Variable("+".to_string()));
        assert_eq!(expressions[3], Expr::Variable("-".to_string()));
        assert_eq!(expressions[4], Expr::Variable("*".to_string()));
        assert_eq!(expressions[5], Expr::Variable("/".to_string()));
        assert_eq!(expressions[6], Expr::Variable("lambda".to_string()));
        assert_eq!(expressions[7], Expr::Variable("define".to_string()));
    }
}

#[cfg(test)]
mod list_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_single_element_list() {
        let tokens = tokenize("(x)").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 1);
                assert_eq!(exprs[0], Expr::Variable("x".to_string()));
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_parse_mixed_type_list() {
        let tokens = tokenize("(42 \"hello\" #t x)").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 4);
                assert_eq!(
                    exprs[0],
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))
                );
                assert_eq!(
                    exprs[1],
                    Expr::Literal(Literal::String("hello".to_string()))
                );
                assert_eq!(exprs[2], Expr::Literal(Literal::Boolean(true)));
                assert_eq!(exprs[3], Expr::Variable("x".to_string()));
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_parse_deeply_nested_lists() {
        let tokens = tokenize("(((1 2) (3 4)) ((5 6) (7 8)))").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 2);
                
                // Check first nested structure
                match &exprs[0] {
                    Expr::List(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert!(matches!(inner[0], Expr::List(_)));
                        assert!(matches!(inner[1], Expr::List(_)));
                    }
                    _ => panic!("Expected nested list"),
                }

                // Check second nested structure
                match &exprs[1] {
                    Expr::List(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert!(matches!(inner[0], Expr::List(_)));
                        assert!(matches!(inner[1], Expr::List(_)));
                    }
                    _ => panic!("Expected nested list"),
                }
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_parse_list_with_various_elements() {
        let tokens = tokenize("(+ 1 2 (* 3 4) (- 5 6))").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 5);
                assert_eq!(exprs[0], Expr::Variable("+".to_string()));
                assert_eq!(
                    exprs[1],
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(1)))
                );
                assert_eq!(
                    exprs[2],
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(2)))
                );
                assert!(matches!(exprs[3], Expr::List(_)));
                assert!(matches!(exprs[4], Expr::List(_)));
            }
            _ => panic!("Expected list expression"),
        }
    }
}

#[cfg(test)]
mod dotted_list_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_simple_dotted_list() {
        let tokens = tokenize("(a . b)").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::DottedList(exprs, tail) => {
                assert_eq!(exprs.len(), 1);
                assert_eq!(exprs[0], Expr::Variable("a".to_string()));
                assert_eq!(*tail, Expr::Variable("b".to_string()));
            }
            _ => panic!("Expected dotted list expression"),
        }
    }

    #[test]
    fn test_parse_multi_element_dotted_list() {
        let tokens = tokenize("(a b c . d)").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::DottedList(exprs, tail) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(exprs[0], Expr::Variable("a".to_string()));
                assert_eq!(exprs[1], Expr::Variable("b".to_string()));
                assert_eq!(exprs[2], Expr::Variable("c".to_string()));
                assert_eq!(*tail, Expr::Variable("d".to_string()));
            }
            _ => panic!("Expected dotted list expression"),
        }
    }

    #[test]
    fn test_parse_dotted_list_with_nested_elements() {
        let tokens = tokenize("((a b) . (c d))").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::DottedList(exprs, tail) => {
                assert_eq!(exprs.len(), 1);
                assert!(matches!(exprs[0], Expr::List(_)));
                assert!(matches!(*tail, Expr::List(_)));
            }
            _ => panic!("Expected dotted list expression"),
        }
    }

    #[test]
    fn test_parse_dotted_list_with_literals() {
        let tokens = tokenize("(42 \"hello\" . #t)").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::DottedList(exprs, tail) => {
                assert_eq!(exprs.len(), 2);
                assert_eq!(
                    exprs[0],
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))
                );
                assert_eq!(
                    exprs[1],
                    Expr::Literal(Literal::String("hello".to_string()))
                );
                assert_eq!(*tail, Expr::Literal(Literal::Boolean(true)));
            }
            _ => panic!("Expected dotted list expression"),
        }
    }
}

#[cfg(test)]
mod quote_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_quoted_atom() {
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
    fn test_parse_quoted_list() {
        let tokens = tokenize("'(a b c)").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Quote(inner) => match inner.as_ref() {
                Expr::List(exprs) => {
                    assert_eq!(exprs.len(), 3);
                    assert_eq!(exprs[0], Expr::Variable("a".to_string()));
                    assert_eq!(exprs[1], Expr::Variable("b".to_string()));
                    assert_eq!(exprs[2], Expr::Variable("c".to_string()));
                }
                _ => panic!("Expected list inside quote"),
            },
            _ => panic!("Expected quote expression"),
        }
    }

    #[test]
    fn test_parse_quoted_number() {
        let tokens = tokenize("'42").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Quote(inner) => {
                assert_eq!(
                    *inner,
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))
                );
            }
            _ => panic!("Expected quote expression"),
        }
    }

    #[test]
    fn test_parse_nested_quotes() {
        let tokens = tokenize("''x").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Quote(inner) => match inner.as_ref() {
                Expr::Quote(inner2) => {
                    assert_eq!(**inner2, Expr::Variable("x".to_string()));
                }
                _ => panic!("Expected nested quote"),
            },
            _ => panic!("Expected quote expression"),
        }
    }

    #[test]
    fn test_parse_quoted_empty_list() {
        let tokens = tokenize("'()").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Quote(inner) => {
                assert_eq!(*inner, Expr::List(vec![]));
            }
            _ => panic!("Expected quote expression"),
        }
    }
}

#[cfg(test)]
mod quasiquote_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_simple_quasiquote() {
        let tokens = tokenize("`x").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Quasiquote(inner) => {
                assert_eq!(*inner, Expr::Variable("x".to_string()));
            }
            _ => panic!("Expected quasiquote expression"),
        }
    }

    #[test]
    fn test_parse_quasiquote_with_unquote() {
        let tokens = tokenize("`(a ,b c)").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Quasiquote(inner) => match inner.as_ref() {
                Expr::List(exprs) => {
                    assert_eq!(exprs.len(), 3);
                    assert_eq!(exprs[0], Expr::Variable("a".to_string()));
                    assert!(matches!(exprs[1], Expr::Unquote(_)));
                    assert_eq!(exprs[2], Expr::Variable("c".to_string()));
                }
                _ => panic!("Expected list inside quasiquote"),
            },
            _ => panic!("Expected quasiquote expression"),
        }
    }

    #[test]
    fn test_parse_quasiquote_with_unquote_splicing() {
        let tokens = tokenize("`(a ,@b c)").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Quasiquote(inner) => match inner.as_ref() {
                Expr::List(exprs) => {
                    assert_eq!(exprs.len(), 3);
                    assert_eq!(exprs[0], Expr::Variable("a".to_string()));
                    assert!(matches!(exprs[1], Expr::UnquoteSplicing(_)));
                    assert_eq!(exprs[2], Expr::Variable("c".to_string()));
                }
                _ => panic!("Expected list inside quasiquote"),
            },
            _ => panic!("Expected quasiquote expression"),
        }
    }

    #[test]
    fn test_parse_nested_quasiquotes() {
        let tokens = tokenize("``x").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Quasiquote(inner) => match inner.as_ref() {
                Expr::Quasiquote(inner2) => {
                    assert_eq!(**inner2, Expr::Variable("x".to_string()));
                }
                _ => panic!("Expected nested quasiquote"),
            },
            _ => panic!("Expected quasiquote expression"),
        }
    }

    #[test]
    fn test_parse_complex_quasiquote() {
        let tokens = tokenize("`(,x (,@y z) ,w)").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Quasiquote(inner) => match inner.as_ref() {
                Expr::List(exprs) => {
                    assert_eq!(exprs.len(), 3);
                    assert!(matches!(exprs[0], Expr::Unquote(_)));
                    assert!(matches!(exprs[1], Expr::List(_)));
                    assert!(matches!(exprs[2], Expr::Unquote(_)));
                }
                _ => panic!("Expected list inside quasiquote"),
            },
            _ => panic!("Expected quasiquote expression"),
        }
    }
}

#[cfg(test)]
mod vector_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_empty_vector() {
        let tokens = tokenize("#()").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Vector(exprs) => {
                assert_eq!(exprs.len(), 0);
            }
            _ => panic!("Expected vector expression"),
        }
    }

    #[test]
    fn test_parse_vector_with_elements() {
        let tokens = tokenize("#(1 2 3)").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Vector(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(
                    exprs[0],
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(1)))
                );
                assert_eq!(
                    exprs[1],
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(2)))
                );
                assert_eq!(
                    exprs[2],
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(3)))
                );
            }
            _ => panic!("Expected vector expression"),
        }
    }

    #[test]
    fn test_parse_vector_with_mixed_types() {
        let tokens = tokenize("#(42 \"hello\" #t x)").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Vector(exprs) => {
                assert_eq!(exprs.len(), 4);
                assert_eq!(
                    exprs[0],
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))
                );
                assert_eq!(
                    exprs[1],
                    Expr::Literal(Literal::String("hello".to_string()))
                );
                assert_eq!(exprs[2], Expr::Literal(Literal::Boolean(true)));
                assert_eq!(exprs[3], Expr::Variable("x".to_string()));
            }
            _ => panic!("Expected vector expression"),
        }
    }

    #[test]
    fn test_parse_vector_with_nested_structures() {
        let tokens = tokenize("#((1 2) '(a b) #(3 4))").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::Vector(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert!(matches!(exprs[0], Expr::List(_)));
                assert!(matches!(exprs[1], Expr::Quote(_)));
                assert!(matches!(exprs[2], Expr::Vector(_)));
            }
            _ => panic!("Expected vector expression"),
        }
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_parse_error_empty_input() {
        let tokens = Vec::new();
        let result = parse(tokens);
        assert!(result.is_err());
        match result {
            Err(LambdustError::ParseError { .. }) => {},
            _ => panic!("Expected ParseError for empty input"),
        }
    }

    #[test]
    fn test_parse_error_unmatched_left_paren() {
        let tokens = tokenize("(+ 1 2").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        match result {
            Err(LambdustError::ParseError { .. }) => {},
            _ => panic!("Expected ParseError for unmatched left paren"),
        }
    }

    #[test]
    fn test_parse_error_unmatched_right_paren() {
        let tokens = tokenize("+ 1 2)").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        match result {
            Err(LambdustError::ParseError { .. }) => {},
            _ => panic!("Expected ParseError for unmatched right paren"),
        }
    }

    #[test]
    fn test_parse_error_dot_at_beginning() {
        let tokens = tokenize("(. a b)").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        match result {
            Err(LambdustError::ParseError { .. }) => {},
            _ => panic!("Expected ParseError for dot at beginning"),
        }
    }

    #[test]
    fn test_parse_error_multiple_dots() {
        let tokens = tokenize("(a . b . c)").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        match result {
            Err(LambdustError::ParseError { .. }) => {},
            _ => panic!("Expected ParseError for multiple dots"),
        }
    }

    #[test]
    fn test_parse_error_missing_tail_after_dot() {
        let tokens = tokenize("(a b .)").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        match result {
            Err(LambdustError::ParseError { .. }) => {},
            _ => panic!("Expected ParseError for missing tail after dot"),
        }
    }

    #[test]
    fn test_parse_error_multiple_expressions_after_dot() {
        let tokens = tokenize("(a . b c)").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        match result {
            Err(LambdustError::ParseError { .. }) => {},
            _ => panic!("Expected ParseError for multiple expressions after dot"),
        }
    }

    #[test]
    fn test_parse_error_unterminated_vector() {
        let tokens = tokenize("#(1 2 3").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        match result {
            Err(LambdustError::ParseError { .. }) => {},
            _ => panic!("Expected ParseError for unterminated vector"),
        }
    }

    #[test]
    fn test_parse_error_incomplete_quote() {
        let tokens = vec![lambdust::lexer::Token::Quote];
        let result = parse(tokens);
        assert!(result.is_err());
        match result {
            Err(LambdustError::ParseError { .. }) => {},
            _ => panic!("Expected ParseError for incomplete quote"),
        }
    }

    #[test]
    fn test_parse_error_incomplete_quasiquote() {
        let tokens = vec![lambdust::lexer::Token::Quasiquote];
        let result = parse(tokens);
        assert!(result.is_err());
        match result {
            Err(LambdustError::ParseError { .. }) => {},
            _ => panic!("Expected ParseError for incomplete quasiquote"),
        }
    }

    #[test]
    fn test_parse_error_incomplete_unquote() {
        let tokens = vec![lambdust::lexer::Token::Unquote];
        let result = parse(tokens);
        assert!(result.is_err());
        match result {
            Err(LambdustError::ParseError { .. }) => {},
            _ => panic!("Expected ParseError for incomplete unquote"),
        }
    }

    #[test]
    fn test_parse_error_incomplete_unquote_splicing() {
        let tokens = vec![lambdust::lexer::Token::UnquoteSplicing];
        let result = parse(tokens);
        assert!(result.is_err());
        match result {
            Err(LambdustError::ParseError { .. }) => {},
            _ => panic!("Expected ParseError for incomplete unquote splicing"),
        }
    }
}

#[cfg(test)]
mod parser_direct_tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let tokens = tokenize("(+ 1 2)").unwrap();
        let _parser = Parser::new(tokens);
        // Basic creation test - just verify it doesn't panic
        // Test passes if no panic occurs
    }

    #[test]
    fn test_parser_with_loop_detection_config() {
        let tokens = tokenize("(+ 1 2)").unwrap();
        let config = LoopDetectionConfig::default();
        let _parser = Parser::with_loop_detection_config(tokens, config);
        // Test custom config creation
        // Test passes if no panic occurs
    }

    #[test]
    fn test_parser_parse_all() {
        let tokens = tokenize("1 2 3 (+ 4 5)").unwrap();
        let mut parser = Parser::new(tokens);
        let expressions = parser.parse_all().unwrap();

        assert_eq!(expressions.len(), 4);
        assert_eq!(
            expressions[0],
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1)))
        );
        assert_eq!(
            expressions[1],
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2)))
        );
        assert_eq!(
            expressions[2],
            Expr::Literal(Literal::Number(SchemeNumber::Integer(3)))
        );
        assert!(matches!(expressions[3], Expr::List(_)));
    }

    #[test]
    fn test_parser_parse_expression() {
        let tokens = tokenize("42").unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expression().unwrap();

        assert_eq!(
            expr,
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))
        );
    }
}

#[cfg(test)]
mod complex_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_lambda_expression() {
        let tokens = tokenize("(lambda (x y) (+ x y))").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(exprs[0], Expr::Variable("lambda".to_string()));
                assert!(matches!(exprs[1], Expr::List(_)));
                assert!(matches!(exprs[2], Expr::List(_)));
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_parse_define_expression() {
        // Use a simpler define expression to avoid triggering loop detection
        let tokens = tokenize("(define square (lambda (x) (* x x)))").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(exprs[0], Expr::Variable("define".to_string()));
                assert_eq!(exprs[1], Expr::Variable("square".to_string()));
                assert!(matches!(exprs[2], Expr::List(_)));
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_parse_if_expression() {
        let tokens = tokenize("(if (> x 0) 'positive 'non-positive)").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 4);
                assert_eq!(exprs[0], Expr::Variable("if".to_string()));
                assert!(matches!(exprs[1], Expr::List(_)));
                assert!(matches!(exprs[2], Expr::Quote(_)));
                assert!(matches!(exprs[3], Expr::Quote(_)));
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_parse_let_expression() {
        let tokens = tokenize("(let ((x 1) (y 2)) (+ x y))").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(exprs[0], Expr::Variable("let".to_string()));
                assert!(matches!(exprs[1], Expr::List(_)));
                assert!(matches!(exprs[2], Expr::List(_)));
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_parse_cond_expression() {
        let tokens = tokenize("(cond ((< x 0) 'negative) ((> x 0) 'positive) (else 'zero))").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 4);
                assert_eq!(exprs[0], Expr::Variable("cond".to_string()));
                assert!(matches!(exprs[1], Expr::List(_)));
                assert!(matches!(exprs[2], Expr::List(_)));
                assert!(matches!(exprs[3], Expr::List(_)));
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_parse_deeply_nested_expression() {
        let tokens = tokenize("(+ (- (* 2 3) (/ 8 4)) (+ 1 2))").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(exprs[0], Expr::Variable("+".to_string()));
                assert!(matches!(exprs[1], Expr::List(_)));
                assert!(matches!(exprs[2], Expr::List(_)));
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_parse_mixed_quotes_and_lists() {
        let tokens = tokenize("(list 'a '(b c) `(,x ,@y))").unwrap();
        let expr = parse(tokens).unwrap();

        match expr {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 4);
                assert_eq!(exprs[0], Expr::Variable("list".to_string()));
                assert!(matches!(exprs[1], Expr::Quote(_)));
                assert!(matches!(exprs[2], Expr::Quote(_)));
                assert!(matches!(exprs[3], Expr::Quasiquote(_)));
            }
            _ => panic!("Expected list expression"),
        }
    }
}

#[cfg(test)]
mod multiple_expression_tests {
    use super::*;

    #[test]
    fn test_parse_multiple_simple_expressions() {
        let tokens = tokenize("1 2 3").unwrap();
        let expressions = parse_multiple(tokens).unwrap();

        assert_eq!(expressions.len(), 3);
        assert_eq!(
            expressions[0],
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1)))
        );
        assert_eq!(
            expressions[1],
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2)))
        );
        assert_eq!(
            expressions[2],
            Expr::Literal(Literal::Number(SchemeNumber::Integer(3)))
        );
    }

    #[test]
    fn test_parse_multiple_mixed_expressions() {
        let tokens = tokenize("42 (+ 1 2) 'hello").unwrap();
        let expressions = parse_multiple(tokens).unwrap();

        assert_eq!(expressions.len(), 3);
        assert_eq!(
            expressions[0],
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))
        );
        assert!(matches!(expressions[1], Expr::List(_)));
        assert!(matches!(expressions[2], Expr::Quote(_)));
    }

    #[test]
    fn test_parse_multiple_complex_expressions() {
        let tokens = tokenize("(define x 42) (define y 24) (+ x y)").unwrap();
        let expressions = parse_multiple(tokens).unwrap();

        assert_eq!(expressions.len(), 3);
        for expr in expressions {
            assert!(matches!(expr, Expr::List(_)));
        }
    }

    #[test]
    fn test_parse_multiple_with_comments() {
        let tokens = tokenize("; This is a comment\n42\n; Another comment\n(+ 1 2)").unwrap();
        let expressions = parse_multiple(tokens).unwrap();

        assert_eq!(expressions.len(), 2);
        assert_eq!(
            expressions[0],
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))
        );
        assert!(matches!(expressions[1], Expr::List(_)));
    }
}
