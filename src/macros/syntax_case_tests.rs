//! Tests for syntax-case macro system

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ast::{Expr, Literal};
    use crate::lexer::SchemeNumber;
    use syntax_case::*;
    use pattern_matching::*;
    use hygiene::*;

    fn create_test_environment() -> HygienicEnvironment {
        HygienicEnvironment::new()
    }

    #[test]
    fn test_simple_pattern_matching() {
        let env = create_test_environment();
        let literals = vec![];
        let matcher = PatternMatcher::new(literals);
        let def_env = create_test_environment();
        let usage_env = create_test_environment();
        let context = ExpansionContext::new(def_env, usage_env);

        // Test simple variable pattern
        let pattern = Pattern::Variable("x".to_string());
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        
        let result = matcher.match_pattern(&pattern, &expr, &context).unwrap();
        assert!(result.success);
        assert!(result.bindings.contains_key("x"));
        
        match result.bindings.get("x").unwrap() {
            BindingValue::Single(matched_expr) => {
                match matched_expr {
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(42))) => (),
                    _ => panic!("Expected integer 42"),
                }
            }
            _ => panic!("Expected single binding"),
        }
    }

    #[test]
    fn test_list_pattern_matching() {
        let env = create_test_environment();
        let literals = vec![];
        let matcher = PatternMatcher::new(literals);
        let def_env = create_test_environment();
        let usage_env = create_test_environment();
        let context = ExpansionContext::new(def_env, usage_env);

        // Test list pattern (+ x y)
        let pattern = Pattern::List(vec![
            Pattern::Variable("op".to_string()),
            Pattern::Variable("x".to_string()),
            Pattern::Variable("y".to_string()),
        ]);
        
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]);
        
        let result = matcher.match_pattern(&pattern, &expr, &context).unwrap();
        assert!(result.success);
        assert!(result.bindings.contains_key("op"));
        assert!(result.bindings.contains_key("x"));
        assert!(result.bindings.contains_key("y"));
    }

    #[test]
    fn test_literal_pattern_matching() {
        let env = create_test_environment();
        let literals = vec!["if".to_string()];
        let matcher = PatternMatcher::new(literals);
        let def_env = create_test_environment();
        let usage_env = create_test_environment();
        let context = ExpansionContext::new(def_env, usage_env);

        // Test literal pattern matching
        let pattern = Pattern::Literal("if".to_string());
        let expr = Expr::Variable("if".to_string());
        
        let result = matcher.match_pattern(&pattern, &expr, &context).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_syntax_case_transformer() {
        let env = create_test_environment();
        let literals = vec![];
        
        // Create a simple syntax-case macro: (test x) => x
        let clause = SyntaxCaseClause {
            pattern: Pattern::List(vec![
                Pattern::Variable("_".to_string()), // macro name
                Pattern::Variable("x".to_string()),
            ]),
            guard: None,
            body: SyntaxCaseBody::Template(Template::Variable("x".to_string())),
        };
        
        let transformer = SyntaxCaseTransformer::new(
            literals,
            vec![clause],
            env.clone(),
        );
        
        let input = Expr::List(vec![
            Expr::Variable("test".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ]);
        
        let result = transformer.transform(&input, &env).unwrap();
        
        match result {
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))) => (),
            _ => panic!("Expected integer 42, got: {:?}", result),
        }
    }

    #[test]
    fn test_syntax_case_with_guard() {
        let env = create_test_environment();
        let literals = vec![];
        
        // Create syntax-case with guard condition
        let clause = SyntaxCaseClause {
            pattern: Pattern::List(vec![
                Pattern::Variable("_".to_string()),
                Pattern::Variable("x".to_string()),
            ]),
            guard: Some(Expr::Variable("#t".to_string())), // Always true for test
            body: SyntaxCaseBody::Template(Template::Variable("x".to_string())),
        };
        
        let transformer = SyntaxCaseTransformer::new(
            literals,
            vec![clause],
            env.clone(),
        );
        
        let input = Expr::List(vec![
            Expr::Variable("test".to_string()),
            Expr::Literal(Literal::String("success".to_string())),
        ]);
        
        let result = transformer.transform(&input, &env).unwrap();
        
        match result {
            Expr::Literal(Literal::String(s)) if s == "success" => (),
            _ => panic!("Expected string 'success', got: {:?}", result),
        }
    }

    #[test] 
    fn test_ellipsis_pattern_matching() {
        let env = create_test_environment();
        let literals = vec![];
        let matcher = PatternMatcher::new(literals);
        let def_env = create_test_environment();
        let usage_env = create_test_environment();
        let context = ExpansionContext::new(def_env, usage_env);

        // Test ellipsis pattern (op args ...)
        let pattern = Pattern::List(vec![
            Pattern::Variable("op".to_string()),
            Pattern::Ellipsis(Box::new(Pattern::Variable("args".to_string()))),
        ]);
        
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
        ]);
        
        let result = matcher.match_pattern(&pattern, &expr, &context).unwrap();
        assert!(result.success);
        assert!(result.bindings.contains_key("op"));
        assert!(result.bindings.contains_key("args"));
        
        // Check that args is bound to a list
        match result.bindings.get("args").unwrap() {
            BindingValue::List(args) => {
                assert_eq!(args.len(), 3);
            }
            _ => panic!("Expected list binding for args"),
        }
    }

    #[test]
    fn test_syntax_object_creation() {
        let env = create_test_environment();
        let def_env = create_test_environment();
        let usage_env = create_test_environment();
        let context = ExpansionContext::new(def_env, usage_env);
        
        let expr = Expr::Variable("test".to_string());
        let syntax_obj = SyntaxObject::new(expr.clone(), context);
        
        assert_eq!(syntax_obj.expression, expr);
        assert!(syntax_obj.source_info.is_none());
    }

    #[test]
    fn test_parse_syntax_case_pattern() {
        let literals = vec!["if".to_string()];
        
        // Test parsing variable pattern
        let expr = Expr::Variable("x".to_string());
        let pattern = parse_syntax_case_pattern(&expr, &literals).unwrap();
        assert_eq!(pattern, Pattern::Variable("x".to_string()));
        
        // Test parsing literal pattern
        let expr = Expr::Variable("if".to_string());
        let pattern = parse_syntax_case_pattern(&expr, &literals).unwrap();
        assert_eq!(pattern, Pattern::Literal("if".to_string()));
        
        // Test parsing list pattern
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Variable("test".to_string()),
            Expr::Variable("then".to_string()),
        ]);
        let pattern = parse_syntax_case_pattern(&expr, &literals).unwrap();
        
        match pattern {
            Pattern::List(patterns) => {
                assert_eq!(patterns.len(), 3);
                assert_eq!(patterns[0], Pattern::Literal("if".to_string()));
                assert_eq!(patterns[1], Pattern::Variable("test".to_string()));
                assert_eq!(patterns[2], Pattern::Variable("then".to_string()));
            }
            _ => panic!("Expected list pattern"),
        }
    }

    #[test]
    fn test_parse_syntax_case_template() {
        // Test parsing variable template
        let expr = Expr::Variable("x".to_string());
        let template = parse_syntax_case_template(&expr).unwrap();
        assert_eq!(template, Template::Variable("x".to_string()));
        
        // Test parsing list template
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Variable("test".to_string()),
            Expr::Variable("then".to_string()),
        ]);
        let template = parse_syntax_case_template(&expr).unwrap();
        
        match template {
            Template::List(templates) => {
                assert_eq!(templates.len(), 3);
                assert_eq!(templates[0], Template::Variable("if".to_string()));
                assert_eq!(templates[1], Template::Variable("test".to_string()));
                assert_eq!(templates[2], Template::Variable("then".to_string()));
            }
            _ => panic!("Expected list template"),
        }
    }

    #[test]
    fn test_advanced_conditional_patterns() {
        let env = create_test_environment();
        let literals = vec![];
        let matcher = PatternMatcher::with_advanced_features(literals);
        let def_env = create_test_environment();
        let usage_env = create_test_environment();
        let context = ExpansionContext::new(def_env, usage_env);

        // Test conditional pattern with guard
        let pattern = Pattern::Conditional {
            pattern: Box::new(Pattern::Variable("x".to_string())),
            guard: Expr::Literal(Literal::Boolean(true)),
        };

        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let result = matcher.match_pattern(&pattern, &expr, &context).unwrap();
        assert!(result.success);
        assert!(result.bindings.contains_key("x"));
    }

    #[test]
    fn test_type_guard_patterns() {
        let env = create_test_environment();
        let literals = vec![];
        let matcher = PatternMatcher::with_advanced_features(literals);
        let def_env = create_test_environment();
        let usage_env = create_test_environment();
        let context = ExpansionContext::new(def_env, usage_env);

        // Test type guard for number
        let pattern = Pattern::TypeGuard {
            pattern: Box::new(Pattern::Variable("x".to_string())),
            expected_type: TypePattern::Number,
        };

        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(123)));
        let result = matcher.match_pattern(&pattern, &expr, &context).unwrap();
        assert!(result.success);

        // Test type guard failure
        let expr_string = Expr::Literal(Literal::String("not a number".to_string()));
        let result = matcher.match_pattern(&pattern, &expr_string, &context).unwrap();
        assert!(!result.success);
    }

    #[test]
    fn test_and_or_patterns() {
        let env = create_test_environment();
        let literals = vec![];
        let matcher = PatternMatcher::with_advanced_features(literals);
        let def_env = create_test_environment();
        let usage_env = create_test_environment();
        let context = ExpansionContext::new(def_env, usage_env);

        // Test AND pattern
        let and_pattern = Pattern::And(vec![
            Pattern::TypeGuard {
                pattern: Box::new(Pattern::Variable("x".to_string())),
                expected_type: TypePattern::Number,
            },
            Pattern::Range {
                start: Some(0),
                end: Some(100),
                inclusive: true,
            },
        ]);

        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(50)));
        let result = matcher.match_pattern(&and_pattern, &expr, &context).unwrap();
        assert!(result.success);

        // Test OR pattern
        let or_pattern = Pattern::Or(vec![
            Pattern::TypeGuard {
                pattern: Box::new(Pattern::Variable("y".to_string())),
                expected_type: TypePattern::String,
            },
            Pattern::TypeGuard {
                pattern: Box::new(Pattern::Variable("y".to_string())),
                expected_type: TypePattern::Number,
            },
        ]);

        let expr_num = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let result = matcher.match_pattern(&or_pattern, &expr_num, &context).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_range_patterns() {
        let env = create_test_environment();
        let literals = vec![];
        let matcher = PatternMatcher::with_advanced_features(literals);
        let def_env = create_test_environment();
        let usage_env = create_test_environment();
        let context = ExpansionContext::new(def_env, usage_env);

        // Test inclusive range pattern
        let pattern = Pattern::Range {
            start: Some(10),
            end: Some(20),
            inclusive: true,
        };

        let expr_in_range = Expr::Literal(Literal::Number(SchemeNumber::Integer(15)));
        let result = matcher.match_pattern(&pattern, &expr_in_range, &context).unwrap();
        assert!(result.success);

        let expr_out_range = Expr::Literal(Literal::Number(SchemeNumber::Integer(25)));
        let result = matcher.match_pattern(&pattern, &expr_out_range, &context).unwrap();
        assert!(!result.success);
    }

    #[test]
    fn test_regex_patterns() {
        let env = create_test_environment();
        let literals = vec![];
        let matcher = PatternMatcher::with_advanced_features(literals);
        let def_env = create_test_environment();
        let usage_env = create_test_environment();
        let context = ExpansionContext::new(def_env, usage_env);

        // Test regex pattern for alphabetic strings
        let pattern = Pattern::Regex("^[a-zA-Z]+$".to_string());

        let expr_alpha = Expr::Literal(Literal::String("hello".to_string()));
        let result = matcher.match_pattern(&pattern, &expr_alpha, &context).unwrap();
        assert!(result.success);

        let expr_numeric = Expr::Literal(Literal::String("123".to_string()));
        let result = matcher.match_pattern(&pattern, &expr_numeric, &context).unwrap();
        assert!(!result.success);
    }

    #[test]
    fn test_not_patterns() {
        let env = create_test_environment();
        let literals = vec![];
        let matcher = PatternMatcher::with_advanced_features(literals);
        let def_env = create_test_environment();
        let usage_env = create_test_environment();
        let context = ExpansionContext::new(def_env, usage_env);

        // Test NOT pattern
        let pattern = Pattern::Not(Box::new(Pattern::TypeGuard {
            pattern: Box::new(Pattern::Variable("x".to_string())),
            expected_type: TypePattern::String,
        }));

        let expr_number = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let result = matcher.match_pattern(&pattern, &expr_number, &context).unwrap();
        assert!(result.success); // NOT string pattern matches number

        let expr_string = Expr::Literal(Literal::String("text".to_string()));
        let result = matcher.match_pattern(&pattern, &expr_string, &context).unwrap();
        assert!(!result.success); // NOT string pattern doesn't match string
    }

    #[test]
    fn test_advanced_template_expansion() {
        let env = create_test_environment();
        let literals = vec![];
        
        // Create syntax-case with conditional template
        let clause = SyntaxCaseClause {
            pattern: Pattern::List(vec![
                Pattern::Variable("_".to_string()),
                Pattern::Variable("condition".to_string()),
                Pattern::Variable("then_part".to_string()),
                Pattern::Variable("else_part".to_string()),
            ]),
            guard: None,
            body: SyntaxCaseBody::Template(Template::Conditional {
                condition: Expr::Variable("condition".to_string()),
                then_template: Box::new(Template::Variable("then_part".to_string())),
                else_template: Some(Box::new(Template::Variable("else_part".to_string()))),
            }),
        };
        
        let transformer = SyntaxCaseTransformer::new(
            literals,
            vec![clause],
            env.clone(),
        );
        
        let input = Expr::List(vec![
            Expr::Variable("conditional_macro".to_string()),
            Expr::Literal(Literal::Boolean(true)),
            Expr::Literal(Literal::String("true_case".to_string())),
            Expr::Literal(Literal::String("false_case".to_string())),
        ]);
        
        let result = transformer.transform(&input, &env).unwrap();
        
        // Should expand to the then_part since condition is true
        match result {
            Expr::Literal(Literal::String(s)) if s == "true_case" => (),
            _ => panic!("Expected 'true_case', got: {:?}", result),
        }
    }

    #[test]
    fn test_repeat_template_expansion() {
        let env = create_test_environment();
        let literals = vec![];
        
        // Create syntax-case with repeat template
        let clause = SyntaxCaseClause {
            pattern: Pattern::List(vec![
                Pattern::Variable("_".to_string()),
                Pattern::Ellipsis(Box::new(Pattern::Variable("items".to_string()))),
            ]),
            guard: None,
            body: SyntaxCaseBody::Template(Template::Repeat {
                template: Box::new(Template::Variable("items".to_string())),
                separator: Some(",".to_string()),
                min_count: 1,
                max_count: Some(5),
            }),
        };
        
        let transformer = SyntaxCaseTransformer::new(
            literals,
            vec![clause],
            env.clone(),
        );
        
        let input = Expr::List(vec![
            Expr::Variable("repeat_macro".to_string()),
            Expr::Literal(Literal::String("a".to_string())),
            Expr::Literal(Literal::String("b".to_string())),
            Expr::Literal(Literal::String("c".to_string())),
        ]);
        
        let result = transformer.transform(&input, &env).unwrap();
        
        match result {
            Expr::List(exprs) => {
                // Should have items with separators: a , b , c
                assert_eq!(exprs.len(), 5); // 3 items + 2 separators
            }
            _ => panic!("Expected list result, got: {:?}", result),
        }
    }

    #[test]
    fn test_transform_template_expansion() {
        let env = create_test_environment();
        let literals = vec![];
        
        // Create syntax-case with transform template
        let clause = SyntaxCaseClause {
            pattern: Pattern::List(vec![
                Pattern::Variable("_".to_string()),
                Pattern::Variable("text".to_string()),
            ]),
            guard: None,
            body: SyntaxCaseBody::Template(Template::Transform {
                template: Box::new(Template::Variable("text".to_string())),
                function: "upcase".to_string(),
            }),
        };
        
        let transformer = SyntaxCaseTransformer::new(
            literals,
            vec![clause],
            env.clone(),
        );
        
        let input = Expr::List(vec![
            Expr::Variable("upcase_macro".to_string()),
            Expr::Variable("hello".to_string()),
        ]);
        
        let result = transformer.transform(&input, &env).unwrap();
        
        match result {
            Expr::Variable(name) if name == "HELLO" => (),
            _ => panic!("Expected 'HELLO', got: {:?}", result),
        }
    }

    #[test]
    fn test_complex_macro_expansion() {
        let env = create_test_environment();
        let literals = vec!["=>".to_string()];
        
        // Create a macro that transforms (when test body ...) => (if test (begin body ...) (void))
        let clause = SyntaxCaseClause {
            pattern: Pattern::List(vec![
                Pattern::Variable("_".to_string()), // macro name
                Pattern::Variable("test".to_string()),
                Pattern::Ellipsis(Box::new(Pattern::Variable("body".to_string()))),
            ]),
            guard: None,
            body: SyntaxCaseBody::Template(Template::List(vec![
                Template::Variable("if".to_string()),
                Template::Variable("test".to_string()),
                Template::List(vec![
                    Template::Variable("begin".to_string()),
                    Template::Ellipsis(Box::new(Template::Variable("body".to_string()))),
                ]),
                Template::List(vec![Template::Variable("void".to_string())]),
            ])),
        };
        
        let transformer = SyntaxCaseTransformer::new(
            literals,
            vec![clause],
            env.clone(),
        );
        
        let input = Expr::List(vec![
            Expr::Variable("when".to_string()),
            Expr::Variable("#t".to_string()),
            Expr::List(vec![
                Expr::Variable("display".to_string()),
                Expr::Literal(Literal::String("hello".to_string())),
            ]),
            Expr::List(vec![
                Expr::Variable("newline".to_string()),
            ]),
        ]);
        
        let result = transformer.transform(&input, &env).unwrap();
        
        // Should expand to: (if #t (begin (display "hello") (newline)) (void))
        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 4);
                match &exprs[0] {
                    Expr::Variable(s) if s == "if" => (),
                    _ => panic!("Expected 'if' as first element"),
                }
                match &exprs[2] {
                    Expr::List(begin_exprs) => {
                        match &begin_exprs[0] {
                            Expr::Variable(s) if s == "begin" => (),
                            _ => panic!("Expected 'begin'"),
                        }
                        assert_eq!(begin_exprs.len(), 3); // begin + 2 body expressions
                    }
                    _ => panic!("Expected list for then clause"),
                }
            }
            _ => panic!("Expected list result"),
        }
    }
}