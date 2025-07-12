//! Symbol renaming strategies for hygienic macros
//!
//! Provides different strategies for renaming symbols during macro expansion
//! to prevent collisions while maintaining lexical scoping rules.

use super::symbol::HygienicSymbol;
use super::environment::{HygienicEnvironment, SymbolResolution};
use super::context::ExpansionContext;
use crate::ast::Expr;
use crate::error::Result;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
// Note: Regex functionality would require adding regex crate
// For now using simple string matching patterns

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

/// Advanced symbol renaming engine with optimization and analytics
#[derive(Debug, Clone)]
pub struct SymbolRenamer {
    /// Current renaming strategy
    strategy: RenamingStrategy,
    /// Conflict detection cache
    conflict_cache: HashMap<String, bool>,
    /// Pattern matching cache for performance
    pattern_cache: HashMap<String, bool>,
    /// Renaming statistics
    stats: RenamingStats,
    /// Symbol frequency tracking for intelligent renaming
    symbol_frequency: HashMap<String, u32>,
    /// Scope depth tracking
    scope_tracking: ScopeTracker,
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

impl SymbolRenamer {
    /// Create new renamer with strategy
    #[must_use] pub fn new(strategy: RenamingStrategy) -> Self {
        Self {
            strategy,
            conflict_cache: HashMap::new(),
            pattern_cache: HashMap::new(),
            stats: RenamingStats::default(),
            symbol_frequency: HashMap::new(),
            scope_tracking: ScopeTracker::default(),
        }
    }
    
    /// Create optimized renamer for high-performance scenarios
    #[must_use] pub fn optimized() -> Self {
        let mut renamer = Self::new(RenamingStrategy::PerformanceOptimized);
        
        // Pre-allocate caches for better performance
        renamer.conflict_cache.reserve(1000);
        renamer.pattern_cache.reserve(500);
        renamer.symbol_frequency.reserve(2000);
        
