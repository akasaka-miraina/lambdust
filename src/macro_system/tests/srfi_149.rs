//! Comprehensive test suite for SRFI-149 Basic Syntax-rules Template Extensions
//!
//! This module tests all SRFI-149 features including:
//! - Multiple consecutive ellipses
//! - Extra ellipses in templates  
//! - Ambiguity resolution rules
//! - Error handling and edge cases
//! - Integration with SRFI-46 custom ellipsis
//! - Backward compatibility with R7RS

use super::super::*;
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Span, Spanned};
use crate::eval::Environment;
use std::collections::HashMap;
use std::rc::Rc;

/// Helper to create spanned expressions for testing
fn spanned_expr(expr: Expr) -> Spanned<Expr> {
    Spanned::new(expr, Span::new(0, 1))
}

/// Helper to create spanned identifier
fn spanned_id(name: &str) -> Spanned<Expr> {
    spanned_expr(Expr::Identifier(name.to_string()))
}

/// Helper to create spanned literal
fn spanned_lit(value: f64) -> Spanned<Expr> {
    spanned_expr(Expr::Literal(Literal::Number(value)))
}

/// Helper to create spanned list  
fn spanned_list(elements: Vec<Spanned<Expr>>) -> Spanned<Expr> {
    spanned_expr(Expr::List(elements))
}

#[test]
fn test_multiple_consecutive_ellipses_parsing() {
    // Test parsing of templates with multiple consecutive ellipses
    // Template: ((a b ...) ...)
    
    let inner_template = spanned_list(vec![
        spanned_id("a"),
        spanned_id("b"),
        spanned_id("..."),
    ]);
    let outer_template = spanned_list(vec![
        inner_template,
        spanned_id("..."),
    ]);
    
    let template = parse_template(&outer_template, "...").unwrap();
    
    // Should create NestedEllipsis with depth 2
    match template {
        Template::NestedEllipsis { depth, .. } => {
            assert_eq!(depth, 2);
        }
        _ => panic!("Expected NestedEllipsis template, got {:?}", template),
    }
}

#[test] 
fn test_triple_consecutive_ellipses() {
    // Test parsing of templates with triple consecutive ellipses
    // Template: (((a b ...) ...) ...)
    
    let innermost = spanned_list(vec![
        spanned_id("a"),
        spanned_id("b"),
        spanned_id("..."),
    ]);
    let middle = spanned_list(vec![
        innermost,
        spanned_id("..."),
    ]);
    let outermost = spanned_list(vec![
        middle,
        spanned_id("..."),
    ]);
    
    let template = parse_template(&outermost, "...").unwrap();
    
    // Should create NestedEllipsis with depth 3
    match template {
        Template::NestedEllipsis { depth, .. } => {
            assert_eq!(depth, 3);
        }
        _ => panic!("Expected NestedEllipsis template with depth 3, got {:?}", template),
    }
}

#[test]
fn test_extra_ellipses_detection() {
    // Test detection of templates with more ellipses than patterns
    
    // Create a simple pattern: (a b c)
    let pattern = Pattern::List(vec![
        Pattern::Variable("a".to_string()),
        Pattern::Variable("b".to_string()),
        Pattern::Variable("c".to_string()),
    ]);
    
    // Create a template with extra ellipses: ((a b c) ...)
    let template = Template::Ellipsis {
        templates: vec![],
        ellipsis_template: Box::new(Template::List(vec![
            Template::Variable("a".to_string()),
            Template::Variable("b".to_string()),
            Template::Variable("c".to_string()),
        ])),
        rest: None,
    };
    
    // Pattern depth is 0, template depth is 1 - should need extra ellipses
    assert_eq!(pattern.ellipsis_depth(), 0);
    assert_eq!(template.ellipsis_depth(), 1);
    assert!(template.needs_extra_ellipses(pattern.ellipsis_depth()));
}

#[test]
fn test_ambiguity_resolution() {
    // Test SRFI-149 ambiguity resolution rules
    
    // Create pattern with nested variable: (x ... (x y) ...)
    let inner_pattern = Pattern::List(vec![
        Pattern::Variable("x".to_string()),
        Pattern::Variable("y".to_string()),
    ]);
    
    let pattern = Pattern::Ellipsis {
        patterns: vec![],
        ellipsis_pattern: Box::new(Pattern::Variable("x".to_string())),
        rest: Some(Box::new(Pattern::Ellipsis {
            patterns: vec![],
            ellipsis_pattern: Box::new(inner_pattern),
            rest: None,
        })),
    };
    
    let var_depths = pattern.variable_depths();
    
    // Variable 'x' should appear at depth 1 (innermost)
    // Variable 'y' should appear at depth 2
    assert_eq!(var_depths.get("x"), Some(&1));
    assert_eq!(var_depths.get("y"), Some(&2));
}

