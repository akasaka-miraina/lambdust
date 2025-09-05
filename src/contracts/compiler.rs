//! Contract compiler for optimized runtime checking.
//!
//! This module compiles contract expressions into optimized runtime
//! checkers that can be efficiently executed. The compiler performs:
//!
//! - Contract normalization and simplification
//! - Dead code elimination for impossible contracts
//! - Predicate inlining and specialization
//! - Cache-friendly code generation
//! - Higher-order contract preparation

use crate::ast::Expr;
use crate::contracts::{
    ContractConfig, ContractError, ContractResult,
    ast::{ComparisonOp, ContractExpr, DependentBinding, FunctionCase},
    blame::{BlameBoundary, BlameInfo, BlameTarget, BoundaryType},
    combinators::{CombinatorContext, ContractCombinators},
    predicates::{ContractPredicate, PredicateComplexity, PredicateRegistry},
};
use crate::diagnostics::{Span, Spanned};
use crate::eval::Value;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

/// Compiled contract representation for efficient runtime checking.
pub struct CompiledContract {
    /// Unique identifier for this compiled contract
    pub id: ContractId,
    /// Original contract expression
    pub original: ContractExpr,
    /// Compiled checker function
    pub checker: ContractChecker,
    /// Optimization level applied
    pub optimization_level: OptimizationLevel,
    /// Performance characteristics
    pub performance: PerformanceInfo,
    /// Dependencies on other contracts
    pub dependencies: HashSet<ContractId>,
    /// Metadata for debugging and introspection
    pub metadata: CompilationMetadata,
    /// Legacy predicate function (for backwards compatibility)
    pub predicate: Box<dyn Fn(&Value) -> bool + Send + Sync>,
    /// Blame information for this contract
    pub blame_info: BlameInfo,
    /// Human-readable contract name
    pub contract_name: String,
}

impl fmt::Debug for CompiledContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompiledContract")
            .field("id", &self.id)
            .field("original", &self.original)
            .field("optimization_level", &self.optimization_level)
            .field("performance", &self.performance)
            .field("dependencies", &self.dependencies)
            .field("metadata", &self.metadata)
            .field("blame_info", &self.blame_info)
            .field("contract_name", &self.contract_name)
            .field("checker", &"<function>")
            .field("predicate", &"<function>")
            .finish()
    }
}

impl Clone for CompiledContract {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            original: self.original.clone(),
            checker: self.checker.clone(), // Arc can be cloned
            optimization_level: self.optimization_level,
            performance: self.performance.clone(),
            dependencies: self.dependencies.clone(),
            metadata: self.metadata.clone(),
            // For function traits, we'll create new instances with same behavior
            predicate: Box::new(|_| true), // Default predicate
            blame_info: self.blame_info.clone(),
            contract_name: self.contract_name.clone(),
        }
    }
}

/// Unique identifier for compiled contracts.
pub type ContractId = u64;

/// Contract checker function type.
pub type ContractChecker = Arc<dyn Fn(&Value, &BlameInfo) -> ContractResult<bool> + Send + Sync>;

/// Contract compiler that optimizes contracts for runtime execution.
#[derive(Debug)]
pub struct ContractCompiler {
    /// Configuration
    config: ContractConfig,
    /// Predicate registry
    predicates: PredicateRegistry,
    /// Contract combinators
    combinators: ContractCombinators,
    /// Next contract ID
    next_id: ContractId,
    /// Compiled contract cache
    cache: HashMap<String, Arc<CompiledContract>>,
    /// Optimization pipeline
    optimizer: ContractOptimizer,
}

/// Compilation context for contract compilation.
#[derive(Debug, Clone)]
pub struct CompilationContext {
    /// Target optimization level
    pub optimization_level: OptimizationLevel,
    /// Available predicates
    pub predicates: PredicateRegistry,
    /// Contract environment (for parametric contracts)
    pub environment: HashMap<String, ContractExpr>,
    /// Current recursion depth
    pub recursion_depth: usize,
    /// Maximum recursion depth
    pub max_recursion_depth: usize,
    /// Blame information
    pub blame: BlameInfo,
}