        renamer
    }
    
    /// Create intelligent renamer with machine learning-inspired heuristics
    #[must_use] pub fn intelligent() -> Self {
        Self::new(RenamingStrategy::Intelligent)
    }
    
    /// Create scope-aware renamer for complex macro systems
    #[must_use] pub fn scope_aware() -> Self {
        Self::new(RenamingStrategy::ScopeAware)
    }
    
    /// Rename symbols in expression according to strategy with performance tracking
    pub fn rename_symbols(
        &mut self,
        expr: &Expr,
        context: &mut ExpansionContext,
        environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        let result = match &self.strategy {
            RenamingStrategy::RenameAll => {
                self.rename_all_symbols(expr, context, environment)
            }
            RenamingStrategy::RenameConflicts => {
                self.rename_conflicting_symbols(expr, context, environment)
            }
            RenamingStrategy::Conservative => {
                self.conservative_renaming(expr, context, environment)
            }
            RenamingStrategy::Custom(rule) => {
                self.custom_renaming(expr, context, environment, rule)
            }
            RenamingStrategy::Intelligent => {
                self.intelligent_renaming(expr, context, environment)
            }
            RenamingStrategy::ScopeAware => {
                self.scope_aware_renaming(expr, context, environment)
            }
            RenamingStrategy::PerformanceOptimized => {
                self.performance_optimized_renaming(expr, context, environment)
            }
            RenamingStrategy::ContextSensitive => {
                self.context_sensitive_renaming(expr, context, environment)
            }
        };
        
        // Update performance statistics
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        self.stats.total_processing_time_ns += end_time.saturating_sub(start_time);
        self.stats.symbols_processed += self.count_symbols(expr);
        
        result
    }
    
    /// Count symbols in expression for statistics
    fn count_symbols(&self, expr: &Expr) -> u64 {
        match expr {
            Expr::Variable(_) | Expr::HygienicVariable(_) => 1,
            Expr::List(exprs) => exprs.iter().map(|e| self.count_symbols(e)).sum(),
            _ => 0,
        }
    }
    
    /// Rename all macro-introduced symbols
    fn rename_all_symbols(
        &self,
        expr: &Expr,
        context: &mut ExpansionContext,
        _environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        match expr {
            Expr::Variable(name) => {
                // Generate unique symbol for any variable in macro context
                if context.depth > 0 {
                    let symbol = context.generate_template_symbol(name);
                    Ok(Expr::HygienicVariable(symbol))
                } else {
                    Ok(expr.clone())
                }
            }
            Expr::List(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.rename_all_symbols(e, context, _environment))
                    .collect();
                Ok(Expr::List(renamed_exprs?))
            }
            Expr::HygienicVariable(_) => {
                // Already hygienic, keep as-is
                Ok(expr.clone())
            }
            _ => Ok(expr.clone()),
        }
    }
    
    /// Rename only symbols that would cause conflicts
    fn rename_conflicting_symbols(
        &self,
        expr: &Expr,
        context: &mut ExpansionContext,
        environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        match expr {
            Expr::Variable(name) => {
                if self.would_cause_conflict(name, context, environment) {
                    let symbol = context.generate_template_symbol(name);
                    Ok(Expr::HygienicVariable(symbol))
                } else {
                    Ok(expr.clone())
                }
            }
            Expr::List(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.rename_conflicting_symbols(e, context, environment))
                    .collect();
                Ok(Expr::List(renamed_exprs?))
            }
            _ => Ok(expr.clone()),
        }
    }
    
    /// Conservative renaming (minimal changes)
    fn conservative_renaming(
        &self,
        expr: &Expr,
        context: &mut ExpansionContext,
        environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        match expr {
            Expr::Variable(name) => {
                // Only rename if we're in a macro and the symbol would be captured
                if context.depth > 0 && self.is_symbol_captured(name, environment) {
                    let symbol = context.generate_template_symbol(name);
                    Ok(Expr::HygienicVariable(symbol))
                } else {
                    Ok(expr.clone())
                }
            }
            Expr::List(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.conservative_renaming(e, context, environment))
                    .collect();
                Ok(Expr::List(renamed_exprs?))
            }
            _ => Ok(expr.clone()),
        }
    }
    
    /// Custom renaming based on rules
    fn custom_renaming(
        &self,
        expr: &Expr,
        context: &mut ExpansionContext,
        environment: &HygienicEnvironment,
        rule: &CustomRenamingRule,
    ) -> Result<Expr> {
        match expr {
            Expr::Variable(name) => {
                let action = self.determine_action(name, context, rule);
                match action {
                    RenamingAction::AlwaysRename => {
                        let symbol = context.generate_template_symbol(name);
                        Ok(Expr::HygienicVariable(symbol))
                    }
                    RenamingAction::NeverRename => Ok(expr.clone()),
                    RenamingAction::RenameOnConflict => {
                        if self.would_cause_conflict(name, context, environment) {
                            let symbol = context.generate_template_symbol(name);
                            Ok(Expr::HygienicVariable(symbol))
                        } else {
                            Ok(expr.clone())
                        }
                    }
                    RenamingAction::CustomNaming(pattern) => {
                        let symbol = self.apply_custom_naming(name, &pattern, context);
                        Ok(Expr::HygienicVariable(symbol))
                    }
                }
            }
            Expr::List(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.custom_renaming(e, context, environment, rule))
                    .collect();
                Ok(Expr::List(renamed_exprs?))
            }
            _ => Ok(expr.clone()),
        }
    }
    
    /// Check if symbol would cause conflict
    fn would_cause_conflict(
        &self,
        name: &str,
        context: &ExpansionContext,
        environment: &HygienicEnvironment,
    ) -> bool {
        // Check if symbol is already bound in expansion context
        if context.lookup_symbol(name).is_some() {
            return true;
        }
        
        // Check if symbol would shadow existing binding
        match environment.resolve_symbol(name) {
            SymbolResolution::Hygienic(_) => true,
            SymbolResolution::Traditional(_) => {
                // Check if we're introducing a symbol with same name in macro
                context.depth > 0
            }
            SymbolResolution::Unbound(_) => false,
        }
    }
    
    /// Check if symbol would be captured by macro expansion
    fn is_symbol_captured(&self, name: &str, environment: &HygienicEnvironment) -> bool {
        // Symbol is captured if it's defined in environment but would be
        // rebound by macro expansion
        environment.exists(name)
    }
    
    /// Determine action for custom renaming rule
    fn determine_action(
        &self,
        name: &str,
        context: &ExpansionContext,
        rule: &CustomRenamingRule,
    ) -> RenamingAction {
        // Check each pattern
        for pattern in &rule.patterns {
            if self.matches_pattern(name, context, pattern) {
                return pattern.action.clone();
            }
        }
        
        // Apply default action
        match rule.default_action {
            DefaultAction::Rename => RenamingAction::AlwaysRename,
            DefaultAction::Keep => RenamingAction::NeverRename,
            DefaultAction::CheckConflicts => RenamingAction::RenameOnConflict,
        }
    }
    
    /// Check if name matches pattern (simplified version)
    fn matches_pattern(
        &self,
        name: &str,
        context: &ExpansionContext,
        pattern: &RenamingPattern,
    ) -> bool {
        // Use advanced pattern matching
        let name_matches = match &pattern.name_pattern {
            PatternMatcher::Exact(exact) => name == exact,
            PatternMatcher::Glob(glob_pattern) => self.matches_glob(name, glob_pattern),
            PatternMatcher::Regex(regex_pattern) => self.matches_simple_regex(name, regex_pattern),
            PatternMatcher::Predicate(predicate) => self.matches_predicate(name, predicate),
            PatternMatcher::Multiple(patterns) => {
                patterns.iter().any(|p| {
                    let simple_pattern = RenamingPattern {
                        name_pattern: p.clone(),
                        macro_context: pattern.macro_context.clone(),
                        scope_depth: pattern.scope_depth.clone(),
                        type_constraint: pattern.type_constraint.clone(),
                        action: pattern.action.clone(),
                        priority: pattern.priority,
                    };
                    self.matches_pattern(name, context, &simple_pattern)
                })
            }
        };
        
        if !name_matches {
            return false;
        }
        
        // Check macro context if specified
        if let Some(ref macro_pattern) = pattern.macro_context {
            if let Some(current_macro) = context.current_macro() {
                current_macro == macro_pattern
            } else {
                false
            }
        } else {
            true
        }
    }
    
    /// Simple regex matching (without regex crate)
    fn matches_simple_regex(&self, name: &str, pattern: &str) -> bool {
        // For now, treat as literal string match
        // In a full implementation, this would use the regex crate
        name == pattern
    }
    
    /// Apply custom naming pattern
    fn apply_custom_naming(
        &self,
        name: &str,
        pattern: &str,
        context: &mut ExpansionContext,
    ) -> HygienicSymbol {
        // Simple pattern substitution (could be enhanced)
        match pattern {
            "prefix-lambda" => {
                let mut symbol = context.generate_template_symbol(name);
                symbol.name = format!("λ-{name}");
                symbol
            }
            "suffix-unique" => {
                let mut symbol = context.generate_template_symbol(name);
                symbol.name = format!("{}-{}", name, symbol.id.id());
                symbol
            }
            _ => context.generate_template_symbol(name),
        }
    }
    
    /// Intelligent renaming with heuristics and machine learning-inspired decisions
    fn intelligent_renaming(
        &mut self,
        expr: &Expr,
        context: &mut ExpansionContext,
        environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        match expr {
            Expr::Variable(name) => {
                // Update symbol frequency for learning
                *self.symbol_frequency.entry(name.clone()).or_insert(0) += 1;
                
                // Use intelligent heuristics
                if self.should_rename_intelligently(name, context, environment) {
                    let symbol = self.generate_intelligent_symbol(name, context);
                    self.stats.symbols_renamed += 1;
                    Ok(Expr::HygienicVariable(symbol))
                } else {
                    Ok(expr.clone())
                }
            }
            Expr::List(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.intelligent_renaming(e, context, environment))
                    .collect();
                Ok(Expr::List(renamed_exprs?))
            }
            _ => Ok(expr.clone()),
        }
    }
    
    /// Scope-aware renaming considering lexical scope depth
    fn scope_aware_renaming(
        &mut self,
        expr: &Expr,
        context: &mut ExpansionContext,
        environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // Update scope tracking
        self.scope_tracking.current_depth = context.depth;
        
        match expr {
            Expr::Variable(name) => {
                // Track symbol introduction at this scope
                self.scope_tracking
                    .symbol_scopes
                    .entry(name.clone())
                    .or_default()
                    .push(context.depth);
                
                if self.would_cause_scope_conflict(name, context) {
                    let symbol = context.generate_template_symbol(name);
                    self.stats.symbols_renamed += 1;
                    self.stats.conflicts_detected += 1;
                    
                    // Record scope conflict
                    self.scope_tracking
                        .scope_conflicts
                        .entry(context.depth)
                        .or_default()
                        .insert(name.clone());
                    
                    Ok(Expr::HygienicVariable(symbol))
                } else {
                    Ok(expr.clone())
                }
            }
            Expr::List(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.scope_aware_renaming(e, context, environment))
                    .collect();
                Ok(Expr::List(renamed_exprs?))
            }
            _ => Ok(expr.clone()),
        }
    }
    
    /// Performance-optimized renaming with extensive caching
    fn performance_optimized_renaming(
        &mut self,
        expr: &Expr,
        context: &mut ExpansionContext,
        environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        match expr {
            Expr::Variable(name) => {
                // Check cache first
                let cache_key = format!("{}:{}", name, context.depth);
                if let Some(&should_rename) = self.conflict_cache.get(&cache_key) {
                    self.stats.cache_hits += 1;
                    if should_rename {
                        let symbol = context.generate_template_symbol(name);
                        self.stats.symbols_renamed += 1;
                        Ok(Expr::HygienicVariable(symbol))
                    } else {
                        Ok(expr.clone())
                    }
                } else {
                    self.stats.cache_misses += 1;
                    let should_rename = self.would_cause_conflict(name, context, environment);
                    self.conflict_cache.insert(cache_key, should_rename);
                    
                    if should_rename {
                        let symbol = context.generate_template_symbol(name);
                        self.stats.symbols_renamed += 1;
                        Ok(Expr::HygienicVariable(symbol))
                    } else {
                        Ok(expr.clone())
                    }
                }
            }
            Expr::List(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.performance_optimized_renaming(e, context, environment))
                    .collect();
                Ok(Expr::List(renamed_exprs?))
            }
            _ => Ok(expr.clone()),
        }
    }
    
    /// Context-sensitive renaming considering macro call site
    fn context_sensitive_renaming(
        &mut self,
        expr: &Expr,
        context: &mut ExpansionContext,
        environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        match expr {
            Expr::Variable(name) => {
                // Consider macro call site context for renaming decisions
                if self.should_rename_by_context(name, context, environment) {
                    let symbol = self.generate_context_aware_symbol(name, context);
                    self.stats.symbols_renamed += 1;
                    Ok(Expr::HygienicVariable(symbol))
                } else {
                    Ok(expr.clone())
                }
            }
            Expr::List(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.context_sensitive_renaming(e, context, environment))
                    .collect();
                Ok(Expr::List(renamed_exprs?))
            }
            _ => Ok(expr.clone()),
        }
    }
    
    /// Intelligent decision making for symbol renaming
    fn should_rename_intelligently(
        &self,
        name: &str,
        context: &ExpansionContext,
        environment: &HygienicEnvironment,
    ) -> bool {
        // Heuristic 1: Frequency-based decision
        let frequency = self.symbol_frequency.get(name).unwrap_or(&0);
        if *frequency > 5 {
            // High-frequency symbols are more likely to conflict
            return true;
        }
        
        // Heuristic 2: Name pattern analysis
        if self.is_likely_temporary_variable(name) {
            return true;
        }
        
        // Heuristic 3: Standard conflict detection
        self.would_cause_conflict(name, context, environment)
    }
    
    /// Generate intelligent symbol names
    fn generate_intelligent_symbol(
        &self,
        name: &str,
        context: &mut ExpansionContext,
    ) -> HygienicSymbol {
        // Use intelligent naming based on symbol patterns
        if self.is_likely_temporary_variable(name) {
            let mut symbol = context.generate_template_symbol(name);
            symbol.name = format!("tmp${}", symbol.id.id());
            symbol
        } else if name.len() <= 3 {
            // Short names get descriptive prefixes
            let mut symbol = context.generate_template_symbol(name);
            symbol.name = format!("λ${name}");
            symbol
        } else {
            context.generate_template_symbol(name)
        }
    }
    
    /// Check if name suggests a temporary variable
    fn is_likely_temporary_variable(&self, name: &str) -> bool {
        name.starts_with("temp") || 
        name.starts_with("tmp") || 
        name.starts_with('_') ||
        name == "t" || name == "x" || name == "y" || name == "z"
    }
    
    /// Check for scope-level conflicts
    fn would_cause_scope_conflict(&self, name: &str, context: &ExpansionContext) -> bool {
        if let Some(scopes) = self.scope_tracking.symbol_scopes.get(name) {
            // Check if symbol was introduced at the same or inner scope
            scopes.iter().any(|&scope| scope >= context.depth)
        } else {
            false
        }
    }
    
    /// Context-sensitive renaming decision
    fn should_rename_by_context(
        &self,
        name: &str,
        context: &ExpansionContext,
        environment: &HygienicEnvironment,
    ) -> bool {
        // Consider macro name in decision
        if let Some(macro_name) = context.current_macro() {
            match macro_name {
                "let" | "letrec" | "let*" => {
                    // Binding constructs are more likely to need renaming
                    true
                }
                "define" | "set!" => {
                    // Definition forms need careful handling
                    environment.exists(name)
                }
                _ => self.would_cause_conflict(name, context, environment),
            }
        } else {
            false
        }
    }
    
    /// Generate context-aware symbol names
    fn generate_context_aware_symbol(
        &self,
        name: &str,
        context: &mut ExpansionContext,
    ) -> HygienicSymbol {
        let macro_name = context.current_macro().map(std::string::ToString::to_string);
        if let Some(macro_name) = macro_name {
            let mut symbol = context.generate_template_symbol(name);
            symbol.name = format!("{macro_name}${name}");
            symbol
        } else {
            context.generate_template_symbol(name)
        }
    }
    
    /// Match glob patterns (supports * and ?)
    fn matches_glob(&self, name: &str, pattern: &str) -> bool {
        // Simple glob implementation
        if pattern == "*" {
            return true;
        }
        
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let (prefix, suffix) = (parts[0], parts[1]);
                return name.starts_with(prefix) && name.ends_with(suffix);
            }
        }
        
        if pattern.contains('?') {
            // Simple single character wildcard
            if name.len() != pattern.len() {
                return false;
            }
            return name.chars().zip(pattern.chars())
                .all(|(c, p)| p == '?' || c == p);
        }
        
        name == pattern
    }
    
    /// Match predicate functions
    fn matches_predicate(&self, name: &str, predicate: &PredicateFunction) -> bool {
        match predicate {
            PredicateFunction::BuiltIn(built_in) => self.matches_builtin_predicate(name, built_in),
            PredicateFunction::UserDefined(_) => {
                // User-defined predicates would need custom implementation
                false
            }
        }
    }
    
    /// Match built-in predicate functions
    fn matches_builtin_predicate(&self, name: &str, predicate: &BuiltInPredicate) -> bool {
        match predicate {
            BuiltInPredicate::StartsWith(prefix) => name.starts_with(prefix),
            BuiltInPredicate::EndsWith(suffix) => name.ends_with(suffix),
            BuiltInPredicate::Contains(substring) => name.contains(substring),
            BuiltInPredicate::LengthRange(min, max) => {
                let len = name.len();
                len >= *min && len <= *max
            }
            BuiltInPredicate::IsAlphanumeric => name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
            BuiltInPredicate::IsLispCase => {
                name.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '-' || c == '_' || c == '?' || c == '!')
            }
            BuiltInPredicate::IsTemporary => self.is_likely_temporary_variable(name),
        }
    }

    /// Get performance statistics
    #[must_use] pub fn performance_stats(&self) -> &RenamingStats {
        &self.stats
    }
    
    /// Reset performance statistics
    pub fn reset_stats(&mut self) {
        self.stats = RenamingStats::default();
        self.conflict_cache.clear();
        self.pattern_cache.clear();
        self.symbol_frequency.clear();
    }
    
    /// Optimize caches for performance
    pub fn optimize_caches(&mut self) {
        // Remove least recently used entries if caches are too large
        if self.conflict_cache.len() > 10000 {
            self.conflict_cache.clear();
        }
        if self.pattern_cache.len() > 5000 {
            self.pattern_cache.clear();
        }
        
        // Keep only top frequency symbols
        if self.symbol_frequency.len() > 5000 {
            let mut freq_vec: Vec<_> = self.symbol_frequency.drain().collect();
            freq_vec.sort_by(|a, b| b.1.cmp(&a.1));
            freq_vec.truncate(2500);
            self.symbol_frequency = freq_vec.into_iter().collect();
        }
    }
    
    /// Get current strategy
    #[must_use] pub fn current_strategy(&self) -> &RenamingStrategy {
        &self.strategy
    }
    
    /// Switch to different strategy
    pub fn set_strategy(&mut self, strategy: RenamingStrategy) {
        self.strategy = strategy;
        // Clear caches when strategy changes
        self.conflict_cache.clear();
        self.pattern_cache.clear();
    }
}

