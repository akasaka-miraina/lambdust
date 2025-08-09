//! Export functionality for making module symbols available.
//!
//! Handles export declarations and symbol visibility:
//! - (export symbol1 symbol2 ...) - Export specific symbols
//! - (export (rename (internal-name external-name))) - Export with renaming
//! - Validation of exported symbols and visibility rules

use super::{ExportSpec, ExportConfig, ModuleError};
use crate::diagnostics::{Error, Result, Spanned};
use crate::ast::Expr;
use crate::eval::Value;
use std::collections::HashMap;

/// Applies export configuration to create the final export map.
pub fn apply_export_config(
    bindings: &HashMap<String, Value>,
    config: &ExportConfig,
    symbols: &[String],
) -> Result<HashMap<String, Value>> {
    match config {
        ExportConfig::Direct => apply_direct_export(bindings, symbols),
        ExportConfig::Rename(rename_map) => apply_rename_export(bindings, rename_map),
    }
}

/// Exports symbols directly without modification.
pub fn apply_direct_export(
    bindings: &HashMap<String, Value>,
    symbols: &[String],
) -> Result<HashMap<String, Value>> {
    let mut exports = HashMap::new();
    
    for symbol in symbols {
        if let Some(value) = bindings.get(symbol) {
            exports.insert(symbol.clone(), value.clone());
        } else {
            return Err(Box::new(Error::from(ModuleError::InvalidDefinition(
                format!("Cannot export undefined symbol: {symbol}")
            ))));
        }
    }
    
    Ok(exports)
}

/// Exports symbols with renaming.
pub fn apply_rename_export(
    bindings: &HashMap<String, Value>,
    rename_map: &HashMap<String, String>,
) -> Result<HashMap<String, Value>> {
    let mut exports = HashMap::new();
    
    for (internal_name, external_name) in rename_map {
        if let Some(value) = bindings.get(internal_name) {
            exports.insert(external_name.clone(), value.clone());
        } else {
            return Err(Box::new(Error::from(ModuleError::InvalidDefinition(
                format!("Cannot export undefined symbol: {internal_name}")
            ))));
        }
    }
    
    Ok(exports)
}

/// Parses export specifications from Scheme syntax.
pub fn parse_export_spec(export_form: &[Spanned<Expr>]) -> Result<ExportSpec> {
    use crate::ast::Expr;
    
    if export_form.is_empty() {
        return Err(Box::new(Error::syntax_error(
            "Empty export specification".to_string(),
            None,
        )));
    }

    let mut symbols = Vec::new();
    let mut rename_map = HashMap::new();
    
    for expr in export_form {
        match &expr.inner {
            Expr::Symbol(symbol) => {
                symbols.push(symbol.clone());
            }
            Expr::List(elements) if !elements.is_empty() => {
                match &elements[0].inner {
                    Expr::Symbol(keyword) if keyword == "rename" => {
                        // Parse rename specifications
                        for rename_spec in &elements[1..] {
                            parse_rename_spec(rename_spec, &mut rename_map)?;
                        }
                    }
                    _ => return Err(Box::new(Error::syntax_error(
                        "Unknown export form".to_string(),
                        Some(elements[0].span),
                    ))),
                }
            }
            _ => return Err(Box::new(Error::syntax_error(
                "Invalid export specification".to_string(),
                Some(expr.span),
            ))),
        }
    }
    
    let config = if rename_map.is_empty() {
        ExportConfig::Direct
    } else if symbols.is_empty() {
        ExportConfig::Rename(rename_map)
    } else {
        return Err(Box::new(Error::syntax_error(
            "Cannot mix direct exports and rename exports".to_string(),
            None,
        )));
    };
    
    Ok(ExportSpec { symbols, config })
}

/// Parses a single rename specification.
fn parse_rename_spec(
    spec: &Spanned<Expr>,
    rename_map: &mut HashMap<String, String>,
) -> Result<()> {
    use crate::ast::Expr;
    
    match &spec.inner {
        Expr::List(pair) if pair.len() == 2 => {
            let internal_name = match &pair[0].inner {
                Expr::Symbol(symbol) => symbol.clone(),
                _ => return Err(Box::new(Error::syntax_error(
                    "Rename specification must contain symbols".to_string(),
                    Some(pair[0].span),
                ))),
            };
            
            let external_name = match &pair[1].inner {
                Expr::Symbol(symbol) => symbol.clone(),
                _ => return Err(Box::new(Error::syntax_error(
                    "Rename specification must contain symbols".to_string(),
                    Some(pair[1].span),
                ))),
            };
            
            rename_map.insert(internal_name, external_name);
            Ok(())
        }
        _ => Err(Box::new(Error::syntax_error(
            "Rename specification must be a pair of symbols".to_string(),
            Some(spec.span),
        ))),
    }
}

