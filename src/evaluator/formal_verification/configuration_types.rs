//! Configuration and Core Types Module
//!
//! このモジュールは形式的検証システムの設定と基本型定義を提供します。
//! 検証設定、統計情報、結果型、タイミング情報などを含みます。

use crate::evaluator::{
    CorrectnessProof, SystemVerificationResult,
};
use crate::value::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Configuration for formal verification
#[derive(Debug, Clone)]
pub struct VerificationConfiguration {
    /// Enable mathematical correctness proofs
    pub enable_correctness_proofs: bool,

    /// Enable semantic equivalence verification
    pub enable_semantic_verification: bool,

    /// Enable theorem proving verification
    pub enable_theorem_proving: bool,

    /// Enable external prover integration
    pub enable_external_provers: bool,

    /// Maximum verification time per expression
    pub max_verification_time: Duration,

    /// Cache verification results
    pub cache_results: bool,

    /// Generate formal proofs
    pub generate_formal_proofs: bool,

    /// Verification depth level
    pub verification_depth: VerificationDepth,

    /// Required confidence level (0.0 to 1.0)
    pub required_confidence: f64,
}

/// Verification depth levels
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationDepth {
    /// Basic structural verification
    Basic,
    /// Semantic equivalence verification
    Semantic,
    /// Full mathematical proof verification
    Mathematical,
    /// Comprehensive verification with external tools
    Comprehensive,
}

/// Verification statistics
#[derive(Debug, Clone, Default)]
pub struct VerificationStatistics {
    /// Total verifications performed
    pub total_verifications: usize,

    /// Successful verifications
    pub successful_verifications: usize,

    /// Failed verifications
    pub failed_verifications: usize,

    /// Verification timeouts
    pub timeout_verifications: usize,

    /// Average verification time
    pub avg_verification_time: Duration,

    /// Correctness proofs generated
    pub correctness_proofs_generated: usize,

    /// Theorem proving successes
    pub theorem_proving_successes: usize,

    /// External prover calls
    pub external_prover_calls: usize,

    /// Cache hit rate
    pub cache_hit_rate: f64,

    /// Confidence distribution
    pub confidence_distribution: HashMap<String, usize>,
}

/// Cached verification result
#[derive(Debug, Clone)]
pub struct CachedVerificationResult {
    /// Verification result
    pub result: FormalVerificationResult,

    /// Timestamp when cached
    pub cached_at: Instant,

    /// Cache hit count
    pub hit_count: usize,

    /// Expiration time
    pub expires_at: Instant,
}

/// Comprehensive formal verification result
#[derive(Debug, Clone)]
pub struct FormalVerificationResult {
    /// Overall verification status
    pub status: FormalVerificationStatus,

    /// Confidence level (0.0 to 1.0)
    pub confidence_level: f64,

    /// Mathematical correctness proof
    pub correctness_proof: Option<CorrectnessProof>,

    /// Semantic verification result
    pub semantic_verification: Option<SystemVerificationResult>,

    /// Theorem proving result
    pub theorem_proving_result: Option<TheoremProvingResult>,

    /// External prover results
    pub external_prover_results: Vec<ExternalProverResult>,

    /// Verification time breakdown
    pub timing_breakdown: VerificationTimingBreakdown,

    /// Generated formal proofs
    pub formal_proofs: Vec<FormalProof>,

    /// Verification evidence
    pub evidence: VerificationEvidence,
}

/// Formal verification status
#[derive(Debug, Clone, PartialEq)]
pub enum FormalVerificationStatus {
    /// Verification passed with mathematical certainty
    Verified,
    /// Verification passed with high confidence
    Validated,
    /// Verification inconclusive
    Inconclusive,
    /// Verification failed
    Failed(String),
    /// Verification timeout
    Timeout,
    /// Verification error
    Error(String),
}

/// Theorem proving result
#[derive(Debug, Clone)]
pub struct TheoremProvingResult {
    /// Theorem proving status
    pub status: TheoremProvingStatus,

    /// Proved theorems
    pub proved_theorems: Vec<String>,

