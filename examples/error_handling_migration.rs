//! Example demonstrating the unified error handling system migration.
//!
//! This example shows how to migrate from the old error handling patterns
//! to the new unified system, demonstrating the performance and maintainability benefits.

use lambdust::diagnostics::{
    ErrorCategory, ErrorSeverity, UnifiedError, UnifiedResult, error_convert, jit_error,
    propagate_error, validate_arity,
};
use lambdust::eval::Value;
use lambdust::jit::CompilationTier;
use lambdust::jit::unified_jit_errors::{
    JitErrorKind, JitUnifiedError, PerformanceImpact, ResourceOverhead,
};
use std::time::Instant;

/// Example showing old vs new error handling patterns.
fn main() {
    println!("=== Lambdust Unified Error Handling Migration Example ===\n");

    // Example 1: Basic error creation comparison
    println!("1. Basic Error Creation:");
    demonstrate_basic_errors();

    // Example 2: Error conversion patterns
    println!("\n2. Error Conversion Patterns:");
    demonstrate_error_conversion();

    // Example 3: JIT-specific error handling
    println!("\n3. JIT Error Handling:");
    demonstrate_jit_errors();

    // Example 4: Performance comparison
    println!("\n4. Performance Comparison:");
    demonstrate_performance();

    println!("\n=== Migration Complete ===");
}

fn demonstrate_basic_errors() {
    // OLD WAY - Verbose and repetitive
    println!("OLD WAY (verbose):");
    let _old_error = lambdust::diagnostics::Error::runtime_error(
        "Type mismatch: expected number, got string".to_string(),
        None,
    );
    println!("  Error::runtime_error(\"Type mismatch: expected number, got string\", None)");

    // NEW WAY - Concise and type-safe
    println!("NEW WAY (concise):");
    let _new_error = UnifiedError::new(
        lambdust::diagnostics::RuntimeError,
        "Type mismatch: expected number, got string",
    )
    .with_context("expected", "number")
    .with_context("actual", "string");
    println!("  UnifiedError::new(RuntimeError, \"Type mismatch\")");
    println!("    .with_context(\"expected\", \"number\")");
    println!("    .with_context(\"actual\", \"string\")");
}

fn demonstrate_error_conversion() {
    // OLD WAY - Manual map_err chains
    println!("OLD WAY (manual map_err):");
    fn old_parse_number(s: &str) -> lambdust::diagnostics::Result<f64> {
        s.parse::<f64>().map_err(|e| {
            lambdust::diagnostics::Error::runtime_error(
                format!("Failed to parse number: {}", e),
                None,
            )
            .boxed()
        })
    }

    match old_parse_number("not_a_number") {
        Err(_) => println!("  Manual error conversion with map_err - verbose!"),
        _ => {}
    }

    // NEW WAY - Macro-based conversion
    println!("NEW WAY (macro-based):");
    fn new_parse_number(s: &str) -> UnifiedResult<f64> {
        error_convert!(
            s.parse::<f64>(),
            lambdust::diagnostics::RuntimeError,
            "Failed to parse number",
            context: s
        )
    }

    match new_parse_number("not_a_number") {
        Err(_) => println!("  Macro-based conversion - concise and consistent!"),
        _ => {}
    }
}

fn demonstrate_jit_errors() {
    println!("JIT Error with Performance Impact Assessment:");

    // Create a JIT error with comprehensive context
    let jit_error = jit_error!(
        JitErrorKind::CodeGeneration,
        "Failed to generate optimized code for arithmetic operation",
        tier: CompilationTier::JitOptimized,
        impact: PerformanceImpact {
            degradation_factor: 2.5,
            fallback_available: true,
            resource_overhead: ResourceOverhead {
                memory_bytes: 4096,
                cpu_cycles: 100_000,
                io_operations: 0,
            }
        }
    );

    println!("  JIT Error Category: {:?}", jit_error.jit_kind);
    println!("  Allows Fallback: {}", jit_error.allows_fallback());
    println!(
        "  Performance Impact: {:.1}x degradation",
        jit_error
            .performance_impact
            .as_ref()
            .unwrap()
            .degradation_factor
    );

    // Demonstrate automatic fallback handling
    fn jit_compile_with_fallback() -> UnifiedResult<String> {
        // Simulate JIT compilation failure
        let jit_result: Result<String, JitUnifiedError> = Err(jit_error!(
            JitErrorKind::Optimization,
            "Optimization failed - falling back to interpreter"
        ));

        // Use jit_try! macro for automatic fallback
        let result = crate::jit_try!(jit_result, fallback: "interpreter_result".to_string());
        Ok(result)
    }

    match jit_compile_with_fallback() {
        Ok(result) => println!("  Fallback successful: {}", result),
        Err(e) => println!("  Fallback failed: {}", e),
    }
}

fn demonstrate_performance() {
    const ITERATIONS: usize = 10_000;

    println!("Performance comparison over {} iterations:", ITERATIONS);

    // Benchmark old error handling
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _error = lambdust::diagnostics::Error::runtime_error(format!("Error {}", i), None);
    }
    let old_duration = start.elapsed();
    println!("  Old error system: {:?}", old_duration);

    // Benchmark new error handling
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _error = UnifiedError::new(lambdust::diagnostics::RuntimeError, format!("Error {}", i));
    }
    let new_duration = start.elapsed();
    println!("  New error system: {:?}", new_duration);

    let speedup = old_duration.as_nanos() as f64 / new_duration.as_nanos() as f64;
    println!("  Speedup: {:.2}x faster", speedup);
}

// Example function using the new validation macros
fn example_arithmetic_function(args: &[Value]) -> UnifiedResult<f64> {
    // Validate argument count
    validate_arity!(args, 2, function: "add");

    // Validate argument types with automatic error generation
    let a = crate::validate_type!(args[0], number)?;
    let b = crate::validate_type!(args[1], number)?;

    Ok(a + b)
}

// Example showing error propagation with context
fn example_nested_operation() -> UnifiedResult<String> {
    let numbers = vec![Value::number(1.0), Value::number(2.0)];

    // This will succeed
    let result = propagate_error!(
        example_arithmetic_function(&numbers),
        context: "nested arithmetic operation"
    );

    Ok(format!("Result: {}", result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_arithmetic_function() {
        let args = vec![Value::number(5.0), Value::number(3.0)];
        let result = example_arithmetic_function(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 8.0);
    }

    #[test]
    fn test_example_arithmetic_function_arity_error() {
        let args = vec![Value::number(5.0)]; // Wrong arity
        let result = example_arithmetic_function(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_operation() {
        let result = example_nested_operation();
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Result: 3"));
    }
}
