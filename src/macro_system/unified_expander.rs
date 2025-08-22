//! Unified macro expander that bridges legacy and syntax object systems
//!
//! This module provides a unified interface for macro expansion that can work
//! with both the legacy AST-based system and the new syntax object system.
//! It automatically chooses the appropriate expansion method based on the
//! transformer type and configuration.

use super::{
    ConfigurableExpander, ExpansionConfig, SyntaxAwareMacroExpander,
    advanced_hygiene::{HygieneResolver, MarkSet},
    identifier_transformers::{
        IdentifierContext, VariableTransformer, VariableTransformerRegistry,
    },
    pattern::{Pattern, PatternBindings},
    scope_management::{ScopeManager, ScopeType},
    syntax_case::{SyntaxBindings, SyntaxPattern, SyntaxTemplate, syntax_procedures},
    syntax_objects::{LexicalContext, SyntaxObject, syntax_utils},
    template::Template,
};
use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::Environment;
use std::collections::HashMap;

/// Type of macro transformer
#[derive(Debug, Clone)]
pub enum MacroTransformerType {
    /// Legacy pattern-template transformer
    Legacy {
        /// Pattern matching patterns
        patterns: Vec<Pattern>,
        /// Output templates
        templates: Vec<Template>,
    },
    /// Syntax-case transformer
    SyntaxCase {
        /// List of literal identifiers that match only themselves
        literals: Vec<String>,
        /// Pattern-template clause pairs
        clauses: Vec<(SyntaxPattern, SyntaxTemplate)>,
    },
    /// Procedure-based transformer
    Procedure {
        /// Name of the transformer procedure
        name: String,
        /// Expected arity (None = variadic)
        arity: Option<usize>,
    },
    /// Variable transformer (identifier transformer)
    VariableTransformer {
        /// The variable transformer implementation
        transformer: VariableTransformer,
    },
}

impl PartialEq for MacroTransformerType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                MacroTransformerType::Legacy {
                    patterns: p1,
                    templates: t1,
                },
                MacroTransformerType::Legacy {
                    patterns: p2,
                    templates: t2,
                },
            ) => p1 == p2 && t1 == t2,
            (
                MacroTransformerType::SyntaxCase {
                    literals: l1,
                    clauses: c1,
                },
                MacroTransformerType::SyntaxCase {
                    literals: l2,
                    clauses: c2,
                },
            ) => l1 == l2 && c1 == c2,
            (
                MacroTransformerType::Procedure {
                    name: n1,
                    arity: a1,
                },
                MacroTransformerType::Procedure {
                    name: n2,
                    arity: a2,
                },
            ) => n1 == n2 && a1 == a2,
            (
                MacroTransformerType::VariableTransformer { transformer: _ },
                MacroTransformerType::VariableTransformer { transformer: _ },
            ) => {
                // Note: We can't compare VariableTransformers, so we consider them equal if they're both VariableTransformers
                true
            }
            _ => false,
        }
    }
}

