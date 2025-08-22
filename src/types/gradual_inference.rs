//! Gradual Type Inference System for Lambdust
//!
//! This module implements a comprehensive gradual type inference system that integrates
//! static type inference with dynamic type checking. It provides:
//!
//! - Automatic type inference with gradual boundaries
//! - Integration with contract system for runtime checking
//! - Blame tracking for precise error attribution
//! - Type-directed optimization opportunities
//! - Seamless migration between typed and untyped code
//!
//! # Architecture
//!
//! The gradual type inference system extends Hindley-Milner with:
//! - Consistency relations for gradual typing
//! - Boundary management for static/dynamic transitions
//! - Cast insertion for type coercions
//! - Contract integration for runtime safety
//!
//! # Example Usage
//!
//! ```scheme
//! ;; Automatic inference
//! (define (map f lst)  ; Inferred: ∀ α β. (α → β) → List α → List β
//!   (if (null? lst)
//!       '()
//!       (cons (f (car lst)) (map f (cdr lst)))))
//!
//! ;; Gradual boundaries
//! (define typed-map : (∀ α β. (α → β) → List α → List β)
//!   (lambda (f lst) (map f lst)))  ; Contract inserted automatically
//! ```

use super::gradual::{
    Cast, approximate_type, consistent, gradualize, insert_cast, is_gradual, is_static, join_types,
    meet_types, staticize,
};
use super::inference::InferenceResult;
use super::{
    ConstraintSolver, Substitution, Type, TypeConstraint, TypeEnv, TypeInference, TypeScheme,
    TypeVar,
};
use crate::ast::{Expr, Formals, Literal};
use crate::contracts::{
    BlameInfo, BlameTracker, CompilationContext, CompiledContract, ContractError, ContractExpr,
    ContractSystem,
};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::Value;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

/// Configuration for gradual type inference
#[derive(Debug, Clone)]
pub struct GradualInferenceConfig {
    /// Enable automatic type inference
    pub enable_inference: bool,
    /// Enable contract generation for gradual boundaries
    pub enable_contract_generation: bool,
    /// Enable blame tracking for gradual violations
    pub enable_blame_tracking: bool,
    /// Inference timeout in milliseconds
    pub inference_timeout_ms: u64,
    /// Maximum inference iterations
    pub max_inference_iterations: usize,
    /// Enable type-directed optimizations
    pub enable_optimizations: bool,
    /// Strictness level for gradual consistency
    pub consistency_strictness: ConsistencyStrictness,
    /// Enable migration assistance
    pub enable_migration_assistance: bool,
}

/// Strictness levels for gradual consistency checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsistencyStrictness {
    /// Permissive: Allow most dynamic transitions
    Permissive,
    /// Balanced: Standard gradual typing consistency
    Balanced,
    /// Strict: Require explicit annotations for boundaries
    Strict,
}

impl Default for GradualInferenceConfig {
    fn default() -> Self {
        Self {
            enable_inference: true,
            enable_contract_generation: true,
            enable_blame_tracking: true,
            inference_timeout_ms: 5000,
            max_inference_iterations: 1000,
            enable_optimizations: true,
            consistency_strictness: ConsistencyStrictness::Balanced,
            enable_migration_assistance: true,
        }
    }
}

/// Result of gradual type inference
#[derive(Debug, Clone)]
pub struct GradualInferenceResult {
    /// Inferred type (may be gradual)
    pub inferred_type: Type,
    /// Substitution applied
    pub substitution: Substitution,
    /// Generated casts for boundaries
    pub casts: Vec<CastInsertion>,
    /// Generated contracts for runtime checking
    pub contracts: Vec<GeneratedContract>,
    /// Blame tracking information
    pub blame_info: Vec<BlameInfo>,
    /// Optimization opportunities
    pub optimizations: Vec<OptimizationHint>,
    /// Migration suggestions
    pub migration_suggestions: Vec<MigrationSuggestion>,
}

/// Information about a cast insertion
#[derive(Debug, Clone)]
pub struct CastInsertion {
    /// Location where cast is needed
    pub location: Span,
    /// Type of cast
    pub cast: Cast,
    /// Reason for cast
    pub reason: CastReason,
    /// Performance impact
    pub performance_impact: PerformanceImpact,
}

