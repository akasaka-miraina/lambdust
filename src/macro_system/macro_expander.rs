//! The main macro expander.

use crate::ast::{Expr, Spanned};
use crate::diagnostics::{Error, Result, Span};
use crate::eval::Environment;
use super::{
    MacroTransformer, MacroEnvironment, HygieneContext, PatternBindings,
    install_builtin_macros, next_hygiene_id, parse_syntax_rules, syntax_rules_to_macro_transformer
};
use std::collections::HashMap;
use std::rc::Rc;

/// The main macro expander.
#[derive(Debug, Clone)]
pub struct MacroExpander {
    /// Environment containing macro definitions
    macro_env: Rc<MacroEnvironment>,
    /// Current expansion depth (for detecting infinite recursion)
    expansion_depth: usize,
    /// Maximum allowed expansion depth
    max_expansion_depth: usize,
    /// Current hygiene context
    hygiene_context: HygieneContext,
}

impl MacroExpander {
    /// Creates a new macro expander.
    pub fn new() -> Self {
        Self {
            macro_env: Rc::new(MacroEnvironment::new()),
            expansion_depth: 0,
            max_expansion_depth: 100,
            hygiene_context: HygieneContext::new(),
        }
    }
    
    /// Ensures that the macro expander can access global syntax elements.
    /// This is crucial for macros that expand to basic special forms like 'lambda', 'if', etc.
    fn ensure_global_syntax_access(&mut self) {
        // Currently, we rely on the global environment binding special forms as symbols
        // This allows macro templates to reference them properly during expansion
        // The actual binding happens in create_global_environment() 
        // where bind_special_forms_as_identifiers() is called
        
        // For now, we just ensure the macro environment is properly initialized
        // Future enhancement: could create a dedicated macro-time environment
        // that has explicit mappings for special forms
    }

    /// Creates a new macro expander with built-in macros.
    pub fn with_builtins() -> Self {
        let mut expander = Self::new();
        
        // Initialize macro environment with access to global environment
        // This ensures macros can reference basic syntax like 'lambda', 'if', etc.
        expander.ensure_global_syntax_access();
        
        install_builtin_macros(&mut expander);
        expander
    }

    /// Expands a single expression.
    pub fn expand(&mut self, expr: &Spanned<Expr>) -> Result<Spanned<Expr>> {
        self.expand_inner(expr, &mut Vec::new())
    }
    
    /// Expands all expressions in a program.
    pub fn expand_program(&mut self, program: &crate::ast::Program) -> Result<crate::ast::Program> {
        let mut expanded_expressions = Vec::new();
        for expr in &program.expressions {
            let expanded = self.expand(expr)?;
            expanded_expressions.push(expanded);
        }
        Ok(crate::ast::Program::with_expressions(expanded_expressions))
    }

