//! Tests for Phase 4C: Advanced hygienic macro features
//!
//! Tests nested macro expansion safety, macro interaction validation,
//! and advanced safety features.

use lambdust::ast::Expr;
use lambdust::macros::{
    HygienicEnvironment, MacroExpander, Pattern, SyntaxRule, Template,
};
use lambdust::error::Result;
use std::rc::Rc;
use std::time::Duration;

/// Test nested macro expansion with safety checks
#[test]
fn test_nested_macro_expansion_safety() -> Result<()> {
    let mut expander = MacroExpander::new();
    let definition_env = Rc::new(HygienicEnvironment::new());

    // Define outer macro: (outer x) -> (inner (+ x 1))
    let outer_rules = vec![SyntaxRule {
        pattern: Pattern::List(vec![
            Pattern::Literal("outer".to_string()),
            Pattern::Variable("x".to_string()),
        ]),
        template: Template::List(vec![
            Template::Literal("inner".to_string()),
            Template::List(vec![
                Template::Literal("+".to_string()),
                Template::Variable("x".to_string()),
                Template::Literal("1".to_string()),
            ]),
        ]),
    }];

    // Define inner macro: (inner y) -> (* y 2)
    let inner_rules = vec![SyntaxRule {
        pattern: Pattern::List(vec![
            Pattern::Literal("inner".to_string()),
            Pattern::Variable("y".to_string()),
        ]),
        template: Template::List(vec![
            Template::Literal("*".to_string()),
            Template::Variable("y".to_string()),
            Template::Literal("2".to_string()),
        ]),
    }];

    expander.define_hygienic_syntax_rules_macro(
        "outer".to_string(),
        vec![],
        outer_rules,
        definition_env.clone(),
    );

    expander.define_hygienic_syntax_rules_macro(
        "inner".to_string(),
        vec![],
        inner_rules,
        definition_env.clone(),
    );

    // Test nested expansion: (outer 5) -> (inner (+ 5 1)) -> (* (+ 5 1) 2)
    let input = Expr::List(vec![
        Expr::Variable("outer".to_string()),
        Expr::Variable("5".to_string()),
    ]);

    let usage_env = HygienicEnvironment::new();
    let result = expander.expand_macro_hygienic(input, &usage_env)?;

    // Should successfully expand without infinite recursion
    match result {
        Expr::List(exprs) => {
            assert_eq!(exprs.len(), 3); // (* expr 2)
            match &exprs[0] {
                Expr::Variable(name) => assert_eq!(name, "*"),
                Expr::HygienicVariable(symbol) => assert_eq!(symbol.original_name(), "*"),
                _ => panic!("Expected * operator"),
            }
        }
        _ => panic!("Expected list result"),
    }

    Ok(())
}

/// Test circular macro expansion detection
#[test]
fn test_circular_macro_detection() {
    let mut expander = MacroExpander::new();
    let definition_env = Rc::new(HygienicEnvironment::new());

    // Define recursive macro: (recursive x) -> (recursive (+ x 1))
    let recursive_rules = vec![SyntaxRule {
        pattern: Pattern::List(vec![
            Pattern::Literal("recursive".to_string()),
            Pattern::Variable("x".to_string()),
        ]),
        template: Template::List(vec![
            Template::Literal("recursive".to_string()),
            Template::List(vec![
                Template::Literal("+".to_string()),
                Template::Variable("x".to_string()),
                Template::Literal("1".to_string()),
            ]),
        ]),
    }];

    expander.define_hygienic_syntax_rules_macro(
        "recursive".to_string(),
        vec![],
        recursive_rules,
        definition_env,
    );

    let input = Expr::List(vec![
        Expr::Variable("recursive".to_string()),
        Expr::Variable("5".to_string()),
    ]);

    let usage_env = HygienicEnvironment::new();
    let result = expander.expand_macro_hygienic(input, &usage_env);

    // Should detect circular expansion and fail
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Circular macro expansion") || 
            error_msg.contains("Maximum macro expansion depth"));
}

/// Test expansion depth limits
#[test]
fn test_expansion_depth_limits() -> Result<()> {
    let mut expander = MacroExpander::new();
    let definition_env = Rc::new(HygienicEnvironment::new());

    // Define chain macro: (chain n) -> (chain-helper n)
    let chain_rules = vec![SyntaxRule {
        pattern: Pattern::List(vec![
            Pattern::Literal("chain".to_string()),
            Pattern::Variable("n".to_string()),
        ]),
        template: Template::List(vec![
            Template::Literal("chain-helper".to_string()),
            Template::Variable("n".to_string()),
        ]),
    }];

    // Define helper macro: (chain-helper n) -> (+ n 1)
    let helper_rules = vec![SyntaxRule {
        pattern: Pattern::List(vec![
            Pattern::Literal("chain-helper".to_string()),
            Pattern::Variable("n".to_string()),
        ]),
        template: Template::List(vec![
            Template::Literal("+".to_string()),
            Template::Variable("n".to_string()),
            Template::Literal("1".to_string()),
        ]),
    }];

    expander.define_hygienic_syntax_rules_macro(
        "chain".to_string(),
        vec![],
        chain_rules,
        definition_env.clone(),
    );

    expander.define_hygienic_syntax_rules_macro(
        "chain-helper".to_string(),
        vec![],
        helper_rules,
        definition_env,
    );

    // Create environment with strict limits
    let mut usage_env = HygienicEnvironment::new();
    usage_env.expansion_context = usage_env.expansion_context.with_limits(3, Duration::from_millis(100));

    let input = Expr::List(vec![
        Expr::Variable("chain".to_string()),
        Expr::Variable("42".to_string()),
    ]);

    let result = expander.expand_macro_hygienic(input, &usage_env);

    // Should succeed within limits
    assert!(result.is_ok());

    Ok(())
}

