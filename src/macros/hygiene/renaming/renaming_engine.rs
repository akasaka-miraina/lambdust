//! Symbol Renaming Engine Implementation
//!
//! このモジュールはシンボルリネーミングエンジンのメイン実装を
//! 提供します。

use super::core_types::{
    RenamingStrategy, RenamingStats, ScopeTracker, CustomRenamingRule,
    RenamingAction, DefaultAction
};
use super::pattern_matching::PatternMatchingEngine;
use super::super::symbol::HygienicSymbol;
use super::super::environment::{HygienicEnvironment, SymbolResolution};
use super::super::context::ExpansionContext;
use crate::ast::Expr;
use crate::error::Result;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Advanced symbol renaming engine with optimization and analytics
#[derive(Debug)]
pub struct SymbolRenamer {
    /// Current renaming strategy
    strategy: RenamingStrategy,
    /// Conflict detection cache
    conflict_cache: HashMap<String, bool>,
    /// Pattern matching engine
    pattern_engine: PatternMatchingEngine,
    /// Renaming statistics
    stats: RenamingStats,
    /// Symbol frequency tracking for intelligent renaming
    symbol_frequency: HashMap<String, u32>,
    /// Scope depth tracking
    scope_tracking: ScopeTracker,
}

impl SymbolRenamer {
    /// Create new renamer with strategy
    #[must_use] pub fn new(strategy: RenamingStrategy) -> Self {
        Self {
            strategy,
            conflict_cache: HashMap::new(),
            pattern_engine: PatternMatchingEngine::new(),
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
        
        let result = match &self.strategy.clone() {
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
                self.custom_renaming(expr, context, environment, &rule)
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
            Expr::Vector(exprs) => exprs.iter().map(|e| self.count_symbols(e)).sum(),
            Expr::Quote(e) => self.count_symbols(e),
            _ => 0,
        }
    }
    