/// Reason for cast insertion
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CastReason {
    /// Boundary between static and dynamic code
    StaticDynamicBoundary,
    /// Function argument type mismatch
    ArgumentTypeMismatch,
    /// Return type coercion
    ReturnTypeCoercion,
    /// Container element type change
    ContainerElementType,
    /// Explicit type annotation
    ExplicitAnnotation,
}

/// Performance impact of a cast
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PerformanceImpact {
    /// No runtime overhead
    None,
    /// Minimal overhead (simple check)
    Minimal,
    /// Moderate overhead (structural check)
    Moderate,
    /// Significant overhead (deep check)
    Significant,
}

/// Generated contract for runtime checking
#[derive(Debug, Clone)]
pub struct GeneratedContract {
    /// Contract expression
    pub contract: ContractExpr,
    /// Location where contract applies
    pub location: Span,
    /// Associated blame information
    pub blame: BlameInfo,
    /// Contract priority
    pub priority: ContractPriority,
}

/// Priority level for contract checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContractPriority {
    /// Critical safety contract (always checked)
    Critical,
    /// Important contract (checked in debug mode)
    Important,
    /// Optional contract (checked when enabled)
    Optional,
}

/// Optimization hint from type inference
#[derive(Debug, Clone)]
pub struct OptimizationHint {
    /// Location of optimization opportunity
    pub location: Span,
    /// Type of optimization
    pub optimization: OptimizationType,
    /// Estimated performance benefit
    pub benefit: PerformanceBenefit,
}

/// Types of optimizations available
#[derive(Debug, Clone)]
pub enum OptimizationType {
    /// Specialize function for known types
    FunctionSpecialization {
        /// Types of the function arguments for specialization
        argument_types: Vec<Type>,
    },
    /// Remove unnecessary casts
    CastElimination,
    /// Inline function call
    FunctionInlining,
    /// Use specialized arithmetic
    ArithmeticSpecialization,
    /// Optimize container operations
    ContainerOptimization,
}

/// Estimated performance benefit
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PerformanceBenefit {
    /// Minor performance improvement
    Minor,
    /// Moderate performance improvement
    Moderate,
    /// Significant performance improvement
    Significant,
    /// Major performance improvement
    Major,
}

/// Migration suggestion for improving type safety
#[derive(Debug, Clone)]
pub struct MigrationSuggestion {
    /// Location needing improvement
    pub location: Span,
    /// Type of suggestion
    pub suggestion: MigrationType,
    /// Benefit of applying suggestion
    pub benefit: MigrationBenefit,
    /// Difficulty of applying suggestion
    pub difficulty: MigrationDifficulty,
}

/// Types of migration suggestions
#[derive(Debug, Clone)]
pub enum MigrationType {
    /// Add type annotation
    AddTypeAnnotation {
        /// The suggested type to annotate with
        suggested_type: Type,
    },
    /// Add contract
    AddContract {
        /// The suggested contract expression to add
        suggested_contract: ContractExpr,
    },
    /// Refactor for better types
    RefactorForTypes {
        /// Human-readable refactoring suggestion
        suggestion: String,
    },
    /// Extract typed function
    ExtractTypedFunction {
        /// Suggested name for the extracted function
        function_name: String,
    },
    /// Use more specific type
    UseMoreSpecificType {
        /// Current (less specific) type
        current: Type,
        /// Suggested (more specific) type
        suggested: Type,
    },
}

/// Benefit of applying migration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MigrationBenefit {
    /// Improved error messages
    ImprovedErrors,
    /// Better performance
    BetterPerformance,
    /// Enhanced safety
    EnhancedSafety,
    /// Better maintainability
    BetterMaintainability,
}

/// Difficulty of applying migration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MigrationDifficulty {
    /// Easy change
    Easy,
    /// Moderate effort required
    Moderate,
    /// Significant refactoring needed
    Significant,
    /// Major architectural change
    Major,
}

/// Type boundary information
#[derive(Debug, Clone)]
pub struct TypeBoundary {
    /// Location of boundary
    pub location: Span,
    /// Source type context
    pub source_context: TypeContext,
    /// Target type context
    pub target_context: TypeContext,
    /// Required cast
    pub cast: Cast,
    /// Blame assignment
    pub blame: BlameInfo,
}

