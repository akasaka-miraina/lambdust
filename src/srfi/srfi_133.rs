//! SRFI 133: Vector Libraries
//!
//! This SRFI provides a complete library of vector manipulation procedures.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// SRFI 133 implementation
pub struct Srfi133;

impl super::SrfiModule for Srfi133 {
    fn srfi_id(&self) -> u32 {
        133
    }

    fn name(&self) -> &'static str {
        "Vector Libraries"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec![]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // vector-empty? predicate
        exports.insert(
            "vector-empty?".to_string(),
            make_builtin_procedure("vector-empty?", Some(1), |args| {
                check_arity(args, 1)?;

                if let Value::Vector(vec) = &args[0] {
                    Ok(Value::Boolean(vec.is_empty()))
                } else {
                    Err(LambdustError::type_error("Expected vector".to_string()))
                }
            }),
        );

        // vector-count procedure
        exports.insert(
            "vector-count".to_string(),
            make_builtin_procedure("vector-count", None, |args| {
                if args.len() < 2 {
                    return Err(LambdustError::arity_error(2, args.len()));
                }

                // For now, implement simple predicate counting
                let _predicate = &args[0];
                let vector = &args[1];

                if let Value::Vector(vec) = vector {
                    // Count non-false values for now (simplified predicate)
                    let count = vec.iter().filter(|v| v.is_truthy()).count();
                    Ok(Value::from(count as i64))
                } else {
                    Err(LambdustError::type_error("Expected vector".to_string()))
                }
            }),
        );

        // vector-cumulate procedure
        exports.insert(
            "vector-cumulate".to_string(),
            make_builtin_procedure("vector-cumulate", Some(3), |args| {
                check_arity(args, 3)?;

                let _combiner = &args[0];
                let initial = &args[1];
                let vector = &args[2];

                if let Value::Vector(vec) = vector {
                    let mut result = Vec::new();
                    let mut acc = initial.clone();

                    result.push(acc.clone());

                    for value in vec {
                        // For now, implement addition for numeric values
                        if let (Some(acc_num), Some(val_num)) =
                            (acc.as_number(), value.as_number())
                        {
                            use crate::lexer::SchemeNumber;
                            let sum = match (acc_num, val_num) {
                                (SchemeNumber::Integer(a), SchemeNumber::Integer(b)) => {
                                    Value::from(a + b)
                                }
                                (SchemeNumber::Real(a), SchemeNumber::Real(b)) => {
                                    Value::from(a + b)
                                }
                                (SchemeNumber::Integer(a), SchemeNumber::Real(b)) => {
                                    Value::from(*a as f64 + b)
                                }
                                (SchemeNumber::Real(a), SchemeNumber::Integer(b)) => {
                                    Value::from(a + *b as f64)
                                }
                                _ => value.clone(), // Fallback
                            };
                            acc = sum;
                        } else {
                            acc = value.clone();
                        }
                        result.push(acc.clone());
                    }

                    Ok(Value::Vector(result))
                } else {
                    Err(LambdustError::type_error("Expected vector".to_string()))
                }
            }),
        );

