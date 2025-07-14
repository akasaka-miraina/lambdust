//! Hygienic Transformer Module
//!
//! このモジュールは衛生的マクロトランスフォーマーの包括的な実装を提供します。
//! パターンマッチング、テンプレート展開、最適化、SRFI 46対応を含みます。
//!
//! ## モジュール構成
//!
//! - `core_types`: 基本型定義（OptimizationLevel, TransformerMetrics, PatternBindings）
//! - `transformer`: メインのHygienicSyntaxRulesTransformer実装
//! - `pattern_matching`: パターンマッチング関連のロジック
//! - `template_expansion`: テンプレート展開関連のロジック

pub mod core_types;
pub mod transformer;
pub mod pattern_matching;
pub mod template_expansion;

// Re-export main types for backward compatibility
pub use core_types::{
    OptimizationLevel, PatternBindings, TransformerMetrics,
};

pub use transformer::HygienicSyntaxRulesTransformer;

pub use pattern_matching::PatternMatcher;

pub use template_expansion::TemplateExpander;

/// Create a new hygienic transformer with default configuration
pub fn create_hygienic_transformer(
    literals: Vec<String>,
    rules: Vec<crate::macros::SyntaxRule>,
    definition_environment: std::rc::Rc<crate::macros::hygiene::environment::HygienicEnvironment>,
    macro_name: String,
) -> HygienicSyntaxRulesTransformer {
    HygienicSyntaxRulesTransformer::new(literals, rules, definition_environment, macro_name)
}

/// Create a production-optimized hygienic transformer
pub fn create_optimized_transformer(
    literals: Vec<String>,
    rules: Vec<crate::macros::SyntaxRule>,
    definition_environment: std::rc::Rc<crate::macros::hygiene::environment::HygienicEnvironment>,
    macro_name: String,
) -> HygienicSyntaxRulesTransformer {
    HygienicSyntaxRulesTransformer::optimized(literals, rules, definition_environment, macro_name)
}

/// Create a scope-aware hygienic transformer
pub fn create_scope_aware_transformer(
    literals: Vec<String>,
    rules: Vec<crate::macros::SyntaxRule>,
    definition_environment: std::rc::Rc<crate::macros::hygiene::environment::HygienicEnvironment>,
    macro_name: String,
) -> HygienicSyntaxRulesTransformer {
    HygienicSyntaxRulesTransformer::scope_aware(literals, rules, definition_environment, macro_name)
}

/// Create a SRFI 46 enabled transformer
pub fn create_srfi46_transformer(
    literals: Vec<String>,
    rules: Vec<crate::macros::SyntaxRule>,
    definition_environment: std::rc::Rc<crate::macros::hygiene::environment::HygienicEnvironment>,
    macro_name: String,
) -> HygienicSyntaxRulesTransformer {
    HygienicSyntaxRulesTransformer::with_srfi46(literals, rules, definition_environment, macro_name)
}

