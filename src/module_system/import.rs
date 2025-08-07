//! Import functionality for bringing modules into scope.
//!
//! Handles various import patterns:
//! - (import (lambdust string)) - Import all exports
//! - (import (lambdust string) (only string-length string-ref)) - Import specific symbols
//! - (import (lambdust string) (except string-fill!)) - Import all except specific symbols  
//! - (import (lambdust string) (rename (string-length str-len))) - Import with renaming
//! - (import (lambdust string) (prefix string:)) - Import with prefix

use super::{ImportSpec, ImportConfig, ModuleError};
use crate::diagnostics::{Error, Result, Spanned};
use crate::ast::Expr;
use crate::eval::Value;
use std::collections::HashMap;

/// Applies import configuration to module exports to get final bindings.
pub fn apply_import_config(
    exports: &HashMap<String, Value>,
    config: &ImportConfig,
) -> Result<HashMap<String, Value>> {
    match config {
        ImportConfig::All => Ok(exports.clone()),
        ImportConfig::Only(symbols) => apply_only_import(exports, symbols),
        ImportConfig::Except(symbols) => apply_except_import(exports, symbols),
        ImportConfig::Rename(rename_map) => apply_rename_import(exports, rename_map),
        ImportConfig::Prefix(prefix) => apply_prefix_import(exports, prefix),
    }
}

/// Imports only the specified symbols.
fn apply_only_import(
    exports: &HashMap<String, Value>,
    symbols: &[String],
) -> Result<HashMap<String, Value>> {
    let mut result = HashMap::new();
    
    for symbol in symbols {
        if let Some(value) = exports.get(symbol) {
            result.insert(symbol.clone()), value.clone());
        } else {
            return Err(Box::new(Error::from(ModuleError::ImportConflict(
                format!("Symbol '{}' not found in module exports", symbol)
            )));
        }
    }
    
    Ok(result)
}

/// Imports all symbols except the specified ones.
fn apply_except_import(
    exports: &HashMap<String, Value>,
    except_symbols: &[String],
) -> Result<HashMap<String, Value>> {
    let mut result = HashMap::new();
    
    for (symbol, value) in exports {
        if !except_symbols.contains(symbol) {
            result.insert(symbol.clone()), value.clone());
        }
    }
    
    Ok(result)
}

/// Imports symbols with renaming.
fn apply_rename_import(
    exports: &HashMap<String, Value>,
    rename_map: &HashMap<String, String>,
) -> Result<HashMap<String, Value>> {
    let mut result = HashMap::new();
    
    for (original_name, new_name) in rename_map {
        if let Some(value) = exports.get(original_name) {
            result.insert(new_name.clone()), value.clone());
        } else {
            return Err(Box::new(Error::from(ModuleError::ImportConflict(
                format!("Symbol '{}' not found in module exports", original_name)
            )));
        }
    }
    
    Ok(result)
}

/// Imports all symbols with a prefix.
fn apply_prefix_import(
    exports: &HashMap<String, Value>,
    prefix: &str,
) -> Result<HashMap<String, Value>> {
    let mut result = HashMap::new();
    
    for (symbol, value) in exports {
        let prefixed_name = format!("{}{}", prefix, symbol);
        result.insert(prefixed_name, value.clone());
    }
    
    Ok(result)
}

