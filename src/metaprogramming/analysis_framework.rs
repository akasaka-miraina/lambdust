//! Core analysis framework and coordination structures.

use crate::ast::{Expr, Program};
use crate::diagnostics::{Result, Span, Spanned};
use crate::eval::Environment;
use super::analysis_types::*;
use super::profiling_analysis::Profiler;
use super::warning_system::AnalysisWarning;
use super::variable_scope_analysis::{VariableUsage, VariableInfo, ScopeInfo};
use super::control_flow_analysis::{ControlFlowGraph, BasicBlock};
use super::quality_metrics::{QualityMetrics, DuplicationInfo, OptimizationOpportunity};
use super::type_analysis::{TypeInformation, FunctionSignature, TypeConstraint, TypeError};
use super::dependency_analysis::{DependencyGraph, DependencyNode, DependencyEdge, DependencyAnalyzer};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Static analysis results for a program.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Dependency graph
    pub dependencies: DependencyGraph,
    /// Variable usage information
    pub variable_usage: VariableUsage,
    /// Control flow information
    pub control_flow: ControlFlowGraph,
    /// Type inference results
    pub type_info: TypeInformation,
    /// Code quality metrics
    pub quality_metrics: QualityMetrics,
    /// Optimization opportunities
    pub optimizations: Vec<OptimizationOpportunity>,
    /// Warnings and suggestions
    pub warnings: Vec<AnalysisWarning>,
}

/// Configuration for analysis.
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Enable dependency analysis
    pub dependency_analysis: bool,
    /// Enable variable usage analysis
    pub variable_usage_analysis: bool,
    /// Enable control flow analysis
    pub control_flow_analysis: bool,
    /// Enable type inference
    pub type_inference: bool,
    /// Enable quality metrics
    pub quality_metrics: bool,
    /// Enable optimization detection
    pub optimization_detection: bool,
    /// Warning levels
    pub warning_levels: HashMap<WarningType, WarningSeverity>,
}

/// Static analyzer for program analysis.
#[derive(Debug)]
pub struct StaticAnalyzer {
    /// Analysis configuration
    config: AnalysisConfig,
    /// Cache for analysis results
    cache: HashMap<String, AnalysisResult>,
}

/// Code analyzer for overall code analysis.
#[derive(Debug)]
pub struct CodeAnalyzer {
    /// Static analyzer
    static_analyzer: StaticAnalyzer,
    /// Dependency analyzer
    dependency_analyzer: DependencyAnalyzer,
    /// Profiler
    profiler: Profiler,
}

impl StaticAnalyzer {
    /// Install primitives for program analysis.
    pub fn install_primitives(&self, _env: &Rc<Environment>) -> Result<()> {
        // Static analysis primitives would be installed here
        Ok(())
    }
    /// Creates a new static analyzer.
    pub fn new() -> Self {
        Self {
            config: AnalysisConfig::default(),
            cache: HashMap::new(),
        }
    }

    /// Gets a reference to the analysis configuration.
    pub fn config(&self) -> &AnalysisConfig {
        &self.config
    }

    /// Analyzes a program.
    pub fn analyze(&mut self, program: &Program) -> Result<AnalysisResult> {
        let mut result = AnalysisResult {
            dependencies: DependencyGraph::new(),
            variable_usage: VariableUsage::new(),
            control_flow: ControlFlowGraph::new(),
            type_info: TypeInformation::new(),
            quality_metrics: QualityMetrics::new(),
            optimizations: Vec::new(),
            warnings: Vec::new(),
        };

        if self.config.dependency_analysis {
            result.dependencies = self.analyze_dependencies(program)?;
        }

        if self.config.variable_usage_analysis {
            result.variable_usage = self.analyze_variable_usage(program)?;
        }

        if self.config.control_flow_analysis {
            result.control_flow = self.analyze_control_flow(program)?;
        }

        if self.config.type_inference {
            result.type_info = self.infer_types(program)?;
        }

        if self.config.quality_metrics {
            result.quality_metrics = self.compute_quality_metrics(program)?;
        }

        if self.config.optimization_detection {
            result.optimizations = self.detect_optimizations(program, &result)?;
        }

        result.warnings = self.generate_warnings(&result)?;

        Ok(result)
    }

    /// Analyzes dependencies in a program.
    pub fn analyze_dependencies(&self, program: &Program) -> Result<DependencyGraph> {
        let mut graph = DependencyGraph::new();
        let mut analyzer = InternalDependencyAnalyzer::new();

        for expr in &program.expressions {
            analyzer.analyze_expression(expr, &mut graph)?;
        }

        // Detect cycles
        graph.cycles = analyzer.find_cycles(&graph);

        Ok(graph)
    }

