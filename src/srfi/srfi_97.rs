//! SRFI 97: SRFI Libraries
//!
//! This SRFI provides standardized access to SRFI libraries through inquiry functions.

use super::SrfiModule;
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// SRFI 97 implementation
pub struct Srfi97;

impl SrfiModule for Srfi97 {
    fn srfi_id(&self) -> u32 {
        97
    }
    
    fn name(&self) -> &'static str {
        "SRFI Libraries"
    }
    
    fn parts(&self) -> Vec<&'static str> {
        vec!["inquiry", "available"]
    }
    
    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // SRFI inquiry functions
        exports.insert("srfi-available?".to_string(), srfi_available_function());
        exports.insert("srfi-supported-ids".to_string(), srfi_supported_ids_function());
        exports.insert("srfi-name".to_string(), srfi_name_function());
        exports.insert("srfi-parts".to_string(), srfi_parts_function());
        
        exports
    }
    
    fn exports_for_parts(&self, parts: &[&str]) -> Result<HashMap<String, Value>> {
        let all_exports = self.exports();
        let mut filtered = HashMap::new();
        
        for part in parts {
            match *part {
                "inquiry" => {
                    // SRFI inquiry functions
                    if let Some(value) = all_exports.get("srfi-available?") {
                        filtered.insert("srfi-available?".to_string(), value.clone());
                    }
                    if let Some(value) = all_exports.get("srfi-name") {
                        filtered.insert("srfi-name".to_string(), value.clone());
                    }
                    if let Some(value) = all_exports.get("srfi-parts") {
                        filtered.insert("srfi-parts".to_string(), value.clone());
                    }
                }
                "available" => {
                    // Available SRFI listing functions
                    if let Some(value) = all_exports.get("srfi-supported-ids") {
                        filtered.insert("srfi-supported-ids".to_string(), value.clone());
                    }
                }
                _ => {
                    return Err(LambdustError::runtime_error(
                        format!("Unknown SRFI 97 part: {}", part)
                    ));
                }
            }
        }
        
        Ok(filtered)
    }
}

/// Creates a function that checks if a SRFI is available
fn srfi_available_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "srfi-available?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match &args[0] {
                Value::Number(n) => {
                    let id = match n {
                        crate::lexer::SchemeNumber::Integer(i) => *i as u32,
                        crate::lexer::SchemeNumber::Real(f) => *f as u32,
                        crate::lexer::SchemeNumber::Rational(num, den) => (*num / *den) as u32,
                        crate::lexer::SchemeNumber::Complex(real, _) => *real as u32,
                    };
                    
                    // Check if this SRFI ID is supported
                    let supported = match id {
                        9 => true,   // Define-record-type
                        45 => true,  // Lazy evaluation
                        46 => true,  // Syntax-rules extensions  
                        97 => true,  // SRFI Libraries (self)
                        _ => false,
                    };
                    
                    Ok(Value::Boolean(supported))
                }
                _ => Err(LambdustError::type_error(
                    "srfi-available? expects a number".to_string(),
                )),
            }
        },
    })
}

/// Creates a function that returns supported SRFI IDs
fn srfi_supported_ids_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "srfi-supported-ids".to_string(),
        arity: Some(0),
        func: |args| {
            if !args.is_empty() {
                return Err(LambdustError::arity_error(0, args.len()));
            }

            // Return list of supported SRFI IDs
            let supported_ids = vec![
                Value::Number(crate::lexer::SchemeNumber::Integer(9)),
                Value::Number(crate::lexer::SchemeNumber::Integer(45)),
                Value::Number(crate::lexer::SchemeNumber::Integer(46)),
                Value::Number(crate::lexer::SchemeNumber::Integer(97)),
            ];
            
            Ok(Value::Vector(supported_ids))
        },
    })
}

/// Creates a function that returns the name of a SRFI
fn srfi_name_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "srfi-name".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match &args[0] {
                Value::Number(n) => {
                    let id = match n {
                        crate::lexer::SchemeNumber::Integer(i) => *i as u32,
                        crate::lexer::SchemeNumber::Real(f) => *f as u32,
                        crate::lexer::SchemeNumber::Rational(num, den) => (*num / *den) as u32,
                        crate::lexer::SchemeNumber::Complex(real, _) => *real as u32,
                    };
                    
                    let name = match id {
                        9 => "Defining Record Types",
                        45 => "Primitives for Expressing Iterative Lazy Algorithms",
                        46 => "Basic Syntax-rules Extensions",
                        97 => "SRFI Libraries",
                        _ => return Err(LambdustError::runtime_error(
                            format!("Unknown SRFI: {}", id)
                        )),
                    };
                    
                    Ok(Value::String(name.to_string()))
                }
                _ => Err(LambdustError::type_error(
                    "srfi-name expects a number".to_string(),
                )),
            }
        },
    })
}

/// Creates a function that returns the parts of a SRFI
fn srfi_parts_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "srfi-parts".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match &args[0] {
                Value::Number(n) => {
                    let id = match n {
                        crate::lexer::SchemeNumber::Integer(i) => *i as u32,
                        crate::lexer::SchemeNumber::Real(f) => *f as u32,
                        crate::lexer::SchemeNumber::Rational(num, den) => (*num / *den) as u32,
                        crate::lexer::SchemeNumber::Complex(real, _) => *real as u32,
                    };
                    
                    let parts = match id {
                        9 => vec!["records", "types"],
                        45 => vec!["lazy", "promises"],
                        46 => vec!["syntax", "ellipsis"],
                        97 => vec!["inquiry", "available"],
                        _ => return Err(LambdustError::runtime_error(
                            format!("Unknown SRFI: {}", id)
                        )),
                    };
                    
                    let part_values: Vec<Value> = parts.into_iter()
                        .map(|s| Value::String(s.to_string()))
                        .collect();
                    
                    Ok(Value::Vector(part_values))
                }
                _ => Err(LambdustError::type_error(
                    "srfi-parts expects a number".to_string(),
                )),
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_srfi_97_info() {
        let srfi97 = Srfi97;
        assert_eq!(srfi97.srfi_id(), 97);
        assert_eq!(srfi97.name(), "SRFI Libraries");
        assert!(srfi97.parts().contains(&"inquiry"));
        assert!(srfi97.parts().contains(&"available"));
    }
    
    #[test]
    fn test_srfi_97_exports() {
        let srfi97 = Srfi97;
        let exports = srfi97.exports();
        
        assert!(exports.contains_key("srfi-available?"));
        assert!(exports.contains_key("srfi-supported-ids"));
        assert!(exports.contains_key("srfi-name"));
        assert!(exports.contains_key("srfi-parts"));
    }
    
    #[test]
    fn test_srfi_available_function() {
        let func = srfi_available_function();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
            // Test with supported SRFI
            let result = func(&[Value::Number(crate::lexer::SchemeNumber::Integer(9))]).unwrap();
            assert_eq!(result, Value::Boolean(true));
            
            // Test with unsupported SRFI
            let result = func(&[Value::Number(crate::lexer::SchemeNumber::Integer(999))]).unwrap();
            assert_eq!(result, Value::Boolean(false));
        }
    }
    
    #[test]
    fn test_srfi_supported_ids_function() {
        let func = srfi_supported_ids_function();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
            let result = func(&[]).unwrap();
            assert!(matches!(result, Value::Vector(_)));
        }
    }
    
    #[test]
    fn test_srfi_name_function() {
        let func = srfi_name_function();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
            let result = func(&[Value::Number(crate::lexer::SchemeNumber::Integer(9))]).unwrap();
            assert_eq!(result, Value::String("Defining Record Types".to_string()));
        }
    }
}