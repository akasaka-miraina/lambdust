//! Integration layer between syntax objects and existing macro system
//!
//! This module provides the integration layer that bridges the new syntax object
//! system with the existing macro infrastructure. It ensures backward compatibility
//! while enabling advanced features like hygiene and syntax-case.

use super::{
    syntax_objects::{SyntaxObject, LexicalContext, BindingInfo, syntax_utils},
    advanced_hygiene::{HygieneResolver, Mark, MarkSet, fresh_mark},
    syntax_case::{SyntaxPattern, SyntaxTemplate, SyntaxBindings, syntax_procedures},
    quasisyntax::{QuasisyntaxTemplate, QuasisyntaxContext, quasisyntax_interface},
    pattern::{Pattern, PatternBindings},
    template::Template,
    expander::ConfigurableExpander,
    environment::MacroEnvironment,
    hygiene::{HygieneContext, MacroContext},
};
use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::Environment;
use std::collections::HashMap;
use std::rc::Rc;

/// Enhanced macro expander that integrates syntax objects with legacy infrastructure
pub struct SyntaxAwareMacroExpander {
    /// Legacy macro expander for compatibility
    legacy_expander: Box<ConfigurableExpander>,
    /// Advanced hygiene resolver
    hygiene_resolver: HygieneResolver,
    /// Current lexical context
    current_context: Option<LexicalContext>,
    /// Context stack for nested expansions
    context_stack: Vec<LexicalContext>,
    /// Mark stack for hygiene tracking
    mark_stack: Vec<Mark>,
    /// Syntax object cache for performance
    syntax_cache: HashMap<String, SyntaxObject>,
    /// Integration statistics
    stats: IntegrationStats,
}

impl std::fmt::Debug for SyntaxAwareMacroExpander {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyntaxAwareMacroExpander")
            .field("legacy_expander", &"<ConfigurableExpander>")
            .field("hygiene_resolver", &self.hygiene_resolver)
            .field("current_context", &self.current_context)
            .field("context_stack", &self.context_stack)
            .field("mark_stack", &self.mark_stack)
            .field("syntax_cache", &self.syntax_cache)
            .field("stats", &self.stats)
            .finish()
    }
}

/// Statistics for syntax object integration
#[derive(Debug, Clone, Default)]
pub struct IntegrationStats {
    /// Number of legacy->syntax conversions
    pub legacy_to_syntax_conversions: usize,
    /// Number of syntax->legacy conversions
    pub syntax_to_legacy_conversions: usize,
    /// Number of hygiene applications
    pub hygiene_applications: usize,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
}

impl SyntaxAwareMacroExpander {
    /// Creates a new syntax-aware macro expander
    pub fn new() -> Self {
        Self {
            legacy_expander: Box::new(ConfigurableExpander::new()),
            hygiene_resolver: HygieneResolver::new(),
            current_context: None,
            context_stack: Vec::new(),
            mark_stack: Vec::new(),
            syntax_cache: HashMap::new(),
            stats: IntegrationStats::default(),
        }
    }

    /// Sets the current lexical context
    pub fn set_context(&mut self, context: LexicalContext) {
        self.current_context = Some(context.clone());
        self.hygiene_resolver.set_context(context);
    }

    /// Enters a new macro expansion context
    pub fn enter_macro_expansion(&mut self, module_path: Vec<String>) -> Result<LexicalContext> {
        let context_id = self.legacy_expander.hygiene_context.enter_scope();
        let context = if let Some(parent) = &self.current_context {
            parent.child(context_id)
        } else {
            LexicalContext::new(context_id, module_path)
        };

        self.context_stack.push(context.clone());
        self.current_context = Some(context.clone());

        let mark = self.hygiene_resolver.enter_scope();
        self.mark_stack.push(mark);

        Ok(context)
    }

    /// Exits the current macro expansion context
    pub fn exit_macro_expansion(&mut self) {
        if let Some(context) = self.context_stack.pop() {
            self.current_context = self.context_stack.last().cloned();
        }

        if !self.mark_stack.is_empty() {
            self.mark_stack.pop();
            self.hygiene_resolver.exit_scope();
        }

        self.legacy_expander.hygiene_context.exit_scope(0);
    }

