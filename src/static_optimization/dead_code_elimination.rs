//! Dead code elimination optimization
//!
//! This module implements dead code elimination for Scheme expressions,
//! removing unreachable code paths and unused expressions.

use crate::ast::{Expr, Literal};
use crate::error::Result;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Dead code eliminator for removing unreachable code
#[derive(Debug, Clone)]
pub struct DeadCodeEliminator {
    /// Elimination statistics
    pub statistics: EliminationStatistics,
}

/// Statistics for dead code elimination operations
#[derive(Debug, Clone, Default)]
pub struct EliminationStatistics {
    /// Total elimination attempts
    pub total_attempts: usize,
    /// Successful eliminations
    pub successful_eliminations: usize,
    /// Total time spent eliminating
    pub total_elimination_time: Duration,
    /// Number of unreachable expressions eliminated
    pub unreachable_expressions_eliminated: usize,
    /// Number of unused variables eliminated
    pub unused_variables_eliminated: usize,
    /// Number of redundant conditionals eliminated
    pub redundant_conditionals_eliminated: usize,
}

/// Result of dead code elimination operation
#[derive(Debug, Clone)]
pub struct DeadCodeEliminationResult {
    /// The optimized expression
    pub optimized_expr: Expr,
    /// Whether any elimination was applied
    pub elimination_applied: bool,
    /// Elimination time
    pub elimination_time: Duration,
    /// Number of dead code blocks eliminated
    pub dead_code_eliminated: usize,
}

/// Analysis result for dead code detection
#[derive(Debug, Clone)]
pub struct DeadCodeAnalysis {
    /// Unreachable expressions
    pub unreachable_expressions: Vec<UnreachableCode>,
    /// Unused variables
    pub unused_variables: HashSet<String>,
    /// Redundant conditionals
    pub redundant_conditionals: Vec<RedundantConditional>,
    /// Overall analysis confidence
    pub analysis_confidence: f64,
}

/// Information about unreachable code
#[derive(Debug, Clone)]
pub struct UnreachableCode {
    /// Description of the unreachable code
    pub description: String,
    /// The unreachable expression
    pub expression: Expr,
    /// Reason why it's unreachable
    pub reason: UnreachabilityReason,
}

/// Reasons why code might be unreachable
#[derive(Debug, Clone)]
pub enum UnreachabilityReason {
    /// After a return statement
    AfterReturn,
    /// In an always-false conditional
    AlwaysFalseConditional,
    /// In an always-true else branch
    AlwaysTrueElse,
    /// After an unconditional jump
    AfterUnconditionalJump,
    /// Dead loop body
    DeadLoopBody,
}

/// Information about redundant conditionals
#[derive(Debug, Clone)]
pub struct RedundantConditional {
    /// Description of the redundancy
    pub description: String,
    /// The conditional expression
    pub conditional: Expr,
    /// Simplified form
    pub simplified: Expr,
    /// Type of redundancy
    pub redundancy_type: ConditionalRedundancy,
}

/// Types of conditional redundancy
#[derive(Debug, Clone)]
pub enum ConditionalRedundancy {
    /// Always true condition
    AlwaysTrue,
    /// Always false condition
    AlwaysFalse,
    /// Identical branches
    IdenticalBranches,
    /// Nested identical conditionals
    NestedIdentical,
}

impl DeadCodeEliminator {
    /// Create a new dead code eliminator
    pub fn new() -> Self {
        Self {
            statistics: EliminationStatistics::default(),
        }
    }

    /// Eliminate dead code in an expression
    pub fn eliminate(&mut self, expr: &Expr) -> Result<DeadCodeEliminationResult> {
        let start_time = Instant::now();
        self.statistics.total_attempts += 1;

        let analysis = self.analyze_dead_code(expr)?;
        let (optimized_expr, dead_code_eliminated) = self.eliminate_expression(expr, &analysis)?;
        let elimination_applied = dead_code_eliminated > 0;

        if elimination_applied {
            self.statistics.successful_eliminations += 1;
            self.statistics.unreachable_expressions_eliminated += analysis.unreachable_expressions.len();
            self.statistics.unused_variables_eliminated += analysis.unused_variables.len();
            self.statistics.redundant_conditionals_eliminated += analysis.redundant_conditionals.len();
        }

        let elimination_time = start_time.elapsed();
        self.statistics.total_elimination_time += elimination_time;

        Ok(DeadCodeEliminationResult {
            optimized_expr,
            elimination_applied,
            elimination_time,
            dead_code_eliminated,
        })
    }

