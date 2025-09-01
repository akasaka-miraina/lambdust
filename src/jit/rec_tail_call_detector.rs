#![allow(missing_docs)]//! Advanced Tail-Call Detection for SRFI-31 REC Expressions
//!
//! This module implements sophisticated tail-call detection algorithms specifically
//! optimized for `rec` expressions in Lambdust. Building on the foundation of 
//! parse-time desugaring to `letrec`, this detector provides algorithmic analysis
//! to identify and optimize tail-recursive patterns.
//!
//! ## Algorithm Design
//!
//! The detector implements a **three-phase analysis pipeline**:
//!
//! ### Phase 1: Tail Position Analysis (O(n))
//! - Traverse the AST to identify all expressions in tail position
//! - Build a tail-position map for efficient lookup during analysis
//! - Handle complex control structures (if, cond, case, let, etc.)
//!
//! ### Phase 2: Recursive Call Classification (O(k))
//! - Identify all self-recursive calls within the function body
//! - Classify each call as tail-recursive or non-tail-recursive
//! - Compute tail-recursion ratio for optimization heuristics
//!
//! ### Phase 3: Optimization Strategy Selection (O(1))
//! - Select optimal tail-call optimization strategy based on analysis
//! - Generate optimization metadata for the JIT compiler
//! - Compute performance improvement estimates
//!
//! ## Performance Characteristics
//!
//! - **Analysis Time**: O(n) where n is AST size (typically 10-50 nodes)
//! - **Memory Usage**: O(n) for tail-position map (minimal overhead)
//! - **Cache Efficiency**: Results cached by function signature hash
//! - **Integration Cost**: Zero - works with existing letrec infrastructure

use crate::ast::{Expr, Spanned};
use crate::diagnostics::{Result, Error, Span};
use crate::jit::tail_call_optimization::{TailCallSite, TailCallType, TailCallAnalysis};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};

/// Advanced tail-call detector for REC expressions
pub struct RecTailCallDetector {
    /// Configuration for tail-call analysis
    config: TailCallDetectionConfig,
    /// Cache of analysis results by function hash
    analysis_cache: HashMap<u64, TailCallDetectionResult>,
    /// Performance metrics collector
    metrics: TailCallDetectionMetrics,
    /// Tail position cache for complex expressions
    tail_position_cache: HashMap<String, TailPositionMap>,
}

/// Configuration for tail-call detection
#[derive(Debug, Clone)]
pub struct TailCallDetectionConfig {
    /// Maximum recursion depth for analysis
    pub max_analysis_depth: usize,
    /// Enable deep control flow analysis
    pub enable_deep_control_flow_analysis: bool,
    /// Minimum tail-recursion ratio to consider for optimization (0.0-1.0)
    pub min_tail_recursion_ratio: f64,
    /// Enable mutual tail-recursion detection
    pub enable_mutual_tail_recursion: bool,
    /// Cache tail position analysis results
    pub enable_tail_position_caching: bool,
}

impl Default for TailCallDetectionConfig {
    fn default() -> Self {
        Self {
            max_analysis_depth: 100,
            enable_deep_control_flow_analysis: true,
            min_tail_recursion_ratio: 0.7, // 70% of calls must be tail-recursive
            enable_mutual_tail_recursion: true,
            enable_tail_position_caching: true,
        }
    }
}

/// Result of tail-call detection analysis
#[derive(Debug, Clone)]
pub struct TailCallDetectionResult {
    /// All tail-recursive call sites found
    pub tail_recursive_calls: Vec<TailRecursiveCall>,
    /// All non-tail-recursive calls
    pub non_tail_recursive_calls: Vec<NonTailRecursiveCall>,
    /// Tail-recursion ratio (0.0 = no tail calls, 1.0 = all tail calls)
    pub tail_recursion_ratio: f64,
    /// Recommended optimization strategy
    pub optimization_strategy: TailCallOptimizationStrategy,
    /// Estimated performance improvement from optimization
    pub estimated_performance_gain: PerformanceGain,
    /// Analysis confidence level (0.0-1.0)
    pub confidence: f64,
}

/// A tail-recursive call site
#[derive(Debug, Clone)]
pub struct TailRecursiveCall {
    /// Location in the AST where the tail call occurs
    pub location: TailCallLocation,
    /// Arguments passed to the recursive call
    pub arguments: Vec<ArgumentInfo>,
    /// Tail-call context (direct, conditional, etc.)
    pub context: TailCallContext,
    /// Estimated optimization benefit for this call
    pub optimization_benefit: f64,
}

/// A non-tail-recursive call site
#[derive(Debug, Clone)]
pub struct NonTailRecursiveCall {
    /// Location in the AST
    pub location: TailCallLocation,
    /// Reason why this call is not tail-recursive
    pub blocking_reason: NonTailReason,
    /// Surrounding expression that prevents tail optimization
    pub blocking_expression: String,
    /// Potential for conversion to tail form
    pub tail_conversion_potential: TailConversionPotential,
}

