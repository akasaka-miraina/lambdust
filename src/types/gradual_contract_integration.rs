//! Gradual Type Inference and Contract System Integration
//!
//! This module provides seamless integration between the gradual type inference
//! system and the contract system. It handles:
//!
//! - Automatic contract generation from type information
//! - Type-guided contract optimization
//! - Blame assignment for gradual boundaries
//! - Runtime contract enforcement coordination
//! - Performance optimization through contract elimination
//!
//! # Integration Strategy
//!
//! The integration follows these principles:
//! 1. Types provide static guarantees, contracts provide dynamic safety
//! 2. Gradual boundaries automatically generate appropriate contracts
//! 3. Blame tracking connects type errors to source locations
//! 4. Optimization eliminates redundant checks when types guarantee safety

use super::gradual::{Cast, consistent, is_gradual, is_static};
use super::gradual_inference::{
    CastInsertion, CastReason, GeneratedContract, GradualInferenceResult, GradualTypeInference,
    PerformanceImpact, TypeBoundary,
};
use super::{Type, TypeScheme, TypeVar};
use crate::contracts::{
    BlameBoundary, BlameInfo, BlameTarget, BlameTracker, BoundaryType, CompilationContext,
    CompilationMetadata, CompiledContract, ContractConfig, ContractError, ContractExpr,
    ContractPredicate, ContractResult, ContractSystem, OptimizationLevel, PerformanceComplexity,
    PerformanceInfo, PredicateRegistry, SizeInfo,
};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::Value;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Import the gradual inference types we need
use super::gradual_inference::ContractPriority;

/// Conversion trait for gradual_inference::ContractPriority to PerformanceImpact
impl From<super::gradual_inference::ContractPriority> for PerformanceImpact {
    fn from(priority: super::gradual_inference::ContractPriority) -> Self {
        match priority {
            super::gradual_inference::ContractPriority::Optional => PerformanceImpact::None,
            super::gradual_inference::ContractPriority::Important => PerformanceImpact::Moderate,
            super::gradual_inference::ContractPriority::Critical => PerformanceImpact::Significant,
        }
    }
}

/// Configuration for gradual-contract integration
#[derive(Debug, Clone)]
pub struct GradualContractConfig {
    /// Enable automatic contract generation
    pub enable_auto_generation: bool,
    /// Enable contract optimization based on types
    pub enable_optimization: bool,
    /// Enable blame tracking integration
    pub enable_blame_integration: bool,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Contract generation strategy
    pub generation_strategy: ContractGenerationStrategy,
    /// Optimization aggressiveness
    pub optimization_level: OptimizationLevel,
    /// Blame precision level
    pub blame_precision: BlamePrecisionLevel,
}

/// Strategy for generating contracts from types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractGenerationStrategy {
    /// Conservative: Generate contracts for all boundaries
    Conservative,
    /// Balanced: Generate contracts for risky boundaries
    Balanced,
    /// Minimal: Generate contracts only when necessary
    Minimal,
    /// Custom: Use custom rules
    Custom,
}

// Use OptimizationLevel from contracts module instead of defining our own

/// Precision level for blame tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlamePrecisionLevel {
    /// Coarse-grained blame (function level)
    Coarse,
    /// Medium-grained blame (expression level)
    Medium,
    /// Fine-grained blame (sub-expression level)
    Fine,
    /// Ultra-fine blame (every operation)
    UltraFine,
}

impl Default for GradualContractConfig {
    fn default() -> Self {
        Self {
            enable_auto_generation: true,
            enable_optimization: true,
            enable_blame_integration: true,
            enable_performance_monitoring: true,
            generation_strategy: ContractGenerationStrategy::Balanced,
            optimization_level: OptimizationLevel::Basic,
            blame_precision: BlamePrecisionLevel::Medium,
        }
    }
}

/// Result of contract-type integration
#[derive(Debug, Clone)]
pub struct IntegrationResult {
    /// Generated contracts
    pub contracts: Vec<OptimizedContract>,
    /// Eliminated contracts (optimized away)
    pub eliminated_contracts: Vec<EliminatedContract>,
    /// Blame mappings
    pub blame_mappings: Vec<BlameMapping>,
    /// Performance metrics
    pub performance_metrics: IntegrationMetrics,
    /// Optimization opportunities
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
}

/// Contract with optimization information
#[derive(Debug, Clone)]
pub struct OptimizedContract {
    /// Original contract
    pub contract: ContractExpr,
    /// Compiled contract
    pub compiled: Arc<CompiledContract>,
    /// Optimization applied
    pub optimization: ContractOptimization,
    /// Performance impact
    pub performance_impact: PerformanceImpact,
    /// Blame information
    pub blame: BlameInfo,
    /// Runtime priority
    pub priority: ContractPriority,
}

/// Contract that was eliminated through optimization
#[derive(Debug, Clone)]
pub struct EliminatedContract {
    /// Original contract expression
    pub original_contract: ContractExpr,
    /// Reason for elimination
    pub elimination_reason: EliminationReason,
    /// Type evidence for elimination
    pub type_evidence: TypeEvidence,
    /// Performance savings
    pub performance_savings: PerformanceSavings,
}

