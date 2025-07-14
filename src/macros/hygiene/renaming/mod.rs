//! Symbol Renaming System for Hygienic Macros
//!
//! このモジュールは衛生的マクロシステムでのシンボルリネーミングの
//! 包括的な実装を提供します。マクロ展開中のシンボル衝突を防ぎながら
//! 字句スコープ規則を維持する異なる戦略を提供します。
//!
//! ## モジュール構成
//!
//! - `core_types`: 基本型定義（RenamingStrategy, RenamingPattern等）
//! - `pattern_matching`: パターンマッチングエンジンと述語関数
//! - `strategies`: 標準的なリネーミング戦略とユーティリティ
//! - `renaming_engine`: メインシンボルリネーミングエンジン

pub mod core_types;
pub mod pattern_matching;
pub mod strategies;
pub mod renaming_engine;

// Re-export main types for backward compatibility
pub use core_types::{
    RenamingStrategy, CustomRenamingRule, RenamingPattern, PatternMatcher,
    RenamingAction, DefaultAction, ScopeConstraint, TypeConstraint,
    PredicateFunction, BuiltInPredicate, RenamingStats, ScopeTracker,
    PerformanceSummary, RenamingRule,
};

pub use pattern_matching::{
    PatternMatchingEngine, PatternMatchingStats, CacheInfo, PatternBuilder,
};

pub use strategies::{
    StandardRenamingStrategies, RenamingStrategyFactory, StrategyConfig,
    StrategyType, RenamingUtils,
};

pub use renaming_engine::SymbolRenamer;

use super::symbol::HygienicSymbol;
use super::environment::HygienicEnvironment;
use super::context::ExpansionContext;
use crate::ast::Expr;
use crate::error::Result;

/// Create a new symbol renamer with default configuration
pub fn create_symbol_renamer() -> SymbolRenamer {
    SymbolRenamer::new(RenamingStrategy::RenameConflicts)
}

/// Create a conservative symbol renamer (minimal renaming)
pub fn create_conservative_renamer() -> SymbolRenamer {
    SymbolRenamer::new(StandardRenamingStrategies::conservative())
}

/// Create an aggressive symbol renamer (rename all macro symbols)
pub fn create_aggressive_renamer() -> SymbolRenamer {
    SymbolRenamer::new(StandardRenamingStrategies::aggressive())
}

/// Create an optimized symbol renamer for high-performance scenarios
pub fn create_optimized_renamer() -> SymbolRenamer {
    SymbolRenamer::optimized()
}

/// Create an intelligent symbol renamer with machine learning-inspired heuristics
pub fn create_intelligent_renamer() -> SymbolRenamer {
    SymbolRenamer::intelligent()
}

/// Create a scope-aware symbol renamer for complex macro systems
pub fn create_scope_aware_renamer() -> SymbolRenamer {
    SymbolRenamer::scope_aware()
}

/// Create a custom symbol renamer with user-defined strategy
pub fn create_custom_renamer(strategy: RenamingStrategy) -> SymbolRenamer {
    SymbolRenamer::new(strategy)
}

/// Create a symbol renamer from configuration
pub fn create_renamer_from_config(config: &StrategyConfig) -> SymbolRenamer {
    let strategy = RenamingStrategyFactory::create_strategy(config);
    SymbolRenamer::new(strategy)
}

/// Rename symbols in expression using default conservative strategy
pub fn rename_symbols_conservative(
    expr: &Expr,
    context: &mut ExpansionContext,
    environment: &HygienicEnvironment,
) -> Result<Expr> {
    let mut renamer = create_conservative_renamer();
    renamer.rename_symbols(expr, context, environment)
}

/// Rename symbols in expression using conflict-aware strategy
pub fn rename_symbols_conflict_aware(
    expr: &Expr,
    context: &mut ExpansionContext,
    environment: &HygienicEnvironment,
) -> Result<Expr> {
    let mut renamer = create_symbol_renamer();
    renamer.rename_symbols(expr, context, environment)
}

/// Rename symbols in expression using aggressive strategy
pub fn rename_symbols_aggressive(
    expr: &Expr,
    context: &mut ExpansionContext,
    environment: &HygienicEnvironment,
) -> Result<Expr> {
    let mut renamer = create_aggressive_renamer();
    renamer.rename_symbols(expr, context, environment)
}

/// Rename symbols in expression using intelligent strategy
pub fn rename_symbols_intelligent(
    expr: &Expr,
    context: &mut ExpansionContext,
    environment: &HygienicEnvironment,
) -> Result<Expr> {
    let mut renamer = create_intelligent_renamer();
    renamer.rename_symbols(expr, context, environment)
}

