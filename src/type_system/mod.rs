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
#[cfg(feature = "parallel-type-checking")]
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
#[cfg(feature = "parallel-type-checking")]
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
#[derive(Debug, Clone)]
pub struct PolynomialUniverseSystem {
    /// Type checker instance
    type_checker: TypeChecker,
    /// Type inference engine
    type_inference: TypeInference,
    /// Universe hierarchy manager
    universe_hierarchy: UniverseHierarchy,
    /// Monad algebra system
    monad_algebra: MonadAlgebra,
    /// Type class registry (HoTT-based)
    type_class_registry: TypeClassRegistry,
    /// Parallel type checker for high-performance compilation
    #[cfg(feature = "parallel-type-checking")]
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
    #[must_use] pub fn new() -> Self {
        Self {
            type_checker: TypeChecker::new(),
            type_inference: TypeInference::new(),
            universe_hierarchy: UniverseHierarchy::new(),
            monad_algebra: MonadAlgebra::new(),
            type_class_registry: TypeClassRegistry::new(),
            #[cfg(feature = "parallel-type-checking")]
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

    /// Compose two polynomial functors
    /// Mathematical operation: (`P_u` ∘ `P_v)(X`) ≅ P_{u∘v}(X)
    pub fn compose_polynomial_functors(&self, f: &PolynomialFunctor, g: &PolynomialFunctor) -> Result<PolynomialFunctor, LambdustError> {
        let composed_level = f.universe_level.next();
        let mut composed = PolynomialFunctor::new(composed_level);
        
        // Compose constructor sets: for each constructor c_f in f and c_g in g,
        // create composed constructor that applies f then g
        for (name_f, constructor_f) in &f.constructors {
            for (name_g, constructor_g) in &g.constructors {
                let composed_name = format!("{name_f}_{name_g}");
                
                // Compose the constructors appropriately
                let composed_constructor = Constructor {
                    name: composed_name.clone(),
                    arg_types: constructor_f.arg_types.clone(),
                    result_type: Box::new(PolynomialType::Application {
                        constructor: Box::new((*constructor_g.result_type).clone()),
                        argument: Box::new((*constructor_f.result_type).clone()),
                    }),
                };
                
                composed.add_constructor(composed_name, composed_constructor)?;
            }
        }
        
        Ok(composed)
    }

    /// Promote a type to a higher universe level
    /// This implements universe level lifting for type-in-type avoidance
    pub fn promote_type(&self, base_type: &PolynomialType, target_level: usize) -> Result<PolynomialType, LambdustError> {
        match base_type {
            PolynomialType::Base(base) => {
                // Base types can be promoted by wrapping in universe
                Ok(PolynomialType::Variable {
                    name: format!("{base:?}"),
                    level: UniverseLevel::new(target_level),
                })
            }
            PolynomialType::Variable { name, level } => {
                if level.0 < target_level {
                    Ok(PolynomialType::Variable {
                        name: name.clone(),
                        level: UniverseLevel::new(target_level),
                    })
                } else {
                    Ok(base_type.clone())
                }
            }
            _ => {
                // Other types require more sophisticated promotion
                Ok(base_type.clone())
            }
        }
    }

    /// Unify two types using polynomial universe semantics
    /// This implements type unification with universe constraints
    pub fn unify_types(&self, type1: &PolynomialType, type2: &PolynomialType) -> Result<std::collections::HashMap<String, PolynomialType>, LambdustError> {
        let mut substitution = std::collections::HashMap::new();
        
        match (type1, type2) {
            (PolynomialType::Variable { name, .. }, t) | (t, PolynomialType::Variable { name, .. }) => {
                substitution.insert(name.clone(), t.clone());
                Ok(substitution)
            }
            (PolynomialType::Base(b1), PolynomialType::Base(b2)) if b1 == b2 => {
                Ok(substitution)
            }
            (PolynomialType::Function { input: i1, output: o1 }, 
             PolynomialType::Function { input: i2, output: o2 }) => {
                let input_unify = self.unify_types(i1, i2)?;
                let output_unify = self.unify_types(o1, o2)?;
                
                // Merge substitutions
                substitution.extend(input_unify);
                substitution.extend(output_unify);
                Ok(substitution)
            }
            _ => Err(LambdustError::type_error(format!("Cannot unify {type1:?} with {type2:?}"))),
        }
    }

    /// Solve universe constraints
    /// This implements constraint solving for universe polymorphic types
    pub fn solve_universe_constraints(&self, constraints: &[crate::type_system::universe_polymorphic_classes::UniversePolymorphicConstraint]) -> Result<std::collections::HashMap<String, PolynomialType>, LambdustError> {
        let mut solution = std::collections::HashMap::new();
        
        for constraint in constraints {
            if constraint.class_name.is_empty() {
                return Err(LambdustError::type_error("Empty constraint class name".to_string()));
            }
            
            // For now, provide a simple solution
            // Real implementation would use constraint solving algorithms
            solution.insert(
                format!("constraint_{}", constraint.class_name),
                PolynomialType::Base(polynomial_types::BaseType::Unit),
            );
        }
        
        Ok(solution)
    }

    /// Instantiate a polymorphic type with a concrete type
    /// This implements type instantiation for System F style polymorphism
    pub fn instantiate_type(&self, poly_type: &PolynomialType, concrete_type: &PolynomialType) -> Result<PolynomialType, LambdustError> {
        match poly_type {
            PolynomialType::Variable {  .. } => {
                // Substitute type variable with concrete type
                Ok(concrete_type.clone())
            }
            PolynomialType::Function { input, output } => {
                let instantiated_input = self.instantiate_type(input, concrete_type)?;
                let instantiated_output = self.instantiate_type(output, concrete_type)?;
                Ok(PolynomialType::Function {
                    input: Box::new(instantiated_input),
                    output: Box::new(instantiated_output),
                })
            }
            _ => Ok(poly_type.clone()),
        }
    }

    /// Type check multiple expressions in parallel (GHC challenge mode)
    #[cfg(feature = "parallel-type-checking")]
    pub fn type_check_parallel(&mut self, expressions: Vec<(crate::ast::Expr, String)>) -> Result<Vec<parallel_type_checker::TypeCheckResult>, LambdustError> {
        self.parallel_type_checker.type_check_parallel(expressions)
    }

    /// Type check with automatic parallelization decision
    #[cfg(feature = "parallel-type-checking")]
    pub fn type_check_auto(&mut self, expressions: Vec<(crate::ast::Expr, String)>) -> Result<Vec<parallel_type_checker::TypeCheckResult>, LambdustError> {
        self.parallel_type_checker.type_check_auto(expressions)
    }

    /// Get parallel type checking performance metrics
    #[cfg(feature = "parallel-type-checking")]
    pub fn get_parallel_metrics(&self) -> ParallelTypeCheckMetrics {
        self.parallel_type_checker.get_metrics().clone()
    }

    /// Add type binding for parallel type checking
    #[cfg(feature = "parallel-type-checking")]
    pub fn add_parallel_type_binding(&mut self, name: String, typ: PolynomialType) {
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
    
    /// Check universe level consistency using `universe_hierarchy`
    pub fn check_universe_level_consistency(&self, type1: &PolynomialType, type2: &PolynomialType) -> Result<bool, LambdustError> {
        // Use the universe_hierarchy to verify level consistency
        let level1 = self.universe_hierarchy.get_type_level(type1);
        let level2 = self.universe_hierarchy.get_type_level(type2);
        
        // Check if the levels are compatible
        Ok(self.universe_hierarchy.levels_compatible(level1, level2))
    }
    
    /// Promote type to higher universe level
    pub fn promote_type_universe(&mut self, poly_type: &PolynomialType, target_level: u32) -> Result<PolynomialType, LambdustError> {
        self.universe_hierarchy.promote_type(poly_type, target_level)
    }
    
    /// Get minimum universe level for type compatibility
    pub fn get_minimum_universe_level(&self, types: &[PolynomialType]) -> Result<u32, LambdustError> {
        self.universe_hierarchy.compute_minimum_level(types)
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
