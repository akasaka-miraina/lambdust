//! Static optimization system for compile-time transformations
//!
//! This module implements production-ready static optimizations that can be
//! applied at compile time without requiring runtime information.
//!
//! ## Module Organization
//!
//! - `constant_folding`: Compile-time constant evaluation and folding
//! - `dead_code_elimination`: Removal of unreachable code paths
//! - `expression_rewriting`: AST-level expression simplification
//! - `common_subexpression`: Common subexpression elimination
//! - `loop_optimization`: Loop unrolling and invariant hoisting

pub mod constant_folding;
pub mod dead_code_elimination;
pub mod expression_rewriting;
pub mod common_subexpression;
pub mod loop_optimization;

// Re-export main types
pub use constant_folding::{ConstantFolder, ConstantFoldingResult, FoldableConstant};
pub use dead_code_elimination::{DeadCodeEliminator, DeadCodeAnalysis, UnreachableCode};
pub use expression_rewriting::{ExpressionRewriter, RewriteRule, RewriteResult};
pub use common_subexpression::{CommonSubexpressionEliminator, SubexpressionMap, CSEResult};
pub use loop_optimization::{LoopOptimizer, LoopAnalysis, OptimizationTarget};

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::Result;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Static optimization engine for compile-time transformations
///
/// This engine applies a series of static optimizations to Scheme expressions
/// before evaluation. All optimizations preserve semantic equivalence and
/// can be verified through static analysis.
#[derive(Debug)]
pub struct StaticOptimizationEngine {
    /// Constant folding optimizer
    constant_folder: ConstantFolder,
    
    /// Dead code elimination optimizer
    dead_code_eliminator: DeadCodeEliminator,
    
    /// Expression rewriting optimizer
    expression_rewriter: ExpressionRewriter,
    
    /// Common subexpression elimination optimizer
    cse_optimizer: CommonSubexpressionEliminator,
    
    /// Loop optimization optimizer
    loop_optimizer: LoopOptimizer,
    
    /// Configuration settings
    config: StaticOptimizationConfig,
    
    /// Optimization statistics
    statistics: StaticOptimizationStatistics,
    
    /// Optimization cache for repeated expressions
    optimization_cache: HashMap<String, CachedOptimization>,
}

/// Configuration for static optimization
#[derive(Debug, Clone)]
pub struct StaticOptimizationConfig {
    /// Enable constant folding
    pub enable_constant_folding: bool,
    
    /// Enable dead code elimination
    pub enable_dead_code_elimination: bool,
    
    /// Enable expression rewriting
    pub enable_expression_rewriting: bool,
    
    /// Enable common subexpression elimination
    pub enable_cse: bool,
    
    /// Enable loop optimization
    pub enable_loop_optimization: bool,
    
    /// Maximum optimization passes
    pub max_optimization_passes: usize,
    
    /// Optimization timeout per expression
    pub optimization_timeout: Duration,
    
    /// Enable optimization caching
    pub enable_caching: bool,
    
    /// Minimum expression complexity for optimization
    pub min_complexity_threshold: usize,
}

/// Statistics for static optimization
#[derive(Debug, Clone, Default)]
pub struct StaticOptimizationStatistics {
    /// Total expressions optimized
    pub total_expressions: usize,
    
    /// Successfully optimized expressions
    pub optimized_expressions: usize,
    
    /// Total optimization time
    pub total_optimization_time: Duration,
    
    /// Average optimization time per expression
    pub average_optimization_time: Duration,
    
    /// Cache hits
    pub cache_hits: usize,
    
    /// Cache misses
    pub cache_misses: usize,
    
    /// Optimization passes per category
    pub optimization_passes: HashMap<String, usize>,
    
    /// Size reduction statistics
    pub size_reduction: SizeReductionStats,
}

/// Size reduction statistics
#[derive(Debug, Clone, Default)]
pub struct SizeReductionStats {
    /// Total nodes before optimization
    pub nodes_before: usize,
    
    /// Total nodes after optimization
    pub nodes_after: usize,
    
    /// Average reduction percentage
    pub average_reduction_percent: f64,
    
    /// Maximum reduction achieved
    pub max_reduction_percent: f64,
}

/// Cached optimization result
#[derive(Debug, Clone)]
pub struct CachedOptimization {
    /// Original expression hash
    pub original_hash: String,
    
    /// Optimized expression
    pub optimized_expr: Expr,
    
