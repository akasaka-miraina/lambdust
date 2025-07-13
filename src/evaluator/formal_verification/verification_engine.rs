//! Verification Engine Module
//!
//! このモジュールはメイン形式的検証エンジンを実装します。
//! 全ての検証活動の調整、結果統合、統計管理を行います。

use super::configuration_types::{
    CachedVerificationResult, FormalVerificationResult, FormalVerificationStatus,
    TheoremProvingResult, TheoremProvingStatus, VerificationConfiguration, 
    VerificationStatistics, VerificationTimingBreakdown
};
use super::external_prover_integration::ExternalProverIntegration;
use super::proof_generation::{ProofGenerationSystem, TheoremProvingHelper};
use super::property_management::{CorrectnessGuaranteeManager, FormalPropertyDatabase};
use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::Result;
use crate::evaluator::{
    ChurchRosserProof, ChurchRosserProofEngine, EvaluationResult, RuntimeOptimizationLevel,
    SemanticEvaluator, SystemVerificationResult, TheoremProvingSupport, VerificationSystem,
};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Formal verification engine that coordinates all verification activities
#[derive(Debug)]
pub struct FormalVerificationEngine {
    /// Semantic evaluator as mathematical reference
    #[allow(dead_code)]
    semantic_evaluator: SemanticEvaluator,

    /// Verification system for runtime results
    verification_system: VerificationSystem,

    /// Theorem proving support for mathematical properties
    theorem_prover: TheoremProvingSupport,

    /// Proof generation system
    proof_generator: ProofGenerationSystem,

    /// External prover integration
    external_prover_integration: ExternalProverIntegration,

    /// Verification configuration
    config: VerificationConfiguration,

    /// Verification statistics and metrics
    statistics: VerificationStatistics,

    /// Cache for verified expressions
    verification_cache: HashMap<String, CachedVerificationResult>,

    /// Formal property database
    #[allow(dead_code)]
    property_database: FormalPropertyDatabase,

    /// Correctness guarantees manager
    #[allow(dead_code)]
    correctness_guarantees: CorrectnessGuaranteeManager,

    /// Church-Rosser proof engine
    church_rosser_engine: ChurchRosserProofEngine,
}

impl FormalVerificationEngine {
    /// Create new formal verification engine
    #[must_use] 
    pub fn new() -> Self {
        Self {
            semantic_evaluator: SemanticEvaluator::new(),
            verification_system: VerificationSystem::new(),
            theorem_prover: TheoremProvingSupport::new(SemanticEvaluator::new()),
            proof_generator: ProofGenerationSystem::new(),
            external_prover_integration: ExternalProverIntegration::new(),
            config: VerificationConfiguration::default(),
            statistics: VerificationStatistics::default(),
            verification_cache: HashMap::new(),
            property_database: FormalPropertyDatabase::new(),
            correctness_guarantees: CorrectnessGuaranteeManager::new(),
            church_rosser_engine: ChurchRosserProofEngine::new(),
        }
    }

    /// Create with custom configuration
    #[must_use] 
    pub fn with_config(config: VerificationConfiguration) -> Self {
        Self {
            semantic_evaluator: SemanticEvaluator::new(),
            verification_system: VerificationSystem::new(),
            theorem_prover: TheoremProvingSupport::new(SemanticEvaluator::new()),
            proof_generator: ProofGenerationSystem::new(),
            external_prover_integration: ExternalProverIntegration::new(),
            config,
            statistics: VerificationStatistics::default(),
            verification_cache: HashMap::new(),
            property_database: FormalPropertyDatabase::new(),
            correctness_guarantees: CorrectnessGuaranteeManager::new(),
            church_rosser_engine: ChurchRosserProofEngine::new(),
        }
    }

