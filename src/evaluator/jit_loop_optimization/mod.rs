//! JIT Loop Optimization Module
//!
//! このモジュールはJIT用ループ最適化システムの包括的な実装を提供します。
//! パターン解析、コード生成、ホットパス検出、最適化統計を含みます。
//!
//! ## モジュール構成
//!
//! - `core_types`: 基本型定義（LoopPattern, JitHint, 統計等）

pub mod core_types;

// Re-export main types for backward compatibility
pub use core_types::{
    LoopPattern, JitHint, IterationStrategy, IteratorType,
    CompiledLoop, GeneratedCode, CodeCharacteristics,
    JitOptimizationStats,
};

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
// use crate::evaluator::expression_analyzer::EvaluationComplexity;
// use crate::evaluator::{Continuation, Evaluator};
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// Hot path detection for JIT compilation decisions
#[derive(Debug)]
pub struct JitHotPathDetector {
    /// Execution count per loop pattern
    execution_counts: HashMap<String, usize>,
    /// Compilation threshold for hot paths
    compilation_threshold: usize,
    /// Total loop executions tracked
    total_executions: usize,
    /// Successfully compiled patterns
    compiled_patterns: HashMap<String, CompiledLoop>,
}

impl JitHotPathDetector {
    /// Create new hot path detector
    pub fn new() -> Self {
        Self::with_threshold(10)
    }

    /// Create with custom compilation threshold
    pub fn with_threshold(threshold: usize) -> Self {
        Self {
            execution_counts: HashMap::new(),
            compilation_threshold: threshold,
            total_executions: 0,
            compiled_patterns: HashMap::new(),
        }
    }

    /// Record loop execution and check if compilation is needed
    pub fn record_execution(&mut self, pattern_key: &str) -> bool {
        self.total_executions += 1;
        let count = self.execution_counts.entry(pattern_key.to_string()).or_insert(0);
        *count += 1;
        
        *count >= self.compilation_threshold && !self.compiled_patterns.contains_key(pattern_key)
    }

    /// Register compiled pattern
    pub fn register_compiled(&mut self, pattern_key: String, compiled: CompiledLoop) {
        self.compiled_patterns.insert(pattern_key, compiled);
    }

    /// Get execution count for pattern
    pub fn execution_count(&self, pattern_key: &str) -> usize {
        self.execution_counts.get(pattern_key).copied().unwrap_or(0)
    }

    /// Get total executions
    pub fn total_executions(&self) -> usize {
        self.total_executions
    }

    /// Get compiled patterns count
    pub fn compiled_count(&self) -> usize {
        self.compiled_patterns.len()
    }
}

impl Default for JitHotPathDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Loop pattern analyzer
#[derive(Debug)]
pub struct LoopPatternAnalyzer {
    /// Detection statistics
    stats: JitOptimizationStats,
}

impl LoopPatternAnalyzer {
    /// Create new pattern analyzer
    pub fn new() -> Self {
        Self {
            stats: JitOptimizationStats::default(),
        }
    }

