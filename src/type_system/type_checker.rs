//! Type Checker Implementation
//! Type checking for polynomial universe type system

use super::polynomial_types::{PolynomialType, BaseType, UniverseLevel};
use crate::value::Value;
use crate::lexer::SchemeNumber;
use std::collections::HashMap;

/// Type check result
#[derive(Debug, Clone, PartialEq)]
pub struct TypeCheckResult {
    /// Whether the type check succeeded
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Inferred type information
    pub inferred_type: Option<PolynomialType>,
}

/// Type checking context
#[derive(Debug, Clone)]
pub struct TypeContext {
    /// Variable type bindings
    variables: HashMap<String, PolynomialType>,
    /// Universe level context
    universe_level: UniverseLevel,
}

impl TypeContext {
    /// Create new empty context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            universe_level: UniverseLevel::new(0),
        }
    }

    /// Add variable binding
    pub fn bind(&mut self, name: String, typ: PolynomialType) {
        self.variables.insert(name, typ);
    }

    /// Look up variable type
    pub fn lookup(&self, name: &str) -> Option<&PolynomialType> {
        self.variables.get(name)
    }

    /// Get current universe level
    pub fn universe_level(&self) -> UniverseLevel {
        self.universe_level
    }

    /// Set universe level
    pub fn set_universe_level(&mut self, level: UniverseLevel) {
        self.universe_level = level;
    }
}

/// Type checker
#[derive(Debug)]
pub struct TypeChecker {
    /// Type checking context
    context: TypeContext,
}

impl TypeChecker {
    /// Create new type checker
    pub fn new() -> Self {
        Self {
            context: TypeContext::new(),
        }
    }

    /// Type check a value against a type
    pub fn check(&mut self, value: &Value, expected_type: &PolynomialType) -> Result<TypeCheckResult, crate::error::LambdustError> {
        match self.infer_value_type(value) {
            Ok(inferred_type) => {
                if self.types_compatible(&inferred_type, expected_type)? {
                    Ok(TypeCheckResult {
                        success: true,
                        error_message: None,
                        inferred_type: Some(inferred_type),
                    })
                } else {
                    Ok(TypeCheckResult {
                        success: false,
                        error_message: Some(format!("Type mismatch: expected {:?}, got {:?}", expected_type, inferred_type)),
                        inferred_type: Some(inferred_type),
                    })
                }
            }
            Err(e) => Ok(TypeCheckResult {
                success: false,
                error_message: Some(format!("Type inference failed: {}", e)),
                inferred_type: None,
            })
        }
    }

    /// Check type equivalence
    pub fn equivalent(&self, type1: &PolynomialType, type2: &PolynomialType) -> Result<bool, crate::error::LambdustError> {
        self.types_compatible(type1, type2)
    }

    /// Infer the type of a value
    pub fn infer_value_type(&self, value: &Value) -> Result<PolynomialType, crate::error::LambdustError> {
        match value {
            Value::Number(SchemeNumber::Integer(_)) => {
                Ok(PolynomialType::Base(BaseType::Integer))
            }
            Value::Number(SchemeNumber::Real(_)) => {
                Ok(PolynomialType::Base(BaseType::Real))
            }
            Value::Number(SchemeNumber::Rational(_, _)) => {
                Ok(PolynomialType::Base(BaseType::Real)) // Treat rationals as reals
            }
            Value::Boolean(_) => {
                Ok(PolynomialType::Base(BaseType::Boolean))
            }
            Value::String(_) => {
                Ok(PolynomialType::Base(BaseType::String))
            }
            Value::Character(_) => {
                Ok(PolynomialType::Base(BaseType::Character))
            }
            Value::Symbol(_) => {
                Ok(PolynomialType::Base(BaseType::Symbol))
            }
            Value::Nil => {
                // Nil can be typed as a list of any type
                Ok(PolynomialType::List {
                    element_type: Box::new(PolynomialType::Variable {
                        name: "α".to_string(),
                        level: UniverseLevel::new(0),
                    })
                })
            }
            Value::Pair(_) => {
                // For now, treat pairs as lists
                Ok(PolynomialType::List {
                    element_type: Box::new(PolynomialType::Variable {
                        name: "α".to_string(),
                        level: UniverseLevel::new(0),
                    })
                })
            }
            Value::Procedure(_) => {
                // Generic function type
                Ok(PolynomialType::Function {
                    input: Box::new(PolynomialType::Variable {
                        name: "α".to_string(),
                        level: UniverseLevel::new(0),
                    }),
                    output: Box::new(PolynomialType::Variable {
                        name: "β".to_string(),
                        level: UniverseLevel::new(0),
                    }),
                })
            }
            _ => {
                // For other values, use a generic type
                Ok(PolynomialType::Variable {
                    name: "unknown".to_string(),
                    level: UniverseLevel::new(0),
                })
            }
        }
    }

