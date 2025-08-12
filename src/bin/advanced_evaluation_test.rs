#!/usr/bin/env rust

//! Advanced Lambdust Implementation Quality Evaluation Tool
//! 
//! This tool runs more comprehensive tests including edge cases, 
//! R7RS compliance checks, and complex nested operations.

use lambdust::Lambdust;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct AdvancedTestCase {
    name: String,
    expression: String,
    expected: Option<String>,
    category: AdvancedTestCategory,
    r7rs_compliance: bool,
    complexity_level: ComplexityLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AdvancedTestCategory {
    R7RSCompliance,
    NestedExpressions,
    EdgeCases,
    Macros,
    Continuations,
    TailRecursion,
    NumericTower,
    StringOperations,
    ModularPrograms,
    ErrorRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ComplexityLevel {
    Basic,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug)]
struct AdvancedTestResult {
    test_name: String,
    success: bool,
    output: String,
    error: Option<String>,
    duration: Duration,
    category: AdvancedTestCategory,
    complexity: ComplexityLevel,
}

#[derive(Debug)]
struct AdvancedEvaluationReport {
    total_tests: usize,
    passed_tests: usize,
    failed_tests: usize,
    by_category: HashMap<AdvancedTestCategory, (usize, usize)>,
    by_complexity: HashMap<ComplexityLevel, (usize, usize)>,
    r7rs_compliance_score: f64,
    results: Vec<AdvancedTestResult>,
    total_duration: Duration,
}

fn create_advanced_test_suite() -> Vec<AdvancedTestCase> {
    vec![
        // R7RS Compliance Tests
        AdvancedTestCase {
            name: "R7RS Exact Numbers".to_string(),
            expression: "(exact? 5)".to_string(),
            expected: Some("#t".to_string()),
            category: AdvancedTestCategory::R7RSCompliance,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Basic,
        },
        AdvancedTestCase {
            name: "R7RS Boolean Constants".to_string(),
            expression: "(and #t #f)".to_string(),
            expected: Some("#f".to_string()),
            category: AdvancedTestCategory::R7RSCompliance,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Basic,
        },
        AdvancedTestCase {
            name: "R7RS Proper List Test".to_string(),
            expression: "(list? '(a b c))".to_string(),
            expected: Some("#t".to_string()),
            category: AdvancedTestCategory::R7RSCompliance,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Basic,
        },
        
        // Nested Expressions Tests
        AdvancedTestCase {
            name: "Deeply Nested Arithmetic".to_string(),
            expression: "(+ (* (- 10 5) (/ 8 2)) (+ 1 2 3))".to_string(),
            expected: Some("26".to_string()),
            category: AdvancedTestCategory::NestedExpressions,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Intermediate,
        },
        AdvancedTestCase {
            name: "Nested List Operations".to_string(),
            expression: "(car (cdr (cons 1 (cons 2 (cons 3 '())))))".to_string(),
            expected: Some("2".to_string()),
            category: AdvancedTestCategory::NestedExpressions,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Intermediate,
        },
        AdvancedTestCase {
            name: "Nested Lambda Application".to_string(),
            expression: "((lambda (x) ((lambda (y) (+ x y)) 5)) 3)".to_string(),
            expected: Some("8".to_string()),
            category: AdvancedTestCategory::NestedExpressions,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Advanced,
        },
        
        // Edge Cases
        AdvancedTestCase {
            name: "Empty List Properties".to_string(),
            expression: "(null? '())".to_string(),
            expected: Some("#t".to_string()),
            category: AdvancedTestCategory::EdgeCases,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Basic,
        },
        AdvancedTestCase {
            name: "Zero Division Handling".to_string(),
            expression: "(/ 1 0)".to_string(),
            expected: None, // Should error
            category: AdvancedTestCategory::EdgeCases,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Basic,
        },
        
        // Numeric Tower
        AdvancedTestCase {
            name: "Integer Operations".to_string(),
            expression: "(+ 1 2 3 4 5)".to_string(),
            expected: Some("15".to_string()),
            category: AdvancedTestCategory::NumericTower,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Basic,
        },
        AdvancedTestCase {
            name: "Mixed Numeric Types".to_string(),
            expression: "(+ 1 2.5 3)".to_string(),
            expected: Some("6.5".to_string()),
            category: AdvancedTestCategory::NumericTower,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Intermediate,
        },
        
        // String Operations
        AdvancedTestCase {
            name: "String Length".to_string(),
            expression: "(string-length \"hello\")".to_string(),
            expected: Some("5".to_string()),
            category: AdvancedTestCategory::StringOperations,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Basic,
        },
        
        // Continuations (Basic)
        AdvancedTestCase {
            name: "Simple Continuation".to_string(),
            expression: "(call/cc (lambda (k) (+ 1 (k 5))))".to_string(),
            expected: Some("5".to_string()),
            category: AdvancedTestCategory::Continuations,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Advanced,
        },
        
        // Tail Recursion Test
        AdvancedTestCase {
            name: "Factorial Tail Recursion".to_string(),
            expression: r#"
(define factorial-iter
  (lambda (n acc)
    (if (= n 0)
        acc
        (factorial-iter (- n 1) (* acc n)))))
(factorial-iter 5 1)
"#.to_string(),
            expected: Some("120".to_string()),
            category: AdvancedTestCategory::TailRecursion,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Advanced,
        },
        
        // Modular Programs
        AdvancedTestCase {
            name: "Define and Use".to_string(),
            expression: r#"
(define square (lambda (x) (* x x)))
(square 4)
"#.to_string(),
            expected: Some("16".to_string()),
            category: AdvancedTestCategory::ModularPrograms,
            r7rs_compliance: true,
            complexity_level: ComplexityLevel::Intermediate,
        },
    ]
}

fn run_advanced_test(lambdust: &mut Lambdust, test: &AdvancedTestCase) -> AdvancedTestResult {
    let start_time = Instant::now();
    
    match lambdust.eval(&test.expression, Some("advanced_test")) {
        Ok(value) => {
            let duration = start_time.elapsed();
            let output = format!("{}", value);
            let success = if let Some(expected) = &test.expected {
                // Normalize whitespace for comparison
                let normalized_output = output.trim().replace('\n', " ").replace("  ", " ");
                let normalized_expected = expected.trim().replace('\n', " ").replace("  ", " ");
                normalized_output == normalized_expected
            } else {
                true // No expected value means we just check for no error
            };
            
            AdvancedTestResult {
                test_name: test.name.clone(),
                success,
                output,
                error: None,
                duration,
                category: test.category.clone(),
                complexity: test.complexity_level.clone(),
            }
        }
        Err(error) => {
            let duration = start_time.elapsed();
            // For some edge cases, errors are expected
            let success = test.expected.is_none() && 
                         test.category == AdvancedTestCategory::EdgeCases;
            
            AdvancedTestResult {
                test_name: test.name.clone(),
                success,
                output: String::new(),
                error: Some(format!("{}", error)),
                duration,
                category: test.category.clone(),
                complexity: test.complexity_level.clone(),
            }
        }
    }
}

fn run_advanced_evaluation_suite() -> AdvancedEvaluationReport {
    let mut lambdust = Lambdust::new();
    let test_suite = create_advanced_test_suite();
    let total_tests = test_suite.len();
    let mut results = Vec::new();
    let mut by_category: HashMap<AdvancedTestCategory, (usize, usize)> = HashMap::new();
    let mut by_complexity: HashMap<ComplexityLevel, (usize, usize)> = HashMap::new();
    
    let suite_start = Instant::now();
    let mut r7rs_tests = 0;
    let mut r7rs_passed = 0;
    
    for test in &test_suite {
        println!("Running advanced test: {}", test.name);
        let result = run_advanced_test(&mut lambdust, test);
        
        // Update category statistics
        let (passed, total) = by_category.entry(test.category.clone()).or_insert((0, 0));
        *total += 1;
        if result.success {
            *passed += 1;
        }
        
        // Update complexity statistics
        let (passed, total) = by_complexity.entry(test.complexity_level.clone()).or_insert((0, 0));
        *total += 1;
        if result.success {
            *passed += 1;
        }
        
        // Track R7RS compliance
        if test.r7rs_compliance {
            r7rs_tests += 1;
            if result.success {
                r7rs_passed += 1;
            }
        }
        
        results.push(result);
    }
    
    let total_duration = suite_start.elapsed();
    let passed_tests = results.iter().filter(|r| r.success).count();
    let failed_tests = total_tests - passed_tests;
    let r7rs_compliance_score = if r7rs_tests > 0 {
        (r7rs_passed as f64 / r7rs_tests as f64) * 100.0
    } else {
        0.0
    };
    
    AdvancedEvaluationReport {
        total_tests,
        passed_tests,
        failed_tests,
        by_category,
        by_complexity,
        r7rs_compliance_score,
        results,
        total_duration,
    }
}

fn print_advanced_report(report: &AdvancedEvaluationReport) {
    println!("\n{}", "=".repeat(70));
    println!("🚀 ADVANCED Lambdust Implementation Quality Report");
    println!("{}", "=".repeat(70));
    
    // Overall Summary
    println!("\n📊 Advanced Test Results:");
    println!("  Total Tests: {}", report.total_tests);
    println!("  Passed: {} ({:.1}%)", report.passed_tests, 
             (report.passed_tests as f64 / report.total_tests as f64) * 100.0);
    println!("  Failed: {} ({:.1}%)", report.failed_tests,
             (report.failed_tests as f64 / report.total_tests as f64) * 100.0);
    println!("  R7RS Compliance: {:.1}%", report.r7rs_compliance_score);
    println!("  Total Duration: {:?}", report.total_duration);
    
    // Category Breakdown
    println!("\n📈 Results by Test Category:");
    for (category, (passed, total)) in &report.by_category {
        let percentage = (*passed as f64 / *total as f64) * 100.0;
        let status = if percentage == 100.0 { "✅" } 
                    else if percentage >= 75.0 { "🟡" } 
                    else if percentage >= 50.0 { "⚠️" } 
                    else { "❌" };
        
        println!("  {} {:?}: {}/{} ({:.1}%)", 
                status, category, passed, total, percentage);
    }
    
    // Complexity Breakdown
    println!("\n🎯 Results by Complexity Level:");
    for (complexity, (passed, total)) in &report.by_complexity {
        let percentage = (*passed as f64 / *total as f64) * 100.0;
        let status = if percentage == 100.0 { "✅" } 
                    else if percentage >= 75.0 { "🟡" } 
                    else if percentage >= 50.0 { "⚠️" } 
                    else { "❌" };
        
        println!("  {} {:?}: {}/{} ({:.1}%)", 
                status, complexity, passed, total, percentage);
    }
    
    // Detailed Results
    println!("\n🔍 Detailed Advanced Test Results:");
    for result in &report.results {
        let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
        let complexity_badge = match result.complexity {
            ComplexityLevel::Basic => "🟢",
            ComplexityLevel::Intermediate => "🟡", 
            ComplexityLevel::Advanced => "🟠",
            ComplexityLevel::Expert => "🔴",
        };
        
        println!("  {} {} {}: {} ({:?})", 
                status, complexity_badge, result.test_name, 
                if result.success { &result.output } else { "ERROR" },
                result.duration);
        
        if let Some(error) = &result.error {
            println!("    💥 Error: {}", error);
        }
    }
    
    // Implementation Quality Assessment
    println!("\n🏆 Implementation Quality Assessment:");
    
    let overall_score = (report.passed_tests as f64 / report.total_tests as f64) * 100.0;
    
    if overall_score >= 95.0 {
        println!("  🌟 EXCELLENT ({}%): Production-ready R7RS implementation", overall_score as u32);
        println!("  🎯 Next: Focus on performance optimization and advanced SRFIs");
    } else if overall_score >= 85.0 {
        println!("  🎉 VERY GOOD ({}%): Solid R7RS core with room for enhancement", overall_score as u32);
        println!("  📈 Next: Address failing advanced features and edge cases");
    } else if overall_score >= 70.0 {
        println!("  👍 GOOD ({}%): Basic R7RS functionality working well", overall_score as u32);
        println!("  🔧 Next: Implement missing core features and improve error handling");
    } else if overall_score >= 50.0 {
        println!("  ⚠️ MODERATE ({}%): Core features present but needs work", overall_score as u32);
        println!("  🛠️ Next: Focus on fundamental operations and data types");
    } else {
        println!("  🚨 NEEDS WORK ({}%): Significant implementation gaps", overall_score as u32);
        println!("  🔨 Next: Build core evaluation engine and basic primitives");
    }
    
    // R7RS Compliance Assessment
    println!("\n📋 R7RS Compliance Status:");
    if report.r7rs_compliance_score >= 95.0 {
        println!("  ✅ FULLY COMPLIANT: Ready for R7RS-large features");
    } else if report.r7rs_compliance_score >= 80.0 {
        println!("  🟡 MOSTLY COMPLIANT: Minor compliance issues to address");
    } else {
        println!("  ⚠️ PARTIAL COMPLIANCE: Focus on R7RS specification adherence");
    }
    
    // Specific Recommendations
    println!("\n💡 Specific Improvement Recommendations:");
    
    let failed_by_category: Vec<_> = report.by_category.iter()
        .filter(|(_, (passed, total))| *passed < *total)
        .collect();
    
    if failed_by_category.is_empty() {
        println!("  🎊 All test categories passing - consider adding more advanced tests!");
    } else {
        for (category, (passed, total)) in failed_by_category {
            let missing = total - passed;
            println!("  🔧 {:?}: {} failing test{}", category, missing, if missing == 1 { "" } else { "s" });
        }
    }
}

fn main() {
    println!("🚀 Starting Advanced Lambdust Implementation Quality Evaluation...\n");
    
    let report = run_advanced_evaluation_suite();
    print_advanced_report(&report);
    
    println!("\n{}", "=".repeat(70));
    println!("Advanced evaluation complete!");
}