//! Quasisyntax implementation for template construction
//!
//! This module implements the #' (syntax) and #` (quasisyntax) forms for constructing
//! syntax objects with template expansion. It provides:
//! - #' (syntax): Direct syntax object construction
//! - #` (quasisyntax): Template-based syntax construction with unquote/unquote-splicing
//! - #, (unsyntax): Unquote within quasisyntax
//! - #,@ (unsyntax-splicing): Unquote-splicing within quasisyntax
//!
//! This implementation follows R6RS specifications for syntax templates.

use super::syntax_case::{SyntaxTemplate, SyntaxBindings};
use super::syntax_objects::{SyntaxObject, LexicalContext};
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Span, Spanned};
use std::collections::VecDeque;

/// Represents a quasisyntax template with embedded computations
#[derive(Debug, Clone, PartialEq)]
pub enum QuasisyntaxTemplate {
    /// A literal syntax value
    Literal(Literal),
    /// An identifier
    Identifier(String),
    /// Empty list
    Nil,
    /// A proper list of templates
    List(Vec<QuasisyntaxTemplate>),
    /// An improper list (dotted pair)
    ImproperList {
        /// Templates for the list elements
        templates: Vec<QuasisyntaxTemplate>,
        /// Template for the tail (final cdr)
        tail: Box<QuasisyntaxTemplate>,
    },
    /// A vector template
    Vector(Vec<QuasisyntaxTemplate>),
    /// Unquote - evaluate the enclosed expression at macro-expansion time
    Unquote(Box<QuasisyntaxTemplate>),
    /// Unquote-splicing - evaluate and splice the result into the surrounding list
    UnquoteSplicing(Box<QuasisyntaxTemplate>),
    /// Nested quasisyntax (for multiple levels)
    Nested {
        /// The nested template
        template: Box<QuasisyntaxTemplate>,
        /// Nesting depth level
        depth: usize,
    },
    /// Reference to a pattern variable
    PatternVariable(String),
    /// Escape to runtime computation
    Escape(Box<QuasisyntaxTemplate>),
    /// Conditional expansion
    Conditional {
        /// Template that evaluates to the condition
        condition: Box<QuasisyntaxTemplate>,
        /// Template to use if condition is true
        then_template: Box<QuasisyntaxTemplate>,
        /// Optional template to use if condition is false
        else_template: Option<Box<QuasisyntaxTemplate>>,
    },
}

/// Context for quasisyntax expansion
#[derive(Debug, Clone)]
pub struct QuasisyntaxContext {
    /// Current nesting depth
    depth: usize,
    /// Available pattern bindings
    bindings: SyntaxBindings,
    /// Lexical context for new syntax objects
    lexical_context: LexicalContext,
    /// Whether we're in a splicing context
    splicing_context: bool,
}

impl QuasisyntaxContext {
    /// Creates a new quasisyntax context
    pub fn new(
        bindings: SyntaxBindings,
        lexical_context: LexicalContext,
    ) -> Self {
        Self {
            depth: 0,
            bindings,
            lexical_context,
            splicing_context: false,
        }
    }

    /// Enters a deeper nesting level
    pub fn enter_level(&mut self) {
        self.depth += 1;
    }

