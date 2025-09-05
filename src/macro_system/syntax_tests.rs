//! Comprehensive tests for the syntax object system and hygiene
//!
//! This module contains extensive tests for all components of the syntax object
//! system, including hygiene, scope management, syntax-case, and integration.

#[cfg(test)]
mod tests {
    use super::super::{
        advanced_hygiene::*, quasisyntax::*, scope_management::*, syntax_case::*,
        syntax_integration::*, syntax_objects::*,
    };
    use crate::ast::{Binding, Expr, Formals, Literal};
    use crate::diagnostics::{Span, Spanned};
    use std::collections::HashMap;

    // Helper function to create test contexts
    fn test_context() -> LexicalContext {
        LexicalContext::new(1, vec!["test".to_string()])
    }

    fn test_span() -> Span {
        Span::new(0, 10)
    }

    mod syntax_object_tests {
        use super::*;

        #[test]
        fn test_syntax_object_creation() {
            let context = test_context();
            let expr = Expr::Identifier("test".to_string());
            let span = test_span();

            let syntax = SyntaxObject::new(expr.clone(), span, context.clone());

            assert_eq!(syntax.expr, expr);
            assert_eq!(syntax.span, span);
            assert_eq!(syntax.context.context_id, context.context_id);
            assert!(syntax.is_identifier());
            assert_eq!(syntax.identifier_name(), Some("test"));
        }

        #[test]
        fn test_syntax_object_marks() {
            let context = test_context();
            let expr = Expr::Identifier("test".to_string());
            let span = test_span();

            let mut syntax = SyntaxObject::new(expr, span, context);

            assert!(syntax.marks.is_empty());

            syntax.add_mark(42);
            assert!(syntax.has_mark(42));
            assert!(!syntax.has_mark(43));

            syntax.remove_mark(42);
            assert!(!syntax.has_mark(42));
        }

        #[test]
        fn test_syntax_object_properties() {
            let context = test_context();
            let expr = Expr::Identifier("test".to_string());
            let span = test_span();

            let mut syntax = SyntaxObject::new(expr, span, context);

            syntax.set_property(
                "type".to_string(),
                SyntaxProperty::String("identifier".to_string()),
            );
            syntax.set_property("count".to_string(), SyntaxProperty::Number(42.0));

            assert_eq!(
                syntax.get_property("type"),
                Some(&SyntaxProperty::String("identifier".to_string()))
            );
            assert_eq!(
                syntax.get_property("count"),
                Some(&SyntaxProperty::Number(42.0))
            );
            assert_eq!(syntax.get_property("missing"), None);
        }

        #[test]
        fn test_syntax_object_list_conversion() {
            let context = test_context();
            let elements = vec![
                Spanned::new(Expr::Identifier("foo".to_string()), test_span()),
                Spanned::new(Expr::Literal(Literal::Number(42.0)), test_span()),
            ];
            let expr = Expr::List(elements);
            let syntax = SyntaxObject::new(expr, test_span(), context);

            assert!(syntax.is_list());
            let list = syntax.as_list().unwrap();
            assert_eq!(list.len(), 2);
            assert_eq!(list[0].identifier_name(), Some("foo"));
        }

        #[test]
        fn test_bound_identifier_equal() {
            let context = test_context();
            let span = test_span();

            let stx1 = SyntaxObject::new(Expr::Identifier("x".to_string()), span, context.clone());
            let mut stx2 = SyntaxObject::new(Expr::Identifier("x".to_string()), span, context);

            // Same context and marks
            assert!(syntax_utils::bound_identifier_equal(&stx1, &stx2));

            // Different marks
            stx2.add_mark(42);
            assert!(!syntax_utils::bound_identifier_equal(&stx1, &stx2));
        }

        #[test]
        fn test_free_identifier_equal() {
            let context = test_context();
            let span = test_span();

            let mut stx1 =
                SyntaxObject::new(Expr::Identifier("x".to_string()), span, context.clone());
            let mut stx2 = SyntaxObject::new(Expr::Identifier("x".to_string()), span, context);

            // Add same binding to both
            let binding = BindingInfo::new("x".to_string(), stx1.context.clone(), 0, false);
            stx1.add_binding("x".to_string(), binding.clone());
            stx2.add_binding("x".to_string(), binding);

            assert!(syntax_utils::free_identifier_equal(&stx1, &stx2));
        }

