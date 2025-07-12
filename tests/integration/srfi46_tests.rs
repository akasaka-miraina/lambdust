//! Comprehensive test suite for SRFI 46 nested ellipsis support
//!
//! This module contains extensive tests for the nested ellipsis functionality,
//! ensuring complete compliance with the SRFI 46 specification.

#[cfg(test)]
mod tests {
    use super::super::srfi46_ellipsis::*;
    use super::super::pattern_matching::{Pattern, Template, BindingValue};
    use super::super::hygiene::{ExpansionContext, HygienicEnvironment};
    use crate::ast::Expr;
    use std::collections::HashMap;

    fn create_test_context() -> ExpansionContext {
        let env = HygienicEnvironment::new();
        ExpansionContext::new(env.clone(), env)
    }

    #[test]
    fn test_nested_ellipsis_processor_creation() {
        let processor = NestedEllipsisProcessor::new();
        // assert_eq!(processor.max_nesting_depth, 10); // Private field
        
        let metrics = processor.metrics();
        assert_eq!(metrics.pattern_matches_attempted, 0);
        assert_eq!(metrics.pattern_matches_successful, 0);
        assert_eq!(metrics.template_expansions, 0);
        
        let _custom_processor = NestedEllipsisProcessor::with_max_depth(5);
        // assert_eq!(custom_processor.max_nesting_depth, 5); // Private field
    }

    #[test]
    fn test_ellipsis_context_operations() {
        let mut ctx = EllipsisContext::new();
        assert_eq!(ctx.current_depth, 0);
        assert_eq!(ctx.max_depth, 0);
        assert_eq!(ctx.iteration_counts.len(), 0);
        
        ctx.enter_level();
        assert_eq!(ctx.current_depth, 1);
        assert_eq!(ctx.max_depth, 1);
        assert_eq!(ctx.iteration_counts.len(), 1);
        assert_eq!(ctx.iteration_counts[0], 0);
        
        ctx.increment_iteration();
        assert_eq!(ctx.iteration_counts[0], 1);
        
        ctx.enter_level();
        assert_eq!(ctx.current_depth, 2);
        assert_eq!(ctx.max_depth, 2);
        assert_eq!(ctx.iteration_counts.len(), 2);
        
        ctx.exit_level();
        assert_eq!(ctx.current_depth, 1);
        assert_eq!(ctx.iteration_counts.len(), 1);
        
        ctx.exit_level();
        assert_eq!(ctx.current_depth, 0);
        assert_eq!(ctx.iteration_counts.len(), 0);
    }

    #[test]
    fn test_multi_dim_value_creation() {
        let scalar = MultiDimValue::Scalar(Expr::Variable("x".to_string()));
        match scalar {
            MultiDimValue::Scalar(_) => {},
            _ => panic!("Expected scalar value"),
        }
        
        let array1d = MultiDimValue::Array1D(vec![
            Expr::Variable("a".to_string()),
            Expr::Variable("b".to_string()),
        ]);
        match array1d {
            MultiDimValue::Array1D(exprs) => assert_eq!(exprs.len(), 2),
            _ => panic!("Expected 1D array"),
        }
        
        let array2d = MultiDimValue::Array2D(vec![
            vec![Expr::Variable("x".to_string())],
            vec![Expr::Variable("y".to_string()), Expr::Variable("z".to_string())],
        ]);
        match array2d {
            MultiDimValue::Array2D(exprs) => {
                assert_eq!(exprs.len(), 2);
                assert_eq!(exprs[0].len(), 1);
                assert_eq!(exprs[1].len(), 2);
            },
            _ => panic!("Expected 2D array"),
        }
    }

    #[test]
    fn test_single_level_ellipsis_matching() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        let pattern = Pattern::NestedEllipsis(
            Box::new(Pattern::Variable("x".to_string())),
            1
        );
        
        let expr = Expr::List(vec![
            Expr::Variable("a".to_string()),
            Expr::Variable("b".to_string()),
            Expr::Variable("c".to_string()),
        ]);
        
