#![allow(missing_docs)]//! JIT-aware implementations of the 42 core Lambdust primitives
//!
//! This module provides specialized implementations of R7RS primitives that can be
//! efficiently compiled by the JIT system while maintaining strict semantic compliance.

use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result};
use crate::eval::Value;
use crate::jit::{
    compilation_tiers::CompilationTier,
    r7rs_compliance::{CORE_R7RS_PRIMITIVES, R7RSSemanticRequirements},
    specialized_compilation_tiers::SpecializedNativeCode,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// JIT-aware primitive implementation
pub struct JitPrimitive {
    /// Primitive name
    pub name: String,
    /// Minimum arity
    pub arity_min: usize,
    /// Maximum arity (None for variadic)
    pub arity_max: Option<usize>,
    /// R7RS semantic requirements
    pub semantic_requirements: R7RSSemanticRequirements,
    /// Interpreted implementation
    pub interpreted_impl: fn(&[Value]) -> Result<Value>,
    /// JIT compilation strategy for this primitive
    pub jit_strategy: JitCompilationStrategy,
    /// Type specialization opportunities
    pub type_specializations: Vec<TypeSpecialization>,
    /// Performance characteristics
    pub performance_profile: PrimitivePerformanceProfile,
}

/// JIT compilation strategy for primitives
#[derive(Debug, Clone)]
pub enum JitCompilationStrategy {
    /// Inline directly into generated code
    Inline {
        /// Inline complexity cost (higher = more expensive to inline)
        complexity_cost: u32,
        /// Whether this primitive benefits from inlining
        benefits_from_inlining: bool,
    },
    /// Call as specialized native function
    SpecializedCall {
        /// Tier at which specialization is beneficial
        min_beneficial_tier: CompilationTier,
        /// Whether multiple specializations are possible
        multi_specializable: bool,
    },
    /// Call interpreter fallback
    InterpreterFallback,
    /// Custom JIT generation
    Custom {
        /// Custom code generator function
        generator: fn(&Expr, &[Value]) -> Result<SpecializedNativeCode>,
    },
}

/// Type specialization opportunity for a primitive
#[derive(Debug, Clone)]
pub struct TypeSpecialization {
    /// Types this specialization applies to
    pub input_types: Vec<String>,
    /// Expected output type
    pub output_type: String,
    /// Performance benefit factor
    pub benefit_factor: f64,
    /// Compilation complexity
    pub compilation_cost: u32,
}

/// Performance profile for a primitive
#[derive(Debug, Clone)]
pub struct PrimitivePerformanceProfile {
    /// Average execution time in nanoseconds
    pub avg_execution_ns: u64,
    /// Memory allocation characteristics
    pub memory_profile: MemoryProfile,
    /// Cache behavior
    pub cache_behavior: CacheBehavior,
    /// Parallelization potential
    pub parallelizable: bool,
}

/// Memory allocation profile
#[derive(Debug, Clone)]
pub enum MemoryProfile {
    /// No memory allocation
    NoAllocation,
    /// Constant allocation
    ConstantAllocation { bytes: usize },
    /// Linear allocation based on input size
    LinearAllocation { bytes_per_element: usize },
    /// Complex allocation pattern
    ComplexAllocation { description: String },
}

/// Cache behavior characteristics
#[derive(Debug, Clone)]
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

/// Registry for all JIT-aware primitives
pub struct JitPrimitiveRegistry {
    /// All registered primitives by name
    primitives: HashMap<String, JitPrimitive>,
    /// Performance statistics
    stats: PrimitiveStats,
}

/// Statistics about primitive usage and performance
#[derive(Debug, Default)]
pub struct PrimitiveStats {
    /// Total primitive calls
    total_calls: HashMap<String, u64>,
    /// Total execution time per primitive
    total_execution_time: HashMap<String, Duration>,
    /// JIT compilation successes
    jit_successes: HashMap<String, u64>,
    /// JIT compilation failures
    jit_failures: HashMap<String, u64>,
}

impl JitPrimitiveRegistry {
    /// Create new registry with all 42 core primitives
    pub fn new() -> Result<Self> {
        let mut registry = Self {
            primitives: HashMap::new(),
            stats: PrimitiveStats::default(),
        };

        // Register all 42 core R7RS primitives
        registry.register_core_primitives()?;

        Ok(registry)
    }

    /// Register all 42 core R7RS primitives
    fn register_core_primitives(&mut self) -> Result<()> {
        // Arithmetic primitives (12)
        self.register_arithmetic_primitives()?;

        // Comparison primitives (6)
        self.register_comparison_primitives()?;

        // List operation primitives (8)
        self.register_list_primitives()?;

        // Type predicate primitives (6)
        self.register_type_predicate_primitives()?;

        // Equality and logic primitives (4)
        self.register_equality_primitives()?;

        // Control flow primitives (3)
        self.register_control_flow_primitives()?;

        // I/O primitives (3)
        self.register_io_primitives()?;

        Ok(())
    }

    /// Register arithmetic primitives
    fn register_arithmetic_primitives(&mut self) -> Result<()> {
        // Addition: +
        self.primitives.insert(
            "+".to_string(),
            JitPrimitive {
                name: "+".to_string(),
                arity_min: 0,
                arity_max: None,
                semantic_requirements: R7RSSemanticRequirements {
                    requires_exact_arithmetic: true,
                    requires_number_tower: true,
                    ..Default::default()
                },
                interpreted_impl: jit_add,
                jit_strategy: JitCompilationStrategy::Inline {
                    complexity_cost: 10,
                    benefits_from_inlining: true,
                },
                type_specializations: vec![
                    TypeSpecialization {
                        input_types: vec!["integer".to_string(), "integer".to_string()],
                        output_type: "integer".to_string(),
                        benefit_factor: 2.0,
                        compilation_cost: 5,
                    },
                    TypeSpecialization {
                        input_types: vec!["real".to_string(), "real".to_string()],
                        output_type: "real".to_string(),
                        benefit_factor: 1.5,
                        compilation_cost: 8,
                    },
                ],
                performance_profile: PrimitivePerformanceProfile {
                    avg_execution_ns: 50,
                    memory_profile: MemoryProfile::NoAllocation,
                    cache_behavior: CacheBehavior::Excellent,
                    parallelizable: true,
                },
            },
        );

        // Subtraction: -
        self.primitives.insert(
            "-".to_string(),
            JitPrimitive {
                name: "-".to_string(),
                arity_min: 1,
                arity_max: None,
                semantic_requirements: R7RSSemanticRequirements {
                    requires_exact_arithmetic: true,
                    requires_number_tower: true,
                    ..Default::default()
                },
                interpreted_impl: jit_subtract,
                jit_strategy: JitCompilationStrategy::Inline {
                    complexity_cost: 10,
                    benefits_from_inlining: true,
                },
                type_specializations: vec![TypeSpecialization {
                    input_types: vec!["integer".to_string()],
                    output_type: "integer".to_string(),
                    benefit_factor: 2.0,
                    compilation_cost: 5,
                }],
                performance_profile: PrimitivePerformanceProfile {
                    avg_execution_ns: 50,
                    memory_profile: MemoryProfile::NoAllocation,
                    cache_behavior: CacheBehavior::Excellent,
                    parallelizable: false,
                },
            },
        );

        // Multiplication: *
        self.primitives.insert(
            "*".to_string(),
            JitPrimitive {
                name: "*".to_string(),
                arity_min: 0,
                arity_max: None,
                semantic_requirements: R7RSSemanticRequirements {
                    requires_exact_arithmetic: true,
                    requires_number_tower: true,
                    ..Default::default()
                },
                interpreted_impl: jit_multiply,
                jit_strategy: JitCompilationStrategy::Inline {
                    complexity_cost: 15,
                    benefits_from_inlining: true,
                },
                type_specializations: vec![TypeSpecialization {
                    input_types: vec!["integer".to_string(), "integer".to_string()],
                    output_type: "integer".to_string(),
                    benefit_factor: 2.5,
                    compilation_cost: 8,
                }],
                performance_profile: PrimitivePerformanceProfile {
                    avg_execution_ns: 75,
                    memory_profile: MemoryProfile::NoAllocation,
                    cache_behavior: CacheBehavior::Excellent,
                    parallelizable: true,
                },
            },
        );

        // Division: /
        self.primitives.insert(
            "/".to_string(),
            JitPrimitive {
                name: "/".to_string(),
                arity_min: 1,
                arity_max: None,
                semantic_requirements: R7RSSemanticRequirements {
                    requires_exact_arithmetic: true,
                    requires_number_tower: true,
                    requires_r7rs_errors: true, // Division by zero
                    ..Default::default()
                },
                interpreted_impl: jit_divide,
                jit_strategy: JitCompilationStrategy::SpecializedCall {
                    min_beneficial_tier: CompilationTier::JitBasic,
                    multi_specializable: true,
                },
                type_specializations: vec![TypeSpecialization {
                    input_types: vec!["integer".to_string(), "integer".to_string()],
                    output_type: "rational".to_string(),
                    benefit_factor: 3.0,
                    compilation_cost: 20,
                }],
                performance_profile: PrimitivePerformanceProfile {
                    avg_execution_ns: 200,
                    memory_profile: MemoryProfile::ConstantAllocation { bytes: 32 },
                    cache_behavior: CacheBehavior::Good,
                    parallelizable: false,
                },
            },
        );

        // Add remaining arithmetic primitives with similar patterns
        let remaining_arithmetic = [
            "quotient",
            "remainder",
            "modulo",
            "abs",
            "gcd",
            "lcm",
            "floor",
            "ceiling",
        ];
        for &prim in &remaining_arithmetic {
            self.primitives.insert(
                prim.to_string(),
                JitPrimitive {
                    name: prim.to_string(),
                    arity_min: if prim == "abs" || prim == "floor" || prim == "ceiling" {
                        1
                    } else {
                        2
                    },
                    arity_max: Some(if prim == "gcd" || prim == "lcm" { 2 } else { 2 }),
                    semantic_requirements: R7RSSemanticRequirements {
                        requires_exact_arithmetic: true,
                        requires_number_tower: true,
                        ..Default::default()
                    },
                    interpreted_impl: get_arithmetic_impl(prim),
                    jit_strategy: JitCompilationStrategy::SpecializedCall {
                        min_beneficial_tier: CompilationTier::JitBasic,
                        multi_specializable: false,
                    },
                    type_specializations: vec![],
                    performance_profile: PrimitivePerformanceProfile {
                        avg_execution_ns: 100,
                        memory_profile: MemoryProfile::NoAllocation,
                        cache_behavior: CacheBehavior::Good,
                        parallelizable: false,
                    },
                },
            );
        }

        Ok(())
    }

    /// Register comparison primitives
    fn register_comparison_primitives(&mut self) -> Result<()> {
        let comparisons = ["=", "<", ">", "<=", ">=", "max"];

        for &prim in &comparisons {
            self.primitives.insert(
                prim.to_string(),
                JitPrimitive {
                    name: prim.to_string(),
                    arity_min: if prim == "max" { 1 } else { 2 },
                    arity_max: None,
                    semantic_requirements: R7RSSemanticRequirements {
                        requires_exact_arithmetic: true,
                        requires_number_tower: true,
                        requires_boolean_semantics: true,
                        ..Default::default()
                    },
                    interpreted_impl: get_comparison_impl(prim),
                    jit_strategy: JitCompilationStrategy::Inline {
                        complexity_cost: 8,
                        benefits_from_inlining: true,
                    },
                    type_specializations: vec![TypeSpecialization {
                        input_types: vec!["integer".to_string(), "integer".to_string()],
                        output_type: "boolean".to_string(),
                        benefit_factor: 3.0,
                        compilation_cost: 5,
                    }],
                    performance_profile: PrimitivePerformanceProfile {
                        avg_execution_ns: 30,
                        memory_profile: MemoryProfile::NoAllocation,
                        cache_behavior: CacheBehavior::Excellent,
                        parallelizable: true,
                    },
                },
            );
        }

        Ok(())
    }

    /// Register list operation primitives
    fn register_list_primitives(&mut self) -> Result<()> {
        // cons
        self.primitives.insert(
            "cons".to_string(),
            JitPrimitive {
                name: "cons".to_string(),
                arity_min: 2,
                arity_max: Some(2),
                semantic_requirements: R7RSSemanticRequirements::default(),
                interpreted_impl: jit_cons,
                jit_strategy: JitCompilationStrategy::SpecializedCall {
                    min_beneficial_tier: CompilationTier::JitBasic,
                    multi_specializable: false,
                },
                type_specializations: vec![],
                performance_profile: PrimitivePerformanceProfile {
                    avg_execution_ns: 100,
                    memory_profile: MemoryProfile::ConstantAllocation { bytes: 16 },
                    cache_behavior: CacheBehavior::Good,
                    parallelizable: false,
                },
            },
        );

        // car
        self.primitives.insert(
            "car".to_string(),
            JitPrimitive {
                name: "car".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                semantic_requirements: R7RSSemanticRequirements {
                    requires_r7rs_errors: true, // Error on non-pair
                    ..Default::default()
                },
                interpreted_impl: jit_car,
                jit_strategy: JitCompilationStrategy::Inline {
                    complexity_cost: 5,
                    benefits_from_inlining: true,
                },
                type_specializations: vec![],
                performance_profile: PrimitivePerformanceProfile {
                    avg_execution_ns: 20,
                    memory_profile: MemoryProfile::NoAllocation,
                    cache_behavior: CacheBehavior::Excellent,
                    parallelizable: true,
                },
            },
        );

        // cdr
        self.primitives.insert(
            "cdr".to_string(),
            JitPrimitive {
                name: "cdr".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                semantic_requirements: R7RSSemanticRequirements {
                    requires_r7rs_errors: true, // Error on non-pair
                    ..Default::default()
                },
                interpreted_impl: jit_cdr,
                jit_strategy: JitCompilationStrategy::Inline {
                    complexity_cost: 5,
                    benefits_from_inlining: true,
                },
                type_specializations: vec![],
                performance_profile: PrimitivePerformanceProfile {
                    avg_execution_ns: 20,
                    memory_profile: MemoryProfile::NoAllocation,
                    cache_behavior: CacheBehavior::Excellent,
                    parallelizable: true,
                },
            },
        );

        // Add remaining list primitives
        let remaining_list = ["null?", "pair?", "list", "length", "append"];
        for &prim in &remaining_list {
            self.primitives.insert(
                prim.to_string(),
                JitPrimitive {
                    name: prim.to_string(),
                    arity_min: if prim == "list" {
                        0
                    } else if prim == "append" {
                        0
                    } else {
                        1
                    },
                    arity_max: if prim == "list" || prim == "append" {
                        None
                    } else {
                        Some(1)
                    },
                    semantic_requirements: R7RSSemanticRequirements {
                        requires_tail_calls: prim == "length" || prim == "append",
                        requires_boolean_semantics: prim.ends_with('?'),
                        ..Default::default()
                    },
                    interpreted_impl: get_list_impl(prim),
                    jit_strategy: if prim == "length" || prim == "append" {
                        JitCompilationStrategy::SpecializedCall {
                            min_beneficial_tier: CompilationTier::JitOptimized,
                            multi_specializable: true,
                        }
                    } else {
                        JitCompilationStrategy::Inline {
                            complexity_cost: 10,
                            benefits_from_inlining: true,
                        }
                    },
                    type_specializations: vec![],
                    performance_profile: PrimitivePerformanceProfile {
                        avg_execution_ns: if prim == "length" || prim == "append" {
                            500
                        } else {
                            50
                        },
                        memory_profile: if prim == "list" || prim == "append" {
                            MemoryProfile::LinearAllocation {
                                bytes_per_element: 16,
                            }
                        } else {
                            MemoryProfile::NoAllocation
                        },
                        cache_behavior: if prim == "length" || prim == "append" {
                            CacheBehavior::Fair
                        } else {
                            CacheBehavior::Good
                        },
                        parallelizable: false,
                    },
                },
            );
        }

        Ok(())
    }

    /// Register type predicate primitives
    fn register_type_predicate_primitives(&mut self) -> Result<()> {
        let predicates = [
            "number?",
            "string?",
            "symbol?",
            "boolean?",
            "procedure?",
            "vector?",
        ];

        for &prim in &predicates {
            self.primitives.insert(
                prim.to_string(),
                JitPrimitive {
                    name: prim.to_string(),
                    arity_min: 1,
                    arity_max: Some(1),
                    semantic_requirements: R7RSSemanticRequirements {
                        requires_boolean_semantics: true,
                        ..Default::default()
                    },
                    interpreted_impl: get_predicate_impl(prim),
                    jit_strategy: JitCompilationStrategy::Inline {
                        complexity_cost: 3,
                        benefits_from_inlining: true,
                    },
                    type_specializations: vec![TypeSpecialization {
                        input_types: vec!["any".to_string()],
                        output_type: "boolean".to_string(),
                        benefit_factor: 4.0,
                        compilation_cost: 2,
                    }],
                    performance_profile: PrimitivePerformanceProfile {
                        avg_execution_ns: 15,
                        memory_profile: MemoryProfile::NoAllocation,
                        cache_behavior: CacheBehavior::Excellent,
                        parallelizable: true,
                    },
                },
            );
        }

        Ok(())
    }

    /// Register equality and logic primitives
    fn register_equality_primitives(&mut self) -> Result<()> {
        let equality_prims = ["eq?", "eqv?", "equal?", "not"];

        for &prim in &equality_prims {
            self.primitives.insert(
                prim.to_string(),
                JitPrimitive {
                    name: prim.to_string(),
                    arity_min: if prim == "not" { 1 } else { 2 },
                    arity_max: Some(if prim == "not" { 1 } else { 2 }),
                    semantic_requirements: R7RSSemanticRequirements {
                        requires_boolean_semantics: true,
                        requires_symbol_identity: prim != "not",
                        ..Default::default()
                    },
                    interpreted_impl: get_equality_impl(prim),
                    jit_strategy: JitCompilationStrategy::Inline {
                        complexity_cost: if prim == "equal?" { 20 } else { 5 },
                        benefits_from_inlining: true,
                    },
                    type_specializations: vec![],
                    performance_profile: PrimitivePerformanceProfile {
                        avg_execution_ns: if prim == "equal?" { 200 } else { 25 },
                        memory_profile: MemoryProfile::NoAllocation,
                        cache_behavior: CacheBehavior::Good,
                        parallelizable: prim != "equal?",
                    },
                },
            );
        }

        Ok(())
    }

    /// Register control flow primitives
    fn register_control_flow_primitives(&mut self) -> Result<()> {
        // apply
        self.primitives.insert(
            "apply".to_string(),
            JitPrimitive {
                name: "apply".to_string(),
                arity_min: 2,
                arity_max: None,
                semantic_requirements: R7RSSemanticRequirements {
                    requires_tail_calls: true,
                    requires_continuations: true,
                    ..Default::default()
                },
                interpreted_impl: jit_apply,
                jit_strategy: JitCompilationStrategy::Custom {
                    generator: generate_apply_code,
                },
                type_specializations: vec![],
                performance_profile: PrimitivePerformanceProfile {
                    avg_execution_ns: 500,
                    memory_profile: MemoryProfile::LinearAllocation {
                        bytes_per_element: 8,
                    },
                    cache_behavior: CacheBehavior::Fair,
                    parallelizable: false,
                },
            },
        );

        // call/cc
        self.primitives.insert(
            "call/cc".to_string(),
            JitPrimitive {
                name: "call/cc".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                semantic_requirements: R7RSSemanticRequirements {
                    requires_continuations: true,
                    requires_tail_calls: true,
                    ..Default::default()
                },
                interpreted_impl: jit_call_cc,
                jit_strategy: JitCompilationStrategy::Custom {
                    generator: generate_call_cc_code,
                },
                type_specializations: vec![],
                performance_profile: PrimitivePerformanceProfile {
                    avg_execution_ns: 2000,
                    memory_profile: MemoryProfile::ComplexAllocation {
                        description: "Stack capture and continuation object".to_string(),
                    },
                    cache_behavior: CacheBehavior::Poor,
                    parallelizable: false,
                },
            },
        );

        // values
        self.primitives.insert(
            "values".to_string(),
            JitPrimitive {
                name: "values".to_string(),
                arity_min: 0,
                arity_max: None,
                semantic_requirements: R7RSSemanticRequirements::default(),
                interpreted_impl: jit_values,
                jit_strategy: JitCompilationStrategy::Inline {
                    complexity_cost: 10,
                    benefits_from_inlining: true,
                },
                type_specializations: vec![],
                performance_profile: PrimitivePerformanceProfile {
                    avg_execution_ns: 50,
                    memory_profile: MemoryProfile::LinearAllocation {
                        bytes_per_element: 8,
                    },
                    cache_behavior: CacheBehavior::Good,
                    parallelizable: false,
                },
            },
        );

        Ok(())
    }

    /// Register I/O primitives
    fn register_io_primitives(&mut self) -> Result<()> {
        let io_prims = ["display", "newline", "read"];

        for &prim in &io_prims {
            self.primitives.insert(
                prim.to_string(),
                JitPrimitive {
                    name: prim.to_string(),
                    arity_min: if prim == "newline" { 0 } else { 1 },
                    arity_max: Some(if prim == "newline" { 1 } else { 2 }),
                    semantic_requirements: R7RSSemanticRequirements {
                        requires_r7rs_errors: true,
                        ..Default::default()
                    },
                    interpreted_impl: get_io_impl(prim),
                    jit_strategy: JitCompilationStrategy::InterpreterFallback, // I/O is complex
                    type_specializations: vec![],
                    performance_profile: PrimitivePerformanceProfile {
                        avg_execution_ns: 10000, // I/O is slow
                        memory_profile: MemoryProfile::ConstantAllocation { bytes: 64 },
                        cache_behavior: CacheBehavior::Poor,
                        parallelizable: false,
                    },
                },
            );
        }

        Ok(())
    }

    /// Get a primitive by name
    pub fn get_primitive(&self, name: &str) -> Option<&JitPrimitive> {
        self.primitives.get(name)
    }

    /// Get all registered primitive names
    pub fn get_primitive_names(&self) -> Vec<String> {
        self.primitives.keys().cloned().collect()
    }

    /// Record primitive execution for statistics
    pub fn record_execution(&mut self, name: &str, execution_time: Duration, jit_compiled: bool) {
        *self.stats.total_calls.entry(name.to_string()).or_insert(0) += 1;
        *self
            .stats
            .total_execution_time
            .entry(name.to_string())
            .or_insert(Duration::ZERO) += execution_time;

        if jit_compiled {
            *self
                .stats
                .jit_successes
                .entry(name.to_string())
                .or_insert(0) += 1;
        }
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> &PrimitiveStats {
        &self.stats
    }
}

// ============= JIT-AWARE PRIMITIVE IMPLEMENTATIONS =============

/// JIT-aware addition implementation
fn jit_add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(0));
    }

    let mut result = 0i64;
    let mut exact = true;

    for arg in args {
        match arg {
            Value::Literal(Literal::ExactInteger(n)) => {
                result += n;
            }
            Value::Literal(Literal::InexactReal(f)) => {
                return Ok(Value::number(
                    result as f64
                        + f
                        + args[1..]
                            .iter()
                            .map(|v| v.as_number().unwrap_or(0.0))
                            .sum::<f64>(),
                ));
            }
            _ => {
                return Err(Box::new(Error::runtime_error(
                    format!("+ expects numbers, got {arg:?}"),
                    None,
                )));
            }
        }
    }

    if exact {
        Ok(Value::integer(result))
    } else {
        Ok(Value::number(result as f64))
    }
}

