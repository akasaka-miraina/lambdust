//! JIT Loop Optimization System
//!
//! This module implements JIT compilation for iterative constructs to eliminate
//! CPS stack overhead through native iteration code generation.
//!
//! Architecture:
//! - `LoopPattern`: Detection and classification of loop structures
//! - `NativeCodeGenerator`: Rust for-loop generation from Scheme constructs
//! - `JitCompiler`: Hot path identification and compile-time optimization
//! - `IterativeCodeCache`: Compiled native code caching and reuse

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::expression_analyzer::EvaluationComplexity;
use crate::evaluator::{Continuation, Evaluator};
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// Loop pattern classification for JIT optimization
#[derive(Debug, Clone, PartialEq)]
pub enum LoopPattern {
    /// Simple counting loop (do ((i start end)) (test) body)
    CountingLoop {
        /// Loop variable name
        variable: String,
        /// Starting value
        start: i64,
        /// Ending value
        end: i64,
        /// Step increment
        step: i64,
    },

    /// List iteration loop (for-each pattern)
    ListIteration {
        /// Iterator variable name
        variable: String,
        /// List expression to iterate over
        list_expr: Expr,
    },

    /// Vector iteration loop
    VectorIteration {
        /// Iterator variable name
        variable: String,
        /// Vector expression to iterate over
        vector_expr: Expr,
    },

    /// Conditional accumulation loop
    AccumulationLoop {
        /// Accumulator variable name
        accumulator: String,
        /// Loop termination condition
        condition: Expr,
        /// Accumulator update expression
        update_expr: Expr,
    },

    /// Complex loop requiring fallback to CPS
    ComplexLoop,
}

/// JIT compilation hint for optimization strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JitHint {
    /// Compile to native iteration immediately
    CompileImmediate,
    /// Compile after threshold executions
    CompileDeferred,
    /// Profile and decide at runtime
    ProfileAndDecide,
    /// Do not compile - use CPS evaluation
    NoCompile,
}

/// Native iteration strategy
#[derive(Debug, Clone, PartialEq)]
pub enum IterationStrategy {
    /// Rust for-loop with integer range
    NativeForLoop {
        /// Starting value
        start: i64,
        /// Ending value
        end: i64,
        /// Step increment
        step: i64,
    },


    /// Manual loop with exit conditions
    ManualLoop {
        /// Maximum allowed iterations
        max_iterations: usize,
    },

}

/// Iterator type for native iteration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IteratorType {
    /// Integer range iteration
    Range,
    /// List iteration
    List,
    /// Vector iteration
    Vector,
    /// Custom iterator
    Custom,
}

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

/// Compiled native loop representation
#[derive(Debug, Clone)]
pub struct CompiledLoop {
    /// Original pattern that was compiled
    #[allow(dead_code)]
    pattern: LoopPattern,
    /// Native iteration strategy
    strategy: IterationStrategy,
    /// Compilation timestamp
    #[allow(dead_code)]
    compiled_at: std::time::Instant,
    /// Execution count since compilation
    #[allow(dead_code)]
    execution_count: usize,
    /// Performance metrics
    #[allow(dead_code)]
    average_execution_time: std::time::Duration,
}

impl JitHotPathDetector {
    /// Create new JIT hot path detector
    #[must_use] pub fn new(compilation_threshold: usize) -> Self {
        JitHotPathDetector {
            execution_counts: HashMap::new(),
            compilation_threshold,
            total_executions: 0,
            compiled_patterns: HashMap::new(),
        }
    }

    /// Record loop execution and return JIT hint
    pub fn record_execution(&mut self, pattern_id: &str) -> JitHint {
        let count = self
            .execution_counts
            .entry(pattern_id.to_string())
            .or_insert(0);
        *count += 1;
        self.total_executions += 1;

        // Check if already compiled
        if self.compiled_patterns.contains_key(pattern_id) {
            return JitHint::CompileImmediate; // Use existing compilation
        }

        // Decide compilation strategy based on execution frequency
        if *count >= self.compilation_threshold {
            JitHint::CompileImmediate
        } else if *count >= self.compilation_threshold / 2 {
            JitHint::CompileDeferred
        } else if *count >= 3 {
            JitHint::ProfileAndDecide
        } else {
            JitHint::NoCompile
        }
    }

    /// Register compiled loop
    pub fn register_compiled_loop(&mut self, pattern_id: String, compiled: CompiledLoop) {
        self.compiled_patterns.insert(pattern_id, compiled);
    }

    /// Get compiled loop if available
    #[must_use] pub fn get_compiled_loop(&self, pattern_id: &str) -> Option<&CompiledLoop> {
        self.compiled_patterns.get(pattern_id)
    }

    /// Get compilation statistics
    #[must_use] pub fn compilation_statistics(&self) -> (usize, usize, f64) {
        let total_patterns = self.execution_counts.len();
        let compiled_count = self.compiled_patterns.len();
        let compilation_rate = if total_patterns > 0 {
            compiled_count as f64 / total_patterns as f64
        } else {
            0.0
        };
        (compiled_count, total_patterns, compilation_rate)
    }

    /// Clear compilation cache
    pub fn clear_cache(&mut self) {
        self.compiled_patterns.clear();
        self.execution_counts.clear();
        self.total_executions = 0;
    }
}