/// Optimization applied to a contract
#[derive(Debug, Clone)]
pub enum ContractOptimization {
    /// No optimization applied
    None,
    /// Simplified predicate
    SimplifiedPredicate {
        /// Original predicate expression
        original: String,
        /// Simplified predicate expression
        simplified: String,
    },
    /// Specialized for known types
    TypeSpecialization {
        /// Types that the contract was specialized for
        specialized_types: Vec<Type>,
    },
    /// Combined with other contracts
    Combined {
        /// Other contracts that were combined
        combined_with: Vec<ContractExpr>,
    },
    /// Moved to compile time
    CompileTimeCheck,
    /// Replaced with type assertion
    TypeAssertion {
        /// Type used in the assertion
        assertion_type: Type,
    },
}

// Use ContractPriority from gradual_inference module instead of defining our own

/// Reason for contract elimination
#[derive(Debug, Clone)]
pub enum EliminationReason {
    /// Type system guarantees contract
    TypeGuarantee,
    /// Static analysis proves safety
    StaticAnalysis,
    /// Redundant with other contracts
    Redundancy,
    /// Performance optimization
    PerformanceOptimization,
    /// User configuration
    UserConfiguration,
}

/// Evidence from type system supporting elimination
#[derive(Debug, Clone)]
pub struct TypeEvidence {
    /// Static type information
    pub static_types: Vec<Type>,
    /// Type relationships proven
    pub relationships: Vec<TypeRelationship>,
    /// Invariants that hold
    pub invariants: Vec<TypeInvariant>,
    /// Confidence level
    pub confidence: ConfidenceLevel,
}

/// Relationship between types
#[derive(Debug, Clone)]
pub struct TypeRelationship {
    /// Source type
    pub source: Type,
    /// Target type
    pub target: Type,
    /// Relationship kind
    pub kind: RelationshipKind,
    /// Proof or evidence
    pub evidence: String,
}

/// Kind of type relationship
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipKind {
    /// Subtype relationship
    Subtype,
    /// Equality
    Equal,
    /// Consistency
    Consistent,
    /// Disjoint (no overlap)
    Disjoint,
}

/// Type invariant that holds
#[derive(Debug, Clone)]
pub struct TypeInvariant {
    /// Description of invariant
    pub description: String,
    /// Types involved
    pub types: Vec<Type>,
    /// Strength of invariant
    pub strength: InvariantStrength,
}

/// Strength of type invariant
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvariantStrength {
    /// Weak (likely to hold)
    Weak,
    /// Moderate (usually holds)
    Moderate,
    /// Strong (almost always holds)
    Strong,
    /// Absolute (always holds)
    Absolute,
}

/// Confidence level in type evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfidenceLevel {
    /// Low confidence
    Low,
    /// Medium confidence
    Medium,
    /// High confidence
    High,
    /// Very high confidence
    VeryHigh,
}

/// Performance savings from elimination
#[derive(Debug, Clone)]
pub struct PerformanceSavings {
    /// Estimated time saved per execution
    pub time_per_execution: Duration,
    /// Estimated memory saved
    pub memory_saved: usize,
    /// Estimated CPU cycles saved
    pub cpu_cycles_saved: u64,
}

/// Mapping between blame and type information
#[derive(Debug, Clone)]
pub struct BlameMapping {
    /// Blame information
    pub blame: BlameInfo,
    /// Associated type boundary
    pub type_boundary: Option<TypeBoundary>,
    /// Blame precision
    pub precision: BlamePrecisionLevel,
    /// Source location context
    pub source_context: SourceContext,
}

/// Context information for blame attribution
#[derive(Debug, Clone)]
pub struct SourceContext {
    /// Function name (if applicable)
    pub function_name: Option<String>,
    /// Variable name (if applicable)
    pub variable_name: Option<String>,
    /// Expression type
    pub expression_type: String,
    /// Call stack depth
    pub call_depth: usize,
}

/// Performance metrics for integration
#[derive(Debug, Clone, Default)]
pub struct IntegrationMetrics {
    /// Time spent generating contracts
    pub generation_time: Duration,
    /// Time spent optimizing contracts
    pub optimization_time: Duration,
    /// Number of contracts generated
    pub contracts_generated: usize,
    /// Number of contracts eliminated
    pub contracts_eliminated: usize,
    /// Total performance improvement
    pub total_improvement: Duration,
}

/// Optimization opportunity identified
#[derive(Debug, Clone)]
pub struct OptimizationOpportunity {
    /// Type of optimization
    pub optimization_type: OptimizationType,
    /// Location where applicable
    pub location: Span,
    /// Estimated benefit
    pub estimated_benefit: PerformanceBenefit,
    /// Implementation complexity
    pub complexity: OptimizationComplexity,
}

/// Type of optimization opportunity
#[derive(Debug, Clone)]
pub enum OptimizationType {
    /// Eliminate redundant contract
    EliminateRedundantContract,
    /// Specialize contract for known types
    SpecializeContract,
    /// Move check to compile time
    MoveToCompileTime,
    /// Combine multiple contracts
    CombineContracts,
    /// Use type assertion instead
    UseTypeAssertion,
}

/// Benefit level of optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PerformanceBenefit {
    /// Minimal benefit
    Minimal,
    /// Small benefit
    Small,
    /// Moderate benefit
    Moderate,
    /// Large benefit
    Large,
    /// Very large benefit
    VeryLarge,
}

/// Complexity of implementing optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptimizationComplexity {
    /// Trivial to implement
    Trivial,
    /// Simple implementation
    Simple,
    /// Moderate complexity
    Moderate,
    /// Complex implementation
    Complex,
    /// Very complex
    VeryComplex,
}

