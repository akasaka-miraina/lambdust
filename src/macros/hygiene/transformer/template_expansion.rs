//! Template Expansion Module
//!
//! このモジュールはテンプレート展開関連のロジックを実装します。
//! 衛生的シンボル解決、エリプシス展開、SRFI 46対応を含みます。

use super::core_types::PatternBindings;
use crate::macros::{Template, NestedEllipsisProcessor, BindingValue};
use crate::macros::hygiene::environment::{HygienicEnvironment, SymbolResolution};
use crate::macros::hygiene::context::ExpansionContext;
use crate::macros::hygiene::symbol::HygienicSymbol;
use crate::macros::hygiene::renaming::SymbolRenamer;
use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use std::rc::Rc;

/// Template expansion functionality for hygienic transformers
pub struct TemplateExpander;

impl TemplateExpander {
    /// Optimized template substitution with advanced symbol renaming
    pub fn substitute_template_optimized(
        template: &Template,
        bindings: PatternBindings,
        usage_environment: &HygienicEnvironment,
        definition_environment: &Rc<HygienicEnvironment>,
        literals: &[String],
        symbol_renamer: &mut SymbolRenamer,
        enable_srfi46: bool,
        ellipsis_processor: &NestedEllipsisProcessor,
        symbol_renamings: &mut u64,
    ) -> Result<Expr> {
        // Create expansion context
        let mut expansion_context = ExpansionContext::new(
            (**definition_environment).clone(),
            usage_environment.clone()
        );
        
        // Add pattern bindings to context
        for (name, expr) in &bindings {
            if let Some(symbol) = Self::extract_symbol_from_binding(expr) {
                expansion_context.bind_symbol(name.clone(), symbol);
            }
        }
        
        // Substitute template with optimization tracking
        let result = Self::substitute_template_recursive(
            template,
            &bindings,
            &mut expansion_context,
            usage_environment,
            literals,
            enable_srfi46,
            ellipsis_processor,
        )?;
        
        // Apply advanced hygienic renaming with integrated symbol renamer
        let renamed_result = symbol_renamer.rename_symbols(
            &result, 
            &mut expansion_context, 
            usage_environment
        )?;
        
        // Update symbol renaming metrics
        let renamer_stats = symbol_renamer.performance_stats();
        *symbol_renamings += renamer_stats.symbols_renamed;
        
        Ok(renamed_result)
    }
    
    /// Substitute template with hygienic renaming (original method)
    pub fn substitute_template(
        template: &Template,
        bindings: PatternBindings,
        usage_environment: &HygienicEnvironment,
        definition_environment: &Rc<HygienicEnvironment>,
        literals: &[String],
        symbol_renamer: &SymbolRenamer,
        enable_srfi46: bool,
        ellipsis_processor: &NestedEllipsisProcessor,
    ) -> Result<Expr> {
        // Create expansion context
        let mut expansion_context = ExpansionContext::new(
            (**definition_environment).clone(),
            usage_environment.clone()
        );
        
        // Add pattern bindings to context
        for (name, expr) in &bindings {
            if let Some(symbol) = Self::extract_symbol_from_binding(expr) {
                expansion_context.bind_symbol(name.clone(), symbol);
            }
        }
        
        // Substitute template
        let result = Self::substitute_template_recursive(
            template,
            &bindings,
            &mut expansion_context,
            usage_environment,
            literals,
            enable_srfi46,
            ellipsis_processor,
        )?;
        
        // Apply hygienic renaming using cloned renamer
        let mut temp_renamer = symbol_renamer.clone();
        temp_renamer.rename_symbols(&result, &mut expansion_context, usage_environment)
    }
    
