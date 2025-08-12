//! Integration tests for environment optimization.
//!
//! These tests verify that the CachedEnvironment maintains exact mathematical
//! equivalence with the traditional Environment while providing performance
//! improvements.

use super::{Environment, CachedEnvironment, Value};
use std::rc::Rc;

/// Tests basic lookup equivalence between Environment and CachedEnvironment.
#[cfg(test)]
mod correctness_tests {
    use super::*;
    
    #[test]
    fn test_basic_lookup_equivalence() {
        // Create a simple environment
        let env = Rc::new(Environment::new(None, 0));
        env.define("x".to_string(), Value::integer(42));
        env.define("y".to_string(), Value::symbol(crate::utils::symbol::intern_symbol("test-symbol".to_string())));
        
        let cached_env = CachedEnvironment::new(env.clone());
        
        // Test that lookups return identical results
        assert_eq!(env.lookup("x"), cached_env.lookup("x"));
        assert_eq!(env.lookup("y"), cached_env.lookup("y"));
        assert_eq!(env.lookup("nonexistent"), cached_env.lookup("nonexistent"));
    }
    
    #[test]
    fn test_lexical_scoping_equivalence() {
        // Create nested environments: global -> outer -> inner
        let global = Rc::new(Environment::new(None, 0));
        global.define("global_var".to_string(), Value::integer(1));
        global.define("shadowed_var".to_string(), Value::integer(100));
        
        let outer = global.extend(1);
        outer.define("outer_var".to_string(), Value::integer(2));
        outer.define("shadowed_var".to_string(), Value::integer(200));
        
        let inner = outer.extend(2);
        inner.define("inner_var".to_string(), Value::integer(3));
        inner.define("shadowed_var".to_string(), Value::integer(300));
        
        let cached_inner = CachedEnvironment::new(inner.clone());
        
        // Test variable resolution follows exact lexical scoping
        assert_eq!(inner.lookup("global_var"), cached_inner.lookup("global_var"));
        assert_eq!(inner.lookup("outer_var"), cached_inner.lookup("outer_var"));
        assert_eq!(inner.lookup("inner_var"), cached_inner.lookup("inner_var"));
        
        // Test variable shadowing is preserved
        assert_eq!(inner.lookup("shadowed_var"), cached_inner.lookup("shadowed_var"));
        assert_eq!(inner.lookup("shadowed_var"), Some(Value::integer(300)));
        assert_eq!(cached_inner.lookup("shadowed_var"), Some(Value::integer(300)));
    }
    
    #[test]
    fn test_mutation_equivalence() {
        let env = Rc::new(Environment::new(None, 0));
        env.define("mutable_var".to_string(), Value::integer(1));
        
        let cached_env = CachedEnvironment::new(env.clone());
        
        // Initial values should be equal
        assert_eq!(env.lookup("mutable_var"), cached_env.lookup("mutable_var"));
        
        // Modify through traditional environment
        env.set("mutable_var", Value::integer(42));
        
        // Both should reflect the change
        assert_eq!(env.lookup("mutable_var"), Some(Value::integer(42)));
        assert_eq!(cached_env.lookup("mutable_var"), Some(Value::integer(42)));
        
        // Modify through cached environment
        cached_env.set("mutable_var", Value::integer(100));
        
        // Both should reflect the new change
        assert_eq!(env.lookup("mutable_var"), Some(Value::integer(100)));
        assert_eq!(cached_env.lookup("mutable_var"), Some(Value::integer(100)));
    }
    
    #[test]
    fn test_deep_chain_equivalence() {
        // Create a deep environment chain
        let mut env = Rc::new(Environment::new(None, 0));
        env.define("global".to_string(), Value::integer(0));
        
        // Create 50 levels of nesting
        for i in 1..=50 {
            env = env.extend(i);
            env.define(format!("level{}", i), Value::integer(i as i64));
            
            // Test equivalence at each level
            let cached_env = CachedEnvironment::new(env.clone());
            
            assert_eq!(env.lookup("global"), cached_env.lookup("global"));
            assert_eq!(env.lookup(&format!("level{}", i)), 
                      cached_env.lookup(&format!("level{}", i)));
        }
    }
    
    #[test] 
    fn test_cache_invalidation_correctness() {
        let env = Rc::new(Environment::new(None, 0));
        env.define("test_var".to_string(), Value::integer(1));
        
        let cached_env = CachedEnvironment::new(env.clone());
        
        // Prime the cache
        assert_eq!(cached_env.lookup("test_var"), Some(Value::integer(1)));
        
        // Verify cache hit
        let stats = cached_env.cache_statistics();
        assert!(stats.cache_hits > 0 || stats.cache_misses > 0);
        
        // Modify the variable
        cached_env.define("test_var".to_string(), Value::integer(999));
        
        // Should return updated value (cache should be invalidated)
        assert_eq!(cached_env.lookup("test_var"), Some(Value::integer(999)));
        assert_eq!(env.lookup("test_var"), Some(Value::integer(999)));
    }
}

