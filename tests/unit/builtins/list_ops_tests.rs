//! Unit tests for list operation builtin functions
//!
//! Tests all list operations for correctness, edge cases, and error handling.

use lambdust::builtins::list_ops::register_list_functions;
use lambdust::error::LambdustError;
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Procedure, Value};
use std::collections::HashMap;

/// Helper function to get a builtin function by name
fn get_builtin(name: &str) -> Value {
    let mut builtins = HashMap::new();
    register_list_functions(&mut builtins);
    builtins.get(name).unwrap().clone()
}

/// Helper function to call a builtin function
fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value, LambdustError> {
    let func = get_builtin(name);
    if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
        func(&args)
    } else {
        panic!("Expected builtin function for {}", name);
    }
}

/// Helper to create integer value
fn int(n: i64) -> Value {
    Value::Number(SchemeNumber::Integer(n))
}

/// Helper to create string value
fn string(s: &str) -> Value {
    Value::String(s.to_string())
}

/// Helper to create symbol value
fn symbol(s: &str) -> Value {
    Value::Symbol(s.to_string())
}

/// Helper to create list value from vector
fn list(items: Vec<Value>) -> Value {
    Value::from_vector(items)
}

/// Helper to create nil value
fn nil() -> Value {
    Value::Nil
}

/// Helper to create pair value
fn pair(car: Value, cdr: Value) -> Value {
    Value::cons(car, cdr)
}

#[cfg(test)]
mod basic_list_operations_tests {
    use super::*;

    #[test]
    fn test_car() {
        // Basic car operation
        let test_pair = pair(int(1), int(2));
        assert_eq!(call_builtin("car", vec![test_pair]).unwrap(), int(1));

        // Car of list
        let test_list = list(vec![int(10), int(20), int(30)]);
        assert_eq!(call_builtin("car", vec![test_list]).unwrap(), int(10));

        // Car of single-element list
        let single_list = list(vec![string("hello")]);
        assert_eq!(
            call_builtin("car", vec![single_list]).unwrap(),
            string("hello")
        );

        // Car of nested list
        let nested = list(vec![list(vec![int(1), int(2)]), int(3)]);
        assert_eq!(
            call_builtin("car", vec![nested]).unwrap(),
            list(vec![int(1), int(2)])
        );

        // Type errors
        assert!(call_builtin("car", vec![nil()]).is_err());
        assert!(call_builtin("car", vec![int(42)]).is_err());
        assert!(call_builtin("car", vec![string("hello")]).is_err());

        // Arity errors
        assert!(call_builtin("car", vec![]).is_err());
        assert!(call_builtin("car", vec![pair(int(1), int(2)), pair(int(3), int(4))]).is_err());
    }

    #[test]
    fn test_cdr() {
        // Basic cdr operation
        let test_pair = pair(int(1), int(2));
        assert_eq!(call_builtin("cdr", vec![test_pair]).unwrap(), int(2));

        // Cdr of list
        let test_list = list(vec![int(10), int(20), int(30)]);
        assert_eq!(
            call_builtin("cdr", vec![test_list]).unwrap(),
            list(vec![int(20), int(30)])
        );

        // Cdr of single-element list
        let single_list = list(vec![string("hello")]);
        assert_eq!(call_builtin("cdr", vec![single_list]).unwrap(), nil());

        // Cdr of two-element list
        let two_list = list(vec![int(1), int(2)]);
        assert_eq!(
            call_builtin("cdr", vec![two_list]).unwrap(),
            list(vec![int(2)])
        );

        // Cdr of nested list
        let nested = list(vec![int(1), list(vec![int(2), int(3)])]);
        assert_eq!(
            call_builtin("cdr", vec![nested]).unwrap(),
            list(vec![list(vec![int(2), int(3)])])
        );

        // Type errors
        assert!(call_builtin("cdr", vec![nil()]).is_err());
        assert!(call_builtin("cdr", vec![int(42)]).is_err());
        assert!(call_builtin("cdr", vec![string("hello")]).is_err());

        // Arity errors
        assert!(call_builtin("cdr", vec![]).is_err());
        assert!(call_builtin("cdr", vec![pair(int(1), int(2)), pair(int(3), int(4))]).is_err());
    }