        // vector-index procedure
        exports.insert(
            "vector-index".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "vector-index".to_string(),
                func: |args| {
                    if args.len() < 2 {
                        return Err(LambdustError::arity_error(2, args.len()));
                    }

                    let _predicate = &args[0];
                    let vector = &args[1];

                    if let Value::Vector(vec) = vector {
                        // Find first truthy value index (simplified predicate)
                        for (i, value) in vec.iter().enumerate() {
                            if value.is_truthy() {
                                return Ok(Value::from(i as i64));
                            }
                        }
                        Ok(Value::Boolean(false)) // Not found
                    } else {
                        Err(LambdustError::type_error("Expected vector".to_string()))
                    }
                },
                arity: None, // Variable arity
            }),
        );

        // vector-take procedure
        exports.insert(
            "vector-take".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "vector-take".to_string(),
                func: |args| {
                    if args.len() != 2 {
                        return Err(LambdustError::arity_error(2, args.len()));
                    }

                    let vector = &args[0];
                    let count = &args[1];

                    if let (Value::Vector(vec), Some(n)) = (vector, count.as_number()) {
                        let take_count = match n {
                            crate::lexer::SchemeNumber::Integer(i) => *i as usize,
                            _ => {
                                return Err(LambdustError::type_error(
                                    "Expected integer count".to_string(),
                                ));
                            }
                        };

                        if take_count > vec.len() {
                            return Err(LambdustError::runtime_error(
                                "Count exceeds vector length".to_string(),
                            ));
                        }

                        let taken: Vec<Value> = vec.iter().take(take_count).cloned().collect();
                        Ok(Value::Vector(taken))
                    } else {
                        Err(LambdustError::type_error(
                            "Expected vector and integer".to_string(),
                        ))
                    }
                },
                arity: Some(2),
            }),
        );

        // vector-drop procedure
        exports.insert(
            "vector-drop".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "vector-drop".to_string(),
                func: |args| {
                    if args.len() != 2 {
                        return Err(LambdustError::arity_error(2, args.len()));
                    }

                    let vector = &args[0];
                    let count = &args[1];

                    if let (Value::Vector(vec), Some(n)) = (vector, count.as_number()) {
                        let drop_count = match n {
                            crate::lexer::SchemeNumber::Integer(i) => *i as usize,
                            _ => {
                                return Err(LambdustError::type_error(
                                    "Expected integer count".to_string(),
                                ));
                            }
                        };

                        if drop_count > vec.len() {
                            return Ok(Value::Vector(Vec::new()));
                        }

                        let dropped: Vec<Value> = vec.iter().skip(drop_count).cloned().collect();
                        Ok(Value::Vector(dropped))
                    } else {
                        Err(LambdustError::type_error(
                            "Expected vector and integer".to_string(),
                        ))
                    }
                },
                arity: Some(2),
            }),
        );

        // vector-concatenate procedure
        exports.insert(
            "vector-concatenate".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "vector-concatenate".to_string(),
                func: |args| {
                    if args.len() != 1 {
                        return Err(LambdustError::arity_error(1, args.len()));
                    }

                    let list_of_vectors = &args[0];

                    // Convert list to vector of vectors
                    let vectors_list = if let Some(vec) = list_of_vectors.to_vector() {
                        vec
                    } else {
                        return Err(LambdustError::type_error(
                            "Expected list of vectors".to_string(),
                        ));
                    };

                    let mut result = Vec::new();
                    for vector_val in vectors_list {
                        if let Value::Vector(vec) = vector_val {
                            result.extend(vec.iter().cloned());
                        } else {
                            return Err(LambdustError::type_error(
                                "Expected vector in list".to_string(),
                            ));
                        }
                    }

                    Ok(Value::Vector(result))
                },
                arity: Some(1),
            }),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 133 has no parts, return all exports
        Ok(self.exports())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srfi::SrfiModule;

    #[test]
    fn test_vector_empty() {
        let srfi = Srfi133;
        let exports = srfi.exports();

        let empty_proc = exports.get("vector-empty?").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = empty_proc {
            // Test empty vector
            let empty_vec = Value::Vector(vec![]);
            let result = func(&[empty_vec]).unwrap();
            assert_eq!(result, Value::Boolean(true));

            // Test non-empty vector
            let non_empty_vec = Value::Vector(vec![Value::from(1i64)]);
            let result = func(&[non_empty_vec]).unwrap();
            assert_eq!(result, Value::Boolean(false));
        }
    }

    #[test]
    fn test_vector_take_drop() {
        let srfi = Srfi133;
        let exports = srfi.exports();

        let take_proc = exports.get("vector-take").unwrap();
        let drop_proc = exports.get("vector-drop").unwrap();

        if let (
            Value::Procedure(Procedure::Builtin {
                func: take_func, ..
            }),
            Value::Procedure(Procedure::Builtin {
                func: drop_func, ..
            }),
        ) = (take_proc, drop_proc)
        {
            // Test vector
            let test_vec = Value::Vector(vec![
                Value::from(1i64),
                Value::from(2i64),
                Value::from(3i64),
                Value::from(4i64),
                Value::from(5i64),
            ]);

            // Test take
            let taken = take_func(&[test_vec.clone(), Value::from(3i64)]).unwrap();
            if let Value::Vector(ref taken_vec) = taken {
                assert_eq!(taken_vec.len(), 3);
                assert_eq!(taken_vec[0], Value::from(1i64));
                assert_eq!(taken_vec[1], Value::from(2i64));
                assert_eq!(taken_vec[2], Value::from(3i64));
            } else {
                panic!("Test assertion failed: Expected vector result from take");
            }

            // Test drop
            let dropped = drop_func(&[test_vec, Value::from(2i64)]).unwrap();
            if let Value::Vector(ref dropped_vec) = dropped {
                assert_eq!(dropped_vec.len(), 3);
                assert_eq!(dropped_vec[0], Value::from(3i64));
                assert_eq!(dropped_vec[1], Value::from(4i64));
                assert_eq!(dropped_vec[2], Value::from(5i64));
            } else {
                panic!("Test assertion failed: Expected vector result from drop");
            }
        }
    }

    #[test]
    fn test_vector_concatenate() {
        let srfi = Srfi133;
        let exports = srfi.exports();

        let concat_proc = exports.get("vector-concatenate").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = concat_proc {
            // Create list of vectors
            let vec1 = Value::Vector(vec![Value::from(1i64), Value::from(2i64)]);
            let vec2 = Value::Vector(vec![Value::from(3i64), Value::from(4i64)]);
            let vec3 = Value::Vector(vec![Value::from(5i64)]);

            let list_of_vectors = Value::from_vector(vec![vec1, vec2, vec3]);

            let result = func(&[list_of_vectors]).unwrap();

            if let Value::Vector(ref result_vec) = result {
                assert_eq!(result_vec.len(), 5);
                assert_eq!(result_vec[0], Value::from(1i64));
                assert_eq!(result_vec[1], Value::from(2i64));
                assert_eq!(result_vec[2], Value::from(3i64));
                assert_eq!(result_vec[3], Value::from(4i64));
                assert_eq!(result_vec[4], Value::from(5i64));
            } else {
                panic!("Test assertion failed: Expected vector result from concatenate");
            }
        }
    }
}
