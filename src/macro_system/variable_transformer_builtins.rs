//! Built-in procedures for variable transformers
//!
//! This module implements the core procedures for creating and working with
//! variable transformers, including `make-variable-transformer` and related
//! utilities as specified in R6RS Scheme.

use super::{
    advanced_hygiene::HygieneResolver,
    identifier_transformers::{
        IdentifierContext, TransformationLogic, TransformerProcedure, VariableTransformer,
        VariableTransformerRegistry,
    },
    syntax_objects::{LexicalContext, SyntaxObject, syntax_utils},
};
use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span};
use crate::eval::{Environment, Value};
use std::collections::HashMap;
use std::rc::Rc;

/// Creates a variable transformer from a procedure
///
/// This implements the R6RS `make-variable-transformer` procedure:
/// ```scheme
/// (make-variable-transformer proc)
/// ```
/// where `proc` is a procedure that takes a syntax object and returns a syntax object.
pub fn make_variable_transformer(
    procedure: Value,
    name: Option<String>,
    definition_context: LexicalContext,
) -> Result<VariableTransformer> {
    // In a real implementation, we would handle Scheme procedure values
    // For now, we'll create a simplified transformer based on the procedure type

    let transformer_name = name.unwrap_or_else(|| "anonymous-variable-transformer".to_string());

    // Create a procedure-based transformer
    // In practice, this would wrap the actual Scheme procedure
    let transformation_logic = TransformationLogic::Complex {
        patterns: vec![
            // Handle reference context: identifier by itself
            (
                IdentifierContext::Reference,
                "id".to_string(),
                "{reference-expansion}".to_string(),
            ),
            // Handle assignment context: (set! identifier value)
            (
                IdentifierContext::Assignment,
                "(set! id val)".to_string(),
                "{assignment-expansion}".to_string(),
            ),
            // Handle procedure call context: (identifier arg ...)
            (
                IdentifierContext::ProcedureCall,
                "(proc . args)".to_string(),
                "{call-expansion}".to_string(),
            ),
        ],
    };

    let transformer_proc = TransformerProcedure::Procedure {
        name: transformer_name.clone(),
        transformation_logic,
    };

    Ok(VariableTransformer::new(
        transformer_name,
        transformer_proc,
        definition_context,
    ))
}

/// Creates a variable transformer with explicit reference and assignment handlers
///
/// This is a convenience function for creating common variable transformers:
/// ```scheme
/// (make-simple-variable-transformer reference-template assignment-template)
/// ```
pub fn make_simple_variable_transformer(
    reference_template: String,
    assignment_template: String,
    name: Option<String>,
    definition_context: LexicalContext,
) -> Result<VariableTransformer> {
    let transformer_name = name.unwrap_or_else(|| "simple-variable-transformer".to_string());

    Ok(VariableTransformer::simple(
        transformer_name,
        reference_template,
        assignment_template,
        definition_context,
    ))
}

/// Predicate to check if a value is a variable transformer
pub fn variable_transformer_p(value: &Value) -> bool {
    // In a real implementation, this would check if the value is a variable transformer
    // For now, we'll always return false since we're working with simplified types
    false
}

/// Applies a variable transformer to a syntax object in a specific context
pub fn apply_variable_transformer(
    transformer: &VariableTransformer,
    syntax: &SyntaxObject,
    context: IdentifierContext,
    hygiene_env: &mut HygieneResolver,
) -> Result<SyntaxObject> {
    transformer.expand(syntax, context, hygiene_env)
}

/// Built-in variable transformer for demonstration: a simple getter/setter pair
pub fn create_accessor_transformer(
    getter_name: String,
    setter_name: String,
    definition_context: LexicalContext,
) -> VariableTransformer {
    let reference_template = format!("({getter_name})");
    let assignment_template = format!("({setter_name} {{val}})");

    VariableTransformer::simple(
        format!("{getter_name}-accessor"),
        reference_template,
        assignment_template,
        definition_context,
    )
}

/// Built-in variable transformer for vector-like access
pub fn create_vector_accessor_transformer(
    vector_name: String,
    index: usize,
    definition_context: LexicalContext,
) -> VariableTransformer {
    let reference_template = format!("(vector-ref {vector_name} {index})");
    let assignment_template = format!("(vector-set! {vector_name} {index} {{val}})");

    VariableTransformer::simple(
        format!("{vector_name}-[{index}]"),
        reference_template,
        assignment_template,
        definition_context,
    )
}

/// Built-in variable transformer for hash table access
pub fn create_hash_accessor_transformer(
    table_name: String,
    key: String,
    definition_context: LexicalContext,
) -> VariableTransformer {
    let reference_template = format!("(hash-ref {table_name} {key})");
    let assignment_template = format!("(hash-set! {table_name} {key} {{val}})");

    VariableTransformer::simple(
        format!("{table_name}.{key}"),
        reference_template,
        assignment_template,
        definition_context,
    )
}