    #[test]
    fn test_cons() {
        // Basic cons operation
        assert_eq!(
            call_builtin("cons", vec![int(1), int(2)]).unwrap(),
            pair(int(1), int(2))
        );

        // Cons with nil to create single-element list
        assert_eq!(
            call_builtin("cons", vec![int(42), nil()]).unwrap(),
            list(vec![int(42)])
        );

        // Cons to extend a list
        let existing_list = list(vec![int(2), int(3)]);
        assert_eq!(
            call_builtin("cons", vec![int(1), existing_list]).unwrap(),
            list(vec![int(1), int(2), int(3)])
        );

        // Cons with different types
        assert_eq!(
            call_builtin("cons", vec![string("hello"), int(42)]).unwrap(),
            pair(string("hello"), int(42))
        );

        // Cons with nested structures
        let inner_list = list(vec![int(10), int(20)]);
        assert_eq!(
            call_builtin("cons", vec![inner_list.clone(), int(30)]).unwrap(),
            pair(inner_list, int(30))
        );

        // Arity errors
        assert!(call_builtin("cons", vec![int(1)]).is_err());
        assert!(call_builtin("cons", vec![]).is_err());
        assert!(call_builtin("cons", vec![int(1), int(2), int(3)]).is_err());
    }

    #[test]
    fn test_list() {
        // Empty list
        assert_eq!(call_builtin("list", vec![]).unwrap(), nil());

        // Single element list
        assert_eq!(
            call_builtin("list", vec![int(42)]).unwrap(),
            list(vec![int(42)])
        );

        // Multi-element list
        assert_eq!(
            call_builtin("list", vec![int(1), int(2), int(3)]).unwrap(),
            list(vec![int(1), int(2), int(3)])
        );

        // Mixed types
        assert_eq!(
            call_builtin("list", vec![int(1), string("hello"), symbol("world")]).unwrap(),
            list(vec![int(1), string("hello"), symbol("world")])
        );

        // Nested lists
        let inner = list(vec![int(1), int(2)]);
        assert_eq!(
            call_builtin("list", vec![inner.clone(), int(3)]).unwrap(),
            list(vec![inner, int(3)])
        );

        // Large list
        let large_list = call_builtin("list", (0..100).map(int).collect()).unwrap();
        assert_eq!(large_list.list_length().unwrap(), 100);
    }

    #[test]
    fn test_length() {
        // Empty list
        assert_eq!(call_builtin("length", vec![nil()]).unwrap(), int(0));

        // Single element
        assert_eq!(
            call_builtin("length", vec![list(vec![int(42)])]).unwrap(),
            int(1)
        );

        // Multi-element list
        assert_eq!(
            call_builtin("length", vec![list(vec![int(1), int(2), int(3)])]).unwrap(),
            int(3)
        );

        // Large list
        let large_list = list((0..1000).map(int).collect());
        assert_eq!(call_builtin("length", vec![large_list]).unwrap(), int(1000));

        // Nested lists (length is still outer length)
        let nested = list(vec![
            list(vec![int(1), int(2)]),
            list(vec![int(3), int(4), int(5)]),
        ]);
        assert_eq!(call_builtin("length", vec![nested]).unwrap(), int(2));

        // Type errors (non-lists)
        assert!(call_builtin("length", vec![int(42)]).is_err());
        assert!(call_builtin("length", vec![string("hello")]).is_err());
        assert!(call_builtin("length", vec![pair(int(1), int(2))]).is_err()); // Improper list

        // Arity errors
        assert!(call_builtin("length", vec![]).is_err());
        assert!(call_builtin("length", vec![nil(), nil()]).is_err());
    }

