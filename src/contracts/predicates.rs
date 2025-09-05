//! Contract predicate system with built-in and custom predicates.
//!
//! This module provides the foundation for contract predicates - functions
//! that test whether values satisfy certain conditions. It includes:
//!
//! - Built-in type predicates (number?, string?, etc.)
//! - Custom user-defined predicates
//! - Predicate combinators and utilities
//! - Optimized predicate evaluation

use crate::ast::Literal;
use crate::diagnostics::{Span, Spanned};
use crate::eval::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

/// A contract predicate that tests values.
pub type PredicateFn = Arc<dyn Fn(&Value) -> bool + Send + Sync>;

/// Contract predicate with metadata.
#[derive(Clone)]
pub struct ContractPredicate {
    /// The predicate function
    pub predicate: PredicateFn,
    /// Human-readable name
    pub name: String,
    /// Description of what the predicate checks
    pub description: String,
    /// Whether this predicate is deterministic
    pub deterministic: bool,
    /// Expected performance characteristics
    pub complexity: PredicateComplexity,
}

/// Performance complexity of predicates.
#[derive(Debug, Clone, PartialEq)]
pub enum PredicateComplexity {
    /// O(1) - constant time
    Constant,
    /// O(n) - linear in input size
    Linear,
    /// O(n²) - quadratic in input size
    Quadratic,
    /// O(log n) - logarithmic in input size
    Logarithmic,
    /// Unknown or variable complexity
    Unknown,
}

/// Registry for contract predicates.
#[derive(Debug, Clone)]
pub struct PredicateRegistry {
    /// Built-in predicates
    builtin_predicates: Arc<RwLock<HashMap<String, ContractPredicate>>>,
    /// User-defined predicates
    user_predicates: Arc<RwLock<HashMap<String, ContractPredicate>>>,
}

impl PredicateRegistry {
    /// Creates a new predicate registry with built-in predicates.
    pub fn new() -> Self {
        let mut registry = Self {
            builtin_predicates: Arc::new(RwLock::new(HashMap::new())),
            user_predicates: Arc::new(RwLock::new(HashMap::new())),
        };

        registry.register_builtin_predicates();
        registry
    }

    /// Registers all built-in predicates.
    fn register_builtin_predicates(&mut self) {
        self.register_builtin("any/c", any_predicate(), "matches any value");
        self.register_builtin("none/c", none_predicate(), "matches no value");

        // Type predicates
        self.register_builtin("number?", number_predicate(), "tests for numbers");
        self.register_builtin("real?", real_predicate(), "tests for real numbers");
        self.register_builtin(
            "rational?",
            rational_predicate(),
            "tests for rational numbers",
        );
        self.register_builtin("integer?", integer_predicate(), "tests for integers");
        self.register_builtin("exact?", exact_predicate(), "tests for exact numbers");
        self.register_builtin("inexact?", inexact_predicate(), "tests for inexact numbers");
        self.register_builtin("complex?", complex_predicate(), "tests for complex numbers");

        self.register_builtin("string?", string_predicate(), "tests for strings");
        self.register_builtin("char?", char_predicate(), "tests for characters");
        self.register_builtin("boolean?", boolean_predicate(), "tests for booleans");
        self.register_builtin("symbol?", symbol_predicate(), "tests for symbols");
        self.register_builtin("keyword?", keyword_predicate(), "tests for keywords");

        // Collection predicates
        self.register_builtin("list?", list_predicate(), "tests for proper lists");
        self.register_builtin("pair?", pair_predicate(), "tests for pairs");
        self.register_builtin("null?", null_predicate(), "tests for empty list");
        self.register_builtin("vector?", vector_predicate(), "tests for vectors");
        self.register_builtin("hash?", hash_predicate(), "tests for hash tables");

        // Procedure predicates
        self.register_builtin("procedure?", procedure_predicate(), "tests for procedures");
        self.register_builtin(
            "primitive?",
            primitive_predicate(),
            "tests for primitive procedures",
        );

        // Advanced predicates
        self.register_builtin("port?", port_predicate(), "tests for I/O ports");
        self.register_builtin("promise?", promise_predicate(), "tests for promises");
        self.register_builtin("syntax?", syntax_predicate(), "tests for syntax objects");

        // Numeric range predicates
        self.register_builtin(
            "positive?",
            positive_predicate(),
            "tests for positive numbers",
        );
        self.register_builtin(
            "negative?",
            negative_predicate(),
            "tests for negative numbers",
        );
        self.register_builtin("zero?", zero_predicate(), "tests for zero");
        self.register_builtin("even?", even_predicate(), "tests for even integers");
        self.register_builtin("odd?", odd_predicate(), "tests for odd integers");

        // String predicates
        self.register_builtin(
            "non-empty-string?",
            non_empty_string_predicate(),
            "tests for non-empty strings",
        );

        // Collection size predicates
        self.register_builtin("empty?", empty_predicate(), "tests for empty collections");
        self.register_builtin(
            "non-empty?",
            non_empty_predicate(),
            "tests for non-empty collections",
        );
    }

