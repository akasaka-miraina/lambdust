//! Value Optimization Performance Verification Runner
//!
//! This binary runs comprehensive performance verification for the Value enum
//! optimizations and generates detailed reports on the effectiveness of the
//! optimization strategy.
//!
//! Usage:
//!   cargo run --bin value_optimization_verifier
//!   cargo run --bin value_optimization_verifier -- --config production
//!   cargo run --bin value_optimization_verifier -- --report-format json

use lambdust::eval::comprehensive_performance_verification::{
    PerformanceVerificationSuite, VerificationConfig
};
use lambdust::eval::arc_allocation_tracker::{
    enable_global_tracking, disable_global_tracking, reset_global_tracking
};
use std::time::Duration;
use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Value Optimization Performance Verifier");
    println!("═══════════════════════════════════════════\n");

    let args: Vec<String> = env::args().collect();
    let config = parse_config(&args)?;
    let report_format = parse_report_format(&args);

    // Enable global Arc tracking
    enable_global_tracking();
    reset_global_tracking();

    println!("Configuration:");
    println!("  Memory Tracking: {}", config.enable_memory_tracking);
    println!("  Arc Tracking: {}", config.enable_arc_tracking);
    println!("  Semantic Verification: {}", config.enable_semantic_verification);
    println!("  Cache Analysis: {}", config.enable_cache_analysis);
    println!("  Production Assessment: {}", config.enable_production_assessment);
    println!("  Benchmark Iterations: {}", config.benchmark_iterations);
    println!("  Report Format: {report_format:?}");
    println!();

    // Create and run verification suite
    let suite = PerformanceVerificationSuite::new(config);
    let report = suite.run_comprehensive_verification();

    // Generate and save reports
    match report_format {
        ReportFormat::Text => {
            let report_text = report.generate_detailed_report();
            println!("{report_text}");
            
            // Save to file
            let output_path = "value_optimization_report.txt";
            fs::write(output_path, &report_text)?;
            println!("\n📁 Detailed report saved to: {output_path}");
        }
        
        ReportFormat::Json => {
            let json_report = create_json_report(&report)?;
            println!("{}", serde_json::to_string_pretty(&json_report)?);
            
            // Save to file
            let output_path = "value_optimization_report.json";
            fs::write(output_path, serde_json::to_string_pretty(&json_report)?)?;
            println!("\n📁 JSON report saved to: {output_path}");
        }
        
        ReportFormat::Both => {
            // Text report
            let report_text = report.generate_detailed_report();
            println!("{report_text}");
            fs::write("value_optimization_report.txt", &report_text)?;
            
            // JSON report
            let json_report = create_json_report(&report)?;
            fs::write("value_optimization_report.json", serde_json::to_string_pretty(&json_report)?)?;
            
            println!("\n📁 Reports saved to:");
            println!("  - value_optimization_report.txt");
            println!("  - value_optimization_report.json");
        }
    }

    // Print summary and recommendations
    print_summary(&report);
    print_recommendations(&report);

    // Cleanup
    disable_global_tracking();

    Ok(())
}

#[derive(Debug)]
enum ReportFormat {
    Text,
    Json,
    Both,
}

fn parse_config(args: &[String]) -> Result<VerificationConfig, Box<dyn std::error::Error>> {
    let mut config = VerificationConfig::default();

    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--config" => {
                if let Some(config_type) = args.get(i + 1) {
                    match config_type.as_str() {
                        "fast" => {
                            config.benchmark_iterations = 1000;
                            config.enable_cache_analysis = false;
                            config.enable_production_assessment = false;
                            config.test_timeout = Duration::from_secs(30);
                        }
                        "standard" => {
                            // Already default
                        }
                        "production" => {
                            config.benchmark_iterations = 50000;
                            config.enable_cache_analysis = true;
                            config.enable_production_assessment = true;
                            config.test_timeout = Duration::from_secs(300);
                        }
                        "comprehensive" => {
                            config.benchmark_iterations = 100000;
                            config.enable_cache_analysis = true;
                            config.enable_production_assessment = true;
                            config.test_timeout = Duration::from_secs(600);
                        }
                        _ => {
                            return Err(format!("Unknown config type: {config_type}").into());
                        }
                    }
                }
            }
            "--iterations" => {
                if let Some(iterations_str) = args.get(i + 1) {
                    config.benchmark_iterations = iterations_str.parse()?;
                }
            }
            "--timeout" => {
                if let Some(timeout_str) = args.get(i + 1) {
                    let seconds: u64 = timeout_str.parse()?;
                    config.test_timeout = Duration::from_secs(seconds);
                }
            }
            "--no-memory" => {
                config.enable_memory_tracking = false;
            }
            "--no-arc" => {
                config.enable_arc_tracking = false;
            }
            "--no-semantic" => {
                config.enable_semantic_verification = false;
            }
            "--no-cache" => {
                config.enable_cache_analysis = false;
            }
            "--no-production" => {
                config.enable_production_assessment = false;
            }
            _ => {}
        }
    }

    Ok(config)
}

