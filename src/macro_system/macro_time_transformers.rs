//! Macro-time Computation Enabled Transformers
//!
//! This module provides enhanced macro transformers that leverage the macro-time
//! computation system for sophisticated compile-time evaluation and template generation.
//! It enables the target functionality from the implementation roadmap.

use super::{
    advanced_hygiene::HygieneResolver,
    advanced_quasisyntax::{AdvancedQuasisyntaxProcessor, AdvancedQuasisyntaxTemplate},
    macro_time_computation::{MacroTimeEnvironment, MacroTimeValue, Phase},
    syntax_case::{SyntaxBindings, SyntaxPattern, SyntaxTemplate},
    syntax_objects::{LexicalContext, SyntaxObject, syntax_utils},
    unified_expander::{MacroTransformerType, UnifiedMacroTransformer},
};
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::Value;
use crate::utils::{SymbolId, intern_symbol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;

/// Enhanced macro transformer with macro-time computation capabilities
#[derive(Debug, Clone)]
pub struct MacroTimeTransformer {
    /// Name of the transformer
    name: String,
    /// Transformation procedure
    transformer_proc: MacroTimeTransformerProc,
    /// Macro-time computation environment
    macro_env: MacroTimeEnvironment,
    /// Advanced quasisyntax processor
    quasisyntax_processor: AdvancedQuasisyntaxProcessor,
    /// Hygiene resolver
    hygiene_env: HygieneResolver,
    /// Transformer statistics
    stats: MacroTimeTransformerStats,
}

/// Types of macro-time transformer procedures
#[derive(Debug, Clone)]
pub enum MacroTimeTransformerProc {
    /// Lambda-based transformer (syntax -> syntax)
    Lambda {
        /// Parameter name for the lambda
        parameter: String,
        /// Body expression of the lambda
        body: Box<SyntaxObject>,
        /// Closure environment with captured bindings
        closure_env: Box<HashMap<String, MacroTimeValue>>,
    },
    /// Syntax-case based transformer with compile-time evaluation
    SyntaxCase {
        /// Literal identifiers that match only themselves (boxed for memory efficiency)
        literals: Box<Vec<String>>,
        /// Pattern-template clauses with macro-time features (boxed for memory efficiency)
        clauses: Box<Vec<MacroTimeClause>>,
    },
    /// Template-based transformer with advanced patterns
    Template {
        /// Pattern to match against input
        pattern: SyntaxPattern,
        /// Advanced template with macro-time computation
        template: AdvancedQuasisyntaxTemplate,
        /// Optional guard condition
        guard: Box<Option<MacroTimeGuard>>,
    },
    /// Procedural transformer with full macro-time computation
    Procedural {
        /// Procedure implementing the transformation
        procedure: MacroTimeProcedure,
    },
}

/// Enhanced clause with macro-time computation support
#[derive(Debug, Clone)]
pub struct MacroTimeClause {
    /// Pattern to match
    pattern: SyntaxPattern,
    /// Advanced template with macro-time features
    template: AdvancedQuasisyntaxTemplate,
    /// Optional guard condition
    guard: Option<MacroTimeGuard>,
    /// Phase at which this clause operates
    phase: Phase,
}

/// Guard condition with macro-time evaluation
#[derive(Debug, Clone)]
pub enum MacroTimeGuard {
    /// Simple boolean expression
    Expression(SyntaxObject),
    /// Macro-time computed guard
    MacroTimeExpression {
        /// Expression to evaluate at macro time
        expr: SyntaxObject,
        /// Phase at which to evaluate the expression
        phase: Phase,
    },
    /// Pattern-based guard
    PatternGuard {
        /// Pattern to match against the test expression
        pattern: SyntaxPattern,
        /// Expression to test against the pattern
        test_expr: SyntaxObject,
    },
    /// Custom predicate name
    Custom(String),
}

/// Macro-time procedure implementation
#[derive(Debug, Clone)]
pub enum MacroTimeProcedure {
    /// Built-in procedure
    Builtin(String),
    /// User-defined procedure with macro-time environment
    UserDefined {
        /// Parameter names for the procedure
        parameters: Vec<String>,
        /// Body expression of the procedure
        body: Box<SyntaxObject>,
        /// Macro-time environment with captured bindings
        environment: Box<HashMap<String, MacroTimeValue>>,
    },
    /// Compiled procedure
    Compiled {
        /// Compiled bytecode (simplified representation)
        bytecode: Vec<u8>,
        /// Procedure metadata and debugging information
        metadata: HashMap<String, String>,
    },
}

/// Statistics for macro-time transformers
#[derive(Debug, Clone, Default)]
pub struct MacroTimeTransformerStats {
    /// Number of transformations performed
    transformations: u64,
    /// Number of macro-time evaluations
    macro_time_evaluations: u64,
    /// Number of template instantiations
    template_instantiations: u64,
    /// Number of pattern matches
    pattern_matches: u64,
    /// Total transformation time
    total_time: std::time::Duration,
    /// Cache hits
    cache_hits: u64,
    /// Cache misses
    cache_misses: u64,
}

impl MacroTimeTransformer {
    /// Creates a new macro-time transformer
    pub fn new(name: String, transformer_proc: MacroTimeTransformerProc) -> Self {
        Self {
            name,
            transformer_proc,
            macro_env: MacroTimeEnvironment::new(),
            quasisyntax_processor: AdvancedQuasisyntaxProcessor::new(),
            hygiene_env: HygieneResolver::new(),
            stats: MacroTimeTransformerStats::default(),
        }
    }

    /// Creates a lambda-based transformer for the target repeat macro
    pub fn create_repeat_transformer() -> Self {
        // Create the lambda body that implements the repeat logic
        let lambda_body = create_repeat_lambda_body();

        let transformer_proc = MacroTimeTransformerProc::Lambda {
            parameter: "stx".to_string(),
            body: Box::new(lambda_body),
            closure_env: Box::new(HashMap::new()),
        };

        Self::new("repeat".to_string(), transformer_proc)
    }

    /// Creates a syntax-case based transformer
    pub fn create_syntax_case_transformer(
        name: String,
        literals: Vec<String>,
        clauses: Vec<MacroTimeClause>,
    ) -> Self {
        let transformer_proc = MacroTimeTransformerProc::SyntaxCase {
            literals: Box::new(literals),
            clauses: Box::new(clauses),
        };

        Self::new(name, transformer_proc)
    }

    /// Creates a template-based transformer
    pub fn create_template_transformer(
        name: String,
        pattern: SyntaxPattern,
        template: AdvancedQuasisyntaxTemplate,
        guard: Option<MacroTimeGuard>,
    ) -> Self {
        let transformer_proc = MacroTimeTransformerProc::Template {
            pattern,
            template,
            guard: Box::new(guard),
        };

        Self::new(name, transformer_proc)
    }

    /// Transforms a syntax object using macro-time computation
    pub fn transform(&mut self, input: &SyntaxObject) -> Result<SyntaxObject> {
        let start_time = std::time::Instant::now();
        self.stats.transformations += 1;

        // Clone the transformer procedure to avoid borrowing issues
        let transformer_proc = self.transformer_proc.clone();
        let result = match transformer_proc {
            MacroTimeTransformerProc::Lambda {
                parameter,
                body,
                closure_env,
            } => self.transform_with_lambda(input, &parameter, &body, &closure_env),
            MacroTimeTransformerProc::SyntaxCase { literals, clauses } => {
                self.transform_with_syntax_case(input, &literals, &clauses)
            }
            MacroTimeTransformerProc::Template {
                pattern,
                template,
                guard,
            } => self.transform_with_template(input, &pattern, &template, guard.as_ref().as_ref()),
            MacroTimeTransformerProc::Procedural { procedure } => {
                self.transform_with_procedure(input, &procedure)
            }
        };

        // Update statistics
        let elapsed = start_time.elapsed();
        self.stats.total_time += elapsed;

        result
    }

    /// Transforms using lambda-based approach
    fn transform_with_lambda(
        &mut self,
        input: &SyntaxObject,
        parameter: &str,
        body: &SyntaxObject,
        closure_env: &HashMap<String, MacroTimeValue>,
    ) -> Result<SyntaxObject> {
        // Set up macro-time environment
        let previous_phase = self.macro_env.enter_phase(Phase::MACRO_TIME);

        // Set up closure environment
        for (name, value) in closure_env.iter() {
            // In a real implementation, these would be bound in the macro environment
            // For now, we'll store them for later use
        }

        // Bind the parameter to the input syntax
        let input_value = MacroTimeValue::Syntax(Box::new(input.clone()));
        // In a real implementation, this would be bound in the environment

        // Evaluate the body
        let result = self
            .macro_env
            .compile_time_eval(body, &mut self.hygiene_env)?;

        // Convert result back to syntax
        let output_syntax = self.macro_time_value_to_syntax(result, &input.context, input.span)?;

        // Restore previous phase
        self.macro_env.exit_phase(previous_phase);

        Ok(output_syntax)
    }

    /// Transforms using syntax-case with macro-time computation
    fn transform_with_syntax_case(
        &mut self,
        input: &SyntaxObject,
        literals: &[String],
        clauses: &[MacroTimeClause],
    ) -> Result<SyntaxObject> {
        for clause in clauses {
            // Try to match the pattern
            if let Ok(bindings) = clause.pattern.match_syntax(input) {
                // Check guard if present
                if let Some(guard) = &clause.guard {
                    if !self.evaluate_guard(guard, input, &bindings)? {
                        continue;
                    }
                }

                // Switch to clause phase
                let previous_phase = self.macro_env.enter_phase(clause.phase);

                // Convert bindings and create context for template processing
                let macro_time_bindings = self.syntax_bindings_to_macro_time_bindings(&bindings);
                let mut generation_context =
                    super::advanced_quasisyntax::AdvancedGenerationContext::new();

                // Expand the template with macro-time computation
                let result = self.quasisyntax_processor.process_template(
                    &clause.template,
                    &macro_time_bindings,
                    &mut generation_context,
                    input.span,
                )?;

                // Restore previous phase
                self.macro_env.exit_phase(previous_phase);

                self.stats.pattern_matches += 1;
                self.stats.template_instantiations += 1;

                return result.into_single();
            }
        }

        Err(Box::new(Error::MacroError {
            message: format!("No matching clause for macro '{}'", self.name),
            span: input.span,
        }))
    }

    /// Transforms using template-based approach
    fn transform_with_template(
        &mut self,
        input: &SyntaxObject,
        pattern: &SyntaxPattern,
        template: &AdvancedQuasisyntaxTemplate,
        guard: Option<&MacroTimeGuard>,
    ) -> Result<SyntaxObject> {
        // Try to match the pattern
        let bindings = pattern.match_syntax(input)?;

        // Check guard if present
        if let Some(guard) = guard {
            if !self.evaluate_guard(guard, input, &bindings)? {
                return Err(Box::new(Error::MacroError {
                    message: "Guard condition failed".to_string(),
                    span: input.span,
                }));
            }
        }

        // Convert bindings and create context for template processing
        let macro_time_bindings = self.syntax_bindings_to_macro_time_bindings(&bindings);
        let mut generation_context = super::advanced_quasisyntax::AdvancedGenerationContext::new();

        // Expand the template
        let result = self.quasisyntax_processor.process_template(
            template,
            &macro_time_bindings,
            &mut generation_context,
            input.span,
        )?;

        self.stats.template_instantiations += 1;
        result.into_single()
    }

    /// Transforms using procedural approach
    fn transform_with_procedure(
        &mut self,
        input: &SyntaxObject,
        procedure: &MacroTimeProcedure,
    ) -> Result<SyntaxObject> {
        match procedure {
            MacroTimeProcedure::Builtin(name) => self.call_builtin_transformer(name, input),
            MacroTimeProcedure::UserDefined {
                parameters,
                body,
                environment,
            } => {
                if parameters.len() != 1 {
                    return Err(Box::new(Error::MacroError {
                        message: "Macro transformer must take exactly one parameter".to_string(),
                        span: input.span,
                    }));
                }

                self.transform_with_lambda(input, &parameters[0], body, environment)
            }
            MacroTimeProcedure::Compiled { .. } => {
                // For now, return the input unchanged
                // A real implementation would execute the compiled bytecode
                Ok(input.clone())
            }
        }
    }

    /// Evaluates a guard condition
    fn evaluate_guard(
        &mut self,
        guard: &MacroTimeGuard,
        input: &SyntaxObject,
        bindings: &SyntaxBindings,
    ) -> Result<bool> {
        match guard {
            MacroTimeGuard::Expression(expr) => {
                let result = self
                    .macro_env
                    .compile_time_eval(expr, &mut self.hygiene_env)?;
                Ok(self.macro_time_value_is_truthy(&result))
            }
            MacroTimeGuard::MacroTimeExpression { expr, phase } => {
                let previous_phase = self.macro_env.enter_phase(*phase);
                let result = self
                    .macro_env
                    .compile_time_eval(expr, &mut self.hygiene_env)?;
                self.macro_env.exit_phase(previous_phase);
                Ok(self.macro_time_value_is_truthy(&result))
            }
            MacroTimeGuard::PatternGuard { pattern, test_expr } => {
                // Try to match the pattern against the test expression result
                let test_result = self
                    .macro_env
                    .compile_time_eval(test_expr, &mut self.hygiene_env)?;
                if let MacroTimeValue::Syntax(test_syntax) = test_result {
                    Ok(pattern.match_syntax(&test_syntax).is_ok())
                } else {
                    Ok(false)
                }
            }
            MacroTimeGuard::Custom(_name) => {
                // Custom guards would be implemented here
                Ok(true)
            }
        }
    }

    /// Checks if a macro-time value is truthy
    fn macro_time_value_is_truthy(&self, value: &MacroTimeValue) -> bool {
        match value {
            MacroTimeValue::Constant(s) => {
                // Check if string represents a false value
                s != "false" && s != "nil" && !s.is_empty()
            }
            MacroTimeValue::SyntaxList(list) => !list.is_empty(),
            _ => true,
        }
    }

    /// Calls a built-in transformer
    fn call_builtin_transformer(
        &mut self,
        name: &str,
        input: &SyntaxObject,
    ) -> Result<SyntaxObject> {
        match name {
            "identity" => Ok(input.clone()),
            "syntax->datum" => {
                let datum = syntax_utils::syntax_to_datum(input);
                let value = self.expr_to_value(&datum)?;
                self.value_to_syntax(value, &input.context, input.span)
            }
            _ => Err(Box::new(Error::MacroError {
                message: format!("Unknown built-in transformer: {name}"),
                span: input.span,
            })),
        }
    }

    /// Converts macro-time value to syntax object
    fn macro_time_value_to_syntax(
        &self,
        value: MacroTimeValue,
        context: &LexicalContext,
        span: Span,
    ) -> Result<SyntaxObject> {
        match value {
            MacroTimeValue::Syntax(syntax) => Ok(*syntax),
            MacroTimeValue::SyntaxList(syntaxes) => {
                let elements: Vec<Spanned<Expr>> =
                    syntaxes.into_iter().map(|s| s.to_spanned()).collect();
                Ok(SyntaxObject::new(
                    Expr::List(elements),
                    span,
                    context.clone(),
                ))
            }
            MacroTimeValue::Constant(value_str) => {
                // For now, treat constant strings as identifiers
                Ok(SyntaxObject::new(
                    Expr::Identifier(value_str),
                    span,
                    context.clone(),
                ))
            }
            _ => Err(Box::new(Error::MacroError {
                message: "Cannot convert macro-time value to syntax".to_string(),
                span,
            })),
        }
    }

    /// Converts a value to a syntax object
    fn value_to_syntax(
        &self,
        value: Value,
        context: &LexicalContext,
        span: Span,
    ) -> Result<SyntaxObject> {
        let expr = match value {
            Value::Literal(lit) => Expr::Literal(lit.clone()),
            Value::Nil => Expr::Literal(Literal::Nil),
            Value::Symbol(s) => Expr::Identifier(format!("symbol-{}", s.id())),
            _ => {
                return Err(Box::new(Error::MacroError {
                    message: "Cannot convert value to syntax".to_string(),
                    span,
                }));
            }
        };
        Ok(SyntaxObject::new(expr, span, context.clone()))
    }

    /// Converts an expression to a value
    fn expr_to_value(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => Ok(self.literal_to_value(lit)),
            Expr::Identifier(name) => Ok(Value::Symbol(intern_symbol(name.clone()))),
            _ => Err(Box::new(Error::MacroError {
                message: "Cannot convert expression to value".to_string(),
                span: Span::new(0, 0),
            })),
        }
    }

    /// Converts a literal to a value
    fn literal_to_value(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Nil => Value::Nil,
            lit => Value::Literal(lit.clone()),
        }
    }

    /// Converts SyntaxBindings to HashMap<String, MacroTimeValue>
    fn syntax_bindings_to_macro_time_bindings(
        &self,
        bindings: &SyntaxBindings,
    ) -> HashMap<String, MacroTimeValue> {
        let mut macro_time_bindings = HashMap::new();

        // Convert single value bindings
        for name in bindings.binding_names() {
            if let Some(syntax) = bindings.get(name) {
                macro_time_bindings.insert(
                    name.to_string(),
                    MacroTimeValue::Syntax(Box::new(syntax.clone())),
                );
            } else if let Some(syntax_list) = bindings.get_ellipsis(name) {
                macro_time_bindings.insert(
                    name.to_string(),
                    MacroTimeValue::SyntaxList(syntax_list.clone()),
                );
            } else if let Some(nested_list) = bindings.get_nested(name) {
                // For nested bindings, flatten to a simple list for now
                if let Some(first_level) = nested_list.first() {
                    macro_time_bindings.insert(
                        name.to_string(),
                        MacroTimeValue::SyntaxList(first_level.clone()),
                    );
                }
            }
        }

        macro_time_bindings
    }

    /// Gets transformer statistics
    pub fn get_stats(&self) -> &MacroTimeTransformerStats {
        &self.stats
    }

    /// Gets the transformer name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Factory functions for creating common macro transformers
