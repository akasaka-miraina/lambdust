#![allow(missing_docs)]
//! Generic primitive system for eliminating JIT primitive redundancy.
//!
//! This module provides a zero-cost abstraction system that generates
//! optimized JIT primitive implementations from high-level specifications.

use crate::diagnostics::{Result, UnifiedResult};
use crate::eval::Value;
use crate::jit::{
    compilation_tiers::CompilationTier, r7rs_compliance::R7RSSemanticRequirements,
    specialized_compilation_tiers::SpecializedNativeCode,
};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::Duration;

/// Unified generic primitive using enum dispatch for dyn compatibility.
#[derive(Debug, Clone)]
pub enum UnifiedGenericPrimitive {
    Arithmetic(ArithmeticPrimitive),
    Comparison(ComparisonPrimitive),
    List(ListPrimitive),
    TypePredicate(TypePredicatePrimitive),
}

/// Runtime metadata for primitives (replaces const CONFIG).
#[derive(Debug, Clone)]
pub struct PrimitiveMetadata {
    pub name: &'static str,
    pub config: PrimitiveConfig,
    pub category_name: &'static str,
    pub jit_strategy: JitCompilationStrategy,
    pub type_specializations: Vec<TypeSpecialization>,
    pub performance_profile: PrimitivePerformanceProfile,
    pub semantic_requirements: R7RSSemanticRequirements,
}

impl UnifiedGenericPrimitive {
    /// Get runtime metadata for this primitive.
    pub fn metadata(&self) -> &'static PrimitiveMetadata {
        match self {
            Self::Arithmetic(p) => p.metadata(),
            Self::Comparison(p) => p.metadata(),
            Self::List(p) => p.metadata(),
            Self::TypePredicate(p) => p.metadata(),
        }
    }

    /// Get the primitive's name.
    pub fn name(&self) -> &'static str {
        self.metadata().name
    }

    /// Interpreted implementation with automatic error handling.
    pub fn evaluate(&self, args: &[Value]) -> UnifiedResult<Value> {
        match self {
            Self::Arithmetic(p) => p.evaluate(args),
            Self::Comparison(p) => p.evaluate(args),
            Self::List(p) => p.evaluate(args),
            Self::TypePredicate(p) => p.evaluate(args),
        }
    }

    /// Get primitive configuration.
    pub fn config(&self) -> &PrimitiveConfig {
        &self.metadata().config
    }

    /// JIT compilation strategy.
    pub fn jit_strategy(&self) -> &JitCompilationStrategy {
        &self.metadata().jit_strategy
    }

    /// Type specializations available for this primitive.
    pub fn type_specializations(&self) -> &[TypeSpecialization] {
        &self.metadata().type_specializations
    }

    /// Performance profile.
    pub fn performance_profile(&self) -> &PrimitivePerformanceProfile {
        &self.metadata().performance_profile
    }

    /// R7RS semantic requirements.
    pub fn semantic_requirements(&self) -> &R7RSSemanticRequirements {
        &self.metadata().semantic_requirements
    }
}

/// Primitive category enumeration for efficient dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PrimitiveCategory {
    Arithmetic = 0,
    Comparison = 1,
    List = 2,
    TypePredicate = 3,
}