/// Type context for boundary analysis
#[derive(Debug, Clone)]
pub enum TypeContext {
    /// Static type context
    Static {
        /// The static type information
        type_: Type,
        /// Certainty level of the type information
        certainty: TypeCertainty,
    },
    /// Dynamic type context
    Dynamic {
        /// Optional runtime type information if available
        runtime_type: Option<Type>,
    },
    /// Gradual type context
    Gradual {
        /// Static (compile-time known) part of the type
        static_part: Type,
        /// Dynamic (runtime-dependent) part of the type
        dynamic_part: Type,
    },
}

/// Certainty level of static type information
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeCertainty {
    /// Type is explicitly annotated
    Explicit,
    /// Type is inferred with high confidence
    Inferred,
    /// Type is approximated
    Approximated,
    /// Type is uncertain
    Uncertain,
}

/// Main gradual type inference engine
#[derive(Debug)]
pub struct GradualTypeInference {
    /// Configuration
    config: GradualInferenceConfig,
    /// Underlying static type inference
    static_inference: TypeInference,
    /// Contract system integration
    contract_system: ContractSystem,
    /// Blame tracker
    blame_tracker: BlameTracker,
    /// Type boundary manager
    boundary_manager: TypeBoundaryManager,
    /// Cast optimizer
    cast_optimizer: CastOptimizer,
    /// Performance profiler
    profiler: PerformanceProfiler,
}

/// Manages type boundaries and transitions
#[derive(Debug)]
pub struct TypeBoundaryManager {
    /// Active type boundaries
    boundaries: HashMap<Span, TypeBoundary>,
    /// Boundary statistics
    stats: BoundaryStatistics,
}

/// Statistics about type boundaries
#[derive(Debug, Clone, Default)]
pub struct BoundaryStatistics {
    /// Total boundaries detected
    pub total_boundaries: usize,
    /// Static to dynamic transitions
    pub static_to_dynamic: usize,
    /// Dynamic to static transitions
    pub dynamic_to_static: usize,
    /// Gradual transitions
    pub gradual_transitions: usize,
    /// Cast insertions
    pub cast_insertions: usize,
    /// Contract generations
    pub contract_generations: usize,
}

/// Optimizes cast insertions
#[derive(Debug)]
pub struct CastOptimizer {
    /// Cast elimination rules
    elimination_rules: Vec<CastEliminationRule>,
    /// Cast combination rules
    combination_rules: Vec<CastCombinationRule>,
}

/// Rule for eliminating unnecessary casts
#[derive(Debug, Clone)]
pub struct CastEliminationRule {
    /// Pattern to match
    pub pattern: CastPattern,
    /// Action to take
    pub action: EliminationAction,
}

/// Pattern for cast elimination
#[derive(Debug, Clone)]
pub enum CastPattern {
    /// Identity cast (T -> T)
    Identity,
    /// Double cast that can be simplified
    DoubleCast {
        /// The intermediate type in the double cast chain
        intermediate: Type,
    },
    /// Cast followed by compatible operation
    CastWithOperation {
        /// Name of the operation that follows the cast
        operation: String,
    },
}

/// Action for cast elimination
#[derive(Debug, Clone, PartialEq)]
pub enum EliminationAction {
    /// Remove the cast entirely
    Remove,
    /// Replace with simpler cast
    Simplify {
        /// The simplified cast to use instead
        new_cast: Cast,
    },
    /// Defer to later optimization phase
    Defer,
}

/// Rule for combining casts
#[derive(Debug, Clone)]
pub struct CastCombinationRule {
    /// First cast pattern
    pub first: Cast,
    /// Second cast pattern
    pub second: Cast,
    /// Combined result
    pub result: Cast,
}

/// Performance profiler for gradual typing
#[derive(Debug)]
pub struct PerformanceProfiler {
    /// Inference timings
    inference_timings: HashMap<String, Duration>,
    /// Cast overhead measurements
    cast_overhead: HashMap<CastReason, Duration>,
    /// Contract checking overhead
    contract_overhead: HashMap<ContractPriority, Duration>,
}

impl GradualTypeInference {
    /// Creates a new gradual type inference engine
    pub fn new() -> Self {
        Self::with_config(GradualInferenceConfig::default())
    }

    /// Creates a new gradual type inference engine with configuration
    pub fn with_config(config: GradualInferenceConfig) -> Self {
        let static_inference = TypeInference::new();
        let contract_system = ContractSystem::new();
        let blame_tracker = BlameTracker::new();
        let boundary_manager = TypeBoundaryManager::new();
        let cast_optimizer = CastOptimizer::new();
        let profiler = PerformanceProfiler::new();

        Self {
            config,
            static_inference,
            contract_system,
            blame_tracker,
            boundary_manager,
            cast_optimizer,
            profiler,
        }
    }