    /// Rename all macro-introduced symbols
    fn rename_all_symbols(
        &self,
        expr: &Expr,
        context: &mut ExpansionContext,
        environment: &HygienicEnvironment,
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
                    .map(|e| self.rename_all_symbols(e, context, environment))
                    .collect();
                Ok(Expr::List(renamed_exprs?))
            }
            Expr::Vector(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.rename_all_symbols(e, context, environment))
                    .collect();
                Ok(Expr::Vector(renamed_exprs?))
            }
            Expr::Quote(e) => {
                let renamed = self.rename_all_symbols(e, context, environment)?;
                Ok(Expr::Quote(Box::new(renamed)))
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
            Expr::Vector(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.rename_conflicting_symbols(e, context, environment))
                    .collect();
                Ok(Expr::Vector(renamed_exprs?))
            }
            Expr::Quote(e) => {
                let renamed = self.rename_conflicting_symbols(e, context, environment)?;
                Ok(Expr::Quote(Box::new(renamed)))
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
            Expr::Vector(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.conservative_renaming(e, context, environment))
                    .collect();
                Ok(Expr::Vector(renamed_exprs?))
            }
            Expr::Quote(e) => {
                let renamed = self.conservative_renaming(e, context, environment)?;
                Ok(Expr::Quote(Box::new(renamed)))
            }
            _ => Ok(expr.clone()),
        }
    }
    
    /// Custom renaming based on rules
    fn custom_renaming(
        &mut self,
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
                let mut renamed_exprs = Vec::new();
                for e in exprs {
                    renamed_exprs.push(self.custom_renaming(e, context, environment, rule)?);
                }
                Ok(Expr::List(renamed_exprs))
            }
            Expr::Vector(exprs) => {
                let mut renamed_exprs = Vec::new();
                for e in exprs {
                    renamed_exprs.push(self.custom_renaming(e, context, environment, rule)?);
                }
                Ok(Expr::Vector(renamed_exprs))
            }
            Expr::Quote(e) => {
                let renamed = self.custom_renaming(e, context, environment, rule)?;
                Ok(Expr::Quote(Box::new(renamed)))
            }
            _ => Ok(expr.clone()),
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
            Expr::Vector(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.intelligent_renaming(e, context, environment))
                    .collect();
                Ok(Expr::Vector(renamed_exprs?))
            }
            Expr::Quote(e) => {
                let renamed = self.intelligent_renaming(e, context, environment)?;
                Ok(Expr::Quote(Box::new(renamed)))
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
            Expr::Vector(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.scope_aware_renaming(e, context, environment))
                    .collect();
                Ok(Expr::Vector(renamed_exprs?))
            }
            Expr::Quote(e) => {
                let renamed = self.scope_aware_renaming(e, context, environment)?;
                Ok(Expr::Quote(Box::new(renamed)))
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
            Expr::Vector(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.performance_optimized_renaming(e, context, environment))
                    .collect();
                Ok(Expr::Vector(renamed_exprs?))
            }
            Expr::Quote(e) => {
                let renamed = self.performance_optimized_renaming(e, context, environment)?;
                Ok(Expr::Quote(Box::new(renamed)))
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
            Expr::Vector(exprs) => {
                let renamed_exprs: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.context_sensitive_renaming(e, context, environment))
                    .collect();
                Ok(Expr::Vector(renamed_exprs?))
            }
            Expr::Quote(e) => {
                let renamed = self.context_sensitive_renaming(e, context, environment)?;
                Ok(Expr::Quote(Box::new(renamed)))
            }
            _ => Ok(expr.clone()),
        }
    }
    
    // Helper methods for conflict detection and naming
    
    /// Check if symbol would cause conflict
    pub fn would_cause_conflict(
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
        &mut self,
        name: &str,
        context: &ExpansionContext,
        rule: &CustomRenamingRule,
    ) -> RenamingAction {
        // Check each pattern by priority
        for pattern in rule.patterns_by_priority() {
            if self.pattern_engine.matches_pattern(name, context, pattern) {
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
    
    /// Apply custom naming pattern
    fn apply_custom_naming(
        &self,
        name: &str,
        pattern: &str,
        context: &mut ExpansionContext,
    ) -> HygienicSymbol {
        // Simple pattern substitution (could be enhanced)
        match pattern {
            "prefix-lambda" | "lambda-prefix" => {
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
        matches!(name, "t" | "x" | "y" | "z" | "i" | "j" | "k")
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
    
    // Public interface methods
    
    /// Get performance statistics
    #[must_use] pub fn performance_stats(&self) -> &RenamingStats {
        &self.stats
    }
    
    /// Reset performance statistics
    pub fn reset_stats(&mut self) {
        self.stats = RenamingStats::default();
        self.conflict_cache.clear();
        self.symbol_frequency.clear();
        self.pattern_engine.clear_cache();
    }
    
    /// Optimize caches for performance
    pub fn optimize_caches(&mut self) {
        // Remove least recently used entries if caches are too large
        if self.conflict_cache.len() > 10000 {
            self.conflict_cache.clear();
        }
        
        // Keep only top frequency symbols
        if self.symbol_frequency.len() > 5000 {
            let mut freq_vec: Vec<_> = self.symbol_frequency.drain().collect();
            freq_vec.sort_by(|a, b| b.1.cmp(&a.1));
            freq_vec.truncate(2500);
            self.symbol_frequency = freq_vec.into_iter().collect();
        }
        
        self.pattern_engine.optimize_cache();
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
        self.pattern_engine.clear_cache();
    }
    
    /// Get scope tracking information
    #[must_use] pub fn scope_tracking(&self) -> &ScopeTracker {
        &self.scope_tracking
    }
    
    /// Clear scope tracking
    pub fn clear_scope_tracking(&mut self) {
        self.scope_tracking.clear();
    }
}

impl Clone for SymbolRenamer {
    fn clone(&self) -> Self {
        Self {
            strategy: self.strategy.clone(),
            conflict_cache: self.conflict_cache.clone(),
            pattern_engine: PatternMatchingEngine::new(), // Fresh instance for cloned renamer
            stats: self.stats.clone(),
            symbol_frequency: self.symbol_frequency.clone(),
            scope_tracking: self.scope_tracking.clone(),
        }
    }
}