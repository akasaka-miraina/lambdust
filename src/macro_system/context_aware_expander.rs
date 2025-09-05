//! Context-aware macro expander with identifier transformer support
//!
//! This module extends the existing macro expansion system to handle
//! identifier transformers with proper context detection and expansion.
//! It integrates with the syntax object system and provides R6RS-compliant
//! variable transformer behavior.

use super::{
    advanced_hygiene::{HygieneResolver, Mark, MarkSet},
    expander::ConfigurableExpander,
    identifier_transformers::{
        ContextDetector, IdentifierContext, VariableTransformer, VariableTransformerRegistry,
    },
    syntax_integration::SyntaxAwareMacroExpander,
    syntax_objects::{LexicalContext, SyntaxObject, syntax_utils},
    unified_expander::{
        ExpansionMode, MacroTransformerType, UnifiedMacroExpander, UnifiedMacroTransformer,
    },
    variable_transformer_builtins::VariableTransformerBuiltins,
};
use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::{Environment, Value};
use std::collections::HashMap;
use std::rc::Rc;

/// Enhanced macro expander that handles both regular macros and variable transformers
pub struct ContextAwareMacroExpander {
    /// Unified expander for regular macros
    unified_expander: Box<UnifiedMacroExpander>,
    /// Registry for variable transformers
    variable_transformer_registry: VariableTransformerRegistry,
    /// Built-in variable transformer procedures
    builtins: VariableTransformerBuiltins,
    /// Hygiene resolver for syntax transformations
    hygiene_resolver: HygieneResolver,
    /// Context detector for identifying usage patterns
    context_detector: ContextDetector,
    /// Current expansion depth
    expansion_depth: usize,
    /// Maximum expansion depth
    max_expansion_depth: usize,
    /// Expansion statistics
    stats: ContextAwareExpansionStats,
}

/// Statistics for context-aware expansion
#[derive(Debug, Clone, Default)]
pub struct ContextAwareExpansionStats {
    /// Regular macro expansions
    pub macro_expansions: usize,
    /// Variable transformer expansions
    pub variable_transformer_expansions: usize,
    /// Context detections performed
    pub context_detections: usize,
    /// Reference context expansions
    pub reference_expansions: usize,
    /// Assignment context expansions
    pub assignment_expansions: usize,
    /// Procedure call context expansions
    pub call_expansions: usize,
    /// Hygiene transformations applied
    pub hygiene_transformations: usize,
}

impl ContextAwareMacroExpander {
    /// Creates a new context-aware macro expander
    pub fn new() -> Self {
        Self {
            unified_expander: Box::new(UnifiedMacroExpander::new()),
            variable_transformer_registry: VariableTransformerRegistry::new(),
            builtins: VariableTransformerBuiltins::new(),
            hygiene_resolver: HygieneResolver::new(),
            context_detector: ContextDetector,
            expansion_depth: 0,
            max_expansion_depth: 100,
            stats: ContextAwareExpansionStats::default(),
        }
    }

    /// Creates an expander with custom configuration
    pub fn with_max_depth(max_depth: usize) -> Self {
        let mut expander = Self::new();
        expander.max_expansion_depth = max_depth;
        expander
    }

    /// Registers a variable transformer
    pub fn register_variable_transformer(&mut self, transformer: VariableTransformer) {
        self.variable_transformer_registry.register(transformer);
    }

    /// Registers a regular macro transformer
    pub fn register_macro_transformer(
        &mut self,
        transformer: UnifiedMacroTransformer,
    ) -> Result<()> {
        self.unified_expander.register_transformer(transformer)
    }

    /// Main expansion entry point that handles both macros and variable transformers
    pub fn expand(&mut self, syntax: &SyntaxObject) -> Result<SyntaxObject> {
        self.expand_with_context(syntax, None, None)
    }