    /// Optimization metadata
    pub metadata: OptimizationMetadata,
    
    /// Cache timestamp
    pub cached_at: Instant,
}

/// Metadata about an optimization
#[derive(Debug, Clone)]
pub struct OptimizationMetadata {
    /// Applied optimizations
    pub applied_optimizations: Vec<String>,
    
    /// Optimization time
    pub optimization_time: Duration,
    
    /// Size reduction
    pub size_reduction: i32,
    
    /// Complexity reduction
    pub complexity_reduction: f64,
}

/// Result of static optimization
#[derive(Debug, Clone)]
pub struct StaticOptimizationResult {
    /// Optimized expression
    pub optimized_expr: Expr,
    
    /// Whether any optimization was applied
    pub optimization_applied: bool,
    
    /// Metadata about the optimization
    pub metadata: OptimizationMetadata,
    
    /// Optimization statistics
    pub statistics: StaticOptimizationStatistics,
}

impl StaticOptimizationEngine {
    /// Create a new static optimization engine
    pub fn new(config: StaticOptimizationConfig) -> Self {
        Self {
            constant_folder: ConstantFolder::new(),
            dead_code_eliminator: DeadCodeEliminator::new(),
            expression_rewriter: ExpressionRewriter::new(),
            cse_optimizer: CommonSubexpressionEliminator::new(),
            loop_optimizer: LoopOptimizer::new(),
            config,
            statistics: StaticOptimizationStatistics::default(),
            optimization_cache: HashMap::new(),
        }
    }
    
    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(StaticOptimizationConfig::default())
    }
    
    /// Optimize an expression using static analysis
    pub fn optimize(&mut self, expr: &Expr, env: Option<Rc<Environment>>) -> Result<StaticOptimizationResult> {
        let start_time = Instant::now();
        self.statistics.total_expressions += 1;
        
        // Check cache first if enabled
        if self.config.enable_caching {
            let expr_hash = self.hash_expression(expr);
            if let Some(cached) = self.optimization_cache.get(&expr_hash) {
                self.statistics.cache_hits += 1;
                return Ok(StaticOptimizationResult {
                    optimized_expr: cached.optimized_expr.clone(),
                    optimization_applied: true,
                    metadata: cached.metadata.clone(),
                    statistics: self.statistics.clone(),
                });
            }
            self.statistics.cache_misses += 1;
        }
        
        // Check complexity threshold
        let complexity = self.calculate_expression_complexity(expr);
        if complexity < self.config.min_complexity_threshold {
            return Ok(StaticOptimizationResult {
                optimized_expr: expr.clone(),
                optimization_applied: false,
                metadata: OptimizationMetadata {
                    applied_optimizations: vec!["skipped_low_complexity".to_string()],
                    optimization_time: start_time.elapsed(),
                    size_reduction: 0,
                    complexity_reduction: 0.0,
                },
                statistics: self.statistics.clone(),
            });
        }
        
        let mut current_expr = expr.clone();
        let mut applied_optimizations = Vec::new();
        let mut _total_size_reduction = 0i32;
        let initial_size = self.expression_size(&current_expr);
        
        // Apply optimization passes
        for pass in 0..self.config.max_optimization_passes {
            let _pass_start = Instant::now();
            let mut pass_changed = false;
            
            // Check timeout
            if start_time.elapsed() > self.config.optimization_timeout {
                break;
            }
            
            // Constant folding
            if self.config.enable_constant_folding {
                if let Ok(folding_result) = self.constant_folder.fold(&current_expr) {
                    if folding_result.optimization_applied {
                        current_expr = folding_result.folded_expr;
                        applied_optimizations.push(format!("constant_folding_pass_{}", pass));
                        pass_changed = true;
                    }
                }
            }
            
            // Dead code elimination
            if self.config.enable_dead_code_elimination {
                if let Ok(dce_result) = self.dead_code_eliminator.eliminate(&current_expr) {
                    if dce_result.elimination_applied {
                        current_expr = dce_result.optimized_expr;
                        applied_optimizations.push(format!("dead_code_elimination_pass_{}", pass));
                        pass_changed = true;
                    }
                }
            }
            
            // Expression rewriting
            if self.config.enable_expression_rewriting {
                if let Ok(rewrite_result) = self.expression_rewriter.rewrite(&current_expr) {
                    if rewrite_result.rewrite_applied {
                        current_expr = rewrite_result.rewritten_expr;
                        applied_optimizations.push(format!("expression_rewriting_pass_{}", pass));
                        pass_changed = true;
                    }
                }
            }
            
            // Common subexpression elimination
            if self.config.enable_cse {
                if let Ok(cse_result) = self.cse_optimizer.eliminate(&current_expr) {
                    if cse_result.elimination_applied {
                        current_expr = cse_result.optimized_expr;
                        applied_optimizations.push(format!("cse_pass_{}", pass));
                        pass_changed = true;
                    }
                }
            }
            
            // Loop optimization
            if self.config.enable_loop_optimization {
                if let Ok(loop_result) = self.loop_optimizer.optimize(&current_expr, env.as_ref()) {
                    if loop_result.optimization_applied {
                        current_expr = loop_result.optimized_expr;
                        applied_optimizations.push(format!("loop_optimization_pass_{}", pass));
                        pass_changed = true;
                    }
                }
            }
            
            // Track pass statistics
            let pass_name = format!("pass_{}", pass);
            *self.statistics.optimization_passes.entry(pass_name).or_insert(0) += 1;
            
            // If no changes were made, we've reached a fixed point
            if !pass_changed {
                break;
            }
        }
        
        let final_size = self.expression_size(&current_expr);
        _total_size_reduction = initial_size as i32 - final_size as i32;
        
        let optimization_time = start_time.elapsed();
        let optimization_applied = !applied_optimizations.is_empty();
        
        if optimization_applied {
            self.statistics.optimized_expressions += 1;
        }
        
        self.statistics.total_optimization_time += optimization_time;
        self.update_average_optimization_time();
        self.update_size_reduction_stats(initial_size, final_size);
        
        let metadata = OptimizationMetadata {
            applied_optimizations,
            optimization_time,
            size_reduction: _total_size_reduction,
            complexity_reduction: (initial_size as f64 - final_size as f64) / initial_size as f64,
        };
        
        // Cache the result if enabled
        if self.config.enable_caching && optimization_applied {
            let expr_hash = self.hash_expression(expr);
            let cached = CachedOptimization {
                original_hash: expr_hash.clone(),
                optimized_expr: current_expr.clone(),
                metadata: metadata.clone(),
                cached_at: Instant::now(),
            };
            self.optimization_cache.insert(expr_hash, cached);
        }
        
        Ok(StaticOptimizationResult {
            optimized_expr: current_expr,
            optimization_applied,
            metadata,
            statistics: self.statistics.clone(),
        })
    }
    
    /// Calculate expression complexity
    fn calculate_expression_complexity(&self, expr: &Expr) -> usize {
        match expr {
            Expr::Literal(_) => 1,
            Expr::Variable(_) => 1,
            Expr::HygienicVariable(_) => 1,
            Expr::List(exprs) => 1 + exprs.iter().map(|e| self.calculate_expression_complexity(e)).sum::<usize>(),
            Expr::Quote(inner) => 1 + self.calculate_expression_complexity(inner),
            Expr::Quasiquote(inner) => 1 + self.calculate_expression_complexity(inner),
            Expr::Unquote(inner) => 1 + self.calculate_expression_complexity(inner),
            Expr::UnquoteSplicing(inner) => 1 + self.calculate_expression_complexity(inner),
            Expr::Vector(exprs) => 1 + exprs.iter().map(|e| self.calculate_expression_complexity(e)).sum::<usize>(),
            Expr::DottedList(exprs, tail) => 1 + exprs.iter().map(|e| self.calculate_expression_complexity(e)).sum::<usize>() + self.calculate_expression_complexity(tail),
        }
    }
    
    /// Calculate expression size (number of nodes)
    fn expression_size(&self, expr: &Expr) -> usize {
        self.calculate_expression_complexity(expr)
    }
    
    /// Hash an expression for caching
    fn hash_expression(&self, expr: &Expr) -> String {
        format!("{:?}", expr) // Simplified hashing
    }
    
    /// Update average optimization time
    fn update_average_optimization_time(&mut self) {
        if self.statistics.total_expressions > 0 {
            self.statistics.average_optimization_time = 
                self.statistics.total_optimization_time / self.statistics.total_expressions as u32;
        }
    }
    
    /// Update size reduction statistics
    fn update_size_reduction_stats(&mut self, initial_size: usize, final_size: usize) {
        self.statistics.size_reduction.nodes_before += initial_size;
        self.statistics.size_reduction.nodes_after += final_size;
        
        if self.statistics.total_expressions > 0 {
            let total_reduction = self.statistics.size_reduction.nodes_before as f64 
                - self.statistics.size_reduction.nodes_after as f64;
            self.statistics.size_reduction.average_reduction_percent = 
                (total_reduction / self.statistics.size_reduction.nodes_before as f64) * 100.0;
        }
        
        if initial_size > 0 {
            let current_reduction = ((initial_size - final_size) as f64 / initial_size as f64) * 100.0;
            if current_reduction > self.statistics.size_reduction.max_reduction_percent {
                self.statistics.size_reduction.max_reduction_percent = current_reduction;
            }
        }
    }
    
    /// Get optimization statistics
    pub fn get_statistics(&self) -> &StaticOptimizationStatistics {
        &self.statistics
    }
    
    /// Clear optimization cache
    pub fn clear_cache(&mut self) {
        self.optimization_cache.clear();
    }
    
    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.optimization_cache.len()
    }
    
    /// Reset statistics
    pub fn reset_statistics(&mut self) {
        self.statistics = StaticOptimizationStatistics::default();
    }
}