    /// Performs gradual type inference on an expression
    pub fn infer_gradual(&mut self, expr: &Spanned<Expr>) -> Result<GradualInferenceResult> {
        let start_time = std::time::Instant::now();

        // Start with static type inference
        let static_result = self.static_inference.infer(expr)?;

        // Analyze gradual boundaries
        let boundaries = self.analyze_boundaries(expr, &static_result.type_)?;

        // Insert necessary casts
        let casts = self.insert_casts(&boundaries)?;

        // Generate contracts for runtime checking
        let contracts = self.generate_contracts(&boundaries, &casts)?;

        // Extract blame information
        let blame_info = self.extract_blame_info(&boundaries);

        // Generate optimization hints
        let optimizations = self.generate_optimizations(expr, &static_result.type_)?;

        // Generate migration suggestions
        let migration_suggestions = if self.config.enable_migration_assistance {
            self.generate_migration_suggestions(expr, &static_result.type_)?
        } else {
            Vec::new()
        };

        // Record performance metrics
        let inference_time = start_time.elapsed();
        self.profiler
            .record_inference_time("gradual_infer", inference_time);

        Ok(GradualInferenceResult {
            inferred_type: static_result.type_,
            substitution: static_result.substitution,
            casts,
            contracts,
            blame_info,
            optimizations,
            migration_suggestions,
        })
    }

    /// Analyzes type boundaries in an expression
    fn analyze_boundaries(
        &mut self,
        expr: &Spanned<Expr>,
        inferred_type: &Type,
    ) -> Result<Vec<TypeBoundary>> {
        let mut boundaries = Vec::new();

        // Walk the expression tree looking for boundaries
        self.walk_expression_for_boundaries(expr, inferred_type, &mut boundaries)?;

        // Update boundary manager statistics
        self.boundary_manager.update_statistics(&boundaries);

        Ok(boundaries)
    }

    /// Walks expression tree to identify type boundaries
    fn walk_expression_for_boundaries(
        &mut self,
        expr: &Spanned<Expr>,
        context_type: &Type,
        boundaries: &mut Vec<TypeBoundary>,
    ) -> Result<()> {
        match &expr.inner {
            Expr::Application { operator, operands } => {
                // Check for function application boundaries
                let operator_result = self.static_inference.infer(operator)?;

                if let Type::Function {
                    params,
                    return_type: _,
                } = &operator_result.type_
                {
                    for (operand, param_type) in operands.iter().zip(params.iter()) {
                        let operand_result = self.static_inference.infer(operand)?;

                        if !consistent(&operand_result.type_, param_type) {
                            // Found a boundary
                            let boundary = self.create_boundary(
                                operand.span,
                                &operand_result.type_,
                                param_type,
                            )?;
                            boundaries.push(boundary);
                        }
                    }
                }
            }

            Expr::If {
                test,
                consequent,
                alternative,
            } => {
                // Check consistency between branches
                let consequent_result = self.static_inference.infer(consequent)?;

                if let Some(alt) = alternative {
                    let alternative_result = self.static_inference.infer(alt)?;

                    if !consistent(&consequent_result.type_, &alternative_result.type_) {
                        // Need boundary for branch consistency
                        let boundary = self.create_boundary(
                            expr.span,
                            &consequent_result.type_,
                            &alternative_result.type_,
                        )?;
                        boundaries.push(boundary);
                    }
                }

                // Recursively check branches
                self.walk_expression_for_boundaries(consequent, context_type, boundaries)?;
                if let Some(alt) = alternative {
                    self.walk_expression_for_boundaries(alt, context_type, boundaries)?;
                }
            }

            Expr::TypeAnnotation {
                expr: inner_expr,
                type_expr: _,
            } => {
                // Type annotations create explicit boundaries
                let inner_result = self.static_inference.infer(inner_expr)?;

                if !consistent(&inner_result.type_, context_type) {
                    let boundary =
                        self.create_boundary(expr.span, &inner_result.type_, context_type)?;
                    boundaries.push(boundary);
                }
            }

            // Handle other expression types...
            _ => {
                // Default: no boundaries detected for this expression type
            }
        }

        Ok(())
    }