        #[test]
        fn test_syntax_with_source_text() {
            let context = test_context();
            let expr = Expr::Identifier("test".to_string());
            let span = test_span();
            let source = "test".to_string();

            let syntax = SyntaxObject::with_source(expr, span, context, source.clone());

            assert_eq!(syntax.source_text, Some(source));
        }
    }

    mod hygiene_tests {
        use super::*;

        #[test]
        fn test_mark_generation() {
            let mark1 = fresh_mark();
            let mark2 = fresh_mark();

            assert_ne!(mark1, mark2);
            assert!(mark2 > mark1);
        }

        #[test]
        fn test_mark_set_operations() {
            let mark1 = Mark::fresh();
            let mark2 = Mark::fresh();

            let mut set1 = MarkSet::singleton(mark1);
            let set2 = MarkSet::singleton(mark2);

            assert!(set1.contains(mark1));
            assert!(!set1.contains(mark2));

            let union = set1.union(&set2);
            assert!(union.contains(mark1));
            assert!(union.contains(mark2));
            assert_eq!(union.len(), 2);

            let intersection = set1.intersection(&set2);
            assert!(intersection.is_empty());
        }

        #[test]
        fn test_hygiene_resolver() {
            let mut resolver = HygieneResolver::new();
            let context = test_context();

            // Enter scope and add binding
            let mark = resolver.enter_scope();
            let marks = MarkSet::singleton(mark);
            let binding_idx =
                resolver.add_binding("x".to_string(), marks.clone(), context.clone(), false);

            // Add reference
            let ref_idx = resolver.add_reference("x".to_string(), marks, context);

            // Resolve bindings
            resolver.resolve_bindings().unwrap();

            // Check resolution
            let resolved = resolver.get_resolved_binding(ref_idx);
            assert!(resolved.is_some());
            assert_eq!(resolved.unwrap().name, "x");
        }

        #[test]
        fn test_hygiene_scoping() {
            let mut resolver = HygieneResolver::new();
            let context = test_context();

            // Outer binding
            let outer_marks = MarkSet::empty();
            resolver.add_binding("x".to_string(), outer_marks.clone(), context.clone(), false);

            // Inner scope
            let inner_mark = resolver.enter_scope();
            let inner_marks = MarkSet::singleton(inner_mark);
            resolver.add_binding("x".to_string(), inner_marks.clone(), context.clone(), false);

            // Reference in inner scope should bind to inner definition
            let ref_idx = resolver.add_reference("x".to_string(), inner_marks, context);
            resolver.resolve_bindings().unwrap();

            let resolved = resolver.get_resolved_binding(ref_idx);
            assert!(resolved.is_some());
            assert!(resolved.unwrap().marks.contains(inner_mark));
        }

        #[test]
        fn test_fresh_name_generation() {
            let mut resolver = HygieneResolver::new();
            let marks = MarkSet::singleton(Mark::fresh());

            let name1 = resolver.generate_fresh_name("x", &marks);
            let name2 = resolver.generate_fresh_name("x", &marks);

            // Same marks should produce same name
            assert_eq!(name1, name2);
            assert!(name1.contains("x"));
            assert!(name1.contains("#"));
        }

        #[test]
        fn test_identifier_equivalence() {
            let mark1 = Mark::fresh();
            let mark2 = Mark::fresh();
            let context = test_context();

            let marks1 = MarkSet::singleton(mark1);
            let marks2 = MarkSet::singleton(mark2);

            // Same name, same marks
            assert!(hygiene_utils::identifiers_equivalent(
                "x", &marks1, &context, "x", &marks1, &context
            ));

            // Same name, different marks
            assert!(!hygiene_utils::identifiers_equivalent(
                "x", &marks1, &context, "x", &marks2, &context
            ));

            // Different names
            assert!(!hygiene_utils::identifiers_equivalent(
                "x", &marks1, &context, "y", &marks1, &context
            ));
        }

        #[test]
        fn test_macro_mark_introduction() {
            let original_marks = MarkSet::empty();
            let intro_mark = Mark::fresh();

            let new_marks = hygiene_utils::introduce_macro_marks(&original_marks, intro_mark);

            assert!(new_marks.contains(intro_mark));
            assert_eq!(new_marks.len(), 1);
        }
    }

    mod syntax_case_tests {
        use super::*;