    #[test]
    fn test_append() {
        // Empty append
        assert_eq!(call_builtin("append", vec![]).unwrap(), nil());

        // Single list
        let single = list(vec![int(1), int(2), int(3)]);
        assert_eq!(
            call_builtin("append", vec![single.clone()]).unwrap(),
            single
        );

        // Two lists
        let list1 = list(vec![int(1), int(2)]);
        let list2 = list(vec![int(3), int(4)]);
        assert_eq!(
            call_builtin("append", vec![list1, list2]).unwrap(),
            list(vec![int(1), int(2), int(3), int(4)])
        );

        // Multiple lists
        let list1 = list(vec![int(1)]);
        let list2 = list(vec![int(2), int(3)]);
        let list3 = list(vec![int(4), int(5), int(6)]);
        assert_eq!(
            call_builtin("append", vec![list1, list2, list3]).unwrap(),
            list(vec![int(1), int(2), int(3), int(4), int(5), int(6)])
        );

        // Append with empty lists
        let empty = nil();
        let non_empty = list(vec![int(1), int(2)]);
        assert_eq!(
            call_builtin("append", vec![empty.clone(), non_empty.clone()]).unwrap(),
            non_empty
        );
        assert_eq!(
            call_builtin("append", vec![non_empty.clone(), empty]).unwrap(),
            non_empty
        );

        // Append with nil as last argument
        let list1 = list(vec![int(1), int(2)]);
        assert_eq!(
            call_builtin("append", vec![list1.clone(), nil()]).unwrap(),
            list1
        );

        // Append with non-list as last argument (creates dotted list)
        let list1 = list(vec![int(1), int(2)]);
        let result = call_builtin("append", vec![list1, int(42)]).unwrap();
        // This should create (1 2 . 42)
        assert_eq!(call_builtin("car", vec![result.clone()]).unwrap(), int(1));
        assert_eq!(
            call_builtin(
                "car",
                vec![call_builtin("cdr", vec![result.clone()]).unwrap()]
            )
            .unwrap(),
            int(2)
        );
        assert_eq!(
            call_builtin("cdr", vec![call_builtin("cdr", vec![result]).unwrap()]).unwrap(),
            int(42)
        );

        // Mixed types
        let list1 = list(vec![string("hello")]);
        let list2 = list(vec![symbol("world")]);
        assert_eq!(
            call_builtin("append", vec![list1, list2]).unwrap(),
            list(vec![string("hello"), symbol("world")])
        );

        // Type errors (non-lists in non-final positions)
        assert!(call_builtin("append", vec![int(42), list(vec![int(1)])]).is_err());
        assert!(
            call_builtin(
                "append",
                vec![list(vec![int(1)]), int(42), list(vec![int(3)])]
            )
            .is_err()
        );
    }

    #[test]
    fn test_reverse() {
        // Empty list
        assert_eq!(call_builtin("reverse", vec![nil()]).unwrap(), nil());

        // Single element
        let single = list(vec![int(42)]);
        assert_eq!(
            call_builtin("reverse", vec![single.clone()]).unwrap(),
            single
        );

        // Two elements
        let two = list(vec![int(1), int(2)]);
        assert_eq!(
            call_builtin("reverse", vec![two]).unwrap(),
            list(vec![int(2), int(1)])
        );

        // Multiple elements
        let multi = list(vec![int(1), int(2), int(3), int(4), int(5)]);
        assert_eq!(
            call_builtin("reverse", vec![multi]).unwrap(),
            list(vec![int(5), int(4), int(3), int(2), int(1)])
        );

        // Mixed types
        let mixed = list(vec![string("a"), int(2), symbol("c")]);
        assert_eq!(
            call_builtin("reverse", vec![mixed]).unwrap(),
            list(vec![symbol("c"), int(2), string("a")])
        );

        // Nested lists
        let nested = list(vec![list(vec![int(1), int(2)]), list(vec![int(3), int(4)])]);
        assert_eq!(
            call_builtin("reverse", vec![nested]).unwrap(),
            list(vec![list(vec![int(3), int(4)]), list(vec![int(1), int(2)])])
        );

        // Type errors
        assert!(call_builtin("reverse", vec![int(42)]).is_err());
        assert!(call_builtin("reverse", vec![string("hello")]).is_err());
        assert!(call_builtin("reverse", vec![pair(int(1), int(2))]).is_err()); // Improper list

        // Arity errors
        assert!(call_builtin("reverse", vec![]).is_err());
        assert!(call_builtin("reverse", vec![nil(), nil()]).is_err());
    }
}

#[cfg(test)]
mod destructive_operations_tests {
    use super::*;

