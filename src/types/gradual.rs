//! Gradual typing support for Lambdust.
//!
//! This module implements gradual typing that allows mixing static and dynamic
//! types in the same program, with smooth interoperability between typed and
//! untyped code.

use super::{Type, TypeVar};
// use crate::diagnostics::{Error, Result, Span};
// use std::collections::HashSet;

/// Consistency relation for gradual typing.
///
/// Two types are consistent if they could be made equal by replacing
/// any dynamic types with concrete types.
pub fn consistent(type1: &Type, type2: &Type) -> bool {
    match (type1, type2) {
        // Dynamic is consistent with everything
        (Type::Dynamic, _) | (_, Type::Dynamic) => true,
        
        // Unknown is consistent with everything
        (Type::Unknown, _) | (_, Type::Unknown) => true,
        
        // Identical types are consistent
        (t1, t2) if t1 == t2 => true,
        
        // Type variables are consistent with anything
        (Type::Variable(_), _) | (_, Type::Variable(_)) => true,
        
        // Compound types are consistent if their components are
        (Type::Pair(a1, b1), Type::Pair(a2, b2)) => {
            consistent(a1, a2) && consistent(b1, b2)
        }
        
        (Type::List(t1), Type::List(t2)) |
        (Type::Vector(t1), Type::Vector(t2)) => {
            consistent(t1, t2)
        }
        
        (Type::Function { params: p1, return_type: r1 },
         Type::Function { params: p2, return_type: r2 }) => {
            p1.len() == p2.len() &&
            p1.iter().zip(p2.iter()).all(|(t1, t2)| consistent(t1, t2)) &&
            consistent(r1, r2)
        }
        
        // Type applications
        (Type::Application { constructor: c1, argument: a1 },
         Type::Application { constructor: c2, argument: a2 }) => {
            consistent(c1, c2) && consistent(a1, a2)
        }
        
        // Polymorphic types (simplified - should handle variable capture)
        (Type::Forall { body: b1, .. }, Type::Forall { body: b2, .. }) => {
            consistent(b1, b2)
        }
        
        // Effect types
        (Type::Effectful { input: i1, output: o1, .. },
         Type::Effectful { input: i2, output: o2, .. }) => {
            consistent(i1, i2) && consistent(o1, o2)
        }
        
        // Otherwise inconsistent
        _ => false,
    }
}

/// Gradual type joining operation.
///
/// Computes the join (least upper bound) of two types in the gradual
/// type lattice. Returns None if the types are inconsistent.
pub fn join_types(type1: &Type, type2: &Type) -> Option<Type> {
    if !consistent(type1, type2) {
        return None;
    }
    
    match (type1, type2) {
        // Dynamic absorbs everything
        (Type::Dynamic, _) | (_, Type::Dynamic) => Some(Type::Dynamic),
        
        // Unknown becomes the other type
        (Type::Unknown, t) | (t, Type::Unknown) => Some(t.clone()),
        
        // Identical types
        (t1, t2) if t1 == t2 => Some(t1.clone()),
        
        // Type variables require unification
        (Type::Variable(v1), Type::Variable(v2)) if v1 == v2 => {
            Some(Type::Variable(v1.clone()))
        }
        (Type::Variable(_), t) | (t, Type::Variable(_)) => {
            // For now, just return the non-variable type
            // In a full implementation, this would require constraint solving
            Some(t.clone())
        }
        
        // Compound types
        (Type::Pair(a1, b1), Type::Pair(a2, b2)) => {
            let joined_a = join_types(a1, a2)?;
            let joined_b = join_types(b1, b2)?;
            Some(Type::pair(joined_a, joined_b))
        }
        
        (Type::List(t1), Type::List(t2)) => {
            let joined = join_types(t1, t2)?;
            Some(Type::list(joined))
        }
        
        (Type::Vector(t1), Type::Vector(t2)) => {
            let joined = join_types(t1, t2)?;
            Some(Type::vector(joined))
        }
        
        (Type::Function { params: p1, return_type: r1 },
         Type::Function { params: p2, return_type: r2 }) => {
            if p1.len() != p2.len() {
                return None;
            }
            
            let mut joined_params = Vec::new();
            for (param1, param2) in p1.iter().zip(p2.iter()) {
                joined_params.push(join_types(param1, param2)?);
            }
            
            let joined_return = join_types(r1, r2)?;
            Some(Type::function(joined_params, joined_return))
        }
        
        // For other cases, if they're consistent, return Dynamic
        _ => Some(Type::Dynamic),
    }
}