    /// Perform comprehensive formal verification
    pub fn verify_formally(
        &mut self,
        expr: &Expr,
        env: &Rc<Environment>,
        runtime_result: &EvaluationResult,
        optimization_level: RuntimeOptimizationLevel,
    ) -> Result<FormalVerificationResult> {
        let start_time = Instant::now();

        // Check cache first
        if self.config.cache_results {
            let cache_key = self.generate_cache_key(expr, &optimization_level);
            if let Some(cached) = self.verification_cache.get_mut(&cache_key) {
                cached.hit_count += 1;
                if cached.expires_at > Instant::now() {
                    self.statistics.cache_hit_rate = (self.statistics.cache_hit_rate
                        * (self.statistics.total_verifications - 1) as f64
                        + 1.0)
                        / self.statistics.total_verifications as f64;
                    return Ok(cached.result.clone());
                }
            }
        }

        // Initialize timing breakdown
        let mut timing = VerificationTimingBreakdown {
            total_time: Duration::new(0, 0),
            semantic_time: Duration::new(0, 0),
            correctness_proof_time: Duration::new(0, 0),
            theorem_proving_time: Duration::new(0, 0),
            external_prover_time: Duration::new(0, 0),
            cache_lookup_time: start_time.elapsed(),
        };

        // Perform semantic verification
        let semantic_verification = if self.config.enable_semantic_verification {
            let semantic_start = Instant::now();
            let result = self.verification_system.verify_execution(
                expr,
                env,
                &crate::evaluator::Continuation::Identity,
                &runtime_result.value,
                optimization_level,
            )?;
            timing.semantic_time = semantic_start.elapsed();
            Some(result)
        } else {
            None
        };

        // Generate correctness proof
        let correctness_proof = if self.config.enable_correctness_proofs {
            let proof_start = Instant::now();
            let result = self.proof_generator.generate_correctness_proof(expr, &runtime_result.value)?;
            timing.correctness_proof_time = proof_start.elapsed();
            Some(result)
        } else {
            None
        };

        // Perform theorem proving
        let theorem_proving_result = if self.config.enable_theorem_proving {
            let theorem_start = Instant::now();
            let result = TheoremProvingHelper::perform_theorem_proving(
                &mut self.theorem_prover, 
                expr, 
                &runtime_result.value,
                self.statistics.total_verifications
            )?;
            timing.theorem_proving_time = theorem_start.elapsed();
            Some(result)
        } else {
            None
        };

        // Call external provers
        let external_prover_results = if self.config.enable_external_provers {
            let external_start = Instant::now();
            let results = self.external_prover_integration.call_external_provers(expr, &runtime_result.value)?;
            timing.external_prover_time = external_start.elapsed();
            results
        } else {
            Vec::new()
        };

        // Generate formal proofs
        let formal_proofs = if self.config.generate_formal_proofs {
            self.proof_generator.generate_formal_proofs(expr, &runtime_result.value)?
        } else {
            Vec::new()
        };

        // Collect verification evidence
        let evidence = self.proof_generator.collect_verification_evidence(
            expr,
            &runtime_result.value,
            &semantic_verification,
            &correctness_proof,
        )?;

        // Calculate confidence level
        let confidence_level = self.calculate_confidence_level(
            &semantic_verification,
            &correctness_proof,
            &theorem_proving_result,
            &external_prover_results,
        );

        // Determine verification status
        let status = self.determine_verification_status(
            &semantic_verification,
            &correctness_proof,
            &theorem_proving_result,
            &external_prover_results,
            confidence_level,
        );

        timing.total_time = start_time.elapsed();

        // Create verification result
        let result = FormalVerificationResult {
            status,
            confidence_level,
            correctness_proof,
            semantic_verification,
            theorem_proving_result,
            external_prover_results,
            timing_breakdown: timing,
            formal_proofs,
            evidence,
        };

        // Cache the result
        if self.config.cache_results {
            let cache_key = self.generate_cache_key(expr, &optimization_level);
            self.verification_cache.insert(
                cache_key,
                CachedVerificationResult {
                    result: result.clone(),
                    cached_at: Instant::now(),
                    hit_count: 0,
                    expires_at: Instant::now() + Duration::from_secs(3600), // 1 hour
                },
            );
        }

        // Update statistics
        self.update_statistics(&result);

        Ok(result)
    }