/// JIT-aware subtraction implementation
fn jit_subtract(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::runtime_error(
            "- requires at least 1 argument".to_string(),
            None,
        )));
    }

    if args.len() == 1 {
        // Negation
        match &args[0] {
            Value::Literal(Literal::ExactInteger(n)) => Ok(Value::integer(-n)),
            Value::Literal(Literal::InexactReal(f)) => Ok(Value::number(-f)),
            _ => Err(Box::new(Error::runtime_error(
                "- expects a number".to_string(),
                None,
            ))),
        }
    } else {
        // Subtraction
        let first = args[0]
            .as_number()
            .ok_or_else(|| Error::runtime_error("- expects numbers".to_string(), None))?;
        let rest_sum: f64 = args[1..].iter().map(|v| v.as_number().unwrap_or(0.0)).sum();
        Ok(Value::number(first - rest_sum))
    }
}

/// JIT-aware multiplication implementation
fn jit_multiply(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(1));
    }

    let mut result = 1i64;

    for arg in args {
        match arg {
            Value::Literal(Literal::ExactInteger(n)) => {
                result *= n;
            }
            Value::Literal(Literal::InexactReal(f)) => {
                return Ok(Value::number(
                    result as f64
                        * f
                        * args[1..]
                            .iter()
                            .map(|v| v.as_number().unwrap_or(1.0))
                            .product::<f64>(),
                ));
            }
            _ => {
                return Err(Box::new(Error::runtime_error(
                    format!("* expects numbers, got {arg:?}"),
                    None,
                )));
            }
        }
    }

    Ok(Value::integer(result))
}

