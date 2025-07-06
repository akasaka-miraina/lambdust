//! Unit tests for Copy-on-Write Environment system (Phase 4 optimization)

use lambdust::environment::{SharedEnvironment, EnvironmentFactory};
use lambdust::value::Value;
use lambdust::lexer::SchemeNumber;
use std::rc::Rc;

#[test]
fn test_shared_environment_creation() {
    let env = SharedEnvironment::new();
    assert_eq!(env.depth(), 0);
    assert!(env.is_empty());
    assert!(!env.is_frozen());
}

#[test]
fn test_shared_environment_define_and_get() {
    let mut env = SharedEnvironment::new();
    let value = Value::Number(SchemeNumber::Integer(42));
    
    env.define("x".to_string(), value.clone());
    assert_eq!(env.get("x"), Some(value));
    assert_eq!(env.get("nonexistent"), None);
}

#[test]
fn test_shared_environment_set_existing() {
    let mut env = SharedEnvironment::new();
    let initial_value = Value::Number(SchemeNumber::Integer(10));
    let new_value = Value::Number(SchemeNumber::Integer(20));
    
    env.define("x".to_string(), initial_value);
    assert!(env.set("x", new_value.clone()).is_ok());
    assert_eq!(env.get("x"), Some(new_value));
}

#[test]
fn test_shared_environment_set_nonexistent() {
    let mut env = SharedEnvironment::new();
    let value = Value::Number(SchemeNumber::Integer(42));
    
    assert!(env.set("nonexistent", value).is_err());
}

#[test]
fn test_shared_environment_with_parent() {
    let mut parent = SharedEnvironment::new();
    parent.define("parent_var".to_string(), Value::Number(SchemeNumber::Integer(100)));
    
    let child = SharedEnvironment::with_parent(Rc::new(parent));
    assert_eq!(child.depth(), 1);
    assert_eq!(child.get("parent_var"), Some(Value::Number(SchemeNumber::Integer(100))));
}

#[test]
fn test_shared_environment_parent_chain_lookup() {
    let mut grandparent = SharedEnvironment::new();
    grandparent.define("gp_var".to_string(), Value::Number(SchemeNumber::Integer(1)));
    
    let mut parent = SharedEnvironment::with_parent(Rc::new(grandparent));
    parent.define("p_var".to_string(), Value::Number(SchemeNumber::Integer(2)));
    
    let mut child = SharedEnvironment::with_parent(Rc::new(parent));
    child.define("c_var".to_string(), Value::Number(SchemeNumber::Integer(3)));
    
    assert_eq!(child.depth(), 2);
    assert_eq!(child.get("gp_var"), Some(Value::Number(SchemeNumber::Integer(1))));
    assert_eq!(child.get("p_var"), Some(Value::Number(SchemeNumber::Integer(2))));
    assert_eq!(child.get("c_var"), Some(Value::Number(SchemeNumber::Integer(3))));
}

#[test]
fn test_shared_environment_shadowing() {
    let mut parent = SharedEnvironment::new();
    parent.define("x".to_string(), Value::Number(SchemeNumber::Integer(10)));
    
    let mut child = SharedEnvironment::with_parent(Rc::new(parent));
    child.define("x".to_string(), Value::Number(SchemeNumber::Integer(20)));
    
    // Child should shadow parent's binding
    assert_eq!(child.get("x"), Some(Value::Number(SchemeNumber::Integer(20))));
}

#[test]
fn test_shared_environment_copy_on_write_set() {
    let mut parent = SharedEnvironment::new();
    parent.define("x".to_string(), Value::Number(SchemeNumber::Integer(10)));
    
    let mut child = SharedEnvironment::with_parent(Rc::new(parent));
    
    // Setting variable that exists in parent should copy-on-write
    assert!(child.set("x", Value::Number(SchemeNumber::Integer(20))).is_ok());
    assert_eq!(child.get("x"), Some(Value::Number(SchemeNumber::Integer(20))));
}

#[test]
fn test_shared_environment_extend_cow_empty() {
    let env = SharedEnvironment::new();
    let extended = env.extend_cow(vec![]);
    
    // Empty extension should return shared reference (same structure)
    assert_eq!(extended.depth(), env.depth());
    assert!(extended.is_empty());
}

#[test]
fn test_shared_environment_extend_cow_with_bindings() {
    let env = SharedEnvironment::new();
    let bindings = vec![
        ("x".to_string(), Value::Number(SchemeNumber::Integer(1))),
        ("y".to_string(), Value::Number(SchemeNumber::Integer(2))),
    ];
    
    let extended = env.extend_cow(bindings);
    
    assert_eq!(extended.get("x"), Some(Value::Number(SchemeNumber::Integer(1))));
    assert_eq!(extended.get("y"), Some(Value::Number(SchemeNumber::Integer(2))));
    assert!(!extended.is_empty());
}

#[test]
fn test_shared_environment_freeze() {
    let mut env = SharedEnvironment::new();
    env.define("x".to_string(), Value::Number(SchemeNumber::Integer(42)));
    
    env.freeze();
    assert!(env.is_frozen());
    
    // Should not be able to set frozen environment
    assert!(env.set("x", Value::Number(SchemeNumber::Integer(100))).is_err());
}