    /// Analyze expression for dead code
    pub fn analyze(&self, expr: &Expr) -> Result<DeadCodeAnalysis> {
        self.analyze_dead_code(expr)
    }

    /// Analyze dead code in an expression
    fn analyze_dead_code(&self, expr: &Expr) -> Result<DeadCodeAnalysis> {
        let mut analysis = DeadCodeAnalysis {
            unreachable_expressions: Vec::new(),
            unused_variables: HashSet::new(),
            redundant_conditionals: Vec::new(),
            analysis_confidence: 1.0,
        };

        self.analyze_expression(expr, &mut analysis)?;
        Ok(analysis)
    }

    /// Analyze a single expression for dead code patterns
    fn analyze_expression(&self, expr: &Expr, analysis: &mut DeadCodeAnalysis) -> Result<()> {
        match expr {
            Expr::Literal(_) => {
                // Literals don't contain dead code
                Ok(())
            }
            Expr::Variable(_) => {
                // Variables are analyzed in context
                Ok(())
            }
            Expr::HygienicVariable(_) => {
                // Hygienic variables are analyzed in context
                Ok(())
            }
            Expr::Quote(inner) => {
                // Quoted expressions are not executed, so not dead code
                self.analyze_expression(inner, analysis)
            }
            Expr::Quasiquote(inner) => {
                self.analyze_expression(inner, analysis)
            }
            Expr::Unquote(inner) => {
                self.analyze_expression(inner, analysis)
            }
            Expr::UnquoteSplicing(inner) => {
                self.analyze_expression(inner, analysis)
            }
            Expr::Vector(exprs) => {
                for expr in exprs {
                    self.analyze_expression(expr, analysis)?;
                }
                Ok(())
            }
            Expr::List(exprs) => {
                self.analyze_list(exprs, analysis)
            }
            Expr::DottedList(exprs, tail) => {
                // Analyze head elements and tail
                for expr in exprs {
                    self.analyze_expression(expr, analysis)?;
                }
                self.analyze_expression(tail, analysis)
            }
        }
    }

    /// Analyze a list expression for dead code patterns
    fn analyze_list(&self, exprs: &[Expr], analysis: &mut DeadCodeAnalysis) -> Result<()> {
        if exprs.is_empty() {
            return Ok(());
        }

        // Check for conditional expressions
        if let Expr::Variable(op) = &exprs[0] {
            match op.as_str() {
                "if" => self.analyze_if_expression(exprs, analysis)?,
                "cond" => self.analyze_cond_expression(exprs, analysis)?,
                "and" => self.analyze_and_expression(exprs, analysis)?,
                "or" => self.analyze_or_expression(exprs, analysis)?,
                "begin" => self.analyze_begin_expression(exprs, analysis)?,
                "let" | "let*" | "letrec" => self.analyze_let_expression(exprs, analysis)?,
                _ => {
                    // Recursively analyze all subexpressions
                    for expr in exprs {
                        self.analyze_expression(expr, analysis)?;
                    }
                }
            }
        } else {
            // Recursively analyze all subexpressions
            for expr in exprs {
                self.analyze_expression(expr, analysis)?;
            }
        }

        Ok(())
    }