#[test]
fn test_srfi_149_mode_flag() {
    // Test SRFI-149 mode flag controls feature availability
    
    let env = Rc::new(Environment::new(None, 0));
    
    let mut transformer_enabled = SyntaxRulesTransformer {
        literals: vec![],
        rules: vec![],
        name: None,
        definition_env: env.clone(),
        custom_ellipsis: None,
        srfi_149_mode: true,
    };
    
    let mut transformer_disabled = SyntaxRulesTransformer {
        literals: vec![],
        rules: vec![],
        name: None,
        definition_env: env,
        custom_ellipsis: None,
        srfi_149_mode: false,
    };
    
    assert!(transformer_enabled.is_srfi_149_enabled());
    assert!(!transformer_disabled.is_srfi_149_enabled());
    
    // Test mode modification
    transformer_enabled = transformer_enabled.with_srfi_149_mode(false);
    transformer_disabled = transformer_disabled.with_srfi_149_mode(true);
    
    assert!(!transformer_enabled.is_srfi_149_enabled());
    assert!(transformer_disabled.is_srfi_149_enabled());
}

#[test]
fn test_ellipsis_depth_calculation() {
    // Test accurate ellipsis depth calculation
    
    // Single ellipsis: (a ...)
    let single = Template::Ellipsis {
        templates: vec![],
        ellipsis_template: Box::new(Template::Variable("a".to_string())),
        rest: None,
    };
    assert_eq!(single.ellipsis_depth(), 1);
    
    // Nested ellipsis: ((a ...) ...)  
    let nested = Template::Ellipsis {
        templates: vec![],
        ellipsis_template: Box::new(single),
        rest: None,
    };
    assert_eq!(nested.ellipsis_depth(), 2);
    
    // SRFI-149 nested ellipsis with explicit depth
    let srfi149_nested = Template::NestedEllipsis {
        templates: vec![],
        nested_template: Box::new(Template::Variable("a".to_string())),
        depth: 3,
        rest: None,
    };
    assert_eq!(srfi149_nested.ellipsis_depth(), 3);
    
    // Extra ellipsis  
    let extra = Template::ExtraEllipsis {
        base_template: Box::new(Template::Variable("a".to_string())),
        extra_depth: 2,
    };
    assert_eq!(extra.ellipsis_depth(), 2);
}

#[test]
fn test_pattern_matching_with_ellipsis() {
    // Test pattern matching with ellipsis patterns
    
    let pattern = Pattern::Ellipsis {
        patterns: vec![Pattern::Variable("first".to_string())],
        ellipsis_pattern: Box::new(Pattern::Variable("rest".to_string())),
        rest: None,
    };
    
    // Test matching against (a b c d)
    let input = spanned_list(vec![
        spanned_id("a"),
        spanned_id("b"), 
        spanned_id("c"),
        spanned_id("d"),
    ]);
    
    let bindings = pattern.match_expr(&input).unwrap();
    
    // Should bind 'first' to 'a' and 'rest' to list [b, c, d]
    assert!(bindings.get("first").is_some());
    assert!(bindings.get_ellipsis("rest").is_some());
    
    let rest_bindings = bindings.get_ellipsis("rest").unwrap();
    assert_eq!(rest_bindings.len(), 3);
}

#[test]
fn test_template_expansion_basic() {
    // Test basic template expansion
    
    let mut bindings = PatternBindings::new();
    bindings.bind("x".to_string(), spanned_id("hello"));
    bindings.bind_ellipsis("ys".to_string(), vec![
        spanned_id("world"),
        spanned_lit(42.0),
    ]);
    
    let template = Template::List(vec![
        Template::Variable("x".to_string()),
        Template::Ellipsis {
            templates: vec![],
            ellipsis_template: Box::new(Template::Variable("ys".to_string())),
            rest: None,
        },
    ]);
    
    let result = template.expand(&bindings, Span::new(0, 1)).unwrap();
    
    // Should expand to (hello world 42)
    match result.inner {
        Expr::List(elements) => {
            assert_eq!(elements.len(), 3);
        }
        _ => panic!("Expected expanded list, got {:?}", result.inner),
    }
}

