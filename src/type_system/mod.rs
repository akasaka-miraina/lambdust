//! Polynomial Universe Type System
//! 
//! Based on arXiv:2409.19176 "Polynomial Universes in Homotopy Type Theory"
//! Implements dependent type theory with monad algebra and distributive laws

pub mod polynomial_types;
pub mod monad_algebra;
pub mod homotopy_types;
pub mod hott_types;
pub mod natural_models;
pub mod dependent_types;
pub mod type_checker;
pub mod type_inference;
pub mod universe_levels;
pub mod parallel_type_checker;
pub mod incremental_inference;
pub mod universe_polymorphic_classes;
pub mod standard_universe_classes;
pub mod monad_transformers;
pub mod standard_transformers;

// Re-export key types
pub use polynomial_types::{PolynomialType, PolynomialFunctor, Constructor};
pub use monad_algebra::{MonadStructure, DistributiveLaw, MonadAlgebra};
pub use homotopy_types::{HoTTType, HigherStructure, UnivalenceAxiom};
pub use hott_types::{TypeClassRegistry, TypeClassDefinition, TypeClassInstance};
pub use natural_models::{NaturalModel, UniverseFunction};
pub use dependent_types::{DependentType, PiType, SigmaType};
pub use type_checker::{TypeChecker, TypeCheckResult};
pub use type_inference::{TypeInference, InferenceContext};
pub use polynomial_types::UniverseLevel;
pub use universe_levels::UniverseHierarchy;
pub use parallel_type_checker::{ParallelTypeChecker, ParallelTypeCheckMetrics};
pub use incremental_inference::{IncrementalTypeInference, IncrementalConfig, CacheStatistics};
pub use universe_polymorphic_classes::{
    UniversePolymorphicRegistry, UniversePolymorphicClass, UniversePolymorphicInstance,
    UniverseConstraint, UniversePolymorphicType, UniverseExpression
};
pub use monad_transformers::{
    MonadTransformerRegistry, MonadTransformer, TransformerStack, TransformerInstance,
    MonadTransformerType, CompositionAnalysis
};

use crate::value::Value;
use crate::error::LambdustError;

/// Main type system interface
pub struct PolynomialUniverseSystem {
    /// Type checker instance
    type_checker: TypeChecker,
    /// Type inference engine
    type_inference: TypeInference,
    /// Universe hierarchy manager
    #[allow(dead_code)]
    universe_hierarchy: UniverseHierarchy,
    /// Monad algebra system
    monad_algebra: MonadAlgebra,
    /// Type class registry (HoTT-based)
    type_class_registry: TypeClassRegistry,
    /// Parallel type checker for high-performance compilation
    parallel_type_checker: ParallelTypeChecker,
    /// Incremental type inference with caching
    incremental_inference: IncrementalTypeInference,
    /// Universe polymorphic type class registry
    universe_polymorphic_registry: UniversePolymorphicRegistry,
    /// Monad transformer registry
    monad_transformer_registry: MonadTransformerRegistry,
}

impl PolynomialUniverseSystem {
    /// Create new polynomial universe type system
    pub fn new() -> Self {
        Self {
            type_checker: TypeChecker::new(),
            type_inference: TypeInference::new(),
            universe_hierarchy: UniverseHierarchy::new(),
            monad_algebra: MonadAlgebra::new(),
            type_class_registry: TypeClassRegistry::new(),
            parallel_type_checker: ParallelTypeChecker::default(),
            incremental_inference: IncrementalTypeInference::default(),
            universe_polymorphic_registry: UniversePolymorphicRegistry::default(),
            monad_transformer_registry: MonadTransformerRegistry::default(),
        }
    }

    /// Type check a value against a type
    pub fn type_check(&mut self, value: &Value, expected_type: &PolynomialType) -> Result<TypeCheckResult, LambdustError> {
        self.type_checker.check(value, expected_type)
    }

