#![allow(clippy::uninlined_format_args)]
//! CI-Optimized Language Processing Validation
//!
//! Fast, lightweight tests that validate core language semantics 
//! after SafeOptimizedValue migration without resource overhead.
//! Designed for CI environments with limited time/memory constraints.

#![cfg(test)]
#![allow(clippy::uninlined_format_args)]

use lambdust::eval::{SafeOptimizedValue, SafeEnvironment, SafeSymbolTable, LegacyValueBridge, Value};
use lambdust::utils::SymbolId;
use lambdust::ast::Literal;

/// Fast symbol identity validation - core R7RS requirement
#[test]
fn ci_symbol_identity_preservation() {
    let bridge = LegacyValueBridge::new_default();
    let sym_id = SymbolId::new(42);
    let sym1 = Value::Symbol(sym_id);
    let sym2 = Value::Symbol(sym_id);
    
    let opt1 = bridge.optimize_value(&sym1);
    let opt2 = bridge.optimize_value(&sym2);
    
    // Critical: Symbol identity must be preserved
    assert_eq!(opt1.as_symbol(), opt2.as_symbol());
    assert_eq!(opt1.as_symbol(), Some(sym_id));
    
    // Test roundtrip preservation
    let restored1 = bridge.deoptimize_value(&opt1);
    let restored2 = bridge.deoptimize_value(&opt2);
    
    if let (Value::Symbol(id1), Value::Symbol(id2)) = (&restored1, &restored2) {
        assert_eq!(id1, id2);
        assert_eq!(*id1, sym_id);
    } else {
        panic!("Symbol roundtrip failed");
    }
}

/// Fast environment chain safety - lexical scoping validation  
#[test]
fn ci_environment_chain_safety() {
    let global = SafeEnvironment::new_global();
    let outer_var = SymbolId::new(1);
    let inner_var = SymbolId::new(2);
    
    // Define outer variable
    global.define(outer_var, SafeOptimizedValue::fixnum(100)).unwrap();
    
    // Create nested scope (minimal depth for CI)
    let child = SafeEnvironment::new_child(global.clone()).unwrap();
    child.define(inner_var, SafeOptimizedValue::fixnum(200)).unwrap();
    
    // Test lexical scoping works correctly
    let outer_value = child.lookup(outer_var).unwrap();
    let inner_value = child.lookup(inner_var).unwrap();
    
    assert_eq!(outer_value.as_integer(), Some(100));
    assert_eq!(inner_value.as_integer(), Some(200));
    
    // Test that global doesn't see inner variable
    assert!(global.lookup(inner_var).is_err());
}

/// Fast value conversion validation - SafeOptimizedValue correctness
#[test] 
fn ci_value_conversion_core() {
    let bridge = LegacyValueBridge::new_default();
    
    // Test essential value types
    let test_values = vec![
        Value::Nil,
        Value::Literal(Literal::Boolean(true)),
        Value::Literal(Literal::ExactInteger(42)),
        Value::Literal(Literal::Character('λ')),
        Value::Symbol(SymbolId::new(123)),
    ];
    
    for original in test_values {
        let optimized = bridge.optimize_value(&original);
        let restored = bridge.deoptimize_value(&optimized);
        
        // Critical: Values must survive optimization roundtrip
        match (&original, &restored) {
            (Value::Nil, Value::Nil) => {},
            (Value::Literal(l1), Value::Literal(l2)) => assert_eq!(l1, l2),
            (Value::Symbol(s1), Value::Symbol(s2)) => assert_eq!(s1, s2),
            _ => panic!("Value conversion failed: {:?} != {:?}", original, restored),
        }
    }
}

