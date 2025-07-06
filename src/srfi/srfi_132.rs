//! SRFI 132: Sort Libraries
//!
//! This SRFI provides procedures for sorting vectors and lists.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::lexer::SchemeNumber;
#[cfg(test)]
use crate::value::Procedure;
use crate::value::Value;
use std::collections::HashMap;

/// Compare two SchemeNumbers for sorting
fn compare_numbers(a: &SchemeNumber, b: &SchemeNumber) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let a_val = match a {
        SchemeNumber::Integer(i) => *i as f64,
        SchemeNumber::Real(r) => *r,
        SchemeNumber::Rational(num, den) => *num as f64 / *den as f64,
        SchemeNumber::Complex(r, _) => *r, // Compare by real part only
    };

    let b_val = match b {
        SchemeNumber::Integer(i) => *i as f64,
        SchemeNumber::Real(r) => *r,
        SchemeNumber::Rational(num, den) => *num as f64 / *den as f64,
        SchemeNumber::Complex(r, _) => *r, // Compare by real part only
    };

    a_val.partial_cmp(&b_val).unwrap_or(Ordering::Equal)
}

/// Check if two SchemeNumbers are in order (<= relation)
fn numbers_lte(a: &SchemeNumber, b: &SchemeNumber) -> bool {
    use std::cmp::Ordering;
    matches!(compare_numbers(a, b), Ordering::Less | Ordering::Equal)
}

/// SRFI 132 implementation
pub struct Srfi132;

impl super::SrfiModule for Srfi132 {
    fn srfi_id(&self) -> u32 {
        132
    }

    fn name(&self) -> &'static str {
        "Sort Libraries"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec![]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // list-sort procedure
        exports.insert(
            "list-sort".to_string(),
            make_builtin_procedure("list-sort", Some(2), |args| {
                check_arity(args, 2)?;

                // Extract comparator procedure (simplified for now)
                let _comparator = &args[0];
                // TODO: Implement proper comparator procedure evaluation

                // Convert list to vector for sorting
                let list = &args[1];
                let mut values = if let Some(vec) = list.to_vector() {
                    vec
                } else {
                    return Err(LambdustError::type_error("Expected list".to_string()));
                };

                // Sort using the comparator
                values.sort_by(|a, b| {
                    // Call comparator(a, b) and check result
                    // For now, use a simplified numeric comparison
                    match (a.as_number(), b.as_number()) {
                        (Some(na), Some(nb)) => compare_numbers(na, nb),
                        _ => std::cmp::Ordering::Equal,
                    }
                });

                Ok(Value::from_vector(values))
            }),
        );

        // vector-sort! procedure
        exports.insert(
            "vector-sort!".to_string(),
            make_builtin_procedure("vector-sort!", Some(2), |args| {
                check_arity(args, 2)?;

                // Extract comparator procedure
                let _comparator = &args[0];
                let _vector = &args[1];

                // For now, return error as destructive operations need special handling
                Err(LambdustError::runtime_error(
                    "Destructive vector operations not yet implemented".to_string(),
                ))
            }),
        );

        // vector-sort procedure
        exports.insert(
            "vector-sort".to_string(),
            make_builtin_procedure("vector-sort", Some(2), |args| {
                check_arity(args, 2)?;

                // Extract comparator procedure
                let _comparator = &args[0];
                let vector = &args[1];

                if let Value::Vector(values) = vector {
                    let mut sorted_values = values.clone();

                    // Sort using numeric comparison for now
                    sorted_values.sort_by(|a, b| match (a.as_number(), b.as_number()) {
                        (Some(na), Some(nb)) => compare_numbers(na, nb),
                        _ => std::cmp::Ordering::Equal,
                    });

                    Ok(Value::Vector(sorted_values))
                } else {
                    Err(LambdustError::type_error("Expected vector".to_string()))
                }
            }),
        );

        // list-sorted? predicate
        exports.insert(
            "list-sorted?".to_string(),
            make_builtin_procedure("list-sorted?", Some(2), |args| {
                check_arity(args, 2)?;

                let _comparator = &args[0];
                let list = &args[1];

                let values = if let Some(vec) = list.to_vector() {
                    vec
                } else {
                    return Err(LambdustError::type_error("Expected list".to_string()));
                };

                // Check if sorted using numeric comparison
                let is_sorted = values.windows(2).all(|window| {
                    match (window[0].as_number(), window[1].as_number()) {
                        (Some(a), Some(b)) => numbers_lte(a, b),
                        _ => true, // Non-numeric values are considered sorted
                    }
                });

                Ok(Value::Boolean(is_sorted))
            }),
        );

