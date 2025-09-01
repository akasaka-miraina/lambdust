//! Middle-end Components for Native Compilation
//!
//! This module implements the middle-end stages including IR transformation,
//! optimization passes, and code analysis for native compilation.

use crate::ast::{Expr, Program};
use crate::compiler::CompilerConfig;
use crate::compiler::frontend::AnalyzedProgram;
use crate::compiler::ir::{CpsExpr, CpsTransform, IRProgram};
use crate::diagnostics::{Error, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Middle-end coordinator
pub struct MiddleEnd {
    /// Configuration
    config: CompilerConfig,
    /// CPS transformer
    cps_transformer: CpsTransform,
    /// Optimization manager
    optimization_manager: OptimizationManager,
    /// Code analyzer
    analyzer: CodeAnalyzer,
}

/// Optimization manager
pub struct OptimizationManager {
    /// Available optimization passes
    passes: HashMap<String, Box<dyn OptimizationPass>>,
    /// Pass scheduler
    scheduler: PassScheduler,
    /// Optimization level
    optimization_level: crate::compiler::OptimizationLevel,
}

/// Optimization pass trait
pub trait OptimizationPass {
    /// Pass name
    fn name(&self) -> &str;

    /// Pass description
    fn description(&self) -> &str;

    /// Run the optimization pass
    fn run(&mut self, program: &mut IRProgram) -> Result<OptimizationResult>;

    /// Dependencies (passes that must run before this one)
    fn dependencies(&self) -> Vec<&str>;

    /// Whether this pass modifies the IR
    fn modifies_ir(&self) -> bool;
}

/// Result of an optimization pass
#[derive(Debug)]
pub struct OptimizationResult {
    /// Whether the pass made changes
    pub changed: bool,
    /// Performance improvement estimate
    pub improvement_estimate: f64,
    /// Statistics about the optimization
    pub statistics: OptimizationStatistics,
}

/// Optimization statistics
#[derive(Debug)]
pub struct OptimizationStatistics {
    /// Instructions eliminated
    pub instructions_eliminated: usize,
    /// Functions inlined
    pub functions_inlined: usize,
    /// Constants folded
    pub constants_folded: usize,
    /// Dead code eliminated
    pub dead_code_eliminated: usize,
    /// Loops optimized
    pub loops_optimized: usize,
}

/// Pass scheduler
pub struct PassScheduler {
    /// Pass execution order
    execution_order: Vec<String>,
    /// Pass dependencies
    dependencies: HashMap<String, Vec<String>>,
    /// Pass execution times
    execution_times: HashMap<String, Duration>,
}

/// Code analyzer for optimization decisions
pub struct CodeAnalyzer {
    /// Hotness analyzer
    hotness_analyzer: HotnessAnalyzer,
    /// Loop analyzer
    loop_analyzer: LoopAnalyzer,
    /// Call analyzer
    call_analyzer: CallAnalyzer,
    /// Data flow analyzer
    dataflow_analyzer: DataFlowAnalyzer,
}

/// Hotness analysis for identifying optimization candidates
pub struct HotnessAnalyzer {
    /// Execution counts
    execution_counts: HashMap<String, u64>,
    /// Hotness threshold
    hotness_threshold: u64,
}

/// Loop analysis for loop optimizations
pub struct LoopAnalyzer {
    /// Detected loops
    loops: Vec<LoopInfo>,
    /// Loop nesting information
    nesting_info: HashMap<String, usize>,
}

/// Loop information
#[derive(Debug)]
pub struct LoopInfo {
    /// Loop identifier
    pub id: String,
    /// Loop header
    pub header: String,
    /// Loop body
    pub body: Vec<String>,
    /// Loop exits
    pub exits: Vec<String>,
    /// Loop type
    pub loop_type: LoopType,
    /// Iteration count estimate
    pub iteration_estimate: Option<u64>,
}

/// Loop types
#[derive(Debug)]
pub enum LoopType {
    /// For loop (bounded)
    For,
    /// While loop (unbounded)
    While,
    /// Tail recursive loop
    TailRecursive,
    /// Continuation-based loop
    Continuation,
}

/// Call analysis for inlining decisions
pub struct CallAnalyzer {
    /// Call sites
    call_sites: Vec<CallSite>,
    /// Inline candidates
    inline_candidates: Vec<InlineCandidate>,
}

/// Call site information
#[derive(Debug)]
pub struct CallSite {
    /// Caller function
    pub caller: String,
    /// Callee function
    pub callee: String,
    /// Call frequency
    pub frequency: u64,
    /// Call overhead estimate
    pub overhead: usize,
}

/// Inline candidate
#[derive(Debug)]
pub struct InlineCandidate {
    /// Function name
    pub function: String,
    /// Function size
    pub size: usize,
    /// Call frequency
    pub call_frequency: u64,
    /// Inline benefit estimate
    pub benefit: f64,
}

/// Data flow analysis
pub struct DataFlowAnalyzer {
    /// Reaching definitions
    reaching_definitions: HashMap<String, Vec<Definition>>,
    /// Available expressions
    available_expressions: HashMap<String, Vec<Expression>>,
    /// Live variables
    live_variables: HashMap<String, Vec<String>>,
}

/// Definition information
#[derive(Debug)]
pub struct Definition {
    /// Variable name
    pub variable: String,
    /// Definition point
    pub definition_point: String,
    /// Definition type
    pub definition_type: DefinitionType,
}

/// Definition types
#[derive(Debug)]
pub enum DefinitionType {
    /// Assignment variant
    Assignment,
    /// Parameter variant
    Parameter,
    /// Initialization variant
    Initialization,
}

/// Expression information
#[derive(Debug)]
pub struct Expression {
    /// Expression identifier
    pub id: String,
    /// Expression text
    pub text: String,
    /// Expression type
    pub expr_type: ExpressionType,
}

/// Expression types for analysis
#[derive(Debug)]
pub enum ExpressionType {
    /// Arithmetic variant
    Arithmetic,
    /// Boolean variant
    Boolean,
    /// Comparison variant
    Comparison,
    /// FunctionCall variant
    FunctionCall,
}

// Specific optimization passes

/// Constant folding optimization
pub struct ConstantFolding {
    /// Folded constants
    folded_constants: usize,
}

/// Dead code elimination
pub struct DeadCodeElimination {
    /// Eliminated instructions
    eliminated_instructions: usize,
}

/// Function inlining
pub struct FunctionInlining {
    /// Inline threshold
    inline_threshold: usize,
    /// Inlined functions
    inlined_functions: Vec<String>,
}

/// Loop optimization
pub struct LoopOptimization {
    /// Loop unrolling factor
    unroll_factor: usize,
    /// Optimized loops
    optimized_loops: Vec<String>,
}

/// Common subexpression elimination
pub struct CommonSubexpressionElimination {
    /// Eliminated expressions
    eliminated_expressions: usize,
}

/// Tail call optimization
pub struct TailCallOptimization {
    /// Optimized tail calls
    optimized_calls: usize,
}

/// Copy propagation
pub struct CopyPropagation {
    /// Propagated copies
    propagated_copies: usize,
}

/// Strength reduction
pub struct StrengthReduction {
    /// Reduced operations
    reduced_operations: usize,
}

impl MiddleEnd {
    /// Creates new middle-end with configuration
    pub fn new(config: &CompilerConfig) -> Result<Self> {
        let mut optimization_manager = OptimizationManager::new(config.optimization_level);
        optimization_manager.register_standard_passes()?;

        Ok(MiddleEnd {
            config: config.clone(),
            cps_transformer: CpsTransform::new(),
            optimization_manager,
            analyzer: CodeAnalyzer::new(),
        })
    }

    /// Transforms analyzed program to optimized IR
    pub fn transform_program(&mut self, analyzed: &AnalyzedProgram) -> Result<IRProgram> {
        let start_time = Instant::now();

        // Step 1: Transform to CPS IR
        let mut ir_program = self.transform_to_ir(&analyzed.program)?;

        // Step 2: Run optimization passes
        ir_program = self.optimization_manager.optimize(ir_program)?;

        // Step 3: Analyze optimized IR
        self.analyzer.analyze(&ir_program)?;

        println!(
            "Middle-end transformation completed in {:?}",
            start_time.elapsed()
        );
        Ok(ir_program)
    }

    /// Transforms AST to IR
    fn transform_to_ir(&mut self, program: &Program) -> Result<IRProgram> {
        let mut functions = HashMap::new();
        let mut constants = HashMap::new();

        // Transform each top-level expression
        for (i, expr) in program.expressions.iter().enumerate() {
            let function_name = format!("toplevel_{}", i);
            let cps_expr = self.cps_transformer.transform_expr(&expr.inner)?;
            functions.insert(crate::utils::intern_symbol(&function_name), cps_expr);
        }

        Ok(IRProgram {
            functions,
            entry_point: crate::utils::intern_symbol("toplevel_0"),
            constants,
            type_info: HashMap::new(),
        })
    }
}

impl OptimizationManager {
    fn new(level: crate::compiler::OptimizationLevel) -> Self {
        OptimizationManager {
            passes: HashMap::new(),
            scheduler: PassScheduler::new(),
            optimization_level: level,
        }
    }

    /// Registers standard optimization passes
    fn register_standard_passes(&mut self) -> Result<()> {
        self.register_pass(Box::new(ConstantFolding::new()));
        self.register_pass(Box::new(DeadCodeElimination::new()));
        self.register_pass(Box::new(CopyPropagation::new()));

        match self.optimization_level {
            crate::compiler::OptimizationLevel::O0 => {
                // No additional optimizations
            }
            crate::compiler::OptimizationLevel::O1 => {
                self.register_pass(Box::new(FunctionInlining::new(50))); // Small threshold
            }
            crate::compiler::OptimizationLevel::O2 => {
                self.register_pass(Box::new(FunctionInlining::new(100)));
                self.register_pass(Box::new(CommonSubexpressionElimination::new()));
                self.register_pass(Box::new(TailCallOptimization::new()));
            }
            crate::compiler::OptimizationLevel::O3 => {
                self.register_pass(Box::new(FunctionInlining::new(200)));
                self.register_pass(Box::new(CommonSubexpressionElimination::new()));
                self.register_pass(Box::new(TailCallOptimization::new()));
                self.register_pass(Box::new(LoopOptimization::new()));
                self.register_pass(Box::new(StrengthReduction::new()));
            }
        }

        self.scheduler.compute_execution_order(&self.passes)?;
        Ok(())
    }

    /// Registers an optimization pass
    fn register_pass(&mut self, pass: Box<dyn OptimizationPass>) {
        let name = pass.name().to_string();
        let deps = pass.dependencies().iter().map(|s| s.to_string()).collect();
        self.scheduler.dependencies.insert(name.clone(), deps);
        self.passes.insert(name, pass);
    }

    /// Runs optimization passes on IR program
    fn optimize(&mut self, mut program: IRProgram) -> Result<IRProgram> {
        let start_time = Instant::now();
        let mut total_changes = false;

        for pass_name in &self.scheduler.execution_order.clone() {
            if let Some(pass) = self.passes.get_mut(pass_name) {
                let pass_start = Instant::now();
                let result = pass.run(&mut program)?;
                let pass_time = pass_start.elapsed();

                self.scheduler
                    .execution_times
                    .insert(pass_name.clone(), pass_time);

                if result.changed {
                    total_changes = true;
                    println!(
                        "Pass {} made changes (improvement: {:.2}%)",
                        pass_name,
                        result.improvement_estimate * 100.0
                    );
                }
            }
        }

        if total_changes {
            println!(
                "Optimization completed in {:?} with improvements",
                start_time.elapsed()
            );
        }

        Ok(program)
    }
}

impl PassScheduler {
    fn new() -> Self {
        PassScheduler {
            execution_order: Vec::new(),
            dependencies: HashMap::new(),
            execution_times: HashMap::new(),
        }
    }

    /// Computes execution order based on dependencies
    fn compute_execution_order(
        &mut self,
        passes: &HashMap<String, Box<dyn OptimizationPass>>,
    ) -> Result<()> {
        // Topological sort of passes based on dependencies
        let mut visited = HashMap::new();
        let mut order = Vec::new();

        for pass_name in passes.keys() {
            if !visited.contains_key(pass_name) {
                self.visit_pass(pass_name, &mut visited, &mut order)?;
            }
        }

        self.execution_order = order;
        Ok(())
    }

    fn visit_pass(
        &self,
        pass_name: &str,
        visited: &mut HashMap<String, bool>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if let Some(true) = visited.get(pass_name) {
            return Ok(());
        }

        if let Some(false) = visited.get(pass_name) {
            return Err(Box::new(Error::compilation_error(format!(
                "Circular dependency in optimization passes involving {}",
                pass_name
            ))));
        }

        visited.insert(pass_name.to_string(), false);

        if let Some(deps) = self.dependencies.get(pass_name) {
            for dep in deps {
                self.visit_pass(dep, visited, order)?;
            }
        }

        visited.insert(pass_name.to_string(), true);
        order.push(pass_name.to_string());
        Ok(())
    }
}

impl CodeAnalyzer {
    fn new() -> Self {
        CodeAnalyzer {
            hotness_analyzer: HotnessAnalyzer::new(),
            loop_analyzer: LoopAnalyzer::new(),
            call_analyzer: CallAnalyzer::new(),
            dataflow_analyzer: DataFlowAnalyzer::new(),
        }
    }

    fn analyze(&mut self, _program: &IRProgram) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }
}