impl PrimitiveCategory {
    /// Get category name for debugging and metrics.
    pub fn name(self) -> &'static str {
        match self {
            Self::Arithmetic => "arithmetic",
            Self::Comparison => "comparison",
            Self::List => "list",
            Self::TypePredicate => "type_predicate",
        }
    }

    /// Default JIT strategy for this category.
    pub fn default_jit_strategy(self) -> JitCompilationStrategy {
        match self {
            Self::Arithmetic => JitCompilationStrategy::ConditionalInline {
                complexity_cost: 10,
                size_threshold: 32,
                call_frequency_threshold: 100.0,
            },
            Self::Comparison => JitCompilationStrategy::AlwaysInline { complexity_cost: 8 },
            Self::List => JitCompilationStrategy::TypeSpecialized {
                specialization_benefit: 1.8,
                fallback_strategy: Box::new(JitCompilationStrategy::NativeCall {
                    call_overhead: Duration::from_nanos(100),
                }),
            },
            Self::TypePredicate => JitCompilationStrategy::AlwaysInline { complexity_cost: 3 },
        }
    }

    /// Default type specializations for this category.
    pub fn default_type_specializations(self) -> Vec<TypeSpecialization> {
        match self {
            Self::Arithmetic => vec![
                TypeSpecialization {
                    input_types: vec![
                        TypePattern::Exact("integer".to_string()),
                        TypePattern::Exact("integer".to_string()),
                    ],
                    output_type: TypePattern::Exact("integer".to_string()),
                    benefit_factor: 3.0,
                    compilation_cost: 5,
                    simd_opportunities: vec![SIMDOpportunity {
                        instruction_set: "AVX2".to_string(),
                        vector_width: 256,
                        speedup_factor: 4.0,
                    }],
                },
                TypeSpecialization {
                    input_types: vec![
                        TypePattern::Exact("real".to_string()),
                        TypePattern::Exact("real".to_string()),
                    ],
                    output_type: TypePattern::Exact("real".to_string()),
                    benefit_factor: 2.5,
                    compilation_cost: 8,
                    simd_opportunities: vec![SIMDOpportunity {
                        instruction_set: "SSE2".to_string(),
                        vector_width: 128,
                        speedup_factor: 2.0,
                    }],
                },
            ],
            Self::Comparison => vec![TypeSpecialization {
                input_types: vec![TypePattern::Numeric, TypePattern::Numeric],
                output_type: TypePattern::Exact("boolean".to_string()),
                benefit_factor: 2.0,
                compilation_cost: 3,
                simd_opportunities: Vec::new(),
            }],
            Self::List => vec![TypeSpecialization {
                input_types: vec![TypePattern::Any, TypePattern::Exact("list".to_string())],
                output_type: TypePattern::Exact("list".to_string()),
                benefit_factor: 1.5,
                compilation_cost: 12,
                simd_opportunities: Vec::new(),
            }],
            Self::TypePredicate => vec![TypeSpecialization {
                input_types: vec![TypePattern::Any],
                output_type: TypePattern::Exact("boolean".to_string()),
                benefit_factor: 5.0, // Very beneficial to inline
                compilation_cost: 1,
                simd_opportunities: Vec::new(),
            }],
        }
    }

    /// Default performance profile for this category.
    pub fn default_performance_profile(self) -> PrimitivePerformanceProfile {
        match self {
            Self::Arithmetic => PrimitivePerformanceProfile {
                avg_execution_ns: 25,
                memory_profile: MemoryProfile::NoAllocation,
                cache_behavior: CacheBehavior::Excellent,
                parallelizable: true,
                branch_predictability: BranchPredictability::HighlyPredictable,
            },
            Self::Comparison => PrimitivePerformanceProfile {
                avg_execution_ns: 15,
                memory_profile: MemoryProfile::NoAllocation,
                cache_behavior: CacheBehavior::Excellent,
                parallelizable: true,
                branch_predictability: BranchPredictability::ModeratelyPredictable,
            },
            Self::List => PrimitivePerformanceProfile {
                avg_execution_ns: 80,
                memory_profile: MemoryProfile::SmallFixed(16),
                cache_behavior: CacheBehavior::Good,
                parallelizable: false,
                branch_predictability: BranchPredictability::ModeratelyPredictable,
            },
            Self::TypePredicate => PrimitivePerformanceProfile {
                avg_execution_ns: 5,
                memory_profile: MemoryProfile::NoAllocation,
                cache_behavior: CacheBehavior::Excellent,
                parallelizable: true,
                branch_predictability: BranchPredictability::HighlyPredictable,
            },
        }
    }

    /// Default R7RS requirements for this category.
    pub fn default_semantic_requirements(self) -> R7RSSemanticRequirements {
        match self {
            Self::Arithmetic => R7RSSemanticRequirements {
                requires_exact_arithmetic: true,
                requires_number_tower: true,
                ..Default::default()
            },
            Self::Comparison => R7RSSemanticRequirements {
                requires_exact_arithmetic: true,
                requires_number_tower: true,
                requires_boolean_semantics: true,
                ..Default::default()
            },
            Self::List => R7RSSemanticRequirements {
                requires_tail_calls: true, // For recursive list operations
                ..Default::default()
            },
            Self::TypePredicate => R7RSSemanticRequirements {
                requires_boolean_semantics: true,
                ..Default::default()
            },
        }
    }
}