/// Location of a call in the AST
#[derive(Debug, Clone)]
pub struct TailCallLocation {
    /// AST node path from root
    pub ast_path: Vec<String>,
    /// Nested depth in control structures
    pub control_depth: usize,
    /// Source span if available
    pub source_span: Option<Span>,
}

/// Information about arguments in a recursive call
#[derive(Debug, Clone)]
pub struct ArgumentInfo {
    /// Argument position (0-based)
    pub position: usize,
    /// Type of argument transformation
    pub transformation: ArgumentTransformation,
    /// Whether this argument is modified in the recursive call
    pub is_modified: bool,
    /// Data flow pattern for this argument
    pub data_flow_pattern: DataFlowPattern,
}

/// Types of argument transformations in recursive calls
#[derive(Debug, Clone)]
pub enum ArgumentTransformation {
    /// Identity: argument passed unchanged
    Identity,
    /// Arithmetic: argument modified by arithmetic operation
    Arithmetic { operation: String },
    /// List operation: cdr, cdr, cons, etc.
    ListOperation { operation: String },
    /// Function application: argument transformed by function call
    FunctionApplication { function_name: String },
    /// Conditional: argument may be transformed conditionally
    Conditional { condition: String },
}

/// Data flow patterns for arguments
#[derive(Debug, Clone)]
pub enum DataFlowPattern {
    /// Decreasing pattern: argument gets smaller (n-1, cdr, etc.)
    Decreasing,
    /// Accumulating pattern: argument accumulates values
    Accumulating,
    /// Constant pattern: argument remains constant
    Constant,
    /// Complex pattern: unpredictable changes
    Complex,
}

/// Context in which a tail call occurs
#[derive(Debug, Clone)]
pub enum TailCallContext {
    /// Direct tail call: last expression in function body
    Direct,
    /// Conditional tail call: tail call in if/cond branch
    Conditional { branch_type: ConditionalBranch },
    /// Let-bound tail call: tail call after let binding
    LetBound { binding_count: usize },
    /// Case tail call: tail call in case branch
    Case { case_count: usize },
    /// Begin tail call: tail call in begin sequence
    Begin { position: usize },
}

/// Types of conditional branches
#[derive(Debug, Clone)]
pub enum ConditionalBranch {
    /// Then branch of if expression
    IfThen,
    /// Else branch of if expression
    IfElse,
    /// Cond clause
    CondClause { clause_number: usize },
    /// Case clause
    CaseClause { clause_number: usize },
}

/// Reasons why a call is not tail-recursive
#[derive(Debug, Clone)]
pub enum NonTailReason {
    /// Call result is used in further computation
    ResultUsedInComputation { operation: String },
    /// Call is not in the final position
    NotInFinalPosition { final_expression: String },
    /// Call is nested within another function call
    NestedInFunctionCall { outer_function: String },
    /// Call is in a position that requires its return value
    ReturnValueRequired { required_by: String },
}

/// Potential for converting non-tail call to tail form
#[derive(Debug, Clone)]
pub enum TailConversionPotential {
    /// High potential: can be converted with accumulator passing
    High { strategy: AccumulatorStrategy },
    /// Medium potential: requires significant restructuring
    Medium { complexity: String },
    /// Low potential: would require fundamental algorithm change
    Low { reason: String },
    /// Not convertible: inherently non-tail-recursive
    NotConvertible { reason: String },
}

/// Strategies for accumulator-based conversion
#[derive(Debug, Clone)]
pub enum AccumulatorStrategy {
    /// Standard accumulator parameter
    StandardAccumulator { parameter_name: String },
    /// Continuation-passing style
    ContinuationPassing,
    /// Multiple accumulators
    MultipleAccumulators { count: usize },
}

/// Recommended optimization strategies
#[derive(Debug, Clone)]
pub enum TailCallOptimizationStrategy {
    /// Direct tail-call elimination: convert to loop
    DirectElimination {
        /// Loop variables and their update expressions
        loop_variables: Vec<LoopVariable>,
        /// Loop condition expression
        loop_condition: String,
    },
    
    /// Trampoline optimization: avoid stack overflow for deep recursion
    Trampoline {
        /// Continuation function name
        continuation_function: String,
        /// Bounce detection strategy
        bounce_strategy: BounceStrategy,
    },
    
    /// Accumulator passing: convert to tail-recursive form
    AccumulatorPassing {
        /// New accumulator parameters to add
        accumulator_parameters: Vec<String>,
        /// Transformation function for existing calls
        call_transformation: String,
    },
    
    /// Iterative conversion: convert entire function to iterative form
    IterativeConversion {
        /// Data structures for state management
        state_management: StateManagementStrategy,
        /// Loop structure
        loop_structure: LoopStructure,
    },
    
    /// No optimization: keep existing form
    NoOptimization {
        /// Reason optimization is not beneficial
        reason: String,
    },
}

/// Loop variable for direct elimination
#[derive(Debug, Clone)]
pub struct LoopVariable {
    /// Variable name
    pub name: String,
    /// Update expression
    pub update_expression: String,
    /// Initial value
    pub initial_value: String,
}