    /// Converts a legacy expression to a syntax object
    pub fn expr_to_syntax(
        &mut self,
        expr: Spanned<Expr>,
        template_context: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        self.stats.legacy_to_syntax_conversions += 1;

        let context = if let Some(template) = template_context {
            template.context.clone()
        } else if let Some(current) = &self.current_context {
            current.clone()
        } else {
            LexicalContext::new(0, vec!["top-level".to_string()])
        };

        let mut syntax = SyntaxObject::from_spanned(expr, context);

        // Apply current marks for hygiene
        let current_marks: MarkSet = self.mark_stack.iter().cloned().collect();
        syntax = syntax.with_marks(&(&current_marks).into());

        // Apply hygiene transformation
        if !self.mark_stack.is_empty() {
            syntax = self.hygiene_resolver.apply_hygiene(&syntax)?;
            self.stats.hygiene_applications += 1;
        }

        Ok(syntax)
    }

    /// Converts a syntax object to a legacy expression
    pub fn syntax_to_expr(&mut self, syntax: &SyntaxObject) -> Result<Spanned<Expr>> {
        self.stats.syntax_to_legacy_conversions += 1;

        // Apply reverse hygiene transformation if needed
        let transformed_syntax = if !self.mark_stack.is_empty() {
            self.hygiene_resolver.apply_hygiene(syntax)?
        } else {
            syntax.clone()
        };

        Ok(transformed_syntax.to_spanned())
    }

    /// Expands a macro using syntax-case if available, fallback to legacy
    pub fn expand_macro_with_syntax(
        &mut self,
        macro_name: &str,
        args: &[SyntaxObject],
        span: Span,
    ) -> Result<SyntaxObject> {
        // Try to find a syntax-case transformer first
        if let Some(syntax_transformer) = self.find_syntax_case_transformer(macro_name) {
            self.expand_with_syntax_case(&syntax_transformer, args, span)
        } else {
            // Fallback to legacy expansion
            self.expand_with_legacy(macro_name, args, span)
        }
    }

    /// Expands using syntax-case transformers
    fn expand_with_syntax_case(
        &mut self,
        transformer: &SyntaxCaseTransformer,
        args: &[SyntaxObject],
        span: Span,
    ) -> Result<SyntaxObject> {
        // Try each clause in the transformer
        for clause in &transformer.clauses {
            let input_syntax = if args.len() == 1 {
                args[0].clone()
            } else {
                // Create a list syntax object from multiple args
                let context = self.current_context.clone()
                    .unwrap_or_else(|| LexicalContext::new(0, vec!["top-level".to_string()]));
                let elements: Vec<_> = args.iter().map(|s| s.to_spanned()).collect();
                SyntaxObject::new(Expr::List(elements), span, context)
            };

            if let Ok(bindings) = clause.0.match_syntax(&input_syntax) {
                let context = self.current_context.as_ref().unwrap_or(&input_syntax.context);
                return clause.1.expand(&bindings, context, span);
            }
        }

        Err(Box::new(Error::macro_error(
            format!("No syntax-case clause matched for macro: {}", transformer.name),
            span,
        )))
    }

    /// Expands using legacy transformers with syntax object conversion
    fn expand_with_legacy(
        &mut self,
        macro_name: &str,
        args: &[SyntaxObject],
        span: Span,
    ) -> Result<SyntaxObject> {
        // Convert syntax objects to legacy expressions
        let legacy_args: Result<Vec<_>> = args
            .iter()
            .map(|s| self.syntax_to_expr(s))
            .collect();
        let legacy_args = legacy_args?;

        // Use legacy expander
        // This is simplified - would need integration with actual legacy transformer
        let result_expr = Expr::List(legacy_args);
        let result_spanned = Spanned::new(result_expr, span);

        // Convert result back to syntax object
        self.expr_to_syntax(result_spanned, None)
    }

    /// Finds a syntax-case transformer by name
    fn find_syntax_case_transformer(&self, _name: &str) -> Option<SyntaxCaseTransformer> {
        // TODO: Implement transformer registry
        None
    }

