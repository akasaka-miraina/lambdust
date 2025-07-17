//! Theorem Derivation Engine Module
//!
//! このモジュールは高度な静的最適化のための定理導出システムを実装します。
//! 数学的基礎から新しい最適化定理を導出し、形式的正当性保証を提供します。
//!
//! ## モジュール構成
//!
//! - `theorem_types`: 定理と数学的構造体の型定義
//! - `database`: 導出された定理のデータベースと管理機能
//! - `theorem_engine`: メインの定理導出エンジン
//! - `proof_tactics`: 高度な証明戦術（帰納法、書き換え、置換など）
//! - `performance_verification`: パフォーマンス検証とベンチマーク

pub mod database;
pub mod performance_verification;
pub mod proof_tactics;
pub mod theorem_engine;
pub mod theorem_types;

// Re-export main types for backward compatibility
pub use database::{
    DerivedTheoremDatabase, DatabaseStatistics, TheoremSearchCriteria, TheoremSearchResults,
};

pub use performance_verification::{
    PerformanceVerificationSystem, PerformanceVerificationResult, BenchmarkExecutor,
    BenchmarkTestCase, BenchmarkCategory, StatisticalAnalyzer, MemoryAnalyzer,
    RegressionTester, VerificationConfig,
};

pub use proof_tactics::{
    InductionPrinciple, RewritingStrategy,
    CompositionStrategy, CaseSplittingStrategy,
};

pub use theorem_engine::{
    DerivationStatistics, TheoremDerivationConfig,
};

pub use theorem_types::{
    FundamentalTheorem, MathematicalStatement, DerivedOptimizationRule,
    OptimizationPattern, OptimizationReplacement, PatternElement, PatternCondition,
    DerivationProof, DerivationStep, PerformanceCharacteristics, ComplexityImprovement,
    MemoryChange, OptimizationScope, ApplicabilityCondition, ValueConstraint,
    TheoremCategory, TheoremCondition, CompositionTheorem, CompositionRule,
    InterferenceAnalysis, PreservationTheorem, PerformanceTheorem,
    OptimizationTheorem, PerformanceVerification, BenchmarkResult,
    MemoryComparison, MemoryAnalysis, RegressionTest, TestResult,
    TheoremMetadata, TheoremComplexity, UsageStatistics, StatisticalAnalysis,
    ExperimentalValidation,
};

use crate::evaluator::SemanticEvaluator;

// Main re-exports using proper module paths

/// Advanced proof tactics for theorem derivation
pub type AdvancedProofTactics = proof_tactics::AdvancedProofTactics;

/// Induction tactic for mathematical induction proofs
pub type InductionTactic = proof_tactics::InductionTactic;

/// Rewriting tactic for term rewriting proofs
pub type RewritingTactic = proof_tactics::RewritingTactic;

/// Substitution tactic for variable substitution proofs
pub type SubstitutionTactic = proof_tactics::SubstitutionTactic;

/// Composition tactic for proof composition
pub type CompositionTactic = proof_tactics::CompositionTactic;

/// Case analysis tactic for case splitting proofs
pub type CaseAnalysisTactic = proof_tactics::CaseAnalysisTactic;

/// Main theorem derivation engine
pub type TheoremDerivationEngine = theorem_engine::TheoremDerivationEngine;

/// Create a new theorem derivation engine with default configuration
pub fn create_theorem_derivation_engine(
    theorem_prover: crate::prover::proof_types::TheoremProvingSupport,
    verification_engine: crate::prover::proof_types::FormalVerificationEngine,
    semantic_evaluator: SemanticEvaluator,
) -> TheoremDerivationEngine {
    theorem_engine::TheoremDerivationEngine::new(
        theorem_prover,
        verification_engine,
        semantic_evaluator,
    )
}

/// Create a default database for derived theorems
pub fn create_default_database() -> DerivedTheoremDatabase {
    database::DerivedTheoremDatabase::new()
}

/// Create a default performance verification system
pub fn create_performance_verification_system() -> PerformanceVerificationSystem {
    performance_verification::PerformanceVerificationSystem::new()
}

/// Create default proof tactics
pub fn create_proof_tactics() -> AdvancedProofTactics {
    proof_tactics::AdvancedProofTactics::new()
}