/// Validates an export specification.
pub fn validate_export_spec(
    spec: &ExportSpec,
    available_bindings: &HashMap<String, Value>,
) -> Result<()> {
    match &spec.config {
        ExportConfig::Direct => {
            // Check that all exported symbols exist
            for symbol in &spec.symbols {
                if !available_bindings.contains_key(symbol) {
                    return Err(Box::new(Error::from(ModuleError::InvalidDefinition(
                        format!("Cannot export undefined symbol: {symbol}")
                    ))));
                }
            }
        }
        ExportConfig::Rename(rename_map) => {
            // Check that all internal names exist
            for internal_name in rename_map.keys() {
                if !available_bindings.contains_key(internal_name) {
                    return Err(Box::new(Error::from(ModuleError::InvalidDefinition(
                        format!("Cannot export undefined symbol: {internal_name}")
                    ))));
                }
            }
            
            // Check for duplicate external names
            let mut external_names = std::collections::HashSet::new();
            for external_name in rename_map.values() {
                if !external_names.insert(external_name) {
                    return Err(Box::new(Error::from(ModuleError::InvalidDefinition(
                        format!("Duplicate export name: {external_name}")
                    ))));
                }
            }
        }
    }
    
    Ok(())
}

/// Creates a default export specification that exports all public bindings.
pub fn create_default_export_spec(bindings: &HashMap<String, Value>) -> ExportSpec {
    let symbols: Vec<String> = bindings.keys()
        .filter(|symbol| !is_private_symbol(symbol))
        .cloned()
        .collect();
    
    ExportSpec {
        symbols,
        config: ExportConfig::Direct,
    }
}

/// Checks if a symbol should be considered private (not exported by default).
pub fn is_private_symbol(symbol: &str) -> bool {
    // Symbols starting with underscore are private
    symbol.starts_with('_') ||
    // Internal helper functions (common convention)
    symbol.starts_with("internal-") ||
    // System symbols
    symbol.starts_with("system:")
}

/// Computes the intersection of export specifications.
pub fn intersect_export_specs(
    spec1: &ExportSpec,
    spec2: &ExportSpec,
) -> Result<ExportSpec> {
    match (&spec1.config, &spec2.config) {
        (ExportConfig::Direct, ExportConfig::Direct) => {
            // Intersect symbol lists
            let intersection: Vec<String> = spec1.symbols.iter()
                .filter(|symbol| spec2.symbols.contains(symbol))
                .cloned()
                .collect();
            
            Ok(ExportSpec {
                symbols: intersection,
                config: ExportConfig::Direct,
            })
        }
        _ => Err(Box::new(Error::syntax_error(
            "Cannot intersect export specifications with different configurations".to_string(),
            None,
        ))),
    }
}

/// Computes the union of export specifications.
pub fn union_export_specs(
    spec1: &ExportSpec,
    spec2: &ExportSpec,
) -> Result<ExportSpec> {
    match (&spec1.config, &spec2.config) {
        (ExportConfig::Direct, ExportConfig::Direct) => {
            // Union symbol lists
            let mut union_symbols = spec1.symbols.clone();
            for symbol in &spec2.symbols {
                if !union_symbols.contains(symbol) {
                    union_symbols.push(symbol.clone());
                }
            }
            
            Ok(ExportSpec {
                symbols: union_symbols,
                config: ExportConfig::Direct,
            })
        }
        _ => Err(Box::new(Error::syntax_error(
            "Cannot union export specifications with different configurations".to_string(),
            None,
        ))),
    }
}

