//! Demonstration of custom type predicates system
//! 
//! This example shows how to use the custom predicates system to define
//! and use custom type checkers in the Lambdust Scheme interpreter.

use lambdust::{
    value::{
        Value, global_custom_predicate_registry,
        register_global_custom_predicate, evaluate_global_custom_predicate,
    },
    lexer::SchemeNumber,
};

fn main() {
    println!("🔍 Custom Type Predicates Demonstration");
    println!("=======================================\n");

    // Clear the global registry to start fresh
    global_custom_predicate_registry().clear().unwrap();
    
    // Demo 1: Basic custom predicate registration
    demo_basic_predicates();
    
    // Demo 2: Complex predicates with business logic
    demo_complex_predicates();
    
    // Demo 3: Predicate management operations
    demo_predicate_management();
    
    // Demo 4: Error handling and edge cases
    demo_error_handling();
    
    // Demo 5: Performance and concurrency
    demo_performance_and_concurrency();
    
    println!("\n✅ Custom predicates demonstration completed!");
}

fn demo_basic_predicates() {
    println!("📋 Demo 1: Basic Custom Predicates");
    println!("----------------------------------");
    
    // Register a simple even number predicate
    let result = register_global_custom_predicate(
        "even-number?".to_string(),
        Some("Check if a number is even".to_string()),
        |value| {
            match value {
                Value::Number(SchemeNumber::Integer(n)) => n % 2 == 0,
                Value::Number(SchemeNumber::Real(r)) => {
                    let rounded = r.round();
                    (r - rounded).abs() < f64::EPSILON && (rounded as i64) % 2 == 0
                },
                _ => false,
            }
        },
    );
    
    match result {
        Ok(()) => println!("✓ Registered 'even-number?' predicate"),
        Err(e) => println!("✗ Failed to register predicate: {}", e),
    }
    
    // Test the predicate with different values
    let test_values = vec![
        ("42", Value::Number(SchemeNumber::Integer(42))),
        ("43", Value::Number(SchemeNumber::Integer(43))),
        ("44.0", Value::Number(SchemeNumber::Real(44.0))),
        ("45.5", Value::Number(SchemeNumber::Real(45.5))),
        ("\"hello\"", Value::String("hello".to_string())),
    ];
    
    for (description, value) in test_values {
        match evaluate_global_custom_predicate("even-number?", &value) {
            Ok(Some(result)) => println!("  even-number?({}) = {}", description, result),
            Ok(None) => println!("  even-number?({}) = <predicate not found>", description),
            Err(e) => println!("  even-number?({}) = <error: {}>", description, e),
        }
    }
    
    println!();
}

fn demo_complex_predicates() {
    println!("🧮 Demo 2: Complex Business Logic Predicates");
    println!("--------------------------------------------");
    
    // Register a predicate for valid email-like strings
    register_global_custom_predicate(
        "email-like?".to_string(),
        Some("Check if string looks like an email address".to_string()),
        |value| {
            if let Some(s) = value.as_string() {
                s.contains('@') && s.contains('.') && s.len() > 5
            } else {
                false
            }
        },
    ).unwrap();
    
    // Register a predicate for positive numbers
    register_global_custom_predicate(
        "positive?".to_string(),
        Some("Check if number is positive".to_string()),
        |value| {
            match value {
                Value::Number(SchemeNumber::Integer(n)) => *n > 0,
                Value::Number(SchemeNumber::Real(r)) => *r > 0.0,
                _ => false,
            }
        },
    ).unwrap();
    
    println!("✓ Registered complex predicates: email-like?, positive?");
    
    // Test email-like predicate
    let email_tests = vec![
        "user@example.com",
        "invalid-email",
        "missing@domain",
        "user@domain.co.uk",
        "@example.com",
        "user@",
    ];
    
    println!("\nEmail-like tests:");
    for email in email_tests {
        let value = Value::String(email.to_string());
        let result = evaluate_global_custom_predicate("email-like?", &value).unwrap().unwrap();
        println!("  email-like?(\"{}\") = {}", email, result);
    }
    
    // Test positive predicate
    let number_tests = vec![
        Value::Number(SchemeNumber::Integer(42)),
        Value::Number(SchemeNumber::Integer(-5)),
        Value::Number(SchemeNumber::Real(3.14)),
        Value::Number(SchemeNumber::Real(-2.71)),
        Value::Number(SchemeNumber::Integer(0)),
    ];
    
    println!("\nPositive number tests:");
    for value in number_tests {
        let result = evaluate_global_custom_predicate("positive?", &value).unwrap().unwrap();
        let desc = match &value {
            Value::Number(SchemeNumber::Integer(n)) => n.to_string(),
            Value::Number(SchemeNumber::Real(r)) => r.to_string(),
            _ => "unknown".to_string(),
        };
        println!("  positive?({}) = {}", desc, result);
    }
    
    println!();
}