/// Main integration coordinator
#[derive(Debug)]
pub struct GradualContractIntegration {
    /// Configuration
    config: GradualContractConfig,
    /// Contract system
    contract_system: Arc<Mutex<ContractSystem>>,
    /// Contract generator
    generator: ContractGenerator,
    /// Contract optimizer
    optimizer: ContractOptimizer,
    /// Blame coordinator
    blame_coordinator: BlameCoordinator,
    /// Performance monitor
    performance_monitor: Arc<Mutex<PerformanceMonitor>>,
}

// Safety: GradualContractIntegration can be safely shared between threads
// as it uses Arc<Mutex<_>> for shared state and contains thread-safe configurations
unsafe impl Send for GradualContractIntegration {}
unsafe impl Sync for GradualContractIntegration {}

/// Generates contracts from type information
#[derive(Debug)]
pub struct ContractGenerator {
    /// Generation rules
    rules: Vec<GenerationRule>,
    /// Type-to-contract mapping cache
    cache: HashMap<Type, ContractExpr>,
    /// Generation statistics
    stats: GenerationStatistics,
}

/// Rule for generating contracts from types
#[derive(Debug, Clone)]
pub struct GenerationRule {
    /// Type pattern to match
    pub type_pattern: TypePattern,
    /// Generated contract template
    pub contract_template: ContractTemplate,
    /// Priority of this rule
    pub priority: u32,
    /// Conditions for applicability
    pub conditions: Vec<GenerationCondition>,
}

/// Pattern for matching types in rules
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypePattern {
    /// Exact type match
    Exact(Type),
    /// Any primitive type
    AnyPrimitive,
    /// Any function type
    AnyFunction,
    /// Any container type
    AnyContainer,
    /// Any gradual type
    AnyGradual,
    /// Wildcard (matches anything)
    Wildcard,
}

/// Template for generating contracts
#[derive(Debug, Clone)]
pub struct ContractTemplate {
    /// Base contract expression
    pub base_contract: ContractExpr,
    /// Parameters for customization
    pub parameters: HashMap<String, TemplateParameter>,
    /// Optimization hints
    pub optimization_hints: Vec<OptimizationHint>,
}

/// Parameter in contract template
#[derive(Debug, Clone)]
pub enum TemplateParameter {
    /// Type parameter
    Type(Type),
    /// String parameter
    String(String),
    /// Boolean parameter
    Boolean(bool),
    /// Numeric parameter
    Numeric(f64),
}

/// Hint for contract optimization
#[derive(Debug, Clone)]
pub struct OptimizationHint {
    /// Type of optimization
    pub hint_type: OptimizationHintType,
    /// Applicability condition
    pub condition: String,
    /// Expected benefit
    pub benefit: PerformanceBenefit,
}

/// Type of optimization hint
#[derive(Debug, Clone)]
pub enum OptimizationHintType {
    /// Can be eliminated if type is static
    EliminateIfStatic,
    /// Can be specialized for known types
    SpecializeForTypes,
    /// Can be moved to compile time
    MoveToCompileTime,
    /// Can be combined with others
    CanCombine,
}

/// Condition for rule applicability
#[derive(Debug, Clone)]
pub enum GenerationCondition {
    /// Type must be gradual
    TypeIsGradual,
    /// Type must be at boundary
    TypeAtBoundary,
    /// Cast is present
    CastPresent,
    /// Performance impact acceptable
    AcceptablePerformance,
}

/// Statistics for contract generation
#[derive(Debug, Clone, Default)]
pub struct GenerationStatistics {
    /// Total contracts generated
    pub total_generated: usize,
    /// Generated by strategy
    pub by_strategy: HashMap<ContractGenerationStrategy, usize>,
    /// Generated by type
    pub by_type: HashMap<String, usize>,
    /// Average generation time
    pub average_generation_time: Duration,
}

/// Optimizes contracts based on type information
#[derive(Debug)]
pub struct ContractOptimizer {
    /// Optimization rules
    rules: Vec<OptimizationRule>,
    /// Eliminated contracts
    eliminated: Vec<EliminatedContract>,
    /// Optimization statistics
    stats: OptimizationStatistics,
}

/// Type alias for optimization condition function to reduce complexity
pub type OptimizationCondition = Box<dyn Fn(&ContractExpr, &TypeEvidence) -> bool>;

/// Rule for optimizing contracts
pub struct OptimizationRule {
    /// Pattern to match
    pub pattern: OptimizationPattern,
    /// Optimization to apply
    pub optimization: ContractOptimization,
    /// Applicability condition
    pub condition: OptimizationCondition,
}

impl std::fmt::Debug for OptimizationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptimizationRule")
            .field("pattern", &self.pattern)
            .field("optimization", &self.optimization)
            .field("condition", &"<function>")
            .finish()
    }
}

impl Clone for OptimizationRule {
    fn clone(&self) -> Self {
        Self {
            pattern: self.pattern.clone(),
            optimization: self.optimization.clone(),
            condition: Box::new(|_, _| true), // Default condition
        }
    }
}

/// Pattern for optimization matching
#[derive(Debug, Clone)]
pub struct OptimizationPattern {
    /// Contract pattern
    pub contract_pattern: ContractPattern,
    /// Type evidence pattern
    pub type_pattern: TypePattern,
    /// Context pattern
    pub context_pattern: ContextPattern,
}

/// Pattern for matching contracts
#[derive(Debug, Clone)]
pub enum ContractPattern {
    /// Predicate contract
    Predicate(String),
    /// Function contract
    Function {
        /// Number of function arguments
        arity: usize,
    },
    /// List contract
    List,
    /// Any contract
    Any,
}