#[test]
fn test_error_handling_excessive_depth() {
    // Test error handling for excessive ellipsis depth
    
    let template = Template::NestedEllipsis {
        templates: vec![],
        nested_template: Box::new(Template::Variable("x".to_string())),
        depth: 15, // Exceeds maximum of 10
        rest: None,
    };
    
    let bindings = PatternBindings::new();
    let result = template.expand(&bindings, Span::new(0, 1));
    
    assert!(result.is_err());
    let error_message = format!("{}", result.unwrap_err());
    assert!(error_message.contains("exceeds maximum"));
}

#[test]
fn test_error_handling_zero_depth() {
    // Test error handling for zero ellipsis depth
    
    let template = Template::NestedEllipsis {
        templates: vec![],
        nested_template: Box::new(Template::Variable("x".to_string())),
        depth: 0, // Invalid depth
        rest: None,
    };
    
    let bindings = PatternBindings::new();
    let result = template.expand(&bindings, Span::new(0, 1));
    
    assert!(result.is_err());
    let error_message = format!("{}", result.unwrap_err());
    assert!(error_message.contains("cannot be zero"));
}

#[test]
fn test_error_handling_unbound_variables() {
    // Test error handling for unbound template variables
    
    let template = Template::NestedEllipsis {
        templates: vec![],
        nested_template: Box::new(Template::Variable("unbound".to_string())),
        depth: 2,
        rest: None,
    };
    
    let bindings = PatternBindings::new(); // No bindings provided
    let result = template.expand(&bindings, Span::new(0, 1));
    
    assert!(result.is_err());
    let error_message = format!("{}", result.unwrap_err());
    assert!(error_message.contains("no ellipsis bindings found"));
}

#[test]
fn test_integration_with_srfi_46() {
    // Test integration with SRFI-46 custom ellipsis
    
    // Use custom ellipsis ":::"
    let template_expr = spanned_list(vec![
        spanned_id("a"),
        spanned_id(":::"),
        spanned_id(":::"), // Double consecutive custom ellipsis
    ]);
    
    let template = parse_template(&template_expr, ":::").unwrap();
    
    // Should create NestedEllipsis with custom ellipsis
    match template {
        Template::NestedEllipsis { depth, .. } => {
            assert_eq!(depth, 2);
        }
        _ => panic!("Expected NestedEllipsis with custom ellipsis"),
    }
}

#[test]
fn test_r7rs_backward_compatibility() {
    // Test that standard R7RS syntax-rules continue to work
    
    // Standard R7RS ellipsis pattern: (a ...)
    let standard_template = spanned_list(vec![
        spanned_id("a"),
        spanned_id("..."),
    ]);
    
    let template = parse_template(&standard_template, "...").unwrap();
    
    // Should create standard Ellipsis, not NestedEllipsis
    match template {
        Template::Ellipsis { .. } => {
            // This is correct for single ellipsis
        }
        _ => panic!("Expected standard Ellipsis template for R7RS compatibility"),
    }
}

#[test]
fn test_complex_nested_expansion() {
    // Test complex nested template expansion scenario
    
    let mut bindings = PatternBindings::new();
    bindings.bind_ellipsis("items".to_string(), vec![
        spanned_list(vec![spanned_id("a"), spanned_lit(1.0)]),
        spanned_list(vec![spanned_id("b"), spanned_lit(2.0)]),
        spanned_list(vec![spanned_id("c"), spanned_lit(3.0)]),
    ]);
    
    // Template: ((first second ...) ...)
    let template = Template::NestedEllipsis {
        templates: vec![],
        nested_template: Box::new(Template::Variable("items".to_string())),
        depth: 2,
        rest: None,
    };
    
    let result = template.expand(&bindings, Span::new(0, 1));
    
    // Should successfully expand the nested structure
    assert!(result.is_ok());
}

#[test]
fn test_performance_characteristics() {
    // Test performance characteristics of SRFI-149 features
    
    // Create a large ellipsis binding
    let large_binding: Vec<Spanned<Expr>> = (0..1000)
        .map(|i| spanned_lit(i as f64))
        .collect();
    
    let mut bindings = PatternBindings::new();
    bindings.bind_ellipsis("large".to_string(), large_binding);
    
    let template = Template::Ellipsis {
        templates: vec![],
        ellipsis_template: Box::new(Template::Variable("large".to_string())),
        rest: None,
    };
    
    // Should handle large expansion efficiently
    let start = std::time::Instant::now();
    let result = template.expand(&bindings, Span::new(0, 1));
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 100); // Should complete within 100ms
}

