//! Formal Verification Module
//!
//! このモジュールは包括的な形式的検証システムを実装します。
//! Lambdustで導入されたすべての理論的革新の数学的正確性を保証します。
//!
//! ## モジュール構成
//!
//! - `core_types`: 基本型定義（ProofObligation, VerificationResult等）
//! - `proof_assistant`: 外部証明支援ツールインターフェース（Agda, Coq, Lean等）
//! - `automatic_prover`: 自動定理証明器（論理式解決、SMTソルバー統合）
//! - `property_tester`: 性質ベーステストシステム
//! - `verification_engine`: メイン検証エンジン（全証明活動の調整）

pub mod core_types;
pub mod proof_assistant;
pub mod automatic_prover;
pub mod property_tester;
pub mod verification_engine;

// Re-export main types for backward compatibility
pub use core_types::{
    ProofObligation, ProofCategory, FormalStatement, Quantifier, QuantifierType,
    ProofPriority, ProofStatus, ProofEvidence, ProofTool, ProofStep,
    VerificationResult, VerificationOutcome, VerificationIssue, IssueSeverity,
    VerificationStatistics, VerificationConfig, ProofObligationManager,
};

pub use proof_assistant::{
    ProofAssistantInterface, ToolConfiguration, ProofSession, ProofSessionState,
};

pub use automatic_prover::{
    AutomaticTheoremProver, ResolutionProver, SMTSolverInterface, LambdustLogicProver,
    ProofStrategy, Clause, ClauseOrigin, ResolutionStrategy, SMTSolver,
    InferenceRule, UniverseRule, CombinatorRule,
};

pub use property_tester::{
    PropertyBasedTester, TestGenerator, PropertySpecification, TestExecutor,
    CounterexampleMinimizer, GenerationStrategy, PropertyOutcome, MinimizationStrategy,
};

pub use verification_engine::{
    FormalVerificationEngine, VerificationReport,
};

use crate::evaluator::SemanticEvaluator;
use crate::type_system::PolynomialUniverseSystem;

/// Create a new formal verification engine with default configuration
pub fn create_formal_verification_engine() -> FormalVerificationEngine {
    FormalVerificationEngine::new(
        SemanticEvaluator::new(),
        PolynomialUniverseSystem::new(),
    )
}

/// Create a configured formal verification engine
pub fn create_configured_verification_engine(
    semantic_evaluator: SemanticEvaluator,
    type_system: PolynomialUniverseSystem,
    config: VerificationConfig,
) -> crate::error::Result<FormalVerificationEngine> {
    let mut engine = FormalVerificationEngine::new(semantic_evaluator, type_system);
    engine.configure(config);
    engine.initialize()?;
    Ok(engine)
}

/// Verify a specific aspect of the system
pub fn verify_system_aspect(
    engine: &mut FormalVerificationEngine,
    aspect: SystemAspect,
) -> crate::error::Result<VerificationResult> {
    let obligation_id = match aspect {
        SystemAspect::UniversePolymorphism => "universe_level_consistency",
        SystemAspect::CombinatoryLogic => "ski_completeness",
        SystemAspect::SemanticCorrectness => "semantic_evaluator_correctness",
        SystemAspect::TypeSystemSoundness => "typeclass_instance_uniqueness",
        SystemAspect::HomotopyTypeTheory => "univalence_consistency",
        SystemAspect::MonadTransformers => "transformer_monad_laws",
    };
    
    engine.verify_obligation(obligation_id)
}

/// System aspects that can be verified
#[derive(Debug, Clone, PartialEq)]
pub enum SystemAspect {
    /// Universe polymorphism correctness
    UniversePolymorphism,
    
    /// Combinatory logic equivalence
    CombinatoryLogic,
    
    /// Semantic evaluator correctness
    SemanticCorrectness,
    
    /// Type system soundness
    TypeSystemSoundness,
    
    /// Homotopy type theory consistency
    HomotopyTypeTheory,
    
    /// Monad transformer composition laws
    MonadTransformers,
}

/// Quick verification of all critical system properties
pub fn quick_verify_all_critical(
    engine: &mut FormalVerificationEngine,
) -> crate::error::Result<Vec<VerificationResult>> {
    let critical_aspects = vec![
        SystemAspect::UniversePolymorphism,
        SystemAspect::CombinatoryLogic,
        SystemAspect::SemanticCorrectness,
        SystemAspect::TypeSystemSoundness,
    ];
    
    let mut results = Vec::new();
    for aspect in critical_aspects {
        match verify_system_aspect(engine, aspect.clone()) {
            Ok(result) => results.push(result),
            Err(e) => {
                eprintln!("Failed to verify {:?}: {}", aspect, e);
            }
        }
    }
    
    Ok(results)
}

/// Generate a comprehensive verification report
pub fn generate_comprehensive_report(
    engine: &FormalVerificationEngine,
) -> VerificationReport {
    engine.generate_report()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = create_formal_verification_engine();
        let stats = engine.get_statistics();
        assert_eq!(stats.total_obligations, 0);
    }

    #[test]
    fn test_engine_initialization() {
        let mut engine = create_formal_verification_engine();
        assert!(engine.initialize().is_ok());
    }

    #[test]
    fn test_verification_report_generation() {
        let engine = create_formal_verification_engine();
        let report = generate_comprehensive_report(&engine);
        assert!(report.overall_confidence >= 0.0);
        assert!(report.overall_confidence <= 1.0);
    }
}