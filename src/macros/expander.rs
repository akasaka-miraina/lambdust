//! `MacroExpander`: Core macro expansion engine for Scheme macros
//!
//! This module implements the macro expansion system for Lambdust, providing:
//! - Built-in macro expansion (let, let*, letrec, cond, case, when, unless, define-record-type)
//! - Syntax-rules macro expansion with SRFI 46 extensions
//! - Recursive macro expansion throughout expression trees
//! - Proper handling of quasiquote contexts and ellipsis patterns
//!
//! The `MacroExpander` maintains a registry of available macros and provides
//! methods for detecting macro calls, expanding individual macros, and
//! recursively expanding entire expressions.

use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use std::collections::HashMap;

// Import from the new modular structure
use super::builtin::{expand_case, expand_cond, expand_define_record_type, expand_let, expand_let_star, expand_letrec, expand_unless, expand_when};
use super::pattern_matching::{Pattern, Template, SyntaxCaseClause};
use super::syntax_case::SyntaxCaseTransformer;
use super::syntax_rules::SyntaxRulesTransformer;
use super::types::{Macro, MacroTransformer};

/// Macro expansion context
///
/// The `MacroExpander` maintains a registry of available macros and provides
/// the core functionality for macro detection and expansion. It supports
/// both built-in macros (implemented as functions) and syntax-rules macros
/// (implemented as pattern-template transformers).
#[derive(Debug, Clone)]
pub struct MacroExpander {
    /// Built-in macros
    macros: HashMap<String, Macro>,
}

impl MacroExpander {
    /// Create a new macro expander
    ///
    /// Initializes the expander with all built-in macros automatically registered.
    #[must_use] pub fn new() -> Self {
        let mut expander = MacroExpander {
            macros: HashMap::new(),
        };