    /// Analyzes variable usage.
    fn analyze_variable_usage(&self, program: &Program) -> Result<VariableUsage> {
        let mut usage = VariableUsage::new();
        let mut analyzer = VariableAnalyzer::new();

        for expr in &program.expressions {
            analyzer.analyze_expression(expr, &mut usage)?;
        }

        // Find unused variables
        usage.unused = analyzer.find_unused_variables(&usage);

        Ok(usage)
    }

    /// Analyzes control flow.
    fn analyze_control_flow(&self, program: &Program) -> Result<ControlFlowGraph> {
        let mut cfg = ControlFlowGraph::new();
        let mut analyzer = ControlFlowAnalyzer::new();

        for expr in &program.expressions {
            analyzer.analyze_expression(expr, &mut cfg)?;
        }

        // Compute dominators
        cfg.dominators = analyzer.compute_dominators(&cfg);

        Ok(cfg)
    }

    /// Infers types for expressions.
    fn infer_types(&self, program: &Program) -> Result<TypeInformation> {
        let mut type_info = TypeInformation::new();
        let mut inferrer = TypeInferrer::new();

        for expr in &program.expressions {
            inferrer.infer_expression(expr, &mut type_info)?;
        }

        // Solve constraints
        inferrer.solve_constraints(&mut type_info)?;

        Ok(type_info)
    }

    /// Computes quality metrics.
    fn compute_quality_metrics(&self, program: &Program) -> Result<QualityMetrics> {
        let mut metrics = QualityMetrics::new();
        let mut analyzer = QualityAnalyzer::new();

        for expr in &program.expressions {
            analyzer.analyze_expression(expr, &mut metrics)?;
        }

        Ok(metrics)
    }

    /// Detects optimization opportunities.
    fn detect_optimizations(&self, program: &Program, analysis: &AnalysisResult) -> Result<Vec<OptimizationOpportunity>> {
        let mut opportunities = Vec::new();
        let mut detector = OptimizationDetector::new();

        for expr in &program.expressions {
            let expr_opportunities = detector.detect_in_expression(expr, analysis)?;
            opportunities.extend(expr_opportunities);
        }

        Ok(opportunities)
    }

    /// Generates warnings based on analysis results.
    fn generate_warnings(&self, analysis: &AnalysisResult) -> Result<Vec<AnalysisWarning>> {
        let mut warnings = Vec::new();

        // Unused variables
        for unused in &analysis.variable_usage.unused {
            warnings.push(AnalysisWarning {
                warning_type: WarningType::UnusedVariable,
                message: format!("Variable '{unused}' is defined but never used"),
                location: None,
                severity: WarningSeverity::Warning,
            });
        }

        // Complex functions
        for (func_name, complexity) in &analysis.quality_metrics.complexity {
            if *complexity > 10 {
                warnings.push(AnalysisWarning {
                    warning_type: WarningType::ComplexFunction,
                    message: format!("Function '{func_name}' has high complexity ({complexity})"),
                    location: None,
                    severity: WarningSeverity::Warning,
                });
            }
        }

        Ok(warnings)
    }
}

impl CodeAnalyzer {
    /// Creates a new code analyzer.
    pub fn new() -> Self {
        Self {
            static_analyzer: StaticAnalyzer::new(),
            dependency_analyzer: DependencyAnalyzer::new(),
            profiler: Profiler::new(),
        }
    }

    /// Performs comprehensive analysis.
    pub fn analyze(&mut self, program: &Program) -> Result<AnalysisResult> {
        self.static_analyzer.analyze(program)
    }

    /// Gets the static analyzer.
    pub fn static_analyzer(&self) -> &StaticAnalyzer {
        &self.static_analyzer
    }

    /// Gets the dependency analyzer.
    pub fn dependency_analyzer(&self) -> &DependencyAnalyzer {
        &self.dependency_analyzer
    }

    /// Gets the profiler.
    pub fn profiler(&self) -> &Profiler {
        &self.profiler
    }
}

/// Dependency analyzer helper (internal).
struct InternalDependencyAnalyzer {
    current_scope: Vec<String>,
}

impl InternalDependencyAnalyzer {
    fn new() -> Self {
        Self {
            current_scope: Vec::new(),
        }
    }

