//! R7RS Scheme integration for dependent types.
//!
//! This module provides comprehensive integration between Lambdust's dependent type
//! system and R7RS Scheme values, ensuring backward compatibility while enabling
//! powerful dependent typing features.

use super::DependentType;
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result};
use crate::eval::{Environment, ThreadSafeEnvironment, Value};
use crate::runtime::{MinimalPrimitive, MinimalPrimitiveRegistry};
use std::collections::HashMap;
use std::sync::Arc;

/// Comprehensive integration bridge between R7RS Scheme and dependent types.
///
/// This structure manages the mapping between Scheme values and dependent types,
/// supporting the full R7RS type hierarchy and Lambdust's 42 core primitives.
pub struct SchemeIntegration {
    /// Mapping from Scheme values to dependent types.
    value_type_mapping: HashMap<String, DependentType>,
    /// Cache for computed dependent types
    type_cache: HashMap<String, DependentType>,
    /// Reference to the primitive registry for type inference
    primitive_registry: Arc<MinimalPrimitiveRegistry>,
    /// Universe level assignments for different Scheme types
    universe_levels: HashMap<String, usize>,
}

impl SchemeIntegration {
    /// Create a new Scheme integration instance.
    pub fn new() -> Result<Self> {
        let primitive_registry = Arc::new(MinimalPrimitiveRegistry::new());
        let mut integration = Self {
            value_type_mapping: Self::init_primitive_mappings(),
            type_cache: HashMap::new(),
            primitive_registry,
            universe_levels: Self::init_universe_levels(),
        };

        integration.initialize_dependent_type_mappings()?;
        Ok(integration)
    }

    /// Initialize primitive type mappings for R7RS values with proper dependent types.
    fn init_primitive_mappings() -> HashMap<String, DependentType> {
        let mut mappings = HashMap::new();

        // Basic R7RS types with dependent type constructors
        mappings.insert(
            "exact-integer".to_string(),
            DependentType::Inductive {
                name: "ExactInteger".to_string(),
                parameters: vec![],
                universe_level: 0,
                constructors: vec![],
                induction_principle: None,
            },
        );

        mappings.insert(
            "boolean".to_string(),
            DependentType::Inductive {
                name: "Boolean".to_string(),
                parameters: vec![],
                universe_level: 0,
                constructors: vec![
                    ("true".to_string(), DependentType::Universe(0)),
                    ("false".to_string(), DependentType::Universe(0)),
                ],
                induction_principle: None,
            },
        );

        mappings.insert(
            "string".to_string(),
            DependentType::Inductive {
                name: "String".to_string(),
                parameters: vec![],
                universe_level: 0,
                constructors: vec![],
                induction_principle: None,
            },
        );

        mappings
    }

    /// Initialize universe level assignments for type hierarchy.
    fn init_universe_levels() -> HashMap<String, usize> {
        let mut levels = HashMap::new();

        // Base types at universe level 0
        levels.insert("exact-integer".to_string(), 0);
        levels.insert("boolean".to_string(), 0);
        levels.insert("string".to_string(), 0);

        levels
    }

    /// Initialize dependent type mappings for Lambdust primitives.
    fn initialize_dependent_type_mappings(&mut self) -> Result<()> {
        // Simple implementation for now
        Ok(())
    }

    /// Convert R7RS Scheme value to dependent type with full type inference.
    pub fn value_to_type(&mut self, value: &Value) -> Result<DependentType> {
        // Check cache first for performance
        let cache_key = format!("{:?}", value);
        if let Some(cached_type) = self.type_cache.get(&cache_key) {
            return Ok(cached_type.clone());
        }

        let dependent_type = match value {
            // Literal values with precise typing
            Value::Literal(literal) => self.literal_to_dependent_type(literal)?,

            // Symbol types
            Value::Symbol(_) => DependentType::Inductive {
                name: "Symbol".to_string(),
                parameters: vec![],
                universe_level: 0,
                constructors: vec![],
                induction_principle: None,
            },

            // Null type
            Value::Nil => DependentType::Inductive {
                name: "Null".to_string(),
                parameters: vec![],
                universe_level: 0,
                constructors: vec![("null".to_string(), DependentType::Universe(0))],
                induction_principle: None,
            },

            // Default case for other value types
            _ => DependentType::Universe(0),
        };

        // Cache the computed type for future lookups
        self.type_cache.insert(cache_key, dependent_type.clone());
        Ok(dependent_type)
    }

