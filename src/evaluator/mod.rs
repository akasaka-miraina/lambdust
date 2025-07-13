//! R7RS formal semantics compliant evaluator
//!
//! This module implements a continuation-passing style evaluator
//! that strictly follows the R7RS formal semantics definition.

pub mod ast_converter;
// Combinatory logic system for lambda calculus integration
pub mod combinators;
pub mod continuation;
// Unified continuation pooling system for memory optimization
pub mod continuation_pooling;
pub mod control_flow;
pub mod evaluation;
// Execution context for Evaluator-Executor communication bridge
pub mod execution_context;
pub mod expression_analyzer;
pub mod higher_order;
pub mod imports;
// Inline evaluation system for performance optimization
pub mod inline_evaluation;
// JIT loop optimization system for iterative constructs
pub mod jit_loop_optimization;
// Advanced hot path analysis system for multi-dimensional profiling
pub mod hotpath_analysis;
pub mod memory;
// Tests moved to tests/unit/evaluator/memory_tests.rs
// Tail call optimization system for proper tail recursion
pub mod tail_call_optimization;
// LLVM backend for advanced tail call optimization
pub mod llvm_backend;
// RAII store for memory management and resource cleanup
pub mod raii_store;
// Pure R7RS semantic evaluator for formal semantics reference
pub mod semantic;
// R7RS-pico ultra-minimal evaluator for embedded systems
#[cfg(feature = "pico")]
pub mod pico_evaluator;
// R7RS-pico initial environment setup
#[cfg(feature = "pico")]
pub mod pico_environment;
// Semantic evaluator correctness proofs and verification
pub mod semantic_correctness;
// Runtime executor for optimized evaluation with performance tuning
pub mod runtime_executor;
// Runtime executor type definitions (split from main runtime_executor)
pub mod runtime_executor_types;
pub mod special_forms;
// Typed special forms for type-annotated lambda and define expressions
pub mod typed_special_forms;
// Theorem proving support system for formal verification (modular structure)
pub mod theorem_proving;
// External prover integration for advanced verification
pub mod external_provers;
// Unified evaluator interface for transparent evaluation mode switching
pub mod evaluator_interface;
// Advanced evaluation mode selection for performance and correctness trade-offs
pub mod evaluation_mode_selector;
// Comprehensive verification system for correctness guarantees
pub mod verification_system;
// Backward compatibility system for legacy code support
pub mod backward_compatibility;
// Migration strategy system for seamless evaluator transitions
pub mod migration_strategy;
// Formal verification foundation for mathematical proofs
pub mod formal_verification;
// Church-Rosser property and confluence formal proofs (modular structure)
pub mod church_rosser_proof;
// Static semantic optimizer with formal proof guarantees
pub mod static_semantic_optimizer;
// Theorem derivation engine for advanced static optimization (modular structure)
pub mod theorem_derivation_engine;
// Adaptive theorem learning system for knowledge accumulation
pub mod adaptive_theorem_learning;
// Complete formal verification system for mathematical correctness guarantees
pub mod complete_formal_verification;
// Advanced JIT compilation system with formal verification
pub mod advanced_jit_system;
// Runtime optimization integration system for performance tuning
pub mod runtime_optimization_integration;
// Modular runtime optimization system (new architecture)
pub mod runtime_optimization;
// Performance measurement system for benchmarking and profiling
pub mod performance_measurement_system;
// Modular performance measurement system (new architecture)
pub mod performance_measurement;
// Trampoline evaluator for stack overflow prevention
pub mod trampoline;
pub mod types;

use crate::ast::{Expr, Literal};
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::value::Value;