impl Default for JitHotPathDetector {
    fn default() -> Self {
        Self::new(5) // Default: compile after 5 executions
    }
}

/// Loop pattern analyzer for JIT compilation
#[derive(Debug)]
pub struct LoopPatternAnalyzer {
    /// Pattern detection statistics
    detection_stats: HashMap<String, usize>,
    /// Detected patterns cache
    pattern_cache: HashMap<String, LoopPattern>,
}

impl LoopPatternAnalyzer {
    /// Create new loop pattern analyzer
    #[must_use] pub fn new() -> Self {
        LoopPatternAnalyzer {
            detection_stats: HashMap::new(),
            pattern_cache: HashMap::new(),
        }
    }

    /// Analyze do-loop for pattern detection
    pub fn analyze_do_loop(&mut self, operands: &[Expr]) -> Result<Option<LoopPattern>> {
        if operands.len() < 2 {
            return Ok(Some(LoopPattern::ComplexLoop));
        }

        // Parse variable bindings
        let bindings = match &operands[0] {
            Expr::List(var_clauses) => self.parse_variable_bindings(var_clauses)?,
            _ => return Ok(Some(LoopPattern::ComplexLoop)),
        };

        // Parse test clause
        let test_expr = match &operands[1] {
            Expr::List(test_clause) if !test_clause.is_empty() => &test_clause[0],
            _ => return Ok(Some(LoopPattern::ComplexLoop)),
        };

        // Detect specific patterns
        if let Some(pattern) = self.detect_counting_loop(&bindings, test_expr)? {
            self.record_pattern_detection("counting_loop");
            return Ok(Some(pattern));
        }

        if let Some(pattern) = self.detect_list_iteration(&bindings, test_expr)? {
            self.record_pattern_detection("list_iteration");
            return Ok(Some(pattern));
        }

        if let Some(pattern) = self.detect_accumulation_loop(&bindings, test_expr)? {
            self.record_pattern_detection("accumulation_loop");
            return Ok(Some(pattern));
        }

        // Default to complex loop
        self.record_pattern_detection("complex_loop");
        Ok(Some(LoopPattern::ComplexLoop))
    }

    /// Parse variable bindings from do-loop
    fn parse_variable_bindings(
        &self,
        var_clauses: &[Expr],
    ) -> Result<Vec<(String, Expr, Option<Expr>)>> {
        let mut bindings = Vec::new();

        for clause in var_clauses {
            match clause {
                Expr::List(parts) if parts.len() >= 2 => {
                    let var_name = match &parts[0] {
                        Expr::Variable(name) => name.clone(),
                        _ => {
                            return Err(LambdustError::syntax_error(
                                "do binding variable must be identifier".to_string(),
                            ));
                        }
                    };

                    let init_expr = parts[1].clone();
                    let step_expr = if parts.len() > 2 {
                        Some(parts[2].clone())
                    } else {
                        None
                    };

                    bindings.push((var_name, init_expr, step_expr));
                }
                _ => {
                    return Err(LambdustError::syntax_error(
                        "invalid do binding format".to_string(),
                    ));
                }
            }
        }

        Ok(bindings)
    }

    /// Detect counting loop pattern: (do ((i 0 (+ i 1))) ((>= i n)) ...)
    fn detect_counting_loop(
        &self,
        bindings: &[(String, Expr, Option<Expr>)],
        test_expr: &Expr,
    ) -> Result<Option<LoopPattern>> {
        // Look for single integer variable with step increment
        if bindings.len() != 1 {
            return Ok(None);
        }

        let (var_name, init_expr, step_expr) = &bindings[0];

        // Check initial value is integer literal
        let start = match init_expr {
            Expr::Literal(crate::ast::Literal::Number(n)) => {
                if let Ok(i) = n.to_string().parse::<i64>() {
                    i
                } else {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        };

        // Check step expression is simple increment
        let step = match step_expr {
            Some(Expr::List(parts)) if parts.len() == 3 => {
                match (&parts[0], &parts[1], &parts[2]) {
                    (
                        Expr::Variable(op),
                        Expr::Variable(var),
                        Expr::Literal(crate::ast::Literal::Number(step_val)),
                    ) if op == "+" && var == var_name => {
                        if let Ok(s) = step_val.to_string().parse::<i64>() {
                            s
                        } else {
                            return Ok(None);
                        }
                    }
                    _ => return Ok(None),
                }
            }
            None => 1, // Default step
            _ => return Ok(None),
        };

        // Check test condition is simple comparison
        let end = match test_expr {
            Expr::List(parts) if parts.len() == 3 => match (&parts[0], &parts[1], &parts[2]) {
                (
                    Expr::Variable(op),
                    Expr::Variable(var),
                    Expr::Literal(crate::ast::Literal::Number(end_val)),
                ) if (op == ">=" || op == "<") && var == var_name => {
                    if let Ok(e) = end_val.to_string().parse::<i64>() {
                        e
                    } else {
                        return Ok(None);
                    }
                }
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };

        Ok(Some(LoopPattern::CountingLoop {
            variable: var_name.clone(),
            start,
            end,
            step,
        }))
    }

    /// Detect list iteration pattern
    fn detect_list_iteration(
        &self,
        bindings: &[(String, Expr, Option<Expr>)],
        _test_expr: &Expr,
    ) -> Result<Option<LoopPattern>> {
        // Simple heuristic: look for list-based variable
        for (var_name, init_expr, _) in bindings {
            if let Expr::List(_) = init_expr {
                return Ok(Some(LoopPattern::ListIteration {
                    variable: var_name.clone(),
                    list_expr: init_expr.clone(),
                }));
            }
        }
        Ok(None)
    }

    /// Detect accumulation loop pattern
    fn detect_accumulation_loop(
        &self,
        bindings: &[(String, Expr, Option<Expr>)],
        test_expr: &Expr,
    ) -> Result<Option<LoopPattern>> {
        // Look for accumulator variable with update expression
        for (var_name, _init_expr, step_expr) in bindings {
            if let Some(update) = step_expr {
                // Check if update expression involves the variable (accumulation)
                if self.expression_references_variable(update, var_name) {
                    return Ok(Some(LoopPattern::AccumulationLoop {
                        accumulator: var_name.clone(),
                        condition: test_expr.clone(),
                        update_expr: update.clone(),
                    }));
                }
            }
        }
        Ok(None)
    }

    /// Check if expression references a specific variable
    #[allow(clippy::only_used_in_recursion)]
    fn expression_references_variable(&self, expr: &Expr, var_name: &str) -> bool {
        match expr {
            Expr::Variable(name) => name == var_name,
            Expr::List(exprs) => exprs
                .iter()
                .any(|e| self.expression_references_variable(e, var_name)),
            Expr::Vector(exprs) => exprs
                .iter()
                .any(|e| self.expression_references_variable(e, var_name)),
            Expr::Quote(inner) => self.expression_references_variable(inner, var_name),
            Expr::Quasiquote(inner) => self.expression_references_variable(inner, var_name),
            _ => false,
        }
    }

    /// Record pattern detection for statistics
    fn record_pattern_detection(&mut self, pattern_type: &str) {
        *self
            .detection_stats
            .entry(pattern_type.to_string())
            .or_insert(0) += 1;
    }

    /// Get pattern detection statistics
    #[must_use] pub fn detection_statistics(&self) -> &HashMap<String, usize> {
        &self.detection_stats
    }

    /// Clear pattern cache
    pub fn clear_cache(&mut self) {
        self.pattern_cache.clear();
        self.detection_stats.clear();
    }
}

impl Default for LoopPatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Native code generator for iterative constructs
#[derive(Debug)]
pub struct NativeCodeGenerator {
    /// Generated code cache
    code_cache: HashMap<String, GeneratedCode>,
    /// Generation statistics
    generation_stats: HashMap<String, usize>,
}

/// Generated native iteration code
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    /// Iteration strategy
    pub strategy: IterationStrategy,
    /// Performance characteristics
    pub characteristics: CodeCharacteristics,
    /// Generated at timestamp
    pub generated_at: std::time::Instant,
}