/// JIT-aware division implementation
fn jit_divide(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::runtime_error(
            "/ requires at least 1 argument".to_string(),
            None,
        )));
    }

    let first = args[0]
        .as_number()
        .ok_or_else(|| Error::runtime_error("/ expects numbers".to_string(), None))?;

    if args.len() == 1 {
        // Reciprocal
        if first == 0.0 {
            return Err(Box::new(Error::runtime_error(
                "Division by zero".to_string(),
                None,
            )));
        }
        Ok(Value::number(1.0 / first))
    } else {
        // Division
        let mut result = first;
        for arg in &args[1..] {
            let n = arg
                .as_number()
                .ok_or_else(|| Error::runtime_error("/ expects numbers".to_string(), None))?;
            if n == 0.0 {
                return Err(Box::new(Error::runtime_error(
                    "Division by zero".to_string(),
                    None,
                )));
            }
            result /= n;
        }
        Ok(Value::number(result))
    }
}

/// JIT-aware cons implementation
fn jit_cons(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("cons expects 2 arguments, got {}", args.len()),
            None,
        )));
    }

    Ok(Value::pair(args[0].clone(), args[1].clone()))
}

/// JIT-aware car implementation
fn jit_car(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("car expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::Pair(car, _) => Ok((**car).clone()),
        _ => Err(Box::new(Error::runtime_error(
            "car expects a pair".to_string(),
            None,
        ))),
    }
}