    /// Exits a nesting level
    pub fn exit_level(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Sets splicing context
    pub fn set_splicing(&mut self, splicing: bool) {
        self.splicing_context = splicing;
    }

    /// Gets current depth
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Checks if we're in a splicing context
    pub fn is_splicing(&self) -> bool {
        self.splicing_context
    }

    /// Gets the lexical context
    pub fn lexical_context(&self) -> &LexicalContext {
        &self.lexical_context
    }

    /// Gets the bindings
    pub fn bindings(&self) -> &SyntaxBindings {
        &self.bindings
    }
}

impl QuasisyntaxTemplate {
    /// Creates a quasisyntax template from an expression
    pub fn from_expr(expr: &Expr) -> Self {
        match expr {
            Expr::Literal(lit) => QuasisyntaxTemplate::Literal(lit.clone()),
            Expr::Identifier(name) | Expr::Symbol(name) => {
                QuasisyntaxTemplate::Identifier(name.clone())
            }
            Expr::List(elements) => {
                let templates: Vec<_> = elements
                    .iter()
                    .map(|elem| Self::from_expr(&elem.inner))
                    .collect();
                QuasisyntaxTemplate::List(templates)
            }
            Expr::Pair { car, cdr } => {
                QuasisyntaxTemplate::ImproperList {
                    templates: vec![Self::from_expr(&car.inner)],
                    tail: Box::new(Self::from_expr(&cdr.inner)),
                }
            }
            Expr::Unquote(inner) => {
                QuasisyntaxTemplate::Unquote(Box::new(Self::from_expr(&inner.inner)))
            }
            Expr::UnquoteSplicing(inner) => {
                QuasisyntaxTemplate::UnquoteSplicing(Box::new(Self::from_expr(&inner.inner)))
            }
            Expr::Quasiquote(inner) => {
                QuasisyntaxTemplate::Nested {
                    template: Box::new(Self::from_expr(&inner.inner)),
                    depth: 1,
                }
            }
            _ => {
                // For other forms, represent as identifier with special encoding
                QuasisyntaxTemplate::Identifier(format!("<expr:{expr:?}>"))
            }
        }
    }

    /// Expands this quasisyntax template to a syntax object
    pub fn expand(
        &self,
        context: &mut QuasisyntaxContext,
        span: Span,
    ) -> Result<QuasisyntaxExpansionResult> {
        match self {
            QuasisyntaxTemplate::Literal(lit) => {
                let syntax = SyntaxObject::new(
                    Expr::Literal(lit.clone()),
                    span,
                    context.lexical_context.clone(),
                );
                Ok(QuasisyntaxExpansionResult::Single(Box::new(syntax)))
            }

            QuasisyntaxTemplate::Identifier(name) => {
                let syntax = SyntaxObject::new(
                    Expr::Identifier(name.clone()),
                    span,
                    context.lexical_context.clone(),
                );
                Ok(QuasisyntaxExpansionResult::Single(Box::new(syntax)))
            }

            QuasisyntaxTemplate::Nil => {
                let syntax = SyntaxObject::new(
                    Expr::Literal(Literal::Nil),
                    span,
                    context.lexical_context.clone(),
                );
                Ok(QuasisyntaxExpansionResult::Single(Box::new(syntax)))
            }

            QuasisyntaxTemplate::List(templates) => {
                self.expand_list(templates, context, span)
            }

            QuasisyntaxTemplate::ImproperList { templates, tail } => {
                self.expand_improper_list(templates, tail, context, span)
            }

            QuasisyntaxTemplate::Vector(templates) => {
                // For now, treat vectors like lists
                self.expand_list(templates, context, span)
            }

            QuasisyntaxTemplate::Unquote(inner) => {
                if context.depth() == 0 {
                    // At depth 0, actually perform the unquote
                    self.expand_unquote(inner, context, span)
                } else {
                    // At deeper levels, just decrease depth
                    let mut inner_context = context.clone();
                    inner_context.exit_level();
                    let result = inner.expand(&mut inner_context, span)?;
                    
                    // Wrap in unquote form
                    match result {
                        QuasisyntaxExpansionResult::Single(syntax) => {
                            let unquote_syntax = SyntaxObject::new(
                                Expr::Unquote(Box::new(syntax.to_spanned())),
                                span,
                                context.lexical_context.clone(),
                            );
                            Ok(QuasisyntaxExpansionResult::Single(Box::new(unquote_syntax)))
                        }
                        QuasisyntaxExpansionResult::Multiple(_) => {
                            Err(Box::new(Error::macro_error(
                                "Cannot unquote multiple values in this context".to_string(),
                                span,
                            )))
                        }
                    }
                }
            }

            QuasisyntaxTemplate::UnquoteSplicing(inner) => {
                if context.depth() == 0 {
                    // At depth 0, actually perform the unquote-splicing
                    self.expand_unquote_splicing(inner, context, span)
                } else {
                    // At deeper levels, just decrease depth
                    let mut inner_context = context.clone();
                    inner_context.exit_level();
                    let result = inner.expand(&mut inner_context, span)?;
                    
                    // Wrap in unquote-splicing form
                    match result {
                        QuasisyntaxExpansionResult::Single(syntax) => {
                            let unquote_syntax = SyntaxObject::new(
                                Expr::UnquoteSplicing(Box::new(syntax.to_spanned())),
                                span,
                                context.lexical_context.clone(),
                            );
                            Ok(QuasisyntaxExpansionResult::Single(Box::new(unquote_syntax)))
                        }
                        QuasisyntaxExpansionResult::Multiple(_) => {
                            Err(Box::new(Error::macro_error(
                                "Cannot unquote-splice multiple values in this context".to_string(),
                                span,
                            )))
                        }
                    }
                }
            }

            QuasisyntaxTemplate::Nested { template, depth } => {
                let mut nested_context = context.clone();
                for _ in 0..*depth {
                    nested_context.enter_level();
                }
                template.expand(&mut nested_context, span)
            }

            QuasisyntaxTemplate::PatternVariable(name) => {
                // Look up pattern variable in bindings
                if let Some(syntax) = context.bindings.get(name) {
                    Ok(QuasisyntaxExpansionResult::Single(Box::new(syntax.clone())))
                } else if let Some(syntaxes) = context.bindings.get_ellipsis(name) {
                    Ok(QuasisyntaxExpansionResult::Multiple(syntaxes.clone()))
                } else {
                    Err(Box::new(Error::macro_error(
                        format!("Unbound pattern variable: {name}"),
                        span,
                    )))
                }
            }

            QuasisyntaxTemplate::Escape(inner) => {
                // For now, just expand the inner template
                // TODO: Implement actual macro-time computation
                inner.expand(context, span)
            }

            QuasisyntaxTemplate::Conditional { condition, then_template, else_template } => {
                // Evaluate condition
                let condition_result = condition.expand(context, span)?;
                
                let use_then = match condition_result {
                    QuasisyntaxExpansionResult::Single(syntax) => {
                        !matches!(syntax.expr, Expr::Literal(Literal::Nil))
                    }
                    QuasisyntaxExpansionResult::Multiple(syntaxes) => {
                        !syntaxes.is_empty()
                    }
                };

                if use_then {
                    then_template.expand(context, span)
                } else if let Some(else_tmpl) = else_template {
                    else_tmpl.expand(context, span)
                } else {
                    let nil_syntax = SyntaxObject::new(
                        Expr::Literal(Literal::Nil),
                        span,
                        context.lexical_context.clone(),
                    );
                    Ok(QuasisyntaxExpansionResult::Single(Box::new(nil_syntax)))
                }
            }
        }
    }