    /// Creates a type boundary
    fn create_boundary(
        &mut self,
        location: Span,
        source_type: &Type,
        target_type: &Type,
    ) -> Result<TypeBoundary> {
        let source_context = if is_static(source_type) {
            TypeContext::Static {
                type_: source_type.clone(),
                certainty: TypeCertainty::Inferred,
            }
        } else if is_gradual(source_type) {
            TypeContext::Gradual {
                static_part: staticize(source_type, &mut || TypeVar::fresh()),
                dynamic_part: gradualize(source_type),
            }
        } else {
            TypeContext::Dynamic { runtime_type: None }
        };

        let target_context = if is_static(target_type) {
            TypeContext::Static {
                type_: target_type.clone(),
                certainty: TypeCertainty::Inferred,
            }
        } else if is_gradual(target_type) {
            TypeContext::Gradual {
                static_part: staticize(target_type, &mut || TypeVar::fresh()),
                dynamic_part: gradualize(target_type),
            }
        } else {
            TypeContext::Dynamic { runtime_type: None }
        };

        let cast = insert_cast(source_type, target_type);
        // Create blame context with proper types
        use crate::contracts::blame::{BlameBoundary, BlameTarget, BoundaryType, CallFrame};
        use std::collections::HashMap;

        let positive_target = BlameTarget::System {
            component: "gradual_inference".to_string(),
            description: "source type".to_string(),
        };

        let negative_target = BlameTarget::System {
            component: "gradual_inference".to_string(),
            description: "target type".to_string(),
        };

        let boundary = BlameBoundary {
            boundary_type: BoundaryType::ExplicitContract,
            contract: "cast".to_string(),
            location,
            context: HashMap::new(),
        };

        let call_stack = vec![];

        let blame = self.blame_tracker.create_blame_context(
            positive_target,
            negative_target,
            boundary,
            call_stack,
        );

        Ok(TypeBoundary {
            location,
            source_context,
            target_context,
            cast,
            blame,
        })
    }

    /// Inserts casts based on boundaries
    fn insert_casts(&mut self, boundaries: &[TypeBoundary]) -> Result<Vec<CastInsertion>> {
        let mut casts = Vec::new();

        for boundary in boundaries {
            let cast_insertion = CastInsertion {
                location: boundary.location,
                cast: boundary.cast.clone(),
                reason: self.determine_cast_reason(boundary),
                performance_impact: self.analyze_performance_impact(&boundary.cast),
            };
            casts.push(cast_insertion);
        }

        // Optimize casts
        if self.config.enable_optimizations {
            self.cast_optimizer.optimize_casts(&mut casts)?;
        }

        Ok(casts)
    }

    /// Determines the reason for a cast
    fn determine_cast_reason(&self, boundary: &TypeBoundary) -> CastReason {
        match (&boundary.source_context, &boundary.target_context) {
            (TypeContext::Static { .. }, TypeContext::Dynamic { .. }) => {
                CastReason::StaticDynamicBoundary
            }
            (TypeContext::Dynamic { .. }, TypeContext::Static { .. }) => {
                CastReason::StaticDynamicBoundary
            }
            _ => CastReason::ArgumentTypeMismatch,
        }
    }

    /// Analyzes performance impact of a cast
    fn analyze_performance_impact(&self, cast: &Cast) -> PerformanceImpact {
        Self::analyze_performance_impact_recursive(cast)
    }

    /// Recursive helper for performance impact analysis (optimized without self parameter)
    fn analyze_performance_impact_recursive(cast: &Cast) -> PerformanceImpact {
        match cast {
            Cast::None => PerformanceImpact::None,
            Cast::Upcast { .. } => PerformanceImpact::Minimal,
            Cast::Downcast { .. } => PerformanceImpact::Moderate,
            Cast::Structural { casts } => {
                let max_impact = casts
                    .iter()
                    .map(Self::analyze_performance_impact_recursive)
                    .max()
                    .unwrap_or(PerformanceImpact::None);

                match max_impact {
                    PerformanceImpact::None => PerformanceImpact::Minimal,
                    PerformanceImpact::Minimal => PerformanceImpact::Moderate,
                    _ => PerformanceImpact::Significant,
                }
            }
        }
    }

