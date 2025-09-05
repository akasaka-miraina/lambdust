//! SRFI-10 External Forms Tests
//!
//! This module provides comprehensive tests for the SRFI-10 external form
//! implementation, covering lexical analysis, parsing, evaluation, and
//! runtime procedures.

use lambdust::ast::{Expr, Program};
use lambdust::eval::{Environment, Evaluator, Value};
use lambdust::lexer::{Lexer, TokenKind};
use lambdust::parser::Parser;
use lambdust::stdlib::srfi10_procedures::register_srfi10_procedures;
use lambdust::utils::SymbolId;
use std::rc::Rc;

/// Test helper to create an environment with SRFI-10 procedures registered.
fn create_srfi10_environment() -> Environment {
    let mut env = Environment::new();
    register_srfi10_procedures(&mut env).expect("Failed to register SRFI-10 procedures");
    env
}

/// Test helper to tokenize a string and check for expected tokens.
fn tokenize_and_check(source: &str, expected_tokens: Vec<TokenKind>) {
    let mut lexer = Lexer::new(source, Some("test"));
    let tokens = lexer.tokenize().expect("Tokenization failed");

    // Filter out EOF token for easier testing
    let tokens: Vec<_> = tokens
        .into_iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();

    assert_eq!(tokens.len(), expected_tokens.len());
    for (token, expected) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(token.kind, *expected);
    }
}

