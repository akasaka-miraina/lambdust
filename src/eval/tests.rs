//! Unit tests for the evaluation engine.

use super::*;
use crate::ast::*;
use crate::diagnostics::{Span, Spanned};
use std::rc::Rc;
use std::collections::HashMap;

/// Helper function to create a test span.
fn test_span() -> Span {
    Span::new(0, 1)
}

/// Helper function to create a spanned expression.
fn spanned<T>(inner: T) -> Spanned<T> {
    Spanned {
        inner,
        span: test_span(),
    }
}

/// Helper to create a test environment with basic arithmetic operations.
fn test_env() -> Rc<Environment> {
    environment::global_environment()
}

mod literal_evaluation {
    use super::*;

    #[test]
    fn test_number_literal() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        let expr = spanned(Expr::Literal(Literal::integer(42)));
        let result = evaluator.eval(&expr, env).unwrap();
        
        match result {
            Value::Literal(Literal::Number(n)) if n.fract() == 0.0 => assert_eq!(n as i64, 42),
            _ => panic!("Expected integer literal, got: {:?}", result),
        }
    }

    #[test]
    fn test_real_literal() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        let expr = spanned(Expr::Literal(Literal::float(3.4)));
        let result = evaluator.eval(&expr, env).unwrap();
        
        match result {
            Value::Literal(Literal::Number(f)) => assert!((f - 3.4).abs() < f64::EPSILON),
            _ => panic!("Expected real literal, got: {result:?}"),
        }
    }

    #[test]
    fn test_string_literal() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        let expr = spanned(Expr::Literal(Literal::string("hello")));
        let result = evaluator.eval(&expr, env).unwrap();
        
        match result {
            Value::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected string literal, got: {:?}", result),
        }
    }

    #[test]
    fn test_boolean_literals() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        let expr_true = spanned(Expr::Literal(Literal::boolean(true)));
        let result_true = evaluator.eval(&expr_true, env.clone()).unwrap();
        
        match result_true {
            Value::Literal(Literal::Boolean(b)) => assert!(b),
            _ => panic!("Expected boolean true, got: {:?}", result_true),
        }
        
        let expr_false = spanned(Expr::Literal(Literal::boolean(false)));
        let result_false = evaluator.eval(&expr_false, env).unwrap();
        
        match result_false {
            Value::Literal(Literal::Boolean(b)) => assert!(!b),
            _ => panic!("Expected boolean false, got: {:?}", result_false),
        }
    }

    #[test]
    fn test_character_literals() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // Test basic character
        let expr_char = spanned(Expr::Literal(Literal::character('a')));
        let result_char = evaluator.eval(&expr_char, env.clone()).unwrap();
        
        match result_char {
            Value::Literal(Literal::Character(ch)) => assert_eq!(ch, 'a'),
            _ => panic!("Expected character 'a', got: {:?}", result_char),
        }
        
        // Test special character (space)
        let expr_space = spanned(Expr::Literal(Literal::character(' ')));
        let result_space = evaluator.eval(&expr_space, env.clone()).unwrap();
        
        match result_space {
            Value::Literal(Literal::Character(ch)) => assert_eq!(ch, ' '),
            _ => panic!("Expected character ' ', got: {:?}", result_space),
        }
        
        // Test Unicode character
        let expr_emoji = spanned(Expr::Literal(Literal::character('😀')));
        let result_emoji = evaluator.eval(&expr_emoji, env).unwrap();
        
        match result_emoji {
            Value::Literal(Literal::Character(ch)) => assert_eq!(ch, '😀'),
            _ => panic!("Expected character '😀', got: {:?}", result_emoji),
        }
    }
}

mod variable_operations {
    use super::*;

    #[test]
    fn test_define_and_lookup() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // (define x 42)
        let define_expr = spanned(Expr::Define {
            name: "x".to_string(),
            value: Box::new(spanned(Expr::Literal(Literal::integer(42)))),
            metadata: HashMap::new(),
        });
        
        let define_result = evaluator.eval(&define_expr, env.clone()).unwrap();
        assert!(matches!(define_result, Value::Unspecified));
        
        // x
        let lookup_expr = spanned(Expr::Identifier("x".to_string()));
        let lookup_result = evaluator.eval(&lookup_expr, env).unwrap();
        