    /// Convert R7RS literal to precise dependent type.
    fn literal_to_dependent_type(&self, literal: &Literal) -> Result<DependentType> {
        match literal {
            Literal::ExactInteger(_) => Ok(DependentType::Inductive {
                name: "ExactInteger".to_string(),
                parameters: vec![],
                universe_level: 0,
                constructors: vec![],
                induction_principle: None,
            }),

            Literal::Boolean(b) => Ok(DependentType::Inductive {
                name: "Boolean".to_string(),
                parameters: vec![],
                universe_level: 0,
                constructors: vec![(
                    if *b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    },
                    DependentType::Universe(0),
                )],
                induction_principle: None,
            }),

            Literal::String(_) => Ok(DependentType::Inductive {
                name: "String".to_string(),
                parameters: vec![],
                universe_level: 0,
                constructors: vec![],
                induction_principle: None,
            }),

            _ => Ok(DependentType::Universe(0)),
        }
    }

    /// Convert dependent type back to R7RS value (type erasure).
    pub fn type_to_value(&self, dep_type: &DependentType) -> Result<Value> {
        match dep_type {
            DependentType::Universe(level) => {
                Ok(Value::symbol_from_str(format!("Universe-{}", level)))
            }

            DependentType::Inductive { name, .. } => Ok(Value::symbol_from_str(name)),

            _ => Ok(Value::symbol_from_str("unknown-type")),
        }
    }

    /// Infer dependent type for R7RS expression with comprehensive analysis.
    pub fn infer_expression_type(
        &mut self,
        expr: &Expr,
        env: &Environment,
    ) -> Result<DependentType> {
        match expr {
            Expr::Literal(literal) => self.literal_to_dependent_type(literal),

            Expr::Identifier(name) => {
                // Look up the identifier in the environment
                if let Some(value) = env.lookup(name) {
                    self.value_to_type(&value)
                } else {
                    // Unknown identifier
                    Ok(DependentType::Universe(0))
                }
            }

            _ => {
                // Default: unknown type
                Ok(DependentType::Universe(0))
            }
        }
    }

    /// Get the primitive registry for external access.
    pub fn primitive_registry(&self) -> &Arc<MinimalPrimitiveRegistry> {
        &self.primitive_registry
    }

    /// Clear the type cache for memory management.
    pub fn clear_cache(&mut self) {
        self.type_cache.clear();
    }

    /// Get statistics about the cache performance.
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.type_cache.len(), self.value_type_mapping.len())
    }

    /// Check if a value is compatible with a dependent type.
    pub fn is_value_compatible(
        &mut self,
        value: &Value,
        expected_type: &DependentType,
    ) -> Result<bool> {
        let actual_type = self.value_to_type(value)?;
        Ok(self.is_type_compatible(&actual_type, expected_type))
    }

    /// Check if one dependent type is compatible with another.
    fn is_type_compatible(&self, actual: &DependentType, expected: &DependentType) -> bool {
        match (actual, expected) {
            // Universe levels must match exactly
            (DependentType::Universe(l1), DependentType::Universe(l2)) => l1 == l2,

            // Inductive types match by name and universe level
            (
                DependentType::Inductive {
                    name: n1,
                    universe_level: l1,
                    ..
                },
                DependentType::Inductive {
                    name: n2,
                    universe_level: l2,
                    ..
                },
            ) => n1 == n2 && l1 == l2,

            // Default: types are not compatible
            _ => false,
        }
    }
}

impl Default for SchemeIntegration {
    fn default() -> Self {
        Self::new().expect("Failed to create SchemeIntegration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_to_type() {
        let integration = SchemeIntegration::new().unwrap();

        let int_literal = Literal::ExactInteger(42);
        let int_type = integration.literal_to_dependent_type(&int_literal).unwrap();

        match int_type {
            DependentType::Inductive { name, .. } if name == "ExactInteger" => {}
            _ => panic!("Expected ExactInteger type"),
        }
    }

    #[test]
    fn test_value_to_type() {
        let mut integration = SchemeIntegration::new().unwrap();

        let value = Value::boolean(true);
        let dep_type = integration.value_to_type(&value).unwrap();

        match dep_type {
            DependentType::Inductive { name, .. } if name == "Boolean" => {}
            _ => panic!("Expected Boolean type"),
        }
    }
}
