//! SRFI-16 (case-lambda) Comprehensive Test Suite
//!
//! This module provides comprehensive tests for SRFI-16 compliance,
//! covering all aspects of case-lambda syntax and semantics.
//!
//! Reference: https://srfi.schemers.org/srfi-16/srfi-16.html

use lambdust::{
    ast::{Expr, Formals},
    diagnostics::Spanned,
    eval::{Evaluator, Value},
    lexer::Lexer,
    parser::Parser,
};

/// Helper function to parse case-lambda expressions
fn parse_case_lambda(source: &str) -> Result<Spanned<Expr>, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(source, Some("test"));
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    if program.expressions.is_empty() {
        return Err("No expressions parsed".into());
    }

    Ok(program.expressions.into_iter().next().unwrap())
}

/// Helper function to evaluate case-lambda expressions
fn eval_case_lambda(source: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(source, Some("test"));
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // Use the pre-initialized global environment which includes standard library
    // This is the same environment used by the regular evaluator
    let mut evaluator = Evaluator::new();

    let result = evaluator.eval_program(&program)?;
    Ok(result)
}

/// SRFI-16 Syntax Parsing Tests
mod syntax_parsing_tests {
    use super::*;

    #[test]
    fn test_empty_case_lambda_rejected() {
        // SRFI-16: case-lambda must have at least one clause
        let result = parse_case_lambda("(case-lambda)");
        assert!(result.is_err(), "Empty case-lambda should be rejected");
    }

    #[test]
    fn test_single_clause_case_lambda() {
        // SRFI-16: Single clause case-lambda
        let expr = parse_case_lambda("(case-lambda ((x) x))").unwrap();

        if let Expr::CaseLambda { clauses, .. } = expr.inner {
            assert_eq!(clauses.len(), 1);

            let clause = &clauses[0];
            match &clause.formals {
                Formals::Fixed(params) => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0], "x");
                }
                _ => panic!("Expected Fixed formals"),
            }
            assert_eq!(clause.body.len(), 1);
        } else {
            panic!("Expected CaseLambda expression");
        }
    }

    #[test]
    fn test_multiple_clause_case_lambda() {
        // SRFI-16: Multiple clauses with different arities
        let expr = parse_case_lambda(
            r#"
            (case-lambda
              (() 'none)
              ((x) x)
              ((x y) (+ x y))
              (args (apply + args)))
        "#,
        )
        .unwrap();

        if let Expr::CaseLambda { clauses, .. } = expr.inner {
            assert_eq!(clauses.len(), 4);

            // Check empty formals
            match &clauses[0].formals {
                Formals::Fixed(params) => assert!(params.is_empty()),
                _ => panic!("Expected Fixed formals for empty clause"),
            }

            // Check single parameter
            match &clauses[1].formals {
                Formals::Fixed(params) => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0], "x");
                }
                _ => panic!("Expected Fixed formals for single parameter"),
            }

            // Check two parameters
            match &clauses[2].formals {
                Formals::Fixed(params) => {
                    assert_eq!(params.len(), 2);
                    assert_eq!(params[0], "x");
                    assert_eq!(params[1], "y");
                }
                _ => panic!("Expected Fixed formals for two parameters"),
            }

            // Check variadic parameters
            match &clauses[3].formals {
                Formals::Variable(param) => {
                    assert_eq!(param, "args");
                }
                _ => panic!("Expected Variable formals for variadic clause"),
            }
        } else {
            panic!("Expected CaseLambda expression");
        }
    }

    #[test]
    fn test_mixed_parameters_case_lambda() {
        // SRFI-16: Mixed parameters (x y . rest)
        let expr = parse_case_lambda("(case-lambda ((x y . rest) (cons (+ x y) rest)))").unwrap();

        if let Expr::CaseLambda { clauses, .. } = expr.inner {
            assert_eq!(clauses.len(), 1);

            match &clauses[0].formals {
                Formals::Mixed { fixed, rest } => {
                    assert_eq!(fixed.len(), 2);
                    assert_eq!(fixed[0], "x");
                    assert_eq!(fixed[1], "y");
                    assert_eq!(rest, "rest");
                }
                _ => panic!("Expected Mixed formals"),
            }
        } else {
            panic!("Expected CaseLambda expression");
        }
    }

    #[test]
    fn test_empty_body_rejected() {
        // SRFI-16: Each clause must have at least one body expression
        let result = parse_case_lambda("(case-lambda ((x)))");
        assert!(result.is_err(), "Empty body should be rejected");
    }

    #[test]
    fn test_multiple_body_expressions() {
        // SRFI-16: Multiple body expressions (implicit begin)
        let expr = parse_case_lambda("(case-lambda ((x) (display x) (newline) x))").unwrap();

        if let Expr::CaseLambda { clauses, .. } = expr.inner {
            assert_eq!(clauses.len(), 1);
            assert_eq!(clauses[0].body.len(), 3);
        } else {
            panic!("Expected CaseLambda expression");
        }
    }
}