fn demo_predicate_management() {
    println!("🔧 Demo 3: Predicate Management Operations");
    println!("------------------------------------------");
    
    let registry = global_custom_predicate_registry();
    
    // List all current predicates
    match registry.list_predicates() {
        Ok(predicates) => {
            println!("Current predicates ({}):", predicates.len());
            for predicate in &predicates {
                println!("  - {}", predicate);
                
                // Get detailed info for each predicate
                if let Ok(Some(info)) = registry.get_info(predicate) {
                    if let Some(desc) = &info.description {
                        println!("    Description: {}", desc);
                    }
                }
            }
        },
        Err(e) => println!("Error listing predicates: {}", e),
    }
    
    // Add a temporary predicate
    println!("\nAdding temporary predicate...");
    register_global_custom_predicate(
        "temp-predicate?".to_string(),
        Some("A temporary predicate for demonstration".to_string()),
        |_| true,
    ).unwrap();
    
    println!("✓ Added temp-predicate?");
    println!("Predicates after addition: {}", registry.list_predicates().unwrap().len());
    
    // Remove the temporary predicate
    println!("\nRemoving temporary predicate...");
    match registry.unregister("temp-predicate?") {
        Ok(true) => println!("✓ Removed temp-predicate?"),
        Ok(false) => println!("✗ temp-predicate? was not found"),
        Err(e) => println!("✗ Error removing predicate: {}", e),
    }
    
    println!("Predicates after removal: {}", registry.list_predicates().unwrap().len());
    println!();
}

fn demo_error_handling() {
    println!("⚠️  Demo 4: Error Handling and Edge Cases");
    println!("------------------------------------------");
    
    let registry = global_custom_predicate_registry();
    
    // Try to register a duplicate predicate
    println!("Attempting to register duplicate predicate...");
    let duplicate_result = register_global_custom_predicate(
        "even-number?".to_string(), // This already exists
        None,
        |_| false,
    );
    
    match duplicate_result {
        Ok(()) => println!("✗ Unexpectedly succeeded in registering duplicate"),
        Err(e) => println!("✓ Correctly rejected duplicate: {}", e),
    }
    
    // Try to evaluate non-existent predicate
    println!("\nEvaluating non-existent predicate...");
    let test_value = Value::Number(SchemeNumber::Integer(42));
    match evaluate_global_custom_predicate("nonexistent-predicate?", &test_value) {
        Ok(Some(result)) => println!("✗ Unexpectedly found result: {}", result),
        Ok(None) => println!("✓ Correctly returned None for non-existent predicate"),
        Err(e) => println!("✗ Unexpected error: {}", e),
    }
    
    // Try to get info for non-existent predicate
    println!("\nGetting info for non-existent predicate...");
    match registry.get_info("nonexistent-predicate?") {
        Ok(Some(_)) => println!("✗ Unexpectedly found info"),
        Ok(None) => println!("✓ Correctly returned None for non-existent predicate"),
        Err(e) => println!("✗ Unexpected error: {}", e),
    }
    
    // Try to remove non-existent predicate
    println!("\nRemoving non-existent predicate...");
    match registry.unregister("nonexistent-predicate?") {
        Ok(true) => println!("✗ Unexpectedly reported removal of non-existent predicate"),
        Ok(false) => println!("✓ Correctly reported predicate not found"),
        Err(e) => println!("✗ Unexpected error: {}", e),
    }
    
    println!();
}

