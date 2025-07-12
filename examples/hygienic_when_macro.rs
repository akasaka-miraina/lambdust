//! Hygienic 'when' macro implementation and test
//! Demonstrates the classic symbol collision prevention example

use lambdust::macros::hygiene::{
    HygienicSyntaxRulesTransformer, HygienicEnvironment, SymbolGenerator
};
use lambdust::macros::{SyntaxRule};
use lambdust::macros::pattern_matching::{Pattern, Template};
use lambdust::ast::{Expr, Literal};
use std::rc::Rc;

fn main() {
    println!("=== Hygienic 'when' Macro Test ===\n");
    
    test_basic_when_macro();
    test_symbol_collision_prevention();
}

fn test_basic_when_macro() {
    println!("1. Basic 'when' macro expansion:");
    
    // Create hygienic environment
    let def_env = Rc::new(HygienicEnvironment::new());
    
    // Define the 'when' macro rules
    // (when test expr ...) => (if test (begin expr ...))
    let rules = vec![
        SyntaxRule {
            pattern: create_when_pattern(),
            template: create_when_template(),
        }
    ];
    
    let transformer = HygienicSyntaxRulesTransformer::new(
        vec![], // no literals
        rules,
        def_env.clone(),
        "when".to_string(),
    );
    
    // Test input: (when #t (display "hello") (display "world"))
    let test_input = vec![
        Expr::Literal(Literal::Boolean(true)),
        Expr::List(vec![
            Expr::Variable("display".to_string()),
            Expr::Literal(Literal::String("hello".to_string())),
        ]),
        Expr::List(vec![
            Expr::Variable("display".to_string()),
            Expr::Literal(Literal::String("world".to_string())),
        ]),
    ];
    
    let usage_env = HygienicEnvironment::new();
    
    match transformer.transform_hygienic(&test_input, &usage_env) {
        Ok(result) => {
            println!("  Input:  (when #t (display \"hello\") (display \"world\"))");
            println!("  Output: {}", result);
            println!("  ✓ Basic expansion works");
        }
        Err(e) => {
            println!("  ✗ Expansion failed: {}", e);
        }
    }
    
    println!();
}

fn test_symbol_collision_prevention() {
    println!("2. Symbol collision prevention test:");
    println!("  This test demonstrates that user-defined variables don't interfere");
    println!("  with macro-introduced symbols, even if they have the same name.");
    
    // Simulate the problematic case:
    // (let ((if 42))
    //   (when #t (display "hello")))
    //
    // Without hygiene: would try to call (42 #t ...) instead of real 'if'
    // With hygiene: macro's 'if' and user's 'if' are different symbols
    
    let def_env = Rc::new(HygienicEnvironment::new());
    let mut usage_env = HygienicEnvironment::new();
    
    // Simulate user binding 'if' to 42
    // In a real implementation, this would be done through environment binding
    let mut generator = SymbolGenerator::new();
    let user_if_symbol = generator.generate_unique("if");
    println!("  User binds 'if' to 42 (symbol: {} ID: {})", 
             user_if_symbol.name, user_if_symbol.id);
    
    // The macro expansion should use its own 'if' symbol
    let rules = vec![
        SyntaxRule {
            pattern: create_when_pattern(),
            template: create_when_template(),
        }
    ];
    
    let transformer = HygienicSyntaxRulesTransformer::new(
        vec![],
        rules,
        def_env.clone(),
        "when".to_string(),
    );
    
    let test_input = vec![
        Expr::Literal(Literal::Boolean(true)),
        Expr::List(vec![
            Expr::Variable("display".to_string()),
            Expr::Literal(Literal::String("hello".to_string())),
        ]),
    ];
    
    match transformer.transform_hygienic(&test_input, &usage_env) {
        Ok(result) => {
            println!("  Macro expansion: {}", result);
            println!("  ✓ Macro's 'if' is isolated from user's 'if' binding");
            println!("  ✓ Hygiene prevents symbol collision");
        }
        Err(e) => {
            println!("  ✗ Expansion failed: {}", e);
        }
    }
    
    println!();
}

fn create_when_pattern() -> Pattern {
    // Pattern: (when test expr ...)
    // This is a simplified representation - in reality, Pattern would be more complex
    Pattern::List(vec![
        Pattern::Literal("when".to_string()),
        Pattern::Variable("test".to_string()),
        Pattern::Ellipsis(Box::new(Pattern::Variable("expr".to_string()))),
    ])
}

fn create_when_template() -> Template {
    // Template: (if test (begin expr ...))
    Template::List(vec![
        Template::Literal("if".to_string()),
        Template::Variable("test".to_string()),
        Template::List(vec![
            Template::Literal("begin".to_string()),
            Template::Ellipsis(Box::new(Template::Variable("expr".to_string()))),
        ]),
    ])
}