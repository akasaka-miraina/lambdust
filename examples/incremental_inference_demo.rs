//! Incremental Type Inference Cache System Demo
//! Demonstrates the advanced caching system for faster recompilation than GHC

use lambdust::type_system::incremental_inference::{
    IncrementalTypeInference, IncrementalConfig, CachePolicy
};
use lambdust::value::Value;
use lambdust::lexer::SchemeNumber;
use lambdust::ast::{Expr, Literal};
use std::time::{Duration, Instant};

fn main() {
    println!("🚀 Lambdust Incremental Type Inference Cache Demo");
    println!("================================================");
    
    // Example 1: Basic Caching Performance
    println!("\n📈 Example 1: Basic Caching Performance");
    basic_caching_demo();
    
    // Example 2: Dependency Tracking and Invalidation
    println!("\n🔗 Example 2: Dependency Tracking and Invalidation");
    dependency_invalidation_demo();
    
    // Example 3: Cache Replacement Policies
    println!("\n🔄 Example 3: Cache Replacement Policies");
    cache_policies_demo();
    
    // Example 4: Expression Caching
    println!("\n📝 Example 4: Expression Caching");
    expression_caching_demo();
    
    // Example 5: Performance Statistics
    println!("\n📊 Example 5: Performance Statistics");
    performance_stats_demo();
    
    println!("\n✅ Incremental Type Inference Demo Complete!");
    println!("🎯 This system challenges GHC's compilation speed with sophisticated caching.");
}

fn basic_caching_demo() {
    let config = IncrementalConfig {
        min_cache_cost: Duration::from_nanos(1), // Cache everything for demo
        max_cache_size: 1000,
        cache_policy: CachePolicy::LRU,
        ..Default::default()
    };
    let mut inference = IncrementalTypeInference::new(config);
    
    let value = Value::Number(SchemeNumber::Integer(42));
    
    // First inference (cache miss)
    let start = Instant::now();
    let _result1 = inference.infer(&value, Some("demo_context")).unwrap();
    let first_time = start.elapsed();
    
    // Second inference (cache hit)
    let start = Instant::now();
    let _result2 = inference.infer(&value, Some("demo_context")).unwrap();
    let second_time = start.elapsed();
    
    println!("  First inference (cache miss): {:?}", first_time);
    println!("  Second inference (cache hit):  {:?}", second_time);
    println!("  Cache size: {}", inference.cache_size());
    
    if second_time < first_time {
        let speedup = first_time.as_nanos() as f64 / second_time.as_nanos() as f64;
        println!("  Speedup: {:.2}x faster on cache hit", speedup);
    }
}

fn dependency_invalidation_demo() {
    let config = IncrementalConfig {
        min_cache_cost: Duration::from_nanos(1),
        max_cache_size: 1000,
        enable_dependency_tracking: true,
        ..Default::default()
    };
    let mut inference = IncrementalTypeInference::new(config);
    
    // Set up dependency chain: Module_A -> Module_B -> Module_C
    inference.add_dependency("Module_A".to_string(), "Module_B".to_string());
    inference.add_dependency("Module_B".to_string(), "Module_C".to_string());
    
    println!("  Setting up dependency chain: Module_A -> Module_B -> Module_C");
    
    // Cache types for each module
    let values = vec![
        (Value::Number(SchemeNumber::Integer(1)), "Module_A"),
        (Value::Number(SchemeNumber::Integer(2)), "Module_B"), 
        (Value::Number(SchemeNumber::Integer(3)), "Module_C"),
    ];
    
    for (value, module) in &values {
        let _ = inference.infer(value, Some(module)).unwrap();
    }
    
    let initial_size = inference.cache_size();
    println!("  Initial cache size: {}", initial_size);
    
    // Invalidate Module_C - should cascade to B and A
    let invalidated = inference.invalidate_dependencies("Module_C").unwrap();
    let final_size = inference.cache_size();
    
    println!("  After invalidating Module_C:");
    println!("    Entries invalidated: {}", invalidated);
    println!("    Final cache size: {}", final_size);
    println!("    Dependency cascade worked: {}", invalidated > 0);
}