    #[test]
    fn test_set_car() {
        // Basic set-car! operation
        let test_pair = pair(int(1), int(2));
        let result = call_builtin("set-car!", vec![test_pair.clone(), int(10)]).unwrap();
        assert_eq!(result, test_pair); // Returns the modified pair

        // Verify the modification
        assert_eq!(call_builtin("car", vec![test_pair]).unwrap(), int(10));

        // Set-car! on list
        let test_list = list(vec![int(1), int(2), int(3)]);
        let result = call_builtin("set-car!", vec![test_list.clone(), string("new")]).unwrap();
        assert_eq!(result, test_list); // Returns the modified list

        // Verify the modification
        assert_eq!(call_builtin("car", vec![test_list]).unwrap(), string("new"));

        // Set-car! with different types
        let test_pair = pair(string("old"), int(42));
        call_builtin("set-car!", vec![test_pair.clone(), symbol("new")]).unwrap();
        assert_eq!(call_builtin("car", vec![test_pair]).unwrap(), symbol("new"));

        // Type errors
        assert!(call_builtin("set-car!", vec![nil(), int(1)]).is_err());
        assert!(call_builtin("set-car!", vec![int(42), int(1)]).is_err());
        assert!(call_builtin("set-car!", vec![string("hello"), int(1)]).is_err());

        // Arity errors
        assert!(call_builtin("set-car!", vec![pair(int(1), int(2))]).is_err());
        assert!(call_builtin("set-car!", vec![]).is_err());
        assert!(call_builtin("set-car!", vec![pair(int(1), int(2)), int(3), int(4)]).is_err());
    }

    #[test]
    fn test_set_cdr() {
        // Basic set-cdr! operation
        let test_pair = pair(int(1), int(2));
        let result = call_builtin("set-cdr!", vec![test_pair.clone(), int(20)]).unwrap();
        assert_eq!(result, test_pair); // Returns the modified pair

        // Verify the modification
        assert_eq!(call_builtin("cdr", vec![test_pair]).unwrap(), int(20));

        // Set-cdr! on list to truncate
        let test_list = list(vec![int(1), int(2), int(3)]);
        let result = call_builtin("set-cdr!", vec![test_list.clone(), nil()]).unwrap();
        assert_eq!(result, test_list); // Returns the modified list

        // Verify the modification (should now be single-element list)
        assert_eq!(call_builtin("cdr", vec![test_list.clone()]).unwrap(), nil());
        assert_eq!(call_builtin("length", vec![test_list]).unwrap(), int(1));

        // Set-cdr! to extend list
        let test_pair = pair(int(1), nil());
        let extension = list(vec![int(2), int(3)]);
        call_builtin("set-cdr!", vec![test_pair.clone(), extension]).unwrap();
        assert_eq!(
            call_builtin("cdr", vec![test_pair]).unwrap(),
            list(vec![int(2), int(3)])
        );

        // Set-cdr! with different types
        let test_pair = pair(int(1), string("old"));
        call_builtin("set-cdr!", vec![test_pair.clone(), symbol("new")]).unwrap();
        assert_eq!(call_builtin("cdr", vec![test_pair]).unwrap(), symbol("new"));

        // Type errors
        assert!(call_builtin("set-cdr!", vec![nil(), int(1)]).is_err());
        assert!(call_builtin("set-cdr!", vec![int(42), int(1)]).is_err());
        assert!(call_builtin("set-cdr!", vec![string("hello"), int(1)]).is_err());

        // Arity errors
        assert!(call_builtin("set-cdr!", vec![pair(int(1), int(2))]).is_err());
        assert!(call_builtin("set-cdr!", vec![]).is_err());
        assert!(call_builtin("set-cdr!", vec![pair(int(1), int(2)), int(3), int(4)]).is_err());
    }
}

#[cfg(test)]
mod predicate_tests {
    use super::*;

