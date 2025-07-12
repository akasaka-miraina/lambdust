//! Syntax-rules macro system tests
//!
//! Tests for the enhanced syntax-rules macro system including SRFI 46 features.

use lambdust::ast::Expr;
use lambdust::macros::{MacroExpander, Pattern, SyntaxRule, Template};

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
        let rules = vec![SyntaxRule {
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
        }];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);
        assert_eq!(transformer.literals.len(), 0);
        assert_eq!(transformer.rules.len(), 1);
    }

    #[test]
    fn test_simple_pattern_matching() {
        let literals = vec![];
        let rules = vec![SyntaxRule {
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
        }];

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
        let rules = vec![SyntaxRule {
            pattern: Pattern::List(vec![
                Pattern::Literal("my-begin".to_string()),
                Pattern::Ellipsis(Box::new(Pattern::Variable("expr".to_string()))),
            ]),
            template: Template::List(vec![
                Template::Literal("begin".to_string()),
                Template::Ellipsis(Box::new(Template::Variable("expr".to_string()))),
            ]),
        }];

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
        let rules = vec![SyntaxRule {
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
        }];

        expander.define_syntax_rules_macro("my-when".to_string(), literals, rules);

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
        let rules = vec![SyntaxRule {
            pattern: Pattern::List(vec![
                Pattern::Literal("my-cond".to_string()),
                Pattern::List(vec![
                    Pattern::Literal("else".to_string()),
                    Pattern::Variable("result".to_string()),
                ]),
            ]),
            template: Template::Variable("result".to_string()),
        }];

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
        let rules = vec![SyntaxRule {
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
        }];

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
        let rules = vec![SyntaxRule {
            pattern: Pattern::List(vec![
                Pattern::Literal("my-let".to_string()),
                Pattern::List(vec![Pattern::Ellipsis(Box::new(Pattern::List(vec![
                    Pattern::Variable("var".to_string()),
                    Pattern::Variable("val".to_string()),
                ])))]),
                Pattern::Variable("body".to_string()),
            ]),
            template: Template::List(vec![
                Template::Literal("let-impl".to_string()),
                Template::List(vec![Template::Ellipsis(Box::new(Template::List(vec![
                    Template::Variable("var".to_string()),
                    Template::Variable("val".to_string()),
                ])))]),
                Template::Variable("body".to_string()),
            ]),
        }];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);

        let input = Expr::List(vec![
            create_test_expr("my-let"),
            Expr::List(vec![
                Expr::List(vec![create_test_expr("x"), create_test_expr("1")]),
                Expr::List(vec![create_test_expr("y"), create_test_expr("2")]),
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
        let rules = vec![SyntaxRule {
            pattern: Pattern::List(vec![
                Pattern::Literal("nested-test".to_string()),
                Pattern::NestedEllipsis(Box::new(Pattern::Variable("item".to_string())), 1),
            ]),
            template: Template::List(vec![
                Template::Literal("nested-result".to_string()),
                Template::NestedEllipsis(Box::new(Template::Variable("item".to_string())), 1),
            ]),
        }];

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

    #[test]
    fn test_vector_pattern_matching() {
        let literals = vec![];
        let rules = vec![SyntaxRule {
            pattern: Pattern::List(vec![
                Pattern::Literal("vector-test".to_string()),
                Pattern::Vector(vec![
                    Pattern::Variable("x".to_string()),
                    Pattern::Variable("y".to_string()),
                ]),
            ]),
            template: Template::List(vec![
                Template::Literal("vector-result".to_string()),
                Template::Vector(vec![
                    Template::Variable("y".to_string()),
                    Template::Variable("x".to_string()),
                ]),
            ]),
        }];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);

        let input = Expr::List(vec![
            create_test_expr("vector-test"),
            Expr::Vector(vec![
                create_test_expr("first"),
                create_test_expr("second"),
            ]),
        ]);

        let result = transformer.transform(&input);
        assert!(result.is_ok());

        match result.unwrap() {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 2);
                assert_eq!(exprs[0], create_test_expr("vector-result"));
                
                // Check that the vector result has swapped elements
                match &exprs[1] {
                    Expr::Vector(vec_exprs) => {
                        assert_eq!(vec_exprs.len(), 2);
                        assert_eq!(vec_exprs[0], create_test_expr("second"));
                        assert_eq!(vec_exprs[1], create_test_expr("first"));
                    }
                    _ => panic!("Expected vector result"),
                }
            }
            _ => panic!("Expected list result"),
        }
    }

    #[test]
    fn test_vector_pattern_length_mismatch() {
        let literals = vec![];
        let rules = vec![SyntaxRule {
            pattern: Pattern::List(vec![
                Pattern::Literal("vector-test".to_string()),
                Pattern::Vector(vec![
                    Pattern::Variable("x".to_string()),
                    Pattern::Variable("y".to_string()),
                ]),
            ]),
            template: Template::List(vec![
                Template::Literal("vector-result".to_string()),
                Template::Variable("x".to_string()),
            ]),
        }];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);

        // Vector with wrong length
        let input = Expr::List(vec![
            create_test_expr("vector-test"),
            Expr::Vector(vec![
                create_test_expr("only-one"),
            ]),
        ]);

        let result = transformer.transform(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_dotted_pattern_matching() {
        let literals = vec![];
        let rules = vec![SyntaxRule {
            pattern: Pattern::List(vec![
                Pattern::Literal("dotted-test".to_string()),
                Pattern::Dotted(
                    vec![Pattern::Variable("first".to_string())],
                    Box::new(Pattern::Variable("rest".to_string()))
                ),
            ]),
            template: Template::List(vec![
                Template::Literal("dotted-result".to_string()),
                Template::Variable("first".to_string()),
                Template::Variable("rest".to_string()),
            ]),
        }];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);

        let input = Expr::List(vec![
            create_test_expr("dotted-test"),
            Expr::List(vec![
                create_test_expr("first-item"),
                create_test_expr("second-item"),
                create_test_expr("third-item"),
            ]),
        ]);

        let result = transformer.transform(&input);
        assert!(result.is_ok());

        match result.unwrap() {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(exprs[0], create_test_expr("dotted-result"));
                assert_eq!(exprs[1], create_test_expr("first-item"));
                
                // Check that rest is a list of remaining items
                match &exprs[2] {
                    Expr::List(rest_exprs) => {
                        assert_eq!(rest_exprs.len(), 2);
                        assert_eq!(rest_exprs[0], create_test_expr("second-item"));
                        assert_eq!(rest_exprs[1], create_test_expr("third-item"));
                    }
                    _ => panic!("Expected list for rest pattern"),
                }
            }
            _ => panic!("Expected list result"),
        }
    }

    #[test]
    fn test_dotted_pattern_empty_rest() {
        let literals = vec![];
        let rules = vec![SyntaxRule {
            pattern: Pattern::List(vec![
                Pattern::Literal("dotted-test".to_string()),
                Pattern::Dotted(
                    vec![Pattern::Variable("first".to_string())],
                    Box::new(Pattern::Variable("rest".to_string()))
                ),
            ]),
            template: Template::List(vec![
                Template::Literal("dotted-result".to_string()),
                Template::Variable("first".to_string()),
                Template::Variable("rest".to_string()),
            ]),
        }];

        let transformer = lambdust::macros::SyntaxRulesTransformer::new(literals, rules);

        // Only one item, so rest should be empty
        let input = Expr::List(vec![
            create_test_expr("dotted-test"),
            Expr::List(vec![
                create_test_expr("only-item"),
            ]),
        ]);

        let result = transformer.transform(&input);
        assert!(result.is_ok());

        match result.unwrap() {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(exprs[0], create_test_expr("dotted-result"));
                assert_eq!(exprs[1], create_test_expr("only-item"));
                
                // Check that rest is empty list
                match &exprs[2] {
                    Expr::List(rest_exprs) => {
                        assert_eq!(rest_exprs.len(), 0);
                    }
                    _ => panic!("Expected empty list for rest pattern"),
                }
            }
            _ => panic!("Expected list result"),
        }
    }
}