    /// Applies hygiene to an expression during macro expansion
    pub fn apply_hygiene_to_expr(
        &mut self,
        expr: &Expr,
        definition_env: &Environment,
    ) -> Result<Expr> {
        // Convert to syntax object
        let spanned = Spanned::new(expr.clone(), Span::new(0, 0));
        let syntax = self.expr_to_syntax(spanned, None)?;

        // Apply hygiene
        let hygiene_syntax = self.hygiene_resolver.apply_hygiene(&syntax)?;

        // Convert back
        let result_spanned = self.syntax_to_expr(&hygiene_syntax)?;
        Ok(result_spanned.inner)
    }

    /// Creates a syntax object with proper context and hygiene
    pub fn make_syntax_object(
        &self,
        expr: Expr,
        span: Span,
        template_syntax: Option<&SyntaxObject>,
    ) -> SyntaxObject {
        let context = if let Some(template) = template_syntax {
            template.context.clone()
        } else if let Some(current) = &self.current_context {
            current.clone()
        } else {
            LexicalContext::new(0, vec!["top-level".to_string()])
        };

        let mut syntax = SyntaxObject::new(expr, span, context);

        // Add current marks
        for mark in &self.mark_stack {
            syntax.add_mark(mark.value());
        }

        syntax
    }

    /// Processes a syntax-case macro definition
    pub fn define_syntax_case_macro(
        &mut self,
        name: String,
        literal_identifiers: Vec<String>,
        clauses: Vec<(SyntaxPattern, SyntaxTemplate)>,
    ) -> Result<()> {
        let transformer = SyntaxCaseTransformer {
            name: name.clone(),
            literal_identifiers,
            clauses,
        };

        // TODO: Store in transformer registry
        Ok(())
    }

    /// Processes datum->syntax conversion with proper context
    pub fn datum_to_syntax_with_context(
        &self,
        datum: Expr,
        template_identifier: Option<&SyntaxObject>,
        span: Span,
    ) -> SyntaxObject {
        syntax_procedures::datum_to_syntax(datum, template_identifier, None)
    }

    /// Processes syntax->datum conversion
    pub fn syntax_to_datum(&self, syntax: &SyntaxObject) -> Expr {
        syntax_procedures::syntax_to_datum(syntax)
    }

    /// Gets integration statistics
    pub fn stats(&self) -> &IntegrationStats {
        &self.stats
    }

    /// Resets statistics
    pub fn reset_stats(&mut self) {
        self.stats = IntegrationStats::default();
    }

    /// Clears all caches
    pub fn clear_caches(&mut self) {
        self.syntax_cache.clear();
        self.hygiene_resolver.clear();
    }
}

impl Default for SyntaxAwareMacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

/// A syntax-case transformer
#[derive(Debug, Clone)]
pub struct SyntaxCaseTransformer {
    /// Name of the transformer
    pub name: String,
    /// Literal identifiers that are not pattern variables
    pub literal_identifiers: Vec<String>,
    /// Pattern-template clauses
    pub clauses: Vec<(SyntaxPattern, SyntaxTemplate)>,
}

/// Bridge functions for legacy compatibility
pub mod legacy_bridge {
    use super::*;

    /// Converts legacy Pattern to SyntaxPattern
    pub fn pattern_to_syntax_pattern(pattern: &Pattern) -> Result<SyntaxPattern> {
        match pattern {
            Pattern::Variable(name) => Ok(SyntaxPattern::PatternVariable(name.clone())),
            Pattern::Literal(lit) => Ok(SyntaxPattern::Literal(lit.clone())),
            Pattern::Identifier(name) => Ok(SyntaxPattern::Identifier {
                name: name.clone(),
                binding_level: None,
            }),
            Pattern::Nil => Ok(SyntaxPattern::Nil),
            Pattern::List(patterns) => {
                let syntax_patterns: Result<Vec<_>> = patterns
                    .iter()
                    .map(pattern_to_syntax_pattern)
                    .collect();
                Ok(SyntaxPattern::List(syntax_patterns?))
            }
            Pattern::Pair { car, cdr } => {
                Ok(SyntaxPattern::ImproperList {
                    patterns: vec![pattern_to_syntax_pattern(car)?],
                    tail: Box::new(pattern_to_syntax_pattern(cdr)?),
                })
            }
            Pattern::Wildcard => Ok(SyntaxPattern::PatternVariable("_".to_string())),
            _ => Err(Box::new(Error::macro_error(
                "Unsupported legacy pattern type".to_string(),
                Span::new(0, 0),
            ))),
        }
    }