/// Interface for registering built-in variable transformers with the runtime
pub struct VariableTransformerBuiltins {
    registry: VariableTransformerRegistry,
}

impl VariableTransformerBuiltins {
    /// Creates a new built-ins registry
    pub fn new() -> Self {
        Self {
            registry: VariableTransformerRegistry::new(),
        }
    }

    /// Registers all built-in variable transformer procedures
    pub fn register_builtins(&mut self, env: &mut Environment) -> Result<()> {
        // In a real implementation, these would be registered as Scheme procedures

        // Register make-variable-transformer
        self.register_procedure(
            env,
            "make-variable-transformer",
            "Creates a variable transformer from a procedure",
        )?;

        // Register variable-transformer?
        self.register_procedure(
            env,
            "variable-transformer?",
            "Predicate to test if a value is a variable transformer",
        )?;

        // Register helper procedures
        self.register_procedure(
            env,
            "make-simple-variable-transformer",
            "Creates a simple variable transformer with reference and assignment templates",
        )?;

        self.register_procedure(
            env,
            "make-accessor-transformer",
            "Creates a variable transformer for getter/setter pairs",
        )?;

        self.register_procedure(
            env,
            "make-vector-accessor-transformer",
            "Creates a variable transformer for vector element access",
        )?;

        self.register_procedure(
            env,
            "make-hash-accessor-transformer",
            "Creates a variable transformer for hash table access",
        )?;

        Ok(())
    }

    /// Helper to register a procedure (simplified for this implementation)
    fn register_procedure(
        &mut self,
        env: &mut Environment,
        name: &str,
        description: &str,
    ) -> Result<()> {
        // In a real implementation, this would create and register actual Scheme procedures
        // For now, we'll just note that they should be registered
        println!("Would register procedure: {name} - {description}");
        Ok(())
    }

    /// Gets the transformer registry
    pub fn registry(&self) -> &VariableTransformerRegistry {
        &self.registry
    }

    /// Gets a mutable reference to the transformer registry
    pub fn registry_mut(&mut self) -> &mut VariableTransformerRegistry {
        &mut self.registry
    }

    /// Creates a standard set of example transformers for testing
    pub fn create_example_transformers(&mut self, context: LexicalContext) {
        // Storage cell transformer
        let storage_transformer = VariableTransformer::simple(
            "storage-cell".to_string(),
            "(storage-ref storage)".to_string(),
            "(storage-set! storage {val})".to_string(),
            context.clone(),
        );
        self.registry.register(storage_transformer);

        // Counter transformer
        let counter_transformer = VariableTransformer::simple(
            "counter".to_string(),
            "(counter-get)".to_string(),
            "(counter-set! {val})".to_string(),
            context.clone(),
        );
        self.registry.register(counter_transformer);

        // Array element transformer
        let array_element_transformer =
            create_vector_accessor_transformer("my-array".to_string(), 0, context.clone());
        self.registry.register(array_element_transformer);

        // Property transformer
        let property_transformer = create_hash_accessor_transformer(
            "properties".to_string(),
            "value".to_string(),
            context,
        );
        self.registry.register(property_transformer);
    }
}

impl Default for VariableTransformerBuiltins {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilities for working with variable transformers in the REPL and runtime
pub mod variable_transformer_utils {
    use super::*;

    /// Creates a variable transformer from a simplified specification
    pub fn create_from_spec(
        name: String,
        reference_expr: String,
        assignment_expr: String,
        call_expr: Option<String>,
        context: LexicalContext,
    ) -> VariableTransformer {
        let patterns = if let Some(call) = call_expr {
            vec![
                (
                    IdentifierContext::Reference,
                    "id".to_string(),
                    reference_expr,
                ),
                (
                    IdentifierContext::Assignment,
                    "(set! id val)".to_string(),
                    assignment_expr,
                ),
                (
                    IdentifierContext::ProcedureCall,
                    "(proc . args)".to_string(),
                    call,
                ),
            ]
        } else {
            vec![
                (
                    IdentifierContext::Reference,
                    "id".to_string(),
                    reference_expr,
                ),
                (
                    IdentifierContext::Assignment,
                    "(set! id val)".to_string(),
                    assignment_expr,
                ),
            ]
        };

        let transformation_logic = TransformationLogic::Complex { patterns };
        let transformer_proc = TransformerProcedure::Procedure {
            name: name.clone(),
            transformation_logic,
        };

        VariableTransformer::new(name, transformer_proc, context)
    }