impl Default for StaticOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_constant_folding: true,
            enable_dead_code_elimination: true,
            enable_expression_rewriting: true,
            enable_cse: true,
            enable_loop_optimization: false, // More complex, disabled by default
            max_optimization_passes: 5,
            optimization_timeout: Duration::from_millis(100),
            enable_caching: true,
            min_complexity_threshold: 3,
        }
    }
}

impl Default for StaticOptimizationEngine {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Create a production-ready static optimization engine
pub fn create_production_optimizer() -> StaticOptimizationEngine {
    let config = StaticOptimizationConfig {
        enable_constant_folding: true,
        enable_dead_code_elimination: true,
        enable_expression_rewriting: true,
        enable_cse: true,
        enable_loop_optimization: true,
        max_optimization_passes: 10,
        optimization_timeout: Duration::from_millis(500),
        enable_caching: true,
        min_complexity_threshold: 5,
    };
    StaticOptimizationEngine::new(config)
}

/// Create a development-friendly static optimization engine
pub fn create_development_optimizer() -> StaticOptimizationEngine {
    let config = StaticOptimizationConfig {
        enable_constant_folding: true,
        enable_dead_code_elimination: false,
        enable_expression_rewriting: true,
        enable_cse: false,
        enable_loop_optimization: false,
        max_optimization_passes: 3,
        optimization_timeout: Duration::from_millis(50),
        enable_caching: false,
        min_complexity_threshold: 1,
    };
    StaticOptimizationEngine::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_static_optimization_engine_creation() {
        let optimizer = StaticOptimizationEngine::with_defaults();
        assert_eq!(optimizer.get_statistics().total_expressions, 0);
        assert_eq!(optimizer.cache_size(), 0);
    }

    #[test]
    fn test_expression_complexity_calculation() {
        let optimizer = StaticOptimizationEngine::with_defaults();
        
        let simple_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        assert_eq!(optimizer.calculate_expression_complexity(&simple_expr), 1);
        
        let list_expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]);
        assert_eq!(optimizer.calculate_expression_complexity(&list_expr), 4);
    }

    #[test]
    fn test_optimization_with_low_complexity() {
        let mut optimizer = StaticOptimizationEngine::with_defaults();
        let simple_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        
        let result = optimizer.optimize(&simple_expr, None).unwrap();
        assert!(!result.optimization_applied);
        assert_eq!(result.optimized_expr, simple_expr);
    }

    #[test]
    fn test_production_optimizer_configuration() {
        let optimizer = create_production_optimizer();
        assert!(optimizer.config.enable_loop_optimization);
        assert_eq!(optimizer.config.max_optimization_passes, 10);
    }

    #[test]
    fn test_development_optimizer_configuration() {
        let optimizer = create_development_optimizer();
        assert!(!optimizer.config.enable_loop_optimization);
        assert_eq!(optimizer.config.max_optimization_passes, 3);
    }

    #[test]
    fn test_cache_operations() {
        let mut optimizer = StaticOptimizationEngine::with_defaults();
        assert_eq!(optimizer.cache_size(), 0);
        
        optimizer.clear_cache();
        assert_eq!(optimizer.cache_size(), 0);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut optimizer = StaticOptimizationEngine::with_defaults();
        let stats = optimizer.get_statistics();
        
        assert_eq!(stats.total_expressions, 0);
        assert_eq!(stats.optimized_expressions, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
    }
}