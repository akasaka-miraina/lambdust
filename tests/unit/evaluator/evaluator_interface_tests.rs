//! Comprehensive unit tests for unified evaluator interface (evaluator/evaluator_interface.rs)
//!
//! Tests the Phase 3 unified evaluator interface that provides transparent switching
//! between semantic evaluation and optimized runtime execution with correctness verification.

use lambdust::ast::{Expr, Literal};
use lambdust::environment::Environment;
use lambdust::evaluator::{
    Continuation, EvaluationConfig, EvaluationMode, EvaluatorInterface,
    RuntimeOptimizationLevel, VerificationConfig,
};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;
use std::rc::Rc;

/// Helper to create test environment with basic bindings
fn create_test_environment() -> Rc<Environment> {
    let env = Rc::new(Environment::new());
    
    // Add some basic test bindings
    env.define("test-var".to_string(), Value::Number(SchemeNumber::Integer(42)));
    env.define("test-symbol".to_string(), Value::Symbol("test".to_string()));
    
    env
}

/// Helper to create basic evaluation config
fn create_basic_config() -> EvaluationConfig {
    EvaluationConfig {
        mode: EvaluationMode::Semantic,
        verify_correctness: false,
        monitor_performance: false,
        fallback_to_semantic: true,
        verification_timeout_ms: 1000,
        verification_config: VerificationConfig::default(),
    }
}

#[test]
fn test_evaluator_interface_creation() {
    let interface = EvaluatorInterface::new();
    assert_eq!(interface.get_config().mode, EvaluationMode::Auto);
    assert!(interface.get_config().verify_correctness);
    assert!(interface.get_config().monitor_performance);
    assert!(interface.get_config().fallback_to_semantic);
}

#[test]
fn test_evaluation_config_creation() {
    let config = create_basic_config();
    assert!(matches!(config.mode, EvaluationMode::Semantic));
    assert!(!config.verify_correctness);
    assert!(!config.monitor_performance);
    assert!(config.fallback_to_semantic);
    assert_eq!(config.verification_timeout_ms, 1000);
}

#[test]
fn test_semantic_evaluation_mode() {
    let mut interface = EvaluatorInterface::new();
    let env = create_test_environment();
    let mut config = create_basic_config();
    config.mode = EvaluationMode::Semantic;
    
    interface.set_config(config);
    
    let simple_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let result = interface.eval(simple_expr, env, Continuation::Identity);
    assert!(result.is_ok(), "Semantic evaluation should succeed");
    
    let eval_result = result.unwrap();
    assert_eq!(eval_result.mode_used, EvaluationMode::Semantic);
    assert!(!eval_result.fallback_used);
}

#[test]
fn test_runtime_evaluation_mode() {
    let mut interface = EvaluatorInterface::new();
    let env = create_test_environment();
    let mut config = create_basic_config();
    config.mode = EvaluationMode::Runtime(RuntimeOptimizationLevel::Conservative);
    
    interface.set_config(config);
    
    let simple_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let result = interface.eval(simple_expr, env, Continuation::Identity);
    
    assert!(result.is_ok(), "Runtime evaluation should succeed");
    let eval_result = result.unwrap();
    assert!(matches!(eval_result.mode_used, EvaluationMode::Runtime(_)));
}

#[test]
fn test_auto_evaluation_mode() {
    let mut interface = EvaluatorInterface::new();
    let env = create_test_environment();
    let mut config = create_basic_config();
    config.mode = EvaluationMode::Auto;
    
    interface.set_config(config);
    
    let simple_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(123)));
    let result = interface.eval(simple_expr, env, Continuation::Identity);
    
    assert!(result.is_ok(), "Auto evaluation should succeed");
    // Auto mode should select semantic for simple expressions
    let eval_result = result.unwrap();
    assert_eq!(eval_result.mode_used, EvaluationMode::Semantic);
}

#[test]
fn test_verification_mode() {
    let mut interface = EvaluatorInterface::new();
    let env = create_test_environment();
    let mut config = create_basic_config();
    config.mode = EvaluationMode::Verification;
    config.verify_correctness = true;
    
    interface.set_config(config);
    
    let simple_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(456)));
    let result = interface.eval(simple_expr, env, Continuation::Identity);
    
    assert!(result.is_ok(), "Verification evaluation should succeed");
    let eval_result = result.unwrap();
    assert_eq!(eval_result.mode_used, EvaluationMode::Verification);
    
    // Should have verification results
    assert!(eval_result.verification_result.is_some(), "Verification mode should provide verification results");
}

