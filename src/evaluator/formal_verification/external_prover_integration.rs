//! External Prover Integration Module
//!
//! このモジュールは外部証明器との統合を提供します。
//! Agda、Coq、Leanなどの外部証明器を使用した形式的検証を行います。

use super::configuration_types::{ExternalProverResult, ExternalProverStatus};
use crate::ast::Expr;
use crate::error::Result;
use crate::evaluator::ExternalProverManager;
use crate::value::Value;
use std::time::{Duration, Instant};

/// External prover integration system
#[derive(Debug)]
pub struct ExternalProverIntegration {
    /// External prover manager
    prover_manager: ExternalProverManager,

    /// Integration statistics
    statistics: ExternalProverStatistics,

    /// Prover timeouts
    timeouts: ExternalProverTimeouts,
}

/// External prover statistics
#[derive(Debug, Clone, Default)]
pub struct ExternalProverStatistics {
    /// Total external prover calls
    pub total_calls: usize,

    /// Successful proofs
    pub successful_proofs: usize,

    /// Failed proofs
    pub failed_proofs: usize,

    /// Timeout occurrences
    pub timeouts: usize,

    /// Average proof time
    pub avg_proof_time: Duration,

    /// Prover success rates
    pub prover_success_rates: std::collections::HashMap<String, f64>,
}

/// External prover timeouts configuration
#[derive(Debug, Clone)]
pub struct ExternalProverTimeouts {
    /// Agda timeout
    pub agda_timeout: Duration,

    /// Coq timeout
    pub coq_timeout: Duration,

    /// Lean timeout
    pub lean_timeout: Duration,

    /// Default timeout
    pub default_timeout: Duration,
}

impl ExternalProverIntegration {
    /// Create new external prover integration
    pub fn new() -> Self {
        Self {
            prover_manager: ExternalProverManager::new(),
            statistics: ExternalProverStatistics::default(),
            timeouts: ExternalProverTimeouts::default(),
        }
    }

    /// Call external provers
    pub fn call_external_provers(
        &mut self,
        expr: &Expr,
        result: &Value,
    ) -> Result<Vec<ExternalProverResult>> {
        let mut results = Vec::new();

        // Call Agda prover
        let agda_result = self.call_agda_prover(expr, result)?;
        results.push(agda_result);

        // Call Coq prover
        let coq_result = self.call_coq_prover(expr, result)?;
        results.push(coq_result);

        // Update statistics
        self.update_statistics(&results);

        Ok(results)
    }

    /// Call Agda prover
    fn call_agda_prover(&mut self, expr: &Expr, result: &Value) -> Result<ExternalProverResult> {
        let agda_start = Instant::now();
        
        // Create statement for Agda verification
        let statement = crate::evaluator::theorem_proving::Statement::SemanticEquivalence(
            expr.clone(),
            crate::ast::Expr::Literal(crate::ast::Literal::String(format!("{result:?}"))),
        );

        let result = match self.prover_manager.verify_with_prover(
            &statement,
            crate::evaluator::external_provers::ExternalProver::Agda,
        ) {
            Ok(agda_result) => ExternalProverResult {
                prover_name: "Agda".to_string(),
                status: if agda_result.success {
                    ExternalProverStatus::Proved
                } else {
                    ExternalProverStatus::Failed
                },
                proof_output: agda_result
                    .proof_term
                    .unwrap_or_else(|| "No proof generated".to_string()),
                verification_time: agda_start.elapsed(),
                confidence_score: if agda_result.success { 0.8 } else { 0.0 },
            },
            Err(_) => ExternalProverResult {
                prover_name: "Agda".to_string(),
                status: ExternalProverStatus::Error("Agda verification failed".to_string()),
                proof_output: String::new(),
                verification_time: agda_start.elapsed(),
                confidence_score: 0.0,
            },
        };

        self.statistics.total_calls += 1;
        
        Ok(result)
    }

    /// Call Coq prover
    fn call_coq_prover(&mut self, expr: &Expr, result: &Value) -> Result<ExternalProverResult> {
        let coq_start = Instant::now();
        
        // Create statement for Coq verification
        let statement = crate::evaluator::theorem_proving::Statement::SemanticEquivalence(
            expr.clone(),
            crate::ast::Expr::Literal(crate::ast::Literal::String(format!("{result:?}"))),
        );

        let result = match self.prover_manager.verify_with_prover(
            &statement,
            crate::evaluator::external_provers::ExternalProver::Coq,
        ) {
            Ok(coq_result) => ExternalProverResult {
                prover_name: "Coq".to_string(),
                status: if coq_result.success {
                    ExternalProverStatus::Proved
                } else {
                    ExternalProverStatus::Failed
                },
                proof_output: coq_result
                    .proof_term
                    .unwrap_or_else(|| "No proof generated".to_string()),
                verification_time: coq_start.elapsed(),
                confidence_score: if coq_result.success { 0.8 } else { 0.0 },
            },
            Err(_) => ExternalProverResult {
                prover_name: "Coq".to_string(),
                status: ExternalProverStatus::Error("Coq verification failed".to_string()),
                proof_output: String::new(),
                verification_time: coq_start.elapsed(),
                confidence_score: 0.0,
            },
        };

        self.statistics.total_calls += 1;
        
        Ok(result)
    }

