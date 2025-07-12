//! Memory usage analysis for string optimization
//! Estimates memory savings from ShortString optimization

use lambdust::value::Value;
use std::mem;

fn main() {
    println!("=== String Memory Usage Analysis ===\n");
    
    analyze_value_sizes();
    estimate_memory_savings();
}

fn analyze_value_sizes() {
    println!("1. Value Enum Size Analysis:");
    println!("  sizeof(Value): {} bytes", mem::size_of::<Value>());
    
    // The actual size depends on the largest variant
    let string_value = Value::from("hello world with long string content for heap allocation");
    let short_string_value = Value::from("hello");
    
    println!("  Example String value (heap): {:?}", string_value);
    println!("  Example ShortString value (stack): {:?}", short_string_value);
    println!();
}

fn estimate_memory_savings() {
    println!("2. Memory Savings Estimation:");
    
    // Typical Scheme programs have many short identifiers
    let common_identifiers = vec![
        "x", "y", "z", "i", "j", "k",           // single char vars
        "car", "cdr", "cons", "list",           // common functions  
        "let", "if", "cond", "lambda",          // keywords
        "define", "begin", "quote",             // more keywords
        "+", "-", "*", "/", "=", "<", ">",     // operators
    ];
    
    let mut total_heap_saved = 0;
    let mut optimized_count = 0;
    
    for identifier in &common_identifiers {
        let len = identifier.len();
        if len <= 15 {
            // Would use ShortString instead of String
            // Each String allocation: ~24 bytes overhead + content
            let heap_saved = 24 + len;
            total_heap_saved += heap_saved;
            optimized_count += 1;
            
            println!("  '{}' ({} bytes) - saves {} bytes of heap allocation", 
                    identifier, len, heap_saved);
        }
    }
    
    println!("\n3. Summary:");
    println!("  Total identifiers analyzed: {}", common_identifiers.len());
    println!("  Optimized (ShortString): {}", optimized_count);
    println!("  Total heap memory saved: {} bytes", total_heap_saved);
    println!("  Average heap savings per optimized string: {:.1} bytes", 
             total_heap_saved as f64 / optimized_count as f64);
    
    // Extrapolate for a typical Scheme program
    let typical_program_identifiers = 1000; // Conservative estimate
    let short_identifier_ratio = 0.8; // 80% of identifiers are short
    let estimated_savings = (typical_program_identifiers as f64 * short_identifier_ratio * 
                           (total_heap_saved as f64 / optimized_count as f64)) as usize;
    
    println!("\n4. Projected Savings for Typical Program:");
    println!("  Estimated identifiers: {}", typical_program_identifiers);
    println!("  Short identifier ratio: {:.1}%", short_identifier_ratio * 100.0);
    println!("  Projected heap memory savings: {} bytes ({:.1} KB)", 
             estimated_savings, estimated_savings as f64 / 1024.0);
}