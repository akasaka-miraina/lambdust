//! Common subexpression elimination optimization
//!
//! This module implements common subexpression elimination for Scheme expressions,
//! identifying and eliminating duplicate computations.

use crate::ast::Expr;
use crate::error::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Common subexpression eliminator
#[derive(Debug, Clone)]
pub struct CommonSubexpressionEliminator {
    /// Elimination statistics
    pub statistics: CSEStatistics,
}

/// Statistics for CSE operations
#[derive(Debug, Clone, Default)]
pub struct CSEStatistics {
    /// Total elimination attempts
    pub total_attempts: usize,
    /// Successful eliminations
    pub successful_eliminations: usize,
    /// Total time spent eliminating
    pub total_elimination_time: Duration,
    /// Number of subexpressions eliminated
    pub subexpressions_eliminated: usize,
}

/// Result of CSE operation
#[derive(Debug, Clone)]
pub struct CSEResult {
    /// The optimized expression
    pub optimized_expr: Expr,
    /// Whether any elimination was applied
    pub elimination_applied: bool,
    /// Elimination time
    pub elimination_time: Duration,
    /// Number of eliminations
    pub eliminations_count: usize,
}

/// Map of subexpressions and their frequencies
pub type SubexpressionMap = HashMap<String, SubexpressionInfo>;

/// Information about a subexpression
#[derive(Debug, Clone)]
pub struct SubexpressionInfo {
    /// The subexpression
    pub expression: Expr,
    /// Number of occurrences
    pub occurrence_count: usize,
    /// Complexity score
    pub complexity: usize,
    /// Generated variable name for replacement
    pub replacement_var: Option<String>,
}

impl CommonSubexpressionEliminator {
    /// Create a new CSE eliminator
    pub fn new() -> Self {
        Self {
            statistics: CSEStatistics::default(),
        }
    }

    /// Eliminate common subexpressions
    pub fn eliminate(&mut self, expr: &Expr) -> Result<CSEResult> {
        let start_time = Instant::now();
        self.statistics.total_attempts += 1;

        let subexpr_map = self.find_common_subexpressions(expr);
        let (optimized_expr, eliminations) = self.eliminate_subexpressions(expr, &subexpr_map);
        let elimination_applied = eliminations > 0;

        if elimination_applied {
            self.statistics.successful_eliminations += 1;
            self.statistics.subexpressions_eliminated += eliminations;
        }

        let elimination_time = start_time.elapsed();
        self.statistics.total_elimination_time += elimination_time;

        Ok(CSEResult {
            optimized_expr,
            elimination_applied,
            elimination_time,
            eliminations_count: eliminations,
        })
    }

    /// Find common subexpressions in the expression
    pub fn find_common_subexpressions(&self, expr: &Expr) -> SubexpressionMap {
        let mut occurrence_map: HashMap<String, (Expr, usize)> = HashMap::new();
        self.count_subexpressions(expr, &mut occurrence_map);

        let mut subexpr_map = SubexpressionMap::new();
        let mut var_counter = 0;

        for (expr_key, (expression, count)) in occurrence_map {
            if count > 1 && self.is_worth_eliminating(&expression) {
                let replacement_var = format!("cse_var_{}", var_counter);
                var_counter += 1;

                subexpr_map.insert(expr_key, SubexpressionInfo {
                    expression: expression.clone(),
                    occurrence_count: count,
                    complexity: self.calculate_complexity(&expression),
                    replacement_var: Some(replacement_var),
                });
            }
        }

        subexpr_map
    }

    /// Count occurrences of subexpressions
    fn count_subexpressions(&self, expr: &Expr, occurrence_map: &mut HashMap<String, (Expr, usize)>) {
        let expr_key = self.expression_key(expr);
        
        // Count this expression
        let entry = occurrence_map.entry(expr_key).or_insert((expr.clone(), 0));
        entry.1 += 1;

        // Recursively count subexpressions
        match expr {
            Expr::List(exprs) | Expr::Vector(exprs) => {
                for sub_expr in exprs {
                    self.count_subexpressions(sub_expr, occurrence_map);
                }
            }
            Expr::Quote(inner) | Expr::Quasiquote(inner) | Expr::Unquote(inner) | Expr::UnquoteSplicing(inner) => {
                self.count_subexpressions(inner, occurrence_map);
            }
            Expr::DottedList(exprs, tail) => {
                for sub_expr in exprs {
                    self.count_subexpressions(sub_expr, occurrence_map);
                }
                self.count_subexpressions(tail, occurrence_map);
            }
            _ => {} // Literals and variables have no subexpressions
        }
    }

    /// Generate a key for an expression
    fn expression_key(&self, expr: &Expr) -> String {
        format!("{:?}", expr)
    }

