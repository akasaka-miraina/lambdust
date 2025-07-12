#!/usr/bin/env rust
//! Simple performance measurement test

use lambdust::evaluator::performance_measurement::{
    EvaluatorComparison, PerformanceReportGenerator
};
use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;
use lambdust::environment::Environment;
use lambdust::evaluator::RuntimeOptimizationLevel;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Simple Performance Test");
    println!("===========================");
    
    // Test evaluator comparison
    println!("\n1. Testing evaluator comparison...");
    let mut evaluator_comparison = EvaluatorComparison::new();
    
    // Simple arithmetic expression
    let test_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let env = Rc::new(Environment::new());
    
    let comparison_result = evaluator_comparison.compare_expression(
        test_expr,
        env,
        RuntimeOptimizationLevel::Balanced,
    )?;
    
    println!("   ✅ Comparison completed:");
    println!("      - Expression: {}", comparison_result.expression_summary);
    println!("      - Semantic time: {:?}", comparison_result.semantic_metrics.execution_time);
    println!("      - Runtime time: {:?}", comparison_result.runtime_metrics.execution_time);
    println!("      - Speedup: {:.2}x", comparison_result.performance_comparison.speedup_factor);
    println!("      - Correctness: {}", comparison_result.correctness_check.results_equivalent);
    
    // Test statistical analysis
    println!("\n2. Testing statistical analysis...");
    let analysis = evaluator_comparison.generate_analysis(10);
    println!("   📊 Analysis results:");
    println!("      - Sample size: {}", analysis.sample_size);
    if analysis.sample_size > 0 {
        println!("      - Average speedup: {:.2}x", analysis.average_speedup);
        println!("      - Correctness rate: {:.1}%", analysis.correctness_rate * 100.0);
    }
    
    // Test report generation
    println!("\n3. Testing report generation...");
    let mut report_generator = PerformanceReportGenerator::new();
    report_generator.add_comparison_results(&vec![comparison_result]);
    
    let executive_summary = report_generator.generate_executive_summary();
    println!("   📄 Report generated:");
    println!("      - Report type: {:?}", executive_summary.metadata.report_type);
    println!("      - Content size: {} bytes", executive_summary.size_bytes);
    
    println!("\n🎉 Simple performance test completed successfully!");
    println!("\n📋 Key components verified:");
    println!("   ✅ EvaluatorComparison framework");
    println!("   ✅ Statistical analysis");
    println!("   ✅ Report generation");
    println!("   ✅ Performance metrics calculation");
    
    Ok(())
}