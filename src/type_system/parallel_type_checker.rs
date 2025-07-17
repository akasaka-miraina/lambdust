//! Parallel Type Checking System
//! This module implements parallel type checking to challenge GHC's compilation speed
//! while maintaining the same level of type safety.

#![cfg(feature = "parallel-type-checking")]

use super::polynomial_types::{PolynomialType, UniverseLevel};
use super::type_checker::TypeChecker;
use super::type_inference::TypeInference;
use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use std::collections::HashMap;
use std::time::Duration;

/// Type checking task for parallel processing
#[derive(Debug, Clone)]
pub struct TypeCheckTask {
    /// Unique identifier for the task
    pub id: TaskId,
    /// Expression to type check
    pub expr: Expr,
    /// Context dependencies (other tasks this depends on)
    pub dependencies: Vec<TaskId>,
    /// Priority level (higher = more urgent)
    pub priority: Priority,
    /// Module or scope identifier
    pub scope: String,
}

/// Task identifier
pub type TaskId = usize;

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Low priority - can wait
    Low = 1,
    /// Normal priority - standard processing
    Normal = 2,
    /// High priority - expedite processing
    High = 3,
    /// Critical priority - process immediately
    Critical = 4,
}

/// Result of a type checking task
#[derive(Debug, Clone)]
pub struct TypeCheckResult {
    /// Task ID that produced this result
    pub task_id: TaskId,
    /// Inferred type
    pub inferred_type: PolynomialType,
    /// Processing time
    pub processing_time: Duration,
    /// Any warnings generated
    pub warnings: Vec<String>,
}

/// Parallel type checker main structure (simplified implementation)
#[derive(Debug, Clone)]
pub struct ParallelTypeChecker {
    /// Number of worker threads
    worker_count: usize,
    /// Sequential type checker fallback
    type_checker: TypeChecker,
    /// Performance metrics
    metrics: ParallelTypeCheckMetrics,
}

/// Performance metrics for parallel type checking
#[derive(Debug, Clone)]
pub struct ParallelTypeCheckMetrics {
    /// Total tasks processed
    pub total_tasks: usize,
    /// Total processing time
    pub total_time: Duration,
    /// Average time per task
    pub avg_time_per_task: Duration,
    /// Number of workers used
    pub workers_used: usize,
    /// Speedup factor achieved
    pub speedup_factor: f64,
    /// Parallel efficiency
    pub efficiency: f64,
}

impl ParallelTypeChecker {
    /// Create new parallel type checker
    pub fn new(worker_count: usize) -> Self {
        Self {
            worker_count,
            type_checker: TypeChecker::new(),
            metrics: ParallelTypeCheckMetrics {
                total_tasks: 0,
                total_time: Duration::ZERO,
                avg_time_per_task: Duration::ZERO,
                workers_used: worker_count,
                speedup_factor: 1.0,
                efficiency: 1.0 / worker_count as f64,
            },
        }
    }

    /// Type check multiple expressions in parallel
    /// Currently falls back to sequential processing to avoid Send/Sync issues
    pub fn type_check_parallel(
        &mut self,
        expressions: Vec<(Expr, String)>, // (expression, scope)
    ) -> Result<Vec<TypeCheckResult>> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();
        
        for (i, (expr, _scope)) in expressions.into_iter().enumerate() {
            let task_start = std::time::Instant::now();
            
            // Fallback to sequential type checking
            let inferred_type = self.type_checker.infer_type(&expr)?;
            
            let processing_time = task_start.elapsed();
            
            results.push(TypeCheckResult {
                task_id: i,
                inferred_type,
                processing_time,
                warnings: Vec::new(),
            });
        }
        
        self.metrics.total_tasks += results.len();
        self.metrics.total_time += start_time.elapsed();
        if self.metrics.total_tasks > 0 {
            self.metrics.avg_time_per_task = self.metrics.total_time / self.metrics.total_tasks as u32;
        }
        
        Ok(results)
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> &ParallelTypeCheckMetrics {
        &self.metrics
    }

    /// Reset metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = ParallelTypeCheckMetrics {
            total_tasks: 0,
            total_time: Duration::ZERO,
            avg_time_per_task: Duration::ZERO,
            workers_used: self.worker_count,
            speedup_factor: 1.0,
            efficiency: 1.0 / self.worker_count as f64,
        };
    }

    /// Auto type check (wrapper for type_check_parallel)
    pub fn type_check_auto(
        &mut self,
        expressions: Vec<(Expr, String)>,
    ) -> Result<Vec<TypeCheckResult>> {
        self.type_check_parallel(expressions)
    }

    /// Add type binding (placeholder implementation)
    pub fn add_type_binding(&mut self, _name: String, _typ: PolynomialType) {
        // Placeholder - in full implementation this would update type environment
    }
}

impl Default for ParallelTypeChecker {
    fn default() -> Self {
        // Use a reasonable default worker count
        Self::new(std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(8))
    }
}

/// Type checking error
#[derive(Debug, Clone)]
pub enum TypeCheckError {
    /// Type inference failed
    InferenceFailed(String),
    /// Dependency cycle detected
    CyclicDependency(Vec<TaskId>),
    /// Task not found
    TaskNotFound(TaskId),
    /// Internal error
    Internal(String),
}

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeCheckError::InferenceFailed(msg) => write!(f, "Type inference failed: {}", msg),
            TypeCheckError::CyclicDependency(tasks) => write!(f, "Cyclic dependency detected in tasks: {:?}", tasks),
            TypeCheckError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            TypeCheckError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for TypeCheckError {}

impl From<TypeCheckError> for LambdustError {
    fn from(err: TypeCheckError) -> Self {
        LambdustError::runtime_error(err.to_string())
    }
}