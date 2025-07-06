//! SRFI 125: Intermediate Hash Tables
//!
//! This SRFI extends SRFI 69 with additional hash table procedures.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;

/// SRFI 125 implementation
pub struct Srfi125;

impl super::SrfiModule for Srfi125 {
    fn srfi_id(&self) -> u32 {
        125
    }

    fn name(&self) -> &'static str {
        "Intermediate Hash Tables"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec![]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // hash-table-unfold procedure
        exports.insert(
            "hash-table-unfold".to_string(),
            make_builtin_procedure("hash-table-unfold", Some(4), |args| {
                check_arity(args, 4)?;

                // For now, return a placeholder
                Err(LambdustError::runtime_error(
                    "hash-table-unfold not yet implemented".to_string(),
                ))
            }),
        );

        // hash-table-find procedure
        exports.insert(
            "hash-table-find".to_string(),
            make_builtin_procedure("hash-table-find", Some(3), |args| {
                check_arity(args, 3)?;

                // Get hash table
                if let Value::HashTable(ht) = &args[1] {
                    let table = ht.borrow();
                    let key = &args[2];

                    // Convert key to HashKey first
                    if let Ok(hash_key) = crate::srfi::srfi_69::types::HashKey::from_value(key) {
                        match table.get(&hash_key) {
                            Some(value) => {
                                // Return (values key value)
                                Ok(Value::Values(vec![key.clone(), value.clone()]))
                            }
                            None => {
                                // Return failure result (simplified)
                                Ok(Value::Boolean(false))
                            }
                        }
                    } else {
                        Ok(Value::Boolean(false))
                    }
                } else {
                    Err(LambdustError::type_error("Expected hash table".to_string()))
                }
            }),
        );

        // hash-table-count procedure
        exports.insert(
            "hash-table-count".to_string(),
            make_builtin_procedure("hash-table-count", Some(2), |args| {
                check_arity(args, 2)?;

                // Get hash table
                if let Value::HashTable(ht) = &args[1] {
                    let table = ht.borrow();

                    // Count elements matching predicate (simplified: count all for now)
                    let count = table.size();
                    Ok(Value::from(count as i64))
                } else {
                    Err(LambdustError::type_error("Expected hash table".to_string()))
                }
            }),
        );

        // hash-table-map->list procedure
        exports.insert(
            "hash-table-map->list".to_string(),
            make_builtin_procedure("hash-table-map->list", Some(2), |args| {
                check_arity(args, 2)?;

                // Get hash table
                if let Value::HashTable(ht) = &args[1] {
                    let table = ht.borrow();

                    // Convert to list of key-value pairs (simplified)
                    let mut result = Vec::new();
                    for (hash_key, value) in table.table.iter() {
                        // Create a simple pair representation
                        result
                            .push(Value::from_vector(vec![hash_key.to_value(), value.clone()]));
                    }

                    Ok(Value::from_vector(result))
                } else {
                    Err(LambdustError::type_error("Expected hash table".to_string()))
                }
            }),
        );

        // hash-table-for-each procedure
        exports.insert(
            "hash-table-for-each".to_string(),
            make_builtin_procedure("hash-table-for-each", Some(2), |args| {
                check_arity(args, 2)?;

                // Get hash table
                if let Value::HashTable(ht) = &args[1] {
                    let _table = ht.borrow();

                    // For now, just return undefined as we would need evaluator integration
                    // to properly call the procedure on each element
                    Ok(Value::Undefined)
                } else {
                    Err(LambdustError::type_error("Expected hash table".to_string()))
                }
            }),
        );

        // hash-table-map! procedure
        exports.insert(
            "hash-table-map!".to_string(),
            make_builtin_procedure("hash-table-map!", Some(2), |args| {
                check_arity(args, 2)?;

                // Get hash table
                if let Value::HashTable(ht) = &args[1] {
                    let _table = ht.borrow();

                    // Destructive mapping would require evaluator integration
                    // For now, return undefined
                    Ok(Value::Undefined)
                } else {
                    Err(LambdustError::type_error("Expected hash table".to_string()))
                }
            }),
        );

        // hash-table-filter! procedure
        exports.insert(
            "hash-table-filter!".to_string(),
            make_builtin_procedure("hash-table-filter!", Some(2), |args| {
                check_arity(args, 2)?;

                // Get hash table
                if let Value::HashTable(ht) = &args[1] {
                    let _table = ht.borrow();

                    // Destructive filtering would require evaluator integration
                    // For now, return undefined
                    Ok(Value::Undefined)
                } else {
                    Err(LambdustError::type_error("Expected hash table".to_string()))
                }
            }),
        );

        // hash-table-remove! procedure
        exports.insert(
            "hash-table-remove!".to_string(),
            make_builtin_procedure("hash-table-remove!", Some(2), |args| {
                check_arity(args, 2)?;

                // Get hash table and key
                if let Value::HashTable(ht) = &args[0] {
                    let mut table = ht.borrow_mut();
                    let key = &args[1];

                    // Remove the key-value pair
                    if let Ok(hash_key) = crate::srfi::srfi_69::types::HashKey::from_value(key) {
                        match table.remove(&hash_key) {
                            Some(_) => Ok(Value::Boolean(true)),
                            None => Ok(Value::Boolean(false)),
                        }
                    } else {
                        Ok(Value::Boolean(false))
                    }
                } else {
                    Err(LambdustError::type_error("Expected hash table".to_string()))
                }
            }),
        );

        // hash-table-clear! procedure
        exports.insert(
            "hash-table-clear!".to_string(),
            make_builtin_procedure("hash-table-clear!", Some(1), |args| {
                check_arity(args, 1)?;

                // Get hash table
                if let Value::HashTable(ht) = &args[0] {
                    let mut table = ht.borrow_mut();

                    // Clear all entries
                    table.clear();
                    Ok(Value::Undefined)
                } else {
                    Err(LambdustError::type_error("Expected hash table".to_string()))
                }
            }),
        );

        // hash-table-union! procedure
        exports.insert(
            "hash-table-union!".to_string(),
            make_builtin_procedure("hash-table-union!", None, |args| {
                if args.len() < 2 {
                    return Err(LambdustError::arity_error(2, args.len()));
                }

                // Get destination hash table
                if let Value::HashTable(ht1) = &args[0] {
                    let mut table1 = ht1.borrow_mut();

                    // Union with other hash tables
                    for other_arg in &args[1..] {
                        if let Value::HashTable(ht2) = other_arg {
                            let table2 = ht2.borrow();
                            for (hash_key, value) in table2.table.iter() {
                                table1.table.insert(hash_key.clone(), value.clone());
                            }
                        } else {
                            return Err(LambdustError::type_error(
                                "Expected hash table".to_string(),
                            ));
                        }
                    }

                    Ok(Value::Undefined)
                } else {
                    Err(LambdustError::type_error("Expected hash table".to_string()))
                }
            }),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 125 has no parts, return all exports
        Ok(self.exports())
    }
}