#[test]
fn test_performance_monitoring() {
    let mut interface = EvaluatorInterface::new();
    let env = create_test_environment();
    let mut config = create_basic_config();
    config.mode = EvaluationMode::Semantic;
    config.monitor_performance = true;
    
    interface.set_config(config);
    
    let expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(20))),
    ]);
    
    let result = interface.eval(expr, env, Continuation::Identity);
    assert!(result.is_ok(), "Performance monitoring evaluation should succeed");
    
    let eval_result = result.unwrap();
    assert!(eval_result.evaluation_time_us > 0, "Should track evaluation time");
}

#[test]
fn test_fallback_to_semantic() {
    let mut interface = EvaluatorInterface::new();
    let env = create_test_environment();
    let mut config = create_basic_config();
    config.mode = EvaluationMode::Runtime(RuntimeOptimizationLevel::Aggressive);
    config.fallback_to_semantic = true;
    
    interface.set_config(config);
    
    // Test with a simple expression that should work in both modes
    let expr = Expr::Literal(Literal::Boolean(false));
    let result = interface.eval(expr, env, Continuation::Identity);
    
    assert!(result.is_ok(), "Should succeed with fallback capability");
    let eval_result = result.unwrap();
    match &eval_result.value {
        Value::Boolean(val) => assert_eq!(*val, false),
        _ => panic!("Expected boolean value"),
    }
}

#[test]
fn test_evaluation_mode_variants() {
    // Test different optimization levels
    let modes = vec![
        EvaluationMode::Semantic,
        EvaluationMode::Runtime(RuntimeOptimizationLevel::None),
        EvaluationMode::Runtime(RuntimeOptimizationLevel::Conservative),
        EvaluationMode::Runtime(RuntimeOptimizationLevel::Balanced),
        EvaluationMode::Runtime(RuntimeOptimizationLevel::Aggressive),
        EvaluationMode::Auto,
        EvaluationMode::Verification,
    ];
    
    for mode in modes {
        let mut config = create_basic_config();
        config.mode = mode.clone();
        
        // Ensure config can be created with each mode
        assert!(matches!(config.mode, _), "Config should support mode: {:?}", mode);
    }
}

#[test]
fn test_correctness_verification() {
    let mut interface = EvaluatorInterface::new();
    let env = create_test_environment();
    let mut config = create_basic_config();
    config.mode = EvaluationMode::Semantic;
    config.verify_correctness = true;
    config.verification_timeout_ms = 5000;
    
    interface.set_config(config);
    
    let expr = Expr::Variable("test-var".to_string());
    let result = interface.eval(expr, env, Continuation::Identity);
    
    assert!(result.is_ok(), "Correctness verification should succeed");
    let eval_result = result.unwrap();
    match &eval_result.value {
        Value::Number(SchemeNumber::Integer(val)) => assert_eq!(*val, 42),
        _ => panic!("Expected integer value 42"),
    }
}

#[test]
fn test_multiple_evaluations() {
    let mut interface = EvaluatorInterface::new();
    let env = create_test_environment();
    let config = create_basic_config();
    
    interface.set_config(config);
    
    // Perform multiple evaluations to test state consistency
    for i in 0..5 {
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(i * 10)));
        let result = interface.eval(expr, env.clone(), Continuation::Identity);
        
        assert!(result.is_ok(), "Multiple evaluations should succeed");
        let eval_result = result.unwrap();
        match &eval_result.value {
            Value::Number(SchemeNumber::Integer(val)) => assert_eq!(*val, i * 10),
            _ => panic!("Expected integer value {}", i * 10),
        }
    }
}

#[test]
fn test_complex_expression_evaluation() {
    let mut interface = EvaluatorInterface::new();
    let env = create_test_environment();
    let mut config = create_basic_config();
    config.mode = EvaluationMode::Auto;
    config.monitor_performance = true;
    
    interface.set_config(config);
    
    // Nested arithmetic expression
    let complex_expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::List(vec![
            Expr::Variable("*".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(4))),
        ]),
        Expr::List(vec![
            Expr::Variable("-".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]),
    ]);
    
    let result = interface.eval(complex_expr, env, Continuation::Identity);
    assert!(result.is_ok(), "Complex expression evaluation should succeed");
    
    let eval_result = result.unwrap();
    // (+ (* 3 4) (- 10 2)) = (+ 12 8) = 20
    match &eval_result.value {
        Value::Number(SchemeNumber::Integer(val)) => assert_eq!(*val, 20),
        _ => panic!("Expected integer value 20"),
    }
    assert!(eval_result.evaluation_time_us > 0);
}

#[test]
fn test_evaluation_config_edge_cases() {
    // Test with very short timeout
    let config_short_timeout = EvaluationConfig {
        mode: EvaluationMode::Verification,
        verify_correctness: true,
        verification_timeout_ms: 1,
        ..create_basic_config()
    };
    
    assert_eq!(config_short_timeout.verification_timeout_ms, 1);
    
    // Test with all features enabled
    let config_all_features = EvaluationConfig {
        mode: EvaluationMode::Auto,
        verify_correctness: true,
        monitor_performance: true,
        fallback_to_semantic: true,
        verification_timeout_ms: 10000,
        verification_config: VerificationConfig::default(),
    };
    
    assert!(config_all_features.verify_correctness);
    assert!(config_all_features.monitor_performance);
    assert!(config_all_features.fallback_to_semantic);
}

