//! Standard Renaming Strategies and Utilities
//!
//! このモジュールは標準的なリネーミング戦略とユーティリティ関数を
//! 提供します。

use super::core_types::{
    RenamingStrategy, CustomRenamingRule, RenamingPattern, PatternMatcher,
    RenamingAction, DefaultAction, BuiltInPredicate, PredicateFunction
};

/// Predefined renaming strategies
pub struct StandardRenamingStrategies;

impl StandardRenamingStrategies {
    /// Conservative strategy (rename only when necessary)
    #[must_use] pub fn conservative() -> RenamingStrategy {
        RenamingStrategy::Conservative
    }
    
    /// Aggressive strategy (rename all macro symbols)
    #[must_use] pub fn aggressive() -> RenamingStrategy {
        RenamingStrategy::RenameAll
    }
    
    /// Conflict-aware strategy (rename only conflicting symbols)
    #[must_use] pub fn conflict_aware() -> RenamingStrategy {
        RenamingStrategy::RenameConflicts
    }
    
    /// Performance-optimized strategy with caching
    #[must_use] pub fn performance_optimized() -> RenamingStrategy {
        RenamingStrategy::PerformanceOptimized
    }
    
    /// Intelligent strategy with machine learning-inspired heuristics
    #[must_use] pub fn intelligent() -> RenamingStrategy {
        RenamingStrategy::Intelligent
    }
    
    /// Scope-aware strategy considering lexical scope depth
    #[must_use] pub fn scope_aware() -> RenamingStrategy {
        RenamingStrategy::ScopeAware
    }
    
    /// Context-sensitive strategy considering macro call site
    #[must_use] pub fn context_sensitive() -> RenamingStrategy {
        RenamingStrategy::ContextSensitive
    }
    
    /// Custom strategy for temporary variables
    #[must_use] pub fn temp_variables() -> RenamingStrategy {
        let patterns = vec![
            RenamingPattern {
                name_pattern: PatternMatcher::Glob("temp*".to_string()),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 10,
            },
            RenamingPattern {
                name_pattern: PatternMatcher::Glob("tmp*".to_string()),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 10,
            },
            RenamingPattern {
                name_pattern: PatternMatcher::Predicate(
                    PredicateFunction::BuiltIn(BuiltInPredicate::IsTemporary)
                ),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 8,
            },
        ];
        
        RenamingStrategy::Custom(CustomRenamingRule {
            patterns,
            default_action: DefaultAction::CheckConflicts,
        })
    }
    
    /// Custom strategy for lambda variables
    #[must_use] pub fn lambda_variables() -> RenamingStrategy {
        let patterns = vec![
            RenamingPattern {
                name_pattern: PatternMatcher::Predicate(
                    PredicateFunction::BuiltIn(BuiltInPredicate::LengthRange(1, 2))
                ),
                macro_context: Some("lambda".to_string()),
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::CustomNaming("lambda-prefix".to_string()),
                priority: 15,
            },
            RenamingPattern {
                name_pattern: PatternMatcher::Glob("λ*".to_string()),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 12,
            },
        ];
        
        RenamingStrategy::Custom(CustomRenamingRule {
            patterns,
            default_action: DefaultAction::CheckConflicts,
        })
    }
    
    /// Custom strategy for let bindings
    #[must_use] pub fn let_bindings() -> RenamingStrategy {
        let patterns = vec![
            RenamingPattern {
                name_pattern: PatternMatcher::Glob("*".to_string()),
                macro_context: Some("let".to_string()),
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::RenameOnConflict,
                priority: 5,
            },
            RenamingPattern {
                name_pattern: PatternMatcher::Glob("*".to_string()),
                macro_context: Some("let*".to_string()),
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 8,
            },
            RenamingPattern {
                name_pattern: PatternMatcher::Glob("*".to_string()),
                macro_context: Some("letrec".to_string()),
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 8,
            },
        ];
        
        RenamingStrategy::Custom(CustomRenamingRule {
            patterns,
            default_action: DefaultAction::Keep,
        })
    }
    
    /// Custom strategy for macro-generated symbols
    #[must_use] pub fn macro_generated() -> RenamingStrategy {
        let patterns = vec![
            RenamingPattern {
                name_pattern: PatternMatcher::Predicate(
                    PredicateFunction::BuiltIn(BuiltInPredicate::StartsWith("gensym".to_string()))
                ),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 20,
            },
            RenamingPattern {
                name_pattern: PatternMatcher::Predicate(
                    PredicateFunction::BuiltIn(BuiltInPredicate::StartsWith("$".to_string()))
                ),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 18,
            },
            RenamingPattern {
                name_pattern: PatternMatcher::Predicate(
                    PredicateFunction::BuiltIn(BuiltInPredicate::Contains("##".to_string()))
                ),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 16,
            },
        ];
        
        RenamingStrategy::Custom(CustomRenamingRule {
            patterns,
            default_action: DefaultAction::CheckConflicts,
        })
    }
    
    /// Defensive strategy (maximum hygiene)
    #[must_use] pub fn defensive() -> RenamingStrategy {
        let patterns = vec![
            RenamingPattern {
                name_pattern: PatternMatcher::Glob("*".to_string()),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 1,
            },
        ];
        
        RenamingStrategy::Custom(CustomRenamingRule {
            patterns,
            default_action: DefaultAction::Rename,
        })
    }
    
    /// Minimal strategy (minimal hygiene for performance)
    #[must_use] pub fn minimal() -> RenamingStrategy {
        let patterns = vec![
            RenamingPattern {
                name_pattern: PatternMatcher::Predicate(
                    PredicateFunction::BuiltIn(BuiltInPredicate::StartsWith("gensym".to_string()))
                ),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 10,
            },
        ];
        
        RenamingStrategy::Custom(CustomRenamingRule {
            patterns,
            default_action: DefaultAction::Keep,
        })
    }
}

