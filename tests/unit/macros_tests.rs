//! Unit tests for macro system functionality

use lambdust::ast::Expr;
use lambdust::lexer::tokenize;
use lambdust::macros::{MacroExpander, Pattern, Template};
use lambdust::parser::parse;

fn parse_expr(input: &str) -> Expr {
    let tokens = tokenize(input).unwrap();
    parse(tokens).unwrap()
}

#[test]
fn test_expand_let() {
    let expander = MacroExpander::new();
    let expr = parse_expr("(let ((x 1) (y 2)) (+ x y))");
    let expanded = expander.expand_macro(expr).unwrap();

    // Should expand to ((lambda (x y) (+ x y)) 1 2)
    match expanded {
        Expr::List(exprs) => {
            assert_eq!(exprs.len(), 3);
            assert!(matches!(exprs[0], Expr::List(_))); // lambda expression
        }
        _ => panic!("Expected list expression"),
    }
}

#[test]
fn test_expand_cond() {
    let expander = MacroExpander::new();
    let expr = parse_expr("(cond ((< x 0) 'negative) ((> x 0) 'positive) (else 'zero))");
    let expanded = expander.expand_macro(expr).unwrap();

    // Should expand to nested if expressions
    match expanded {
        Expr::List(exprs) => {
            assert_eq!(exprs[0], Expr::Variable("if".to_string()));
        }
        _ => panic!("Expected if expression"),
    }
}

#[test]
fn test_expand_when() {
    let expander = MacroExpander::new();
    let expr = parse_expr("(when (> x 0) (display x) (newline))");
    let expanded = expander.expand_macro(expr).unwrap();

    // Should expand to (if (> x 0) (begin (display x) (newline)))
    match expanded {
        Expr::List(exprs) => {
            assert_eq!(exprs[0], Expr::Variable("if".to_string()));
            assert_eq!(exprs.len(), 3);
        }
        _ => panic!("Expected if expression"),
    }
}

#[test]
fn test_is_macro_call() {
    let expander = MacroExpander::new();
    let let_expr = parse_expr("(let ((x 1)) x)");
    let regular_expr = parse_expr("(+ 1 2)");

    assert!(expander.is_macro_call(&let_expr));
    assert!(!expander.is_macro_call(&regular_expr));
}

#[test]
fn test_srfi_46_ellipsis_count() {
    let expander = MacroExpander::new();

    // Test ellipsis variable
    let ellipsis_var = Expr::Variable("...".to_string());
    assert_eq!(MacroExpander::count_ellipsis_level(&ellipsis_var), 1);

    // Test nested ellipsis in list
    let nested = Expr::List(vec![
        Expr::Variable("...".to_string()),
        Expr::List(vec![Expr::Variable("...".to_string())]),
    ]);
    assert_eq!(MacroExpander::count_ellipsis_level(&nested), 1);
}

#[test]
fn test_srfi_46_pattern_parsing() {
    let expander = MacroExpander::new();

    // Test simple pattern
    let simple = Expr::Variable("x".to_string());
    let pattern = expander.parse_pattern_srfi46(&simple).unwrap();
    assert_eq!(pattern, Pattern::Variable("x".to_string()));

    // Test list pattern
    let list = Expr::List(vec![
        Expr::Variable("x".to_string()),
        Expr::Variable("y".to_string()),
        Expr::Variable("z".to_string()),
    ]);
    let pattern = expander.parse_pattern_srfi46(&list).unwrap();
    assert!(matches!(pattern, Pattern::List(_)));

    // Test ellipsis pattern
    let ellipsis = Expr::List(vec![
        Expr::Variable("x".to_string()),
        Expr::Variable("...".to_string()),
    ]);
    let pattern = expander.parse_pattern_srfi46(&ellipsis).unwrap();
    if let Pattern::List(patterns) = pattern {
        assert_eq!(patterns.len(), 1);
        assert!(matches!(patterns[0], Pattern::Ellipsis(_)));
    } else {
        panic!("Expected list pattern with ellipsis");
    }
}

#[test]
fn test_srfi_46_template_parsing() {
    let expander = MacroExpander::new();

    // Test simple template
    let simple = Expr::Variable("x".to_string());
    let template = expander.parse_template_srfi46(&simple).unwrap();
    assert_eq!(template, Template::Variable("x".to_string()));

    // Test list template
    let list = Expr::List(vec![
        Expr::Variable("x".to_string()),
        Expr::Variable("y".to_string()),
        Expr::Variable("z".to_string()),
    ]);
    let template = expander.parse_template_srfi46(&list).unwrap();
    assert!(matches!(template, Template::List(_)));

    // Test ellipsis template
    let ellipsis = Expr::List(vec![
        Expr::Variable("x".to_string()),
        Expr::Variable("...".to_string()),
    ]);
    let template = expander.parse_template_srfi46(&ellipsis).unwrap();
    if let Template::List(templates) = template {
        assert_eq!(templates.len(), 1);
        assert!(matches!(templates[0], Template::Ellipsis(_)));
    } else {
        panic!("Expected list template with ellipsis");
    }
}

#[test]
fn test_expand_define_record_type() {
    let expander = MacroExpander::new();

    // Test basic define-record-type expansion
    let expr = parse_expr(
        r#"
        (define-record-type point
          (make-point x y)
          point?
          (x point-x set-point-x!)
          (y point-y set-point-y!))
    "#,
    );

    let expanded = expander.expand_macro(expr).unwrap();

    // Should expand to a begin expression with multiple definitions
    match expanded {
        Expr::List(exprs) if !exprs.is_empty() => {
            if let Expr::Variable(name) = &exprs[0] {
                assert_eq!(name, "begin");
            }
            // Should contain multiple define expressions for constructor, predicate, and accessors
            assert!(exprs.len() > 3); // begin + at least constructor, predicate, and accessors
        }
        _ => panic!("Expected begin expression with definitions"),
    }
}

#[test]
fn test_expand_define_record_type_minimal() {
    let expander = MacroExpander::new();

    // Test minimal define-record-type with no fields
    let expr = parse_expr(
        r#"
        (define-record-type empty-record
          (make-empty-record)
          empty-record?)
    "#,
    );

    let expanded = expander.expand_macro(expr).unwrap();

    // Should still expand to a begin expression
    match expanded {
        Expr::List(exprs) if !exprs.is_empty() => {
            if let Expr::Variable(name) = &exprs[0] {
                assert_eq!(name, "begin");
            }
            // Should contain at least constructor and predicate
            assert!(exprs.len() >= 3); // begin + constructor + predicate
        }
        _ => panic!("Expected begin expression with definitions"),
    }
}

#[test]
fn test_expand_define_record_type_field_without_modifier() {
    let expander = MacroExpander::new();

    // Test define-record-type with field that has no modifier
    let expr = parse_expr(
        r#"
        (define-record-type person
          (make-person name age)
          person?
          (name person-name)
          (age person-age set-person-age!))
    "#,
    );

    let expanded = expander.expand_macro(expr).unwrap();

    // Should expand successfully even with mixed field specifications
    match expanded {
        Expr::List(exprs) if !exprs.is_empty() => {
            if let Expr::Variable(name) = &exprs[0] {
                assert_eq!(name, "begin");
            }
            assert!(exprs.len() > 4); // begin + constructor + predicate + accessors
        }
        _ => panic!("Expected begin expression with definitions"),
    }
}