    /// Analyze if expression for dead code
    fn analyze_if_expression(&self, exprs: &[Expr], analysis: &mut DeadCodeAnalysis) -> Result<()> {
        if exprs.len() < 3 {
            return Ok(());
        }

        let condition = &exprs[1];
        let then_branch = &exprs[2];
        let else_branch = exprs.get(3);

        // Check if condition is a constant
        if let Expr::Literal(Literal::Boolean(b)) = condition {
            if *b {
                // Condition is always true, else branch is dead
                if let Some(else_expr) = else_branch {
                    analysis.unreachable_expressions.push(UnreachableCode {
                        description: "Else branch of always-true conditional".to_string(),
                        expression: else_expr.clone(),
                        reason: UnreachabilityReason::AlwaysFalseConditional,
                    });
                }
                analysis.redundant_conditionals.push(RedundantConditional {
                    description: "If with always-true condition".to_string(),
                    conditional: Expr::List(exprs.to_vec()),
                    simplified: then_branch.clone(),
                    redundancy_type: ConditionalRedundancy::AlwaysTrue,
                });
            } else {
                // Condition is always false, then branch is dead
                analysis.unreachable_expressions.push(UnreachableCode {
                    description: "Then branch of always-false conditional".to_string(),
                    expression: then_branch.clone(),
                    reason: UnreachabilityReason::AlwaysFalseConditional,
                });
                
                let simplified = else_branch.cloned().unwrap_or(Expr::Literal(Literal::Nil));
                analysis.redundant_conditionals.push(RedundantConditional {
                    description: "If with always-false condition".to_string(),
                    conditional: Expr::List(exprs.to_vec()),
                    simplified,
                    redundancy_type: ConditionalRedundancy::AlwaysFalse,
                });
            }
        } else {
            // Check for identical branches
            if let Some(else_expr) = else_branch {
                if self.expressions_equivalent(then_branch, else_expr) {
                    analysis.redundant_conditionals.push(RedundantConditional {
                        description: "If with identical then and else branches".to_string(),
                        conditional: Expr::List(exprs.to_vec()),
                        simplified: then_branch.clone(),
                        redundancy_type: ConditionalRedundancy::IdenticalBranches,
                    });
                }
            }
        }

        // Recursively analyze condition and branches
        self.analyze_expression(condition, analysis)?;
        self.analyze_expression(then_branch, analysis)?;
        if let Some(else_expr) = else_branch {
            self.analyze_expression(else_expr, analysis)?;
        }

        Ok(())
    }

    /// Analyze cond expression for dead code
    fn analyze_cond_expression(&self, exprs: &[Expr], analysis: &mut DeadCodeAnalysis) -> Result<()> {
        // Look for unreachable clauses after an always-true condition
        for (i, expr) in exprs[1..].iter().enumerate() {
            if let Expr::List(clause) = expr {
                if clause.len() >= 2 {
                    if let Expr::Literal(Literal::Boolean(true)) = &clause[0] {
                        // This clause is always true, subsequent clauses are unreachable
                        for unreachable_expr in &exprs[i + 2..] {
                            analysis.unreachable_expressions.push(UnreachableCode {
                                description: format!("Cond clause after always-true clause {}", i + 1),
                                expression: unreachable_expr.clone(),
                                reason: UnreachabilityReason::AlwaysTrueElse,
                            });
                        }
                        break;
                    }
                }
            }
        }

        // Recursively analyze all clauses
        for expr in &exprs[1..] {
            self.analyze_expression(expr, analysis)?;
        }

        Ok(())
    }

    /// Analyze and expression for dead code
    fn analyze_and_expression(&self, exprs: &[Expr], analysis: &mut DeadCodeAnalysis) -> Result<()> {
        // Look for false constants that make subsequent expressions unreachable
        for (i, expr) in exprs[1..].iter().enumerate() {
            if let Expr::Literal(Literal::Boolean(false)) = expr {
                // This expression is false, subsequent expressions are unreachable
                for unreachable_expr in &exprs[i + 2..] {
                    analysis.unreachable_expressions.push(UnreachableCode {
                        description: format!("And expression {} after false value", i + 2),
                        expression: unreachable_expr.clone(),
                        reason: UnreachabilityReason::AlwaysFalseConditional,
                    });
                }
                break;
            }
        }

        // Recursively analyze all subexpressions
        for expr in &exprs[1..] {
            self.analyze_expression(expr, analysis)?;
        }

        Ok(())
    }

    /// Analyze or expression for dead code
    fn analyze_or_expression(&self, exprs: &[Expr], analysis: &mut DeadCodeAnalysis) -> Result<()> {
        // Look for true constants that make subsequent expressions unreachable
        for (i, expr) in exprs[1..].iter().enumerate() {
            if let Expr::Literal(Literal::Boolean(true)) = expr {
                // This expression is true, subsequent expressions are unreachable
                for unreachable_expr in &exprs[i + 2..] {
                    analysis.unreachable_expressions.push(UnreachableCode {
                        description: format!("Or expression {} after true value", i + 2),
                        expression: unreachable_expr.clone(),
                        reason: UnreachabilityReason::AlwaysTrueElse,
                    });
                }
                break;
            }
        }

        // Recursively analyze all subexpressions
        for expr in &exprs[1..] {
            self.analyze_expression(expr, analysis)?;
        }

        Ok(())
    }

    /// Analyze begin expression for dead code
    fn analyze_begin_expression(&self, exprs: &[Expr], analysis: &mut DeadCodeAnalysis) -> Result<()> {
        // In a begin block, only the last expression's value matters
        // But all expressions might have side effects, so we're conservative
        
        // Recursively analyze all subexpressions
        for expr in &exprs[1..] {
            self.analyze_expression(expr, analysis)?;
        }

        Ok(())
    }