    /// Recursive template substitution
    fn substitute_template_recursive(
        template: &Template,
        bindings: &PatternBindings,
        expansion_context: &mut ExpansionContext,
        usage_environment: &HygienicEnvironment,
        literals: &[String],
        enable_srfi46: bool,
        ellipsis_processor: &NestedEllipsisProcessor,
    ) -> Result<Expr> {
        match template {
            Template::Literal(literal) => {
                // Check if literal should be treated hygienically
                if literals.contains(literal) {
                    // Literal identifiers maintain their identity
                    Ok(Expr::Variable(literal.clone()))
                } else {
                    // Other literals might be hygienic symbols from definition site
                    Self::resolve_template_symbol(literal, expansion_context, usage_environment)
                }
            }
            Template::Variable(var) => {
                if let Some(bound_expr) = bindings.get(var) {
                    Ok(bound_expr.clone())
                } else {
                    // Unbound template variable - resolve from definition environment
                    Self::resolve_template_symbol(var, expansion_context, usage_environment)
                }
            }
            Template::List(templates) => {
                let substituted: Result<Vec<_>> = templates
                    .iter()
                    .map(|t| Self::substitute_template_recursive(
                        t, 
                        bindings, 
                        expansion_context, 
                        usage_environment,
                        literals,
                        enable_srfi46,
                        ellipsis_processor,
                    ))
                    .collect();
                Ok(Expr::List(substituted?))
            }
            Template::Ellipsis(sub_template) => {
                // Handle ellipsis expansion
                Self::expand_ellipsis_template(
                    sub_template, 
                    bindings, 
                    expansion_context, 
                    usage_environment,
                    literals,
                    enable_srfi46,
                    ellipsis_processor,
                )
            }
            Template::NestedEllipsis(sub_template, level) => {
                // SRFI 46 nested ellipsis support
                if enable_srfi46 {
                    Self::expand_nested_ellipsis_template(
                        sub_template, 
                        bindings, 
                        (*level) as u32, 
                        expansion_context, 
                        usage_environment,
                        literals,
                        ellipsis_processor,
                    )
                } else {
                    // Fallback to regular ellipsis
                    Self::expand_ellipsis_template(
                        sub_template, 
                        bindings, 
                        expansion_context, 
                        usage_environment,
                        literals,
                        enable_srfi46,
                        ellipsis_processor,
                    )
                }
            }
            Template::Dotted(_, _) | Template::Vector(_) => {
                // TODO: Implement dotted and vector template support
                Err(LambdustError::runtime_error("Dotted and vector templates not yet supported in hygienic transformer".to_string()))
            }
            Template::HygienicVariable(symbol) => {
                // Generate fresh hygienic identifier
                let fresh_name = expansion_context.generate_template_symbol(symbol.original_name());
                Ok(Expr::Variable(fresh_name.unique_name()))
            }
            Template::SyntaxObject(inner_template) => {
                // Process inner template as syntax object
                Self::substitute_template_recursive(
                    inner_template, 
                    bindings, 
                    expansion_context, 
                    usage_environment,
                    literals,
                    enable_srfi46,
                    ellipsis_processor,
                )
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
        name: &str,
        expansion_context: &mut ExpansionContext,
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // First check if we have a hygienic binding in expansion context
        if let Some(symbol) = expansion_context.lookup_symbol(name) {
            return Ok(Expr::HygienicVariable(symbol.clone()));
        }
        
        // Check definition environment for symbol
        match usage_environment.resolve_symbol(name) {
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
        sub_template: &Template,
        bindings: &PatternBindings,
        expansion_context: &mut ExpansionContext,
        usage_environment: &HygienicEnvironment,
        literals: &[String],
        enable_srfi46: bool,
        ellipsis_processor: &NestedEllipsisProcessor,
    ) -> Result<Expr> {
        // For simplicity, expand once for now
        // Real implementation would need to handle multiple ellipsis variables
        let expanded = Self::substitute_template_recursive(
            sub_template,
            bindings,
            expansion_context,
            usage_environment,
            literals,
            enable_srfi46,
            ellipsis_processor,
        )?;
        Ok(expanded)
    }
    
    /// Expand nested ellipsis template (SRFI 46)
    fn expand_nested_ellipsis_template(
        _sub_template: &Template,
        bindings: &PatternBindings,
        _level: u32,
        _expansion_context: &mut ExpansionContext,
        _usage_environment: &HygienicEnvironment,
        _literals: &[String],
        _ellipsis_processor: &NestedEllipsisProcessor,
    ) -> Result<Expr> {
        // Use the SRFI 46 processor for nested ellipsis expansion
        let _nested_bindings = Self::convert_pattern_bindings_to_ellipsis(bindings);
        
        // TODO: Fix method call - temporarily using placeholder
        match Ok(Vec::new()) as Result<Vec<Expr>> {
            Ok(expanded_expr) => {
                // If we got a vector, take the first element
                if let Some(first_expr) = expanded_expr.first() {
                    Ok(first_expr.clone())
                } else {
                    Err(crate::error::LambdustError::syntax_error("Empty expansion result".to_string()))
                }
            }
            Err(e) => Err(LambdustError::runtime_error(format!(
                "Nested ellipsis template expansion failed: {e}"
            ))),
        }
    }
    
    /// Convert pattern bindings to ellipsis bindings for SRFI 46
    fn convert_pattern_bindings_to_ellipsis(bindings: &PatternBindings) -> std::collections::HashMap<String, BindingValue> {
        bindings.iter().map(|(name, expr)| {
            let binding_value = match expr {
                Expr::List(exprs) => {
                    if exprs.len() == 1 {
                        BindingValue::Single(exprs[0].clone())
                    } else {
                        BindingValue::List(exprs.clone())
                    }
                }
                _ => BindingValue::Single(expr.clone()),
            };
            (name.clone(), binding_value)
        }).collect()
    }
    
    /// Extract symbol from binding expression
    fn extract_symbol_from_binding(expr: &Expr) -> Option<HygienicSymbol> {
        match expr {
            Expr::HygienicVariable(symbol) => Some(symbol.clone()),
            Expr::Variable(name) => {
                // Create a simple hygienic symbol from a regular variable
                Some(HygienicSymbol::new(name.clone(), crate::macros::hygiene::symbol::SymbolId(0), crate::macros::hygiene::symbol::MacroSite::new("template".to_string(), 0, crate::macros::hygiene::symbol::EnvironmentId(0))))
            }
            _ => None,
        }
    }
}