        #[test]
        fn test_pattern_variable_matching() {
            let context = test_context();
            let syntax =
                SyntaxObject::new(Expr::Identifier("foo".to_string()), test_span(), context);

            let pattern = SyntaxPattern::PatternVariable("x".to_string());
            let bindings = pattern.match_syntax(&syntax).unwrap();

            assert_eq!(bindings.get("x").unwrap().identifier_name(), Some("foo"));
        }

        #[test]
        fn test_literal_pattern_matching() {
            let context = test_context();
            let syntax =
                SyntaxObject::new(Expr::Literal(Literal::Number(42.0)), test_span(), context);

            let pattern = SyntaxPattern::Literal(Literal::Number(42.0));
            assert!(pattern.match_syntax(&syntax).is_ok());

            let wrong_pattern = SyntaxPattern::Literal(Literal::Number(43.0));
            assert!(wrong_pattern.match_syntax(&syntax).is_err());
        }

        #[test]
        fn test_list_pattern_matching() {
            let context = test_context();
            let elements = vec![
                Spanned::new(Expr::Identifier("foo".to_string()), test_span()),
                Spanned::new(Expr::Literal(Literal::Number(42.0)), test_span()),
            ];
            let syntax = SyntaxObject::new(Expr::List(elements), test_span(), context);

            let pattern = SyntaxPattern::List(vec![
                SyntaxPattern::PatternVariable("x".to_string()),
                SyntaxPattern::PatternVariable("y".to_string()),
            ]);

            let bindings = pattern.match_syntax(&syntax).unwrap();

            assert_eq!(bindings.get("x").unwrap().identifier_name(), Some("foo"));
            assert!(matches!(
                bindings.get("y").unwrap().expr,
                Expr::Literal(Literal::Number(n)) if n == 42.0
            ));
        }

        #[test]
        fn test_ellipsis_pattern_matching() {
            let context = test_context();
            let elements = vec![
                Spanned::new(Expr::Identifier("a".to_string()), test_span()),
                Spanned::new(Expr::Identifier("b".to_string()), test_span()),
                Spanned::new(Expr::Identifier("c".to_string()), test_span()),
            ];
            let syntax = SyntaxObject::new(Expr::List(elements), test_span(), context);

            let pattern = SyntaxPattern::Ellipsis {
                pattern: Box::new(SyntaxPattern::PatternVariable("x".to_string())),
                min_count: 0,
                max_count: None,
            };

            let bindings = pattern.match_syntax(&syntax).unwrap();
            let ellipsis_bindings = bindings.get_ellipsis("x").unwrap();

            assert_eq!(ellipsis_bindings.len(), 3);
            assert_eq!(ellipsis_bindings[0].identifier_name(), Some("a"));
            assert_eq!(ellipsis_bindings[1].identifier_name(), Some("b"));
            assert_eq!(ellipsis_bindings[2].identifier_name(), Some("c"));
        }

        #[test]
        fn test_template_expansion() {
            let mut bindings = SyntaxBindings::new();
            let context = test_context();

            let bound_syntax = SyntaxObject::new(
                Expr::Identifier("foo".to_string()),
                test_span(),
                context.clone(),
            );
            bindings.bind("x".to_string(), bound_syntax);

            let template = SyntaxTemplate::List(vec![
                SyntaxTemplate::Identifier("lambda".to_string()),
                SyntaxTemplate::List(vec![SyntaxTemplate::PatternVariable("x".to_string())]),
                SyntaxTemplate::PatternVariable("x".to_string()),
            ]);

            let result = template.expand(&bindings, &context, test_span()).unwrap();

            // Should produce (lambda (foo) foo)
            assert!(matches!(result.expr, Expr::List(_)));
            if let Expr::List(elements) = &result.expr {
                assert_eq!(elements.len(), 3);
                assert!(
                    matches!(elements[0].inner, Expr::Identifier(ref name) if name == "lambda")
                );
            }
        }