impl HotnessAnalyzer {
    fn new() -> Self {
        HotnessAnalyzer {
            execution_counts: HashMap::new(),
            hotness_threshold: 1000,
        }
    }
}

impl LoopAnalyzer {
    fn new() -> Self {
        LoopAnalyzer {
            loops: Vec::new(),
            nesting_info: HashMap::new(),
        }
    }
}

impl CallAnalyzer {
    fn new() -> Self {
        CallAnalyzer {
            call_sites: Vec::new(),
            inline_candidates: Vec::new(),
        }
    }
}

impl DataFlowAnalyzer {
    fn new() -> Self {
        DataFlowAnalyzer {
            reaching_definitions: HashMap::new(),
            available_expressions: HashMap::new(),
            live_variables: HashMap::new(),
        }
    }
}

// Implementation of optimization passes

impl ConstantFolding {
    fn new() -> Self {
        ConstantFolding {
            folded_constants: 0,
        }
    }
}

impl OptimizationPass for ConstantFolding {
    fn name(&self) -> &str {
        "constant_folding"
    }
    fn description(&self) -> &str {
        "Folds constant expressions at compile time"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }
    fn modifies_ir(&self) -> bool {
        true
    }

    fn run(&mut self, _program: &mut IRProgram) -> Result<OptimizationResult> {
        // Placeholder implementation
        Ok(OptimizationResult {
            changed: false,
            improvement_estimate: 0.05,
            statistics: OptimizationStatistics {
                instructions_eliminated: 0,
                functions_inlined: 0,
                constants_folded: self.folded_constants,
                dead_code_eliminated: 0,
                loops_optimized: 0,
            },
        })
    }
}

