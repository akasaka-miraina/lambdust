//! Expression rewriting optimization
//!
//! This module implements expression rewriting for Scheme expressions,
//! applying algebraic simplifications and pattern-based transformations.

use crate::ast::{Expr, Literal};
use crate::error::{LambdustError, Result};
use crate::lexer::SchemeNumber;
use std::time::{Duration, Instant};

/// Expression rewriter for algebraic simplifications
#[derive(Debug, Clone)]
pub struct ExpressionRewriter {
    /// Rewriting rules
    pub rules: Vec<RewriteRule>,
    /// Rewriting statistics
    pub statistics: RewritingStatistics,
}

/// Statistics for expression rewriting operations
#[derive(Debug, Clone, Default)]
pub struct RewritingStatistics {
    /// Total rewriting attempts
    pub total_attempts: usize,
    /// Successful rewrites
    pub successful_rewrites: usize,
    /// Total time spent rewriting
    pub total_rewriting_time: Duration,
    /// Rules applied by name
    pub rules_applied: std::collections::HashMap<String, usize>,
}

/// Result of expression rewriting operation
#[derive(Debug, Clone)]
pub struct RewriteResult {
    /// The rewritten expression
    pub rewritten_expr: Expr,
    /// Whether any rewriting was applied
    pub rewrite_applied: bool,
    /// Rewriting time
    pub rewriting_time: Duration,
    /// Number of rules applied
    pub rules_applied_count: usize,
    /// Names of applied rules
    pub applied_rule_names: Vec<String>,
}

/// A rewrite rule for expression transformation
#[derive(Debug, Clone)]
pub struct RewriteRule {
    /// Name of the rule
    pub name: String,
    /// Pattern to match
    pub pattern: RewritePattern,
    /// Replacement template
    pub replacement: RewriteReplacement,
    /// Conditions for applying the rule
    pub conditions: Vec<RewriteCondition>,
    /// Priority of the rule (higher = applied first)
    pub priority: i32,
}

/// Pattern for matching expressions
#[derive(Debug, Clone)]
pub enum RewritePattern {
    /// Match any expression
    Any,
    /// Match specific literal
    Literal(Literal),
    /// Match specific variable
    Variable(String),
    /// Match variable with any name
    AnyVariable,
    /// Match list with specific structure
    List(Vec<RewritePattern>),
    /// Match list with operator and any number of operands
    ListWithOperator(String, Box<RewritePattern>),
    /// Capture variable for use in replacement
    Capture(String, Box<RewritePattern>),
}

/// Replacement template for generating new expressions
#[derive(Debug, Clone)]
pub enum RewriteReplacement {
    /// Use captured variable
    CapturedVar(String),
    /// Literal replacement
    Literal(Literal),
    /// Variable replacement
    Variable(String),
    /// List replacement
    List(Vec<RewriteReplacement>),
    /// Conditional replacement
    Conditional(Box<RewriteCondition>, Box<RewriteReplacement>, Box<RewriteReplacement>),
}

/// Condition for applying rewrite rules
#[derive(Debug, Clone)]
pub enum RewriteCondition {
    /// Always true
    Always,
    /// Check if captured variable is literal
    IsLiteral(String),
    /// Check if captured variable is number
    IsNumber(String),
    /// Check if captured variable equals specific value
    Equals(String, Literal),
    /// Check if captured variable is zero
    IsZero(String),
    /// Check if captured variable is one
    IsOne(String),
    /// Logical AND of conditions
    And(Vec<RewriteCondition>),
    /// Logical OR of conditions
    Or(Vec<RewriteCondition>),
}

impl ExpressionRewriter {
    /// Create a new expression rewriter with default rules
    pub fn new() -> Self {
        let mut rewriter = Self {
            rules: Vec::new(),
            statistics: RewritingStatistics::default(),
        };
        rewriter.add_default_rules();
        rewriter
    }

