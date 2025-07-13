//! `DoLoop` specialized continuation implementation
//!
//! This module implements optimized do-loop continuation handling with:
//! - State machine optimization for iteration tracking
//! - Memory pool integration for continuation reuse
//! - Inline evaluation for simple loops
//! - Performance tracking and optimization hints

use crate::error::Result;
use crate::evaluator::{Continuation, DoLoopState, Evaluator};
use crate::value::Value;

#[cfg(test)]
use crate::ast::Expr;

impl Evaluator {
    /// Apply `DoLoop` specialized continuation
    /// High-performance iteration handling with memory pool integration
    pub fn apply_doloop_continuation(
        &mut self,
        test_value: Value,
        mut iteration_state: DoLoopState,
        pool_id: Option<usize>,
        parent: Continuation,
    ) -> Result<Value> {
        // Increment iteration counter and check bounds
        iteration_state.next_iteration()?;

        // Update optimization tracking
        if iteration_state.can_optimize() && !iteration_state.is_optimized {
            iteration_state.mark_optimized();
            self.record_doloop_optimization(&iteration_state);
        }

        // Check termination condition
        let test_is_true = match test_value {
            Value::Boolean(false) => false,
            _ => true, // Everything except #f is true in Scheme
        };

        if test_is_true {
            // Loop terminated - evaluate result expressions
            self.handle_doloop_termination(iteration_state, parent, pool_id)
        } else {
            // Continue iteration - execute body and update variables
            self.handle_doloop_iteration(iteration_state, parent, pool_id)
        }
    }

    /// Handle loop termination and result evaluation
    fn handle_doloop_termination(
        &mut self,
        iteration_state: DoLoopState,
        parent: Continuation,
        pool_id: Option<usize>,
    ) -> Result<Value> {
        // Return continuation to pool if available
        if let Some(id) = pool_id {
            self.return_continuation_to_pool(id);
        }

        // Evaluate result expressions
        if iteration_state.result_exprs.is_empty() {
            // No result expressions, return undefined
            self.apply_evaluator_continuation(parent, Value::Undefined)
        } else if iteration_state.result_exprs.len() == 1 {
            // Single result expression
            self.eval_with_continuation(
                iteration_state.result_exprs[0].clone(),
                iteration_state.loop_env,
                parent,
            )
        } else {
            // Multiple result expressions, evaluate as sequence
            self.eval_sequence(
                iteration_state.result_exprs,
                iteration_state.loop_env,
                parent,
            )
        }
    }

    /// Handle loop iteration and variable updates
    fn handle_doloop_iteration(
        &mut self,
        mut iteration_state: DoLoopState,
        parent: Continuation,
        pool_id: Option<usize>,
    ) -> Result<Value> {
        // Execute body expressions (side effects)
        if !iteration_state.body_exprs.is_empty() {
            for body_expr in &iteration_state.body_exprs {
                self.eval_with_continuation(
                    body_expr.clone(),
                    iteration_state.loop_env.clone(),
                    Continuation::Identity,
                )?;
            }
        }

        // Update variables with step expressions
        let updated_variables = self.update_doloop_variables(&iteration_state)?;
        iteration_state.update_variables(updated_variables);

        // Update loop environment with new variable values
        for (var_name, new_value) in &iteration_state.variables {
            iteration_state.loop_env.set(var_name, new_value.clone())?;
        }

        // Clone needed values before creating continuation
        let test_expr = iteration_state.test_expr.clone();
        let loop_env = iteration_state.loop_env.clone();

        // Create next iteration continuation
        let doloop_cont = if iteration_state.is_optimized {
            // Use optimized continuation for simple loops
            self.create_optimized_doloop_continuation(iteration_state, parent, pool_id)
        } else {
            // Use standard continuation
            Continuation::DoLoop {
                iteration_state,
                pool_id,
                parent: Box::new(parent),
            }
        };

        // Re-evaluate test expression for next iteration
        self.eval_with_continuation(test_expr, loop_env, doloop_cont)
    }