/// Code performance characteristics
#[derive(Debug, Clone)]
pub struct CodeCharacteristics {
    /// Estimated iterations per second
    pub iterations_per_second: f64,
    /// Memory overhead per iteration
    pub memory_overhead: usize,
    /// CPU cache friendliness (0.0-1.0)
    pub cache_friendliness: f64,
}

impl NativeCodeGenerator {
    /// Create new native code generator
    #[must_use] pub fn new() -> Self {
        NativeCodeGenerator {
            code_cache: HashMap::new(),
            generation_stats: HashMap::new(),
        }
    }

    /// Generate native iteration code for pattern
    pub fn generate_native_code(&mut self, pattern: &LoopPattern) -> Result<GeneratedCode> {
        let strategy = self.select_iteration_strategy(pattern)?;
        let characteristics = self.estimate_performance_characteristics(&strategy);

        let code = GeneratedCode {
            strategy,
            characteristics,
            generated_at: std::time::Instant::now(),
        };

        // Cache generated code
        let pattern_id = self.pattern_to_id(pattern);
        self.code_cache.insert(pattern_id.clone(), code.clone());

        // Update generation statistics
        *self.generation_stats.entry(pattern_id).or_insert(0) += 1;

        Ok(code)
    }

    /// Select optimal iteration strategy for pattern
    fn select_iteration_strategy(&self, pattern: &LoopPattern) -> Result<IterationStrategy> {
        match pattern {
            LoopPattern::CountingLoop {
                start, end, step, ..
            } => Ok(IterationStrategy::NativeForLoop {
                start: *start,
                end: *end,
                step: *step,
            }),

            LoopPattern::ListIteration { .. } => Ok(IterationStrategy::ManualLoop {
                max_iterations: 10000,
            }),

            LoopPattern::VectorIteration { .. } => Ok(IterationStrategy::ManualLoop {
                max_iterations: 10000,
            }),

            LoopPattern::AccumulationLoop { .. } => {
                Ok(IterationStrategy::ManualLoop {
                    max_iterations: 100_000, // Safety limit
                })
            }

            LoopPattern::ComplexLoop => Ok(IterationStrategy::ManualLoop {
                max_iterations: 1000,
            }),
        }
    }

    /// Estimate performance characteristics of strategy
    #[must_use] pub fn estimate_performance_characteristics(
        &self,
        strategy: &IterationStrategy,
    ) -> CodeCharacteristics {
        match strategy {
            IterationStrategy::NativeForLoop { .. } => {
                CodeCharacteristics {
                    iterations_per_second: 10_000_000.0, // Very fast
                    memory_overhead: 16,                 // Minimal overhead
                    cache_friendliness: 0.95,            // Excellent cache locality
                }
            }

            IterationStrategy::ManualLoop { .. } => {
                CodeCharacteristics {
                    iterations_per_second: 500_000.0, // Moderate speed
                    memory_overhead: 96,              // Higher overhead
                    cache_friendliness: 0.50,         // Poor cache locality
                }
            }

        }
    }

    /// Convert pattern to cache ID
    fn pattern_to_id(&self, pattern: &LoopPattern) -> String {
        match pattern {
            LoopPattern::CountingLoop {
                variable,
                start,
                end,
                step,
            } => {
                format!("counting_{variable}_{start}_{end}_{step}")
            }
            LoopPattern::ListIteration { variable, .. } => {
                format!("list_iter_{variable}")
            }
            LoopPattern::VectorIteration { variable, .. } => {
                format!("vector_iter_{variable}")
            }
            LoopPattern::AccumulationLoop { accumulator, .. } => {
                format!("accumulation_{accumulator}")
            }
            LoopPattern::ComplexLoop => "complex".to_string(),
        }
    }

    /// Get generation statistics
    #[must_use] pub fn generation_statistics(&self) -> &HashMap<String, usize> {
        &self.generation_stats
    }

    /// Clear code cache
    pub fn clear_cache(&mut self) {
        self.code_cache.clear();
        self.generation_stats.clear();
    }
}