    /// Check if a subexpression is worth eliminating
    fn is_worth_eliminating(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Literal(_) | Expr::Variable(_) => false, // Too simple
            Expr::Quote(_) => false, // Usually not computed
            Expr::List(exprs) => {
                // Only eliminate non-trivial expressions
                exprs.len() >= 2 && self.calculate_complexity(expr) >= 3
            }
            Expr::HygienicVariable(_) => false, // Variables are simple
            Expr::Quasiquote(_) | Expr::Unquote(_) | Expr::UnquoteSplicing(_) => false, // Meta-programming constructs
            Expr::Vector(_) => true, // Vectors might be worth eliminating
            Expr::DottedList(_, _) => false, // Dotted lists are usually simple
        }
    }

    /// Calculate complexity of an expression
    fn calculate_complexity(&self, expr: &Expr) -> usize {
        match expr {
            Expr::Literal(_) | Expr::Variable(_) => 1,
            Expr::Quote(inner) => 1 + self.calculate_complexity(inner),
            Expr::List(exprs) => {
                1 + exprs.iter().map(|e| self.calculate_complexity(e)).sum::<usize>()
            }
            Expr::HygienicVariable(_) => 1, // Variables have complexity 1
            Expr::Quasiquote(inner) => 1 + self.calculate_complexity(inner),
            Expr::Unquote(inner) => 1 + self.calculate_complexity(inner),
            Expr::UnquoteSplicing(inner) => 1 + self.calculate_complexity(inner),
            Expr::Vector(exprs) => {
                1 + exprs.iter().map(|e| self.calculate_complexity(e)).sum::<usize>()
            }
            Expr::DottedList(exprs, tail) => {
                1 + exprs.iter().map(|e| self.calculate_complexity(e)).sum::<usize>()
                    + self.calculate_complexity(tail)
            }
        }
    }

    /// Eliminate subexpressions using the substitution map
    fn eliminate_subexpressions(&self, expr: &Expr, subexpr_map: &SubexpressionMap) -> (Expr, usize) {
        let expr_key = self.expression_key(expr);
        
        // Check if this expression should be replaced
        if let Some(info) = subexpr_map.get(&expr_key) {
            if let Some(ref replacement_var) = info.replacement_var {
                return (Expr::Variable(replacement_var.clone()), 1);
            }
        }

        // Recursively eliminate in subexpressions
        match expr {
            Expr::Literal(_) | Expr::Variable(_) | Expr::HygienicVariable(_) => (expr.clone(), 0),
            Expr::Quote(inner) | Expr::Quasiquote(inner) | Expr::Unquote(inner) | Expr::UnquoteSplicing(inner) => {
                let (eliminated_inner, count) = self.eliminate_subexpressions(inner, subexpr_map);
                match expr {
                    Expr::Quote(_) => (Expr::Quote(Box::new(eliminated_inner)), count),
                    Expr::Quasiquote(_) => (Expr::Quasiquote(Box::new(eliminated_inner)), count),
                    Expr::Unquote(_) => (Expr::Unquote(Box::new(eliminated_inner)), count),
                    Expr::UnquoteSplicing(_) => (Expr::UnquoteSplicing(Box::new(eliminated_inner)), count),
                    _ => unreachable!(),
                }
            }
            Expr::List(exprs) | Expr::Vector(exprs) => {
                let mut eliminated_exprs = Vec::new();
                let mut total_count = 0;

                for sub_expr in exprs {
                    let (eliminated, count) = self.eliminate_subexpressions(sub_expr, subexpr_map);
                    eliminated_exprs.push(eliminated);
                    total_count += count;
                }

                match expr {
                    Expr::List(_) => (Expr::List(eliminated_exprs), total_count),
                    Expr::Vector(_) => (Expr::Vector(eliminated_exprs), total_count),
                    _ => unreachable!(),
                }
            }
            Expr::DottedList(exprs, tail) => {
                let mut eliminated_exprs = Vec::new();
                let mut total_count = 0;

                for sub_expr in exprs {
                    let (eliminated, count) = self.eliminate_subexpressions(sub_expr, subexpr_map);
                    eliminated_exprs.push(eliminated);
                    total_count += count;
                }

                let (eliminated_tail, tail_count) = self.eliminate_subexpressions(tail, subexpr_map);
                total_count += tail_count;

                (Expr::DottedList(eliminated_exprs, Box::new(eliminated_tail)), total_count)
            }
        }
    }

    /// Generate let bindings for eliminated subexpressions
    pub fn generate_let_bindings(&self, subexpr_map: &SubexpressionMap) -> Vec<(String, Expr)> {
        let mut bindings = Vec::new();
        
        for info in subexpr_map.values() {
            if let Some(ref var_name) = info.replacement_var {
                bindings.push((var_name.clone(), info.expression.clone()));
            }
        }

        // Sort by complexity (more complex first)
        bindings.sort_by(|a, b| {
            let complexity_a = self.calculate_complexity(&a.1);
            let complexity_b = self.calculate_complexity(&b.1);
            complexity_b.cmp(&complexity_a)
        });

        bindings
    }

    /// Wrap expression with let bindings
    pub fn wrap_with_let_bindings(&self, expr: Expr, bindings: Vec<(String, Expr)>) -> Expr {
        if bindings.is_empty() {
            return expr;
        }

        let binding_exprs: Vec<Expr> = bindings
            .into_iter()
            .map(|(var, value)| {
                Expr::List(vec![Expr::Variable(var), value])
            })
            .collect();

        Expr::List(vec![
            Expr::Variable("let".to_string()),
            Expr::List(binding_exprs),
            expr,
        ])
    }

    /// Perform complete CSE optimization with let bindings
    pub fn optimize_with_let(&mut self, expr: &Expr) -> Result<CSEResult> {
        let start_time = Instant::now();
        self.statistics.total_attempts += 1;

        let subexpr_map = self.find_common_subexpressions(expr);
        let (eliminated_expr, eliminations) = self.eliminate_subexpressions(expr, &subexpr_map);
        
        let optimized_expr = if eliminations > 0 {
            let bindings = self.generate_let_bindings(&subexpr_map);
            self.wrap_with_let_bindings(eliminated_expr, bindings)
        } else {
            eliminated_expr
        };

        let elimination_applied = eliminations > 0;

        if elimination_applied {
            self.statistics.successful_eliminations += 1;
            self.statistics.subexpressions_eliminated += eliminations;
        }

        let elimination_time = start_time.elapsed();
        self.statistics.total_elimination_time += elimination_time;

        Ok(CSEResult {
            optimized_expr,
            elimination_applied,
            elimination_time,
            eliminations_count: eliminations,
        })
    }

    /// Get CSE statistics
    pub fn get_statistics(&self) -> &CSEStatistics {
        &self.statistics
    }

    /// Reset statistics
    pub fn reset_statistics(&mut self) {
        self.statistics = CSEStatistics::default();
    }
}

