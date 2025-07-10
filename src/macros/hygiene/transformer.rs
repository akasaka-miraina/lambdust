//! Hygienic syntax-rules transformer
//!
//! Implements the core hygienic macro transformation logic, handling symbol renaming
//! and hygiene preservation during macro expansion.

use super::symbol::{HygienicSymbol, MacroSite};
use super::environment::{HygienicEnvironment, SymbolResolution};
use super::context::ExpansionContext;
use super::renaming::{RenamingStrategy, SymbolRenamer};
use crate::macros::{Pattern, SyntaxRule, Template};
use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use std::collections::HashMap;
use std::rc::Rc;

/// Pattern bindings for macro expansion
pub type PatternBindings = HashMap<String, Expr>;

/// Hygienic version of syntax-rules transformer
#[derive(Debug, Clone)]
pub struct HygienicSyntaxRulesTransformer {
    /// Literal identifiers that shouldn't be renamed
    pub literals: Vec<String>,
    /// Transformation rules
    pub rules: Vec<SyntaxRule>,
    /// Symbol renaming strategy
    pub renaming_strategy: RenamingStrategy,
    /// Lexical environment at definition site
    pub definition_environment: Rc<HygienicEnvironment>,
    /// Macro name for debugging
    pub macro_name: String,
}

impl HygienicSyntaxRulesTransformer {
    /// Create new hygienic transformer
    pub fn new(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
    ) -> Self {
        Self {
            literals,
            rules,
            renaming_strategy: RenamingStrategy::RenameConflicts,
            definition_environment,
            macro_name,
        }
    }
    
    /// Create transformer with custom renaming strategy
    pub fn with_renaming_strategy(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
        strategy: RenamingStrategy,
    ) -> Self {
        let mut transformer = Self::new(literals, rules, definition_environment, macro_name);
        transformer.renaming_strategy = strategy;
        transformer
    }
    
    /// Apply hygienic transformation with safety checks
    pub fn transform_hygienic(
        &self,
        input: &[Expr],
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // Check safety limits before transformation
        usage_environment.expansion_context.is_within_limits()
            .map_err(|e| LambdustError::runtime_error(format!("Expansion safety check failed: {}", e)))?;
        
        // Validate macro interaction safety
        usage_environment.expansion_context.validate_macro_interaction(&self.macro_name)
            .map_err(|e| LambdustError::runtime_error(e))?;
        
        // Reconstruct the full expression including macro name
        let mut full_expr = vec![Expr::Variable(self.macro_name.clone())];
        full_expr.extend_from_slice(input);
        let expr = Expr::List(full_expr);
        
        // Try each rule until one matches
        for rule in &self.rules {
            if let Ok(bindings) = self.match_pattern(&rule.pattern, &expr, usage_environment) {
                return self.substitute_template(
                    &rule.template,
                    bindings,
                    usage_environment,
                );
            }
        }
        
        Err(LambdustError::runtime_error(format!(
            "No matching rule for macro {} with {} arguments. Expansion path: {}",
            self.macro_name,
            input.len(),
            usage_environment.expansion_context.expansion_path()
        )))
    }
    
    /// Match pattern against input with hygiene
    fn match_pattern(
        &self,
        pattern: &Pattern,
        input: &Expr,
        usage_environment: &HygienicEnvironment,
    ) -> Result<PatternBindings> {
        let mut bindings = HashMap::new();
        self.match_pattern_recursive(pattern, input, &mut bindings, usage_environment)?;
        Ok(bindings)
    }
    
