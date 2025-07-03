//! SRFI 45: Primitives for Expressing Iterative Lazy Algorithms
//!
//! This SRFI provides primitives for lazy evaluation with proper iterative behavior.

use super::SrfiModule;
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Promise, PromiseState, Value};
use std::collections::HashMap;

/// SRFI 45 implementation
pub struct Srfi45;

impl SrfiModule for Srfi45 {
    fn srfi_id(&self) -> u32 {
        45
    }
    
    fn name(&self) -> &'static str {
        "Primitives for Expressing Iterative Lazy Algorithms"
    }
    
    fn parts(&self) -> Vec<&'static str> {
        vec!["lazy", "promises"]
    }
    
    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Lazy evaluation primitives
        exports.insert("delay".to_string(), delay_function());
        exports.insert("lazy".to_string(), lazy_function());
        exports.insert("force".to_string(), force_function());
        exports.insert("promise?".to_string(), promise_predicate());
        
        exports
    }
    
    fn exports_for_parts(&self, parts: &[&str]) -> Result<HashMap<String, Value>> {
        let all_exports = self.exports();
        let mut filtered = HashMap::new();
        
        for part in parts {
            match *part {
                "lazy" => {
                    // Lazy evaluation functions
                    if let Some(value) = all_exports.get("delay") {
                        filtered.insert("delay".to_string(), value.clone());
                    }
                    if let Some(value) = all_exports.get("lazy") {
                        filtered.insert("lazy".to_string(), value.clone());
                    }
                    if let Some(value) = all_exports.get("force") {
                        filtered.insert("force".to_string(), value.clone());
                    }
                }
                "promises" => {
                    // Promise-related functions
                    if let Some(value) = all_exports.get("promise?") {
                        filtered.insert("promise?".to_string(), value.clone());
                    }
                    if let Some(value) = all_exports.get("force") {
                        filtered.insert("force".to_string(), value.clone());
                    }
                }
                _ => {
                    return Err(LambdustError::runtime_error(
                        format!("Unknown SRFI 45 part: {}", part)
                    ));
                }
            }
        }
        
        Ok(filtered)
    }
}

// Lazy evaluation implementations (migrated from builtins/lazy.rs)

/// Creates a delayed computation (basic delay)
fn delay_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "delay".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            // For now, create a simple lazy promise
            // In a full implementation, this would need access to the expression and environment
            let promise = Promise {
                state: PromiseState::Lazy {
                    expr: crate::ast::Expr::Literal(crate::ast::Literal::Nil), // Placeholder
                    env: std::rc::Rc::new(crate::environment::Environment::new()),
                },
            };

            Ok(Value::Promise(promise))
        },
    })
}

/// Creates a lazy computation (SRFI 45 extension)
fn lazy_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "lazy".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            // Create a lazy promise (similar to delay but with different semantics)
            let promise = Promise {
                state: PromiseState::Lazy {
                    expr: crate::ast::Expr::Literal(crate::ast::Literal::Nil), // Placeholder
                    env: std::rc::Rc::new(crate::environment::Environment::new()),
                },
            };

            Ok(Value::Promise(promise))
        },
    })
}

/// Forces evaluation of a promise
fn force_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "force".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match &args[0] {
                Value::Promise(promise) => {
                    match &promise.state {
                        PromiseState::Lazy { .. } => {
                            // For now, return a placeholder
                            // Full implementation would evaluate the expression
                            Ok(Value::Undefined)
                        }
                        PromiseState::Eager { value } => {
                            Ok((**value).clone())
                        }
                    }
                }
                // If not a promise, return as-is (R7RS behavior)
                value => Ok(value.clone()),
            }
        },
    })
}

/// Checks if a value is a promise
fn promise_predicate() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "promise?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            Ok(Value::Boolean(matches!(args[0], Value::Promise(_))))
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_srfi_45_info() {
        let srfi45 = Srfi45;
        assert_eq!(srfi45.srfi_id(), 45);
        assert_eq!(srfi45.name(), "Primitives for Expressing Iterative Lazy Algorithms");
        assert!(srfi45.parts().contains(&"lazy"));
        assert!(srfi45.parts().contains(&"promises"));
    }
    
    #[test]
    fn test_srfi_45_exports() {
        let srfi45 = Srfi45;
        let exports = srfi45.exports();
        
        assert!(exports.contains_key("delay"));
        assert!(exports.contains_key("lazy"));
        assert!(exports.contains_key("force"));
        assert!(exports.contains_key("promise?"));
    }
    
    #[test]
    fn test_promise_predicate() {
        let promise_pred = promise_predicate();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = promise_pred {
            // Test with non-promise
            let result = func(&[Value::Number(crate::lexer::SchemeNumber::Integer(42))]).unwrap();
            assert_eq!(result, Value::Boolean(false));
        }
    }
}