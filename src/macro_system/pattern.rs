//! Pattern matching for macro expansion.

use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Spanned};
use std::collections::HashMap;

/// A pattern that can be matched against expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Matches any expression and binds it to the given variable name
    Variable(String),
    /// Matches a specific literal value (number, string, boolean, etc.)
    Literal(Literal),
    /// Matches a specific identifier name
    Identifier(String),
    /// Matches a specific keyword
    Keyword(String),
    /// Matches the nil/empty list value
    Nil,
    /// Matches a list containing patterns in sequence
    List(Vec<Pattern>),
    /// Matches a list with ellipsis (...) for repetitive patterns
    Ellipsis {
        /// Fixed patterns that must appear before the ellipsis
        patterns: Vec<Pattern>,
        /// The pattern that can be repeated zero or more times
        ellipsis_pattern: Box<Pattern>,
        /// Optional patterns that must appear after the ellipsis
        rest: Option<Box<Pattern>>,
    },
    /// Matches a cons pair (car . cdr)
    Pair {
        /// Pattern for the first element of the pair
        car: Box<Pattern>,
        /// Pattern for the rest/tail of the pair
        cdr: Box<Pattern>,
    },
    /// Matches any expression without binding (anonymous pattern)
    Wildcard,
    /// Matches expressions that do NOT match the inner pattern
    Not(Box<Pattern>),
    /// Matches if any of the patterns match (logical OR)
    Or(Vec<Pattern>),
    /// Matches if all patterns match (logical AND)
    And(Vec<Pattern>),
}

/// Bindings created during pattern matching.
#[derive(Debug, Clone, Default)]
pub struct PatternBindings {
    bindings: HashMap<String, Spanned<Expr>>,
    ellipsis_bindings: HashMap<String, Vec<Spanned<Expr>>>,
}

impl PatternBindings {
    /// Creates a new empty set of pattern bindings.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Binds a variable name to a single expression.
    pub fn bind(&mut self, name: String, expr: Spanned<Expr>) {
        self.bindings.insert(name, expr);
    }
    
    /// Binds a variable name to a list of expressions (for ellipsis patterns).
    pub fn bind_ellipsis(&mut self, name: String, exprs: Vec<Spanned<Expr>>) {
        self.ellipsis_bindings.insert(name, exprs);
    }
    
    /// Gets the expression bound to a variable name.
    pub fn get(&self, name: &str) -> Option<&Spanned<Expr>> {
        self.bindings.get(name)
    }
    
    /// Gets the list of expressions bound to an ellipsis variable name.
    pub fn get_ellipsis(&self, name: &str) -> Option<&Vec<Spanned<Expr>>> {
        self.ellipsis_bindings.get(name)
    }
    
    /// Gets all single-expression bindings.
    pub fn bindings(&self) -> &HashMap<String, Spanned<Expr>> {
        &self.bindings
    }
    
    /// Gets all ellipsis (multi-expression) bindings.
    pub fn ellipsis_bindings(&self) -> &HashMap<String, Vec<Spanned<Expr>>> {
        &self.ellipsis_bindings
    }
}

impl Pattern {
    /// Creates a variable pattern that binds to the given name.
    pub fn variable(name: impl Into<String>) -> Self {
        Pattern::Variable(name.into())
    }
    
    /// Creates a literal pattern that matches the given literal value.
    pub fn literal(lit: Literal) -> Self {
        Pattern::Literal(lit)
    }
    
    /// Creates an identifier pattern that matches the given identifier name.
    pub fn identifier(name: impl Into<String>) -> Self {
        Pattern::Identifier(name.into())
    }
    
    /// Creates a list pattern with the given sub-patterns.
    pub fn list(patterns: Vec<Pattern>) -> Self {
        Pattern::List(patterns)
    }
    
    /// Creates an ellipsis pattern with fixed patterns, a repeating pattern, and optional rest patterns.
    pub fn ellipsis(patterns: Vec<Pattern>, ellipsis_pattern: Pattern, rest: Option<Pattern>) -> Self {
        Pattern::Ellipsis {
            patterns,
            ellipsis_pattern: Box::new(ellipsis_pattern),
            rest: rest.map(Box::new),
        }
    }
    
    /// Attempts to match this pattern against an expression, returning variable bindings on success.
    pub fn match_expr(&self, expr: &Spanned<Expr>) -> Result<PatternBindings> {
        let mut bindings = PatternBindings::new();
        self.match_expr_with_bindings(expr, &mut bindings)?;
        Ok(bindings)
    }
    
    fn match_expr_with_bindings(
        &self,
        expr: &Spanned<Expr>,
        bindings: &mut PatternBindings,
    ) -> Result<()> {
        match (self, &expr.inner) {
            (Pattern::Variable(name), _) => {
                bindings.bind(name.clone(), expr.clone());
                Ok(())
            }
            
            (Pattern::Wildcard, _) => Ok(()),
            
            (Pattern::Literal(pat_lit), Expr::Literal(expr_lit)) => {
                if pat_lit == expr_lit {
                    Ok(())
                } else {
                    Err(Box::new(Error::macro_error(
                        format!("Literal mismatch: expected {pat_lit:?}, got {expr_lit:?}"),
                        expr.span,
                    )))
                }
            }
            
            _ => Err(Box::new(Error::macro_error(
                format!("Pattern type mismatch: {self:?} vs {:?}", expr.inner),
                expr.span,
            ))),
        }
    }
    
    /// Collects all variable names that would be bound by this pattern.
    pub fn bound_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_variables(&mut vars);
        vars
    }
    
    fn collect_variables(&self, vars: &mut Vec<String>) {
        match self {
            Pattern::Variable(name) => vars.push(name.clone()),
            Pattern::List(patterns) => {
                for pat in patterns {
                    pat.collect_variables(vars);
                }
            }
            _ => {}
        }
    }
}