/// Gradual type meeting operation.
///
/// Computes the meet (greatest lower bound) of two types in the gradual
/// type lattice. Returns None if the types are inconsistent.
pub fn meet_types(type1: &Type, type2: &Type) -> Option<Type> {
    if !consistent(type1, type2) {
        return None;
    }
    
    match (type1, type2) {
        // Dynamic meets with the other type
        (Type::Dynamic, t) | (t, Type::Dynamic) => Some(t.clone()),
        
        // Unknown meets with Unknown
        (Type::Unknown, Type::Unknown) => Some(Type::Unknown),
        
        // Unknown meets with concrete type
        (Type::Unknown, t) | (t, Type::Unknown) => Some(t.clone()),
        
        // Otherwise same as join for consistent types
        _ => join_types(type1, type2),
    }
}

/// Converts a static type to a gradual type.
///
/// This operation makes all type variables dynamic, allowing the type
/// to be used in a gradual context.
pub fn gradualize(type_: &Type) -> Type {
    match type_ {
        Type::Variable(_) => Type::Dynamic,
        Type::Pair(a, b) => {
            Type::pair(gradualize(a), gradualize(b))
        }
        Type::List(t) => Type::list(gradualize(t)),
        Type::Vector(t) => Type::vector(gradualize(t)),
        Type::Function { params, return_type } => {
            let grad_params = params.iter().map(gradualize).collect();
            Type::function(grad_params, gradualize(return_type))
        }
        Type::Application { constructor, argument } => {
            Type::Application {
                constructor: Box::new(gradualize(constructor)),
                argument: Box::new(gradualize(argument)),
            }
        }
        Type::Forall { vars: _, body } => {
            // Remove quantification and gradualize body
            gradualize(body)
        }
        Type::Effectful { input, effects, output } => {
            Type::Effectful {
                input: Box::new(gradualize(input)),
                effects: effects.clone()),
                output: Box::new(gradualize(output)),
            }
        }
        // Base types and Dynamic remain unchanged
        _ => type_.clone()),
    }
}

/// Converts a gradual type to a static type by replacing Dynamic with type variables.
///
/// This is used when transitioning from gradual to static typing.
pub fn staticize(type_: &Type, fresh_var_supply: &mut impl FnMut() -> TypeVar) -> Type {
    match type_ {
        Type::Dynamic => Type::Variable(fresh_var_supply()),
        Type::Unknown => Type::Variable(fresh_var_supply()),
        Type::Pair(a, b) => {
            Type::pair(
                staticize(a, fresh_var_supply),
                staticize(b, fresh_var_supply),
            )
        }
        Type::List(t) => Type::list(staticize(t, fresh_var_supply)),
        Type::Vector(t) => Type::vector(staticize(t, fresh_var_supply)),
        Type::Function { params, return_type } => {
            let static_params = params.iter()
                .map(|p| staticize(p, fresh_var_supply))
                .collect();
            Type::function(static_params, staticize(return_type, fresh_var_supply))
        }
        Type::Application { constructor, argument } => {
            Type::Application {
                constructor: Box::new(staticize(constructor, fresh_var_supply)),
                argument: Box::new(staticize(argument, fresh_var_supply)),
            }
        }
        Type::Effectful { input, effects, output } => {
            Type::Effectful {
                input: Box::new(staticize(input, fresh_var_supply)),
                effects: effects.clone()),
                output: Box::new(staticize(output, fresh_var_supply)),
            }
        }
        // Other types remain unchanged
        _ => type_.clone()),
    }
}

/// Cast insertion for gradual typing.
///
/// Determines where runtime type checks (casts) need to be inserted
/// when values flow between static and dynamic contexts.
#[derive(Debug, Clone)]
pub enum Cast {
    /// No cast needed (types are compatible)
    None,
    /// Upcast from static to dynamic
    Upcast { from: Type, to: Type },
    /// Downcast from dynamic to static (requires runtime check)
    Downcast { from: Type, to: Type },
    /// Structural cast (for compound types)
    Structural { casts: Vec<Cast> },
}

