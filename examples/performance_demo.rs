#!/usr/bin/env rust
//! Performance measurement system demonstration

use lambdust::evaluator::performance_measurement::{
    PerformanceMeasurementSystem, PracticalBenchmarkSuite, EvaluatorComparison,
    RegressionDetector, PerformanceReportGenerator
};
use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;
use lambdust::environment::Environment;
use lambdust::evaluator::RuntimeOptimizationLevel;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Performance Measurement System Demo");
    println!("=====================================");
    
    // 1. Initialize performance measurement system
    println!("\n1. Initializing performance measurement system...");
    let mut perf_system = PerformanceMeasurementSystem::new();
    perf_system.initialize()?;
    println!("   ✅ System initialized successfully");
    
    // 2. Run practical benchmarks
    println!("\n2. Running practical benchmark suite...");
    let mut benchmark_suite = PracticalBenchmarkSuite::new();
    
    // Run all benchmarks
    let benchmark_results = benchmark_suite.run_all_benchmarks()?;
    println!("   📊 Benchmark suite results:");
    println!("      - Total benchmarks: {}", benchmark_results.benchmark_results.len());
    println!("      - Average speedup: {:.2}x", benchmark_results.overall_stats.average_speedup);
    println!("      - Best speedup: {:.2}x", benchmark_results.overall_stats.best_speedup);
    println!("      - Improved count: {}", benchmark_results.overall_stats.improved_count);
    
    // 3. Run evaluator comparison
    println!("\n3. Running evaluator comparison...");
    let mut evaluator_comparison = EvaluatorComparison::new();
    
    // Compare simple expressions
    let test_expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(20))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(30))),
    ]);
    
    let env = Rc::new(Environment::new());
    let comparison_result = evaluator_comparison.compare_expression(
        test_expr,
        env,
        RuntimeOptimizationLevel::Balanced,
    )?;
    
    println!("   🔍 Expression comparison:");
    println!("      - Expression: {}", comparison_result.expression_summary);
    println!("      - Semantic time: {:?}", comparison_result.semantic_metrics.execution_time);
    println!("      - Runtime time: {:?}", comparison_result.runtime_metrics.execution_time);
    println!("      - Speedup: {:.2}x", comparison_result.performance_comparison.speedup_factor);
    println!("      - Correctness: {}", comparison_result.correctness_check.results_equivalent);
    
    // 4. Generate analysis
    println!("\n4. Generating statistical analysis...");
    let analysis = evaluator_comparison.generate_analysis(10);
    println!("   📈 Analysis results:");
    println!("      - Sample size: {}", analysis.sample_size);
    if analysis.sample_size > 0 {
        println!("      - Average speedup: {:.2}x", analysis.average_speedup);
        println!("      - Correctness rate: {:.1}%", analysis.correctness_rate * 100.0);
    }
    
    // 5. Initialize regression detector
    println!("\n5. Performance regression detection...");
    let mut regression_detector = RegressionDetector::new();
    println!("   🚨 Regression detector initialized");
    println!("      - Ready to monitor performance trends");
    println!("      - Alert thresholds configured");
    
    // 6. Generate performance report
    println!("\n6. Generating performance report...");
    let mut report_generator = PerformanceReportGenerator::new();
    report_generator.add_comparison_results(&vec![comparison_result.clone()]);
    
    let executive_summary = report_generator.generate_executive_summary();
    println!("   📄 Executive Summary generated:");
    println!("      - Report type: {:?}", executive_summary.metadata.report_type);
    println!("      - Content size: {} bytes", executive_summary.size_bytes);
    
    // 7. System statistics
    println!("\n7. System performance statistics:");
    let stats = perf_system.get_system_stats();
    println!("   📊 Current statistics:");
    println!("      - Total measurements: {}", stats.total_measurements);
    println!("      - Total benchmark runs: {}", stats.total_benchmark_runs);
    println!("      - Total analyses: {}", stats.total_analyses);
    
    // 8. Shutdown
    println!("\n8. Shutting down system...");
    perf_system.shutdown()?;
    println!("   ✅ System shutdown complete");
    
    println!("\n🎉 Performance measurement system demonstration completed successfully!");
    println!("\n💡 Key achievements:");
    println!("   • Comprehensive benchmark suite implemented");
    println!("   • SemanticEvaluator vs RuntimeExecutor comparison framework");
    println!("   • Performance regression detection system");
    println!("   • Automated report generation");
    println!("   • Statistical analysis and trend monitoring");
    
    Ok(())
}