    /// Expands with explicit context information
    pub fn expand_with_context(
        &mut self,
        syntax: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
        parent_form: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        // Check expansion depth
        if self.expansion_depth >= self.max_expansion_depth {
            return Err(Box::new(Error::MacroError {
                message: format!(
                    "Maximum expansion depth {} exceeded",
                    self.max_expansion_depth
                ),
                span: syntax.span,
            }));
        }

        self.expansion_depth += 1;
        let result = self.expand_internal(syntax, containing_form, parent_form);
        self.expansion_depth -= 1;

        result
    }

    /// Internal expansion logic
    fn expand_internal(
        &mut self,
        syntax: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
        parent_form: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        match &syntax.expr {
            Expr::Identifier(name) | Expr::Symbol(name) => {
                self.expand_identifier(name, syntax, containing_form, parent_form)
            }

            Expr::List(elements) => {
                self.expand_list(elements, syntax, containing_form, parent_form)
            }

            Expr::Application { operator, operands } => {
                self.expand_application(operator, operands, syntax, containing_form, parent_form)
            }

            // For other expressions, recursively expand sub-expressions
            _ => self.expand_other(syntax),
        }
    }

    /// Expands an identifier, checking for variable transformers first
    fn expand_identifier(
        &mut self,
        name: &str,
        syntax: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
        parent_form: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        self.stats.context_detections += 1;

        // Check if this is a variable transformer
        if self
            .variable_transformer_registry
            .is_variable_transformer(name)
        {
            return self.expand_variable_transformer(name, syntax, containing_form, parent_form);
        }

        // Check if this is a regular macro
        if self.unified_expander.is_macro(name) {
            self.stats.macro_expansions += 1;
            // For identifier expansion, we need to convert to proper arguments
            // Since this is just an identifier, create an empty args list
            let empty_args: &[Spanned<Expr>] = &[];
            let result = self.unified_expander.expand_macro(
                name,
                empty_args,
                syntax.span,
                &Environment::new(None, 0),
            )?;
            // Convert result back to syntax object
            return self.unified_expander.expr_to_syntax(result, None);
        }

        // Not a transformer or macro, return as-is
        Ok(syntax.clone())
    }

    /// Expands a variable transformer based on context
    fn expand_variable_transformer(
        &mut self,
        name: &str,
        syntax: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
        parent_form: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        self.stats.variable_transformer_expansions += 1;

        // Detect the usage context
        let context = if let Some(grandparent) = parent_form {
            ContextDetector::detect_context_from_structure(
                syntax,
                containing_form.unwrap_or(syntax),
                Some(grandparent),
            )
        } else {
            ContextDetector::detect_context(syntax, containing_form)
        };

        // Update statistics based on context
        match context {
            IdentifierContext::Reference => self.stats.reference_expansions += 1,
            IdentifierContext::Assignment => self.stats.assignment_expansions += 1,
            IdentifierContext::ProcedureCall => self.stats.call_expansions += 1,
            _ => {}
        }

        // Expand the variable transformer
        let result = self
            .variable_transformer_registry
            .expand_variable_transformer(
                name,
                syntax,
                containing_form,
                &mut self.hygiene_resolver,
            )?;

        self.stats.hygiene_transformations += 1;

        // Recursively expand the result in case it contains more macros
        self.expand_with_context(&result, containing_form, parent_form)
    }

    /// Expands a list expression, handling special forms appropriately
    fn expand_list(
        &mut self,
        elements: &[Spanned<Expr>],
        syntax: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
        parent_form: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        if elements.is_empty() {
            return Ok(syntax.clone());
        }

        // Check for special forms that affect context detection
        if let Expr::Identifier(name) | Expr::Symbol(name) = &elements[0].inner {
            match name.as_str() {
                "set!" => {
                    return self.expand_set_form(elements, syntax, containing_form, parent_form);
                }
                "define" => {
                    return self.expand_define_form(elements, syntax, containing_form, parent_form);
                }
                "lambda" => {
                    return self.expand_lambda_form(elements, syntax, containing_form, parent_form);
                }
                "let" | "let*" | "letrec" => {
                    return self.expand_let_form(elements, syntax, containing_form, parent_form);
                }
                _ => {}
            }
        }

        // Regular list expansion - expand each element
        let mut expanded_elements = Vec::new();
        for (i, element) in elements.iter().enumerate() {
            let element_syntax =
                SyntaxObject::from_spanned(element.clone(), syntax.context.clone());

            // For the first element in a list, it might be in procedure position
            let context_hint = if i == 0 {
                Some(IdentifierContext::ProcedureCall)
            } else {
                None
            };

            let expanded =
                self.expand_with_context(&element_syntax, Some(syntax), containing_form)?;
            expanded_elements.push(expanded.to_spanned())
        }

        Ok(syntax.with_expr(Expr::List(expanded_elements)))
    }