    /// Generates contracts for runtime checking
    fn generate_contracts(
        &mut self,
        boundaries: &[TypeBoundary],
        casts: &[CastInsertion],
    ) -> Result<Vec<GeneratedContract>> {
        if !self.config.enable_contract_generation {
            return Ok(Vec::new());
        }

        let mut contracts = Vec::new();

        for (boundary, cast_insertion) in boundaries.iter().zip(casts.iter()) {
            if matches!(cast_insertion.cast, Cast::Downcast { .. }) {
                // Generate contract for downcast verification
                let contract = self.generate_downcast_contract(boundary)?;
                contracts.push(contract);
            }
        }

        Ok(contracts)
    }

    /// Generates a contract for downcast verification
    fn generate_downcast_contract(&mut self, boundary: &TypeBoundary) -> Result<GeneratedContract> {
        // Create a contract expression based on the target type
        let contract = match &boundary.target_context {
            TypeContext::Static { type_, .. } => self.type_to_contract_expr(type_)?,
            _ => {
                // Default to any/c for dynamic contexts
                ContractExpr::Predicate {
                    name: "any/c".to_string(),
                    location: boundary.location,
                }
            }
        };

        Ok(GeneratedContract {
            contract,
            location: boundary.location,
            blame: boundary.blame.clone(),
            priority: ContractPriority::Important,
        })
    }

    /// Converts a type to a contract expression
    fn type_to_contract_expr(&self, type_: &Type) -> Result<ContractExpr> {
        Self::type_to_contract_expr_recursive(type_)
    }

    /// Recursive helper for type to contract conversion (optimized without self parameter)
    fn type_to_contract_expr_recursive(type_: &Type) -> Result<ContractExpr> {
        match type_ {
            Type::Number => Ok(ContractExpr::Predicate {
                name: "number?".to_string(),
                location: Span::new(0, 0),
            }),
            Type::String => Ok(ContractExpr::Predicate {
                name: "string?".to_string(),
                location: Span::new(0, 0),
            }),
            Type::Boolean => Ok(ContractExpr::Predicate {
                name: "boolean?".to_string(),
                location: Span::new(0, 0),
            }),
            Type::Symbol => Ok(ContractExpr::Predicate {
                name: "symbol?".to_string(),
                location: Span::new(0, 0),
            }),
            Type::Char => Ok(ContractExpr::Predicate {
                name: "char?".to_string(),
                location: Span::new(0, 0),
            }),
            Type::List(element_type) => {
                let element_contract = Self::type_to_contract_expr_recursive(element_type)?;
                Ok(ContractExpr::ListOf {
                    element_contract: Box::new(Spanned {
                        inner: element_contract,
                        span: Span::new(0, 0),
                    }),
                    location: Span::new(0, 0),
                })
            }
            Type::Function {
                params,
                return_type,
            } => {
                let param_contracts: Result<Vec<_>> = params
                    .iter()
                    .map(|p| {
                        Self::type_to_contract_expr_recursive(p).map(|c| Spanned {
                            inner: c,
                            span: Span::new(0, 0),
                        })
                    })
                    .collect();
                let return_contract = Self::type_to_contract_expr_recursive(return_type)?;

                Ok(ContractExpr::Function {
                    domain: param_contracts?,
                    codomain: Box::new(Spanned {
                        inner: return_contract,
                        span: Span::new(0, 0),
                    }),
                    location: Span::new(0, 0),
                })
            }
            Type::Dynamic => Ok(ContractExpr::Predicate {
                name: "any/c".to_string(),
                location: Span::new(0, 0),
            }),
            _ => {
                // For other types, use a generic contract
                Ok(ContractExpr::Predicate {
                    name: "any/c".to_string(),
                    location: Span::new(0, 0),
                })
            }
        }
    }

    /// Extracts blame information from boundaries
    fn extract_blame_info(&self, boundaries: &[TypeBoundary]) -> Vec<BlameInfo> {
        boundaries.iter().map(|b| b.blame.clone()).collect()
    }

    /// Generates optimization hints
    fn generate_optimizations(
        &mut self,
        expr: &Spanned<Expr>,
        inferred_type: &Type,
    ) -> Result<Vec<OptimizationHint>> {
        if !self.config.enable_optimizations {
            return Ok(Vec::new());
        }

        let mut optimizations = Vec::new();

        // Look for function specialization opportunities
        if let Type::Function { params, .. } = inferred_type {
            if params.iter().all(is_static) {
                optimizations.push(OptimizationHint {
                    location: expr.span,
                    optimization: OptimizationType::FunctionSpecialization {
                        argument_types: params.clone(),
                    },
                    benefit: PerformanceBenefit::Significant,
                });
            }
        }

        // Look for cast elimination opportunities
        // This would be implemented based on the expression structure

        Ok(optimizations)
    }