    /// Recursive pattern matching
    fn match_pattern_recursive(
        &self,
        pattern: &Pattern,
        expr: &Expr,
        bindings: &mut PatternBindings,
        _usage_environment: &HygienicEnvironment,
    ) -> Result<()> {
        match pattern {
            Pattern::Literal(literal) => {
                self.match_literal(literal, expr)
            }
            Pattern::Variable(var) => {
                bindings.insert(var.clone(), expr.clone());
                Ok(())
            }
            Pattern::List(patterns) => {
                self.match_list_pattern(patterns, expr, bindings, _usage_environment)
            }
            Pattern::Ellipsis(sub_pattern) => {
                self.match_ellipsis_pattern(sub_pattern, expr, bindings, _usage_environment)
            }
            Pattern::NestedEllipsis(sub_pattern, _level) => {
                // For now, treat as regular ellipsis
                self.match_ellipsis_pattern(sub_pattern, expr, bindings, _usage_environment)
            }
            Pattern::Dotted(_, _) | Pattern::Vector(_) => {
                // TODO: Implement dotted and vector pattern support
                Err(LambdustError::runtime_error("Dotted and vector patterns not yet supported in hygienic transformer".to_string()))
            }
            Pattern::HygienicVariable(_) => {
                // Hygienic variables always match
                Ok(())
            }
            Pattern::SyntaxObject(inner_pattern) => {
                // Match against the inner pattern
                self.match_pattern_recursive(inner_pattern, expr, bindings, _usage_environment)
            }
            Pattern::Any => {
                // Any pattern always matches
                Ok(())
            }
            // Advanced patterns not yet implemented in hygienic transformer
            Pattern::Conditional { .. } | Pattern::TypeGuard { .. } | Pattern::And(_)
            | Pattern::Or(_) | Pattern::Not(_) | Pattern::Range { .. } | Pattern::Regex(_) => {
                Err(LambdustError::runtime_error(
                    "Advanced patterns not yet supported in hygienic transformer".to_string(),
                ))
            }
        }
    }
    
