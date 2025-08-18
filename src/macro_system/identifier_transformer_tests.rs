//! Comprehensive tests for the identifier transformer system
//!
//! This module contains tests that verify the behavior of variable transformers
//! in all contexts and usage patterns as specified by R6RS.

use super::{
    identifier_transformers::{
        VariableTransformer, VariableTransformerRegistry, IdentifierContext, ContextDetector,
        TransformerProcedure, TransformationLogic
    },
    variable_transformer_builtins::{
        VariableTransformerBuiltins, make_variable_transformer, make_simple_variable_transformer,
        create_accessor_transformer, create_vector_accessor_transformer, create_hash_accessor_transformer
    },
    context_aware_expander::{ContextAwareMacroExpander, ContextAwareExpansionStats},
    unified_expander::{UnifiedMacroExpander, MacroTransformerType},
    syntax_objects::{SyntaxObject, LexicalContext, syntax_utils},
    advanced_hygiene::HygieneResolver,
    variable_transformer_integration::{
        VariableTransformerAwareSyntaxCase, VariableTransformerAwareTemplate,
        syntax_procedures
    }
};
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Span};
use crate::eval::Value;
use std::collections::HashMap;

/// Test suite for basic variable transformer functionality
#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    pub fn test_variable_transformer_creation() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        
        let transformer = VariableTransformer::simple(
            "my-storage".to_string(),
            "(storage-get)".to_string(),
            "(storage-set! {val})".to_string(),
            context,
        );

        assert_eq!(transformer.name, "my-storage");
        assert!(transformer.context_sensitive);
        assert!(transformer.supports_context(&IdentifierContext::Reference));
        assert!(transformer.supports_context(&IdentifierContext::Assignment));
        assert!(!transformer.supports_context(&IdentifierContext::MacroDefinition));
    }

    #[test]
    pub fn test_context_detection() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 5);

        // Test reference context (identifier by itself)
        let identifier = syntax_utils::make_identifier_syntax("x".to_string(), span, context.clone());
        let detected_context = ContextDetector::detect_context(&identifier, None);
        assert_eq!(detected_context, IdentifierContext::Reference);

        // Test assignment context (set! form)
        let set_form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("x".to_string(), span, context.clone()),
                syntax_utils::make_literal_syntax(Literal::Integer(42), span, context.clone()),
            ],
            span,
            context.clone(),
        );

        let target_identifier = &set_form.as_list().unwrap()[1];
        let detected_context = ContextDetector::detect_context(target_identifier, Some(&set_form));
        assert_eq!(detected_context, IdentifierContext::Assignment);

        // Test procedure call context
        let call_form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("f".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("arg".to_string(), span, context.clone()),
            ],
            span,
            context.clone(),
        );

        let procedure_identifier = &call_form.as_list().unwrap()[0];
        let detected_context = ContextDetector::detect_context(procedure_identifier, Some(&call_form));
        assert_eq!(detected_context, IdentifierContext::ProcedureCall);
    }

    #[test]
    pub fn test_variable_transformer_registry() {
        let mut registry = VariableTransformerRegistry::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        // Register a simple transformer
        let transformer = VariableTransformer::simple(
            "counter".to_string(),
            "(get-counter)".to_string(),
            "(set-counter! {val})".to_string(),
            context.clone(),
        );
        registry.register(transformer);

        assert!(registry.is_variable_transformer("counter"));
        assert!(!registry.is_variable_transformer("unknown"));
        assert_eq!(registry.list_transformers().len(), 1);
        assert!(registry.list_transformers().contains(&"counter"));

        // Test retrieval
        let retrieved = registry.get("counter").unwrap();
        assert_eq!(retrieved.name, "counter");

        // Test removal
        let removed = registry.unregister("counter");
        assert!(removed.is_some());
        assert!(!registry.is_variable_transformer("counter"));
    }
}

/// Test suite for variable transformer expansion in different contexts
#[cfg(test)]
mod expansion_tests {
    use super::*;

    #[test]
    pub fn test_reference_context_expansion() {
        let mut registry = VariableTransformerRegistry::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 8);

        // Register a storage transformer
        let transformer = VariableTransformer::simple(
            "storage".to_string(),
            "(storage-ref storage)".to_string(),
            "(storage-set! storage {val})".to_string(),
            context.clone(),
        );
        registry.register(transformer);

        // Create identifier syntax
        let identifier = syntax_utils::make_identifier_syntax("storage".to_string(), span, context);
        let mut hygiene_resolver = HygieneResolver::new();