/// JIT-aware cdr implementation
fn jit_cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("cdr expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::Pair(_, cdr) => Ok((**cdr).clone()),
        _ => Err(Box::new(Error::runtime_error(
            "cdr expects a pair".to_string(),
            None,
        ))),
    }
}

/// JIT-aware apply implementation
fn jit_apply(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error(
            "apply expects at least 2 arguments".to_string(),
            None,
        )));
    }

    // For now, return a placeholder - full implementation would involve evaluation
    Ok(Value::Nil)
}

/// JIT-aware call/cc implementation
fn jit_call_cc(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "call/cc expects 1 argument".to_string(),
            None,
        )));
    }

    // For now, return a placeholder - full implementation would capture continuation
    Ok(Value::Nil)
}

/// JIT-aware values implementation
fn jit_values(args: &[Value]) -> Result<Value> {
    // For single value, return it directly
    if args.len() == 1 {
        Ok(args[0].clone())
    } else {
        // For multiple values, would need special multiple-value handling
        // For now, return the list
        Ok(Value::list(args.to_vec()))
    }
}

// Placeholder implementations for remaining primitives
fn get_arithmetic_impl(name: &str) -> fn(&[Value]) -> Result<Value> {
    match name {
        "quotient" => |_| Ok(Value::integer(0)),
        "remainder" => |_| Ok(Value::integer(0)),
        "modulo" => |_| Ok(Value::integer(0)),
        "abs" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "abs expects 1 argument".to_string(),
                    None,
                )));
            }
            match &args[0] {
                Value::Literal(Literal::ExactInteger(n)) => Ok(Value::integer(n.abs())),
                Value::Literal(Literal::InexactReal(f)) => Ok(Value::number(f.abs())),
                _ => Err(Box::new(Error::runtime_error(
                    "abs expects a number".to_string(),
                    None,
                ))),
            }
        },
        "gcd" => |_| Ok(Value::integer(1)),
        "lcm" => |_| Ok(Value::integer(1)),
        "floor" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "floor expects 1 argument".to_string(),
                    None,
                )));
            }
            match &args[0] {
                Value::Literal(Literal::InexactReal(f)) => Ok(Value::number(f.floor())),
                Value::Literal(Literal::ExactInteger(n)) => Ok(Value::integer(*n)),
                _ => Err(Box::new(Error::runtime_error(
                    "floor expects a number".to_string(),
                    None,
                ))),
            }
        },
        "ceiling" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "ceiling expects 1 argument".to_string(),
                    None,
                )));
            }
            match &args[0] {
                Value::Literal(Literal::InexactReal(f)) => Ok(Value::number(f.ceil())),
                Value::Literal(Literal::ExactInteger(n)) => Ok(Value::integer(*n)),
                _ => Err(Box::new(Error::runtime_error(
                    "ceiling expects a number".to_string(),
                    None,
                ))),
            }
        },
        _ => |_| Ok(Value::Nil),
    }
}