        match lookup_result {
            Value::Literal(Literal::Number(n)) if n.fract() == 0.0 => assert_eq!(n as i64, 42),
            _ => panic!("Expected integer 42, got: {:?}", lookup_result),
        }
    }

    #[test]
    fn test_unbound_variable_error() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        let expr = spanned(Expr::Identifier("unbound-var".to_string()));
        let result = evaluator.eval(&expr, env);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unbound variable"));
    }
}

mod conditional_evaluation {
    use super::*;

    #[test]
    fn test_if_true_condition() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // (if #t 42 24)
        let expr = spanned(Expr::If {
            test: Box::new(spanned(Expr::Literal(Literal::boolean(true)))),
            consequent: Box::new(spanned(Expr::Literal(Literal::integer(42)))),
            alternative: Some(Box::new(spanned(Expr::Literal(Literal::integer(24))))),
        });
        
        let result = evaluator.eval(&expr, env).unwrap();
        
        match result {
            Value::Literal(Literal::Number(n)) if n.fract() == 0.0 => assert_eq!(n as i64, 42),
            _ => panic!("Expected integer 42, got: {:?}", result),
        }
    }

    #[test]
    fn test_if_false_condition() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // (if #f 42 24)
        let expr = spanned(Expr::If {
            test: Box::new(spanned(Expr::Literal(Literal::boolean(false)))),
            consequent: Box::new(spanned(Expr::Literal(Literal::integer(42)))),
            alternative: Some(Box::new(spanned(Expr::Literal(Literal::integer(24))))),
        });
        
        let result = evaluator.eval(&expr, env).unwrap();
        
        match result {
            Value::Literal(Literal::Number(n)) if n.fract() == 0.0 => assert_eq!(n as i64, 24),
            _ => panic!("Expected integer 24, got: {:?}", result),
        }
    }
}

mod sequence_evaluation {
    use super::*;

    #[test]
    fn test_begin_sequence() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // (begin 1 2 3)
        let expr = spanned(Expr::Begin(vec![
            spanned(Expr::Literal(Literal::integer(1))),
            spanned(Expr::Literal(Literal::integer(2))),
            spanned(Expr::Literal(Literal::integer(3))),
        ]));
        
        let result = evaluator.eval(&expr, env).unwrap();
        
        match result {
            Value::Literal(Literal::Number(n)) if n.fract() == 0.0 => assert_eq!(n as i64, 3),
            _ => panic!("Expected integer 3, got: {:?}", result),
        }
    }

    #[test]
    fn test_empty_begin_error() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // (begin)
        let expr = spanned(Expr::Begin(vec![]));
        let result = evaluator.eval(&expr, env);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Begin form cannot be empty"));
    }
}

mod quote_evaluation {
    use super::*;

    #[test]
    fn test_quote_symbol() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // (quote symbol)
        let expr = spanned(Expr::Quote(Box::new(spanned(Expr::Identifier("symbol".to_string())))));
        let result = evaluator.eval(&expr, env).unwrap();
        
        match result {
            Value::Symbol(_) => {}, // Symbol ID comparison would need more setup
            _ => panic!("Expected symbol, got: {:?}", result),
        }
    }
}

mod quasiquote_evaluation {
    use super::*;

    #[test]
    fn test_simple_quasiquote() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // `(a b c) should return (a b c)
        let expr = spanned(Expr::Quasiquote(Box::new(spanned(Expr::List(vec![
            spanned(Expr::Identifier("a".to_string())),
            spanned(Expr::Identifier("b".to_string())),
            spanned(Expr::Identifier("c".to_string())),
        ])))));
        
        let result = evaluator.eval(&expr, env).unwrap();
        