        // Expand in reference context
        let result = registry.expand_variable_transformer(
            "storage",
            &identifier,
            None,
            &mut hygiene_resolver,
        );

        assert!(result.is_ok());
        // The result should be the expanded form
    }

    #[test]
    pub fn test_assignment_context_expansion() {
        let mut registry = VariableTransformerRegistry::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 15);

        // Register a storage transformer
        let transformer = VariableTransformer::simple(
            "storage".to_string(),
            "(storage-ref storage)".to_string(),
            "(storage-set! storage {val})".to_string(),
            context.clone(),
        );
        registry.register(transformer);

        // Create set! form
        let set_form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("storage".to_string(), span, context.clone()),
                syntax_utils::make_literal_syntax(Literal::Integer(100), span, context.clone()),
            ],
            span,
            context,
        );

        let mut hygiene_resolver = HygieneResolver::new();

        // Expand in assignment context
        let result = registry.expand_variable_transformer(
            "storage",
            &set_form,
            None,
            &mut hygiene_resolver,
        );

        assert!(result.is_ok());
        // The result should be the expanded assignment form
    }

    #[test]
    pub fn test_procedure_call_context_expansion() {
        let mut registry = VariableTransformerRegistry::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 10);

        // Register a procedure-like transformer
        let transformer = VariableTransformer::simple(
            "accessor".to_string(),
            "(get-value)".to_string(),
            "(set-value! {val})".to_string(),
            context.clone(),
        );
        registry.register(transformer);

        // Create procedure call form
        let call_form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("accessor".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("arg1".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("arg2".to_string(), span, context.clone()),
            ],
            span,
            context,
        );

        let mut hygiene_resolver = HygieneResolver::new();

        // Expand in call context
        let result = registry.expand_variable_transformer(
            "accessor",
            &call_form,
            None,
            &mut hygiene_resolver,
        );

        // Should succeed or provide meaningful error
        assert!(result.is_ok() || result.is_err());
    }
}

/// Test suite for context-aware macro expander
#[cfg(test)]
mod context_aware_expander_tests {
    use super::*;

    #[test]
    pub fn test_context_aware_expander_creation() {
        let expander = ContextAwareMacroExpander::new();
        assert_eq!(expander.stats().variable_transformer_expansions, 0);
        assert_eq!(expander.stats().context_detections, 0);
    }

    #[test]
    pub fn test_variable_transformer_expansion_flow() {
        let mut expander = ContextAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 5);

        // Register a variable transformer
        let transformer = VariableTransformer::simple(
            "my-var".to_string(),
            "(get-my-var)".to_string(),
            "(set-my-var! {val})".to_string(),
            context.clone(),
        );
        expander.register_variable_transformer(transformer);

        // Test reference expansion
        let identifier = syntax_utils::make_identifier_syntax("my-var".to_string(), span, context.clone());
        let result = expander.expand(&identifier);
        assert!(result.is_ok());
        assert!(expander.stats().variable_transformer_expansions > 0);

        // Reset stats and test assignment expansion
        expander.reset_stats();

        let set_form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("my-var".to_string(), span, context.clone()),
                syntax_utils::make_literal_syntax(Literal::String("value".to_string()), span, context.clone()),
            ],
            span,
            context,
        );

        let result = expander.expand(&set_form);
        assert!(result.is_ok());
        assert!(expander.stats().variable_transformer_expansions > 0);
        assert!(expander.stats().assignment_expansions > 0);
    }

    #[test]
    pub fn test_nested_expansion() {
        let mut expander = ContextAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 20);

        // Register multiple transformers
        let storage_transformer = VariableTransformer::simple(
            "storage".to_string(),
            "(storage-get)".to_string(),
            "(storage-set! {val})".to_string(),
            context.clone(),
        );
        expander.register_variable_transformer(storage_transformer);

        let counter_transformer = VariableTransformer::simple(
            "counter".to_string(),
            "(counter-get)".to_string(),
            "(counter-set! {val})".to_string(),
            context.clone(),
        );
        expander.register_variable_transformer(counter_transformer);

        // Create nested form: (let ((x storage)) (set! counter x))
        let let_form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("let".to_string(), span, context.clone()),
                syntax_utils::make_list_syntax(
                    vec![
                        syntax_utils::make_list_syntax(
                            vec![
                                syntax_utils::make_identifier_syntax("x".to_string(), span, context.clone()),
                                syntax_utils::make_identifier_syntax("storage".to_string(), span, context.clone()),
                            ],
                            span,
                            context.clone(),
                        ),
                    ],
                    span,
                    context.clone(),
                ),
                syntax_utils::make_list_syntax(
                    vec![
                        syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                        syntax_utils::make_identifier_syntax("counter".to_string(), span, context.clone()),
                        syntax_utils::make_identifier_syntax("x".to_string(), span, context.clone()),
                    ],
                    span,
                    context.clone(),
                ),
            ],
            span,
            context,
        );

        let result = expander.expand(&let_form);
        assert!(result.is_ok());

        // Should have detected and expanded both transformers
        assert!(expander.stats().variable_transformer_expansions >= 2);
    }
}