    /// Registers a built-in predicate.
    fn register_builtin(&mut self, name: &str, predicate: PredicateFn, description: &str) {
        let contract_predicate = ContractPredicate {
            predicate,
            name: name.to_string(),
            description: description.to_string(),
            deterministic: true,
            complexity: PredicateComplexity::Constant,
        };

        let mut builtin = self.builtin_predicates.write().unwrap();
        builtin.insert(name.to_string(), contract_predicate);
    }

    /// Registers a user-defined predicate.
    pub fn register(&mut self, name: String, predicate: ContractPredicate) {
        let mut user = self.user_predicates.write().unwrap();
        user.insert(name, predicate);
    }

    /// Looks up a predicate by name.
    pub fn lookup(&self, name: &str) -> Option<ContractPredicate> {
        // Check user predicates first (they can override built-ins)
        {
            let user = self.user_predicates.read().unwrap();
            if let Some(predicate) = user.get(name) {
                return Some(predicate.clone());
            }
        }

        // Check built-in predicates
        {
            let builtin = self.builtin_predicates.read().unwrap();
            builtin.get(name).cloned()
        }
    }

    /// Lists all available predicate names.
    pub fn list_predicates(&self) -> Vec<String> {
        let mut names = Vec::new();

        {
            let builtin = self.builtin_predicates.read().unwrap();
            names.extend(builtin.keys().cloned());
        }

        {
            let user = self.user_predicates.read().unwrap();
            names.extend(user.keys().cloned());
        }

        names.sort();
        names.dedup(); // Remove duplicates where user predicates override built-ins
        names
    }

    /// Gets information about a predicate.
    pub fn predicate_info(&self, name: &str) -> Option<PredicateInfo> {
        self.lookup(name).map(|pred| PredicateInfo {
            name: pred.name,
            description: pred.description,
            deterministic: pred.deterministic,
            complexity: pred.complexity,
            is_builtin: self.builtin_predicates.read().unwrap().contains_key(name),
        })
    }
}

/// Information about a predicate.
#[derive(Debug, Clone)]
pub struct PredicateInfo {
    /// Name of the predicate
    pub name: String,
    /// Human-readable description of what the predicate checks
    pub description: String,
    /// Whether the predicate is deterministic (always returns same result for same input)
    pub deterministic: bool,
    /// Computational complexity classification
    pub complexity: PredicateComplexity,
    /// Whether this is a built-in system predicate
    pub is_builtin: bool,
}

impl ContractPredicate {
    /// Creates a new contract predicate.
    pub fn new(name: String, description: String, predicate: PredicateFn) -> Self {
        Self {
            predicate,
            name,
            description,
            deterministic: true,
            complexity: PredicateComplexity::Unknown,
        }
    }

    /// Creates a predicate with custom properties.
    pub fn with_properties(
        name: String,
        description: String,
        predicate: PredicateFn,
        deterministic: bool,
        complexity: PredicateComplexity,
    ) -> Self {
        Self {
            predicate,
            name,
            description,
            deterministic,
            complexity,
        }
    }

    /// Tests a value against this predicate.
    pub fn test(&self, value: &Value) -> bool {
        (self.predicate)(value)
    }
}

impl fmt::Debug for ContractPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContractPredicate")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("deterministic", &self.deterministic)
            .field("complexity", &self.complexity)
            .finish()
    }
}

// ============= BUILT-IN PREDICATE IMPLEMENTATIONS =============

/// Always returns true (matches any value).
pub fn any_predicate() -> PredicateFn {
    Arc::new(|_| true)
}

/// Always returns false (matches no value).
pub fn none_predicate() -> PredicateFn {
    Arc::new(|_| false)
}