        #[test]
        fn test_ellipsis_template_expansion() {
            let mut bindings = SyntaxBindings::new();
            let context = test_context();

            let ellipsis_syntaxes = vec![
                SyntaxObject::new(
                    Expr::Identifier("a".to_string()),
                    test_span(),
                    context.clone(),
                ),
                SyntaxObject::new(
                    Expr::Identifier("b".to_string()),
                    test_span(),
                    context.clone(),
                ),
                SyntaxObject::new(
                    Expr::Identifier("c".to_string()),
                    test_span(),
                    context.clone(),
                ),
            ];
            bindings.bind_ellipsis("xs".to_string(), ellipsis_syntaxes);

            let template = SyntaxTemplate::Ellipsis {
                template: Box::new(SyntaxTemplate::PatternVariable("xs".to_string())),
                separator: None,
            };

            let result = template.expand(&bindings, &context, test_span()).unwrap();

            if let Expr::List(elements) = &result.expr {
                assert_eq!(elements.len(), 3);
                assert!(matches!(elements[0].inner, Expr::Identifier(ref name) if name == "a"));
                assert!(matches!(elements[1].inner, Expr::Identifier(ref name) if name == "b"));
                assert!(matches!(elements[2].inner, Expr::Identifier(ref name) if name == "c"));
            } else {
                panic!("Expected list expression");
            }
        }

        #[test]
        fn test_syntax_case_procedures() {
            let context = test_context();
            let expr = Expr::Identifier("test".to_string());
            let syntax = SyntaxObject::new(expr.clone(), test_span(), context.clone());

            // Test syntax->datum
            let datum = syntax_procedures::syntax_to_datum(&syntax);
            assert_eq!(datum, expr);

            // Test datum->syntax
            let new_syntax = syntax_procedures::datum_to_syntax(expr, Some(&syntax), None);
            assert_eq!(new_syntax.expr, syntax.expr);
            assert_eq!(new_syntax.context.context_id, syntax.context.context_id);

            // Test identifier?
            assert!(syntax_procedures::is_identifier(&syntax));

            let non_id =
                SyntaxObject::new(Expr::Literal(Literal::Number(42.0)), test_span(), context);
            assert!(!syntax_procedures::is_identifier(&non_id));
        }
    }

    mod quasisyntax_tests {
        use super::*;

        #[test]
        fn test_quasisyntax_parser() {
            let mut parser = QuasisyntaxParser::new();

            // Test literal
            let expr = Expr::Literal(Literal::Number(42.0));
            let template = parser.parse(&expr).unwrap();
            assert!(
                matches!(template, QuasisyntaxTemplate::Literal(Literal::Number(n)) if n == 42.0)
            );

            // Test identifier
            let expr = Expr::Identifier("test".to_string());
            let template = parser.parse(&expr).unwrap();
            assert!(
                matches!(template, QuasisyntaxTemplate::Identifier(ref name) if name == "test")
            );
        }

        #[test]
        fn test_quasisyntax_expansion() {
            let context = test_context();
            let mut bindings = SyntaxBindings::new();

            let bound_syntax = SyntaxObject::new(
                Expr::Identifier("foo".to_string()),
                test_span(),
                context.clone(),
            );
            bindings.bind("x".to_string(), bound_syntax);

            let template = QuasisyntaxTemplate::List(vec![
                QuasisyntaxTemplate::Identifier("lambda".to_string()),
                QuasisyntaxTemplate::List(vec![QuasisyntaxTemplate::PatternVariable(
                    "x".to_string(),
                )]),
                QuasisyntaxTemplate::PatternVariable("x".to_string()),
            ]);

            let mut expansion_context = QuasisyntaxContext::new(bindings, context);
            let result = template
                .expand(&mut expansion_context, test_span())
                .unwrap();

            match result {
                QuasisyntaxExpansionResult::Single(syntax) => {
                    assert!(matches!(syntax.expr, Expr::List(_)));
                }
                _ => panic!("Expected single result"),
            }
        }

        #[test]
        fn test_unquote_handling() {
            let inner_template = QuasisyntaxTemplate::Identifier("test".to_string());
            let unquote_template = QuasisyntaxTemplate::Unquote(Box::new(inner_template));

            let context = test_context();
            let bindings = SyntaxBindings::new();
            let mut expansion_context = QuasisyntaxContext::new(bindings, context);

            let result = unquote_template
                .expand(&mut expansion_context, test_span())
                .unwrap();

            match result {
                QuasisyntaxExpansionResult::Single(syntax) => {
                    assert_eq!(syntax.identifier_name(), Some("test"));
                }
                _ => panic!("Expected single result"),
            }
        }