    fn analyze_expression(&mut self, expr: &Spanned<Expr>, graph: &mut DependencyGraph) -> Result<()> {
        match &expr.inner {
            Expr::Identifier(name) => {
                if let Some(current) = self.current_scope.last() {
                    graph.add_dependency(current.clone(), name.clone(), DependencyType::Reference, Some(expr.span));
                }
            }
            Expr::Define { name, value, .. } => {
                self.current_scope.push(name.clone());
                graph.add_node(name.clone(), DefinitionType::Variable, Some(expr.span));
                self.analyze_expression(value, graph)?;
                self.current_scope.pop();
            }
            Expr::Lambda { body, .. } => {
                for body_expr in body {
                    self.analyze_expression(body_expr, graph)?;
                }
            }
            Expr::Application { operator, operands } => {
                self.analyze_expression(operator, graph)?;
                for operand in operands {
                    self.analyze_expression(operand, graph)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn find_cycles(&self, graph: &DependencyGraph) -> Vec<Vec<String>> {
        // Simplified cycle detection using DFS
        let mut visited = HashSet::new();
        let mut cycles = Vec::new();
        
        for node_name in graph.nodes.keys() {
            if !visited.contains(node_name) {
                let mut path = Vec::new();
                let mut path_set = HashSet::new();
                self.dfs_cycles(node_name, graph, &mut visited, &mut path, &mut path_set, &mut cycles);
            }
        }
        
        cycles
    }

    #[allow(clippy::only_used_in_recursion)]
    fn dfs_cycles(
        &self,
        node: &str,
        graph: &DependencyGraph,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
        path_set: &mut HashSet<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        if path_set.contains(node) {
            // Found a cycle
            if let Some(start_idx) = path.iter().position(|n| n == node) {
                cycles.push(path[start_idx..].to_vec());
            }
            return;
        }

        if visited.contains(node) {
            return;
        }

        visited.insert(node.to_string());
        path.push(node.to_string());
        path_set.insert(node.to_string());

        if let Some(node_info) = graph.nodes.get(node) {
            for dep in &node_info.dependencies {
                self.dfs_cycles(dep, graph, visited, path, path_set, cycles);
            }
        }

        path.pop();
        path_set.remove(node);
    }
}

/// Variable usage analyzer helper.
struct VariableAnalyzer {
    scopes: Vec<HashMap<String, VariableInfo>>,
}

impl VariableAnalyzer {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Global scope
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn analyze_expression(&mut self, expr: &Spanned<Expr>, usage: &mut VariableUsage) -> Result<()> {
        match &expr.inner {
            Expr::Identifier(name) => {
                if let Some(var_info) = self.find_variable_mut(name) {
                    var_info.read = true;
                    var_info.uses.push(expr.span);
                }
            }
            Expr::Define { name, value, .. } => {
                self.define_variable(name.clone(), Some(expr.span));
                self.analyze_expression(value, usage)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn define_variable(&mut self, name: String, location: Option<Span>) {
        let scope_level = self.scopes.len() - 1;
        let scope_id = format!("scope-{scope_level}");
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.clone(), VariableInfo {
            name,
            definition: location,
            uses: Vec::new(),
            read: false,
            written: true,
            captured: false,
            scope: ScopeInfo {
                scope_type: ScopeType::Global,
                level: scope_level,
                scope_id,
            },
        });
    }

    fn find_variable_mut(&mut self, name: &str) -> Option<&mut VariableInfo> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                return Some(var_info);
            }
        }
        None
    }

    fn find_unused_variables(&self, usage: &VariableUsage) -> Vec<String> {
        usage.variables.iter()
            .filter(|(_, info)| !info.read)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

// Placeholder implementations for other analyzers
struct ControlFlowAnalyzer;
struct TypeInferrer;
struct QualityAnalyzer;
struct OptimizationDetector;

impl ControlFlowAnalyzer {
    fn new() -> Self { Self }
    fn analyze_expression(&mut self, _expr: &Spanned<Expr>, _cfg: &mut ControlFlowGraph) -> Result<()> { Ok(()) }
    fn compute_dominators(&self, _cfg: &ControlFlowGraph) -> HashMap<String, String> { HashMap::new() }
}

impl TypeInferrer {
    fn new() -> Self { Self }
    fn infer_expression(&mut self, _expr: &Spanned<Expr>, _type_info: &mut TypeInformation) -> Result<()> { Ok(()) }
    fn solve_constraints(&mut self, _type_info: &mut TypeInformation) -> Result<()> { Ok(()) }
}

impl QualityAnalyzer {
    fn new() -> Self { Self }
    fn analyze_expression(&mut self, _expr: &Spanned<Expr>, _metrics: &mut QualityMetrics) -> Result<()> { Ok(()) }
}

impl OptimizationDetector {
    fn new() -> Self { Self }
    fn detect_in_expression(&mut self, _expr: &Spanned<Expr>, _analysis: &AnalysisResult) -> Result<Vec<OptimizationOpportunity>> { Ok(Vec::new()) }
}

// Default implementations
impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            dependency_analysis: true,
            variable_usage_analysis: true,
            control_flow_analysis: true,
            type_inference: true,
            quality_metrics: true,
            optimization_detection: true,
            warning_levels: HashMap::new(),
        }
    }
}

impl Default for StaticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}