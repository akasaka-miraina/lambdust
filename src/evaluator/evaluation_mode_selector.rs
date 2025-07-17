//! Advanced evaluation mode selection system
//!
//! This module provides intelligent evaluation mode selection based on
//! expression analysis, historical performance data, and optimization hints.

use crate::ast::Expr;
use crate::evaluator::{EvaluationMode, PerformanceMetrics};
use crate::executor::RuntimeOptimizationLevel;
use std::collections::HashMap;

/// Intelligent evaluation mode selector
pub struct EvaluationModeSelector {
    /// Performance history for different expression types
    performance_history: HashMap<ExpressionType, PerformanceStats>,

    /// Optimization effectiveness tracking
    #[allow(dead_code)]
    optimization_effectiveness: HashMap<RuntimeOptimizationLevel, f64>,

    /// Expression complexity threshold for mode switching
    complexity_threshold: usize,

    /// Minimum performance improvement required for optimization
    min_improvement_threshold: f64,

    /// Historical decision tracking
    decision_history: Vec<ModeDecision>,
}

/// Expression type classification for performance tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpressionType {
    /// Simple literal values (numbers, strings, booleans)
    Literal,
    /// Variable lookups
    Variable,
    /// Basic arithmetic operations (+, -, *, /)
    SimpleArithmetic,
    /// Complex mathematical operations
    ComplexArithmetic,
    /// Function call expressions
    FunctionCall,
    /// Lambda expressions and closures
    Lambda,
    /// Conditional expressions (if, cond, case)
    ConditionalExpression,
    /// List processing operations
    ListProcessing,
    /// Recursive function calls
    RecursiveFunction,
    /// Complex nested expressions
    ComplexNested,
    /// Unknown or unclassified expression type
    Unknown,
}

/// Performance statistics for expression types
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    /// Average semantic evaluation time (microseconds)
    avg_semantic_time: f64,

    /// Average runtime evaluation time (microseconds)
    avg_runtime_time: f64,

    /// Average speedup achieved by runtime optimization
    avg_speedup: f64,

    /// Success rate of runtime optimization
    success_rate: f64,

    /// Number of evaluations tracked
    sample_count: usize,
}

/// Mode selection decision record
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ModeDecision {
    /// Expression type that was evaluated
    expression_type: ExpressionType,

    /// Selected evaluation mode
    selected_mode: EvaluationMode,

    /// Actual performance achieved
    actual_performance: PerformanceMetrics,

    /// Whether the decision was optimal in hindsight
    was_optimal: bool,

    /// Timestamp of decision
    timestamp: std::time::SystemTime,
}

/// Mode selection criteria
#[derive(Debug, Clone)]
pub struct SelectionCriteria {
    /// Expression being evaluated
    pub expression: Expr,

    /// Expected value type (if known)
    pub expected_type: Option<String>,

    /// Performance requirements
    pub performance_requirements: PerformanceRequirements,

    /// Evaluation context
    pub context: EvaluationContext,
}

/// Performance requirements for evaluation
#[derive(Debug, Clone)]
pub struct PerformanceRequirements {
    /// Maximum acceptable evaluation time (microseconds)
    pub max_time_us: Option<u64>,

    /// Minimum acceptable accuracy
    pub min_accuracy: f64,

    /// Priority: 0.0 (low) to 1.0 (high)
    pub priority: f64,

    /// Whether correctness verification is required
    pub require_verification: bool,
}

/// Evaluation context information
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Current recursion depth
    pub recursion_depth: usize,

    /// Available memory (bytes)
    pub available_memory: usize,

    /// Whether this is a hot path
    pub is_hot_path: bool,

    /// Whether this is in a loop
    pub is_in_loop: bool,

    /// Recent performance trend
    pub recent_performance: PerformanceTrend,
}

/// Performance trend indicator
#[derive(Debug, Clone)]
pub enum PerformanceTrend {
    /// Performance metrics are showing consistent improvement over time
    Improving,
    /// Performance metrics remain stable with minimal fluctuation
    Stable,
    /// Performance metrics are showing degradation or decline
    Degrading,
    /// Performance trend cannot be determined due to insufficient data
    Unknown,
}