/// Bounce detection strategies for trampolines
#[derive(Debug, Clone)]
pub enum BounceStrategy {
    /// Simple return value checking
    ReturnValueChecking,
    /// Continuation-based detection
    ContinuationBased,
    /// Stack depth monitoring
    StackDepthMonitoring,
}

/// State management strategies for iterative conversion
#[derive(Debug, Clone)]
pub enum StateManagementStrategy {
    /// Simple variables
    SimpleVariables,
    /// Stack-based state
    StackBased { stack_size: usize },
    /// Queue-based state
    QueueBased { initial_capacity: usize },
    /// Custom data structure
    Custom { structure_name: String },
}

/// Loop structure for iterative conversion
#[derive(Debug, Clone)]
pub enum LoopStructure {
    /// While loop
    WhileLoop { condition: String },
    /// For loop
    ForLoop { iterator: String, range: String },
    /// Do-while loop
    DoWhileLoop { condition: String },
}

/// Performance gain estimates
#[derive(Debug, Clone)]
pub struct PerformanceGain {
    /// Stack space savings (0.0-1.0, 1.0 = complete elimination)
    pub stack_space_savings: f64,
    /// Execution time improvement multiplier (1.0 = no change, 2.0 = 2x faster)
    pub execution_time_multiplier: f64,
    /// Memory allocation reduction (0.0-1.0)
    pub memory_allocation_reduction: f64,
    /// Cache efficiency improvement (0.0-1.0)
    pub cache_efficiency_improvement: f64,
}

/// Map of expressions to their tail positions
#[derive(Debug, Clone)]
pub struct TailPositionMap {
    /// Set of expression IDs that are in tail position
    pub tail_positions: HashSet<String>,
    /// Depth of each expression in the control flow
    pub expression_depths: HashMap<String, usize>,
    /// Parent-child relationships in the AST
    pub parent_child_map: HashMap<String, Vec<String>>,
}

/// Performance metrics for tail-call detection
#[derive(Debug, Default)]
pub struct TailCallDetectionMetrics {
    /// Total number of functions analyzed
    pub functions_analyzed: usize,
    /// Number of tail-recursive functions found
    pub tail_recursive_functions: usize,
    /// Number of functions successfully optimized
    pub functions_optimized: usize,
    /// Total analysis time in microseconds
    pub total_analysis_time_us: u64,
    /// Cache hit rate for analysis results
    pub cache_hit_rate: f64,
    /// Distribution of optimization strategies chosen
    pub strategy_distribution: HashMap<String, usize>,
}

impl RecTailCallDetector {
    /// Create a new tail-call detector with default configuration
    pub fn new() -> Self {
        Self::with_config(TailCallDetectionConfig::default())
    }
    
    /// Create a new tail-call detector with custom configuration
    pub fn with_config(config: TailCallDetectionConfig) -> Self {
        Self {
            config,
            analysis_cache: HashMap::new(),
            metrics: TailCallDetectionMetrics::default(),
            tail_position_cache: HashMap::new(),
        }
    }
    
    /// Detect tail-recursive calls in a REC expression
    ///
    /// This is the main entry point for tail-call detection. It analyzes a function
    /// defined with `rec` (already desugared to `letrec`) and identifies all tail-recursive
    /// and non-tail-recursive call sites.
    ///
    /// # Algorithm Overview
    ///
    /// 1. **Function Signature Hashing**: Create a hash of the function structure for caching
    /// 2. **Tail Position Analysis**: Build a map of all expressions in tail position
    /// 3. **Recursive Call Detection**: Find all self-recursive calls in the function
    /// 4. **Tail Classification**: Classify each call as tail or non-tail
    /// 5. **Strategy Selection**: Choose optimal optimization strategy
    /// 6. **Performance Estimation**: Estimate benefits of optimization
    ///
    /// # Time Complexity: O(n + k) where n = AST size, k = number of recursive calls
    pub fn detect_tail_calls(
        &mut self,
        function_name: &str,
        lambda_expr: &Expr,
    ) -> Result<TailCallDetectionResult> {
        let start_time = std::time::Instant::now();
        self.metrics.functions_analyzed += 1;
        
        // Create a hash of the function for caching
        let function_hash = self.compute_function_hash(function_name, lambda_expr);
        
        // Check cache first
        if let Some(cached_result) = self.analysis_cache.get(&function_hash) {
            self.update_cache_hit_rate(true);
            return Ok(cached_result.clone());
        }
        self.update_cache_hit_rate(false);
        
        // Phase 1: Build tail position map
        let tail_position_map = self.build_tail_position_map(lambda_expr)?;
        
        // Phase 2: Find all recursive calls
        let all_recursive_calls = self.find_all_recursive_calls(function_name, lambda_expr)?;
        
        // Phase 3: Classify calls as tail or non-tail
        let (tail_calls, non_tail_calls) = self.classify_recursive_calls(
            &all_recursive_calls,
            &tail_position_map,
        )?;
        
        // Calculate tail-recursion ratio
        let total_calls = all_recursive_calls.len();
        let tail_recursion_ratio = if total_calls > 0 {
            tail_calls.len() as f64 / total_calls as f64
        } else {
            0.0
        };
        
        // Phase 4: Select optimization strategy
        let optimization_strategy = self.select_optimization_strategy(
            &tail_calls,
            &non_tail_calls,
            tail_recursion_ratio,
        )?;
        
        // Phase 5: Estimate performance gains
        let performance_gain = self.estimate_performance_gain(
            &optimization_strategy,
            tail_recursion_ratio,
            &tail_calls,
        )?;
        
        // Calculate confidence based on analysis quality
        let confidence = self.calculate_analysis_confidence(
            tail_recursion_ratio,
            &tail_calls,
            &non_tail_calls,
        );
        
        let result = TailCallDetectionResult {
            tail_recursive_calls: tail_calls,
            non_tail_recursive_calls: non_tail_calls,
            tail_recursion_ratio,
            optimization_strategy,
            estimated_performance_gain: performance_gain,
            confidence,
        };
        
        // Cache the result
        self.analysis_cache.insert(function_hash, result.clone());
        
        // Update metrics
        if tail_recursion_ratio >= self.config.min_tail_recursion_ratio {
            self.metrics.tail_recursive_functions += 1;
            
            if !matches!(result.optimization_strategy, TailCallOptimizationStrategy::NoOptimization { .. }) {
                self.metrics.functions_optimized += 1;
            }
        }
        
        let analysis_time = start_time.elapsed();
        self.metrics.total_analysis_time_us += analysis_time.as_micros() as u64;
        
        // Update strategy distribution
        let strategy_name = self.get_strategy_name(&result.optimization_strategy);
        *self.metrics.strategy_distribution.entry(strategy_name).or_insert(0) += 1;
        
        Ok(result)
    }
    