        #[test]
        fn test_unquote_splicing() {
            let list_template = QuasisyntaxTemplate::List(vec![
                QuasisyntaxTemplate::Identifier("a".to_string()),
                QuasisyntaxTemplate::Identifier("b".to_string()),
            ]);
            let splice_template = QuasisyntaxTemplate::UnquoteSplicing(Box::new(list_template));

            let context = test_context();
            let bindings = SyntaxBindings::new();
            let mut expansion_context = QuasisyntaxContext::new(bindings, context);

            let result = splice_template
                .expand(&mut expansion_context, test_span())
                .unwrap();

            match result {
                QuasisyntaxExpansionResult::Multiple(syntaxes) => {
                    assert_eq!(syntaxes.len(), 1); // The list itself
                }
                _ => panic!("Expected multiple result"),
            }
        }

        #[test]
        fn test_quasisyntax_interface() {
            let context = test_context();
            let expr = Expr::Identifier("test".to_string());
            let span = test_span();

            let syntax = quasisyntax_interface::syntax(&expr, &context, span).unwrap();

            assert_eq!(syntax.expr, expr);
            assert_eq!(syntax.span, span);
            assert_eq!(syntax.context.context_id, context.context_id);
        }
    }

    mod scope_management_tests {
        use super::*;

        #[test]
        fn test_scope_creation() {
            let mut manager = ScopeManager::new();
            let scope_id = manager.create_scope(ScopeType::Module, vec!["test".to_string()]);

            assert!(manager.get_scope(scope_id).is_some());
            let scope = manager.get_scope(scope_id).unwrap();
            assert_eq!(scope.scope_type, ScopeType::Module);
            assert_eq!(scope.module_path, vec!["test".to_string()]);
        }

        #[test]
        fn test_scope_hierarchy() {
            let mut manager = ScopeManager::new();
            let parent_id = manager
                .push_scope(ScopeType::Module, vec!["test".to_string()])
                .unwrap();
            let child_id = manager
                .push_scope(ScopeType::Function, vec!["test".to_string()])
                .unwrap();

            assert!(manager.scope_contains_or_inherits(parent_id, child_id));
            assert!(!manager.scope_contains_or_inherits(child_id, parent_id));

            let parent_scope = manager.get_scope(parent_id).unwrap();
            assert!(parent_scope.children.contains(&child_id));

            let child_scope = manager.get_scope(child_id).unwrap();
            assert_eq!(child_scope.parent, Some(parent_id));
        }

        #[test]
        fn test_binding_operations() {
            let mut manager = ScopeManager::new();
            let scope_id = manager
                .push_scope(ScopeType::Module, vec!["test".to_string()])
                .unwrap();

            let binding = ScopeBinding::new("x".to_string(), scope_id, 0, false);
            manager.add_binding(binding).unwrap();

            let marks = MarkSet::empty();
            let found = manager.lookup_binding("x", &marks);
            assert!(found.is_some());
            assert_eq!(found.unwrap().name, "x");
            assert_eq!(found.unwrap().scope_id, scope_id);
        }

        #[test]
        fn test_binding_lookup_hierarchy() {
            let mut manager = ScopeManager::new();
            let parent_id = manager
                .push_scope(ScopeType::Module, vec!["test".to_string()])
                .unwrap();

            let parent_binding = ScopeBinding::new("x".to_string(), parent_id, 0, false);
            manager.add_binding(parent_binding).unwrap();

            let _child_id = manager
                .push_scope(ScopeType::Function, vec!["test".to_string()])
                .unwrap();

            let marks = MarkSet::empty();
            let found = manager.lookup_binding("x", &marks);
            assert!(found.is_some());
            assert_eq!(found.unwrap().scope_id, parent_id);
        }

        #[test]
        fn test_binding_shadowing() {
            let mut manager = ScopeManager::new();
            let parent_id = manager
                .push_scope(ScopeType::Module, vec!["test".to_string()])
                .unwrap();

            let parent_binding = ScopeBinding::new("x".to_string(), parent_id, 0, false);
            manager.add_binding(parent_binding).unwrap();

            let child_id = manager
                .push_scope(ScopeType::Function, vec!["test".to_string()])
                .unwrap();

            let child_binding = ScopeBinding::new("x".to_string(), child_id, 0, false);
            manager.add_binding(child_binding).unwrap();

            let marks = MarkSet::empty();
            let found = manager.lookup_binding("x", &marks);
            assert!(found.is_some());
            assert_eq!(found.unwrap().scope_id, child_id); // Child shadows parent
        }

