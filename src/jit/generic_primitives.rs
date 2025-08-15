//! Generic primitive system for eliminating JIT primitive redundancy.
//!
//! This module provides a zero-cost abstraction system that generates
//! optimized JIT primitive implementations from high-level specifications.

use crate::eval::Value;
use crate::jit::{
    r7rs_compliance::R7RSSemanticRequirements,
    compilation_tiers::CompilationTier,
    specialized_compilation_tiers::SpecializedNativeCode,
};
use crate::diagnostics::{Result, UnifiedResult};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::Duration;

/// Generic primitive trait providing compile-time polymorphism.
pub trait GenericPrimitive: Send + Sync + 'static {
    /// Primitive category for grouping similar operations.
    type Category: PrimitiveCategory;
    
    /// Compile-time primitive configuration.
    const CONFIG: PrimitiveConfig;
    
    /// The primitive's name.
    fn name(&self) -> &'static str;
    
    /// Interpreted implementation with automatic error handling.
    fn evaluate(&self, args: &[Value]) -> UnifiedResult<Value>;
    
    /// JIT compilation strategy (with default for category).
    fn jit_strategy(&self) -> JitCompilationStrategy {
        Self::Category::default_jit_strategy()
    }
    
    /// Type specializations available for this primitive.
    fn type_specializations(&self) -> Vec<TypeSpecialization> {
        Self::Category::default_type_specializations()
    }
    
    /// Performance profile (with category defaults).
    fn performance_profile(&self) -> PrimitivePerformanceProfile {
        Self::Category::default_performance_profile()
    }
    
    /// R7RS semantic requirements (with category defaults).
    fn semantic_requirements(&self) -> R7RSSemanticRequirements {
        Self::Category::default_semantic_requirements()
    }
}

/// Primitive category trait for shared behavior and optimizations.
pub trait PrimitiveCategory: Send + Sync + 'static {
    /// Category name for debugging and metrics.
    const CATEGORY_NAME: &'static str;
    
    /// Default JIT strategy for this category.
    fn default_jit_strategy() -> JitCompilationStrategy;
    
    /// Default type specializations for this category.
    fn default_type_specializations() -> Vec<TypeSpecialization>;
    
    /// Default performance profile for this category.
    fn default_performance_profile() -> PrimitivePerformanceProfile;
    
    /// Default R7RS requirements for this category.
    fn default_semantic_requirements() -> R7RSSemanticRequirements;
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
    AlwaysInline {
        complexity_cost: u32,
    },
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
    NativeCall {
        call_overhead: Duration,
    },
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

// Primitive categories with shared optimizations

/// Arithmetic operation category.
pub struct ArithmeticCategory;

impl PrimitiveCategory for ArithmeticCategory {
    const CATEGORY_NAME: &'static str = "arithmetic";
    
    fn default_jit_strategy() -> JitCompilationStrategy {
        JitCompilationStrategy::ConditionalInline {
            complexity_cost: 10,
            size_threshold: 32,
            call_frequency_threshold: 100.0,
        }
    }
    
    fn default_type_specializations() -> Vec<TypeSpecialization> {
        vec![
            TypeSpecialization {
                input_types: vec![TypePattern::Exact("integer".to_string()), TypePattern::Exact("integer".to_string())],
                output_type: TypePattern::Exact("integer".to_string()),
                benefit_factor: 3.0,
                compilation_cost: 5,
                simd_opportunities: vec![
                    SIMDOpportunity {
                        instruction_set: "AVX2".to_string(),
                        vector_width: 256,
                        speedup_factor: 4.0,
                    }
                ],
            },
            TypeSpecialization {
                input_types: vec![TypePattern::Exact("real".to_string()), TypePattern::Exact("real".to_string())],
                output_type: TypePattern::Exact("real".to_string()),
                benefit_factor: 2.5,
                compilation_cost: 8,
                simd_opportunities: vec![
                    SIMDOpportunity {
                        instruction_set: "SSE2".to_string(),
                        vector_width: 128,
                        speedup_factor: 2.0,
                    }
                ],
            },
        ]
    }
    
    fn default_performance_profile() -> PrimitivePerformanceProfile {
        PrimitivePerformanceProfile {
            avg_execution_ns: 25,
            memory_profile: MemoryProfile::NoAllocation,
            cache_behavior: CacheBehavior::Excellent,
            parallelizable: true,
            branch_predictability: BranchPredictability::HighlyPredictable,
        }
    }
    