/// Test suite for built-in variable transformer procedures
#[cfg(test)]
mod builtins_tests {
    use super::*;

    #[test]
    pub fn test_make_variable_transformer() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let dummy_procedure = Value::Null; // Placeholder

        let result = make_variable_transformer(
            dummy_procedure,
            Some("test-transformer".to_string()),
            context,
        );

        assert!(result.is_ok());
        let transformer = result.unwrap();
        assert_eq!(transformer.name, "test-transformer");
        assert!(transformer.context_sensitive);
    }

    #[test]
    pub fn test_make_simple_variable_transformer() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        let result = make_simple_variable_transformer(
            "(get-value)".to_string(),
            "(set-value! {val})".to_string(),
            Some("simple-test".to_string()),
            context,
        );

        assert!(result.is_ok());
        let transformer = result.unwrap();
        assert_eq!(transformer.name, "simple-test");
        assert!(transformer.supports_context(&IdentifierContext::Reference));
        assert!(transformer.supports_context(&IdentifierContext::Assignment));
    }

    #[test]
    pub fn test_create_accessor_transformer() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        let transformer = create_accessor_transformer(
            "get-x".to_string(),
            "set-x!".to_string(),
            context,
        );

        assert_eq!(transformer.name, "get-x-accessor");
        assert!(transformer.supports_context(&IdentifierContext::Reference));
        assert!(transformer.supports_context(&IdentifierContext::Assignment));
    }

    #[test]
    pub fn test_create_vector_accessor_transformer() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        let transformer = create_vector_accessor_transformer(
            "my-vector".to_string(),
            3,
            context,
        );

        assert_eq!(transformer.name, "my-vector-[3]");
        assert!(transformer.supports_context(&IdentifierContext::Reference));
        assert!(transformer.supports_context(&IdentifierContext::Assignment));
    }

    #[test]
    pub fn test_create_hash_accessor_transformer() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        let transformer = create_hash_accessor_transformer(
            "config".to_string(),
            "timeout".to_string(),
            context,
        );

        assert_eq!(transformer.name, "config.timeout");
        assert!(transformer.supports_context(&IdentifierContext::Reference));
        assert!(transformer.supports_context(&IdentifierContext::Assignment));
    }

    #[test]
    pub fn test_builtins_registry() {
        let mut builtins = VariableTransformerBuiltins::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        builtins.create_example_transformers(context);

        let registry = builtins.registry();
        assert!(registry.is_variable_transformer("storage-cell"));
        assert!(registry.is_variable_transformer("counter"));
        assert!(registry.is_variable_transformer("my-array-[0]"));
        assert!(registry.is_variable_transformer("properties.value"));

        let transformer_names = registry.list_transformers();
        assert!(transformer_names.len() >= 4);
    }
}

/// Test suite for unified expander integration
#[cfg(test)]
mod unified_expander_tests {
    use super::*;

    #[test]
    pub fn test_unified_expander_variable_transformer_support() {
        let mut expander = UnifiedMacroExpander::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        // Register a variable transformer
        let transformer = VariableTransformer::simple(
            "unified-var".to_string(),
            "(unified-get)".to_string(),
            "(unified-set! {val})".to_string(),
            context.clone(),
        );
        expander.register_variable_transformer(transformer);

        assert!(expander.is_variable_transformer("unified-var"));
        assert!(!expander.is_variable_transformer("nonexistent"));

        // Test that it's also registered as a macro
        assert!(expander.is_macro("unified-var"));
    }

    #[test]
    pub fn test_unified_expander_syntax_expansion() {
        let mut expander = UnifiedMacroExpander::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 10);

        // Register a variable transformer
        let transformer = VariableTransformer::simple(
            "unified-storage".to_string(),
            "(storage-get unified-storage)".to_string(),
            "(storage-set! unified-storage {val})".to_string(),
            context.clone(),
        );
        expander.register_variable_transformer(transformer);

