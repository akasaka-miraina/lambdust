//! Core Types for Symbol Renaming System
//!
//! このモジュールは衛生的マクロシステムでのシンボルリネーミングの
//! 基本的な型定義とデータ構造を定義します。

use std::collections::{HashMap, HashSet};

/// Advanced strategy for renaming symbols during macro expansion
#[derive(Debug, Clone)]
pub enum RenamingStrategy {
    /// Rename all macro-introduced symbols
    RenameAll,
    /// Rename only symbols that would cause conflicts
    RenameConflicts,
    /// Custom renaming using provided rules
    Custom(CustomRenamingRule),
    /// Conservative renaming (minimal changes)
    Conservative,
    /// Intelligent renaming with machine learning-inspired heuristics
    Intelligent,
    /// Scope-aware renaming (considers lexical scope depth)
    ScopeAware,
    /// Performance-optimized renaming with caching
    PerformanceOptimized,
    /// Context-sensitive renaming (considers macro call site)
    ContextSensitive,
}

/// Custom renaming rule specification
#[derive(Debug, Clone)]
pub struct CustomRenamingRule {
    /// Patterns to match for renaming
    pub patterns: Vec<RenamingPattern>,
    /// Default action for unmatched symbols
    pub default_action: DefaultAction,
}

/// Advanced pattern for matching symbols to rename
#[derive(Debug, Clone)]
pub struct RenamingPattern {
    /// Name pattern (supports glob, regex, and custom patterns)
    pub name_pattern: PatternMatcher,
    /// Macro context pattern
    pub macro_context: Option<String>,
    /// Scope depth constraint
    pub scope_depth: Option<ScopeConstraint>,
    /// Type constraint (if available)
    pub type_constraint: Option<TypeConstraint>,
    /// Action to take
    pub action: RenamingAction,
    /// Priority (higher values take precedence)
    pub priority: u32,
}

/// Pattern matching strategies for symbol names
#[derive(Debug, Clone)]
pub enum PatternMatcher {
    /// Exact string match
    Exact(String),
    /// Glob-style pattern (supports * and ?)
    Glob(String),
    /// Regular expression pattern
    Regex(String),
    /// Custom predicate function
    Predicate(PredicateFunction),
    /// Multiple patterns (any match)
    Multiple(Vec<PatternMatcher>),
}

/// Scope depth constraint for pattern matching
#[derive(Debug, Clone)]
pub enum ScopeConstraint {
    /// Exact depth
    Exact(usize),
    /// Minimum depth
    AtLeast(usize),
    /// Maximum depth
    AtMost(usize),
    /// Range of depths
    Range(usize, usize),
}

/// Type constraint for symbols (when type information is available)
#[derive(Debug, Clone)]
pub enum TypeConstraint {
    /// Symbol refers to a procedure
    Procedure,
    /// Symbol refers to a variable
    Variable,
    /// Symbol refers to a macro
    Macro,
    /// Symbol refers to a syntax keyword
    Syntax,
    /// Custom type predicate
    Custom(String),
}

/// Predicate function for custom pattern matching
#[derive(Debug, Clone)]
pub enum PredicateFunction {
    /// Built-in predicates
    BuiltIn(BuiltInPredicate),
    /// User-defined predicate (function name)
    UserDefined(String),
}

/// Built-in predicate functions
#[derive(Debug, Clone)]
pub enum BuiltInPredicate {
    /// Symbol starts with prefix
    StartsWith(String),
    /// Symbol ends with suffix
    EndsWith(String),
    /// Symbol contains substring
    Contains(String),
    /// Symbol length check
    LengthRange(usize, usize),
    /// Symbol is alphanumeric
    IsAlphanumeric,
    /// Symbol follows naming convention
    IsLispCase,
    /// Symbol is a temporary variable pattern
    IsTemporary,
}

/// Action to take when renaming
#[derive(Debug, Clone)]
pub enum RenamingAction {
    /// Always rename
    AlwaysRename,
    /// Never rename
    NeverRename,
    /// Rename if conflicts detected
    RenameOnConflict,
    /// Apply custom naming function
    CustomNaming(String), // Function name or pattern
}

/// Default action for unmatched symbols
#[derive(Debug, Clone)]
pub enum DefaultAction {
    /// Rename by default
    Rename,
    /// Don't rename by default
    Keep,
    /// Check for conflicts
    CheckConflicts,
}

/// Statistics for renaming operations
#[derive(Debug, Clone, Default)]
pub struct RenamingStats {
    /// Total symbols processed
    pub symbols_processed: u64,
    /// Symbols renamed
    pub symbols_renamed: u64,
    /// Conflicts detected
    pub conflicts_detected: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Total processing time (nanoseconds)
    pub total_processing_time_ns: u64,
    /// Pattern matching time (nanoseconds)
    pub pattern_matching_time_ns: u64,
}

/// Scope tracking for intelligent renaming decisions
#[derive(Debug, Clone, Default)]
pub struct ScopeTracker {
    /// Current scope depth
    pub current_depth: usize,
    /// Symbol introduction points by scope
    pub symbol_scopes: HashMap<String, Vec<usize>>,
    /// Scope collision detection
    pub scope_conflicts: HashMap<usize, HashSet<String>>,
}

