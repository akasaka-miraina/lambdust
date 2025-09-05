#![allow(missing_docs)]
//! Deoptimization system for JIT compilation
//!
//! Provides safe fallback mechanisms when JIT assumptions are violated.

use crate::ast::Expr;
use crate::diagnostics::{Error, Result};
use crate::eval::{Environment, Value};
use crate::jit::code_generator::NativeCode;
use crate::jit::compilation_tiers::CompilationTier;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Manages deoptimization events and fallback strategies
pub struct DeoptimizationManager {
    config: DeoptimizationConfig,
    stats: Arc<RwLock<DeoptimizationStats>>,
}

impl DeoptimizationManager {
    /// Creates a new deoptimization manager
    pub fn new(config: DeoptimizationConfig) -> Result<Self> {
        Ok(Self {
            config,
            stats: Arc::new(RwLock::new(DeoptimizationStats::new())),
        })
    }

    /// Triggers deoptimization for a specific reason
    pub fn trigger_deoptimization(
        &mut self,
        context: &DeoptimizationContext,
        reason: DeoptimizationReason,
    ) -> Result<DeoptimizationResult> {
        let start_time = Instant::now();

        let target_tier = self.determine_target_tier(context, &reason)?;

        let result = DeoptimizationResult {
            target_tier,
            deoptimization_time: start_time.elapsed(),
            reason: reason.clone(),
        };

        // Update statistics
        {
            let mut stats = self.stats.write().map_err(|_| {
                Error::runtime_error("Failed to acquire stats lock".to_string(), None)
            })?;
            stats.record_deoptimization(reason, start_time.elapsed());
        }

        Ok(result)
    }

    fn determine_target_tier(
        &self,
        context: &DeoptimizationContext,
        reason: &DeoptimizationReason,
    ) -> Result<CompilationTier> {
        match reason {
            DeoptimizationReason::SecurityViolation { .. } => Ok(CompilationTier::Interpreter),
            DeoptimizationReason::PerformanceRegression { .. } => {
                self.get_lower_tier(context.current_tier)
            }
            DeoptimizationReason::MemoryPressure => self.get_lower_tier(context.current_tier),
            _ => self.get_lower_tier(context.current_tier),
        }
    }

    fn get_lower_tier(&self, current: CompilationTier) -> Result<CompilationTier> {
        match current {
            CompilationTier::JitOptimized => Ok(CompilationTier::JitBasic),
            CompilationTier::JitBasic => Ok(CompilationTier::Bytecode),
            CompilationTier::Bytecode => Ok(CompilationTier::Interpreter),
            CompilationTier::Interpreter => Ok(CompilationTier::Interpreter),
        }
    }
}

/// Context for deoptimization decisions
#[derive(Debug, Clone)]
pub struct DeoptimizationContext {
    pub function_id: String,
    pub current_tier: CompilationTier,
    pub original_ast: Expr,
    pub environment: Arc<Environment>,
}

/// Reasons for deoptimization
#[derive(Debug, Clone)]
pub enum DeoptimizationReason {
    SecurityViolation {
        violation_type: String,
    },
    PerformanceRegression {
        expected_speedup: f64,
        actual_speedup: f64,
    },
    MemoryPressure,
    TypeAssumptionViolation {
        expected_type: String,
        actual_type: String,
    },
    Manual {
        reason: String,
    },
}

impl DeoptimizationReason {
    pub fn get_category(&self) -> String {
        match self {
            Self::SecurityViolation { .. } => "security".to_string(),
            Self::PerformanceRegression { .. } => "performance".to_string(),
            Self::MemoryPressure => "memory".to_string(),
            Self::TypeAssumptionViolation { .. } => "type".to_string(),
            Self::Manual { .. } => "manual".to_string(),
        }
    }
}

/// Result of deoptimization
#[derive(Debug, Clone)]
pub struct DeoptimizationResult {
    pub target_tier: CompilationTier,
    pub deoptimization_time: Duration,
    pub reason: DeoptimizationReason,
}

/// Configuration for deoptimization
#[derive(Debug, Clone)]
pub struct DeoptimizationConfig {
    pub enable_performance_deoptimization: bool,
    pub max_compilation_overhead_ratio: f64,
    pub performance_regression_threshold: f64,
}

impl Default for DeoptimizationConfig {
    fn default() -> Self {
        Self {
            enable_performance_deoptimization: true,
            max_compilation_overhead_ratio: 10.0,
            performance_regression_threshold: 20.0,
        }
    }
}

/// Statistics for deoptimization events
#[derive(Debug, Clone)]
pub struct DeoptimizationStats {
    pub events: Vec<DeoptimizationEvent>,
    pub total_executions: u64,
}

impl Default for DeoptimizationStats {
    fn default() -> Self {
        Self::new()
    }
}

impl DeoptimizationStats {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            total_executions: 0,
        }
    }

    pub fn record_deoptimization(&mut self, reason: DeoptimizationReason, time: Duration) {
        let event = DeoptimizationEvent {
            reason,
            timestamp: Instant::now(),
            deoptimization_time: time,
        };
        self.events.push(event);
    }
}

/// Individual deoptimization event
#[derive(Debug, Clone)]
pub struct DeoptimizationEvent {
    pub reason: DeoptimizationReason,
    pub timestamp: Instant,
    pub deoptimization_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deoptimization_manager_creation() {
        let config = DeoptimizationConfig::default();
        let manager = DeoptimizationManager::new(config);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_deoptimization_reason_categories() {
        let security_reason = DeoptimizationReason::SecurityViolation {
            violation_type: "test".to_string(),
        };
        assert_eq!(security_reason.get_category(), "security");
    }
}
