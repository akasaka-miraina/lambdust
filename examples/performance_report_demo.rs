#!/usr/bin/env rust
//! Performance report generation demonstration

use lambdust::evaluator::performance_measurement::{
    EvaluatorComparison, PerformanceReportGenerator, ReportFormat
};
use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;
use lambdust::environment::Environment;
use lambdust::evaluator::RuntimeOptimizationLevel;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Performance Report Generation Demo");
    println!("=====================================");
    
    let mut evaluator_comparison = EvaluatorComparison::new();
    let mut results = Vec::new();
    
    // Test different types of expressions
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
    ];
    
    println!("\nRunning performance comparisons...");
    let env = Rc::new(Environment::new());
    
    for (name, expr) in test_expressions {
        println!("  • Testing {}: {}", name, expr);
        let result = evaluator_comparison.compare_expression(
            expr,
            env.clone(),
            RuntimeOptimizationLevel::Balanced,
        )?;
        
        println!("    - Speedup: {:.2}x", result.performance_comparison.speedup_factor);
        println!("    - Correctness: {}", result.correctness_check.results_equivalent);
        results.push(result);
    }
    
    // Generate comprehensive report
    println!("\nGenerating performance reports...");
    let mut report_generator = PerformanceReportGenerator::new();
    report_generator.add_comparison_results(&results);
    
    // Executive Summary
    let executive_summary = report_generator.generate_executive_summary();
    println!("\n🎯 Executive Summary:");
    println!("   Report Type: {:?}", executive_summary.metadata.report_type);
    println!("   Generated At: {:?}", executive_summary.generated_at);
    println!("   Content Size: {} bytes", executive_summary.size_bytes);
    
    // Technical Analysis
    let technical_analysis = report_generator.generate_technical_analysis();
    println!("\n🔬 Technical Analysis:");
    println!("   Report Type: {:?}", technical_analysis.metadata.report_type);
    println!("   Content Size: {} bytes", technical_analysis.size_bytes);
    
    // Display executive summary content (first 500 characters)
    println!("\n📄 Executive Summary Content (excerpt):");
    let content_preview = if executive_summary.content.len() > 500 {
        format!("{}...", &executive_summary.content[..500])
    } else {
        executive_summary.content.clone()
    };
    println!("{}", content_preview);
    
    // Statistical analysis
    let analysis = evaluator_comparison.generate_analysis(results.len());
    println!("\n📈 Statistical Analysis:");
    println!("   Sample Size: {}", analysis.sample_size);
    println!("   Average Speedup: {:.2}x", analysis.average_speedup);
    println!("   Median Speedup: {:.2}x", analysis.median_speedup);
    println!("   Correctness Rate: {:.1}%", analysis.correctness_rate * 100.0);
    
    println!("\n🎉 Performance report generation demonstration completed!");
    println!("\n💡 Key achievements:");
    println!("   • Multi-expression performance comparison");
    println!("   • Automated report generation");
    println!("   • Executive and technical summary creation");
    println!("   • Statistical analysis of performance trends");
    
    Ok(())
}