/// Optimization levels for contract compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptimizationLevel {
    /// No optimization - direct interpretation
    None,
    /// Basic optimizations - predicate inlining, constant folding
    Basic,
    /// Standard optimizations - dead code elimination, simplification
    Standard,
    /// Aggressive optimizations - specialization, loop unrolling
    Aggressive,
    /// Maximum optimizations - all available optimizations
    Maximum,
}

/// Performance characteristics of compiled contracts.
#[derive(Debug, Clone)]
pub struct PerformanceInfo {
    /// Estimated time complexity
    pub time_complexity: PerformanceComplexity,
    /// Estimated space complexity
    pub space_complexity: PerformanceComplexity,
    /// Whether the contract is deterministic
    pub deterministic: bool,
    /// Estimated number of operations
    pub operation_count: usize,
    /// Whether the contract can be inlined
    pub inlinable: bool,
}

/// Performance complexity categories.
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceComplexity {
    /// O(1) constant time complexity
    Constant,
    /// O(log n) logarithmic time complexity
    Logarithmic,
    /// O(n) linear time complexity
    Linear,
    /// O(n²) quadratic time complexity
    Quadratic,
    /// O(2^n) exponential time complexity
    Exponential,
    /// Unknown or undetermined complexity
    Unknown,
}

/// Metadata about contract compilation.
#[derive(Debug, Clone)]
pub struct CompilationMetadata {
    /// Compilation timestamp
    pub timestamp: std::time::SystemTime,
    /// Source location of original contract
    pub source_location: Span,
    /// Applied optimizations
    pub optimizations: Vec<String>,
    /// Warnings generated during compilation
    pub warnings: Vec<String>,
    /// Size metrics
    pub size_info: SizeInfo,
}

/// Size information about compiled contracts.
#[derive(Debug, Clone)]
pub struct SizeInfo {
    /// Number of AST nodes in original contract
    pub original_nodes: usize,
    /// Number of operations in compiled form
    pub compiled_operations: usize,
    /// Memory usage estimate
    pub estimated_memory: usize,
}

/// Contract optimizer for applying various optimizations.
#[derive(Debug)]
pub struct ContractOptimizer {
    /// Enabled optimizations
    enabled_optimizations: HashSet<OptimizationType>,
}

/// Types of contract optimizations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OptimizationType {
    /// Inline simple predicates
    PredicateInlining,
    /// Eliminate unreachable code
    DeadCodeElimination,
    /// Fold constant expressions
    ConstantFolding,
    /// Simplify contract expressions
    Simplification,
    /// Specialize for common cases
    Specialization,
    /// Combine adjacent checks
    CheckCombining,
    /// Hoist loop-invariant checks
    LoopHoisting,
    /// Cache predicate results
    PredicateCaching,
}

impl ContractCompiler {
    /// Creates a new contract compiler.
    pub fn new(config: &ContractConfig, predicates: PredicateRegistry) -> Self {
        let combinators = ContractCombinators::new(predicates.clone());
        let optimizer = ContractOptimizer::new(config);

        Self {
            config: config.clone(),
            predicates,
            combinators,
            next_id: 1,
            cache: HashMap::new(),
            optimizer,
        }
    }

    /// Updates the compiler configuration.
    pub fn update_config(&mut self, config: &ContractConfig) {
        self.config = config.clone();
        self.optimizer = ContractOptimizer::new(config);
    }

