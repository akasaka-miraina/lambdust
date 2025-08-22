//! Test suite for continuation system (simplified).

use super::*;
use crate::ast::{Expr, Literal};
use crate::eval::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

/// Test utilities for continuation system testing.
pub struct ContinuationTestHarness {
    /// Test generation counter
    generation: ContinuationGeneration,
}

impl ContinuationTestHarness {
    /// Creates a new test harness.
    pub fn new() -> Self {
        Self { generation: 0 }
    }

    /// Creates a test continuation frame.
    pub fn create_test_frame(&mut self) -> ContinuationFrame {
        self.generation += 1;
        let id = self.generation;

        ContinuationFrame::new(
            ContinuationId::from(id),
            Expr::Literal(Literal::integer(42)),
            self.generation,
        )
    }

    /// Creates a test optimized continuation.
    pub fn create_test_continuation(&mut self) -> OptimizedContinuation {
        let frame = self.create_test_frame();
        OptimizedContinuation::single_owned(frame)
    }
}

impl Default for ContinuationTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuation_frame_creation() {
        let mut harness = ContinuationTestHarness::new();
        let frame = harness.create_test_frame();

        assert!(frame.is_valid());
        assert_eq!(frame.depth(), 0);
        assert!(frame.stack.is_empty());
        assert!(frame.locals.is_empty());
    }

    #[test]
    fn test_continuation_frame_stack_operations() {
        let mut harness = ContinuationTestHarness::new();
        let mut frame = harness.create_test_frame();

        // Test push
        frame.push_value(Value::Literal(Literal::integer(1)));
        frame.push_value(Value::Literal(Literal::integer(2)));
        assert_eq!(frame.stack.len(), 2);

        // Test pop
        let value = frame.pop_value().unwrap();
        assert_eq!(value, Value::Literal(Literal::integer(2)));
        assert_eq!(frame.stack.len(), 1);
    }

    #[test]
    fn test_continuation_frame_local_variables() {
        let mut harness = ContinuationTestHarness::new();
        let mut frame = harness.create_test_frame();

        let name = "test_var".to_string();
        let value = Value::Literal(Literal::integer(42));

        // Test define
        frame.define_local(name.clone(), value.clone());
        assert_eq!(frame.locals.len(), 1);

        // Test lookup (simplified since we use String keys now)
        assert!(frame.locals.contains_key(&name));
    }

    #[test]
    fn test_continuation_chain_creation() {
        let mut harness = ContinuationTestHarness::new();
        let frame = harness.create_test_frame();
        let frame_rc = Rc::new(RefCell::new(frame));

        let chain = ContinuationChain::from_frame(ContinuationFrameRef::Local(frame_rc));
        assert_eq!(chain.len(), 1);
        assert!(!chain.is_empty());
        assert!(chain.validate());
    }

    #[test]
    fn test_optimized_continuation_single_owned() {
        let mut harness = ContinuationTestHarness::new();
        let frame = harness.create_test_frame();
        let id = frame.id;

        let continuation = OptimizedContinuation::single_owned(frame);

        assert_eq!(continuation.id(), id);
        assert!(continuation.is_valid());
        assert!(!continuation.is_shareable());
        assert!(continuation.can_invoke());
    }

    #[test]
    fn test_optimized_continuation_shared_weak() {
        let mut harness = ContinuationTestHarness::new();
        let frame = harness.create_test_frame();
        let id = frame.id;

        let continuation = OptimizedContinuation::shared_adaptive(frame);

        assert_eq!(continuation.id(), id);
        assert!(continuation.is_valid());
        assert!(continuation.is_shareable());
        assert!(continuation.can_invoke());
    }

    #[test]
    fn test_continuation_gc_basic() {
        let mut gc = ContinuationGC::new();
        let mut continuations = HashMap::new();

        // Add some test continuations
        let mut harness = ContinuationTestHarness::new();
        for _ in 0..5 {
            let continuation = harness.create_test_continuation();
            let id = continuation.id();
            continuations.insert(id, continuation);
        }

        let initial_count = continuations.len();
        gc.collect(&mut continuations);

        let stats = gc.stats();
        assert!(stats.collection_count > 0);
    }

    #[test]
    fn test_continuation_environment_creation() {
        let generation = 1;
        let cont_env = ContinuationEnvironment::new(generation);

        assert_eq!(cont_env.generation(), generation);
    }

    #[test]
    fn test_usage_stats_optimization_decisions() {
        let mut stats = super::optimization::UsageStats::default();

        // Low usage - should not use JIT
        stats.invocation_count = 10;
        stats.avg_execution_time = 500;
        assert!(!stats.should_use_jit());
        assert!(!stats.should_share());

        // High usage - should use JIT
        stats.invocation_count = 150;
        stats.avg_execution_time = 2000;
        assert!(stats.should_use_jit());
    }

    #[test]
    fn test_continuation_registry_operations() {
        let registry = ContinuationRegistry::new();
        let mut harness = ContinuationTestHarness::new();

        // Register continuation
        let continuation = harness.create_test_continuation();
        let original_id = continuation.id();
        let id = registry.register(continuation.clone());

        // Retrieve continuation
        let retrieved = registry.get(id).unwrap();
        assert_eq!(retrieved.id(), original_id);

        // Check stats
        let stats = registry.stats();
        assert_eq!(stats.captures, 1);
    }
}