    /// Validates a variable transformer specification
    pub fn validate_transformer_spec(
        reference_expr: &str,
        assignment_expr: &str,
        call_expr: Option<&str>,
    ) -> Result<()> {
        // Basic validation - in practice this would be more sophisticated
        if reference_expr.is_empty() {
            return Err(Box::new(Error::MacroError {
                message: "Reference expression cannot be empty".to_string(),
                span: Span::new(0, 0),
            }));
        }

        if assignment_expr.is_empty() {
            return Err(Box::new(Error::MacroError {
                message: "Assignment expression cannot be empty".to_string(),
                span: Span::new(0, 0),
            }));
        }

        // Check that assignment expression contains {val} placeholder
        if !assignment_expr.contains("{val}") {
            return Err(Box::new(Error::MacroError {
                message: "Assignment expression must contain {val} placeholder".to_string(),
                span: Span::new(0, 0),
            }));
        }

        // If call expression is provided, validate it
        if let Some(call) = call_expr {
            if call.is_empty() {
                return Err(Box::new(Error::MacroError {
                    message: "Call expression cannot be empty".to_string(),
                    span: Span::new(0, 0),
                }));
            }
        }

        Ok(())
    }

    /// Pretty-prints a variable transformer for debugging
    pub fn format_transformer(transformer: &VariableTransformer) -> String {
        let mut result = format!("Variable Transformer: {}\n", transformer.name);
        result.push_str(&format!(
            "  Context Sensitive: {}\n",
            transformer.context_sensitive
        ));

        match &transformer.transformer_proc {
            TransformerProcedure::Procedure {
                name,
                transformation_logic,
            } => {
                result.push_str(&format!("  Type: Procedure ({name})\n"));
                match transformation_logic {
                    TransformationLogic::Complex { patterns } => {
                        result.push_str("  Patterns:\n");
                        for (context, pattern, template) in patterns {
                            result.push_str(&format!("    {context}: {pattern} -> {template}\n"));
                        }
                    }
                    _ => {
                        result.push_str("  Type: Simple\n");
                    }
                }
            }
            TransformerProcedure::SyntaxCase { literals, clauses } => {
                result.push_str("  Type: Syntax-case\n");
                result.push_str(&format!("  Literals: {literals:?}\n"));
                result.push_str(&format!("  Clauses: {}\n", clauses.len()));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_make_variable_transformer() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let dummy_value = Value::Nil; // Placeholder

        let transformer =
            make_variable_transformer(dummy_value, Some("test-transformer".to_string()), context)
                .unwrap();

        assert_eq!(transformer.name, "test-transformer");
        assert!(transformer.context_sensitive);
    }

    #[test]
    fn test_make_simple_variable_transformer() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        let transformer = make_simple_variable_transformer(
            "(get-value)".to_string(),
            "(set-value! {val})".to_string(),
            Some("simple-test".to_string()),
            context,
        )
        .unwrap();

        assert_eq!(transformer.name, "simple-test");
        assert!(transformer.supports_context(&IdentifierContext::Reference));
        assert!(transformer.supports_context(&IdentifierContext::Assignment));
    }

    #[test]
    fn test_create_accessor_transformer() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        let transformer =
            create_accessor_transformer("get-x".to_string(), "set-x!".to_string(), context);

        assert_eq!(transformer.name, "get-x-accessor");
        assert!(transformer.supports_context(&IdentifierContext::Reference));
        assert!(transformer.supports_context(&IdentifierContext::Assignment));
    }

    #[test]
    fn test_create_vector_accessor_transformer() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        let transformer = create_vector_accessor_transformer("my-vec".to_string(), 5, context);

        assert_eq!(transformer.name, "my-vec-[5]");
    }

    #[test]
    fn test_variable_transformer_utils() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        // Test validation
        assert!(
            variable_transformer_utils::validate_transformer_spec("(get)", "(set {val})", None)
                .is_ok()
        );

        assert!(
            variable_transformer_utils::validate_transformer_spec("", "(set {val})", None).is_err()
        );

        assert!(
            variable_transformer_utils::validate_transformer_spec(
                "(get)",
                "(set value)", // Missing {val}
                None
            )
            .is_err()
        );

        // Test creation from spec
        let transformer = variable_transformer_utils::create_from_spec(
            "test-spec".to_string(),
            "(getter)".to_string(),
            "(setter {val})".to_string(),
            Some("(caller . {args})".to_string()),
            context,
        );

        assert_eq!(transformer.name, "test-spec");
        assert!(transformer.supports_context(&IdentifierContext::Reference));
        assert!(transformer.supports_context(&IdentifierContext::Assignment));
        assert!(transformer.supports_context(&IdentifierContext::ProcedureCall));
    }

    #[test]
    fn test_builtins_registry() {
        let mut builtins = VariableTransformerBuiltins::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        builtins.create_example_transformers(context);

        let registry = builtins.registry();
        assert!(registry.is_variable_transformer("storage-cell"));
        assert!(registry.is_variable_transformer("counter"));
        assert!(registry.is_variable_transformer("my-array-[0]"));
        assert!(!registry.is_variable_transformer("nonexistent"));
    }
}
