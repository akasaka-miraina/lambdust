//! SRFI 125: Intermediate Hash Tables
//!
//! This SRFI extends SRFI 69 with additional hash table procedures.

use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
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
            Value::Procedure(Procedure::Builtin {
                name: "hash-table-unfold".to_string(),
                func: |args| {
                    if args.len() != 4 {
                        return Err(LambdustError::arity_error(4, args.len()));
                    }

                    // For now, return a placeholder
                    Err(LambdustError::runtime_error(
                        "hash-table-unfold not yet implemented".to_string(),
                    ))
                },
                arity: Some(4),
            }),
        );

        // hash-table-find procedure
        exports.insert(
            "hash-table-find".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "hash-table-find".to_string(),
                func: |args| {
                    if args.len() != 3 {
                        return Err(LambdustError::arity_error(3, args.len()));
                    }

                    // Get hash table
                    if let Value::HashTable(ht) = &args[1] {
                        let table = ht.borrow();
                        let key = &args[2];

                        // Simple key lookup using HashTable API
                        match table.get(key) {
                            Ok(Some(value)) => {
                                // Return (values key value)
                                Ok(Value::Values(vec![key.clone(), value]))
                            }
                            _ => {
                                // Return failure result (simplified)
                                Ok(Value::Boolean(false))
                            }
                        }
                    } else {
                        Err(LambdustError::type_error("Expected hash table".to_string()))
                    }
                },
                arity: Some(3),
            }),
        );

        // hash-table-count procedure
        exports.insert(
            "hash-table-count".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "hash-table-count".to_string(),
                func: |args| {
                    if args.len() != 2 {
                        return Err(LambdustError::arity_error(2, args.len()));
                    }

                    // Get hash table
                    if let Value::HashTable(ht) = &args[1] {
                        let table = ht.borrow();

                        // Count elements matching predicate (simplified: count all for now)
                        let count = table.size();
                        Ok(Value::from(count as i64))
                    } else {
                        Err(LambdustError::type_error("Expected hash table".to_string()))
                    }
                },
                arity: Some(2),
            }),
        );

        // hash-table-map->list procedure
        exports.insert(
            "hash-table-map->list".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "hash-table-map->list".to_string(),
                func: |args| {
                    if args.len() != 2 {
                        return Err(LambdustError::arity_error(2, args.len()));
                    }

                    // Get hash table
                    if let Value::HashTable(ht) = &args[1] {
                        let table = ht.borrow();

                        // Convert to list of key-value pairs (simplified)
                        let mut result = Vec::new();
                        for (hash_key, value) in table.iter() {
                            // Create a simple pair representation
                            result
                                .push(Value::from_vector(vec![hash_key.to_value(), value.clone()]));
                        }

                        Ok(Value::from_vector(result))
                    } else {
                        Err(LambdustError::type_error("Expected hash table".to_string()))
                    }
                },
                arity: Some(2),
            }),
        );

        // hash-table-for-each procedure
        exports.insert(
            "hash-table-for-each".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "hash-table-for-each".to_string(),
                func: |args| {
                    if args.len() != 2 {
                        return Err(LambdustError::arity_error(2, args.len()));
                    }

                    // Get hash table
                    if let Value::HashTable(ht) = &args[1] {
                        let _table = ht.borrow();

                        // For now, just return undefined as we would need evaluator integration
                        // to properly call the procedure on each element
                        Ok(Value::Undefined)
                    } else {
                        Err(LambdustError::type_error("Expected hash table".to_string()))
                    }
                },
                arity: Some(2),
            }),
        );

        // hash-table-map! procedure
        exports.insert(
            "hash-table-map!".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "hash-table-map!".to_string(),
                func: |args| {
                    if args.len() != 2 {
                        return Err(LambdustError::arity_error(2, args.len()));
                    }

                    // Get hash table
                    if let Value::HashTable(ht) = &args[1] {
                        let _table = ht.borrow();

                        // Destructive mapping would require evaluator integration
                        // For now, return undefined
                        Ok(Value::Undefined)
                    } else {
                        Err(LambdustError::type_error("Expected hash table".to_string()))
                    }
                },
                arity: Some(2),
            }),
        );

        // hash-table-filter! procedure
        exports.insert(
            "hash-table-filter!".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "hash-table-filter!".to_string(),
                func: |args| {
                    if args.len() != 2 {
                        return Err(LambdustError::arity_error(2, args.len()));
                    }

                    // Get hash table
                    if let Value::HashTable(ht) = &args[1] {
                        let _table = ht.borrow();

                        // Destructive filtering would require evaluator integration
                        // For now, return undefined
                        Ok(Value::Undefined)
                    } else {
                        Err(LambdustError::type_error("Expected hash table".to_string()))
                    }
                },
                arity: Some(2),
            }),
        );

        // hash-table-remove! procedure
        exports.insert(
            "hash-table-remove!".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "hash-table-remove!".to_string(),
                func: |args| {
                    if args.len() != 2 {
                        return Err(LambdustError::arity_error(2, args.len()));
                    }

                    // Get hash table and key
                    if let Value::HashTable(ht) = &args[0] {
                        let mut table = ht.borrow_mut();
                        let key = &args[1];

                        // Remove the key-value pair
                        match table.remove(key) {
                            Ok(Some(_)) => Ok(Value::Boolean(true)),
                            _ => Ok(Value::Boolean(false)),
                        }
                    } else {
                        Err(LambdustError::type_error("Expected hash table".to_string()))
                    }
                },
                arity: Some(2),
            }),
        );

        // hash-table-clear! procedure
        exports.insert(
            "hash-table-clear!".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "hash-table-clear!".to_string(),
                func: |args| {
                    if args.len() != 1 {
                        return Err(LambdustError::arity_error(1, args.len()));
                    }

                    // Get hash table
                    if let Value::HashTable(ht) = &args[0] {
                        let mut table = ht.borrow_mut();

                        // Clear all entries
                        table.clear();
                        Ok(Value::Undefined)
                    } else {
                        Err(LambdustError::type_error("Expected hash table".to_string()))
                    }
                },
                arity: Some(1),
            }),
        );

        // hash-table-union! procedure
        exports.insert(
            "hash-table-union!".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "hash-table-union!".to_string(),
                func: |args| {
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
                                for (hash_key, value) in table2.iter() {
                                    table1.insert_raw(hash_key.clone(), value.clone());
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
                },
                arity: None, // Variable arity
            }),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 125 has no parts, return all exports
        Ok(self.exports())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srfi::SrfiModule;
    use crate::srfi::srfi_69::HashTable;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_hash_table_find() {
        let srfi = Srfi125;
        let exports = srfi.exports();

        let find_proc = exports.get("hash-table-find").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = find_proc {
            // Create a hash table
            let mut ht = HashTable::new();
            ht.set(Value::String("key1".to_string()), Value::from(42i64))
                .unwrap();
            let hash_table = Value::HashTable(Rc::new(RefCell::new(ht)));

            // Test finding existing key
            let dummy_proc = Value::Boolean(true); // Placeholder for procedure
            let result = func(&[
                dummy_proc,
                hash_table.clone(),
                Value::String("key1".to_string()),
            ])
            .unwrap();

            // Should return values with key and value
            if let Value::Values(values) = result {
                assert_eq!(values.len(), 2);
                assert_eq!(values[0], Value::String("key1".to_string()));
                assert_eq!(values[1], Value::from(42i64));
            } else {
                panic!("Test assertion failed: Expected Values result");
            }
        }
    }

    #[test]
    fn test_hash_table_count() {
        let srfi = Srfi125;
        let exports = srfi.exports();

        let count_proc = exports.get("hash-table-count").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = count_proc {
            // Create a hash table with some entries
            let mut ht = HashTable::new();
            ht.set(Value::String("key1".to_string()), Value::from(1i64))
                .unwrap();
            ht.set(Value::String("key2".to_string()), Value::from(2i64))
                .unwrap();
            ht.set(Value::String("key3".to_string()), Value::from(3i64))
                .unwrap();
            let hash_table = Value::HashTable(Rc::new(RefCell::new(ht)));

            let dummy_proc = Value::Boolean(true); // Placeholder for procedure
            let result = func(&[dummy_proc, hash_table]).unwrap();

            assert_eq!(result, Value::from(3i64));
        }
    }

    #[test]
    fn test_hash_table_remove() {
        let srfi = Srfi125;
        let exports = srfi.exports();

        let remove_proc = exports.get("hash-table-remove!").unwrap();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = remove_proc {
            // Create a hash table with entries
            let mut ht = HashTable::new();
            ht.set(Value::String("key1".to_string()), Value::from(1i64))
                .unwrap();
            ht.set(Value::String("key2".to_string()), Value::from(2i64))
                .unwrap();
            let hash_table = Value::HashTable(Rc::new(RefCell::new(ht)));

            // Test removing existing key
            let result = func(&[hash_table.clone(), Value::String("key1".to_string())]).unwrap();
            assert_eq!(result, Value::Boolean(true));

            // Test removing non-existent key
            let result = func(&[hash_table, Value::String("key3".to_string())]).unwrap();
            assert_eq!(result, Value::Boolean(false));
        }
    }
}
