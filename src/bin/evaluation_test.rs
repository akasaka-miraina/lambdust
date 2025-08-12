#!/usr/bin/env rust

//! Lambdust Implementation Quality Evaluation Tool
//! 
//! This tool runs a comprehensive test suite to evaluate the current implementation
//! quality of Lambdust and measure improvements over time.

use lambdust::Lambdust;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct TestCase {
    name: String,
    expression: String,
    expected: Option<String>,
    category: TestCategory,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TestCategory {
    BasicArithmetic,
    QuoteSystem,
    ListOperations,
    IoOperations,
    Bindings,
    HigherOrder,
    ControlStructures,
    ModuleSystem,
    AdvancedFeatures,
    ErrorHandling,
    Performance,
}

#[derive(Debug)]
struct TestResult {
    test_name: String,
    success: bool,
    output: String,
    error: Option<String>,
    duration: Duration,
    category: TestCategory,
}

#[derive(Debug)]
struct EvaluationReport {
    total_tests: usize,
    passed_tests: usize,
    failed_tests: usize,
    by_category: HashMap<TestCategory, (usize, usize)>, // (passed, total)
    results: Vec<TestResult>,
    total_duration: Duration,
}

fn create_test_suite() -> Vec<TestCase> {
    vec![
        // Basic Arithmetic Tests
        TestCase {
            name: "Addition".to_string(),
            expression: "(+ 1 2 3)".to_string(),
            expected: Some("6".to_string()),
            category: TestCategory::BasicArithmetic,
        },
        TestCase {
            name: "Subtraction".to_string(),
            expression: "(- 10 3)".to_string(),
            expected: Some("7".to_string()),
            category: TestCategory::BasicArithmetic,
        },
        TestCase {
            name: "Multiplication".to_string(),
            expression: "(* 2 3 4)".to_string(),
            expected: Some("24".to_string()),
            category: TestCategory::BasicArithmetic,
        },
        TestCase {
            name: "Division".to_string(),
            expression: "(/ 12 3)".to_string(),
            expected: Some("4".to_string()),
            category: TestCategory::BasicArithmetic,
        },
        
        // Quote System Tests
        TestCase {
            name: "Quote List".to_string(),
            expression: "'(1 2 3)".to_string(),
            expected: Some("(1 2 3)".to_string()),
            category: TestCategory::QuoteSystem,
        },
        TestCase {
            name: "Quote Symbol".to_string(),
            expression: "'hello".to_string(),
            expected: Some("hello".to_string()),
            category: TestCategory::QuoteSystem,
        },
        TestCase {
            name: "Quote Expression".to_string(),
            expression: "'(+ 1 2)".to_string(),
            expected: Some("(+ 1 2)".to_string()),
            category: TestCategory::QuoteSystem,
        },
        
        // List Operations Tests
        TestCase {
            name: "Car Operation".to_string(),
            expression: "(car '(1 2 3))".to_string(),
            expected: Some("1".to_string()),
            category: TestCategory::ListOperations,
        },
        TestCase {
            name: "Cdr Operation".to_string(),
            expression: "(cdr '(1 2 3))".to_string(),
            expected: Some("(2 3)".to_string()),
            category: TestCategory::ListOperations,
        },
        TestCase {
            name: "Cons Operation".to_string(),
            expression: "(cons 1 '(2 3))".to_string(),
            expected: Some("(1 2 3)".to_string()),
            category: TestCategory::ListOperations,
        },
        
        // I/O Operations Tests
        TestCase {
            name: "Display Output".to_string(),
            expression: "(display \"Hello World\")".to_string(),
            expected: None, // I/O operations don't return meaningful values
            category: TestCategory::IoOperations,
        },
        
        // Bindings Tests
        TestCase {
            name: "Simple Let".to_string(),
            expression: "(let ((x 1)) x)".to_string(),
            expected: Some("1".to_string()),
            category: TestCategory::Bindings,
        },
        TestCase {
            name: "Multiple Let".to_string(),
            expression: "(let ((x 1) (y 2)) (+ x y))".to_string(),
            expected: Some("3".to_string()),
            category: TestCategory::Bindings,
        },
        
        // Higher-Order Function Tests
        TestCase {
            name: "Map Function".to_string(),
            expression: "(map (lambda (x) (* x 2)) '(1 2 3))".to_string(),
            expected: Some("(2 4 6)".to_string()),
            category: TestCategory::HigherOrder,
        },
        
        // Control Structures Tests
        TestCase {
            name: "If True".to_string(),
            expression: "(if #t 'yes 'no)".to_string(),
            expected: Some("yes".to_string()),
            category: TestCategory::ControlStructures,
        },
        TestCase {
            name: "If False".to_string(),
            expression: "(if #f 'yes 'no)".to_string(),
            expected: Some("no".to_string()),
            category: TestCategory::ControlStructures,
        },
        
        // Advanced Features Tests
        TestCase {
            name: "Let* Sequential".to_string(),
            expression: "(let* ((x 1) (y (+ x 1))) y)".to_string(),
            expected: Some("2".to_string()),
            category: TestCategory::AdvancedFeatures,
        },
        
        // Performance Tests
        TestCase {
            name: "List Length".to_string(),
            expression: "(length '(1 2 3 4 5 6 7 8 9 10))".to_string(),
            expected: Some("10".to_string()),
            category: TestCategory::Performance,
        },
    ]
}

fn run_test(lambdust: &mut Lambdust, test: &TestCase) -> TestResult {
    let start_time = Instant::now();
    
    match lambdust.eval(&test.expression, Some("test")) {
        Ok(value) => {
            let duration = start_time.elapsed();
            let output = format!("{}", value);
            let success = if let Some(expected) = &test.expected {
                output.trim() == expected.trim()
            } else {
                true // I/O operations are considered successful if they don't error
            };
            
            TestResult {
                test_name: test.name.clone(),
                success,
                output,
                error: None,
                duration,
                category: test.category.clone(),
            }
        }
        Err(error) => {
            let duration = start_time.elapsed();
            TestResult {
                test_name: test.name.clone(),
                success: false,
                output: String::new(),
                error: Some(format!("{}", error)),
                duration,
                category: test.category.clone(),
            }
        }
    }
}

fn run_evaluation_suite() -> EvaluationReport {
    let mut lambdust = Lambdust::new();
    let test_suite = create_test_suite();
    let total_tests = test_suite.len();
    let mut results = Vec::new();
    let mut by_category: HashMap<TestCategory, (usize, usize)> = HashMap::new();
    
    let suite_start = Instant::now();
    
    for test in &test_suite {
        println!("Running test: {}", test.name);
        let result = run_test(&mut lambdust, test);
        
        // Update category statistics
        let (passed, total) = by_category.entry(test.category.clone()).or_insert((0, 0));
        *total += 1;
        if result.success {
            *passed += 1;
        }
        
        results.push(result);
    }
    
    let total_duration = suite_start.elapsed();
    let passed_tests = results.iter().filter(|r| r.success).count();
    let failed_tests = total_tests - passed_tests;
    
    EvaluationReport {
        total_tests,
        passed_tests,
        failed_tests,
        by_category,
        results,
        total_duration,
    }
}

fn print_report(report: &EvaluationReport) {
    println!("\n{}", "=".repeat(60));
    println!("Lambdust Implementation Quality Evaluation Report");
    println!("{}", "=".repeat(60));
    
    // Overall Summary
    println!("\n📊 Overall Summary:");
    println!("  Total Tests: {}", report.total_tests);
    println!("  Passed: {} ({:.1}%)", report.passed_tests, 
             (report.passed_tests as f64 / report.total_tests as f64) * 100.0);
    println!("  Failed: {} ({:.1}%)", report.failed_tests,
             (report.failed_tests as f64 / report.total_tests as f64) * 100.0);
    println!("  Total Duration: {:?}", report.total_duration);
    
    // Category Breakdown
    println!("\n📈 Results by Category:");
    for (category, (passed, total)) in &report.by_category {
        let percentage = (*passed as f64 / *total as f64) * 100.0;
        let status = if percentage == 100.0 { "✅" } 
                    else if percentage >= 50.0 { "⚠️" } 
                    else { "❌" };
        
        println!("  {} {:?}: {}/{} ({:.1}%)", 
                status, category, passed, total, percentage);
    }
    
    // Detailed Results
    println!("\n🔍 Detailed Test Results:");
    for result in &report.results {
        let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
        println!("  {} {}: {} ({:?})", 
                status, result.test_name, 
                if result.success { &result.output } else { "ERROR" },
                result.duration);
        
        if let Some(error) = &result.error {
            println!("    Error: {}", error);
        }
    }
    
    // Recommendations
    println!("\n💡 Recommendations:");
    
    let arithmetic_stats = report.by_category.get(&TestCategory::BasicArithmetic);
    let quote_stats = report.by_category.get(&TestCategory::QuoteSystem);
    let list_stats = report.by_category.get(&TestCategory::ListOperations);
    let binding_stats = report.by_category.get(&TestCategory::Bindings);
    
    if let Some((passed, total)) = arithmetic_stats {
        if *passed < *total {
            println!("  🔧 Priority: Fix basic arithmetic operations");
        }
    }
    
    if let Some((passed, total)) = quote_stats {
        if *passed < *total {
            println!("  🔧 Priority: Improve quote/quasiquote system");
        }
    }
    
    if let Some((passed, total)) = list_stats {
        if *passed < *total {
            println!("  🔧 Priority: Fix car/cdr/cons operations");
        }
    }
    
    if let Some((passed, total)) = binding_stats {
        if *passed < *total {
            println!("  🔧 Priority: Implement let/let*/letrec bindings");
        }
    }
    
    if report.passed_tests == 0 {
        println!("  🚨 Critical: Core evaluation system needs implementation");
        println!("  📝 Start with: Basic value representation and environment");
    } else if report.passed_tests < report.total_tests / 2 {
        println!("  ⚠️  Warning: Less than 50% tests passing - focus on core functionality");
    } else if report.passed_tests == report.total_tests {
        println!("  🎉 Excellent: All tests passing! Ready for advanced features");
    } else {
        println!("  👍 Good progress: Focus on failing tests to reach 100%");
    }
}

fn main() {
    println!("Starting Lambdust Implementation Quality Evaluation...\n");
    
    let report = run_evaluation_suite();
    print_report(&report);
    
    println!("\n{}", "=".repeat(60));
    println!("Evaluation complete!");
}