    /// Add default algebraic rewriting rules
    fn add_default_rules(&mut self) {
        // Identity rules
        self.add_rule(RewriteRule {
            name: "add_zero".to_string(),
            pattern: RewritePattern::List(vec![
                RewritePattern::Variable("+".to_string()),
                RewritePattern::Capture("x".to_string(), Box::new(RewritePattern::Any)),
                RewritePattern::Capture("zero".to_string(), Box::new(RewritePattern::Any)),
            ]),
            replacement: RewriteReplacement::CapturedVar("x".to_string()),
            conditions: vec![RewriteCondition::IsZero("zero".to_string())],
            priority: 10,
        });

        self.add_rule(RewriteRule {
            name: "multiply_one".to_string(),
            pattern: RewritePattern::List(vec![
                RewritePattern::Variable("*".to_string()),
                RewritePattern::Capture("x".to_string(), Box::new(RewritePattern::Any)),
                RewritePattern::Capture("one".to_string(), Box::new(RewritePattern::Any)),
            ]),
            replacement: RewriteReplacement::CapturedVar("x".to_string()),
            conditions: vec![RewriteCondition::IsOne("one".to_string())],
            priority: 10,
        });

        // Absorption rules
        self.add_rule(RewriteRule {
            name: "multiply_zero".to_string(),
            pattern: RewritePattern::List(vec![
                RewritePattern::Variable("*".to_string()),
                RewritePattern::Capture("x".to_string(), Box::new(RewritePattern::Any)),
                RewritePattern::Capture("zero".to_string(), Box::new(RewritePattern::Any)),
            ]),
            replacement: RewriteReplacement::CapturedVar("zero".to_string()),
            conditions: vec![RewriteCondition::IsZero("zero".to_string())],
            priority: 10,
        });

        // Boolean simplifications
        self.add_rule(RewriteRule {
            name: "and_true".to_string(),
            pattern: RewritePattern::List(vec![
                RewritePattern::Variable("and".to_string()),
                RewritePattern::Capture("x".to_string(), Box::new(RewritePattern::Any)),
                RewritePattern::Literal(Literal::Boolean(true)),
            ]),
            replacement: RewriteReplacement::CapturedVar("x".to_string()),
            conditions: vec![RewriteCondition::Always],
            priority: 5,
        });

        self.add_rule(RewriteRule {
            name: "or_false".to_string(),
            pattern: RewritePattern::List(vec![
                RewritePattern::Variable("or".to_string()),
                RewritePattern::Capture("x".to_string(), Box::new(RewritePattern::Any)),
                RewritePattern::Literal(Literal::Boolean(false)),
            ]),
            replacement: RewriteReplacement::CapturedVar("x".to_string()),
            conditions: vec![RewriteCondition::Always],
            priority: 5,
        });

        // Double negation
        self.add_rule(RewriteRule {
            name: "double_not".to_string(),
            pattern: RewritePattern::List(vec![
                RewritePattern::Variable("not".to_string()),
                RewritePattern::List(vec![
                    RewritePattern::Variable("not".to_string()),
                    RewritePattern::Capture("x".to_string(), Box::new(RewritePattern::Any)),
                ]),
            ]),
            replacement: RewriteReplacement::CapturedVar("x".to_string()),
            conditions: vec![RewriteCondition::Always],
            priority: 5,
        });
    }

    /// Add a rewrite rule
    pub fn add_rule(&mut self, rule: RewriteRule) {
        // Insert rule in priority order
        let mut insert_position = None;
        for (i, existing_rule) in self.rules.iter().enumerate() {
            if rule.priority > existing_rule.priority {
                insert_position = Some(i);
                break;
            }
        }
        
        if let Some(pos) = insert_position {
            self.rules.insert(pos, rule);
        } else {
            self.rules.push(rule);
        }
    }

