//! Unit tests for expression analyzer (Phase 5-Step1)
//!
//! Tests the static analysis capabilities including constant folding,
//! type hints, complexity estimation, and optimization suggestions.

use lambdust::ast::{Expr, Literal};
use lambdust::evaluator::{EvaluationComplexity, ExpressionAnalyzer, OptimizationHint, TypeHint};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_literal_analysis() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test number literal
    let number_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let result = analyzer.analyze(&number_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::Number(SchemeNumber::Integer(42)))
    );
    assert_eq!(result.type_hint, TypeHint::Number);
    assert_eq!(result.complexity, EvaluationComplexity::Constant);
    assert!(!result.has_side_effects);
    assert!(result.dependencies.is_empty());
    assert_eq!(result.optimizations.len(), 1);
    assert!(matches!(
        result.optimizations[0],
        OptimizationHint::ConstantFold(_)
    ));

    // Test boolean literal
    let bool_expr = Expr::Literal(Literal::Boolean(true));
    let result = analyzer.analyze(&bool_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(result.constant_value, Some(Value::Boolean(true)));
    assert_eq!(result.type_hint, TypeHint::Boolean);
    assert_eq!(result.complexity, EvaluationComplexity::Constant);

    // Test string literal
    let string_expr = Expr::Literal(Literal::String("hello".to_string()));
    let result = analyzer.analyze(&string_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::String("hello".to_string()))
    );
    assert_eq!(result.type_hint, TypeHint::String);
    assert_eq!(result.complexity, EvaluationComplexity::Constant);
}

#[test]
fn test_variable_analysis() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test unknown variable
    let var_expr = Expr::Variable("x".to_string());
    let result = analyzer.analyze(&var_expr, None).unwrap();

    assert!(!result.is_constant);
    assert_eq!(result.constant_value, None);
    assert_eq!(result.type_hint, TypeHint::Unknown);
    assert_eq!(result.complexity, EvaluationComplexity::Variable);
    assert!(!result.has_side_effects);
    assert_eq!(result.dependencies, vec!["x".to_string()]);

    // Test known constant variable
    analyzer.add_constant(
        "pi".to_string(),
        Value::Number(SchemeNumber::Real(std::f64::consts::PI)),
    );
    let pi_expr = Expr::Variable("pi".to_string());
    let result = analyzer.analyze(&pi_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::Number(SchemeNumber::Real(std::f64::consts::PI)))
    );
    assert_eq!(result.type_hint, TypeHint::Number); // Updated after adding type hint
    assert_eq!(result.complexity, EvaluationComplexity::Constant);
    assert_eq!(result.optimizations.len(), 1);
    assert!(matches!(
        result.optimizations[0],
        OptimizationHint::InlineVariable(_, _)
    ));
}

#[test]
fn test_quote_analysis() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test quoted symbol
    let quote_expr = Expr::Quote(Box::new(Expr::Variable("symbol".to_string())));
    let result = analyzer.analyze(&quote_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::Symbol("symbol".to_string()))
    );
    assert_eq!(result.type_hint, TypeHint::Symbol);
    assert_eq!(result.complexity, EvaluationComplexity::Constant);
    assert!(!result.has_side_effects);
    assert!(result.dependencies.is_empty());

    // Test quoted list
    let list_expr = Expr::Quote(Box::new(Expr::List(vec![
        Expr::Variable("a".to_string()),
        Expr::Variable("b".to_string()),
    ])));
    let result = analyzer.analyze(&list_expr, None).unwrap();

    assert!(result.is_constant);
    assert!(result.constant_value.is_some());
    assert_eq!(result.type_hint, TypeHint::List);
    assert_eq!(result.complexity, EvaluationComplexity::Constant);
}

#[test]
fn test_vector_analysis() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test constant vector
    let vector_expr = Expr::Vector(vec![
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
    ]);
    let result = analyzer.analyze(&vector_expr, None).unwrap();

    assert!(result.is_constant);
    assert!(result.constant_value.is_some());
    if let Some(Value::Vector(vec)) = &result.constant_value {
        assert_eq!(vec.len(), 3);
    }
    assert_eq!(result.type_hint, TypeHint::Vector);
    assert_eq!(result.complexity, EvaluationComplexity::Constant);
    assert!(!result.has_side_effects);
    assert!(result.dependencies.is_empty());

    // Test non-constant vector
    let non_const_vector = Expr::Vector(vec![
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Variable("x".to_string()),
    ]);
    let result = analyzer.analyze(&non_const_vector, None).unwrap();

    assert!(!result.is_constant);
    assert_eq!(result.constant_value, None);
    assert_eq!(result.type_hint, TypeHint::Vector);
    assert_eq!(result.complexity, EvaluationComplexity::Variable);
    assert_eq!(result.dependencies, vec!["x".to_string()]);
}

