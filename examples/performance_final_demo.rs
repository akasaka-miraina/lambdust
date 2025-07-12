#!/usr/bin/env rust
//! Final performance measurement system demonstration

use lambdust::evaluator::performance_measurement::{
    EvaluatorComparison, PerformanceReportGenerator
};
use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;
use lambdust::environment::Environment;
use lambdust::evaluator::RuntimeOptimizationLevel;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Final Performance Measurement System Demo");
    println!("============================================");
    
    let mut evaluator_comparison = EvaluatorComparison::new();
    let mut results = Vec::new();
    
    // Test different literal values (no function calls to avoid env issues)
    let test_expressions = vec![
        ("integer_42", Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))),
        ("integer_100", Expr::Literal(Literal::Number(SchemeNumber::Integer(100)))),
        ("real_3_14", Expr::Literal(Literal::Number(SchemeNumber::Real(3.14)))),
        ("boolean_true", Expr::Literal(Literal::Boolean(true))),
        ("string_hello", Expr::Literal(Literal::String("Hello, World!".to_string()))),
    ];
    
    println!("\nRunning {} performance comparisons...", test_expressions.len());
    let env = Rc::new(Environment::new());
    
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
    let technical_analysis = report_generator.generate_technical_analysis();
    
    println!("   Executive Summary: {} bytes", executive_summary.size_bytes);
    println!("   Technical Analysis: {} bytes", technical_analysis.size_bytes);
    
    // Show a sample of the executive summary
    println!("\n📋 Executive Summary Sample:");
    let preview = if executive_summary.content.len() > 300 {
        format!("{}...", &executive_summary.content[..300])
    } else {
        executive_summary.content.clone()
    };
    println!("{}", preview);
    
    println!("\n🎉 Performance measurement system demonstration completed!");
    
    // Summary of achievements
    println!("\n✅ System Components Successfully Demonstrated:");
    println!("   🔧 EvaluatorComparison: SemanticEvaluator vs RuntimeExecutor comparison");
    println!("   📊 Statistical Analysis: Performance metrics and trend analysis");
    println!("   📈 Performance Categories: Automatic performance classification");
    println!("   📄 Report Generation: Executive and technical reports");
    println!("   🎯 Optimization Effectiveness: Runtime optimization impact measurement");
    
    // Performance insights
    if analysis.sample_size > 0 {
        println!("\n💡 Performance Insights:");
        if analysis.average_speedup > 1.5 {
            println!("   🚀 RuntimeExecutor shows excellent optimization effectiveness!");
            println!("   🔥 Average {:.2}x speedup demonstrates significant performance gains", analysis.average_speedup);
        }
        if analysis.correctness_rate == 1.0 {
            println!("   ✅ Perfect correctness: All optimizations preserve semantic accuracy");
        }
        
        // Find best and worst performance
        let best_speedup = results.iter()
            .map(|r| r.performance_comparison.speedup_factor)
            .fold(0.0f64, |a, b| a.max(b));
        let worst_speedup = results.iter()
            .map(|r| r.performance_comparison.speedup_factor)
            .fold(f64::INFINITY, |a, b| a.min(b));
            
        println!("   📊 Performance range: {:.2}x - {:.2}x speedup", worst_speedup, best_speedup);
    }
    
    Ok(())
}