/// Compile-time primitive configuration.
#[derive(Debug, Clone)]
pub struct PrimitiveConfig {
    /// Minimum arity (number of arguments)
    pub arity_min: usize,
    /// Maximum arity (None for variadic functions)
    pub arity_max: Option<usize>,
    /// Whether this primitive is commutative
    pub is_commutative: bool,
    /// Whether this primitive is associative
    pub is_associative: bool,
    /// Whether this primitive has side effects
    pub has_side_effects: bool,
    /// Whether this primitive can be constant-folded
    pub constant_foldable: bool,
}

/// JIT compilation strategy with performance characteristics.
#[derive(Debug, Clone)]
pub enum JitCompilationStrategy {
    /// Always inline for maximum performance
    AlwaysInline { complexity_cost: u32 },
    /// Inline only when beneficial
    ConditionalInline {
        complexity_cost: u32,
        size_threshold: usize,
        call_frequency_threshold: f64,
    },
    /// Specialize based on argument types
    TypeSpecialized {
        specialization_benefit: f64,
        fallback_strategy: Box<JitCompilationStrategy>,
    },
    /// Call as optimized native function
    NativeCall { call_overhead: Duration },
    /// Interpret (fallback for complex operations)
    Interpret,
}

/// Type specialization descriptor for JIT optimization.
#[derive(Debug, Clone)]
pub struct TypeSpecialization {
    /// Input type patterns
    pub input_types: Vec<TypePattern>,
    /// Output type
    pub output_type: TypePattern,
    /// Performance benefit factor (1.0 = no benefit)
    pub benefit_factor: f64,
    /// Compilation cost (relative units)
    pub compilation_cost: u32,
    /// SIMD opportunities for this specialization
    pub simd_opportunities: Vec<SIMDOpportunity>,
}

/// Type pattern for specialization matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypePattern {
    /// Exact type match
    Exact(String),
    /// Any numeric type
    Numeric,
    /// Any type (no specialization)
    Any,
    /// Union of specific types
    Union(Vec<String>),
}

/// SIMD optimization opportunity.
#[derive(Debug, Clone)]
pub struct SIMDOpportunity {
    /// SIMD instruction set required
    pub instruction_set: String,
    /// Vector width (e.g., 128, 256, 512 bits)
    pub vector_width: usize,
    /// Speedup factor for SIMD implementation
    pub speedup_factor: f64,
}

/// Performance profile with detailed characteristics.
#[derive(Debug, Clone)]
pub struct PrimitivePerformanceProfile {
    /// Average execution time in nanoseconds
    pub avg_execution_ns: u64,
    /// Memory allocation behavior
    pub memory_profile: MemoryProfile,
    /// CPU cache behavior
    pub cache_behavior: CacheBehavior,
    /// Whether operation can be parallelized
    pub parallelizable: bool,
    /// Branch prediction characteristics
    pub branch_predictability: BranchPredictability,
}

/// Memory allocation patterns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryProfile {
    /// No heap allocation
    NoAllocation,
    /// Small fixed allocation
    SmallFixed(usize),
    /// Large fixed allocation
    LargeFixed(usize),
    /// Variable allocation based on input
    Variable,
    /// Stack-only allocation
    StackOnly,
}

/// CPU cache behavior characteristics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheBehavior {
    /// Excellent cache locality
    Excellent,
    /// Good cache locality
    Good,
    /// Fair cache locality
    Fair,
    /// Poor cache locality
    Poor,
}

/// Branch prediction characteristics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchPredictability {
    /// Branches are highly predictable
    HighlyPredictable,
    /// Branches are moderately predictable
    ModeratelyPredictable,
    /// Branches are unpredictable
    Unpredictable,
}

// Concrete primitive implementations

/// Arithmetic primitive implementation.
#[derive(Debug, Clone)]
pub struct ArithmeticPrimitive {
    pub name: &'static str,
    pub operation: ArithmeticOperation,
}

/// Arithmetic operation variants.
#[derive(Debug, Clone)]
pub enum ArithmeticOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Abs,
    Min,
    Max,
}

