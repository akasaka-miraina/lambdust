//! SRFI 145: Assumptions
//!
//! This SRFI defines a syntax for expressing assumptions about code.
//! Assumptions are contracts that help with program verification and debugging.
//! They provide a form of Design by Contract programming.

use crate::error::{LambdustError, Result};
use crate::value::{Value, Procedure};
use std::collections::HashMap;

/// SRFI 145 module implementation
pub struct Srfi145Module;

impl crate::srfi::SrfiModule for Srfi145Module {
    fn srfi_id(&self) -> u32 {
        145
    }

    fn name(&self) -> &'static str {
        "SRFI 145"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["assumptions"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Core assumption operations
        exports.insert("assume".to_string(), Value::Procedure(Procedure::Builtin { name: "assume".to_string(), arity: Some(1), func: assume }));
        exports.insert("assumption-violation".to_string(), Value::Procedure(Procedure::Builtin { name: "assumption-violation".to_string(), arity: None, func: assumption_violation }));
        
        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // For now, return all exports regardless of parts
        Ok(self.exports())
    }
}

impl Srfi145Module {
    /// Creates a new SRFI-145 module instance
    pub fn new() -> Self {
        Self
    }
}

/// Assume that a condition is true
/// If the condition is false, this is an assumption violation
fn assume(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let condition = &args[0];
    
    // Check if the condition is true
    match condition {
        Value::Boolean(true) => {
            // Assumption holds, return void/unspecified
            Ok(Value::Undefined)
        }
        Value::Boolean(false) => {
            // Assumption violation
            Err(LambdustError::runtime_error("assumption violation: condition is false"))
        }
        _ => {
            // In Scheme, any non-#f value is considered true
            Ok(Value::Undefined)
        }
    }
}

/// Report an assumption violation with optional message and irritants
fn assumption_violation(args: &[Value]) -> Result<Value> {
    let message = if args.is_empty() {
        "assumption violation".to_string()
    } else {
        // First argument should be who (procedure name or description)
        let who = &args[0];
        let who_str = match who {
            Value::String(s) => s.clone(),
            Value::Symbol(s) => s.clone(),
            _ => format!("{:?}", who),
        };
        
        if args.len() > 1 {
            // Second argument should be message
            let msg = &args[1];
            let msg_str = match msg {
                Value::String(s) => s.clone(),
                _ => format!("{:?}", msg),
            };
            
            if args.len() > 2 {
                // Additional arguments are irritants
                let irritants: Vec<String> = args[2..]
                    .iter()
                    .map(|v| format!("{:?}", v))
                    .collect();
                format!("assumption violation in {}: {} (irritants: {})", who_str, msg_str, irritants.join(", "))
            } else {
                format!("assumption violation in {}: {}", who_str, msg_str)
            }
        } else {
            format!("assumption violation in {}", who_str)
        }
    };
    
    Err(LambdustError::runtime_error(message))
}

#[cfg(test)]
mod tests {
    use super::*;
        use crate::srfi::SrfiModule;
    use std::sync::Arc;

    #[test]
    fn test_assume_true() {
        
        let result = assume(&[Value::Boolean(true)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Undefined);
    }

    #[test]
    fn test_assume_false() {
        
        let result = assume(&[Value::Boolean(false)]);
        assert!(result.is_err());
        if let Err(LambdustError::RuntimeError { message, .. }) = result {
            assert!(message.contains("assumption violation"));
        } else {
            panic!("Expected RuntimeError");
        }
    }

    #[test]
    fn test_assume_truthy_value() {
        
        let result = assume(&[Value::Number(SchemeNumber::Integer(42))]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Undefined);
    }

    #[test]
    fn test_assumption_violation_no_args() {
        
        let result = assumption_violation(&[]);
        assert!(result.is_err());
        if let Err(LambdustError::RuntimeError { message, .. }) = result {
            assert_eq!(message, "assumption violation");
        } else {
            panic!("Expected RuntimeError");
        }
    }

    #[test]
    fn test_assumption_violation_with_who() {
        
        let args = vec![Value::Symbol("test-function".to_string())];
        let result = assumption_violation(&args);
        assert!(result.is_err());
        if let Err(LambdustError::RuntimeError { message, .. }) = result {
            assert!(message.contains("test-function"));
        } else {
            panic!("Expected RuntimeError");
        }
    }

    #[test]
    fn test_assumption_violation_with_message() {
        
        let args = vec![
            Value::Symbol("test-function".to_string()),
            Value::String("invalid input".to_string()),
        ];
        let result = assumption_violation(&args);
        assert!(result.is_err());
        if let Err(LambdustError::RuntimeError { message, .. }) = result {
            assert!(message.contains("test-function"));
            assert!(message.contains("invalid input"));
        } else {
            panic!("Expected RuntimeError");
        }
    }

    #[test]
    fn test_assumption_violation_with_irritants() {
        
        let args = vec![
            Value::Symbol("test-function".to_string()),
            Value::String("invalid input".to_string()),
            Value::Number(SchemeNumber::Integer(-1)),
            Value::String("bad-value".to_string()),
        ];
        let result = assumption_violation(&args);
        assert!(result.is_err());
        if let Err(LambdustError::RuntimeError { message, .. }) = result {
            assert!(message.contains("test-function"));
            assert!(message.contains("invalid input"));
            assert!(message.contains("irritants"));
            assert!(message.contains("-1"));
            assert!(message.contains("bad-value"));
        } else {
            panic!("Expected RuntimeError");
        }
    }

    #[test]
    fn test_srfi_145_module() {
        let module = Srfi145Module::new();
        assert_eq!(module.srfi_id(), 145);
        assert_eq!(module.name(), "SRFI 145");
        assert_eq!(module.parts(), vec!["assumptions"]);
        
        let exports = module.exports();
        assert!(exports.contains_key("assume"));
        assert!(exports.contains_key("assumption-violation"));
        
        // Test exports_for_parts
        let partial_exports = module.exports_for_parts(&["assumptions"]).unwrap();
        assert_eq!(partial_exports.len(), exports.len());
    }
}