    /// Infer the type of a value
    pub fn infer_type(&mut self, value: &Value) -> Result<PolynomialType, LambdustError> {
        self.type_inference.infer(value)
    }

    /// Check if two types are equivalent
    pub fn types_equivalent(&self, type1: &PolynomialType, type2: &PolynomialType) -> Result<bool, LambdustError> {
        self.type_checker.equivalent(type1, type2)
    }

    /// Apply distributive law between two monads
    pub fn apply_distributive_law(&mut self, monad1: &str, monad2: &str, value: &Value) -> Result<Value, LambdustError> {
        self.monad_algebra.apply_distributive_law_between(monad1, monad2, value)
    }

    /// Register a type class
    pub fn register_type_class(&mut self, class: TypeClassDefinition) -> Result<(), LambdustError> {
        self.type_class_registry.register_class(class)
    }

    /// Register a type class instance
    pub fn register_type_class_instance(&mut self, instance: TypeClassInstance) -> Result<(), LambdustError> {
        self.type_class_registry.register_instance(instance)
    }

    /// Resolve type class instance
    pub fn resolve_type_class(&mut self, class_name: &str, instance_type: &PolynomialType) -> Result<TypeClassInstance, LambdustError> {
        self.type_class_registry.resolve_instance(class_name, instance_type)
    }

    /// Type check multiple expressions in parallel (GHC challenge mode)
    pub fn type_check_parallel(&self, expressions: Vec<(crate::ast::Expr, String)>) -> Result<Vec<parallel_type_checker::TypeCheckResult>, LambdustError> {
        self.parallel_type_checker.type_check_parallel(expressions)
    }

    /// Type check with automatic parallelization decision
    pub fn type_check_auto(&self, expressions: Vec<(crate::ast::Expr, String)>) -> Result<Vec<parallel_type_checker::TypeCheckResult>, LambdustError> {
        self.parallel_type_checker.type_check_auto(expressions)
    }

    /// Get parallel type checking performance metrics
    pub fn get_parallel_metrics(&self) -> ParallelTypeCheckMetrics {
        self.parallel_type_checker.get_metrics()
    }

    /// Add type binding for parallel type checking
    pub fn add_parallel_type_binding(&self, name: String, typ: PolynomialType) {
        self.parallel_type_checker.add_type_binding(name, typ);
    }

    /// Infer type with incremental caching (GHC challenge mode)
    pub fn infer_incremental(&mut self, value: &Value, context_hint: Option<&str>) -> Result<PolynomialType, LambdustError> {
        self.incremental_inference.infer(value, context_hint)
    }

    /// Infer expression type with incremental caching
    pub fn infer_expression_incremental(&mut self, expr: &crate::ast::Expr, context_hint: Option<&str>) -> Result<PolynomialType, LambdustError> {
        self.incremental_inference.infer_expression(expr, context_hint)
    }

    /// Invalidate cache when symbols change
    pub fn invalidate_incremental_cache(&mut self, changed_symbol: &str) -> Result<u64, LambdustError> {
        self.incremental_inference.invalidate_dependencies(changed_symbol)
    }

    /// Get incremental inference cache statistics
    pub fn get_incremental_stats(&self) -> CacheStatistics {
        self.incremental_inference.get_statistics()
    }

    /// Add dependency for incremental inference
    pub fn add_incremental_dependency(&mut self, symbol: String, dependency: String) {
        self.incremental_inference.add_dependency(symbol, dependency);
    }

    /// Register universe polymorphic class
    pub fn register_universe_polymorphic_class(&mut self, class: UniversePolymorphicClass) -> Result<(), LambdustError> {
        self.universe_polymorphic_registry.register_class(class)
    }

    /// Register universe polymorphic instance
    pub fn register_universe_polymorphic_instance(&mut self, instance: UniversePolymorphicInstance) -> Result<(), LambdustError> {
        self.universe_polymorphic_registry.register_instance(instance)
    }