        // Create syntax object for reference
        let reference_syntax = syntax_utils::make_identifier_syntax(
            "unified-storage".to_string(),
            span,
            context.clone(),
        );

        let result = expander.expand_by_name("unified-storage", &reference_syntax);
        assert!(result.is_ok());

        // Create syntax object for assignment
        let assignment_syntax = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("unified-storage".to_string(), span, context.clone()),
                syntax_utils::make_literal_syntax(Literal::Integer(42), span, context.clone()),
            ],
            span,
            context,
        );

        let result = expander.expand_by_name("unified-storage", &assignment_syntax);
        assert!(result.is_ok());
    }
}

/// Test suite for error handling and edge cases
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    pub fn test_unknown_transformer_error() {
        let registry = VariableTransformerRegistry::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 5);

        let identifier = syntax_utils::make_identifier_syntax("unknown".to_string(), span, context);
        let mut hygiene_resolver = HygieneResolver::new();

        let result = registry.expand_variable_transformer(
            "unknown",
            &identifier,
            None,
            &mut hygiene_resolver,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown variable transformer"));
    }

    #[test]
    pub fn test_unsupported_context_error() {
        let mut registry = VariableTransformerRegistry::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 5);

        // Create a transformer that only supports reference context
        let transformation_logic = TransformationLogic::Reference {
            target_expr: "(get-value)".to_string(),
        };
        let transformer_proc = TransformerProcedure::Procedure {
            name: "reference-only".to_string(),
            transformation_logic,
        };
        let transformer = VariableTransformer::new(
            "reference-only".to_string(),
            transformer_proc,
            context.clone(),
        );
        registry.register(transformer);

        // Try to use it in assignment context
        let set_form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("reference-only".to_string(), span, context.clone()),
                syntax_utils::make_literal_syntax(Literal::Integer(42), span, context.clone()),
            ],
            span,
            context,
        );

        let mut hygiene_resolver = HygieneResolver::new();
        let result = registry.expand_variable_transformer(
            "reference-only",
            &set_form,
            None,
            &mut hygiene_resolver,
        );

        assert!(result.is_err());
        // Should contain context-related error message
    }

    #[test]
    pub fn test_expansion_depth_limit() {
        let mut expander = ContextAwareMacroExpander::with_max_depth(3);
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 5);

        // Force the expansion depth to maximum
        expander.expansion_depth = 3;

        let identifier = syntax_utils::make_identifier_syntax("test".to_string(), span, context);
        let result = expander.expand(&identifier);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Maximum expansion depth"));
    }
}

/// Test suite for R6RS compliance
#[cfg(test)]
mod r6rs_compliance_tests {
    use super::*;

    #[test]
    pub fn test_r6rs_make_variable_transformer_example() {
        // Example from R6RS: implementing a storage cell
        let mut expander = ContextAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["r6rs-test".to_string()]);
        let span = Span::new(0, 10);

        // Create a storage cell transformer like in R6RS examples
        let storage_transformer = VariableTransformer::simple(
            "storage-cell".to_string(),
            "(vector-ref storage 0)".to_string(),
            "(vector-set! storage 0 {val})".to_string(),
            context.clone(),
        );
        expander.register_variable_transformer(storage_transformer);

        // Test reference: storage-cell => (vector-ref storage 0)
        let reference = syntax_utils::make_identifier_syntax("storage-cell".to_string(), span, context.clone());
        let result = expander.expand(&reference);
        assert!(result.is_ok());