    /// Rewrite an expression using all applicable rules
    pub fn rewrite(&mut self, expr: &Expr) -> Result<RewriteResult> {
        let start_time = Instant::now();
        self.statistics.total_attempts += 1;

        let (rewritten_expr, rules_applied, applied_rule_names) = self.rewrite_expression(expr)?;
        let rewrite_applied = rules_applied > 0;

        if rewrite_applied {
            self.statistics.successful_rewrites += 1;
            for rule_name in &applied_rule_names {
                *self.statistics.rules_applied.entry(rule_name.clone()).or_insert(0) += 1;
            }
        }

        let rewriting_time = start_time.elapsed();
        self.statistics.total_rewriting_time += rewriting_time;

        Ok(RewriteResult {
            rewritten_expr,
            rewrite_applied,
            rewriting_time,
            rules_applied_count: rules_applied,
            applied_rule_names,
        })
    }

    /// Rewrite an expression recursively
    fn rewrite_expression(&self, expr: &Expr) -> Result<(Expr, usize, Vec<String>)> {
        let mut current_expr = expr.clone();
        let mut total_rules_applied = 0;
        let mut all_applied_rules = Vec::new();
        let mut changed = true;

        // Apply rules until no more changes
        while changed {
            changed = false;
            
            // Try to apply each rule
            for rule in &self.rules {
                if let Some((_new_expr, captures)) = self.try_match_pattern(&rule.pattern, &current_expr)? {
                    if self.check_conditions(&rule.conditions, &captures) {
                        let replacement_expr = self.apply_replacement(&rule.replacement, &captures)?;
                        if !self.expressions_equal(&current_expr, &replacement_expr) {
                            current_expr = replacement_expr;
                            total_rules_applied += 1;
                            all_applied_rules.push(rule.name.clone());
                            changed = true;
                            break; // Start over with highest priority rules
                        }
                    }
                }
            }
        }

        // Recursively rewrite subexpressions
        match &current_expr {
            Expr::List(exprs) => {
                let mut rewritten_exprs = Vec::new();
                let mut sub_rules_applied = 0;
                let mut sub_applied_rules = Vec::new();

                for expr in exprs {
                    let (rewritten, rules, rule_names) = self.rewrite_expression(expr)?;
                    rewritten_exprs.push(rewritten);
                    sub_rules_applied += rules;
                    sub_applied_rules.extend(rule_names);
                }

                if sub_rules_applied > 0 {
                    current_expr = Expr::List(rewritten_exprs);
                    total_rules_applied += sub_rules_applied;
                    all_applied_rules.extend(sub_applied_rules);
                }
            }
            Expr::Quote(inner) => {
                let (rewritten_inner, rules, rule_names) = self.rewrite_expression(inner)?;
                if rules > 0 {
                    current_expr = Expr::Quote(Box::new(rewritten_inner));
                    total_rules_applied += rules;
                    all_applied_rules.extend(rule_names);
                }
            }
            _ => {} // Literals and variables are already handled
        }

        Ok((current_expr, total_rules_applied, all_applied_rules))
    }

    /// Try to match a pattern against an expression
    fn try_match_pattern(&self, pattern: &RewritePattern, expr: &Expr) -> Result<Option<(Expr, std::collections::HashMap<String, Expr>)>> {
        let mut captures = std::collections::HashMap::new();
        
        if self.match_pattern_impl(pattern, expr, &mut captures) {
            Ok(Some((expr.clone(), captures)))
        } else {
            Ok(None)
        }
    }