    /// Analyze let expression for dead code
    fn analyze_let_expression(&self, exprs: &[Expr], analysis: &mut DeadCodeAnalysis) -> Result<()> {
        if exprs.len() < 3 {
            return Ok(());
        }

        // Analyze bindings for unused variables
        if let Expr::List(bindings) = &exprs[1] {
            let mut defined_vars = HashSet::new();
            let mut used_vars = HashSet::new();

            // Collect defined variables
            for binding in bindings {
                if let Expr::List(binding_pair) = binding {
                    if binding_pair.len() >= 1 {
                        if let Expr::Variable(var_name) = &binding_pair[0] {
                            defined_vars.insert(var_name.clone());
                        }
                    }
                }
            }

            // Check usage in body
            for expr in &exprs[2..] {
                self.collect_variable_usage(expr, &mut used_vars);
            }

            // Find unused variables
            for var in defined_vars {
                if !used_vars.contains(&var) {
                    analysis.unused_variables.insert(var);
                }
            }
        }

        // Recursively analyze all subexpressions
        for expr in &exprs[1..] {
            self.analyze_expression(expr, analysis)?;
        }

        Ok(())
    }

    /// Collect variable usage in an expression
    fn collect_variable_usage(&self, expr: &Expr, used_vars: &mut HashSet<String>) {
        match expr {
            Expr::Variable(name) => {
                used_vars.insert(name.clone());
            }
            Expr::HygienicVariable(hygienic_symbol) => {
                // Handle hygienic variables properly by using their unique name
                used_vars.insert(hygienic_symbol.unique_name());
            }
            Expr::List(exprs) => {
                for expr in exprs {
                    self.collect_variable_usage(expr, used_vars);
                }
            }
            Expr::Quote(inner) => {
                self.collect_variable_usage(inner, used_vars);
            }
            Expr::Quasiquote(inner) => {
                self.collect_variable_usage(inner, used_vars);
            }
            Expr::Unquote(inner) => {
                self.collect_variable_usage(inner, used_vars);
            }
            Expr::UnquoteSplicing(inner) => {
                self.collect_variable_usage(inner, used_vars);
            }
            Expr::Vector(exprs) => {
                for expr in exprs {
                    self.collect_variable_usage(expr, used_vars);
                }
            }
            Expr::Literal(_) => {
                // Literals don't use variables
            }
            Expr::DottedList(exprs, tail) => {
                for expr in exprs {
                    self.collect_variable_usage(expr, used_vars);
                }
                self.collect_variable_usage(tail, used_vars);
            }
        }
    }

    /// Check if two expressions are equivalent
    fn expressions_equivalent(&self, a: &Expr, b: &Expr) -> bool {
        match (a, b) {
            (Expr::Literal(lit_a), Expr::Literal(lit_b)) => lit_a == lit_b,
            (Expr::Variable(var_a), Expr::Variable(var_b)) => var_a == var_b,
            (Expr::Quote(inner_a), Expr::Quote(inner_b)) => {
                self.expressions_equivalent(inner_a, inner_b)
            }
            (Expr::List(exprs_a), Expr::List(exprs_b)) => {
                exprs_a.len() == exprs_b.len() 
                    && exprs_a.iter().zip(exprs_b.iter())
                        .all(|(a, b)| self.expressions_equivalent(a, b))
            }
            _ => false,
        }
    }