    /// Internal expansion method with recursion tracking.
    fn expand_inner(
        &mut self,
        expr: &Spanned<Expr>,
        expansion_trail: &mut Vec<String>,
    ) -> Result<Spanned<Expr>> {
        // Check for infinite recursion
        if self.expansion_depth >= self.max_expansion_depth {
            return Err(Box::new(Error::macro_error(
                "Macro expansion depth exceeded".to_string(),
                expr.span,
            ));
        }

        match &expr.inner {
            Expr::Application { operator, operands } => {
                // Check if the operator is a macro
                if let Expr::Identifier(name) = &operator.inner {
                    if let Some(transformer) = self.macro_env.lookup(name) {
                        // Prevent infinite recursion
                        if expansion_trail.contains(name) {
                            return Err(Box::new(Error::macro_error(
                                format!("Recursive macro expansion detected: {name}"),
                                expr.span,
                            ));
                        }

                        expansion_trail.push(name.clone());
                        self.expansion_depth += 1;

                        let result = self.expand_macro(&transformer, operands, expr.span);

                        self.expansion_depth -= 1;
                        expansion_trail.pop();

                        // Recursively expand the result
                        match result {
                            Ok(expanded) => self.expand_inner(&expanded, expansion_trail),
                            Err(e) => Err(e),
                        }
                    } else {
                        // Not a macro, expand operands
                        self.expand_application(operator, operands, expr.span)
                    }
                } else {
                    // Operator is not an identifier, expand normally
                    self.expand_application(operator, operands, expr.span)
                }
            }
            // Special forms that contain expressions to expand
            Expr::Lambda { formals, metadata, body } => {
                let expanded_body = self.expand_body(body)?;
                let expanded_metadata = self.expand_metadata(metadata)?;
                Ok(Spanned::new(
                    Expr::Lambda {
                        formals: formals.clone()),
                        metadata: expanded_metadata,
                        body: expanded_body,
                    },
                    expr.span,
                ))
            }
            Expr::If { test, consequent, alternative } => {
                let expanded_test = self.expand_inner(test, expansion_trail)?;
                let expanded_consequent = self.expand_inner(consequent, expansion_trail)?;
                let expanded_alternative = if let Some(alt) = alternative {
                    Some(Box::new(self.expand_inner(alt, expansion_trail)?))
                } else {
                    None
                };
                Ok(Spanned::new(
                    Expr::If {
                        test: Box::new(expanded_test),
                        consequent: Box::new(expanded_consequent),
                        alternative: expanded_alternative,
                    },
                    expr.span,
                ))
            }
            Expr::Define { name, value, metadata } => {
                let expanded_value = self.expand_inner(value, expansion_trail)?;
                let expanded_metadata = self.expand_metadata(metadata)?;
                Ok(Spanned::new(
                    Expr::Define {
                        name: name.clone()),
                        value: Box::new(expanded_value),
                        metadata: expanded_metadata,
                    },
                    expr.span,
                ))
            }
            Expr::DefineSyntax { name, transformer } => {
                // Define-syntax creates a new macro
                let transformer_value = self.evaluate_transformer(transformer, expansion_trail.clone())?;
                self.macro_env.define(name.clone()), transformer_value);
                Ok(Spanned::new(Expr::Begin(vec![]), expr.span)) // Expand to empty begin
            }
            Expr::SyntaxRules { literals, rules } => {
                // syntax-rules should not appear at top level normally, but handle it
                // Convert to macro transformer representation
                let syntax_rules_transformer = super::parse_syntax_rules(expr, Rc::new(Environment::new(None, 0)))?;
                let macro_transformer = super::syntax_rules_to_macro_transformer(syntax_rules_transformer);
                
                // For demonstration, create a temporary identifier
                let temp_name = format!("temp-syntax-{}", next_hygiene_id());
                self.macro_env.define(temp_name, macro_transformer);
                Ok(Spanned::new(Expr::Begin(vec![]), expr.span)) // Expand to empty begin
            }
            // Other expression types that don't need expansion
            _ => Ok(expr.clone()),
        }
    }

    /// Expands a macro application.
    fn expand_macro(
        &mut self,
        transformer: &MacroTransformer,
        operands: &[Spanned<Expr>],
        span: Span,
    ) -> Result<Spanned<Expr>> {
        // Create input expression for pattern matching
        let input_expr = Spanned::new(
            Expr::Application {
                operator: Box::new(Spanned::new(
                    Expr::Identifier(transformer.name.clone()).unwrap_or_default()),
                    span,
                )),
                operands: operands.to_vec(),
            },
            span,
        );

        // Try to match the pattern against the input
        let bindings = self.match_pattern(&transformer.pattern, &input_expr)?;

        // Expand the template with the bindings
        let expanded = self.expand_template(&transformer.template, &bindings, span)?;

        // Apply hygiene transformations
        self.apply_hygiene(expanded, &transformer.definition_env)
    }

    /// Expands a function application (non-macro).
    fn expand_application(
        &mut self,
        operator: &Spanned<Expr>,
        operands: &[Spanned<Expr>],
        span: Span,
    ) -> Result<Spanned<Expr>> {
        let expanded_operator = self.expand_inner(operator, &mut Vec::new())?;
        let mut expanded_operands = Vec::new();
        for operand in operands {
            expanded_operands.push(self.expand_inner(operand, &mut Vec::new())?);
        }
        Ok(Spanned::new(
            Expr::Application {
                operator: Box::new(expanded_operator),
                operands: expanded_operands,
            },
            span,
        ))
    }

    /// Expands a sequence of expressions.
    fn expand_body(&mut self, body: &[Spanned<Expr>]) -> Result<Vec<Spanned<Expr>>> {
        let mut expanded = Vec::new();
        for expr in body {
            expanded.push(self.expand_inner(expr, &mut Vec::new())?);
        }
        Ok(expanded)
    }

    /// Expands metadata expressions.
    fn expand_metadata(
        &mut self,
        metadata: &HashMap<String, Spanned<Expr>>,
    ) -> Result<HashMap<String, Spanned<Expr>>> {
        let mut expanded = HashMap::new();
        for (key, value) in metadata {
            expanded.insert(key.clone()), self.expand_inner(value, &mut Vec::new())?);
        }
        Ok(expanded)
    }

    /// Evaluates a macro transformer expression.
    fn evaluate_transformer(&self, transformer_expr: &Spanned<Expr>, _expansion_trail: Vec<String>) -> Result<MacroTransformer> {
        // Check if this is a syntax-rules form
        match &transformer_expr.inner {
            Expr::Application { operator, .. } => {
                if let Expr::Identifier(name) = &operator.inner {
                    if name == "syntax-rules" {
                        // Parse syntax-rules and convert to MacroTransformer
                        // For now, create a basic environment - this will be improved when
                        // macro/evaluator integration is complete
                        let empty_env = Rc::new(Environment::new(None, 0));
                        let syntax_rules = parse_syntax_rules(
                            transformer_expr,
                            empty_env,
                        )?;
                        return Ok(syntax_rules_to_macro_transformer(syntax_rules));
                    }
                }
            }
            _ => {}
        }
        
        Err(Box::new(Error::macro_error(
            "Expected syntax-rules transformer".to_string(),
            transformer_expr.span,
        ))
    }

    /// Matches a pattern against an expression.
    fn match_pattern(
        &self,
        pattern: &super::Pattern,
        expr: &Spanned<Expr>,
    ) -> Result<PatternBindings> {
        pattern.match_expr(expr)
    }

    /// Expands a template with the given bindings.
    fn expand_template(
        &self,
        template: &super::Template,
        bindings: &PatternBindings,
        span: Span,
    ) -> Result<Spanned<Expr>> {
        template.expand(bindings, span)
    }

    /// Applies hygiene transformations to ensure lexical scoping.
    fn apply_hygiene(
        &mut self,
        expr: Spanned<Expr>,
        definition_env: &Environment,
    ) -> Result<Spanned<Expr>> {
        self.hygiene_context.rename_identifiers(expr, definition_env)
    }

    /// Defines a new macro.
    pub fn define_macro(&mut self, name: String, transformer: MacroTransformer) {
        self.macro_env.define(name, transformer);
    }
    
    /// Defines a syntax-rules macro directly.
    pub fn define_syntax_rules_macro(&mut self, name: String, syntax_rules: crate::macro_system::SyntaxRulesTransformer) {
        let transformer = crate::macro_system::syntax_rules_to_macro_transformer(syntax_rules);
        self.macro_env.define(name, transformer);
    }

    /// Gets the macro environment.
    pub fn macro_env(&self) -> &MacroEnvironment {
        &self.macro_env
    }

}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::with_builtins()
    }
}