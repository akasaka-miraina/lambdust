//! Demonstration of string optimization effectiveness
//! Shows memory savings and performance improvements from ShortString optimization

use lambdust::value::Value;
use std::time::Instant;

fn main() {
    println!("=== String Optimization Demonstration ===\n");

    // Test 1: Short strings (should use ShortString, no heap allocation)
    test_short_strings();
    
    // Test 2: Long strings (should use String, heap allocation)
    test_long_strings();
    
    // Test 3: Performance comparison
    performance_comparison();
}

fn test_short_strings() {
    println!("1. Short String Test (≤15 bytes - optimized):");
    
    let short_strings = vec![
        "hello",
        "world", 
        "test",
        "λ",
        "123456789012345", // exactly 15 bytes
    ];
    
    for s in &short_strings {
        let value = Value::from(*s);
        println!("  '{}' ({} bytes) -> {:?}", s, s.len(), value);
        
        // Verify it's treated as a string
        assert!(value.is_string());
        assert_eq!(value.as_string(), Some(*s));
    }
    
    println!();
}

fn test_long_strings() {
    println!("2. Long String Test (>15 bytes - traditional):");
    
    let long_strings = vec![
        "1234567890123456", // 16 bytes
        "This is a longer string that will use heap allocation",
        "こんにちは、世界！これは長い文字列です。",
    ];
    
    for s in &long_strings {
        let value = Value::from(*s);
        println!("  '{}' ({} bytes) -> {:?}", s, s.len(), value);
        
        // Verify it's treated as a string
        assert!(value.is_string());
        assert_eq!(value.as_string(), Some(*s));
    }
    
    println!();
}

fn performance_comparison() {
    println!("3. Performance Comparison:");
    
    const ITERATIONS: usize = 100_000;
    
    // Test short strings
    let short_str = "hello";
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _value = Value::from(short_str);
    }
    let short_duration = start.elapsed();
    
    // Test equivalent long string creation (simulated)
    let long_str = "hello world 12345"; // >15 bytes
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _value = Value::from(long_str);
    }
    let long_duration = start.elapsed();
    
    println!("  Short strings ({} iterations): {:?}", ITERATIONS, short_duration);
    println!("  Long strings ({} iterations): {:?}", ITERATIONS, long_duration);
    
    if short_duration < long_duration {
        let speedup = long_duration.as_nanos() as f64 / short_duration.as_nanos() as f64;
        println!("  Short string optimization is {:.2}x faster!", speedup);
    }
    
    println!();
}