/// Strategy factory for creating custom strategies
pub struct RenamingStrategyFactory;

impl RenamingStrategyFactory {
    /// Create strategy based on configuration
    #[must_use] pub fn create_strategy(config: &StrategyConfig) -> RenamingStrategy {
        match config.strategy_type {
            StrategyType::Conservative => StandardRenamingStrategies::conservative(),
            StrategyType::Aggressive => StandardRenamingStrategies::aggressive(),
            StrategyType::ConflictAware => StandardRenamingStrategies::conflict_aware(),
            StrategyType::PerformanceOptimized => StandardRenamingStrategies::performance_optimized(),
            StrategyType::Intelligent => StandardRenamingStrategies::intelligent(),
            StrategyType::ScopeAware => StandardRenamingStrategies::scope_aware(),
            StrategyType::ContextSensitive => StandardRenamingStrategies::context_sensitive(),
            StrategyType::Custom => Self::create_custom_strategy(config),
        }
    }
    
    /// Create custom strategy from configuration
    fn create_custom_strategy(config: &StrategyConfig) -> RenamingStrategy {
        let mut patterns = Vec::new();
        
        // Add patterns based on configuration
        if config.rename_temporary_variables {
            patterns.extend(Self::temporary_variable_patterns());
        }
        
        if config.rename_lambda_variables {
            patterns.extend(Self::lambda_variable_patterns());
        }
        
        if config.rename_let_bindings {
            patterns.extend(Self::let_binding_patterns());
        }
        
        if config.rename_macro_generated {
            patterns.extend(Self::macro_generated_patterns());
        }
        
        // Add user-defined patterns
        patterns.extend(config.custom_patterns.clone());
        
        RenamingStrategy::Custom(CustomRenamingRule {
            patterns,
            default_action: config.default_action.clone(),
        })
    }
    
    /// Get temporary variable patterns
    fn temporary_variable_patterns() -> Vec<RenamingPattern> {
        vec![
            RenamingPattern {
                name_pattern: PatternMatcher::Predicate(
                    PredicateFunction::BuiltIn(BuiltInPredicate::IsTemporary)
                ),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 10,
            },
        ]
    }
    
    /// Get lambda variable patterns
    fn lambda_variable_patterns() -> Vec<RenamingPattern> {
        vec![
            RenamingPattern {
                name_pattern: PatternMatcher::Predicate(
                    PredicateFunction::BuiltIn(BuiltInPredicate::LengthRange(1, 2))
                ),
                macro_context: Some("lambda".to_string()),
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::CustomNaming("lambda-prefix".to_string()),
                priority: 8,
            },
        ]
    }
    
    /// Get let binding patterns
    fn let_binding_patterns() -> Vec<RenamingPattern> {
        vec![
            RenamingPattern {
                name_pattern: PatternMatcher::Glob("*".to_string()),
                macro_context: Some("let".to_string()),
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::RenameOnConflict,
                priority: 5,
            },
        ]
    }
    
    /// Get macro-generated patterns
    fn macro_generated_patterns() -> Vec<RenamingPattern> {
        vec![
            RenamingPattern {
                name_pattern: PatternMatcher::Predicate(
                    PredicateFunction::BuiltIn(BuiltInPredicate::StartsWith("gensym".to_string()))
                ),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 15,
            },
        ]
    }
}

