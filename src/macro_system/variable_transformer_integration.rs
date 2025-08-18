//! Integration of variable transformers with syntax-case and hygiene systems
//!
//! This module provides the integration layer that allows variable transformers
//! to work seamlessly with the existing syntax-case macro system and hygiene
//! mechanisms. It includes enhanced pattern matching and template expansion
//! that is aware of identifier transformers.

use super::{
    identifier_transformers::{
        VariableTransformer, VariableTransformerRegistry, IdentifierContext, ContextDetector
    },
    syntax_case::{SyntaxPattern, SyntaxBindings},
    syntax_objects::{SyntaxObject, LexicalContext, syntax_utils},
    advanced_hygiene::{HygieneResolver, Mark, MarkSet},
    context_aware_expander::ContextAwareMacroExpander,
};
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Span};
use std::collections::{HashMap, HashSet};

/// Enhanced syntax-case that is aware of variable transformers
pub struct VariableTransformerAwareSyntaxCase {
    /// Pattern to match
    pub pattern: SyntaxPattern,
    /// Template for expansion
    pub template: SyntaxTemplate,
    /// Literals that should not be expanded as transformers
    pub literals: HashSet<String>,
    /// Variable transformer registry for context checking
    pub transformer_registry: VariableTransformerRegistry,
}

impl VariableTransformerAwareSyntaxCase {
    /// Creates a new transformer-aware syntax-case
    pub fn new(
        pattern: SyntaxPattern,
        template: SyntaxTemplate,
        literals: Vec<String>,
        transformer_registry: VariableTransformerRegistry,
    ) -> Self {
        Self {
            pattern,
            template,
            literals: literals.into_iter().collect(),
            transformer_registry,
        }
    }

    /// Matches a syntax object against the pattern, taking variable transformers into account
    pub fn match_with_transformers(
        &self,
        syntax: &SyntaxObject,
        context: IdentifierContext,
        hygiene_resolver: &mut HygieneResolver,
    ) -> Result<Option<SyntaxBindings>> {
        let mut bindings = SyntaxBindings::new();
        
        if self.match_pattern_with_transformers(
            &self.pattern,
            syntax,
            &mut bindings,
            context,
            hygiene_resolver,
        )? {
            Ok(Some(bindings))
        } else {
            Ok(None)
        }
    }