/// SRFI-16 Evaluation and Dispatch Tests
mod evaluation_tests {
    use super::*;

    #[test]
    fn test_debug_cons_availability() {
        // Debug test to check if cons is available in the environment
        let source = "(cons 1 2)";
        let result = eval_case_lambda(source);
        match &result {
            Ok(value) => println!("cons works: {value:?}"),
            Err(e) => println!("cons failed: {e}"),
        }
        assert!(result.is_ok(), "cons should be available");
    }

    #[test]
    fn test_zero_argument_dispatch() {
        // Test dispatch to zero-argument clause
        let source = r#"
            (define plus
              (case-lambda
                (() 0)
                ((x) x)
                ((x y) (+ x y))))
            (plus)
        "#;

        let result = eval_case_lambda(source).unwrap();
        if let Some(n) = result.as_integer() {
            assert_eq!(n, 0);
        } else {
            panic!("Expected integer 0, got {result:?}");
        }
    }

    #[test]
    fn test_single_argument_dispatch() {
        // Test dispatch to single-argument clause
        let source = r#"
            (define plus
              (case-lambda
                (() 0)
                ((x) x)
                ((x y) (+ x y))))
            (plus 42)
        "#;

        let result = eval_case_lambda(source).unwrap();
        if let Some(n) = result.as_integer() {
            assert_eq!(n, 42);
        } else {
            panic!("Expected integer 42, got {result:?}");
        }
    }

    #[test]
    fn test_two_argument_dispatch() {
        // Test dispatch to two-argument clause
        let source = r#"
            (define plus
              (case-lambda
                (() 0)
                ((x) x)
                ((x y) (+ x y))))
            (plus 10 32)
        "#;

        let result = eval_case_lambda(source).unwrap();
        if let Some(n) = result.as_integer() {
            assert_eq!(n, 42);
        } else {
            panic!("Expected integer 42, got {result:?}");
        }
    }

    #[test]
    fn test_variadic_argument_dispatch() {
        // Test dispatch to variadic clause
        let source = r#"
            (define plus
              (case-lambda
                (() 0)
                ((x) x)
                ((x y) (+ x y))
                (args (apply + args))))
            (plus 1 2 3 4 5)
        "#;

        let result = eval_case_lambda(source).unwrap();
        if let Some(n) = result.as_integer() {
            assert_eq!(n, 15);
        } else {
            panic!("Expected integer 15, got {result:?}");
        }
    }

    #[test]
    fn test_mixed_parameter_dispatch() {
        // Test dispatch to mixed parameter clause (x y . rest)
        let source = r#"
            (define list-proc
              (case-lambda
                ((x y . rest) (cons x (cons y rest)))))
            (list-proc 1 2 3 4 5)
        "#;

        let result = eval_case_lambda(source).unwrap();
        // Should return (1 2 3 4 5)
        if let Some(list) = result.as_list() {
            assert_eq!(list.len(), 5);
            // Verify the list contents
            for (i, val) in list.iter().enumerate() {
                if let Some(n) = val.as_integer() {
                    assert_eq!(n, (i + 1) as i64);
                } else {
                    panic!("Expected integer at position {i}, got {val:?}");
                }
            }
        } else {
            panic!("Expected list, got {result:?}");
        }
    }

    #[test]
    fn test_no_matching_clause_error() {
        // Test error when no clause matches the argument count
        let source = r#"
            (define strict-plus
              (case-lambda
                (() 0)
                ((x y) (+ x y))))
            (strict-plus 1)
        "#;

        let result = eval_case_lambda(source);
        assert!(result.is_err(), "Should produce error for unmatched arity");

        let error_msg = format!("{}", result.unwrap_err());
        assert!(
            error_msg.contains("no clause matches"),
            "Error message should indicate no matching clause"
        );
    }