    fn default_semantic_requirements() -> R7RSSemanticRequirements {
        R7RSSemanticRequirements {
            requires_exact_arithmetic: true,
            requires_number_tower: true,
            ..Default::default()
        }
    }
}

/// Comparison operation category.
pub struct ComparisonCategory;

impl PrimitiveCategory for ComparisonCategory {
    const CATEGORY_NAME: &'static str = "comparison";
    
    fn default_jit_strategy() -> JitCompilationStrategy {
        JitCompilationStrategy::AlwaysInline {
            complexity_cost: 8,
        }
    }
    
    fn default_type_specializations() -> Vec<TypeSpecialization> {
        vec![
            TypeSpecialization {
                input_types: vec![TypePattern::Numeric, TypePattern::Numeric],
                output_type: TypePattern::Exact("boolean".to_string()),
                benefit_factor: 2.0,
                compilation_cost: 3,
                simd_opportunities: Vec::new(),
            }
        ]
    }
    
    fn default_performance_profile() -> PrimitivePerformanceProfile {
        PrimitivePerformanceProfile {
            avg_execution_ns: 15,
            memory_profile: MemoryProfile::NoAllocation,
            cache_behavior: CacheBehavior::Excellent,
            parallelizable: true,
            branch_predictability: BranchPredictability::ModeratelyPredictable,
        }
    }
    
    fn default_semantic_requirements() -> R7RSSemanticRequirements {
        R7RSSemanticRequirements {
            requires_exact_arithmetic: true,
            requires_number_tower: true,
            requires_boolean_semantics: true,
            ..Default::default()
        }
    }
}

/// List operation category.
pub struct ListCategory;

impl PrimitiveCategory for ListCategory {
    const CATEGORY_NAME: &'static str = "list";
    
    fn default_jit_strategy() -> JitCompilationStrategy {
        JitCompilationStrategy::TypeSpecialized {
            specialization_benefit: 1.8,
            fallback_strategy: Box::new(JitCompilationStrategy::NativeCall {
                call_overhead: Duration::from_nanos(100),
            }),
        }
    }
    
    fn default_type_specializations() -> Vec<TypeSpecialization> {
        vec![
            TypeSpecialization {
                input_types: vec![TypePattern::Any, TypePattern::Exact("list".to_string())],
                output_type: TypePattern::Exact("list".to_string()),
                benefit_factor: 1.5,
                compilation_cost: 12,
                simd_opportunities: Vec::new(),
            }
        ]
    }
    
    fn default_performance_profile() -> PrimitivePerformanceProfile {
        PrimitivePerformanceProfile {
            avg_execution_ns: 80,
            memory_profile: MemoryProfile::SmallFixed(16),
            cache_behavior: CacheBehavior::Good,
            parallelizable: false,
            branch_predictability: BranchPredictability::ModeratelyPredictable,
        }
    }
    
    fn default_semantic_requirements() -> R7RSSemanticRequirements {
        R7RSSemanticRequirements {
            requires_tail_calls: true, // For recursive list operations
            ..Default::default()
        }
    }
}

/// Type predicate category.
pub struct TypePredicateCategory;

impl PrimitiveCategory for TypePredicateCategory {
    const CATEGORY_NAME: &'static str = "type_predicate";
    
    fn default_jit_strategy() -> JitCompilationStrategy {
        JitCompilationStrategy::AlwaysInline {
            complexity_cost: 3,
        }
    }
    
    fn default_type_specializations() -> Vec<TypeSpecialization> {
        vec![
            TypeSpecialization {
                input_types: vec![TypePattern::Any],
                output_type: TypePattern::Exact("boolean".to_string()),
                benefit_factor: 5.0, // Very beneficial to inline
                compilation_cost: 1,
                simd_opportunities: Vec::new(),
            }
        ]
    }
    
    fn default_performance_profile() -> PrimitivePerformanceProfile {
        PrimitivePerformanceProfile {
            avg_execution_ns: 5,
            memory_profile: MemoryProfile::NoAllocation,
            cache_behavior: CacheBehavior::Excellent,
            parallelizable: true,
            branch_predictability: BranchPredictability::HighlyPredictable,
        }
    }
    
    fn default_semantic_requirements() -> R7RSSemanticRequirements {
        R7RSSemanticRequirements {
            requires_boolean_semantics: true,
            ..Default::default()
        }
    }
}