impl ArithmeticPrimitive {
    pub fn metadata(&self) -> &'static PrimitiveMetadata {
        // Use lazy_static or once_cell for static metadata in production
        // For now, return a reference to statically allocated metadata
        match self.operation {
            ArithmeticOperation::Add => &ADD_METADATA,
            ArithmeticOperation::Subtract => SUB_METADATA,
            ArithmeticOperation::Multiply => MUL_METADATA,
            ArithmeticOperation::Divide => DIV_METADATA,
            ArithmeticOperation::Modulo => MOD_METADATA,
            ArithmeticOperation::Abs => ABS_METADATA,
            ArithmeticOperation::Min => MIN_METADATA,
            ArithmeticOperation::Max => MAX_METADATA,
        }
    }

    pub fn evaluate(&self, args: &[Value]) -> UnifiedResult<Value> {
        match self.operation {
            ArithmeticOperation::Add => {
                if args.is_empty() {
                    return Ok(Value::number(0.0));
                }

                let mut result = 0.0;
                for arg in args {
                    if let Some(num) = arg.as_number() {
                        result += num;
                    } else {
                        return Err(
                            crate::eval::unified_eval_errors::EvalUnifiedError::type_mismatch(
                                "number",
                                arg.clone(),
                            )
                            .into_unified(),
                        );
                    }
                }

                Ok(Value::number(result))
            }
            // Implement other operations similarly
            _ => todo!("Implement other arithmetic operations"),
        }
    }
}

/// Comparison primitive implementation.
#[derive(Debug, Clone)]
pub struct ComparisonPrimitive {
    pub name: &'static str,
    pub operation: ComparisonOperation,
}

/// Comparison operation variants.
#[derive(Debug, Clone)]
pub enum ComparisonOperation {
    Equal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    NumEqual,
}

impl ComparisonPrimitive {
    pub fn metadata(&self) -> &'static PrimitiveMetadata {
        match self.operation {
            ComparisonOperation::Equal => EQUAL_METADATA,
            ComparisonOperation::Less => LESS_METADATA,
            ComparisonOperation::Greater => GREATER_METADATA,
            ComparisonOperation::LessEqual => LESSEQ_METADATA,
            ComparisonOperation::GreaterEqual => GREATEREQ_METADATA,
            ComparisonOperation::NumEqual => NUMEQUAL_METADATA,
        }
    }

    pub fn evaluate(&self, args: &[Value]) -> UnifiedResult<Value> {
        // Implementation stub - would contain actual comparison logic
        todo!("Implement comparison operations")
    }
}

/// List primitive implementation.
#[derive(Debug, Clone)]
pub struct ListPrimitive {
    pub name: &'static str,
    pub operation: ListOperation,
}

/// List operation variants.
#[derive(Debug, Clone)]
pub enum ListOperation {
    Car,
    Cdr,
    Cons,
    List,
    Length,
    Append,
    Reverse,
}

impl ListPrimitive {
    pub fn metadata(&self) -> &'static PrimitiveMetadata {
        match self.operation {
            ListOperation::Car => CAR_METADATA,
            ListOperation::Cdr => CDR_METADATA,
            ListOperation::Cons => CONS_METADATA,
            ListOperation::List => LIST_METADATA,
            ListOperation::Length => LENGTH_METADATA,
            ListOperation::Append => APPEND_METADATA,
            ListOperation::Reverse => REVERSE_METADATA,
        }
    }

    pub fn evaluate(&self, args: &[Value]) -> UnifiedResult<Value> {
        // Implementation stub - would contain actual list operation logic
        todo!("Implement list operations")
    }
}

/// Type predicate primitive implementation.
#[derive(Debug, Clone)]
pub struct TypePredicatePrimitive {
    pub name: &'static str,
    pub operation: TypePredicateOperation,
}

/// Type predicate operation variants.
#[derive(Debug, Clone)]
pub enum TypePredicateOperation {
    IsNumber,
    IsString,
    IsSymbol,
    IsList,
    IsPair,
    IsNull,
    IsBoolean,
    IsProcedure,
}