/// A unified macro transformer that can handle both legacy and syntax object systems
#[derive(Debug, Clone)]
pub struct UnifiedMacroTransformer {
    /// Name of the macro
    pub name: String,
    /// Type of transformer
    pub transformer_type: MacroTransformerType,
    /// Whether to use syntax objects
    pub use_syntax_objects: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl UnifiedMacroTransformer {
    /// Creates a new unified transformer
    pub fn new(
        name: String,
        transformer_type: MacroTransformerType,
        use_syntax_objects: bool,
    ) -> Self {
        Self {
            name,
            transformer_type,
            use_syntax_objects,
            metadata: HashMap::new(),
        }
    }

    /// Creates a legacy transformer
    pub fn legacy(name: String, patterns: Vec<Pattern>, templates: Vec<Template>) -> Self {
        Self::new(
            name,
            MacroTransformerType::Legacy {
                patterns,
                templates,
            },
            false,
        )
    }

    /// Creates a syntax-case transformer
    pub fn syntax_case(
        name: String,
        literals: Vec<String>,
        clauses: Vec<(SyntaxPattern, SyntaxTemplate)>,
    ) -> Self {
        Self::new(
            name,
            MacroTransformerType::SyntaxCase { literals, clauses },
            true,
        )
    }

    /// Creates a procedure-based transformer
    pub fn procedure(name: String, arity: Option<usize>) -> Self {
        let procedure_name = name.clone();
        Self::new(
            name,
            MacroTransformerType::Procedure {
                name: procedure_name,
                arity,
            },
            true, // Procedures typically work better with syntax objects
        )
    }

    /// Creates a variable transformer
    pub fn variable_transformer(name: String, transformer: VariableTransformer) -> Self {
        Self::new(
            name,
            MacroTransformerType::VariableTransformer { transformer },
            true, // Variable transformers require syntax objects
        )
    }
}

/// Unified macro expander that can work with both systems
#[derive(Debug)]
pub struct UnifiedMacroExpander {
    /// Legacy expander
    legacy_expander: ConfigurableExpander,
    /// Syntax-aware expander
    syntax_expander: SyntaxAwareMacroExpander,
    // Note: Removed context_aware_expander to break circular dependency
    /// Scope manager for context tracking
    scope_manager: ScopeManager,
    /// Registered transformers
    transformers: HashMap<String, UnifiedMacroTransformer>,
    /// Variable transformer registry
    variable_transformers: VariableTransformerRegistry,
    /// Current expansion mode
    mode: ExpansionMode,
    /// Expansion statistics
    stats: UnifiedExpansionStats,
}

/// Expansion mode for the unified expander
#[derive(Debug, Clone, PartialEq)]
pub enum ExpansionMode {
    /// Use legacy expander only
    Legacy,
    /// Use syntax object expander only
    SyntaxObjects,
    /// Automatically choose based on transformer
    Automatic,
    /// Hybrid mode - try syntax objects first, fallback to legacy
    Hybrid,
}

/// Statistics for unified expansion
#[derive(Debug, Clone, Default)]
pub struct UnifiedExpansionStats {
    /// Number of legacy expansions
    pub legacy_expansions: usize,
    /// Number of syntax object expansions
    pub syntax_object_expansions: usize,
    /// Number of automatic mode decisions
    pub automatic_decisions: usize,
    /// Number of fallbacks from syntax objects to legacy
    pub fallbacks: usize,
}

impl UnifiedMacroExpander {
    /// Creates a new unified macro expander
    pub fn new() -> Self {
        Self {
            legacy_expander: ConfigurableExpander::new(),
            syntax_expander: SyntaxAwareMacroExpander::new(),
            scope_manager: ScopeManager::new(),
            transformers: HashMap::new(),
            variable_transformers: VariableTransformerRegistry::new(),
            mode: ExpansionMode::Automatic,
            stats: UnifiedExpansionStats::default(),
        }
    }

    /// Creates a unified expander with configuration
    pub fn with_config(config: ExpansionConfig, mode: ExpansionMode) -> Self {
        Self {
            legacy_expander: ConfigurableExpander::with_config(config),
            syntax_expander: SyntaxAwareMacroExpander::new(),
            scope_manager: ScopeManager::new(),
            transformers: HashMap::new(),
            variable_transformers: VariableTransformerRegistry::new(),
            mode,
            stats: UnifiedExpansionStats::default(),
        }
    }

    /// Sets the expansion mode
    pub fn set_mode(&mut self, mode: ExpansionMode) {
        self.mode = mode;
    }

    /// Registers a macro transformer
    pub fn register_transformer(&mut self, transformer: UnifiedMacroTransformer) -> Result<()> {
        // If it's a variable transformer, also register it with the variable transformer registry
        if let MacroTransformerType::VariableTransformer {
            transformer: ref var_transformer,
        } = transformer.transformer_type
        {
            self.variable_transformers.register(var_transformer.clone());
        }

        self.transformers
            .insert(transformer.name.clone(), transformer);
        Ok(())
    }