/// Rename symbols in expression using custom strategy
pub fn rename_symbols_custom(
    expr: &Expr,
    context: &mut ExpansionContext,
    environment: &HygienicEnvironment,
    strategy: RenamingStrategy,
) -> Result<Expr> {
    let mut renamer = SymbolRenamer::new(strategy);
    renamer.rename_symbols(expr, context, environment)
}

/// Check if a symbol would cause conflicts without actually renaming
pub fn check_symbol_conflicts(
    name: &str,
    context: &ExpansionContext,
    environment: &HygienicEnvironment,
) -> bool {
    let renamer = create_symbol_renamer();
    renamer.would_cause_conflict(name, context, environment)
}

/// Generate a renamed symbol using default strategy
pub fn generate_renamed_symbol(
    name: &str,
    context: &mut ExpansionContext,
) -> HygienicSymbol {
    context.generate_template_symbol(name)
}

/// Utility function to check if a symbol name suggests it's temporary
pub fn is_temporary_symbol(name: &str) -> bool {
    RenamingUtils::is_temporary_variable(name)
}

/// Utility function to check if a symbol name suggests it's compiler/macro generated
pub fn is_generated_symbol(name: &str) -> bool {
    RenamingUtils::is_generated_symbol(name)
}

/// Utility function to check if a symbol name follows Lisp naming conventions
pub fn is_lisp_case_symbol(name: &str) -> bool {
    RenamingUtils::is_lisp_case(name)
}

/// Utility function to get a descriptive name for a renaming pattern
pub fn describe_renaming_pattern(pattern: &RenamingPattern) -> String {
    RenamingUtils::describe_pattern(pattern)
}

/// Create a simple exact match pattern
pub fn create_exact_pattern(
    name: String,
    action: RenamingAction,
    priority: u32,
) -> RenamingPattern {
    RenamingPattern::new(
        PatternMatcher::Exact(name),
        action,
        priority,
    )
}

/// Create a simple glob pattern
pub fn create_glob_pattern(
    pattern: String,
    action: RenamingAction,
    priority: u32,
) -> RenamingPattern {
    RenamingPattern::new(
        PatternMatcher::Glob(pattern),
        action,
        priority,
    )
}

/// Create a predicate-based pattern
pub fn create_predicate_pattern(
    predicate: PredicateFunction,
    action: RenamingAction,
    priority: u32,
) -> RenamingPattern {
    RenamingPattern::new(
        PatternMatcher::Predicate(predicate),
        action,
        priority,
    )
}

/// Create a temporary variable detection pattern
pub fn create_temporary_variable_pattern(priority: u32) -> RenamingPattern {
    RenamingPattern::new(
        PatternMatcher::Predicate(
            PredicateFunction::BuiltIn(BuiltInPredicate::IsTemporary)
        ),
        RenamingAction::AlwaysRename,
        priority,
    )
}

/// Create a pattern for lambda variables
pub fn create_lambda_variable_pattern(priority: u32) -> RenamingPattern {
    RenamingPattern::new(
        PatternMatcher::Predicate(
            PredicateFunction::BuiltIn(BuiltInPredicate::LengthRange(1, 2))
        ),
        RenamingAction::CustomNaming("lambda-prefix".to_string()),
        priority,
    )
    .with_macro_context("lambda".to_string())
}

/// Performance benchmark for renaming strategies
pub fn benchmark_renaming_strategies(
    expr: &Expr,
    context: &mut ExpansionContext,
    environment: &HygienicEnvironment,
    iterations: usize,
) -> BenchmarkResults {
    let strategies = vec![
        ("Conservative", StandardRenamingStrategies::conservative()),
        ("ConflictAware", StandardRenamingStrategies::conflict_aware()),
        ("Aggressive", StandardRenamingStrategies::aggressive()),
        ("PerformanceOptimized", StandardRenamingStrategies::performance_optimized()),
        ("Intelligent", StandardRenamingStrategies::intelligent()),
    ];
    
    let mut results = BenchmarkResults {
        strategy_times: std::collections::HashMap::new(),
        strategy_stats: std::collections::HashMap::new(),
    };
    
    for (name, strategy) in strategies {
        let start_time = std::time::Instant::now();
        let mut renamer = SymbolRenamer::new(strategy);
        
        for _ in 0..iterations {
            let mut ctx_clone = context.clone();
            let _ = renamer.rename_symbols(expr, &mut ctx_clone, environment);
        }
        
        let elapsed = start_time.elapsed();
        let stats = renamer.performance_stats().clone();
        
        results.strategy_times.insert(name.to_string(), elapsed);
        results.strategy_stats.insert(name.to_string(), stats);
    }
    
    results
}

