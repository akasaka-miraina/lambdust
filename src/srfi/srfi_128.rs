//! SRFI 128: Comparators
//!
//! Comparators bundle together type test, equality, comparison, and hash functions
//! to support generic algorithms on different data types.

use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;
use std::rc::Rc;

// Type aliases to reduce complexity warnings
type TypeTestFn = Option<Rc<dyn Fn(&Value) -> bool>>;
type EqualityFn = Rc<dyn Fn(&Value, &Value) -> bool>;
type ComparisonFn = Option<Rc<dyn Fn(&Value, &Value) -> Result<i32>>>;
type HashFn = Option<Rc<dyn Fn(&Value) -> Result<i64>>>;

/// Comparator data structure
#[derive(Clone)]
pub struct Comparator {
    /// Type test procedure: obj -> boolean
    pub type_test: TypeTestFn,
    /// Equality procedure: obj1 obj2 -> boolean  
    pub equality: EqualityFn,
    /// Comparison procedure: obj1 obj2 -> -1|0|1
    pub comparison: ComparisonFn,
    /// Hash function: obj -> integer
    pub hash_fn: HashFn,
    /// Comparator name for debugging
    pub name: String,
}

impl Comparator {
    /// Create a new comparator
    pub fn new(
        name: String,
        type_test: TypeTestFn,
        equality: EqualityFn,
        comparison: ComparisonFn,
        hash_fn: HashFn,
    ) -> Self {
        Self {
            type_test,
            equality,
            comparison,
            hash_fn,
            name,
        }
    }

    /// Test if the comparator can handle the given object
    #[must_use] pub fn test_type(&self, obj: &Value) -> bool {
        match &self.type_test {
            Some(test) => test(obj),
            None => true, // Accept all types if no test provided
        }
    }

    /// Test equality of two objects
    #[must_use] pub fn equal(&self, obj1: &Value, obj2: &Value) -> bool {
        (self.equality)(obj1, obj2)
    }

    /// Compare two objects (-1: less, 0: equal, 1: greater)
    pub fn compare(&self, obj1: &Value, obj2: &Value) -> Result<i32> {
        match &self.comparison {
            Some(cmp) => cmp(obj1, obj2),
            None => Err(LambdustError::runtime_error(
                "Comparator does not support comparison".to_string(),
            )),
        }
    }

    /// Compute hash of an object
    pub fn hash(&self, obj: &Value) -> Result<i64> {
        match &self.hash_fn {
            Some(hash) => hash(obj),
            None => Err(LambdustError::runtime_error(
                "Comparator does not support hashing".to_string(),
            )),
        }
    }

    /// Check if comparator supports comparison
    #[must_use] pub fn has_comparison(&self) -> bool {
        self.comparison.is_some()
    }

    /// Check if comparator supports hashing
    #[must_use] pub fn has_hash(&self) -> bool {
        self.hash_fn.is_some()
    }
}

impl PartialEq for Comparator {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl std::fmt::Debug for Comparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Comparator")
            .field("name", &self.name)
            .field("has_type_test", &self.type_test.is_some())
            .field("has_comparison", &self.comparison.is_some())
            .field("has_hash", &self.hash_fn.is_some())
            .finish()
    }
}

/// Standard number comparator
#[must_use] pub fn default_number_comparator() -> Comparator {
    Comparator::new(
        "number-comparator".to_string(),
        Some(Rc::new(|obj| matches!(obj, Value::Number(_)))),
        Rc::new(|obj1, obj2| {
            if let (Value::Number(n1), Value::Number(n2)) = (obj1, obj2) {
                (n1.to_f64() - n2.to_f64()).abs() < f64::EPSILON
            } else {
                false
            }
        }),
        Some(Rc::new(|obj1, obj2| {
            if let (Value::Number(n1), Value::Number(n2)) = (obj1, obj2) {
                let f1 = n1.to_f64();
                let f2 = n2.to_f64();
                if f1 < f2 {
                    Ok(-1)
                } else if f1 > f2 {
                    Ok(1)
                } else {
                    Ok(0)
                }
            } else {
                Err(LambdustError::type_error("Expected numbers".to_string()))
            }
        })),
        Some(Rc::new(|obj| {
            if let Value::Number(n) = obj {
                Ok(n.to_f64().to_bits() as i64)
            } else {
                Err(LambdustError::type_error("Expected number".to_string()))
            }
        })),
    )
}