impl DeadCodeElimination {
    fn new() -> Self {
        DeadCodeElimination {
            eliminated_instructions: 0,
        }
    }
}

impl OptimizationPass for DeadCodeElimination {
    fn name(&self) -> &str {
        "dead_code_elimination"
    }
    fn description(&self) -> &str {
        "Eliminates unreachable and unused code"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }
    fn modifies_ir(&self) -> bool {
        true
    }

    fn run(&mut self, _program: &mut IRProgram) -> Result<OptimizationResult> {
        Ok(OptimizationResult {
            changed: false,
            improvement_estimate: 0.1,
            statistics: OptimizationStatistics {
                instructions_eliminated: self.eliminated_instructions,
                functions_inlined: 0,
                constants_folded: 0,
                dead_code_eliminated: self.eliminated_instructions,
                loops_optimized: 0,
            },
        })
    }
}

impl FunctionInlining {
    fn new(threshold: usize) -> Self {
        FunctionInlining {
            inline_threshold: threshold,
            inlined_functions: Vec::new(),
        }
    }
}

impl OptimizationPass for FunctionInlining {
    fn name(&self) -> &str {
        "function_inlining"
    }
    fn description(&self) -> &str {
        "Inlines small frequently called functions"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec!["dead_code_elimination"]
    }
    fn modifies_ir(&self) -> bool {
        true
    }

