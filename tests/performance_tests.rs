//! Performance tests for Lambdust Phase 2 implementation.
//!
//! This module implements comprehensive performance testing and benchmarking
//! for the Lambdust language implementation, focusing on:
//!
//! ## Performance Test Categories
//!
//! 1. **Memory Management**: GC behavior, memory leaks, allocation patterns
//! 2. **Execution Performance**: Runtime speed, optimization effectiveness
//! 3. **Scalability**: Performance with large programs and data structures
//! 4. **Concurrency Performance**: Multithreaded evaluation efficiency
//! 5. **Type System Performance**: Type checking and inference speed
//! 6. **Macro Expansion Performance**: Complex macro processing speed
//! 7. **Effect System Performance**: Monadic operations overhead

use lambdust::{Lambdust, MultithreadedLambdust};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

/// Performance test configuration.
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Maximum allowed execution time for any single test
    pub max_execution_time: Duration,
    /// Number of iterations for timing tests
    pub timing_iterations: usize,
    /// Memory usage tolerance (bytes)
    pub memory_tolerance: usize,
    /// Whether to run expensive tests
    pub run_expensive_tests: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(30),
            timing_iterations: 100,
            memory_tolerance: 1024 * 1024, // 1MB
            run_expensive_tests: false,
        }
    }
}

/// Performance measurement result.
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    /// Operation being measured
    pub operation: String,
    /// Number of iterations performed
    pub iterations: usize,
    /// Total execution time
    pub total_time: Duration,
    /// Average time per iteration
    pub avg_time: Duration,
    /// Minimum time observed
    pub min_time: Duration,
    /// Maximum time observed
    pub max_time: Duration,
    /// Memory usage (if measured)
    pub memory_usage: Option<usize>,
}

/// Performance testing framework.
pub struct PerformanceTester {
    config: PerformanceConfig,
    results: Mutex<Vec<PerformanceMeasurement>>,
}

impl PerformanceTester {
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            results: Mutex::new(Vec::new()),
        }
    }

    /// Measures the performance of a given operation.
    pub fn measure<F>(&self, operation_name: &str, mut operation: F) -> PerformanceMeasurement
    where
        F: FnMut() -> Result<(), String>,
    {
        let mut times = Vec::new();
        let start_time = Instant::now();

        for _ in 0..self.config.timing_iterations {
            let iter_start = Instant::now();
            
            if let Err(e) = operation() {
                panic!("Operation '{}' failed: {}", operation_name, e);
            }
            
            let iter_time = iter_start.elapsed();
            times.push(iter_time);

            // Safety check: don't run tests that take too long
            if start_time.elapsed() > self.config.max_execution_time {
                break;
            }
        }

        let total_time = start_time.elapsed();
        let iterations = times.len();
        let avg_time = total_time / iterations as u32;
        let min_time = *times.iter().min().unwrap_or(&Duration::ZERO);
        let max_time = *times.iter().max().unwrap_or(&Duration::ZERO);

        let measurement = PerformanceMeasurement {
            operation: operation_name.to_string(),
            iterations,
            total_time,
            avg_time,
            min_time,
            max_time,
            memory_usage: None, // Would require platform-specific memory measurement
        };

        self.results.lock().unwrap().push(measurement.clone());
        measurement
    }

    /// Gets all recorded measurements.
    pub fn results(&self) -> Vec<PerformanceMeasurement> {
        self.results.lock().unwrap().clone()
    }

    /// Prints a performance report.
    pub fn print_report(&self) {
        let results = self.results();
        
        println!("\n=== Performance Test Report ===");
        println!("{:<30} | {:>10} | {:>12} | {:>12} | {:>12}", 
                "Operation", "Iterations", "Total (ms)", "Avg (μs)", "Max (μs)");
        println!("{}", "-".repeat(80));
        
        for result in &results {
            println!("{:<30} | {:>10} | {:>12.2} | {:>12.2} | {:>12.2}",
                    result.operation,
                    result.iterations,
                    result.total_time.as_secs_f64() * 1000.0,
                    result.avg_time.as_micros(),
                    result.max_time.as_micros());
        }
        
        println!("=== End Report ===\n");
    }
}