    /// Update loop variables with step expressions
    fn update_doloop_variables(
        &mut self,
        iteration_state: &DoLoopState,
    ) -> Result<Vec<(String, Value)>> {
        let mut updated_variables = Vec::new();

        for (i, (var_name, current_value)) in iteration_state.variables.iter().enumerate() {
            let new_value =
                if let Some(step_expr) = iteration_state.step_exprs.get(i).unwrap_or(&None) {
                    // Evaluate step expression
                    self.eval_with_continuation(
                        step_expr.clone(),
                        iteration_state.loop_env.clone(),
                        Continuation::Identity,
                    )?
                } else {
                    // No step expression - keep current value
                    current_value.clone()
                };

            updated_variables.push((var_name.clone(), new_value));
        }

        Ok(updated_variables)
    }

    /// Create optimized continuation for simple loops
    fn create_optimized_doloop_continuation(
        &mut self,
        iteration_state: DoLoopState,
        parent: Continuation,
        pool_id: Option<usize>,
    ) -> Continuation {
        // For optimized loops, try to reuse continuation from pool
        if let Some(reused_cont) = self.try_reuse_continuation_from_pool(pool_id) {
            reused_cont
        } else {
            // Create new optimized continuation
            Continuation::DoLoop {
                iteration_state,
                pool_id,
                parent: Box::new(parent),
            }
        }
    }

    /// Record `DoLoop` optimization for statistics
    fn record_doloop_optimization(&mut self, _iteration_state: &DoLoopState) {
        // Update expression analyzer with optimization information
        // Note: Using placeholder implementation since record_optimization is not available
        // In a full implementation, this would track optimization statistics
        let _stats = self.expression_analyzer().optimization_stats();
    }

    /// Try to reuse continuation from memory pool
    fn try_reuse_continuation_from_pool(&mut self, pool_id: Option<usize>) -> Option<Continuation> {
        // Placeholder implementation - in a full implementation this would
        // check a continuation pool for reusable continuations
        if let Some(_id) = pool_id {
            // For now, return None to use new continuation
            // Future implementation would maintain a pool of reusable continuations
            None
        } else {
            None
        }
    }

    /// Return continuation to pool for reuse
    fn return_continuation_to_pool(&mut self, _pool_id: usize) {
        // Placeholder implementation - in a full implementation this would
        // return the continuation to a memory pool for later reuse
        // This helps reduce heap allocation/deallocation overhead
    }
}

/// `DoLoop` continuation pool for memory optimization
/// Continuation reuse system
#[derive(Debug)]
pub struct DoLoopContinuationPool {
    /// Pool of reusable continuations
    pool: Vec<Continuation>,
    /// Maximum pool size to prevent unbounded growth
    max_size: usize,
    /// Statistics for pool utilization
    allocations: usize,
    /// Number of reuses
    reuses: usize,
}