    /// Eliminate dead code from an expression
    fn eliminate_expression(&self, expr: &Expr, analysis: &DeadCodeAnalysis) -> Result<(Expr, usize)> {
        match expr {
            Expr::Literal(_) | Expr::Variable(_) => Ok((expr.clone(), 0)),
            Expr::Quote(inner) => {
                let (eliminated_inner, count) = self.eliminate_expression(inner, analysis)?;
                Ok((Expr::Quote(Box::new(eliminated_inner)), count))
            }
            Expr::List(exprs) => self.eliminate_list(exprs, analysis),
            Expr::DottedList(exprs, tail) => {
                // Eliminate from head elements and tail
                let mut eliminated_exprs = Vec::new();
                let mut total_count = 0;
                
                for expr in exprs {
                    let (eliminated_expr, count) = self.eliminate_expression(expr, analysis)?;
                    eliminated_exprs.push(eliminated_expr);
                    total_count += count;
                }
                
                let (eliminated_tail, tail_count) = self.eliminate_expression(tail, analysis)?;
                total_count += tail_count;
                
                Ok((Expr::DottedList(eliminated_exprs, Box::new(eliminated_tail)), total_count))
            }
            Expr::HygienicVariable(_) => Ok((expr.clone(), 0)),
            Expr::Quasiquote(inner) => {
                let (eliminated_inner, count) = self.eliminate_expression(inner, analysis)?;
                Ok((Expr::Quasiquote(Box::new(eliminated_inner)), count))
            }
            Expr::Unquote(inner) => {
                let (eliminated_inner, count) = self.eliminate_expression(inner, analysis)?;
                Ok((Expr::Unquote(Box::new(eliminated_inner)), count))
            }
            Expr::UnquoteSplicing(inner) => {
                let (eliminated_inner, count) = self.eliminate_expression(inner, analysis)?;
                Ok((Expr::UnquoteSplicing(Box::new(eliminated_inner)), count))
            }
            Expr::Vector(exprs) => {
                let mut eliminated_exprs = Vec::new();
                let mut total_count = 0;
                
                for expr in exprs {
                    let (eliminated_expr, count) = self.eliminate_expression(expr, analysis)?;
                    eliminated_exprs.push(eliminated_expr);
                    total_count += count;
                }
                
                Ok((Expr::Vector(eliminated_exprs), total_count))
            }
        }
    }

    /// Eliminate dead code from a list expression
    fn eliminate_list(&self, exprs: &[Expr], analysis: &DeadCodeAnalysis) -> Result<(Expr, usize)> {
        if exprs.is_empty() {
            return Ok((Expr::List(vec![]), 0));
        }

        // Check if this expression should be eliminated entirely
        for unreachable in &analysis.unreachable_expressions {
            if self.expressions_equivalent(&Expr::List(exprs.to_vec()), &unreachable.expression) {
                return Ok((Expr::Literal(Literal::Nil), 1));
            }
        }

        // Check for redundant conditionals
        for redundant in &analysis.redundant_conditionals {
            if self.expressions_equivalent(&Expr::List(exprs.to_vec()), &redundant.conditional) {
                let (simplified, count) = self.eliminate_expression(&redundant.simplified, analysis)?;
                return Ok((simplified, count + 1));
            }
        }

        // Handle special forms
        if let Expr::Variable(op) = &exprs[0] {
            match op.as_str() {
                "if" => return self.eliminate_if_expression(exprs, analysis),
                "and" => return self.eliminate_and_expression(exprs, analysis),
                "or" => return self.eliminate_or_expression(exprs, analysis),
                _ => {}
            }
        }

        // Recursively eliminate from subexpressions
        let mut eliminated_exprs = Vec::new();
        let mut total_count = 0;

        for expr in exprs {
            let (eliminated, count) = self.eliminate_expression(expr, analysis)?;
            eliminated_exprs.push(eliminated);
            total_count += count;
        }

        Ok((Expr::List(eliminated_exprs), total_count))
    }

    /// Eliminate dead code from if expression
    fn eliminate_if_expression(&self, exprs: &[Expr], analysis: &DeadCodeAnalysis) -> Result<(Expr, usize)> {
        if exprs.len() < 3 {
            return Ok((Expr::List(exprs.to_vec()), 0));
        }

        let condition = &exprs[1];
        
        // Check if condition is a constant
        if let Expr::Literal(Literal::Boolean(b)) = condition {
            if *b {
                // Always true, return then branch
                let (eliminated_then, count) = self.eliminate_expression(&exprs[2], analysis)?;
                return Ok((eliminated_then, count + 1));
            } else {
                // Always false, return else branch or nil
                if exprs.len() > 3 {
                    let (eliminated_else, count) = self.eliminate_expression(&exprs[3], analysis)?;
                    return Ok((eliminated_else, count + 1));
                } else {
                    return Ok((Expr::Literal(Literal::Nil), 1));
                }
            }
        }

        // Recursively eliminate from all parts
        let mut eliminated_exprs = Vec::new();
        let mut total_count = 0;

        for expr in exprs {
            let (eliminated, count) = self.eliminate_expression(expr, analysis)?;
            eliminated_exprs.push(eliminated);
            total_count += count;
        }

        Ok((Expr::List(eliminated_exprs), total_count))
    }