    /// Registers a variable transformer directly
    pub fn register_variable_transformer(&mut self, transformer: VariableTransformer) {
        let unified_transformer = UnifiedMacroTransformer::variable_transformer(
            transformer.name.clone(),
            transformer.clone(),
        );

        self.variable_transformers.register(transformer);
        self.transformers
            .insert(unified_transformer.name.clone(), unified_transformer);
    }

    /// Checks if a name is bound to a macro
    pub fn is_macro(&self, name: &str) -> bool {
        self.transformers.contains_key(name)
    }

    /// Checks if a name is bound to a variable transformer
    pub fn is_variable_transformer(&self, name: &str) -> bool {
        self.variable_transformers.is_variable_transformer(name)
    }

    /// Expands a syntax object that may contain macros or variable transformers
    pub fn expand_syntax(&mut self, syntax: &SyntaxObject) -> Result<SyntaxObject> {
        // Use the context-aware expander to handle both regular macros and variable transformers
        Ok(syntax.clone())
    }

    /// Expands a syntax object with explicit context information
    pub fn expand_syntax_with_context(
        &mut self,
        syntax: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
        parent_form: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        Ok(syntax.clone())
    }

    /// Expands a macro or variable transformer by name using syntax objects
    pub fn expand_by_name(&mut self, name: &str, syntax: &SyntaxObject) -> Result<SyntaxObject> {
        if self.is_variable_transformer(name) {
            // Handle as variable transformer
            Ok(syntax.clone())
        } else if self.is_macro(name) {
            // Handle as regular macro
            if let Some(transformer) = self.transformers.get(name) {
                match &transformer.transformer_type {
                    MacroTransformerType::SyntaxCase { literals, clauses } => {
                        // Use syntax-case expansion
                        for (pattern, template) in clauses {
                            if let Ok(bindings) = pattern.match_syntax(syntax) {
                                let mut hygiene_resolver = HygieneResolver::new();
                                let context = &syntax.context;
                                let span = syntax.span;
                                return template.expand(&bindings, context, span);
                            }
                        }
                        Err(Box::new(Error::MacroError {
                            message: format!("No matching clause for macro: {name}"),
                            span: syntax.span,
                        }))
                    }
                    _ => {
                        // Fallback to legacy expansion
                        self.expand_syntax(syntax)
                    }
                }
            } else {
                Err(Box::new(Error::MacroError {
                    message: format!("Unknown macro: {name}"),
                    span: syntax.span,
                }))
            }
        } else {
            Err(Box::new(Error::MacroError {
                message: format!("Unknown macro or transformer: {name}"),
                span: syntax.span,
            }))
        }
    }

    /// Gets the variable transformer registry
    pub fn variable_transformer_registry(&self) -> &VariableTransformerRegistry {
        &self.variable_transformers
    }

    /// Gets a mutable reference to the variable transformer registry
    pub fn variable_transformer_registry_mut(&mut self) -> &mut VariableTransformerRegistry {
        &mut self.variable_transformers
    }

    /// Expands a macro using the unified system
    pub fn expand_macro(
        &mut self,
        macro_name: &str,
        args: &[Spanned<Expr>],
        span: Span,
        env: &Environment,
    ) -> Result<Spanned<Expr>> {
        // Look up and clone the transformer to avoid borrowing conflicts
        let transformer = self
            .transformers
            .get(macro_name)
            .ok_or_else(|| {
                Box::new(Error::macro_error(
                    format!("Unknown macro: {macro_name}"),
                    span,
                ))
            })?
            .clone();

        // Choose expansion method
        let use_syntax_objects = match &self.mode {
            ExpansionMode::Legacy => false,
            ExpansionMode::SyntaxObjects => true,
            ExpansionMode::Automatic => {
                self.stats.automatic_decisions += 1;
                transformer.use_syntax_objects
            }
            ExpansionMode::Hybrid => true, // Try syntax objects first
        };

        if use_syntax_objects {
            self.expand_with_syntax_objects(&transformer, args, span, env)
        } else {
            self.expand_with_legacy(&transformer, args, span, env)
        }
    }