    /// Match literal pattern
    fn match_literal(&self, literal: &str, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Variable(name) | Expr::HygienicVariable(HygienicSymbol { name, .. }) => {
                if name == literal {
                    Ok(())
                } else {
                    Err(LambdustError::runtime_error(format!(
                        "Expected literal '{}', got '{}'",
                        literal, name
                    )))
                }
            }
            _ => Err(LambdustError::runtime_error(format!(
                "Expected literal '{}', got non-symbol",
                literal
            ))),
        }
    }
    
    /// Match list pattern
    fn match_list_pattern(
        &self,
        patterns: &[Pattern],
        expr: &Expr,
        bindings: &mut PatternBindings,
        usage_environment: &HygienicEnvironment,
    ) -> Result<()> {
        match expr {
            Expr::List(exprs) => {
                if patterns.len() != exprs.len() {
                    return Err(LambdustError::runtime_error(format!(
                        "Pattern length {} doesn't match expression length {}",
                        patterns.len(),
                        exprs.len()
                    )));
                }
                
                for (pattern, expr) in patterns.iter().zip(exprs.iter()) {
                    self.match_pattern_recursive(pattern, expr, bindings, usage_environment)?;
                }
                Ok(())
            }
            _ => Err(LambdustError::runtime_error(
                "Expected list for list pattern".to_string(),
            )),
        }
    }
    
    /// Match ellipsis pattern
    fn match_ellipsis_pattern(
        &self,
        sub_pattern: &Pattern,
        expr: &Expr,
        bindings: &mut PatternBindings,
        usage_environment: &HygienicEnvironment,
    ) -> Result<()> {
        match expr {
            Expr::List(exprs) => {
                // Match each expression against the sub-pattern
                for expr in exprs {
                    self.match_pattern_recursive(sub_pattern, expr, bindings, usage_environment)?;
                }
                Ok(())
            }
            _ => {
                // Single expression, try to match against sub-pattern
                self.match_pattern_recursive(sub_pattern, expr, bindings, usage_environment)
            }
        }
    }
    
    /// Substitute template with hygienic renaming
    fn substitute_template(
        &self,
        template: &Template,
        bindings: PatternBindings,
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // Create expansion context
        let mut expansion_context = usage_environment
            .expansion_context
            .enter_macro(self.macro_name.clone())
            .map_err(|e| LambdustError::runtime_error(e))?;
        
        // Add pattern bindings to context
        for (name, expr) in &bindings {
            if let Some(symbol) = self.extract_symbol_from_binding(expr) {
                expansion_context.bind_symbol(name.clone(), symbol);
            }
        }
        
        // Substitute template
        let result = self.substitute_template_recursive(
            template,
            &bindings,
            &mut expansion_context,
            usage_environment,
        )?;
        
        // Apply hygienic renaming
        let renamer = SymbolRenamer::new(self.renaming_strategy.clone());
        renamer.rename_symbols(&result, &mut expansion_context, usage_environment)
    }
    
    /// Recursive template substitution
    fn substitute_template_recursive(
        &self,
        template: &Template,
        bindings: &PatternBindings,
        expansion_context: &mut ExpansionContext,
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        match template {
            Template::Literal(literal) => {
                // Check if literal should be treated hygienically
                if self.literals.contains(literal) {
                    // Literal identifiers maintain their identity
                    Ok(Expr::Variable(literal.clone()))
                } else {
                    // Other literals might be hygienic symbols from definition site
                    self.resolve_template_symbol(literal, expansion_context, usage_environment)
                }
            }
            Template::Variable(var) => {
                if let Some(bound_expr) = bindings.get(var) {
                    Ok(bound_expr.clone())
                } else {
                    // Unbound template variable - resolve from definition environment
                    self.resolve_template_symbol(var, expansion_context, usage_environment)
                }
            }
            Template::List(templates) => {
                let substituted: Result<Vec<_>> = templates
                    .iter()
                    .map(|t| self.substitute_template_recursive(t, bindings, expansion_context, usage_environment))
                    .collect();
                Ok(Expr::List(substituted?))
            }
            Template::Ellipsis(sub_template) => {
                // Handle ellipsis expansion
                self.expand_ellipsis_template(sub_template, bindings, expansion_context, usage_environment)
            }
            Template::NestedEllipsis(sub_template, _level) => {
                // For now, treat as regular ellipsis
                self.expand_ellipsis_template(sub_template, bindings, expansion_context, usage_environment)
            }
            Template::Dotted(_, _) | Template::Vector(_) => {
                // TODO: Implement dotted and vector template support
                Err(LambdustError::runtime_error("Dotted and vector templates not yet supported in hygienic transformer".to_string()))
            }
            Template::HygienicVariable(symbol) => {
                // Generate fresh hygienic identifier
                let fresh_name = expansion_context.generate_template_symbol(&symbol.original_name());
                Ok(Expr::Variable(fresh_name.unique_name()))
            }
            Template::SyntaxObject(inner_template) => {
                // Process inner template as syntax object
                self.substitute_template_recursive(inner_template, bindings, expansion_context, usage_environment)
            }
            // Advanced templates not yet implemented in hygienic transformer
            Template::Conditional { .. } | Template::Repeat { .. } | Template::Transform { .. } => {
                Err(LambdustError::runtime_error(
                    "Advanced templates not yet supported in hygienic transformer".to_string(),
                ))
            }
        }
    }
    
    /// Resolve symbol from template
    fn resolve_template_symbol(
        &self,
        name: &str,
        expansion_context: &mut ExpansionContext,
        _usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // First check if we have a hygienic binding in expansion context
        if let Some(symbol) = expansion_context.lookup_symbol(name) {
            return Ok(Expr::HygienicVariable(symbol.clone()));
        }
        
        // Check definition environment for symbol
        match self.definition_environment.resolve_symbol(name) {
            SymbolResolution::Hygienic(symbol) => {
                Ok(Expr::HygienicVariable(symbol))
            }
            SymbolResolution::Traditional(_) => {
                // Use traditional variable but mark for potential renaming
                Ok(Expr::Variable(name.to_string()))
            }
            SymbolResolution::Unbound(_) => {
                // Generate new hygienic symbol for introduced variable
                let symbol = expansion_context.generate_template_symbol(name);
                Ok(Expr::HygienicVariable(symbol))
            }
        }
    }
    
    /// Expand ellipsis template
    fn expand_ellipsis_template(
        &self,
        sub_template: &Template,
        bindings: &PatternBindings,
        expansion_context: &mut ExpansionContext,
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // For simplicity, expand once for now
        // Real implementation would need to handle multiple ellipsis variables
        let expanded = self.substitute_template_recursive(
            sub_template,
            bindings,
            expansion_context,
            usage_environment,
        )?;
        Ok(expanded)
    }
    
    /// Extract hygienic symbol from pattern binding
    fn extract_symbol_from_binding(&self, expr: &Expr) -> Option<HygienicSymbol> {
        match expr {
            Expr::HygienicVariable(symbol) => Some(symbol.clone()),
            Expr::Variable(name) => {
                // Convert traditional variable to hygienic symbol
                let env_id = self.definition_environment.id;
                let macro_site = MacroSite::new(
                    self.macro_name.clone(),
                    0,
                    env_id,
                );
                let symbol_id = super::generator::SymbolGenerator::generate_symbol_id();
                Some(HygienicSymbol::new(name.clone(), symbol_id, macro_site))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::hygiene::generator::SymbolGenerator;

    fn create_test_environment() -> Rc<HygienicEnvironment> {
        Rc::new(HygienicEnvironment::new())
    }

    #[test]
    fn test_hygienic_transformer_creation() {
        let env = create_test_environment();
        let transformer = HygienicSyntaxRulesTransformer::new(
            vec![],
            vec![],
            env,
            "test-macro".to_string(),
        );
        
        assert_eq!(transformer.macro_name, "test-macro");
        assert!(transformer.literals.is_empty());
        assert!(transformer.rules.is_empty());
    }
    
    #[test]
    fn test_literal_matching() {
        let env = create_test_environment();
        let transformer = HygienicSyntaxRulesTransformer::new(
            vec!["else".to_string()],
            vec![],
            env,
            "test-macro".to_string(),
        );
        
        let literal_expr = Expr::Variable("else".to_string());
        assert!(transformer.match_literal("else", &literal_expr).is_ok());
        
        let wrong_expr = Expr::Variable("other".to_string());
        assert!(transformer.match_literal("else", &wrong_expr).is_err());
    }
    
    #[test]
    fn test_pattern_variable_binding() {
        let env = create_test_environment();
        let transformer = HygienicSyntaxRulesTransformer::new(
            vec![],
            vec![],
            env.clone(),
            "test-macro".to_string(),
        );
        
        let mut bindings = HashMap::new();
        let test_expr = Expr::Variable("test-value".to_string());
        
        let result = transformer.match_pattern_recursive(
            &Pattern::Variable("x".to_string()),
            &test_expr,
            &mut bindings,
            &env,
        );
        
        assert!(result.is_ok());
        assert_eq!(bindings.get("x"), Some(&test_expr));
    }
    
    #[test]
    fn test_list_pattern_matching() {
        let env = create_test_environment();
        let transformer = HygienicSyntaxRulesTransformer::new(
            vec![],
            vec![],
            env.clone(),
            "test-macro".to_string(),
        );
        
        let pattern = Pattern::List(vec![
            Pattern::Literal("if".to_string()),
            Pattern::Variable("test".to_string()),
            Pattern::Variable("then".to_string()),
        ]);
        
        let expr = Expr::List(vec![
            Expr::Variable("if".to_string()),
            Expr::Variable("condition".to_string()),
            Expr::Variable("body".to_string()),
        ]);
        
        let mut bindings = HashMap::new();
        let result = transformer.match_pattern_recursive(
            &pattern,
            &expr,
            &mut bindings,
            &env,
        );
        
        assert!(result.is_ok());
        assert_eq!(bindings.len(), 2);
        assert!(bindings.contains_key("test"));
        assert!(bindings.contains_key("then"));
    }
}