    /// Resolve universe polymorphic instance
    pub fn resolve_universe_polymorphic_instance(
        &self,
        class_name: &str,
        type_args: &[PolynomialType],
        universe_level: UniverseLevel,
    ) -> Result<UniversePolymorphicInstance, LambdustError> {
        self.universe_polymorphic_registry.resolve_instance(class_name, type_args, universe_level)
    }

    /// Get universe polymorphic class definition
    pub fn get_universe_polymorphic_class(&self, name: &str) -> Option<UniversePolymorphicClass> {
        self.universe_polymorphic_registry.get_class(name)
    }

    /// List all universe polymorphic classes
    pub fn list_universe_polymorphic_classes(&self) -> Vec<String> {
        self.universe_polymorphic_registry.list_classes()
    }

    /// Initialize standard universe polymorphic classes (Functor, Applicative, Monad)
    pub fn initialize_standard_universe_classes(&self) -> Result<(), LambdustError> {
        standard_universe_classes::initialize_standard_classes(&self.universe_polymorphic_registry)
    }

    /// Register monad transformer
    pub fn register_monad_transformer(&mut self, transformer: MonadTransformer) -> Result<(), LambdustError> {
        self.monad_transformer_registry.register_transformer(transformer)
    }

    /// Register transformer instance
    pub fn register_transformer_instance(&mut self, instance: TransformerInstance) -> Result<(), LambdustError> {
        self.monad_transformer_registry.register_instance(instance)
    }

    /// Build transformer stack
    pub fn build_transformer_stack(&self, transformers: Vec<String>, base_monad: PolynomialType) -> Result<TransformerStack, LambdustError> {
        self.monad_transformer_registry.build_stack(transformers, base_monad)
    }

    /// Analyze transformer composition
    pub fn analyze_transformer_composition(&self, stack: &TransformerStack) -> Result<CompositionAnalysis, LambdustError> {
        self.monad_transformer_registry.analyze_composition(stack)
    }

    /// Resolve transformer instance
    pub fn resolve_transformer_instance(&self, transformer: &str, base_monad: &PolynomialType) -> Result<TransformerInstance, LambdustError> {
        self.monad_transformer_registry.resolve_instance(transformer, base_monad)
    }

    /// Get transformer definition
    pub fn get_monad_transformer(&self, name: &str) -> Option<MonadTransformer> {
        self.monad_transformer_registry.get_transformer(name)
    }

    /// List all monad transformers
    pub fn list_monad_transformers(&self) -> Vec<String> {
        self.monad_transformer_registry.list_transformers()
    }

    /// Initialize standard monad transformers
    pub fn initialize_standard_transformers(&self) -> Result<(), LambdustError> {
        standard_transformers::initialize_standard_transformers(&self.monad_transformer_registry)
    }
}

impl Default for PolynomialUniverseSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polynomial_universe_system_creation() {
        let system = PolynomialUniverseSystem::new();
        // Basic creation should work without errors
        assert!(system.universe_hierarchy.max_level() >= UniverseLevel::new(0));
    }

    #[test]
    fn test_type_system_integration() {
        let mut system = PolynomialUniverseSystem::new();
        
        // Test basic type operations
        let nat_type = PolynomialType::Base(polynomial_types::BaseType::Natural);
        let value = Value::Number(crate::lexer::SchemeNumber::Integer(42));
        
        let result = system.type_check(&value, &nat_type);
        assert!(result.is_ok());
    }

    #[test]
    fn test_universe_polymorphic_integration() {
        let system = PolynomialUniverseSystem::new();
        
        // Test that universe polymorphic registry is accessible
        let classes = system.list_universe_polymorphic_classes();
        assert!(classes.is_empty()); // Should start empty
        
        // Test that we can access the registry
        let functor_class = system.get_universe_polymorphic_class("Functor");
        assert!(functor_class.is_none()); // Should not exist yet
    }
}