pub mod transformer_factory {
    use super::super::syntax_case::SyntaxPattern;
    use super::*;

    /// Creates the target repeat transformer from the roadmap
    pub fn create_repeat_transformer() -> MacroTimeTransformer {
        // Create syntax-case clauses for the repeat macro
        let pattern = SyntaxPattern::List(vec![
            SyntaxPattern::Identifier {
                name: "_".to_string(),
                binding_level: Some(0),
            },
            SyntaxPattern::Identifier {
                name: "n".to_string(),
                binding_level: Some(0),
            },
            SyntaxPattern::Identifier {
                name: "expr".to_string(),
                binding_level: Some(0),
            },
        ]);

        // Create the template: #`(begin #,@(make-list n #'expr))
        let make_list_call = AdvancedQuasisyntaxTemplate::Basic(
            super::super::quasisyntax::QuasisyntaxTemplate::List(vec![
                super::super::quasisyntax::QuasisyntaxTemplate::Identifier("make-list".to_string()),
                super::super::quasisyntax::QuasisyntaxTemplate::PatternVariable("n".to_string()),
                super::super::quasisyntax::QuasisyntaxTemplate::List(vec![
                    super::super::quasisyntax::QuasisyntaxTemplate::Identifier(
                        "syntax".to_string(),
                    ),
                    super::super::quasisyntax::QuasisyntaxTemplate::PatternVariable(
                        "expr".to_string(),
                    ),
                ]),
            ]),
        );

        let splicing_template = AdvancedQuasisyntaxTemplate::AdvancedSplicing {
            splice_expr: Box::new(make_list_call),
            transform: None,
        };

        let begin_template = AdvancedQuasisyntaxTemplate::Basic(
            super::super::quasisyntax::QuasisyntaxTemplate::List(vec![
                super::super::quasisyntax::QuasisyntaxTemplate::Identifier("begin".to_string()),
            ]),
        );

        let final_template = AdvancedQuasisyntaxTemplate::Composition {
            templates: vec![begin_template, splicing_template],
            combiner: super::super::advanced_quasisyntax::TemplateCombiner::List,
        };

        let clause = MacroTimeClause {
            pattern,
            template: final_template,
            guard: None,
            phase: Phase::MACRO_TIME,
        };

        MacroTimeTransformer::create_syntax_case_transformer(
            "repeat".to_string(),
            vec![], // no literals
            vec![clause],
        )
    }