#[test]
fn test_shared_environment_cache_building() {
    let mut parent = SharedEnvironment::new();
    parent.define("p1".to_string(), Value::Number(SchemeNumber::Integer(1)));
    parent.define("p2".to_string(), Value::Number(SchemeNumber::Integer(2)));
    
    let mut child = SharedEnvironment::with_parent(Rc::new(parent));
    child.define("c1".to_string(), Value::Number(SchemeNumber::Integer(3)));
    
    // Build cache
    child.build_cache();
    
    // Cache should allow fast lookups
    assert_eq!(child.get("p1"), Some(Value::Number(SchemeNumber::Integer(1))));
    assert_eq!(child.get("p2"), Some(Value::Number(SchemeNumber::Integer(2))));
    assert_eq!(child.get("c1"), Some(Value::Number(SchemeNumber::Integer(3))));
}

#[test]
fn test_shared_environment_exists() {
    let mut parent = SharedEnvironment::new();
    parent.define("parent_var".to_string(), Value::Number(SchemeNumber::Integer(1)));
    
    let mut child = SharedEnvironment::with_parent(Rc::new(parent));
    child.define("child_var".to_string(), Value::Number(SchemeNumber::Integer(2)));
    
    assert!(child.exists("parent_var"));
    assert!(child.exists("child_var"));
    assert!(!child.exists("nonexistent"));
}

#[test]
fn test_shared_environment_total_bindings() {
    let mut parent = SharedEnvironment::new();
    parent.define("p1".to_string(), Value::Number(SchemeNumber::Integer(1)));
    parent.define("p2".to_string(), Value::Number(SchemeNumber::Integer(2)));
    
    let mut child = SharedEnvironment::with_parent(Rc::new(parent));
    child.define("c1".to_string(), Value::Number(SchemeNumber::Integer(3)));
    
    assert_eq!(child.total_bindings(), 3);
}

#[test]
fn test_shared_environment_memory_usage() {
    let mut env = SharedEnvironment::new();
    let initial_usage = env.memory_usage();
    
    env.define("x".to_string(), Value::Number(SchemeNumber::Integer(42)));
    let after_define_usage = env.memory_usage();
    
    assert!(after_define_usage > initial_usage);
    
    env.build_cache();
    let after_cache_usage = env.memory_usage();
    
    assert!(after_cache_usage > after_define_usage);
}

#[test]
fn test_shared_environment_iter_all_bindings() {
    let mut parent = SharedEnvironment::new();
    parent.define("parent_var".to_string(), Value::Number(SchemeNumber::Integer(1)));
    
    let mut child = SharedEnvironment::with_parent(Rc::new(parent));
    child.define("child_var".to_string(), Value::Number(SchemeNumber::Integer(2)));
    child.define("parent_var".to_string(), Value::Number(SchemeNumber::Integer(10))); // Shadow parent
    
    let all_bindings = child.iter_all_bindings();
    
    assert_eq!(all_bindings.len(), 2);
    assert_eq!(all_bindings.get("child_var"), Some(&Value::Number(SchemeNumber::Integer(2))));
    assert_eq!(all_bindings.get("parent_var"), Some(&Value::Number(SchemeNumber::Integer(10)))); // Shadowed value
}

#[test]
fn test_environment_factory() {
    let traditional = EnvironmentFactory::new_traditional();
    let shared = EnvironmentFactory::new_shared();
    
    assert_eq!(traditional.depth(), 0);
    assert_eq!(shared.depth(), 0);
}

#[test]
fn test_environment_ops_trait() {
    let mut env = SharedEnvironment::new();
    
    // Test through trait
    env.define("x".to_string(), Value::Number(SchemeNumber::Integer(42)));
    assert_eq!(env.get("x"), Some(Value::Number(SchemeNumber::Integer(42))));
    assert!(env.exists("x"));
    assert_eq!(env.depth(), 0);
    
    assert!(env.set("x", Value::Number(SchemeNumber::Integer(100))).is_ok());
    assert_eq!(env.get("x"), Some(Value::Number(SchemeNumber::Integer(100))));
}

#[test]
fn test_shared_environment_performance_characteristics() {
    let mut env = SharedEnvironment::new();
    
    // Add many bindings
    for i in 0..100 {
        env.define(format!("var{}", i), Value::Number(SchemeNumber::Integer(i)));
    }
    
    let memory_before_cache = env.memory_usage();
    
    // Build cache
    env.build_cache();
    
    let memory_after_cache = env.memory_usage();
    
    // Cache should increase memory usage but provide faster lookups
    assert!(memory_after_cache > memory_before_cache);
    
    // All variables should still be accessible
    for i in 0..100 {
        assert_eq!(env.get(&format!("var{}", i)), Some(Value::Number(SchemeNumber::Integer(i))));
    }
}

#[test]
fn test_shared_environment_cow_optimization() {
    let parent = SharedEnvironment::new();
    
    // Empty extension should be very cheap (COW)
    let child1 = parent.extend_cow(vec![]);
    let child2 = parent.extend_cow(vec![]);
    
    // Both children should have same structure as parent
    assert_eq!(child1.depth(), parent.depth());
    assert_eq!(child2.depth(), parent.depth());
    assert!(child1.is_empty());
    assert!(child2.is_empty());
    
    // Extension with bindings should create new environment
    let child3 = parent.extend_cow(vec![
        ("x".to_string(), Value::Number(SchemeNumber::Integer(1)))
    ]);
    
    assert!(!child3.is_empty());
    assert_eq!(child3.get("x"), Some(Value::Number(SchemeNumber::Integer(1))));
}