    /// Generate cache key for verification results
    fn generate_cache_key(
        &self,
        expr: &Expr,
        optimization_level: &RuntimeOptimizationLevel,
    ) -> String {
        format!("{expr:?}_{optimization_level:?}")
    }

    /// Calculate confidence level
    fn calculate_confidence_level(
        &self,
        semantic_verification: &Option<SystemVerificationResult>,
        correctness_proof: &Option<crate::evaluator::CorrectnessProof>,
        theorem_proving_result: &Option<TheoremProvingResult>,
        external_prover_results: &[super::configuration_types::ExternalProverResult],
    ) -> f64 {
        let mut confidence = 0.0;
        let mut weight_sum = 0.0;

        // Semantic verification contribution (40% weight)
        if let Some(verification) = semantic_verification {
            confidence += verification.analysis.confidence_level * 0.4;
            weight_sum += 0.4;
        }

        // Correctness proof contribution (30% weight)
        if correctness_proof.is_some() {
            confidence += 0.95 * 0.3; // High confidence for correctness proofs
            weight_sum += 0.3;
        }

        // Theorem proving contribution (20% weight)
        if let Some(theorem_result) = theorem_proving_result {
            let theorem_confidence = match theorem_result.status {
                TheoremProvingStatus::AllProved => 1.0,
                TheoremProvingStatus::PartiallyProved => 0.7,
                TheoremProvingStatus::NotProved => 0.3,
                TheoremProvingStatus::Failed => 0.0,
            };
            confidence += theorem_confidence * 0.2;
            weight_sum += 0.2;
        }

        // External prover contribution (10% weight)
        if !external_prover_results.is_empty() {
            let external_confidence: f64 = external_prover_results
                .iter()
                .map(|result| result.confidence_score)
                .sum::<f64>()
                / external_prover_results.len() as f64;
            confidence += external_confidence * 0.1;
            weight_sum += 0.1;
        }

        if weight_sum > 0.0 {
            confidence / weight_sum
        } else {
            0.0
        }
    }

    /// Determine verification status
    fn determine_verification_status(
        &self,
        semantic_verification: &Option<SystemVerificationResult>,
        _correctness_proof: &Option<crate::evaluator::CorrectnessProof>,
        _theorem_proving_result: &Option<TheoremProvingResult>,
        _external_prover_results: &[super::configuration_types::ExternalProverResult],
        confidence_level: f64,
    ) -> FormalVerificationStatus {
        // Check for failures
        if let Some(verification) = semantic_verification {
            if matches!(
                verification.status,
                crate::evaluator::VerificationStatus::Failed(_)
            ) {
                return FormalVerificationStatus::Failed(
                    "Semantic verification failed".to_string(),
                );
            }
        }

        // Check confidence level
        if confidence_level >= self.config.required_confidence {
            if confidence_level >= 0.95 {
                FormalVerificationStatus::Verified
            } else {
                FormalVerificationStatus::Validated
            }
        } else if confidence_level >= 0.5 {
            FormalVerificationStatus::Inconclusive
        } else {
            FormalVerificationStatus::Failed("Insufficient confidence level".to_string())
        }
    }

