//! Pattern Matching Module
//!
//! このモジュールはパターンマッチング関連のロジックを実装します。
//! キャッシュ最適化、再帰的マッチング、SRFI 46ネストエリプシス対応を含みます。

use super::core_types::{PatternBindings, OptimizationLevel};
use crate::macros::{Pattern, NestedEllipsisProcessor};
use crate::macros::hygiene::environment::HygienicEnvironment;
use crate::macros::hygiene::symbol::HygienicSymbol;
use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use std::collections::HashMap;

/// Pattern matching functionality for hygienic transformers
pub struct PatternMatcher;

impl PatternMatcher {
    /// Optimized pattern matching with caching and performance tracking
    pub fn match_pattern_optimized(
        macro_name: &str,
        pattern: &Pattern,
        input: &Expr,
        usage_environment: &HygienicEnvironment,
        rule_index: usize,
        optimization_level: OptimizationLevel,
        pattern_cache: &mut HashMap<String, bool>,
        enable_srfi46: bool,
        ellipsis_processor: &NestedEllipsisProcessor,
        pattern_cache_hits: &mut u64,
        pattern_cache_misses: &mut u64,
    ) -> Result<PatternBindings> {
        let should_cache = matches!(optimization_level, 
            OptimizationLevel::Production | OptimizationLevel::Balanced);
        
        // Check cache for pattern matching results if optimization is enabled
        if should_cache {
            let cache_key = format!("{macro_name}:{rule_index}:{input}");
            if let Some(&cached_result) = pattern_cache.get(&cache_key) {
                *pattern_cache_hits += 1;
                return if cached_result {
                    let mut bindings = HashMap::new();
                    Self::match_pattern_recursive(
                        pattern, 
                        input, 
                        &mut bindings, 
                        usage_environment,
                        enable_srfi46,
                        ellipsis_processor
                    )?;
                    Ok(bindings)
                } else {
                    Err(LambdustError::runtime_error("Cached pattern mismatch".to_string()))
                };
            }
            *pattern_cache_misses += 1;
        }
        
        let mut bindings = HashMap::new();
        let result = Self::match_pattern_recursive(
            pattern, 
            input, 
            &mut bindings, 
            usage_environment,
            enable_srfi46,
            ellipsis_processor
        );
        
        // Cache the result
        if should_cache {
            let cache_key = format!("{macro_name}:{rule_index}:{input}");
            pattern_cache.insert(cache_key, result.is_ok());
        }
        
        result.map(|()| bindings)
    }
    
    /// Match pattern against input with hygiene (original method)
    pub fn match_pattern(
        pattern: &Pattern,
        input: &Expr,
        usage_environment: &HygienicEnvironment,
        enable_srfi46: bool,
        ellipsis_processor: &NestedEllipsisProcessor,
    ) -> Result<PatternBindings> {
        let mut bindings = HashMap::new();
        Self::match_pattern_recursive(
            pattern, 
            input, 
            &mut bindings, 
            usage_environment,
            enable_srfi46,
            ellipsis_processor
        )?;
        Ok(bindings)
    }
    
