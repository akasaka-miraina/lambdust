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