        // Add built-in macros
        expander.add_builtin_macros();
        expander
    }

    /// Define a new syntax-rules macro
    ///
    /// Registers a new syntax-rules macro with the given name, literals, and transformation rules.
    pub fn define_syntax_rules_macro(
        &mut self,
        name: String,
        literals: Vec<String>,
        rules: Vec<crate::macros::pattern_matching::SyntaxRule>,
    ) {
        let transformer = SyntaxRulesTransformer::new(literals, rules);
        self.macros
            .insert(name.clone(), Macro::SyntaxRules { name, transformer });
    }

    /// Define a new hygienic syntax-rules macro
    ///
    /// Registers a new hygienic syntax-rules macro with symbol collision prevention.
    /// The definition environment is captured at definition time for proper hygiene.
    pub fn define_hygienic_syntax_rules_macro(
        &mut self,
        name: String,
        literals: Vec<String>,
        rules: Vec<crate::macros::pattern_matching::SyntaxRule>,
        definition_environment: std::rc::Rc<super::hygiene::HygienicEnvironment>,
    ) {
        let transformer = super::hygiene::HygienicSyntaxRulesTransformer::new(
            literals,
            rules,
            definition_environment,
            name.clone(),
        );
        self.macros.insert(
            name.clone(),
            Macro::HygienicSyntaxRules { name, transformer },
        );
    }

    /// Define a new syntax-case macro
    ///
    /// Registers a new syntax-case macro with advanced pattern matching and guard conditions.
    /// This provides more sophisticated macro capabilities compared to syntax-rules.
    pub fn define_syntax_case_macro(
        &mut self,
        name: String,
        literals: Vec<String>,
        clauses: Vec<SyntaxCaseClause>,
        definition_environment: super::hygiene::HygienicEnvironment,
    ) {
        let transformer = SyntaxCaseTransformer::new(
            literals,
            clauses,
            definition_environment,
        );
        self.macros.insert(
            name.clone(),
            Macro::SyntaxCase { name, transformer },
        );
    }

    /// Add built-in macros
    ///
    /// Registers all built-in macros including let, let*, letrec, cond, case,
    /// when, unless, and define-record-type.
    fn add_builtin_macros(&mut self) {
        // let macro
        self.macros.insert(
            "let".to_string(),
            Macro::Builtin {
                name: "let".to_string(),
                transformer: expand_let,
            },
        );

        // let* macro
        self.macros.insert(
            "let*".to_string(),
            Macro::Builtin {
                name: "let*".to_string(),
                transformer: expand_let_star,
            },
        );

        // letrec macro
        self.macros.insert(
            "letrec".to_string(),
            Macro::Builtin {
                name: "letrec".to_string(),
                transformer: expand_letrec,
            },
        );

        // cond macro
        self.macros.insert(
            "cond".to_string(),
            Macro::Builtin {
                name: "cond".to_string(),
                transformer: expand_cond,
            },
        );

        // case macro
        self.macros.insert(
            "case".to_string(),
            Macro::Builtin {
                name: "case".to_string(),
                transformer: expand_case,
            },
        );

        // when macro
        self.macros.insert(
            "when".to_string(),
            Macro::Builtin {
                name: "when".to_string(),
                transformer: expand_when,
            },
        );

        // unless macro
        self.macros.insert(
            "unless".to_string(),
            Macro::Builtin {
                name: "unless".to_string(),
                transformer: expand_unless,
            },
        );

        // define-record-type macro (SRFI 9)
        self.macros.insert(
            "define-record-type".to_string(),
            Macro::Builtin {
                name: "define-record-type".to_string(),
                transformer: expand_define_record_type,
            },
        );
    }

    /// Check if an expression is a macro call
    ///
    /// Returns true if the expression is a list starting with a registered macro name.
    #[must_use] pub fn is_macro_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => match &exprs[0] {
                Expr::Variable(name) => self.macros.contains_key(name),
                _ => false,
            },
            _ => false,
        }
    }

    /// Expand a macro call
    ///
    /// Expands a single macro call using the appropriate transformer.
    /// Returns the original expression if it's not a macro call.
    pub fn expand_macro(&self, expr: Expr) -> Result<Expr> {
        match &expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                match &exprs[0] {
                    Expr::Variable(name) => {
                        if let Some(macro_def) = self.macros.get(name) {
                            match macro_def {
                                Macro::Builtin { transformer, .. } => {
                                    let args = &exprs[1..];
                                    transformer(args)
                                }
                                Macro::SyntaxRules { transformer, .. } => {
                                    transformer.transform(&expr)
                                }
                                Macro::HygienicSyntaxRules { transformer, .. } => {
                                    // For hygienic macros, we need usage environment
                                    // For now, use a default environment - this should be passed as parameter
                                    let usage_env = super::hygiene::HygienicEnvironment::new();
                                    let args = &exprs[1..];
                                    transformer.transform_hygienic(args, &usage_env)
                                }
                                Macro::SyntaxCase { transformer, .. } => {
                                    // For syntax-case, use a default hygienic environment
                                    let usage_env = super::hygiene::HygienicEnvironment::new();
                                    transformer.transform(&Expr::List(exprs.clone()), &usage_env)
                                }
                            }
                        } else {
                            Ok(expr) // Not a macro
                        }
                    }
                    _ => Ok(expr), // Not a macro
                }
            }
            _ => Ok(expr), // Not a macro
        }
    }

    /// Expand a macro call with hygienic environment and safety checks
    ///
    /// Expands a single macro call using the appropriate transformer with proper
    /// hygienic environment context for symbol collision prevention.
    pub fn expand_macro_hygienic(
        &self,
        expr: Expr,
        usage_env: &super::hygiene::HygienicEnvironment,
    ) -> Result<Expr> {
        match &expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                match &exprs[0] {
                    Expr::Variable(name) => {
                        if let Some(macro_def) = self.macros.get(name) {
                            match macro_def {
                                Macro::Builtin { transformer, .. } => {
                                    let args = &exprs[1..];
                                    transformer(args)
                                }
                                Macro::SyntaxRules { transformer, .. } => {
                                    transformer.transform(&expr)
                                }
                                Macro::HygienicSyntaxRules { transformer, .. } => {
                                    self.expand_hygienic_syntax_rules(transformer, &exprs[1..], usage_env)
                                }
                                Macro::SyntaxCase { transformer, .. } => {
                                    // For syntax-case, we pass the entire expression including the macro name
                                    transformer.transform(&Expr::List(exprs.clone()), usage_env)
                                }
                            }
                        } else {
                            Ok(expr) // Not a macro
                        }
                    }
                    _ => Ok(expr), // Not a macro
                }
            }
            _ => Ok(expr), // Not a macro
        }
    }
    
    /// Helper method to expand hygienic syntax rules
    ///
    /// Extracted from nested pattern matching to reduce nesting depth
    fn expand_hygienic_syntax_rules(
        &self,
        transformer: &super::hygiene::HygienicSyntaxRulesTransformer,
        args: &[Expr],
        usage_env: &super::hygiene::HygienicEnvironment,
    ) -> Result<Expr> {
        transformer.transform_hygienic(args, usage_env)
    }

    /// Recursively expand nested macros in an expression
    ///
    /// This method safely handles nested macro expansion while checking for
    /// circular dependencies and respecting safety limits.
    #[allow(dead_code)]
    fn expand_nested_macros(
        &self,
        expr: Expr,
        env: &super::hygiene::HygienicEnvironment,
    ) -> Result<Expr> {
        match expr {
            Expr::List(exprs) => {
                let mut expanded_exprs = Vec::new();
                for sub_expr in exprs {
                    let expanded = self.expand_nested_macros(sub_expr, env)?;
                    // Check if this sub-expression is a macro call
                    if let Expr::List(ref sub_list) = expanded {
                        if let Some(Expr::Variable(name)) = sub_list.first() {
                            if self.macros.contains_key(name) {
                                // This is a macro call, expand it
                                let further_expanded = self.expand_macro_hygienic(expanded, env)?;
                                expanded_exprs.push(further_expanded);
                                continue;
                            }
                        }
                    }
                    expanded_exprs.push(expanded);
                }
                Ok(Expr::List(expanded_exprs))
            }
            other => Ok(other),
        }
    }

    /// Recursively expand all macros in an expression
    ///
    /// Performs a complete macro expansion of the given expression,
    /// recursively expanding all nested macro calls while respecting
    /// quote contexts and proper expansion order.
    pub fn expand_all(&self, expr: Expr) -> Result<Expr> {
        match expr {
            Expr::List(exprs) => {
                // First expand any macro calls
                let expanded = if self.is_macro_call(&Expr::List(exprs.clone())) {
                    self.expand_macro(Expr::List(exprs))?
                } else {
                    Expr::List(exprs)
                };

                // Then recursively expand subexpressions
                match expanded {
                    Expr::List(exprs) => {
                        let mut expanded_exprs = Vec::new();
                        for expr in exprs {
                            expanded_exprs.push(self.expand_all(expr)?);
                        }
                        Ok(Expr::List(expanded_exprs))
                    }
                    other => self.expand_all(other),
                }
            }
            Expr::Quote(expr) => Ok(Expr::Quote(expr)), // Don't expand inside quotes
            Expr::Quasiquote(expr) => {
                // Handle quasiquote expansion specially
                Ok(Expr::Quasiquote(Box::new(self.expand_all(*expr)?)))
            }
            Expr::Unquote(expr) => Ok(Expr::Unquote(Box::new(self.expand_all(*expr)?))),
            Expr::UnquoteSplicing(expr) => {
                Ok(Expr::UnquoteSplicing(Box::new(self.expand_all(*expr)?)))
            }
            Expr::Vector(exprs) => {
                let mut expanded_exprs = Vec::new();
                for expr in exprs {
                    expanded_exprs.push(self.expand_all(expr)?);
                }
                Ok(Expr::Vector(expanded_exprs))
            }
            Expr::DottedList(exprs, tail) => {
                let mut expanded_exprs = Vec::new();
                for expr in exprs {
                    expanded_exprs.push(self.expand_all(expr)?);
                }
                let expanded_tail = self.expand_all(*tail)?;
                Ok(Expr::DottedList(expanded_exprs, Box::new(expanded_tail)))
            }
            other => Ok(other), // Literals and variables don't need expansion
        }
    }

    /// Define a new macro
    ///
    /// Registers a new built-in macro with the given name and transformer function.
    pub fn define_macro(&mut self, name: String, transformer: MacroTransformer) {
        self.macros
            .insert(name.clone(), Macro::Builtin { name, transformer });
    }

    /// SRFI 46: Count ellipsis nesting level
    ///
    /// Counts the maximum nesting level of ellipsis patterns in an expression.
    /// This is used for SRFI 46 nested ellipsis support.
    pub fn count_ellipsis_level(expr: &Expr) -> usize {
        match expr {
            Expr::Variable(name) if name == "..." => 1,
            Expr::List(exprs) => exprs
                .iter()
                .map(Self::count_ellipsis_level)
                .max()
                .unwrap_or(0),
            _ => 0,
        }
    }

    /// SRFI 46: Parse pattern with SRFI 46 extensions
    ///
    /// Parses a syntax-rules pattern with support for SRFI 46 nested ellipsis.
    /// Handles nested ellipsis patterns and proper error checking.
    #[allow(clippy::only_used_in_recursion)]
    pub fn parse_pattern_srfi46(&self, expr: &Expr) -> Result<Pattern> {
        match expr {
            Expr::Variable(name) => {
                if name == "..." {
                    return Err(LambdustError::syntax_error(
                        "syntax-rules: unexpected ellipsis".to_string(),
                    ));
                }
                Ok(Pattern::Variable(name.clone()))
            }
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    return Ok(Pattern::List(vec![]));
                }

                let mut patterns = Vec::new();
                let mut i = 0;

                while i < exprs.len() {
                    if let Expr::Variable(name) = &exprs[i] {
                        if name == "..." {
                            if patterns.is_empty() {
                                return Err(LambdustError::syntax_error(
                                    "syntax-rules: ellipsis without preceding pattern".to_string(),
                                ));
                            }

                            // Check for nested ellipsis (SRFI 46)
                            let ellipsis_count = if i + 1 < exprs.len() {
                                Self::count_ellipsis_level(&exprs[i + 1])
                            } else {
                                0
                            };

                            let last_pattern = patterns.pop().unwrap();
                            if ellipsis_count > 0 {
                                patterns.push(Pattern::NestedEllipsis(
                                    Box::new(last_pattern),
                                    ellipsis_count + 1,
                                ));
                                i += ellipsis_count + 1; // Skip the additional ellipses
                            } else {
                                patterns.push(Pattern::Ellipsis(Box::new(last_pattern)));
                                i += 1;
                            }
                            continue;
                        }
                    }

                    patterns.push(self.parse_pattern_srfi46(&exprs[i])?);
                    i += 1;
                }

                Ok(Pattern::List(patterns))
            }
            Expr::Literal(lit) => Ok(Pattern::Literal(format!("{lit:?}"))),
            _ => Err(LambdustError::syntax_error(
                "syntax-rules: invalid pattern".to_string(),
            )),
        }
    }

    /// SRFI 46: Parse template with SRFI 46 extensions
    ///
    /// Parses a syntax-rules template with support for SRFI 46 nested ellipsis.
    /// Handles nested ellipsis patterns and proper error checking.
    #[allow(clippy::only_used_in_recursion)]
    pub fn parse_template_srfi46(&self, expr: &Expr) -> Result<Template> {
        match expr {
            Expr::Variable(name) => {
                if name == "..." {
                    return Err(LambdustError::syntax_error(
                        "syntax-rules: unexpected ellipsis in template".to_string(),
                    ));
                }
                Ok(Template::Variable(name.clone()))
            }
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    return Ok(Template::List(vec![]));
                }

                let mut templates = Vec::new();
                let mut i = 0;

                while i < exprs.len() {
                    if let Expr::Variable(name) = &exprs[i] {
                        if name == "..." {
                            if templates.is_empty() {
                                return Err(LambdustError::syntax_error(
                                    "syntax-rules: ellipsis without preceding template".to_string(),
                                ));
                            }

                            // Check for nested ellipsis (SRFI 46)
                            let ellipsis_count = if i + 1 < exprs.len() {
                                Self::count_ellipsis_level(&exprs[i + 1])
                            } else {
                                0
                            };

                            let last_template = templates.pop().unwrap();
                            if ellipsis_count > 0 {
                                templates.push(Template::NestedEllipsis(
                                    Box::new(last_template),
                                    ellipsis_count + 1,
                                ));
                                i += ellipsis_count + 1;
                            } else {
                                templates.push(Template::Ellipsis(Box::new(last_template)));
                                i += 1;
                            }
                            continue;
                        }
                    }

                    templates.push(self.parse_template_srfi46(&exprs[i])?);
                    i += 1;
                }

                Ok(Template::List(templates))
            }
            Expr::Literal(lit) => Ok(Template::Literal(format!("{lit:?}"))),
            _ => Err(LambdustError::syntax_error(
                "syntax-rules: invalid template".to_string(),
            )),
        }
    }
}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::new()
    }
}