    /// Update verification statistics
    fn update_statistics(&mut self, result: &FormalVerificationResult) {
        self.statistics.total_verifications += 1;

        match result.status {
            FormalVerificationStatus::Verified | FormalVerificationStatus::Validated => {
                self.statistics.successful_verifications += 1;
            }
            FormalVerificationStatus::Failed(_) => {
                self.statistics.failed_verifications += 1;
            }
            FormalVerificationStatus::Timeout => {
                self.statistics.timeout_verifications += 1;
            }
            _ => {}
        }

        // Update average verification time
        let total_time = self.statistics.avg_verification_time.as_millis() as f64
            * (self.statistics.total_verifications - 1) as f64;
        let new_time = result.timing_breakdown.total_time.as_millis() as f64;
        self.statistics.avg_verification_time = Duration::from_millis(
            ((total_time + new_time) / self.statistics.total_verifications as f64) as u64,
        );

        // Update other statistics
        if result.correctness_proof.is_some() {
            self.statistics.correctness_proofs_generated += 1;
        }

        if let Some(theorem_result) = &result.theorem_proving_result {
            if matches!(
                theorem_result.status,
                TheoremProvingStatus::AllProved | TheoremProvingStatus::PartiallyProved
            ) {
                self.statistics.theorem_proving_successes += 1;
            }
        }

        self.statistics.external_prover_calls += result.external_prover_results.len();

        // Update confidence distribution
        let confidence_bucket = format!("{:.1}", (result.confidence_level * 10.0).floor() / 10.0);
        *self
            .statistics
            .confidence_distribution
            .entry(confidence_bucket)
            .or_insert(0) += 1;
    }

    /// Get verification statistics
    #[must_use] 
    pub fn get_statistics(&self) -> &VerificationStatistics {
        &self.statistics
    }

    /// Get configuration
    #[must_use] 
    pub fn get_config(&self) -> &VerificationConfiguration {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: VerificationConfiguration) {
        self.config = config;
    }

    /// Clear verification cache
    pub fn clear_cache(&mut self) {
        self.verification_cache.clear();
        self.proof_generator.clear_cache();
    }

    /// Get cache statistics
    #[must_use] 
    pub fn get_cache_stats(&self) -> (usize, f64) {
        (
            self.verification_cache.len(),
            self.statistics.cache_hit_rate,
        )
    }

    /// Prove Church-Rosser properties for combinatory expressions
    pub fn prove_church_rosser_properties(&mut self, expr: &Expr) -> Result<ChurchRosserProof> {
        // Convert expression to combinator form for Church-Rosser analysis
        let combinator_expr = self.convert_to_combinator_form(expr)?;

        // Use Church-Rosser proof engine
        self.church_rosser_engine
            .prove_church_rosser(&combinator_expr)
    }

    /// Convert expression to combinator form
    fn convert_to_combinator_form(
        &self,
        expr: &Expr,
    ) -> Result<crate::evaluator::combinators::CombinatorExpr> {
        use crate::evaluator::combinators::BracketAbstraction;
        BracketAbstraction::lambda_to_combinators(expr)
    }

    /// Verify confluence properties via Church-Rosser engine
    pub fn verify_confluence(
        &mut self,
        expr: &Expr,
    ) -> Result<crate::evaluator::church_rosser_proof::ChurchRosserProof> {
        let combinator_expr = self.convert_to_combinator_form(expr)?;
        // Use the main prove_church_rosser method which handles all aspects
        self.church_rosser_engine.prove_church_rosser(&combinator_expr)
    }

    /// Verify termination properties via Church-Rosser engine
    pub fn verify_termination(
        &mut self,
        expr: &Expr,
    ) -> Result<crate::evaluator::church_rosser_proof::ChurchRosserProof> {
        let combinator_expr = self.convert_to_combinator_form(expr)?;
        // Use the main prove_church_rosser method which handles all aspects
        self.church_rosser_engine.prove_church_rosser(&combinator_expr)
    }

    /// Verify normalization properties via Church-Rosser engine
    pub fn verify_normalization(
        &mut self,
        expr: &Expr,
    ) -> Result<crate::evaluator::church_rosser_proof::ChurchRosserProof> {
        let combinator_expr = self.convert_to_combinator_form(expr)?;
        // Use the main prove_church_rosser method which handles all aspects
        self.church_rosser_engine.prove_church_rosser(&combinator_expr)
    }

    /// Get Church-Rosser proof statistics
    #[must_use] 
    pub fn get_church_rosser_statistics(
        &self,
    ) -> &crate::evaluator::church_rosser_proof::ChurchRosserStatistics {
        self.church_rosser_engine.get_statistics()
    }
}

impl Default for FormalVerificationEngine {
    fn default() -> Self {
        Self::new()
    }
}