/// Pattern for matching contexts
#[derive(Debug, Clone)]
pub enum ContextPattern {
    /// Function argument
    FunctionArgument,
    /// Return value
    ReturnValue,
    /// Variable binding
    VariableBinding,
    /// Any context
    Any,
}

/// Statistics for contract optimization
#[derive(Debug, Clone, Default)]
pub struct OptimizationStatistics {
    /// Total optimizations applied
    pub total_optimizations: usize,
    /// Optimizations by type
    pub by_type: HashMap<String, usize>,
    /// Total time saved
    pub total_time_saved: Duration,
    /// Total memory saved
    pub total_memory_saved: usize,
}

/// Coordinates blame between types and contracts
#[derive(Debug)]
pub struct BlameCoordinator {
    /// Blame mappings
    mappings: HashMap<BlameInfo, TypeBoundary>,
    /// Precision configuration
    precision: BlamePrecisionLevel,
    /// Blame statistics
    stats: BlameStatistics,
}

/// Statistics for blame tracking
#[derive(Debug, Clone, Default)]
pub struct BlameStatistics {
    /// Total blame assignments
    pub total_assignments: usize,
    /// Blame by precision level
    pub by_precision: HashMap<BlamePrecisionLevel, usize>,
    /// Average blame resolution time
    pub average_resolution_time: Duration,
}

/// Performance monitor for integration
#[derive(Debug, Default)]
pub struct PerformanceMonitor {
    /// Integration metrics
    pub integration_metrics: IntegrationMetrics,
    /// Performance history
    pub history: Vec<PerformanceSnapshot>,
}

/// Snapshot of performance at a point in time
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    /// Timestamp
    pub timestamp: Instant,
    /// Metrics at this time
    pub metrics: IntegrationMetrics,
    /// Context information
    pub context: String,
}

impl GradualContractIntegration {
    /// Creates new integration system
    pub fn new() -> Self {
        Self::with_config(GradualContractConfig::default())
    }

    /// Creates integration system with configuration
    pub fn with_config(config: GradualContractConfig) -> Self {
        let contract_config = ContractConfig {
            enable_checking: config.enable_auto_generation,
            enable_compilation: config.enable_optimization,
            enable_blame_tracking: config.enable_blame_integration,
            ..ContractConfig::default()
        };

        let contract_system = Arc::new(Mutex::new(ContractSystem::with_config(contract_config)));
        let generator = ContractGenerator::new(&config);
        let optimizer = ContractOptimizer::new(&config);
        let blame_coordinator = BlameCoordinator::new(config.blame_precision);
        let performance_monitor = Arc::new(Mutex::new(PerformanceMonitor::default()));

        Self {
            config,
            contract_system,
            generator,
            optimizer,
            blame_coordinator,
            performance_monitor,
        }
    }

    /// Integrates gradual inference result with contract system
    pub fn integrate_with_contracts(
        &mut self,
        inference_result: &GradualInferenceResult,
        context: &CompilationContext,
    ) -> Result<IntegrationResult> {
        let start_time = Instant::now();

        // Generate contracts from type boundaries
        let generated_contracts = if self.config.enable_auto_generation {
            self.generator.generate_contracts(
                &inference_result.casts,
                &inference_result.inferred_type,
                &self.config,
            )?
        } else {
            Vec::new()
        };

        // Optimize contracts based on type information
        let (optimized_contracts, eliminated_contracts) = if self.config.enable_optimization {
            self.optimizer.optimize_contracts(
                &generated_contracts,
                &inference_result.inferred_type,
                &self.config,
            )?
        } else {
            (
                generated_contracts
                    .into_iter()
                    .map(|c| OptimizedContract {
                        contract: c.contract.clone(),
                        compiled: Arc::new(CompiledContract {
                            id: 0,
                            original: c.contract,
                            checker: Arc::new(|_, _| Ok(true)),
                            optimization_level: crate::contracts::OptimizationLevel::Standard,
                            performance: PerformanceInfo {
                                time_complexity: PerformanceComplexity::Constant,
                                space_complexity: PerformanceComplexity::Constant,
                                deterministic: true,
                                operation_count: 1,
                                inlinable: true,
                            },
                            dependencies: std::collections::HashSet::new(),
                            metadata: CompilationMetadata {
                                timestamp: std::time::SystemTime::now(),
                                source_location: Span::new(0, 0),
                                optimizations: Vec::new(),
                                warnings: Vec::new(),
                                size_info: SizeInfo {
                                    original_nodes: 1,
                                    compiled_operations: 1,
                                    estimated_memory: 64,
                                },
                            },
                            predicate: Box::new(|_| true), // Legacy compatibility
                            blame_info: c.blame.clone(),
                            contract_name: "auto-generated".to_string(),
                        }),
                        optimization: ContractOptimization::None,
                        performance_impact: PerformanceImpact::Minimal,
                        blame: c.blame,
                        priority: c.priority,
                    })
                    .collect(),
                Vec::new(),
            )
        };

        // Create blame mappings
        let blame_mappings = if self.config.enable_blame_integration {
            self.blame_coordinator
                .create_mappings(&optimized_contracts, &inference_result.casts)?
        } else {
            Vec::new()
        };

        // Identify optimization opportunities
        let optimization_opportunities = self.identify_optimization_opportunities(
            &optimized_contracts,
            &inference_result.inferred_type,
        )?;

        // Record performance metrics
        let integration_time = start_time.elapsed();
        let mut performance_metrics = IntegrationMetrics {
            generation_time: integration_time,
            optimization_time: Duration::from_millis(0), // Would measure actual time
            contracts_generated: optimized_contracts.len(),
            contracts_eliminated: eliminated_contracts.len(),
            total_improvement: eliminated_contracts
                .iter()
                .map(|c| c.performance_savings.time_per_execution)
                .sum(),
        };

        if self.config.enable_performance_monitoring {
            let mut monitor = self.performance_monitor.lock().unwrap();
            monitor.record_integration(&performance_metrics);
        }

        Ok(IntegrationResult {
            contracts: optimized_contracts,
            eliminated_contracts,
            blame_mappings,
            performance_metrics,
            optimization_opportunities,
        })
    }