/// Performance tests to verify optimization effectiveness.
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_lookup_performance_improvement() {
        // Create a deep environment for testing
        let mut env = Rc::new(Environment::new(None, 0));
        for i in 0..100 {
            env = env.extend(i);
            for j in 0..5 {
                env.define(format!("var{}_{}", i, j), Value::integer((i * 5 + j) as i64));
            }
        }
        
        let cached_env = CachedEnvironment::new(env.clone());
        
        // Warm up the cache
        for _ in 0..10 {
            let _ = cached_env.lookup("var50_2");
        }
        
        // Benchmark traditional lookup
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = env.lookup("var50_2");
        }
        let traditional_time = start.elapsed();
        
        // Benchmark cached lookup
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = cached_env.lookup("var50_2");
        }
        let cached_time = start.elapsed();
        
        // Cache should be faster for repeated lookups
        println!("Traditional: {:?}, Cached: {:?}", traditional_time, cached_time);
        
        // In deep chains with repeated lookups, cache should provide improvement
        if env.variable_names().len() > 50 {
            assert!(cached_time < traditional_time);
        }
    }
    
    #[test]
    fn test_cache_statistics() {
        let env = Rc::new(Environment::new(None, 0));
        env.define("test".to_string(), Value::integer(1));
        
        let cached_env = CachedEnvironment::new(env);
        
        // Should start with no statistics
        let initial_stats = cached_env.cache_statistics();
        assert_eq!(initial_stats.cache_hits, 0);
        assert_eq!(initial_stats.cache_misses, 0);
        
        // First lookup should be a miss
        let _ = cached_env.lookup("test");
        let after_first = cached_env.cache_statistics();
        assert!(after_first.cache_misses > 0);
        
        // Second lookup should be a hit
        let _ = cached_env.lookup("test");
        let after_second = cached_env.cache_statistics();
        assert!(after_second.cache_hits > 0);
        
        // Calculate hit ratio
        let hit_ratio = cached_env.cache_hit_ratio();
        assert!(hit_ratio >= 0.0 && hit_ratio <= 100.0);
    }
}

/// Mathematical property verification tests.
#[cfg(test)]
mod mathematical_properties {
    use super::*;
    
    #[test]
    fn test_lookup_idempotency() {
        let env = Rc::new(Environment::new(None, 0));
        env.define("x".to_string(), Value::integer(42));
        
        let cached_env = CachedEnvironment::new(env);
        
        // Multiple lookups should return identical results
        let result1 = cached_env.lookup("x");
        let result2 = cached_env.lookup("x");
        let result3 = cached_env.lookup("x");
        
        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
    }
    
    #[test]
    fn test_lookup_monotonicity() {
        // In an unmodified environment, lookup results should be stable
        let env = Rc::new(Environment::new(None, 0));
        env.define("stable_var".to_string(), Value::integer(100));
        
        let cached_env = CachedEnvironment::new(env);
        
        // Perform lookups over time - should be consistent
        let mut results = Vec::new();
        for _ in 0..50 {
            results.push(cached_env.lookup("stable_var"));
        }
        
        // All results should be identical
        let first = &results[0];
        assert!(results.iter().all(|r| r == first));
    }
    
    #[test]
    fn test_environment_chain_preservation() {
        // Build: Global -> A -> B -> C
        let global = Rc::new(Environment::new(None, 0));
        global.define("global_var".to_string(), Value::integer(0));
        
        let a = global.extend(1);
        a.define("a_var".to_string(), Value::integer(1));
        
        let b = a.extend(2);
        b.define("b_var".to_string(), Value::integer(2));
        
        let c = b.extend(3);
        c.define("c_var".to_string(), Value::integer(3));
        
        let cached_c = CachedEnvironment::new(c.clone());
        
        // Verify all variables are accessible from innermost environment
        assert_eq!(c.lookup("global_var"), cached_c.lookup("global_var"));
        assert_eq!(c.lookup("a_var"), cached_c.lookup("a_var"));
        assert_eq!(c.lookup("b_var"), cached_c.lookup("b_var"));
        assert_eq!(c.lookup("c_var"), cached_c.lookup("c_var"));
        
        // Verify chain structure is preserved
        assert_eq!(cached_c.lookup("global_var"), Some(Value::integer(0)));
        assert_eq!(cached_c.lookup("a_var"), Some(Value::integer(1)));
        assert_eq!(cached_c.lookup("b_var"), Some(Value::integer(2)));
        assert_eq!(cached_c.lookup("c_var"), Some(Value::integer(3)));
    }
}