    /// Expands an application expression
    fn expand_application(
        &mut self,
        operator: &Spanned<Expr>,
        operands: &[Spanned<Expr>],
        syntax: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
        parent_form: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        // Expand the operator (might be a variable transformer in call context)
        let operator_syntax = SyntaxObject::from_spanned(operator.clone(), syntax.context.clone());
        let expanded_operator =
            self.expand_with_context(&operator_syntax, Some(syntax), containing_form)?;

        // Expand the operands
        let mut expanded_operands = Vec::new();
        for operand in operands {
            let operand_syntax =
                SyntaxObject::from_spanned(operand.clone(), syntax.context.clone());
            let expanded =
                self.expand_with_context(&operand_syntax, Some(syntax), containing_form)?;
            expanded_operands.push(expanded.to_spanned())
        }

        Ok(syntax.with_expr(Expr::Application {
            operator: Box::new(expanded_operator.to_spanned()),
            operands: expanded_operands,
        }))
    }

    /// Expands a set! form with special handling for variable transformers
    fn expand_set_form(
        &mut self,
        elements: &[Spanned<Expr>],
        syntax: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
        parent_form: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        if elements.len() != 3 {
            return Err(Box::new(Error::MacroError {
                message: "set! requires exactly 2 arguments".to_string(),
                span: syntax.span,
            }));
        }

        // Check if the target is a variable transformer
        if let Expr::Identifier(name) | Expr::Symbol(name) = &elements[1].inner {
            if self
                .variable_transformer_registry
                .is_variable_transformer(name)
            {
                // Create a syntax object for the entire set! form
                let set_syntax = syntax.clone();

                // Expand the variable transformer in assignment context
                return self.expand_variable_transformer(
                    name,
                    &set_syntax,
                    containing_form,
                    parent_form,
                );
            }
        }

        // Regular set! expansion
        let target_syntax = SyntaxObject::from_spanned(elements[1].clone(), syntax.context.clone());
        let value_syntax = SyntaxObject::from_spanned(elements[2].clone(), syntax.context.clone());

        let expanded_target =
            self.expand_with_context(&target_syntax, Some(syntax), containing_form)?;
        let expanded_value =
            self.expand_with_context(&value_syntax, Some(syntax), containing_form)?;

        let expanded_elements = vec![
            elements[0].clone(), // set! keyword
            expanded_target.to_spanned(),
            expanded_value.to_spanned(),
        ];

        Ok(syntax.with_expr(Expr::List(expanded_elements)))
    }

    /// Expands a define form
    fn expand_define_form(
        &mut self,
        elements: &[Spanned<Expr>],
        syntax: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
        parent_form: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        if elements.len() < 3 {
            return Err(Box::new(Error::MacroError {
                message: "define requires at least 2 arguments".to_string(),
                span: syntax.span,
            }));
        }

        // Expand the definition value(s)
        let mut expanded_elements = vec![elements[0].clone(), elements[1].clone()]; // define and name

        for element in &elements[2..] {
            let element_syntax =
                SyntaxObject::from_spanned(element.clone(), syntax.context.clone());
            let expanded =
                self.expand_with_context(&element_syntax, Some(syntax), containing_form)?;
            expanded_elements.push(expanded.to_spanned())
        }

        Ok(syntax.with_expr(Expr::List(expanded_elements)))
    }

