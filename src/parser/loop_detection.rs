//! Infinite loop detection system for preventing runtime hangs

use crate::ast::Expr;
use crate::error::{LambdustError, Result, SourceSpan};
use crate::parser::cycle_detector::{CycleDetector, CycleType};
use crate::parser::dependency_analyzer::DependencyAnalyzer;
// use std::collections::HashMap;

/// Configuration for loop detection
#[derive(Debug, Clone)]
pub struct LoopDetectionConfig {
    /// Enable cycle detection
    pub enable_cycle_detection: bool,
    /// Maximum recursion depth for analysis
    pub max_recursion_depth: usize,
    /// Maximum dependency depth to analyze
    pub max_dependency_depth: usize,
    /// Only warn instead of erroring
    pub warn_only: bool,
}

impl Default for LoopDetectionConfig {
    fn default() -> Self {
        Self {
            enable_cycle_detection: true,
            max_recursion_depth: 100,
            max_dependency_depth: 50,
            warn_only: false,
        }
    }
}

/// Types of infinite loops that can be detected
#[derive(Debug, Clone, PartialEq)]
pub enum InfiniteLoopType {
    /// Variable circular dependency
    VariableCircularDependency,
    /// Function infinite recursion
    FunctionInfiniteRecursion,
    /// Mutual recursion without base case
    MutualRecursionWithoutBaseCase,
    /// Structural circular reference
    StructuralCircularReference,
}

/// Information about detected infinite loop
#[derive(Debug, Clone)]
pub struct InfiniteLoopInfo {
    /// Type of infinite loop
    pub loop_type: InfiniteLoopType,
    /// Nodes involved in the loop
    pub involved_nodes: Vec<String>,
    /// Cycle path
    pub cycle_path: Vec<String>,
    /// Suggested fix
    pub suggested_fix: String,
}

/// Infinite loop detection engine
pub struct InfiniteLoopDetector {
    /// Configuration
    config: LoopDetectionConfig,
    /// Dependency analyzer
    dependency_analyzer: DependencyAnalyzer,
}