#[test]
fn test_if_analysis() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test if with constant condition (true)
    let if_true_expr = Expr::List(vec![
        Expr::Variable("if".to_string()),
        Expr::Literal(Literal::Boolean(true)),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);
    let result = analyzer.analyze(&if_true_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::Number(SchemeNumber::Integer(1)))
    );
    assert_eq!(result.type_hint, TypeHint::Number);
    assert_eq!(result.complexity, EvaluationComplexity::Constant);
    assert!(result
        .optimizations
        .iter()
        .any(|opt| matches!(opt, OptimizationHint::ConstantFold(_))));

    // Test if with constant condition (false)
    let if_false_expr = Expr::List(vec![
        Expr::Variable("if".to_string()),
        Expr::Literal(Literal::Boolean(false)),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);
    let result = analyzer.analyze(&if_false_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::Number(SchemeNumber::Integer(2)))
    );
    assert_eq!(result.type_hint, TypeHint::Number);
    assert!(result
        .optimizations
        .iter()
        .any(|opt| matches!(opt, OptimizationHint::ConstantFold(_))));

    // Test if with variable condition
    let if_var_expr = Expr::List(vec![
        Expr::Variable("if".to_string()),
        Expr::Variable("condition".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);
    let result = analyzer.analyze(&if_var_expr, None).unwrap();

    assert!(!result.is_constant);
    assert_eq!(result.constant_value, None);
    assert_eq!(result.dependencies, vec!["condition".to_string()]);
}

#[test]
fn test_and_analysis() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test empty and (should be true)
    let empty_and = Expr::List(vec![Expr::Variable("and".to_string())]);
    let result = analyzer.analyze(&empty_and, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(result.constant_value, Some(Value::Boolean(true)));
    assert_eq!(result.type_hint, TypeHint::Boolean);
    assert_eq!(result.complexity, EvaluationComplexity::Constant);

    // Test and with false constant (short circuit)
    let and_false = Expr::List(vec![
        Expr::Variable("and".to_string()),
        Expr::Literal(Literal::Boolean(true)),
        Expr::Literal(Literal::Boolean(false)),
        Expr::Variable("never-evaluated".to_string()),
    ]);
    let result = analyzer.analyze(&and_false, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(result.constant_value, Some(Value::Boolean(false)));
    assert_eq!(result.type_hint, TypeHint::Boolean);
    assert!(result
        .optimizations
        .iter()
        .any(|opt| matches!(opt, OptimizationHint::ConstantFold(_))));

    // Test and with all true constants
    let and_true = Expr::List(vec![
        Expr::Variable("and".to_string()),
        Expr::Literal(Literal::Boolean(true)),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
    ]);
    let result = analyzer.analyze(&and_true, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::Number(SchemeNumber::Integer(42)))
    );
}

#[test]
fn test_or_analysis() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test empty or (should be false)
    let empty_or = Expr::List(vec![Expr::Variable("or".to_string())]);
    let result = analyzer.analyze(&empty_or, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(result.constant_value, Some(Value::Boolean(false)));
    assert_eq!(result.type_hint, TypeHint::Boolean);
    assert_eq!(result.complexity, EvaluationComplexity::Constant);

    // Test or with true constant (short circuit)
    let or_true = Expr::List(vec![
        Expr::Variable("or".to_string()),
        Expr::Literal(Literal::Boolean(false)),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        Expr::Variable("never-evaluated".to_string()),
    ]);
    let result = analyzer.analyze(&or_true, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::Number(SchemeNumber::Integer(42)))
    );
    assert!(result
        .optimizations
        .iter()
        .any(|opt| matches!(opt, OptimizationHint::ConstantFold(_))));
}