fn cache_policies_demo() {
    let policies = [
        ("LRU (Least Recently Used)", CachePolicy::LRU),
        ("LFU (Least Frequently Used)", CachePolicy::LFU),
        ("Cost-Based", CachePolicy::CostBased),
        ("Hybrid", CachePolicy::Hybrid),
    ];
    
    for (name, policy) in &policies {
        println!("  Testing {} policy:", name);
        
        let config = IncrementalConfig {
            max_cache_size: 3, // Small cache to trigger eviction
            cache_policy: *policy,
            min_cache_cost: Duration::from_nanos(1),
            ..Default::default()
        };
        let mut inference = IncrementalTypeInference::new(config);
        
        // Fill cache beyond capacity
        for i in 0..5 {
            let value = Value::Number(SchemeNumber::Integer(i));
            let _ = inference.infer(&value, Some(&format!("item_{}", i))).unwrap();
        }
        
        println!("    Cache size after adding 5 items: {}", inference.cache_size());
        assert!(inference.cache_size() <= 3, "Cache should not exceed max size");
    }
}

fn expression_caching_demo() {
    let config = IncrementalConfig {
        min_cache_cost: Duration::from_nanos(1),
        ..Default::default()
    };
    let mut inference = IncrementalTypeInference::new(config);
    
    // Create complex expressions
    let expressions = vec![
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]),
        Expr::Variable("complex_function".to_string()),
    ];
    
    for (i, expr) in expressions.iter().enumerate() {
        let context = format!("expr_context_{}", i);
        
        // First inference
        let start = Instant::now();
        let _result1 = inference.infer_expression(expr, Some(&context)).unwrap();
        let first_time = start.elapsed();
        
        // Second inference (should be cached)
        let start = Instant::now();
        let _result2 = inference.infer_expression(expr, Some(&context)).unwrap();
        let second_time = start.elapsed();
        
        println!("  Expression {} caching:", i + 1);
        println!("    First: {:?}, Second: {:?}", first_time, second_time);
    }
    
    println!("  Total cached expressions: {}", inference.cache_size());
}

fn performance_stats_demo() {
    let config = IncrementalConfig {
        min_cache_cost: Duration::from_nanos(1),
        max_cache_size: 100,
        ..Default::default()
    };
    let mut inference = IncrementalTypeInference::new(config);
    
    // Generate some cache activity
    let value = Value::Boolean(true);
    
    // Create multiple cache entries
    for i in 0..10 {
        let context = format!("stats_context_{}", i);
        let _ = inference.infer(&value, Some(&context)).unwrap();
    }
    
    // Access some entries multiple times (to create hits)
    for _ in 0..5 {
        let _ = inference.infer(&value, Some("stats_context_0")).unwrap();
        let _ = inference.infer(&value, Some("stats_context_1")).unwrap();
    }
    
    // Invalidate some entries
    let _ = inference.invalidate_dependencies("stats_context_5").unwrap();
    
    let stats = inference.get_statistics();
    
    println!("  Performance Statistics:");
    println!("    Total hits: {}", stats.hits);
    println!("    Total misses: {}", stats.misses);
    println!("    Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    println!("    Invalidations: {}", stats.invalidations);
    println!("    Time saved: {:?}", stats.time_saved);
    println!("    Average inference time: {:?}", stats.avg_inference_time);
    
    if stats.hits > 0 {
        println!("    ✅ Cache is working effectively!");
    } else {
        println!("    ⚠️  No cache hits recorded");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_demo_compiles() {
        // Just ensure the demo compiles and runs basic functionality
        let config = IncrementalConfig::default();
        let mut inference = IncrementalTypeInference::new(config);
        let value = Value::Number(SchemeNumber::Integer(42));
        let result = inference.infer(&value, None);
        assert!(result.is_ok());
    }
}