    fn run(&mut self, _program: &mut IRProgram) -> Result<OptimizationResult> {
        Ok(OptimizationResult {
            changed: false,
            improvement_estimate: 0.15,
            statistics: OptimizationStatistics {
                instructions_eliminated: 0,
                functions_inlined: self.inlined_functions.len(),
                constants_folded: 0,
                dead_code_eliminated: 0,
                loops_optimized: 0,
            },
        })
    }
}

impl LoopOptimization {
    fn new() -> Self {
        LoopOptimization {
            unroll_factor: 4,
            optimized_loops: Vec::new(),
        }
    }
}

impl OptimizationPass for LoopOptimization {
    fn name(&self) -> &str {
        "loop_optimization"
    }
    fn description(&self) -> &str {
        "Optimizes loops through unrolling and other techniques"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec!["constant_folding"]
    }
    fn modifies_ir(&self) -> bool {
        true
    }

    fn run(&mut self, _program: &mut IRProgram) -> Result<OptimizationResult> {
        Ok(OptimizationResult {
            changed: false,
            improvement_estimate: 0.2,
            statistics: OptimizationStatistics {
                instructions_eliminated: 0,
                functions_inlined: 0,
                constants_folded: 0,
                dead_code_eliminated: 0,
                loops_optimized: self.optimized_loops.len(),
            },
        })
    }
}

impl CommonSubexpressionElimination {
    fn new() -> Self {
        CommonSubexpressionElimination {
            eliminated_expressions: 0,
        }
    }
}

