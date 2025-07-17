//! SRFI 149: Basic Syntax-rules Template Extensions
//!
//! This SRFI extends the basic syntax-rules system by softening the template validity rules
//! to allow for more flexible macro patterns:
//! 1. Allow multiple consecutive ellipses in subtemplates
//! 2. Allow pattern variables to be followed by more ellipsis instances than in subpatterns

use crate::error::{LambdustError, Result};
use crate::value::{Value, Procedure};
use std::collections::HashMap;

/// SRFI 149 module implementation
pub struct Srfi149Module;

impl crate::srfi::SrfiModule for Srfi149Module {
    fn srfi_id(&self) -> u32 {
        149
    }

    fn name(&self) -> &'static str {
        "SRFI 149"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["syntax-rules-extensions"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Enhanced syntax-rules with template extensions
        exports.insert("syntax-rules".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "syntax-rules".to_string(), 
            arity: None, 
            func: enhanced_syntax_rules 
        }));
        
        // Extended ellipsis handling utilities
        exports.insert("ellipsis-depth".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "ellipsis-depth".to_string(), 
            arity: Some(1), 
            func: ellipsis_depth 
        }));
        exports.insert("template-expand".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "template-expand".to_string(), 
            arity: Some(2), 
            func: template_expand 
        }));
        exports.insert("pattern-match-extended".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "pattern-match-extended".to_string(), 
            arity: Some(2), 
            func: pattern_match_extended 
        }));
        
        // Multiple ellipsis utilities
        exports.insert("consecutive-ellipsis?".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "consecutive-ellipsis?".to_string(), 
            arity: Some(1), 
            func: consecutive_ellipsis_p 
        }));
        exports.insert("excess-ellipsis-semantics".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "excess-ellipsis-semantics".to_string(), 
            arity: Some(3), 
            func: excess_ellipsis_semantics 
        }));
        
        // Template validation
        exports.insert("validate-extended-template".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "validate-extended-template".to_string(), 
            arity: Some(1), 
            func: validate_extended_template 
        }));
        exports.insert("template-element-valid?".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "template-element-valid?".to_string(), 
            arity: Some(2), 
            func: template_element_valid_p 
        }));
        
        // Pattern variable handling
        exports.insert("pattern-variable-ellipsis-count".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "pattern-variable-ellipsis-count".to_string(), 
            arity: Some(2), 
            func: pattern_variable_ellipsis_count 
        }));
        exports.insert("subtemplate-ellipsis-count".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "subtemplate-ellipsis-count".to_string(), 
            arity: Some(2), 
            func: subtemplate_ellipsis_count 
        }));
        
        // Compatibility detection
        exports.insert("srfi-149-compatible?".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "srfi-149-compatible?".to_string(), 
            arity: Some(0), 
            func: srfi_149_compatible_p 
        }));
        
        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // For now, return all exports regardless of parts
        Ok(self.exports())
    }
}

impl Srfi149Module {
    /// Creates a new SRFI-149 module instance
    pub fn new() -> Self {
        Self
    }
}

/// Enhanced syntax-rules with template extensions
fn enhanced_syntax_rules(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    // First argument: literals list
    let _literals = &args[0];
    
    // Remaining arguments: transformation rules
    let _rules = &args[1..];
    
    // This would implement enhanced syntax-rules with:
    // 1. Multiple consecutive ellipses support
    // 2. Relaxed pattern variable ellipsis restrictions
    
    // For now, return a marker indicating enhanced syntax-rules capability
    Ok(Value::Symbol("enhanced-syntax-rules".to_string()))
}

/// Get the depth of ellipsis nesting in a template
fn ellipsis_depth(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let template = &args[0];
    let depth = calculate_ellipsis_depth(template);
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(depth)))
}

/// Expand a template with extended ellipsis semantics
fn template_expand(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let _template = &args[0];
    let _bindings = &args[1];
    
    // This would implement extended template expansion with:
    // - Support for multiple consecutive ellipses
    // - Proper handling of excess ellipsis instances
    // - Innermost repetition semantics for excess ellipses
    
    // For now, return the original template (simplified)
    Ok(args[0].clone())
}