impl DoLoopContinuationPool {
    /// Create new continuation pool
    #[must_use] pub fn new(max_size: usize) -> Self {
        DoLoopContinuationPool {
            pool: Vec::with_capacity(max_size),
            max_size,
            allocations: 0,
            reuses: 0,
        }
    }

    /// Allocate continuation from pool or create new one
    pub fn allocate(
        &mut self,
        iteration_state: DoLoopState,
        parent: Continuation,
    ) -> (Continuation, Option<usize>) {
        if let Some(mut reused_cont) = self.pool.pop() {
            // Reuse existing continuation
            if let Continuation::DoLoop {
                iteration_state: ref mut state,
                pool_id: ref mut id,
                parent: ref mut p,
            } = reused_cont
            {
                *state = iteration_state;
                *p = Box::new(parent);
                let pool_id = *id;
                self.reuses += 1;
                (reused_cont, pool_id)
            } else {
                // Pool contained wrong type, create new
                self.allocations += 1;
                let pool_id = Some(self.allocations);
                (
                    Continuation::DoLoop {
                        iteration_state,
                        pool_id,
                        parent: Box::new(parent),
                    },
                    pool_id,
                )
            }
        } else {
            // Create new continuation
            self.allocations += 1;
            let pool_id = Some(self.allocations);
            (
                Continuation::DoLoop {
                    iteration_state,
                    pool_id,
                    parent: Box::new(parent),
                },
                pool_id,
            )
        }
    }

    /// Return continuation to pool
    pub fn deallocate(&mut self, cont: Continuation) {
        if self.pool.len() < self.max_size {
            if let Continuation::DoLoop { .. } = cont {
                self.pool.push(cont);
            }
        }
        // If pool is full or wrong type, just drop the continuation
    }

    /// Get pool statistics
    #[must_use] pub fn statistics(&self) -> (usize, usize, f64) {
        let reuse_rate = if self.allocations > 0 {
            self.reuses as f64 / self.allocations as f64
        } else {
            0.0
        };
        (self.allocations, self.reuses, reuse_rate)
    }

    /// Clear pool
    pub fn clear(&mut self) {
        self.pool.clear();
    }
}

impl Default for DoLoopContinuationPool {
    fn default() -> Self {
        Self::new(100) // Default pool size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::environment::Environment;
    use std::rc::Rc;

    #[test]
    fn test_doloop_continuation_pool() {
        let mut pool = DoLoopContinuationPool::new(2);
        let env = Rc::new(Environment::new());

        // Create test DoLoopState
        let state = DoLoopState::new(
            vec![("i".to_string(), Value::from(0i64))],
            vec![None],
            Expr::Literal(Literal::Boolean(true)),
            vec![],
            vec![],
            env,
        );

        // Test allocation
        let (cont1, id1) = pool.allocate(state.clone(), Continuation::Identity);
        assert!(matches!(cont1, Continuation::DoLoop { .. }));
        assert!(id1.is_some());

        // Test pool statistics
        let (allocs, reuses, rate) = pool.statistics();
        assert_eq!(allocs, 1);
        assert_eq!(reuses, 0);
        assert_eq!(rate, 0.0);

        // Test deallocation
        pool.deallocate(cont1);

        // Test reuse
        let (cont2, id2) = pool.allocate(state, Continuation::Identity);
        assert!(matches!(cont2, Continuation::DoLoop { .. }));
        assert_eq!(id2, id1); // Should reuse same ID

        let (allocs, reuses, rate) = pool.statistics();
        assert_eq!(allocs, 1);
        assert_eq!(reuses, 1);
        assert_eq!(rate, 1.0);
    }

    #[test]
    fn test_doloop_state_optimization() {
        let env = Rc::new(Environment::new());
        let mut state = DoLoopState::new(
            vec![("i".to_string(), Value::from(0i64))],
            vec![None],
            Expr::Literal(Literal::Boolean(true)),
            vec![],
            vec![],
            env,
        );

        // Test optimization candidacy
        assert!(state.can_optimize());
        assert!(!state.is_optimized);

        // Test optimization marking
        state.mark_optimized();
        assert!(state.is_optimized);

        // Test iteration increment
        assert!(state.next_iteration().is_ok());
        assert_eq!(state.iteration_count, 1);

        // Test memory usage calculation
        let usage = state.memory_usage();
        assert!(usage > 0);
    }

    #[test]
    fn test_doloop_state_iteration_limit() {
        let env = Rc::new(Environment::new());
        let mut state = DoLoopState::new(
            vec![("i".to_string(), Value::from(0i64))],
            vec![None],
            Expr::Literal(Literal::Boolean(true)),
            vec![],
            vec![],
            env,
        );

        // Set low iteration limit for testing
        state.max_iterations = 5;

        // Test normal iterations
        for _ in 0..5 {
            assert!(state.next_iteration().is_ok());
        }

        // Test iteration limit exceeded
        let result = state.next_iteration();
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(format!("{:?}", e).contains("exceeded maximum iterations"));
        }
    }
}