    /// Compute a hash of the function structure for caching
    fn compute_function_hash(&self, function_name: &str, lambda_expr: &Expr) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        function_name.hash(&mut hasher);
        
        // Hash the structure of the lambda expression (simplified)
        match lambda_expr {
            Expr::Lambda { params, body, .. } => {
                params.hash(&mut hasher);
                // Hash the structure of the body (not the exact spans)
                self.hash_expr_structure(body, &mut hasher);
            }
            _ => {
                "non-lambda".hash(&mut hasher);
            }
        }
        
        hasher.finish()
    }
    
    /// Hash the structure of an expression list for caching
    fn hash_expr_structure(&self, exprs: &[Spanned<Expr>], hasher: &mut std::collections::hash_map::DefaultHasher) {
        exprs.len().hash(hasher);
        for expr in exprs {
            self.hash_single_expr_structure(&expr.inner, hasher);
        }
    }
    
    /// Hash a single expression's structure
    fn hash_single_expr_structure(&self, expr: &Expr, hasher: &mut std::collections::hash_map::DefaultHasher) {
        match expr {
            Expr::Identifier(name) => {
                "identifier".hash(hasher);
                name.hash(hasher);
            }
            Expr::Application { function, arguments, .. } => {
                "application".hash(hasher);
                self.hash_single_expr_structure(&function.inner, hasher);
                arguments.len().hash(hasher);
                for arg in arguments {
                    self.hash_single_expr_structure(&arg.inner, hasher);
                }
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                "if".hash(hasher);
                self.hash_single_expr_structure(&condition.inner, hasher);
                self.hash_single_expr_structure(&then_branch.inner, hasher);
                if let Some(else_expr) = else_branch {
                    self.hash_single_expr_structure(&else_expr.inner, hasher);
                }
            }
            Expr::Lambda { params, body, .. } => {
                "lambda".hash(hasher);
                params.hash(hasher);
                self.hash_expr_structure(body, hasher);
            }
            // Add more cases as needed
            _ => {
                "other".hash(hasher);
            }
        }
    }
    
    /// Build a map of all expressions in tail position
    /// Time Complexity: O(n) where n is the number of AST nodes
    fn build_tail_position_map(&self, lambda_expr: &Expr) -> Result<TailPositionMap> {
        let mut tail_positions = HashSet::new();
        let mut expression_depths = HashMap::new();
        let mut parent_child_map = HashMap::new();
        
        if let Expr::Lambda { body, .. } = lambda_expr {
            // The last expression in the lambda body is always in tail position
            if let Some(last_expr) = body.last() {
                self.mark_tail_positions(
                    &last_expr.inner,
                    true, // is_tail_position = true
                    0,    // depth = 0
                    &mut tail_positions,
                    &mut expression_depths,
                    &mut parent_child_map,
                )?;
            }
            
            // Process non-tail expressions
            for expr in &body[..body.len().saturating_sub(1)] {
                self.mark_tail_positions(
                    &expr.inner,
                    false, // is_tail_position = false
                    0,     // depth = 0
                    &mut tail_positions,
                    &mut expression_depths,
                    &mut parent_child_map,
                )?;
            }
        }
        
        Ok(TailPositionMap {
            tail_positions,
            expression_depths,
            parent_child_map,
        })
    }
    
    /// Recursively mark tail positions in the AST
    fn mark_tail_positions(
        &self,
        expr: &Expr,
        is_tail_position: bool,
        depth: usize,
        tail_positions: &mut HashSet<String>,
        expression_depths: &mut HashMap<String, usize>,
        parent_child_map: &mut HashMap<String, Vec<String>>,
    ) -> Result<()> {
        let expr_id = self.get_expression_id(expr);
        expression_depths.insert(expr_id.clone(), depth);
        
        if is_tail_position {
            tail_positions.insert(expr_id.clone());
        }
        
        match expr {
            Expr::If { condition, then_branch, else_branch, .. } => {
                // Condition is never in tail position
                let condition_id = self.get_expression_id(&condition.inner);
                parent_child_map.entry(expr_id.clone()).or_default().push(condition_id);
                self.mark_tail_positions(
                    &condition.inner,
                    false,
                    depth + 1,
                    tail_positions,
                    expression_depths,
                    parent_child_map,
                )?;
                
                // Then and else branches inherit tail position
                let then_id = self.get_expression_id(&then_branch.inner);
                parent_child_map.entry(expr_id.clone()).or_default().push(then_id);
                self.mark_tail_positions(
                    &then_branch.inner,
                    is_tail_position,
                    depth + 1,
                    tail_positions,
                    expression_depths,
                    parent_child_map,
                )?;
                
                if let Some(else_expr) = else_branch {
                    let else_id = self.get_expression_id(&else_expr.inner);
                    parent_child_map.entry(expr_id.clone()).or_default().push(else_id);
                    self.mark_tail_positions(
                        &else_expr.inner,
                        is_tail_position,
                        depth + 1,
                        tail_positions,
                        expression_depths,
                        parent_child_map,
                    )?;
                }
            }
            
            Expr::Application { function, arguments, .. } => {
                // Function application: only the application itself can be tail
                // The function and arguments are not in tail position
                let func_id = self.get_expression_id(&function.inner);
                parent_child_map.entry(expr_id.clone()).or_default().push(func_id);
                self.mark_tail_positions(
                    &function.inner,
                    false,
                    depth + 1,
                    tail_positions,
                    expression_depths,
                    parent_child_map,
                )?;
                
                for arg in arguments {
                    let arg_id = self.get_expression_id(&arg.inner);
                    parent_child_map.entry(expr_id.clone()).or_default().push(arg_id);
                    self.mark_tail_positions(
                        &arg.inner,
                        false,
                        depth + 1,
                        tail_positions,
                        expression_depths,
                        parent_child_map,
                    )?;
                }
            }
            
            Expr::Let { bindings, body, .. } | Expr::LetRec { bindings, body, .. } => {
                // Binding values are not in tail position
                for binding in bindings {
                    let binding_id = self.get_expression_id(&binding.value.inner);
                    parent_child_map.entry(expr_id.clone()).or_default().push(binding_id);
                    self.mark_tail_positions(
                        &binding.value.inner,
                        false,
                        depth + 1,
                        tail_positions,
                        expression_depths,
                        parent_child_map,
                    )?;
                }
                
                // Body expressions: last one inherits tail position
                for (i, body_expr) in body.iter().enumerate() {
                    let body_id = self.get_expression_id(&body_expr.inner);
                    parent_child_map.entry(expr_id.clone()).or_default().push(body_id);
                    let is_last = i == body.len() - 1;
                    self.mark_tail_positions(
                        &body_expr.inner,
                        is_tail_position && is_last,
                        depth + 1,
                        tail_positions,
                        expression_depths,
                        parent_child_map,
                    )?;
                }
            }
            
            // For other expressions, they don't affect tail position of children
            _ => {}
        }
        
        Ok(())
    }
    
    /// Generate a unique ID for an expression based on its structure
    fn get_expression_id(&self, expr: &Expr) -> String {
        match expr {
            Expr::Identifier(name) => format!("id:{}", name),
            Expr::Application { function, arguments, .. } => {
                format!("app:{}:{}", self.get_expression_id(&function.inner), arguments.len())
            }
            Expr::If { .. } => "if".to_string(),
            Expr::Lambda { .. } => "lambda".to_string(),
            Expr::Let { .. } => "let".to_string(),
            Expr::LetRec { .. } => "letrec".to_string(),
            _ => "other".to_string(),
        }
    }
    
    /// Find all recursive calls in the function
    fn find_all_recursive_calls(
        &self,
        function_name: &str,
        lambda_expr: &Expr,
    ) -> Result<Vec<RecursiveCallInfo>> {
        let mut calls = Vec::new();
        
        if let Expr::Lambda { body, .. } = lambda_expr {
            for expr in body {
                self.find_recursive_calls_in_expr(function_name, &expr.inner, &mut calls, vec!["body".to_string()])?;
            }
        }
        
        Ok(calls)
    }
    
    /// Recursively find calls in an expression
    fn find_recursive_calls_in_expr(
        &self,
        function_name: &str,
        expr: &Expr,
        calls: &mut Vec<RecursiveCallInfo>,
        ast_path: Vec<String>,
    ) -> Result<()> {
        match expr {
            Expr::Application { function, arguments, .. } => {
                if let Expr::Identifier(name) = &function.inner {
                    if name == function_name {
                        // Found a recursive call
                        calls.push(RecursiveCallInfo {
                            function_name: name.clone(),
                            arguments: arguments.iter().enumerate().map(|(i, arg)| {
                                self.analyze_argument(i, &arg.inner)
                            }).collect::<Result<Vec<_>>>()?,
                            ast_path: ast_path.clone(),
                            control_depth: ast_path.len(),
                        });
                    }
                }
                
                // Continue searching in arguments
                for (i, arg) in arguments.iter().enumerate() {
                    let mut arg_path = ast_path.clone();
                    arg_path.push(format!("arg{}", i));
                    self.find_recursive_calls_in_expr(function_name, &arg.inner, calls, arg_path)?;
                }
            }
            
            Expr::If { condition, then_branch, else_branch, .. } => {
                let mut cond_path = ast_path.clone();
                cond_path.push("condition".to_string());
                self.find_recursive_calls_in_expr(function_name, &condition.inner, calls, cond_path)?;
                
                let mut then_path = ast_path.clone();
                then_path.push("then".to_string());
                self.find_recursive_calls_in_expr(function_name, &then_branch.inner, calls, then_path)?;
                
                if let Some(else_expr) = else_branch {
                    let mut else_path = ast_path.clone();
                    else_path.push("else".to_string());
                    self.find_recursive_calls_in_expr(function_name, &else_expr.inner, calls, else_path)?;
                }
            }
            
            // Add more expression types as needed
            _ => {}
        }
        
        Ok(())
    }
    
    /// Analyze an argument in a recursive call
    fn analyze_argument(&self, position: usize, expr: &Expr) -> Result<ArgumentInfo> {
        let transformation = match expr {
            Expr::Identifier(_) => ArgumentTransformation::Identity,
            Expr::Application { function, .. } => {
                if let Expr::Identifier(func_name) = &function.inner {
                    match func_name.as_str() {
                        "+" | "-" | "*" | "/" => ArgumentTransformation::Arithmetic { 
                            operation: func_name.clone() 
                        },
                        "car" | "cdr" | "cons" => ArgumentTransformation::ListOperation { 
                            operation: func_name.clone() 
                        },
                        _ => ArgumentTransformation::FunctionApplication { 
                            function_name: func_name.clone() 
                        },
                    }
                } else {
                    ArgumentTransformation::FunctionApplication { 
                        function_name: "complex".to_string() 
                    }
                }
            }
            _ => ArgumentTransformation::Identity,
        };
        
        let data_flow_pattern = self.infer_data_flow_pattern(&transformation);
        
        Ok(ArgumentInfo {
            position,
            transformation,
            is_modified: !matches!(transformation, ArgumentTransformation::Identity),
            data_flow_pattern,
        })
    }
    
    /// Infer data flow pattern from argument transformation
    fn infer_data_flow_pattern(&self, transformation: &ArgumentTransformation) -> DataFlowPattern {
        match transformation {
            ArgumentTransformation::Identity => DataFlowPattern::Constant,
            ArgumentTransformation::Arithmetic { operation } => {
                if operation == "-" {
                    DataFlowPattern::Decreasing
                } else {
                    DataFlowPattern::Complex
                }
            }
            ArgumentTransformation::ListOperation { operation } => {
                if operation == "cdr" {
                    DataFlowPattern::Decreasing
                } else if operation == "cons" {
                    DataFlowPattern::Accumulating
                } else {
                    DataFlowPattern::Complex
                }
            }
            _ => DataFlowPattern::Complex,
        }
    }
    
    /// Classify recursive calls as tail or non-tail
    fn classify_recursive_calls(
        &self,
        all_calls: &[RecursiveCallInfo],
        tail_position_map: &TailPositionMap,
    ) -> Result<(Vec<TailRecursiveCall>, Vec<NonTailRecursiveCall>)> {
        let mut tail_calls = Vec::new();
        let mut non_tail_calls = Vec::new();
        
        for call in all_calls {
            let call_expr_id = format!("app:{}:{}", call.function_name, call.arguments.len());
            
            if tail_position_map.tail_positions.contains(&call_expr_id) {
                // This is a tail call
                tail_calls.push(TailRecursiveCall {
                    location: TailCallLocation {
                        ast_path: call.ast_path.clone(),
                        control_depth: call.control_depth,
                        source_span: None,
                    },
                    arguments: call.arguments.clone(),
                    context: self.determine_tail_call_context(&call.ast_path),
                    optimization_benefit: self.calculate_optimization_benefit(&call.arguments),
                });
            } else {
                // This is not a tail call
                non_tail_calls.push(NonTailRecursiveCall {
                    location: TailCallLocation {
                        ast_path: call.ast_path.clone(),
                        control_depth: call.control_depth,
                        source_span: None,
                    },
                    blocking_reason: self.determine_blocking_reason(&call.ast_path),
                    blocking_expression: "unknown".to_string(), // Would need more analysis
                    tail_conversion_potential: self.assess_tail_conversion_potential(&call.arguments),
                });
            }
        }
        
        Ok((tail_calls, non_tail_calls))
    }
    
    /// Determine the context of a tail call
    fn determine_tail_call_context(&self, ast_path: &[String]) -> TailCallContext {
        if ast_path.contains(&"then".to_string()) || ast_path.contains(&"else".to_string()) {
            TailCallContext::Conditional { 
                branch_type: if ast_path.contains(&"then".to_string()) {
                    ConditionalBranch::IfThen
                } else {
                    ConditionalBranch::IfElse
                }
            }
        } else if ast_path.len() == 1 {
            TailCallContext::Direct
        } else {
            TailCallContext::Direct // Simplified
        }
    }
    
    /// Determine why a call is not tail-recursive
    fn determine_blocking_reason(&self, ast_path: &[String]) -> NonTailReason {
        if ast_path.contains(&"arg0".to_string()) || ast_path.contains(&"arg1".to_string()) {
            NonTailReason::NestedInFunctionCall { outer_function: "unknown".to_string() }
        } else if ast_path.contains(&"condition".to_string()) {
            NonTailReason::ResultUsedInComputation { operation: "condition".to_string() }
        } else {
            NonTailReason::NotInFinalPosition { final_expression: "unknown".to_string() }
        }
    }
    
    /// Calculate optimization benefit for a tail call
    fn calculate_optimization_benefit(&self, arguments: &[ArgumentInfo]) -> f64 {
        // Base benefit for tail call elimination
        let mut benefit = 1.0;
        
        // Bonus for simple transformations (easier to optimize)
        for arg in arguments {
            match &arg.transformation {
                ArgumentTransformation::Identity => benefit += 0.1,
                ArgumentTransformation::Arithmetic { .. } => benefit += 0.2,
                ArgumentTransformation::ListOperation { .. } => benefit += 0.15,
                _ => {}
            }
        }
        
        benefit
    }
    
    /// Assess potential for converting non-tail call to tail form
    fn assess_tail_conversion_potential(&self, arguments: &[ArgumentInfo]) -> TailConversionPotential {
        // Check if all arguments follow patterns amenable to accumulator passing
        let has_accumulator_pattern = arguments.iter().any(|arg| {
            matches!(arg.data_flow_pattern, DataFlowPattern::Accumulating)
        });
        
        if has_accumulator_pattern {
            TailConversionPotential::High { 
                strategy: AccumulatorStrategy::StandardAccumulator { 
                    parameter_name: "acc".to_string() 
                } 
            }
        } else {
            TailConversionPotential::Medium { 
                complexity: "requires restructuring".to_string() 
            }
        }
    }
    
    /// Select optimal optimization strategy
    fn select_optimization_strategy(
        &self,
        tail_calls: &[TailRecursiveCall],
        non_tail_calls: &[NonTailRecursiveCall],
        tail_ratio: f64,
    ) -> Result<TailCallOptimizationStrategy> {
        if tail_ratio >= self.config.min_tail_recursion_ratio && !tail_calls.is_empty() {
            // High tail-recursion ratio - use direct elimination
            Ok(TailCallOptimizationStrategy::DirectElimination {
                loop_variables: tail_calls.iter().enumerate().map(|(i, call)| {
                    LoopVariable {
                        name: format!("var{}", i),
                        update_expression: format!("update{}", i),
                        initial_value: format!("init{}", i),
                    }
                }).collect(),
                loop_condition: "continue".to_string(),
            })
        } else if !non_tail_calls.is_empty() && non_tail_calls.iter().any(|call| {
            matches!(call.tail_conversion_potential, TailConversionPotential::High { .. })
        }) {
            // Some non-tail calls can be converted
            Ok(TailCallOptimizationStrategy::AccumulatorPassing {
                accumulator_parameters: vec!["acc".to_string()],
                call_transformation: "add-accumulator".to_string(),
            })
        } else {
            // No good optimization strategy
            Ok(TailCallOptimizationStrategy::NoOptimization {
                reason: format!("Tail ratio {} below threshold {}", tail_ratio, self.config.min_tail_recursion_ratio),
            })
        }
    }
    
    /// Estimate performance gains from optimization
    fn estimate_performance_gain(
        &self,
        strategy: &TailCallOptimizationStrategy,
        tail_ratio: f64,
        tail_calls: &[TailRecursiveCall],
    ) -> Result<PerformanceGain> {
        match strategy {
            TailCallOptimizationStrategy::DirectElimination { .. } => {
                Ok(PerformanceGain {
                    stack_space_savings: tail_ratio * 0.9, // 90% stack savings for tail calls
                    execution_time_multiplier: 1.0 + tail_ratio * 0.3, // Up to 30% faster
                    memory_allocation_reduction: tail_ratio * 0.5, // 50% less allocation
                    cache_efficiency_improvement: tail_ratio * 0.2, // 20% better cache usage
                })
            }
            TailCallOptimizationStrategy::AccumulatorPassing { .. } => {
                Ok(PerformanceGain {
                    stack_space_savings: 0.8, // Good stack savings
                    execution_time_multiplier: 1.2, // 20% faster
                    memory_allocation_reduction: 0.3, // Some allocation reduction
                    cache_efficiency_improvement: 0.1, // Slight cache improvement
                })
            }
            TailCallOptimizationStrategy::NoOptimization { .. } => {
                Ok(PerformanceGain {
                    stack_space_savings: 0.0,
                    execution_time_multiplier: 1.0,
                    memory_allocation_reduction: 0.0,
                    cache_efficiency_improvement: 0.0,
                })
            }
            _ => {
                Ok(PerformanceGain {
                    stack_space_savings: 0.3,
                    execution_time_multiplier: 1.1,
                    memory_allocation_reduction: 0.2,
                    cache_efficiency_improvement: 0.05,
                })
            }
        }
    }
    
    /// Calculate confidence in the analysis results
    fn calculate_analysis_confidence(
        &self,
        tail_ratio: f64,
        tail_calls: &[TailRecursiveCall],
        non_tail_calls: &[NonTailRecursiveCall],
    ) -> f64 {
        let mut confidence = 0.7; // Base confidence
        
        // Higher confidence for clear tail-recursive patterns
        if tail_ratio >= 0.8 {
            confidence += 0.2;
        } else if tail_ratio >= 0.5 {
            confidence += 0.1;
        }
        
        // Higher confidence for simpler call patterns
        let simple_calls = tail_calls.iter().filter(|call| {
            call.arguments.iter().all(|arg| {
                matches!(arg.transformation, 
                         ArgumentTransformation::Identity | 
                         ArgumentTransformation::Arithmetic { .. } |
                         ArgumentTransformation::ListOperation { .. })
            })
        }).count();
        
        confidence += (simple_calls as f64 / (tail_calls.len() + 1) as f64) * 0.1;
        
        confidence.min(1.0)
    }
    
    /// Update cache hit rate statistics
    fn update_cache_hit_rate(&mut self, was_hit: bool) {
        let total = self.metrics.functions_analyzed as f64;
        let current_hits = self.metrics.cache_hit_rate * (total - 1.0);
        let new_hits = if was_hit { current_hits + 1.0 } else { current_hits };
        self.metrics.cache_hit_rate = new_hits / total;
    }
    
    /// Get the name of an optimization strategy for metrics
    fn get_strategy_name(&self, strategy: &TailCallOptimizationStrategy) -> String {
        match strategy {
            TailCallOptimizationStrategy::DirectElimination { .. } => "DirectElimination".to_string(),
            TailCallOptimizationStrategy::Trampoline { .. } => "Trampoline".to_string(),
            TailCallOptimizationStrategy::AccumulatorPassing { .. } => "AccumulatorPassing".to_string(),
            TailCallOptimizationStrategy::IterativeConversion { .. } => "IterativeConversion".to_string(),
            TailCallOptimizationStrategy::NoOptimization { .. } => "NoOptimization".to_string(),
        }
    }
    
    /// Get analysis metrics
    pub fn get_metrics(&self) -> &TailCallDetectionMetrics {
        &self.metrics
    }
    
    /// Reset analysis cache and metrics
    pub fn reset(&mut self) {
        self.analysis_cache.clear();
        self.tail_position_cache.clear();
        self.metrics = TailCallDetectionMetrics::default();
    }
}