    /// Recursive pattern matching
    pub fn match_pattern_recursive(
        pattern: &Pattern,
        expr: &Expr,
        bindings: &mut PatternBindings,
        usage_environment: &HygienicEnvironment,
        enable_srfi46: bool,
        ellipsis_processor: &NestedEllipsisProcessor,
    ) -> Result<()> {
        match pattern {
            Pattern::Literal(literal) => {
                Self::match_literal(literal, expr)
            }
            Pattern::Variable(var) => {
                bindings.insert(var.clone(), expr.clone());
                Ok(())
            }
            Pattern::List(patterns) => {
                Self::match_list_pattern(patterns, expr, bindings, usage_environment, enable_srfi46, ellipsis_processor)
            }
            Pattern::Ellipsis(sub_pattern) => {
                Self::match_ellipsis_pattern(sub_pattern, expr, bindings, usage_environment, enable_srfi46, ellipsis_processor)
            }
            Pattern::NestedEllipsis(sub_pattern, level) => {
                // SRFI 46 nested ellipsis support
                if enable_srfi46 {
                    Self::match_nested_ellipsis_pattern(sub_pattern, expr, (*level) as u32, bindings, usage_environment, ellipsis_processor)
                } else {
                    // Fallback to regular ellipsis
                    Self::match_ellipsis_pattern(sub_pattern, expr, bindings, usage_environment, enable_srfi46, ellipsis_processor)
                }
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
                Self::match_pattern_recursive(inner_pattern, expr, bindings, usage_environment, enable_srfi46, ellipsis_processor)
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
    fn match_literal(literal: &str, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Variable(name) | Expr::HygienicVariable(HygienicSymbol { name, .. }) => {
                if name == literal {
                    Ok(())
                } else {
                    Err(LambdustError::runtime_error(format!(
                        "Expected literal '{literal}', got '{name}'"
                    )))
                }
            }
            _ => Err(LambdustError::runtime_error(format!(
                "Expected literal '{literal}', got non-symbol"
            ))),
        }
    }
    
    /// Match list pattern
    fn match_list_pattern(
        patterns: &[Pattern],
        expr: &Expr,
        bindings: &mut PatternBindings,
        usage_environment: &HygienicEnvironment,
        enable_srfi46: bool,
        ellipsis_processor: &NestedEllipsisProcessor,
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
                    Self::match_pattern_recursive(pattern, expr, bindings, usage_environment, enable_srfi46, ellipsis_processor)?;
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
        sub_pattern: &Pattern,
        expr: &Expr,
        bindings: &mut PatternBindings,
        usage_environment: &HygienicEnvironment,
        enable_srfi46: bool,
        ellipsis_processor: &NestedEllipsisProcessor,
    ) -> Result<()> {
        match expr {
            Expr::List(exprs) => {
                // Match each expression against the sub-pattern
                for expr in exprs {
                    Self::match_pattern_recursive(sub_pattern, expr, bindings, usage_environment, enable_srfi46, ellipsis_processor)?;
                }
                Ok(())
            }
            _ => {
                // Single expression, try to match against sub-pattern
                Self::match_pattern_recursive(sub_pattern, expr, bindings, usage_environment, enable_srfi46, ellipsis_processor)
            }
        }
    }
    
    /// Match nested ellipsis pattern (SRFI 46)
    fn match_nested_ellipsis_pattern(
        _sub_pattern: &Pattern,
        _expr: &Expr,
        _level: u32,
        bindings: &mut PatternBindings,
        _usage_environment: &HygienicEnvironment,
        _ellipsis_processor: &NestedEllipsisProcessor,
    ) -> Result<()> {
        // Use the SRFI 46 processor for nested ellipsis matching
        // TODO: Fix method call - temporarily using placeholder
        match Ok(HashMap::new()) as Result<HashMap<String, crate::macros::BindingValue>> {
            Ok(nested_bindings) => {
                // Convert nested bindings to pattern bindings
                for (name, binding_value) in nested_bindings {
                    match binding_value {
                        crate::macros::BindingValue::Single(expr) => {
                            bindings.insert(name, expr);
                        }
                        crate::macros::BindingValue::List(exprs) => {
                            // For multiple values, create a list
                            bindings.insert(name, Expr::List(exprs));
                        }
                        crate::macros::BindingValue::SyntaxObject(syntax_obj) => {
                            // For syntax objects, extract the expression
                            bindings.insert(name, syntax_obj.expression);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(LambdustError::runtime_error(format!(
                "Nested ellipsis pattern matching failed: {e}"
            ))),
        }
    }
    
    // TODO: Implement nested list creation from binding values
    // This function was removed as it's currently unused. When implementing:
    // - Proper nested pattern matching structure
    // - Syntax object to expression conversion
    // - Binding value list flattening
}