fn demo_performance_and_concurrency() {
    println!("🚀 Demo 5: Performance and Concurrency");
    println!("--------------------------------------");
    
    use std::time::Instant;
    use std::thread;
    use std::sync::Arc;
    
    // Performance test: Register and evaluate many predicates
    println!("Performance test: Registering multiple predicates...");
    let start = Instant::now();
    
    for i in 0..100 {
        let name = format!("perf-test-{}?", i);
        register_global_custom_predicate(
            name,
            Some(format!("Performance test predicate {}", i)),
            move |value| {
                // Simple predicate that checks if number equals the index
                match value {
                    Value::Number(SchemeNumber::Integer(n)) => *n == i,
                    _ => false,
                }
            },
        ).unwrap();
    }
    
    let registration_time = start.elapsed();
    println!("✓ Registered 100 predicates in {:?}", registration_time);
    
    // Performance test: Evaluate predicates
    println!("\nPerformance test: Evaluating predicates...");
    let start = Instant::now();
    let test_value = Value::Number(SchemeNumber::Integer(50));
    
    for i in 0..100 {
        let name = format!("perf-test-{}?", i);
        evaluate_global_custom_predicate(&name, &test_value).unwrap();
    }
    
    let evaluation_time = start.elapsed();
    println!("✓ Evaluated 100 predicates in {:?}", evaluation_time);
    
    // Concurrency test: Access registry from multiple threads
    println!("\nConcurrency test: Multi-threaded access...");
    let registry = Arc::new(global_custom_predicate_registry());
    let mut handles = vec![];
    
    for thread_id in 0..5 {
        let registry_clone = Arc::clone(&registry);
        let handle = thread::spawn(move || {
            let test_value = Value::Number(SchemeNumber::Integer(thread_id));
            
            // Each thread evaluates some predicates
            for i in 0..20 {
                let name = format!("perf-test-{}?", i);
                registry_clone.evaluate(&name, &test_value).unwrap();
            }
            
            thread_id
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    let mut completed_threads = Vec::new();
    for handle in handles {
        completed_threads.push(handle.join().unwrap());
    }
    
    println!("✓ Completed concurrent access from {} threads: {:?}", 
             completed_threads.len(), completed_threads);
    
    // Clean up performance test predicates
    println!("\nCleaning up performance test predicates...");
    for i in 0..100 {
        let name = format!("perf-test-{}?", i);
        global_custom_predicate_registry().unregister(&name).unwrap();
    }
    println!("✓ Cleaned up 100 performance test predicates");
    
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_runs_without_panic() {
        // This test ensures the demo can run without panicking
        // In a real scenario, you might want more detailed testing
        main();
    }
    
    #[test]
    fn test_even_number_predicate_logic() {
        let predicate = |value: &Value| {
            match value {
                Value::Number(SchemeNumber::Integer(n)) => n % 2 == 0,
                Value::Number(SchemeNumber::Real(r)) => {
                    let rounded = r.round();
                    (r - rounded).abs() < f64::EPSILON && (rounded as i64) % 2 == 0
                },
                _ => false,
            }
        };
        
        // Test cases
        assert!(predicate(&Value::Number(SchemeNumber::Integer(42))));
        assert!(!predicate(&Value::Number(SchemeNumber::Integer(43))));
        assert!(predicate(&Value::Number(SchemeNumber::Real(44.0))));
        assert!(!predicate(&Value::Number(SchemeNumber::Real(45.0))));
        assert!(!predicate(&Value::Number(SchemeNumber::Real(42.5))));
        assert!(!predicate(&Value::String("not a number".to_string())));
    }
    
    #[test]
    fn test_email_like_predicate_logic() {
        let predicate = |value: &Value| {
            match value {
                Value::String(s) => {
                    s.contains('@') && s.contains('.') && s.len() > 5
                },
                _ => false,
            }
        };
        
        assert!(predicate(&Value::String("user@example.com".to_string())));
        assert!(!predicate(&Value::String("invalid".to_string())));
        assert!(!predicate(&Value::String("user@".to_string())));
        assert!(!predicate(&Value::Number(SchemeNumber::Integer(42))));
    }
}