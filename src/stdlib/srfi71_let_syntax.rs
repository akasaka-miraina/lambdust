//! SRFI-71: Extended LET-syntax for multiple values
//!
//! This module implements SRFI-71, which extends `let`, `let*`, and `letrec` forms
//! to handle multiple values more conveniently. It allows binding multiple variables
//! to multiple values returned by a single expression within standard let forms.
//!
//! ## R7RS Integration
//!
//! SRFI-71 is a syntax extension that makes multiple value binding more natural
//! and concise, building upon SRFI-8 (receive) and SRFI-11 (let-values).
//!
//! ## Extended Syntax
//!
//! ```scheme
//! ;; Standard let extended with multiple values
//! (let ((a b (values 1 2))
//!       (c d e (values 3 4 5)))
//!   (+ a b c d e))  ; => 15
//!
//! ;; Rest arguments
//! (let (((values first . rest) (values 1 2 3 4)))
//!   (cons first rest))  ; => (1 2 3 4)
//!
//! ;; Mixed single and multiple values  
//! (let ((x 10)
//!       (a b (quotient-remainder 17 5)))
//!   (+ x a b))  ; => 15
//! ```
//!
//! ## Utility Procedures
//!
//! SRFI-71 also defines utility procedures for value decomposition:
//! - `uncons` - decompose a pair
//! - `unlist` - decompose a list
//! - `values->list` - convert multiple values to list
//! - `values->vector` - convert multiple values to vector

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveProcedure, PrimitiveImpl, Value, MultipleValues};
use crate::eval::value::{Environment, Generation};
use std::sync::Arc;

/// Install SRFI-71 extended let syntax procedures and utilities into the environment.
pub fn install_srfi71_procedures(env: &mut Environment) {
    // Utility procedures for value decomposition
    env.define(
        "uncons".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "uncons".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(uncons),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "unlist".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "unlist".to_string(),
            arity_min: 1,
            arity_max: Some(2), // Optional length argument
            implementation: PrimitiveImpl::RustFn(unlist),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "values->list".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "values->list".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(values_to_list),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "values->vector".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "values->vector".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(values_to_vector),
            effects: vec![Effect::Pure],
        })),
    );

    // Value construction utilities
    env.define(
        "list->values".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "list->values".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(list_to_values),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "vector->values".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "vector->values".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(vector_to_values),
            effects: vec![Effect::Pure],
        })),
    );
}

/// `(uncons pair)` - Decompose a pair into two values: car and cdr
///
/// Returns the car and cdr of the pair as two separate values.
///
/// ## Arguments
/// - `pair` - A pair (cons cell)
///
/// ## Returns
/// Two values: the car and cdr of the pair
///
/// ## Examples
/// ```scheme
/// (let ((a b (uncons '(1 . 2))))
///   (list a b))  ; => (1 2)
///
/// (let ((first rest (uncons '(x y z))))
///   (cons first rest))  ; => (x y z)
/// ```
pub fn uncons(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("uncons expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::Pair(car, cdr) => {
            // Return two values: car and cdr
            let values = vec![car.as_ref().clone(), cdr.as_ref().clone()];
            Ok(Value::MultipleValues(Arc::new(MultipleValues::new(values))))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "uncons: argument must be a pair".to_string(),
            None,
        ))),
    }
}

/// `(unlist list [n])` - Decompose a list into its elements as separate values
///
/// Returns the first n elements of the list as separate values.
/// If n is not provided, returns all elements.
///
/// ## Arguments
/// - `list` - A proper list
/// - `n` - Optional number of elements to extract
///
/// ## Returns
/// The first n elements of the list as separate values
///
/// ## Examples
/// ```scheme
/// (let ((a b c (unlist '(1 2 3))))
///   (+ a b c))  ; => 6
///
/// (let ((x y (unlist '(10 20 30 40) 2)))
///   (* x y))  ; => 200
/// ```
pub fn unlist(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("unlist expects 1-2 arguments, got {}", args.len()),
            None,
        )));
    }

    // Extract list elements
    let mut elements = Vec::new();
    let mut current = &args[0];
    
    loop {
        match current {
            Value::Nil => break,
            Value::Pair(car, cdr) => {
                elements.push(car.as_ref().clone());
                current = cdr.as_ref();
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "unlist: first argument must be a proper list".to_string(),
                    None,
                )));
            }
        }
    }

    // Handle optional length argument
    if args.len() == 2 {
        let n = match &args[1] {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => *n as usize,
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "unlist: second argument must be an integer".to_string(),
                    None,
                )));
            }
        };

        if n > elements.len() {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("unlist: list has {} elements but {} requested", elements.len(), n),
                None,
            )));
        }

        elements.truncate(n);
    }

    if elements.is_empty() {
        Ok(Value::Nil)
    } else if elements.len() == 1 {
        Ok(elements.into_iter().next().unwrap())
    } else {
        Ok(Value::MultipleValues(Arc::new(MultipleValues::new(elements))))
    }
}

/// `(values->list values-producer)` - Convert multiple values to a list
///
/// Converts multiple values to a list.
///
/// ## Arguments
/// - `values` - Multiple values or single value
///
/// ## Returns
/// A list containing the values
pub fn values_to_list(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("values->list expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::MultipleValues(mv) => Ok(Value::list(mv.as_slice().to_vec())),
        single_value => Ok(Value::list(vec![single_value.clone()])),
    }
}

/// `(values->vector values-producer)` - Convert multiple values to a vector
pub fn values_to_vector(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("values->vector expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::MultipleValues(mv) => Ok(Value::vector(mv.as_slice().to_vec())),
        single_value => Ok(Value::vector(vec![single_value.clone()])),
    }
}

