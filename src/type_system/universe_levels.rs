//! Universe Level Management
//! 
//! Manages the hierarchy of type universes: Type₀ : Type₁ : Type₂ : ...

use crate::error::LambdustError;
use super::polynomial_types::{PolynomialType, UniverseLevel};
use std::collections::HashMap;

/// Universe hierarchy manager
#[derive(Debug, Clone)]
pub struct UniverseHierarchy {
    /// Maximum universe level currently in use
    max_level: UniverseLevel,
    /// Level constraints and relationships
    level_constraints: HashMap<String, UniverseLevel>,
    /// Type-in-universe cache
    type_universe_cache: HashMap<String, UniverseLevel>,
}

impl UniverseHierarchy {
    /// Create new universe hierarchy
    #[must_use] pub fn new() -> Self {
        Self {
            max_level: UniverseLevel::new(0),
            level_constraints: HashMap::new(),
            type_universe_cache: HashMap::new(),
        }
    }

    /// Get maximum universe level
    #[must_use] pub fn max_level(&self) -> UniverseLevel {
        self.max_level
    }

    /// Add type to specific universe level
    pub fn add_type_at_level(&mut self, type_name: String, level: UniverseLevel) -> Result<(), LambdustError> {
        if level.0 > self.max_level.0 {
            self.max_level = level;
        }
        
        self.type_universe_cache.insert(type_name.clone(), level);
        self.level_constraints.insert(type_name, level);
        Ok(())
    }

    /// Get universe level for a type
    #[must_use] pub fn get_type_level(&self, type_expr: &PolynomialType) -> UniverseLevel {
        type_expr.universe_level()
    }

    /// Check if a type can inhabit a given universe level
    #[must_use] pub fn can_inhabit_level(&self, type_expr: &PolynomialType, level: UniverseLevel) -> bool {
        let type_level = self.get_type_level(type_expr);
        type_level.0 <= level.0
    }

    /// Get the universe level that contains a given type
    #[must_use] pub fn containing_universe(&self, type_expr: &PolynomialType) -> UniverseLevel {
        let type_level = self.get_type_level(type_expr);
        UniverseLevel::new(type_level.0 + 1)
    }

    /// Check universe level consistency
    pub fn check_consistency(&self) -> Result<(), LambdustError> {
        // Check for level inconsistencies
        for (type_name, &level) in &self.level_constraints {
            if level.0 > self.max_level.0 + 1 {
                return Err(LambdustError::type_error(format!(
                    "Type '{}' at level {} exceeds maximum level {}",
                    type_name, level.0, self.max_level.0
                )));
            }
        }
        Ok(())
    }

    /// Infer minimum universe level for a type expression
    pub fn infer_minimum_level(&mut self, type_expr: &PolynomialType) -> UniverseLevel {
        match type_expr {
            PolynomialType::Base(_) => UniverseLevel::new(0),
            PolynomialType::Universe(level) => level.next(),
            PolynomialType::Pi { param_type, body_type, .. } => {
                let param_level = self.infer_minimum_level(param_type);
                let body_level = self.infer_minimum_level(body_type);
                UniverseLevel::new(param_level.0.max(body_level.0))
            },
            PolynomialType::Sigma { param_type, body_type, .. } => {
                let param_level = self.infer_minimum_level(param_type);
                let body_level = self.infer_minimum_level(body_type);
                UniverseLevel::new(param_level.0.max(body_level.0))
            },
            PolynomialType::Function { input, output } => {
                let input_level = self.infer_minimum_level(input);
                let output_level = self.infer_minimum_level(output);
                UniverseLevel::new(input_level.0.max(output_level.0))
            },
            PolynomialType::Product { left, right } => {
                let left_level = self.infer_minimum_level(left);
                let right_level = self.infer_minimum_level(right);
                UniverseLevel::new(left_level.0.max(right_level.0))
            },
            PolynomialType::Sum { left, right } => {
                let left_level = self.infer_minimum_level(left);
                let right_level = self.infer_minimum_level(right);
                UniverseLevel::new(left_level.0.max(right_level.0))
            },
            PolynomialType::List { element_type } => {
                self.infer_minimum_level(element_type)
            },
            PolynomialType::Vector { element_type, length } => {
                let elem_level = self.infer_minimum_level(element_type);
                let length_level = self.infer_minimum_level(length);
                UniverseLevel::new(elem_level.0.max(length_level.0))
            },
            PolynomialType::Polynomial { constructors, .. } => {
                let max_constructor_level = constructors.iter()
                    .map(|c| self.infer_minimum_level(&c.result_type))
                    .max()
                    .unwrap_or(UniverseLevel::new(0));
                max_constructor_level
            },
            PolynomialType::Variable { level, .. } => *level,
            PolynomialType::Application { constructor, argument } => {
                let constructor_level = self.infer_minimum_level(constructor);
                let argument_level = self.infer_minimum_level(argument);
                UniverseLevel::new(constructor_level.0.max(argument_level.0))
            },
            PolynomialType::Identity { base_type, left, right } => {
                let base_level = self.infer_minimum_level(base_type);
                let left_level = self.infer_minimum_level(left);
                let right_level = self.infer_minimum_level(right);
                UniverseLevel::new(base_level.0.max(left_level.0).max(right_level.0))
            },
        }
    }

    /// Update maximum level if needed
    pub fn update_max_level(&mut self, level: UniverseLevel) {
        if level.0 > self.max_level.0 {
            self.max_level = level;
        }
    }

    /// Get all types at a specific level
    #[must_use] pub fn types_at_level(&self, level: UniverseLevel) -> Vec<String> {
        self.level_constraints.iter()
            .filter(|(_, &l)| l == level)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Check if two types are at compatible universe levels
    #[must_use] pub fn levels_compatible(&self, level1: UniverseLevel, level2: UniverseLevel) -> bool {
        // Types are compatible if they're at the same level or one can be lifted
        level1.0.abs_diff(level2.0) <= 1
    }
    
    /// Promote a type to a higher universe level
    pub fn promote_type(&self, poly_type: &PolynomialType, _target_level: u32) -> Result<PolynomialType, crate::error::LambdustError> {
        // Create a new type at the target level
        // For now, just return the original type (placeholder implementation)
        Ok(poly_type.clone())
    }
    
    /// Compute minimum universe level for a collection of types
    pub fn compute_minimum_level(&self, types: &[PolynomialType]) -> Result<u32, crate::error::LambdustError> {
        if types.is_empty() {
            return Ok(0);
        }
        
        let max_level = types.iter()
            .map(|t| self.get_type_level(t).0)
            .max()
            .unwrap_or(0);
            
        Ok(max_level as u32)
    }

    /// Create a new universe level above all existing ones
    pub fn create_new_universe(&mut self) -> UniverseLevel {
        let new_level = UniverseLevel::new(self.max_level.0 + 1);
        self.max_level = new_level;
        new_level
    }
}

impl Default for UniverseHierarchy {
    fn default() -> Self {
        Self::new()
    }
}
