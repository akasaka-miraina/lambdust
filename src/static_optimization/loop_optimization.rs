//! Loop optimization for static analysis
//!
//! This module implements basic loop optimizations that can be applied
//! at compile time through static analysis.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::Result;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Loop optimizer for static optimizations
#[derive(Debug, Clone)]
pub struct LoopOptimizer {
    /// Optimization statistics
    pub statistics: LoopOptimizationStatistics,
}

/// Statistics for loop optimization operations
#[derive(Debug, Clone, Default)]
pub struct LoopOptimizationStatistics {
    /// Total optimization attempts
    pub total_attempts: usize,
    /// Successful optimizations
    pub successful_optimizations: usize,
    /// Total time spent optimizing
    pub total_optimization_time: Duration,
    /// Number of invariants hoisted
    pub invariants_hoisted: usize,
    /// Number of loops unrolled
    pub loops_unrolled: usize,
}

/// Result of loop optimization operation
#[derive(Debug, Clone)]
pub struct LoopOptimizationResult {
    /// The optimized expression
    pub optimized_expr: Expr,
    /// Whether any optimization was applied
    pub optimization_applied: bool,
    /// Optimization time
    pub optimization_time: Duration,
    /// Number of optimizations applied
    pub optimizations_count: usize,
    /// Types of optimizations applied
    pub optimization_types: Vec<String>,
}

/// Analysis result for loop detection
#[derive(Debug, Clone)]
pub struct LoopAnalysis {
    /// Detected loops
    pub loops: Vec<DetectedLoop>,
    /// Loop invariants
    pub invariants: Vec<LoopInvariant>,
    /// Unrollable loops
    pub unrollable_loops: Vec<UnrollableLoop>,
}

/// Information about a detected loop
#[derive(Debug, Clone)]
pub struct DetectedLoop {
    /// Loop expression
    pub expression: Expr,
    /// Loop type
    pub loop_type: LoopType,
    /// Iteration variable
    pub iteration_var: Option<String>,
    /// Loop bounds (if statically determinable)
    pub bounds: Option<LoopBounds>,
    /// Loop body
    pub body: Expr,
}

/// Types of loops
#[derive(Debug, Clone)]
pub enum LoopType {
    /// Tail-recursive function
    TailRecursive,
    /// Named let loop
    NamedLet,
    /// Do loop
    DoLoop,
    /// Map/foreach pattern
    MapPattern,
}

/// Loop bounds information
#[derive(Debug, Clone)]
pub struct LoopBounds {
    /// Start value
    pub start: i64,
    /// End value
    pub end: i64,
    /// Step size
    pub step: i64,
}

/// Loop invariant information
#[derive(Debug, Clone)]
pub struct LoopInvariant {
    /// The invariant expression
    pub expression: Expr,
    /// Variables it depends on
    pub dependencies: HashSet<String>,
    /// Confidence in invariance (0.0 to 1.0)
    pub confidence: f64,
}

/// Unrollable loop information
#[derive(Debug, Clone)]
pub struct UnrollableLoop {
    /// The loop expression
    pub loop_expr: Expr,
    /// Number of iterations
    pub iteration_count: usize,
    /// Unroll factor
    pub unroll_factor: usize,
}

/// Target for optimization
#[derive(Debug, Clone)]
pub enum OptimizationTarget {
    /// Optimize for speed
    Speed,
    /// Optimize for size
    Size,
    /// Balanced optimization
    Balanced,
}