/// Standard string comparator
#[must_use] pub fn default_string_comparator() -> Comparator {
    Comparator::new(
        "string-comparator".to_string(),
        Some(Rc::new(|obj| matches!(obj, Value::String(_)))),
        Rc::new(|obj1, obj2| {
            if let (Value::String(s1), Value::String(s2)) = (obj1, obj2) {
                s1 == s2
            } else {
                false
            }
        }),
        Some(Rc::new(|obj1, obj2| {
            if let (Value::String(s1), Value::String(s2)) = (obj1, obj2) {
                Ok(s1.cmp(s2) as i32)
            } else {
                Err(LambdustError::type_error("Expected strings".to_string()))
            }
        })),
        Some(Rc::new(|obj| {
            if let Value::String(s) = obj {
                let mut hash: i64 = 0;
                for byte in s.bytes() {
                    hash = hash.wrapping_mul(31).wrapping_add(i64::from(byte));
                }
                Ok(hash)
            } else {
                Err(LambdustError::type_error("Expected string".to_string()))
            }
        })),
    )
}

/// Standard symbol comparator  
#[must_use] pub fn default_symbol_comparator() -> Comparator {
    Comparator::new(
        "symbol-comparator".to_string(),
        Some(Rc::new(|obj| matches!(obj, Value::Symbol(_)))),
        Rc::new(|obj1, obj2| {
            if let (Value::Symbol(s1), Value::Symbol(s2)) = (obj1, obj2) {
                s1 == s2
            } else {
                false
            }
        }),
        Some(Rc::new(|obj1, obj2| {
            if let (Value::Symbol(s1), Value::Symbol(s2)) = (obj1, obj2) {
                Ok(s1.cmp(s2) as i32)
            } else {
                Err(LambdustError::type_error("Expected symbols".to_string()))
            }
        })),
        Some(Rc::new(|obj| {
            if let Value::Symbol(s) = obj {
                let mut hash: i64 = 0;
                for byte in s.bytes() {
                    hash = hash.wrapping_mul(31).wrapping_add(i64::from(byte));
                }
                Ok(hash)
            } else {
                Err(LambdustError::type_error("Expected symbol".to_string()))
            }
        })),
    )
}

/// SRFI 128 implementation
pub struct Srfi128;

impl super::SrfiModule for Srfi128 {
    fn srfi_id(&self) -> u32 {
        128
    }

