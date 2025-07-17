//! Dependent Type System
//! Implementation of dependent types (Π and Σ types)

use super::polynomial_types::PolynomialType;
use crate::value::Value;

/// Dependent type trait
pub trait DependentType {
    /// Get the parameter name
    fn param_name(&self) -> &str;
    /// Get the parameter type
    fn param_type(&self) -> &PolynomialType;
    /// Get the body type (potentially dependent on parameter)
    fn body_type(&self) -> &PolynomialType;
}

/// Pi type (dependent function type): Π(x:A).B(x)
#[derive(Debug, Clone, PartialEq)]
pub struct PiType {
    /// Parameter name
    pub param_name: String,
    /// Parameter type A
    pub param_type: PolynomialType,
    /// Body type B(x) (dependent on parameter)
    pub body_type: PolynomialType,
}

/// Sigma type (dependent sum type): Σ(x:A).B(x)
#[derive(Debug, Clone, PartialEq)]
pub struct SigmaType {
    /// Parameter name
    pub param_name: String,
    /// Parameter type A
    pub param_type: PolynomialType,
    /// Body type B(x) (dependent on parameter)
    pub body_type: PolynomialType,
}

impl DependentType for PiType {
    fn param_name(&self) -> &str {
        &self.param_name
    }

    fn param_type(&self) -> &PolynomialType {
        &self.param_type
    }

    fn body_type(&self) -> &PolynomialType {
        &self.body_type
    }
}

impl DependentType for SigmaType {
    fn param_name(&self) -> &str {
        &self.param_name
    }

    fn param_type(&self) -> &PolynomialType {
        &self.param_type
    }

    fn body_type(&self) -> &PolynomialType {
        &self.body_type
    }
}

impl PiType {
    /// Create new Pi type
    #[must_use] pub fn new(param_name: String, param_type: PolynomialType, body_type: PolynomialType) -> Self {
        Self {
            param_name,
            param_type,
            body_type,
        }
    }

    /// Apply Pi type to an argument (β-reduction)
    pub fn apply(&self, _arg: &Value) -> Result<PolynomialType, crate::error::LambdustError> {
        // Placeholder: would perform substitution [arg/param_name]body_type
        Ok(self.body_type.clone())
    }

    /// Check if this is a simple function type (non-dependent)
    #[must_use] pub fn is_simple_function(&self) -> bool {
        // Check if body_type doesn't mention param_name
        // Placeholder implementation
        false
    }
}

impl SigmaType {
    /// Create new Sigma type
    #[must_use] pub fn new(param_name: String, param_type: PolynomialType, body_type: PolynomialType) -> Self {
        Self {
            param_name,
            param_type,
            body_type,
        }
    }

    /// Get first projection type
    #[must_use] pub fn first_projection_type(&self) -> &PolynomialType {
        &self.param_type
    }

    /// Get second projection type for a given first component
    pub fn second_projection_type(&self, _first_component: &Value) -> Result<PolynomialType, crate::error::LambdustError> {
        // Placeholder: would perform substitution
        Ok(self.body_type.clone())
    }

    /// Check if this is a simple product type (non-dependent)
    #[must_use] pub fn is_simple_product(&self) -> bool {
        // Check if body_type doesn't mention param_name
        // Placeholder implementation
        false
    }
}