//! Tests for SRFI-31: A special form `rec` for recursive evaluation
//!
//! This module tests the implementation of the `rec` special form which provides
//! syntactic sugar for simple recursive definitions.
//!
//! The `rec` form: (rec <variable> <expression>)
//! Is equivalent to: (letrec ((<variable> <expression>)) <variable>)

use lambdust::ast::Expr;
use lambdust::diagnostics::Result;
use lambdust::eval::value::Value;
use lambdust::lexer::Lexer;
use lambdust::parser::Parser;

/// Helper function to parse and evaluate a Scheme expression
fn eval_expression(source: &str) -> Result<Value> {
    let mut lexer = Lexer::new(source, Some("test"));
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_expression()?;

    // For testing purposes, we'll just verify the AST structure
    // In a real test, this would go through the full evaluator
    match ast.inner {
        Expr::LetRec { bindings, body } => {
            // Verify the desugaring worked correctly
            assert_eq!(bindings.len(), 1);
            assert_eq!(body.len(), 1);
            if let Expr::Identifier(name) = &body[0].inner {
                assert_eq!(&bindings[0].name, name);
            }
            Ok(Value::Nil) // Placeholder for successful parsing
        }
        _ => panic!("Expected LetRec after rec desugaring"),
    }
}

/// Helper function to parse an expression and check its structure
fn parse_expression(source: &str) -> Result<lambdust::ast::Spanned<Expr>> {
    let mut lexer = Lexer::new(source, Some("test"));
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_expression()
}

#[test]
fn test_rec_basic_parsing() {
    let source = "(rec f (lambda (x) x))";
    let result = parse_expression(source).unwrap();

    // Verify desugaring to letrec
    match result.inner {
        Expr::LetRec { bindings, body } => {
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].name, "f");
            assert_eq!(body.len(), 1);
            match &body[0].inner {
                Expr::Identifier(name) => assert_eq!(name, "f"),
                _ => panic!("Expected identifier in body"),
            }
        }
        _ => panic!("Expected LetRec after desugaring"),
    }
}

#[test]
fn test_rec_factorial_parsing() {
    let source = r#"(rec factorial 
                      (lambda (n) 
                        (if (= n 0) 
                            1 
                            (* n (factorial (- n 1))))))"#;
    let result = parse_expression(source).unwrap();

    // Verify correct desugaring
    match result.inner {
        Expr::LetRec { bindings, body } => {
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].name, "factorial");
            assert_eq!(body.len(), 1);
            match &body[0].inner {
                Expr::Identifier(name) => assert_eq!(name, "factorial"),
                _ => panic!("Expected identifier in body"),
            }

            // Verify the lambda structure
            match &bindings[0].value.inner {
                Expr::Lambda { .. } => {} // Lambda found as expected
                _ => panic!("Expected lambda in binding value"),
            }
        }
        _ => panic!("Expected LetRec after desugaring"),
    }
}

#[test]
fn test_rec_complex_expression() {
    let source = r#"(rec map-tree
                      (lambda (f tree)
                        (cond
                          ((null? tree) '())
                          ((pair? tree) (cons (map-tree f (car tree))
                                              (map-tree f (cdr tree))))
                          (else (f tree)))))"#;

    let result = parse_expression(source).unwrap();

    match result.inner {
        Expr::LetRec { bindings, body } => {
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].name, "map-tree");
            assert_eq!(body.len(), 1);
            match &body[0].inner {
                Expr::Identifier(name) => assert_eq!(name, "map-tree"),
                _ => panic!("Expected identifier in body"),
            }
        }
        _ => panic!("Expected LetRec after desugaring"),
    }
}

#[test]
#[should_panic(expected = "Expected identifier as first argument to rec")]
fn test_rec_invalid_variable_string() {
    let source = r#"(rec "not-identifier" (lambda (x) x))"#;
    parse_expression(source).unwrap();
}

#[test]
#[should_panic(expected = "Expected identifier as first argument to rec")]
fn test_rec_invalid_variable_number() {
    let source = r#"(rec 123 (lambda (x) x))"#;
    parse_expression(source).unwrap();
}

#[test]
fn test_rec_with_non_lambda() {
    // rec can work with any expression, not just lambdas
    let source = r#"(rec x 42)"#;
    let result = parse_expression(source).unwrap();

    match result.inner {
        Expr::LetRec { bindings, body } => {
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].name, "x");
            match &bindings[0].value.inner {
                Expr::Literal(_) => {} // Number literal as expected
                _ => panic!("Expected literal in binding value"),
            }
        }
        _ => panic!("Expected LetRec after desugaring"),
    }
}

#[test]
fn test_rec_nested() {
    // Test rec within another rec (though unusual)
    let source = r#"(rec outer (rec inner (lambda (x) x)))"#;
    let result = parse_expression(source).unwrap();

    match result.inner {
        Expr::LetRec { bindings, body } => {
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].name, "outer");
            // The inner rec should also be desugared to letrec
            match &bindings[0].value.inner {
                Expr::LetRec { .. } => {} // Nested letrec as expected
                _ => panic!("Expected nested LetRec in binding value"),
            }
        }
        _ => panic!("Expected LetRec after desugaring"),
    }
}

// Note: Full integration tests with the evaluator would go here
// These would test actual recursive execution, but require the full
// Lambdust runtime environment to be set up.

#[cfg(test)]
mod integration_tests {
    // These tests would require a full Lambdust interpreter instance
    // to test actual recursive execution behavior

    // #[test]
    // fn test_rec_factorial_execution() {
    //     let program = r#"
    //         (define fact (rec factorial
    //           (lambda (n)
    //             (if (= n 0) 1 (* n (factorial (- n 1)))))))
    //         (fact 5)
    //     "#;
    //     let result = run_program(program).unwrap();
    //     assert_eq!(result, Value::Integer(120));
    // }

    // #[test]
    // fn test_rec_fibonacci_execution() {
    //     let program = r#"
    //         (define fib (rec fibonacci
    //           (lambda (n)
    //             (if (<= n 1)
    //                 n
    //                 (+ (fibonacci (- n 1))
    //                    (fibonacci (- n 2)))))))
    //         (fib 10)
    //     "#;
    //     let result = run_program(program).unwrap();
    //     assert_eq!(result, Value::Integer(55));
    // }
}
