//! Integration tests for hygienic macro system
//!
//! Tests the complete integration of hygienic macros including symbol collision
//! prevention, environment separation, and proper hygiene preservation.

use lambdust::ast::Expr;
use lambdust::macros::{
    HygienicEnvironment, HygienicSyntaxRulesTransformer, MacroExpander, Pattern, SyntaxRule,
    Template,
};
use lambdust::error::Result;
use std::rc::Rc;

/// Create a test environment for hygienic macro testing
fn create_test_environment() -> Rc<HygienicEnvironment> {
    Rc::new(HygienicEnvironment::new())
}

/// Test basic hygienic macro creation and registration
#[test]
fn test_hygienic_macro_creation() {
    let mut expander = MacroExpander::new();
    let definition_env = create_test_environment();

    // Define a simple hygienic macro: (when test body) -> (if test body)
    let rules = vec![SyntaxRule {
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

    expander.define_hygienic_syntax_rules_macro(
        "when".to_string(),
        vec![],
        rules,
        definition_env,
    );

    // Verify the macro was registered
    let test_expr = Expr::List(vec![
        Expr::Variable("when".to_string()),
        Expr::Variable("#t".to_string()),
        Expr::Variable("body".to_string()),
    ]);

    assert!(expander.is_macro_call(&test_expr));
}

/// Test hygienic macro expansion
#[test]
fn test_hygienic_macro_expansion() -> Result<()> {
    let mut expander = MacroExpander::new();
    let definition_env = create_test_environment();

    // Define hygienic when macro
    let rules = vec![SyntaxRule {
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

    expander.define_hygienic_syntax_rules_macro(
        "when".to_string(),
        vec![],
        rules,
        definition_env,
    );

    // Test expansion
    let input = Expr::List(vec![
        Expr::Variable("when".to_string()),
        Expr::Variable("#t".to_string()),
        Expr::List(vec![
            Expr::Variable("display".to_string()),
            Expr::Variable("\"hello\"".to_string()),
        ]),
    ]);

    let expanded = expander.expand_macro(input)?;

    // Debug: print the expanded result
    println!("Expanded result: {:?}", expanded);

    // Should expand to (if #t (display "hello"))
    match expanded {
        Expr::List(exprs) => {
            assert_eq!(exprs.len(), 3);
            match &exprs[0] {
                Expr::Variable(name) => assert_eq!(name, "if"),
                Expr::HygienicVariable(symbol) => {
                    assert_eq!(symbol.original_name(), "if");
                }
                other => panic!("Expected if keyword, got: {:?}", other),
            }
        }
        _ => panic!("Expected list result"),
    }

    Ok(())
}

/// Test symbol collision prevention
#[test]
fn test_symbol_collision_prevention() -> Result<()> {
    let definition_env = create_test_environment();
    let _usage_env = HygienicEnvironment::new();

    // Create transformer for a macro that introduces 'temp' variable
    let rules = vec![SyntaxRule {
        pattern: Pattern::List(vec![
            Pattern::Literal("with-temp".to_string()),
            Pattern::Variable("body".to_string()),
        ]),
        template: Template::List(vec![
            Template::Literal("let".to_string()),
            Template::List(vec![Template::List(vec![
                Template::Literal("temp".to_string()),
                Template::Literal("42".to_string()),
            ])]),
            Template::Variable("body".to_string()),
        ]),
    }];

    let _transformer = HygienicSyntaxRulesTransformer::new(
        vec![],
        rules,
        definition_env,
        "with-temp".to_string(),
    );

    // Expand macro
    let input = vec![Expr::Variable("body-expr".to_string())];
    let result = transformer.transform_hygienic(&input, &usage_env)?;

    // Debug: print the actual result
    println!("Symbol collision result: {:?}", result);

    // The result should contain hygienic symbols where appropriate
    match result {
        Expr::List(exprs) => {
            assert_eq!(exprs.len(), 3); // let, bindings, body
            match &exprs[0] {
                Expr::Variable(name) => assert_eq!(name, "let"),
                Expr::HygienicVariable(symbol) => {
                    assert_eq!(symbol.original_name(), "let");
                }
                other => panic!("Expected let keyword, got: {:?}", other),
            }
        }
        _ => panic!("Expected list result"),
    }

    Ok(())
}

/// Test environment separation in hygienic macros
#[test]
fn test_environment_separation() {
    let definition_env = create_test_environment();
    let _usage_env = HygienicEnvironment::new();

    // Definition environment should be separate from usage environment
    assert_ne!(definition_env.id, usage_env.id);

    // Each environment should have its own expansion context
    assert_eq!(definition_env.expansion_context.depth, 0);
    assert_eq!(usage_env.expansion_context.depth, 0);
}

/// Test macro expansion with nested contexts
#[test]
fn test_nested_macro_expansion() -> Result<()> {
    let definition_env = create_test_environment();
    let _usage_env = HygienicEnvironment::new();

    // Create a macro that can be nested
    let rules = vec![SyntaxRule {
        pattern: Pattern::List(vec![
            Pattern::Literal("debug".to_string()),
            Pattern::Variable("expr".to_string()),
        ]),
        template: Template::List(vec![
            Template::Literal("begin".to_string()),
            Template::List(vec![
                Template::Literal("display".to_string()),
                Template::Literal("\"Debug: \"".to_string()),
            ]),
            Template::Variable("expr".to_string()),
        ]),
    }];

    let _transformer = HygienicSyntaxRulesTransformer::new(
        vec![],
        rules,
        definition_env,
        "debug".to_string(),
    );

    // Test nested expansion context
    let expanded_env = usage_env.enter_macro_expansion("debug".to_string()).unwrap();
    assert_eq!(expanded_env.expansion_context.depth, 1);
    assert_eq!(
        expanded_env.expansion_context.current_macro(),
        Some("debug")
    );

    Ok(())
}

/// Test literal identifier preservation
#[test]
fn test_literal_preservation() -> Result<()> {
    let definition_env = create_test_environment();
    let _usage_env = HygienicEnvironment::new();

    // Create macro with literal identifiers
    let rules = vec![SyntaxRule {
        pattern: Pattern::List(vec![
            Pattern::Literal("cond".to_string()),
            Pattern::List(vec![
                Pattern::Literal("else".to_string()),
                Pattern::Variable("body".to_string()),
            ]),
        ]),
        template: Template::Variable("body".to_string()),
    }];

    let _transformer = HygienicSyntaxRulesTransformer::new(
        vec!["else".to_string()], // 'else' is a literal
        rules,
        definition_env,
        "cond".to_string(),
    );

    // Literal identifiers should be preserved
    assert!(transformer.literals.contains(&"else".to_string()));

    Ok(())
}

/// Test symbol generator uniqueness
#[test]
fn test_symbol_generator_uniqueness() {
    use lambdust::macros::hygiene::SymbolGenerator;

    let mut generator1 = SymbolGenerator::new();
    let mut generator2 = SymbolGenerator::new();

    let symbol1 = generator1.generate_unique("test");
    let symbol2 = generator1.generate_unique("test");
    let symbol3 = generator2.generate_unique("test");

    // All symbols should have different IDs
    assert_ne!(symbol1.id, symbol2.id);
    assert_ne!(symbol1.id, symbol3.id);
    assert_ne!(symbol2.id, symbol3.id);

    // But same original name
    assert_eq!(symbol1.original_name(), "test");
    assert_eq!(symbol2.original_name(), "test");
    assert_eq!(symbol3.original_name(), "test");
}

/// Test hygienic environment symbol resolution
#[test]
fn test_hygienic_symbol_resolution() {
    use lambdust::macros::hygiene::{SymbolGenerator, environment::SymbolResolution};
    use lambdust::value::Value;
    use lambdust::lexer::SchemeNumber;

    let mut env = HygienicEnvironment::new();
    let mut generator = SymbolGenerator::new();
    generator.set_environment(env.id);

    // Define traditional variable
    env.define("traditional".to_string(), Value::Number(SchemeNumber::Integer(1)));

    // Define hygienic symbol
    let symbol = generator.generate_unique("hygienic");
    env.define_hygienic(symbol.clone(), Value::Number(SchemeNumber::Integer(2)));

    // Test resolution
    match env.resolve_symbol("traditional") {
        SymbolResolution::Traditional(name) => assert_eq!(name, "traditional"),
        _ => panic!("Expected traditional resolution"),
    }

    match env.resolve_symbol("hygienic") {
        SymbolResolution::Hygienic(sym) => assert_eq!(sym.original_name(), "hygienic"),
        _ => panic!("Expected hygienic resolution"),
    }

    match env.resolve_symbol("unbound") {
        SymbolResolution::Unbound(name) => assert_eq!(name, "unbound"),
        _ => panic!("Expected unbound resolution"),
    }
}

/// Integration test with MacroExpander hygienic methods
#[test]
fn test_macro_expander_hygienic_integration() -> Result<()> {
    let mut expander = MacroExpander::new();
    let definition_env = create_test_environment();
    let _usage_env = HygienicEnvironment::new();

    // Define a hygienic macro
    let rules = vec![SyntaxRule {
        pattern: Pattern::List(vec![
            Pattern::Literal("test-macro".to_string()),
            Pattern::Variable("x".to_string()),
        ]),
        template: Template::List(vec![
            Template::Literal("+".to_string()),
            Template::Variable("x".to_string()),
            Template::Literal("1".to_string()),
        ]),
    }];

    expander.define_hygienic_syntax_rules_macro(
        "test-macro".to_string(),
        vec![],
        rules,
        definition_env,
    );

    // Test hygienic expansion
    let input = Expr::List(vec![
        Expr::Variable("test-macro".to_string()),
        Expr::Variable("42".to_string()),
    ]);

    let result = expander.expand_macro_hygienic(input, &usage_env)?;

    // Debug: print the actual result
    println!("Actual result: {:?}", result);

    // Should expand to (+ 42 1)
    match result {
        Expr::List(exprs) => {
            assert_eq!(exprs.len(), 3);
            match &exprs[0] {
                Expr::Variable(name) => assert_eq!(name, "+"),
                Expr::HygienicVariable(symbol) => {
                    assert_eq!(symbol.original_name(), "+");
                }
                other => panic!("Expected + operator, got: {:?}", other),
            }
        }
        _ => panic!("Expected list result"),
    }

    Ok(())
}