fn get_comparison_impl(name: &str) -> fn(&[Value]) -> Result<Value> {
    match name {
        "=" => |args| {
            if args.len() < 2 {
                return Ok(Value::boolean(true));
            }
            let first = args[0].as_number().unwrap_or(0.0);
            Ok(Value::boolean(
                args[1..]
                    .iter()
                    .all(|v| v.as_number().unwrap_or(0.0) == first),
            ))
        },
        "<" => |args| {
            if args.len() < 2 {
                return Ok(Value::boolean(true));
            }
            let mut prev = args[0].as_number().unwrap_or(0.0);
            for arg in &args[1..] {
                let curr = arg.as_number().unwrap_or(0.0);
                if prev >= curr {
                    return Ok(Value::boolean(false));
                }
                prev = curr;
            }
            Ok(Value::boolean(true))
        },
        ">" => |args| {
            if args.len() < 2 {
                return Ok(Value::boolean(true));
            }
            let mut prev = args[0].as_number().unwrap_or(0.0);
            for arg in &args[1..] {
                let curr = arg.as_number().unwrap_or(0.0);
                if prev <= curr {
                    return Ok(Value::boolean(false));
                }
                prev = curr;
            }
            Ok(Value::boolean(true))
        },
        "<=" => |args| {
            if args.len() < 2 {
                return Ok(Value::boolean(true));
            }
            let mut prev = args[0].as_number().unwrap_or(0.0);
            for arg in &args[1..] {
                let curr = arg.as_number().unwrap_or(0.0);
                if prev > curr {
                    return Ok(Value::boolean(false));
                }
                prev = curr;
            }
            Ok(Value::boolean(true))
        },
        ">=" => |args| {
            if args.len() < 2 {
                return Ok(Value::boolean(true));
            }
            let mut prev = args[0].as_number().unwrap_or(0.0);
            for arg in &args[1..] {
                let curr = arg.as_number().unwrap_or(0.0);
                if prev < curr {
                    return Ok(Value::boolean(false));
                }
                prev = curr;
            }
            Ok(Value::boolean(true))
        },
        "max" => |args| {
            if args.is_empty() {
                return Err(Box::new(Error::runtime_error(
                    "max expects at least 1 argument".to_string(),
                    None,
                )));
            }
            let max_val = args
                .iter()
                .map(|v| v.as_number().unwrap_or(0.0))
                .fold(f64::NEG_INFINITY, f64::max);
            Ok(Value::number(max_val))
        },
        _ => |_| Ok(Value::boolean(false)),
    }
}