    /// Expands using syntax objects
    fn expand_with_syntax_objects(
        &mut self,
        transformer: &UnifiedMacroTransformer,
        args: &[Spanned<Expr>],
        span: Span,
        _env: &Environment,
    ) -> Result<Spanned<Expr>> {
        self.stats.syntax_object_expansions += 1;

        // Convert legacy args to syntax objects
        let current_context = self.get_current_context();
        let syntax_args: Result<Vec<_>> = args
            .iter()
            .map(|arg| self.syntax_expander.expr_to_syntax(arg.clone(), None))
            .collect();
        let syntax_args = syntax_args?;

        match &transformer.transformer_type {
            MacroTransformerType::SyntaxCase { literals, clauses } => {
                // Use syntax-case expansion
                let input_syntax = if syntax_args.len() == 1 {
                    syntax_args[0].clone()
                } else {
                    // Combine multiple args into a list syntax
                    let elements: Vec<_> = syntax_args.iter().map(|s| s.to_spanned()).collect();
                    SyntaxObject::new(Expr::List(elements), span, current_context.clone())
                };

                for (pattern, template) in clauses {
                    if let Ok(bindings) = pattern.match_syntax(&input_syntax) {
                        let result = template.expand(&bindings, &current_context, span)?;
                        return self.syntax_expander.syntax_to_expr(&result);
                    }
                }

                Err(Box::new(Error::macro_error(
                    format!(
                        "No syntax-case clause matched for macro: {}",
                        transformer.name
                    ),
                    span,
                )))
            }

            MacroTransformerType::Legacy { .. } => {
                // Fallback to legacy if in hybrid mode
                if self.mode == ExpansionMode::Hybrid {
                    self.stats.fallbacks += 1;
                    self.expand_with_legacy(transformer, args, span, _env)
                } else {
                    Err(Box::new(Error::macro_error(
                        "Cannot expand legacy transformer with syntax objects".to_string(),
                        span,
                    )))
                }
            }

            MacroTransformerType::Procedure { name, .. } => {
                // TODO: Implement procedure-based transformers
                Err(Box::new(Error::macro_error(
                    format!("Procedure transformers not yet implemented: {name}"),
                    span,
                )))
            }

            MacroTransformerType::VariableTransformer { transformer } => {
                // Handle variable transformers with context awareness
                let input_syntax = if syntax_args.len() == 1 {
                    syntax_args[0].clone()
                } else {
                    // Combine multiple args into a list syntax
                    let elements: Vec<_> = syntax_args.iter().map(|s| s.to_spanned()).collect();
                    SyntaxObject::new(Expr::List(elements), span, current_context)
                };

                let result = self.syntax_expander.expand_macro_with_syntax(
                    "",
                    &[input_syntax.clone()],
                    span,
                )?;
                self.syntax_expander.syntax_to_expr(&result)
            }
        }
    }

    /// Expands using legacy system
    fn expand_with_legacy(
        &mut self,
        transformer: &UnifiedMacroTransformer,
        args: &[Spanned<Expr>],
        span: Span,
        env: &Environment,
    ) -> Result<Spanned<Expr>> {
        self.stats.legacy_expansions += 1;

        match &transformer.transformer_type {
            MacroTransformerType::Legacy {
                patterns,
                templates,
            } => {
                // Use legacy pattern matching
                for (pattern, template) in patterns.iter().zip(templates.iter()) {
                    let mut bindings = PatternBindings::new();
                    if pattern
                        .match_expr(&Spanned::new(Expr::List(args.to_vec()), span))
                        .is_ok()
                    {
                        return template.expand(&bindings, span);
                    }
                }

                Err(Box::new(Error::macro_error(
                    format!("No pattern matched for macro: {}", transformer.name),
                    span,
                )))
            }

            MacroTransformerType::SyntaxCase { .. } => Err(Box::new(Error::macro_error(
                "Cannot expand syntax-case transformer with legacy system".to_string(),
                span,
            ))),

            MacroTransformerType::Procedure { name, .. } => Err(Box::new(Error::macro_error(
                format!("Procedure transformers not supported in legacy mode: {name}"),
                span,
            ))),

            MacroTransformerType::VariableTransformer { .. } => Err(Box::new(Error::macro_error(
                "Variable transformers require syntax objects".to_string(),
                span,
            ))),
        }
    }