fn parse_report_format(args: &[String]) -> ReportFormat {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--report-format" {
            if let Some(format_str) = args.get(i + 1) {
                match format_str.as_str() {
                    "text" => return ReportFormat::Text,
                    "json" => return ReportFormat::Json,
                    "both" => return ReportFormat::Both,
                    _ => {}
                }
            }
        }
    }
    ReportFormat::Text
}

fn create_json_report(report: &lambdust::eval::comprehensive_performance_verification::ComprehensiveVerificationReport) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use serde_json::json;
    
    let timestamp = report.timestamp
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    Ok(json!({
        "timestamp": timestamp,
        "total_execution_time_seconds": report.total_execution_time.as_secs_f64(),
        "memory_analysis": {
            "total_memory_saved_bytes": report.memory_analysis.total_memory_saved,
            "average_savings_percentage": report.memory_analysis.average_savings_percentage,
            "total_values_tested": report.memory_analysis.total_values_tested,
            "immediate_value_results": report.memory_analysis.immediate_value_results.len(),
            "compound_value_results": report.memory_analysis.compound_value_results.len(),
            "complex_value_results": report.memory_analysis.complex_value_results.len()
        },
        "arc_analysis": {
            "total_arc_usage": report.arc_analysis.total_arc_usage,
            "arc_reduction_percentage": report.arc_analysis.arc_reduction_percentage,
            "immediate_arc_usage": report.arc_analysis.immediate_arc_usage,
            "compound_arc_usage": report.arc_analysis.compound_arc_usage,
            "complex_arc_usage": report.arc_analysis.complex_arc_usage
        },
        "performance_benchmarks": {
            "count": report.performance_benchmarks.len(),
            "benchmarks": report.performance_benchmarks.iter().map(|b| json!({
                "name": b.name,
                "improvement_percentage": b.improvement_percentage,
                "arc_reduction": b.arc_reduction,
                "semantic_compliance": b.semantic_compliance,
                "legacy_ops_per_second": b.legacy_performance.operations_per_second,
                "optimized_ops_per_second": b.optimized_performance.operations_per_second
            })).collect::<Vec<_>>()
        },
        "semantic_verification": {
            "total_tests": report.semantic_verification.total_tests,
            "passed_tests": report.semantic_verification.passed_tests,
            "failed_tests": report.semantic_verification.failed_tests,
            "compliance_percentage": report.semantic_verification.compliance_percentage
        },
        "cache_analysis": {
            "cache_hit_ratio": report.cache_analysis.cache_hit_ratio,
            "memory_locality_score": report.cache_analysis.memory_locality_score,
            "cache_line_utilization": report.cache_analysis.cache_line_utilization
        },
        "production_assessment": {
            "overall_readiness_score": report.production_assessment.overall_readiness_score,
            "max_sustained_ops_per_second": report.production_assessment.load_test_results.max_sustained_ops_per_second,
            "memory_leak_detected": report.production_assessment.memory_leak_assessment.leak_detected,
            "thread_safety_safe": report.production_assessment.thread_safety_assessment.concurrent_access_safe
        }
    }))
}

fn print_summary(report: &lambdust::eval::comprehensive_performance_verification::ComprehensiveVerificationReport) {
    println!("\n🎯 VERIFICATION SUMMARY");
    println!("══════════════════════════════════════════");
    
    // Check target achievements
    let arc_reduction_target = 90.0;
    let memory_savings_target = 50.0;
    let performance_improvement_target = 50.0;
    let semantic_compliance_target = 100.0;
    
    let arc_achieved = report.arc_analysis.arc_reduction_percentage >= arc_reduction_target;
    let memory_achieved = report.memory_analysis.average_savings_percentage >= memory_savings_target;
    let semantic_achieved = report.semantic_verification.compliance_percentage >= semantic_compliance_target;
    
    let avg_performance_improvement = if !report.performance_benchmarks.is_empty() {
        report.performance_benchmarks.iter()
            .map(|b| b.improvement_percentage)
            .sum::<f64>() / report.performance_benchmarks.len() as f64
    } else {
        0.0
    };
    let performance_achieved = avg_performance_improvement >= performance_improvement_target;
    
    println!("Target Achievement Status:");
    println!("  Arc Reduction (≥90%):      {} ({:.1}%)", 
        if arc_achieved { "✅ ACHIEVED" } else { "❌ MISSED" },
        report.arc_analysis.arc_reduction_percentage);
    println!("  Memory Savings (≥50%):     {} ({:.1}%)", 
        if memory_achieved { "✅ ACHIEVED" } else { "❌ MISSED" },
        report.memory_analysis.average_savings_percentage);
    println!("  Performance Improvement:   {} ({:.1}%)", 
        if performance_achieved { "✅ ACHIEVED" } else { "❌ MISSED" },
        avg_performance_improvement);
    println!("  Semantic Compliance:       {} ({:.1}%)", 
        if semantic_achieved { "✅ ACHIEVED" } else { "❌ MISSED" },
        report.semantic_verification.compliance_percentage);
    
    let overall_success = arc_achieved && memory_achieved && performance_achieved && semantic_achieved;
    println!("\nOverall Status: {}", 
        if overall_success { "✅ ALL TARGETS ACHIEVED" } else { "⚠️  SOME TARGETS MISSED" });
}

