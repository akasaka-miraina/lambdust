//! Legacy optimization module (deprecated)
//!
//! This module is deprecated and maintained for backward compatibility only.
//! Please use the new optimization system structure:
//!
//! - **Static optimization** (compile-time): `crate::static_optimization`
//! - **Runtime optimization** (dynamic): `crate::evaluator::runtime_optimization`
//! - **Research optimization** (development): `crate::prover::optimization`
//!
//! ## Migration Guide
//!
//! - Replace `crate::optimization::EvolvingOptimizationEngine` with `crate::evaluator::runtime_optimization::EvolvingOptimizationEngine`
//! - Use `crate::static_optimization::StaticOptimizationEngine` for compile-time optimizations
//! - Use `crate::prover::optimization` for formal verification research

#[deprecated(since = "3.0.0", note = "Use crate::evaluator::runtime_optimization::EvolvingOptimizationEngine instead")]
#[cfg(feature = "development")]
pub use crate::evaluator::runtime_optimization::evolving_optimization::EvolvingOptimizationEngine;

#[deprecated(since = "3.0.0", note = "Use crate::prover::optimization instead")]
#[cfg(feature = "development")]
pub use crate::prover::optimization::{InferenceRule, LearnedPattern, TheoremDerivationEngine};

#[deprecated(since = "3.0.0", note = "Use crate::prover::optimization instead")]
#[cfg(feature = "development")]
pub use crate::prover::optimization::{OptimizationController, VerificationSystem, VerifiedOptimization};

// Re-export for backward compatibility
#[deprecated(since = "3.0.0", note = "Use crate::evaluator::runtime_optimization types instead")]
#[cfg(feature = "development")]
pub use crate::evaluator::runtime_optimization::evolving_optimization::{
    PerformanceMetrics, OptimizationResult, EvolutionResult, TrainingResult, OptimizationStatistics
};

// Deprecation notice for the entire module
#[deprecated(since = "3.0.0", note = "This module is deprecated. Use the new optimization system structure.")]
pub const DEPRECATED_MODULE_NOTICE: &str = r#"
This optimization module has been reorganized for better clarity:

1. Static optimization (compile-time): crate::static_optimization
   - ConstantFolder, DeadCodeEliminator, ExpressionRewriter
   - CommonSubexpressionEliminator, LoopOptimizer

2. Runtime optimization (dynamic): crate::evaluator::runtime_optimization  
   - EvolvingOptimizationEngine, IntegratedOptimizationManager
   - Performance monitoring and caching systems

3. Research optimization (development): crate::prover::optimization
   - Formal verification, theorem proving
   - StaticSemanticOptimizer with mathematical proofs

Please update your imports accordingly.
"#;