//! Module name parsing and resolution.
//!
//! Handles the parsing of module names from various syntactic forms:
//! - (lambdust string) - Lambdust built-in modules
//! - (:: string) - Deprecated Lambdust built-in syntax (backwards compatibility)
//! - (scheme base) - R7RS standard library
//! - (user my-module) - User-defined modules
//! - (file "path/to/module.scm") - File-based modules

use super::{ModuleId, ModuleNamespace};
use crate::diagnostics::{Error, Result};

/// Parses a module name from a string representation.
pub fn parse_module_name(input: &str) -> Result<ModuleId> {
    let trimmed = input.trim();
    
    // Must start and end with parentheses
    if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
        return Err(Box::new(Error::syntax_error(
            format!("Module name must be parenthesized: {input}"),
            None,
        )));
    }
    
    // Remove outer parentheses
    let inner = &trimmed[1..trimmed.len()-1].trim();
    let parts: Vec<&str> = inner.split_whitespace().collect();
    
    if parts.is_empty() {
        return Err(Box::new(Error::syntax_error(
            "Empty module name".to_string(),
            None,
        )));
    }
    
    match parts[0] {
        "::" => {
            // Deprecated syntax - emit warning but still support
            eprintln!("Warning: The '::' module syntax is deprecated. Use '(lambdust {})' instead of '(:: {})'.", 
                     parts[1..].join(" "), parts[1..].join(" "));
            parse_builtin_module(&parts[1..])
        },
        "lambdust" => parse_builtin_module(&parts[1..]),
        "scheme" => parse_r7rs_module(&parts[1..]),
        "srfi" => parse_srfi_module(&parts[1..]),
        "user" => parse_user_module(&parts[1..]),
        "file" => parse_file_module(&parts[1..]),
        _ => {
            // Default to user module if no explicit namespace
            let components = parts.iter().map(|s| s.to_string()).collect();
            Ok(ModuleId {
                components,
                namespace: ModuleNamespace::User,
            })
        }
    }
}

/// Parses a Lambdust built-in module name.
fn parse_builtin_module(parts: &[&str]) -> Result<ModuleId> {
    if parts.is_empty() {
        return Err(Box::new(Error::syntax_error(
            "Built-in module name cannot be empty".to_string(),
            None,
        )));
    }
    
    let components = parts.iter().map(|s| s.to_string()).collect();
    Ok(ModuleId {
        components,
        namespace: ModuleNamespace::Builtin,
    })
}

/// Parses an R7RS standard library module name.
fn parse_r7rs_module(parts: &[&str]) -> Result<ModuleId> {
    if parts.is_empty() {
        return Err(Box::new(Error::syntax_error(
            "R7RS module name cannot be empty".to_string(),
            None,
        )));
    }
    
    let components = parts.iter().map(|s| s.to_string()).collect();
    Ok(ModuleId {
        components,
        namespace: ModuleNamespace::R7RS,
    })
}

/// Parses a SRFI module name.
/// Supports both single SRFI numbers and lists of SRFI numbers:
/// - (srfi 1) -> SRFI-1
/// - (srfi (1 13 14)) -> Multiple SRFIs
fn parse_srfi_module(parts: &[&str]) -> Result<ModuleId> {
    if parts.is_empty() {
        return Err(Box::new(Error::syntax_error(
            "SRFI module name cannot be empty".to_string(),
            None,
        )));
    }
    
    // Handle different SRFI syntaxes
    if parts.len() == 1 {
        // Single SRFI number: (srfi 1)
        let srfi_num = parts[0];
        validate_srfi_number(srfi_num)?;
        let components = vec![srfi_num.to_string()];
        Ok(ModuleId {
            components,
            namespace: ModuleNamespace::SRFI,
        })
    } else if parts.len() > 1 && parts[0].starts_with('(') {
        // Multiple SRFI numbers: (srfi (1 13 14))
        // Parse the parenthesized list
        let list_str = parts.join(" ");
        if !list_str.starts_with('(') || !list_str.ends_with(')') {
            return Err(Box::new(Error::syntax_error(
                "SRFI list must be parenthesized".to_string(),
                None,
            )));
        }
        
        let inner = &list_str[1..list_str.len()-1];
        let srfi_numbers: Vec<&str> = inner.split_whitespace().collect();
        
        if srfi_numbers.is_empty() {
            return Err(Box::new(Error::syntax_error(
                "SRFI list cannot be empty".to_string(),
                None,
            )));
        }
        
        // Validate all SRFI numbers
        for srfi_num in &srfi_numbers {
            validate_srfi_number(srfi_num)?;
        }
        
        let components = srfi_numbers.iter().map(|s| s.to_string()).collect();
        Ok(ModuleId {
            components,
            namespace: ModuleNamespace::SRFI,
        })
    } else {
        // Multiple individual numbers: not supported in standard syntax
        return Err(Box::new(Error::syntax_error(
            "Invalid SRFI module syntax. Use (srfi <number>) or (srfi (<number> ...))".to_string(),
            None,
        )));
    }
}