impl InfiniteLoopDetector {
    /// Create a new infinite loop detector
    #[must_use] pub fn new(config: LoopDetectionConfig) -> Self {
        Self {
            config,
            dependency_analyzer: DependencyAnalyzer::new(),
        }
    }

    /// Detect infinite loops in a list of expressions
    pub fn detect_infinite_loops(&mut self, expressions: &[Expr]) -> Result<Vec<InfiniteLoopInfo>> {
        if !self.config.enable_cycle_detection {
            return Ok(Vec::new());
        }

        // Step 1: Build dependency graph
        let graph = self.dependency_analyzer.analyze_expressions(expressions);

        // Step 2: Detect cycles
        let cycle_detector = CycleDetector::new(graph.clone());
        let cycle_result = cycle_detector.detect_cycles();

        // Step 3: Analyze cycles for infinite loops
        let mut infinite_loops = Vec::new();

        for cycle in &cycle_result.cycles {
            // Skip cycles that are not truly infinite
            if self.is_cycle_actually_infinite(cycle) {
                let loop_info = self.analyze_cycle(cycle, &cycle_detector)?;
                infinite_loops.push(loop_info);
            }
        }

        // Step 4: Return results or errors
        if !infinite_loops.is_empty() && !self.config.warn_only {
            // Create ParseError for the first infinite loop
            let first_loop = &infinite_loops[0];
            return Err(self.create_infinite_loop_error(first_loop));
        }

        Ok(infinite_loops)
    }

    /// Check if a cycle is actually infinite (no escape conditions)
    fn is_cycle_actually_infinite(&self, cycle: &crate::parser::cycle_detector::Cycle) -> bool {
        match cycle.cycle_type {
            CycleType::SelfReference => {
                // Check if it's a variable or function
                if self.is_variable_definition(&cycle.nodes[0]) {
                    // Variable circular dependency is always infinite
                    true
                } else {
                    // For functions, check if they have base cases
                    !self.has_base_case(&cycle.nodes)
                }
            }
            CycleType::Direct | CycleType::Indirect => {
                // For multi-node cycles, check if any have escape conditions
                !self.has_escape_condition(&cycle.nodes)
            }
        }
    }

    /// Analyze a cycle to determine if it's an infinite loop
    fn analyze_cycle(
        &self,
        cycle: &crate::parser::cycle_detector::Cycle,
        _detector: &CycleDetector,
    ) -> Result<InfiniteLoopInfo> {
        let loop_type = match cycle.cycle_type {
            CycleType::SelfReference => {
                // Check if it's a variable or function
                if self.is_variable_definition(&cycle.nodes[0]) {
                    InfiniteLoopType::VariableCircularDependency
                } else {
                    InfiniteLoopType::FunctionInfiniteRecursion
                }
            }
            CycleType::Direct => {
                // Check if functions have base cases
                if self.has_base_case(&cycle.nodes) {
                    // Not actually infinite - has escape condition
                    InfiniteLoopType::MutualRecursionWithoutBaseCase
                } else {
                    InfiniteLoopType::MutualRecursionWithoutBaseCase
                }
            }
            CycleType::Indirect => {
                // Complex cycle - analyze for escape conditions
                if self.has_escape_condition(&cycle.nodes) {
                    InfiniteLoopType::MutualRecursionWithoutBaseCase
                } else {
                    InfiniteLoopType::VariableCircularDependency
                }
            }
        };

        let suggested_fix = self.generate_suggested_fix(&loop_type, &cycle.nodes);

        Ok(InfiniteLoopInfo {
            loop_type,
            involved_nodes: cycle.nodes.clone(),
            cycle_path: cycle.nodes.clone(),
            suggested_fix,
        })
    }

    /// Check if a node represents a variable definition
    fn is_variable_definition(&self, node_name: &str) -> bool {
        let graph = self.dependency_analyzer.get_graph();
        if let Some(node) = graph.get_nodes().get(node_name) {
            matches!(
                node.dependency_type,
                crate::parser::dependency_analyzer::DependencyType::Variable
            )
        } else {
            false
        }
    }

    /// Check if functions in the cycle have base cases
    fn has_base_case(&self, nodes: &[String]) -> bool {
        let graph = self.dependency_analyzer.get_graph();

        for node_name in nodes {
            if let Some(node) = graph.get_nodes().get(node_name) {
                if matches!(
                    node.dependency_type,
                    crate::parser::dependency_analyzer::DependencyType::Function
                ) {
                    // Analyze the function body for conditional expressions
                    if self.has_conditional_expression(&node.expr) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if there's an escape condition in the cycle
    fn has_escape_condition(&self, nodes: &[String]) -> bool {
        // For now, assume that any conditional expression provides an escape
        // This is a simplification - more sophisticated analysis would be needed
        self.has_base_case(nodes)
    }

    /// Check if an expression contains conditional logic
    #[allow(clippy::only_used_in_recursion)]
    fn has_conditional_expression(&self, expr: &Expr) -> bool {
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                if let Expr::Variable(name) = &exprs[0] {
                    match name.as_str() {
                        "if" | "cond" | "case" | "when" | "unless" => true,
                        _ => {
                            // Recursively check subexpressions
                            exprs.iter().any(|e| self.has_conditional_expression(e))
                        }
                    }
                } else {
                    exprs.iter().any(|e| self.has_conditional_expression(e))
                }
            }
            Expr::Vector(exprs) => exprs.iter().any(|e| self.has_conditional_expression(e)),
            Expr::DottedList(exprs, tail) => {
                exprs.iter().any(|e| self.has_conditional_expression(e))
                    || self.has_conditional_expression(tail)
            }
            Expr::Quote(_) => false, // Quoted expressions don't execute
            Expr::Quasiquote(expr) => self.has_conditional_expression(expr),
            Expr::Unquote(expr) => self.has_conditional_expression(expr),
            Expr::UnquoteSplicing(expr) => self.has_conditional_expression(expr),
            _ => false,
        }
    }

    /// Generate a suggested fix for the infinite loop
    fn generate_suggested_fix(&self, loop_type: &InfiniteLoopType, nodes: &[String]) -> String {
        match loop_type {
            InfiniteLoopType::VariableCircularDependency => {
                format!(
                    "Break the circular dependency by removing the cycle: {}",
                    nodes.join(" → ")
                )
            }
            InfiniteLoopType::FunctionInfiniteRecursion => {
                format!(
                    "Add a base case to the function '{}' to prevent infinite recursion",
                    nodes[0]
                )
            }
            InfiniteLoopType::MutualRecursionWithoutBaseCase => {
                format!(
                    "Add termination conditions to the mutually recursive functions: {}",
                    nodes.join(", ")
                )
            }
            InfiniteLoopType::StructuralCircularReference => {
                format!(
                    "Remove the structural circular reference involving: {}",
                    nodes.join(", ")
                )
            }
        }
    }

    /// Create a `ParseError` for an infinite loop
    fn create_infinite_loop_error(&self, loop_info: &InfiniteLoopInfo) -> LambdustError {
        let error_message = match loop_info.loop_type {
            InfiniteLoopType::VariableCircularDependency => {
                format!(
                    "Circular dependency detected: {}",
                    loop_info.cycle_path.join(" → ")
                )
            }
            InfiniteLoopType::FunctionInfiniteRecursion => {
                format!(
                    "Infinite recursion detected in function '{}'",
                    loop_info.involved_nodes[0]
                )
            }
            InfiniteLoopType::MutualRecursionWithoutBaseCase => {
                format!(
                    "Mutual recursion without base case: {}",
                    loop_info.involved_nodes.join(" ↔ ")
                )
            }
            InfiniteLoopType::StructuralCircularReference => {
                format!(
                    "Structural circular reference detected: {}",
                    loop_info.involved_nodes.join(", ")
                )
            }
        };

        LambdustError::ParseError {
            message: format!("Infinite loop detected: {error_message}"),
            location: SourceSpan::unknown(),
        }
    }
}

/// Convenience function to check expressions for infinite loops
pub fn check_for_infinite_loops(expressions: &[Expr]) -> Result<()> {
    let mut detector = InfiniteLoopDetector::new(LoopDetectionConfig::default());
    let loops = detector.detect_infinite_loops(expressions)?;

    if loops.is_empty() {
        Ok(())
    } else {
        // This should already be handled by detect_infinite_loops when warn_only is false
        Ok(())
    }
}

/// Convenience function to check expressions with custom config
pub fn check_for_infinite_loops_with_config(
    expressions: &[Expr],
    config: LoopDetectionConfig,
) -> Result<Vec<InfiniteLoopInfo>> {
    let mut detector = InfiniteLoopDetector::new(config);
    detector.detect_infinite_loops(expressions)
}