/// Configuration for strategy creation
#[derive(Debug, Clone)]
pub struct StrategyConfig {
    /// Base strategy type
    pub strategy_type: StrategyType,
    /// Whether to rename temporary variables
    pub rename_temporary_variables: bool,
    /// Whether to rename lambda variables
    pub rename_lambda_variables: bool,
    /// Whether to rename let bindings
    pub rename_let_bindings: bool,
    /// Whether to rename macro-generated symbols
    pub rename_macro_generated: bool,
    /// Custom patterns to include
    pub custom_patterns: Vec<RenamingPattern>,
    /// Default action for unmatched symbols
    pub default_action: DefaultAction,
}

/// Strategy types
#[derive(Debug, Clone, PartialEq)]
pub enum StrategyType {
    /// Conservative renaming
    Conservative,
    /// Aggressive renaming
    Aggressive,
    /// Conflict-aware renaming
    ConflictAware,
    /// Performance-optimized renaming
    PerformanceOptimized,
    /// Intelligent renaming
    Intelligent,
    /// Scope-aware renaming
    ScopeAware,
    /// Context-sensitive renaming
    ContextSensitive,
    /// Custom strategy
    Custom,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            strategy_type: StrategyType::ConflictAware,
            rename_temporary_variables: true,
            rename_lambda_variables: false,
            rename_let_bindings: true,
            rename_macro_generated: true,
            custom_patterns: Vec::new(),
            default_action: DefaultAction::CheckConflicts,
        }
    }
}

impl StrategyConfig {
    /// Create new strategy configuration
    #[must_use] pub fn new(strategy_type: StrategyType) -> Self {
        Self {
            strategy_type,
            ..Default::default()
        }
    }
    
    /// Enable temporary variable renaming
    #[must_use] pub fn with_temporary_variables(mut self) -> Self {
        self.rename_temporary_variables = true;
        self
    }
    
    /// Enable lambda variable renaming
    #[must_use] pub fn with_lambda_variables(mut self) -> Self {
        self.rename_lambda_variables = true;
        self
    }
    
    /// Enable let binding renaming
    #[must_use] pub fn with_let_bindings(mut self) -> Self {
        self.rename_let_bindings = true;
        self
    }
    
    /// Enable macro-generated symbol renaming
    #[must_use] pub fn with_macro_generated(mut self) -> Self {
        self.rename_macro_generated = true;
        self
    }
    
    /// Add custom pattern
    #[must_use] pub fn with_pattern(mut self, pattern: RenamingPattern) -> Self {
        self.custom_patterns.push(pattern);
        self
    }
    
    /// Set default action
    #[must_use] pub fn with_default_action(mut self, action: DefaultAction) -> Self {
        self.default_action = action;
        self
    }
    
    /// Create minimal configuration (best performance)
    #[must_use] pub fn minimal() -> Self {
        Self {
            strategy_type: StrategyType::Conservative,
            rename_temporary_variables: false,
            rename_lambda_variables: false,
            rename_let_bindings: false,
            rename_macro_generated: true,
            custom_patterns: Vec::new(),
            default_action: DefaultAction::Keep,
        }
    }
    
    /// Create maximal configuration (maximum hygiene)
    #[must_use] pub fn maximal() -> Self {
        Self {
            strategy_type: StrategyType::Aggressive,
            rename_temporary_variables: true,
            rename_lambda_variables: true,
            rename_let_bindings: true,
            rename_macro_generated: true,
            custom_patterns: Vec::new(),
            default_action: DefaultAction::Rename,
        }
    }
    
    /// Create balanced configuration (good hygiene with reasonable performance)
    #[must_use] pub fn balanced() -> Self {
        Self {
            strategy_type: StrategyType::ConflictAware,
            rename_temporary_variables: true,
            rename_lambda_variables: false,
            rename_let_bindings: true,
            rename_macro_generated: true,
            custom_patterns: Vec::new(),
            default_action: DefaultAction::CheckConflicts,
        }
    }
}