    /// Expands a list template
    fn expand_list(
        &self,
        templates: &[QuasisyntaxTemplate],
        context: &mut QuasisyntaxContext,
        span: Span,
    ) -> Result<QuasisyntaxExpansionResult> {
        let mut elements = Vec::new();

        for template in templates {
            let result = template.expand(context, span)?;
            match result {
                QuasisyntaxExpansionResult::Single(syntax) => {
                    elements.push(syntax.to_spanned());
                }
                QuasisyntaxExpansionResult::Multiple(syntaxes) => {
                    // Splice in multiple values
                    elements.extend(syntaxes.into_iter().map(|s| s.to_spanned()));
                }
            }
        }

        let list_syntax = SyntaxObject::new(
            Expr::List(elements),
            span,
            context.lexical_context.clone(),
        );
        Ok(QuasisyntaxExpansionResult::Single(Box::new(list_syntax)))
    }

    /// Expands an improper list template
    fn expand_improper_list(
        &self,
        templates: &[QuasisyntaxTemplate],
        tail: &QuasisyntaxTemplate,
        context: &mut QuasisyntaxContext,
        span: Span,
    ) -> Result<QuasisyntaxExpansionResult> {
        let mut elements = Vec::new();

        // Expand fixed elements
        for template in templates {
            let result = template.expand(context, span)?;
            match result {
                QuasisyntaxExpansionResult::Single(syntax) => {
                    elements.push(syntax.to_spanned());
                }
                QuasisyntaxExpansionResult::Multiple(syntaxes) => {
                    elements.extend(syntaxes.into_iter().map(|s| s.to_spanned()));
                }
            }
        }

        // Expand tail
        let tail_result = tail.expand(context, span)?;
        let tail_syntax = match tail_result {
            QuasisyntaxExpansionResult::Single(syntax) => *syntax,
            QuasisyntaxExpansionResult::Multiple(_) => {
                return Err(Box::new(Error::macro_error(
                    "Cannot use multiple values as improper list tail".to_string(),
                    span,
                )));
            }
        };

        // Build improper list from right to left
        let mut result = tail_syntax;
        for element in elements.into_iter().rev() {
            let pair_expr = Expr::Pair {
                car: Box::new(element),
                cdr: Box::new(result.to_spanned()),
            };
            result = SyntaxObject::new(pair_expr, span, context.lexical_context.clone());
        }

        Ok(QuasisyntaxExpansionResult::Single(Box::new(result)))
    }