    /// Expands a lambda form
    fn expand_lambda_form(
        &mut self,
        elements: &[Spanned<Expr>],
        syntax: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
        parent_form: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        if elements.len() < 3 {
            return Err(Box::new(Error::MacroError {
                message: "lambda requires at least 2 arguments".to_string(),
                span: syntax.span,
            }));
        }

        // Don't expand the parameter list, but expand the body
        let mut expanded_elements = vec![elements[0].clone(), elements[1].clone()]; // lambda and params

        for element in &elements[2..] {
            let element_syntax =
                SyntaxObject::from_spanned(element.clone(), syntax.context.clone());
            let expanded =
                self.expand_with_context(&element_syntax, Some(syntax), containing_form)?;
            expanded_elements.push(expanded.to_spanned())
        }

        Ok(syntax.with_expr(Expr::List(expanded_elements)))
    }

    /// Expands let/let*/letrec forms
    fn expand_let_form(
        &mut self,
        elements: &[Spanned<Expr>],
        syntax: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
        parent_form: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        if elements.len() < 3 {
            return Err(Box::new(Error::MacroError {
                message: "let form requires at least 2 arguments".to_string(),
                span: syntax.span,
            }));
        }

        // Expand bindings and body, but not the binding names
        let mut expanded_elements = vec![elements[0].clone()]; // let keyword

        // Handle bindings
        if let Expr::List(bindings) = &elements[1].inner {
            let mut expanded_bindings = Vec::new();
            for binding in bindings {
                if let Expr::List(binding_pair) = &binding.inner {
                    if binding_pair.len() == 2 {
                        // Don't expand the variable name, but expand the value
                        let value_syntax = SyntaxObject::from_spanned(
                            binding_pair[1].clone(),
                            syntax.context.clone(),
                        );
                        let expanded_value =
                            self.expand_with_context(&value_syntax, Some(syntax), containing_form)?;

                        let expanded_binding = Spanned::new(
                            Expr::List(vec![binding_pair[0].clone(), expanded_value.to_spanned()]),
                            binding.span,
                        );
                        expanded_bindings.push(expanded_binding);
                    } else {
                        expanded_bindings.push(binding.clone())
                    }
                } else {
                    expanded_bindings.push(binding.clone())
                }
            }
            expanded_elements.push(Spanned::new(
                Expr::List(expanded_bindings),
                elements[1].span,
            ))
        } else {
            expanded_elements.push(elements[1].clone())
        }

        // Expand the body
        for element in &elements[2..] {
            let element_syntax =
                SyntaxObject::from_spanned(element.clone(), syntax.context.clone());
            let expanded =
                self.expand_with_context(&element_syntax, Some(syntax), containing_form)?;
            expanded_elements.push(expanded.to_spanned())
        }

        Ok(syntax.with_expr(Expr::List(expanded_elements)))
    }

    /// Expands other expression types
    fn expand_other(&mut self, syntax: &SyntaxObject) -> Result<SyntaxObject> {
        // For other expression types, we don't need to expand
        Ok(syntax.clone())
    }

    /// Gets the variable transformer registry
    pub fn variable_transformer_registry(&self) -> &VariableTransformerRegistry {
        &self.variable_transformer_registry
    }

    /// Gets a mutable reference to the variable transformer registry
    pub fn variable_transformer_registry_mut(&mut self) -> &mut VariableTransformerRegistry {
        &mut self.variable_transformer_registry
    }

    /// Gets the built-ins registry
    pub fn builtins(&self) -> &VariableTransformerBuiltins {
        &self.builtins
    }

    /// Gets a mutable reference to the built-ins registry
    pub fn builtins_mut(&mut self) -> &mut VariableTransformerBuiltins {
        &mut self.builtins
    }

    /// Gets the expansion statistics
    pub fn stats(&self) -> &ContextAwareExpansionStats {
        &self.stats
    }