/// Utility functions for common renaming tasks
pub struct RenamingUtils;

impl RenamingUtils {
    /// Check if symbol name suggests it's a temporary variable
    #[must_use] pub fn is_temporary_variable(name: &str) -> bool {
        name.starts_with("temp") || 
        name.starts_with("tmp") || 
        name.starts_with('_') ||
        matches!(name, "t" | "x" | "y" | "z" | "i" | "j" | "k")
    }
    
    /// Check if symbol name suggests it's compiler/macro generated
    #[must_use] pub fn is_generated_symbol(name: &str) -> bool {
        name.starts_with("gensym") ||
        name.starts_with('$') ||
        name.contains("##") ||
        name.contains("__")
    }
    
    /// Check if symbol name follows Lisp naming conventions
    #[must_use] pub fn is_lisp_case(name: &str) -> bool {
        name.chars().all(|c| {
            c.is_lowercase() || c.is_numeric() || 
            c == '-' || c == '_' || c == '?' || c == '!'
        })
    }
    
    /// Check if symbol name is likely a lambda parameter
    #[must_use] pub fn is_lambda_parameter(name: &str) -> bool {
        name.len() <= 2 && name.chars().all(char::is_lowercase)
    }
    
    /// Generate a descriptive name for a renaming pattern
    #[must_use] pub fn describe_pattern(pattern: &RenamingPattern) -> String {
        let name_desc = match &pattern.name_pattern {
            PatternMatcher::Exact(s) => format!("exact '{s}'"),
            PatternMatcher::Glob(s) => format!("glob '{s}'"),
            PatternMatcher::Regex(s) => format!("regex '{s}'"),
            PatternMatcher::Predicate(PredicateFunction::BuiltIn(pred)) => {
                match pred {
                    BuiltInPredicate::StartsWith(s) => format!("starts with '{s}'"),
                    BuiltInPredicate::EndsWith(s) => format!("ends with '{s}'"),
                    BuiltInPredicate::Contains(s) => format!("contains '{s}'"),
                    BuiltInPredicate::LengthRange(min, max) => format!("length {min}-{max}"),
                    BuiltInPredicate::IsAlphanumeric => "alphanumeric".to_string(),
                    BuiltInPredicate::IsLispCase => "lisp-case".to_string(),
                    BuiltInPredicate::IsTemporary => "temporary variable".to_string(),
                }
            }
            PatternMatcher::Predicate(PredicateFunction::UserDefined(s)) => format!("user predicate '{s}'"),
            PatternMatcher::Multiple(_) => "multiple patterns".to_string(),
        };
        
        let mut desc = name_desc;
        
        if let Some(ref macro_ctx) = pattern.macro_context {
            desc.push_str(&format!(" in macro '{macro_ctx}'"));
        }
        
        if let Some(ref scope) = pattern.scope_depth {
            let scope_desc = match scope {
                super::core_types::ScopeConstraint::Exact(d) => format!(" at scope depth {d}"),
                super::core_types::ScopeConstraint::AtLeast(d) => format!(" at scope depth >= {d}"),
                super::core_types::ScopeConstraint::AtMost(d) => format!(" at scope depth <= {d}"),
                super::core_types::ScopeConstraint::Range(min, max) => format!(" at scope depth {min}-{max}"),
            };
            desc.push_str(&scope_desc);
        }
        
        desc
    }
}