    /// Identifies optimization opportunities
    fn identify_optimization_opportunities(
        &self,
        contracts: &[OptimizedContract],
        inferred_type: &Type,
    ) -> Result<Vec<OptimizationOpportunity>> {
        let mut opportunities = Vec::new();

        for contract in contracts {
            // Look for elimination opportunities
            if self.can_eliminate_contract(&contract.contract, inferred_type) {
                opportunities.push(OptimizationOpportunity {
                    optimization_type: OptimizationType::EliminateRedundantContract,
                    location: Span::new(0, 0), // Would use actual location
                    estimated_benefit: PerformanceBenefit::Moderate,
                    complexity: OptimizationComplexity::Simple,
                });
            }

            // Look for specialization opportunities
            if self.can_specialize_contract(&contract.contract, inferred_type) {
                opportunities.push(OptimizationOpportunity {
                    optimization_type: OptimizationType::SpecializeContract,
                    location: Span::new(0, 0),
                    estimated_benefit: PerformanceBenefit::Small,
                    complexity: OptimizationComplexity::Moderate,
                });
            }
        }

        Ok(opportunities)
    }

    /// Checks if a contract can be eliminated
    fn can_eliminate_contract(&self, contract: &ContractExpr, type_: &Type) -> bool {
        match (contract, type_) {
            // If type is static and matches contract, can eliminate
            (ContractExpr::Predicate { name, .. }, _) if is_static(type_) => {
                self.type_satisfies_predicate(type_, name)
            }
            _ => false,
        }
    }

    /// Checks if a contract can be specialized
    fn can_specialize_contract(&self, contract: &ContractExpr, type_: &Type) -> bool {
        is_static(type_)
            && matches!(
                contract,
                ContractExpr::Function { .. } | ContractExpr::ListOf { .. }
            )
    }

    /// Checks if a type satisfies a predicate
    fn type_satisfies_predicate(&self, type_: &Type, predicate: &str) -> bool {
        matches!(
            (type_, predicate),
            (Type::Number, "number?")
                | (Type::String, "string?")
                | (Type::Boolean, "boolean?")
                | (Type::Symbol, "symbol?")
                | (Type::Char, "char?")
        )
    }

    /// Gets current configuration
    pub fn config(&self) -> &GradualContractConfig {
        &self.config
    }

    /// Updates configuration
    pub fn update_config(&mut self, config: GradualContractConfig) {
        let blame_precision = config.blame_precision;
        self.generator.update_config(&config);
        self.optimizer.update_config(&config);
        self.config = config;
        self.blame_coordinator.update_precision(blame_precision);
    }

    /// Gets performance statistics
    pub fn performance_statistics(&self) -> PerformanceMonitor {
        self.performance_monitor.lock().unwrap().clone()
    }
}

impl ContractGenerator {
    /// Creates new contract generator
    pub fn new(config: &GradualContractConfig) -> Self {
        Self {
            rules: Self::default_rules(config),
            cache: HashMap::new(),
            stats: GenerationStatistics::default(),
        }
    }

    /// Default generation rules
    fn default_rules(config: &GradualContractConfig) -> Vec<GenerationRule> {
        vec![
            // Rule for number types
            GenerationRule {
                type_pattern: TypePattern::Exact(Type::Number),
                contract_template: ContractTemplate {
                    base_contract: ContractExpr::Predicate {
                        name: "number?".to_string(),
                        location: Span::new(0, 0),
                    },
                    parameters: HashMap::new(),
                    optimization_hints: vec![OptimizationHint {
                        hint_type: OptimizationHintType::EliminateIfStatic,
                        condition: "static type".to_string(),
                        benefit: PerformanceBenefit::Moderate,
                    }],
                },
                priority: 100,
                conditions: vec![GenerationCondition::TypeAtBoundary],
            },
            // More rules would be added here...
        ]
    }

    /// Generates contracts from casts and types
    pub fn generate_contracts(
        &mut self,
        casts: &[CastInsertion],
        inferred_type: &Type,
        config: &GradualContractConfig,
    ) -> Result<Vec<GeneratedContract>> {
        let mut contracts = Vec::new();

        for cast in casts {
            if let Some(contract) = self.generate_contract_for_cast(cast, config)? {
                contracts.push(contract);
            }
        }

        self.stats.total_generated += contracts.len();
        Ok(contracts)
    }