    /// Creates a simple identifier transformation macro
    pub fn create_identifier_transformer(
        name: String,
        from_pattern: String,
        to_template: String,
    ) -> MacroTimeTransformer {
        let pattern = SyntaxPattern::Identifier {
            name: from_pattern,
            binding_level: Some(0),
        };
        let template = AdvancedQuasisyntaxTemplate::Basic(
            super::super::quasisyntax::QuasisyntaxTemplate::Identifier(to_template),
        );

        MacroTimeTransformer::create_template_transformer(name, pattern, template, None)
    }

    /// Creates a list transformation macro with splicing
    pub fn create_list_transformer_with_splicing(
        name: String,
        pattern: SyntaxPattern,
        generator_expr: AdvancedQuasisyntaxTemplate,
    ) -> MacroTimeTransformer {
        let splicing_template = AdvancedQuasisyntaxTemplate::AdvancedSplicing {
            splice_expr: Box::new(generator_expr),
            transform: None,
        };

        MacroTimeTransformer::create_template_transformer(name, pattern, splicing_template, None)
    }
}

/// Creates the lambda body for the repeat transformer
fn create_repeat_lambda_body() -> SyntaxObject {
    // This creates a syntax object representing:
    // (syntax-case stx ()
    //   ((_ n expr)
    //    #`(begin #,@(make-list n #'expr))))

    let context = LexicalContext::new(0, vec!["repeat-transformer".to_string()]);
    let span = Span::new(0, 100);

    // For simplicity, create a basic structure
    // A real implementation would construct the full syntax-case form
    let lambda_body = Expr::List(vec![
        Spanned::new(Expr::Identifier("syntax-case".to_string()), span),
        Spanned::new(Expr::Identifier("stx".to_string()), span),
        Spanned::new(Expr::List(vec![]), span), // empty literals list
        Spanned::new(
            Expr::List(vec![
                Spanned::new(
                    Expr::List(vec![
                        Spanned::new(Expr::Identifier("_".to_string()), span),
                        Spanned::new(Expr::Identifier("n".to_string()), span),
                        Spanned::new(Expr::Identifier("expr".to_string()), span),
                    ]),
                    span,
                ),
                Spanned::new(Expr::Identifier("template-expansion".to_string()), span),
            ]),
            span,
        ),
    ]);

    SyntaxObject::new(lambda_body, span, context)
}