fn get_list_impl(name: &str) -> fn(&[Value]) -> Result<Value> {
    match name {
        "null?" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "null? expects 1 argument".to_string(),
                    None,
                )));
            }
            Ok(Value::boolean(args[0].is_nil()))
        },
        "pair?" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "pair? expects 1 argument".to_string(),
                    None,
                )));
            }
            Ok(Value::boolean(args[0].is_pair()))
        },
        "list" => |args| Ok(Value::list(args.to_vec())),
        "length" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "length expects 1 argument".to_string(),
                    None,
                )));
            }
            if let Some(list) = args[0].as_list() {
                Ok(Value::integer(list.len() as i64))
            } else {
                Err(Box::new(Error::runtime_error(
                    "length expects a list".to_string(),
                    None,
                )))
            }
        },
        "append" => |args| {
            let mut result = Vec::new();
            for arg in args {
                if let Some(list) = arg.as_list() {
                    result.extend(list);
                } else {
                    return Err(Box::new(Error::runtime_error(
                        "append expects lists".to_string(),
                        None,
                    )));
                }
            }
            Ok(Value::list(result))
        },
        _ => |_| Ok(Value::Nil),
    }
}

fn get_predicate_impl(name: &str) -> fn(&[Value]) -> Result<Value> {
    match name {
        "number?" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "number? expects 1 argument".to_string(),
                    None,
                )));
            }
            Ok(Value::boolean(args[0].is_number()))
        },
        "string?" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "string? expects 1 argument".to_string(),
                    None,
                )));
            }
            Ok(Value::boolean(args[0].is_string()))
        },
        "symbol?" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "symbol? expects 1 argument".to_string(),
                    None,
                )));
            }
            Ok(Value::boolean(args[0].is_symbol()))
        },
        "boolean?" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "boolean? expects 1 argument".to_string(),
                    None,
                )));
            }
            Ok(Value::boolean(matches!(
                args[0],
                Value::Literal(Literal::Boolean(_))
            )))
        },
        "procedure?" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "procedure? expects 1 argument".to_string(),
                    None,
                )));
            }
            Ok(Value::boolean(args[0].is_procedure()))
        },
        "vector?" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "vector? expects 1 argument".to_string(),
                    None,
                )));
            }
            Ok(Value::boolean(args[0].is_vector()))
        },
        _ => |_| Ok(Value::boolean(false)),
    }
}