/// Generic primitive registry for efficient lookup and compilation.
pub struct GenericPrimitiveRegistry {
    /// Registered primitives by name
    primitives: HashMap<String, Box<dyn GenericPrimitive<Category = dyn PrimitiveCategory>>>,
    /// Category-specific optimizations
    category_optimizations: HashMap<String, CategoryOptimization>,
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
    pub fn register<P: GenericPrimitive + 'static>(&mut self, primitive: P) -> Result<()> {
        let name = primitive.name().to_string();
        
        // Store category optimization if not present
        let category_name = P::Category::CATEGORY_NAME.to_string();
        if !self.category_optimizations.contains_key(&category_name) {
            self.category_optimizations.insert(
                category_name.clone(),
                self.default_category_optimization(&category_name),
            );
        }
        
        // Register the primitive
        self.primitives.insert(name, Box::new(primitive));
        Ok(())
    }
    
    /// Gets a primitive by name.
    pub fn get(&self, name: &str) -> Option<&dyn GenericPrimitive<Category = dyn PrimitiveCategory>> {
        self.primitives.get(name).map(|p| p.as_ref())
    }
    
    /// Returns all registered primitive names.
    pub fn primitive_names(&self) -> Vec<String> {
        self.primitives.keys().cloned().collect()
    }
    
    /// Returns performance statistics.
    pub fn get_stats(&self) -> &PrimitiveStats {
        &self.performance_stats
    }
    
    /// Creates default category optimization.
    fn default_category_optimization(&self, category: &str) -> CategoryOptimization {
        CategoryOptimization {
            enabled_simd: match category {
                "arithmetic" => vec!["SSE2".to_string(), "AVX2".to_string()],
                "comparison" => vec!["SSE2".to_string()],
                _ => Vec::new(),
            },
            inline_threshold: match category {
                "type_predicate" => 0, // Always inline
                "arithmetic" => 32,
                "comparison" => 16,
                _ => 64,
            },
            specialization_aggressiveness: match category {
                "arithmetic" => 0.8,
                "comparison" => 0.9,
                "type_predicate" => 1.0,
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

#[cfg(test)]
mod tests {
    use super::*;

    // Example primitive implementation
    struct AddPrimitive;
    
    impl GenericPrimitive for AddPrimitive {
        type Category = ArithmeticCategory;
        
        const CONFIG: PrimitiveConfig = PrimitiveConfig {
            arity_min: 0,
            arity_max: None,
            is_commutative: true,
            is_associative: true,
            has_side_effects: false,
            constant_foldable: true,
        };
        
        fn name(&self) -> &'static str {
            "+"
        }
        
        fn evaluate(&self, args: &[Value]) -> UnifiedResult<Value> {
            if args.is_empty() {
                return Ok(Value::number(0.0));
            }
            
            let mut result = 0.0;
            for arg in args {
                if let Some(num) = arg.as_number() {
                    result += num;
                } else {
                    return Err(crate::eval::unified_eval_errors::EvalUnifiedError::type_mismatch(
                        "number", arg.clone()
                    ).into_unified());
                }
            }
            
            Ok(Value::number(result))
        }
    }

    #[test]
    fn test_primitive_registration() {
        let mut registry = GenericPrimitiveRegistry::new();
        registry.register(AddPrimitive).unwrap();
        
        assert!(registry.get("+").is_some());
        assert_eq!(registry.primitive_names().len(), 1);
    }

    #[test]
    fn test_primitive_evaluation() {
        let add = AddPrimitive;
        let args = vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)];
        let result = add.evaluate(&args).unwrap();
        
        assert_eq!(result.as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_primitive_configuration() {
        let add = AddPrimitive;
        
        assert_eq!(add.name(), "+");
        assert!(AddPrimitive::CONFIG.is_commutative);
        assert!(AddPrimitive::CONFIG.is_associative);
        assert!(!AddPrimitive::CONFIG.has_side_effects);
    }

    #[test]
    fn test_category_defaults() {
        let strategy = ArithmeticCategory::default_jit_strategy();
        let profile = ArithmeticCategory::default_performance_profile();
        
        assert!(matches!(strategy, JitCompilationStrategy::ConditionalInline { .. }));
        assert_eq!(profile.memory_profile, MemoryProfile::NoAllocation);
        assert_eq!(profile.cache_behavior, CacheBehavior::Excellent);
    }
}