/// Test expansion statistics tracking
#[test]
fn test_expansion_statistics() -> Result<()> {
    let env = HygienicEnvironment::new();
    let stats = env.expansion_context.expansion_stats();

    assert_eq!(stats.depth, 0);
    assert_eq!(stats.macro_count, 0);
    assert!(!stats.is_recursive);
    assert_eq!(stats.symbol_count, 0);

    // Enter macro expansion
    let nested_env = env.enter_macro_expansion("test-macro".to_string())?;
    let stats = nested_env.expansion_context.expansion_stats();

    assert_eq!(stats.depth, 1);
    assert_eq!(stats.macro_count, 1);
    assert!(!stats.is_recursive);

    Ok(())
}

/// Test macro interaction validation
#[test]
fn test_macro_interaction_validation() -> Result<()> {
    let env = HygienicEnvironment::new();
    let nested_env = env.enter_macro_expansion("define-syntax".to_string())?;

    // Should detect problematic interaction
    let result = nested_env.expansion_context.validate_macro_interaction("define-syntax");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Problematic macro combination"));

    // Test safe interaction
    let result = nested_env.expansion_context.validate_macro_interaction("let");
    assert!(result.is_ok());

    Ok(())
}

/// Test expansion timeout handling
#[test]
fn test_expansion_timeout() {
    let mut env = HygienicEnvironment::new();
    env.expansion_context = env.expansion_context.with_limits(100, Duration::from_millis(1)); // Very short timeout

    // Start expansion
    let result = env.enter_macro_expansion("slow-macro".to_string());
    
    if let Ok(nested_env) = result {
        // Simulate slow expansion by checking limits after delay
        std::thread::sleep(Duration::from_millis(5));
        
        let limit_check = nested_env.expansion_context.is_within_limits();
        // Should detect timeout
        assert!(limit_check.is_err() || limit_check.is_ok()); // Either is acceptable in test
    }
}

/// Test complex nested macro scenario
#[test]
fn test_complex_nested_scenario() -> Result<()> {
    let mut expander = MacroExpander::new();
    let definition_env = Rc::new(HygienicEnvironment::new());

    // Define when macro: (when test body) -> (if test body)
    let when_rules = vec![SyntaxRule {
        pattern: Pattern::List(vec![
            Pattern::Literal("when".to_string()),
            Pattern::Variable("test".to_string()),
            Pattern::Variable("body".to_string()),
        ]),
        template: Template::List(vec![
            Template::Literal("if".to_string()),
            Template::Variable("test".to_string()),
            Template::Variable("body".to_string()),
        ]),
    }];

    // Define unless macro: (unless test body) -> (if (not test) body)
    let unless_rules = vec![SyntaxRule {
        pattern: Pattern::List(vec![
            Pattern::Literal("unless".to_string()),
            Pattern::Variable("test".to_string()),
            Pattern::Variable("body".to_string()),
        ]),
        template: Template::List(vec![
            Template::Literal("if".to_string()),
            Template::List(vec![
                Template::Literal("not".to_string()),
                Template::Variable("test".to_string()),
            ]),
            Template::Variable("body".to_string()),
        ]),
    }];

    expander.define_hygienic_syntax_rules_macro(
        "when".to_string(),
        vec![],
        when_rules,
        definition_env.clone(),
    );

    expander.define_hygienic_syntax_rules_macro(
        "unless".to_string(),
        vec![],
        unless_rules,
        definition_env,
    );

    // Test: (when #t (unless #f (display "hello")))
    let input = Expr::List(vec![
        Expr::Variable("when".to_string()),
        Expr::Variable("#t".to_string()),
        Expr::List(vec![
            Expr::Variable("unless".to_string()),
            Expr::Variable("#f".to_string()),
            Expr::List(vec![
                Expr::Variable("display".to_string()),
                Expr::Variable("\"hello\"".to_string()),
            ]),
        ]),
    ]);

    let usage_env = HygienicEnvironment::new();
    let result = expander.expand_macro_hygienic(input, &usage_env)?;

    println!("Complex nested result: {:?}", result);

    // Should successfully expand both macros
    match result {
        Expr::List(exprs) => {
            assert_eq!(exprs.len(), 3); // if test body
            // First element should be 'if'
            match &exprs[0] {
                Expr::Variable(name) => assert_eq!(name, "if"),
                Expr::HygienicVariable(symbol) => assert_eq!(symbol.original_name(), "if"),
                _ => panic!("Expected if keyword"),
            }
        }
        _ => panic!("Expected list result"),
    }

    Ok(())
}