/// Validates that a string represents a valid SRFI number.
fn validate_srfi_number(srfi_str: &str) -> Result<()> {
    match srfi_str.parse::<u32>() {
        Ok(num) if num > 0 => Ok(()),
        _ => Err(Box::new(Error::syntax_error(
            format!("Invalid SRFI number: {srfi_str}"),
            None,
        ))),
    }
}

/// Parses a user-defined module name.
fn parse_user_module(parts: &[&str]) -> Result<ModuleId> {
    if parts.is_empty() {
        return Err(Box::new(Error::syntax_error(
            "User module name cannot be empty".to_string(),
            None,
        )));
    }
    
    let components = parts.iter().map(|s| s.to_string()).collect();
    Ok(ModuleId {
        components,
        namespace: ModuleNamespace::User,
    })
}

/// Parses a file-based module name.
fn parse_file_module(parts: &[&str]) -> Result<ModuleId> {
    if parts.len() != 1 {
        return Err(Box::new(Error::syntax_error(
            "File module must specify exactly one path".to_string(),
            None,
        )));
    }
    
    let path_str = parts[0];
    
    // Remove quotes if present
    let path = if path_str.starts_with('"') && path_str.ends_with('"') {
        &path_str[1..path_str.len()-1]
    } else {
        path_str
    };
    
    let components = vec![path.to_string()];
    Ok(ModuleId {
        components,
        namespace: ModuleNamespace::File,
    })
}

/// Validates a module identifier.
pub fn validate_module_id(id: &ModuleId) -> Result<()> {
    if id.components.is_empty() {
        return Err(Box::new(Error::syntax_error(
            "Module ID cannot have empty components".to_string(),
            None,
        )));
    }
    
    // Check for invalid characters in components
    for component in &id.components {
        if component.is_empty() {
            return Err(Box::new(Error::syntax_error(
                "Module component cannot be empty".to_string(),
                None,
            )));
        }
        
        // Additional validation rules can be added here
        // For example, checking for valid identifier characters
    }
    
    Ok(())
}

/// Converts a module ID back to its canonical string representation.
pub fn module_id_to_string(id: &ModuleId) -> String {
    match id.namespace {
        ModuleNamespace::Builtin => {
            format!("(lambdust {})", id.components.join(" "))
        }
        ModuleNamespace::R7RS => {
            format!("(scheme {})", id.components.join(" "))
        }
        ModuleNamespace::SRFI => {
            if id.components.len() == 1 {
                format!("(srfi {})", id.components[0])
            } else {
                format!("(srfi ({}))", id.components.join(" "))
            }
        }
        ModuleNamespace::User => {
            format!("(user {})", id.components.join(" "))
        }
        ModuleNamespace::File => {
            format!("(file \"{}\")", id.components.join("/"))
        }
    }
}

/// Checks if a module ID represents a built-in module.
pub fn is_builtin_module(id: &ModuleId) -> bool {
    matches!(id.namespace, ModuleNamespace::Builtin)
}

/// Checks if a module ID represents an R7RS standard library module.
pub fn is_r7rs_module(id: &ModuleId) -> bool {
    matches!(id.namespace, ModuleNamespace::R7RS)
}

/// Checks if a module ID represents a SRFI module.
pub fn is_srfi_module(id: &ModuleId) -> bool {
    matches!(id.namespace, ModuleNamespace::SRFI)
}

/// Checks if a module ID represents a user-defined module.
pub fn is_user_module(id: &ModuleId) -> bool {
    matches!(id.namespace, ModuleNamespace::User)
}

/// Checks if a module ID represents a file-based module.
pub fn is_file_module(id: &ModuleId) -> bool {
    matches!(id.namespace, ModuleNamespace::File)
}

/// Gets the module name without namespace prefix.
pub fn get_module_name(id: &ModuleId) -> String {
    id.components.join(" ")
}

/// Creates a module ID for a built-in module.
pub fn builtin_module(name: &str) -> ModuleId {
    ModuleId {
        components: vec![name.to_string()],
        namespace: ModuleNamespace::Builtin,
    }
}

/// Creates a module ID for an R7RS module.
pub fn r7rs_module(name: &str) -> ModuleId {
    ModuleId {
        components: vec![name.to_string()],
        namespace: ModuleNamespace::R7RS,
    }
}

/// Creates a module ID for a single SRFI module.
pub fn srfi_module(number: u32) -> ModuleId {
    ModuleId {
        components: vec![number.to_string()],
        namespace: ModuleNamespace::SRFI,
    }
}

/// Creates a module ID for multiple SRFI modules.
pub fn srfi_modules(numbers: &[u32]) -> ModuleId {
    ModuleId {
        components: numbers.iter().map(|n| n.to_string()).collect(),
        namespace: ModuleNamespace::SRFI,
    }
}