/// Determines the cast needed when a value of one type is used in a context
/// expecting another type.
pub fn insert_cast(source: &Type, target: &Type) -> Cast {
    if source == target {
        return Cast::None;
    }
    
    match (source, target) {
        // Upcast to Dynamic
        (_, Type::Dynamic) => Cast::Upcast {
            from: source.clone()),
            to: target.clone()),
        },
        
        // Downcast from Dynamic
        (Type::Dynamic, _) => Cast::Downcast {
            from: source.clone()),
            to: target.clone()),
        },
        
        // Structural casts for compound types
        (Type::Pair(a1, b1), Type::Pair(a2, b2)) => {
            let cast_a = insert_cast(a1, a2);
            let cast_b = insert_cast(b1, b2);
            
            match (&cast_a, &cast_b) {
                (Cast::None, Cast::None) => Cast::None,
                _ => Cast::Structural {
                    casts: vec![cast_a, cast_b],
                },
            }
        }
        
        (Type::List(t1), Type::List(t2)) |
        (Type::Vector(t1), Type::Vector(t2)) => {
            let element_cast = insert_cast(t1, t2);
            match element_cast {
                Cast::None => Cast::None,
                _ => Cast::Structural {
                    casts: vec![element_cast],
                },
            }
        }
        
        (Type::Function { params: p1, return_type: r1 },
         Type::Function { params: p2, return_type: r2 }) => {
            if p1.len() != p2.len() {
                // Arity mismatch - need runtime check
                return Cast::Downcast {
                    from: source.clone()),
                    to: target.clone()),
                };
            }
            
            // Function types are contravariant in arguments, covariant in return
            let mut param_casts = Vec::new();
            for (param1, param2) in p1.iter().zip(p2.iter()) {
                // Contravariant: cast from target to source for parameters
                param_casts.push(insert_cast(param2, param1));
            }
            
            let return_cast = insert_cast(r1, r2);
            
            let all_none = param_casts.iter().all(|c| matches!(c, Cast::None)) &&
                          matches!(return_cast, Cast::None);
            
            if all_none {
                Cast::None
            } else {
                let mut casts = param_casts;
                casts.push(return_cast);
                Cast::Structural { casts }
            }
        }
        
        // For other cases, if consistent, use upcast/downcast as appropriate
        _ => {
            if consistent(source, target) {
                Cast::Upcast {
                    from: source.clone()),
                    to: target.clone()),
                }
            } else {
                Cast::Downcast {
                    from: source.clone()),
                    to: target.clone()),
                }
            }
        }
    }
}

/// Checks if a type is gradual (contains Dynamic or Unknown).
pub fn is_gradual(type_: &Type) -> bool {
    match type_ {
        Type::Dynamic | Type::Unknown => true,
        Type::Pair(a, b) => is_gradual(a) || is_gradual(b),
        Type::List(t) | Type::Vector(t) => is_gradual(t),
        Type::Function { params, return_type } => {
            params.iter().any(is_gradual) || is_gradual(return_type)
        }
        Type::Application { constructor, argument } => {
            is_gradual(constructor) || is_gradual(argument)
        }
        Type::Forall { body, .. } | Type::Exists { body, .. } => is_gradual(body),
        Type::Constrained { type_, .. } => is_gradual(type_),
        Type::Effectful { input, output, .. } => is_gradual(input) || is_gradual(output),
        Type::Record(row) | Type::Variant(row) => {
            row.fields.values().any(is_gradual)
        }
        Type::Recursive { body, .. } => is_gradual(body),
        _ => false,
    }
}

/// Checks if a type is fully static (no Dynamic, Unknown, or unresolved variables).
pub fn is_static(type_: &Type) -> bool {
    match type_ {
        Type::Dynamic | Type::Unknown | Type::Variable(_) => false,
        Type::Pair(a, b) => is_static(a) && is_static(b),
        Type::List(t) | Type::Vector(t) => is_static(t),
        Type::Function { params, return_type } => {
            params.iter().all(is_static) && is_static(return_type)
        }
        Type::Application { constructor, argument } => {
            is_static(constructor) && is_static(argument)
        }
        Type::Forall { body, .. } | Type::Exists { body, .. } => is_static(body),
        Type::Constrained { type_, .. } => is_static(type_),
        Type::Effectful { input, output, .. } => is_static(input) && is_static(output),
        Type::Record(row) | Type::Variant(row) => {
            row.fields.values().all(is_static) && row.rest.is_none()
        }
        Type::Recursive { body, .. } => is_static(body),
        _ => true, // Base types are static
    }
}