impl Default for CommonSubexpressionEliminator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_cse_eliminator_creation() {
        let eliminator = CommonSubexpressionEliminator::new();
        assert_eq!(eliminator.get_statistics().total_attempts, 0);
    }

    #[test]
    fn test_find_common_subexpressions() {
        let eliminator = CommonSubexpressionEliminator::new();
        
        // (+ (+ x y) (* (+ x y) 2))
        let common_subexpr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
        ]);
        
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            common_subexpr.clone(),
            Expr::List(vec![
                Expr::Variable("*".to_string()),
                common_subexpr.clone(),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            ]),
        ]);

        let subexpr_map = eliminator.find_common_subexpressions(&expr);
        
        // Should find the common subexpression (+ x y)
        assert!(!subexpr_map.is_empty());
        
        let common_key = eliminator.expression_key(&common_subexpr);
        assert!(subexpr_map.contains_key(&common_key));
        assert_eq!(subexpr_map[&common_key].occurrence_count, 2);
    }

    #[test]
    fn test_eliminate_common_subexpressions() {
        let mut eliminator = CommonSubexpressionEliminator::new();
        
        // (+ (+ x y) (* (+ x y) 2))
        let common_subexpr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
        ]);
        
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            common_subexpr.clone(),
            Expr::List(vec![
                Expr::Variable("*".to_string()),
                common_subexpr.clone(),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            ]),
        ]);

        let result = eliminator.eliminate(&expr).unwrap();
        assert!(result.elimination_applied);
        assert_eq!(result.eliminations_count, 2);
    }

    #[test]
    fn test_optimize_with_let_bindings() {
        let mut eliminator = CommonSubexpressionEliminator::new();
        
        // (+ (+ x y) (* (+ x y) 2))
        let common_subexpr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
        ]);
        
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            common_subexpr.clone(),
            Expr::List(vec![
                Expr::Variable("*".to_string()),
                common_subexpr.clone(),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            ]),
        ]);

        let result = eliminator.optimize_with_let(&expr).unwrap();
        assert!(result.elimination_applied);
        
        // Should generate a let expression
        match result.optimized_expr {
            Expr::List(exprs) if exprs.len() == 3 => {
                match &exprs[0] {
                    Expr::Variable(op) if op == "let" => {},
                    _ => panic!("Expected let expression"),
                }
            },
            _ => panic!("Expected let expression with 3 elements"),
        }
    }

    #[test]
    fn test_complexity_calculation() {
        let eliminator = CommonSubexpressionEliminator::new();
        
        let simple_expr = Expr::Variable("x".to_string());
        assert_eq!(eliminator.calculate_complexity(&simple_expr), 1);
        
        let complex_expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Variable("x".to_string()),
            Expr::List(vec![
                Expr::Variable("*".to_string()),
                Expr::Variable("y".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            ]),
        ]);
        assert_eq!(eliminator.calculate_complexity(&complex_expr), 6);
    }

    #[test]
    fn test_no_common_subexpressions() {
        let mut eliminator = CommonSubexpressionEliminator::new();
        
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
        ]);

        let result = eliminator.eliminate(&expr).unwrap();
        assert!(!result.elimination_applied);
        assert_eq!(result.eliminations_count, 0);
    }
}