    #[test]
    fn test_clause_order_precedence() {
        // Test that clauses are tried in order (first match wins)
        let source = r#"
            (define test-order
              (case-lambda
                ((x) 'specific)
                (args 'variadic)))
            (test-order 42)
        "#;

        let result = eval_case_lambda(source).unwrap();
        if let Some(sym_id) = result.as_symbol() {
            if let Some(name) = lambdust::utils::symbol_name(sym_id) {
                assert_eq!(name, "specific");
            } else {
                panic!("Could not get symbol name for {sym_id:?}");
            }
        } else {
            panic!("Expected symbol 'specific', got {result:?}");
        }
    }
}

/// SRFI-16 Compatibility and Edge Case Tests
mod compatibility_tests {
    use super::*;

    #[test]
    fn test_nested_case_lambda() {
        // Test case-lambda returning another case-lambda
        let source = r#"
            (define make-adder
              (case-lambda
                (() (case-lambda ((x) x)))
                ((n) (case-lambda ((x) (+ x n))))))
            ((make-adder 10) 5)
        "#;

        let result = eval_case_lambda(source).unwrap();
        if let Some(n) = result.as_integer() {
            assert_eq!(n, 15);
        } else {
            panic!("Expected integer 15, got {result:?}");
        }
    }

    #[test]
    fn test_case_lambda_as_argument() {
        // Test passing case-lambda as argument to other procedures
        let source = r#"
            (define apply-twice
              (lambda (f x)
                (f (f x))))
            
            (define increment
              (case-lambda
                ((x) (+ x 1))))
            
            (apply-twice increment 5)
        "#;

        let result = eval_case_lambda(source).unwrap();
        if let Some(n) = result.as_integer() {
            assert_eq!(n, 7);
        } else {
            panic!("Expected integer 7, got {result:?}");
        }
    }

    #[test]
    fn test_recursive_case_lambda() {
        // Test recursive case-lambda procedures
        let source = r#"
            (define factorial
              (case-lambda
                ((n) 
                 (if (<= n 1)
                     1
                     (* n (factorial (- n 1)))))))
            (factorial 5)
        "#;

        let result = eval_case_lambda(source).unwrap();
        if let Some(n) = result.as_integer() {
            assert_eq!(n, 120);
        } else {
            panic!("Expected integer 120, got {result:?}");
        }
    }

    #[test]
    fn test_case_lambda_with_closures() {
        // Test case-lambda with closure over lexical environment
        let source = r#"
            (define make-counter
              (lambda (initial)
                (let ((count initial))
                  (case-lambda
                    (() count)
                    ((n) (set! count (+ count n)) count)))))
            
            (define counter (make-counter 10))
            (counter 5)
        "#;

        let result = eval_case_lambda(source).unwrap();
        if let Some(n) = result.as_integer() {
            assert_eq!(n, 15);
        } else {
            panic!("Expected integer 15, got {result:?}");
        }
    }
}

/// SRFI-16 Error Handling Tests
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_descriptive_error_messages() {
        // Test that error messages are descriptive and helpful
        let source = r#"
            (define strict-binary
              (case-lambda
                ((x y) (+ x y))))
            (strict-binary 1 2 3)
        "#;

        let result = eval_case_lambda(source);
        assert!(result.is_err());

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("case-lambda"));
        assert!(error_msg.contains("3 arguments"));
        assert!(error_msg.contains("clause"));
    }

    #[test]
    fn test_multiple_clause_error_information() {
        // Test error message includes information about all available clauses
        let source = r#"
            (define multi-clause
              (case-lambda
                (() 'zero)
                ((x y) 'two)
                ((x y z w) 'four)))
            (multi-clause 1 2 3)
        "#;

        let result = eval_case_lambda(source);
        assert!(result.is_err());

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("3 arguments"));
        assert!(error_msg.contains("clause"));
    }
}

/// SRFI-16 Performance and Memory Tests
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_case_lambda_compilation() {
        // Test that large case-lambda expressions can be parsed efficiently
        let mut source = String::from("(case-lambda\n");

        // Generate many clauses
        for i in 0..100 {
            source.push_str(&"  ((".to_string());
            for j in 0..i {
                source.push_str(&format!("x{j} "));
            }
            source.push_str(&format!(") {i})\n"));
        }
        source.push(')');

        let result = parse_case_lambda(&source);
        assert!(
            result.is_ok(),
            "Large case-lambda should parse successfully"
        );

        if let Ok(expr) = result {
            if let Expr::CaseLambda { clauses, .. } = expr.inner {
                assert_eq!(clauses.len(), 100);
            }
        }
    }

    #[test]
    fn test_dispatch_performance() {
        // Test that dispatch to the correct clause is efficient
        // This is more of a smoke test - actual performance measurement
        // would require benchmarking infrastructure
        let source = r#"
            (define big-case
              (case-lambda
                ((a) 1)
                ((a b) 2)
                ((a b c) 3)
                ((a b c d) 4)
                ((a b c d e) 5)
                ((a b c d e f) 6)
                ((a b c d e f g) 7)
                ((a b c d e f g h) 8)
                ((a b c d e f g h i) 9)
                ((a b c d e f g h i j) 10)))
            (big-case 1 2 3 4 5 6 7 8 9 10)
        "#;

        let result = eval_case_lambda(source).unwrap();
        if let Some(n) = result.as_integer() {
            assert_eq!(n, 10);
        } else {
            panic!("Expected integer 10, got {result:?}");
        }
    }
}