impl Default for NativeCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// JIT Loop Optimizer - main coordination system
#[derive(Debug)]
pub struct JitLoopOptimizer {
    /// Loop pattern analyzer
    pub pattern_analyzer: LoopPatternAnalyzer,
    /// Native code generator
    pub code_generator: NativeCodeGenerator,
    /// Hot path detector
    hot_path_detector: JitHotPathDetector,
    /// Optimization enabled flag
    optimization_enabled: bool,
}

impl JitLoopOptimizer {
    /// Create new JIT loop optimizer
    #[must_use] pub fn new() -> Self {
        JitLoopOptimizer {
            pattern_analyzer: LoopPatternAnalyzer::new(),
            code_generator: NativeCodeGenerator::new(),
            hot_path_detector: JitHotPathDetector::new(5),
            optimization_enabled: true,
        }
    }

    /// Create with custom compilation threshold
    #[must_use] pub fn with_threshold(threshold: usize) -> Self {
        JitLoopOptimizer {
            pattern_analyzer: LoopPatternAnalyzer::new(),
            code_generator: NativeCodeGenerator::new(),
            hot_path_detector: JitHotPathDetector::new(threshold),
            optimization_enabled: true,
        }
    }

    /// Attempt JIT optimization of do-loop
    pub fn try_optimize_do_loop(
        &mut self,
        evaluator: &mut Evaluator,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Option<Value>> {
        if !self.optimization_enabled {
            return Ok(None);
        }

        // Enhanced analysis using ExpressionAnalyzer
        let loop_complexity = self.analyze_loop_complexity(evaluator, operands, &env)?;

        // Skip optimization for very complex loops
        if matches!(loop_complexity, EvaluationComplexity::High) {
            return Ok(None);
        }

        // Analyze loop pattern
        let pattern = match self.pattern_analyzer.analyze_do_loop(operands)? {
            Some(pattern) => pattern,
            None => return Ok(None),
        };
        let pattern_id = self.code_generator.pattern_to_id(&pattern);

        // Record execution and get JIT hint (influenced by complexity analysis)
        let base_hint = self.hot_path_detector.record_execution(&pattern_id);
        let jit_hint = self.adjust_hint_by_complexity(base_hint, loop_complexity);

        match jit_hint {
            JitHint::CompileImmediate => {
                // Check if already compiled
                if let Some(compiled) = self.hot_path_detector.get_compiled_loop(&pattern_id) {
                    self.execute_compiled_loop(evaluator, compiled, operands, env, cont)
                } else {
                    // Compile and execute
                    let generated = self.code_generator.generate_native_code(&pattern)?;
                    let compiled = CompiledLoop {
                        pattern,
                        strategy: generated.strategy.clone(),
                        compiled_at: std::time::Instant::now(),
                        execution_count: 0,
                        average_execution_time: std::time::Duration::from_nanos(0),
                    };

                    self.hot_path_detector
                        .register_compiled_loop(pattern_id, compiled.clone());
                    self.execute_compiled_loop(evaluator, &compiled, operands, env, cont)
                }
            }

            JitHint::CompileDeferred | JitHint::ProfileAndDecide => {
                // For now, defer to CPS evaluation
                // In a full implementation, we would queue for background compilation
                Ok(None)
            }

            JitHint::NoCompile => {
                // Use regular CPS evaluation
                Ok(None)
            }
        }
    }

    /// Execute compiled native loop
    fn execute_compiled_loop(
        &self,
        evaluator: &mut Evaluator,
        compiled: &CompiledLoop,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Option<Value>> {
        match &compiled.strategy {
            IterationStrategy::NativeForLoop { start, end, step } => {
                self.execute_native_for_loop(evaluator, *start, *end, *step, operands, env, cont)
            }

            // IteratorBased removed - using ManualLoop instead

            IterationStrategy::ManualLoop { max_iterations } => {
                self.execute_manual_loop(evaluator, *max_iterations, operands, env, cont)
            }

            // CpsFallback removed - using ManualLoop instead
        }
    }

    /// Execute native for-loop iteration
    #[allow(clippy::too_many_arguments)]
    fn execute_native_for_loop(
        &self,
        evaluator: &mut Evaluator,
        start: i64,
        end: i64,
        step: i64,
        operands: &[Expr],
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Option<Value>> {
        // Parse loop structure
        let (var_name, body_exprs, result_exprs) = self.parse_loop_structure(operands)?;

        // Create loop environment
        let loop_env = Environment::new();
        let loop_env_rc = Rc::new(loop_env.extend());

        // Native Rust for-loop - zero CPS overhead!
        let mut current = start;
        while (step > 0 && current < end) || (step < 0 && current > end) {
            // Set loop variable
            loop_env_rc.define(var_name.clone(), Value::from(current));

            // Execute body expressions
            for expr in &body_exprs {
                evaluator.eval(expr.clone(), loop_env_rc.clone(), Continuation::Identity)?;
            }

            current += step;
        }

        // Evaluate result expressions
        let result = if result_exprs.is_empty() {
            Value::Undefined
        } else if result_exprs.len() == 1 {
            evaluator.eval(result_exprs[0].clone(), loop_env_rc, Continuation::Identity)?
        } else {
            // Multiple results - evaluate last one
            let last_idx = result_exprs.len() - 1;
            let mut final_result = Value::Undefined;
            for (i, expr) in result_exprs.iter().enumerate() {
                if i == last_idx {
                    final_result = evaluator.eval(
                        expr.clone(),
                        loop_env_rc.clone(),
                        Continuation::Identity,
                    )?;
                } else {
                    evaluator.eval(expr.clone(), loop_env_rc.clone(), Continuation::Identity)?;
                }
            }
            final_result
        };

        // Apply continuation with result
        Ok(Some(evaluator.apply_continuation(cont, result)?))
    }

    /// Execute iterator-based loop
    #[allow(dead_code)]
    fn execute_iterator_based_loop(
        &self,
        _evaluator: &mut Evaluator,
        _iterator_type: &IteratorType,
        _operands: &[Expr],
        _env: Rc<Environment>,
        _cont: Continuation,
    ) -> Result<Option<Value>> {
        // Placeholder for iterator-based execution
        // In a full implementation, this would handle list/vector iteration
        Ok(None)
    }

    /// Execute manual loop with safety limits
    fn execute_manual_loop(
        &self,
        _evaluator: &mut Evaluator,
        _max_iterations: usize,
        _operands: &[Expr],
        _env: Rc<Environment>,
        _cont: Continuation,
    ) -> Result<Option<Value>> {
        // Placeholder for manual loop execution
        // In a full implementation, this would handle complex accumulation patterns
        Ok(None)
    }

    /// Parse do-loop structure for execution
    fn parse_loop_structure(&self, operands: &[Expr]) -> Result<(String, Vec<Expr>, Vec<Expr>)> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "invalid do-loop structure".to_string(),
            ));
        }

        // Extract variable name (simplified - assumes single variable)
        let var_name = match &operands[0] {
            Expr::List(bindings) if !bindings.is_empty() => match &bindings[0] {
                Expr::List(binding) if !binding.is_empty() => match &binding[0] {
                    Expr::Variable(name) => name.clone(),
                    _ => {
                        return Err(LambdustError::syntax_error(
                            "invalid variable binding".to_string(),
                        ));
                    }
                },
                _ => {
                    return Err(LambdustError::syntax_error(
                        "invalid binding format".to_string(),
                    ));
                }
            },
            _ => return Err(LambdustError::syntax_error("invalid bindings".to_string())),
        };

        // Extract result expressions
        let result_exprs = match &operands[1] {
            Expr::List(test_clause) => test_clause[1..].to_vec(),
            _ => Vec::new(),
        };

        // Body expressions
        let body_exprs = operands[2..].to_vec();

        Ok((var_name, body_exprs, result_exprs))
    }

    /// Enable/disable JIT optimization
    pub fn set_optimization_enabled(&mut self, enabled: bool) {
        self.optimization_enabled = enabled;
    }

    /// Get comprehensive optimization statistics
    #[must_use] pub fn optimization_statistics(&self) -> JitOptimizationStats {
        let (compiled_count, total_patterns, compilation_rate) =
            self.hot_path_detector.compilation_statistics();

        JitOptimizationStats {
            total_patterns,
            compiled_patterns: compiled_count,
            compilation_rate,
            pattern_detections: self.pattern_analyzer.detection_statistics().clone(),
            code_generations: self.code_generator.generation_statistics().clone(),
        }
    }

    /// Clear all optimization caches
    pub fn clear_caches(&mut self) {
        self.pattern_analyzer.clear_cache();
        self.code_generator.clear_cache();
        self.hot_path_detector.clear_cache();
    }

    /// Analyze loop complexity using `ExpressionAnalyzer`
    pub fn analyze_loop_complexity(
        &self,
        _evaluator: &Evaluator,
        operands: &[Expr],
        _env: &Rc<Environment>,
    ) -> Result<EvaluationComplexity> {
        let _analyzer = _evaluator.expression_analyzer();
        let mut max_complexity = EvaluationComplexity::Constant;

        // Analyze all operands for complexity (need mutable access)
        // For now, return simple heuristic based on operand structure
        for operand in operands {
            let operand_complexity = match operand {
                Expr::Literal(_) => EvaluationComplexity::Constant,
                Expr::Variable(_) => EvaluationComplexity::Variable,
                Expr::List(exprs) if exprs.len() <= 3 => EvaluationComplexity::Simple,
                Expr::List(exprs) if exprs.len() <= 10 => EvaluationComplexity::Moderate,
                _ => EvaluationComplexity::High,
            };

            if operand_complexity > max_complexity {
                max_complexity = operand_complexity;
            }
        }

        Ok(max_complexity)
    }

    /// Adjust JIT hint based on expression complexity analysis
    #[must_use] pub fn adjust_hint_by_complexity(
        &self,
        base_hint: JitHint,
        complexity: EvaluationComplexity,
    ) -> JitHint {
        match (base_hint, complexity) {
            // Simple expressions can be compiled more aggressively
            (JitHint::ProfileAndDecide, EvaluationComplexity::Constant) => {
                JitHint::CompileImmediate
            }
            (JitHint::ProfileAndDecide, EvaluationComplexity::Variable) => {
                JitHint::CompileImmediate
            }

            // Moderate complexity requires more conservative approach
            (JitHint::CompileImmediate, EvaluationComplexity::Moderate) => JitHint::CompileDeferred,
            (JitHint::CompileDeferred, EvaluationComplexity::Moderate) => JitHint::ProfileAndDecide,

            // High complexity should not be compiled
            (_, EvaluationComplexity::High) => JitHint::NoCompile,

            // Keep original hint for other cases
            _ => base_hint,
        }
    }
}