/// Create a legacy transformer (SRFI 46 disabled)
pub fn create_legacy_transformer(
    literals: Vec<String>,
    rules: Vec<crate::macros::SyntaxRule>,
    definition_environment: std::rc::Rc<crate::macros::hygiene::environment::HygienicEnvironment>,
    macro_name: String,
) -> HygienicSyntaxRulesTransformer {
    HygienicSyntaxRulesTransformer::legacy(literals, rules, definition_environment, macro_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::hygiene::environment::HygienicEnvironment;
    use std::rc::Rc;

    fn create_test_environment() -> Rc<HygienicEnvironment> {
        Rc::new(HygienicEnvironment::new())
    }

    #[test]
    fn test_hygienic_transformer_creation() {
        let env = create_test_environment();
        let transformer = create_hygienic_transformer(
            vec!["if".to_string()],
            vec![],
            env,
            "test-macro".to_string(),
        );
        
        assert_eq!(transformer.macro_name, "test-macro");
        assert!(transformer.is_srfi46_enabled());
        assert_eq!(transformer.optimization_level(), &OptimizationLevel::Balanced);
    }

    #[test]
    fn test_optimized_transformer_creation() {
        let env = create_test_environment();
        let transformer = create_optimized_transformer(
            vec![],
            vec![],
            env,
            "optimized-macro".to_string(),
        );
        
        assert_eq!(transformer.optimization_level(), &OptimizationLevel::Production);
        assert!(transformer.pattern_cache.capacity() >= 1000);
    }

    #[test]
    fn test_scope_aware_transformer_creation() {
        let env = create_test_environment();
        let transformer = create_scope_aware_transformer(
            vec![],
            vec![],
            env,
            "scope-macro".to_string(),
        );
        
        match transformer.optimization_level() {
            OptimizationLevel::Custom { enable_scope_analysis: true, .. } => {},
            _ => panic!("Expected custom optimization with scope analysis enabled"),
        }
    }

    #[test]
    fn test_srfi46_transformer_creation() {
        let env = create_test_environment();
        let transformer = create_srfi46_transformer(
            vec![],
            vec![],
            env,
            "srfi46-macro".to_string(),
        );
        
        assert!(transformer.is_srfi46_enabled());
        // SRFI 46 processors should have higher depth
        assert!(transformer.ellipsis_processor.max_depth() >= 20);
    }

    #[test]
    fn test_legacy_transformer_creation() {
        let env = create_test_environment();
        let transformer = create_legacy_transformer(
            vec![],
            vec![],
            env,
            "legacy-macro".to_string(),
        );
        
        assert!(!transformer.is_srfi46_enabled());
    }

    #[test]
    fn test_transformer_metrics() {
        let env = create_test_environment();
        let transformer = create_hygienic_transformer(
            vec![],
            vec![],
            env,
            "metrics-macro".to_string(),
        );
        
        let metrics = transformer.metrics();
        assert_eq!(metrics.transformations_count, 0);
        assert_eq!(metrics.successful_transformations, 0);
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[test]
    fn test_optimization_level_changes() {
        let env = create_test_environment();
        let mut transformer = create_hygienic_transformer(
            vec![],
            vec![],
            env,
            "opt-test-macro".to_string(),
        );
        
        // Start with balanced
        assert_eq!(transformer.optimization_level(), &OptimizationLevel::Balanced);
        
        // Change to production
        transformer.set_optimization_level(OptimizationLevel::Production);
        assert_eq!(transformer.optimization_level(), &OptimizationLevel::Production);
        
        // Change to development
        transformer.set_optimization_level(OptimizationLevel::Development);
        assert_eq!(transformer.optimization_level(), &OptimizationLevel::Development);
    }

    #[test]
    fn test_cache_operations() {
        let env = create_test_environment();
        let mut transformer = create_optimized_transformer(
            vec![],
            vec![],
            env,
            "cache-test-macro".to_string(),
        );
        
        // Fill cache with dummy data
        for i in 0..100 {
            transformer.pattern_cache.insert(format!("key{}", i), i % 2 == 0);
        }
        
        assert!(!transformer.pattern_cache.is_empty());
        
        // Test cache optimization
        transformer.optimize_caches();
        
        // Cache should still exist (not too large)
        assert!(!transformer.pattern_cache.is_empty());
        
        // Test reset
        transformer.reset_metrics();
        assert_eq!(transformer.metrics().transformations_count, 0);
    }

    #[test]
    fn test_performance_analysis() {
        let env = create_test_environment();
        let transformer = create_hygienic_transformer(
            vec![],
            vec![],
            env,
            "perf-macro".to_string(),
        );
        
        let analysis = transformer.performance_analysis();
        assert!(analysis.contains("Hygienic Transformer Performance Analysis"));
        assert!(analysis.contains("perf-macro"));
        assert!(analysis.contains("SRFI 46 Support: true"));
    }
}