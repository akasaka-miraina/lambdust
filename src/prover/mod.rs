//! Theorem Proving Support System for Lambdust
//!
//! This module provides comprehensive theorem proving capabilities for the Lambdust Scheme interpreter.
//! All theorem proving functionality is gated behind the `development` feature to keep production
//! builds lean while enabling advanced research capabilities.
//!
//! ## Module Organization
//!
//! - `theorem_proving`: Core theorem proving support and database
//! - `theorem_derivation`: Automatic theorem derivation from base theorems
//! - `adaptive_learning`: Adaptive theorem learning from successful optimizations
//! - `formal_verification`: Complete formal verification system
//! - `church_rosser`: Church-Rosser theorem and confluence proofs
//! - `verification_system`: Comprehensive verification orchestration
//! - `external_provers`: Integration with external theorem provers (Agda, Coq)
//! - `optimization`: Theorem-based optimization derivation

#[cfg(feature = "development")]
pub mod theorem_proving;

#[cfg(feature = "development")]
pub mod theorem_derivation;

#[cfg(feature = "development")]
pub mod adaptive_learning;

#[cfg(feature = "development")]
pub mod formal_verification;

#[cfg(feature = "development")]
pub mod church_rosser;

#[cfg(feature = "development")]
pub mod verification_system;

#[cfg(feature = "development")]
pub mod complete_formal_verification;

#[cfg(feature = "development")]
pub mod external_provers;

#[cfg(feature = "development")]
pub mod optimization;

#[cfg(feature = "development")]
pub mod proof_types;

// Re-export main types for convenient access
#[cfg(feature = "development")]
pub use theorem_proving::{
    ProofState,
    VerificationResult as TheoremVerificationResult,
};

#[cfg(feature = "development")]
pub use theorem_derivation::{
    TheoremDerivationEngine, DerivedTheoremDatabase, FundamentalTheorem, MathematicalStatement,
    DerivedOptimizationRule, OptimizationPattern, OptimizationReplacement, DerivationProof,
    OptimizationTheorem, PerformanceCharacteristics, TheoremCategory, TheoremComplexity,
    ApplicabilityCondition, CompositionTheorem, PreservationTheorem, PerformanceTheorem,
    AdvancedProofTactics, TheoremDerivationConfig, DerivationStatistics,
};

#[cfg(feature = "development")]
pub use adaptive_learning::{
    AdaptiveTheoremLearningSystem, TheoremKnowledgeBase, DiscoveredPattern, LearnedOptimizationPattern,
    PerformanceAnalyzer, PatternDiscoveryEngine, OccurrenceContext, SourceInfo, 
    ContextPerformanceData, StyleIndicators, PatternType, LearnedPerformanceCharacteristics,
    MemoryImpactData, ScalabilityCharacteristics, PerformanceInsight, PerformanceImpactQuantification,
};

#[cfg(feature = "development")]
pub use proof_types::{
    CorrectnessGuarantee, FormalProof, FormalVerificationResult,
    FormalVerificationStatus, VerificationConfiguration, VerificationDepth,
    ProofMethod, ProofStep, ProofTerm, ProofTermType, Statement,
    ProofTransformation, TheoremProvingResult, FormalVerificationEngine,
    TheoremProvingSupport, ProofGoal, GoalType, ProofTactic, ProofResult,
};

// Note: formal_verification types moved to proof_types
// #[cfg(feature = "development")]
// pub use formal_verification::{};

#[cfg(feature = "development")]
pub use church_rosser::{
    ChurchRosserProof, ChurchRosserProofEngine, ConfluenceProof, ConfluenceVerifier,
    NormalizationProof, NormalizationVerifier, TerminationProof, TerminationVerifier,
};

#[cfg(feature = "development")]
pub use verification_system::{
    VerificationAnalysis, VerificationConfig, VerificationResult as SystemVerificationResult,
    VerificationStatistics, VerificationStatus, VerificationSystem,
};

#[cfg(feature = "development")]
pub use complete_formal_verification::{
    CompleteFormalVerificationSystem, CompleteSystemVerificationResult,
    SystemCorrectnessGuarantees, CompleteVerificationConfig,
};

#[cfg(feature = "development")]
pub use external_provers::{
    ExternalProver, ExternalProverManager, ExternalVerificationResult, ProverConfig,
};

#[cfg(feature = "development")]
pub use optimization::{
    StaticSemanticOptimizer, ProvenOptimization, FormalProof as OptimizerFormalProof,
    TypeInferenceEngine, InferredType, ConstantPropagationEngine,
    DeadCodeEliminationEngine, CommonSubexpressionEngine, LoopOptimizationEngine,
    StaticOptimizerConfiguration, OptimizationStatistics as OptimizerStatistics,
};