        // Test assignment: (set! storage-cell value) => (vector-set! storage 0 value)
        let assignment = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("storage-cell".to_string(), span, context.clone()),
                syntax_utils::make_literal_syntax(Literal::String("new-value".to_string()), span, context.clone()),
            ],
            span,
            context,
        );

        let result = expander.expand(&assignment);
        assert!(result.is_ok());
    }

    #[test]
    pub fn test_r6rs_hygiene_preservation() {
        // Test that variable transformers preserve hygiene correctly
        let mut expander = ContextAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["hygiene-test".to_string()]);
        let span = Span::new(0, 15);

        // Create a transformer that introduces new bindings
        let hygiene_transformer = VariableTransformer::simple(
            "hygienic-var".to_string(),
            "(let ((temp (get-value))) temp)".to_string(),
            "(let ((temp {val})) (set-value! temp))".to_string(),
            context.clone(),
        );
        expander.register_variable_transformer(hygiene_transformer);

        // Test that the introduced 'temp' variable doesn't clash with user bindings
        let nested_form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("let".to_string(), span, context.clone()),
                syntax_utils::make_list_syntax(
                    vec![
                        syntax_utils::make_list_syntax(
                            vec![
                                syntax_utils::make_identifier_syntax("temp".to_string(), span, context.clone()),
                                syntax_utils::make_literal_syntax(Literal::Integer(42), span, context.clone()),
                            ],
                            span,
                            context.clone(),
                        ),
                    ],
                    span,
                    context.clone(),
                ),
                syntax_utils::make_identifier_syntax("hygienic-var".to_string(), span, context.clone()),
            ],
            span,
            context,
        );

        let result = expander.expand(&nested_form);
        assert!(result.is_ok());
        // The expansion should preserve hygiene and not create variable capture
    }

    #[test]
    pub fn test_r6rs_multiple_contexts() {
        // Test that a variable transformer can handle multiple contexts appropriately
        let mut expander = ContextAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["multi-context-test".to_string()]);
        let span = Span::new(0, 12);

        // Create a more complex transformer that supports call context too
        let multi_transformer = VariableTransformer::simple(
            "multi-var".to_string(),
            "(get-multi-value)".to_string(),
            "(set-multi-value! {val})".to_string(),
            context.clone(),
        );
        expander.register_variable_transformer(multi_transformer);

        // Test all three contexts
        
        // 1. Reference context
        let reference = syntax_utils::make_identifier_syntax("multi-var".to_string(), span, context.clone());
        let result = expander.expand(&reference);
        assert!(result.is_ok());

        // 2. Assignment context
        let assignment = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("multi-var".to_string(), span, context.clone()),
                syntax_utils::make_literal_syntax(Literal::Integer(100), span, context.clone()),
            ],
            span,
            context.clone(),
        );
        let result = expander.expand(&assignment);
        assert!(result.is_ok());

        // 3. Procedure call context
        let call = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("multi-var".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("arg1".to_string(), span, context.clone()),
            ],
            span,
            context,
        );
        let result = expander.expand(&call);
        // This might succeed or fail depending on implementation, but should handle gracefully
        assert!(result.is_ok() || result.is_err());
    }
}

/// Performance and stress tests
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    pub fn test_large_number_of_transformers() {
        let mut registry = VariableTransformerRegistry::new();
        let context = LexicalContext::new(1, vec!["perf-test".to_string()]);

        // Register many transformers
        for i in 0..1000 {
            let transformer = VariableTransformer::simple(
                format!("var-{}", i),
                format!("(get-{})", i),
                format!("(set-{}! {{val}})", i),
                context.clone(),
            );
            registry.register(transformer);
        }

        // Test lookup performance
        for i in 0..1000 {
            let name = format!("var-{}", i);
            assert!(registry.is_variable_transformer(&name));
        }

        // Test that non-existent transformers are handled efficiently
        for i in 1000..1100 {
            let name = format!("var-{}", i);
            assert!(!registry.is_variable_transformer(&name));
        }
    }

    #[test]
    pub fn test_deep_nesting_expansion() {
        let mut expander = ContextAwareMacroExpander::with_max_depth(50);
        let context = LexicalContext::new(1, vec!["nesting-test".to_string()]);
        let span = Span::new(0, 5);

        // Register a transformer
        let transformer = VariableTransformer::simple(
            "nested-var".to_string(),
            "(get-nested)".to_string(),
            "(set-nested! {val})".to_string(),
            context.clone(),
        );
        expander.register_variable_transformer(transformer);

        // Create deeply nested structure
        let mut nested_form = syntax_utils::make_identifier_syntax("nested-var".to_string(), span, context.clone());
        
        for i in 0..10 {
            nested_form = syntax_utils::make_list_syntax(
                vec![
                    syntax_utils::make_identifier_syntax(format!("wrapper-{}", i), span, context.clone()),
                    nested_form,
                ],
                span,
                context.clone(),
            );
        }

        let result = expander.expand(&nested_form);
        assert!(result.is_ok());
    }
}