    /// Expands an unquote form
    fn expand_unquote(
        &self,
        inner: &QuasisyntaxTemplate,
        context: &mut QuasisyntaxContext,
        span: Span,
    ) -> Result<QuasisyntaxExpansionResult> {
        // For unquote, we need to evaluate the inner expression
        // For now, just expand it normally
        inner.expand(context, span)
    }

    /// Expands an unquote-splicing form
    fn expand_unquote_splicing(
        &self,
        inner: &QuasisyntaxTemplate,
        context: &mut QuasisyntaxContext,
        span: Span,
    ) -> Result<QuasisyntaxExpansionResult> {
        let result = inner.expand(context, span)?;
        
        match result {
            QuasisyntaxExpansionResult::Single(syntax) => {
                // Convert single value to list for splicing
                match &syntax.expr {
                    Expr::List(elements) => {
                        let syntaxes: Vec<_> = elements
                            .iter()
                            .map(|elem| SyntaxObject::from_spanned(
                                elem.clone(),
                                context.lexical_context.clone(),
                            ))
                            .collect();
                        Ok(QuasisyntaxExpansionResult::Multiple(syntaxes))
                    }
                    _ => {
                        // Non-list values become single-element splicing
                        Ok(QuasisyntaxExpansionResult::Multiple(vec![*syntax]))
                    }
                }
            }
            QuasisyntaxExpansionResult::Multiple(syntaxes) => {
                // Already multiple values
                Ok(QuasisyntaxExpansionResult::Multiple(syntaxes))
            }
        }
    }
}

/// Result of quasisyntax expansion
#[derive(Debug, Clone)]
pub enum QuasisyntaxExpansionResult {
    /// Single syntax object
    Single(Box<SyntaxObject>),
    /// Multiple syntax objects (from splicing)
    Multiple(Vec<SyntaxObject>),
}

impl QuasisyntaxExpansionResult {
    /// Converts to a single syntax object (fails if multiple)
    pub fn into_single(self) -> Result<SyntaxObject> {
        match self {
            QuasisyntaxExpansionResult::Single(syntax) => Ok(*syntax),
            QuasisyntaxExpansionResult::Multiple(_) => {
                Err(Box::new(Error::macro_error(
                    "Expected single value, got multiple".to_string(),
                    Span::new(0, 0),
                )))
            }
        }
    }

    /// Converts to multiple syntax objects
    pub fn into_multiple(self) -> Vec<SyntaxObject> {
        match self {
            QuasisyntaxExpansionResult::Single(syntax) => vec![*syntax],
            QuasisyntaxExpansionResult::Multiple(syntaxes) => syntaxes,
        }
    }
}

/// Parser for quasisyntax forms
pub struct QuasisyntaxParser {
    /// Current nesting depth
    depth: usize,
}

impl QuasisyntaxParser {
    /// Creates a new parser
    pub fn new() -> Self {
        Self { depth: 0 }
    }