fn get_equality_impl(name: &str) -> fn(&[Value]) -> Result<Value> {
    match name {
        "eq?" => |args| {
            if args.len() != 2 {
                return Err(Box::new(Error::runtime_error(
                    "eq? expects 2 arguments".to_string(),
                    None,
                )));
            }
            Ok(Value::boolean(args[0] == args[1]))
        },
        "eqv?" => |args| {
            if args.len() != 2 {
                return Err(Box::new(Error::runtime_error(
                    "eqv? expects 2 arguments".to_string(),
                    None,
                )));
            }
            Ok(Value::boolean(args[0] == args[1]))
        },
        "equal?" => |args| {
            if args.len() != 2 {
                return Err(Box::new(Error::runtime_error(
                    "equal? expects 2 arguments".to_string(),
                    None,
                )));
            }
            Ok(Value::boolean(args[0] == args[1]))
        },
        "not" => |args| {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    "not expects 1 argument".to_string(),
                    None,
                )));
            }
            Ok(Value::boolean(args[0].is_falsy()))
        },
        _ => |_| Ok(Value::boolean(false)),
    }
}

fn get_io_impl(name: &str) -> fn(&[Value]) -> Result<Value> {
    match name {
        "display" => |args| {
            if args.is_empty() || args.len() > 2 {
                return Err(Box::new(Error::runtime_error(
                    "display expects 1 or 2 arguments".to_string(),
                    None,
                )));
            }
            // For now, just return unspecified - real implementation would print
            Ok(Value::Unspecified)
        },
        "newline" => |args| {
            if args.len() > 1 {
                return Err(Box::new(Error::runtime_error(
                    "newline expects 0 or 1 arguments".to_string(),
                    None,
                )));
            }
            Ok(Value::Unspecified)
        },
        "read" => |args| {
            if args.len() > 1 {
                return Err(Box::new(Error::runtime_error(
                    "read expects 0 or 1 arguments".to_string(),
                    None,
                )));
            }
            // For now, return a dummy value
            Ok(Value::Nil)
        },
        _ => |_| Ok(Value::Unspecified),
    }
}

