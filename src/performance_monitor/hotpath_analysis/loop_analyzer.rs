//! Loop Characteristics Analysis Module
//!
//! このモジュールはループ特性解析システムを実装します。
//! ループネスト解析、アンロール最適化、ベクトル化解析を含みます。

use crate::ast::Expr;
use crate::error::Result;
use super::core_types::{
    LoopCharacteristics, LoopDependency, DependencyType,
    LoopNestingAnalyzer, LoopUnrollingAnalyzer, VectorizationAnalyzer,
};
use std::collections::HashMap;
use std::time::Duration;

/// Loop characteristics analysis
#[derive(Debug)]
pub struct LoopCharacteristicsAnalyzer {
    /// Detected loops
    pub loops: HashMap<String, LoopCharacteristics>,
    
    /// Loop nesting analysis
    pub nesting_analyzer: LoopNestingAnalyzer,
    
    /// Loop unrolling potential
    pub unrolling_analyzer: LoopUnrollingAnalyzer,
    
    /// Vectorization potential
    pub vectorization_analyzer: VectorizationAnalyzer,
}

/// Loop optimization opportunity
#[derive(Debug, Clone)]
pub struct LoopOptimizationOpportunity {
    /// Loop identifier
    pub loop_id: String,
    
    /// Optimization type
    pub optimization_type: LoopOptimizationType,
    
    /// Expected performance improvement
    pub expected_improvement: f64,
    
    /// Implementation complexity
    pub complexity: OptimizationComplexity,
}

/// Types of loop optimizations
#[derive(Debug, Clone, PartialEq)]
pub enum LoopOptimizationType {
    /// Loop unrolling
    Unrolling,
    
    /// Loop vectorization
    Vectorization,
    
    /// Loop parallelization
    Parallelization,
    
    /// Loop fusion
    Fusion,
    
    /// Loop tiling
    Tiling,
}

/// Optimization complexity levels
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationComplexity {
    /// Simple optimization
    Simple,
    
    /// Moderate complexity
    Moderate,
    
    /// Complex optimization
    Complex,
    
    /// Very complex optimization
    VeryComplex,
}

impl LoopCharacteristicsAnalyzer {
    #[must_use] 
    /// Create a new loop analyzer with default configuration
    pub fn new() -> Self { 
        Self { 
            loops: HashMap::new(), 
            nesting_analyzer: LoopNestingAnalyzer, 
            unrolling_analyzer: LoopUnrollingAnalyzer, 
            vectorization_analyzer: VectorizationAnalyzer,
        } 
    }
    
    /// Analyze loop characteristics for optimization opportunities
    /// 
    /// Examines loop structure and execution patterns to identify
    /// potential optimizations like unrolling and vectorization.
    pub fn analyze_loop(&mut self, expr_hash: &str, expr: &Expr, execution_time: Duration) -> Result<()> {
        // Pre-compute values to avoid borrowing issues
        let body_expressions = self.extract_loop_body(expr);
        let dependencies = self.analyze_dependencies(expr);
        let unroll_potential = self.calculate_unroll_potential(expr);
        let vectorizable = self.analyze_vectorization_potential(expr);
        let parallelizable = self.analyze_parallelization_potential(expr);
        
        let characteristics = self.loops.entry(expr_hash.to_string()).or_insert_with(|| {
            LoopCharacteristics {
                body_expressions,
                avg_iterations: 10.0, // Will be updated with actual measurements
                iteration_variance: 2.0, // Will be updated with actual measurements
                dependencies,
                memory_patterns: Vec::new(), // Will be updated by memory analyzer
                unroll_potential,
                vectorizable,
                parallelizable,
            }
        });
        
        // Update iteration statistics with moving average
        let new_iteration_estimate = Self::estimate_iterations_from_time_static(execution_time);
        characteristics.avg_iterations = characteristics.avg_iterations * 0.9 + new_iteration_estimate * 0.1;
        
        // Update variance
        let variance = (new_iteration_estimate - characteristics.avg_iterations).powi(2);
        characteristics.iteration_variance = characteristics.iteration_variance * 0.9 + variance * 0.1;
        
        Ok(())
    }
    