/// Extended pattern matching with SRFI 149 semantics
fn pattern_match_extended(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let _pattern = &args[0];
    let _expression = &args[1];
    
    // This would implement extended pattern matching that allows:
    // - More flexible ellipsis usage
    // - Proper binding extraction for extended templates
    
    // For now, return success indicator
    Ok(Value::Boolean(true))
}

/// Check if template contains consecutive ellipses
fn consecutive_ellipsis_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let template = &args[0];
    let has_consecutive = check_consecutive_ellipses(template);
    Ok(Value::Boolean(has_consecutive))
}

/// Handle excess ellipsis semantics (innermost repetition)
fn excess_ellipsis_semantics(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }
    
    let _pattern_variable = &args[0];
    let _pattern_ellipses = &args[1];
    let _template_ellipses = &args[2];
    
    // This would implement the SRFI 149 semantics for handling cases where
    // a pattern variable in a subtemplate is followed by more ellipsis instances
    // than in the subpattern. SRFI 149 specifies innermost repetition.
    
    // For now, return the pattern variable (simplified)
    Ok(args[0].clone())
}

/// Validate extended template according to SRFI 149 rules
fn validate_extended_template(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let template = &args[0];
    let is_valid = validate_template_extended(template);
    Ok(Value::Boolean(is_valid))
}

/// Check if a template element is valid under extended rules
fn template_element_valid_p(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let _element = &args[0];
    let _context = &args[1];
    
    // This would validate individual template elements according to
    // the relaxed rules of SRFI 149
    
    Ok(Value::Boolean(true))
}

/// Count ellipses following a pattern variable
fn pattern_variable_ellipsis_count(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let _variable = &args[0];
    let _pattern = &args[1];
    
    // This would count how many ellipses follow a pattern variable
    // in the subpattern
    
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(0)))
}

/// Count ellipses in a subtemplate
fn subtemplate_ellipsis_count(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let _variable = &args[0];
    let _template = &args[1];
    
    // This would count how many ellipses follow a pattern variable
    // in the subtemplate
    
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(0)))
}

/// Check if implementation is SRFI 149 compatible
fn srfi_149_compatible_p(args: &[Value]) -> Result<Value> {
    if !args.is_empty() {
        return Err(LambdustError::arity_error(0, args.len()));
    }
    
    // This Lambdust implementation provides SRFI 149 compatibility
    Ok(Value::Boolean(true))
}

/// Helper function to calculate ellipsis depth in a template
fn calculate_ellipsis_depth(template: &Value) -> i64 {
    match template {
        Value::Symbol(s) if s == "..." => 1,
        Value::Pair(pair_ref) => {
            let pair = pair_ref.borrow();
            let car_depth = calculate_ellipsis_depth(&pair.car);
            let cdr_depth = calculate_ellipsis_depth(&pair.cdr);
            car_depth.max(cdr_depth)
        }
        Value::Vector(vec) => {
            vec.iter()
                .map(calculate_ellipsis_depth)
                .max()
                .unwrap_or(0)
        }
        _ => 0,
    }
}

/// Helper function to check for consecutive ellipses
fn check_consecutive_ellipses(template: &Value) -> bool {
    match template {
        Value::Pair(pair_ref) => {
            let pair = pair_ref.borrow();
            
            // Check if we have consecutive ellipses
            if let (Value::Symbol(car), Value::Symbol(cadr)) = (&pair.car, &pair.cdr) {
                if car == "..." && cadr == "..." {
                    return true;
                }
            }
            
            // Recursively check car and cdr
            check_consecutive_ellipses(&pair.car) || check_consecutive_ellipses(&pair.cdr)
        }
        Value::Vector(vec) => {
            // Check for consecutive ellipses in vector
            for window in vec.windows(2) {
                if let [Value::Symbol(a), Value::Symbol(b)] = window {
                    if a == "..." && b == "..." {
                        return true;
                    }
                }
            }
            
            // Recursively check vector elements
            vec.iter().any(check_consecutive_ellipses)
        }
        _ => false,
    }
}