/// Parses import specifications from Scheme syntax.
pub fn parse_import_spec(import_form: &[Spanned<Expr>]) -> Result<ImportSpec> {
    if import_form.is_empty() {
        return Err(Box::new(Error::syntax_error(
            "Empty import specification".to_string(),
            None,
        ));
    }

    // First element should be the module identifier
    let module_name = extract_module_name(&import_form[0])?;
    let module_id = super::name::parse_module_name(&module_name)?;
    
    // Parse import configuration from remaining elements
    let config = if import_form.len() == 1 {
        ImportConfig::All
    } else {
        parse_import_config(&import_form[1..])?
    };

    Ok(ImportSpec {
        module_id,
        config,
    })
}

/// Extracts module name from an expression.
fn extract_module_name(expr: &Spanned<Expr>) -> Result<String> {
    use crate::ast::Expr;
    
    match &expr.inner {
        Expr::List(elements) => {
            // Convert list of symbols to module name string
            let mut parts = Vec::new();
            for element in elements {
                match &element.inner {
                    Expr::Symbol(symbol) => parts.push(symbol.clone()),
                    _ => return Err(Box::new(Error::syntax_error(
                        "Module name must contain only symbols".to_string(),
                        Some(element.span),
                    )),
                }
            }
            Ok(format!("({})", parts.join(" ")))
        }
        Expr::Symbol(symbol) => {
            // Single symbol module name
            Ok(format!("({})", symbol))
        }
        _ => Err(Box::new(Error::syntax_error(
            "Invalid module name format".to_string(),
            Some(expr.span),
        )),
    }
}

/// Parses import configuration (only, except, rename, prefix).
fn parse_import_config(config_forms: &[Spanned<Expr>]) -> Result<ImportConfig> {
    use crate::ast::Expr;
    
    if config_forms.len() != 1 {
        return Err(Box::new(Error::syntax_error(
            "Import configuration must be a single form".to_string(),
            None,
        ));
    }

    match &config_forms[0].inner {
        Expr::List(elements) if !elements.is_empty() => {
            match &elements[0].inner {
                Expr::Symbol(keyword) => {
                    match keyword.as_str() {
                        "only" => parse_only_config(&elements[1..]),
                        "except" => parse_except_config(&elements[1..]),
                        "rename" => parse_rename_config(&elements[1..]),
                        "prefix" => parse_prefix_config(&elements[1..]),
                        _ => Err(Box::new(Error::syntax_error(
                            format!("Unknown import keyword: {}", keyword),
                            Some(elements[0].span),
                        )),
                    }
                }
                _ => Err(Box::new(Error::syntax_error(
                    "Import configuration must start with a keyword".to_string(),
                    Some(elements[0].span),
                )),
            }
        }
        _ => Err(Box::new(Error::syntax_error(
            "Import configuration must be a list".to_string(),
            Some(config_forms[0].span),
        )),
    }
}

/// Parses 'only' import configuration.
fn parse_only_config(elements: &[Spanned<Expr>]) -> Result<ImportConfig> {
    let mut symbols = Vec::new();
    
    for element in elements {
        match &element.inner {
            Expr::Identifier(symbol) => symbols.push(symbol.clone()),
            _ => return Err(Box::new(Error::syntax_error(
                "Only configuration must contain only symbols".to_string(),
                Some(element.span),
            )),
        }
    }
    
    Ok(ImportConfig::Only(symbols))
}

/// Parses 'except' import configuration.
fn parse_except_config(elements: &[Spanned<Expr>]) -> Result<ImportConfig> {
    let mut symbols = Vec::new();
    
    for element in elements {
        match &element.inner {
            Expr::Identifier(symbol) => symbols.push(symbol.clone()),
            _ => return Err(Box::new(Error::syntax_error(
                "Except configuration must contain only symbols".to_string(),
                Some(element.span),
            )),
        }
    }
    
    Ok(ImportConfig::Except(symbols))
}

/// Parses 'rename' import configuration.
fn parse_rename_config(elements: &[Spanned<Expr>]) -> Result<ImportConfig> {
    let mut rename_map = HashMap::new();
    
    for element in elements {
        match &element.inner {
            Expr::Application { operator, operands } if operands.len() == 1 => {
                let original = match &operator.inner {
                    Expr::Identifier(symbol) => symbol.clone()),
                    _ => return Err(Box::new(Error::syntax_error(
                        "Rename pair must contain symbols".to_string(),
                        Some(operator.span),
                    )),
                };
                
                let new_name = match &operands[0].inner {
                    Expr::Identifier(symbol) => symbol.clone()),
                    _ => return Err(Box::new(Error::syntax_error(
                        "Rename pair must contain symbols".to_string(),
                        Some(operands[0].span),
                    )),
                };
                
                rename_map.insert(original, new_name);
            }
            Expr::Pair { car, cdr } => {
                let original = match &car.inner {
                    Expr::Identifier(symbol) => symbol.clone()),
                    _ => return Err(Box::new(Error::syntax_error(
                        "Rename pair must contain symbols".to_string(),
                        Some(car.span),
                    )),
                };
                
                let new_name = match &cdr.inner {
                    Expr::Identifier(symbol) => symbol.clone()),
                    _ => return Err(Box::new(Error::syntax_error(
                        "Rename pair must contain symbols".to_string(),
                        Some(cdr.span),
                    )),
                };
                
                rename_map.insert(original, new_name);
            }
            _ => return Err(Box::new(Error::syntax_error(
                "Rename configuration must contain pairs of symbols".to_string(),
                Some(element.span),
            )),
        }
    }
    
    Ok(ImportConfig::Rename(rename_map))
}

/// Parses 'prefix' import configuration.
fn parse_prefix_config(elements: &[Spanned<Expr>]) -> Result<ImportConfig> {
    if elements.len() != 1 {
        return Err(Box::new(Error::syntax_error(
            "Prefix configuration must contain exactly one symbol".to_string(),
            None,
        ));
    }
    
    match &elements[0].inner {
        Expr::Identifier(prefix) => Ok(ImportConfig::Prefix(prefix.clone())),
        _ => Err(Box::new(Error::syntax_error(
            "Prefix must be a symbol".to_string(),
            Some(elements[0].span),
        )),
    }
}

/// Validates an import specification.
pub fn validate_import_spec(spec: &ImportSpec) -> Result<()> {
    super::name::validate_module_id(&spec.module_id)?;
    
    match &spec.config {
        ImportConfig::Only(symbols) | ImportConfig::Except(symbols) => {
            if symbols.is_empty() {
                return Err(Box::new(Error::syntax_error(
                    "Import configuration cannot be empty".to_string(),
                    None,
                ));
            }
        }
        ImportConfig::Rename(rename_map) => {
            if rename_map.is_empty() {
                return Err(Box::new(Error::syntax_error(
                    "Rename configuration cannot be empty".to_string(),
                    None,
                ));
            }
            
            // Check for duplicate target names
            let mut target_names = std::collections::HashSet::new();
            for target in rename_map.values() {
                if !target_names.insert(target) {
                    return Err(Box::new(Error::syntax_error(
                        format!("Duplicate rename target: {}", target),
                        None,
                    ));
                }
            }
        }
        ImportConfig::Prefix(prefix) => {
            if prefix.is_empty() {
                return Err(Box::new(Error::syntax_error(
                    "Prefix cannot be empty".to_string(),
                    None,
                ));
            }
        }
        ImportConfig::All => {
            // No validation needed for 'all' imports
        }
    }
    
    Ok(())
}

/// Merges multiple import bindings, detecting conflicts.
pub fn merge_import_bindings(
    bindings_list: &[HashMap<String, Value>],
) -> Result<HashMap<String, Value>> {
    let mut result = HashMap::new();
    
    for bindings in bindings_list {
        for (symbol, value) in bindings {
            if let Some(existing_value) = result.get(symbol) {
                // Check if it's the same value (allowing re-import of same binding)
                if !values_equivalent(existing_value, value) {
                    return Err(Box::new(Error::from(ModuleError::ImportConflict(
                        format!("Symbol '{}' imported from multiple modules with different values", symbol)
                    )));
                }
            } else {
                result.insert(symbol.clone()), value.clone());
            }
        }
    }
    
    Ok(result)
}

/// Checks if two values are equivalent for import conflict detection.
fn values_equivalent(a: &Value, b: &Value) -> bool {
    // For now, use structural equality
    // In a full implementation, this might check for procedure identity
    a == b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Value;
    use std::collections::HashMap;

    #[test]
    fn test_apply_only_import() {
        let mut exports = HashMap::new();
        exports.insert("string-length".to_string(), Value::integer(42));
        exports.insert("string-ref".to_string(), Value::integer(43));
        exports.insert("string-set!".to_string(), Value::integer(44));

        let symbols = vec!["string-length".to_string(), "string-ref".to_string()];
        let result = apply_only_import(&exports, &symbols).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("string-length"));
        assert!(result.contains_key("string-ref"));
        assert!(!result.contains_key("string-set!"));
    }

    #[test]
    fn test_apply_except_import() {
        let mut exports = HashMap::new();
        exports.insert("string-length".to_string(), Value::integer(42));
        exports.insert("string-ref".to_string(), Value::integer(43));
        exports.insert("string-set!".to_string(), Value::integer(44));

        let except_symbols = vec!["string-set!".to_string()];
        let result = apply_except_import(&exports, &except_symbols).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("string-length"));
        assert!(result.contains_key("string-ref"));
        assert!(!result.contains_key("string-set!"));
    }

    #[test]
    fn test_apply_rename_import() {
        let mut exports = HashMap::new();
        exports.insert("string-length".to_string(), Value::integer(42));
        exports.insert("string-ref".to_string(), Value::integer(43));

        let mut rename_map = HashMap::new();
        rename_map.insert("string-length".to_string(), "str-len".to_string());
        rename_map.insert("string-ref".to_string(), "str-ref".to_string());

        let result = apply_rename_import(&exports, &rename_map).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("str-len"));
        assert!(result.contains_key("str-ref"));
        assert!(!result.contains_key("string-length"));
        assert!(!result.contains_key("string-ref"));
    }

    #[test]
    fn test_apply_prefix_import() {
        let mut exports = HashMap::new();
        exports.insert("length".to_string(), Value::integer(42));
        exports.insert("ref".to_string(), Value::integer(43));

        let result = apply_prefix_import(&exports, "string:").unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("string:length"));
        assert!(result.contains_key("string:ref"));
        assert!(!result.contains_key("length"));
        assert!(!result.contains_key("ref"));
    }

    #[test]
    fn test_merge_import_bindings_no_conflict() {
        let mut bindings1 = HashMap::new();
        bindings1.insert("a".to_string(), Value::integer(1));
        bindings1.insert("b".to_string(), Value::integer(2));

        let mut bindings2 = HashMap::new();
        bindings2.insert("c".to_string(), Value::integer(3));
        bindings2.insert("d".to_string(), Value::integer(4));

        let result = merge_import_bindings(&[bindings1, bindings2]).unwrap();

        assert_eq!(result.len(), 4);
        assert!(result.contains_key("a"));
        assert!(result.contains_key("b"));
        assert!(result.contains_key("c"));
        assert!(result.contains_key("d"));
    }

    #[test]
    fn test_merge_import_bindings_with_conflict() {
        let mut bindings1 = HashMap::new();
        bindings1.insert("a".to_string(), Value::integer(1));

        let mut bindings2 = HashMap::new();
        bindings2.insert("a".to_string(), Value::integer(2)); // Different value

        let result = merge_import_bindings(&[bindings1, bindings2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_import_bindings_same_value() {
        let mut bindings1 = HashMap::new();
        bindings1.insert("a".to_string(), Value::integer(1));

        let mut bindings2 = HashMap::new();
        bindings2.insert("a".to_string(), Value::integer(1)); // Same value

        let result = merge_import_bindings(&[bindings1, bindings2]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("a"), Some(&Value::integer(1)));
    }
}