/// Creates a module ID for a user module.
pub fn user_module(name: &str) -> ModuleId {
    ModuleId {
        components: vec![name.to_string()],
        namespace: ModuleNamespace::User,
    }
}

/// Creates a module ID for a file module.
pub fn file_module(path: &str) -> ModuleId {
    ModuleId {
        components: vec![path.to_string()],
        namespace: ModuleNamespace::File,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_builtin_module_new_syntax() {
        let id = parse_module_name("(lambdust string)").unwrap();
        assert_eq!(id.namespace, ModuleNamespace::Builtin);
        assert_eq!(id.components, vec!["string"]);
    }

    #[test]
    fn test_parse_builtin_module_deprecated_syntax() {
        // Test that old :: syntax still works (with warning)
        let id = parse_module_name("(:: string)").unwrap();
        assert_eq!(id.namespace, ModuleNamespace::Builtin);
        assert_eq!(id.components, vec!["string"]);
    }

    #[test]
    fn test_parse_r7rs_module() {
        let id = parse_module_name("(scheme base)").unwrap();
        assert_eq!(id.namespace, ModuleNamespace::R7RS);
        assert_eq!(id.components, vec!["base"]);
    }

    #[test]
    fn test_parse_srfi_single_number() {
        let id = parse_module_name("(srfi 1)").unwrap();
        assert_eq!(id.namespace, ModuleNamespace::SRFI);
        assert_eq!(id.components, vec!["1"]);
    }

    #[test]
    fn test_parse_srfi_multiple_numbers() {
        let id = parse_module_name("(srfi (1 13 14))").unwrap();
        assert_eq!(id.namespace, ModuleNamespace::SRFI);
        assert_eq!(id.components, vec!["1", "13", "14"]);
    }

    #[test]
    fn test_parse_srfi_zero_number_invalid() {
        assert!(parse_module_name("(srfi 0)").is_err());
    }

    #[test]
    fn test_parse_srfi_negative_number_invalid() {
        assert!(parse_module_name("(srfi -1)").is_err());
    }

    #[test]
    fn test_parse_srfi_non_number_invalid() {
        assert!(parse_module_name("(srfi abc)").is_err());
    }

    #[test]
    fn test_parse_user_module() {
        let id = parse_module_name("(user my-module)").unwrap();
        assert_eq!(id.namespace, ModuleNamespace::User);
        assert_eq!(id.components, vec!["my-module"]);
    }

    #[test]
    fn test_parse_file_module() {
        let id = parse_module_name("(file \"path/to/module.scm\")").unwrap();
        assert_eq!(id.namespace, ModuleNamespace::File);
        assert_eq!(id.components, vec!["path/to/module.scm"]);
    }

    #[test]
    fn test_parse_multipart_module() {
        let id = parse_module_name("(scheme char numeric)").unwrap();
        assert_eq!(id.namespace, ModuleNamespace::R7RS);
        assert_eq!(id.components, vec!["char", "numeric"]);
    }

    #[test]
    fn test_module_id_roundtrip_new_syntax() {
        let original = "(lambdust string)";
        let id = parse_module_name(original).unwrap();
        let reconstructed = module_id_to_string(&id);
        assert_eq!(original, reconstructed);
    }

    #[test]
    fn test_module_id_roundtrip_deprecated_to_new() {
        // Old syntax should parse but output new format
        let id = parse_module_name("(:: string)").unwrap();
        let reconstructed = module_id_to_string(&id);
        assert_eq!(reconstructed, "(lambdust string)");
    }

    #[test]
    fn test_invalid_module_names() {
        assert!(parse_module_name("no-parens").is_err());
        assert!(parse_module_name("()").is_err());
        assert!(parse_module_name("(::)").is_err());
    }

    #[test]
    fn test_srfi_module_roundtrip() {
        let single_srfi = "(srfi 1)";
        let id = parse_module_name(single_srfi).unwrap();
        let reconstructed = module_id_to_string(&id);
        assert_eq!(single_srfi, reconstructed);

        let multi_srfi = "(srfi (1 13 14))";
        let id = parse_module_name(multi_srfi).unwrap();
        let reconstructed = module_id_to_string(&id);
        assert_eq!(multi_srfi, reconstructed);
    }

    #[test]
    fn test_module_predicates() {
        let builtin = builtin_module("string");
        let r7rs = r7rs_module("base");
        let srfi = srfi_module(1);
        let user = user_module("my-module");
        let file = file_module("module.scm");

        assert!(is_builtin_module(&builtin));
        assert!(is_r7rs_module(&r7rs));
        assert!(is_srfi_module(&srfi));
        assert!(is_user_module(&user));
        assert!(is_file_module(&file));

        assert!(!is_builtin_module(&r7rs));
        assert!(!is_r7rs_module(&srfi));
        assert!(!is_srfi_module(&user));
        assert!(!is_user_module(&file));
        assert!(!is_file_module(&builtin));
    }
}