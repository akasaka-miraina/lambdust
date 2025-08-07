//! R7RS compliance tests for the macro system.
//!
//! This module contains comprehensive tests to verify that the macro system
//! complies with R7RS-small specification requirements.

#[cfg(test)]
mod tests {
    use crate::ast::{Expr, Literal};
    use crate::diagnostics::{Span, Spanned};
    use crate::macro_system::*;
    use crate::eval::Environment;
    use std::rc::Rc;

/// Helper to create spanned expressions for testing.
fn make_spanned<T>(value: T) -> Spanned<T> {
    Spanned::new(value, Span::new(0, 1))
}

/// Helper to create a basic syntax-rules transformer for testing.
fn create_test_syntax_rules(literals: Vec<String>, rules: Vec<(Pattern, Template)>) -> SyntaxRulesTransformer {
    SyntaxRulesTransformer {
        literals,
        rules: rules.into_iter().map(|(pattern, template)| SyntaxRule { pattern, template }).collect(),
        name: Some("test-macro".to_string()),
        definition_env: Rc::new(Environment::new(None, 0)),
    }
}

#[test]
fn test_basic_syntax_rules_parsing() {
    // Test parsing of basic syntax-rules form:
    // (syntax-rules () ((test-macro x) x))
    let syntax_rules_expr = make_spanned(Expr::Application {
        operator: Box::new(make_spanned(Expr::Identifier("syntax-rules".to_string()))),
        operands: vec![
            // Empty literals list
            make_spanned(Expr::List(vec![])),
            // Single rule: ((test-macro x) x)
            make_spanned(Expr::List(vec![
                make_spanned(Expr::List(vec![
                    make_spanned(Expr::Identifier("test-macro".to_string())),
                    make_spanned(Expr::Identifier("x".to_string())),
                ])),
                make_spanned(Expr::Identifier("x".to_string())),
            ])),
        ],
    });
    
    let env = Rc::new(Environment::new(None, 0));
    let result = parse_syntax_rules(&syntax_rules_expr, env);
    
    assert!(result.is_ok());
    let transformer = result.unwrap();
    assert!(transformer.literals.is_empty());
    assert_eq!(transformer.rules.len(), 1);
}

#[test]
fn test_syntax_rules_with_literals() {
    // Test parsing syntax-rules with literals:
    // (syntax-rules (else) 
    //   ((test-macro else x) x)
    //   ((test-macro y x) y))
    let syntax_rules_expr = make_spanned(Expr::Application {
        operator: Box::new(make_spanned(Expr::Identifier("syntax-rules".to_string()))),
        operands: vec![
            // Literals list: (else)
            make_spanned(Expr::List(vec![
                make_spanned(Expr::Identifier("else".to_string())),
            ])),
            // First rule: ((test-macro else x) x)
            make_spanned(Expr::List(vec![
                make_spanned(Expr::List(vec![
                    make_spanned(Expr::Identifier("test-macro".to_string())),
                    make_spanned(Expr::Identifier("else".to_string())),
                    make_spanned(Expr::Identifier("x".to_string())),
                ])),
                make_spanned(Expr::Identifier("x".to_string())),
            ])),
            // Second rule: ((test-macro y x) y)
            make_spanned(Expr::List(vec![
                make_spanned(Expr::List(vec![
                    make_spanned(Expr::Identifier("test-macro".to_string())),
                    make_spanned(Expr::Identifier("y".to_string())),
                    make_spanned(Expr::Identifier("x".to_string())),
                ])),
                make_spanned(Expr::Identifier("y".to_string())),
            ])),
        ],
    });
    
    let env = Rc::new(Environment::new(None, 0));
    let result = parse_syntax_rules(&syntax_rules_expr, env);
    
    assert!(result.is_ok());
    let transformer = result.unwrap();
    assert_eq!(transformer.literals, vec!["else"]);
    assert_eq!(transformer.rules.len(), 2);
}

#[test]
fn test_ellipsis_pattern_parsing() {
    // Test parsing ellipsis patterns:
    // (syntax-rules ()
    //   ((test-macro x y ...) (list x y ...)))
    let syntax_rules_expr = make_spanned(Expr::Application {
        operator: Box::new(make_spanned(Expr::Identifier("syntax-rules".to_string()))),
        operands: vec![
            // Empty literals list
            make_spanned(Expr::List(vec![])),
            // Rule with ellipsis: ((test-macro x y ...) (list x y ...))
            make_spanned(Expr::List(vec![
                make_spanned(Expr::List(vec![
                    make_spanned(Expr::Identifier("test-macro".to_string())),
                    make_spanned(Expr::Identifier("x".to_string())),
                    make_spanned(Expr::Identifier("y".to_string())),
                    make_spanned(Expr::Identifier("...".to_string())),
                ])),
                make_spanned(Expr::List(vec![
                    make_spanned(Expr::Identifier("list".to_string())),
                    make_spanned(Expr::Identifier("x".to_string())),
                    make_spanned(Expr::Identifier("y".to_string())),
                    make_spanned(Expr::Identifier("...".to_string())),
                ])),
            ])),
        ],
    });
    
    let env = Rc::new(Environment::new(None, 0));
    let result = parse_syntax_rules(&syntax_rules_expr, env);
    
    assert!(result.is_ok());
    let transformer = result.unwrap();
    assert_eq!(transformer.rules.len(), 1);
    
    // Check that the pattern contains ellipsis
    match &transformer.rules[0].pattern {
        Pattern::List(_) => {}, // Pattern parsing handles ellipsis internally
        _ => panic!("Expected list pattern"),
    }
}

#[test]
fn test_multiple_rule_expansion() {
    // Test that syntax-rules with multiple rules tries them in order
    let pattern1 = Pattern::list(vec![
        Pattern::identifier("test-macro"),
        Pattern::identifier("special"),
        Pattern::variable("x"),
    ]);
    let template1 = Template::list(vec![
        Template::identifier("special-case"),
        Template::variable("x"),
    ]);
    
    let pattern2 = Pattern::list(vec![
        Pattern::identifier("test-macro"),
        Pattern::variable("x"),
    ]);
    let template2 = Template::list(vec![
        Template::identifier("general-case"),
        Template::variable("x"),
    ]);
    
    let transformer = create_test_syntax_rules(
        vec![],
        vec![(pattern1, template1), (pattern2, template2)],
    );
    
    // Test first rule (special case)
    let input1 = make_spanned(Expr::Application {
        operator: Box::new(make_spanned(Expr::Identifier("test-macro".to_string()))),
        operands: vec![
            make_spanned(Expr::Identifier("special".to_string())),
            make_spanned(Expr::Identifier("value".to_string())),
        ],
    });
    
    let result1 = expand_syntax_rules(&transformer, &input1);
    assert!(result1.is_ok());
    
    // Test second rule (general case)
    let input2 = make_spanned(Expr::Application {
        operator: Box::new(make_spanned(Expr::Identifier("test-macro".to_string()))),
        operands: vec![
            make_spanned(Expr::Identifier("value".to_string())),
        ],
    });
    
    let result2 = expand_syntax_rules(&transformer, &input2);
    assert!(result2.is_ok());
}

#[test]
fn test_hygiene_preservation() {
    // Test that macro expansion preserves hygiene
    let mut expander = MacroExpander::new();
    
    // Define a simple macro that introduces a binding
    let pattern = Pattern::list(vec![
        Pattern::identifier("let-macro"),
        Pattern::variable("x"),
        Pattern::variable("body"),
    ]);
    
    let template = Template::list(vec![
        Template::identifier("let"),
        Template::list(vec![
            Template::list(vec![
                Template::identifier("temp"),
                Template::variable("x"),
            ])
        ]),
        Template::variable("body"),
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: Rc::new(Environment::new(None, 0)),
        name: Some("let-macro".to_string()),
        source: None,
    };
    
    expander.define_macro("let-macro".to_string(), transformer);
    
    // Test expansion
    let input_expr = make_spanned(Expr::Application {
        operator: Box::new(make_spanned(Expr::Identifier("let-macro".to_string()))),
        operands: vec![
            make_spanned(Expr::Literal(Literal::Number(42.0))),
            make_spanned(Expr::Identifier("temp".to_string())),
        ],
    });
    
    let result = expander.expand(&input_expr);
    assert!(result.is_ok());
    
    // The result should have hygienically renamed identifiers
    // (exact testing would require checking the hygiene context)
}

#[test]
fn test_builtin_macro_presence() {
    // Test that all required R7RS macros are present
    let expander = MacroExpander::with_builtins();
    
    // Core derived forms
    assert!(expander.macro_env().lookup("let").is_some());
    assert!(expander.macro_env().lookup("let*").is_some());
    assert!(expander.macro_env().lookup("letrec").is_some());
    assert!(expander.macro_env().lookup("cond").is_some());
    assert!(expander.macro_env().lookup("case").is_some());
    assert!(expander.macro_env().lookup("and").is_some());
    assert!(expander.macro_env().lookup("or").is_some());
    assert!(expander.macro_env().lookup("when").is_some());
    assert!(expander.macro_env().lookup("unless").is_some());
    
    // R7RS convenience macros
    assert!(expander.macro_env().lookup("case-lambda").is_some());
    assert!(expander.macro_env().lookup("cond-expand").is_some());
    assert!(expander.macro_env().lookup("assert").is_some());
}

#[test]
fn test_pattern_validation() {
    let literals = vec!["else".to_string()];
    
    // Valid pattern
    let valid_pattern = Pattern::list(vec![
        Pattern::identifier("macro"),
        Pattern::variable("x"),
        Pattern::identifier("else"),
    ]);
    
    assert!(validate_pattern(&valid_pattern, &literals).is_ok());
    
    // Invalid pattern - variable conflicts with literal
    let invalid_pattern = Pattern::variable("else");
    assert!(validate_pattern(&invalid_pattern, &literals).is_err());
    
    // Invalid pattern - duplicate variable binding
    let duplicate_pattern = Pattern::list(vec![
        Pattern::variable("x"),
        Pattern::variable("x"),
    ]);
    
    // This should be caught by validation
    assert!(validate_pattern(&duplicate_pattern, &literals).is_err());
}

#[test]
fn test_template_variable_validation() {
    use std::collections::HashSet;
    
    let mut pattern_vars = HashSet::new();
    pattern_vars.insert("x".to_string());
    pattern_vars.insert("y".to_string());
    
    let ellipsis_vars = HashSet::new();
    
    // Valid template - uses bound variables
    let valid_template = Template::list(vec![
        Template::identifier("result"),
        Template::variable("x"),
        Template::variable("y"),
    ]);
    
    assert!(validate_template(&valid_template, &pattern_vars, &ellipsis_vars).is_ok());
    
    // Invalid template - uses unbound variable
    let invalid_template = Template::list(vec![
        Template::identifier("result"),
        Template::variable("unbound"),
    ]);
    
    assert!(validate_template(&invalid_template, &pattern_vars, &ellipsis_vars).is_err());
}

#[test]
fn test_macro_expansion_recursion_prevention() {
    let mut expander = MacroExpander::new();
    
    // Define a recursive macro
    let pattern = Pattern::list(vec![
        Pattern::identifier("recursive-macro"),
        Pattern::variable("x"),
    ]);
    
    let template = Template::list(vec![
        Template::identifier("recursive-macro"),
        Template::variable("x"),
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: Rc::new(Environment::new(None, 0)),
        name: Some("recursive-macro".to_string()),
        source: None,
    };
    
    expander.define_macro("recursive-macro".to_string(), transformer);
    
    // Try to expand - should detect infinite recursion
    let input_expr = make_spanned(Expr::Application {
        operator: Box::new(make_spanned(Expr::Identifier("recursive-macro".to_string()))),
        operands: vec![
            make_spanned(Expr::Literal(Literal::Number(42.0))),
        ],
    });
    
    let result = expander.expand(&input_expr);
    assert!(result.is_err());
    
    // Error should mention recursive expansion
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(error_msg.contains("recursive") || error_msg.contains("Recursive"));
}

#[test]
fn test_nested_macro_expansion() {
    let mut expander = MacroExpander::with_builtins();
    
    // Test that macros can expand to other macros
    let input_expr = make_spanned(Expr::Application {
        operator: Box::new(make_spanned(Expr::Identifier("when".to_string()))),
        operands: vec![
            make_spanned(Expr::Literal(Literal::Boolean(true))),
            make_spanned(Expr::Application {
                operator: Box::new(make_spanned(Expr::Identifier("let".to_string()))),
                operands: vec![
                    make_spanned(Expr::List(vec![
                        make_spanned(Expr::List(vec![
                            make_spanned(Expr::Identifier("x".to_string())),
                            make_spanned(Expr::Literal(Literal::Number(42.0))),
                        ])),
                    ])),
                    make_spanned(Expr::Identifier("x".to_string())),
                ],
            }),
        ],
    });
    
    let result = expander.expand(&input_expr);
    assert!(result.is_ok());
    
    // Result should be fully expanded (no more macro calls)
    let expanded = result.unwrap();
    // The exact structure depends on how when and let expand,
    // but it should not contain macro calls
    assert!(is_fully_expanded(&expanded.inner));
}

/// Helper function to check if an expression is fully expanded (contains no macro calls).
fn is_fully_expanded(expr: &Expr) -> bool {
    match expr {
        Expr::Application { operator, operands } => {
            // Check if operator is a known macro name
            if let Expr::Identifier(name) = &operator.inner {
                let macro_names = vec!["let", "let*", "letrec", "cond", "case", "and", "or", "when", "unless"];
                if macro_names.contains(&name.as_str()) {
                    return false;
                }
            }
            
            // Recursively check operands
            is_fully_expanded(&operator.inner) && operands.iter().all(|op| is_fully_expanded(&op.inner))
        }
        Expr::Lambda { body, .. } => {
            body.iter().all(|expr| is_fully_expanded(&expr.inner))
        }
        Expr::If { test, consequent, alternative } => {
            is_fully_expanded(&test.inner) 
                && is_fully_expanded(&consequent.inner)
                && alternative.as_ref().map_or(true, |alt| is_fully_expanded(&alt.inner))
        }
        Expr::Define { value, .. } => is_fully_expanded(&value.inner),
        Expr::Set { value, .. } => is_fully_expanded(&value.inner),
        Expr::Begin(exprs) => exprs.iter().all(|expr| is_fully_expanded(&expr.inner)),
        _ => true, // Literals, identifiers, etc. are always fully expanded
    }
}
}