    fn name(&self) -> &'static str {
        "Comparators"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec![]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // comparator? predicate
        exports.insert(
            "comparator?".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "comparator?".to_string(),
                func: |args| {
                    if args.len() != 1 {
                        return Err(LambdustError::arity_error(1, args.len()));
                    }
                    Ok(Value::Boolean(matches!(args[0], Value::Comparator(_))))
                },
                arity: Some(1),
            }),
        );

        // comparator-ordered? predicate
        exports.insert(
            "comparator-ordered?".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "comparator-ordered?".to_string(),
                func: |args| {
                    if args.len() != 1 {
                        return Err(LambdustError::arity_error(1, args.len()));
                    }
                    if let Value::Comparator(comp) = &args[0] {
                        Ok(Value::Boolean(comp.has_comparison()))
                    } else {
                        Err(LambdustError::type_error("Expected comparator".to_string()))
                    }
                },
                arity: Some(1),
            }),
        );

        // comparator-hashable? predicate
        exports.insert(
            "comparator-hashable?".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "comparator-hashable?".to_string(),
                func: |args| {
                    if args.len() != 1 {
                        return Err(LambdustError::arity_error(1, args.len()));
                    }
                    if let Value::Comparator(comp) = &args[0] {
                        Ok(Value::Boolean(comp.has_hash()))
                    } else {
                        Err(LambdustError::type_error("Expected comparator".to_string()))
                    }
                },
                arity: Some(1),
            }),
        );

        // make-comparator constructor
        exports.insert(
            "make-comparator".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "make-comparator".to_string(),
                func: |args| {
                    if args.len() < 2 || args.len() > 4 {
                        return Err(LambdustError::arity_error_range(2, 4, args.len()));
                    }

                    // For now, create a simple comparator that works with basic types
                    // In a full implementation, we would parse the procedure arguments
                    let name = format!("custom-comparator-{}", std::ptr::addr_of!(args) as usize);

                    let comparator = Comparator::new(
                        name,
                        None, // Accept all types for simplicity
                        Rc::new(|obj1, obj2| {
                            // Basic equality comparison
                            match (obj1, obj2) {
                                (Value::Number(n1), Value::Number(n2)) => {
                                    (n1.to_f64() - n2.to_f64()).abs() < f64::EPSILON
                                }
                                (Value::String(s1), Value::String(s2)) => s1 == s2,
                                (Value::Symbol(s1), Value::Symbol(s2)) => s1 == s2,
                                (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
                                _ => false,
                            }
                        }),
                        Some(Rc::new(|obj1, obj2| {
                            // Basic comparison
                            match (obj1, obj2) {
                                (Value::Number(n1), Value::Number(n2)) => {
                                    let f1 = n1.to_f64();
                                    let f2 = n2.to_f64();
                                    if f1 < f2 {
                                        Ok(-1)
                                    } else if f1 > f2 {
                                        Ok(1)
                                    } else {
                                        Ok(0)
                                    }
                                }
                                (Value::String(s1), Value::String(s2)) => Ok(s1.cmp(s2) as i32),
                                (Value::Symbol(s1), Value::Symbol(s2)) => Ok(s1.cmp(s2) as i32),
                                _ => Err(LambdustError::type_error(
                                    "Cannot compare these types".to_string(),
                                )),
                            }
                        })),
                        None, // No hash function for custom comparators yet
                    );

                    Ok(Value::Comparator(Rc::new(comparator)))
                },
                arity: None, // Variable arity
            }),
        );

        // =? equality test
        exports.insert(
            "=?".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "=?".to_string(),
                func: |args| {
                    if args.len() < 3 {
                        return Err(LambdustError::arity_error_min(3, args.len()));
                    }

                    if let Value::Comparator(comp) = &args[0] {
                        // Compare all objects for equality
                        for i in 1..args.len() - 1 {
                            if !comp.equal(&args[i], &args[i + 1]) {
                                return Ok(Value::Boolean(false));
                            }
                        }
                        Ok(Value::Boolean(true))
                    } else {
                        Err(LambdustError::type_error("Expected comparator".to_string()))
                    }
                },
                arity: None, // Variable arity
            }),
        );

        // <? less than test
        exports.insert(
            "<?".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "<?".to_string(),
                func: |args| {
                    if args.len() < 3 {
                        return Err(LambdustError::arity_error_min(3, args.len()));
                    }

                    if let Value::Comparator(comp) = &args[0] {
                        // Check if all objects are in increasing order
                        for i in 1..args.len() - 1 {
                            match comp.compare(&args[i], &args[i + 1])? {
                                -1 => {},                              // Less than, good
                                _ => return Ok(Value::Boolean(false)), // Not less than
                            }
                        }
                        Ok(Value::Boolean(true))
                    } else {
                        Err(LambdustError::type_error("Expected comparator".to_string()))
                    }
                },
                arity: None, // Variable arity
            }),
        );

        // Standard comparators
        exports.insert(
            "default-comparator".to_string(),
            Value::Comparator(Rc::new(Comparator::new(
                "default-comparator".to_string(),
                None,                               // Accept all types
                Rc::new(|obj1, obj2| obj1 == obj2), // Use Value's PartialEq
                None,                               // No ordering for default comparator
                None,                               // No hash for default comparator
            ))),
        );

        exports.insert(
            "boolean-comparator".to_string(),
            Value::Comparator(Rc::new(Comparator::new(
                "boolean-comparator".to_string(),
                Some(Rc::new(|obj| matches!(obj, Value::Boolean(_)))),
                Rc::new(|obj1, obj2| {
                    if let (Value::Boolean(b1), Value::Boolean(b2)) = (obj1, obj2) {
                        b1 == b2
                    } else {
                        false
                    }
                }),
                Some(Rc::new(|obj1, obj2| {
                    if let (Value::Boolean(b1), Value::Boolean(b2)) = (obj1, obj2) {
                        Ok(match (b1, b2) {
                            (false, false) | (true, true) => 0,
                            (false, true) => -1,
                            (true, false) => 1,
                        })
                    } else {
                        Err(LambdustError::type_error("Expected booleans".to_string()))
                    }
                })),
                Some(Rc::new(|obj| {
                    if let Value::Boolean(b) = obj {
                        Ok(i64::from(*b))
                    } else {
                        Err(LambdustError::type_error("Expected boolean".to_string()))
                    }
                })),
            ))),
        );

        exports.insert(
            "real-comparator".to_string(),
            Value::Comparator(Rc::new(default_number_comparator())),
        );

        exports.insert(
            "string-comparator".to_string(),
            Value::Comparator(Rc::new(default_string_comparator())),
        );

        exports.insert(
            "symbol-comparator".to_string(),
            Value::Comparator(Rc::new(default_symbol_comparator())),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 128 has no parts, return all exports
        Ok(self.exports())
    }
}