impl Default for JitLoopOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// JIT optimization statistics
#[derive(Debug, Clone)]
pub struct JitOptimizationStats {
    /// Total loop patterns detected
    pub total_patterns: usize,
    /// Successfully compiled patterns
    pub compiled_patterns: usize,
    /// Compilation success rate
    pub compilation_rate: f64,
    /// Pattern detection breakdown
    pub pattern_detections: HashMap<String, usize>,
    /// Code generation breakdown
    pub code_generations: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::evaluator::Evaluator;

    #[test]
    fn test_counting_loop_pattern_detection() {
        let mut analyzer = LoopPatternAnalyzer::new();

        // (do ((i 0 (+ i 1))) ((>= i 10)) body)
        let bindings = Expr::List(vec![Expr::List(vec![
            Expr::Variable("i".to_string()),
            Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(0))),
            Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Variable("i".to_string()),
                Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(1))),
            ]),
        ])]);

        let test_clause = Expr::List(vec![Expr::List(vec![
            Expr::Variable(">=".to_string()),
            Expr::Variable("i".to_string()),
            Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(10))),
        ])]);

        let operands = &[bindings, test_clause];
        let pattern = analyzer.analyze_do_loop(operands).unwrap();

        match pattern {
            Some(LoopPattern::CountingLoop {
                variable,
                start,
                end,
                step,
            }) => {
                assert_eq!(variable, "i");
                assert_eq!(start, 0);
                assert_eq!(end, 10);
                assert_eq!(step, 1);
            }
            _ => panic!("Expected counting loop pattern"),
        }
    }

    #[test]
    fn test_jit_hot_path_detector() {
        let mut detector = JitHotPathDetector::new(3);

        // Based on threshold=3: compilation_threshold/2 = 1, compilation_threshold = 3
        assert_eq!(
            detector.record_execution("pattern1"),
            JitHint::CompileDeferred
        ); // count=1 >= threshold/2=1
        assert_eq!(
            detector.record_execution("pattern1"),
            JitHint::CompileDeferred
        ); // count=2 >= threshold/2=1
        assert_eq!(
            detector.record_execution("pattern1"),
            JitHint::CompileImmediate
        ); // count=3 >= threshold=3

        // Subsequent executions should compile immediately
        assert_eq!(
            detector.record_execution("pattern1"),
            JitHint::CompileImmediate
        );

        let (compiled, total, rate) = detector.compilation_statistics();
        assert_eq!(total, 1);
        assert_eq!(compiled, 0); // No loops actually compiled yet
        assert_eq!(rate, 0.0);
    }

    #[test]
    fn test_native_code_generation() {
        let mut generator = NativeCodeGenerator::new();

        let pattern = LoopPattern::CountingLoop {
            variable: "i".to_string(),
            start: 0,
            end: 100,
            step: 1,
        };

        let code = generator.generate_native_code(&pattern).unwrap();

        match code.strategy {
            IterationStrategy::NativeForLoop { start, end, step } => {
                assert_eq!(start, 0);
                assert_eq!(end, 100);
                assert_eq!(step, 1);
            }
            _ => panic!("Expected native for-loop strategy"),
        }

        // Check performance characteristics
        assert!(code.characteristics.iterations_per_second > 1_000_000.0);
        assert!(code.characteristics.cache_friendliness > 0.8);
    }

    #[test]
    fn test_jit_loop_optimizer_integration() {
        let mut optimizer = JitLoopOptimizer::with_threshold(2);
        let mut evaluator = Evaluator::new();
        let env = Rc::new(Environment::new());
        let cont = Continuation::Identity;

        // Simple counting loop
        let operands = &[
            Expr::List(vec![Expr::List(vec![
                Expr::Variable("i".to_string()),
                Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(0))),
                Expr::List(vec![
                    Expr::Variable("+".to_string()),
                    Expr::Variable("i".to_string()),
                    Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(1))),
                ]),
            ])]),
            Expr::List(vec![
                Expr::List(vec![
                    Expr::Variable(">=".to_string()),
                    Expr::Variable("i".to_string()),
                    Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(5))),
                ]),
                Expr::Variable("i".to_string()),
            ]),
        ];

        // First execution - should not optimize
        let result1 =
            optimizer.try_optimize_do_loop(&mut evaluator, operands, env.clone(), cont.clone());
        assert!(result1.is_ok());
        assert!(result1.unwrap().is_none()); // No optimization yet

        // Second execution - should trigger compilation
        let result2 =
            optimizer.try_optimize_do_loop(&mut evaluator, operands, env.clone(), cont.clone());
        assert!(result2.is_ok());
        // May return Some(value) if compilation succeeded

        let stats = optimizer.optimization_statistics();
        assert!(stats.total_patterns >= 1);
    }

    #[test]
    fn test_pattern_analyzer_statistics() {
        let mut analyzer = LoopPatternAnalyzer::new();

        // Analyze multiple patterns
        let operands1 = &[
            Expr::List(vec![Expr::List(vec![
                Expr::Variable("i".to_string()),
                Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(0))),
            ])]),
            Expr::List(vec![Expr::Literal(Literal::Boolean(false))]),
        ];

        analyzer.analyze_do_loop(operands1).unwrap();
        analyzer.analyze_do_loop(operands1).unwrap();

        let stats = analyzer.detection_statistics();
        assert!(stats.contains_key("complex_loop"));
        assert_eq!(stats["complex_loop"], 2);
    }
}