    /// Generates migration suggestions
    fn generate_migration_suggestions(
        &mut self,
        expr: &Spanned<Expr>,
        inferred_type: &Type,
    ) -> Result<Vec<MigrationSuggestion>> {
        let mut suggestions = Vec::new();

        // Suggest type annotations for dynamic types
        if matches!(inferred_type, Type::Dynamic) {
            suggestions.push(MigrationSuggestion {
                location: expr.span,
                suggestion: MigrationType::AddTypeAnnotation {
                    suggested_type: approximate_type(inferred_type),
                },
                benefit: MigrationBenefit::ImprovedErrors,
                difficulty: MigrationDifficulty::Easy,
            });
        }

        // Suggest more specific types for gradual types
        if is_gradual(inferred_type) && !is_static(inferred_type) {
            let static_type = staticize(inferred_type, &mut || TypeVar::fresh());
            suggestions.push(MigrationSuggestion {
                location: expr.span,
                suggestion: MigrationType::UseMoreSpecificType {
                    current: inferred_type.clone(),
                    suggested: static_type,
                },
                benefit: MigrationBenefit::BetterPerformance,
                difficulty: MigrationDifficulty::Moderate,
            });
        }

        Ok(suggestions)
    }

    /// Gets the current configuration
    pub fn config(&self) -> &GradualInferenceConfig {
        &self.config
    }

    /// Updates the configuration
    pub fn update_config(&mut self, config: GradualInferenceConfig) {
        self.config = config;
    }

    /// Gets boundary statistics
    pub fn boundary_statistics(&self) -> &BoundaryStatistics {
        &self.boundary_manager.stats
    }

    /// Gets performance metrics
    pub fn performance_metrics(&self) -> &PerformanceProfiler {
        &self.profiler
    }
}

impl TypeBoundaryManager {
    /// Creates a new type boundary manager
    pub fn new() -> Self {
        Self {
            boundaries: HashMap::new(),
            stats: BoundaryStatistics::default(),
        }
    }

    /// Updates statistics based on boundaries
    pub fn update_statistics(&mut self, boundaries: &[TypeBoundary]) {
        self.stats.total_boundaries += boundaries.len();

        for boundary in boundaries {
            match (&boundary.source_context, &boundary.target_context) {
                (TypeContext::Static { .. }, TypeContext::Dynamic { .. }) => {
                    self.stats.static_to_dynamic += 1;
                }
                (TypeContext::Dynamic { .. }, TypeContext::Static { .. }) => {
                    self.stats.dynamic_to_static += 1;
                }
                (TypeContext::Gradual { .. }, _) | (_, TypeContext::Gradual { .. }) => {
                    self.stats.gradual_transitions += 1;
                }
                _ => {}
            }
        }
    }
}

impl CastOptimizer {
    /// Creates a new cast optimizer
    pub fn new() -> Self {
        Self {
            elimination_rules: Self::default_elimination_rules(),
            combination_rules: Self::default_combination_rules(),
        }
    }

    /// Default cast elimination rules
    fn default_elimination_rules() -> Vec<CastEliminationRule> {
        vec![CastEliminationRule {
            pattern: CastPattern::Identity,
            action: EliminationAction::Remove,
        }]
    }

    /// Default cast combination rules
    fn default_combination_rules() -> Vec<CastCombinationRule> {
        vec![]
    }

    /// Optimizes a list of cast insertions
    pub fn optimize_casts(&self, casts: &mut Vec<CastInsertion>) -> Result<()> {
        // Apply elimination rules
        casts.retain(|cast| !self.should_eliminate(cast));

        // Apply combination rules
        self.combine_casts(casts)?;

        Ok(())
    }

    /// Checks if a cast should be eliminated
    fn should_eliminate(&self, cast: &CastInsertion) -> bool {
        for rule in &self.elimination_rules {
            if self.matches_pattern(&rule.pattern, &cast.cast) {
                return rule.action == EliminationAction::Remove;
            }
        }
        false
    }

    /// Checks if a cast matches a pattern
    fn matches_pattern(&self, pattern: &CastPattern, cast: &Cast) -> bool {
        matches!((pattern, cast), (CastPattern::Identity, Cast::None))
    }