/// Tests for numbers.
pub fn number_predicate() -> PredicateFn {
    Arc::new(|value| {
        matches!(
            value,
            Value::Literal(Literal::ExactInteger(_))
                | Value::Literal(Literal::InexactReal(_))
                | Value::Literal(Literal::Number(_))
                | Value::Literal(Literal::Integer(_))
                | Value::Literal(Literal::Rational(_))
                | Value::Literal(Literal::Complex(_))
        )
    })
}

/// Tests for real numbers.
pub fn real_predicate() -> PredicateFn {
    Arc::new(|value| {
        matches!(
            value,
            Value::Literal(Literal::ExactInteger(_))
                | Value::Literal(Literal::InexactReal(_)) 
                | Value::Literal(Literal::Number(_))
                | Value::Literal(Literal::Integer(_))
                | Value::Literal(Literal::Rational(_))
        )
    })
}

/// Tests for rational numbers.
pub fn rational_predicate() -> PredicateFn {
    Arc::new(|value| {
        matches!(
            value,
            Value::Literal(Literal::ExactInteger(_))
                | Value::Literal(Literal::InexactReal(_))
                | Value::Literal(Literal::Number(_))
                | Value::Literal(Literal::Integer(_))
                | Value::Literal(Literal::Rational(_))
        )
    })
}

/// Tests for integers.
pub fn integer_predicate() -> PredicateFn {
    Arc::new(|value| match value {
        Value::Literal(Literal::InexactReal(n)) | Value::Literal(Literal::Number(n)) => {
            n.fract() == 0.0
        }
        Value::Literal(Literal::ExactInteger(_)) | Value::Literal(Literal::Integer(_)) => true,
        _ => false,
    })
}

/// Tests for exact numbers.
pub fn exact_predicate() -> PredicateFn {
    Arc::new(|value| {
        matches!(
            value,
            Value::Literal(Literal::ExactInteger(_))
                | Value::Literal(Literal::Integer(_))
                | Value::Literal(Literal::Rational(_))
        )
    })
}

/// Tests for inexact numbers.
pub fn inexact_predicate() -> PredicateFn {
    Arc::new(|value| {
        matches!(
            value,
            Value::Literal(Literal::InexactReal(_)) | Value::Literal(Literal::Number(_))
        )
    })
}

/// Tests for complex numbers.
pub fn complex_predicate() -> PredicateFn {
    Arc::new(|value| {
        matches!(
            value,
            Value::Literal(Literal::Complex(_))
                | Value::Literal(Literal::Number(_))
                | Value::Literal(Literal::Integer(_))
                | Value::Literal(Literal::Rational(_))
        )
    })
}

/// Tests for strings.
pub fn string_predicate() -> PredicateFn {
    Arc::new(|value| {
        matches!(
            value,
            Value::Literal(Literal::String(_)) | Value::MutableString(_)
        )
    })
}

/// Tests for characters.
pub fn char_predicate() -> PredicateFn {
    Arc::new(|value| matches!(value, Value::Literal(Literal::Character(_))))
}

/// Tests for booleans.
pub fn boolean_predicate() -> PredicateFn {
    Arc::new(|value| matches!(value, Value::Literal(Literal::Boolean(_))))
}

/// Tests for symbols.
pub fn symbol_predicate() -> PredicateFn {
    Arc::new(|value| matches!(value, Value::Symbol(_)))
}

/// Tests for keywords.
pub fn keyword_predicate() -> PredicateFn {
    Arc::new(|value| matches!(value, Value::Keyword(_)))
}

/// Tests for proper lists.
pub fn list_predicate() -> PredicateFn {
    Arc::new(|value| {
        fn is_proper_list(val: &Value) -> bool {
            match val {
                Value::Nil => true,
                Value::Pair(car, cdr) => is_proper_list(cdr),
                _ => false,
            }
        }
        is_proper_list(value)
    })
}

/// Tests for pairs.
pub fn pair_predicate() -> PredicateFn {
    Arc::new(|value| matches!(value, Value::Pair(_, _) | Value::MutablePair(_, _)))
}

/// Tests for empty list.
pub fn null_predicate() -> PredicateFn {
    Arc::new(|value| matches!(value, Value::Nil))
}

/// Tests for vectors.
pub fn vector_predicate() -> PredicateFn {
    Arc::new(|value| matches!(value, Value::Vector(_)))
}

