//! Symbol renaming strategies for hygienic macros
//!
//! Provides different strategies for renaming symbols during macro expansion
//! to prevent collisions while maintaining lexical scoping rules.

use super::symbol::HygienicSymbol;
use super::environment::{HygienicEnvironment, SymbolResolution};
use super::context::ExpansionContext;
use crate::ast::Expr;
use crate::error::Result;

/// Strategy for renaming symbols during macro expansion
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
}

/// Custom renaming rule specification
#[derive(Debug, Clone)]
pub struct CustomRenamingRule {
    /// Patterns to match for renaming
    pub patterns: Vec<RenamingPattern>,
    /// Default action for unmatched symbols
    pub default_action: DefaultAction,
}

/// Pattern for matching symbols to rename
#[derive(Debug, Clone)]
pub struct RenamingPattern {
    /// Name pattern (glob-style)
    pub name_pattern: String,
    /// Macro context pattern
    pub macro_context: Option<String>,
    /// Action to take
    pub action: RenamingAction,
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

/// Symbol renaming engine
#[derive(Debug)]
pub struct SymbolRenamer {
    strategy: RenamingStrategy,
}

impl SymbolRenamer {
    /// Create new renamer with strategy
    pub fn new(strategy: RenamingStrategy) -> Self {
        Self { strategy }
    }
    
    /// Rename symbols in expression according to strategy
    pub fn rename_symbols(
        &self,
        expr: &Expr,
        context: &mut ExpansionContext,
        environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        match &self.strategy {
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
    
    /// Check if name matches pattern
    fn matches_pattern(
        &self,
        name: &str,
        context: &ExpansionContext,
        pattern: &RenamingPattern,
    ) -> bool {
        // Simple glob matching (could be enhanced)
        let name_matches = if pattern.name_pattern == "*" {
            true
        } else if pattern.name_pattern.ends_with('*') {
            let prefix = &pattern.name_pattern[..pattern.name_pattern.len() - 1];
            name.starts_with(prefix)
        } else {
            name == pattern.name_pattern
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
                symbol.name = format!("λ-{}", name);
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
    pub fn conservative() -> RenamingStrategy {
        RenamingStrategy::Conservative
    }
    
    /// Aggressive strategy (rename all macro symbols)
    pub fn aggressive() -> RenamingStrategy {
        RenamingStrategy::RenameAll
    }
    
    /// Conflict-aware strategy (rename only conflicting symbols)
    pub fn conflict_aware() -> RenamingStrategy {
        RenamingStrategy::RenameConflicts
    }
    
    /// Custom strategy for temporary variables
    pub fn temp_variables() -> RenamingStrategy {
        let patterns = vec![
            RenamingPattern {
                name_pattern: "temp*".to_string(),
                macro_context: None,
                action: RenamingAction::AlwaysRename,
            },
            RenamingPattern {
                name_pattern: "tmp*".to_string(),
                macro_context: None,
                action: RenamingAction::AlwaysRename,
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
    use crate::macros::hygiene::generator::SymbolGenerator;

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
        let context = ExpansionContext::new(env_id);
        
        let pattern = RenamingPattern {
            name_pattern: "temp*".to_string(),
            macro_context: None,
            action: RenamingAction::AlwaysRename,
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
        let mut context = ExpansionContext::new(env_id);
        
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
                name_pattern: "test*".to_string(),
                macro_context: None,
                action: RenamingAction::AlwaysRename,
            }],
            default_action: DefaultAction::Keep,
        };
        
        let renamer = SymbolRenamer::new(RenamingStrategy::Custom(rule));
        let env_id = super::super::symbol::EnvironmentId::new(1);
        let context = ExpansionContext::new(env_id);
        
        // Test action determination
        if let RenamingStrategy::Custom(ref rule) = renamer.strategy {
            let action = renamer.determine_action("test123", &context, rule);
            assert!(matches!(action, RenamingAction::AlwaysRename));
            
            let action2 = renamer.determine_action("other", &context, rule);
            assert!(matches!(action2, RenamingAction::NeverRename));
        }
    }
}