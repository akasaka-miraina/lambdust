//! Formal Verification Module
//!
//! このモジュールは包括的な形式的検証システムを実装します。
//! Lambdustで導入されたすべての理論的革新の数学的正確性を保証します。
//!
//! ## モジュール構成
//!
//! - `configuration_types`: 設定と基本型定義（VerificationConfiguration, 結果型等）
//! - `verification_engine`: メイン検証エンジン（全検証活動の調整）
//! - `proof_generation`: 形式証明生成システム（意味論的同値性証明、正確性証明）
//! - `external_prover_integration`: 外部証明器統合（Agda, Coq, Lean等）
//! - `property_management`: 形式的性質データベースと正確性保証管理

pub mod configuration_types;
pub mod verification_engine;
pub mod proof_generation;
pub mod external_prover_integration;
pub mod property_management;

// Re-export main types for backward compatibility
pub use configuration_types::{
    VerificationConfiguration, VerificationDepth, VerificationStatistics,
    CachedVerificationResult, FormalVerificationResult, FormalVerificationStatus,
    TheoremProvingResult, TheoremProvingStatus, ExternalProverResult, ExternalProverStatus,
    VerificationTimingBreakdown, FormalProof, FormalProofType, ProofStep, 
    ProofVerificationStatus, VerificationEvidence,
};

pub use verification_engine::FormalVerificationEngine;

pub use proof_generation::{ProofGenerationSystem, TheoremProvingHelper};

pub use external_prover_integration::{
    ExternalProverIntegration, ExternalProverStatistics, ExternalProverTimeouts,
    ExternalProverHelper,
};

pub use property_management::{
    FormalPropertyDatabase, FormalProperty, FormalPropertyType,
    CorrectnessGuaranteeManager, CorrectnessGuarantee, GuaranteeType,
    GuaranteeScope, GuaranteeValidity, GuaranteeViolation, ViolationSeverity,
    GuaranteeStatistics,
};

/// Create a new formal verification engine with default configuration
pub fn create_formal_verification_engine() -> FormalVerificationEngine {
    FormalVerificationEngine::new()
}

/// Create a configured formal verification engine
pub fn create_configured_verification_engine(
    config: VerificationConfiguration,
) -> FormalVerificationEngine {
    FormalVerificationEngine::with_config(config)
}

/// Quick verification for simple expressions
pub fn quick_verify(
    expr: &crate::ast::Expr,
    env: &std::rc::Rc<crate::environment::Environment>,
    result: &crate::evaluator::EvaluationResult,
) -> crate::error::Result<FormalVerificationResult> {
    let mut engine = FormalVerificationEngine::new();
    engine.verify_formally(
        expr, 
        env, 
        result, 
        crate::evaluator::RuntimeOptimizationLevel::None
    )
}

/// Comprehensive verification with all features enabled
pub fn comprehensive_verify(
    expr: &crate::ast::Expr,
    env: &std::rc::Rc<crate::environment::Environment>,
    result: &crate::evaluator::EvaluationResult,
) -> crate::error::Result<FormalVerificationResult> {
    let config = VerificationConfiguration {
        enable_correctness_proofs: true,
        enable_semantic_verification: true,
        enable_theorem_proving: true,
        enable_external_provers: true,
        max_verification_time: std::time::Duration::from_secs(60),
        cache_results: true,
        generate_formal_proofs: true,
        verification_depth: VerificationDepth::Comprehensive,
        required_confidence: 0.95,
    };
    
    let mut engine = FormalVerificationEngine::with_config(config);
    engine.verify_formally(
        expr, 
        env, 
        result, 
        crate::evaluator::RuntimeOptimizationLevel::Aggressive
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::evaluator::EvaluationResult;
    use crate::lexer::SchemeNumber;
    use crate::value::Value;
    use crate::environment::Environment;
    use std::rc::Rc;

    #[test]
    fn test_engine_creation() {
        let engine = create_formal_verification_engine();
        let stats = engine.get_statistics();
        assert_eq!(stats.total_verifications, 0);
    }

    #[test]
    fn test_configured_engine_creation() {
        let config = VerificationConfiguration {
            verification_depth: VerificationDepth::Mathematical,
            required_confidence: 0.99,
            ..Default::default()
        };
        
        let engine = create_configured_verification_engine(config);
        assert_eq!(engine.get_config().verification_depth, VerificationDepth::Mathematical);
        assert_eq!(engine.get_config().required_confidence, 0.99);
    }

    #[test]
    fn test_quick_verify() {
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        let result = EvaluationResult {
            value: Value::Number(SchemeNumber::Integer(42)),
            computation_time: std::time::Duration::from_millis(1),
        };

        let verification_result = quick_verify(&expr, &env, &result);
        assert!(verification_result.is_ok());
    }

    #[test]
    fn test_comprehensive_verify() {
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        let result = EvaluationResult {
            value: Value::Number(SchemeNumber::Integer(42)),
            computation_time: std::time::Duration::from_millis(1),
        };

        let verification_result = comprehensive_verify(&expr, &env, &result);
        assert!(verification_result.is_ok());
    }

    #[test]
    fn test_verification_depth_ordering() {
        assert_ne!(VerificationDepth::Basic, VerificationDepth::Comprehensive);
        assert_eq!(VerificationDepth::Semantic, VerificationDepth::Semantic);
    }

    #[test]
    fn test_verification_status_variants() {
        let verified = FormalVerificationStatus::Verified;
        let validated = FormalVerificationStatus::Validated;
        let failed = FormalVerificationStatus::Failed("test error".to_string());
        
        assert_ne!(verified, validated);
        assert!(matches!(failed, FormalVerificationStatus::Failed(_)));
    }

    #[test]
    fn test_theorem_proving_status() {
        let all_proved = TheoremProvingStatus::AllProved;
        let partial = TheoremProvingStatus::PartiallyProved;
        
        assert_ne!(all_proved, partial);
        assert_eq!(all_proved, TheoremProvingStatus::AllProved);
    }

    #[test]
    fn test_external_prover_status() {
        let proved = ExternalProverStatus::Proved;
        let failed = ExternalProverStatus::Failed;
        let error = ExternalProverStatus::Error("test error".to_string());
        
        assert_ne!(proved, failed);
        assert!(matches!(error, ExternalProverStatus::Error(_)));
    }

    #[test]
    fn test_formal_proof_types() {
        let semantic = FormalProofType::SemanticEquivalence;
        let correctness = FormalProofType::Correctness;
        let custom = FormalProofType::Custom("test proof".to_string());
        
        assert!(matches!(semantic, FormalProofType::SemanticEquivalence));
        assert!(matches!(correctness, FormalProofType::Correctness));
        assert!(matches!(custom, FormalProofType::Custom(_)));
    }

    #[test]
    fn test_proof_verification_status() {
        let verified = ProofVerificationStatus::Verified;
        let pending = ProofVerificationStatus::Pending;
        let failed = ProofVerificationStatus::Failed("test failure".to_string());
        
        assert_ne!(verified, pending);
        assert!(matches!(failed, ProofVerificationStatus::Failed(_)));
    }
}