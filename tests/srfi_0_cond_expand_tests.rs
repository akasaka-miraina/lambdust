#![allow(clippy::uninlined_format_args)]
//! Tests for SRFI-0 cond-expand implementation.
//!
//! This module tests the parsing and basic functionality of
//! conditional expansion (cond-expand) forms.

use lambdust::{
    ast::{Expr, FeatureRequirement},
    lexer::Lexer,
    parser::Parser,
};

/// Test basic cond-expand parsing
#[test]
fn test_basic_cond_expand_parsing() {
    let source = r#"
    (cond-expand
      (lambdust
        (display "Running on Lambdust"))
      (else
        (display "Running on another implementation")))
    "#;

    let mut lexer = Lexer::new(source, Some("test"));
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    assert_eq!(program.expressions.len(), 1);

    match &program.expressions[0].inner {
        Expr::CondExpand {
            clauses,
            else_clause,
        } => {
            assert_eq!(clauses.len(), 1);
            assert!(else_clause.is_some());

            // Check the feature requirement
            match &clauses[0].feature_requirement {
                FeatureRequirement::Feature(name) => {
                    assert_eq!(name, "lambdust");
                }
                _ => panic!("Expected simple feature requirement"),
            }

            // Check body has one expression
            assert_eq!(clauses[0].body.len(), 1);

            // Check else clause has one expression
            let else_body = else_clause.as_ref().unwrap();
            assert_eq!(else_body.len(), 1);
        }
        _ => panic!("Expected CondExpand expression"),
    }
}

/// Test complex feature requirements with AND/OR/NOT
#[test]
fn test_complex_feature_requirements() {
    let source = r#"
    (cond-expand
      ((and lambdust (not debug))
        (display "Optimized Lambdust"))
      ((or srfi-1 builtin-lists)
        (display "Lists available"))
      (else
        (display "Fallback")))
    "#;

    let mut lexer = Lexer::new(source, Some("test"));
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    assert_eq!(program.expressions.len(), 1);

    match &program.expressions[0].inner {
        Expr::CondExpand {
            clauses,
            else_clause,
        } => {
            assert_eq!(clauses.len(), 2);
            assert!(else_clause.is_some());

            // Check first clause: (and lambdust (not debug))
            match &clauses[0].feature_requirement {
                FeatureRequirement::And(reqs) => {
                    assert_eq!(reqs.len(), 2);

                    match &reqs[0] {
                        FeatureRequirement::Feature(name) => assert_eq!(name, "lambdust"),
                        _ => panic!("Expected lambdust feature"),
                    }

                    match &reqs[1] {
                        FeatureRequirement::Not(inner) => match inner.as_ref() {
                            FeatureRequirement::Feature(name) => assert_eq!(name, "debug"),
                            _ => panic!("Expected debug feature in NOT"),
                        },
                        _ => panic!("Expected NOT requirement"),
                    }
                }
                _ => panic!("Expected AND requirement"),
            }

            // Check second clause: (or srfi-1 builtin-lists)
            match &clauses[1].feature_requirement {
                FeatureRequirement::Or(reqs) => {
                    assert_eq!(reqs.len(), 2);

                    match &reqs[0] {
                        FeatureRequirement::Feature(name) => assert_eq!(name, "srfi-1"),
                        _ => panic!("Expected srfi-1 feature"),
                    }

                    match &reqs[1] {
                        FeatureRequirement::Feature(name) => assert_eq!(name, "builtin-lists"),
                        _ => panic!("Expected builtin-lists feature"),
                    }
                }
                _ => panic!("Expected OR requirement"),
            }
        }
        _ => panic!("Expected CondExpand expression"),
    }
}

/// Test library identifiers in feature requirements
#[test]
fn test_library_identifiers() {
    let source = r#"
    (cond-expand
      ((srfi 1)
        (import (srfi 1)))
      ((lambdust core)
        (import (lambdust core)))
      (else
        (display "No libraries available")))
    "#;

    let mut lexer = Lexer::new(source, Some("test"));
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.expressions[0].inner {
        Expr::CondExpand { clauses, .. } => {
            assert_eq!(clauses.len(), 2);

            // Check first clause: (srfi 1)
            match &clauses[0].feature_requirement {
                FeatureRequirement::Library(components) => {
                    assert_eq!(components, &&["srfi".to_string(), "1".to_string()]);
                }
                _ => panic!("Expected Library requirement for (srfi 1)"),
            }

            // Check second clause: (lambdust core)
            match &clauses[1].feature_requirement {
                FeatureRequirement::Library(components) => {
                    assert_eq!(components, &&["lambdust".to_string(), "core".to_string()]);
                }
                _ => panic!("Expected Library requirement for (lambdust core)"),
            }
        }
        _ => panic!("Expected CondExpand expression"),
    }
}

/// Test cond-expand without else clause
#[test]
fn test_cond_expand_no_else() {
    let source = r#"
    (cond-expand
      (lambdust
        (display "Only on Lambdust")))
    "#;

    let mut lexer = Lexer::new(source, Some("test"));
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.expressions[0].inner {
        Expr::CondExpand {
            clauses,
            else_clause,
        } => {
            assert_eq!(clauses.len(), 1);
            assert!(else_clause.is_none());

            match &clauses[0].feature_requirement {
                FeatureRequirement::Feature(name) => assert_eq!(name, "lambdust"),
                _ => panic!("Expected simple feature requirement"),
            }
        }
        _ => panic!("Expected CondExpand expression"),
    }
}

/// Test multiple body expressions in clauses
#[test]
fn test_multiple_body_expressions() {
    let source = r#"
    (cond-expand
      (lambdust
        (define x 42)
        (display "Hello")
        (+ x 1))
      (else
        (display "Fallback")))
    "#;

    let mut lexer = Lexer::new(source, Some("test"));
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.expressions[0].inner {
        Expr::CondExpand {
            clauses,
            else_clause,
        } => {
            assert_eq!(clauses.len(), 1);
            assert!(else_clause.is_some());

            // Check body has three expressions
            assert_eq!(clauses[0].body.len(), 3);

            // Check else clause has one expression
            let else_body = else_clause.as_ref().unwrap();
            assert_eq!(else_body.len(), 1);
        }
        _ => panic!("Expected CondExpand expression"),
    }
}

/// Test error case: multiple else clauses
#[test]
fn test_error_multiple_else_clauses() {
    let source =
        "(cond-expand (lambdust (display \"A\")) (else (display \"B\")) (else (display \"C\")))";

    let mut lexer = Lexer::new(source, Some("test"));
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    // Should either fail parsing or have error recorded
    let errors = parser.errors();
    let has_multiple_else_error = errors.iter().any(|e| match e {
        lambdust::diagnostics::Error::ParseError { message, .. } => {
            message.contains("Multiple else clauses")
        }
        _ => false,
    });

    match result {
        Ok(_) => {
            // If parse succeeded, there should be an error in the error list
            assert!(
                has_multiple_else_error,
                "Expected 'Multiple else clauses' error"
            );
        }
        Err(_) => {
            // If parse failed, the error should be about multiple else clauses
            assert!(
                has_multiple_else_error,
                "Expected 'Multiple else clauses' error"
            );
        }
    }
}