    /// Resets the expansion statistics
    pub fn reset_stats(&mut self) {
        self.stats = ContextAwareExpansionStats::default();
    }

    /// Initializes the expander with standard variable transformers
    pub fn initialize_with_builtins(&mut self, context: LexicalContext) -> Result<()> {
        self.builtins.create_example_transformers(context);
        Ok(())
    }
}

impl Default for ContextAwareMacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::diagnostics::Span;

    #[test]
    fn test_context_aware_expander_creation() {
        let expander = ContextAwareMacroExpander::new();
        assert_eq!(expander.expansion_depth, 0);
        assert_eq!(expander.max_expansion_depth, 100);
    }

    #[test]
    fn test_variable_transformer_registration() {
        let mut expander = ContextAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        let transformer = VariableTransformer::simple(
            "test-var".to_string(),
            "(get-test)".to_string(),
            "(set-test! {val})".to_string(),
            context,
        );

        expander.register_variable_transformer(transformer);

        assert!(
            expander
                .variable_transformer_registry()
                .is_variable_transformer("test-var")
        )
    }

    #[test]
    fn test_identifier_expansion() {
        let mut expander = ContextAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 1);

        // Register a variable transformer
        let transformer = VariableTransformer::simple(
            "my-var".to_string(),
            "(storage-ref)".to_string(),
            "(storage-set! {val})".to_string(),
            context.clone(),
        );
        expander.register_variable_transformer(transformer);

        // Test identifier that's not a transformer
        let regular_id =
            syntax_utils::make_identifier_syntax("x".to_string(), span, context.clone());
        let result = expander.expand(&regular_id).unwrap();

        // Should be unchanged
        assert_eq!(result.identifier_name(), Some("x"))
    }

    #[test]
    fn test_set_form_expansion() {
        let mut expander = ContextAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 10);

        // Register a variable transformer
        let transformer = VariableTransformer::simple(
            "storage".to_string(),
            "(storage-ref)".to_string(),
            "(storage-set! {val})".to_string(),
            context.clone(),
        );
        expander.register_variable_transformer(transformer);

        // Create a set! form: (set! storage 42)
        let set_form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("storage".to_string(), span, context.clone()),
                syntax_utils::make_literal_syntax(Literal::Integer(42), span, context.clone()),
            ],
            span,
            context,
        );

        let result = expander.expand(&set_form);
        assert!(result.is_ok());

        // Check that variable transformer expansion was performed
        assert!(expander.stats().variable_transformer_expansions > 0);
        assert!(expander.stats().assignment_expansions > 0);
    }

    #[test]
    fn test_expansion_depth_limit() {
        let mut expander = ContextAwareMacroExpander::with_max_depth(5);
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 1);

        // Set depth to maximum
        expander.expansion_depth = 5;

        let syntax = syntax_utils::make_identifier_syntax("x".to_string(), span, context);
        let result = expander.expand(&syntax);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Maximum expansion depth")
        )
    }

    #[test]
    fn test_statistics_tracking() {
        let mut expander = ContextAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 1);

        // Register a variable transformer
        let transformer = VariableTransformer::simple(
            "counter".to_string(),
            "(get-count)".to_string(),
            "(set-count! {val})".to_string(),
            context.clone(),
        );
        expander.register_variable_transformer(transformer);

        // Test reference context
        let ref_syntax =
            syntax_utils::make_identifier_syntax("counter".to_string(), span, context.clone());
        let _ = expander.expand(&ref_syntax);

        assert!(expander.stats().variable_transformer_expansions > 0);
        assert!(expander.stats().reference_expansions > 0);

        // Reset and test assignment context
        expander.reset_stats();

        let set_form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("counter".to_string(), span, context.clone()),
                syntax_utils::make_literal_syntax(Literal::Integer(10), span, context.clone()),
            ],
            span,
            context,
        );

        let _ = expander.expand(&set_form);

        assert!(expander.stats().assignment_expansions > 0);
    }
}
