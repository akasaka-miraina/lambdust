//! Simple performance verification for Value enum optimization
//!
//! This provides a lightweight verification system to validate our optimization achievements.

use std::time::{Instant, Duration};
use std::mem;

use crate::eval::Value;
use crate::eval::optimized_value::OptimizedValue;

/// Simple performance test results
#[derive(Debug)]
pub struct SimpleTestResult {
    pub test_name: String,
    pub legacy_time: Duration,
    pub optimized_time: Duration,
    pub performance_improvement: f64,
    pub legacy_size: usize,
    pub optimized_size: usize,
    pub memory_savings: f64,
    pub passed: bool,
}

/// Simple performance verifier
pub struct SimplePerformanceVerifier;

impl SimplePerformanceVerifier {
    /// Create a new verifier
    pub fn new() -> Self {
        Self
    }
    
    /// Run basic performance verification
    pub fn run_basic_verification(&self) -> Vec<SimpleTestResult> {
        vec![
            self.test_basic_value_creation(),
            self.test_value_cloning(),
            self.test_memory_sizes(),
        ]
    }
    
    /// Test basic value creation performance
    fn test_basic_value_creation(&self) -> SimpleTestResult {
        const ITERATIONS: usize = 1000;
        
        // Test legacy Value creation
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _val = Value::number(42.0);
            std::hint::black_box(_val);
        }
        let legacy_time = start.elapsed();
        
        // Test optimized Value creation  
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _val = OptimizedValue::number(42.0);
            std::hint::black_box(_val);
        }
        let optimized_time = start.elapsed();
        
        let performance_improvement = if optimized_time.as_nanos() > 0 {
            (legacy_time.as_nanos() as f64 / optimized_time.as_nanos() as f64 - 1.0) * 100.0
        } else {
            0.0
        };
        
        SimpleTestResult {
            test_name: "Basic Value Creation".to_string(),
            legacy_time,
            optimized_time,
            performance_improvement,
            legacy_size: mem::size_of::<Value>(),
            optimized_size: mem::size_of::<OptimizedValue>(),
            memory_savings: 0.0, // Calculate based on Arc usage
            passed: performance_improvement >= 0.0, // Any improvement is good
        }
    }
    
    /// Test value cloning performance
    fn test_value_cloning(&self) -> SimpleTestResult {
        const ITERATIONS: usize = 1000;
        
        let legacy_val = Value::number(42.0);
        let optimized_val = OptimizedValue::number(42.0);
        
        // Test legacy Value cloning
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _val = legacy_val.clone();
            std::hint::black_box(_val);
        }
        let legacy_time = start.elapsed();
        
        // Test optimized Value cloning
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _val = optimized_val.clone();
            std::hint::black_box(_val);
        }
        let optimized_time = start.elapsed();
        
        let performance_improvement = if optimized_time.as_nanos() > 0 {
            (legacy_time.as_nanos() as f64 / optimized_time.as_nanos() as f64 - 1.0) * 100.0
        } else {
            0.0
        };
        
        SimpleTestResult {
            test_name: "Value Cloning".to_string(),
            legacy_time,
            optimized_time,
            performance_improvement,
            legacy_size: mem::size_of::<Value>(),
            optimized_size: mem::size_of::<OptimizedValue>(),
            memory_savings: 0.0,
            passed: performance_improvement >= 0.0,
        }
    }
    
    /// Test memory sizes
    fn test_memory_sizes(&self) -> SimpleTestResult {
        let legacy_size = mem::size_of::<Value>();
        let optimized_size = mem::size_of::<OptimizedValue>();
        
        let memory_savings = if legacy_size > 0 {
            ((legacy_size - optimized_size) as f64 / legacy_size as f64) * 100.0
        } else {
            0.0
        };
        
        SimpleTestResult {
            test_name: "Memory Size Comparison".to_string(),
            legacy_time: Duration::new(0, 0),
            optimized_time: Duration::new(0, 0),
            performance_improvement: 0.0,
            legacy_size,
            optimized_size,
            memory_savings,
            passed: memory_savings >= 10.0, // Expect at least 10% memory savings
        }
    }
    
    /// Print results in a human-readable format
    pub fn print_results(&self, results: &[SimpleTestResult]) {
        println!("🎯 Value Optimization Performance Results");
        println!("========================================");
        println!();
        
        let mut all_passed = true;
        
        for result in results {
            let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
            all_passed &= result.passed;
            
            println!("{} {}", status, result.test_name);
            
            if result.legacy_time.as_nanos() > 0 {
                println!("  Legacy Time:    {:?}", result.legacy_time);
                println!("  Optimized Time: {:?}", result.optimized_time);
                println!("  Performance:    {:.1}% improvement", result.performance_improvement);
            }
            
            if result.legacy_size > 0 {
                println!("  Legacy Size:    {} bytes", result.legacy_size);
                println!("  Optimized Size: {} bytes", result.optimized_size);
                println!("  Memory Savings: {:.1}%", result.memory_savings);
            }
            
            println!();
        }
        
        println!("Overall Status: {}", if all_passed { "✅ ALL TESTS PASSED" } else { "❌ SOME TESTS FAILED" });
        
        // Summary
        let performance_improvements: Vec<f64> = results.iter()
            .filter(|r| r.performance_improvement > 0.0)
            .map(|r| r.performance_improvement)
            .collect();
            
        if !performance_improvements.is_empty() {
            let avg_improvement = performance_improvements.iter().sum::<f64>() / performance_improvements.len() as f64;
            println!("Average Performance Improvement: {:.1}%", avg_improvement);
        }
        
        let memory_savings: Vec<f64> = results.iter()
            .filter(|r| r.memory_savings > 0.0)
            .map(|r| r.memory_savings)
            .collect();
            
        if !memory_savings.is_empty() {
            let avg_savings = memory_savings.iter().sum::<f64>() / memory_savings.len() as f64;
            println!("Average Memory Savings: {:.1}%", avg_savings);
        }
    }
}

/// Simple CLI function to run verification
pub fn run_simple_verification() {
    println!("Starting simple Value optimization verification...");
    
    let verifier = SimplePerformanceVerifier::new();
    let results = verifier.run_basic_verification();
    verifier.print_results(&results);
    
    // Check targets
    let performance_ok = results.iter().any(|r| r.performance_improvement >= 50.0);
    let memory_ok = results.iter().any(|r| r.memory_savings >= 50.0);
    
    println!("\n🎯 Target Achievement:");
    println!("  50%+ Performance: {}", if performance_ok { "✅ ACHIEVED" } else { "❌ NOT YET" });
    println!("  50%+ Memory:      {}", if memory_ok { "✅ ACHIEVED" } else { "❌ NOT YET" });
}