        #[test]
        fn test_phase_management() {
            let mut manager = ScopeManager::new();
            assert_eq!(manager.current_phase(), 0);

            manager.enter_phase(1);
            assert_eq!(manager.current_phase(), 1);

            manager.enter_phase(2);
            assert_eq!(manager.current_phase(), 2);

            manager.exit_phase();
            assert_eq!(manager.current_phase(), 1);

            manager.exit_phase();
            assert_eq!(manager.current_phase(), 0);
        }

        #[test]
        fn test_lambda_scope_creation() {
            let mut manager = ScopeManager::new();
            let _parent_id = manager
                .push_scope(ScopeType::Module, vec!["test".to_string()])
                .unwrap();

            let formals = Formals::Fixed(vec!["x".to_string(), "y".to_string()]);
            let _lambda_scope =
                scope_utils::create_lambda_scope(&mut manager, &formals, vec!["test".to_string()])
                    .unwrap();

            let marks = MarkSet::empty();
            assert!(manager.lookup_binding("x", &marks).is_some());
            assert!(manager.lookup_binding("y", &marks).is_some());
            assert!(manager.lookup_binding("z", &marks).is_none());
        }

        #[test]
        fn test_binding_usage_tracking() {
            let mut manager = ScopeManager::new();
            let scope_id = manager
                .push_scope(ScopeType::Module, vec!["test".to_string()])
                .unwrap();

            let binding = ScopeBinding::new("x".to_string(), scope_id, 0, false);
            manager.add_binding(binding).unwrap();

            // Initially unused
            let scope = manager.get_scope(scope_id).unwrap();
            let stats = scope.stats();
            assert_eq!(stats.unused_bindings, 1);

            // Mark as used
            let marks = MarkSet::empty();
            manager.mark_binding_used("x", &marks);

            let scope = manager.get_scope(scope_id).unwrap();
            let stats = scope.stats();
            assert_eq!(stats.used_bindings, 1);
            assert_eq!(stats.unused_bindings, 0);
        }

        #[test]
        fn test_context_scope_conversion() {
            let mut manager = ScopeManager::new();
            let scope_id = manager.create_scope(ScopeType::Module, vec!["test".to_string()]);

            let context = manager.scope_to_context(scope_id).unwrap();
            assert_eq!(context.context_id, scope_id.as_u64());
            assert_eq!(context.module_path, vec!["test".to_string()]);

            let new_scope_id = manager.context_to_scope(&context).unwrap();
            let new_scope = manager.get_scope(new_scope_id).unwrap();
            assert_eq!(new_scope.module_path, context.module_path);
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_syntax_aware_expander_creation() {
            let expander = SyntaxAwareMacroExpander::new();
            assert_eq!(expander.stats().legacy_to_syntax_conversions, 0);
        }

        #[test]
        fn test_expr_to_syntax_conversion() {
            let mut expander = SyntaxAwareMacroExpander::new();
            let context = test_context();
            expander.set_context(context);

            let expr = Spanned::new(Expr::Identifier("test".to_string()), test_span());

            let syntax = expander.expr_to_syntax(expr, None).unwrap();
            assert_eq!(syntax.identifier_name(), Some("test"));
            assert_eq!(expander.stats().legacy_to_syntax_conversions, 1);
        }

        #[test]
        fn test_syntax_to_expr_conversion() {
            let mut expander = SyntaxAwareMacroExpander::new();
            let context = test_context();

            let syntax =
                SyntaxObject::new(Expr::Identifier("test".to_string()), test_span(), context);

            let expr = expander.syntax_to_expr(&syntax).unwrap();
            assert_eq!(expr.inner, Expr::Identifier("test".to_string()));
            assert_eq!(expander.stats().syntax_to_legacy_conversions, 1);
        }

        #[test]
        fn test_macro_expansion_context() {
            let mut expander = SyntaxAwareMacroExpander::new();

            let context = expander
                .enter_macro_expansion(vec!["test".to_string()])
                .unwrap();
            assert_eq!(context.module_path, vec!["test".to_string()]);

            expander.exit_macro_expansion();
            // Context should be reset appropriately
        }

        #[test]
        fn test_hygiene_application() {
            let mut expander = SyntaxAwareMacroExpander::new();
            let context = test_context();
            expander.set_context(context.clone());

            // Enter a macro context to get marks
            let _macro_context = expander
                .enter_macro_expansion(vec!["test".to_string()])
                .unwrap();

            let expr = Spanned::new(Expr::Identifier("test".to_string()), test_span());

            let syntax = expander.expr_to_syntax(expr, None).unwrap();
            // Should have hygiene applied due to macro context
            assert!(!syntax.marks.is_empty());
        }

        #[test]
        fn test_legacy_pattern_conversion() {
            use crate::macro_system::pattern::Pattern;

            let legacy_pattern = Pattern::Variable("x".to_string());
            let syntax_pattern = legacy_bridge::pattern_to_syntax_pattern(&legacy_pattern).unwrap();

            assert!(matches!(syntax_pattern, SyntaxPattern::PatternVariable(name) if name == "x"));
        }

        #[test]
        fn test_integration_stats_tracking() {
            let mut expander = SyntaxAwareMacroExpander::new();
            assert_eq!(expander.stats().legacy_to_syntax_conversions, 0);

            let context = test_context();
            expander.set_context(context);

            let expr = Spanned::new(Expr::Identifier("test".to_string()), test_span());
            let _syntax = expander.expr_to_syntax(expr, None).unwrap();

            assert_eq!(expander.stats().legacy_to_syntax_conversions, 1);

            expander.reset_stats();
            assert_eq!(expander.stats().legacy_to_syntax_conversions, 0);
        }

        #[test]
        fn test_complete_macro_expansion_flow() {
            let mut expander = SyntaxAwareMacroExpander::new();
            let context = test_context();
            expander.set_context(context.clone());

            // Simulate macro definition and expansion
            let _macro_context = expander
                .enter_macro_expansion(vec!["test".to_string()])
                .unwrap();

            // Create input syntax
            let input_expr = Spanned::new(
                Expr::List(vec![
                    Spanned::new(Expr::Identifier("my-macro".to_string()), test_span()),
                    Spanned::new(Expr::Identifier("arg".to_string()), test_span()),
                ]),
                test_span(),
            );

            let input_syntax = expander.expr_to_syntax(input_expr, None).unwrap();

            // Convert back to verify round-trip
            let output_expr = expander.syntax_to_expr(&input_syntax).unwrap();

            assert!(matches!(output_expr.inner, Expr::List(_)));

            expander.exit_macro_expansion();
        }
    }