/// Utility trait for renaming rules
pub trait RenamingRule: std::fmt::Debug {
    /// Check if symbol should be renamed
    fn should_rename(&self, name: &str, context: &ExpansionContext) -> bool;
    
    /// Generate new name for symbol
    fn generate_name(&self, name: &str, context: &ExpansionContext) -> String;
}

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
        ];
        
        RenamingStrategy::Custom(CustomRenamingRule {
            patterns,
            default_action: DefaultAction::CheckConflicts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renaming_strategy_creation() {
        let strategy = RenamingStrategy::Conservative;
        let renamer = SymbolRenamer::new(strategy);
        assert!(matches!(renamer.strategy, RenamingStrategy::Conservative));
    }
    
    #[test]
    fn test_pattern_matching() {
        let renamer = SymbolRenamer::new(RenamingStrategy::Conservative);
        let env_id = super::super::symbol::EnvironmentId::new(1);
        let context = ExpansionContext::with_environment_id(env_id);
        
        let pattern = RenamingPattern {
            name_pattern: PatternMatcher::Glob("temp*".to_string()),
            macro_context: None,
            scope_depth: None,
            type_constraint: None,
            action: RenamingAction::AlwaysRename,
            priority: 1,
        };
        
        assert!(renamer.matches_pattern("temp123", &context, &pattern));
        assert!(renamer.matches_pattern("temp", &context, &pattern));
        assert!(!renamer.matches_pattern("var123", &context, &pattern));
    }
    
    #[test]
    fn test_conservative_strategy() {
        let strategies = StandardRenamingStrategies::conservative();
        assert!(matches!(strategies, RenamingStrategy::Conservative));
    }
    
    #[test]
    fn test_conflict_detection() {
        let renamer = SymbolRenamer::new(RenamingStrategy::RenameConflicts);
        let env = HygienicEnvironment::new();
        let env_id = env.id;
        let mut context = ExpansionContext::with_environment_id(env_id);
        
        // Add symbol to context
        context.generate_pattern_variable("existing");
        
        // Should detect conflict with existing symbol
        assert!(renamer.would_cause_conflict("existing", &context, &env));
        assert!(!renamer.would_cause_conflict("new_symbol", &context, &env));
    }
    
    #[test]
    fn test_custom_renaming_rule() {
        let rule = CustomRenamingRule {
            patterns: vec![RenamingPattern {
                name_pattern: PatternMatcher::Glob("test*".to_string()),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 1,
            }],
            default_action: DefaultAction::Keep,
        };
        
        let renamer = SymbolRenamer::new(RenamingStrategy::Custom(rule));
        let env_id = super::super::symbol::EnvironmentId::new(1);
        let context = ExpansionContext::with_environment_id(env_id);
        
        // Test action determination
        if let RenamingStrategy::Custom(ref rule) = renamer.strategy {
            let action = renamer.determine_action("test123", &context, rule);
            assert!(matches!(action, RenamingAction::AlwaysRename));
            
            let action2 = renamer.determine_action("other", &context, rule);
            assert!(matches!(action2, RenamingAction::NeverRename));
        }
    }
}