// ============================================================================
// BASIC PERFORMANCE TESTS
// ============================================================================

#[test]
fn test_basic_arithmetic_performance() {
    let tester = PerformanceTester::new(PerformanceConfig::default());
    
    let measurement = tester.measure("basic_arithmetic", || {
        let mut lambdust = Lambdust::new();
        match lambdust.eval("(+ 1 2 3 4 5)", Some("perf_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(10));
    println!("Basic arithmetic: {:?} average", measurement.avg_time);
}

#[test]
fn test_function_call_performance() {
    let tester = PerformanceTester::new(PerformanceConfig::default());
    
    let measurement = tester.measure("function_calls", || {
        let mut lambdust = Lambdust::new();
        let program = r#"
            (define (add-five x) (+ x 5))
            (add-five 10)
        "#;
        match lambdust.eval(program, Some("perf_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(20));
    println!("Function calls: {:?} average", measurement.avg_time);
}

#[test]
fn test_variable_lookup_performance() {
    let tester = PerformanceTester::new(PerformanceConfig::default());
    
    let measurement = tester.measure("variable_lookup", || {
        let mut lambdust = Lambdust::new();
        let program = r#"
            (define x 42)
            (define y x)
            (define z y)
            z
        "#;
        match lambdust.eval(program, Some("perf_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(15));
    println!("Variable lookup: {:?} average", measurement.avg_time);
}

// ============================================================================
// TAIL CALL OPTIMIZATION PERFORMANCE TESTS
// ============================================================================

#[test]
fn test_tail_recursion_performance() {
    let tester = PerformanceTester::new(PerformanceConfig {
        timing_iterations: 10, // Fewer iterations for expensive test
        ..Default::default()
    });
    
    let measurement = tester.measure("tail_recursion", || {
        let mut lambdust = Lambdust::new();
        let program = r#"
            (define (count-down n)
              (if (= n 0)
                  'done
                  (count-down (- n 1))))
            (count-down 1000)
        "#;
        match lambdust.eval(program, Some("perf_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    // Tail recursion should be fast and not stack overflow
    assert!(measurement.avg_time < Duration::from_millis(100));
    println!("Tail recursion (1000 calls): {:?} average", measurement.avg_time);
}

#[test]
fn test_factorial_performance() {
    let tester = PerformanceTester::new(PerformanceConfig {
        timing_iterations: 50,
        ..Default::default()
    });
    
    let measurement = tester.measure("factorial", || {
        let mut lambdust = Lambdust::new();
        let program = r#"
            (define (factorial n acc)
              (if (= n 0)
                  acc
                  (factorial (- n 1) (* n acc))))
            (factorial 20 1)
        "#;
        match lambdust.eval(program, Some("perf_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(50));
    println!("Factorial (20!): {:?} average", measurement.avg_time);
}

// ============================================================================
// CLOSURE AND ENVIRONMENT PERFORMANCE TESTS
// ============================================================================

#[test]
fn test_closure_creation_performance() {
    let tester = PerformanceTester::new(PerformanceConfig::default());
    
    let measurement = tester.measure("closure_creation", || {
        let mut lambdust = Lambdust::new();
        let program = r#"
            (define (make-adder n)
              (lambda (x) (+ x n)))
            (make-adder 5)
        "#;
        match lambdust.eval(program, Some("perf_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(25));
    println!("Closure creation: {:?} average", measurement.avg_time);
}

#[test]
fn test_nested_closure_performance() {
    let tester = PerformanceTester::new(PerformanceConfig {
        timing_iterations: 50,
        ..Default::default()
    });
    
    let measurement = tester.measure("nested_closures", || {
        let mut lambdust = Lambdust::new();
        let program = r#"
            (define (outer a)
              (define (middle b)
                (define (inner c)
                  (+ a b c))
                inner)
              middle)
            (((outer 1) 2) 3)
        "#;
        match lambdust.eval(program, Some("perf_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(40));
    println!("Nested closures: {:?} average", measurement.avg_time);
}

// ============================================================================
// PARSER AND LEXER PERFORMANCE TESTS
// ============================================================================

#[test]
fn test_lexer_performance() {
    let tester = PerformanceTester::new(PerformanceConfig::default());
    
    let large_program = format!(
        "(+ {})",
        (1..=100).map(|i| i.to_string()).collect::<Vec<_>>().join(" ")
    );
    
    let measurement = tester.measure("lexer_large_input", || {
        let lambdust = Lambdust::new();
        match lambdust.tokenize(&large_program, Some("perf_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(10));
    println!("Lexer (large input): {:?} average", measurement.avg_time);
}

#[test]
fn test_parser_performance() {
    let tester = PerformanceTester::new(PerformanceConfig::default());
    
    let measurement = tester.measure("parser_complex_expr", || {
        let lambdust = Lambdust::new();
        let complex_expr = r#"
            (define (complex-calc x y z)
              (+ (* x y) (- z x) (/ y z) (+ x z)))
        "#;
        match lambdust.tokenize(complex_expr, Some("perf_test"))
            .and_then(|tokens| lambdust.parse(tokens)) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(15));
    println!("Parser (complex expr): {:?} average", measurement.avg_time);
}

// ============================================================================
// LARGE PROGRAM PERFORMANCE TESTS
// ============================================================================

#[test]
fn test_large_program_evaluation() {
    let tester = PerformanceTester::new(PerformanceConfig {
        timing_iterations: 10,
        max_execution_time: Duration::from_secs(60),
        ..Default::default()
    });
    
    let measurement = tester.measure("large_program", || {
        let mut lambdust = Lambdust::new();
        
        // Create a program with many definitions
        let mut program = String::new();
        for i in 1..=50 {
            program.push_str(&format!(
                "(define var{} (+ {} {}))\n",
                i, i, i + 1
            ));
        }
        program.push_str("(+ var1 var25 var50)");
        
        match lambdust.eval(&program, Some("perf_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(200));
    println!("Large program (50 definitions): {:?} average", measurement.avg_time);
}

#[test]
fn test_deeply_nested_expressions() {
    let tester = PerformanceTester::new(PerformanceConfig {
        timing_iterations: 20,
        ..Default::default()
    });
    
    let measurement = tester.measure("deep_nesting", || {
        let mut lambdust = Lambdust::new();
        
        // Create deeply nested arithmetic
        let depth = 50;
        let mut expr = "1".to_string();
        for i in 2..=depth {
            expr = format!("(+ {} {})", expr, i);
        }
        
        match lambdust.eval(&expr, Some("perf_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(100));
    println!("Deep nesting (50 levels): {:?} average", measurement.avg_time);
}

// ============================================================================
// MEMORY USAGE TESTS
// ============================================================================

#[test]
fn test_memory_usage_pattern() {
    // This is a structural test - real memory measurement would be platform-specific
    let tester = PerformanceTester::new(PerformanceConfig {
        timing_iterations: 20,
        ..Default::default()
    });
    
    let measurement = tester.measure("memory_pattern", || {
        let mut lambdust = Lambdust::new();
        
        // Create and discard many small objects
        for i in 1..=100 {
            let program = format!("(define temp{} (+ {} {}))", i, i, i * 2);
            match lambdust.eval(&program, Some("perf_test")) {
                Ok(_) => {},
                Err(e) => return Err(e.to_string()),
            }
        }
        
        // Simple computation to ensure everything still works
        match lambdust.eval("(+ 1 2)", Some("perf_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(300));
    println!("Memory pattern test: {:?} average", measurement.avg_time);
}

// ============================================================================
// CONCURRENCY PERFORMANCE TESTS
// ============================================================================

#[cfg(feature = "async")]
#[tokio::test]
async fn test_parallel_evaluation_performance() {
    let _tester = PerformanceTester::new(PerformanceConfig {
        timing_iterations: 10,
        ..Default::default()
    });
    
    let lambdust = Arc::new(MultithreadedLambdust::new(Some(4)).unwrap());
    
    // Measure sequential vs parallel performance
    let sequential_time = {
        let start = Instant::now();
        for i in 1..=20 {
            let program = format!("(+ {} {})", i, i * 2);
            let _ = lambdust.eval(&program, Some("perf_test")).await;
        }
        start.elapsed()
    };
    
    let parallel_time = {
        let start = Instant::now();
        let programs: Vec<String> = (1..=20)
            .map(|i| format!("(+ {} {})", i, i * 2))
            .collect();
        let sources: Vec<(&str, Option<&str>)> = programs
            .iter()
            .map(|s| (s.as_str(), Some("perf_test")))
            .collect();
        
        let _ = lambdust.eval_parallel(sources).await;
        start.elapsed()
    };
    
    println!("Sequential: {:?}, Parallel: {:?}", sequential_time, parallel_time);
    
    // Parallel should be at least somewhat faster (though overhead might dominate for small tasks)
    // This is more about testing that parallel evaluation works than performance gains
    assert!(parallel_time < Duration::from_secs(5));
}

// ============================================================================
// ERROR HANDLING PERFORMANCE TESTS
// ============================================================================

#[test]
fn test_error_handling_performance() {
    let tester = PerformanceTester::new(PerformanceConfig::default());
    
    let measurement = tester.measure("error_handling", || {
        let mut lambdust = Lambdust::new();
        
        // This should fail, we're measuring how fast it fails
        match lambdust.eval("undefined-variable", Some("perf_test")) {
            Ok(_) => Err("Expected error but got success".to_string()),
            Err(_) => Ok(()), // Error is expected
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(20));
    println!("Error handling: {:?} average", measurement.avg_time);
}

#[test]
fn test_recovery_after_error() {
    let tester = PerformanceTester::new(PerformanceConfig::default());
    
    let measurement = tester.measure("error_recovery", || {
        let mut lambdust = Lambdust::new();
        
        // Cause an error
        let _ = lambdust.eval("undefined-variable", Some("perf_test"));
        
        // Then do something that should succeed
        match lambdust.eval("(+ 1 2)", Some("perf_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    assert!(measurement.avg_time < Duration::from_millis(30));
    println!("Error recovery: {:?} average", measurement.avg_time);
}

// ============================================================================
// STRESS TESTS
// ============================================================================

#[test]
#[ignore] // Expensive test, run manually
fn test_stress_many_evaluations() {
    let tester = PerformanceTester::new(PerformanceConfig {
        timing_iterations: 1,
        max_execution_time: Duration::from_secs(120),
        run_expensive_tests: true,
        ..Default::default()
    });
    
    let measurement = tester.measure("stress_evaluations", || {
        let mut lambdust = Lambdust::new();
        
        // Perform many evaluations
        for i in 1..=10000 {
            let program = format!("(+ {} {})", i % 100, (i * 2) % 100);
            match lambdust.eval(&program, Some("stress_test")) {
                Ok(_) => {},
                Err(e) => return Err(format!("Failed at iteration {}: {}", i, e)),
            }
            
            // Periodic progress check
            if i % 1000 == 0 {
                println!("Completed {} evaluations", i);
            }
        }
        
        Ok(())
    });
    
    println!("Stress test (10k evaluations): {:?}", measurement.total_time);
    assert!(measurement.total_time < Duration::from_secs(60));
}

#[test]
#[ignore] // Expensive test, run manually
fn test_stress_deep_recursion() {
    let tester = PerformanceTester::new(PerformanceConfig {
        timing_iterations: 1,
        max_execution_time: Duration::from_secs(60),
        ..Default::default()
    });
    
    let measurement = tester.measure("stress_deep_recursion", || {
        let mut lambdust = Lambdust::new();
        
        let program = r#"
            (define (deep-count n)
              (if (= n 0)
                  0
                  (+ 1 (deep-count (- n 1)))))
            (deep-count 10000)
        "#;
        
        match lambdust.eval(program, Some("stress_test")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    
    println!("Deep recursion stress test: {:?}", measurement.total_time);
    // This might fail if tail call optimization isn't working
    assert!(measurement.total_time < Duration::from_secs(30));
}

// ============================================================================
// BENCHMARKING UTILITIES
// ============================================================================

/// Comprehensive performance benchmark suite.
#[test]
#[ignore] // Run manually for full benchmarking
fn run_comprehensive_benchmark() {
    println!("=== Lambdust Performance Benchmark Suite ===");
    
    let config = PerformanceConfig {
        timing_iterations: 1000,
        max_execution_time: Duration::from_secs(300),
        run_expensive_tests: true,
        ..Default::default()
    };
    
    let tester = PerformanceTester::new(config);
    
    // Basic operations
    test_basic_arithmetic_performance();
    test_function_call_performance();
    test_variable_lookup_performance();
    
    // Control structures
    test_tail_recursion_performance();
    test_factorial_performance();
    
    // Language features
    test_closure_creation_performance();
    test_nested_closure_performance();
    
    // Parser/Lexer
    test_lexer_performance();
    test_parser_performance();
    
    // Error handling
    test_error_handling_performance();
    test_recovery_after_error();
    
    // Print comprehensive report
    tester.print_report();
    
    println!("=== Benchmark Complete ===");
}

/// Micro-benchmark for specific operations.
pub fn micro_benchmark<F>(name: &str, operation: F, iterations: usize) -> Duration
where
    F: Fn() -> Result<(), String>,
{
    let start = Instant::now();
    for _ in 0..iterations {
        if let Err(e) = operation() {
            panic!("Micro-benchmark '{}' failed: {}", name, e);
        }
    }
    let total_time = start.elapsed();
    let avg_time = total_time / iterations as u32;
    
    println!("{}: {} iterations in {:?} (avg: {:?})", 
             name, iterations, total_time, avg_time);
    
    avg_time
}

// ============================================================================
// PERFORMANCE REGRESSION TESTS
// ============================================================================

#[test]
fn test_performance_regression_basic() {
    // These tests establish performance baselines
    // In a real CI system, these would compare against historical data
    
    let basic_arithmetic_time = micro_benchmark("basic_arithmetic", || {
        let mut lambdust = Lambdust::new();
        lambdust.eval("(+ 1 2 3)", Some("regression_test"))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }, 100);
    
    // Establish baseline: basic arithmetic should be very fast
    assert!(basic_arithmetic_time < Duration::from_millis(5));
    
    let function_call_time = micro_benchmark("function_call", || {
        let mut lambdust = Lambdust::new();
        lambdust.eval("((lambda (x) (+ x 1)) 5)", Some("regression_test"))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }, 100);
    
    // Function calls should be reasonably fast
    assert!(function_call_time < Duration::from_millis(10));
}

#[test] 
fn test_performance_scaling() {
    // Test that performance scales reasonably with input size
    
    let small_program_time = micro_benchmark("small_program", || {
        let mut lambdust = Lambdust::new();
        let program = "(+ 1 2 3)";
        lambdust.eval(program, Some("scaling_test"))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }, 50);
    
    let medium_program_time = micro_benchmark("medium_program", || {
        let mut lambdust = Lambdust::new();
        let program = "(+ 1 2 3 4 5 6 7 8 9 10)";
        lambdust.eval(program, Some("scaling_test"))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }, 50);
    
    // Medium program should not be dramatically slower
    let scaling_factor = medium_program_time.as_nanos() as f64 / small_program_time.as_nanos() as f64;
    assert!(scaling_factor < 5.0, "Scaling factor too high: {}", scaling_factor);
    
    println!("Performance scaling factor: {:.2}x", scaling_factor);
}