    /// Failed theorem attempts
    pub failed_theorems: Vec<String>,

    /// Proof tactics used
    pub tactics_used: Vec<String>,

    /// Proof time
    pub proof_time: Duration,
}

/// Theorem proving status
#[derive(Debug, Clone, PartialEq)]
pub enum TheoremProvingStatus {
    /// All theorems proved
    AllProved,
    /// Some theorems proved
    PartiallyProved,
    /// No theorems proved
    NotProved,
    /// Theorem proving failed
    Failed,
}

/// External prover result
#[derive(Debug, Clone)]
pub struct ExternalProverResult {
    /// Prover name
    pub prover_name: String,

    /// Verification status
    pub status: ExternalProverStatus,

    /// Proof output
    pub proof_output: String,

    /// Verification time
    pub verification_time: Duration,

    /// Confidence score
    pub confidence_score: f64,
}

/// External prover status
#[derive(Debug, Clone, PartialEq)]
pub enum ExternalProverStatus {
    /// Proof successful
    Proved,
    /// Proof failed
    Failed,
    /// Prover timeout
    Timeout,
    /// Prover error
    Error(String),
}

/// Verification timing breakdown
#[derive(Debug, Clone)]
pub struct VerificationTimingBreakdown {
    /// Total verification time
    pub total_time: Duration,

    /// Semantic evaluation time
    pub semantic_time: Duration,

    /// Correctness proof time
    pub correctness_proof_time: Duration,

    /// Theorem proving time
    pub theorem_proving_time: Duration,

    /// External prover time
    pub external_prover_time: Duration,

    /// Cache lookup time
    pub cache_lookup_time: Duration,
}

/// Formal proof representation
#[derive(Debug, Clone)]
pub struct FormalProof {
    /// Proof type
    pub proof_type: FormalProofType,

    /// Proof statement
    pub statement: String,

    /// Proof steps
    pub steps: Vec<ProofStep>,

    /// Proof conclusion
    pub conclusion: String,

    /// Proof verification status
    pub verification_status: ProofVerificationStatus,
}

/// Formal proof types
#[derive(Debug, Clone)]
pub enum FormalProofType {
    /// Semantic equivalence proof
    SemanticEquivalence,
    /// Correctness proof
    Correctness,
    /// Termination proof
    Termination,
    /// Type safety proof
    TypeSafety,
    /// R7RS compliance proof
    R7RSCompliance,
    /// Custom mathematical proof
    Custom(String),
}

/// Proof step
#[derive(Debug, Clone)]
pub struct ProofStep {
    /// Step number
    pub step_number: usize,

    /// Step description
    pub description: String,

    /// Applied rule or tactic
    pub rule_applied: String,

    /// Step result
    pub result: String,

    /// Step justification
    pub justification: String,
}

/// Proof verification status
#[derive(Debug, Clone, PartialEq)]
pub enum ProofVerificationStatus {
    /// Proof verified
    Verified,
    /// Proof pending verification
    Pending,
    /// Proof failed verification
    Failed(String),
    /// Proof incomplete
    Incomplete,
}

/// Verification evidence
#[derive(Debug, Clone)]
pub struct VerificationEvidence {
    /// Reference computation trace
    pub reference_trace: Vec<String>,

    /// Comparison evidence
    pub comparison_evidence: Vec<String>,

    /// Mathematical justifications
    pub mathematical_justifications: Vec<String>,

    /// Supporting lemmas
    pub supporting_lemmas: Vec<String>,

    /// Witness values
    pub witness_values: HashMap<String, Value>,
}

impl Default for VerificationConfiguration {
    fn default() -> Self {
        Self {
            enable_correctness_proofs: true,
            enable_semantic_verification: true,
            enable_theorem_proving: true,
            enable_external_provers: false, // Disabled by default for performance
            max_verification_time: Duration::from_secs(10),
            cache_results: true,
            generate_formal_proofs: true,
            verification_depth: VerificationDepth::Semantic,
            required_confidence: 0.9,
        }
    }
}