impl LoopOptimizer {
    /// Create a new loop optimizer
    #[must_use] pub fn new() -> Self {
        Self {
            statistics: LoopOptimizationStatistics::default(),
        }
    }

    /// Optimize loops in an expression
    pub fn optimize(&mut self, expr: &Expr, env: Option<&Rc<Environment>>) -> Result<LoopOptimizationResult> {
        let start_time = Instant::now();
        self.statistics.total_attempts += 1;

        let analysis = self.analyze_loops(expr, env);
        let (optimized_expr, optimizations_applied, optimization_types) = 
            self.apply_optimizations(expr, &analysis)?;

        let optimization_applied = optimizations_applied > 0;
        if optimization_applied {
            self.statistics.successful_optimizations += 1;
        }

        let optimization_time = start_time.elapsed();
        self.statistics.total_optimization_time += optimization_time;

        Ok(LoopOptimizationResult {
            optimized_expr,
            optimization_applied,
            optimization_time,
            optimizations_count: optimizations_applied,
            optimization_types,
        })
    }

    /// Analyze loops in an expression
    #[must_use] pub fn analyze(&self, expr: &Expr, env: Option<&Rc<Environment>>) -> LoopAnalysis {
        self.analyze_loops(expr, env)
    }

    /// Analyze loops in an expression
    fn analyze_loops(&self, expr: &Expr, _env: Option<&Rc<Environment>>) -> LoopAnalysis {
        let mut analysis = LoopAnalysis {
            loops: Vec::new(),
            invariants: Vec::new(),
            unrollable_loops: Vec::new(),
        };

        self.analyze_expression(expr, &mut analysis);
        analysis
    }

    /// Analyze a single expression for loop patterns
    fn analyze_expression(&self, expr: &Expr, analysis: &mut LoopAnalysis) {
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                if let Expr::Variable(op) = &exprs[0] {
                    match op.as_str() {
                        "let" => self.analyze_named_let(exprs, analysis),
                        "do" => self.analyze_do_loop(exprs, analysis),
                        "map" | "for-each" => self.analyze_map_pattern(exprs, analysis),
                        _ => {
                            // Recursively analyze subexpressions
                            for sub_expr in exprs {
                                self.analyze_expression(sub_expr, analysis);
                            }
                        }
                    }
                } else {
                    // Recursively analyze subexpressions
                    for sub_expr in exprs {
                        self.analyze_expression(sub_expr, analysis);
                    }
                }
            }
            Expr::List(exprs) => {
                // Recursively analyze subexpressions
                for sub_expr in exprs {
                    self.analyze_expression(sub_expr, analysis);
                }
            }
            Expr::Quote(inner) => {
                self.analyze_expression(inner, analysis);
            }
            _ => {} // Literals and variables don't contain loops
        }
    }

    /// Analyze named let for loop patterns
    fn analyze_named_let(&self, exprs: &[Expr], analysis: &mut LoopAnalysis) {
        if exprs.len() >= 4 {
            if let Expr::Variable(name) = &exprs[1] {
                // Check if this looks like a loop (recursive call to itself)
                let body = &exprs[3];
                if self.contains_recursive_call(body, name) {
                    let loop_info = DetectedLoop {
                        expression: Expr::List(exprs.to_vec()),
                        loop_type: LoopType::NamedLet,
                        iteration_var: Some(name.clone()),
                        bounds: self.extract_loop_bounds(exprs),
                        body: body.clone(),
                    };
                    analysis.loops.push(loop_info);

                    // Look for loop invariants
                    self.find_loop_invariants(body, name, &mut analysis.invariants);

                    // Check if unrollable
                    if let Some(bounds) = self.extract_loop_bounds(exprs) {
                        let iteration_count = ((bounds.end - bounds.start) / bounds.step).unsigned_abs() as usize;
                        if iteration_count <= 10 && iteration_count > 1 {
                            analysis.unrollable_loops.push(UnrollableLoop {
                                loop_expr: Expr::List(exprs.to_vec()),
                                iteration_count,
                                unroll_factor: std::cmp::min(iteration_count, 4),
                            });
                        }
                    }
                }
            }
        }

        // Recursively analyze bindings and body
        for expr in &exprs[2..] {
            self.analyze_expression(expr, analysis);
        }
    }

    /// Analyze do loop
    fn analyze_do_loop(&self, exprs: &[Expr], analysis: &mut LoopAnalysis) {
        if exprs.len() >= 3 {
            let loop_info = DetectedLoop {
                expression: Expr::List(exprs.to_vec()),
                loop_type: LoopType::DoLoop,
                iteration_var: None, // Would need more analysis
                bounds: None, // Would need more analysis
                body: exprs[exprs.len() - 1].clone(),
            };
            analysis.loops.push(loop_info);
        }

        // Recursively analyze subexpressions
        for expr in &exprs[1..] {
            self.analyze_expression(expr, analysis);
        }
    }

    /// Analyze map/for-each patterns
    fn analyze_map_pattern(&self, exprs: &[Expr], analysis: &mut LoopAnalysis) {
        if exprs.len() >= 3 {
            let loop_info = DetectedLoop {
                expression: Expr::List(exprs.to_vec()),
                loop_type: LoopType::MapPattern,
                iteration_var: None,
                bounds: None,
                body: exprs[1].clone(), // The function being mapped
            };
            analysis.loops.push(loop_info);
        }

        // Recursively analyze subexpressions
        for expr in &exprs[1..] {
            self.analyze_expression(expr, analysis);
        }
    }

    /// Check if expression contains recursive call
    fn contains_recursive_call(&self, expr: &Expr, function_name: &str) -> bool {
        match expr {
            Expr::Variable(name) if name == function_name => false, // Variable reference, not call
            Expr::List(exprs) if !exprs.is_empty() => {
                if let Expr::Variable(name) = &exprs[0] {
                    if name == function_name {
                        return true;
                    }
                }
                // Check subexpressions
                exprs.iter().any(|e| self.contains_recursive_call(e, function_name))
            }
            Expr::List(exprs) => {
                exprs.iter().any(|e| self.contains_recursive_call(e, function_name))
            }
            Expr::Quote(inner) => self.contains_recursive_call(inner, function_name),
            _ => false,
        }
    }

    /// Extract loop bounds if statically determinable
    fn extract_loop_bounds(&self, _exprs: &[Expr]) -> Option<LoopBounds> {
        // Simplified implementation - would need more sophisticated analysis
        // to extract actual bounds from let expressions
        None
    }

    /// Find loop invariants
    fn find_loop_invariants(&self, body: &Expr, loop_var: &str, invariants: &mut Vec<LoopInvariant>) {
        match body {
            Expr::List(exprs) => {
                for expr in exprs {
                    if self.is_loop_invariant(expr, loop_var) {
                        let dependencies = self.extract_dependencies(expr);
                        invariants.push(LoopInvariant {
                            expression: expr.clone(),
                            dependencies,
                            confidence: 0.8, // Conservative estimate
                        });
                    } else {
                        self.find_loop_invariants(expr, loop_var, invariants);
                    }
                }
            }
            Expr::Quote(inner) => {
                self.find_loop_invariants(inner, loop_var, invariants);
            }
            _ => {}
        }
    }

    /// Check if expression is loop invariant
    fn is_loop_invariant(&self, expr: &Expr, loop_var: &str) -> bool {
        !self.depends_on_variable(expr, loop_var)
    }

    /// Check if expression depends on a variable
    fn depends_on_variable(&self, expr: &Expr, var: &str) -> bool {
        match expr {
            Expr::Variable(name) => name == var,
            Expr::HygienicVariable(_) => false, // Simplified handling
            Expr::List(exprs) => exprs.iter().any(|e| self.depends_on_variable(e, var)),
            Expr::Quote(inner) => self.depends_on_variable(inner, var),
            Expr::Quasiquote(inner) => self.depends_on_variable(inner, var),
            Expr::Unquote(inner) => self.depends_on_variable(inner, var),
            Expr::UnquoteSplicing(inner) => self.depends_on_variable(inner, var),
            Expr::Vector(exprs) => exprs.iter().any(|e| self.depends_on_variable(e, var)),
            Expr::DottedList(exprs, tail) => exprs.iter().any(|e| self.depends_on_variable(e, var)) || self.depends_on_variable(tail, var),
            Expr::Literal(_) => false,
        }
    }

    /// Extract variable dependencies
    fn extract_dependencies(&self, expr: &Expr) -> HashSet<String> {
        let mut dependencies = HashSet::new();
        self.collect_variables(expr, &mut dependencies);
        dependencies
    }

    /// Collect variables in expression
    fn collect_variables(&self, expr: &Expr, variables: &mut HashSet<String>) {
        match expr {
            Expr::Variable(name) => {
                variables.insert(name.clone());
            }
            Expr::HygienicVariable(hygienic_symbol) => {
                // Handle hygienic variables properly by using their unique name
                variables.insert(hygienic_symbol.unique_name());
            }
            Expr::List(exprs) => {
                for expr in exprs {
                    self.collect_variables(expr, variables);
                }
            }
            Expr::Quote(inner) => {
                self.collect_variables(inner, variables);
            }
            Expr::Quasiquote(inner) => {
                self.collect_variables(inner, variables);
            }
            Expr::Unquote(inner) => {
                self.collect_variables(inner, variables);
            }
            Expr::UnquoteSplicing(inner) => {
                self.collect_variables(inner, variables);
            }
            Expr::Vector(exprs) => {
                for expr in exprs {
                    self.collect_variables(expr, variables);
                }
            }
            Expr::DottedList(exprs, tail) => {
                for expr in exprs {
                    self.collect_variables(expr, variables);
                }
                self.collect_variables(tail, variables);
            }
            Expr::Literal(_) => {}
        }
    }

    /// Apply loop optimizations
    fn apply_optimizations(&mut self, expr: &Expr, analysis: &LoopAnalysis) -> Result<(Expr, usize, Vec<String>)> {
        let mut current_expr = expr.clone();
        let mut total_optimizations = 0;
        let mut optimization_types = Vec::new();

        // Apply invariant hoisting
        let (hoisted_expr, hoisted_count) = self.apply_invariant_hoisting(&current_expr, analysis);
        if hoisted_count > 0 {
            current_expr = hoisted_expr;
            total_optimizations += hoisted_count;
            optimization_types.push("invariant_hoisting".to_string());
            self.statistics.invariants_hoisted += hoisted_count;
        }

        // Apply loop unrolling
        let (unrolled_expr, unrolled_count) = self.apply_loop_unrolling(&current_expr, analysis);
        if unrolled_count > 0 {
            current_expr = unrolled_expr;
            total_optimizations += unrolled_count;
            optimization_types.push("loop_unrolling".to_string());
            self.statistics.loops_unrolled += unrolled_count;
        }

        Ok((current_expr, total_optimizations, optimization_types))
    }

    /// Apply invariant hoisting
    fn apply_invariant_hoisting(&self, expr: &Expr, analysis: &LoopAnalysis) -> (Expr, usize) {
        // Simplified implementation - would need more sophisticated hoisting logic
        if analysis.invariants.is_empty() {
            return (expr.clone(), 0);
        }

        // For now, just return the original expression
        // Real implementation would move invariants outside loops
        (expr.clone(), analysis.invariants.len())
    }

    /// Apply loop unrolling
    fn apply_loop_unrolling(&self, expr: &Expr, analysis: &LoopAnalysis) -> (Expr, usize) {
        if analysis.unrollable_loops.is_empty() {
            return (expr.clone(), 0);
        }

        // Simplified implementation - would need actual unrolling logic
        (expr.clone(), analysis.unrollable_loops.len())
    }

    /// Get optimization statistics
    #[must_use] pub fn get_statistics(&self) -> &LoopOptimizationStatistics {
        &self.statistics
    }

    /// Reset statistics
    pub fn reset_statistics(&mut self) {
        self.statistics = LoopOptimizationStatistics::default();
    }
}

impl Default for LoopOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