    /// Extract loop body expressions from the AST
    fn extract_loop_body(&self, expr: &Expr) -> Vec<String> {
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                match &exprs[0] {
                    Expr::Variable(name) if matches!(name.as_str(), "do" | "while" | "for") => {
                        // Extract loop body (simplified)
                        exprs.iter().skip(1).map(|e| format!("{e:?}")).collect()
                    },
                    _ => Vec::new(),
                }
            },
            _ => Vec::new(),
        }
    }
    
    /// Analyze loop-carried dependencies
    fn analyze_dependencies(&self, expr: &Expr) -> Vec<LoopDependency> {
        let mut dependencies = Vec::new();
        
        // Simplified dependency analysis
        if let Expr::List(exprs) = expr {
            for (i, expr) in exprs.iter().enumerate() {
                if let Expr::Variable(var) = expr {
                    // Look for uses of the same variable later in the loop
                    for (j, later_expr) in exprs.iter().enumerate().skip(i + 1) {
                        if self.expression_uses_variable(later_expr, var) {
                            dependencies.push(LoopDependency {
                                source: var.clone(),
                                target: format!("expr_{j}"),
                                distance: (j - i) as isize,
                                dependency_type: DependencyType::ReadAfterWrite,
                            });
                        }
                    }
                }
            }
        }
        
        dependencies
    }
    
    /// Check if an expression uses a specific variable
    fn expression_uses_variable(&self, expr: &Expr, var: &str) -> bool {
        match expr {
            Expr::Variable(name) => name == var,
            Expr::List(exprs) => exprs.iter().any(|e| self.expression_uses_variable(e, var)),
            _ => false,
        }
    }
    
    /// Calculate loop unrolling potential
    fn calculate_unroll_potential(&self, expr: &Expr) -> u32 {
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                match &exprs[0] {
                    Expr::Variable(name) if name == "do" => {
                        // Simple heuristic: short loops with few dependencies can be unrolled more
                        let body_size = exprs.len() - 1;
                        if body_size <= 3 {
                            8 // High unroll potential
                        } else if body_size <= 6 {
                            4 // Medium unroll potential
                        } else {
                            2 // Low unroll potential
                        }
                    },
                    _ => 1,
                }
            },
            _ => 1,
        }
    }
    
    /// Analyze vectorization potential
    fn analyze_vectorization_potential(&self, expr: &Expr) -> bool {
        match expr {
            Expr::List(exprs) => {
                // Look for arithmetic operations on arrays/vectors
                exprs.iter().any(|e| match e {
                    Expr::List(inner) if !inner.is_empty() => {
                        match &inner[0] {
                            Expr::Variable(name) => matches!(name.as_str(), "+" | "-" | "*" | "/" | "map"),
                            _ => false,
                        }
                    },
                    _ => false,
                })
            },
            _ => false,
        }
    }
    
    /// Analyze parallelization potential
    fn analyze_parallelization_potential(&self, expr: &Expr) -> bool {
        // Simplified analysis: loops with no dependencies can be parallelized
        let dependencies = self.analyze_dependencies(expr);
        
        // Check for loop-carried dependencies
        dependencies.iter().all(|dep| dep.distance > 1 || dep.dependency_type == DependencyType::ReadAfterWrite)
    }
    
    /// Estimate number of iterations from execution time
    fn estimate_iterations_from_time_static(execution_time: Duration) -> f64 {
        // Very rough heuristic: assume each iteration takes about 1μs
        let micros = execution_time.as_micros() as f64;
        (micros / 1000.0).max(1.0)
    }
    
    /// Identify optimization opportunities for analyzed loops
    /// 
    /// Returns a list of potential optimizations based on loop characteristics
    /// and performance analysis.
    #[must_use] pub fn identify_optimization_opportunities(&self) -> Vec<LoopOptimizationOpportunity> {
        self.loops.iter().map(|(loop_id, characteristics)| {
            // Determine best optimization based on loop characteristics
            let (opt_type, improvement, complexity) = if characteristics.unroll_potential > 4 && characteristics.avg_iterations < 100.0 {
                (LoopOptimizationType::Unrolling, 
                 f64::from(characteristics.unroll_potential) * 0.15, 
                 OptimizationComplexity::Simple)
            } else if characteristics.vectorizable && characteristics.avg_iterations > 50.0 {
                (LoopOptimizationType::Vectorization, 
                 2.0, // Vectorization can give significant speedup
                 OptimizationComplexity::Moderate)
            } else if characteristics.parallelizable && characteristics.avg_iterations > 100.0 {
                (LoopOptimizationType::Parallelization, 
                 3.0, // Parallelization can give even more speedup
                 OptimizationComplexity::Complex)
            } else if characteristics.dependencies.is_empty() {
                (LoopOptimizationType::Fusion, 
                 1.2, // Loop fusion gives modest improvement
                 OptimizationComplexity::Moderate)
            } else {
                (LoopOptimizationType::Tiling, 
                 1.1, // Cache tiling gives small improvement
                 OptimizationComplexity::VeryComplex)
            };
            
            LoopOptimizationOpportunity {
                loop_id: loop_id.clone(),
                optimization_type: opt_type,
                expected_improvement: improvement,
                complexity,
            }
        }).collect()
    }
    
    /// Get loop statistics
    #[must_use] pub fn get_loop_statistics(&self) -> LoopStatistics {
        let total_loops = self.loops.len();
        let total_iterations: f64 = self.loops.values()
            .map(|chars| chars.avg_iterations)
            .sum();
        
        let average_iterations = if total_loops > 0 {
            total_iterations / total_loops as f64
        } else {
            0.0
        };
        
        let vectorizable_count = self.loops.values()
            .filter(|chars| chars.vectorizable)
            .count();
        
        let parallelizable_count = self.loops.values()
            .filter(|chars| chars.parallelizable)
            .count();
        
        let high_unroll_potential_count = self.loops.values()
            .filter(|chars| chars.unroll_potential > 4)
            .count();
        
        LoopStatistics {
            total_loops_tracked: total_loops,
            average_iterations_per_loop: average_iterations,
            vectorizable_loops: vectorizable_count,
            parallelizable_loops: parallelizable_count,
            high_unroll_potential_loops: high_unroll_potential_count,
        }
    }
    
    /// Identify loop hotspots
    #[must_use] pub fn identify_loop_hotspots(&self, min_iterations: f64) -> Vec<LoopHotspot> {
        let mut hotspots = Vec::new();
        
        for (loop_id, characteristics) in &self.loops {
            if characteristics.avg_iterations >= min_iterations {
                let optimization_score = self.calculate_optimization_score(characteristics);
                
                hotspots.push(LoopHotspot {
                    loop_id: loop_id.clone(),
                    average_iterations: characteristics.avg_iterations,
                    iteration_variance: characteristics.iteration_variance,
                    unroll_potential: characteristics.unroll_potential,
                    vectorizable: characteristics.vectorizable,
                    parallelizable: characteristics.parallelizable,
                    optimization_score,
                });
            }
        }
        
        // Sort by optimization score (descending)
        hotspots.sort_by(|a, b| b.optimization_score.partial_cmp(&a.optimization_score)
                               .unwrap_or(std::cmp::Ordering::Equal));
        hotspots
    }
    
    /// Calculate overall optimization score for a loop
    fn calculate_optimization_score(&self, characteristics: &LoopCharacteristics) -> f64 {
        let iteration_factor = (characteristics.avg_iterations / 10.0).ln().max(0.0);
        let unroll_factor = f64::from(characteristics.unroll_potential) * 0.1;
        let vectorization_bonus = if characteristics.vectorizable { 1.0 } else { 0.0 };
        let parallelization_bonus = if characteristics.parallelizable { 2.0 } else { 0.0 };
        
        iteration_factor + unroll_factor + vectorization_bonus + parallelization_bonus
    }
}

/// Loop analysis statistics
#[derive(Debug, Clone)]
pub struct LoopStatistics {
    /// Total number of loops being tracked
    pub total_loops_tracked: usize,
    
    /// Average iterations per loop
    pub average_iterations_per_loop: f64,
    
    /// Number of vectorizable loops
    pub vectorizable_loops: usize,
    
    /// Number of parallelizable loops
    pub parallelizable_loops: usize,
    
    /// Number of loops with high unroll potential
    pub high_unroll_potential_loops: usize,
}

/// Loop hotspot information
#[derive(Debug, Clone)]
pub struct LoopHotspot {
    /// Loop identifier
    pub loop_id: String,
    
    /// Average number of iterations
    pub average_iterations: f64,
    
    /// Variance in iteration count
    pub iteration_variance: f64,
    
    /// Unrolling potential
    pub unroll_potential: u32,
    
    /// Whether the loop is vectorizable
    pub vectorizable: bool,
    
    /// Whether the loop is parallelizable
    pub parallelizable: bool,
    
    /// Overall optimization score
    pub optimization_score: f64,
}

impl Default for LoopCharacteristicsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}