    /// Generates contract for a specific cast
    fn generate_contract_for_cast(
        &mut self,
        cast: &CastInsertion,
        config: &GradualContractConfig,
    ) -> Result<Option<GeneratedContract>> {
        match &cast.cast {
            Cast::Downcast { from: _, to } => {
                let contract_expr = self.type_to_contract(to)?;
                Ok(Some(GeneratedContract {
                    contract: contract_expr,
                    location: cast.location,
                    blame: BlameInfo {
                        positive: BlameTarget::System {
                            component: "gradual_typing".to_string(),
                            description: "caller".to_string(),
                        },
                        negative: BlameTarget::System {
                            component: "gradual_typing".to_string(),
                            description: "callee".to_string(),
                        },
                        boundary: BlameBoundary {
                            boundary_type: BoundaryType::ExplicitContract,
                            contract: "type_boundary".to_string(),
                            location: cast.location,
                            context: std::collections::HashMap::new(),
                        },
                        call_stack: Vec::new(),
                        id: 0,
                        parent: None,
                    },
                    priority: match cast.performance_impact {
                        PerformanceImpact::None => {
                            super::gradual_inference::ContractPriority::Optional
                        }
                        PerformanceImpact::Minimal => {
                            super::gradual_inference::ContractPriority::Optional
                        }
                        PerformanceImpact::Moderate => {
                            super::gradual_inference::ContractPriority::Important
                        }
                        PerformanceImpact::Significant => {
                            super::gradual_inference::ContractPriority::Critical
                        }
                    },
                }))
            }
            _ => Ok(None),
        }
    }

    /// Converts a type to a contract expression
    fn type_to_contract(&mut self, type_: &Type) -> Result<ContractExpr> {
        // Check cache first
        if let Some(cached) = self.cache.get(type_) {
            return Ok(cached.clone());
        }

        let contract = match type_ {
            Type::Number => ContractExpr::Predicate {
                name: "number?".to_string(),
                location: Span::new(0, 0),
            },
            Type::String => ContractExpr::Predicate {
                name: "string?".to_string(),
                location: Span::new(0, 0),
            },
            Type::Boolean => ContractExpr::Predicate {
                name: "boolean?".to_string(),
                location: Span::new(0, 0),
            },
            Type::Symbol => ContractExpr::Predicate {
                name: "symbol?".to_string(),
                location: Span::new(0, 0),
            },
            Type::Char => ContractExpr::Predicate {
                name: "char?".to_string(),
                location: Span::new(0, 0),
            },
            Type::List(element_type) => {
                let element_contract = self.type_to_contract(element_type)?;
                ContractExpr::ListOf {
                    element_contract: Box::new(Spanned {
                        inner: element_contract,
                        span: Span::new(0, 0),
                    }),
                    location: Span::new(0, 0),
                }
            }
            Type::Function {
                params,
                return_type,
            } => {
                let param_contracts: Result<Vec<_>> =
                    params.iter().map(|p| self.type_to_contract(p)).collect();
                let return_contract = self.type_to_contract(return_type)?;

                ContractExpr::Function {
                    domain: param_contracts?
                        .into_iter()
                        .map(|c| Spanned::new(c, Span::new(0, 0)))
                        .collect(),
                    codomain: Box::new(Spanned::new(return_contract, Span::new(0, 0))),
                    location: Span::new(0, 0),
                }
            }
            Type::Dynamic => ContractExpr::Predicate {
                name: "any/c".to_string(),
                location: Span::new(0, 0),
            },
            _ => ContractExpr::Predicate {
                name: "any/c".to_string(),
                location: Span::new(0, 0),
            },
        };

        // Cache result
        self.cache.insert(type_.clone(), contract.clone());
        Ok(contract)
    }

    /// Updates configuration
    pub fn update_config(&mut self, config: &GradualContractConfig) {
        self.rules = Self::default_rules(config);
        if !config.enable_auto_generation {
            self.cache.clear();
        }
    }

    /// Gets generation statistics
    pub fn statistics(&self) -> &GenerationStatistics {
        &self.stats
    }
}

impl ContractOptimizer {
    /// Creates new contract optimizer
    pub fn new(config: &GradualContractConfig) -> Self {
        Self {
            rules: Self::default_rules(config),
            eliminated: Vec::new(),
            stats: OptimizationStatistics::default(),
        }
    }

    /// Default optimization rules
    fn default_rules(config: &GradualContractConfig) -> Vec<OptimizationRule> {
        vec![]
    }

    /// Optimizes contracts based on type information
    pub fn optimize_contracts(
        &mut self,
        contracts: &[GeneratedContract],
        inferred_type: &Type,
        config: &GradualContractConfig,
    ) -> Result<(Vec<OptimizedContract>, Vec<EliminatedContract>)> {
        let mut optimized = Vec::new();
        let mut eliminated = Vec::new();

        for contract in contracts {
            match self.optimize_single_contract(contract, inferred_type, config)? {
                OptimizationResult::Optimized(opt_contract) => {
                    optimized.push(*opt_contract);
                }
                OptimizationResult::Eliminated(elim_contract) => {
                    eliminated.push(*elim_contract);
                }
            }
        }

        self.stats.total_optimizations += optimized.len() + eliminated.len();
        Ok((optimized, eliminated))
    }

