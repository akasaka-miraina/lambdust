//! Complete Formal Verification System Module
//!
//! このモジュールは世界初の完全形式検証システムfor Scheme処理系を提供します。
//! 数学的保証をすべての評価結果に対して提供し、SemanticEvaluator、
//! RuntimeExecutor、EvaluatorInterfaceの完全な分離と一貫性を検証します。
//!
//! ## モジュール構成
//!
//! - `verification_types`: 基本型、結果構造体、設定、メトリクス
//! - `component_verifiers`: 個別コンポーネント検証器
//! - `consistency_verifiers`: 一貫性と責務分離検証システム
//! - `core_system`: メイン検証システムと実装

pub mod verification_types;
pub mod component_verifiers;
pub mod consistency_verifiers;
pub mod core_system;

// Re-export main types for backward compatibility
pub use verification_types::{
    CompleteVerificationConfig, ComprehensiveVerificationMetrics,
    SystemCorrectnessGuarantees, CompleteSystemVerificationResult,
    CrossComponentVerificationResult, ComponentConsistencyResult,
    ResponsibilitySeparationResult, SemanticVerificationResult,
    RuntimeVerificationResult, InterfaceVerificationResult,
    CorrectnessGuarantee, GuaranteeLevel, VerificationEvidence,
    EvidenceType, CoverageMetrics, QualityMetrics,
    ComplianceGuarantee, PerformanceGuarantee, MemorySafetyGuarantee,
    DeterminismGuarantee, SecurityGuarantee, SeparationGuarantee,
    ConsistencyViolation, VerificationTest, CICDVerificationSuite,
};

pub use component_verifiers::{
    SemanticEvaluatorVerifier, RuntimeExecutorVerifier, EvaluatorInterfaceVerifier,
    MathematicalPurityChecker,
};

pub use consistency_verifiers::{
    ComponentConsistencyVerifier, ResponsibilitySeparationVerifier,
};

pub use core_system::CompleteFormalVerificationSystem;

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::Result;
use crate::value::Value;

/// Create a new complete formal verification system
pub fn create_verification_system() -> Result<CompleteFormalVerificationSystem> {
    CompleteFormalVerificationSystem::new()
}

/// Create a verification system with custom configuration
pub fn create_verification_system_with_config(
    config: CompleteVerificationConfig,
) -> Result<CompleteFormalVerificationSystem> {
    CompleteFormalVerificationSystem::new_with_config(config)
}

/// Perform quick verification for critical expressions
pub fn verify_expression_quickly(
    expr: &Expr,
    env: &Environment,
    result: &Value,
) -> Result<bool> {
    let mut system = CompleteFormalVerificationSystem::new()?;
    system.verify_lightweight(expr, env, result)
}

/// Verify cross-component consistency for two results
pub fn verify_component_consistency(
    expr: &Expr,
    env: &Environment,
    semantic_result: &Value,
    runtime_result: &Value,
) -> Result<CrossComponentVerificationResult> {
    let mut system = CompleteFormalVerificationSystem::new()?;
    system.verify_cross_component_consistency(expr, env, semantic_result, runtime_result)
}