    /// Compiles a contract expression.
    pub fn compile(
        &mut self,
        contract: &ContractExpr,
        context: &CompilationContext,
    ) -> ContractResult<Arc<CompiledContract>> {
        // Check cache first
        if self.config.enable_contract_caching {
            let cache_key = self.generate_cache_key(contract, context);
            if let Some(cached) = self.cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Compile the contract
        let id = self.next_contract_id();
        let optimized = self.optimizer.optimize(contract, context)?;
        let checker = self.compile_to_checker(&optimized, context)?;
        let performance = self.analyze_performance(&optimized, context);
        let dependencies = self.analyze_dependencies(&optimized);
        let metadata = self.create_metadata(&optimized, context);

        let compiled = Arc::new(CompiledContract {
            id,
            original: contract.clone(),
            checker: checker.clone(),
            optimization_level: context.optimization_level,
            performance,
            dependencies,
            metadata,
            // Legacy compatibility fields
            predicate: Box::new(|_| true), // Default predicate
            blame_info: context.blame.clone(),
            contract_name: format!("contract_{id}"),
        });

        // Cache the result
        if self.config.enable_contract_caching {
            let cache_key = self.generate_cache_key(contract, context);
            self.cache.insert(cache_key, compiled.clone());
        }

        Ok(compiled)
    }

    /// Generates a cache key for a contract and context.
    fn generate_cache_key(&self, contract: &ContractExpr, context: &CompilationContext) -> String {
        format!("{:?}:{:?}", contract, context.optimization_level)
    }

    /// Generates the next contract ID.
    fn next_contract_id(&mut self) -> ContractId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Compiles an optimized contract to a checker function.
    fn compile_to_checker(
        &self,
        contract: &ContractExpr,
        context: &CompilationContext,
    ) -> ContractResult<ContractChecker> {
        match contract {
            ContractExpr::Predicate { name, location } => {
                self.compile_predicate_checker(name, *location, context)
            }
            ContractExpr::Any { .. } => Ok(Arc::new(|_, _| Ok(true))),
            ContractExpr::None { .. } => Ok(Arc::new(|_, _| Ok(false))),
            ContractExpr::And {
                contracts,
                location,
            } => self.compile_and_checker(contracts, *location, context),
            ContractExpr::Or {
                contracts,
                location,
            } => self.compile_or_checker(contracts, *location, context),
            ContractExpr::Not { contract, location } => {
                self.compile_not_checker(contract, *location, context)
            }
            ContractExpr::Function {
                domain,
                codomain,
                location,
            } => self.compile_function_checker(domain, codomain, *location, context),
            ContractExpr::ListOf {
                element_contract,
                location,
            } => self.compile_listof_checker(element_contract, *location, context),
            ContractExpr::VectorOf {
                element_contract,
                location,
            } => self.compile_vectorof_checker(element_contract, *location, context),
            _ => {
                // For unimplemented contract types, use the combinator evaluator
                let contract_copy = contract.clone();
                let predicates = context.predicates.clone();

                Ok(Arc::new(move |value, blame| {
                    let mut ctx = CombinatorContext::new(predicates.clone(), blame.clone());
                    let combinators = ContractCombinators::new(predicates.clone());
                    combinators.evaluate_contract(&contract_copy, value, &mut ctx)
                }))
            }
        }
    }

    /// Compiles a predicate checker.
    fn compile_predicate_checker(
        &self,
        name: &str,
        location: Span,
        context: &CompilationContext,
    ) -> ContractResult<ContractChecker> {
        let predicate =
            context
                .predicates
                .lookup(name)
                .ok_or_else(|| ContractError::CompilationError {
                    message: format!("Unknown predicate: {name}"),
                    location,
                })?;

        // For simple, deterministic predicates, we can inline them
        if predicate.deterministic && predicate.complexity == PredicateComplexity::Constant {
            let pred_fn = predicate.predicate.clone();
            Ok(Arc::new(move |value, _blame| Ok(pred_fn(value))))
        } else {
            // For complex predicates, use the original function
            let pred_fn = predicate.predicate.clone();
            Ok(Arc::new(move |value, _blame| Ok(pred_fn(value))))
        }
    }

    /// Compiles an and combinator checker.
    fn compile_and_checker(
        &self,
        contracts: &[Spanned<ContractExpr>],
        location: Span,
        context: &CompilationContext,
    ) -> ContractResult<ContractChecker> {
        let mut checkers = Vec::new();
        for contract in contracts {
            let checker = self.compile_to_checker(&contract.inner, context)?;
            checkers.push(checker);
        }

        Ok(Arc::new(move |value, blame| {
            for checker in &checkers {
                if !checker(value, blame)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }))
    }

    /// Compiles an or combinator checker.
    fn compile_or_checker(
        &self,
        contracts: &[Spanned<ContractExpr>],
        location: Span,
        context: &CompilationContext,
    ) -> ContractResult<ContractChecker> {
        let mut checkers = Vec::new();
        for contract in contracts {
            let checker = self.compile_to_checker(&contract.inner, context)?;
            checkers.push(checker);
        }

        Ok(Arc::new(move |value, blame| {
            for checker in &checkers {
                if checker(value, blame)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }))
    }

    /// Compiles a not combinator checker.
    fn compile_not_checker(
        &self,
        contract: &Spanned<ContractExpr>,
        location: Span,
        context: &CompilationContext,
    ) -> ContractResult<ContractChecker> {
        let checker = self.compile_to_checker(&contract.inner, context)?;

        Ok(Arc::new(move |value, blame| Ok(!checker(value, blame)?)))
    }

    /// Compiles a function contract checker.
    fn compile_function_checker(
        &self,
        domain: &[Spanned<ContractExpr>],
        codomain: &Spanned<ContractExpr>,
        location: Span,
        context: &CompilationContext,
    ) -> ContractResult<ContractChecker> {
        // Function contracts require wrapper generation
        // For now, just check if the value is a procedure
        Ok(Arc::new(|value, _blame| {
            Ok(matches!(
                value,
                Value::Procedure(_) | Value::CaseLambda(_) | Value::Primitive(_)
            ))
        }))
    }

    /// Compiles a listof contract checker.
    fn compile_listof_checker(
        &self,
        element_contract: &Spanned<ContractExpr>,
        location: Span,
        context: &CompilationContext,
    ) -> ContractResult<ContractChecker> {
        let element_checker = self.compile_to_checker(&element_contract.inner, context)?;

        Ok(Arc::new(move |value, blame| {
            fn check_list_elements(
                value: &Value,
                checker: &ContractChecker,
                blame: &BlameInfo,
            ) -> ContractResult<bool> {
                match value {
                    Value::Nil => Ok(true),
                    Value::Pair(car, cdr) => {
                        if !checker(car, blame)? {
                            return Ok(false);
                        }
                        check_list_elements(cdr, checker, blame)
                    }
                    _ => Ok(false), // Not a proper list
                }
            }

            check_list_elements(value, &element_checker, blame)
        }))
    }

    /// Compiles a vectorof contract checker.
    fn compile_vectorof_checker(
        &self,
        element_contract: &Spanned<ContractExpr>,
        location: Span,
        context: &CompilationContext,
    ) -> ContractResult<ContractChecker> {
        let element_checker = self.compile_to_checker(&element_contract.inner, context)?;

        Ok(Arc::new(move |value, blame| match value {
            Value::Vector(vec) => {
                if let Ok(vector) = vec.try_borrow() {
                    for element in vector.iter() {
                        if !element_checker(element, blame)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Err(ContractError::RuntimeError {
                        message: "Failed to read vector".to_string(),
                        location: Span::new(0, 0),
                    }
                    .into())
                }
            }
            _ => Ok(false),
        }))
    }

    /// Analyzes the performance characteristics of a contract.
    fn analyze_performance(
        &self,
        contract: &ContractExpr,
        context: &CompilationContext,
    ) -> PerformanceInfo {
        let (time_complexity, operation_count) = self.analyze_time_complexity(contract);
        let space_complexity = self.analyze_space_complexity(contract);
        let deterministic = self.is_deterministic(contract);
        let inlinable = self.is_inlinable(contract);

        PerformanceInfo {
            time_complexity,
            space_complexity,
            deterministic,
            operation_count,
            inlinable,
        }
    }

    /// Analyzes time complexity of a contract.
    #[allow(clippy::only_used_in_recursion)]
    fn analyze_time_complexity(&self, contract: &ContractExpr) -> (PerformanceComplexity, usize) {
        match contract {
            ContractExpr::Predicate { .. } => (PerformanceComplexity::Constant, 1),
            ContractExpr::Any { .. } | ContractExpr::None { .. } => {
                (PerformanceComplexity::Constant, 1)
            }
            ContractExpr::And { contracts, .. } | ContractExpr::Or { contracts, .. } => {
                let total_ops: usize = contracts
                    .iter()
                    .map(|c| self.analyze_time_complexity(&c.inner).1)
                    .sum();
                (PerformanceComplexity::Linear, total_ops)
            }
            ContractExpr::Not { contract, .. } => {
                let (complexity, ops) = self.analyze_time_complexity(&contract.inner);
                (complexity, ops + 1)
            }
            ContractExpr::ListOf { .. } | ContractExpr::VectorOf { .. } => {
                (PerformanceComplexity::Linear, 10) // Estimate
            }
            _ => (PerformanceComplexity::Unknown, 5), // Default estimate
        }
    }

    /// Analyzes space complexity of a contract.
    fn analyze_space_complexity(&self, contract: &ContractExpr) -> PerformanceComplexity {
        match contract {
            ContractExpr::Predicate { .. }
            | ContractExpr::Any { .. }
            | ContractExpr::None { .. } => PerformanceComplexity::Constant,
            ContractExpr::ListOf { .. } | ContractExpr::VectorOf { .. } => {
                PerformanceComplexity::Linear // Stack depth for recursion
            }
            _ => PerformanceComplexity::Constant,
        }
    }

    /// Checks if a contract is deterministic.
    fn is_deterministic(&self, contract: &ContractExpr) -> bool {
        match contract {
            ContractExpr::Predicate { name, .. } => {
                if let Some(pred) = self.predicates.lookup(name) {
                    pred.deterministic
                } else {
                    false
                }
            }
            ContractExpr::Any { .. } | ContractExpr::None { .. } => true,
            ContractExpr::And { contracts, .. } | ContractExpr::Or { contracts, .. } => {
                contracts.iter().all(|c| self.is_deterministic(&c.inner))
            }
            ContractExpr::Not { contract, .. } => self.is_deterministic(&contract.inner),
            _ => true, // Assume deterministic for other types
        }
    }

    /// Checks if a contract can be inlined.
    #[allow(clippy::only_used_in_recursion)]
    fn is_inlinable(&self, contract: &ContractExpr) -> bool {
        match contract {
            ContractExpr::Predicate { .. }
            | ContractExpr::Any { .. }
            | ContractExpr::None { .. } => true,
            ContractExpr::And { contracts, .. } | ContractExpr::Or { contracts, .. } => {
                contracts.len() <= 3 && contracts.iter().all(|c| self.is_inlinable(&c.inner))
            }
            ContractExpr::Not { contract, .. } => self.is_inlinable(&contract.inner),
            _ => false,
        }
    }

    /// Analyzes contract dependencies.
    fn analyze_dependencies(&self, contract: &ContractExpr) -> HashSet<ContractId> {
        // TODO: Implement dependency analysis for recursive and parametric contracts
        HashSet::new()
    }

    /// Creates compilation metadata.
    fn create_metadata(
        &self,
        contract: &ContractExpr,
        context: &CompilationContext,
    ) -> CompilationMetadata {
        let original_nodes = self.count_ast_nodes(contract);

        CompilationMetadata {
            timestamp: std::time::SystemTime::now(),
            source_location: contract.location(),
            optimizations: Vec::new(), // TODO: Track applied optimizations
            warnings: Vec::new(),
            size_info: SizeInfo {
                original_nodes,
                compiled_operations: original_nodes,   // Estimate
                estimated_memory: original_nodes * 64, // Rough estimate
            },
        }
    }

    /// Counts AST nodes in a contract.
    #[allow(clippy::only_used_in_recursion)]
    fn count_ast_nodes(&self, contract: &ContractExpr) -> usize {
        match contract {
            ContractExpr::Predicate { .. }
            | ContractExpr::Any { .. }
            | ContractExpr::None { .. } => 1,
            ContractExpr::And { contracts, .. } | ContractExpr::Or { contracts, .. } => {
                1 + contracts
                    .iter()
                    .map(|c| self.count_ast_nodes(&c.inner))
                    .sum::<usize>()
            }
            ContractExpr::Not { contract, .. } => 1 + self.count_ast_nodes(&contract.inner),
            ContractExpr::ListOf {
                element_contract, ..
            }
            | ContractExpr::VectorOf {
                element_contract, ..
            } => 1 + self.count_ast_nodes(&element_contract.inner),
            _ => 1, // Default for other types
        }
    }
}

impl ContractOptimizer {
    /// Creates a new contract optimizer.
    pub fn new(config: &ContractConfig) -> Self {
        let mut enabled_optimizations = HashSet::new();

        if config.enable_compilation {
            enabled_optimizations.insert(OptimizationType::PredicateInlining);
            enabled_optimizations.insert(OptimizationType::ConstantFolding);
            enabled_optimizations.insert(OptimizationType::Simplification);
            enabled_optimizations.insert(OptimizationType::DeadCodeElimination);
        }

        Self {
            enabled_optimizations,
        }
    }

    /// Optimizes a contract expression.
    pub fn optimize(
        &self,
        contract: &ContractExpr,
        context: &CompilationContext,
    ) -> ContractResult<ContractExpr> {
        let mut optimized = contract.clone();

        // Apply optimizations based on level and enabled flags
        if context.optimization_level >= OptimizationLevel::Basic {
            if self
                .enabled_optimizations
                .contains(&OptimizationType::ConstantFolding)
            {
                optimized = self.constant_fold(&optimized)?;
            }
            if self
                .enabled_optimizations
                .contains(&OptimizationType::Simplification)
            {
                optimized = self.simplify(&optimized)?;
            }
        }

        if context.optimization_level >= OptimizationLevel::Standard
            && self
                .enabled_optimizations
                .contains(&OptimizationType::DeadCodeElimination)
        {
            optimized = self.eliminate_dead_code(&optimized)?;
        }

        Ok(optimized)
    }

    /// Performs constant folding on a contract.
    #[allow(clippy::only_used_in_recursion)]
    fn constant_fold(&self, contract: &ContractExpr) -> ContractResult<ContractExpr> {
        match contract {
            ContractExpr::And {
                contracts,
                location,
            } => {
                let mut folded_contracts = Vec::new();
                for contract in contracts {
                    let folded = self.constant_fold(&contract.inner)?;
                    match folded {
                        ContractExpr::Any { .. } => {
                            // any/c in AND can be removed
                            continue;
                        }
                        ContractExpr::None { .. } => {
                            // none/c in AND makes the whole thing none/c
                            return Ok(ContractExpr::None {
                                location: *location,
                            });
                        }
                        _ => {
                            folded_contracts.push(Spanned::new(folded, contract.span));
                        }
                    }
                }

                match folded_contracts.len() {
                    0 => Ok(ContractExpr::Any {
                        location: *location,
                    }),
                    1 => Ok(folded_contracts.into_iter().next().unwrap().inner),
                    _ => Ok(ContractExpr::And {
                        contracts: folded_contracts,
                        location: *location,
                    }),
                }
            }
            ContractExpr::Or {
                contracts,
                location,
            } => {
                let mut folded_contracts = Vec::new();
                for contract in contracts {
                    let folded = self.constant_fold(&contract.inner)?;
                    match folded {
                        ContractExpr::Any { .. } => {
                            // any/c in OR makes the whole thing any/c
                            return Ok(ContractExpr::Any {
                                location: *location,
                            });
                        }
                        ContractExpr::None { .. } => {
                            // none/c in OR can be removed
                            continue;
                        }
                        _ => {
                            folded_contracts.push(Spanned::new(folded, contract.span));
                        }
                    }
                }

                match folded_contracts.len() {
                    0 => Ok(ContractExpr::None {
                        location: *location,
                    }),
                    1 => Ok(folded_contracts.into_iter().next().unwrap().inner),
                    _ => Ok(ContractExpr::Or {
                        contracts: folded_contracts,
                        location: *location,
                    }),
                }
            }
            ContractExpr::Not { contract, location } => {
                let folded = self.constant_fold(&contract.inner)?;
                match folded {
                    ContractExpr::Any { .. } => Ok(ContractExpr::None {
                        location: *location,
                    }),
                    ContractExpr::None { .. } => Ok(ContractExpr::Any {
                        location: *location,
                    }),
                    ContractExpr::Not {
                        contract: inner, ..
                    } => {
                        // Double negation elimination
                        Ok(inner.inner)
                    }
                    _ => Ok(ContractExpr::Not {
                        contract: Box::new(Spanned::new(folded, contract.span)),
                        location: *location,
                    }),
                }
            }
            _ => Ok(contract.clone()),
        }
    }

    /// Simplifies a contract expression.
    fn simplify(&self, contract: &ContractExpr) -> ContractResult<ContractExpr> {
        // TODO: Implement contract simplification
        Ok(contract.clone())
    }

    /// Eliminates dead code from a contract.
    fn eliminate_dead_code(&self, contract: &ContractExpr) -> ContractResult<ContractExpr> {
        // TODO: Implement dead code elimination
        Ok(contract.clone())
    }
}

impl CompilationContext {
    /// Creates a new compilation context.
    pub fn new(predicates: PredicateRegistry, blame: BlameInfo) -> Self {
        Self {
            optimization_level: OptimizationLevel::Standard,
            predicates,
            environment: HashMap::new(),
            recursion_depth: 0,
            max_recursion_depth: 100,
            blame,
        }
    }

    /// Creates a default compilation context (for compatibility).
    pub fn new_default() -> Self {
        use crate::contracts::blame::{BlameBoundary, BlameInfo, BlameTarget, BoundaryType};
        use crate::contracts::predicates::PredicateRegistry;

        let predicates = PredicateRegistry::new();
        let blame = BlameInfo {
            positive: BlameTarget::System {
                component: "default".to_string(),
                description: "default context".to_string(),
            },
            negative: BlameTarget::System {
                component: "default".to_string(),
                description: "default context".to_string(),
            },
            boundary: BlameBoundary {
                boundary_type: BoundaryType::ExplicitContract,
                contract: "default".to_string(),
                location: crate::diagnostics::Span::default(),
                context: HashMap::new(),
            },
            call_stack: Vec::new(),
            id: 0,
            parent: None,
        };

        Self::new(predicates, blame)
    }

    /// Creates a context with custom optimization level.
    pub fn with_optimization_level(
        predicates: PredicateRegistry,
        blame: BlameInfo,
        level: OptimizationLevel,
    ) -> Self {
        Self {
            optimization_level: level,
            predicates,
            environment: HashMap::new(),
            recursion_depth: 0,
            max_recursion_depth: 100,
            blame,
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for OptimizationLevel {
    fn default() -> Self {
        OptimizationLevel::Standard
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{
        blame::{BlameBoundary, BlameInfo, BlameTarget, BoundaryType},
        predicates::PredicateRegistry,
    };
    use crate::diagnostics::Span;
    use std::collections::HashMap;

    fn create_test_context() -> CompilationContext {
        let predicates = PredicateRegistry::new();
        let blame = BlameInfo {
            positive: BlameTarget::System {
                component: "test".to_string(),
                description: "test context".to_string(),
            },
            negative: BlameTarget::System {
                component: "test".to_string(),
                description: "test context".to_string(),
            },
            boundary: BlameBoundary {
                boundary_type: BoundaryType::ExplicitContract,
                contract: "test".to_string(),
                location: Span::new(0, 0),
                context: HashMap::new(),
            },
            call_stack: Vec::new(),
            id: 1,
            parent: None,
        };

        CompilationContext::new(predicates, blame)
    }

    #[test]
    fn test_contract_compiler_creation() {
        let config = ContractConfig::default();
        let predicates = PredicateRegistry::new();
        let compiler = ContractCompiler::new(&config, predicates);

        assert_eq!(compiler.next_id, 1);
        assert!(compiler.cache.is_empty());
    }

    #[test]
    fn test_predicate_compilation() {
        let config = ContractConfig::default();
        let predicates = PredicateRegistry::new();
        let mut compiler = ContractCompiler::new(&config, predicates);
        let context = create_test_context();

        let contract = ContractExpr::Predicate {
            name: "number?".to_string(),
            location: Span::new(0, 0),
        };

        let compiled = compiler.compile(&contract, &context).unwrap();
        assert_eq!(compiled.id, 1);
        assert_eq!(compiled.optimization_level, OptimizationLevel::Standard);
        assert!(compiled.performance.deterministic);
    }

    #[test]
    fn test_any_none_compilation() {
        let config = ContractConfig::default();
        let predicates = PredicateRegistry::new();
        let mut compiler = ContractCompiler::new(&config, predicates);
        let context = create_test_context();

        let any_contract = ContractExpr::Any {
            location: Span::new(0, 0),
        };
        let none_contract = ContractExpr::None {
            location: Span::new(0, 0),
        };

        let compiled_any = compiler.compile(&any_contract, &context).unwrap();
        let compiled_none = compiler.compile(&none_contract, &context).unwrap();

        // Test the compiled checkers
        let dummy_value = Value::Literal(crate::ast::Literal::Number(42.0));
        let dummy_blame = context.blame.clone();

        assert!((compiled_any.checker)(&dummy_value, &dummy_blame).unwrap());
        assert!(!(compiled_none.checker)(&dummy_value, &dummy_blame).unwrap());
    }

    #[test]
    fn test_constant_folding() {
        let config = ContractConfig::default();
        let optimizer = ContractOptimizer::new(&config);
        let context = create_test_context();

        // Test AND with any/c - should be simplified
        let and_with_any = ContractExpr::And {
            contracts: vec![
                Spanned::new(
                    ContractExpr::Any {
                        location: Span::new(0, 0),
                    },
                    Span::new(0, 0),
                ),
                Spanned::new(
                    ContractExpr::Predicate {
                        name: "number?".to_string(),
                        location: Span::new(0, 0),
                    },
                    Span::new(0, 0),
                ),
            ],
            location: Span::new(0, 0),
        };

        let optimized = optimizer.constant_fold(&and_with_any).unwrap();
        assert!(matches!(optimized, ContractExpr::Predicate { .. }));

        // Test AND with none/c - should become none/c
        let and_with_none = ContractExpr::And {
            contracts: vec![
                Spanned::new(
                    ContractExpr::None {
                        location: Span::new(0, 0),
                    },
                    Span::new(0, 0),
                ),
                Spanned::new(
                    ContractExpr::Predicate {
                        name: "number?".to_string(),
                        location: Span::new(0, 0),
                    },
                    Span::new(0, 0),
                ),
            ],
            location: Span::new(0, 0),
        };

        let optimized = optimizer.constant_fold(&and_with_none).unwrap();
        assert!(matches!(optimized, ContractExpr::None { .. }));
    }

    #[test]
    fn test_performance_analysis() {
        let config = ContractConfig::default();
        let predicates = PredicateRegistry::new();
        let compiler = ContractCompiler::new(&config, predicates);
        let context = create_test_context();

        let simple_contract = ContractExpr::Predicate {
            name: "number?".to_string(),
            location: Span::new(0, 0),
        };

        let performance = compiler.analyze_performance(&simple_contract, &context);
        assert_eq!(performance.time_complexity, PerformanceComplexity::Constant);
        assert_eq!(performance.operation_count, 1);
        assert!(performance.deterministic);
        assert!(performance.inlinable);
    }

    #[test]
    fn test_cache_key_generation() {
        let config = ContractConfig::default();
        let predicates = PredicateRegistry::new();
        let compiler = ContractCompiler::new(&config, predicates);
        let context = create_test_context();

        let contract = ContractExpr::Predicate {
            name: "number?".to_string(),
            location: Span::new(0, 0),
        };

        let key1 = compiler.generate_cache_key(&contract, &context);
        let key2 = compiler.generate_cache_key(&contract, &context);
        assert_eq!(key1, key2);

        let different_context = CompilationContext::with_optimization_level(
            context.predicates.clone(),
            context.blame.clone(),
            OptimizationLevel::Maximum,
        );
        let key3 = compiler.generate_cache_key(&contract, &different_context);
        assert_ne!(key1, key3);
    }
}