    /// Parses an expression into a quasisyntax template
    pub fn parse(&mut self, expr: &Expr) -> Result<QuasisyntaxTemplate> {
        match expr {
            Expr::Literal(lit) => Ok(QuasisyntaxTemplate::Literal(lit.clone())),
            
            Expr::Identifier(name) | Expr::Symbol(name) => {
                Ok(QuasisyntaxTemplate::Identifier(name.clone()))
            }

            Expr::List(elements) => {
                if elements.is_empty() {
                    return Ok(QuasisyntaxTemplate::Nil);
                }

                // Check for special quasisyntax forms
                if let Some(first) = elements.first() {
                    if let Expr::Identifier(name) = &first.inner {
                        match name.as_str() {
                            "unsyntax" | "unquote" => {
                                if elements.len() != 2 {
                                    return Err(Box::new(Error::macro_error(
                                        "unsyntax requires exactly one argument".to_string(),
                                        first.span,
                                    )));
                                }
                                let inner = self.parse(&elements[1].inner)?;
                                return Ok(QuasisyntaxTemplate::Unquote(Box::new(inner)));
                            }
                            "unsyntax-splicing" | "unquote-splicing" => {
                                if elements.len() != 2 {
                                    return Err(Box::new(Error::macro_error(
                                        "unsyntax-splicing requires exactly one argument".to_string(),
                                        first.span,
                                    )));
                                }
                                let inner = self.parse(&elements[1].inner)?;
                                return Ok(QuasisyntaxTemplate::UnquoteSplicing(Box::new(inner)));
                            }
                            "quasisyntax" => {
                                if elements.len() != 2 {
                                    return Err(Box::new(Error::macro_error(
                                        "quasisyntax requires exactly one argument".to_string(),
                                        first.span,
                                    )));
                                }
                                self.depth += 1;
                                let inner = self.parse(&elements[1].inner)?;
                                self.depth -= 1;
                                return Ok(QuasisyntaxTemplate::Nested {
                                    template: Box::new(inner),
                                    depth: 1,
                                });
                            }
                            _ => {}
                        }
                    }
                }

                // Regular list
                let mut templates = Vec::new();
                for element in elements {
                    templates.push(self.parse(&element.inner)?);
                }
                Ok(QuasisyntaxTemplate::List(templates))
            }

            Expr::Pair { car, cdr } => {
                let car_template = self.parse(&car.inner)?;
                let cdr_template = self.parse(&cdr.inner)?;
                Ok(QuasisyntaxTemplate::ImproperList {
                    templates: vec![car_template],
                    tail: Box::new(cdr_template),
                })
            }

            Expr::Unquote(inner) => {
                let inner_template = self.parse(&inner.inner)?;
                Ok(QuasisyntaxTemplate::Unquote(Box::new(inner_template)))
            }

            Expr::UnquoteSplicing(inner) => {
                let inner_template = self.parse(&inner.inner)?;
                Ok(QuasisyntaxTemplate::UnquoteSplicing(Box::new(inner_template)))
            }

            Expr::Quasiquote(inner) => {
                self.depth += 1;
                let inner_template = self.parse(&inner.inner)?;
                self.depth -= 1;
                Ok(QuasisyntaxTemplate::Nested {
                    template: Box::new(inner_template),
                    depth: 1,
                })
            }

            _ => {
                // For other expressions, create an escape template
                let inner_template = QuasisyntaxTemplate::Identifier(format!("<expr:{expr:?}>"));
                Ok(QuasisyntaxTemplate::Escape(Box::new(inner_template)))
            }
        }
    }
}

impl Default for QuasisyntaxParser {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level interface for quasisyntax operations
pub mod quasisyntax_interface {
    use super::*;

    /// Creates a syntax object using the #' (syntax) form
    pub fn syntax(
        expr: &Expr,
        context: &LexicalContext,
        span: Span,
    ) -> Result<SyntaxObject> {
        // Direct syntax construction - no template expansion
        Ok(SyntaxObject::new(expr.clone(), span, context.clone()))
    }