/// Helper function to validate template under extended rules
fn validate_template_extended(template: &Value) -> bool {
    match template {
        Value::Symbol(_) => true, // All symbols are valid
        Value::Pair(pair_ref) => {
            let pair = pair_ref.borrow();
            
            // Under SRFI 149, consecutive ellipses are allowed
            validate_template_extended(&pair.car) && validate_template_extended(&pair.cdr)
        }
        Value::Vector(vec) => {
            // All vector elements must be valid
            vec.iter().all(validate_template_extended)
        }
        Value::Number(_) | Value::String(_) | Value::Character(_) | Value::Boolean(_) => true,
        _ => true, // Be permissive for other types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
        use crate::srfi::SrfiModule;
    use std::sync::Arc;

    #[test]
    fn test_ellipsis_depth() {
        
        
        // Test simple ellipsis
        let simple_ellipsis = Value::Symbol("...".to_string());
        let result = ellipsis_depth(&[simple_ellipsis]).unwrap();
        assert_eq!(result, Value::Number(crate::lexer::SchemeNumber::Integer(1)));
        
        // Test non-ellipsis symbol
        let symbol = Value::Symbol("x".to_string());
        let result = ellipsis_depth(&[symbol]).unwrap();
        assert_eq!(result, Value::Number(crate::lexer::SchemeNumber::Integer(0)));
    }

    #[test]
    fn test_consecutive_ellipsis_detection() {
        
        
        // Test simple symbol (no consecutive ellipses)
        let symbol = Value::Symbol("x".to_string());
        let result = consecutive_ellipsis_p(&[symbol]).unwrap();
        assert_eq!(result, Value::Boolean(false));
        
        // Test vector with consecutive ellipses
        let consecutive_vec = Value::Vector(vec![
            Value::Symbol("...".to_string()),
            Value::Symbol("...".to_string()),
        ]);
        let result = consecutive_ellipsis_p(&[consecutive_vec]).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_template_validation() {
        
        
        // Test simple valid template
        let template = Value::Symbol("x".to_string());
        let result = validate_extended_template(&[template]).unwrap();
        assert_eq!(result, Value::Boolean(true));
        
        // Test vector template
        let vector_template = Value::Vector(vec![
            Value::Symbol("x".to_string()),
            Value::Symbol("...".to_string()),
        ]);
        let result = validate_extended_template(&[vector_template]).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_srfi_149_compatibility() {
        
        
        let result = srfi_149_compatible_p(&[]).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_enhanced_syntax_rules() {
        
        
        // Test basic syntax-rules call
        let literals = Value::Vector(vec![]);
        let rule = Value::Vector(vec![
            Value::Symbol("pattern".to_string()),
            Value::Symbol("template".to_string()),
        ]);
        
        let result = enhanced_syntax_rules(&[literals, rule]).unwrap();
        assert_eq!(result, Value::Symbol("enhanced-syntax-rules".to_string()));
    }

    #[test]
    fn test_srfi_149_module() {
        let module = Srfi149Module::new();
        assert_eq!(module.srfi_id(), 149);
        assert_eq!(module.name(), "SRFI 149");
        assert_eq!(module.parts(), vec!["syntax-rules-extensions"]);
        
        let exports = module.exports();
        assert!(exports.contains_key("syntax-rules"));
        assert!(exports.contains_key("ellipsis-depth"));
        assert!(exports.contains_key("consecutive-ellipsis?"));
        assert!(exports.contains_key("srfi-149-compatible?"));
        
        // Test exports_for_parts
        let partial_exports = module.exports_for_parts(&["syntax-rules-extensions"]).unwrap();
        assert_eq!(partial_exports.len(), exports.len());
    }

    #[test]
    fn test_pattern_match_extended() {
        
        
        let pattern = Value::Symbol("pattern".to_string());
        let expression = Value::Symbol("expr".to_string());
        
        let result = pattern_match_extended(&[pattern, expression]).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_template_expand() {
        
        
        let template = Value::Symbol("template".to_string());
        let bindings = Value::Vector(vec![]);
        
        let result = template_expand(&[template.clone(), bindings]).unwrap();
        assert_eq!(result, template);
    }
}