impl TypePredicatePrimitive {
    pub fn metadata(&self) -> &'static PrimitiveMetadata {
        match self.operation {
            TypePredicateOperation::IsNumber => IS_NUMBER_METADATA,
            TypePredicateOperation::IsString => IS_STRING_METADATA,
            TypePredicateOperation::IsSymbol => IS_SYMBOL_METADATA,
            TypePredicateOperation::IsList => IS_LIST_METADATA,
            TypePredicateOperation::IsPair => IS_PAIR_METADATA,
            TypePredicateOperation::IsNull => IS_NULL_METADATA,
            TypePredicateOperation::IsBoolean => IS_BOOLEAN_METADATA,
            TypePredicateOperation::IsProcedure => IS_PROCEDURE_METADATA,
        }
    }

    pub fn evaluate(&self, args: &[Value]) -> UnifiedResult<Value> {
        // Implementation stub - would contain actual type predicate logic
        todo!("Implement type predicate operations")
    }
}

/// Generic primitive registry for efficient lookup and compilation.
pub struct GenericPrimitiveRegistry {
    /// Registered primitives by name (now using enum dispatch)
    primitives: HashMap<String, UnifiedGenericPrimitive>,
    /// Category-specific optimizations
    category_optimizations: HashMap<PrimitiveCategory, CategoryOptimization>,
    /// Performance statistics
    performance_stats: PrimitiveStats,
}

/// Category-specific optimization rules.
#[derive(Debug, Clone)]
pub struct CategoryOptimization {
    /// SIMD instruction sets to enable
    pub enabled_simd: Vec<String>,
    /// Inlining threshold for this category
    pub inline_threshold: usize,
    /// Type specialization aggressiveness
    pub specialization_aggressiveness: f64,
}

/// Performance tracking for primitives.
#[derive(Debug, Default)]
pub struct PrimitiveStats {
    /// Total primitive invocations
    pub total_invocations: u64,
    /// Total compilation time
    pub total_compilation_time: Duration,
    /// Average execution time per primitive
    pub avg_execution_times: HashMap<String, Duration>,
    /// Specialization success rate
    pub specialization_success_rate: f64,
}

impl GenericPrimitiveRegistry {
    /// Creates a new generic primitive registry.
    pub fn new() -> Self {
        Self {
            primitives: HashMap::new(),
            category_optimizations: HashMap::new(),
            performance_stats: PrimitiveStats::default(),
        }
    }

    /// Registers a primitive with the registry.
    pub fn register(&mut self, primitive: UnifiedGenericPrimitive) -> Result<()> {
        let name = primitive.name().to_string();
        let category = self.get_category(&primitive);

        // Store category optimization if not present
        if !self.category_optimizations.contains_key(&category) {
            self.category_optimizations
                .insert(category, self.default_category_optimization(category));
        }

        // Register the primitive
        self.primitives.insert(name, primitive);
        Ok(())
    }

    /// Gets a primitive by name.
    pub fn get(&self, name: &str) -> Option<&UnifiedGenericPrimitive> {
        self.primitives.get(name)
    }

    /// Returns all registered primitive names.
    pub fn primitive_names(&self) -> Vec<String> {
        self.primitives.keys().cloned().collect()
    }

    /// Returns performance statistics.
    pub fn get_stats(&self) -> &PrimitiveStats {
        &self.performance_stats
    }

    /// Get category for a primitive.
    fn get_category(&self, primitive: &UnifiedGenericPrimitive) -> PrimitiveCategory {
        match primitive {
            UnifiedGenericPrimitive::Arithmetic(_) => PrimitiveCategory::Arithmetic,
            UnifiedGenericPrimitive::Comparison(_) => PrimitiveCategory::Comparison,
            UnifiedGenericPrimitive::List(_) => PrimitiveCategory::List,
            UnifiedGenericPrimitive::TypePredicate(_) => PrimitiveCategory::TypePredicate,
        }
    }

