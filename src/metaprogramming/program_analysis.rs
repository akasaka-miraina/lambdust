//! Program analysis tools for static analysis and optimization.
//!
//! This module provides comprehensive program analysis capabilities including
//! static analysis, dependency analysis, profiling, and code quality metrics.

use crate::ast::{Expr, Program};
use crate::diagnostics::{Result, Span, Spanned};
use crate::eval::Environment;
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

/// Dependency graph representing relationships between definitions.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Nodes in the graph (definitions)
    pub nodes: HashMap<String, DependencyNode>,
    /// Edges representing dependencies
    pub edges: Vec<DependencyEdge>,
    /// Strongly connected components (cycles)
    pub cycles: Vec<Vec<String>>,
}

/// Node in the dependency graph.
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// Name of the definition
    pub name: String,
    /// Type of definition
    pub definition_type: DefinitionType,
    /// Source location
    pub location: Option<Span>,
    /// Dependencies (outgoing edges)
    pub dependencies: HashSet<String>,
    /// Dependents (incoming edges)
    pub dependents: HashSet<String>,
}

/// Type of definition.
#[derive(Debug, Clone, PartialEq)]
pub enum DefinitionType {
    /// Variable definition
    Variable,
    /// Function definition
    Function,
    /// Macro definition
    Macro,
    /// Constant definition
    Constant,
    /// Type definition
    Type,
}

/// Edge in the dependency graph.
#[derive(Debug, Clone)]
pub struct DependencyEdge {
    /// Source node
    pub from: String,
    /// Target node
    pub to: String,
    /// Type of dependency
    pub dependency_type: DependencyType,
    /// Source location where dependency occurs
    pub location: Option<Span>,
}

/// Type of dependency.
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    /// Direct reference
    Reference,
    /// Function call
    Call,
    /// Macro use
    MacroUse,
    /// Type constraint
    TypeConstraint,
}

/// Variable usage analysis.
#[derive(Debug, Clone)]
pub struct VariableUsage {
    /// Variables and their usage patterns
    pub variables: HashMap<String, VariableInfo>,
    /// Unused variables
    pub unused: Vec<String>,
    /// Potentially uninitialized variables
    pub uninitialized: Vec<String>,
}

/// Information about a variable.
#[derive(Debug, Clone)]
pub struct VariableInfo {
    /// Variable name
    pub name: String,
    /// Definition location
    pub definition: Option<Span>,
    /// Usage locations
    pub uses: Vec<Span>,
    /// Whether it's read
    pub read: bool,
    /// Whether it's written
    pub written: bool,
    /// Whether it's captured in a closure
    pub captured: bool,
    /// Scope information
    pub scope: ScopeInfo,
}

/// Scope information for variables.
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    /// Scope type
    pub scope_type: ScopeType,
    /// Nesting level
    pub level: usize,
    /// Scope identifier
    pub scope_id: String,
}

/// Type of scope.
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeType {
    /// Global scope
    Global,
    /// Function definition scope
    Function,
    /// Lambda expression scope
    Lambda,
    /// Let binding scope
    Let,
    /// Letrec binding scope
    Letrec,
    /// Let* binding scope
    LetStar,
    /// Do loop scope
    Do,
    /// Macro expansion scope
    Macro,
}

/// Control flow graph for a program.
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// Basic blocks
    pub blocks: HashMap<String, BasicBlock>,
    /// Entry block
    pub entry: String,
    /// Exit blocks
    pub exits: Vec<String>,
    /// Dominator tree
    pub dominators: HashMap<String, String>,
}

/// Basic block in control flow.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Block identifier
    pub id: String,
    /// Expressions in the block
    pub expressions: Vec<Spanned<Expr>>,
    /// Successor blocks
    pub successors: Vec<String>,
    /// Predecessor blocks
    pub predecessors: Vec<String>,
}

/// Type information from inference.
#[derive(Debug, Clone)]
pub struct TypeInformation {
    /// Inferred types for expressions
    pub expression_types: HashMap<Span, InferredType>,
    /// Function signatures
    pub function_signatures: HashMap<String, FunctionSignature>,
    /// Type constraints
    pub constraints: Vec<TypeConstraint>,
    /// Type errors
    pub errors: Vec<TypeError>,
}