    #[test]
    fn test_null_predicate() {
        // True cases
        assert_eq!(
            call_builtin("null?", vec![nil()]).unwrap(),
            Value::Boolean(true)
        );

        // False cases
        assert_eq!(
            call_builtin("null?", vec![list(vec![int(1)])]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("null?", vec![pair(int(1), int(2))]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("null?", vec![int(42)]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("null?", vec![string("hello")]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("null?", vec![symbol("test")]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("null?", vec![Value::Boolean(true)]).unwrap(),
            Value::Boolean(false)
        );

        // Arity errors
        assert!(call_builtin("null?", vec![]).is_err());
        assert!(call_builtin("null?", vec![nil(), nil()]).is_err());
    }

    #[test]
    fn test_pair_predicate() {
        // True cases
        assert_eq!(
            call_builtin("pair?", vec![pair(int(1), int(2))]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            call_builtin("pair?", vec![list(vec![int(1)])]).unwrap(),
            Value::Boolean(true)
        ); // Single-element list
        assert_eq!(
            call_builtin("pair?", vec![list(vec![int(1), int(2)])]).unwrap(),
            Value::Boolean(true)
        ); // Multi-element list

        // Dotted pair
        let dotted = pair(int(1), int(2)); // (1 . 2)
        assert_eq!(
            call_builtin("pair?", vec![dotted]).unwrap(),
            Value::Boolean(true)
        );

        // False cases
        assert_eq!(
            call_builtin("pair?", vec![nil()]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("pair?", vec![int(42)]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("pair?", vec![string("hello")]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("pair?", vec![symbol("test")]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("pair?", vec![Value::Boolean(true)]).unwrap(),
            Value::Boolean(false)
        );

        // Arity errors
        assert!(call_builtin("pair?", vec![]).is_err());
        assert!(call_builtin("pair?", vec![pair(int(1), int(2)), pair(int(3), int(4))]).is_err());
    }

    #[test]
    fn test_list_predicate() {
        // True cases
        assert_eq!(
            call_builtin("list?", vec![nil()]).unwrap(),
            Value::Boolean(true)
        ); // Empty list
        assert_eq!(
            call_builtin("list?", vec![list(vec![int(1)])]).unwrap(),
            Value::Boolean(true)
        ); // Single-element
        assert_eq!(
            call_builtin("list?", vec![list(vec![int(1), int(2), int(3)])]).unwrap(),
            Value::Boolean(true)
        ); // Multi-element

        // Nested lists
        let nested = list(vec![list(vec![int(1)]), int(2)]);
        assert_eq!(
            call_builtin("list?", vec![nested]).unwrap(),
            Value::Boolean(true)
        );

        // False cases (improper lists/dotted pairs)
        assert_eq!(
            call_builtin("list?", vec![pair(int(1), int(2))]).unwrap(),
            Value::Boolean(false)
        ); // Dotted pair

        // Non-list types
        assert_eq!(
            call_builtin("list?", vec![int(42)]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("list?", vec![string("hello")]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("list?", vec![symbol("test")]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("list?", vec![Value::Boolean(true)]).unwrap(),
            Value::Boolean(false)
        );

        // Arity errors
        assert!(call_builtin("list?", vec![]).is_err());
        assert!(call_builtin("list?", vec![nil(), nil()]).is_err());
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_nested_structures() {
        // Deep nesting
        let deep = list(vec![
            list(vec![list(vec![int(1), int(2)]), int(3)]),
            list(vec![int(4), list(vec![int(5), int(6)])]),
        ]);

        // Test car/cdr on nested structures
        let first = call_builtin("car", vec![deep.clone()]).unwrap();
        assert_eq!(first, list(vec![list(vec![int(1), int(2)]), int(3)]));

        // Test car of the first element
        let first_of_first = call_builtin("car", vec![first]).unwrap();
        assert_eq!(first_of_first, list(vec![int(1), int(2)]));

        let rest = call_builtin("cdr", vec![deep]).unwrap();
        assert_eq!(
            rest,
            list(vec![list(vec![int(4), list(vec![int(5), int(6)])])])
        );

        // Test length of nested structure
        let nested = list(vec![
            list(vec![int(1), int(2), int(3)]),
            list(vec![int(4), int(5)]),
        ]);
        assert_eq!(call_builtin("length", vec![nested]).unwrap(), int(2)); // Outer length
    }

    #[test]
    fn test_circular_reference_safety() {
        // Test that we don't get into infinite loops
        let test_pair = pair(int(1), int(2));

        // Modify to create self-reference would be: (set-cdr! test-pair test-pair)
        // But we test that our current implementation handles normal cases correctly
        call_builtin("set-cdr!", vec![test_pair.clone(), test_pair.clone()]).unwrap();

        // This should still work (accessing the modified structure)
        assert_eq!(
            call_builtin("car", vec![test_pair.clone()]).unwrap(),
            int(1)
        );
        // Note: This creates a circular structure, but we test that basic operations still work
    }

    #[test]
    fn test_type_consistency() {
        // Ensure all list operations maintain type consistency
        let mixed_list = list(vec![
            int(1),
            string("hello"),
            symbol("world"),
            Value::Boolean(true),
        ]);

        // All operations should work with mixed types
        assert_eq!(
            call_builtin("length", vec![mixed_list.clone()]).unwrap(),
            int(4)
        );
        assert_eq!(
            call_builtin("car", vec![mixed_list.clone()]).unwrap(),
            int(1)
        );

        let reversed = call_builtin("reverse", vec![mixed_list.clone()]).unwrap();
        assert_eq!(
            call_builtin("car", vec![reversed]).unwrap(),
            Value::Boolean(true)
        );

        // Append with mixed types
        let list1 = list(vec![int(1), string("a")]);
        let list2 = list(vec![symbol("b"), Value::Boolean(false)]);
        let appended = call_builtin("append", vec![list1, list2]).unwrap();
        assert_eq!(call_builtin("length", vec![appended]).unwrap(), int(4));
    }

    #[test]
    fn test_large_lists() {
        // Test with reasonably large lists to ensure performance
        let large_list = list((0..1000).map(int).collect());

        // Basic operations should work
        assert_eq!(
            call_builtin("length", vec![large_list.clone()]).unwrap(),
            int(1000)
        );
        assert_eq!(
            call_builtin("car", vec![large_list.clone()]).unwrap(),
            int(0)
        );

        // Reverse should work
        let reversed = call_builtin("reverse", vec![large_list]).unwrap();
        assert_eq!(call_builtin("car", vec![reversed]).unwrap(), int(999));
    }

    #[test]
    fn test_empty_list_operations() {
        // All operations should handle empty lists appropriately
        let empty = nil();

        // Predicates
        assert_eq!(
            call_builtin("null?", vec![empty.clone()]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            call_builtin("pair?", vec![empty.clone()]).unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            call_builtin("list?", vec![empty.clone()]).unwrap(),
            Value::Boolean(true)
        );

        // Operations that should work
        assert_eq!(call_builtin("length", vec![empty.clone()]).unwrap(), int(0));
        assert_eq!(call_builtin("reverse", vec![empty.clone()]).unwrap(), empty);
        assert_eq!(
            call_builtin("append", vec![empty.clone(), empty.clone()]).unwrap(),
            empty
        );

        // Operations that should fail
        assert!(call_builtin("car", vec![empty.clone()]).is_err());
        assert!(call_builtin("cdr", vec![empty.clone()]).is_err());
        assert!(call_builtin("set-car!", vec![empty.clone(), int(1)]).is_err());
        assert!(call_builtin("set-cdr!", vec![empty, int(1)]).is_err());
    }

    #[test]
    fn test_improper_lists() {
        // Test dotted pair (improper list) behavior
        let dotted = pair(int(1), int(2)); // (1 . 2)

        // Should be a pair but not a proper list
        assert_eq!(
            call_builtin("pair?", vec![dotted.clone()]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            call_builtin("list?", vec![dotted.clone()]).unwrap(),
            Value::Boolean(false)
        );

        // Car and cdr should work
        assert_eq!(call_builtin("car", vec![dotted.clone()]).unwrap(), int(1));
        assert_eq!(call_builtin("cdr", vec![dotted.clone()]).unwrap(), int(2));

        // Length should fail (not a proper list)
        assert!(call_builtin("length", vec![dotted.clone()]).is_err());
        assert!(call_builtin("reverse", vec![dotted]).is_err());

        // Test semi-proper list: (1 2 . 3)
        let semi_proper = pair(int(1), pair(int(2), int(3)));
        assert_eq!(
            call_builtin("pair?", vec![semi_proper.clone()]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            call_builtin("list?", vec![semi_proper.clone()]).unwrap(),
            Value::Boolean(false)
        );
        assert!(call_builtin("length", vec![semi_proper]).is_err());
    }
}