#[test]
fn test_begin_analysis() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test empty begin
    let empty_begin = Expr::List(vec![Expr::Variable("begin".to_string())]);
    let result = analyzer.analyze(&empty_begin, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(result.constant_value, Some(Value::Undefined));
    assert_eq!(result.complexity, EvaluationComplexity::Constant);

    // Test begin with constants (no side effects)
    let const_begin = Expr::List(vec![
        Expr::Variable("begin".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        Expr::Literal(Literal::String("result".to_string())),
    ]);
    let result = analyzer.analyze(&const_begin, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::String("result".to_string()))
    );
    assert_eq!(result.type_hint, TypeHint::String);
}

#[test]
fn test_function_application_analysis() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test pure function with constant arguments
    let add_expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);
    let result = analyzer.analyze(&add_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::Number(SchemeNumber::Real(3.0)))
    );
    assert_eq!(result.type_hint, TypeHint::Number);
    assert_eq!(result.complexity, EvaluationComplexity::Simple);
    assert!(!result.has_side_effects);

    // Test pure function with variables
    let add_var_expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Variable("x".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);
    let result = analyzer.analyze(&add_var_expr, None).unwrap();

    assert!(!result.is_constant);
    assert_eq!(result.constant_value, None);
    assert_eq!(result.type_hint, TypeHint::Number);
    assert_eq!(result.complexity, EvaluationComplexity::Simple);
    assert!(!result.has_side_effects);
    assert_eq!(result.dependencies, vec!["x".to_string()]);

    // Test comparison function
    let eq_expr = Expr::List(vec![
        Expr::Variable("=".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ]);
    let result = analyzer.analyze(&eq_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(result.constant_value, Some(Value::Boolean(true)));
    assert_eq!(result.type_hint, TypeHint::Boolean);
}

#[test]
fn test_constant_folding_arithmetic() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test subtraction (binary)
    let sub_expr = Expr::List(vec![
        Expr::Variable("-".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
    ]);
    let result = analyzer.analyze(&sub_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::Number(SchemeNumber::Real(7.0)))
    );

    // Test multiplication
    let mul_expr = Expr::List(vec![
        Expr::Variable("*".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(4))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ]);
    let result = analyzer.analyze(&mul_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::Number(SchemeNumber::Real(20.0)))
    );

    // Test division
    let div_expr = Expr::List(vec![
        Expr::Variable("/".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(15))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
    ]);
    let result = analyzer.analyze(&div_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(
        result.constant_value,
        Some(Value::Number(SchemeNumber::Real(5.0)))
    );
}

#[test]
fn test_constant_folding_comparison() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test less than
    let lt_expr = Expr::List(vec![
        Expr::Variable("<".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ]);
    let result = analyzer.analyze(&lt_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(result.constant_value, Some(Value::Boolean(true)));

    // Test greater than
    let gt_expr = Expr::List(vec![
        Expr::Variable(">".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(8))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ]);
    let result = analyzer.analyze(&gt_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(result.constant_value, Some(Value::Boolean(true)));

    // Test equality (false case)
    let eq_false_expr = Expr::List(vec![
        Expr::Variable("=".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
    ]);
    let result = analyzer.analyze(&eq_false_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(result.constant_value, Some(Value::Boolean(false)));
}

#[test]
fn test_constant_folding_logical() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test not with true
    let not_true_expr = Expr::List(vec![
        Expr::Variable("not".to_string()),
        Expr::Literal(Literal::Boolean(true)),
    ]);
    let result = analyzer.analyze(&not_true_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(result.constant_value, Some(Value::Boolean(false)));

    // Test not with false
    let not_false_expr = Expr::List(vec![
        Expr::Variable("not".to_string()),
        Expr::Literal(Literal::Boolean(false)),
    ]);
    let result = analyzer.analyze(&not_false_expr, None).unwrap();

    assert!(result.is_constant);
    assert_eq!(result.constant_value, Some(Value::Boolean(true)));
}

#[test]
fn test_lambda_analysis() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test lambda expression
    let lambda_expr = Expr::List(vec![
        Expr::Variable("lambda".to_string()),
        Expr::List(vec![Expr::Variable("x".to_string())]),
        Expr::Variable("x".to_string()),
    ]);
    let result = analyzer.analyze(&lambda_expr, None).unwrap();

    assert!(!result.is_constant);
    assert_eq!(result.constant_value, None);
    assert_eq!(result.type_hint, TypeHint::Procedure);
    assert_eq!(result.complexity, EvaluationComplexity::Simple);
    assert!(!result.has_side_effects);
    assert!(result.dependencies.is_empty());
}

#[test]
fn test_define_analysis() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Test define expression
    let define_expr = Expr::List(vec![
        Expr::Variable("define".to_string()),
        Expr::Variable("x".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
    ]);
    let result = analyzer.analyze(&define_expr, None).unwrap();

    assert!(!result.is_constant);
    assert_eq!(result.constant_value, None);
    assert_eq!(result.type_hint, TypeHint::Unknown);
    assert_eq!(result.complexity, EvaluationComplexity::Constant);
    assert!(result.has_side_effects); // Define has side effects
    assert!(result.dependencies.is_empty());
}

#[test]
fn test_type_hints() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Add type hints
    analyzer.add_type_hint("number-var".to_string(), TypeHint::Number);
    analyzer.add_type_hint("string-var".to_string(), TypeHint::String);

    // Test variable with type hint
    let var_expr = Expr::Variable("number-var".to_string());
    let result = analyzer.analyze(&var_expr, None).unwrap();

    assert_eq!(result.type_hint, TypeHint::Number);
    assert_eq!(result.complexity, EvaluationComplexity::Variable);
}

#[test]
fn test_optimization_statistics() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Analyze several expressions to generate optimizations
    let _result1 = analyzer.analyze(
        &Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        None,
    );
    let _result2 = analyzer.analyze(
        &Expr::Quote(Box::new(Expr::Variable("symbol".to_string()))),
        None,
    );
    let _result3 = analyzer.analyze(
        &Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Literal(Literal::Boolean(true)),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        ]),
        None,
    );

    let stats = analyzer.optimization_stats();

    assert!(stats.total_analyses > 0);
    assert!(stats.constant_folds > 0);
    assert!(stats.optimization_ratio() > 0.0);
}