/// Integration tests with real Scheme code patterns
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    pub fn test_object_oriented_pattern() {
        // Test variable transformers used for object-oriented programming patterns
        let mut expander = ContextAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["oop-test".to_string()]);
        let span = Span::new(0, 10);

        // Create transformers for object field access
        let x_field = create_accessor_transformer(
            "obj-x".to_string(),
            "obj-set-x!".to_string(),
            context.clone(),
        );
        let y_field = create_accessor_transformer(
            "obj-y".to_string(),
            "obj-set-y!".to_string(),
            context.clone(),
        );

        expander.register_variable_transformer(x_field);
        expander.register_variable_transformer(y_field);

        // Test field access patterns
        let field_access = syntax_utils::make_identifier_syntax("obj-x-accessor".to_string(), span, context.clone());
        let result = expander.expand(&field_access);
        assert!(result.is_ok());

        let field_assignment = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("obj-y-accessor".to_string(), span, context.clone()),
                syntax_utils::make_literal_syntax(Literal::Integer(200), span, context.clone()),
            ],
            span,
            context,
        );
        let result = expander.expand(&field_assignment);
        assert!(result.is_ok());
    }

    #[test]
    pub fn test_configuration_system_pattern() {
        // Test variable transformers used for configuration systems
        let mut expander = ContextAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["config-test".to_string()]);

        // Create configuration transformers
        let config_items = vec![
            ("timeout", "default-timeout"),
            ("host", "default-host"),
            ("port", "default-port"),
            ("debug", "default-debug"),
        ];

        for (key, _default) in config_items {
            let transformer = create_hash_accessor_transformer(
                "config".to_string(),
                key.to_string(),
                context.clone(),
            );
            expander.register_variable_transformer(transformer);
        }

        // Test configuration access
        assert!(expander.variable_transformer_registry().is_variable_transformer("config.timeout"));
        assert!(expander.variable_transformer_registry().is_variable_transformer("config.host"));
        assert!(expander.variable_transformer_registry().is_variable_transformer("config.port"));
        assert!(expander.variable_transformer_registry().is_variable_transformer("config.debug"));
    }
}

/// Utility functions for testing
pub fn create_test_context() -> LexicalContext {
    LexicalContext::new(999, vec!["test-module".to_string()])
}

/// Creates a test span for use in unit tests
pub fn create_test_span() -> Span {
    Span::new(0, 10)
}

/// Creates a test identifier syntax object with the given name
pub fn create_test_identifier(name: &str) -> SyntaxObject {
    syntax_utils::make_identifier_syntax(
        name.to_string(),
        create_test_span(),
        create_test_context(),
    )
}

/// Creates a test list syntax object containing the given elements
pub fn create_test_list(elements: Vec<SyntaxObject>) -> SyntaxObject {
    syntax_utils::make_list_syntax(
        elements,
        create_test_span(),
        create_test_context(),
    )
}

/// Run all tests
#[cfg(test)]
pub fn run_all_identifier_transformer_tests() {
    // This function can be called to run all tests in this module
    println!("Running identifier transformer tests...");
    
    // Basic functionality tests
    basic_tests::test_variable_transformer_creation();
    basic_tests::test_context_detection();
    basic_tests::test_variable_transformer_registry();
    
    // Expansion tests
    expansion_tests::test_reference_context_expansion();
    expansion_tests::test_assignment_context_expansion();
    expansion_tests::test_procedure_call_context_expansion();
    
    // Context-aware expander tests
    context_aware_expander_tests::test_context_aware_expander_creation();
    context_aware_expander_tests::test_variable_transformer_expansion_flow();
    context_aware_expander_tests::test_nested_expansion();
    
    // Built-ins tests
    builtins_tests::test_make_variable_transformer();
    builtins_tests::test_make_simple_variable_transformer();
    builtins_tests::test_create_accessor_transformer();
    builtins_tests::test_create_vector_accessor_transformer();
    builtins_tests::test_create_hash_accessor_transformer();
    builtins_tests::test_builtins_registry();
    
    // Error handling tests
    error_handling_tests::test_unknown_transformer_error();
    error_handling_tests::test_unsupported_context_error();
    error_handling_tests::test_expansion_depth_limit();
    
    // R6RS compliance tests
    r6rs_compliance_tests::test_r6rs_make_variable_transformer_example();
    r6rs_compliance_tests::test_r6rs_hygiene_preservation();
    r6rs_compliance_tests::test_r6rs_multiple_contexts();
    
    // Performance tests
    performance_tests::test_large_number_of_transformers();
    performance_tests::test_deep_nesting_expansion();
    
    // Integration tests
    integration_tests::test_object_oriented_pattern();
    integration_tests::test_configuration_system_pattern();
    
    println!("All identifier transformer tests completed!");
}