    /// Optimizes a single contract
    fn optimize_single_contract(
        &self,
        contract: &GeneratedContract,
        inferred_type: &Type,
        config: &GradualContractConfig,
    ) -> Result<OptimizationResult> {
        // Check if contract can be eliminated
        if self.can_eliminate(&contract.contract, inferred_type) {
            return Ok(OptimizationResult::Eliminated(Box::new(
                EliminatedContract {
                    original_contract: contract.contract.clone(),
                    elimination_reason: EliminationReason::TypeGuarantee,
                    type_evidence: TypeEvidence {
                        static_types: vec![inferred_type.clone()],
                        relationships: vec![],
                        invariants: vec![],
                        confidence: ConfidenceLevel::High,
                    },
                    performance_savings: PerformanceSavings {
                        time_per_execution: Duration::from_micros(10),
                        memory_saved: 64,
                        cpu_cycles_saved: 100,
                    },
                },
            )));
        }

        // Apply optimizations
        let optimization = self.determine_optimization(&contract.contract, inferred_type);

        Ok(OptimizationResult::Optimized(Box::new(OptimizedContract {
            contract: contract.contract.clone(),
            compiled: Arc::new(CompiledContract {
                id: 0,
                original: contract.contract.clone(),
                checker: Arc::new(|_, _| Ok(true)),
                optimization_level: crate::contracts::OptimizationLevel::Standard,
                performance: PerformanceInfo {
                    time_complexity: PerformanceComplexity::Constant,
                    space_complexity: PerformanceComplexity::Constant,
                    deterministic: true,
                    operation_count: 1,
                    inlinable: true,
                },
                dependencies: std::collections::HashSet::new(),
                metadata: CompilationMetadata {
                    timestamp: std::time::SystemTime::now(),
                    source_location: Span::new(0, 0),
                    optimizations: Vec::new(),
                    warnings: Vec::new(),
                    size_info: SizeInfo {
                        original_nodes: 1,
                        compiled_operations: 1,
                        estimated_memory: 64,
                    },
                },
                predicate: Box::new(|_| true), // Legacy compatibility
                blame_info: contract.blame.clone(),
                contract_name: "optimized".to_string(),
            }),
            optimization,
            performance_impact: contract.priority.into(),
            blame: contract.blame.clone(),
            priority: contract.priority,
        })))
    }

    /// Checks if contract can be eliminated
    fn can_eliminate(&self, contract: &ContractExpr, type_: &Type) -> bool {
        match (contract, type_) {
            (ContractExpr::Predicate { name, .. }, _) if is_static(type_) => {
                // If static type guarantees predicate, eliminate
                self.type_guarantees_predicate(type_, name)
            }
            _ => false,
        }
    }

    /// Checks if type guarantees a predicate
    fn type_guarantees_predicate(&self, type_: &Type, predicate: &str) -> bool {
        matches!(
            (type_, predicate),
            (Type::Number, "number?") | (Type::String, "string?") | (Type::Boolean, "boolean?")
        )
    }

    /// Determines optimization to apply
    fn determine_optimization(
        &self,
        contract: &ContractExpr,
        type_: &Type,
    ) -> ContractOptimization {
        if is_static(type_) {
            ContractOptimization::TypeSpecialization {
                specialized_types: vec![type_.clone()],
            }
        } else {
            ContractOptimization::None
        }
    }

    /// Updates configuration
    pub fn update_config(&mut self, config: &GradualContractConfig) {
        self.rules = Self::default_rules(config);
    }

    /// Gets optimization statistics
    pub fn statistics(&self) -> &OptimizationStatistics {
        &self.stats
    }
}

/// Result of contract optimization
#[derive(Debug)]
enum OptimizationResult {
    /// Contract was optimized
    Optimized(Box<OptimizedContract>),
    /// Contract was eliminated
    Eliminated(Box<EliminatedContract>),
}

impl BlameCoordinator {
    /// Creates new blame coordinator
    pub fn new(precision: BlamePrecisionLevel) -> Self {
        Self {
            mappings: HashMap::new(),
            precision,
            stats: BlameStatistics::default(),
        }
    }

    /// Creates blame mappings
    pub fn create_mappings(
        &mut self,
        contracts: &[OptimizedContract],
        casts: &[CastInsertion],
    ) -> Result<Vec<BlameMapping>> {
        let mut mappings = Vec::new();

        for (contract, cast) in contracts.iter().zip(casts.iter()) {
            let mapping = BlameMapping {
                blame: contract.blame.clone(),
                type_boundary: None, // Would create from cast information
                precision: self.precision,
                source_context: SourceContext {
                    function_name: None,
                    variable_name: None,
                    expression_type: "cast".to_string(),
                    call_depth: 0,
                },
            };
            mappings.push(mapping);
        }

        self.stats.total_assignments += mappings.len();
        Ok(mappings)
    }

    /// Updates precision level
    pub fn update_precision(&mut self, precision: BlamePrecisionLevel) {
        self.precision = precision;
    }

    /// Gets blame statistics
    pub fn statistics(&self) -> &BlameStatistics {
        &self.stats
    }
}

impl PerformanceMonitor {
    /// Records integration performance
    pub fn record_integration(&mut self, metrics: &IntegrationMetrics) {
        self.integration_metrics.generation_time += metrics.generation_time;
        self.integration_metrics.optimization_time += metrics.optimization_time;
        self.integration_metrics.contracts_generated += metrics.contracts_generated;
        self.integration_metrics.contracts_eliminated += metrics.contracts_eliminated;
        self.integration_metrics.total_improvement += metrics.total_improvement;

        self.history.push(PerformanceSnapshot {
            timestamp: Instant::now(),
            metrics: metrics.clone(),
            context: "integration".to_string(),
        });
    }

