//! Comprehensive verification system using `SemanticEvaluator` as reference
//!
//! This module implements automatic verification of runtime execution results
//! against the pure R7RS semantic evaluation. It serves as a correctness
//! guarantee system for all optimized execution paths.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
#[cfg(feature = "development")]
use crate::evaluator::{
    Continuation, CorrectnessProof, CorrectnessProperty,
    SemanticCorrectnessProver, SemanticEvaluator,
};
use crate::prover::proof_types::{ProofResult, ProofTactic};
#[cfg(feature = "development")]
use crate::executor::RuntimeOptimizationLevel;
#[cfg(feature = "development")]
use crate::prover::{
    GoalType, ProofGoal, Statement, TheoremProvingSupport,
};
#[cfg(not(feature = "development"))]
use crate::evaluator::{
    Continuation, CorrectnessProof, CorrectnessProperty,
    RuntimeOptimizationLevel, SemanticCorrectnessProver, SemanticEvaluator,
};
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Verification system configuration
#[derive(Debug, Clone)]
pub struct VerificationConfig {
    /// Enable semantic equivalence verification
    pub verify_semantic_equivalence: bool,
    /// Enable correctness proof generation
    pub generate_correctness_proofs: bool,
    /// Enable theorem proving verification
    pub use_theorem_proving: bool,
    /// Maximum verification time per expression (milliseconds)
    pub max_verification_time_ms: u64,
    /// Enable statistical analysis
    pub enable_statistics: bool,
    /// Store verification history
    pub store_verification_history: bool,
    /// Maximum history entries to keep
    pub max_history_entries: usize,
}

/// Verification result types
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationStatus {
    /// Verification passed successfully
    Passed,
    /// Verification failed with specific reason
    Failed(String),
    /// Verification timed out
    Timeout,
    /// Verification was skipped
    Skipped,
    /// Verification encountered an error
    Error(String),
}

/// Comprehensive verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Overall verification status
    pub status: VerificationStatus,
    /// Reference result from `SemanticEvaluator`
    pub reference_result: Option<Value>,
    /// Actual result from optimized execution
    pub actual_result: Option<Value>,
    /// Semantic equivalence verification
    pub semantic_equivalence: Option<bool>,
    /// Generated correctness proof
    pub correctness_proof: Option<CorrectnessProof>,
    /// Theorem proving verification result
    pub theorem_proof: Option<String>,
    /// Verification time taken
    pub verification_time: Duration,
    /// Detailed analysis
    pub analysis: VerificationAnalysis,
}

/// Detailed verification analysis
#[derive(Debug, Clone)]
pub struct VerificationAnalysis {
    /// Value type consistency
    pub value_type_match: bool,
    /// Structural equivalence
    pub structural_match: bool,
    /// Numerical precision match (for numbers)
    pub numerical_precision_match: Option<bool>,
    /// String content match (for strings)
    pub string_content_match: Option<bool>,
    /// List structure match (for lists)
    pub list_structure_match: Option<bool>,
    /// Detected discrepancies
    pub discrepancies: Vec<String>,
    /// Confidence level (0.0 to 1.0)
    pub confidence_level: f64,
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
    /// Timed out verifications
    pub timeout_verifications: usize,
    /// Average verification time
    pub average_verification_time_ms: f64,
    /// Verification success rate
    pub success_rate: f64,
    /// Most common failure reasons
    pub common_failures: HashMap<String, usize>,
}

/// Verification history entry
#[derive(Debug, Clone)]
pub struct VerificationHistoryEntry {
    /// Expression that was verified
    pub expression: Expr,
    /// Optimization level used
    pub optimization_level: RuntimeOptimizationLevel,
    /// Verification result
    pub result: VerificationResult,
    /// Timestamp
    pub timestamp: Instant,
}

/// Main verification system
#[derive(Debug)]
pub struct VerificationSystem {
    /// Configuration
    config: VerificationConfig,
    /// Reference semantic evaluator
    semantic_evaluator: SemanticEvaluator,
    /// Correctness prover
    correctness_prover: SemanticCorrectnessProver,
    /// Theorem proving system
    theorem_prover: TheoremProvingSupport,
    /// Verification statistics
    statistics: VerificationStatistics,
    /// Verification history
    history: Vec<VerificationHistoryEntry>,
    /// Value comparison cache
    comparison_cache: HashMap<String, bool>,
}