impl MacroTimeTransformerStats {
    /// Calculates cache hit ratio
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Gets average transformation time
    pub fn average_transformation_time(&self) -> std::time::Duration {
        if self.transformations == 0 {
            std::time::Duration::ZERO
        } else {
            self.total_time / self.transformations as u32
        }
    }

    /// Gets transformations per second
    pub fn transformations_per_second(&self) -> f64 {
        if self.total_time.is_zero() {
            0.0
        } else {
            self.transformations as f64 / self.total_time.as_secs_f64()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::transformer_factory::*;
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_repeat_transformer_creation() {
        let transformer = create_repeat_transformer();
        assert_eq!(transformer.name(), "repeat");

        // Test basic structure
        assert!(matches!(
            transformer.transformer_proc,
            MacroTimeTransformerProc::SyntaxCase { .. }
        ));
    }

    #[test]
    fn test_macro_time_transformer_stats() {
        let mut stats = MacroTimeTransformerStats::default();

        assert_eq!(stats.cache_hit_ratio(), 0.0);
        assert_eq!(
            stats.average_transformation_time(),
            std::time::Duration::ZERO
        );
        assert_eq!(stats.transformations_per_second(), 0.0);

        stats.transformations = 10;
        stats.total_time = std::time::Duration::from_millis(100);

        assert_eq!(stats.transformations_per_second(), 100.0);
    }

    #[test]
    fn test_identifier_transformer() {
        let transformer = create_identifier_transformer(
            "my-var".to_string(),
            "old-name".to_string(),
            "new-name".to_string(),
        );

        assert_eq!(transformer.name(), "my-var");
    }

    #[test]
    fn test_macro_time_guard_evaluation() {
        let mut transformer = MacroTimeTransformer::new(
            "test".to_string(),
            MacroTimeTransformerProc::Procedural {
                procedure: MacroTimeProcedure::Builtin("identity".to_string()),
            },
        );

        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 4);
        let true_expr =
            SyntaxObject::new(Expr::Literal(Literal::Boolean(true)), span, context.clone());

        let guard = MacroTimeGuard::Expression(true_expr);
        let input = SyntaxObject::new(Expr::Identifier("test".to_string()), span, context);
        let bindings = SyntaxBindings::new();

        let result = transformer
            .evaluate_guard(&guard, &input, &bindings)
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_macro_time_value_conversion() {
        let transformer = MacroTimeTransformer::new(
            "test".to_string(),
            MacroTimeTransformerProc::Procedural {
                procedure: MacroTimeProcedure::Builtin("identity".to_string()),
            },
        );

        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 4);

        let value =
            MacroTimeValue::Constant(Value::Literal(Literal::InexactReal(42.0)).to_string());
        let syntax = transformer
            .macro_time_value_to_syntax(value, &context, span)
            .unwrap();

        assert!(matches!(syntax.expr, Expr::Literal(Literal::InexactReal(n)) if n == 42.0));
    }

    #[test]
    fn test_template_transformer() {
        use super::super::syntax_case::SyntaxPattern;

        let pattern = SyntaxPattern::Identifier {
            name: "test".to_string(),
            binding_level: Some(0),
        };
        let template = AdvancedQuasisyntaxTemplate::Basic(
            super::super::quasisyntax::QuasisyntaxTemplate::Identifier("result".to_string()),
        );

        let transformer = MacroTimeTransformer::create_template_transformer(
            "test-macro".to_string(),
            pattern,
            template,
            None,
        );

        assert_eq!(transformer.name(), "test-macro");
        assert!(matches!(
            transformer.transformer_proc,
            MacroTimeTransformerProc::Template { .. }
        ));
    }
}
