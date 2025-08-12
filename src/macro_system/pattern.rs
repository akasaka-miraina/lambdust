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
            
            (Pattern::Identifier(name), Expr::Identifier(expr_name)) => {
                if name == expr_name {
                    Ok(())
                } else {
                    Err(Box::new(Error::macro_error(
                        format!("Identifier mismatch: expected {name}, got {expr_name}"),
                        expr.span,
                    )))
                }
            }
            
            (Pattern::Keyword(name), Expr::Keyword(expr_name)) => {
                if name == expr_name {
                    Ok(())
                } else {
                    Err(Box::new(Error::macro_error(
                        format!("Keyword mismatch: expected {name}, got {expr_name}"),
                        expr.span,
                    )))
                }
            }
            
            (Pattern::Nil, Expr::Literal(Literal::Nil)) => Ok(()),
            
            (Pattern::List(patterns), Expr::List(elements)) => {
                self.match_list_patterns(patterns, elements, bindings)
            }
            
            (Pattern::List(patterns), Expr::Application { operator, operands }) => {
                let mut all_elements = vec![(**operator).clone()];
                all_elements.extend(operands.iter().cloned());
                self.match_list_patterns(patterns, &all_elements, bindings)
            }
            
            (Pattern::Ellipsis { patterns, ellipsis_pattern, rest }, Expr::List(elements)) => {
                self.match_ellipsis_pattern(patterns, ellipsis_pattern, rest.as_ref().map(|v| &**v), elements, bindings)
            }
            
            (Pattern::Ellipsis { patterns, ellipsis_pattern, rest }, Expr::Application { operator, operands }) => {
                let mut all_elements = vec![(**operator).clone()];
                all_elements.extend(operands.iter().cloned());
                self.match_ellipsis_pattern(patterns, ellipsis_pattern, rest.as_ref().map(|v| &**v), &all_elements, bindings)
            }
            
            (Pattern::Pair { car, cdr }, Expr::Pair { car: expr_car, cdr: expr_cdr }) => {
                car.match_expr_with_bindings(expr_car, bindings)?;
                cdr.match_expr_with_bindings(expr_cdr, bindings)?;
                Ok(())
            }
            
            _ => Err(Box::new(Error::macro_error(
                format!("Pattern type mismatch: {self:?} vs {:?}", expr.inner),
                expr.span,
            ))),
        }
    }

    /// Matches a list of patterns against a list of expressions.
    fn match_list_patterns(
        &self,
        patterns: &[Pattern],
        elements: &[Spanned<Expr>],
        bindings: &mut PatternBindings,
    ) -> Result<()> {
        if patterns.len() != elements.len() {
            return Err(Box::new(Error::macro_error(
                format!("Length mismatch: pattern has {} elements, expression has {}", 
                       patterns.len(), elements.len()),
                crate::diagnostics::Span::new(0, 0),
            )));
        }
        
        for (pattern, element) in patterns.iter().zip(elements.iter()) {
            pattern.match_expr_with_bindings(element, bindings)?;
        }
        Ok(())
    }

    /// Matches an ellipsis pattern against a list of expressions.
    fn match_ellipsis_pattern(
        &self,
        fixed_patterns: &[Pattern],
        ellipsis_pattern: &Pattern,
        rest_pattern: Option<&Pattern>,
        elements: &[Spanned<Expr>],
        bindings: &mut PatternBindings,
    ) -> Result<()> {
        // Check minimum length
        let min_length = fixed_patterns.len() + rest_pattern.map(|_| 1).unwrap_or(0);
        if elements.len() < min_length {
            return Err(Box::new(Error::macro_error(
                format!("Not enough elements: need at least {min_length}, got {}", elements.len()),
                crate::diagnostics::Span::new(0, 0),
            )));
        }
        
        // Match fixed patterns at the beginning
        for (i, pattern) in fixed_patterns.iter().enumerate() {
            pattern.match_expr_with_bindings(&elements[i], bindings)?;
        }
        
        // Determine how many elements belong to the ellipsis
        let rest_count = rest_pattern.map(|_| 1).unwrap_or(0);
        let ellipsis_end = elements.len() - rest_count;
        
        // Match ellipsis pattern for each repetition
        let mut ellipsis_matches = Vec::new();
        for element in elements.iter().take(ellipsis_end).skip(fixed_patterns.len()) {
            let mut ellipsis_bindings = PatternBindings::new();
            ellipsis_pattern.match_expr_with_bindings(element, &mut ellipsis_bindings)?;
            ellipsis_matches.push(element.clone());
            
            // Collect ellipsis variable bindings
            for var in ellipsis_pattern.bound_variables() {
                if let Some(expr) = ellipsis_bindings.get(&var) {
                    if let Some(existing_list) = bindings.ellipsis_bindings.get_mut(&var) {
                        existing_list.push(expr.clone());
                    } else {
                        bindings.bind_ellipsis(var, vec![expr.clone()]);
                    }
                }
            }
        }
        
        // Match rest pattern if present
        if let Some(rest_pat) = rest_pattern {
            if let Some(last_element) = elements.last() {
                rest_pat.match_expr_with_bindings(last_element, bindings)?;
            }
        }
        
        Ok(())
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
            _ => {}
        }
    }

    /// Computes the ellipsis depth of this pattern (how many nested ellipses).
    pub fn ellipsis_depth(&self) -> usize {
        match self {
            Pattern::Ellipsis { ellipsis_pattern, .. } => {
                1 + ellipsis_pattern.ellipsis_depth()
            }
            Pattern::List(patterns) => {
                patterns.iter().map(|p| p.ellipsis_depth()).max().unwrap_or(0)
            }
            Pattern::Pair { car, cdr } => {
                car.ellipsis_depth().max(cdr.ellipsis_depth())
            }
            Pattern::Or(alternatives) => {
                alternatives.iter().map(|p| p.ellipsis_depth()).max().unwrap_or(0)
            }
            Pattern::And(conjuncts) => {
                conjuncts.iter().map(|p| p.ellipsis_depth()).max().unwrap_or(0)
            }
            Pattern::Not(sub_pattern) => sub_pattern.ellipsis_depth(),
            _ => 0,
        }
    }

    /// SRFI-149: Computes the binding depths of all variables in this pattern
    /// Returns a map of variable name to its ellipsis depth
    pub fn variable_depths(&self) -> std::collections::HashMap<String, usize> {
        let mut depths = std::collections::HashMap::new();
        self.collect_variable_depths(&mut depths, 0);
        depths
    }

    fn collect_variable_depths(&self, depths: &mut std::collections::HashMap<String, usize>, current_depth: usize) {
        match self {
            Pattern::Variable(name) => {
                depths.insert(name.clone(), current_depth);
            }
            Pattern::List(patterns) => {
                for pattern in patterns {
                    pattern.collect_variable_depths(depths, current_depth);
                }
            }
            Pattern::Ellipsis { patterns, ellipsis_pattern, rest } => {
                for pattern in patterns {
                    pattern.collect_variable_depths(depths, current_depth);
                }
                ellipsis_pattern.collect_variable_depths(depths, current_depth + 1);
                if let Some(rest_pat) = rest {
                    rest_pat.collect_variable_depths(depths, current_depth);
                }
            }
            Pattern::Pair { car, cdr } => {
                car.collect_variable_depths(depths, current_depth);
                cdr.collect_variable_depths(depths, current_depth);
            }
            Pattern::Or(alternatives) => {
                for alt in alternatives {
                    alt.collect_variable_depths(depths, current_depth);
                }
            }
            Pattern::And(conjuncts) => {
                for conj in conjuncts {
                    conj.collect_variable_depths(depths, current_depth);
                }
            }
            Pattern::Not(sub_pattern) => {
                sub_pattern.collect_variable_depths(depths, current_depth);
            }
            _ => {}
        }
    }
}