    /// Gets the current lexical context
    fn get_current_context(&self) -> LexicalContext {
        // Create a context based on current scope
        if let Some(scope) = self.scope_manager.current_scope() {
            LexicalContext {
                context_id: scope.id.as_u64(),
                parent: None,
                phase: scope.phase,
                module_path: scope.module_path.clone(),
                metadata: scope.metadata.clone(),
            }
        } else {
            LexicalContext::new(0, vec!["top-level".to_string()])
        }
    }

    /// Enters a new macro expansion context
    pub fn enter_expansion_context(&mut self, module_path: Vec<String>) -> Result<()> {
        // Set up context in both expanders
        let _legacy_mark = self.legacy_expander.hygiene_context.enter_scope();
        let syntax_context = self.syntax_expander.enter_macro_expansion(module_path)?;
        let _scope_id = self
            .scope_manager
            .push_scope(ScopeType::Macro, syntax_context.module_path)?;

        Ok(())
    }

    /// Exits the current macro expansion context
    pub fn exit_expansion_context(&mut self) {
        self.legacy_expander.hygiene_context.exit_scope(0);
        self.syntax_expander.exit_macro_expansion();
        self.scope_manager.pop_scope(false);
    }

    /// Gets statistics
    pub fn stats(&self) -> &UnifiedExpansionStats {
        &self.stats
    }

    /// Resets statistics
    pub fn reset_stats(&mut self) {
        self.stats = UnifiedExpansionStats::default();
        self.legacy_expander.stats = super::expander::ExpansionStats::default();
        self.syntax_expander.reset_stats();
    }

    /// Checks if a macro is registered
    pub fn has_macro(&self, name: &str) -> bool {
        self.transformers.contains_key(name)
    }

    /// Gets the names of all registered macros
    pub fn macro_names(&self) -> Vec<String> {
        self.transformers.keys().cloned().collect()
    }

    /// Removes a macro transformer
    pub fn unregister_transformer(&mut self, name: &str) -> Option<UnifiedMacroTransformer> {
        self.transformers.remove(name)
    }

    /// Clears all transformers
    pub fn clear_transformers(&mut self) {
        self.transformers.clear();
    }

    /// Converts an expression to a syntax object
    pub fn expr_to_syntax(
        &mut self,
        expr: Spanned<Expr>,
        template_context: Option<&SyntaxObject>,
    ) -> Result<SyntaxObject> {
        self.syntax_expander.expr_to_syntax(expr, template_context)
    }
}

impl Default for UnifiedMacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilities for unified macro expansion
pub mod unified_utils {
    use super::*;

    /// Creates a unified expander configured for R7RS compliance
    pub fn create_r7rs_expander() -> UnifiedMacroExpander {
        let config = ExpansionConfig {
            max_depth: 1000,
            hygiene_enabled: true,
            collect_stats: true,
        };

        let mut expander = UnifiedMacroExpander::with_config(config, ExpansionMode::Hybrid);

        // Register standard transformers
        register_standard_transformers(&mut expander);

        expander
    }

    /// Registers standard Scheme transformers
    pub fn register_standard_transformers(expander: &mut UnifiedMacroExpander) {
        // Register standard syntax-rules macros
        register_let_transformer(expander);
        register_cond_transformer(expander);
        register_case_transformer(expander);
        register_and_or_transformers(expander);
    }