impl VerificationSystem {
    /// Create new verification system
    #[must_use] pub fn new() -> Self {
        Self {
            config: VerificationConfig::default(),
            semantic_evaluator: SemanticEvaluator::new(),
            correctness_prover: SemanticCorrectnessProver::new(),
            theorem_prover: TheoremProvingSupport::new(),
            statistics: VerificationStatistics::default(),
            history: Vec::new(),
            comparison_cache: HashMap::new(),
        }
    }

    /// Create with custom configuration
    #[must_use] pub fn with_config(config: VerificationConfig) -> Self {
        Self {
            config,
            semantic_evaluator: SemanticEvaluator::new(),
            correctness_prover: SemanticCorrectnessProver::new(),
            theorem_prover: TheoremProvingSupport::new(),
            statistics: VerificationStatistics::default(),
            history: Vec::new(),
            comparison_cache: HashMap::new(),
        }
    }

    /// Verify runtime execution result against semantic reference
    pub fn verify_execution(
        &mut self,
        expr: &Expr,
        env: &Rc<Environment>,
        cont: &Continuation,
        runtime_result: &Value,
        optimization_level: RuntimeOptimizationLevel,
    ) -> Result<VerificationResult> {
        let start_time = Instant::now();

        // Check timeout
        let timeout = Duration::from_millis(self.config.max_verification_time_ms);

        // Get reference result from SemanticEvaluator
        let reference_result = if self.config.verify_semantic_equivalence {
            let eval_start = Instant::now();
            let result = self
                .semantic_evaluator
                .eval_pure(expr.clone(), env.clone(), cont.clone());

            // Check if semantic evaluation timed out
            if eval_start.elapsed() > timeout {
                return Ok(VerificationResult {
                    status: VerificationStatus::Timeout,
                    reference_result: None,
                    actual_result: Some(runtime_result.clone()),
                    semantic_equivalence: None,
                    correctness_proof: None,
                    theorem_proof: None,
                    verification_time: start_time.elapsed(),
                    analysis: VerificationAnalysis::default(),
                });
            }

            match result {
                Ok(value) => Some(value),
                Err(e) => {
                    return Ok(VerificationResult {
                        status: VerificationStatus::Error(e.to_string()),
                        reference_result: None,
                        actual_result: Some(runtime_result.clone()),
                        semantic_equivalence: None,
                        correctness_proof: None,
                        theorem_proof: None,
                        verification_time: start_time.elapsed(),
                        analysis: VerificationAnalysis::default(),
                    });
                }
            }
        } else {
            None
        };

        // Perform semantic equivalence check
        let semantic_equivalence = if let Some(ref reference) = reference_result {
            Some(self.verify_semantic_equivalence(reference, runtime_result)?)
        } else {
            None
        };

        // Generate correctness proof if enabled
        let correctness_proof = if self.config.generate_correctness_proofs {
            match self.generate_correctness_proof(expr, runtime_result) {
                Ok(proof) => Some(proof),
                Err(_) => None, // Continue verification even if proof generation fails
            }
        } else {
            None
        };

        // Perform theorem proving verification if enabled
        let theorem_proof = if self.config.use_theorem_proving {
            match self.verify_with_theorem_proving(expr, runtime_result) {
                Ok(proof) => Some(proof),
                Err(_) => None, // Continue verification even if theorem proving fails
            }
        } else {
            None
        };

        // Perform detailed analysis
        let analysis = if let Some(ref reference) = reference_result {
            self.analyze_verification(reference, runtime_result)
        } else {
            VerificationAnalysis::default()
        };

        // Determine overall verification status
        let status = self.determine_verification_status(
            &reference_result,
            runtime_result,
            &semantic_equivalence,
            &analysis,
        );

        let verification_time = start_time.elapsed();

        // Create verification result
        let result = VerificationResult {
            status: status.clone(),
            reference_result,
            actual_result: Some(runtime_result.clone()),
            semantic_equivalence,
            correctness_proof,
            theorem_proof,
            verification_time,
            analysis,
        };

        // Update statistics
        self.update_statistics(&result);

        // Store in history if enabled
        if self.config.store_verification_history {
            self.store_verification_history(expr.clone(), optimization_level, result.clone());
        }

        Ok(result)
    }

    /// Verify semantic equivalence between two values
    fn verify_semantic_equivalence(&mut self, reference: &Value, actual: &Value) -> Result<bool> {
        // Check cache first
        let cache_key = format!("{reference:?}||{actual:?}");
        if let Some(&cached_result) = self.comparison_cache.get(&cache_key) {
            return Ok(cached_result);
        }

        let result = self.deep_value_comparison(reference, actual)?;

        // Cache the result
        self.comparison_cache.insert(cache_key, result);

        Ok(result)
    }

    /// Deep comparison of two values
    fn deep_value_comparison(&self, v1: &Value, v2: &Value) -> Result<bool> {
        match (v1, v2) {
            // Exact matches
            (Value::Boolean(b1), Value::Boolean(b2)) => Ok(b1 == b2),
            (Value::Number(n1), Value::Number(n2)) => Ok(n1 == n2),
            (Value::String(s1), Value::String(s2)) => Ok(s1 == s2),
            (Value::Character(c1), Value::Character(c2)) => Ok(c1 == c2),
            (Value::Symbol(s1), Value::Symbol(s2)) => Ok(s1 == s2),
            (Value::Nil, Value::Nil) => Ok(true),
            (Value::Undefined, Value::Undefined) => Ok(true),

            // Vector comparison
            (Value::Vector(v1), Value::Vector(v2)) => {
                if v1.len() != v2.len() {
                    return Ok(false);
                }
                for (val1, val2) in v1.iter().zip(v2.iter()) {
                    if !self.deep_value_comparison(val1, val2)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            // Pair comparison
            (Value::Pair(p1), Value::Pair(p2)) => {
                let p1_borrowed = p1.borrow();
                let p2_borrowed = p2.borrow();
                Ok(
                    self.deep_value_comparison(&p1_borrowed.car, &p2_borrowed.car)?
                        && self.deep_value_comparison(&p1_borrowed.cdr, &p2_borrowed.cdr)?,
                )
            }

            // Multiple values comparison
            (Value::Values(v1), Value::Values(v2)) => {
                if v1.len() != v2.len() {
                    return Ok(false);
                }
                for (val1, val2) in v1.iter().zip(v2.iter()) {
                    if !self.deep_value_comparison(val1, val2)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            // Different types
            _ => Ok(false),
        }
    }

    /// Generate correctness proof for the result
    fn generate_correctness_proof(
        &mut self,
        expr: &Expr,
        result: &Value,
    ) -> Result<CorrectnessProof> {
        let property = CorrectnessProperty::ReferentialTransparency(expr.clone(), result.clone());

        self.correctness_prover.prove_property(property)
    }

    /// Verify result using theorem proving
    fn verify_with_theorem_proving(&mut self, expr: &Expr, _result: &Value) -> Result<String> {
        // Create semantic equivalence statement
        let statement = Statement::R7RSCompliance(expr.clone());

        // Create proof goal
        let goal = ProofGoal {
            statement,
            goal_type: GoalType::R7RSCompliance,
            expressions: vec![expr.clone()],
            id: format!("verification_{}", self.statistics.total_verifications),
        };

        // Add goal to theorem prover
        self.theorem_prover.add_goal(goal)?;

        // Apply R7RS semantics verification
        let tactic_result = self
            .theorem_prover
            .apply_tactic(ProofTactic::R7RSSemantics)?;

        match tactic_result {
            ProofResult::Success => {
                Ok("R7RS semantic compliance verified".to_string())
            }
            ProofResult::Failed(msg) => {
                Err(LambdustError::runtime_error(
                    format!("Theorem proving verification failed: {}", msg),
                ))
            }
            ProofResult::Incomplete => {
                Err(LambdustError::runtime_error(
                    "Theorem proving verification incomplete".to_string(),
                ))
            }
        }
    }

    /// Perform detailed analysis of verification
    fn analyze_verification(&self, reference: &Value, actual: &Value) -> VerificationAnalysis {
        let mut analysis = VerificationAnalysis::default();

        // Check value type consistency
        analysis.value_type_match = self.get_value_type(reference) == self.get_value_type(actual);

        // Check structural equivalence
        analysis.structural_match = self.check_structural_equivalence(reference, actual);

        // Type-specific analysis
        match (reference, actual) {
            (Value::Number(n1), Value::Number(n2)) => {
                analysis.numerical_precision_match = Some(n1 == n2);
            }
            (Value::String(s1), Value::String(s2)) => {
                analysis.string_content_match = Some(s1 == s2);
            }
            (Value::Vector(v1), Value::Vector(v2)) => {
                analysis.list_structure_match = Some(v1.len() == v2.len());
            }
            _ => {}
        }

        // Detect discrepancies
        analysis.discrepancies = self.detect_discrepancies(reference, actual);

        // Calculate confidence level
        analysis.confidence_level = self.calculate_confidence_level(&analysis);

        analysis
    }

    /// Get value type string
    fn get_value_type(&self, value: &Value) -> String {
        match value {
            Value::Boolean(_) => "boolean".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Character(_) => "character".to_string(),
            Value::Symbol(_) => "symbol".to_string(),
            Value::Nil => "nil".to_string(),
            Value::Pair(_) => "pair".to_string(),
            Value::Vector(_) => "vector".to_string(),
            Value::Procedure(_) => "procedure".to_string(),
            Value::Port(_) => "port".to_string(),
            Value::Undefined => "undefined".to_string(),
            Value::Values(_) => "values".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Check structural equivalence
    fn check_structural_equivalence(&self, v1: &Value, v2: &Value) -> bool {
        match (v1, v2) {
            (Value::Vector(vec1), Value::Vector(vec2)) => vec1.len() == vec2.len(),
            (Value::Pair(_), Value::Pair(_)) => true,
            (Value::Values(vals1), Value::Values(vals2)) => vals1.len() == vals2.len(),
            _ => true,
        }
    }

    /// Detect discrepancies between values
    fn detect_discrepancies(&self, reference: &Value, actual: &Value) -> Vec<String> {
        let mut discrepancies = Vec::new();

        let ref_type = self.get_value_type(reference);
        let actual_type = self.get_value_type(actual);

        if ref_type != actual_type {
            discrepancies.push(format!(
                "Type mismatch: expected {ref_type}, got {actual_type}"
            ));
        }

        match (reference, actual) {
            (Value::Number(n1), Value::Number(n2)) => {
                if n1 != n2 {
                    discrepancies.push(format!(
                        "Numerical value mismatch: expected {n1:?}, got {n2:?}"
                    ));
                }
            }
            (Value::String(s1), Value::String(s2)) => {
                if s1 != s2 {
                    discrepancies.push(format!(
                        "String content mismatch: expected '{s1}', got '{s2}'"
                    ));
                }
            }
            (Value::Vector(v1), Value::Vector(v2)) => {
                if v1.len() != v2.len() {
                    discrepancies.push(format!(
                        "Vector length mismatch: expected {}, got {}",
                        v1.len(),
                        v2.len()
                    ));
                }
            }
            _ => {}
        }

        discrepancies
    }

    /// Calculate confidence level based on analysis
    fn calculate_confidence_level(&self, analysis: &VerificationAnalysis) -> f64 {
        let mut confidence = 1.0;

        if !analysis.value_type_match {
            confidence -= 0.5;
        }

        if !analysis.structural_match {
            confidence -= 0.3;
        }

        if let Some(false) = analysis.numerical_precision_match {
            confidence -= 0.2;
        }

        if let Some(false) = analysis.string_content_match {
            confidence -= 0.2;
        }

        if let Some(false) = analysis.list_structure_match {
            confidence -= 0.2;
        }

        confidence -= analysis.discrepancies.len() as f64 * 0.1;

        confidence.max(0.0)
    }

    /// Determine overall verification status
    fn determine_verification_status(
        &self,
        _reference: &Option<Value>,
        _actual: &Value,
        semantic_equivalence: &Option<bool>,
        analysis: &VerificationAnalysis,
    ) -> VerificationStatus {
        if let Some(false) = semantic_equivalence {
            return VerificationStatus::Failed("Semantic equivalence check failed".to_string());
        }

        if !analysis.value_type_match {
            return VerificationStatus::Failed("Value type mismatch".to_string());
        }

        if analysis.confidence_level < 0.5 {
            return VerificationStatus::Failed("Low confidence level".to_string());
        }

        if !analysis.discrepancies.is_empty() {
            return VerificationStatus::Failed(format!(
                "Discrepancies detected: {:?}",
                analysis.discrepancies
            ));
        }

        VerificationStatus::Passed
    }

    /// Update verification statistics
    fn update_statistics(&mut self, result: &VerificationResult) {
        self.statistics.total_verifications += 1;

        match result.status {
            VerificationStatus::Passed => {
                self.statistics.successful_verifications += 1;
            }
            VerificationStatus::Failed(ref reason) => {
                self.statistics.failed_verifications += 1;
                let count = self
                    .statistics
                    .common_failures
                    .entry(reason.clone())
                    .or_insert(0);
                *count += 1;
            }
            VerificationStatus::Timeout => {
                self.statistics.timeout_verifications += 1;
            }
            _ => {}
        }

        // Update average verification time
        let total_time = self.statistics.average_verification_time_ms
            * (self.statistics.total_verifications - 1) as f64;
        let new_time = result.verification_time.as_millis() as f64;
        self.statistics.average_verification_time_ms =
            (total_time + new_time) / self.statistics.total_verifications as f64;

        // Update success rate
        self.statistics.success_rate = self.statistics.successful_verifications as f64
            / self.statistics.total_verifications as f64;
    }

    /// Store verification in history
    fn store_verification_history(
        &mut self,
        expr: Expr,
        optimization_level: RuntimeOptimizationLevel,
        result: VerificationResult,
    ) {
        let entry = VerificationHistoryEntry {
            expression: expr,
            optimization_level,
            result,
            timestamp: Instant::now(),
        };

        self.history.push(entry);

        // Keep only recent entries
        if self.history.len() > self.config.max_history_entries {
            self.history.remove(0);
        }
    }

    /// Get verification statistics
    #[must_use] pub fn get_statistics(&self) -> &VerificationStatistics {
        &self.statistics
    }

    /// Get verification history
    #[must_use] pub fn get_history(&self) -> &[VerificationHistoryEntry] {
        &self.history
    }

    /// Clear verification history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Clear comparison cache
    pub fn clear_cache(&mut self) {
        self.comparison_cache.clear();
    }

    /// Get reference to semantic evaluator for direct access
    #[must_use] pub fn get_semantic_evaluator(&self) -> &SemanticEvaluator {
        &self.semantic_evaluator
    }

    /// Get mutable reference to semantic evaluator
    pub fn get_semantic_evaluator_mut(&mut self) -> &mut SemanticEvaluator {
        &mut self.semantic_evaluator
    }

    /// Verify expression using semantic evaluator
    pub fn verify_with_semantic_evaluator(
        &mut self,
        expr: &Expr,
        env: Rc<Environment>,
        expected: &Value,
    ) -> Result<bool> {
        let result = self.semantic_evaluator.eval_pure(
            expr.clone(),
            env,
            Continuation::Identity,
        )?;
        Ok(self.values_equal(&result, expected))
    }

    /// Compare two values for equality
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => n1 == n2,
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Character(c1), Value::Character(c2)) => c1 == c2,
            (Value::Symbol(s1), Value::Symbol(s2)) => s1 == s2,
            (Value::Nil, Value::Nil) => true,
            (Value::Undefined, Value::Undefined) => true,
            _ => false, // More sophisticated comparison would be needed for complex types
        }
    }

    /// Get configuration
    #[must_use] pub fn get_config(&self) -> &VerificationConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: VerificationConfig) {
        self.config = config;
    }

    /// Reset statistics
    pub fn reset_statistics(&mut self) {
        self.statistics = VerificationStatistics::default();
    }

    /// Get cache statistics
    #[must_use] pub fn get_cache_stats(&self) -> (usize, usize) {
        (
            self.comparison_cache.len(),
            self.comparison_cache.capacity(),
        )
    }
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            verify_semantic_equivalence: true,
            generate_correctness_proofs: true,
            use_theorem_proving: false, // Disabled by default for performance
            max_verification_time_ms: 1000,
            enable_statistics: true,
            store_verification_history: true,
            max_history_entries: 1000,
        }
    }
}

impl Default for VerificationAnalysis {
    fn default() -> Self {
        Self {
            value_type_match: true,
            structural_match: true,
            numerical_precision_match: None,
            string_content_match: None,
            list_structure_match: None,
            discrepancies: Vec::new(),
            confidence_level: 1.0,
        }
    }
}

impl Default for VerificationSystem {
    fn default() -> Self {
        Self::new()
    }
}