    /// Check if two types are compatible (including subtyping)
    pub fn types_compatible(&self, type1: &PolynomialType, type2: &PolynomialType) -> Result<bool, crate::error::LambdustError> {
        match (type1, type2) {
            // Exact equality
            (t1, t2) if t1 == t2 => Ok(true),
            
            // Base type compatibility
            (PolynomialType::Base(b1), PolynomialType::Base(b2)) => {
                Ok(self.base_types_compatible(b1, b2))
            }
            
            // Function type compatibility (contravariant in input, covariant in output)
            (PolynomialType::Function { input: i1, output: o1 }, 
             PolynomialType::Function { input: i2, output: o2 }) => {
                let input_compatible = self.types_compatible(i2, i1)?; // Contravariant
                let output_compatible = self.types_compatible(o1, o2)?; // Covariant
                Ok(input_compatible && output_compatible)
            }
            
            // List type compatibility (covariant in element type)
            (PolynomialType::List { element_type: e1 }, 
             PolynomialType::List { element_type: e2 }) => {
                self.types_compatible(e1, e2)
            }
            
            // Product type compatibility (covariant in both components)
            (PolynomialType::Product { left: l1, right: r1 }, 
             PolynomialType::Product { left: l2, right: r2 }) => {
                let left_compatible = self.types_compatible(l1, l2)?;
                let right_compatible = self.types_compatible(r1, r2)?;
                Ok(left_compatible && right_compatible)
            }
            
            // Type variables are compatible with anything
            (PolynomialType::Variable { .. }, _) => Ok(true),
            (_, PolynomialType::Variable { .. }) => Ok(true),
            
            // Universe compatibility
            (PolynomialType::Universe(l1), PolynomialType::Universe(l2)) => {
                Ok(l1 <= l2) // Lower universes can be embedded in higher ones
            }
            
            // Identity types (simplified: always compatible for now)
            (PolynomialType::Identity { .. }, PolynomialType::Identity { .. }) => Ok(true),
            
            // Default: incompatible
            _ => Ok(false),
        }
    }

    /// Check base type compatibility
    fn base_types_compatible(&self, type1: &BaseType, type2: &BaseType) -> bool {
        match (type1, type2) {
            // Exact match
            (t1, t2) if t1 == t2 => true,
            
            // Numeric type compatibility
            (BaseType::Natural, BaseType::Integer) => true,
            (BaseType::Natural, BaseType::Real) => true,
            (BaseType::Integer, BaseType::Real) => true,
            
            // Default: incompatible
            _ => false,
        }
    }

    /// Add variable to context
    pub fn bind_variable(&mut self, name: String, typ: PolynomialType) {
        self.context.bind(name, typ);
    }

    /// Look up variable type
    pub fn lookup_variable(&self, name: &str) -> Option<&PolynomialType> {
        self.context.lookup(name)
    }

    /// Set universe level
    pub fn set_universe_level(&mut self, level: UniverseLevel) {
        self.context.set_universe_level(level);
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_basic_type_checking() {
        let mut checker = TypeChecker::new();
        
        let int_value = Value::Number(SchemeNumber::Integer(42));
        let int_type = PolynomialType::Base(BaseType::Integer);
        
        let result = checker.check(&int_value, &int_type).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_type_compatibility() {
        let checker = TypeChecker::new();
        
        let nat_type = PolynomialType::Base(BaseType::Natural);
        let int_type = PolynomialType::Base(BaseType::Integer);
        
        assert!(checker.types_compatible(&nat_type, &int_type).unwrap());
        assert!(!checker.types_compatible(&int_type, &nat_type).unwrap());
    }

    #[test]
    fn test_function_type_compatibility() {
        let checker = TypeChecker::new();
        
        let nat_type = PolynomialType::Base(BaseType::Natural);
        let int_type = PolynomialType::Base(BaseType::Integer);
        
        let func1 = PolynomialType::Function {
            input: Box::new(int_type.clone()),
            output: Box::new(nat_type.clone()),
        };
        
        let func2 = PolynomialType::Function {
            input: Box::new(nat_type.clone()),
            output: Box::new(int_type.clone()),
        };
        
        // func1 is compatible with func2 (contravariant input, covariant output)
        assert!(checker.types_compatible(&func1, &func2).unwrap());
    }

    #[test]
    fn test_type_inference() {
        let checker = TypeChecker::new();
        
        let int_value = Value::Number(SchemeNumber::Integer(42));
        let inferred = checker.infer_value_type(&int_value).unwrap();
        
        assert_eq!(inferred, PolynomialType::Base(BaseType::Integer));
    }
}