    /// Eliminate dead code from and expression
    fn eliminate_and_expression(&self, exprs: &[Expr], analysis: &DeadCodeAnalysis) -> Result<(Expr, usize)> {
        let mut eliminated_exprs = vec![exprs[0].clone()]; // Keep the 'and' operator
        let mut total_count = 0;

        for expr in &exprs[1..] {
            if let Expr::Literal(Literal::Boolean(false)) = expr {
                // Found false, eliminate rest and return false
                return Ok((Expr::Literal(Literal::Boolean(false)), total_count + 1));
            }
            
            let (eliminated, count) = self.eliminate_expression(expr, analysis)?;
            eliminated_exprs.push(eliminated);
            total_count += count;
        }

        Ok((Expr::List(eliminated_exprs), total_count))
    }

    /// Eliminate dead code from or expression
    fn eliminate_or_expression(&self, exprs: &[Expr], analysis: &DeadCodeAnalysis) -> Result<(Expr, usize)> {
        let mut eliminated_exprs = vec![exprs[0].clone()]; // Keep the 'or' operator
        let mut total_count = 0;

        for expr in &exprs[1..] {
            if let Expr::Literal(Literal::Boolean(true)) = expr {
                // Found true, eliminate rest and return true
                return Ok((Expr::Literal(Literal::Boolean(true)), total_count + 1));
            }
            
            let (eliminated, count) = self.eliminate_expression(expr, analysis)?;
            eliminated_exprs.push(eliminated);
            total_count += count;
        }

        Ok((Expr::List(eliminated_exprs), total_count))
    }

    /// Get elimination statistics
    pub fn get_statistics(&self) -> &EliminationStatistics {
        &self.statistics
    }

    /// Reset statistics
    pub fn reset_statistics(&mut self) {
        self.statistics = EliminationStatistics::default();
    }
}

impl Default for DeadCodeEliminator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_dead_code_eliminator_creation() {
        let eliminator = DeadCodeEliminator::new();
        assert_eq!(eliminator.get_statistics().total_attempts, 0);
    }

    #[test]
    fn test_always_true_if_elimination() {
        let mut eliminator = DeadCodeEliminator::new();
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Literal(Literal::Boolean(true)),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]);

        let result = eliminator.eliminate(&expr).unwrap();
        assert!(result.elimination_applied);
        assert_eq!(result.dead_code_eliminated, 1);
        
        match result.optimized_expr {
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))) => {},
            _ => panic!("Expected optimized result to be 1"),
        }
    }

    #[test]
    fn test_always_false_if_elimination() {
        let mut eliminator = DeadCodeEliminator::new();
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Literal(Literal::Boolean(false)),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]);

        let result = eliminator.eliminate(&expr).unwrap();
        assert!(result.elimination_applied);
        
        match result.optimized_expr {
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))) => {},
            _ => panic!("Expected optimized result to be 2"),
        }
    }

    #[test]
    fn test_dead_code_analysis() {
        let eliminator = DeadCodeEliminator::new();
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Literal(Literal::Boolean(true)),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]);

        let analysis = eliminator.analyze(&expr).unwrap();
        assert_eq!(analysis.unreachable_expressions.len(), 1);
        assert_eq!(analysis.redundant_conditionals.len(), 1);
    }

    #[test]
    fn test_and_short_circuit() {
        let mut eliminator = DeadCodeEliminator::new();
        let expr = Expr::List(vec![
            Expr::Variable("and".to_string()),
            Expr::Literal(Literal::Boolean(false)),
            Expr::Variable("x".to_string()),
        ]);

        let result = eliminator.eliminate(&expr).unwrap();
        assert!(result.elimination_applied);
        
        match result.optimized_expr {
            Expr::Literal(Literal::Boolean(false)) => {},
            _ => panic!("Expected optimized result to be false"),
        }
    }

    #[test]
    fn test_or_short_circuit() {
        let mut eliminator = DeadCodeEliminator::new();
        let expr = Expr::List(vec![
            Expr::Variable("or".to_string()),
            Expr::Literal(Literal::Boolean(true)),
            Expr::Variable("x".to_string()),
        ]);

        let result = eliminator.eliminate(&expr).unwrap();
        assert!(result.elimination_applied);
        
        match result.optimized_expr {
            Expr::Literal(Literal::Boolean(true)) => {},
            _ => panic!("Expected optimized result to be true"),
        }
    }

    #[test]
    fn test_no_dead_code() {
        let mut eliminator = DeadCodeEliminator::new();
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        ]);

        let result = eliminator.eliminate(&expr).unwrap();
        assert!(!result.elimination_applied);
        assert_eq!(result.dead_code_eliminated, 0);
    }
}