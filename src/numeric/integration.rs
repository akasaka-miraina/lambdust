//! Integration with existing primitive system
//!
//! Provides registration functions to integrate the advanced numeric primitives
//! with the existing Lambdust primitive system.

use crate::runtime::{MinimalPrimitiveRegistry, MinimalPrimitive, MinimalPrimitiveCategory};
use super::primitives::*;

/// Registers all numeric primitives with the primitive registry
pub fn register_numeric_primitives(registry: &mut MinimalPrimitiveRegistry) {
    // ============= ARITHMETIC OPERATIONS =============
    
    registry.register(MinimalPrimitive {
        name: "+".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_add,
        arity_min: 0,
        arity_max: None,
        documentation: "Addition with automatic type promotion and overflow handling".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "-".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_subtract,
        arity_min: 1,
        arity_max: None,
        documentation: "Subtraction with automatic type promotion".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "*".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_multiply,
        arity_min: 0,
        arity_max: None,
        documentation: "Multiplication with automatic type promotion".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "/".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_divide,
        arity_min: 1,
        arity_max: None,
        documentation: "Division with automatic type promotion to rational".to_string(),
        r7rs_required: true,
    });

    // ============= COMPARISON OPERATIONS =============
    
    registry.register(MinimalPrimitive {
        name: "=".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_numeric_equal,
        arity_min: 2,
        arity_max: None,
        documentation: "Numeric equality with type coercion".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "<".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_less_than,
        arity_min: 2,
        arity_max: None,
        documentation: "Numeric less than comparison".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: ">".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_greater_than,
        arity_min: 2,
        arity_max: None,
        documentation: "Numeric greater than comparison".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "<=".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_less_equal,
        arity_min: 2,
        arity_max: None,
        documentation: "Numeric less than or equal comparison".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: ">=".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_greater_equal,
        arity_min: 2,
        arity_max: None,
        documentation: "Numeric greater than or equal comparison".to_string(),
        r7rs_required: true,
    });

    // ============= MATHEMATICAL FUNCTIONS =============
    
    registry.register(MinimalPrimitive {
        name: "exp".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_exp,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Exponential function (e^x)".to_string(),
        r7rs_required: false,
    });

    registry.register(MinimalPrimitive {
        name: "log".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_log,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Natural logarithm with complex support".to_string(),
        r7rs_required: false,
    });

    registry.register(MinimalPrimitive {
        name: "sqrt".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_sqrt,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Square root with complex support".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "expt".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_expt,
        arity_min: 2,
        arity_max: Some(2),
        documentation: "Exponentiation with complex support".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "sin".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_sin,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Sine function with complex support".to_string(),
        r7rs_required: false,
    });

    registry.register(MinimalPrimitive {
        name: "cos".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_cos,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Cosine function with complex support".to_string(),
        r7rs_required: false,
    });

    registry.register(MinimalPrimitive {
        name: "tan".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_tan,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Tangent function with complex support".to_string(),
        r7rs_required: false,
    });

    // ============= SPECIAL FUNCTIONS =============
    
    registry.register(MinimalPrimitive {
        name: "gamma".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_gamma,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Gamma function using Lanczos approximation".to_string(),
        r7rs_required: false,
    });

    registry.register(MinimalPrimitive {
        name: "erf".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_erf,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Error function using Chebyshev approximation".to_string(),
        r7rs_required: false,
    });

    registry.register(MinimalPrimitive {
        name: "bessel-j0".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_bessel_j0,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Bessel function of the first kind, order 0".to_string(),
        r7rs_required: false,
    });

    // ============= TYPE PREDICATES =============
    
    registry.register(MinimalPrimitive {
        name: "number?".to_string(),
        category: MinimalPrimitiveCategory::Types,
        implementation: primitive_number_p,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Test if value is a number".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "exact?".to_string(),
        category: MinimalPrimitiveCategory::Types,
        implementation: primitive_exact_p,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Test if number is exact".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "inexact?".to_string(),
        category: MinimalPrimitiveCategory::Types,
        implementation: primitive_inexact_p,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Test if number is inexact".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "integer?".to_string(),
        category: MinimalPrimitiveCategory::Types,
        implementation: primitive_integer_p,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Test if value is an integer".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "real?".to_string(),
        category: MinimalPrimitiveCategory::Types,
        implementation: primitive_real_p,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Test if value is real".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "complex?".to_string(),
        category: MinimalPrimitiveCategory::Types,
        implementation: primitive_complex_p,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Test if value is complex".to_string(),
        r7rs_required: true,
    });

    // ============= EXACTNESS CONVERSION =============
    
    registry.register(MinimalPrimitive {
        name: "exact".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_exact,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Convert number to exact representation".to_string(),
        r7rs_required: true,
    });

    registry.register(MinimalPrimitive {
        name: "inexact".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_inexact,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Convert number to inexact representation".to_string(),
        r7rs_required: true,
    });

    // ============= CONSTANTS AND UTILITIES =============
    
    registry.register(MinimalPrimitive {
        name: "constant".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_constant,
        arity_min: 1,
        arity_max: Some(1),
        documentation: "Get mathematical or physical constant by name".to_string(),
        r7rs_required: false,
    });

    registry.register(MinimalPrimitive {
        name: "list-constants".to_string(),
        category: MinimalPrimitiveCategory::Arithmetic,
        implementation: primitive_list_constants,
        arity_min: 0,
        arity_max: Some(0),
        documentation: "List all available constants".to_string(),
        r7rs_required: false,
    });
}

/// Creates a new primitive registry with all numeric primitives pre-registered
pub fn create_numeric_registry() -> MinimalPrimitiveRegistry {
    let mut registry = MinimalPrimitiveRegistry::new();
    register_numeric_primitives(&mut registry);
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = create_numeric_registry();
        
        // Test that basic arithmetic primitives are registered
        assert!(registry.get_primitive("+").is_some());
        assert!(registry.get_primitive("-").is_some());
        assert!(registry.get_primitive("*").is_some());
        assert!(registry.get_primitive("/").is_some());
        
        // Test that comparison primitives are registered
        assert!(registry.get_primitive("=").is_some());
        assert!(registry.get_primitive("<").is_some());
        assert!(registry.get_primitive(">").is_some());
        
        // Test that mathematical functions are registered
        assert!(registry.get_primitive("sin").is_some());
        assert!(registry.get_primitive("cos").is_some());
        assert!(registry.get_primitive("exp").is_some());
        assert!(registry.get_primitive("sqrt").is_some());
        
        // Test that type predicates are registered
        assert!(registry.get_primitive("number?").is_some());
        assert!(registry.get_primitive("exact?").is_some());
        assert!(registry.get_primitive("integer?").is_some());
        
        // Test that special functions are registered
        assert!(registry.get_primitive("gamma").is_some());
        assert!(registry.get_primitive("erf").is_some());
        
        // Test that constants utilities are registered
        assert!(registry.get_primitive("constant").is_some());
        assert!(registry.get_primitive("list-constants").is_some());
    }

    #[test]
    fn test_primitive_categorization() {
        let registry = create_numeric_registry();
        
        let arithmetic_primitives = registry.get_primitives_by_category(&MinimalPrimitiveCategory::Arithmetic);
        assert!(!arithmetic_primitives.is_empty());
        
        let type_primitives = registry.get_primitives_by_category(&MinimalPrimitiveCategory::Types);
        assert!(!type_primitives.is_empty());
    }
}