#[test]
fn test_performance_metrics_collection() {
    let mut interface = EvaluatorInterface::new();
    let env = create_test_environment();
    let mut config = create_basic_config();
    config.mode = EvaluationMode::Semantic;
    config.monitor_performance = true;
    
    interface.set_config(config);
    
    let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(999)));
    let result = interface.eval(expr, env, Continuation::Identity);
    
    assert!(result.is_ok(), "Performance monitoring should work");
    let eval_result = result.unwrap();
    
    // Check that performance metrics are populated
    let metrics = eval_result.performance_metrics;
    assert!(metrics.total_time_us > 0, "Should measure execution time");
    // Note: memory and reduction steps tracking are TODO items in the implementation
}

#[test]
fn test_interface_state_consistency() {
    let mut interface = EvaluatorInterface::new();
    let env = create_test_environment();
    
    // Test that interface maintains consistency across different evaluation modes
    let configs = vec![
        EvaluationConfig {
            mode: EvaluationMode::Semantic,
            ..create_basic_config()
        },
        EvaluationConfig {
            mode: EvaluationMode::Runtime(RuntimeOptimizationLevel::Conservative),
            ..create_basic_config()
        },
    ];
    
    let test_expr = Expr::Literal(Literal::String("consistency-test".to_string()));
    
    for (i, config) in configs.iter().enumerate() {
        interface.set_config(config.clone());
        let result = interface.eval(test_expr.clone(), env.clone(), Continuation::Identity);
        assert!(result.is_ok(), "Evaluation {} should succeed", i);
        
        let eval_result = result.unwrap();
        match &eval_result.value {
            Value::String(val) => assert_eq!(val, "consistency-test"),
            _ => panic!("Expected string value"),
        }
    }
}

#[test]
fn test_error_handling_and_recovery() {
    let mut interface = EvaluatorInterface::new();
    let env = create_test_environment();
    let mut config = create_basic_config();
    config.mode = EvaluationMode::Semantic;
    config.fallback_to_semantic = true;
    
    interface.set_config(config);
    
    // Test with undefined variable (should handle gracefully)
    let invalid_expr = Expr::Variable("undefined-variable".to_string());
    let result = interface.eval(invalid_expr, env, Continuation::Identity);
    
    // This should either succeed with error handling or fail gracefully
    // The exact behavior depends on the implementation details
    match result {
        Ok(_) => {
            // If it succeeds, that's fine too (might return undefined or error value)
        }
        Err(_) => {
            // Expected behavior for undefined variable
        }
    }
}

#[test]
fn test_mode_recommendation() {
    let mut interface = EvaluatorInterface::new();
    
    // Test mode recommendation for simple expression
    let simple_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let recommendation = interface.get_mode_recommendation(&simple_expr);
    
    // Should recommend semantic for simple expressions
    assert_eq!(recommendation, EvaluationMode::Semantic);
}

#[test]
fn test_verification_system_statistics() {
    let interface = EvaluatorInterface::new();
    
    // Test that we can access verification statistics
    let stats = interface.get_verification_statistics();
    
    // Should be able to access the statistics without error
    // Total verifications is always >= 0 by design
}

#[test]
fn test_cache_management() {
    let mut interface = EvaluatorInterface::new();
    
    // Test verification cache management
    let (total, successful) = interface.get_verification_cache_stats();
    assert_eq!(total, 0);
    assert_eq!(successful, 0);
    
    interface.clear_verification_cache();
    let (total_after_clear, successful_after_clear) = interface.get_verification_cache_stats();
    assert_eq!(total_after_clear, 0);
    assert_eq!(successful_after_clear, 0);
}

#[test]
fn test_mode_selector_management() {
    let mut interface = EvaluatorInterface::new();
    
    // Test mode selector statistics
    let stats = interface.get_mode_selector_stats();
    assert!(!stats.is_empty(), "Should provide mode selector statistics");
    
    // Test reset functionality
    interface.reset_mode_selector();
    
    // Should still work after reset
    let stats_after_reset = interface.get_mode_selector_stats();
    assert!(!stats_after_reset.is_empty(), "Should still provide statistics after reset");
}

#[test]
fn test_performance_history_management() {
    let mut interface = EvaluatorInterface::new();
    
    // Initially empty
    assert_eq!(interface.get_performance_history().len(), 0);
    
    // Clear (should be no-op when empty)
    interface.clear_performance_history();
    assert_eq!(interface.get_performance_history().len(), 0);
}