/// Tests for hash tables.
pub fn hash_predicate() -> PredicateFn {
    Arc::new(|value| matches!(value, Value::Hashtable(_) | Value::AdvancedHashTable(_)))
}

/// Tests for procedures.
pub fn procedure_predicate() -> PredicateFn {
    Arc::new(|value| {
        matches!(
            value,
            Value::Procedure(_)
                | Value::CaseLambda(_)
                | Value::Primitive(_)
                | Value::Continuation(_)
        )
    })
}

/// Tests for primitive procedures.
pub fn primitive_predicate() -> PredicateFn {
    Arc::new(|value| matches!(value, Value::Primitive(_)))
}

/// Tests for I/O ports.
pub fn port_predicate() -> PredicateFn {
    Arc::new(|value| matches!(value, Value::Port(_)))
}

/// Tests for promises.
pub fn promise_predicate() -> PredicateFn {
    Arc::new(|value| matches!(value, Value::Promise(_)))
}

/// Tests for syntax objects.
pub fn syntax_predicate() -> PredicateFn {
    Arc::new(|value| matches!(value, Value::Syntax(_)))
}

/// Tests for positive numbers.
pub fn positive_predicate() -> PredicateFn {
    Arc::new(|value| match value {
        Value::Literal(Literal::InexactReal(n)) | Value::Literal(Literal::Number(n)) => *n > 0.0,
        Value::Literal(Literal::ExactInteger(n)) | Value::Literal(Literal::Integer(n)) => *n > 0,
        Value::Literal(Literal::Rational(r)) => {
            (r.numerator > 0 && r.denominator > 0) || (r.numerator < 0 && r.denominator < 0)
        }
        _ => false,
    })
}

/// Tests for negative numbers.
pub fn negative_predicate() -> PredicateFn {
    Arc::new(|value| match value {
        Value::Literal(Literal::InexactReal(n)) | Value::Literal(Literal::Number(n)) => *n < 0.0,
        Value::Literal(Literal::ExactInteger(n)) | Value::Literal(Literal::Integer(n)) => *n < 0,
        Value::Literal(Literal::Rational(r)) => {
            (r.numerator < 0 && r.denominator > 0) || (r.numerator > 0 && r.denominator < 0)
        }
        _ => false,
    })
}

/// Tests for zero.
pub fn zero_predicate() -> PredicateFn {
    Arc::new(|value| match value {
        Value::Literal(Literal::InexactReal(n)) | Value::Literal(Literal::Number(n)) => *n == 0.0,
        Value::Literal(Literal::ExactInteger(n)) | Value::Literal(Literal::Integer(n)) => *n == 0,
        Value::Literal(Literal::Rational(r)) => r.numerator == 0,
        _ => false,
    })
}

/// Tests for even integers.
pub fn even_predicate() -> PredicateFn {
    Arc::new(|value| match value {
        Value::Literal(Literal::ExactInteger(n)) | Value::Literal(Literal::Integer(n)) => {
            *n % 2 == 0
        }
        Value::Literal(Literal::InexactReal(n)) | Value::Literal(Literal::Number(n)) => {
            let int_part = *n as i64;
            *n == int_part as f64 && int_part % 2 == 0
        }
        _ => false,
    })
}

/// Tests for odd integers.
pub fn odd_predicate() -> PredicateFn {
    Arc::new(|value| match value {
        Value::Literal(Literal::ExactInteger(n)) | Value::Literal(Literal::Integer(n)) => {
            *n % 2 != 0
        }
        Value::Literal(Literal::InexactReal(n)) | Value::Literal(Literal::Number(n)) => {
            let int_part = *n as i64;
            *n == int_part as f64 && int_part % 2 != 0
        }
        _ => false,
    })
}

/// Tests for non-empty strings.
pub fn non_empty_string_predicate() -> PredicateFn {
    Arc::new(|value| match value {
        Value::Literal(Literal::String(s)) => !s.is_empty(),
        Value::MutableString(s) => {
            if let Ok(chars) = s.try_borrow() {
                !chars.is_empty()
            } else {
                false
            }
        }
        _ => false,
    })
}

/// Tests for empty collections.
pub fn empty_predicate() -> PredicateFn {
    Arc::new(|value| match value {
        Value::Nil => true,
        Value::Literal(Literal::String(s)) => s.is_empty(),
        Value::Vector(v) => {
            if let Ok(vec) = v.try_borrow() {
                vec.is_empty()
            } else {
                false
            }
        }
        Value::Hashtable(h) => {
            if let Ok(hash) = h.try_borrow() {
                hash.is_empty()
            } else {
                false
            }
        }
        Value::MutableString(s) => {
            if let Ok(chars) = s.try_borrow() {
                chars.is_empty()
            } else {
                false
            }
        }
        _ => false,
    })
}