fn print_recommendations(report: &lambdust::eval::comprehensive_performance_verification::ComprehensiveVerificationReport) {
    println!("\n💡 RECOMMENDATIONS");
    println!("══════════════════════════════════════════");
    
    // Arc reduction recommendations
    if report.arc_analysis.arc_reduction_percentage < 90.0 {
        println!("🔗 Arc Optimization:");
        println!("  - Review compound values with high Arc usage");
        println!("  - Consider direct boxing for pairs");
        println!("  - Implement inline storage for small values");
        println!();
    }
    
    // Memory savings recommendations
    if report.memory_analysis.average_savings_percentage < 50.0 {
        println!("💾 Memory Optimization:");
        println!("  - Increase use of immediate value representations");
        println!("  - Implement value interning for common constants");
        println!("  - Review container overhead optimization");
        println!();
    }
    
    // Performance recommendations
    let avg_improvement = if !report.performance_benchmarks.is_empty() {
        report.performance_benchmarks.iter()
            .map(|b| b.improvement_percentage)
            .sum::<f64>() / report.performance_benchmarks.len() as f64
    } else {
        0.0
    };
    
    if avg_improvement < 50.0 {
        println!("🏃 Performance Optimization:");
        println!("  - Focus on hot path operation optimization");
        println!("  - Implement SIMD optimizations for numeric operations");
        println!("  - Review allocation patterns in tight loops");
        println!();
    }
    
    // Semantic compliance recommendations
    if report.semantic_verification.compliance_percentage < 100.0 {
        println!("🔍 Semantic Compliance:");
        println!("  - Review failed semantic tests");
        println!("  - Ensure R7RS specification compliance");
        println!("  - Add more comprehensive test coverage");
        println!();
    }
    
    // Production readiness recommendations
    if report.production_assessment.overall_readiness_score < 95.0 {
        println!("🏭 Production Readiness:");
        if report.production_assessment.memory_leak_assessment.leak_detected {
            println!("  - Address detected memory leaks");
        }
        if !report.production_assessment.thread_safety_assessment.concurrent_access_safe {
            println!("  - Fix thread safety issues");
        }
        if report.production_assessment.load_test_results.error_rate_under_load > 0.01 {
            println!("  - Reduce error rate under load");
        }
        println!("  - Run extended stability testing");
        println!();
    }
    
    // General recommendations
    println!("🎯 Next Steps:");
    println!("  1. Prioritize fixes for failed targets");
    println!("  2. Run verification suite regularly during development");
    println!("  3. Monitor production metrics after deployment");
    println!("  4. Consider A/B testing for performance validation");
}

fn print_usage() {
    println!("Usage: value_optimization_verifier [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  --config <TYPE>        Configuration preset (fast|standard|production|comprehensive)");
    println!("  --iterations <N>       Number of benchmark iterations");
    println!("  --timeout <SECONDS>    Test timeout in seconds");
    println!("  --report-format <FMT>  Report format (text|json|both)");
    println!("  --no-memory           Disable memory tracking");
    println!("  --no-arc             Disable Arc tracking");
    println!("  --no-semantic        Disable semantic verification");
    println!("  --no-cache           Disable cache analysis");
    println!("  --no-production      Disable production assessment");
    println!("  --help               Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("  # Quick verification");
    println!("  cargo run --bin value_optimization_verifier -- --config fast");
    println!();
    println!("  # Full production verification");
    println!("  cargo run --bin value_optimization_verifier -- --config production");
    println!();
    println!("  # Custom configuration");
    println!("  cargo run --bin value_optimization_verifier -- --iterations 25000 --report-format both");
}