    /// Update statistics based on results
    fn update_statistics(&mut self, results: &[ExternalProverResult]) {
        for result in results {
            match result.status {
                ExternalProverStatus::Proved => self.statistics.successful_proofs += 1,
                ExternalProverStatus::Failed => self.statistics.failed_proofs += 1,
                ExternalProverStatus::Timeout => self.statistics.timeouts += 1,
                ExternalProverStatus::Error(_) => self.statistics.failed_proofs += 1,
            }

            // Update prover-specific success rate
            let success = matches!(result.status, ExternalProverStatus::Proved);
            let current_rate = self.statistics.prover_success_rates
                .get(&result.prover_name)
                .copied()
                .unwrap_or(0.0);
            
            let calls_for_prover = results.iter()
                .filter(|r| r.prover_name == result.prover_name)
                .count();
            
            let new_rate = if calls_for_prover > 0 {
                if success {
                    (current_rate * (calls_for_prover - 1) as f64 + 1.0) / calls_for_prover as f64
                } else {
                    current_rate * (calls_for_prover - 1) as f64 / calls_for_prover as f64
                }
            } else {
                current_rate
            };
            
            self.statistics.prover_success_rates.insert(result.prover_name.clone(), new_rate);
        }

        // Update average proof time
        if !results.is_empty() {
            let total_time: Duration = results.iter()
                .map(|r| r.verification_time)
                .sum();
            let avg_time = total_time / results.len() as u32;
            
            let previous_avg = self.statistics.avg_proof_time;
            let previous_count = self.statistics.total_calls - results.len();
            
            if previous_count > 0 {
                let total_previous_time = previous_avg * previous_count as u32;
                self.statistics.avg_proof_time = (total_previous_time + total_time) / self.statistics.total_calls as u32;
            } else {
                self.statistics.avg_proof_time = avg_time;
            }
        }
    }

    /// Get integration statistics
    #[must_use]
    pub fn get_statistics(&self) -> &ExternalProverStatistics {
        &self.statistics
    }

    /// Configure timeouts
    pub fn set_timeouts(&mut self, timeouts: ExternalProverTimeouts) {
        self.timeouts = timeouts;
    }

    /// Get current timeouts
    #[must_use]
    pub fn get_timeouts(&self) -> &ExternalProverTimeouts {
        &self.timeouts
    }
}

impl Default for ExternalProverIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ExternalProverTimeouts {
    fn default() -> Self {
        Self {
            agda_timeout: Duration::from_secs(30),
            coq_timeout: Duration::from_secs(30),
            lean_timeout: Duration::from_secs(30),
            default_timeout: Duration::from_secs(30),
        }
    }
}

/// Helper functions for external prover integration
pub struct ExternalProverHelper;

impl ExternalProverHelper {
    /// Check if external provers are available
    pub fn are_external_provers_available() -> bool {
        // In a real implementation, this would check if external tools are installed
        // For now, return false to disable by default
        false
    }

    /// Format expression for external prover
    pub fn format_for_prover(expr: &Expr, prover: &str) -> String {
        match prover {
            "Agda" => format!("agda-expression : {expr:?}"),
            "Coq" => format!("Coq expression: {expr:?}"),
            "Lean" => format!("lean expression: {expr:?}"),
            _ => format!("expression: {expr:?}"),
        }
    }

    /// Parse prover output
    pub fn parse_prover_output(output: &str, prover: &str) -> (bool, Option<String>) {
        match prover {
            "Agda" => {
                if output.contains("QED") || output.contains("refl") {
                    (true, Some(output.to_string()))
                } else {
                    (false, None)
                }
            }
            "Coq" => {
                if output.contains("Qed.") || output.contains("Defined.") {
                    (true, Some(output.to_string()))
                } else {
                    (false, None)
                }
            }
            "Lean" => {
                if output.contains("no goals") {
                    (true, Some(output.to_string()))
                } else {
                    (false, None)
                }
            }
            _ => (false, None),
        }
    }
}