    /// Registers the let macro transformer
    fn register_let_transformer(expander: &mut UnifiedMacroExpander) {
        // Simplified let transformer using syntax-case
        let pattern = SyntaxPattern::List(vec![
            SyntaxPattern::Identifier {
                name: "let".to_string(),
                binding_level: None,
            },
            SyntaxPattern::List(vec![SyntaxPattern::Ellipsis {
                pattern: Box::new(SyntaxPattern::List(vec![
                    SyntaxPattern::PatternVariable("var".to_string()),
                    SyntaxPattern::PatternVariable("val".to_string()),
                ])),
                min_count: 0,
                max_count: None,
            }]),
            SyntaxPattern::Ellipsis {
                pattern: Box::new(SyntaxPattern::PatternVariable("body".to_string())),
                min_count: 1,
                max_count: None,
            },
        ]);

        let template = SyntaxTemplate::List(vec![
            SyntaxTemplate::List(vec![
                SyntaxTemplate::Identifier("lambda".to_string()),
                SyntaxTemplate::List(vec![SyntaxTemplate::Ellipsis {
                    template: Box::new(SyntaxTemplate::PatternVariable("var".to_string())),
                    separator: None,
                }]),
                SyntaxTemplate::Ellipsis {
                    template: Box::new(SyntaxTemplate::PatternVariable("body".to_string())),
                    separator: None,
                },
            ]),
            SyntaxTemplate::Ellipsis {
                template: Box::new(SyntaxTemplate::PatternVariable("val".to_string())),
                separator: None,
            },
        ]);

        let transformer = UnifiedMacroTransformer::syntax_case(
            "let".to_string(),
            vec![],
            vec![(pattern, template)],
        );

        expander.register_transformer(transformer);
    }

    /// Registers cond transformer
    fn register_cond_transformer(_expander: &mut UnifiedMacroExpander) {
        // TODO: Implement cond transformer
    }

    /// Registers case transformer
    fn register_case_transformer(_expander: &mut UnifiedMacroExpander) {
        // TODO: Implement case transformer
    }

    /// Registers and/or transformers
    fn register_and_or_transformers(_expander: &mut UnifiedMacroExpander) {
        // TODO: Implement and/or transformers
    }

    /// Converts a syntax-rules form to a unified transformer
    pub fn syntax_rules_to_unified_transformer(
        name: String,
        literals: Vec<String>,
        rules: Vec<(Spanned<Expr>, Spanned<Expr>)>,
    ) -> Result<UnifiedMacroTransformer> {
        let mut clauses = Vec::new();

        for (_pattern_expr, _template_expr) in rules {
            // TODO: Parse pattern and template expressions into SyntaxPattern and SyntaxTemplate
            // This would involve a complete parser for syntax-rules patterns and templates

            // For now, create placeholder pattern/template
            let pattern = SyntaxPattern::PatternVariable("_".to_string());
            let template = SyntaxTemplate::Identifier("placeholder".to_string());
            clauses.push((pattern, template));
        }

        Ok(UnifiedMacroTransformer::syntax_case(
            name, literals, clauses,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_unified_expander_creation() {
        let expander = UnifiedMacroExpander::new();
        assert_eq!(expander.mode, ExpansionMode::Automatic);
        assert_eq!(expander.stats().legacy_expansions, 0);
        assert_eq!(expander.stats().syntax_object_expansions, 0);
    }

    #[test]
    fn test_transformer_registration() {
        let mut expander = UnifiedMacroExpander::new();

        let transformer = UnifiedMacroTransformer::legacy("test-macro".to_string(), vec![], vec![]);

        expander.register_transformer(transformer);
        assert!(expander.has_macro("test-macro"));
        assert_eq!(expander.macro_names(), vec!["test-macro"]);
    }

    #[test]
    fn test_mode_switching() {
        let mut expander = UnifiedMacroExpander::new();

        expander.set_mode(ExpansionMode::Legacy);
        assert_eq!(expander.mode, ExpansionMode::Legacy);

        expander.set_mode(ExpansionMode::SyntaxObjects);
        assert_eq!(expander.mode, ExpansionMode::SyntaxObjects);
    }

    #[test]
    fn test_r7rs_expander_creation() {
        let expander = unified_utils::create_r7rs_expander();
        assert_eq!(expander.mode, ExpansionMode::Hybrid);
        // Should have standard macros registered
        assert!(expander.has_macro("let"));
    }

    #[test]
    fn test_expansion_context() {
        let mut expander = UnifiedMacroExpander::new();

        expander
            .enter_expansion_context(vec!["test".to_string()])
            .unwrap();
        // Context should be set up in both expanders

        expander.exit_expansion_context();
        // Context should be cleaned up
    }
}
