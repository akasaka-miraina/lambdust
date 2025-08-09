//! Scheme Implementation Performance Comparison Tool
//!
//! This binary provides a command-line interface for running performance comparisons
//! between Lambdust and other popular Scheme implementations.

use std::process;
use clap::{Parser, Subcommand};
use lambdust::benchmarks::scheme_comparison::{run_scheme_comparison, ComparisonConfig, SchemeBenchmarkSuite};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "scheme-comparison")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the full performance comparison suite
    Run {
        /// Number of iterations for each benchmark
        #[arg(short, long, default_value_t = 5)]
        iterations: usize,
        
        /// Timeout for each benchmark in seconds
        #[arg(short, long, default_value_t = 30)]
        timeout: u64,
        
        /// Output directory for results
        #[arg(short, long, default_value = "benchmark_results")]
        output: String,
    },
    /// Detect available Scheme implementations
    Detect,
    /// Show configuration template
    Config,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Run { iterations, timeout, output }) => {
            run_comparison(*iterations, *timeout, output.clone());
        }
        Some(Commands::Detect) => {
            detect_implementations();
        }
        Some(Commands::Config) => {
            show_config();
        }
        None => {
            println!("Lambdust Scheme Implementation Performance Comparison");
            println!("Use --help for available commands");
        }
    }
}

fn run_comparison(iterations: usize, timeout: u64, output_dir: String) {
    println!("🚀 Starting Lambdust Performance Comparison");
    println!("   Iterations: {iterations}");
    println!("   Timeout: {timeout}s");
    println!("   Output: {output_dir}");
    println!();

    let mut config = ComparisonConfig::default();
    config.iterations = iterations;
    config.timeout_secs = timeout;
    config.output_dir = output_dir;

    match run_scheme_comparison() {
        Ok(report) => {
            println!("✅ Comparison completed successfully!");
            println!();
            
            // Print quick summary
            println!("📊 Quick Summary:");
            println!("   Total benchmarks: {}", report.summary.total_benchmarks);
            println!("   Successful: {}", report.summary.successful_benchmarks);
            println!("   Success rate: {:.1}%", 
                (report.summary.successful_benchmarks as f64 / report.summary.total_benchmarks as f64) * 100.0);
            println!();
            
            println!("🏆 Performance Ranking:");
            for (i, rank) in report.summary.performance_ranking.iter().enumerate() {
                let medal = match i {
                    0 => "🥇",
                    1 => "🥈", 
                    2 => "🥉",
                    _ => "  ",
                };
                println!("   {} {}. {} - {:.2}ms ({}x faster than baseline)",
                    medal,
                    i + 1,
                    rank.implementation,
                    rank.avg_execution_time_ns as f64 / 1_000_000.0,
                    rank.relative_performance
                );
            }
            
            println!();
            println!("📁 Detailed results saved to benchmark_results/");
        }
        Err(e) => {
            eprintln!("❌ Comparison failed: {e}");
            process::exit(1);
        }
    }
}

fn detect_implementations() {
    println!("🔍 Detecting available Scheme implementations...");
    println!();

    let mut config = ComparisonConfig::default();
    let mut suite = SchemeBenchmarkSuite::new(config.clone());
    suite.detect_implementations();
    config = suite.get_config().clone();

    let available_count = config.implementations.iter().filter(|impl_config| impl_config.available).count();
    
    println!("Found {available_count} available implementations:");
    println!();

    for impl_config in &config.implementations {
        let status_icon = if impl_config.available { "✅" } else { "❌" };
        let version = impl_config.version.as_ref()
            .map(|v| format!(" ({v})"))
            .unwrap_or_else(|| " (version unknown)".to_string());
        
        println!("   {} {} - {}{}", 
            status_icon, 
            impl_config.name,
            impl_config.command,
            if impl_config.available { version } else { " (not found)".to_string() }
        );
    }
    
    println!();
    if available_count > 1 {
        println!("🎯 Ready to run comparisons with {available_count} implementations!");
    } else if available_count == 1 {
        println!("⚠️  Only Lambdust is available. Install other Scheme implementations for comparison.");
        println!("   Recommended installations:");
        println!("   - Gauche: brew install gauche (macOS) or apt-get install gauche (Ubuntu)");
        println!("   - Chicken: brew install chicken (macOS) or apt-get install chicken-bin (Ubuntu)");
        println!("   - Racket: brew install racket (macOS) or apt-get install racket (Ubuntu)");
        println!("   - Guile: brew install guile (macOS) or apt-get install guile-3.0 (Ubuntu)");
    } else {
        println!("❌ No Scheme implementations found!");
    }
}

fn show_config() {
    println!("📋 Configuration Template");
    println!();
    
    let config = ComparisonConfig::default();
    match serde_json::to_string_pretty(&config) {
        Ok(json) => {
            println!("```json");
            println!("{json}");
            println!("```");
            println!();
            println!("Save this as 'comparison_config.json' and use with:");
            println!("cargo run --bin scheme-comparison run --config comparison_config.json");
        }
        Err(e) => {
            eprintln!("Error generating config: {e}");
            process::exit(1);
        }
    }
}