/// `(list->values list)` - Convert a list to multiple values
pub fn list_to_values(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("list->values expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    let mut elements = Vec::new();
    let mut current = &args[0];
    
    loop {
        match current {
            Value::Nil => break,
            Value::Pair(car, cdr) => {
                elements.push(car.as_ref().clone());
                current = cdr.as_ref();
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "list->values: argument must be a proper list".to_string(),
                    None,
                )));
            }
        }
    }

    if elements.is_empty() {
        Ok(Value::Nil)
    } else if elements.len() == 1 {
        Ok(elements.into_iter().next().unwrap())
    } else {
        Ok(Value::MultipleValues(Arc::new(MultipleValues::new(elements))))
    }
}

/// `(vector->values vector)` - Convert a vector to multiple values
pub fn vector_to_values(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("vector->values expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::Vector(vec) => {
            let elements = vec.borrow().clone();
            if elements.is_empty() {
                Ok(Value::Nil)
            } else if elements.len() == 1 {
                Ok(elements.into_iter().next().unwrap())
            } else {
                Ok(Value::MultipleValues(Arc::new(MultipleValues::new(elements))))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "vector->values: argument must be a vector".to_string(),
            None,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test uncons procedure
    #[test]
    fn test_uncons() {
        let mut env = Environment::new(None, 0 as Generation);
        install_srfi71_procedures(&mut env);

        // Test basic uncons
        let pair = Value::cons(Value::integer(1), Value::integer(2));
        let result = uncons(&[pair]).unwrap();
        
        if let Value::MultipleValues(mv) = result {
            let values = mv.as_slice();
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], Value::integer(1));
            assert_eq!(values[1], Value::integer(2));
        } else {
            panic!("Expected MultipleValues result from uncons");
        }

        // Test uncons with list
        let list = Value::list(vec![Value::integer(10), Value::integer(20), Value::integer(30)]);
        let result = uncons(&[list]).unwrap();
        
        if let Value::MultipleValues(mv) = result {
            let values = mv.as_slice();
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], Value::integer(10));
            // Second value should be the rest of the list
            assert!(matches!(values[1], Value::Pair(_, _)));
        }
    }

    /// Test unlist procedure
    #[test]
    fn test_unlist() {
        // Test basic unlist
        let list = Value::list(vec![
            Value::integer(1),
            Value::integer(2), 
            Value::integer(3),
        ]);
        let result = unlist(&[list]).unwrap();
        
        if let Value::MultipleValues(mv) = result {
            let values = mv.as_slice();
            assert_eq!(values.len(), 3);
            assert_eq!(values[0], Value::integer(1));
            assert_eq!(values[1], Value::integer(2));
            assert_eq!(values[2], Value::integer(3));
        } else {
            panic!("Expected MultipleValues result from unlist");
        }

        // Test unlist with count
        let list = Value::list(vec![
            Value::integer(10),
            Value::integer(20),
            Value::integer(30),
            Value::integer(40),
        ]);
        let result = unlist(&[list, Value::Literal(crate::ast::Literal::ExactInteger(2))]).unwrap();
        
        if let Value::MultipleValues(mv) = result {
            let values = mv.as_slice();
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], Value::integer(10));
            assert_eq!(values[1], Value::integer(20));
        }
    }

    /// Test list->values and values->list conversion
    #[test]
    fn test_list_values_conversion() {
        // Test list->values
        let list = Value::list(vec![Value::integer(1), Value::integer(2), Value::integer(3)]);
        let values_result = list_to_values(&[list]).unwrap();
        
        // Test values->list (using the MultipleValues directly)
        let list_result = values_to_list(&[values_result]).unwrap();
        
        // Should get back the original list structure
        assert!(matches!(list_result, Value::Pair(_, _) | Value::Nil));
        
        // Verify elements
        let mut current = &list_result;
        let mut elements = Vec::new();
        loop {
            match current {
                Value::Nil => break,
                Value::Pair(car, cdr) => {
                    elements.push(car.as_ref().clone());
                    current = cdr.as_ref();
                }
                _ => panic!("Expected list structure"),
            }
        }
        
        assert_eq!(elements.len(), 3);
        assert_eq!(elements[0], Value::integer(1));
        assert_eq!(elements[1], Value::integer(2));
        assert_eq!(elements[2], Value::integer(3));
    }

    /// Test error conditions
    #[test]
    fn test_srfi71_errors() {
        // Wrong number of arguments for uncons
        assert!(uncons(&[]).is_err());
        assert!(uncons(&[Value::integer(1), Value::integer(2)]).is_err());

        // Wrong argument type for uncons
        assert!(uncons(&[Value::integer(42)]).is_err());

        // Wrong number of arguments for unlist
        assert!(unlist(&[]).is_err());
        assert!(unlist(&[Value::integer(1), Value::integer(2), Value::integer(3)]).is_err());

        // Wrong argument type for unlist
        assert!(unlist(&[Value::integer(42)]).is_err());
    }

    /// Test SRFI-71 environment integration
    #[test]
    fn test_srfi71_environment_integration() {
        let mut env = Environment::new(None, 0 as Generation);
        install_srfi71_procedures(&mut env);

        // Verify all procedures are defined
        assert!(env.lookup("uncons").is_some());
        assert!(env.lookup("unlist").is_some());
        assert!(env.lookup("values->list").is_some());
        assert!(env.lookup("values->vector").is_some());
        assert!(env.lookup("list->values").is_some());
        assert!(env.lookup("vector->values").is_some());

        // Verify they are callable primitives
        let uncons_proc = env.lookup("uncons").unwrap();
        assert!(matches!(uncons_proc, Value::Primitive(_)));

        let unlist_proc = env.lookup("unlist").unwrap();
        assert!(matches!(unlist_proc, Value::Primitive(_)));
    }
}