#[test]
fn test_cache_functionality() {
    let mut analyzer = ExpressionAnalyzer::new();

    let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));

    // First analysis should cache the result
    let _result1 = analyzer.analyze(&expr, None).unwrap();

    // Second analysis should use cached result (test by checking it doesn't fail)
    let result2 = analyzer.analyze(&expr, None).unwrap();

    assert!(result2.is_constant);
    assert_eq!(
        result2.constant_value,
        Some(Value::Number(SchemeNumber::Integer(42)))
    );

    // Clear cache and verify it still works
    analyzer.clear_cache();
    let result3 = analyzer.analyze(&expr, None).unwrap();

    assert!(result3.is_constant);
    assert_eq!(
        result3.constant_value,
        Some(Value::Number(SchemeNumber::Integer(42)))
    );
}

#[test]
fn test_complexity_estimation() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Constant complexity
    let const_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let result = analyzer.analyze(&const_expr, None).unwrap();
    assert_eq!(result.complexity, EvaluationComplexity::Constant);

    // Variable complexity
    let var_expr = Expr::Variable("x".to_string());
    let result = analyzer.analyze(&var_expr, None).unwrap();
    assert_eq!(result.complexity, EvaluationComplexity::Variable);

    // Simple complexity (function application)
    let simple_expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);
    let result = analyzer.analyze(&simple_expr, None).unwrap();
    assert_eq!(result.complexity, EvaluationComplexity::Simple);
}

#[test]
fn test_side_effects_detection() {
    let mut analyzer = ExpressionAnalyzer::new();

    // No side effects (pure expression)
    let pure_expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);
    let result = analyzer.analyze(&pure_expr, None).unwrap();
    assert!(!result.has_side_effects);

    // Has side effects (define)
    let side_effect_expr = Expr::List(vec![
        Expr::Variable("define".to_string()),
        Expr::Variable("x".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
    ]);
    let result = analyzer.analyze(&side_effect_expr, None).unwrap();
    assert!(result.has_side_effects);
}

#[test]
fn test_dependency_tracking() {
    let mut analyzer = ExpressionAnalyzer::new();

    // Single dependency
    let single_dep = Expr::Variable("x".to_string());
    let result = analyzer.analyze(&single_dep, None).unwrap();
    assert_eq!(result.dependencies, vec!["x".to_string()]);

    // Multiple dependencies
    let multi_dep = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Variable("x".to_string()),
        Expr::Variable("y".to_string()),
    ]);
    let result = analyzer.analyze(&multi_dep, None).unwrap();
    assert_eq!(result.dependencies, vec!["x".to_string(), "y".to_string()]);

    // No dependencies (constants)
    let no_dep = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let result = analyzer.analyze(&no_dep, None).unwrap();
    assert!(result.dependencies.is_empty());
}
