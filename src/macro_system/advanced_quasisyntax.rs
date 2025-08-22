//! Advanced Quasisyntax System for Lambdust
//!
//! This module implements an enhanced quasisyntax system that goes beyond traditional
//! template processing to enable advanced compile-time computation and code generation.
//! Key features include:
//!
//! - Advanced template composition and nesting capabilities
//! - Compile-time evaluation with macro-time computation integration
//! - Sophisticated splicing with conditional and computed generation
//! - Performance optimizations for complex template expansions
//! - Meta-programming utilities for macro writers
//!
//! This system enables complex template patterns like:
//! ```scheme
//! (define-syntax complex-loop
//!   (lambda (stx)
//!     (syntax-case stx ()
//!       ((_ ((var start) ...) body ...)
//!        #`(begin
//!            #,@(map (lambda (v s)
//!                     #`(let ((#,v #,s))
//!                         #,@body))
//!                   #'(var ...) #'(start ...)))))))
//! ```

use super::{
    advanced_hygiene::{HygieneResolver, Mark, MarkSet},
    macro_time_computation::{MacroTimeEnvironment, MacroTimeValue, Phase},
    quasisyntax::{
        QuasisyntaxContext, QuasisyntaxExpansionResult as BasicQuasisyntaxExpansionResult,
        QuasisyntaxTemplate,
    },
    syntax_case::SyntaxBindings,
    syntax_objects::{LexicalContext, SyntaxObject, syntax_utils},
    unified_expander::{MacroTransformerType, UnifiedMacroTransformer},
};
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::{Environment, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

/// Advanced quasisyntax template supporting complex generation patterns
#[derive(Debug, Clone)]
pub enum AdvancedQuasisyntaxTemplate {
    /// Basic quasisyntax template
    Basic(QuasisyntaxTemplate),
    /// Template composition (combining multiple templates)
    Composition {
        /// Templates to be composed together
        templates: Vec<AdvancedQuasisyntaxTemplate>,
        /// Method for combining the templates
        combiner: TemplateCombiner,
    },
    /// Conditional template generation
    Conditional {
        /// Condition template to evaluate
        condition: Box<AdvancedQuasisyntaxTemplate>,
        /// Template to use if condition is true
        then_template: Box<AdvancedQuasisyntaxTemplate>,
        /// Optional template to use if condition is false
        else_template: Option<Box<AdvancedQuasisyntaxTemplate>>,
    },
    /// Computed template generation (evaluate at compile-time)
    ComputedGeneration {
        /// Template generator expression
        generator: Box<AdvancedQuasisyntaxTemplate>,
        /// Context for template generation
        context: Box<AdvancedGenerationContext>,
    },
    /// Splicing with advanced patterns
    AdvancedSplicing {
        /// Expression to splice
        splice_expr: Box<AdvancedQuasisyntaxTemplate>,
        /// Optional transformation to apply during splicing
        transform: Option<Box<SpliceTransform>>,
    },
    /// Meta-template (template that generates templates)
    MetaTemplate {
        /// Meta-expression that generates templates
        meta_expr: Box<AdvancedQuasisyntaxTemplate>,
        /// Level of meta-template generation
        generation_level: u32,
    },
}

/// Methods for combining templates
#[derive(Debug, Clone)]
pub enum TemplateCombiner {
    /// Append templates sequentially
    Append,
    /// Create a list containing all templates
    List,
    /// Nest templates (create nested structure)
    Nested,
    /// Interleave templates with a separator
    Interleave(Box<AdvancedQuasisyntaxTemplate>),
}

/// Context for advanced template generation
#[derive(Debug, Clone)]
pub struct AdvancedGenerationContext {
    /// Base quasisyntax context
    base_context: QuasisyntaxContext,
    /// Current generation depth (for preventing infinite recursion)
    generation_depth: u32,
    /// Maximum allowed generation depth
    max_generation_depth: u32,
    /// Template variables available for generation
    template_variables: HashMap<String, AdvancedQuasisyntaxTemplate>,
    /// Macro-time evaluation environment
    macro_env: MacroTimeEnvironment,
    /// Current macro expansion phase
    current_phase: Phase,
}

/// Transformations that can be applied during splicing
#[derive(Debug, Clone)]
pub enum SpliceTransform {
    /// Map a function over each spliced element
    Map(String), // procedure name
    /// Filter elements based on a predicate
    Filter(String), // predicate name
    /// Fold/reduce elements
    Fold {
        /// Name of the fold procedure to apply
        procedure: String,
        /// Initial value template for the fold operation
        initial: Box<AdvancedQuasisyntaxTemplate>,
    },
    /// Interleave with a separator
    Interleave(Box<AdvancedQuasisyntaxTemplate>),
    /// Flatten nested lists
    Flatten,
}

/// Result of processing an advanced quasisyntax template
#[derive(Debug, Clone)]
pub enum QuasisyntaxExpansionResult {
    /// Single syntax object result
    Single(Box<SyntaxObject>),
    /// Multiple syntax objects (for splicing)
    Multiple(Vec<SyntaxObject>),
    /// Template for further processing
    Template(AdvancedQuasisyntaxTemplate),
}

/// Performance statistics for quasisyntax processing
#[derive(Debug, Clone, Default)]
pub struct QuasisyntaxStats {
    /// Number of templates processed
    templates_processed: u64,
    /// Number of compile-time evaluations
    compile_time_evaluations: u64,
    /// Number of template compositions
    template_compositions: u64,
    /// Number of conditional generations
    conditional_generations: u64,
    /// Number of meta-template expansions
    meta_template_expansions: u64,
    /// Total processing time
    total_processing_time: std::time::Duration,
}

/// Advanced quasisyntax processor
#[derive(Debug, Clone)]
pub struct AdvancedQuasisyntaxProcessor {
    /// Template cache for performance
    template_cache: HashMap<String, AdvancedQuasisyntaxTemplate>,
    /// Hygiene resolver
    hygiene_resolver: HygieneResolver,
    /// Performance statistics
    stats: QuasisyntaxStats,
    /// Unique identifier counter
    unique_counter: std::sync::Arc<AtomicU64>,
}

impl AdvancedQuasisyntaxProcessor {
    /// Creates a new advanced quasisyntax processor
    pub fn new() -> Self {
        Self {
            template_cache: HashMap::new(),
            hygiene_resolver: HygieneResolver::new(),
            stats: QuasisyntaxStats::default(),
            unique_counter: std::sync::Arc::new(AtomicU64::new(0)),
        }
    }

    /// Processes an advanced quasisyntax template
    pub fn process_template(
        &mut self,
        template: &AdvancedQuasisyntaxTemplate,
        bindings: &HashMap<String, MacroTimeValue>,
        context: &mut AdvancedGenerationContext,
        span: Span,
    ) -> Result<QuasisyntaxExpansionResult> {
        let start_time = std::time::Instant::now();
        self.stats.templates_processed += 1;

        let result = match template {
            AdvancedQuasisyntaxTemplate::Basic(basic) => {
                self.process_basic_template(basic, bindings, context, span)
            }
            AdvancedQuasisyntaxTemplate::Composition {
                templates,
                combiner,
            } => self.process_template_composition(templates, combiner, bindings, context, span),
            AdvancedQuasisyntaxTemplate::Conditional {
                condition,
                then_template,
                else_template,
            } => self.process_conditional_template(
                condition,
                then_template,
                else_template,
                bindings,
                context,
                span,
            ),
            AdvancedQuasisyntaxTemplate::ComputedGeneration {
                generator,
                context: gen_context,
            } => self.process_computed_generation(generator, gen_context, bindings, context, span),
            AdvancedQuasisyntaxTemplate::AdvancedSplicing {
                splice_expr,
                transform,
            } => self.process_advanced_splicing(
                splice_expr,
                transform.as_ref().map(|t| t.as_ref()),
                bindings,
                context,
                span,
            ),
            AdvancedQuasisyntaxTemplate::MetaTemplate {
                meta_expr,
                generation_level,
            } => self.process_meta_template(meta_expr, *generation_level, bindings, context, span),
        };

        // Update statistics
        let elapsed = start_time.elapsed();
        self.stats.total_processing_time += elapsed;

        result
    }

    /// Processes a basic quasisyntax template
    fn process_basic_template(
        &mut self,
        template: &QuasisyntaxTemplate,
        bindings: &HashMap<String, MacroTimeValue>,
        context: &mut AdvancedGenerationContext,
        span: Span,
    ) -> Result<QuasisyntaxExpansionResult> {
        // Convert basic template to syntax object
        let syntax = self.expand_basic_template(template, bindings, &context.base_context, span)?;
        Ok(QuasisyntaxExpansionResult::Single(Box::new(syntax)))
    }

    /// Processes template composition
    fn process_template_composition(
        &mut self,
        templates: &[AdvancedQuasisyntaxTemplate],
        combiner: &TemplateCombiner,
        bindings: &HashMap<String, MacroTimeValue>,
        context: &mut AdvancedGenerationContext,
        span: Span,
    ) -> Result<QuasisyntaxExpansionResult> {
        self.stats.template_compositions += 1;

        let mut all_results = Vec::new();

        // Expand all templates
        for template in templates {
            let result = self.process_template(template, bindings, context, span)?;
            all_results.extend(result.into_multiple());
        }

        match combiner {
            TemplateCombiner::Append => Ok(QuasisyntaxExpansionResult::Multiple(all_results)),

            TemplateCombiner::List => {
                // Create a single list containing all results
                let list_elements: Vec<Spanned<Expr>> =
                    all_results.into_iter().map(|s| s.to_spanned()).collect();

                let list_syntax = SyntaxObject::new(
                    Expr::List(list_elements),
                    span,
                    context.base_context.lexical_context().clone(),
                );
                Ok(QuasisyntaxExpansionResult::Single(Box::new(list_syntax)))
            }

            TemplateCombiner::Nested => {
                // Create nested structure
                let result = all_results.into_iter().rev().reduce(|acc, curr| {
                    let pair_expr = Expr::Pair {
                        car: Box::new(curr.to_spanned()),
                        cdr: Box::new(acc.to_spanned()),
                    };
                    SyntaxObject::new(
                        pair_expr,
                        span,
                        context.base_context.lexical_context().clone(),
                    )
                });

                if let Some(nested) = result {
                    Ok(QuasisyntaxExpansionResult::Single(Box::new(nested)))
                } else {
                    // Empty case
                    let nil_syntax = SyntaxObject::new(
                        Expr::Literal(Literal::Nil),
                        span,
                        context.base_context.lexical_context().clone(),
                    );
                    Ok(QuasisyntaxExpansionResult::Single(Box::new(nil_syntax)))
                }
            }

            TemplateCombiner::Interleave(separator) => {
                let sep_result = self.process_template(separator, bindings, context, span)?;
                let separator_syntax = sep_result.into_single()?;

                let mut interleaved = Vec::new();
                for (i, syntax) in all_results.into_iter().enumerate() {
                    if i > 0 {
                        interleaved.push(separator_syntax.clone());
                    }
                    interleaved.push(syntax);
                }

                Ok(QuasisyntaxExpansionResult::Multiple(interleaved))
            }
        }
    }

    /// Processes conditional template generation
    fn process_conditional_template(
        &mut self,
        condition: &AdvancedQuasisyntaxTemplate,
        then_template: &AdvancedQuasisyntaxTemplate,
        else_template: &Option<Box<AdvancedQuasisyntaxTemplate>>,
        bindings: &HashMap<String, MacroTimeValue>,
        context: &mut AdvancedGenerationContext,
        span: Span,
    ) -> Result<QuasisyntaxExpansionResult> {
        // Check generation depth to prevent infinite recursion
        if context.generation_depth >= context.max_generation_depth {
            return Err(Box::new(Error::MacroError {
                message: "Maximum template generation depth exceeded".to_string(),
                span,
            }));
        }

        context.generation_depth += 1;

        // Switch to meta-macro-time phase
        let meta_phase = context.current_phase.increment();
        let previous_phase = context.macro_env.enter_phase(meta_phase);

        // Evaluate condition at compile-time
        self.stats.conditional_generations += 1;
        let condition_result = self.process_template(condition, bindings, context, span)?;

        // Convert result to boolean
        let is_true = match condition_result {
            QuasisyntaxExpansionResult::Single(syntax) => self.syntax_to_boolean(&syntax)?,
            _ => false,
        };

        // Choose template based on condition
        let chosen_template = if is_true {
            then_template
        } else if let Some(else_tmpl) = else_template {
            else_tmpl
        } else {
            // No else clause, return empty
            return Ok(QuasisyntaxExpansionResult::Multiple(vec![]));
        };

        let result = self.process_template(chosen_template, bindings, context, span);

        // Restore previous phase
        context.macro_env.exit_phase(previous_phase);
        context.generation_depth -= 1;

        result
    }

    /// Processes computed template generation
    fn process_computed_generation(
        &mut self,
        generator: &AdvancedQuasisyntaxTemplate,
        gen_context: &AdvancedGenerationContext,
        bindings: &HashMap<String, MacroTimeValue>,
        context: &mut AdvancedGenerationContext,
        span: Span,
    ) -> Result<QuasisyntaxExpansionResult> {
        self.stats.compile_time_evaluations += 1;

        // Process the generator template to get a macro-time value
        let mut gen_context_mut = gen_context.clone();
        let generator_result =
            self.process_template(generator, bindings, &mut gen_context_mut, span)?;

        // Convert result to syntax and then evaluate at compile-time
        let syntax = generator_result.into_single()?;
        let mut macro_env = context.macro_env.clone();
        let mut hygiene_env = self.hygiene_resolver.clone();

        let macro_time_value = macro_env.compile_time_eval(&syntax, &mut hygiene_env)?;

        // Convert macro-time value back to syntax
        match macro_time_value {
            MacroTimeValue::Syntax(syntax) => Ok(QuasisyntaxExpansionResult::Single(syntax)),
            MacroTimeValue::SyntaxList(syntaxes) => {
                Ok(QuasisyntaxExpansionResult::Multiple(syntaxes))
            }
            MacroTimeValue::Constant(value_str) => {
                // Convert constant string to syntax
                // This is a simplified implementation - would need proper parsing
                let expr = if value_str == "true" {
                    Expr::Literal(Literal::Boolean(true))
                } else if value_str == "false" {
                    Expr::Literal(Literal::Boolean(false))
                } else if value_str == "nil" {
                    Expr::Literal(Literal::Nil)
                } else if let Ok(num) = value_str.parse::<f64>() {
                    Expr::Literal(Literal::Number(num))
                } else if let Ok(int) = value_str.parse::<i64>() {
                    Expr::Literal(Literal::Number(int as f64))
                } else {
                    // Treat as identifier
                    Expr::Identifier(value_str.clone())
                };
                Ok(QuasisyntaxExpansionResult::Single(Box::new(
                    SyntaxObject::new(expr, span, context.base_context.lexical_context().clone()),
                )))
            }
            _ => Err(Box::new(Error::MacroError {
                message: "Cannot convert macro-time value to syntax".to_string(),
                span,
            })),
        }
    }

    /// Processes advanced splicing
    fn process_advanced_splicing(
        &mut self,
        splice_expr: &AdvancedQuasisyntaxTemplate,
        transform: Option<&SpliceTransform>,
        bindings: &HashMap<String, MacroTimeValue>,
        context: &mut AdvancedGenerationContext,
        span: Span,
    ) -> Result<QuasisyntaxExpansionResult> {
        // Process the splice expression
        let splice_result = self.process_template(splice_expr, bindings, context, span)?;
        let mut elements = splice_result.into_multiple();

        // Apply transform if specified
        if let Some(transform) = transform {
            elements = self.apply_splice_transform(transform, elements, context, span)?;
        }

        Ok(QuasisyntaxExpansionResult::Multiple(elements))
    }

    /// Processes meta-template
    fn process_meta_template(
        &mut self,
        meta_expr: &AdvancedQuasisyntaxTemplate,
        generation_level: u32,
        bindings: &HashMap<String, MacroTimeValue>,
        context: &mut AdvancedGenerationContext,
        span: Span,
    ) -> Result<QuasisyntaxExpansionResult> {
        self.stats.meta_template_expansions += 1;

        if generation_level == 0 {
            // Base case: process normally
            self.process_template(meta_expr, bindings, context, span)
        } else {
            // Recursive case: generate a template that generates a template
            let inner_result = self.process_template(meta_expr, bindings, context, span)?;

            // Convert result to advanced template
            let advanced_template = self.result_to_advanced_template(inner_result, span)?;

            // Return as template for further processing
            Ok(QuasisyntaxExpansionResult::Template(advanced_template))
        }
    }

    /// Converts a macro-time value to an advanced template
    fn result_to_advanced_template(
        &self,
        result: QuasisyntaxExpansionResult,
        span: Span,
    ) -> Result<AdvancedQuasisyntaxTemplate> {
        match result {
            QuasisyntaxExpansionResult::Single(syntax) => {
                let basic_template = self.syntax_to_basic_template(&syntax)?;
                Ok(AdvancedQuasisyntaxTemplate::Basic(basic_template))
            }
            QuasisyntaxExpansionResult::Multiple(syntaxes) => {
                let templates: Result<Vec<_>> = syntaxes
                    .iter()
                    .map(|s| {
                        let basic = self.syntax_to_basic_template(s)?;
                        Ok(AdvancedQuasisyntaxTemplate::Basic(basic))
                    })
                    .collect();

                Ok(AdvancedQuasisyntaxTemplate::Composition {
                    templates: templates?,
                    combiner: TemplateCombiner::Append,
                })
            }
            QuasisyntaxExpansionResult::Template(template) => Ok(template),
        }
    }

    /// Converts a syntax object to a basic template
    #[allow(clippy::only_used_in_recursion)]
    fn syntax_to_basic_template(&self, syntax: &SyntaxObject) -> Result<QuasisyntaxTemplate> {
        match &syntax.expr {
            Expr::Literal(lit) => Ok(QuasisyntaxTemplate::Literal(lit.clone())),
            Expr::Identifier(name) => Ok(QuasisyntaxTemplate::Identifier(name.clone())),
            Expr::List(elements) => {
                let templates: Result<Vec<_>> = elements
                    .iter()
                    .map(|e| {
                        let syntax = SyntaxObject::from_spanned(e.clone(), syntax.context.clone());
                        self.syntax_to_basic_template(&syntax)
                    })
                    .collect();
                Ok(QuasisyntaxTemplate::List(templates?))
            }
            _ => Err(Box::new(Error::MacroError {
                message: "Cannot convert macro-time value to template".to_string(),
                span: Span::new(0, 0),
            })),
        }
    }

    /// Gets processing statistics
    pub fn get_stats(&self) -> &QuasisyntaxStats {
        &self.stats
    }

    /// Clears the template cache
    pub fn clear_cache(&mut self) {
        self.template_cache.clear();
    }

    // Helper methods (simplified implementations)

    fn expand_basic_template(
        &self,
        template: &QuasisyntaxTemplate,
        bindings: &HashMap<String, MacroTimeValue>,
        context: &QuasisyntaxContext,
        span: Span,
    ) -> Result<SyntaxObject> {
        // This would integrate with the existing quasisyntax system
        // For now, create a simple syntax object
        match template {
            QuasisyntaxTemplate::Literal(lit) => Ok(SyntaxObject::new(
                Expr::Literal(lit.clone()),
                span,
                context.lexical_context().clone(),
            )),
            QuasisyntaxTemplate::Identifier(name) => Ok(SyntaxObject::new(
                Expr::Identifier(name.clone()),
                span,
                context.lexical_context().clone(),
            )),
            _ => {
                // Simplified - would need full quasisyntax expansion
                Ok(SyntaxObject::new(
                    Expr::Literal(Literal::Nil),
                    span,
                    context.lexical_context().clone(),
                ))
            }
        }
    }

    fn syntax_to_boolean(&self, syntax: &SyntaxObject) -> Result<bool> {
        match &syntax.expr {
            Expr::Literal(Literal::Boolean(b)) => Ok(*b),
            Expr::Literal(Literal::Nil) => Ok(false),
            _ => Ok(true), // Non-false values are truthy
        }
    }

    fn apply_splice_transform(
        &self,
        _transform: &SpliceTransform,
        elements: Vec<SyntaxObject>,
        _context: &mut AdvancedGenerationContext,
        _span: Span,
    ) -> Result<Vec<SyntaxObject>> {
        // Simplified implementation - would need full transform support
        Ok(elements)
    }
}

impl Default for AdvancedQuasisyntaxProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl QuasisyntaxExpansionResult {
    /// Converts result to a single syntax object (errors if multiple)
    pub fn into_single(self) -> Result<SyntaxObject> {
        match self {
            QuasisyntaxExpansionResult::Single(syntax) => Ok(*syntax),
            QuasisyntaxExpansionResult::Multiple(mut syntaxes) => {
                if syntaxes.len() == 1 {
                    Ok(syntaxes.pop().unwrap())
                } else {
                    Err(Box::new(Error::MacroError {
                        message: "Expected single result, got multiple".to_string(),
                        span: Span::new(0, 0),
                    }))
                }
            }
            QuasisyntaxExpansionResult::Template(_) => Err(Box::new(Error::MacroError {
                message: "Expected syntax result, got template".to_string(),
                span: Span::new(0, 0),
            })),
        }
    }

    /// Converts result to multiple syntax objects
    pub fn into_multiple(self) -> Vec<SyntaxObject> {
        match self {
            QuasisyntaxExpansionResult::Single(syntax) => vec![*syntax],
            QuasisyntaxExpansionResult::Multiple(syntaxes) => syntaxes,
            QuasisyntaxExpansionResult::Template(_) => vec![], // Templates don't convert to syntax directly
        }
    }
}

impl AdvancedGenerationContext {
    /// Creates a new advanced generation context
    pub fn new() -> Self {
        Self {
            base_context: QuasisyntaxContext::new(
                SyntaxBindings::new(),
                LexicalContext::new(0, vec![]),
            ),
            generation_depth: 0,
            max_generation_depth: 100,
            template_variables: HashMap::new(),
            macro_env: MacroTimeEnvironment::new(),
            current_phase: Phase::MACRO_TIME,
        }
    }
}

impl Default for AdvancedGenerationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Public interface module for advanced quasisyntax
pub mod advanced_quasisyntax_interface {
    use super::*;

    /// Creates a template composition
    pub fn make_template_composition(
        templates: Vec<AdvancedQuasisyntaxTemplate>,
        combiner: TemplateCombiner,
    ) -> Result<AdvancedQuasisyntaxTemplate> {
        Ok(AdvancedQuasisyntaxTemplate::Composition {
            templates,
            combiner,
        })
    }

    /// Creates a conditional generation template
    pub fn make_conditional_generation(
        condition: AdvancedQuasisyntaxTemplate,
        then_template: AdvancedQuasisyntaxTemplate,
        else_template: Option<AdvancedQuasisyntaxTemplate>,
    ) -> Result<AdvancedQuasisyntaxTemplate> {
        Ok(AdvancedQuasisyntaxTemplate::Conditional {
            condition: Box::new(condition),
            then_template: Box::new(then_template),
            else_template: else_template.map(Box::new),
        })
    }

    /// Creates a computed generation template
    pub fn make_computed_generation(
        generator: AdvancedQuasisyntaxTemplate,
        context: AdvancedGenerationContext,
    ) -> Result<AdvancedQuasisyntaxTemplate> {
        Ok(AdvancedQuasisyntaxTemplate::ComputedGeneration {
            generator: Box::new(generator),
            context: Box::new(context),
        })
    }

    /// Creates an advanced splicing template
    pub fn make_advanced_splicing(
        splice_expr: AdvancedQuasisyntaxTemplate,
        transform: Option<SpliceTransform>,
    ) -> Result<AdvancedQuasisyntaxTemplate> {
        Ok(AdvancedQuasisyntaxTemplate::AdvancedSplicing {
            splice_expr: Box::new(splice_expr),
            transform: transform.map(Box::new),
        })
    }

    /// Creates a meta-template
    pub fn make_meta_template(
        meta_expr: AdvancedQuasisyntaxTemplate,
        generation_level: u32,
    ) -> Result<AdvancedQuasisyntaxTemplate> {
        Ok(AdvancedQuasisyntaxTemplate::MetaTemplate {
            meta_expr: Box::new(meta_expr),
            generation_level,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_basic_template_processing() {
        let mut processor = AdvancedQuasisyntaxProcessor::new();
        let mut context = AdvancedGenerationContext::new();
        let bindings = HashMap::new();

        let template =
            AdvancedQuasisyntaxTemplate::Basic(QuasisyntaxTemplate::Identifier("test".to_string()));

        let result = processor
            .process_template(&template, &bindings, &mut context, Span::new(0, 4))
            .unwrap();

        let syntax = result.into_single().unwrap();
        assert_eq!(syntax.identifier_name(), Some("test"))
    }

    #[test]
    fn test_macro_time_eval_template() {
        let condition = AdvancedQuasisyntaxTemplate::Basic(QuasisyntaxTemplate::Literal(
            Literal::Boolean(true),
        ));
        let then_template =
            AdvancedQuasisyntaxTemplate::Basic(QuasisyntaxTemplate::Identifier("test".to_string()));

        let conditional = AdvancedQuasisyntaxTemplate::Conditional {
            condition: Box::new(condition),
            then_template: Box::new(then_template),
            else_template: None,
        };

        let mut processor = AdvancedQuasisyntaxProcessor::new();
        let mut context = AdvancedGenerationContext::new();
        let bindings = HashMap::new();

        let result =
            processor.process_template(&conditional, &bindings, &mut context, Span::new(0, 4));

        assert!(result.is_ok());
    }

    #[test]
    fn test_template_composition() {
        let template1 =
            AdvancedQuasisyntaxTemplate::Basic(QuasisyntaxTemplate::Identifier("x".to_string()));
        let template2 =
            AdvancedQuasisyntaxTemplate::Basic(QuasisyntaxTemplate::Identifier("y".to_string()));

        let composition = AdvancedQuasisyntaxTemplate::Composition {
            templates: vec![template1, template2],
            combiner: TemplateCombiner::List,
        };

        let mut processor = AdvancedQuasisyntaxProcessor::new();
        let mut context = AdvancedGenerationContext::new();
        let bindings = HashMap::new();

        let result = processor
            .process_template(&composition, &bindings, &mut context, Span::new(0, 4))
            .unwrap();

        let syntax = result.into_single().unwrap();
        assert!(syntax.is_list());

        if let Some(list) = syntax.as_list() {
            assert_eq!(list.len(), 2);
            assert!(list[0].is_identifier());
            assert!(list[1].is_identifier());
        }
    }

    #[test]
    fn test_conditional_generation() {
        let condition = AdvancedQuasisyntaxTemplate::Basic(QuasisyntaxTemplate::Literal(
            Literal::Boolean(true),
        ));
        let then_template =
            AdvancedQuasisyntaxTemplate::Basic(QuasisyntaxTemplate::Identifier("then".to_string()));
        let else_template =
            AdvancedQuasisyntaxTemplate::Basic(QuasisyntaxTemplate::Identifier("else".to_string()));

        let conditional = AdvancedQuasisyntaxTemplate::Conditional {
            condition: Box::new(condition),
            then_template: Box::new(then_template),
            else_template: Some(Box::new(else_template)),
        };

        let mut processor = AdvancedQuasisyntaxProcessor::new();
        let mut context = AdvancedGenerationContext::new();
        let bindings = HashMap::new();

        let result = processor
            .process_template(&conditional, &bindings, &mut context, Span::new(0, 4))
            .unwrap();

        let syntax = result.into_single().unwrap();
        assert_eq!(syntax.identifier_name(), Some("then"));
    }

    #[test]
    fn test_advanced_interface() {
        use advanced_quasisyntax_interface::*;

        let template1 =
            AdvancedQuasisyntaxTemplate::Basic(QuasisyntaxTemplate::Identifier("x".to_string()));
        let template2 =
            AdvancedQuasisyntaxTemplate::Basic(QuasisyntaxTemplate::Identifier("y".to_string()));

        let composition =
            make_template_composition(vec![template1, template2], TemplateCombiner::Append);

        assert!(composition.is_ok());

        let conditional = make_conditional_generation(
            AdvancedQuasisyntaxTemplate::Basic(QuasisyntaxTemplate::Literal(Literal::Boolean(
                true,
            ))),
            AdvancedQuasisyntaxTemplate::Basic(QuasisyntaxTemplate::Identifier("yes".to_string())),
            None,
        );

        assert!(conditional.is_ok());
    }
}