/// Tests for non-empty collections.
pub fn non_empty_predicate() -> PredicateFn {
    Arc::new(|value| {
        !empty_predicate()(value)
            && (list_predicate()(value)
                || string_predicate()(value)
                || vector_predicate()(value)
                || hash_predicate()(value))
    })
}

impl Default for PredicateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_predicate_registry() {
        let registry = PredicateRegistry::new();

        // Test built-in predicates
        assert!(registry.lookup("number?").is_some());
        assert!(registry.lookup("string?").is_some());
        assert!(registry.lookup("boolean?").is_some());

        // Test non-existent predicate
        assert!(registry.lookup("nonexistent?").is_none());
    }

    #[test]
    fn test_number_predicates() {
        let number_pred = number_predicate();
        let integer_pred = integer_predicate();
        let positive_pred = positive_predicate();

        let num_val = Value::Literal(Literal::InexactReal(42.5));
        let int_val = Value::Literal(Literal::ExactInteger(42));
        let string_val = Value::Literal(Literal::String(Box::new("hello".to_string())));

        assert!(number_pred(&num_val));
        assert!(number_pred(&int_val));
        assert!(!number_pred(&string_val));

        assert!(!integer_pred(&num_val));
        assert!(integer_pred(&int_val));
        assert!(!integer_pred(&string_val));

        assert!(positive_pred(&num_val));
        assert!(positive_pred(&int_val));
        assert!(!positive_pred(&string_val));
    }

    #[test]
    fn test_string_predicates() {
        let string_pred = string_predicate();
        let non_empty_pred = non_empty_string_predicate();

        let string_val = Value::Literal(Literal::String(Box::new("hello".to_string())));
        let empty_string_val = Value::Literal(Literal::String(Box::new("".to_string())));
        let number_val = Value::Literal(Literal::InexactReal(42.0));

        assert!(string_pred(&string_val));
        assert!(string_pred(&empty_string_val));
        assert!(!string_pred(&number_val));

        assert!(non_empty_pred(&string_val));
        assert!(!non_empty_pred(&empty_string_val));
        assert!(!non_empty_pred(&number_val));
    }

    #[test]
    fn test_boolean_predicates() {
        let boolean_pred = boolean_predicate();

        let true_val = Value::Literal(Literal::Boolean(true));
        let false_val = Value::Literal(Literal::Boolean(false));
        let string_val = Value::Literal(Literal::String(Box::new("hello".to_string())));

        assert!(boolean_pred(&true_val));
        assert!(boolean_pred(&false_val));
        assert!(!boolean_pred(&string_val));
    }

    #[test]
    fn test_collection_predicates() {
        let null_pred = null_predicate();
        let empty_pred = empty_predicate();

        let nil_val = Value::Nil;
        let empty_string = Value::Literal(Literal::String(Box::new("".to_string())));
        let non_empty_string = Value::Literal(Literal::String(Box::new("hello".to_string())));

        assert!(null_pred(&nil_val));
        assert!(!null_pred(&empty_string));

        assert!(empty_pred(&nil_val));
        assert!(empty_pred(&empty_string));
        assert!(!empty_pred(&non_empty_string));
    }

    #[test]
    fn test_predicate_info() {
        let registry = PredicateRegistry::new();

        let info = registry.predicate_info("number?").unwrap();
        assert_eq!(info.name, "number?");
        assert_eq!(info.description, "tests for numbers");
        assert!(info.is_builtin);
        assert!(info.deterministic);
        assert_eq!(info.complexity, PredicateComplexity::Constant);
    }

    #[test]
    fn test_custom_predicate_registration() {
        let mut registry = PredicateRegistry::new();

        let custom_pred = ContractPredicate::new(
            "custom?".to_string(),
            "custom test predicate".to_string(),
            Arc::new(|_| true),
        );

        registry.register("custom?".to_string(), custom_pred);

        assert!(registry.lookup("custom?").is_some());
        let info = registry.predicate_info("custom?").unwrap();
        assert_eq!(info.name, "custom?");
        assert!(!info.is_builtin);
    }
}