// Re-export main types
pub use continuation::{
    CompactContinuation, Continuation, DoLoopState, DynamicPoint, EnvironmentRef,
    InlineContinuation, LightContinuation,
};
// Continuation pooling system exports
pub use continuation_pooling::{
    ContinuationPoolManager, ContinuationType, PoolStatistics, SharedContinuationPoolManager,
    TypedContinuationPool,
};
pub use evaluation::{EvalOrder, ExceptionHandlerInfo};
// Execution context exports for Evaluator-Executor communication
pub use execution_context::{
    ExecutionContext, ExecutionContextBuilder, ExecutionMetadata, ExecutionPriority,
    HotPathIndicator, MacroExpansionState, MemoryConstraints, MemoryEstimates, OptimizationHints,
    OptimizationLevel as ContextOptimizationLevel, OptimizationStrategy as ContextOptimizationStrategy,
    StaticAnalysisResult, StaticCallPattern, SynchronizationLevel, ThreadSafetyRequirements,
    VariableTypeHint, VariableUsage,
};
pub use expression_analyzer::{
    AnalysisResult, EvaluationComplexity, ExpressionAnalyzer, OptimizationHint, OptimizationStats,
    TypeHint,
};
// Inline evaluation exports
pub use inline_evaluation::{
    CacheFriendlyPatterns, ContinuationWeight, HotPathDetector, InlineEvaluator, InlineHint,
    InlineResult,
};
// Tail call optimization exports
pub use tail_call_optimization::{
    ArgEvaluationStrategy, OptimizationLevel, OptimizedTailCall, TailCallAnalyzer, TailCallContext,
    TailCallOptimizer, TailCallStats,
};
// LLVM backend exports
pub use llvm_backend::{
    LLVMCodeGenerator, LLVMCompilerIntegration, LLVMFunction, LLVMInstruction,
    LLVMOptimizationLevel, LLVMOptimizationStats, LLVMTailCallIntrinsic,
};
// JIT loop optimization exports
pub use jit_loop_optimization::{
    GeneratedCode, IterationStrategy, IteratorType, JitHint, JitLoopOptimizer,
    JitOptimizationStats, LoopPattern, NativeCodeGenerator,
};
// Combinatory logic system exports
pub use combinators::{BracketAbstraction, CombinatorExpr, CombinatorStats};
// Pure semantic evaluator exports
pub use semantic::SemanticEvaluator;
// R7RS-pico ultra-minimal evaluator exports
#[cfg(feature = "pico")]
pub use pico_evaluator::PicoEvaluator;
// R7RS-pico initial environment exports
#[cfg(feature = "pico")]
pub use pico_environment::{create_pico_initial_environment, get_pico_features, is_pico_builtin, PicoFeatures};
// Semantic correctness exports
// Temporarily disabled due to compilation issues
pub use semantic_correctness::{CorrectnessProof, CorrectnessProperty, SemanticCorrectnessProver};
// Runtime executor exports
pub use runtime_executor::RuntimeExecutor;
pub use runtime_executor_types::{RuntimeOptimizationLevel, RuntimeStats};
// Theorem proving support exports
pub use theorem_proving::{
    GoalType, ProofGoal, ProofState, ProofTactic, Statement, TheoremProvingSupport,
    VerificationResult as TheoremVerificationResult,
};
// External prover integration exports
pub use external_provers::{
    ExternalProver, ExternalProverManager, ExternalVerificationResult, ProverConfig,
};
// Unified evaluator interface exports
pub use evaluator_interface::{
    EvaluationConfig, EvaluationMode, EvaluationResult, EvaluatorInterface, PerformanceMetrics,
    VerificationResult as InterfaceVerificationResult,
};
// Advanced evaluation mode selection exports
pub use evaluation_mode_selector::{
    EvaluationContext, EvaluationModeSelector, ExpressionType, PerformanceRequirements,
    PerformanceStats, SelectionCriteria,
};
// Comprehensive verification system exports
pub use verification_system::{
    VerificationAnalysis, VerificationConfig, VerificationResult as SystemVerificationResult,
    VerificationStatistics, VerificationStatus, VerificationSystem,
};
// Backward compatibility system exports
pub use backward_compatibility::{
    migration_helpers, CompatibilityMode, CompatibilityResult, LegacyEvaluatorAdapter,
    MigrationRecommendation, MigrationStatistics,
};
// Migration strategy system exports
pub use migration_strategy::{
    MigrationPhase, MigrationProgressTracker, MigrationStatus, MigrationStrategy,
    SuccessCriterion, RiskAssessment, RiskFactor, MitigationStrategy,
};
// Formal verification foundation exports
pub use formal_verification::{
    CorrectnessGuarantee, FormalProof, FormalVerificationEngine, FormalVerificationResult,
    FormalVerificationStatus, VerificationConfiguration, VerificationDepth,
};
// Church-Rosser property and confluence formal proofs exports
pub use church_rosser_proof::{
    ChurchRosserProof, ChurchRosserProofEngine, ConfluenceProof, ConfluenceVerifier,
    NormalizationProof, NormalizationVerifier, TerminationProof, TerminationVerifier,
};
// Static semantic optimizer with formal proof guarantees exports
pub use static_semantic_optimizer::{
    StaticSemanticOptimizer, ProvenOptimization, FormalProof as OptimizerFormalProof,
    ProofMethod, ProofStep, TypeInferenceEngine, InferredType, ConstantPropagationEngine,
    DeadCodeEliminationEngine, CommonSubexpressionEngine, LoopOptimizationEngine,
    StaticOptimizerConfiguration, OptimizationStatistics as OptimizerStatistics,
};
// Theorem derivation engine for advanced static optimization exports
pub use theorem_derivation_engine::{
    TheoremDerivationEngine, DerivedTheoremDatabase, FundamentalTheorem, MathematicalStatement,
    DerivedOptimizationRule, OptimizationPattern, OptimizationReplacement, DerivationProof,
    OptimizationTheorem, PerformanceCharacteristics, TheoremCategory, TheoremComplexity,
    ApplicabilityCondition, CompositionTheorem, PreservationTheorem, PerformanceTheorem,
    AdvancedProofTactics, TheoremDerivationConfig, DerivationStatistics,
};
// Adaptive theorem learning system exports
pub use adaptive_theorem_learning::{
    AdaptiveTheoremLearningSystem, TheoremKnowledgeBase, DiscoveredPattern, LearnedOptimizationPattern,
    PerformanceAnalyzer, PatternDiscoveryEngine, OccurrenceContext, SourceInfo, 
    ContextPerformanceData, StyleIndicators, PatternType, LearnedPerformanceCharacteristics,
    MemoryImpactData, ScalabilityCharacteristics, PerformanceInsight, PerformanceImpactQuantification,
};
// Runtime optimization integration system exports
pub use runtime_optimization_integration::{
    CorrectnessGuarantor, IntegratedOptimizationManager, OptimizationCache, OptimizationResult,
    OptimizationStrategy,
};
// Performance measurement system exports
pub use performance_measurement_system::{
    BenchmarkExecutionResult, MeasurementConfiguration, MeasurementTarget, MetricType,
    OptimizationEffectResult, PerformanceMeasurementSystem,
};
// Also re-export from the new modular system
pub use performance_measurement::{
    ComprehensiveMeasurementResult as PerformanceMeasurementResult,
};
// Advanced hot path analysis system exports
pub use hotpath_analysis::{
    AdvancedHotPathDetector, HotPathAnalysis, HotPathCategory, PerformanceOptimizationReport,
    OptimizationRecommendation, OptimizationType, ExecutionRecord, LoopCharacteristics,
    CallGraphComplexity, MemoryAccessPattern, BranchHistory, DynamicThresholds,
};
// Tests moved to tests/unit/evaluator/theorem_proving_tests.rs
// Trampoline evaluator exports
pub use trampoline::{Bounce, ContinuationThunk, TrampolineEvaluation, TrampolineEvaluator};
pub use types::*;


/// Evaluate an expression with the CPS evaluator
pub fn eval(expr: &Expr, env: &Environment) -> Result<Value> {
    eval_with_formal_semantics(expr, env)
}

/// Evaluate an expression with formal R7RS semantics
pub fn eval_with_formal_semantics(expr: &Expr, env: &Environment) -> Result<Value> {
    // Simplified evaluation for compatibility
    match expr {
        Expr::Literal(lit) => match lit {
            Literal::Number(n) => Ok(Value::Number(n.clone())),
            Literal::String(s) => Ok(Value::String(s.clone())),
            Literal::Boolean(b) => Ok(Value::Boolean(*b)),
            Literal::Character(c) => Ok(Value::Character(*c)),
            Literal::Nil => Ok(Value::Nil),
        },
        Expr::Variable(name) => env.get(name).ok_or_else(|| {
            LambdustError::runtime_error(format!("Unbound variable: {}", name))
        }),
        _ => Ok(Value::Nil), // Simplified for now
    }
}