    /// Converts legacy Template to SyntaxTemplate
    pub fn template_to_syntax_template(template: &Template) -> Result<SyntaxTemplate> {
        match template {
            Template::Variable(name) => Ok(SyntaxTemplate::PatternVariable(name.clone())),
            Template::Literal(lit) => Ok(SyntaxTemplate::Literal(lit.clone())),
            Template::Identifier(name) => Ok(SyntaxTemplate::Identifier(name.clone())),
            Template::Nil => Ok(SyntaxTemplate::Nil),
            Template::List(templates) => {
                let syntax_templates: Result<Vec<_>> = templates
                    .iter()
                    .map(template_to_syntax_template)
                    .collect();
                Ok(SyntaxTemplate::List(syntax_templates?))
            }
            Template::Pair { car, cdr } => {
                Ok(SyntaxTemplate::ImproperList {
                    templates: vec![template_to_syntax_template(car)?],
                    tail: Box::new(template_to_syntax_template(cdr)?),
                })
            }
            _ => Err(Box::new(Error::macro_error(
                "Unsupported legacy template type".to_string(),
                Span::new(0, 0),
            ))),
        }
    }

    /// Converts legacy PatternBindings to SyntaxBindings
    pub fn pattern_bindings_to_syntax_bindings(
        bindings: &PatternBindings,
        context: &LexicalContext,
    ) -> SyntaxBindings {
        let mut syntax_bindings = SyntaxBindings::new();

        for (name, expr) in bindings.bindings() {
            let syntax = SyntaxObject::from_spanned(expr.clone(), context.clone());
            syntax_bindings.bind(name.clone(), syntax);
        }

        for (name, exprs) in bindings.ellipsis_bindings() {
            let syntaxes: Vec<_> = exprs
                .iter()
                .map(|expr| SyntaxObject::from_spanned(expr.clone(), context.clone()))
                .collect();
            syntax_bindings.bind_ellipsis(name.clone(), syntaxes);
        }

        syntax_bindings
    }
}

/// Utilities for syntax object system integration
pub mod integration_utils {
    use super::*;

    /// Creates a properly hygienic macro expander for the given environment
    pub fn create_hygienic_expander(env: &Environment) -> SyntaxAwareMacroExpander {
        let mut expander = SyntaxAwareMacroExpander::new();
        
        // Set up initial context based on environment
        let context = LexicalContext::new(
            0,
            vec!["user".to_string()], // Default module path
        );
        expander.set_context(context);
        
        expander
    }

    /// Processes a macro definition and integrates it with the syntax system
    pub fn define_integrated_macro(
        expander: &mut SyntaxAwareMacroExpander,
        name: String,
        transformer_expr: &Expr,
        env: &Environment,
    ) -> Result<()> {
        // This would parse the transformer expression and create appropriate
        // syntax-case or legacy transformers
        if let Expr::Application { operator, operands } = transformer_expr {
            if let Expr::Identifier(op_name) = &operator.inner {
                if op_name == "syntax-rules" {
                    // Parse syntax-rules form
                    return parse_syntax_rules_definition(expander, name, operands);
                }
            }
        }

        // Fallback to legacy handling
        Ok(())
    }

    /// Parses a syntax-rules definition
    fn parse_syntax_rules_definition(
        expander: &mut SyntaxAwareMacroExpander,
        name: String,
        operands: &[Spanned<Expr>],
    ) -> Result<()> {
        if operands.len() < 2 {
            return Err(Box::new(Error::macro_error(
                "syntax-rules requires at least literals and one rule".to_string(),
                Span::new(0, 0),
            )));
        }

        // Parse literals
        let literals = match &operands[0].inner {
            Expr::List(lit_exprs) => {
                let mut literals = Vec::new();
                for lit_expr in lit_exprs {
                    if let Expr::Identifier(name) = &lit_expr.inner {
                        literals.push(name.clone());
                    }
                }
                literals
            }
            _ => Vec::new(),
        };

        // Parse rules (simplified)
        let mut clauses = Vec::new();
        for rule_expr in &operands[1..] {
            if let Expr::List(rule_parts) = &rule_expr.inner {
                if rule_parts.len() == 2 {
                    // Convert pattern and template
                    // This is a simplified version - full implementation would be more complex
                    let pattern = SyntaxPattern::PatternVariable("_".to_string());
                    let template = SyntaxTemplate::Identifier("result".to_string());
                    clauses.push((pattern, template));
                }
            }
        }

        expander.define_syntax_case_macro(name, literals, clauses)
    }

