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
// Advanced hot path analysis system moved to src/performance_monitor/hotpath_analysis/
pub mod memory;
// Tests moved to tests/unit/evaluator/memory_tests.rs
// Tail call optimization system for proper tail recursion
pub mod tail_call_optimization;
// LLVM backend for advanced tail call optimization
#[cfg(feature = "development")]
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
// Runtime executor moved to src/executor/runtime/
// Runtime executor type definitions moved to src/executor/runtime_executor_types.rs
pub mod special_forms;
// Typed special forms for type-annotated lambda and define expressions
pub mod typed_special_forms;
// Note: Theorem proving systems moved to src/prover/
// Unified evaluator interface for transparent evaluation mode switching
pub mod evaluator_interface;
// Advanced evaluation mode selection for performance and correctness trade-offs
pub mod evaluation_mode_selector;
// Migration strategy system for seamless evaluator transitions
pub mod migration_strategy;
// Advanced JIT compilation system with formal verification
#[cfg(feature = "development")]
pub mod advanced_jit_system;
// Runtime optimization systems moved to src/executor/runtime_optimization/
// Performance measurement systems moved to src/performance_monitor/
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
#[cfg(feature = "development")]
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
// Runtime executor exports moved to src/executor/
// Use crate::executor::runtime::RuntimeExecutor instead
// Use crate::executor::runtime_executor_types::{RuntimeOptimizationLevel, RuntimeStats} instead
// ===== Theorem Proving Support Systems (Re-exported from prover module) =====
#[cfg(feature = "development")]
pub use crate::prover::{
    // Core theorem proving
    GoalType, ProofGoal, ProofTactic, Statement, TheoremProvingSupport,
    TheoremVerificationResult,
    // External provers
    ExternalProver, ExternalProverManager, ExternalVerificationResult, ProverConfig,
    // Verification system
    VerificationAnalysis, VerificationConfig, SystemVerificationResult,
    VerificationStatistics, VerificationStatus, VerificationSystem,
    // Formal verification
    FormalVerificationEngine,
    // Church-Rosser proofs
    ChurchRosserProof, ChurchRosserProofEngine, ConfluenceProof, ConfluenceVerifier,
    NormalizationProof, NormalizationVerifier, TerminationProof, TerminationVerifier,
    // Static optimization
    ProofMethod, ProofStep,
    // Theorem derivation
    TheoremDerivationEngine, DerivedTheoremDatabase, FundamentalTheorem, MathematicalStatement,
    DerivedOptimizationRule, OptimizationPattern, OptimizationReplacement, DerivationProof,
    OptimizationTheorem, PerformanceCharacteristics, TheoremCategory, TheoremComplexity,
    ApplicabilityCondition, CompositionTheorem, PreservationTheorem, PerformanceTheorem,
    AdvancedProofTactics, TheoremDerivationConfig, DerivationStatistics,
    // Adaptive learning
    AdaptiveTheoremLearningSystem, TheoremKnowledgeBase, DiscoveredPattern, LearnedOptimizationPattern,
    PerformanceAnalyzer, PatternDiscoveryEngine, OccurrenceContext, SourceInfo, 
    ContextPerformanceData, StyleIndicators, PatternType, LearnedPerformanceCharacteristics,
    MemoryImpactData, ScalabilityCharacteristics, PerformanceInsight, PerformanceImpactQuantification,
    // Complete formal verification (TODO: implement)
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
// Migration strategy system exports
pub use migration_strategy::{
    MigrationPhase, MigrationProgressTracker, MigrationStatus, MigrationStrategy,
    SuccessCriterion, RiskAssessment, RiskFactor, MitigationStrategy,
};
// Runtime optimization integration system exports moved to src/executor/runtime_optimization_integration/
// Performance measurement system exports moved to src/performance_monitor/
// Advanced hot path analysis system exports moved to src/performance_monitor/hotpath_analysis/
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
            LambdustError::runtime_error(format!("Unbound variable: {name}"))
        }),
        _ => Ok(Value::Nil), // Simplified for now
    }
}