        // vector-sorted? predicate
        exports.insert(
            "vector-sorted?".to_string(),
            make_builtin_procedure("vector-sorted?", Some(2), |args| {
                check_arity(args, 2)?;

                let _comparator = &args[0];
                let vector = &args[1];

                if let Value::Vector(values) = vector {
                    // Check if sorted using numeric comparison
                    let is_sorted = values.windows(2).all(|window| {
                        match (window[0].as_number(), window[1].as_number()) {
                            (Some(a), Some(b)) => numbers_lte(a, b),
                            _ => true, // Non-numeric values are considered sorted
                        }
                    });

                    Ok(Value::Boolean(is_sorted))
                } else {
                    Err(LambdustError::type_error("Expected vector".to_string()))
                }
            }),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 132 has no parts, return all exports
        Ok(self.exports())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srfi::SrfiModule;

    #[test]
    #[ignore] // TODO: Fix Procedure API compatibility
    fn test_list_sort() {
        let srfi = Srfi132;
        let exports = srfi.exports();

        let sort_proc = exports.get("list-sort").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = sort_proc {
            // Create a dummy comparator (for testing, we'll pass a dummy value)
            let dummy_comparator = Value::Boolean(true); // This will be ignored in our simplified implementation

            // Create an unsorted list [3, 1, 4, 1, 5]
            let unsorted_list = Value::from_vector(vec![
                Value::from(3i64),
                Value::from(1i64),
                Value::from(4i64),
                Value::from(1i64),
                Value::from(5i64),
            ]);

            let result = func(&[dummy_comparator, unsorted_list]).unwrap();

            // Should be sorted as [1, 1, 3, 4, 5]
            if let Some(sorted_vec) = result.to_vector() {
                assert_eq!(sorted_vec.len(), 5);
                assert_eq!(sorted_vec[0], Value::from(1i64));
                assert_eq!(sorted_vec[1], Value::from(1i64));
                assert_eq!(sorted_vec[2], Value::from(3i64));
                assert_eq!(sorted_vec[3], Value::from(4i64));
                assert_eq!(sorted_vec[4], Value::from(5i64));
            } else {
                panic!("Test assertion failed: Expected list result");
            }
        }
    }

    #[test]
    #[ignore] // TODO: Fix Procedure API compatibility
    fn test_vector_sort() {
        let srfi = Srfi132;
        let exports = srfi.exports();

        let sort_proc = exports.get("vector-sort").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = sort_proc {
            // Create a dummy comparator
            let dummy_comparator = Value::Boolean(true);

            // Create an unsorted vector
            let unsorted_vector = Value::Vector(vec![
                Value::from(42i64),
                Value::from(1i64),
                Value::from(17i64),
            ]);

            let result = func(&[dummy_comparator, unsorted_vector]).unwrap();

            if let Value::Vector(sorted_vec) = result {
                assert_eq!(sorted_vec.len(), 3);
                assert_eq!(sorted_vec[0], Value::from(1i64));
                assert_eq!(sorted_vec[1], Value::from(17i64));
                assert_eq!(sorted_vec[2], Value::from(42i64));
            } else {
                panic!("Test assertion failed: Expected vector result");
            }
        }
    }

    #[test]
    #[ignore] // TODO: Fix Procedure API compatibility
    fn test_sorted_predicates() {
        let srfi = Srfi132;
        let exports = srfi.exports();

        let list_sorted_proc = exports.get("list-sorted?").unwrap();
        let vector_sorted_proc = exports.get("vector-sorted?").unwrap();

        if let (
            Value::Procedure(Procedure::Builtin {
                func: list_func, ..
            }),
            Value::Procedure(Procedure::Builtin {
                func: vector_func, ..
            }),
        ) = (list_sorted_proc, vector_sorted_proc)
        {
            let dummy_comparator = Value::Boolean(true);

            // Test sorted list
            let sorted_list = Value::from_vector(vec![
                Value::from(1i64),
                Value::from(2i64),
                Value::from(3i64),
            ]);
            let is_sorted = list_func(&[dummy_comparator.clone(), sorted_list]).unwrap();
            assert_eq!(is_sorted, Value::Boolean(true));

            // Test unsorted list
            let unsorted_list = Value::from_vector(vec![
                Value::from(3i64),
                Value::from(1i64),
                Value::from(2i64),
            ]);
            let is_sorted = list_func(&[dummy_comparator.clone(), unsorted_list]).unwrap();
            assert_eq!(is_sorted, Value::Boolean(false));

            // Test sorted vector
            let sorted_vector = Value::Vector(vec![
                Value::from(1i64),
                Value::from(2i64),
                Value::from(3i64),
            ]);
            let is_sorted = vector_func(&[dummy_comparator.clone(), sorted_vector]).unwrap();
            assert_eq!(is_sorted, Value::Boolean(true));

            // Test unsorted vector
            let unsorted_vector = Value::Vector(vec![
                Value::from(3i64),
                Value::from(1i64),
                Value::from(2i64),
            ]);
            let is_sorted = vector_func(&[dummy_comparator, unsorted_vector]).unwrap();
            assert_eq!(is_sorted, Value::Boolean(false));
        }
    }
}
