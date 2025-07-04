//! Syntax-rules macro system tests
//! 
//! Tests for the enhanced syntax-rules macro system including SRFI 46 features.

use lambdust::macros::{MacroExpander, Pattern, Template, SyntaxRule};
use lambdust::ast::Expr;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_expr(code: &str) -> Expr {
        // Helper to create test expressions
        match code {
            "test" => Expr::Variable("test".to_string()),
            "body" => Expr::Variable("body".to_string()),
            "if" => Expr::Variable("if".to_string()),
            "begin" => Expr::Variable("begin".to_string()),
            _ => Expr::Variable(code.to_string()),
        }
    }

    #[test]
    fn test_syntax_rules_transformer_creation() {
        let literals = vec![];
        let rules = vec![
            SyntaxRule {
                pattern: Pattern::List(vec![
                    Pattern::Literal("my-when".to_string()),
                    Pattern::Variable("test".to_string()),
                    Pattern::Ellipsis(Box::new(Pattern::Variable("body".to_string()))),
                ]),
                template: Template::List(vec![
                    Template::Literal("if".to_string()),
                    Template::Variable("test".to_string()),
                    Template::List(vec![
                        Template::Literal("begin".to_string()),
                        Template::Ellipsis(Box::new(Template::Variable("body".to_string()))),
                    ]),
                ]),
            }
        ];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);
        assert_eq!(transformer.literals.len(), 0);
        assert_eq!(transformer.rules.len(), 1);
    }

    #[test]
    fn test_simple_pattern_matching() {
        let literals = vec![];
        let rules = vec![
            SyntaxRule {
                pattern: Pattern::List(vec![
                    Pattern::Literal("my-when".to_string()),
                    Pattern::Variable("test".to_string()),
                    Pattern::Variable("body".to_string()),
                ]),
                template: Template::List(vec![
                    Template::Literal("if".to_string()),
                    Template::Variable("test".to_string()),
                    Template::Variable("body".to_string()),
                ]),
            }
        ];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);
        
        let input = Expr::List(vec![
            create_test_expr("my-when"),
            create_test_expr("test-condition"),
            create_test_expr("test-body"),
        ]);

        let result = transformer.transform(&input);
        assert!(result.is_ok());

        match result.unwrap() {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(exprs[0], create_test_expr("if"));
            }
            _ => panic!("Expected list result"),
        }
    }

    #[test]
    fn test_ellipsis_pattern_matching() {
        let literals = vec![];
        let rules = vec![
            SyntaxRule {
                pattern: Pattern::List(vec![
                    Pattern::Literal("my-begin".to_string()),
                    Pattern::Ellipsis(Box::new(Pattern::Variable("expr".to_string()))),
                ]),
                template: Template::List(vec![
                    Template::Literal("begin".to_string()),
                    Template::Ellipsis(Box::new(Template::Variable("expr".to_string()))),
                ]),
            }
        ];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);
        
        let input = Expr::List(vec![
            create_test_expr("my-begin"),
            create_test_expr("expr1"),
            create_test_expr("expr2"),
            create_test_expr("expr3"),
        ]);

        let result = transformer.transform(&input);
        assert!(result.is_ok());

        match result.unwrap() {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 4); // begin + 3 expressions
                assert_eq!(exprs[0], create_test_expr("begin"));
            }
            _ => panic!("Expected list result"),
        }
    }

    #[test]
    fn test_macro_expander_syntax_rules_integration() {
        let mut expander = MacroExpander::new();
        
        // Define a simple my-when macro
        let literals = vec![];
        let rules = vec![
            SyntaxRule {
                pattern: Pattern::List(vec![
                    Pattern::Literal("my-when".to_string()),
                    Pattern::Variable("test".to_string()),
                    Pattern::Variable("body".to_string()),
                ]),
                template: Template::List(vec![
                    Template::Literal("if".to_string()),
                    Template::Variable("test".to_string()),
                    Template::Variable("body".to_string()),
                ]),
            }
        ];

        expander.define_syntax_rules_macro(
            "my-when".to_string(),
            literals,
            rules,
        );

        // Test expansion
        let input = Expr::List(vec![
            create_test_expr("my-when"),
            create_test_expr("test-condition"),
            create_test_expr("test-body"),
        ]);

        let result = expander.expand_macro(input);
        assert!(result.is_ok());

        match result.unwrap() {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(exprs[0], create_test_expr("if"));
            }
            _ => panic!("Expected list result"),
        }
    }

    #[test]
    fn test_literal_matching() {
        let literals = vec!["else".to_string()];
        let rules = vec![
            SyntaxRule {
                pattern: Pattern::List(vec![
                    Pattern::Literal("my-cond".to_string()),
                    Pattern::List(vec![
                        Pattern::Literal("else".to_string()),
                        Pattern::Variable("result".to_string()),
                    ]),
                ]),
                template: Template::Variable("result".to_string()),
            }
        ];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);
        
        let input = Expr::List(vec![
            create_test_expr("my-cond"),
            Expr::List(vec![
                create_test_expr("else"),
                create_test_expr("default-value"),
            ]),
        ]);

        let result = transformer.transform(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), create_test_expr("default-value"));
    }

    #[test]
    fn test_pattern_matching_failure() {
        let literals = vec![];
        let rules = vec![
            SyntaxRule {
                pattern: Pattern::List(vec![
                    Pattern::Literal("my-when".to_string()),
                    Pattern::Variable("test".to_string()),
                    Pattern::Variable("body".to_string()),
                ]),
                template: Template::List(vec![
                    Template::Literal("if".to_string()),
                    Template::Variable("test".to_string()),
                    Template::Variable("body".to_string()),
                ]),
            }
        ];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);
        
        // Wrong number of arguments
        let input = Expr::List(vec![
            create_test_expr("my-when"),
            create_test_expr("test-condition"),
        ]);

        let result = transformer.transform(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_ellipsis_with_multiple_variables() {
        let literals = vec![];
        let rules = vec![
            SyntaxRule {
                pattern: Pattern::List(vec![
                    Pattern::Literal("my-let".to_string()),
                    Pattern::List(vec![
                        Pattern::Ellipsis(Box::new(
                            Pattern::List(vec![
                                Pattern::Variable("var".to_string()),
                                Pattern::Variable("val".to_string()),
                            ])
                        )),
                    ]),
                    Pattern::Variable("body".to_string()),
                ]),
                template: Template::List(vec![
                    Template::Literal("let-impl".to_string()),
                    Template::List(vec![
                        Template::Ellipsis(Box::new(
                            Template::List(vec![
                                Template::Variable("var".to_string()),
                                Template::Variable("val".to_string()),
                            ])
                        )),
                    ]),
                    Template::Variable("body".to_string()),
                ]),
            }
        ];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);
        
        let input = Expr::List(vec![
            create_test_expr("my-let"),
            Expr::List(vec![
                Expr::List(vec![
                    create_test_expr("x"),
                    create_test_expr("1"),
                ]),
                Expr::List(vec![
                    create_test_expr("y"),
                    create_test_expr("2"),
                ]),
            ]),
            create_test_expr("body-expr"),
        ]);

        let result = transformer.transform(&input);
        assert!(result.is_ok());

        match result.unwrap() {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(exprs[0], create_test_expr("let-impl"));
            }
            _ => panic!("Expected list result"),
        }
    }

    #[test]
    fn test_nested_ellipsis_basic() {
        let literals = vec![];
        let rules = vec![
            SyntaxRule {
                pattern: Pattern::List(vec![
                    Pattern::Literal("nested-test".to_string()),
                    Pattern::NestedEllipsis(
                        Box::new(Pattern::Variable("item".to_string())),
                        1
                    ),
                ]),
                template: Template::List(vec![
                    Template::Literal("nested-result".to_string()),
                    Template::NestedEllipsis(
                        Box::new(Template::Variable("item".to_string())),
                        1
                    ),
                ]),
            }
        ];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);
        
        let input = Expr::List(vec![
            create_test_expr("nested-test"),
            create_test_expr("item1"),
            create_test_expr("item2"),
        ]);

        // This should work (though nested ellipsis is placeholder implementation)
        let result = transformer.transform(&input);
        assert!(result.is_ok());
    }
}