        let result = processor.match_nested_ellipsis(&pattern, &expr, 1, &context);
        assert!(result.is_ok());
        
        let match_result = result.unwrap();
        assert!(match_result.success);
        
        let metrics = processor.metrics();
        assert_eq!(metrics.pattern_matches_attempted, 1);
        assert_eq!(metrics.pattern_matches_successful, 1);
        assert_eq!(metrics.max_nesting_depth_seen, 1);
    }

    #[test]
    fn test_double_level_ellipsis_matching() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        let pattern = Pattern::NestedEllipsis(
            Box::new(Pattern::List(vec![
                Pattern::NestedEllipsis(
                    Box::new(Pattern::Variable("x".to_string())),
                    1
                )
            ])),
            2
        );
        
        let expr = Expr::List(vec![
            Expr::List(vec![
                Expr::Variable("a".to_string()),
                Expr::Variable("b".to_string()),
            ]),
            Expr::List(vec![
                Expr::Variable("c".to_string()),
                Expr::Variable("d".to_string()),
                Expr::Variable("e".to_string()),
            ]),
        ]);
        
        let result = processor.match_nested_ellipsis(&pattern, &expr, 2, &context);
        assert!(result.is_ok());
        
        let match_result = result.unwrap();
        assert!(match_result.success);
        
        let metrics = processor.metrics();
        assert_eq!(metrics.max_nesting_depth_seen, 2);
    }

    #[test]
    fn test_triple_level_ellipsis_matching() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        let pattern = Pattern::NestedEllipsis(
            Box::new(Pattern::List(vec![
                Pattern::NestedEllipsis(
                    Box::new(Pattern::List(vec![
                        Pattern::NestedEllipsis(
                            Box::new(Pattern::Variable("x".to_string())),
                            1
                        )
                    ])),
                    2
                )
            ])),
            3
        );
        
        let expr = Expr::List(vec![
            Expr::List(vec![
                Expr::List(vec![
                    Expr::Variable("a".to_string()),
                    Expr::Variable("b".to_string()),
                ]),
                Expr::List(vec![
                    Expr::Variable("c".to_string()),
                ]),
            ]),
            Expr::List(vec![
                Expr::List(vec![
                    Expr::Variable("d".to_string()),
                    Expr::Variable("e".to_string()),
                    Expr::Variable("f".to_string()),
                ]),
            ]),
        ]);
        
        let result = processor.match_nested_ellipsis(&pattern, &expr, 3, &context);
        assert!(result.is_ok());
        
        let match_result = result.unwrap();
        assert!(match_result.success);
        
        let metrics = processor.metrics();
        assert_eq!(metrics.max_nesting_depth_seen, 3);
    }

    #[test]
    fn test_nesting_level_mismatch_error() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        let pattern = Pattern::NestedEllipsis(
            Box::new(Pattern::Variable("x".to_string())),
            3  // Pattern expects level 3
        );
        
        let expr = Expr::Variable("test".to_string());
        
        let result = processor.match_nested_ellipsis(&pattern, &expr, 1, &context);
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("nesting level mismatch"));
    }

    #[test]
    fn test_max_depth_exceeded_error() {
        let mut processor = NestedEllipsisProcessor::with_max_depth(3);
        let context = create_test_context();
        
        let pattern = Pattern::NestedEllipsis(
            Box::new(Pattern::Variable("x".to_string())),
            5  // Exceeds max depth of 3
        );
        
        let expr = Expr::Variable("test".to_string());
        
        let result = processor.match_nested_ellipsis(&pattern, &expr, 5, &context);
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("exceeds maximum allowed depth"));
    }

    #[test]
    fn test_simple_template_expansion() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        let mut bindings = HashMap::new();
        bindings.insert(
            "x".to_string(),
            BindingValue::List(vec![
                Expr::Variable("a".to_string()),
                Expr::Variable("b".to_string()),
                Expr::Variable("c".to_string()),
            ])
        );
        
        let template = Template::NestedEllipsis(
            Box::new(Template::Variable("x".to_string())),
            1
        );
        
        let result = processor.expand_nested_ellipsis(&template, &bindings, 1, &context);
        assert!(result.is_ok());
        
        let expanded = result.unwrap();
        match expanded {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
            },
            _ => panic!("Expected list expansion"),
        }
        
        let metrics = processor.metrics();
        assert_eq!(metrics.template_expansions, 1);
    }

    #[test]
    fn test_nested_template_expansion() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        let mut bindings = HashMap::new();
        bindings.insert(
            "items".to_string(),
            BindingValue::List(vec![
                Expr::List(vec![
                    Expr::Variable("first".to_string()),
                    Expr::Variable("second".to_string()),
                ]),
                Expr::List(vec![
                    Expr::Variable("third".to_string()),
                    Expr::Variable("fourth".to_string()),
                ]),
            ])
        );
        
        let template = Template::NestedEllipsis(
            Box::new(Template::List(vec![
                Template::Literal("process".to_string()),
                Template::Variable("items".to_string()),
            ])),
            2
        );
        
        let result = processor.expand_nested_ellipsis(&template, &bindings, 2, &context);
        assert!(result.is_ok());
        
        let expanded = result.unwrap();
        match expanded {
            Expr::List(_) => {
                // Template expanded successfully
            },
            _ => panic!("Expected list template expansion"),
        }
        
        let metrics = processor.metrics();
        assert_eq!(metrics.template_expansions, 1);
    }

    #[test]
    fn test_template_level_mismatch_error() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        let bindings = HashMap::new();
        
        let template = Template::NestedEllipsis(
            Box::new(Template::Variable("x".to_string())),
            2  // Template expects level 2
        );
        
        let result = processor.expand_nested_ellipsis(&template, &bindings, 1, &context);
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("nesting level mismatch"));
    }

    #[test]
    fn test_empty_bindings_template_expansion() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        let bindings = HashMap::new();
        
        let template = Template::NestedEllipsis(
            Box::new(Template::Variable("missing".to_string())),
            1
        );
        
        let result = processor.expand_nested_ellipsis(&template, &bindings, 1, &context);
        assert!(result.is_ok());
        
        // Should return the variable name when binding is missing
        let expanded = result.unwrap();
        match expanded {
            Expr::Variable(name) => assert_eq!(name, "missing"),
            _ => panic!("Expected variable for missing binding"),
        }
    }

    #[test]
    fn test_mixed_pattern_and_template() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        // First match a pattern
        let pattern = Pattern::NestedEllipsis(
            Box::new(Pattern::Variable("data".to_string())),
            1
        );
        
        let expr = Expr::List(vec![
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
            Expr::Variable("z".to_string()),
        ]);
        
        let match_result = processor.match_nested_ellipsis(&pattern, &expr, 1, &context);
        assert!(match_result.is_ok());
        
        // Then expand a template
        let mut bindings = HashMap::new();
        bindings.insert(
            "data".to_string(),
            BindingValue::List(vec![
                Expr::Variable("transformed_x".to_string()),
                Expr::Variable("transformed_y".to_string()),
                Expr::Variable("transformed_z".to_string()),
            ])
        );
        
        let template = Template::NestedEllipsis(
            Box::new(Template::List(vec![
                Template::Literal("quote".to_string()),
                Template::Variable("data".to_string()),
            ])),
            1
        );
        
        let expand_result = processor.expand_nested_ellipsis(&template, &bindings, 1, &context);
        assert!(expand_result.is_ok());
        
        let metrics = processor.metrics();
        assert_eq!(metrics.pattern_matches_attempted, 1);
        assert_eq!(metrics.pattern_matches_successful, 1);
        assert_eq!(metrics.template_expansions, 1);
    }

    #[test]
    fn test_metrics_accuracy() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        let pattern = Pattern::NestedEllipsis(
            Box::new(Pattern::Variable("x".to_string())),
            1
        );
        
        let valid_expr = Expr::List(vec![Expr::Variable("a".to_string())]);
        let invalid_pattern = Pattern::NestedEllipsis(
            Box::new(Pattern::Variable("y".to_string())),
            5  // Will exceed depth
        );
        
        // Successful match
        let _ = processor.match_nested_ellipsis(&pattern, &valid_expr, 1, &context);
        
        // Failed match (depth exceeded)
        let _ = processor.match_nested_ellipsis(&invalid_pattern, &valid_expr, 5, &context);
        
        let metrics = processor.metrics();
        assert_eq!(metrics.pattern_matches_attempted, 2);
        assert_eq!(metrics.pattern_matches_successful, 1);
        assert_eq!(metrics.success_rate(), 50.0);
        assert!(metrics.total_processing_time_ns > 0);
        assert!(metrics.average_processing_time_us() > 0.0);
    }

    #[test]
    fn test_metrics_format_summary() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        let pattern = Pattern::NestedEllipsis(
            Box::new(Pattern::Variable("x".to_string())),
            1
        );
        
        let expr = Expr::List(vec![Expr::Variable("a".to_string())]);
        
        let _ = processor.match_nested_ellipsis(&pattern, &expr, 1, &context);
        
        let metrics = processor.metrics();
        let summary = metrics.format_summary();
        
        assert!(summary.contains("SRFI 46 Nested Ellipsis Metrics"));
        assert!(summary.contains("Pattern Matches"));
        assert!(summary.contains("Template Expansions"));
        assert!(summary.contains("Max Nesting Depth"));
        assert!(summary.contains("Average Time"));
        assert!(summary.contains("Total Processing"));
    }

    #[test]
    fn test_performance_under_load() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        let pattern = Pattern::NestedEllipsis(
            Box::new(Pattern::Variable("x".to_string())),
            1
        );
        
        let expr = Expr::List(vec![
            Expr::Variable("a".to_string()),
            Expr::Variable("b".to_string()),
            Expr::Variable("c".to_string()),
        ]);
        
        let start_time = std::time::Instant::now();
        
        // Run 1000 operations
        for _ in 0..1000 {
            let _ = processor.match_nested_ellipsis(&pattern, &expr, 1, &context);
        }
        
        let duration = start_time.elapsed();
        let metrics = processor.metrics();
        
        assert_eq!(metrics.pattern_matches_attempted, 1000);
        assert_eq!(metrics.pattern_matches_successful, 1000);
        assert_eq!(metrics.success_rate(), 100.0);
        
        // Performance should be reasonable (less than 10ms for 1000 operations)
        assert!(duration.as_millis() < 10);
        
        println!("Performance test: {} operations in {:?}", 
                metrics.pattern_matches_attempted, duration);
        println!("Average time per operation: {:.2}μs", 
                metrics.average_processing_time_us());
    }

    #[test]
    fn test_reset_metrics() {
        let mut processor = NestedEllipsisProcessor::new();
        let context = create_test_context();
        
        let pattern = Pattern::NestedEllipsis(
            Box::new(Pattern::Variable("x".to_string())),
            1
        );
        
        let expr = Expr::List(vec![Expr::Variable("a".to_string())]);
        
        // Perform some operations
        let _ = processor.match_nested_ellipsis(&pattern, &expr, 1, &context);
        let _ = processor.match_nested_ellipsis(&pattern, &expr, 1, &context);
        
        let metrics_before = processor.metrics().clone();
        assert_eq!(metrics_before.pattern_matches_attempted, 2);
        
        // Reset metrics
        processor.reset_metrics();
        
        let metrics_after = processor.metrics();
        assert_eq!(metrics_after.pattern_matches_attempted, 0);
        assert_eq!(metrics_after.pattern_matches_successful, 0);
        assert_eq!(metrics_after.template_expansions, 0);
        assert_eq!(metrics_after.total_processing_time_ns, 0);
    }
}