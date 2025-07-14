//! Expression Analyzer Module
//!
//! このモジュールは式解析システムの包括的な実装を提供します。
//! コンパイル時最適化、定数畳み込み、型推論、特殊形式解析を含みます。
//!
//! ## モジュール構成
//!
//! - `core_types`: 基本型定義（AnalysisResult, TypeHint, ComplexityLevel等）
//! - `analyzer`: メイン式解析器（解析ロジック、キャッシュ管理）
//! - `constant_folding`: 定数畳み込み最適化（算術演算、比較演算等）
//! - `special_forms`: 特殊形式解析（if, and, or, begin, lambda, define等）

pub mod core_types;
pub mod analyzer;
pub mod constant_folding;
pub mod special_forms;

// Re-export main types for backward compatibility
pub use core_types::{
    AnalysisResult, TypeHint, EvaluationComplexity, OptimizationHint, OptimizationStats,
};

pub use analyzer::ExpressionAnalyzer;

pub use constant_folding::ConstantFolder;

pub use special_forms::SpecialFormsAnalyzer;

/// Create a new expression analyzer with default configuration
pub fn create_expression_analyzer() -> ExpressionAnalyzer {
    ExpressionAnalyzer::new()
}

/// Quick analysis for simple expressions
pub fn quick_analyze(
    expr: &crate::ast::Expr,
    env: Option<&crate::environment::Environment>,
) -> crate::error::Result<AnalysisResult> {
    let mut analyzer = ExpressionAnalyzer::new();
    analyzer.analyze(expr, env)
}

/// Analyze with type hints
pub fn analyze_with_hints(
    expr: &crate::ast::Expr,
    env: Option<&crate::environment::Environment>,
    type_hints: std::collections::HashMap<String, TypeHint>,
) -> crate::error::Result<AnalysisResult> {
    let mut analyzer = ExpressionAnalyzer::new();
    
    // Add type hints
    for (name, hint) in type_hints {
        analyzer.add_type_hint(name, hint);
    }
    
    analyzer.analyze(expr, env)
}

/// Analyze with known constants
pub fn analyze_with_constants(
    expr: &crate::ast::Expr,
    env: Option<&crate::environment::Environment>,
    constants: std::collections::HashMap<String, crate::value::Value>,
) -> crate::error::Result<AnalysisResult> {
    let mut analyzer = ExpressionAnalyzer::new();
    
    // Add constants
    for (name, value) in constants {
        analyzer.add_constant(name, value);
    }
    
    analyzer.analyze(expr, env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = create_expression_analyzer();
        let stats = analyzer.optimization_stats();
        assert_eq!(stats.total_analyzed, 0);
    }

    #[test]
    fn test_literal_analysis() {
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let result = quick_analyze(&expr, None).unwrap();
        
        assert!(result.is_constant);
        assert_eq!(result.type_hint, TypeHint::Number);
        assert_eq!(result.complexity, EvaluationComplexity::Constant);
        assert!(!result.has_side_effects);
    }

    #[test]
    fn test_variable_analysis() {
        let expr = Expr::Variable("x".to_string());
        let result = quick_analyze(&expr, None).unwrap();
        
        assert!(!result.is_constant);
        assert_eq!(result.type_hint, TypeHint::Unknown);
        assert_eq!(result.complexity, EvaluationComplexity::Variable);
        assert!(!result.has_side_effects);
        assert_eq!(result.dependencies, vec!["x".to_string()]);
    }

    #[test]
    fn test_arithmetic_analysis() {
        // (+ 1 2)
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]);
        
        let result = quick_analyze(&expr, None).unwrap();
        
        assert!(result.is_constant);
        assert_eq!(result.type_hint, TypeHint::Number);
        assert!(!result.optimizations.is_empty());
    }

    #[test]
    fn test_type_hints() {
        let expr = Expr::Variable("x".to_string());
        let mut type_hints = std::collections::HashMap::new();
        type_hints.insert("x".to_string(), TypeHint::Number);
        
        let result = analyze_with_hints(&expr, None, type_hints).unwrap();
        assert_eq!(result.type_hint, TypeHint::Number);
    }

    #[test]
    fn test_constants() {
        let expr = Expr::Variable("PI".to_string());
        let mut constants = std::collections::HashMap::new();
        constants.insert("PI".to_string(), crate::value::Value::Number(SchemeNumber::Real(3.14159)));
        
        let result = analyze_with_constants(&expr, None, constants).unwrap();
        assert!(result.is_constant);
        assert!(!result.optimizations.is_empty());
    }

    #[test]
    fn test_if_analysis() {
        // (if #t 1 2)
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Literal(Literal::Boolean(true)),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]);
        
        let result = quick_analyze(&expr, None).unwrap();
        
        assert!(result.is_constant);
        assert_eq!(result.type_hint, TypeHint::Number);
        assert!(!result.optimizations.is_empty()); // Should detect dead code
    }

    #[test]
    fn test_vector_analysis() {
        // #(1 2 3)
        let expr = Expr::Vector(vec![
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
        ]);
        
        let result = quick_analyze(&expr, None).unwrap();
        
        assert!(result.is_constant);
        assert_eq!(result.type_hint, TypeHint::Vector);
        assert_eq!(result.complexity, EvaluationComplexity::Constant);
    }

    #[test]
    fn test_lambda_analysis() {
        // (lambda (x) x)
        let expr = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![Expr::Variable("x".to_string())]),
            Expr::Variable("x".to_string()),
        ]);
        
        let result = quick_analyze(&expr, None).unwrap();
        
        assert!(result.is_constant); // Lambda expressions are constant
        assert_eq!(result.type_hint, TypeHint::Procedure);
        assert!(!result.has_side_effects);
    }

    #[test]
    fn test_optimization_stats() {
        let mut analyzer = ExpressionAnalyzer::new();
        
        // Analyze some expressions
        let expr1 = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let expr2 = Expr::Variable("x".to_string());
        
        analyzer.analyze(&expr1, None).unwrap();
        analyzer.analyze(&expr2, None).unwrap();
        
        let stats = analyzer.optimization_stats();
        assert_eq!(stats.total_analyzed, 2);
        assert!(stats.constants_found > 0);
    }

    #[test]
    fn test_cache_functionality() {
        let mut analyzer = ExpressionAnalyzer::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        
        // First analysis
        let result1 = analyzer.analyze(&expr, None).unwrap();
        
        // Second analysis (should use cache)
        let result2 = analyzer.analyze(&expr, None).unwrap();
        
        assert_eq!(result1.is_constant, result2.is_constant);
        assert_eq!(result1.type_hint, result2.type_hint);
    }
}