/// Test helper to parse a string into an AST.
fn parse_string(source: &str) -> Program {
    let mut lexer = Lexer::new(source, Some("test"));
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    parser.parse().expect("Parsing failed")
}

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_external_form_tokenization_basic() {
        tokenize_and_check(
            "#,(point 3 4)",
            vec![
                TokenKind::ExternalForm,
                TokenKind::LeftParen,
                TokenKind::Identifier,
                TokenKind::IntegerNumber,
                TokenKind::IntegerNumber,
                TokenKind::RightParen,
            ],
        );
    }

    #[test]
    fn test_external_form_tokenization_complex() {
        tokenize_and_check(
            r#"#,(date "2023-01-01" "ISO-8601")"#,
            vec![
                TokenKind::ExternalForm,
                TokenKind::LeftParen,
                TokenKind::Identifier,
                TokenKind::String,
                TokenKind::String,
                TokenKind::RightParen,
            ],
        );
    }

    #[test]
    fn test_external_form_with_nested_structure() {
        tokenize_and_check(
            "#,(complex-object (nested 1 2) (list 3 4 5))",
            vec![
                TokenKind::ExternalForm,
                TokenKind::LeftParen,
                TokenKind::Identifier,
                TokenKind::LeftParen,
                TokenKind::Identifier,
                TokenKind::IntegerNumber,
                TokenKind::IntegerNumber,
                TokenKind::RightParen,
                TokenKind::LeftParen,
                TokenKind::Identifier,
                TokenKind::IntegerNumber,
                TokenKind::IntegerNumber,
                TokenKind::IntegerNumber,
                TokenKind::RightParen,
                TokenKind::RightParen,
            ],
        );
    }

    #[test]
    fn test_multiple_external_forms() {
        tokenize_and_check(
            "#,(point 1 2) #,(color red) #,(size large)",
            vec![
                TokenKind::ExternalForm,
                TokenKind::LeftParen,
                TokenKind::Identifier,
                TokenKind::IntegerNumber,
                TokenKind::IntegerNumber,
                TokenKind::RightParen,
                TokenKind::ExternalForm,
                TokenKind::LeftParen,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::RightParen,
                TokenKind::ExternalForm,
                TokenKind::LeftParen,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::RightParen,
            ],
        );
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parse_external_form_basic() {
        let program = parse_string("#,(point 3 4)");
        assert_eq!(program.expressions.len(), 1);

        match &program.expressions[0].inner {
            Expr::ExternalForm { tag, args } => {
                assert_eq!(tag, "point");
                assert_eq!(args.len(), 2);

                match &args[0].inner {
                    Expr::Literal(lit) => {
                        // Check that first arg is number 3
                        assert!(matches!(lit.to_string().as_str(), "3"));
                    }
                    _ => panic!("Expected literal for first argument"),
                }

                match &args[1].inner {
                    Expr::Literal(lit) => {
                        // Check that second arg is number 4
                        assert!(matches!(lit.to_string().as_str(), "4"));
                    }
                    _ => panic!("Expected literal for second argument"),
                }
            }
            _ => panic!("Expected ExternalForm expression"),
        }
    }

    #[test]
    fn test_parse_external_form_with_strings() {
        let program = parse_string(r#"#,(person "Alice" "Bob")"#);
        assert_eq!(program.expressions.len(), 1);

        match &program.expressions[0].inner {
            Expr::ExternalForm { tag, args } => {
                assert_eq!(tag, "person");
                assert_eq!(args.len(), 2);

                match &args[0].inner {
                    Expr::Literal(lit) => {
                        assert!(lit.to_string().contains("Alice"));
                    }
                    _ => panic!("Expected literal for first argument"),
                }
            }
            _ => panic!("Expected ExternalForm expression"),
        }
    }

    #[test]
    fn test_parse_external_form_nested() {
        let program = parse_string("#,(container (list 1 2 3) (pair a b))");
        assert_eq!(program.expressions.len(), 1);

        match &program.expressions[0].inner {
            Expr::ExternalForm { tag, args } => {
                assert_eq!(tag, "container");
                assert_eq!(args.len(), 2);

                // First argument should be a list expression
                match &args[0].inner {
                    Expr::Application {
                        function,
                        arguments,
                    } => {
                        match &function.inner {
                            Expr::Identifier(name) => assert_eq!(name, "list"),
                            _ => panic!("Expected identifier 'list'"),
                        }
                        assert_eq!(arguments.len(), 3);
                    }
                    _ => panic!("Expected application for first argument"),
                }
            }
            _ => panic!("Expected ExternalForm expression"),
        }
    }

    #[test]
    fn test_parse_external_form_error_no_paren() {
        let mut lexer = Lexer::new("#,point", Some("test"));
        let tokens = lexer.tokenize().expect("Tokenization failed");
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_external_form_error_no_tag() {
        let mut lexer = Lexer::new("#,()", Some("test"));
        let tokens = lexer.tokenize().expect("Tokenization failed");
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_external_form_error_no_closing_paren() {
        let mut lexer = Lexer::new("#,(point 1 2", Some("test"));
        let tokens = lexer.tokenize().expect("Tokenization failed");
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod registry_tests {
    use super::*;
    use lambdust::stdlib::srfi10_external_forms::ExternalFormRegistry;

    #[test]
    fn test_registry_basic_operations() {
        let registry = ExternalFormRegistry::new();
        let tag = lambdust::utils::intern_symbol("test-tag");
        let constructor = Value::Primitive {
            name: "test-constructor".to_string(),
            arity: 2,
            pointer: std::ptr::null(),
        };

        // Initially empty
        assert_eq!(registry.len().unwrap(), 0);
        assert!(registry.is_empty().unwrap());
        assert!(!registry.has_constructor(tag).unwrap());

        // Register constructor
        assert!(
            registry
                .define_constructor(tag, constructor.clone())
                .is_ok()
        );
        assert_eq!(registry.len().unwrap(), 1);
        assert!(!registry.is_empty().unwrap());
        assert!(registry.has_constructor(tag).unwrap());

        // Retrieve constructor
        let retrieved = registry.get_constructor(tag).unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_registry_invalid_constructor() {
        let registry = ExternalFormRegistry::new();
        let tag = lambdust::utils::intern_symbol("test-tag");
        let invalid_constructor = Value::Integer(42);

        // Should reject non-procedure values
        assert!(
            registry
                .define_constructor(tag, invalid_constructor)
                .is_err()
        );
    }

    #[test]
    fn test_registry_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let registry = Arc::new(ExternalFormRegistry::new());
        let mut handles = &[];

        // Spawn multiple threads to register constructors
        for i in 0..5 {
            let registry = Arc::clone(&registry);
            let handle = thread::spawn(move || {
                let tag = lambdust::utils::intern_symbol(&format!("tag-{}", i));
                let constructor = Value::Primitive {
                    name: format!("constructor-{}", i),
                    arity: 1,
                    pointer: std::ptr::null(),
                };
                registry.define_constructor(tag, constructor)
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            assert!(handle.join().unwrap().is_ok());
        }

        // Verify all constructors were registered
        assert_eq!(registry.len().unwrap(), 5);
    }
}

#[cfg(test)]
mod procedure_tests {
    use super::*;

    #[test]
    fn test_define_reader_ctor_basic() {
        let env = create_srfi10_environment();

        // Test basic constructor definition
        let tag = Value::symbol("test-tag".to_string());
        let constructor = Value::Primitive {
            name: "test-constructor".to_string(),
            arity: 2,
            pointer: std::ptr::null(),
        };

        let args = &[tag, constructor];
        let result = lambdust::stdlib::srfi10_procedures::define_reader_ctor(&args, &env);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Unspecified));
    }

    #[test]
    fn test_define_reader_ctor_errors() {
        let env = create_srfi10_environment();

        // Test wrong number of arguments
        let result =
            lambdust::stdlib::srfi10_procedures::define_reader_ctor(&[Value::Integer(42)], &env);
        assert!(result.is_err());

        // Test invalid tag type
        let invalid_tag = Value::Integer(42);
        let constructor = Value::Primitive {
            name: "test-constructor".to_string(),
            arity: 1,
            pointer: std::ptr::null(),
        };
        let result = lambdust::stdlib::srfi10_procedures::define_reader_ctor(
            &[invalid_tag, constructor],
            &env,
        );
        assert!(result.is_err());

        // Test invalid constructor type
        let tag = Value::symbol("test-tag".to_string());
        let invalid_constructor = Value::Integer(42);
        let result = lambdust::stdlib::srfi10_procedures::define_reader_ctor(
            &[tag, invalid_constructor],
            &env,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_reader_ctor_predicate() {
        let env = create_srfi10_environment();
        let tag = Value::symbol("test-tag".to_string());

        // Initially should return false
        let result =
            lambdust::stdlib::srfi10_procedures::reader_ctor_predicate(&[tag.clone()], &env);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Boolean(false)));

        // Register a constructor
        let constructor = Value::Primitive {
            name: "test-constructor".to_string(),
            arity: 1,
            pointer: std::ptr::null(),
        };
        lambdust::stdlib::srfi10_procedures::define_reader_ctor(&[tag.clone(), constructor], &env)
            .unwrap();

        // Now should return true
        let result = lambdust::stdlib::srfi10_procedures::reader_ctor_predicate(&[tag], &env);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Boolean(true)));
    }

    #[test]
    fn test_reader_ctor_ref() {
        let env = create_srfi10_environment();
        let tag = Value::symbol("test-tag".to_string());
        let constructor = Value::Primitive {
            name: "test-constructor".to_string(),
            arity: 1,
            pointer: std::ptr::null(),
        };

        // Initially should fail
        let result = lambdust::stdlib::srfi10_procedures::reader_ctor_ref(&[tag.clone()], &env);
        assert!(result.is_err());

        // Register a constructor
        lambdust::stdlib::srfi10_procedures::define_reader_ctor(
            &[tag.clone(), constructor.clone()],
            &env,
        )
        .unwrap();

        // Now should return the constructor
        let result = lambdust::stdlib::srfi10_procedures::reader_ctor_ref(&[tag], &env);
        assert!(result.is_ok());
        // Constructor should be retrievable (exact equality testing would require more setup)
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_external_form_parsing_integration() {
        // Test that we can parse multiple different external forms
        let source = r#"
            #,(point 3 4)
            #,(color "red")
            #,(complex-object 
                (nested-data 1 2 3)
                #:metadata "value")
        "#;

        let program = parse_string(source);
        assert_eq!(program.expressions.len(), 3);

        // First expression: #,(point 3 4)
        match &program.expressions[0].inner {
            Expr::ExternalForm { tag, args } => {
                assert_eq!(tag, "point");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected ExternalForm for first expression"),
        }

        // Second expression: #,(color "red")
        match &program.expressions[1].inner {
            Expr::ExternalForm { tag, args } => {
                assert_eq!(tag, "color");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected ExternalForm for second expression"),
        }

        // Third expression: complex object
        match &program.expressions[2].inner {
            Expr::ExternalForm { tag, args } => {
                assert_eq!(tag, "complex-object");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected ExternalForm for third expression"),
        }
    }

    #[test]
    fn test_srfi10_procedure_registration() {
        let mut env = Environment::new();
        assert!(register_srfi10_procedures(&mut env).is_ok());

        // Check that all procedures were registered with correct names
        assert!(env.lookup("define-reader-ctor").is_some());
        assert!(env.lookup("reader-ctor?").is_some());
        assert!(env.lookup("reader-ctor-ref").is_some());
    }

    #[test]
    fn test_global_registry_singleton() {
        use lambdust::stdlib::srfi10_external_forms::global_registry;

        let registry1 = global_registry();
        let registry2 = global_registry();

        // Should be the same instance
        assert!(std::ptr::eq(registry1, registry2));
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_malformed_external_forms() {
        // Test various malformed external form syntaxes
        let malformed_cases = vec![
            "#,",     // No form at all
            "#,()",   // Empty form (no tag)
            "#,(tag", // Unclosed form
            "#,tag",  // No parentheses
        ];

        for case in malformed_cases {
            let mut lexer = Lexer::new(case, Some("test"));
            if let Ok(tokens) = lexer.tokenize() {
                let mut parser = Parser::new(tokens);
                let result = parser.parse();
                // Should either fail to tokenize or fail to parse
                if result.is_ok() {
                    panic!("Expected parsing to fail for malformed case: {}", case);
                }
            }
            // If tokenization fails, that's also acceptable for malformed input
        }
    }

    #[test]
    fn test_unregistered_constructor_error() {
        // This test would require a full evaluator setup to test runtime errors
        // For now, we verify that the registry correctly reports missing constructors
        use lambdust::stdlib::srfi10_external_forms::global_registry;

        let registry = global_registry();
        let nonexistent_tag = lambdust::utils::intern_symbol("nonexistent-tag");

        let result = registry.get_constructor(nonexistent_tag);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_procedure_argument_validation() {
        let env = create_srfi10_environment();

        // Test wrong number of arguments for each procedure
        let procedures = [
            lambdust::stdlib::srfi10_procedures::define_reader_ctor,
            lambdust::stdlib::srfi10_procedures::reader_ctor_predicate,
            lambdust::stdlib::srfi10_procedures::reader_ctor_ref,
        ];

        for proc in procedures.iter() {
            // Test with wrong number of arguments
            let result = proc(&[], &env);
            assert!(result.is_err(), "Procedure should reject empty arguments");

            let too_many_args = &[Value::integer(1), Value::integer(2), Value::integer(3)];
            let result = proc(&too_many_args, &env);
            assert!(
                result.is_err(),
                "Procedure should reject too many arguments"
            );
        }
    }
}