/// Inferred type for an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum InferredType {
    /// Primitive types
    /// Boolean type.
    Boolean,
    /// Numeric type.
    Number,
    /// String type.
    String,
    /// Character type.
    Character,
    /// Symbol type.
    Symbol,
    /// Compound types
    Pair(Box<InferredType>, Box<InferredType>),
    /// List with element type.
    List(Box<InferredType>),
    /// Vector with element type.
    Vector(Box<InferredType>),
    /// Function type
    Function {
        /// Parameter types.
        parameters: Vec<InferredType>,
        /// Return type.
        return_type: Box<InferredType>,
    },
    /// Unknown type
    Unknown,
    /// Type variable
    Variable(String),
}

/// Function signature.
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    /// Parameter types
    pub parameters: Vec<InferredType>,
    /// Return type
    pub return_type: InferredType,
    /// Whether it's variadic
    pub variadic: bool,
}

/// Type constraint.
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    /// Left side of constraint
    pub left: InferredType,
    /// Right side of constraint
    pub right: InferredType,
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Source location
    pub location: Option<Span>,
}

/// Type of constraint.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    /// Type equality constraint
    Equal,
    /// Subtype constraint
    Subtype,
    /// Supertype constraint
    Supertype,
}

/// Type error.
#[derive(Debug, Clone)]
pub struct TypeError {
    /// Error message
    pub message: String,
    /// Expected type
    pub expected: Option<InferredType>,
    /// Actual type
    pub actual: Option<InferredType>,
    /// Source location
    pub location: Option<Span>,
}

/// Code quality metrics.
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    /// Cyclomatic complexity
    pub complexity: HashMap<String, usize>,
    /// Lines of code
    pub lines_of_code: usize,
    /// Number of functions
    pub function_count: usize,
    /// Average function length
    pub avg_function_length: f64,
    /// Nesting depth
    pub max_nesting_depth: usize,
    /// Code duplication
    pub duplication: Vec<DuplicationInfo>,
}

/// Code duplication information.
#[derive(Debug, Clone)]
pub struct DuplicationInfo {
    /// Duplicated code blocks
    pub blocks: Vec<Span>,
    /// Similarity score
    pub similarity: f64,
    /// Length of duplication
    pub length: usize,
}

/// Optimization opportunity.
#[derive(Debug, Clone)]
pub struct OptimizationOpportunity {
    /// Type of optimization
    pub optimization_type: OptimizationType,
    /// Description
    pub description: String,
    /// Location where optimization can be applied
    pub location: Option<Span>,
    /// Estimated impact
    pub impact: OptimizationImpact,
    /// Suggested changes
    pub suggestion: String,
}

/// Type of optimization.
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationType {
    /// Tail call optimization
    TailCall,
    /// Constant folding
    ConstantFolding,
    /// Dead code elimination
    DeadCode,
    /// Common subexpression elimination
    CommonSubexpression,
    /// Loop optimization
    Loop,
    /// Inlining
    Inline,
    /// Strength reduction
    StrengthReduction,
}

/// Impact of optimization.
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationImpact {
    /// Low performance impact
    Low,
    /// Medium performance impact
    Medium,
    /// High performance impact
    High,
}

/// Analysis warning.
#[derive(Debug, Clone)]
pub struct AnalysisWarning {
    /// Warning type
    pub warning_type: WarningType,
    /// Warning message
    pub message: String,
    /// Location
    pub location: Option<Span>,
    /// Severity
    pub severity: WarningSeverity,
}

/// Type of warning.
#[derive(Debug, Clone, PartialEq)]
pub enum WarningType {
    /// Unused variable warning
    UnusedVariable,
    /// Unused function warning
    UnusedFunction,
    /// Potentially uninitialized variable
    PotentiallyUninitializedVariable,
    /// Variable shadowing warning
    ShadowedVariable,
    /// Dead code warning
    DeadCode,
    /// Complex function warning
    ComplexFunction,
    /// Duplicated code warning
    DuplicatedCode,
    /// Performance issue warning
    PerformanceIssue,
}

/// Warning severity.
#[derive(Debug, Clone, PartialEq)]
pub enum WarningSeverity {
    /// Informational severity
    Info,
    /// Warning severity
    Warning,
    /// Error severity
    Error,
}

/// Profiling information.
#[derive(Debug, Clone)]
pub struct ProfilingInfo {
    /// Function call counts
    pub call_counts: HashMap<String, usize>,
    /// Function execution times
    pub execution_times: HashMap<String, Duration>,
    /// Memory allocation tracking
    pub allocations: HashMap<String, AllocationInfo>,
    /// Hot spots (most time-consuming functions)
    pub hot_spots: Vec<HotSpot>,
}