impl EvaluationModeSelector {
    /// Create new evaluation mode selector
    #[must_use] pub fn new() -> Self {
        Self {
            performance_history: HashMap::new(),
            optimization_effectiveness: Self::init_optimization_effectiveness(),
            complexity_threshold: 10,
            min_improvement_threshold: 0.1, // 10% minimum improvement
            decision_history: Vec::new(),
        }
    }

    /// Initialize optimization effectiveness tracking
    fn init_optimization_effectiveness() -> HashMap<RuntimeOptimizationLevel, f64> {
        let mut effectiveness = HashMap::new();
        effectiveness.insert(RuntimeOptimizationLevel::None, 1.0);
        effectiveness.insert(RuntimeOptimizationLevel::Conservative, 0.8);
        effectiveness.insert(RuntimeOptimizationLevel::Balanced, 0.6);
        effectiveness.insert(RuntimeOptimizationLevel::Aggressive, 0.4);
        effectiveness
    }

    /// Select optimal evaluation mode based on criteria
    pub fn select_mode(&mut self, criteria: &SelectionCriteria) -> EvaluationMode {
        let expression_type = self.classify_expression(&criteria.expression);
        let complexity = self.calculate_complexity(&criteria.expression);

        // Get historical performance data
        let performance_stats = self
            .performance_history
            .get(&expression_type)
            .cloned()
            .unwrap_or_default();

        // Decision algorithm based on multiple factors
        let selected_mode = self.make_decision(
            &expression_type,
            complexity,
            &performance_stats,
            &criteria.performance_requirements,
            &criteria.context,
        );

        // Record decision for future learning
        self.record_decision(ModeDecision {
            expression_type,
            selected_mode: selected_mode.clone(),
            actual_performance: PerformanceMetrics::default(), // Will be updated later
            was_optimal: false,                                // Will be determined later
            timestamp: std::time::SystemTime::now(),
        });

        selected_mode
    }

    /// Classify expression type for performance tracking
    fn classify_expression(&self, expr: &Expr) -> ExpressionType {
        match expr {
            Expr::Literal(_) => ExpressionType::Literal,
            Expr::Variable(_) => ExpressionType::Variable,
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    return ExpressionType::Unknown;
                }

                match &exprs[0] {
                    Expr::Variable(name) => {
                        match name.as_str() {
                            "+" | "-" | "*" | "/" => {
                                if exprs.len() <= 3 {
                                    ExpressionType::SimpleArithmetic
                                } else {
                                    ExpressionType::ComplexArithmetic
                                }
                            }
                            "if" | "cond" => ExpressionType::ConditionalExpression,
                            "lambda" => ExpressionType::Lambda,
                            "map" | "filter" | "fold" | "reduce" => ExpressionType::ListProcessing,
                            _ => {
                                // Check for recursive patterns
                                if self.is_recursive_pattern(expr) {
                                    ExpressionType::RecursiveFunction
                                } else if self.is_complex_nested(expr) {
                                    ExpressionType::ComplexNested
                                } else {
                                    ExpressionType::FunctionCall
                                }
                            }
                        }
                    }
                    _ => {
                        if self.is_complex_nested(expr) {
                            ExpressionType::ComplexNested
                        } else {
                            ExpressionType::FunctionCall
                        }
                    }
                }
            }
            _ => ExpressionType::Unknown,
        }
    }

    /// Calculate expression complexity
    fn calculate_complexity(&self, expr: &Expr) -> usize {
        match expr {
            Expr::Literal(_) | Expr::Variable(_) => 1,
            Expr::List(exprs) => {
                1 + exprs
                    .iter()
                    .map(|e| self.calculate_complexity(e))
                    .sum::<usize>()
            }
            Expr::Quote(expr) => 1 + self.calculate_complexity(expr),
            Expr::Vector(exprs) => {
                1 + exprs
                    .iter()
                    .map(|e| self.calculate_complexity(e))
                    .sum::<usize>()
            }
            _ => 3, // Default complexity for other expressions
        }
    }

    /// Check if expression contains recursive patterns
    fn is_recursive_pattern(&self, expr: &Expr) -> bool {
        match expr {
            Expr::List(exprs) => {
                if exprs.len() >= 2 {
                    if let (Expr::Variable(func_name), Expr::Variable(arg_name)) =
                        (&exprs[0], &exprs[1])
                    {
                        if func_name == arg_name {
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Check if expression is complex nested
    fn is_complex_nested(&self, expr: &Expr) -> bool {
        self.calculate_complexity(expr) > 20
    }

    /// Make evaluation mode decision
    fn make_decision(
        &self,
        expression_type: &ExpressionType,
        complexity: usize,
        performance_stats: &PerformanceStats,
        requirements: &PerformanceRequirements,
        context: &EvaluationContext,
    ) -> EvaluationMode {
        // Rule 1: High priority or verification required -> use verification mode
        if requirements.require_verification || requirements.priority > 0.9 {
            return EvaluationMode::Verification;
        }

        // Rule 2: Simple expressions always use semantic evaluation
        if complexity <= 3
            && matches!(
                expression_type,
                ExpressionType::Literal | ExpressionType::Variable
            )
        {
            return EvaluationMode::Semantic;
        }

        // Rule 3: Hot path with proven optimization effectiveness
        if context.is_hot_path && performance_stats.avg_speedup > self.min_improvement_threshold {
            return EvaluationMode::Runtime(self.select_optimization_level(
                expression_type,
                complexity,
                performance_stats,
                context,
            ));
        }

        // Rule 4: Complex expressions with good optimization history
        if complexity > self.complexity_threshold
            && performance_stats.success_rate > 0.8
            && performance_stats.avg_speedup > 0.2
        {
            return EvaluationMode::Runtime(self.select_optimization_level(
                expression_type,
                complexity,
                performance_stats,
                context,
            ));
        }

        // Rule 5: Memory constraints favor semantic evaluation
        if context.available_memory < 1_000_000 {
            // Less than 1MB
            return EvaluationMode::Semantic;
        }

        // Rule 6: Loop context with balanced optimization
        if context.is_in_loop {
            return EvaluationMode::Runtime(RuntimeOptimizationLevel::Balanced);
        }

        // Rule 7: Performance degradation trend -> fall back to semantic
        if matches!(context.recent_performance, PerformanceTrend::Degrading) {
            return EvaluationMode::Semantic;
        }

        // Default: Auto mode for adaptive selection
        EvaluationMode::Auto
    }

    /// Select appropriate optimization level
    fn select_optimization_level(
        &self,
        expression_type: &ExpressionType,
        complexity: usize,
        _performance_stats: &PerformanceStats,
        _context: &EvaluationContext,
    ) -> RuntimeOptimizationLevel {
        match expression_type {
            ExpressionType::RecursiveFunction => RuntimeOptimizationLevel::Aggressive,
            ExpressionType::ComplexNested => RuntimeOptimizationLevel::Balanced,
            ExpressionType::ListProcessing => RuntimeOptimizationLevel::Balanced,
            _ => {
                if complexity > 50 {
                    RuntimeOptimizationLevel::Aggressive
                } else if complexity > 20 {
                    RuntimeOptimizationLevel::Balanced
                } else {
                    RuntimeOptimizationLevel::Conservative
                }
            }
        }
    }

    /// Record mode selection decision
    fn record_decision(&mut self, decision: ModeDecision) {
        self.decision_history.push(decision);

        // Keep only recent decisions (last 1000)
        if self.decision_history.len() > 1000 {
            self.decision_history.remove(0);
        }
    }

    /// Update performance statistics after evaluation
    pub fn update_performance_stats(
        &mut self,
        expression_type: ExpressionType,
        mode_used: EvaluationMode,
        performance: PerformanceMetrics,
    ) {
        let stats = self
            .performance_history
            .entry(expression_type.clone())
            .or_default();

        // Update statistics using exponential moving average
        let alpha = 0.1; // Learning rate

        match mode_used {
            EvaluationMode::Semantic => {
                stats.avg_semantic_time = alpha * performance.semantic_time_us as f64
                    + (1.0 - alpha) * stats.avg_semantic_time;
            }
            EvaluationMode::Runtime(_) => {
                stats.avg_runtime_time = alpha * performance.runtime_time_us as f64
                    + (1.0 - alpha) * stats.avg_runtime_time;

                if performance.semantic_time_us > 0 {
                    let speedup =
                        performance.semantic_time_us as f64 / performance.runtime_time_us as f64;
                    stats.avg_speedup = alpha * speedup + (1.0 - alpha) * stats.avg_speedup;
                }
            }
            EvaluationMode::Verification => {
                // Update both semantic and runtime statistics
                stats.avg_semantic_time = alpha * performance.semantic_time_us as f64
                    + (1.0 - alpha) * stats.avg_semantic_time;
                stats.avg_runtime_time = alpha * performance.runtime_time_us as f64
                    + (1.0 - alpha) * stats.avg_runtime_time;
            }
            EvaluationMode::Auto => {
                // Handle auto mode based on actual mode used
                // This would need additional context about which mode was actually selected
            }
        }

        stats.sample_count += 1;
    }

    /// Get performance statistics for an expression type
    #[must_use] pub fn get_performance_stats(
        &self,
        expression_type: &ExpressionType,
    ) -> Option<&PerformanceStats> {
        self.performance_history.get(expression_type)
    }

    /// Get recent decision history
    #[must_use] pub fn get_decision_history(&self) -> &[ModeDecision] {
        &self.decision_history
    }

    /// Clear performance history
    pub fn clear_history(&mut self) {
        self.performance_history.clear();
        self.decision_history.clear();
    }

    /// Get mode selection recommendations
    #[must_use] pub fn get_recommendations(&self, expression_type: &ExpressionType) -> Vec<String> {
        let mut recommendations = Vec::new();

        if let Some(stats) = self.performance_history.get(expression_type) {
            if stats.success_rate < 0.5 {
                recommendations
                    .push("Consider using semantic evaluation for better reliability".to_string());
            }

            if stats.avg_speedup < 0.1 {
                recommendations.push(
                    "Runtime optimization provides minimal benefit for this expression type"
                        .to_string(),
                );
            }

            if stats.sample_count < 10 {
                recommendations.push("More data needed for reliable mode selection".to_string());
            }
        } else {
            recommendations
                .push("No historical data available for this expression type".to_string());
        }

        recommendations
    }
}

impl Default for EvaluationModeSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PerformanceRequirements {
    fn default() -> Self {
        Self {
            max_time_us: None,
            min_accuracy: 1.0,
            priority: 0.5,
            require_verification: false,
        }
    }
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self {
            recursion_depth: 0,
            available_memory: 100_000_000, // 100MB default
            is_hot_path: false,
            is_in_loop: false,
            recent_performance: PerformanceTrend::Unknown,
        }
    }
}

