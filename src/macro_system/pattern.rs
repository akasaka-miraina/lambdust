//! Pattern matching for macro expansion.
//!
//! This module implements the pattern matching system used in Scheme macro
//! definitions. Patterns can match against literal values, identifiers, and
//! complex nested structures with ellipsis support.

#![allow(missing_docs)]

use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Spanned};
// use crate::utils::{intern_symbol, symbol_name, SymbolId};
use std::collections::HashMap;

/// A pattern that can be matched against expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Match any expression and bind it to a variable
    Variable(String),
    
    /// Match a literal value exactly
    Literal(Literal),
    
    /// Match an identifier exactly
    Identifier(String),
    
    /// Match a keyword exactly
    Keyword(String),
    
    /// Match the empty list
    Nil,
    
    /// Match a list with specific patterns
    List(Vec<Pattern>),
    
    /// Match a list with a pattern that can repeat (ellipsis)
    Ellipsis {
        patterns: Vec<Pattern>,
        ellipsis_pattern: Box<Pattern>,
        rest: Option<Box<Pattern>>,
    },
    
    /// Match a pair (dotted list)
    Pair {
        car: Box<Pattern>,
        cdr: Box<Pattern>,
    },
    
    /// Match any expression (wildcard)
    Wildcard,
    
    /// Match if the sub-pattern does NOT match (negative lookahead)
    Not(Box<Pattern>),
    
    /// Match if any of the alternative patterns match
    Or(Vec<Pattern>),
    
    /// Match if all patterns match the same expression
    And(Vec<Pattern>),
}

/// Bindings created during pattern matching.
#[derive(Debug, Clone, Default)]
pub struct PatternBindings {
    /// Single-value bindings (variable -> expression)
    bindings: HashMap<String, Spanned<Expr>>,
    /// Multi-value bindings for ellipsis patterns (variable -> list of expressions)
    ellipsis_bindings: HashMap<String, Vec<Spanned<Expr>>>,
}

impl PatternBindings {
    /// Creates new empty bindings.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Binds a variable to an expression.
    pub fn bind(&mut self, name: String, expr: Spanned<Expr>) {
        self.bindings.insert(name, expr);
    }
    
    /// Binds a variable to a list of expressions (for ellipsis).
    pub fn bind_ellipsis(&mut self, name: String, exprs: Vec<Spanned<Expr>>) {
        self.ellipsis_bindings.insert(name, exprs);
    }
    
    /// Gets a single binding.
    pub fn get(&self, name: &str) -> Option<&Spanned<Expr>> {
        self.bindings.get(name)
    }
    
    /// Gets an ellipsis binding.
    pub fn get_ellipsis(&self, name: &str) -> Option<&Vec<Spanned<Expr>>> {
        self.ellipsis_bindings.get(name)
    }
    
    /// Gets all single bindings.
    pub fn bindings(&self) -> &HashMap<String, Spanned<Expr>> {
        &self.bindings
    }
    
    /// Gets all ellipsis bindings.
    pub fn ellipsis_bindings(&self) -> &HashMap<String, Vec<Spanned<Expr>>> {
        &self.ellipsis_bindings
    }
    
    /// Merges another set of bindings into this one.
    pub fn merge(&mut self, other: PatternBindings) -> Result<()> {
        for (name, expr) in other.bindings {
            if self.bindings.contains_key(&name) {
                return Err(Box::new(Error::macro_error(
                    format!("Duplicate binding for variable: {name}"),
                    expr.span,
                ))))));
            }
            self.bindings.insert(name, expr);
        }
        
        for (name, exprs) in other.ellipsis_bindings {
            if self.ellipsis_bindings.contains_key(&name) {
                return Err(Box::new(Error::macro_error(
                    format!("Duplicate ellipsis binding for variable: {name}"),
                    crate::diagnostics::Span::new(0, 0),
                ))))));
            }
            self.ellipsis_bindings.insert(name, exprs);
        }
        
        Ok(())))
    }
}

impl Pattern {
    /// Creates a variable pattern.
    pub fn variable(name: impl Into<String>) -> Self {
        Pattern::Variable(name)
    }
    
    /// Creates a literal pattern.
    pub fn literal(lit: Literal) -> Self {
        Pattern::Literal(lit)
    }
    
    /// Creates an identifier pattern.
    pub fn identifier(name: impl Into<String>) -> Self {
        Pattern::Identifier(name)
    }
    