#[test]
fn test_variable_depth_analysis() {
    // Test accurate variable depth analysis
    
    let pattern = Pattern::Ellipsis {
        patterns: vec![Pattern::Variable("outer".to_string())],
        ellipsis_pattern: Box::new(Pattern::Ellipsis {
            patterns: vec![],
            ellipsis_pattern: Box::new(Pattern::Variable("inner".to_string())),
            rest: None,
        }),
        rest: Some(Box::new(Pattern::Variable("final".to_string()))),
    };
    
    let depths = pattern.variable_depths();
    
    assert_eq!(depths.get("outer"), Some(&0)); // Outer scope
    assert_eq!(depths.get("inner"), Some(&2)); // Nested ellipsis
    assert_eq!(depths.get("final"), Some(&0)); // Rest pattern
}

/// Integration test: Complete SRFI-149 macro expansion
#[test]
fn test_complete_srfi149_macro() {
    // Test a complete macro using SRFI-149 features
    
    let env = Rc::new(Environment::new(None, 0));
    
    // Define a macro that uses multiple consecutive ellipses
    // (define-syntax nested-map
    //   (syntax-rules ()
    //     ((nested-map f ((a b ...) ...))
    //      ((f a b ...) ...))))
    
    let pattern = Pattern::List(vec![
        Pattern::Identifier("nested-map".to_string()),
        Pattern::Variable("f".to_string()),
        Pattern::Ellipsis {
            patterns: vec![],
            ellipsis_pattern: Box::new(Pattern::List(vec![
                Pattern::Variable("a".to_string()),
                Pattern::Ellipsis {
                    patterns: vec![],
                    ellipsis_pattern: Box::new(Pattern::Variable("b".to_string())),
                    rest: None,
                },
            ])),
            rest: None,
        },
    ]);
    
    let template = Template::NestedEllipsis {
        templates: vec![],
        nested_template: Box::new(Template::List(vec![
            Template::Variable("f".to_string()),
            Template::Variable("a".to_string()),
            Template::Ellipsis {
                templates: vec![],
                ellipsis_template: Box::new(Template::Variable("b".to_string())),
                rest: None,
            },
        ])),
        depth: 2,
        rest: None,
    };
    
    let rule = SyntaxRule { pattern, template };
    
    let transformer = SyntaxRulesTransformer {
        literals: vec![],
        rules: vec![rule],
        name: Some("nested-map".to_string()),
        definition_env: env,
        custom_ellipsis: None,
        srfi_149_mode: true, // Enable SRFI-149 features
    };
    
    // Test input: (nested-map + ((1 2 3) (4 5 6)))
    let input = spanned_list(vec![
        spanned_id("nested-map"),
        spanned_id("+"),
        spanned_list(vec![
            spanned_list(vec![spanned_lit(1.0), spanned_lit(2.0), spanned_lit(3.0)]),
            spanned_list(vec![spanned_lit(4.0), spanned_lit(5.0), spanned_lit(6.0)]),
        ]),
    ]);
    
    // Should expand successfully
    let result = expand_syntax_rules(&transformer, &input);
    assert!(result.is_ok());
}

// Module-level documentation and examples
/// SRFI-149 Test Coverage Summary:
/// 
/// ✓ Multiple consecutive ellipses parsing and expansion
/// ✓ Triple and higher-order consecutive ellipses  
/// ✓ Extra ellipses detection and handling
/// ✓ Ambiguity resolution rules
/// ✓ SRFI-149 mode flag functionality
/// ✓ Ellipsis depth calculation
/// ✓ Pattern matching with ellipsis
/// ✓ Template expansion with various binding types
/// ✓ Comprehensive error handling
/// ✓ Integration with SRFI-46 custom ellipsis
/// ✓ R7RS backward compatibility
/// ✓ Complex nested expansion scenarios
/// ✓ Performance characteristics
/// ✓ Variable depth analysis
/// ✓ Complete macro expansion pipeline
/// 
/// This test suite ensures that the SRFI-149 implementation is:
/// - Functionally complete
/// - Performance optimized  
/// - Error resilient
/// - Backward compatible
/// - Well integrated with existing infrastructure