/// Benchmark results for renaming strategies
#[derive(Debug)]
pub struct BenchmarkResults {
    /// Timing results per strategy
    pub strategy_times: std::collections::HashMap<String, std::time::Duration>,
    /// Statistics per strategy
    pub strategy_stats: std::collections::HashMap<String, RenamingStats>,
}

impl BenchmarkResults {
    /// Get the fastest strategy
    #[must_use] pub fn fastest_strategy(&self) -> Option<(&String, &std::time::Duration)> {
        self.strategy_times
            .iter()
            .min_by_key(|(_, &duration)| duration)
    }
    
    /// Get strategy with highest rename rate
    #[must_use] pub fn most_aggressive_strategy(&self) -> Option<(&String, &RenamingStats)> {
        self.strategy_stats
            .iter()
            .max_by(|(_, a), (_, b)| a.rename_rate().partial_cmp(&b.rename_rate()).unwrap_or(std::cmp::Ordering::Equal))
    }
    
    /// Get strategy with best cache performance
    #[must_use] pub fn best_cache_performance(&self) -> Option<(&String, &RenamingStats)> {
        self.strategy_stats
            .iter()
            .max_by(|(_, a), (_, b)| a.cache_hit_rate().partial_cmp(&b.cache_hit_rate()).unwrap_or(std::cmp::Ordering::Equal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_symbol_renamer() {
        let renamer = create_symbol_renamer();
        assert!(matches!(renamer.current_strategy(), RenamingStrategy::RenameConflicts));
    }

    #[test]
    fn test_conservative_renamer() {
        let renamer = create_conservative_renamer();
        assert!(matches!(renamer.current_strategy(), RenamingStrategy::Conservative));
    }

    #[test]
    fn test_pattern_creation() {
        let pattern = create_exact_pattern(
            "test".to_string(),
            RenamingAction::AlwaysRename,
            10,
        );
        
        match pattern.name_pattern {
            PatternMatcher::Exact(name) => assert_eq!(name, "test"),
            _ => panic!("Expected exact pattern"),
        }
        
        assert!(matches!(pattern.action, RenamingAction::AlwaysRename));
        assert_eq!(pattern.priority, 10);
    }

    #[test]
    fn test_utility_functions() {
        assert!(is_temporary_symbol("temp123"));
        assert!(is_temporary_symbol("tmp"));
        assert!(is_temporary_symbol("_var"));
        assert!(!is_temporary_symbol("normal"));

        assert!(is_generated_symbol("gensym123"));
        assert!(is_generated_symbol("$var"));
        assert!(!is_generated_symbol("normal"));

        assert!(is_lisp_case_symbol("my-variable"));
        assert!(is_lisp_case_symbol("test?"));
        assert!(!is_lisp_case_symbol("CamelCase"));
    }

    #[test]
    fn test_temporary_variable_pattern() {
        let pattern = create_temporary_variable_pattern(5);
        
        match pattern.name_pattern {
            PatternMatcher::Predicate(PredicateFunction::BuiltIn(BuiltInPredicate::IsTemporary)) => (),
            _ => panic!("Expected temporary variable predicate"),
        }
        
        assert!(matches!(pattern.action, RenamingAction::AlwaysRename));
        assert_eq!(pattern.priority, 5);
    }

    #[test]
    fn test_lambda_variable_pattern() {
        let pattern = create_lambda_variable_pattern(8);
        
        match pattern.name_pattern {
            PatternMatcher::Predicate(PredicateFunction::BuiltIn(BuiltInPredicate::LengthRange(1, 2))) => (),
            _ => panic!("Expected length range predicate"),
        }
        
        assert_eq!(pattern.macro_context, Some("lambda".to_string()));
        assert!(matches!(pattern.action, RenamingAction::CustomNaming(_)));
        assert_eq!(pattern.priority, 8);
    }

    #[test]
    fn test_strategy_config() {
        let config = StrategyConfig::balanced()
            .with_temporary_variables()
            .with_lambda_variables();
        
        assert!(config.rename_temporary_variables);
        assert!(config.rename_lambda_variables);
        assert_eq!(config.strategy_type, StrategyType::ConflictAware);
    }
}