// Additional methods for AdvancedJITSystem integration
impl JitLoopOptimizer {
    /// Detect loop pattern from expression
    pub fn detect_loop_pattern(&mut self, expr: &Expr) -> Result<Option<LoopPattern>> {
        match expr {
            Expr::List(elements) if !elements.is_empty() => {
                if let Expr::Variable(name) = &elements[0] {
                    if name == "do" && elements.len() >= 3 {
                        // This is a do-loop
                        let operands = &elements[1..];
                        return self.pattern_analyzer.analyze_do_loop(operands);
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }
    
    /// Compile loop pattern to native implementation
    pub fn compile_loop(&mut self, pattern: &LoopPattern) -> Result<crate::evaluator::advanced_jit_system::NativeLoopImplementation> {
        match pattern {
            LoopPattern::CountingLoop { variable, start, end, step } => {
                let rust_code = format!(
                    "for {} in ({})..({}).step_by({}) {{ /* body */ }}",
                    variable, start, end, step.abs()
                );
                
                Ok(crate::evaluator::advanced_jit_system::NativeLoopImplementation {
                    rust_code,
                    machine_code_size: 128, // Estimated
                    estimated_cycles: ((end - start) / step).max(1) as u64 * 10,
                })
            }
            LoopPattern::ListIteration { variable, .. } => {
                let rust_code = format!("for {} in list.iter() {{ /* body */ }}", variable);
                
                Ok(crate::evaluator::advanced_jit_system::NativeLoopImplementation {
                    rust_code,
                    machine_code_size: 96, // Estimated
                    estimated_cycles: 100, // Estimated
                })
            }
            LoopPattern::VectorIteration { variable, .. } => {
                let rust_code = format!("for {} in vector.iter() {{ /* body */ }}", variable);
                
                Ok(crate::evaluator::advanced_jit_system::NativeLoopImplementation {
                    rust_code,
                    machine_code_size: 80, // Estimated
                    estimated_cycles: 80, // Estimated
                })
            }
            LoopPattern::AccumulationLoop { accumulator, .. } => {
                let rust_code = format!("let mut {} = init; while condition {{ {} = update({}); }}", accumulator, accumulator, accumulator);
                
                Ok(crate::evaluator::advanced_jit_system::NativeLoopImplementation {
                    rust_code,
                    machine_code_size: 150, // Estimated
                    estimated_cycles: 1000, // Variable
                })
            }
            LoopPattern::ComplexLoop => {
                Err(crate::error::LambdustError::runtime_error(
                    "Cannot compile complex loop pattern"
                ))
            }
        }
    }
}