    /// Internal pattern matching that considers variable transformers
    fn match_pattern_with_transformers(
        &self,
        pattern: &SyntaxPattern,
        syntax: &SyntaxObject,
        bindings: &mut SyntaxBindings,
        context: IdentifierContext,
        hygiene_resolver: &mut HygieneResolver,
    ) -> Result<bool> {
        match pattern {
            SyntaxPattern::PatternVariable(name) => {
                // Check if this syntax object is a variable transformer reference
                if let Some(id_name) = syntax.identifier_name() {
                    if self.transformer_registry.is_variable_transformer(id_name) 
                        && !self.literals.contains(id_name) {
                        // Expand the variable transformer in the current context
                        let expanded = self.transformer_registry.expand_variable_transformer(
                            id_name,
                            syntax,
                            None, // No containing form for pattern matching
                            hygiene_resolver,
                        )?;
                        bindings.bind(name.clone(), expanded);
                    } else {
                        bindings.bind(name.clone(), syntax.clone());
                    }
                } else {
                    bindings.bind(name.clone(), syntax.clone());
                }
                Ok(true)
            }

            SyntaxPattern::Literal(literal) => {
                Ok(syntax.is_literal() && self.literal_matches(literal, syntax))
            }

            SyntaxPattern::Identifier { name, binding_level } => {
                if let Some(id_name) = syntax.identifier_name() {
                    if id_name == name {
                        // Check if this is a variable transformer and handle accordingly
                        if self.transformer_registry.is_variable_transformer(id_name) 
                            && !self.literals.contains(id_name) {
                            // For identifiers in patterns, we might want to match the expanded form
                            // or the original form depending on context
                            match context {
                                IdentifierContext::Pattern => {
                                    // In pattern context, match the identifier as-is
                                    Ok(true)
                                }
                                _ => {
                                    // In other contexts, consider if we should expand
                                    Ok(true)
                                }
                            }
                        } else {
                            Ok(true)
                        }
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }

            SyntaxPattern::Nil => {
                Ok(syntax.is_list() && syntax.as_list().is_some_and(|list| list.is_empty()))
            }

            SyntaxPattern::List(patterns) => {
                if let Some(list) = syntax.as_list() {
                    if patterns.len() != list.len() {
                        return Ok(false);
                    }

                    for (pattern, syntax_elem) in patterns.iter().zip(list.iter()) {
                        if !self.match_pattern_with_transformers(
                            pattern,
                            syntax_elem,
                            bindings,
                            context.clone(),
                            hygiene_resolver,
                        )? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }

            SyntaxPattern::ImproperList { patterns, tail } => {
                if let Some(list) = syntax.as_list() {
                    if list.len() < patterns.len() {
                        return Ok(false);
                    }

                    // Match the initial patterns
                    for (pattern, syntax_elem) in patterns.iter().zip(list.iter()) {
                        if !self.match_pattern_with_transformers(
                            pattern,
                            syntax_elem,
                            bindings,
                            context.clone(),
                            hygiene_resolver,
                        )? {
                            return Ok(false);
                        }
                    }

                    // Match the tail against the remaining elements
                    let tail_elements = list.iter().skip(patterns.len()).cloned().collect();
                    let tail_syntax = syntax_utils::make_list_syntax(
                        tail_elements,
                        syntax.span,
                        syntax.context.clone(),
                    );

                    self.match_pattern_with_transformers(
                        tail,
                        &tail_syntax,
                        bindings,
                        context,
                        hygiene_resolver,
                    )
                } else {
                    Ok(false)
                }
            }

            SyntaxPattern::Ellipsis { pattern, min_count, max_count } => {
                self.match_ellipsis_pattern_with_transformers(
                    pattern,
                    syntax,
                    bindings,
                    *min_count,
                    *max_count,
                    context,
                    hygiene_resolver,
                )
            }

            SyntaxPattern::Alternative(alternatives) => {
                for alt_pattern in alternatives {
                    let mut alt_bindings = bindings.clone();
                    if self.match_pattern_with_transformers(
                        alt_pattern,
                        syntax,
                        &mut alt_bindings,
                        context.clone(),
                        hygiene_resolver,
                    )? {
                        bindings.merge(alt_bindings);
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            SyntaxPattern::Guard { pattern, predicate } => {
                if self.match_pattern_with_transformers(
                    pattern,
                    syntax,
                    bindings,
                    context,
                    hygiene_resolver,
                )? {
                    Ok(predicate(syntax))
                } else {
                    Ok(false)
                }
            }

            SyntaxPattern::WithProperties { pattern, required_properties } => {
                // Check if the syntax has the required properties
                for prop_name in required_properties {
                    if syntax.get_property(prop_name).is_none() {
                        return Ok(false);
                    }
                }

                self.match_pattern_with_transformers(
                    pattern,
                    syntax,
                    bindings,
                    context,
                    hygiene_resolver,
                )
            }

            SyntaxPattern::Vector(_patterns) => {
                // Vector patterns would be implemented similarly to lists
                // For now, return false as vectors need special handling
                Ok(false)
            }
        }
    }

    /// Matches an ellipsis pattern with transformer awareness
    #[allow(clippy::too_many_arguments)]
    fn match_ellipsis_pattern_with_transformers(
        &self,
        pattern: &SyntaxPattern,
        syntax: &SyntaxObject,
        bindings: &mut SyntaxBindings,
        min_count: usize,
        max_count: Option<usize>,
        context: IdentifierContext,
        hygiene_resolver: &mut HygieneResolver,
    ) -> Result<bool> {
        if let Some(list) = syntax.as_list() {
            let count = list.len();
            
            if count < min_count {
                return Ok(false);
            }
            
            if let Some(max) = max_count {
                if count > max {
                    return Ok(false);
                }
            }

            // Match each element against the pattern
            let mut ellipsis_bindings = HashMap::new();
            for syntax_elem in list {
                let mut elem_bindings = SyntaxBindings::new();
                if !self.match_pattern_with_transformers(
                    pattern,
                    &syntax_elem,
                    &mut elem_bindings,
                    context.clone(),
                    hygiene_resolver,
                )? {
                    return Ok(false);
                }

                // Collect ellipsis bindings
                for binding_name in elem_bindings.binding_names() {
                    if let Some(bound_syntax) = elem_bindings.get(binding_name) {
                        ellipsis_bindings
                            .entry(binding_name.to_string())
                            .or_insert_with(Vec::new)
                            .push(bound_syntax.clone());
                    }
                }
            }

            // Add ellipsis bindings to the main bindings
            for (name, syntaxes) in ellipsis_bindings {
                bindings.bind_ellipsis(name, syntaxes);
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Checks if a literal matches the syntax object
    fn literal_matches(&self, literal: &Literal, syntax: &SyntaxObject) -> bool {
        if let Expr::Literal(syntax_literal) = &syntax.expr {
            match (literal, syntax_literal) {
                (Literal::ExactInteger(a), Literal::ExactInteger(b)) => a == b,
                (Literal::InexactReal(a), Literal::InexactReal(b)) => (a - b).abs() < f64::EPSILON,
                (Literal::String(a), Literal::String(b)) => **a == **b,
                (Literal::Character(a), Literal::Character(b)) => a == b,
                (Literal::Boolean(a), Literal::Boolean(b)) => a == b,
                _ => false,
            }
        } else {
            false
        }
    }
}

/// Enhanced syntax template that can handle variable transformer expansion
#[derive(Debug, Clone)]
pub struct VariableTransformerAwareTemplate {
    /// The template pattern
    pub template: SyntaxTemplate,
    /// Variable transformer registry
    pub transformer_registry: VariableTransformerRegistry,
    /// Context for template expansion
    pub expansion_context: IdentifierContext,
}

impl VariableTransformerAwareTemplate {
    /// Creates a new transformer-aware template
    pub fn new(
        template: SyntaxTemplate,
        transformer_registry: VariableTransformerRegistry,
        expansion_context: IdentifierContext,
    ) -> Self {
        Self {
            template,
            transformer_registry,
            expansion_context,
        }
    }

    /// Expands the template with transformer awareness
    pub fn expand_with_transformers(
        &self,
        bindings: &SyntaxBindings,
        hygiene_resolver: &mut HygieneResolver,
    ) -> Result<SyntaxObject> {
        self.template.expand_with_transformer_support(
            bindings,
            &self.transformer_registry,
            self.expansion_context.clone(),
            hygiene_resolver,
        )
    }
}

/// Extension trait for SyntaxTemplate to add transformer support
pub trait SyntaxTemplateTransformerSupport {
    /// Expands a template with variable transformer support
    fn expand_with_transformer_support(
        &self,
        bindings: &SyntaxBindings,
        transformer_registry: &VariableTransformerRegistry,
        context: IdentifierContext,
        hygiene_resolver: &mut HygieneResolver,
    ) -> Result<SyntaxObject>;
}

/// Simple implementation of SyntaxTemplate for this integration
#[derive(Debug, Clone)]
pub struct SyntaxTemplate {
    /// Template elements
    elements: Vec<TemplateElement>,
}

/// Elements that can appear in a syntax template
#[derive(Debug, Clone)]
pub enum TemplateElement {
    /// A literal syntax object
    Literal(Box<SyntaxObject>),
    /// A pattern variable reference
    Variable(String),
    /// An ellipsis expansion
    Ellipsis {
        /// Pattern to expand under ellipsis
        pattern: Box<TemplateElement>,
        /// Optional separator between expanded elements
        separator: Option<Box<TemplateElement>>,
    },
    /// A list of template elements
    List(Vec<TemplateElement>),
    /// A variable transformer reference that should be expanded
    TransformerReference {
        /// Name of the variable transformer
        name: String,
        /// Context in which the transformer is referenced
        context: IdentifierContext,
    },
}

impl SyntaxTemplate {
    /// Creates a new syntax template
    pub fn new(elements: Vec<TemplateElement>) -> Self {
        Self { elements }
    }

    /// Creates a simple template from a pattern variable
    pub fn variable(name: String) -> Self {
        Self::new(vec![TemplateElement::Variable(name)])
    }

    /// Creates a template from a literal
    pub fn literal(syntax: SyntaxObject) -> Self {
        Self::new(vec![TemplateElement::Literal(Box::new(syntax))])
    }

    /// Expands the template using bindings
    pub fn expand(
        &self,
        bindings: &SyntaxBindings,
        hygiene_resolver: &mut HygieneResolver,
    ) -> Result<SyntaxObject> {
        if self.elements.len() == 1 {
            self.expand_element(&self.elements[0], bindings, hygiene_resolver)
        } else {
            let expanded_elements: Result<Vec<SyntaxObject>> = self.elements
                .iter()
                .map(|elem| self.expand_element(elem, bindings, hygiene_resolver))
                .collect();

            let elements = expanded_elements?;
            let first_span = elements.first().map_or(Span::new(0, 0), |e| e.span);
            let first_context = elements.first().map_or_else(
                || LexicalContext::new(0, vec!["template".to_string()]),
                |e| e.context.clone(),
            );

            Ok(syntax_utils::make_list_syntax(elements, first_span, first_context))
        }
    }

    /// Expands a single template element
    fn expand_element(
        &self,
        element: &TemplateElement,
        bindings: &SyntaxBindings,
        hygiene_resolver: &mut HygieneResolver,
    ) -> Result<SyntaxObject> {
        match element {
            TemplateElement::Literal(syntax) => Ok((**syntax).clone()),

            TemplateElement::Variable(name) => {
                bindings.get(name).cloned().ok_or_else(|| Box::new(Error::MacroError {
                    message: format!("Unbound pattern variable: {name}"),
                    span: Span::new(0, 0),
                }))
            }

            TemplateElement::List(elements) => {
                let expanded_elements: Result<Vec<SyntaxObject>> = elements
                    .iter()
                    .map(|elem| self.expand_element(elem, bindings, hygiene_resolver))
                    .collect();

                let elements = expanded_elements?;
                let first_span = elements.first().map_or(Span::new(0, 0), |e| e.span);
                let first_context = elements.first().map_or_else(
                    || LexicalContext::new(0, vec!["template".to_string()]),
                    |e| e.context.clone(),
                );

                Ok(syntax_utils::make_list_syntax(elements, first_span, first_context))
            }

            TemplateElement::Ellipsis { pattern, separator: _separator } => {
                // For ellipsis expansion, we need to handle pattern variables that are bound to lists
                self.expand_ellipsis_element(pattern, bindings, hygiene_resolver)
            }

            TemplateElement::TransformerReference { name, context } => {
                // Create a placeholder syntax object for the transformer reference
                let placeholder = syntax_utils::make_identifier_syntax(
                    name.clone(),
                    Span::new(0, 0),
                    LexicalContext::new(0, vec!["template".to_string()]),
                );

                // Note: In a full implementation, we would expand the transformer here
                // For now, just return the placeholder
                Ok(placeholder)
            }
        }
    }

    /// Expands an ellipsis pattern
    fn expand_ellipsis_element(
        &self,
        pattern: &TemplateElement,
        bindings: &SyntaxBindings,
        hygiene_resolver: &mut HygieneResolver,
    ) -> Result<SyntaxObject> {
        // This is a simplified ellipsis expansion
        // In a real implementation, this would handle multiple pattern variables
        // and complex ellipsis structures
        
        if let TemplateElement::Variable(name) = pattern {
            if let Some(ellipsis_binding) = bindings.get_ellipsis(name) {
                let first_span = ellipsis_binding.first().map_or(Span::new(0, 0), |e| e.span);
                let first_context = ellipsis_binding.first().map_or_else(
                    || LexicalContext::new(0, vec!["ellipsis".to_string()]),
                    |e| e.context.clone(),
                );

                return Ok(syntax_utils::make_list_syntax(
                    ellipsis_binding.clone(),
                    first_span,
                    first_context,
                ));
            }
        }

        // Fallback: expand the pattern as a single element
        self.expand_element(pattern, bindings, hygiene_resolver)
    }
}

impl SyntaxTemplateTransformerSupport for SyntaxTemplate {
    fn expand_with_transformer_support(
        &self,
        bindings: &SyntaxBindings,
        transformer_registry: &VariableTransformerRegistry,
        context: IdentifierContext,
        hygiene_resolver: &mut HygieneResolver,
    ) -> Result<SyntaxObject> {
        // For now, delegate to the regular expand method
        // In a full implementation, this would handle transformer-specific expansions
        self.expand(bindings, hygiene_resolver)
    }
}

/// Procedures for working with variable-transformer-aware syntax-case
pub mod syntax_procedures {
    use super::*;

    /// Creates a variable-transformer-aware syntax-case matcher
    pub fn make_transformer_aware_syntax_case(
        pattern: SyntaxPattern,
        template: SyntaxTemplate,
        literals: Vec<String>,
        transformer_registry: VariableTransformerRegistry,
    ) -> VariableTransformerAwareSyntaxCase {
        VariableTransformerAwareSyntaxCase::new(pattern, template, literals, transformer_registry)
    }

    /// Utility for creating common syntax patterns with transformer support
    pub fn identifier_pattern(name: String) -> SyntaxPattern {
        SyntaxPattern::Identifier {
            name,
            binding_level: None,
        }
    }

    /// Utility for creating list patterns
    pub fn list_pattern(patterns: Vec<SyntaxPattern>) -> SyntaxPattern {
        SyntaxPattern::List(patterns)
    }

    /// Utility for creating ellipsis patterns
    pub fn ellipsis_pattern(pattern: SyntaxPattern, min_count: usize) -> SyntaxPattern {
        SyntaxPattern::Ellipsis {
            pattern: Box::new(pattern),
            min_count,
            max_count: None,
        }
    }

    /// Utility for creating pattern variables
    pub fn pattern_variable(name: String) -> SyntaxPattern {
        SyntaxPattern::PatternVariable(name)
    }

    /// Utility for creating literal patterns
    pub fn literal_pattern(literal: Literal) -> SyntaxPattern {
        SyntaxPattern::Literal(literal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::diagnostics::Span;

    #[test]
    fn test_transformer_aware_syntax_case() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let mut registry = VariableTransformerRegistry::new();
        
        // Register a simple variable transformer
        let transformer = VariableTransformer::simple(
            "my-var".to_string(),
            "(get-value)".to_string(),
            "(set-value! {val})".to_string(),
            context.clone(),
        );
        registry.register(transformer);

        // Create a pattern that matches an identifier
        let pattern = SyntaxPattern::Identifier {
            name: "my-var".to_string(),
            binding_level: None,
        };

        // Create a simple template
        let template = SyntaxTemplate::variable("my-var".to_string());

        // Create the transformer-aware syntax-case
        let syntax_case = VariableTransformerAwareSyntaxCase::new(
            pattern,
            template,
            vec![],
            registry,
        );

        // Test matching
        let span = Span::new(0, 1);
        let test_syntax = syntax_utils::make_identifier_syntax(
            "my-var".to_string(),
            span,
            context,
        );

        let mut hygiene_resolver = HygieneResolver::new();
        let result = syntax_case.match_with_transformers(
            &test_syntax,
            IdentifierContext::Reference,
            &mut hygiene_resolver,
        );

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_syntax_template_expansion() {
        let bindings = {
            let mut b = SyntaxBindings::new();
            let context = LexicalContext::new(1, vec!["test".to_string()]);
            let span = Span::new(0, 1);
            
            b.bind(
                "x".to_string(),
                syntax_utils::make_identifier_syntax("value".to_string(), span, context),
            );
            b
        };

        let template = SyntaxTemplate::variable("x".to_string());
        let mut hygiene_resolver = HygieneResolver::new();
        
        let result = template.expand(&bindings, &mut hygiene_resolver);
        assert!(result.is_ok());
        
        let expanded = result.unwrap();
        assert_eq!(expanded.identifier_name(), Some("value"));
    }

    #[test]
    fn test_syntax_pattern_utilities() {
        // Test pattern creation utilities
        let id_pattern = syntax_procedures::identifier_pattern("test".to_string());
        assert!(matches!(id_pattern, SyntaxPattern::Identifier { .. }));

        let var_pattern = syntax_procedures::pattern_variable("x".to_string());
        assert!(matches!(var_pattern, SyntaxPattern::PatternVariable(_)));

        let literal_pattern = syntax_procedures::literal_pattern(Literal::Integer(42));
        assert!(matches!(literal_pattern, SyntaxPattern::Literal(_)));

        let list_pattern = syntax_procedures::list_pattern(vec![var_pattern.clone()]);
        assert!(matches!(list_pattern, SyntaxPattern::List(_)));

        let ellipsis_pattern = syntax_procedures::ellipsis_pattern(var_pattern, 0);
        assert!(matches!(ellipsis_pattern, SyntaxPattern::Ellipsis { .. }));
    }
}