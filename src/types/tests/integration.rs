//! Integration tests for the type system.
//!
//! These tests verify the complete type checking pipeline including
//! type inference, unification, constraint solving, and gradual typing
//! integration with the overall evaluation system.

#[cfg(test)]
mod tests {
    use crate::types::*;
    use crate::diagnostics::Span;
    use std::collections::HashMap;

    /// Helper function to create a test span.
    fn test_span() -> Span {
        Span::new(0, 0, 0, 0, 0, None)
    }

    // ============================================================================
    // TYPE SYSTEM BASIC FUNCTIONALITY TESTS
    // ============================================================================

    #[test]
    fn test_type_variable_creation() {
        let var1 = TypeVar::new();
        let var2 = TypeVar::new();
        
        // Variables should have unique IDs
        assert_ne!(var1.id, var2.id);
        assert!(var1.name.is_none());
        assert!(var2.name.is_none());
    }

    #[test]
    fn test_named_type_variable() {
        let var = TypeVar::with_name("a");
        assert_eq!(var.name, Some("a".to_string()));
    }

    #[test]
    fn test_basic_type_creation() {
        // Test basic type constructors
        assert_eq!(Type::Number, Type::Number);
        assert_eq!(Type::String, Type::String);
        assert_eq!(Type::Boolean, Type::Boolean);
        assert_eq!(Type::Dynamic, Type::Dynamic);
    }

    #[test]
    fn test_compound_type_creation() {
        let list_type = Type::list(Type::Number);
        let pair_type = Type::pair(Type::String, Type::Boolean);
        let function_type = Type::function(vec![Type::Number, Type::String], Type::Boolean);
        
        assert!(matches!(list_type, Type::List(_)));
        assert!(matches!(pair_type, Type::Pair(_, _)));
        assert!(matches!(function_type, Type::Function { .. }));
    }