    /// Creates a syntax object using the #` (quasisyntax) form
    pub fn quasisyntax(
        expr: &Expr,
        bindings: SyntaxBindings,
        context: &LexicalContext,
        span: Span,
    ) -> Result<SyntaxObject> {
        let mut parser = QuasisyntaxParser::new();
        let template = parser.parse(expr)?;
        
        let mut expansion_context = QuasisyntaxContext::new(bindings, context.clone());
        let result = template.expand(&mut expansion_context, span)?;
        
        result.into_single()
    }

    /// Processes an unquote (unsyntax) form
    pub fn unsyntax(
        expr: &Expr,
        bindings: &SyntaxBindings,
        context: &LexicalContext,
        span: Span,
    ) -> Result<SyntaxObject> {
        // For unquote, we evaluate the expression in the current context
        // This is a simplified implementation
        Ok(SyntaxObject::new(expr.clone(), span, context.clone()))
    }

    /// Processes an unquote-splicing (unsyntax-splicing) form
    pub fn unsyntax_splicing(
        expr: &Expr,
        bindings: &SyntaxBindings,
        context: &LexicalContext,
        span: Span,
    ) -> Result<Vec<SyntaxObject>> {
        // For unquote-splicing, we evaluate and expect a list
        match expr {
            Expr::List(elements) => {
                Ok(elements
                    .iter()
                    .map(|elem| SyntaxObject::from_spanned(elem.clone(), context.clone()))
                    .collect())
            }
            _ => {
                // Non-list becomes single element
                Ok(vec![SyntaxObject::new(expr.clone(), span, context.clone())])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::diagnostics::Span;

    #[test]
    fn test_quasisyntax_parser() {
        let mut parser = QuasisyntaxParser::new();
        
        // Test literal
        let expr = Expr::Literal(Literal::Number(42.0));
        let template = parser.parse(&expr).unwrap();
        assert!(matches!(template, QuasisyntaxTemplate::Literal(Literal::Number(n)) if n == 42.0));
    }

    #[test]
    fn test_quasisyntax_expansion() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let mut bindings = SyntaxBindings::new();
        
        // Add a binding
        let bound_syntax = SyntaxObject::new(
            Expr::Identifier("foo".to_string()),
            Span::new(0, 3),
            context.clone(),
        );
        bindings.bind("x".to_string(), bound_syntax);

        // Create template
        let template = QuasisyntaxTemplate::List(vec![
            QuasisyntaxTemplate::Identifier("lambda".to_string()),
            QuasisyntaxTemplate::List(vec![QuasisyntaxTemplate::PatternVariable("x".to_string())]),
            QuasisyntaxTemplate::PatternVariable("x".to_string()),
        ]);

        // Expand
        let mut expansion_context = QuasisyntaxContext::new(bindings, context);
        let result = template.expand(&mut expansion_context, Span::new(0, 10)).unwrap();

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

        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let bindings = SyntaxBindings::new();
        let mut expansion_context = QuasisyntaxContext::new(bindings, context);

        let result = unquote_template.expand(&mut expansion_context, Span::new(0, 4)).unwrap();
        
        match result {
            QuasisyntaxExpansionResult::Single(syntax) => {
                assert_eq!(syntax.identifier_name(), Some("test"));
            }
            _ => panic!("Expected single result"),
        }
    }

    #[test]
    fn test_syntax_interface() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let expr = Expr::Identifier("test".to_string());
        let span = Span::new(0, 4);

        let syntax = quasisyntax_interface::syntax(&expr, &context, span).unwrap();
        
        assert_eq!(syntax.expr, expr);
        assert_eq!(syntax.span, span);
        assert_eq!(syntax.context.context_id, context.context_id);
    }

    #[test]
    fn test_quasisyntax_interface() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let mut bindings = SyntaxBindings::new();
        
        // Simple expression without unquotes
        let expr = Expr::List(vec![
            Spanned::new(Expr::Identifier("lambda".to_string()), Span::new(1, 7)),
            Spanned::new(Expr::List(vec![]), Span::new(8, 10)),
            Spanned::new(Expr::Identifier("body".to_string()), Span::new(11, 15)),
        ]);
        
        let result = quasisyntax_interface::quasisyntax(&expr, bindings, &context, Span::new(0, 16));
        assert!(result.is_ok());
    }
}