    /// Creates a list pattern.
    pub fn list(patterns: Vec<Pattern>) -> Self {
        Pattern::List(patterns)
    }
    
    /// Creates an ellipsis pattern.
    pub fn ellipsis(
        patterns: Vec<Pattern>,
        ellipsis_pattern: Pattern,
        rest: Option<Pattern>,
    ) -> Self {
        Pattern::Ellipsis {
            patterns,
            ellipsis_pattern: Box::new(ellipsis_pattern),
            rest: rest.map(Box::new),
        }
    }
    
    /// Matches this pattern against an expression.
    pub fn match_expr(&self, expr: &Spanned<Expr>) -> Result<PatternBindings> {
        let mut bindings = PatternBindings::new();
        self.match_expr_with_bindings(expr, &mut bindings)?;
        Ok(bindings)
    }
    
    /// Internal matching method that accumulates bindings.
    fn match_expr_with_bindings(
        &self,
        expr: &Spanned<Expr>,
        bindings: &mut PatternBindings,
    ) -> Result<()> {
        match (self, &expr.inner) {
            // Variable patterns bind to any expression
            (Pattern::Variable(name), _) => {
                bindings.bind(name.clone(), expr.clone());
                Ok(())
            }
            
            // Wildcard matches anything without binding
            (Pattern::Wildcard, _) => Ok(()),
            
            // Literal patterns match exact values
            (Pattern::Literal(pat_lit), Expr::Literal(expr_lit)) => {
                if pat_lit == expr_lit {
                    Ok(())))
                } else {
                    Err(Box::new(Error::macro_error(
                        format!("Literal mismatch: expected {pat_lit:?}, got {expr_lit:?}"),
                        expr.span,
                    ))))
                }
            }
            
            // Identifier patterns match exact names
            (Pattern::Identifier(pat_name), Expr::Identifier(expr_name)))) => {
                if pat_name == expr_name {
                    Ok(())))
                } else {
                    Err(Box::new(Error::macro_error(
                        format!("Identifier mismatch: expected {pat_name}, got {expr_name}"),
                        expr.span,
                    ))))
                }
            }
            
            // Keyword patterns match exact keywords
            (Pattern::Keyword(pat_kw), Expr::Keyword(expr_kw)))) => {
                if pat_kw == expr_kw {
                    Ok(())))
                } else {
                    Err(Box::new(Error::macro_error(
                        format!("Keyword mismatch: expected #{pat_kw}, got #{expr_kw}"),
                        expr.span,
                    ))))
                }
            }
            
            // Nil pattern matches empty list expressions
            (Pattern::Nil, _) => {
                if self.is_nil_expr(expr) {
                    Ok(())))
                } else {
                    Err(Box::new(Error::macro_error(
                        "Expected empty list".to_string(),
                        expr.span,
                    ))))
                }
            }
            
            // List patterns match application expressions or explicit lists
            (Pattern::List(pat_items), _) => {
                let expr_items = self.expr_to_list(expr)?;
                self.match_list_patterns(pat_items, &expr_items, bindings)
            }
            
            // Ellipsis patterns handle repeating elements
            (Pattern::Ellipsis { patterns, ellipsis_pattern, rest }, _) => {
                let expr_items = self.expr_to_list(expr)?;
                self.match_ellipsis_pattern(patterns, ellipsis_pattern, rest.as_deref(), &expr_items, bindings)
            }
            
            // Pair patterns match dotted lists
            (Pattern::Pair { car, cdr }, Expr::Pair { car: expr_car, cdr: expr_cdr }) => {
                car.match_expr_with_bindings(expr_car, bindings)?;
                cdr.match_expr_with_bindings(expr_cdr, bindings)
            }
            
            // Or patterns try each alternative
            (Pattern::Or(alternatives), _) => {
                for alt in alternatives {
                    let mut alt_bindings = bindings.clone();
                    if alt.match_expr_with_bindings(expr, &mut alt_bindings).is_ok() {
                        *bindings = alt_bindings;
                        return Ok(())));
                    }
                }
                Err(Box::new(Error::macro_error(
                    "No alternative pattern matched".to_string(),
                    expr.span,
                ))))
            }
            
            // And patterns require all to match
            (Pattern::And(conjuncts), _) => {
                for conj in conjuncts {
                    conj.match_expr_with_bindings(expr, bindings)?;
                }
                Ok(())))
            }
            
            // Not patterns require the sub-pattern to fail
            (Pattern::Not(sub_pattern), _) => {
                let mut dummy_bindings = PatternBindings::new();
                if sub_pattern.match_expr_with_bindings(expr, &mut dummy_bindings).is_ok() {
                    Err(Box::new(Error::macro_error(
                        "Negative pattern matched unexpectedly".to_string(),
                        expr.span,
                    ))))
                } else {
                    Ok(())))
                }
            }
            
            // Mismatched pattern types
            _ => Err(Box::new(Error::macro_error(
                format!("Pattern type mismatch: {self:?} vs {:?}", expr.inner),
                expr.span,
            )))),
        }
    }
    
    /// Matches a list of patterns against a list of expressions.
    fn match_list_patterns(
        &self,
        patterns: &[Pattern],
        exprs: &[Spanned<Expr>],
        bindings: &mut PatternBindings,
    ) -> Result<()> {
        if patterns.len() != exprs.len() {
            return Err(Box::new(Error::macro_error(
                format!("List length mismatch: expected {}, got {}", patterns.len(), exprs.len()))),
                crate::diagnostics::Span::new(0, 0),
            ))));
        }
        
        for (pattern, expr) in patterns.iter().zip(exprs.iter()))) {
            pattern.match_expr_with_bindings(expr, bindings)?;
        }
        
        Ok(())))
    }
    
    /// Matches an ellipsis pattern against a list of expressions.
    fn match_ellipsis_pattern(
        &self,
        pre_patterns: &[Pattern],
        ellipsis_pattern: &Pattern,
        rest_pattern: Option<&Pattern>,
        exprs: &[Spanned<Expr>],
        bindings: &mut PatternBindings,
    ) -> Result<()> {
        // Match pre-patterns
        if exprs.len() < pre_patterns.len() {
            return Err(Box::new(Error::macro_error(
                format!(
                    "Not enough expressions for pre-patterns: expected at least {}, got {}",
                    pre_patterns.len(),
                    exprs.len()
                ),
                crate::diagnostics::Span::new(0, 0),
            ))));
        }
        
        for (i, pattern) in pre_patterns.iter().enumerate() {
            pattern.match_expr_with_bindings(&exprs[i], bindings)?;
        }
        
        // Determine how many expressions the ellipsis should consume
        let rest_count = if rest_pattern.is_some() { 1 } else { 0 };
        let available_for_ellipsis = exprs.len() - pre_patterns.len() - rest_count;
        
        // Match ellipsis pattern against remaining expressions
        let ellipsis_start = pre_patterns.len();
        let ellipsis_end = ellipsis_start + available_for_ellipsis;
        
        let mut ellipsis_bindings_map: HashMap<String, Vec<Spanned<Expr>>> = HashMap::new();
        
        // Handle zero-length ellipsis matching
        if available_for_ellipsis == 0 {
            // For zero-length matches, bind all variables in the ellipsis pattern to empty lists
            let pattern_vars = ellipsis_pattern.bound_variables();
            for var in pattern_vars {
                ellipsis_bindings_map.insert(var, Vec::new())));
            }
        } else {
            // Match each expression in the ellipsis range
            for expr in exprs.iter().take(ellipsis_end).skip(ellipsis_start) {
                let mut local_bindings = PatternBindings::new();
                ellipsis_pattern.match_expr_with_bindings(expr, &mut local_bindings)?;
                
                // Collect bindings for ellipsis variables
                for (var, expr) in local_bindings.bindings() {
                    ellipsis_bindings_map
                        .entry(var.clone())))
                        .or_default()
                        .push(expr.clone())));
                }
                
                // Handle nested ellipsis bindings
                for (var, expr_list) in local_bindings.ellipsis_bindings() {
                    for expr in expr_list {
                        ellipsis_bindings_map
                            .entry(var.clone())))
                            .or_default()
                            .push(expr.clone())));
                    }
                }
            }
        }
        
        // Add ellipsis bindings to the main bindings
        for (var, expr_list) in ellipsis_bindings_map {
            bindings.bind_ellipsis(var, expr_list);
        }
        
        // Match rest pattern if present
        if let Some(rest_pat) = rest_pattern {
            if ellipsis_end < exprs.len() {
                rest_pat.match_expr_with_bindings(&exprs[ellipsis_end], bindings)?;
            } else {
                return Err(Box::new(Error::macro_error(
                    "No expression available for rest pattern".to_string(),
                    crate::diagnostics::Span::new(0, 0),
                ))));
            }
        }
        
        Ok(())))
    }
    
    /// Converts an expression to a list of sub-expressions for pattern matching.
    fn expr_to_list(&self, expr: &Spanned<Expr>) -> Result<Vec<Spanned<Expr>>> {
        match &expr.inner {
            Expr::Application { operator, operands } => {
                let mut items = vec![(**operator).clone()];
                items.extend(operands.iter().cloned())));
                Ok(items)
            }
            _ => {
                // For non-application expressions, convert to single-item list
                Ok(vec![expr.clone()])
            }
        }
    }
    
    /// Checks if an expression represents nil (empty list).
    fn is_nil_expr(&self, expr: &Spanned<Expr>) -> bool {
        matches!(expr.inner, Expr::Literal(Literal::Nil))))
            || (match &expr.inner {
                Expr::Application { operands, .. } => operands.is_empty(),
                _ => false,
            })
    }
    
    /// Gets all variables bound by this pattern.
    pub fn bound_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_variables(&mut vars);
        vars
    }
    
    /// Recursively collects all variable names in this pattern.
    fn collect_variables(&self, vars: &mut Vec<String>) {
        match self {
            Pattern::Variable(name) => vars.push(name.clone()))),
            Pattern::List(patterns) => {
                for pat in patterns {
                    pat.collect_variables(vars);
                }
            }
            Pattern::Ellipsis { patterns, ellipsis_pattern, rest } => {
                for pat in patterns {
                    pat.collect_variables(vars);
                }
                ellipsis_pattern.collect_variables(vars);
                if let Some(rest_pat) = rest {
                    rest_pat.collect_variables(vars);
                }
            }
            Pattern::Pair { car, cdr } => {
                car.collect_variables(vars);
                cdr.collect_variables(vars);
            }
            Pattern::Or(alternatives) => {
                for alt in alternatives {
                    alt.collect_variables(vars);
                }
            }
            Pattern::And(conjuncts) => {
                for conj in conjuncts {
                    conj.collect_variables(vars);
                }
            }
            Pattern::Not(sub_pattern) => {
                sub_pattern.collect_variables(vars);
            }
            _ => {} // Literals, identifiers, etc. don't bind variables
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;
    
    fn make_spanned<T>(value: T) -> Spanned<T> {
        Spanned::new(value, Span::new(0, 1))))
    }
    
    #[test]
    fn test_variable_pattern() {
        let pattern = Pattern::variable("x");
        let expr = make_spanned(Expr::Literal(Literal::Number(42.0))))));
        
        let bindings = pattern.match_expr(&expr).unwrap();
        assert!(bindings.get("x").is_some())));
    }
    
    #[test]
    fn test_literal_pattern() {
        let pattern = Pattern::literal(Literal::Number(42.0))));
        let expr = make_spanned(Expr::Literal(Literal::Number(42.0))))));
        
        assert!(pattern.match_expr(&expr).is_ok())));
        
        let wrong_expr = make_spanned(Expr::Literal(Literal::Number(43.0))))));
        assert!(pattern.match_expr(&wrong_expr).is_err())));
    }
    
    #[test]
    fn test_identifier_pattern() {
        let pattern = Pattern::identifier("foo");
        let expr = make_spanned(Expr::Identifier("foo".to_string())))));
        
        assert!(pattern.match_expr(&expr).is_ok())));
        
        let wrong_expr = make_spanned(Expr::Identifier("bar".to_string())))));
        assert!(pattern.match_expr(&wrong_expr).is_err())));
    }
    
    #[test]
    fn test_list_pattern() {
        let pattern = Pattern::list(vec![
            Pattern::identifier("if"),
            Pattern::variable("test"),
            Pattern::variable("then"),
            Pattern::variable("else"),
        ]);
        
        let expr = make_spanned(Expr::Application {
            operator: Box::new(make_spanned(Expr::Identifier("if".to_string())))))))),
            operands: vec![
                make_spanned(Expr::Identifier("condition".to_string()))))),
                make_spanned(Expr::Literal(Literal::Number(1.0)))))),
                make_spanned(Expr::Literal(Literal::Number(2.0)))))),
            ],
        });
        
        let bindings = pattern.match_expr(&expr).unwrap();
        assert!(bindings.get("test").is_some())));
        assert!(bindings.get("then").is_some())));
        assert!(bindings.get("else").is_some())));
    }
}