impl RenamingStats {
    /// Calculate rename rate
    #[must_use] pub fn rename_rate(&self) -> f64 {
        if self.symbols_processed == 0 {
            0.0
        } else {
            self.symbols_renamed as f64 / self.symbols_processed as f64
        }
    }

    /// Calculate cache hit rate
    #[must_use] pub fn cache_hit_rate(&self) -> f64 {
        let total_accesses = self.cache_hits + self.cache_misses;
        if total_accesses == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_accesses as f64
        }
    }

    /// Calculate average processing time per symbol (nanoseconds)
    #[must_use] pub fn avg_processing_time_per_symbol(&self) -> f64 {
        if self.symbols_processed == 0 {
            0.0
        } else {
            self.total_processing_time_ns as f64 / self.symbols_processed as f64
        }
    }

    /// Get performance summary
    #[must_use] pub fn performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            symbols_processed: self.symbols_processed,
            symbols_renamed: self.symbols_renamed,
            rename_rate: self.rename_rate(),
            cache_hit_rate: self.cache_hit_rate(),
            avg_processing_time_ns: self.avg_processing_time_per_symbol(),
            total_processing_time_ns: self.total_processing_time_ns,
        }
    }
}

/// Performance summary for renaming operations
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    /// Total symbols processed
    pub symbols_processed: u64,
    /// Symbols renamed
    pub symbols_renamed: u64,
    /// Rename rate (0.0 to 1.0)
    pub rename_rate: f64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Average processing time per symbol (nanoseconds)
    pub avg_processing_time_ns: f64,
    /// Total processing time (nanoseconds)
    pub total_processing_time_ns: u64,
}

impl ScopeTracker {
    /// Create new scope tracker
    #[must_use] pub fn new() -> Self {
        Self::default()
    }

    /// Enter new scope
    pub fn enter_scope(&mut self) {
        self.current_depth += 1;
    }

    /// Exit current scope
    pub fn exit_scope(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
    }

    /// Record symbol introduction at current scope
    pub fn record_symbol(&mut self, name: String) {
        self.symbol_scopes
            .entry(name.clone())
            .or_default()
            .push(self.current_depth);
    }

    /// Record scope conflict
    pub fn record_conflict(&mut self, name: String) {
        self.scope_conflicts
            .entry(self.current_depth)
            .or_default()
            .insert(name);
    }

    /// Check if symbol has conflicts at current scope
    #[must_use] pub fn has_conflict_at_scope(&self, name: &str, scope: usize) -> bool {
        self.scope_conflicts
            .get(&scope)
            .map_or(false, |conflicts| conflicts.contains(name))
    }

    /// Get scope depth for symbol
    #[must_use] pub fn get_symbol_scopes(&self, name: &str) -> Option<&Vec<usize>> {
        self.symbol_scopes.get(name)
    }

    /// Clear tracking data
    pub fn clear(&mut self) {
        self.current_depth = 0;
        self.symbol_scopes.clear();
        self.scope_conflicts.clear();
    }

    /// Get total number of tracked symbols
    #[must_use] pub fn symbol_count(&self) -> usize {
        self.symbol_scopes.len()
    }

    /// Get total number of conflicts
    #[must_use] pub fn conflict_count(&self) -> usize {
        self.scope_conflicts.values().map(|set| set.len()).sum()
    }
}

/// Utility trait for renaming rules
pub trait RenamingRule: std::fmt::Debug {
    /// Check if symbol should be renamed
    fn should_rename(&self, name: &str, context: &super::super::context::ExpansionContext) -> bool;
    
    /// Generate new name for symbol
    fn generate_name(&self, name: &str, context: &super::super::context::ExpansionContext) -> String;
}

impl CustomRenamingRule {
    /// Create new custom renaming rule
    #[must_use] pub fn new(patterns: Vec<RenamingPattern>, default_action: DefaultAction) -> Self {
        Self {
            patterns,
            default_action,
        }
    }

    /// Add pattern to rule
    pub fn add_pattern(&mut self, pattern: RenamingPattern) {
        self.patterns.push(pattern);
        // Sort by priority (higher priority first)
        self.patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Get patterns sorted by priority
    #[must_use] pub fn patterns_by_priority(&self) -> Vec<&RenamingPattern> {
        let mut patterns: Vec<_> = self.patterns.iter().collect();
        patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
        patterns
    }

    /// Check if rule is empty
    #[must_use] pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Get number of patterns
    #[must_use] pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

impl RenamingPattern {
    /// Create new renaming pattern
    #[must_use] pub fn new(
        name_pattern: PatternMatcher,
        action: RenamingAction,
        priority: u32,
    ) -> Self {
        Self {
            name_pattern,
            macro_context: None,
            scope_depth: None,
            type_constraint: None,
            action,
            priority,
        }
    }

    /// Set macro context constraint
    pub fn with_macro_context(mut self, macro_context: String) -> Self {
        self.macro_context = Some(macro_context);
        self
    }

    /// Set scope depth constraint
    pub fn with_scope_constraint(mut self, scope_depth: ScopeConstraint) -> Self {
        self.scope_depth = Some(scope_depth);
        self
    }

    /// Set type constraint
    pub fn with_type_constraint(mut self, type_constraint: TypeConstraint) -> Self {
        self.type_constraint = Some(type_constraint);
        self
    }

    /// Check if pattern has constraints
    #[must_use] pub fn has_constraints(&self) -> bool {
        self.macro_context.is_some() || 
        self.scope_depth.is_some() || 
        self.type_constraint.is_some()
    }
}

impl ScopeConstraint {
    /// Check if scope depth satisfies constraint
    #[must_use] pub fn satisfies(&self, depth: usize) -> bool {
        match self {
            ScopeConstraint::Exact(exact) => depth == *exact,
            ScopeConstraint::AtLeast(min) => depth >= *min,
            ScopeConstraint::AtMost(max) => depth <= *max,
            ScopeConstraint::Range(min, max) => depth >= *min && depth <= *max,
        }
    }
}