#!/usr/bin/env rust
//! Performance measurement with built-in functions demo

use lambdust::evaluator::performance_measurement::{
    EvaluatorComparison, PerformanceReportGenerator
};
use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;
use lambdust::environment::Environment;
use lambdust::evaluator::RuntimeOptimizationLevel;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Performance Test with Built-in Functions");
    println!("===========================================");
    
    let mut evaluator_comparison = EvaluatorComparison::new();
    let mut results = Vec::new();
    
    // Test with built-in functions available
    let test_expressions = vec![
        ("simple_number", Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))),
        ("simple_addition", Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(20))),
        ])),
        ("complex_arithmetic", Expr::List(vec![
            Expr::Variable("*".to_string()),
            Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
            ]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(7))),
        ])),
        ("subtraction", Expr::List(vec![
            Expr::Variable("-".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(100))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ])),
        ("division", Expr::List(vec![
            Expr::Variable("/".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(84))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ])),
    ];
    
    println!("\nRunning {} performance comparisons with built-ins...", test_expressions.len());
    
    // Use environment with built-in functions
    let env = Rc::new(Environment::with_builtins());
    
    for (name, expr) in test_expressions {
        println!("  • Testing {}: {}", name, expr);
        let result = evaluator_comparison.compare_expression(
            expr,
            env.clone(),
            RuntimeOptimizationLevel::Balanced,
        )?;
        
        println!("    - Semantic time: {:?}", result.semantic_metrics.execution_time);
        println!("    - Runtime time: {:?}", result.runtime_metrics.execution_time);
        println!("    - Speedup: {:.2}x", result.performance_comparison.speedup_factor);
        println!("    - Correctness: {}", result.correctness_check.results_equivalent);
        println!("    - Performance category: {:?}", result.performance_comparison.performance_category);
        
        results.push(result);
    }
    
    // Statistical analysis
    println!("\n📈 Statistical Analysis:");
    let analysis = evaluator_comparison.generate_analysis(results.len());
    println!("   Sample Size: {}", analysis.sample_size);
    println!("   Average Speedup: {:.2}x", analysis.average_speedup);
    println!("   Median Speedup: {:.2}x", analysis.median_speedup);
    println!("   Standard Deviation: {:.2}", analysis.speedup_std_dev);
    println!("   Correctness Rate: {:.1}%", analysis.correctness_rate * 100.0);
    
    // Performance distribution
    println!("\n📊 Performance Distribution:");
    for (category, count) in &analysis.performance_distribution {
        println!("   {:?}: {} benchmarks", category, count);
    }
    
    // Generate reports
    println!("\n📄 Report Generation:");
    let mut report_generator = PerformanceReportGenerator::new();
    report_generator.add_comparison_results(&results);
    
    let executive_summary = report_generator.generate_executive_summary();
    
    println!("   Executive Summary: {} bytes", executive_summary.size_bytes);
    
    println!("\n🎉 Performance measurement with built-in functions completed!");
    
    // Performance insights for arithmetic operations
    if analysis.sample_size > 0 {
        println!("\n💡 Arithmetic Performance Insights:");
        if analysis.average_speedup > 1.5 {
            println!("   🚀 RuntimeExecutor shows excellent optimization for arithmetic operations!");
        }
        if analysis.correctness_rate == 1.0 {
            println!("   ✅ Perfect correctness: All arithmetic optimizations preserve semantic accuracy");
        }
        
        // Check arithmetic-specific performance
        let arithmetic_results: Vec<_> = results.iter()
            .filter(|r| r.expression_summary.contains('+') || 
                       r.expression_summary.contains('-') || 
                       r.expression_summary.contains('*') || 
                       r.expression_summary.contains('/'))
            .collect();
            
        if !arithmetic_results.is_empty() {
            let arithmetic_speedup: f64 = arithmetic_results.iter()
                .map(|r| r.performance_comparison.speedup_factor)
                .sum::<f64>() / arithmetic_results.len() as f64;
            println!("   🧮 Arithmetic operations average speedup: {:.2}x", arithmetic_speedup);
        }
    }
    
    Ok(())
}