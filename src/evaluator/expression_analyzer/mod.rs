//! Expression Analyzer Module
//!
//! このモジュールは式解析システムの包括的な実装を提供します。
//! コンパイル時最適化、定数畳み込み、型推論、特殊形式解析を含みます。
//!
//! ## モジュール構成
//!
//! - `core_types`: 基本型定義（AnalysisResult, `TypeHint`, `ComplexityLevel等`）
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
#[must_use] pub fn create_expression_analyzer() -> ExpressionAnalyzer {
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