    /// Combines adjacent casts
    fn combine_casts(&self, _casts: &mut [CastInsertion]) -> Result<()> {
        // Implementation would combine compatible casts
        Ok(())
    }
}

impl PerformanceProfiler {
    /// Creates a new performance profiler
    pub fn new() -> Self {
        Self {
            inference_timings: HashMap::new(),
            cast_overhead: HashMap::new(),
            contract_overhead: HashMap::new(),
        }
    }

    /// Records inference timing
    pub fn record_inference_time(&mut self, operation: &str, duration: Duration) {
        self.inference_timings
            .insert(operation.to_string(), duration);
    }

    /// Records cast overhead
    pub fn record_cast_overhead(&mut self, reason: CastReason, duration: Duration) {
        self.cast_overhead.insert(reason, duration);
    }

    /// Records contract overhead
    pub fn record_contract_overhead(&mut self, priority: ContractPriority, duration: Duration) {
        self.contract_overhead.insert(priority, duration);
    }

    /// Gets average inference time
    pub fn average_inference_time(&self) -> Option<Duration> {
        if self.inference_timings.is_empty() {
            None
        } else {
            let total: Duration = self.inference_timings.values().sum();
            Some(total / self.inference_timings.len() as u32)
        }
    }
}

impl Default for GradualTypeInference {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TypeBoundaryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CastOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;
    use crate::diagnostics::{Span, spanned};

    #[test]
    fn test_gradual_inference_creation() {
        let inference = GradualTypeInference::new();
        assert!(inference.config().enable_inference);
        assert!(inference.config().enable_contract_generation);
    }

    #[test]
    fn test_gradual_inference_with_config() {
        let config = GradualInferenceConfig {
            enable_inference: true,
            enable_contract_generation: false,
            enable_blame_tracking: true,
            inference_timeout_ms: 1000,
            max_inference_iterations: 500,
            enable_optimizations: false,
            consistency_strictness: ConsistencyStrictness::Strict,
            enable_migration_assistance: false,
        };

        let inference = GradualTypeInference::with_config(config);
        assert!(inference.config().enable_inference);
        assert!(!inference.config().enable_contract_generation);
        assert!(!inference.config().enable_optimizations);
        assert_eq!(
            inference.config().consistency_strictness,
            ConsistencyStrictness::Strict
        );
    }

    #[test]
    fn test_type_boundary_creation() {
        let mut inference = GradualTypeInference::new();
        let span = Span::new(0, 10);

        let boundary = inference
            .create_boundary(span, &Type::Number, &Type::Dynamic)
            .unwrap();

        assert_eq!(boundary.location, span);
        assert!(matches!(boundary.cast, Cast::Upcast { .. }));
    }

    #[test]
    fn test_cast_performance_analysis() {
        let inference = GradualTypeInference::new();

        assert_eq!(
            inference.analyze_performance_impact(&Cast::None),
            PerformanceImpact::None
        );

        assert_eq!(
            inference.analyze_performance_impact(&Cast::Upcast {
                from: Type::Number,
                to: Type::Dynamic
            }),
            PerformanceImpact::Minimal
        );
    }

    #[test]
    fn test_type_to_contract_conversion() {
        let inference = GradualTypeInference::new();

        let number_contract = inference.type_to_contract_expr(&Type::Number).unwrap();
        assert!(
            matches!(number_contract, ContractExpr::Predicate { name, .. } if name == "number?")
        );

        let list_contract = inference
            .type_to_contract_expr(&Type::List(Box::new(Type::String)))
            .unwrap();
        assert!(matches!(list_contract, ContractExpr::ListOf { .. }));
    }

    #[test]
    fn test_boundary_statistics() {
        let mut manager = TypeBoundaryManager::new();
        let boundaries = vec![]; // Empty for test

        manager.update_statistics(&boundaries);
        assert_eq!(manager.stats.total_boundaries, 0);
    }

    #[test]
    fn test_cast_optimizer() {
        let optimizer = CastOptimizer::new();
        let mut casts = vec![CastInsertion {
            location: Span::new(0, 5),
            cast: Cast::None,
            reason: CastReason::StaticDynamicBoundary,
            performance_impact: PerformanceImpact::None,
        }];

        optimizer.optimize_casts(&mut casts).unwrap();
        // Identity casts should be eliminated
        assert_eq!(casts.len(), 0);
    }
}
