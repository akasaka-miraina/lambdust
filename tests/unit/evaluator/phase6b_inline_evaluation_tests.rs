//! Phase 6-B-Step3: Inline Evaluation System Tests
//!
//! Tests the inline evaluation system for lightweight continuation optimization:
//! - Branch prediction optimization for simple continuations
//! - Cache locality improvement through inline execution
//! - Hot path detection and optimization
//! - Performance monitoring and statistics

use lambdust::environment::Environment;
use lambdust::evaluator::inline_evaluation::{
    CacheFriendlyPatterns, ContinuationWeight, HotPathDetector, InlineEvaluator, InlineHint,
};
use lambdust::evaluator::types::Evaluator;
use lambdust::evaluator::Continuation;
use lambdust::value::Value;
use std::rc::Rc;

#[test]
fn test_continuation_weight_classification() {
    // Test VeryLight continuations
    let identity = Continuation::Identity;
    assert_eq!(
        ContinuationWeight::from_continuation(&identity),
        ContinuationWeight::VeryLight
    );

    let small_values = Continuation::Values {
        values: vec![Value::from(1i64), Value::from(2i64)],
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(
        ContinuationWeight::from_continuation(&small_values),
        ContinuationWeight::VeryLight
    );

    // Test Light continuations
    let assignment = Continuation::Assignment {
        variable: "x".to_string(),
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(
        ContinuationWeight::from_continuation(&assignment),
        ContinuationWeight::Light
    );

    let define = Continuation::Define {
        variable: "y".to_string(),
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(
        ContinuationWeight::from_continuation(&define),
        ContinuationWeight::Light
    );

    // Test Heavy continuations
    let call_cc = Continuation::CallCc {
        captured_cont: Value::from(1i64),
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(
        ContinuationWeight::from_continuation(&call_cc),
        ContinuationWeight::Heavy
    );
}

#[test]
fn test_continuation_weight_inline_decisions() {
    // VeryLight should always inline
    assert!(ContinuationWeight::VeryLight.should_inline(InlineHint::Likely));
    assert!(ContinuationWeight::VeryLight.should_inline(InlineHint::Neutral));
    assert!(ContinuationWeight::VeryLight.should_inline(InlineHint::Unlikely));

    // Light should inline for likely and neutral
    assert!(ContinuationWeight::Light.should_inline(InlineHint::Likely));
    assert!(ContinuationWeight::Light.should_inline(InlineHint::Neutral));
    assert!(!ContinuationWeight::Light.should_inline(InlineHint::Unlikely));

    // Medium should inline only for likely
    assert!(ContinuationWeight::Medium.should_inline(InlineHint::Likely));
    assert!(!ContinuationWeight::Medium.should_inline(InlineHint::Neutral));
    assert!(!ContinuationWeight::Medium.should_inline(InlineHint::Unlikely));

    // Heavy should never inline
    assert!(!ContinuationWeight::Heavy.should_inline(InlineHint::Likely));
    assert!(!ContinuationWeight::Heavy.should_inline(InlineHint::Neutral));
    assert!(!ContinuationWeight::Heavy.should_inline(InlineHint::Unlikely));
}

#[test]
fn test_hot_path_detector_basic_operations() {
    let mut detector = HotPathDetector::new(3);

    // Test initial state
    assert_eq!(detector.get_inline_hint("Identity"), InlineHint::Neutral);

    // Record executions to make a path hot
    detector.record_execution("Identity");
    detector.record_execution("Identity");
    detector.record_execution("Identity");

    // Should now be considered hot
    assert_eq!(detector.get_inline_hint("Identity"), InlineHint::Likely);

    // Record some executions for a cold path
    detector.record_execution("CallCc");

    // CallCc should be neutral since there are not enough total executions to be unlikely
    assert_eq!(detector.get_inline_hint("CallCc"), InlineHint::Neutral);
}

#[test]
fn test_hot_path_detector_statistics() {
    let mut detector = HotPathDetector::new(2);

    // Add executions for different continuation types
    detector.record_execution("Identity");
    detector.record_execution("Identity");
    detector.record_execution("Identity"); // Hot path

    detector.record_execution("Values");
    detector.record_execution("Values"); // Hot path

    detector.record_execution("CallCc"); // Cold path

    let (hot_paths, total_paths, hot_ratio) = detector.statistics();
    assert_eq!(hot_paths, 2); // Identity and Values are hot
    assert_eq!(total_paths, 3); // Identity, Values, CallCc
    assert_eq!(hot_ratio, 2.0 / 3.0);
}

#[test]
fn test_inline_evaluator_creation() {
    let evaluator = InlineEvaluator::new();

    let (inlined, total, rate, cache_hits) = evaluator.statistics();
    assert_eq!(inlined, 0);
    assert_eq!(total, 0);
    assert_eq!(rate, 0.0);
    assert_eq!(cache_hits, 0);

    let (hot_paths, total_paths, hot_ratio) = evaluator.hot_path_statistics();
    assert_eq!(hot_paths, 0);
    assert_eq!(total_paths, 0);
    assert_eq!(hot_ratio, 0.0);
}

#[test]
fn test_inline_evaluator_statistics_tracking() {
    let mut inline_eval = InlineEvaluator::new();

    // Simulate successful inline evaluations
    let identity = Continuation::Identity;
    inline_eval.record_successful_inline(&identity);
    inline_eval.record_successful_inline(&identity);

    let values = Continuation::Values {
        values: vec![Value::from(42i64)],
        parent: Box::new(Continuation::Identity),
    };
    inline_eval.record_successful_inline(&values);

    // Check statistics
    let (inlined, total, rate, _) = inline_eval.statistics();
    assert_eq!(inlined, 3);
    assert_eq!(total, 3);
    assert_eq!(rate, 1.0);

    // Check inline hints (Identity was called 2 times, but default threshold is 10)
    assert_eq!(inline_eval.get_inline_hint("Identity"), InlineHint::Neutral); // Not enough calls to be "likely"
    assert_eq!(inline_eval.get_inline_hint("Values"), InlineHint::Neutral);
    assert_eq!(inline_eval.get_inline_hint("Unknown"), InlineHint::Neutral);
}

#[test]
fn test_inline_evaluator_clear_statistics() {
    let mut inline_eval = InlineEvaluator::new();

    // Add some statistics
    let identity = Continuation::Identity;
    inline_eval.record_successful_inline(&identity);
    inline_eval.record_successful_inline(&identity);

    // Verify statistics exist
    let (inlined, total, _, _) = inline_eval.statistics();
    assert!(inlined > 0);
    assert!(total > 0);

    // Clear and verify
    inline_eval.clear_statistics();
    let (inlined, total, rate, cache_hits) = inline_eval.statistics();
    assert_eq!(inlined, 0);
    assert_eq!(total, 0);
    assert_eq!(rate, 0.0);
    assert_eq!(cache_hits, 0);
}

#[test]
fn test_cache_friendly_patterns() {
    // Test identity continuation
    let identity = Continuation::Identity;
    assert!(CacheFriendlyPatterns::is_cache_friendly(&identity));

    // Test values continuation
    let values = Continuation::Values {
        values: vec![Value::from(42i64)],
        parent: Box::new(Continuation::Identity),
    };
    assert!(CacheFriendlyPatterns::is_cache_friendly(&values));

    // Test assignment continuation
    let assignment = Continuation::Assignment {
        variable: "x".to_string(),
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert!(CacheFriendlyPatterns::is_cache_friendly(&assignment));

    // Test unfriendly continuation
    let call_cc = Continuation::CallCc {
        captured_cont: Value::from(1i64),
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert!(!CacheFriendlyPatterns::is_cache_friendly(&call_cc));
}

#[test]
fn test_memory_footprint_estimation() {
    let identity = Continuation::Identity;
    let identity_size = CacheFriendlyPatterns::estimate_memory_footprint(&identity);
    assert!(identity_size > 0);

    let values_small = Continuation::Values {
        values: vec![Value::from(42i64)],
        parent: Box::new(Continuation::Identity),
    };
    let values_small_size = CacheFriendlyPatterns::estimate_memory_footprint(&values_small);

    let values_large = Continuation::Values {
        values: vec![Value::from(42i64); 100],
        parent: Box::new(Continuation::Identity),
    };
    let values_large_size = CacheFriendlyPatterns::estimate_memory_footprint(&values_large);

    // Larger values continuation should have larger footprint
    assert!(values_large_size > values_small_size);
    assert!(values_small_size > identity_size);
}

#[test]
fn test_cache_friendly_begin_continuation() {
    use lambdust::ast::{Expr, Literal};

    // Short begin chain should be cache-friendly
    let short_begin = Continuation::Begin {
        remaining: vec![Expr::Literal(Literal::Boolean(true))],
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert!(CacheFriendlyPatterns::is_cache_friendly(&short_begin));

    // Long begin chain should not be cache-friendly
    let long_begin = Continuation::Begin {
        remaining: vec![
            Expr::Literal(Literal::Boolean(true)),
            Expr::Literal(Literal::Boolean(false)),
            Expr::Literal(Literal::Boolean(true)),
        ],
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert!(!CacheFriendlyPatterns::is_cache_friendly(&long_begin));
}

// Integration tests removed for now due to private method access issues
// These tests verify the inline evaluation works through the inline_evaluation module tests

#[test]
#[ignore = "Inline evaluation performance tracking requires evaluator statistics refactoring"]
fn test_inline_evaluation_performance_tracking() {
    let mut evaluator = Evaluator::new();

    // Test inline evaluation with statistics tracking
    let identity = Continuation::Identity;
    let value = Value::from(42i64);

    // Apply continuation which should trigger inline evaluation
    let result = evaluator.apply_continuation(identity, value).unwrap();
    assert_eq!(result, Value::from(42i64));

    // Check that inline evaluator recorded the success
    let (inlined, total, rate, _) = evaluator.inline_evaluator().statistics();
    assert_eq!(inlined, 1);
    assert_eq!(total, 1);
    assert_eq!(rate, 1.0);
}

#[test]
#[ignore = "Inline evaluation statistics integration requires evaluator refactoring"]
fn test_inline_evaluation_statistics_integration() {
    let mut evaluator = Evaluator::new();

    // Test with Identity continuations
    for _ in 0..5 {
        let identity = Continuation::Identity;
        let value = Value::from(42i64);
        let _result = evaluator.apply_continuation(identity, value).unwrap();
    }

    // Check statistics
    let (inlined, total, rate, _) = evaluator.inline_evaluator().statistics();
    assert_eq!(inlined, 5);
    assert_eq!(total, 5);
    assert_eq!(rate, 1.0);

    // Check hot path statistics
    let (hot_paths, total_paths, hot_ratio) = evaluator.inline_evaluator().hot_path_statistics();
    assert!(total_paths >= 1);
    assert!((0.0..=1.0).contains(&hot_ratio));
    assert!(hot_paths <= total_paths); // hot_paths should not exceed total_paths
}