        // Should create a list with three symbols
        if let Value::Pair(car, cdr) = result {
            assert!(matches!(car.as_ref(), Value::Symbol(_)));
            if let Value::Pair(car2, cdr2) = cdr.as_ref() {
                assert!(matches!(car2.as_ref(), Value::Symbol(_)));
                if let Value::Pair(car3, cdr3) = cdr2.as_ref() {
                    assert!(matches!(car3.as_ref(), Value::Symbol(_)));
                    assert!(matches!(cdr3.as_ref(), Value::Nil));
                }
            }
        } else {
            panic!("Expected list result, got: {:?}", result);
        }
    }

    #[test]
    fn test_unquote_basic() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // First define a variable: (define x 42)
        let define_expr = spanned(Expr::Define {
            name: "x".to_string(),
            value: Box::new(spanned(Expr::Literal(Literal::integer(42)))),
            metadata: HashMap::new(),
        });
        evaluator.eval(&define_expr, env.clone()).unwrap();
        
        // `(a ,x b) should return (a 42 b)
        let expr = spanned(Expr::Quasiquote(Box::new(spanned(Expr::List(vec![
            spanned(Expr::Identifier("a".to_string())),
            spanned(Expr::Unquote(Box::new(spanned(Expr::Identifier("x".to_string()))))),
            spanned(Expr::Identifier("b".to_string())),
        ])))));
        
        let result = evaluator.eval(&expr, env).unwrap();
        
        // Should create (a 42 b)
        if let Value::Pair(car, cdr) = result {
            assert!(matches!(car.as_ref(), Value::Symbol(_)));
            if let Value::Pair(car2, cdr2) = cdr.as_ref() {
                if let Value::Literal(Literal::Number(n)) = car2.as_ref() {
                    assert_eq!(*n as i64, 42);
                }
                if let Value::Pair(car3, cdr3) = cdr2.as_ref() {
                    assert!(matches!(car3.as_ref(), Value::Symbol(_)));
                    assert!(matches!(cdr3.as_ref(), Value::Nil));
                }
            }
        } else {
            panic!("Expected list result, got: {:?}", result);
        }
    }

    #[test]
    fn test_unquote_splicing() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // First define a list: (define lst '(1 2 3))
        let list_literal = spanned(Expr::Quote(Box::new(spanned(Expr::List(vec![
            spanned(Expr::Literal(Literal::integer(1))),
            spanned(Expr::Literal(Literal::integer(2))),
            spanned(Expr::Literal(Literal::integer(3))),
        ])))));
        let define_expr = spanned(Expr::Define {
            name: "lst".to_string(),
            value: Box::new(list_literal),
            metadata: HashMap::new(),
        });
        evaluator.eval(&define_expr, env.clone()).unwrap();
        
        // `(a ,@lst b) should return (a 1 2 3 b)
        let expr = spanned(Expr::Quasiquote(Box::new(spanned(Expr::List(vec![
            spanned(Expr::Identifier("a".to_string())),
            spanned(Expr::UnquoteSplicing(Box::new(spanned(Expr::Identifier("lst".to_string()))))),
            spanned(Expr::Identifier("b".to_string())),
        ])))));
        
        let result = evaluator.eval(&expr, env).unwrap();
        
        // Should create (a 1 2 3 b) - a list of 5 elements
        let mut current = &result;
        let mut count = 0;
        while let Value::Pair(_car, cdr) = current {
            count += 1;
            current = cdr.as_ref();
        }
        assert_eq!(count, 5);
    }

    #[test]
    fn test_nested_quasiquote() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // Define x = 42
        let define_expr = spanned(Expr::Define {
            name: "x".to_string(),
            value: Box::new(spanned(Expr::Literal(Literal::integer(42)))),
            metadata: HashMap::new(),
        });
        evaluator.eval(&define_expr, env.clone()).unwrap();
        
        // ``(a ,,x) should return `(a ,42)
        let inner_quasiquote = spanned(Expr::Quasiquote(Box::new(spanned(Expr::List(vec![
            spanned(Expr::Identifier("a".to_string())),
            spanned(Expr::Unquote(Box::new(spanned(Expr::Unquote(Box::new(spanned(Expr::Identifier("x".to_string())))))))),
        ])))));
        let expr = spanned(Expr::Quasiquote(Box::new(inner_quasiquote)));
        
        let result = evaluator.eval(&expr, env).unwrap();
        
        // The result should be a list starting with 'quasiquote
        if let Value::Pair(car, _cdr) = result {
            assert!(matches!(car.as_ref(), Value::Symbol(_)));
        } else {
            panic!("Expected list result, got: {:?}", result);
        }
    }

    #[test]
    fn test_unquote_outside_quasiquote_error() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // ,x should produce an error
        let expr = spanned(Expr::Unquote(Box::new(spanned(Expr::Identifier("x".to_string())))));
        let result = evaluator.eval(&expr, env);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not in quasiquote"));
    }

    #[test]
    fn test_unquote_splicing_outside_quasiquote_error() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // ,@x should produce an error
        let expr = spanned(Expr::UnquoteSplicing(Box::new(spanned(Expr::Identifier("x".to_string())))));
        let result = evaluator.eval(&expr, env);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not in quasiquote"));
    }

    #[test]
    fn test_unquote_splicing_not_list_error() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // Define x = 42 (not a list)
        let define_expr = spanned(Expr::Define {
            name: "x".to_string(),
            value: Box::new(spanned(Expr::Literal(Literal::integer(42)))),
            metadata: HashMap::new(),
        });
        evaluator.eval(&define_expr, env.clone()).unwrap();
        
        // `(a ,@x) should produce an error since x is not a list
        let expr = spanned(Expr::Quasiquote(Box::new(spanned(Expr::List(vec![
            spanned(Expr::Identifier("a".to_string())),
            spanned(Expr::UnquoteSplicing(Box::new(spanned(Expr::Identifier("x".to_string()))))),
        ])))));
        
        let result = evaluator.eval(&expr, env);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a list"));
    }

    #[test]
    fn test_complex_quasiquote() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // Define variables
        let define_a = spanned(Expr::Define {
            name: "a".to_string(),
            value: Box::new(spanned(Expr::Literal(Literal::integer(1)))),
            metadata: HashMap::new(),
        });
        let define_b = spanned(Expr::Define {
            name: "b".to_string(),
            value: Box::new(spanned(Expr::Literal(Literal::integer(2)))),
            metadata: HashMap::new(),
        });
        let define_c = spanned(Expr::Define {
            name: "c".to_string(),
            value: Box::new(spanned(Expr::Quote(Box::new(spanned(Expr::List(vec![
                spanned(Expr::Literal(Literal::integer(3))),
                spanned(Expr::Literal(Literal::integer(4))),
            ])))))),
            metadata: HashMap::new(),
        });
        
        evaluator.eval(&define_a, env.clone()).unwrap();
        evaluator.eval(&define_b, env.clone()).unwrap();
        evaluator.eval(&define_c, env.clone()).unwrap();
        
        // `(sum ,a ,b ,@c) should return (sum 1 2 3 4)
        let expr = spanned(Expr::Quasiquote(Box::new(spanned(Expr::List(vec![
            spanned(Expr::Identifier("sum".to_string())),
            spanned(Expr::Unquote(Box::new(spanned(Expr::Identifier("a".to_string()))))),
            spanned(Expr::Unquote(Box::new(spanned(Expr::Identifier("b".to_string()))))),
            spanned(Expr::UnquoteSplicing(Box::new(spanned(Expr::Identifier("c".to_string()))))),
        ])))));
        
        let result = evaluator.eval(&expr, env).unwrap();
        
        // Should create (sum 1 2 3 4) - verify it's a list with 5 elements
        let mut current = &result;
        let mut count = 0;
        while let Value::Pair(_car, cdr) = current {
            count += 1;
            current = cdr.as_ref();
        }
        assert_eq!(count, 5);
    }

    #[test]
    fn test_empty_list_splicing() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // Define empty list
        let define_expr = spanned(Expr::Define {
            name: "empty".to_string(),
            value: Box::new(spanned(Expr::Quote(Box::new(spanned(Expr::List(vec![])))))),
            metadata: HashMap::new(),
        });
        evaluator.eval(&define_expr, env.clone()).unwrap();
        
        // `(a ,@empty b) should return (a b)
        let expr = spanned(Expr::Quasiquote(Box::new(spanned(Expr::List(vec![
            spanned(Expr::Identifier("a".to_string())),
            spanned(Expr::UnquoteSplicing(Box::new(spanned(Expr::Identifier("empty".to_string()))))),
            spanned(Expr::Identifier("b".to_string())),
        ])))));
        
        let result = evaluator.eval(&expr, env).unwrap();
        
        // Should create (a b) - a list with 2 elements
        let mut current = &result;
        let mut count = 0;
        while let Value::Pair(_car, cdr) = current {
            count += 1;
            current = cdr.as_ref();
        }
        assert_eq!(count, 2);
    }

    #[test] 
    fn test_r7rs_small_examples() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // R7RS-small section 4.2.8 examples
        
        // Example 1: `(list ,(+ 1 2) 4) => (list 3 4)
        let expr1 = spanned(Expr::Quasiquote(Box::new(spanned(Expr::List(vec![
            spanned(Expr::Identifier("list".to_string())),
            spanned(Expr::Unquote(Box::new(spanned(Expr::Application {
                operator: Box::new(spanned(Expr::Identifier("+".to_string()))),
                operands: vec![
                    spanned(Expr::Literal(Literal::integer(1))),
                    spanned(Expr::Literal(Literal::integer(2))),
                ],
            })))),
            spanned(Expr::Literal(Literal::integer(4))),
        ])))));
        
        let result1 = evaluator.eval(&expr1, env.clone()).unwrap();
        // Should be (list 3 4)
        if let Value::Pair(car, cdr) = result1 {
            assert!(matches!(car.as_ref(), Value::Symbol(_)));
            if let Value::Pair(car2, cdr2) = cdr.as_ref() {
                if let Value::Literal(Literal::Number(n)) = car2.as_ref() {
                    assert_eq!(*n as i64, 3);
                }
                if let Value::Pair(car3, cdr3) = cdr2.as_ref() {
                    if let Value::Literal(Literal::Number(n)) = car3.as_ref() {
                        assert_eq!(*n as i64, 4);
                    }
                    assert!(matches!(cdr3.as_ref(), Value::Nil));
                }
            }
        }
        
        // Example 2: Define a and b for splicing test
        let define_a = spanned(Expr::Define {
            name: "a".to_string(),
            value: Box::new(spanned(Expr::Literal(Literal::integer(1)))),
            metadata: HashMap::new(),
        });
        let define_b = spanned(Expr::Define {
            name: "b".to_string(),
            value: Box::new(spanned(Expr::Quote(Box::new(spanned(Expr::List(vec![
                spanned(Expr::Literal(Literal::integer(2))),
                spanned(Expr::Literal(Literal::integer(3))),
            ])))))),
            metadata: HashMap::new(),
        });
        
        evaluator.eval(&define_a, env.clone()).unwrap();
        evaluator.eval(&define_b, env.clone()).unwrap();
        
        // `(a ,a ,@b) => (a 1 2 3)
        let expr2 = spanned(Expr::Quasiquote(Box::new(spanned(Expr::List(vec![
            spanned(Expr::Identifier("a".to_string())),
            spanned(Expr::Unquote(Box::new(spanned(Expr::Identifier("a".to_string()))))),
            spanned(Expr::UnquoteSplicing(Box::new(spanned(Expr::Identifier("b".to_string()))))),
        ])))));
        
        let result2 = evaluator.eval(&expr2, env.clone()).unwrap();
        // Should be (a 1 2 3) - 4 elements
        let mut current = &result2;
        let mut count = 0;
        while let Value::Pair(_car, cdr) = current {
            count += 1;
            current = cdr.as_ref();
        }
        assert_eq!(count, 4);
    }

    #[test]
    fn test_r7rs_small_nested_quasiquote() {
        let mut evaluator = Evaluator::new();
        let env = test_env();
        
        // Define x for testing
        let define_x = spanned(Expr::Define {
            name: "x".to_string(),
            value: Box::new(spanned(Expr::Literal(Literal::integer(5)))),
            metadata: HashMap::new(),
        });
        evaluator.eval(&define_x, env.clone()).unwrap();
        
        // R7RS-small example: ``(a ,,x)
        let inner_quasi = spanned(Expr::List(vec![
            spanned(Expr::Identifier("a".to_string())),
            spanned(Expr::Unquote(Box::new(spanned(Expr::Unquote(Box::new(spanned(Expr::Identifier("x".to_string())))))))),
        ]));
        
        let outer_quasi = spanned(Expr::Quasiquote(Box::new(spanned(Expr::Quasiquote(Box::new(inner_quasi))))));
        
        let result = evaluator.eval(&outer_quasi, env).unwrap();
        
        // Result should be `(a ,5) - represented as (quasiquote (a (unquote 5)))
        if let Value::Pair(car, cdr) = result {
            assert!(matches!(car.as_ref(), Value::Symbol(_))); // quasiquote symbol
            if let Value::Pair(inner_car, _inner_cdr) = cdr.as_ref() {
                if let Value::Pair(a_sym, remaining) = inner_car.as_ref() {
                    assert!(matches!(a_sym.as_ref(), Value::Symbol(_))); // 'a symbol
                    if let Value::Pair(unquote_expr, _) = remaining.as_ref() {
                        if let Value::Pair(unquote_sym, val) = unquote_expr.as_ref() {
                            assert!(matches!(unquote_sym.as_ref(), Value::Symbol(_))); // unquote symbol
                            if let Value::Pair(five, _) = val.as_ref() {
                                if let Value::Literal(Literal::Number(n)) = five.as_ref() {
                                    assert_eq!(*n as i64, 5);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}