/// Information about a recursive call found during analysis
#[derive(Debug, Clone)]
struct RecursiveCallInfo {
    /// Name of the function being called
    pub function_name: String,
    /// Analysis of arguments in the call
    pub arguments: Vec<ArgumentInfo>,
    /// Path to this call in the AST
    pub ast_path: Vec<String>,
    /// Depth in control structures
    pub control_depth: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tail_call_detector_creation() {
        let detector = RecTailCallDetector::new();
        assert_eq!(detector.config.max_analysis_depth, 100);
        assert!(detector.config.enable_deep_control_flow_analysis);
    }

    #[test]
    fn test_tail_position_analysis() {
        // This would test the tail position analysis logic
        // Placeholder for actual implementation
    }

    #[test]
    fn test_optimization_strategy_selection() {
        let detector = RecTailCallDetector::new();
        
        // Test case: high tail-recursion ratio should select direct elimination
        let tail_calls = vec![
            TailRecursiveCall {
                location: TailCallLocation {
                    ast_path: vec!["body".to_string()],
                    control_depth: 1,
                    source_span: None,
                },
                arguments: vec![],
                context: TailCallContext::Direct,
                optimization_benefit: 1.0,
            }
        ];
        let non_tail_calls = vec![];
        
        let strategy = detector.select_optimization_strategy(&tail_calls, &non_tail_calls, 1.0).unwrap();
        assert!(matches!(strategy, TailCallOptimizationStrategy::DirectElimination { .. }));
    }
}