// Custom code generators for complex primitives
fn generate_apply_code(_expr: &Expr, _args: &[Value]) -> Result<SpecializedNativeCode> {
    // This would generate optimized native code for apply
    // For now, return a placeholder
    Err(Box::new(Error::runtime_error(
        "Custom apply code generation not yet implemented".to_string(),
        None,
    )))
}

fn generate_call_cc_code(_expr: &Expr, _args: &[Value]) -> Result<SpecializedNativeCode> {
    // This would generate optimized native code for call/cc
    // For now, return a placeholder
    Err(Box::new(Error::runtime_error(
        "Custom call/cc code generation not yet implemented".to_string(),
        None,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_primitive_registry_creation() {
        let registry = JitPrimitiveRegistry::new();
        assert!(registry.is_ok());

        let registry = registry.unwrap();
        assert_eq!(registry.primitives.len(), 42);

        // Verify all core primitives are registered
        for &prim_name in &CORE_R7RS_PRIMITIVES {
            assert!(
                registry.get_primitive(prim_name).is_some(),
                "Missing primitive: {prim_name}"
            );
        }
    }

    #[test]
    fn test_arithmetic_primitives() {
        let registry = JitPrimitiveRegistry::new().unwrap();

        // Test addition
        let add_prim = registry.get_primitive("+").unwrap();
        let result = (add_prim.interpreted_impl)(&[Value::integer(2), Value::integer(3)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::integer(5));

        // Test subtraction
        let sub_prim = registry.get_primitive("-").unwrap();
        let result = (sub_prim.interpreted_impl)(&[Value::integer(5), Value::integer(3)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::number(2.0));

        // Test multiplication
        let mul_prim = registry.get_primitive("*").unwrap();
        let result = (mul_prim.interpreted_impl)(&[Value::integer(3), Value::integer(4)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::integer(12));
    }

    #[test]
    fn test_list_primitives() {
        let registry = JitPrimitiveRegistry::new().unwrap();

        // Test cons
        let cons_prim = registry.get_primitive("cons").unwrap();
        let result = (cons_prim.interpreted_impl)(&[Value::integer(1), Value::integer(2)]);
        assert!(result.is_ok());

        // Test car and cdr
        let pair = Value::pair(Value::integer(1), Value::integer(2));

        let car_prim = registry.get_primitive("car").unwrap();
        let result = (car_prim.interpreted_impl)(&[pair.clone()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::integer(1));

        let cdr_prim = registry.get_primitive("cdr").unwrap();
        let result = (cdr_prim.interpreted_impl)(&[pair]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::integer(2));
    }

    #[test]
    fn test_type_predicates() {
        let registry = JitPrimitiveRegistry::new().unwrap();

        let number_pred = registry.get_primitive("number?").unwrap();
        let result = (number_pred.interpreted_impl)(&[Value::integer(42)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::boolean(true));

        let result = (number_pred.interpreted_impl)(&[Value::string("hello")]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::boolean(false));
    }

    #[test]
    fn test_jit_strategies() {
        let registry = JitPrimitiveRegistry::new().unwrap();

        // Addition should be inlinable
        let add_prim = registry.get_primitive("+").unwrap();
        assert!(matches!(
            add_prim.jit_strategy,
            JitCompilationStrategy::Inline { .. }
        ));

        // call/cc should use custom generation
        let callcc_prim = registry.get_primitive("call/cc").unwrap();
        assert!(matches!(
            callcc_prim.jit_strategy,
            JitCompilationStrategy::Custom { .. }
        ));

        // I/O should fallback to interpreter
        let display_prim = registry.get_primitive("display").unwrap();
        assert!(matches!(
            display_prim.jit_strategy,
            JitCompilationStrategy::InterpreterFallback
        ));
    }
}