    /// Applies comprehensive hygiene checking to an expression
    pub fn check_hygiene_correctness(
        expr: &Expr,
        expander: &mut SyntaxAwareMacroExpander,
    ) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Convert to syntax object for analysis
        let spanned = Spanned::new(expr.clone(), Span::new(0, 0));
        let syntax = expander.expr_to_syntax(spanned, None)?;

        // Check for potential hygiene violations
        check_identifier_capture(&syntax, &mut warnings);
        check_binding_consistency(&syntax, &mut warnings);

        Ok(warnings)
    }

    /// Checks for potential identifier capture
    fn check_identifier_capture(syntax: &SyntaxObject, warnings: &mut Vec<String>) {
        // Simplified implementation - would need more sophisticated analysis
        if syntax.is_identifier() {
            if let Some(name) = syntax.identifier_name() {
                if name.contains("#") && !name.starts_with("hyg") {
                    warnings.push(format!("Potential identifier capture: {name}"));
                }
            }
        }
    }

    /// Checks for binding consistency
    fn check_binding_consistency(_syntax: &SyntaxObject, _warnings: &mut [String]) {
        // Placeholder for binding consistency checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::diagnostics::Span;

    #[test]
    fn test_syntax_aware_expander_creation() {
        let expander = SyntaxAwareMacroExpander::new();
        assert_eq!(expander.stats().legacy_to_syntax_conversions, 0);
    }

    #[test]
    fn test_expr_to_syntax_conversion() {
        let mut expander = SyntaxAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        expander.set_context(context);

        let expr = Spanned::new(
            Expr::Identifier("test".to_string()),
            Span::new(0, 4),
        );

        let syntax = expander.expr_to_syntax(expr, None).unwrap();
        assert_eq!(syntax.identifier_name(), Some("test"));
        assert_eq!(expander.stats().legacy_to_syntax_conversions, 1);
    }

    #[test]
    fn test_syntax_to_expr_conversion() {
        let mut expander = SyntaxAwareMacroExpander::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        
        let syntax = SyntaxObject::new(
            Expr::Identifier("test".to_string()),
            Span::new(0, 4),
            context,
        );

        let expr = expander.syntax_to_expr(&syntax).unwrap();
        assert_eq!(expr.inner, Expr::Identifier("test".to_string()));
        assert_eq!(expander.stats().syntax_to_legacy_conversions, 1);
    }

    #[test]
    fn test_macro_expansion_context() {
        let mut expander = SyntaxAwareMacroExpander::new();
        
        let context = expander.enter_macro_expansion(vec!["test".to_string()]).unwrap();
        assert_eq!(context.module_path, vec!["test".to_string()]);
        
        expander.exit_macro_expansion();
        assert!(expander.current_context.is_none());
    }

    #[test]
    fn test_legacy_pattern_conversion() {
        let legacy_pattern = Pattern::Variable("x".to_string());
        let syntax_pattern = legacy_bridge::pattern_to_syntax_pattern(&legacy_pattern).unwrap();
        
        assert!(matches!(syntax_pattern, SyntaxPattern::PatternVariable(name) if name == "x"));
    }

    #[test]
    fn test_integration_stats() {
        let mut expander = SyntaxAwareMacroExpander::new();
        assert_eq!(expander.stats().legacy_to_syntax_conversions, 0);
        
        // Perform some operations
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        expander.set_context(context);
        
        let expr = Spanned::new(Expr::Identifier("test".to_string()), Span::new(0, 4));
        let _syntax = expander.expr_to_syntax(expr, None).unwrap();
        
        assert_eq!(expander.stats().legacy_to_syntax_conversions, 1);
        
        expander.reset_stats();
        assert_eq!(expander.stats().legacy_to_syntax_conversions, 0);
    }
}