/// Filters export specification to remove specific symbols.
pub fn filter_export_spec(
    spec: &ExportSpec,
    excluded_symbols: &[String],
) -> ExportSpec {
    match &spec.config {
        ExportConfig::Direct => {
            let filtered_symbols: Vec<String> = spec.symbols.iter()
                .filter(|symbol| !excluded_symbols.contains(symbol))
                .cloned()
                .collect();
            
            ExportSpec {
                symbols: filtered_symbols,
                config: ExportConfig::Direct,
            }
        }
        ExportConfig::Rename(rename_map) => {
            let filtered_map: HashMap<String, String> = rename_map.iter()
                .filter(|(internal_name, _)| !excluded_symbols.contains(internal_name))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            
            ExportSpec {
                symbols: Vec::new(),
                config: ExportConfig::Rename(filtered_map),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Value;
    use std::collections::HashMap;

    #[test]
    fn test_apply_direct_export() {
        let mut bindings = HashMap::new();
        bindings.insert("string-length".to_string(), Value::integer(42));
        bindings.insert("string-ref".to_string(), Value::integer(43));
        bindings.insert("internal-helper".to_string(), Value::integer(44));

        let symbols = vec!["string-length".to_string(), "string-ref".to_string()];
        let result = apply_direct_export(&bindings, &symbols).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("string-length"));
        assert!(result.contains_key("string-ref"));
        assert!(!result.contains_key("internal-helper"));
    }

    #[test]
    fn test_apply_rename_export() {
        let mut bindings = HashMap::new();
        bindings.insert("internal-length".to_string(), Value::integer(42));
        bindings.insert("internal-ref".to_string(), Value::integer(43));

        let mut rename_map = HashMap::new();
        rename_map.insert("internal-length".to_string(), "string-length".to_string());
        rename_map.insert("internal-ref".to_string(), "string-ref".to_string());

        let result = apply_rename_export(&bindings, &rename_map).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("string-length"));
        assert!(result.contains_key("string-ref"));
        assert!(!result.contains_key("internal-length"));
        assert!(!result.contains_key("internal-ref"));
    }

    #[test]
    fn test_export_undefined_symbol() {
        let bindings = HashMap::new(); // Empty bindings
        let symbols = vec!["nonexistent".to_string()];
        
        let result = apply_direct_export(&bindings, &symbols);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_default_export_spec() {
        let mut bindings = HashMap::new();
        bindings.insert("public-function".to_string(), Value::integer(1));
        bindings.insert("_private-function".to_string(), Value::integer(2));
        bindings.insert("internal-helper".to_string(), Value::integer(3));
        bindings.insert("system:internal".to_string(), Value::integer(4));

        let spec = create_default_export_spec(&bindings);

        assert_eq!(spec.symbols.len(), 1);
        assert!(spec.symbols.contains(&"public-function".to_string()));
        assert!(!spec.symbols.contains(&"_private-function".to_string()));
        assert!(!spec.symbols.contains(&"internal-helper".to_string()));
        assert!(!spec.symbols.contains(&"system:internal".to_string()));
    }

    #[test]
    fn test_is_private_symbol() {
        assert!(is_private_symbol("_private"));
        assert!(is_private_symbol("internal-helper"));
        assert!(is_private_symbol("system:something"));
        assert!(!is_private_symbol("public-function"));
        assert!(!is_private_symbol("string-length"));
    }

    #[test]
    fn test_union_export_specs() {
        let spec1 = ExportSpec {
            symbols: vec!["a".to_string(), "b".to_string()],
            config: ExportConfig::Direct,
        };
        
        let spec2 = ExportSpec {
            symbols: vec!["b".to_string(), "c".to_string()],
            config: ExportConfig::Direct,
        };

        let union = union_export_specs(&spec1, &spec2).unwrap();
        
        assert_eq!(union.symbols.len(), 3);
        assert!(union.symbols.contains(&"a".to_string()));
        assert!(union.symbols.contains(&"b".to_string()));
        assert!(union.symbols.contains(&"c".to_string()));
    }

    #[test]
    fn test_intersect_export_specs() {
        let spec1 = ExportSpec {
            symbols: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            config: ExportConfig::Direct,
        };
        
        let spec2 = ExportSpec {
            symbols: vec!["b".to_string(), "c".to_string(), "d".to_string()],
            config: ExportConfig::Direct,
        };

        let intersection = intersect_export_specs(&spec1, &spec2).unwrap();
        
        assert_eq!(intersection.symbols.len(), 2);
        assert!(intersection.symbols.contains(&"b".to_string()));
        assert!(intersection.symbols.contains(&"c".to_string()));
        assert!(!intersection.symbols.contains(&"a".to_string()));
        assert!(!intersection.symbols.contains(&"d".to_string()));
    }

    #[test]
    fn test_filter_export_spec() {
        let spec = ExportSpec {
            symbols: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            config: ExportConfig::Direct,
        };
        
        let excluded = vec!["b".to_string()];
        let filtered = filter_export_spec(&spec, &excluded);
        
        assert_eq!(filtered.symbols.len(), 2);
        assert!(filtered.symbols.contains(&"a".to_string()));
        assert!(filtered.symbols.contains(&"c".to_string()));
        assert!(!filtered.symbols.contains(&"b".to_string()));
    }
}