    /// Implementation of pattern matching
    fn match_pattern_impl(&self, pattern: &RewritePattern, expr: &Expr, captures: &mut std::collections::HashMap<String, Expr>) -> bool {
        match pattern {
            RewritePattern::Any => true,
            RewritePattern::Literal(lit) => {
                matches!(expr, Expr::Literal(expr_lit) if expr_lit == lit)
            }
            RewritePattern::Variable(var) => {
                matches!(expr, Expr::Variable(expr_var) if expr_var == var)
            }
            RewritePattern::AnyVariable => {
                matches!(expr, Expr::Variable(_))
            }
            RewritePattern::List(patterns) => {
                if let Expr::List(exprs) = expr {
                    if patterns.len() != exprs.len() {
                        return false;
                    }
                    for (pattern, expr) in patterns.iter().zip(exprs.iter()) {
                        if !self.match_pattern_impl(pattern, expr, captures) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            RewritePattern::ListWithOperator(op, operand_pattern) => {
                if let Expr::List(exprs) = expr {
                    if exprs.is_empty() {
                        return false;
                    }
                    if let Expr::Variable(expr_op) = &exprs[0] {
                        if expr_op == op {
                            for operand in &exprs[1..] {
                                if !self.match_pattern_impl(operand_pattern, operand, captures) {
                                    return false;
                                }
                            }
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            RewritePattern::Capture(name, inner_pattern) => {
                if self.match_pattern_impl(inner_pattern, expr, captures) {
                    captures.insert(name.clone(), expr.clone());
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Check if all conditions are satisfied
    fn check_conditions(&self, conditions: &[RewriteCondition], captures: &std::collections::HashMap<String, Expr>) -> bool {
        conditions.iter().all(|condition| self.check_condition(condition, captures))
    }

    /// Check a single condition
    fn check_condition(&self, condition: &RewriteCondition, captures: &std::collections::HashMap<String, Expr>) -> bool {
        match condition {
            RewriteCondition::Always => true,
            RewriteCondition::IsLiteral(var) => {
                captures.get(var).map_or(false, |expr| matches!(expr, Expr::Literal(_)))
            }
            RewriteCondition::IsNumber(var) => {
                captures.get(var).map_or(false, |expr| {
                    matches!(expr, Expr::Literal(Literal::Number(_)))
                })
            }
            RewriteCondition::Equals(var, expected) => {
                captures.get(var).map_or(false, |expr| {
                    matches!(expr, Expr::Literal(lit) if lit == expected)
                })
            }
            RewriteCondition::IsZero(var) => {
                captures.get(var).map_or(false, |expr| {
                    match expr {
                        Expr::Literal(Literal::Number(SchemeNumber::Integer(0))) => true,
                        Expr::Literal(Literal::Number(SchemeNumber::Real(x))) => *x == 0.0,
                        Expr::Literal(Literal::Number(SchemeNumber::Rational(num, _))) => *num == 0,
                        _ => false,
                    }
                })
            }
            RewriteCondition::IsOne(var) => {
                captures.get(var).map_or(false, |expr| {
                    match expr {
                        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))) => true,
                        Expr::Literal(Literal::Number(SchemeNumber::Real(x))) => *x == 1.0,
                        Expr::Literal(Literal::Number(SchemeNumber::Rational(num, den))) => *num == *den,
                        _ => false,
                    }
                })
            }
            RewriteCondition::And(conditions) => {
                conditions.iter().all(|cond| self.check_condition(cond, captures))
            }
            RewriteCondition::Or(conditions) => {
                conditions.iter().any(|cond| self.check_condition(cond, captures))
            }
        }
    }

    /// Apply replacement template with captured variables
    fn apply_replacement(&self, replacement: &RewriteReplacement, captures: &std::collections::HashMap<String, Expr>) -> Result<Expr> {
        match replacement {
            RewriteReplacement::CapturedVar(var) => {
                captures.get(var).cloned()
                    .ok_or_else(|| LambdustError::runtime_error(format!("Captured variable '{}' not found", var)))
            }
            RewriteReplacement::Literal(lit) => Ok(Expr::Literal(lit.clone())),
            RewriteReplacement::Variable(var) => Ok(Expr::Variable(var.clone())),
            RewriteReplacement::List(replacements) => {
                let mut exprs = Vec::new();
                for replacement in replacements {
                    exprs.push(self.apply_replacement(replacement, captures)?);
                }
                Ok(Expr::List(exprs))
            }
            RewriteReplacement::Conditional(condition, then_replacement, else_replacement) => {
                if self.check_condition(condition, captures) {
                    self.apply_replacement(then_replacement, captures)
                } else {
                    self.apply_replacement(else_replacement, captures)
                }
            }
        }
    }

    /// Check if two expressions are equal
    fn expressions_equal(&self, a: &Expr, b: &Expr) -> bool {
        match (a, b) {
            (Expr::Literal(lit_a), Expr::Literal(lit_b)) => lit_a == lit_b,
            (Expr::Variable(var_a), Expr::Variable(var_b)) => var_a == var_b,
            (Expr::Quote(inner_a), Expr::Quote(inner_b)) => {
                self.expressions_equal(inner_a, inner_b)
            }
            (Expr::List(exprs_a), Expr::List(exprs_b)) => {
                exprs_a.len() == exprs_b.len() 
                    && exprs_a.iter().zip(exprs_b.iter())
                        .all(|(a, b)| self.expressions_equal(a, b))
            }
            _ => false,
        }
    }

    /// Get rewriting statistics
    pub fn get_statistics(&self) -> &RewritingStatistics {
        &self.statistics
    }

    /// Reset statistics
    pub fn reset_statistics(&mut self) {
        self.statistics = RewritingStatistics::default();
    }

    /// Get number of rules
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for ExpressionRewriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_rewriter_creation() {
        let rewriter = ExpressionRewriter::new();
        assert!(rewriter.rule_count() > 0);
        assert_eq!(rewriter.get_statistics().total_attempts, 0);
    }

    #[test]
    fn test_add_zero_rewrite() {
        let mut rewriter = ExpressionRewriter::new();
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
        ]);

        let result = rewriter.rewrite(&expr).unwrap();
        assert!(result.rewrite_applied);
        assert_eq!(result.rules_applied_count, 1);
        
        match result.rewritten_expr {
            Expr::Variable(var) if var == "x" => {},
            _ => panic!("Expected rewritten result to be 'x'"),
        }
    }

    #[test]
    fn test_multiply_one_rewrite() {
        let mut rewriter = ExpressionRewriter::new();
        let expr = Expr::List(vec![
            Expr::Variable("*".to_string()),
            Expr::Variable("y".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        ]);

        let result = rewriter.rewrite(&expr).unwrap();
        assert!(result.rewrite_applied);
        
        match result.rewritten_expr {
            Expr::Variable(var) if var == "y" => {},
            _ => panic!("Expected rewritten result to be 'y'"),
        }
    }

    #[test]
    fn test_multiply_zero_rewrite() {
        let mut rewriter = ExpressionRewriter::new();
        let expr = Expr::List(vec![
            Expr::Variable("*".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
        ]);

        let result = rewriter.rewrite(&expr).unwrap();
        assert!(result.rewrite_applied);
        
        match result.rewritten_expr {
            Expr::Literal(Literal::Number(SchemeNumber::Integer(0))) => {},
            _ => panic!("Expected rewritten result to be 0"),
        }
    }

    #[test]
    fn test_double_not_rewrite() {
        let mut rewriter = ExpressionRewriter::new();
        let expr = Expr::List(vec![
            Expr::Variable("not".to_string()),
            Expr::List(vec![
                Expr::Variable("not".to_string()),
                Expr::Variable("x".to_string()),
            ]),
        ]);

        let result = rewriter.rewrite(&expr).unwrap();
        assert!(result.rewrite_applied);
        
        match result.rewritten_expr {
            Expr::Variable(var) if var == "x" => {},
            _ => panic!("Expected rewritten result to be 'x'"),
        }
    }

    #[test]
    fn test_no_applicable_rules() {
        let mut rewriter = ExpressionRewriter::new();
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
        ]);

        let result = rewriter.rewrite(&expr).unwrap();
        assert!(!result.rewrite_applied);
        assert_eq!(result.rules_applied_count, 0);
    }
}