/// Memory allocation information.
#[derive(Debug, Clone)]
pub struct AllocationInfo {
    /// Number of allocations
    pub count: usize,
    /// Total bytes allocated
    pub bytes: usize,
    /// Average allocation size
    pub avg_size: f64,
}

/// Hot spot in profiling.
#[derive(Debug, Clone)]
pub struct HotSpot {
    /// Function name
    pub function: String,
    /// Percentage of total execution time
    pub time_percentage: f64,
    /// Number of calls
    pub call_count: usize,
    /// Average time per call
    pub avg_time: Duration,
}

/// Static analyzer for program analysis.
#[derive(Debug)]
pub struct StaticAnalyzer {
    /// Analysis configuration
    config: AnalysisConfig,
    /// Cache for analysis results
    cache: HashMap<String, AnalysisResult>,
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
    fn analyze_dependencies(&self, program: &Program) -> Result<DependencyGraph> {
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

// Implement default and other helper methods for the structures
impl DependencyGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            cycles: Vec::new(),
        }
    }

    fn add_node(&mut self, name: String, def_type: DefinitionType, location: Option<Span>) {
        let node = DependencyNode {
            name: name.clone(),
            definition_type: def_type,
            location,
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
        };
        self.nodes.insert(name, node);
    }

    fn add_dependency(&mut self, from: String, to: String, dep_type: DependencyType, location: Option<Span>) {
        // Add edge
        self.edges.push(DependencyEdge {
            from: from.clone(),
            to: to.clone(),
            dependency_type: dep_type,
            location,
        });

        // Update node dependencies
        if let Some(from_node) = self.nodes.get_mut(&from) {
            from_node.dependencies.insert(to.clone());
        }
        if let Some(to_node) = self.nodes.get_mut(&to) {
            to_node.dependents.insert(from);
        }
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


/// Dependency analyzer for program dependencies.
#[derive(Debug)]
pub struct DependencyAnalyzer {
    /// Internal analyzer state
    analyzer: StaticAnalyzer,
}

impl DependencyAnalyzer {
    /// Creates a new dependency analyzer.
    pub fn new() -> Self {
        Self {
            analyzer: StaticAnalyzer::new(),
        }
    }

    /// Analyzes dependencies in a program.
    pub fn analyze_dependencies(&mut self, program: &Program) -> Result<DependencyGraph> {
        self.analyzer.analyze_dependencies(program)
    }
}

/// Profiler for runtime performance analysis.
#[derive(Debug)]
pub struct Profiler {
    /// Profiling data
    profiling_info: ProfilingInfo,
    /// Start time
    start_time: Instant,
}

impl Profiler {
    /// Creates a new profiler.
    pub fn new() -> Self {
        Self {
            profiling_info: ProfilingInfo {
                call_counts: HashMap::new(),
                execution_times: HashMap::new(),
                allocations: HashMap::new(),
                hot_spots: Vec::new(),
            },
            start_time: Instant::now(),
        }
    }

    /// Records a function call.
    pub fn record_call(&mut self, function_name: String, duration: Duration) {
        *self.profiling_info.call_counts.entry(function_name.clone()).or_insert(0) += 1;
        *self.profiling_info.execution_times.entry(function_name).or_insert(Duration::from_secs(0)) += duration;
    }

    /// Gets profiling results.  
    pub fn get_results(&self) -> &ProfilingInfo {
        &self.profiling_info
    }
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

impl Default for StaticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// Minimal implementations for the new types to satisfy the compiler
impl VariableUsage {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            unused: Vec::new(),
            uninitialized: Vec::new(),
        }
    }
}

impl ControlFlowGraph {
    fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            entry: String::new(),
            exits: Vec::new(),
            dominators: HashMap::new(),
        }
    }
}

impl TypeInformation {
    fn new() -> Self {
        Self {
            expression_types: HashMap::new(),
            function_signatures: HashMap::new(),
            constraints: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl QualityMetrics {
    fn new() -> Self {
        Self {
            complexity: HashMap::new(),
            lines_of_code: 0,
            function_count: 0,
            avg_function_length: 0.0,
            max_nesting_depth: 0,
            duplication: Vec::new(),
        }
    }
}