/// Approximates a type by replacing unknown parts with Dynamic.
///
/// This is used when type inference fails but we want to continue
/// with gradual typing.
pub fn approximate_type(type_: &Type) -> Type {
    match type_ {
        Type::Variable(_) | Type::Unknown => Type::Dynamic,
        Type::Pair(a, b) => {
            Type::pair(approximate_type(a), approximate_type(b))
        }
        Type::List(t) => Type::list(approximate_type(t)),
        Type::Vector(t) => Type::vector(approximate_type(t)),
        Type::Function { params, return_type } => {
            let approx_params = params.iter().map(approximate_type).collect();
            Type::function(approx_params, approximate_type(return_type))
        }
        Type::Application { constructor, argument } => {
            Type::Application {
                constructor: Box::new(approximate_type(constructor)),
                argument: Box::new(approximate_type(argument)),
            }
        }
        Type::Effectful { input, effects, output } => {
            Type::Effectful {
                input: Box::new(approximate_type(input)),
                effects: effects.clone()),
                output: Box::new(approximate_type(output)),
            }
        }
        // Other types remain unchanged
        _ => type_.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency() {
        assert!(consistent(&Type::Dynamic, &Type::Number));
        assert!(consistent(&Type::Number, &Type::Dynamic));
        assert!(consistent(&Type::Number, &Type::Number));
        assert!(!consistent(&Type::Number, &Type::String));
        
        // Function types
        let func1 = Type::function(vec![Type::Number], Type::String);
        let func2 = Type::function(vec![Type::Dynamic], Type::String);
        assert!(consistent(&func1, &func2));
        
        let func3 = Type::function(vec![Type::Number], Type::Number);
        assert!(!consistent(&func1, &func3));
    }

    #[test]
    fn test_join_types() {
        assert_eq!(join_types(&Type::Dynamic, &Type::Number), Some(Type::Dynamic));
        assert_eq!(join_types(&Type::Number, &Type::Number), Some(Type::Number));
        assert_eq!(join_types(&Type::Number, &Type::String), None);
        
        let pair1 = Type::pair(Type::Number, Type::Dynamic);
        let pair2 = Type::pair(Type::Dynamic, Type::String);
        let expected = Type::pair(Type::Dynamic, Type::Dynamic);
        assert_eq!(join_types(&pair1, &pair2), Some(expected));
    }

    #[test]
    fn test_gradualize() {
        let var = TypeVar::with_id(1);
        let static_type = Type::function(vec![Type::Variable(var)], Type::Number);
        let gradual_type = gradualize(&static_type);
        
        match gradual_type {
            Type::Function { params, return_type } => {
                assert_eq!(params[0], Type::Dynamic);
                assert_eq!(*return_type, Type::Number);
            }
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_staticize() {
        let mut counter = 0;
        let mut fresh_var = || {
            counter += 1;
            TypeVar::with_id(counter)
        };
        
        let gradual_type = Type::function(vec![Type::Dynamic], Type::Number);
        let static_type = staticize(&gradual_type, &mut fresh_var);
        
        match static_type {
            Type::Function { params, return_type } => {
                assert!(matches!(params[0], Type::Variable(_)));
                assert_eq!(*return_type, Type::Number);
            }
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_cast_insertion() {
        // Upcast
        let cast = insert_cast(&Type::Number, &Type::Dynamic);
        assert!(matches!(cast, Cast::Upcast { .. }));
        
        // Downcast
        let cast = insert_cast(&Type::Dynamic, &Type::Number);
        assert!(matches!(cast, Cast::Downcast { .. }));
        
        // No cast needed
        let cast = insert_cast(&Type::Number, &Type::Number);
        assert!(matches!(cast, Cast::None));
    }

    #[test]
    fn test_is_gradual() {
        assert!(is_gradual(&Type::Dynamic));
        assert!(is_gradual(&Type::Unknown));
        assert!(!is_gradual(&Type::Number));
        
        let gradual_pair = Type::pair(Type::Number, Type::Dynamic);
        assert!(is_gradual(&gradual_pair));
        
        let static_pair = Type::pair(Type::Number, Type::String);
        assert!(!is_gradual(&static_pair));
    }

    #[test]
    fn test_is_static() {
        assert!(is_static(&Type::Number));
        assert!(!is_static(&Type::Dynamic));
        assert!(!is_static(&Type::Variable(TypeVar::with_id(1))));
        
        let static_func = Type::function(vec![Type::Number], Type::String);
        assert!(is_static(&static_func));
        
        let gradual_func = Type::function(vec![Type::Dynamic], Type::String);
        assert!(!is_static(&gradual_func));
    }
}