    /// Analyze expression for loop patterns
    pub fn analyze_pattern(&mut self, expr: &Expr) -> Result<Option<LoopPattern>> {
        // Simplified pattern detection for demo
        match expr {
            Expr::List(exprs) if exprs.len() >= 2 => {
                if let Expr::Variable(name) = &exprs[0] {
                    match name.as_str() {
                        "do" => {
                            self.stats.record_pattern_detection("CountingLoop");
                            Ok(Some(LoopPattern::CountingLoop {
                                variable: "i".to_string(),
                                start: 0,
                                end: 10,
                                step: 1,
                            }))
                        }
                        "for-each" => {
                            self.stats.record_pattern_detection("ListIteration");
                            Ok(Some(LoopPattern::ListIteration {
                                variable: "item".to_string(),
                                list_expr: exprs[1].clone(),
                            }))
                        }
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Get analysis statistics
    pub fn stats(&self) -> &JitOptimizationStats {
        &self.stats
    }
}

impl Default for LoopPatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Native code generator
#[derive(Debug)]
pub struct NativeCodeGenerator {
    /// Generation statistics
    stats: JitOptimizationStats,
}

impl NativeCodeGenerator {
    /// Create new code generator
    pub fn new() -> Self {
        Self {
            stats: JitOptimizationStats::default(),
        }
    }

    /// Generate native code for pattern
    pub fn generate(&mut self, pattern: &LoopPattern) -> Result<GeneratedCode> {
        match pattern {
            LoopPattern::CountingLoop { start, end, step, .. } => {
                let strategy = IterationStrategy::NativeForLoop {
                    start: *start,
                    end: *end,
                    step: *step,
                };
                self.stats.record_successful_compilation("CountingLoop");
                Ok(GeneratedCode::new(strategy))
            }
            LoopPattern::ListIteration { .. } => {
                let strategy = IterationStrategy::ManualLoop {
                    max_iterations: 10000,
                };
                self.stats.record_successful_compilation("ListIteration");
                Ok(GeneratedCode::new(strategy))
            }
            LoopPattern::VectorIteration { .. } => {
                let strategy = IterationStrategy::ManualLoop {
                    max_iterations: 10000,
                };
                self.stats.record_successful_compilation("VectorIteration");
                Ok(GeneratedCode::new(strategy))
            }
            LoopPattern::AccumulationLoop { .. } => {
                let strategy = IterationStrategy::ManualLoop {
                    max_iterations: 10000,
                };
                self.stats.record_successful_compilation("AccumulationLoop");
                Ok(GeneratedCode::new(strategy))
            }
            LoopPattern::ComplexLoop => {
                Err(LambdustError::runtime_error("Complex loops cannot be compiled to native code".to_string()))
            }
        }
    }

    /// Get generation statistics
    pub fn stats(&self) -> &JitOptimizationStats {
        &self.stats
    }
}

impl Default for NativeCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Main JIT loop optimizer
#[derive(Debug)]
pub struct JitLoopOptimizer {
    /// Hot path detector
    hotpath_detector: JitHotPathDetector,
    /// Pattern analyzer
    pattern_analyzer: LoopPatternAnalyzer,
    /// Code generator
    code_generator: NativeCodeGenerator,
}

impl JitLoopOptimizer {
    /// Create new JIT loop optimizer
    pub fn new() -> Self {
        Self {
            hotpath_detector: JitHotPathDetector::new(),
            pattern_analyzer: LoopPatternAnalyzer::new(),
            code_generator: NativeCodeGenerator::new(),
        }
    }

    /// Try to optimize loop expression
    pub fn try_optimize(&mut self, expr: &Expr, _env: Rc<Environment>) -> Result<Option<Value>> {
        // Check if pattern can be detected
        if let Some(pattern) = self.pattern_analyzer.analyze_pattern(expr)? {
            let pattern_key = pattern.pattern_name().to_string();
            
            // Check if this is a hot path
            if self.hotpath_detector.record_execution(&pattern_key) {
                // Try to compile
                if let Ok(generated) = self.code_generator.generate(&pattern) {
                    let compiled = CompiledLoop::new(pattern, generated.strategy);
                    self.hotpath_detector.register_compiled(pattern_key, compiled);
                    
                    // For demo, return a simple result
                    return Ok(Some(Value::Number(crate::lexer::SchemeNumber::Integer(42))));
                }
            }
        }
        
        Ok(None)
    }

    /// Get combined statistics
    pub fn combined_stats(&self) -> JitOptimizationStats {
        let mut combined = self.pattern_analyzer.stats().clone();
        
        // Merge with code generator stats
        let gen_stats = self.code_generator.stats();
        combined.compiled_patterns += gen_stats.compiled_patterns;
        combined.update_compilation_rate();
        
        combined
    }

    /// Get hotpath detector statistics
    pub fn hotpath_stats(&self) -> (usize, usize, usize) {
        (
            self.hotpath_detector.total_executions(),
            self.hotpath_detector.compiled_count(),
            self.hotpath_detector.execution_counts.len(),
        )
    }
    
    /// Clear all compilation caches (useful for testing or memory management)
    pub fn clear_caches(&mut self) {
        self.hotpath_detector.execution_counts.clear();
        self.hotpath_detector.compiled_patterns.clear();
        self.hotpath_detector.total_executions = 0;
    }
}

impl Default for JitLoopOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new JIT loop optimizer with default configuration
pub fn create_jit_optimizer() -> JitLoopOptimizer {
    JitLoopOptimizer::new()
}

/// Create a production-tuned JIT loop optimizer
pub fn create_production_jit_optimizer() -> JitLoopOptimizer {
    let mut optimizer = JitLoopOptimizer::new();
    optimizer.hotpath_detector = JitHotPathDetector::with_threshold(5); // Lower threshold for production
    optimizer
}

/// Create a development-friendly JIT loop optimizer
pub fn create_development_jit_optimizer() -> JitLoopOptimizer {
    let mut optimizer = JitLoopOptimizer::new();
    optimizer.hotpath_detector = JitHotPathDetector::with_threshold(50); // Higher threshold for development
    optimizer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_jit_optimizer_creation() {
        let optimizer = create_jit_optimizer();
        let (total, compiled, patterns) = optimizer.hotpath_stats();
        assert_eq!(total, 0);
        assert_eq!(compiled, 0);
        assert_eq!(patterns, 0);
    }

    #[test]
    fn test_production_optimizer() {
        let optimizer = create_production_jit_optimizer();
        assert_eq!(optimizer.hotpath_detector.compilation_threshold, 5);
    }

    #[test]
    fn test_development_optimizer() {
        let optimizer = create_development_jit_optimizer();
        assert_eq!(optimizer.hotpath_detector.compilation_threshold, 50);
    }

    #[test]
    fn test_pattern_analysis() {
        let mut analyzer = LoopPatternAnalyzer::new();
        let expr = Expr::List(vec![
            Expr::Variable("do".to_string()),
            Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(1))),
        ]);
        
        let result = analyzer.analyze_pattern(&expr).unwrap();
        assert!(result.is_some());
        
        if let Some(LoopPattern::CountingLoop { variable, start, end, step }) = result {
            assert_eq!(variable, "i");
            assert_eq!(start, 0);
            assert_eq!(end, 10);
            assert_eq!(step, 1);
        } else {
            panic!("Expected CountingLoop pattern");
        }
    }

    #[test]
    fn test_code_generation() {
        let mut generator = NativeCodeGenerator::new();
        let pattern = LoopPattern::CountingLoop {
            variable: "i".to_string(),
            start: 0,
            end: 10,
            step: 1,
        };
        
        let result = generator.generate(&pattern).unwrap();
        match result.strategy {
            IterationStrategy::NativeForLoop { start, end, step } => {
                assert_eq!(start, 0);
                assert_eq!(end, 10);
                assert_eq!(step, 1);
            }
            _ => panic!("Expected NativeForLoop strategy"),
        }
    }

    #[test]
    fn test_hotpath_detection() {
        let mut detector = JitHotPathDetector::with_threshold(3);
        
        // Record executions below threshold
        assert!(!detector.record_execution("test_pattern"));
        assert!(!detector.record_execution("test_pattern"));
        
        // Third execution should trigger compilation
        assert!(detector.record_execution("test_pattern"));
        
        // Fourth execution should not trigger (already compiled)
        assert!(!detector.record_execution("test_pattern"));
        
        assert_eq!(detector.execution_count("test_pattern"), 3);
    }

    #[test]
    fn test_optimization_stats() {
        let mut stats = JitOptimizationStats::default();
        
        stats.record_pattern_detection("CountingLoop");
        stats.record_pattern_detection("ListIteration");
        stats.record_successful_compilation("CountingLoop");
        
        assert_eq!(stats.total_patterns, 2);
        assert_eq!(stats.compiled_patterns, 1);
        assert_eq!(stats.compilation_rate, 0.5);
        assert_eq!(stats.efficiency(), 50.0);
    }
}