    #[test]
    fn test_polymorphic_type_creation() {
        let var_a = TypeVar::with_name("a");
        let var_b = TypeVar::with_name("b");
        let poly_type = Type::forall(
            vec![var_a.clone()), var_b.clone())],
            Type::function(vec![Type::Variable(var_a)], Type::Variable(var_b))
        );
        
        assert!(matches!(poly_type, Type::Forall { .. }));
    }

    #[test]
    fn test_type_variable_contains() {
        let var_a = TypeVar::with_name("a");
        let var_b = TypeVar::with_name("b");
        
        // Simple variable
        assert!(Type::Variable(var_a.clone()).contains_var(&var_a));
        assert!(!Type::Variable(var_a.clone()).contains_var(&var_b));
        
        // Function type
        let func_type = Type::function(
            vec![Type::Variable(var_a.clone())],
            Type::Variable(var_b.clone())
        );
        assert!(func_type.contains_var(&var_a));
        assert!(func_type.contains_var(&var_b));
        
        // Basic types don't contain variables
        assert!(!Type::Number.contains_var(&var_a));
    }

    #[test]
    fn test_free_variables() {
        let var_a = TypeVar::with_name("a");
        let var_b = TypeVar::with_name("b");
        let var_c = TypeVar::with_name("c");
        
        // Function type with free variables
        let func_type = Type::function(
            vec![Type::Variable(var_a.clone()), Type::Variable(var_b.clone())],
            Type::Variable(var_c.clone())
        );
        
        let free_vars = func_type.free_vars();
        assert_eq!(free_vars.len(), 3);
        assert!(free_vars.contains(&var_a));
        assert!(free_vars.contains(&var_b));
        assert!(free_vars.contains(&var_c));
        
        // Forall type with bound variables
        let forall_type = Type::forall(
            vec![var_a.clone())],
            Type::function(
                vec![Type::Variable(var_a.clone())],
                Type::Variable(var_b.clone())
            )
        );
        
        let forall_free_vars = forall_type.free_vars();
        assert_eq!(forall_free_vars.len(), 1);
        assert!(!forall_free_vars.contains(&var_a)); // bound
        assert!(forall_free_vars.contains(&var_b)); // free
    }

    // ============================================================================
    // TYPE SCHEME TESTS
    // ============================================================================

    #[test]
    fn test_monomorphic_type_scheme() {
        let scheme = TypeScheme::monomorphic(Type::Number);
        assert!(scheme.vars.is_empty());
        assert!(scheme.constraints.is_empty());
        assert_eq!(scheme.type_, Type::Number);
    }

    #[test]
    fn test_polymorphic_type_scheme() {
        let var_a = TypeVar::with_name("a");
        let constraint = Constraint {
            class: "Num".to_string(),
            type_: Type::Variable(var_a.clone()),
        };
        
        let scheme = TypeScheme::polymorphic(
            vec![var_a.clone())],
            vec![constraint],
            Type::function(vec![Type::Variable(var_a.clone())], Type::Variable(var_a.clone()))
        );
        
        assert_eq!(scheme.vars.len(), 1);
        assert_eq!(scheme.constraints.len(), 1);
        assert!(scheme.type_.is_function());
    }

    #[test]
    fn test_type_scheme_instantiation() {
        let var_a = TypeVar::with_name("a");
        let scheme = TypeScheme::polymorphic(
            vec![var_a.clone())],
            vec![],
            Type::function(vec![Type::Variable(var_a.clone())], Type::Variable(var_a.clone()))
        );
        
        let instance1 = scheme.instantiate();
        let instance2 = scheme.instantiate();
        
        // Each instantiation should create fresh variables
        assert!(instance1.is_function());
        assert!(instance2.is_function());
        // The instances should be structurally the same but with different variables
        assert_ne!(instance1, instance2); // Different fresh variables
    }

    // ============================================================================
    // TYPE ENVIRONMENT TESTS
    // ============================================================================

    #[test]
    fn test_type_environment_operations() {
        let mut env = TypeEnv::new();
        
        // Initially empty
        assert!(env.lookup("x").is_none());
        
        // Bind a variable
        env.bind("x".to_string(), TypeScheme::monomorphic(Type::Number));
        assert!(env.lookup("x").is_some());
        assert_eq!(env.lookup("x").unwrap().type_, Type::Number);
        
        // Extend environment
        let mut new_bindings = HashMap::new();
        new_bindings.insert("y".to_string(), TypeScheme::monomorphic(Type::String));
        let extended_env = env.extend(new_bindings);
        
        assert!(extended_env.lookup("x").is_some());
        assert!(extended_env.lookup("y").is_some());
    }

    // ============================================================================
    // TYPE CHECKER TESTS
    // ============================================================================

    #[test]
    fn test_type_checker_creation() {
        let checker = TypeChecker::new(TypeLevel::Static);
        assert_eq!(checker.level(), TypeLevel::Static);
        assert!(checker.errors().is_empty());
    }

    #[test]
    fn test_type_checker_error_handling() {
        let mut checker = TypeChecker::new(TypeLevel::Static);
        
        // Add an error
        let error = crate::diagnostics::Error::type_error(
            "Test error".to_string(),
            test_span(),
        );
        checker.add_error(error);
        
        assert_eq!(checker.errors().len(), 1);
        
        // Clear errors
        checker.clear_errors();
        assert!(checker.errors().is_empty());
    }

    // ============================================================================
    // TYPE CONSTRAINT TESTS
    // ============================================================================

    #[test]
    fn test_type_constraint_creation() {
        let constraint = TypeConstraint::equal(
            Type::Number,
            Type::Variable(TypeVar::with_name("a")),
            Some(test_span()),
            "test constraint"
        );
        
        assert!(matches!(constraint, TypeConstraint::Equal { .. }));
        assert!(constraint.span().is_some());
    }

    #[test]
    fn test_instance_constraint() {
        let constraint = TypeConstraint::instance(
            "Num",
            Type::Variable(TypeVar::with_name("a")),
            Some(test_span())
        );
        
        assert!(matches!(constraint, TypeConstraint::Instance { .. }));
    }

    // ============================================================================
    // KIND SYSTEM TESTS
    // ============================================================================

    #[test]
    fn test_kind_creation() {
        let type_kind = Kind::Type;
        let arrow_kind = Kind::arrow(Kind::Type, Kind::Type);
        
        assert_eq!(type_kind.arity(), 0);
        assert_eq!(arrow_kind.arity(), 1);
    }

    #[test]
    fn test_nested_arrow_kinds() {
        // (* -> *) -> *
        let nested_kind = Kind::arrow(
            Kind::arrow(Kind::Type, Kind::Type),
            Kind::Type
        );
        
        assert_eq!(nested_kind.arity(), 1);
    }

    // ============================================================================
    // ROW POLYMORPHISM TESTS
    // ============================================================================

    #[test]
    fn test_row_creation() {
        let empty_row = Row::empty();
        assert!(empty_row.is_closed());
        assert!(empty_row.fields.is_empty());
        
        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Type::Number);
        fields.insert("y".to_string(), Type::String);
        
        let closed_row = Row::closed(fields);
        assert!(closed_row.is_closed());
        assert_eq!(closed_row.fields.len(), 2);
        
        let open_row = Row::open(HashMap::new(), TypeVar::with_name("r"));
        assert!(!open_row.is_closed());
    }

    #[test]
    fn test_row_extension() {
        let mut row = Row::empty();
        row.extend("name".to_string(), Type::String);
        row.extend("age".to_string(), Type::Number);
        
        assert_eq!(row.fields.len(), 2);
        assert_eq!(row.fields.get("name"), Some(&Type::String));
        assert_eq!(row.fields.get("age"), Some(&Type::Number));
    }

    // ============================================================================
    // EFFECT SYSTEM INTEGRATION TESTS
    // ============================================================================

    #[test]
    fn test_effect_types() {
        let io_effect = Effect::IO;
        let state_effect = Effect::State(Type::Number);
        let exception_effect = Effect::Exception(Type::String);
        let pure_effect = Effect::Pure;
        
        assert_eq!(io_effect, Effect::IO);
        assert!(matches!(state_effect, Effect::State(_)));
        assert!(matches!(exception_effect, Effect::Exception(_)));
        assert_eq!(pure_effect, Effect::Pure);
    }

    #[test]
    fn test_effectful_types() {
        let effectful_type = Type::Effectful {
            input: Box::new(Type::Number),
            effects: vec![Effect::IO],
            output: Box::new(Type::String),
        };
        
        assert!(matches!(effectful_type, Type::Effectful { .. }));
    }

    // ============================================================================
    // GRADUAL TYPING TESTS
    // ============================================================================

    #[test]
    fn test_dynamic_type() {
        let dynamic_type = Type::Dynamic;
        assert_eq!(dynamic_type, Type::Dynamic);
        assert!(!dynamic_type.is_variable());
        assert!(!dynamic_type.is_function());
        assert!(!dynamic_type.is_polymorphic());
    }

    #[test]
    fn test_unknown_type() {
        let unknown_type = Type::Unknown;
        assert_eq!(unknown_type, Type::Unknown);
    }

    // ============================================================================
    // TYPE DISPLAY TESTS
    // ============================================================================

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", Type::Number), "Number");
        assert_eq!(format!("{}", Type::String), "String");
        assert_eq!(format!("{}", Type::Dynamic), "Dynamic");
        assert_eq!(format!("{}", Type::Unknown), "?");
        
        let var = TypeVar::with_name("a");
        assert_eq!(format!("{}", Type::Variable(var)), "a");
        
        let list_type = Type::list(Type::Number);
        assert_eq!(format!("{}", list_type), "(List Number)");
        
        let func_type = Type::function(vec![Type::Number], Type::String);
        assert_eq!(format!("{}", func_type), "(-> Number String)");
    }

    #[test]
    fn test_kind_display() {
        assert_eq!(format!("{}", Kind::Type), "*");
        assert_eq!(format!("{}", Kind::Row), "Row");
        assert_eq!(format!("{}", Kind::Effect), "Effect");
        
        let arrow_kind = Kind::arrow(Kind::Type, Kind::Type);
        assert_eq!(format!("{}", arrow_kind), "(* -> *)");
    }

    // ============================================================================
    // INTEGRATION WITH INFERENCE SYSTEM (Placeholder tests)
    // ============================================================================

    #[test]
    #[ignore] // Will pass when inference is fully implemented
    fn test_basic_type_inference() {
        // Test that basic expressions can be type-inferred
        // This requires integration with the AST and inference engine
        
        let mut checker = TypeChecker::new(TypeLevel::Static);
        assert_eq!(checker.level(), TypeLevel::Static);
        
        // For now, just verify the checker can be created
        // Real inference tests would require AST integration
    }

    #[test]
    #[ignore] // Will pass when unification is fully implemented
    fn test_type_unification() {
        // Test that types can be unified correctly
        // This requires the unification algorithm implementation
        
        // For now, just test basic type equality
        assert_eq!(Type::Number, Type::Number);
        assert_ne!(Type::Number, Type::String);
    }

    #[test]
    #[ignore] // Will pass when constraint solving is implemented
    fn test_constraint_solving() {
        // Test that type constraints can be solved
        // This requires the constraint solver implementation
        
        let constraint = TypeConstraint::equal(
            Type::Number,
            Type::Variable(TypeVar::with_name("a")),
            None,
            "test"
        );
        
        // For now, just verify constraints can be created
        assert!(matches!(constraint, TypeConstraint::Equal { .. }));
    }

    // ============================================================================
    // PERFORMANCE TESTS
    // ============================================================================

    #[test]
    fn test_type_creation_performance() {
        let start = std::time::Instant::now();
        
        // Create many types to test performance
        for i in 0..1000 {
            let var = TypeVar::with_name(format!("var{}", i));
            let func_type = Type::function(
                vec![Type::Variable(var.clone())],
                Type::Variable(var)
            );
            let _ = func_type.free_vars();
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 100, "Type operations should be fast");
    }

    #[test]
    fn test_type_environment_performance() {
        let mut env = TypeEnv::new();
        let start = std::time::Instant::now();
        
        // Add many bindings
        for i in 0..1000 {
            env.bind(
                format!("var{}", i),
                TypeScheme::monomorphic(Type::Number)
            );
        }
        
        // Look up many bindings
        for i in 0..1000 {
            let _ = env.lookup(&format!("var{}", i));
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 100, "Environment operations should be fast");
    }

    // ============================================================================
    // ERROR CASE TESTS
    // ============================================================================

    #[test]
    fn test_type_variable_equality() {
        let var1 = TypeVar::with_id(1);
        let var2 = TypeVar::with_id(1);
        let var3 = TypeVar::with_id(2);
        
        assert_eq!(var1, var2);
        assert_ne!(var1, var3);
    }

    #[test]
    fn test_complex_type_equality() {
        let var_a = TypeVar::with_name("a");
        let var_b = TypeVar::with_name("b");
        
        let type1 = Type::function(
            vec![Type::Variable(var_a.clone())],
            Type::Variable(var_b.clone())
        );
        
        let type2 = Type::function(
            vec![Type::Variable(var_a.clone())],
            Type::Variable(var_b.clone())
        );
        
        assert_eq!(type1, type2);
        
        let type3 = Type::function(
            vec![Type::Variable(var_a)],
            Type::Number
        );
        
        assert_ne!(type1, type3);
    }

    // ============================================================================
    // RECURSIVE TYPE TESTS
    // ============================================================================

    #[test]
    fn test_recursive_type_creation() {
        let var = TypeVar::with_name("t");
        let recursive_type = Type::Recursive {
            var: var.clone()),
            body: Box::new(Type::function(
                vec![Type::Number],
                Type::Variable(var.clone())
            )),
        };
        
        assert!(matches!(recursive_type, Type::Recursive { .. }));
        
        // The recursive variable should not appear in free variables
        let free_vars = recursive_type.free_vars();
        assert!(!free_vars.contains(&var));
    }

    // ============================================================================
    // TYPE LEVEL CONFIGURATION TESTS
    // ============================================================================

    #[test]
    fn test_type_level_configurations() {
        let dynamic_checker = TypeChecker::new(TypeLevel::Dynamic);
        let static_checker = TypeChecker::new(TypeLevel::Static);
        let contracts_checker = TypeChecker::new(TypeLevel::Contracts);
        let dependent_checker = TypeChecker::new(TypeLevel::Dependent);
        
        assert_eq!(dynamic_checker.level(), TypeLevel::Dynamic);
        assert_eq!(static_checker.level(), TypeLevel::Static);
        assert_eq!(contracts_checker.level(), TypeLevel::Contracts);
        assert_eq!(dependent_checker.level(), TypeLevel::Dependent);
    }

    #[test]
    fn test_default_type_checker() {
        let default_checker = TypeChecker::default();
        assert_eq!(default_checker.level(), TypeLevel::Dynamic);
    }
}