    /// Creates default category optimization.
    fn default_category_optimization(&self, category: PrimitiveCategory) -> CategoryOptimization {
        CategoryOptimization {
            enabled_simd: match category {
                PrimitiveCategory::Arithmetic => vec!["SSE2".to_string(), "AVX2".to_string()],
                PrimitiveCategory::Comparison => vec!["SSE2".to_string()],
                _ => Vec::new(),
            },
            inline_threshold: match category {
                PrimitiveCategory::TypePredicate => 0, // Always inline
                PrimitiveCategory::Arithmetic => 32,
                PrimitiveCategory::Comparison => 16,
                _ => 64,
            },
            specialization_aggressiveness: match category {
                PrimitiveCategory::Arithmetic => 0.8,
                PrimitiveCategory::Comparison => 0.9,
                PrimitiveCategory::TypePredicate => 1.0,
                _ => 0.5,
            },
        }
    }
}

impl Default for GenericPrimitiveRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Static metadata definitions (would be generated by macro in production)
static ADD_METADATA: PrimitiveMetadata = PrimitiveMetadata {
    name: "+",
    config: PrimitiveConfig {
        arity_min: 0,
        arity_max: None,
        is_commutative: true,
        is_associative: true,
        has_side_effects: false,
        constant_foldable: true,
    },
    category_name: "arithmetic",
    jit_strategy: JitCompilationStrategy::ConditionalInline {
        complexity_cost: 10,
        size_threshold: 32,
        call_frequency_threshold: 100.0,
    },
    type_specializations: vec![], // Would be populated in production
    performance_profile: PrimitivePerformanceProfile {
        avg_execution_ns: 25,
        memory_profile: MemoryProfile::NoAllocation,
        cache_behavior: CacheBehavior::Excellent,
        parallelizable: true,
        branch_predictability: BranchPredictability::HighlyPredictable,
    },
    semantic_requirements: R7RSSemanticRequirements {
        requires_exact_arithmetic: true,
        requires_number_tower: true,
        requires_boolean_semantics: false,
        requires_tail_calls: false,
        requires_continuations: false,
        requires_lexical_scoping: true,
        requires_r7rs_errors: true,
        requires_symbol_identity: true,
    },
};

// Placeholder metadata - use references to avoid move errors
static SUB_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static MUL_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static DIV_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static MOD_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static ABS_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static MIN_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static MAX_METADATA: &PrimitiveMetadata = &ADD_METADATA;

static EQUAL_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static LESS_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static GREATER_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static LESSEQ_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static GREATEREQ_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static NUMEQUAL_METADATA: &PrimitiveMetadata = &ADD_METADATA;

static CAR_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static CDR_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static CONS_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static LIST_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static LENGTH_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static APPEND_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static REVERSE_METADATA: &PrimitiveMetadata = &ADD_METADATA;

static IS_NUMBER_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static IS_STRING_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static IS_SYMBOL_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static IS_LIST_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static IS_PAIR_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static IS_NULL_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static IS_BOOLEAN_METADATA: &PrimitiveMetadata = &ADD_METADATA;
static IS_PROCEDURE_METADATA: &PrimitiveMetadata = &ADD_METADATA;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_add_primitive() -> UnifiedGenericPrimitive {
        UnifiedGenericPrimitive::Arithmetic(ArithmeticPrimitive {
            name: "+",
            operation: ArithmeticOperation::Add,
        })
    }

    #[test]
    fn test_primitive_registration() {
        let mut registry = GenericPrimitiveRegistry::new();
        registry.register(create_add_primitive()).unwrap();

        assert!(registry.get("+").is_some());
        assert_eq!(registry.primitive_names().len(), 1);
    }

    #[test]
    fn test_primitive_evaluation() {
        let add = create_add_primitive();
        let args = vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)];
        let result = add.evaluate(&args).unwrap();

        assert_eq!(result.as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_primitive_configuration() {
        let add = create_add_primitive();

        assert_eq!(add.name(), "+");
        assert!(add.config().is_commutative);
        assert!(add.config().is_associative);
        assert!(!add.config().has_side_effects);
    }

    #[test]
    fn test_category_defaults() {
        let category = PrimitiveCategory::Arithmetic;
        let strategy = category.default_jit_strategy();
        let profile = category.default_performance_profile();

        assert!(matches!(
            strategy,
            JitCompilationStrategy::ConditionalInline { .. }
        ));
        assert_eq!(profile.memory_profile, MemoryProfile::NoAllocation);
        assert_eq!(profile.cache_behavior, CacheBehavior::Excellent);
    }
}