    #[cfg(feature = "property-testing")]
    mod property_based_tests {
        use super::*;

        #[test]
        fn prop_syntax_object_roundtrip() {
            let test_names = vec!["x", "foo", "test-name", "my-var"];

            for name in test_names {
                let context = test_context();
                let expr = Expr::Identifier(name.to_string());
                let syntax = SyntaxObject::new(expr.clone(), test_span(), context);

                let recovered_expr = syntax_procedures::syntax_to_datum(&syntax);
                assert_eq!(recovered_expr, expr);
            }
        }

        #[test]
        fn prop_mark_set_associativity() {
            let test_cases = vec![
                (vec![1, 2], vec![3, 4], vec![5, 6]),
                (vec![1], vec![2], vec![3]),
                (vec![], vec![1], vec![]),
            ];

            for (marks1, marks2, marks3) in test_cases {
                let set1 =
                    MarkSet::from(marks1.into_iter().collect::<std::collections::HashSet<_>>());
                let set2 =
                    MarkSet::from(marks2.into_iter().collect::<std::collections::HashSet<_>>());
                let set3 =
                    MarkSet::from(marks3.into_iter().collect::<std::collections::HashSet<_>>());

                let left = set1.union(&set2).union(&set3);
                let right = set1.union(&set2.union(&set3));

                assert_eq!(left, right);
            }
        }

        #[test]
        fn prop_scope_hierarchy_transitivity() {
            for depth in 1..=5 {
                let mut manager = ScopeManager::new();
                let mut scopes = Vec::new();

                // Create hierarchy
                for i in 0..depth {
                    let scope_id = manager
                        .push_scope(ScopeType::Block, vec![format!("level{}", i)])
                        .unwrap();
                    scopes.push(scope_id);
                }

                // Check transitivity: if A contains B and B contains C, then A contains C
                for i in 0..scopes.len() {
                    for j in i + 1..scopes.len() {
                        assert!(manager.scope_contains_or_inherits(scopes[i], scopes[j]));
                    }
                }
            }
        }
    }
}