impl OptimizationPass for CommonSubexpressionElimination {
    fn name(&self) -> &str {
        "common_subexpression_elimination"
    }
    fn description(&self) -> &str {
        "Eliminates redundant computations"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec!["copy_propagation"]
    }
    fn modifies_ir(&self) -> bool {
        true
    }

    fn run(&mut self, _program: &mut IRProgram) -> Result<OptimizationResult> {
        Ok(OptimizationResult {
            changed: false,
            improvement_estimate: 0.08,
            statistics: OptimizationStatistics {
                instructions_eliminated: self.eliminated_expressions,
                functions_inlined: 0,
                constants_folded: 0,
                dead_code_eliminated: 0,
                loops_optimized: 0,
            },
        })
    }
}

impl TailCallOptimization {
    fn new() -> Self {
        TailCallOptimization { optimized_calls: 0 }
    }
}

impl OptimizationPass for TailCallOptimization {
    fn name(&self) -> &str {
        "tail_call_optimization"
    }
    fn description(&self) -> &str {
        "Optimizes tail calls to loops"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }
    fn modifies_ir(&self) -> bool {
        true
    }

    fn run(&mut self, _program: &mut IRProgram) -> Result<OptimizationResult> {
        Ok(OptimizationResult {
            changed: false,
            improvement_estimate: 0.12,
            statistics: OptimizationStatistics {
                instructions_eliminated: self.optimized_calls,
                functions_inlined: 0,
                constants_folded: 0,
                dead_code_eliminated: 0,
                loops_optimized: 0,
            },
        })
    }
}

impl CopyPropagation {
    fn new() -> Self {
        CopyPropagation {
            propagated_copies: 0,
        }
    }
}

impl OptimizationPass for CopyPropagation {
    fn name(&self) -> &str {
        "copy_propagation"
    }
    fn description(&self) -> &str {
        "Propagates variable copies to eliminate redundant assignments"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }
    fn modifies_ir(&self) -> bool {
        true
    }

    fn run(&mut self, _program: &mut IRProgram) -> Result<OptimizationResult> {
        Ok(OptimizationResult {
            changed: false,
            improvement_estimate: 0.06,
            statistics: OptimizationStatistics {
                instructions_eliminated: self.propagated_copies,
                functions_inlined: 0,
                constants_folded: 0,
                dead_code_eliminated: 0,
                loops_optimized: 0,
            },
        })
    }
}

impl StrengthReduction {
    fn new() -> Self {
        StrengthReduction {
            reduced_operations: 0,
        }
    }
}

impl OptimizationPass for StrengthReduction {
    fn name(&self) -> &str {
        "strength_reduction"
    }
    fn description(&self) -> &str {
        "Replaces expensive operations with cheaper equivalents"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec!["loop_optimization"]
    }
    fn modifies_ir(&self) -> bool {
        true
    }

    fn run(&mut self, _program: &mut IRProgram) -> Result<OptimizationResult> {
        Ok(OptimizationResult {
            changed: false,
            improvement_estimate: 0.1,
            statistics: OptimizationStatistics {
                instructions_eliminated: self.reduced_operations,
                functions_inlined: 0,
                constants_folded: 0,
                dead_code_eliminated: 0,
                loops_optimized: 0,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::CompilerConfig;

    #[test]
    fn test_middle_end_creation() {
        let config = CompilerConfig::default();
        let middle_end = MiddleEnd::new(&config);
        assert!(middle_end.is_ok());
    }

    #[test]
    fn test_optimization_manager() {
        let mut manager = OptimizationManager::new(crate::compiler::OptimizationLevel::O2);
        let result = manager.register_standard_passes();
        assert!(result.is_ok());
        assert!(!manager.passes.is_empty());
    }

    #[test]
    fn test_pass_scheduler() {
        let mut scheduler = PassScheduler::new();
        scheduler
            .dependencies
            .insert("pass1".to_string(), vec!["pass2".to_string()]);
        scheduler.dependencies.insert("pass2".to_string(), vec![]);

        let passes = HashMap::new();
        let result = scheduler.compute_execution_order(&passes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimization_passes() {
        let mut constant_folding = ConstantFolding::new();
        assert_eq!(constant_folding.name(), "constant_folding");
        assert!(!constant_folding.dependencies().contains(&"nonexistent"));

        let mut dead_code = DeadCodeElimination::new();
        assert_eq!(dead_code.name(), "dead_code_elimination");
        assert!(dead_code.modifies_ir());
    }
}