/// Fast memory safety smoke test - SIGSEGV elimination verification
#[test]
fn ci_memory_safety_smoke() {
    // Operations that previously could cause SIGSEGV
    let problematic_operations = vec![
        SafeOptimizedValue::fixnum(0),
        SafeOptimizedValue::character('\0'),
        SafeOptimizedValue::nil(),
        SafeOptimizedValue::boolean(false),
    ];
    
    // Test type checking operations don't crash
    for value in &problematic_operations {
        let _is_nil = value.is_nil();
        let _is_bool = value.is_boolean(); 
        let _is_num = value.is_number();
        // If we reach here without crash, SIGSEGV is eliminated
    }
    
    // Test value access operations
    let num_val = SafeOptimizedValue::fixnum(42);
    assert_eq!(num_val.as_integer(), Some(42));
    
    let bool_val = SafeOptimizedValue::boolean(true);
    assert_eq!(bool_val.as_boolean(), Some(true));
}

/// Fast container operations safety - ordered set basic validation
#[test]
fn ci_container_basic_safety() {
    use lambdust::containers::ordered_set::OrderedSet;
    
    let mut set = OrderedSet::new();
    
    // Test basic operations that previously caused SIGSEGV
    for i in 0..10 {  // Minimal count for CI
        let value = Value::Literal(Literal::ExactInteger(i));
        set.insert(value);
    }
    
    // Test lookups don't crash
    for i in 0..10 {
        let value = Value::Literal(Literal::ExactInteger(i));
        assert!(set.contains(&value));
    }
    
    // Test deletions don't crash (red-black tree critical path)
    for i in (0..10).step_by(2) {
        let value = Value::Literal(Literal::ExactInteger(i));
        assert!(set.remove(&value));
    }
    
    // If we reach here, container operations are memory-safe
}

/// Fast symbol interning safety - concurrent symbol table validation
#[test]
fn ci_symbol_interning_safety() {
    let table = SafeSymbolTable::new();
    
    // Test basic symbol interning (minimal scale for CI)
    let symbols = vec!["test", "symbol", "interning", "safety", "validation"];
    let mut symbol_ids = Vec::new();
    
    // Intern symbols
    for symbol_name in &symbols {
        let id = table.intern(symbol_name);
        symbol_ids.push(id);
        
        // Test retrieval
        let retrieved = table.get_string(id).unwrap();
        assert_eq!(&*retrieved, *symbol_name);
    }
    
    // Test symbol identity - same name should give same ID
    for symbol_name in &symbols {
        let id1 = table.intern(symbol_name);
        let id2 = table.intern(symbol_name);
        assert_eq!(id1, id2);
    }
}

/// Fast environment depth limit validation - stack overflow prevention
#[test] 
fn ci_environment_depth_limit() {
    let mut current_env = SafeEnvironment::new_global();
    let mut depth_reached = 0;
    
    // Create environments up to a reasonable limit for CI
    for _i in 0..50 {  // Much less than production limit
        match SafeEnvironment::new_child(current_env) {
            Ok(new_env) => {
                current_env = new_env;
                depth_reached += 1;
            }
            Err(_) => break,  // Hit limit - expected behavior
        }
    }
    
    // Should successfully create reasonable depth for CI
    assert!(depth_reached >= 40, "Should handle reasonable nesting depth");
}

/// Integration test - validate that SIGSEGV fixes preserve language semantics
#[test]
fn ci_sigsegv_fixes_preserve_semantics() {
    let bridge = LegacyValueBridge::new_default();
    let table = SafeSymbolTable::new();
    let env = SafeEnvironment::new_global();
    
    // Test integrated language operations
    let sym_id = table.intern("test-var");
    let symbol = Value::Symbol(sym_id);
    let value = Value::Literal(Literal::ExactInteger(42));
    
    // Test value optimization preserves semantics
    let opt_symbol = bridge.optimize_value(&symbol);
    let opt_value = bridge.optimize_value(&value);
    
    // Test environment operations with optimized values
    env.define(sym_id, opt_value.clone()).unwrap();
    let retrieved = env.lookup(sym_id).unwrap();
    
    // Critical: All operations should work without SIGSEGV
    assert_eq!(opt_symbol.as_symbol(), Some(sym_id));
    assert_eq!(retrieved.as_integer(), Some(42));
    assert_eq!(opt_value.as_integer(), Some(42));
}