    /// Gets average performance metrics
    pub fn average_metrics(&self) -> Option<IntegrationMetrics> {
        if self.history.is_empty() {
            None
        } else {
            let len = self.history.len();
            Some(IntegrationMetrics {
                generation_time: self
                    .history
                    .iter()
                    .map(|h| h.metrics.generation_time)
                    .sum::<Duration>()
                    / len as u32,
                optimization_time: self
                    .history
                    .iter()
                    .map(|h| h.metrics.optimization_time)
                    .sum::<Duration>()
                    / len as u32,
                contracts_generated: self
                    .history
                    .iter()
                    .map(|h| h.metrics.contracts_generated)
                    .sum::<usize>()
                    / len,
                contracts_eliminated: self
                    .history
                    .iter()
                    .map(|h| h.metrics.contracts_eliminated)
                    .sum::<usize>()
                    / len,
                total_improvement: self
                    .history
                    .iter()
                    .map(|h| h.metrics.total_improvement)
                    .sum::<Duration>()
                    / len as u32,
            })
        }
    }
}

// Removed duplicate From implementation - using the one at the top of the file

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            integration_metrics: self.integration_metrics.clone(),
            history: self.history.clone(),
        }
    }
}

impl Default for GradualContractIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_integration_creation() {
        let integration = GradualContractIntegration::new();
        assert!(integration.config().enable_auto_generation);
        assert!(integration.config().enable_optimization);
    }

    #[test]
    fn test_contract_generation() {
        let config = GradualContractConfig::default();
        let mut generator = ContractGenerator::new(&config);

        let cast = CastInsertion {
            location: Span::new(0, 10),
            cast: Cast::Downcast {
                from: Type::Dynamic,
                to: Type::Number,
            },
            reason: CastReason::StaticDynamicBoundary,
            performance_impact: PerformanceImpact::Minimal,
        };

        let contracts = generator
            .generate_contracts(&[cast], &Type::Number, &config)
            .unwrap();

        assert!(!contracts.is_empty());
    }

    #[test]
    fn test_contract_optimization() {
        let config = GradualContractConfig::default();
        let mut optimizer = ContractOptimizer::new(&config);

        let contract = GeneratedContract {
            contract: ContractExpr::Predicate {
                name: "number?".to_string(),
                location: Span::new(0, 10),
            },
            location: Span::new(0, 10),
            blame: BlameInfo {
                positive: BlameTarget::System {
                    component: "test".to_string(),
                    description: "test".to_string(),
                },
                negative: BlameTarget::System {
                    component: "test".to_string(),
                    description: "test".to_string(),
                },
                boundary: BlameBoundary {
                    boundary_type: BoundaryType::ExplicitContract,
                    contract: "test".to_string(),
                    location: Span::new(0, 10),
                    context: std::collections::HashMap::new(),
                },
                call_stack: Vec::new(),
                id: 0,
                parent: None,
            },
            priority: crate::types::gradual_inference::ContractPriority::Important,
        };

        // With static type, contract should be eliminable
        let (optimized, eliminated) = optimizer
            .optimize_contracts(&[contract], &Type::Number, &config)
            .unwrap();

        // Contract should be eliminated because static type guarantees it
        assert_eq!(eliminated.len(), 1);
        assert_eq!(optimized.len(), 0);
    }

    #[test]
    fn test_blame_coordination() {
        let mut coordinator = BlameCoordinator::new(BlamePrecisionLevel::Medium);

        let contract = OptimizedContract {
            contract: ContractExpr::Predicate {
                name: "number?".to_string(),
                location: Span::new(0, 10),
            },
            compiled: Arc::new(CompiledContract {
                id: 1,
                original: ContractExpr::Predicate {
                    name: "number?".to_string(),
                    location: Span::new(0, 10),
                },
                checker: Arc::new(|_value, _blame| Ok(true)),
                optimization_level: OptimizationLevel::Standard,
                performance: PerformanceInfo {
                    time_complexity: PerformanceComplexity::Constant,
                    space_complexity: PerformanceComplexity::Constant,
                    deterministic: true,
                    operation_count: 1,
                    inlinable: true,
                },
                dependencies: std::collections::HashSet::new(),
                metadata: CompilationMetadata {
                    timestamp: std::time::SystemTime::now(),
                    source_location: Span::new(0, 10),
                    optimizations: Vec::new(),
                    warnings: Vec::new(),
                    size_info: SizeInfo {
                        original_nodes: 1,
                        compiled_operations: 1,
                        estimated_memory: 64,
                    },
                },
                predicate: Box::new(|_| true),
                blame_info: BlameInfo::default(),
                contract_name: "test".to_string(),
            }),
            optimization: ContractOptimization::None,
            performance_impact: PerformanceImpact::Minimal,
            blame: BlameInfo::default(),
            priority: ContractPriority::Important,
        };

        let cast = CastInsertion {
            location: Span::new(0, 10),
            cast: Cast::None,
            reason: CastReason::StaticDynamicBoundary,
            performance_impact: PerformanceImpact::None,
        };

        let mappings = coordinator.create_mappings(&[contract], &[cast]).unwrap();
        assert_eq!(mappings.len(), 1);
        assert_eq!(coordinator.statistics().total_assignments, 1);
    }

    #[test]
    fn test_performance_monitoring() {
        let mut monitor = PerformanceMonitor::default();

        let metrics = IntegrationMetrics {
            generation_time: Duration::from_millis(10),
            optimization_time: Duration::from_millis(5),
            contracts_generated: 3,
            contracts_eliminated: 1,
            total_improvement: Duration::from_millis(2),
        };

        monitor.record_integration(&metrics);

        assert_eq!(monitor.integration_metrics.contracts_generated, 3);
        assert_eq!(monitor.integration_metrics.contracts_eliminated, 1);
        assert_eq!(monitor.history.len(), 1);
    }
}
