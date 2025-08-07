//! Scheme Implementation Performance Comparison Benchmarks
//!
//! This module provides benchmarks to compare Lambdust's performance against other popular
//! Scheme implementations like Gauche, Chicken Scheme, Racket, Guile, etc.
//!
//! The benchmarks are designed to be language-agnostic and focus on core Scheme operations
//! that are common across implementations.

use std::time::Instant;
use std::process::Command;
use std::fs;
use serde::{Serialize, Deserialize};

/// Configuration for comparison benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonConfig {
    /// Scheme implementations to compare against
    pub implementations: Vec<SchemeImplementation>,
    /// Number of iterations for each benchmark
    pub iterations: usize,
    /// Timeout for each benchmark (seconds)
    pub timeout_secs: u64,
    /// Output directory for results
    pub output_dir: String,
}

/// Scheme implementation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeImplementation {
    /// Implementation name
    pub name: String,
    /// Command to run the implementation
    pub command: String,
    /// Command-line arguments
    pub args: Vec<String>,
    /// Whether this implementation is available on the system
    pub available: bool,
    /// Version information
    pub version: Option<String>,
}

/// Benchmark result for a single test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Implementation name
    pub implementation: String,
    /// Benchmark name
    pub benchmark: String,
    /// Execution time in nanoseconds
    pub execution_time_ns: u64,
    /// Memory usage in bytes (if available)
    pub memory_usage_bytes: Option<u64>,
    /// Whether the benchmark succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Complete comparison report
#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonReport {
    /// Timestamp of the comparison
    pub timestamp: String,
    /// Configuration used
    pub config: ComparisonConfig,
    /// All benchmark results
    pub results: Vec<BenchmarkResult>,
    /// Summary statistics
    pub summary: ComparisonSummary,
}

/// Summary statistics for the comparison
#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonSummary {
    /// Total benchmarks run
    pub total_benchmarks: usize,
    /// Successful benchmarks
    pub successful_benchmarks: usize,
    /// Performance ranking by implementation
    pub performance_ranking: Vec<PerformanceRank>,
}

/// Performance ranking entry
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceRank {
    /// Implementation name
    pub implementation: String,
    /// Average execution time across all benchmarks (ns)
    pub avg_execution_time_ns: u64,
    /// Relative performance (1.0 = fastest)
    pub relative_performance: f64,
    /// Number of benchmarks where this was fastest
    pub fastest_count: usize,
}

impl Default for ComparisonConfig {
    fn default() -> Self {
        Self {
            implementations: vec![
                SchemeImplementation {
                    name: "Lambdust".to_string(),
                    command: "cargo".to_string(),
                    args: vec!["run".to_string(), "--release".to_string(), "--".to_string()],
                    available: true,
                    version: Some("0.1.0".to_string()),
                },
                SchemeImplementation {
                    name: "Gauche".to_string(),
                    command: "gosh".to_string(),
                    args: vec![],
                    available: false,
                    version: None,
                },
                SchemeImplementation {
                    name: "Chicken".to_string(),
                    command: "csi".to_string(),
                    args: vec!["-q".to_string()],
                    available: false,
                    version: None,
                },
                SchemeImplementation {
                    name: "Racket".to_string(),
                    command: "racket".to_string(),
                    args: vec![],
                    available: false,
                    version: None,
                },
                SchemeImplementation {
                    name: "Guile".to_string(),
                    command: "guile".to_string(),
                    args: vec!["--no-auto-compile".to_string()],
                    available: false,
                    version: None,
                },
                SchemeImplementation {
                    name: "MIT Scheme".to_string(),
                    command: "mit-scheme".to_string(),
                    args: vec!["--batch-mode".to_string()],
                    available: false,
                    version: None,
                },
            ],
            iterations: 5,
            timeout_secs: 30,
            output_dir: "benchmark_results".to_string(),
        }
    }
}

/// Benchmark suite for Scheme comparison
pub struct SchemeBenchmarkSuite {
    config: ComparisonConfig,
}

impl SchemeBenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(config: ComparisonConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn get_config(&self) -> &ComparisonConfig {
        &self.config
    }

    /// Detect available Scheme implementations on the system
    pub fn detect_implementations(&mut self) {
        for impl_config in &mut self.config.implementations {
            if impl_config.name == "Lambdust" {
                continue; // Skip Lambdust - always available
            }

            // Try to run the implementation with --version or similar
            let result = Command::new(&impl_config.command)
                .arg("--version")
                .output();

            impl_config.available = result.is_ok();
            
            if let Ok(output) = result {
                if let Ok(version_str) = String::from_utf8(output.stdout) {
                    impl_config.version = Some(version_str.trim().to_string());
                }
            }
        }
    }

    /// Run all comparison benchmarks
    pub fn run_comparison(&self) -> ComparisonReport {
        let mut results = Vec::new();
        let available_impls: Vec<_> = self.config.implementations.iter()
            .filter(|impl_config| impl_config.available)
            .collect();

        println!("Running benchmarks on {} implementations:", available_impls.len());
        for impl_config in &available_impls {
            println!("  - {} ({})", impl_config.name, 
                impl_config.version.as_ref().unwrap_or(&"unknown version".to_string()));
        }

        // Core arithmetic benchmarks
        for impl_config in &available_impls {
            results.extend(self.run_arithmetic_benchmarks(impl_config));
            results.extend(self.run_list_benchmarks(impl_config));
            results.extend(self.run_recursion_benchmarks(impl_config));
            results.extend(self.run_allocation_benchmarks(impl_config));
        }

        let summary = self.generate_summary(&results);

        ComparisonReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            config: self.config.clone()),
            results,
            summary,
        }
    }

    /// Run arithmetic operation benchmarks
    fn run_arithmetic_benchmarks(&self, impl_config: &SchemeImplementation) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // Integer arithmetic benchmark
        let arithmetic_code = r#"
(define (arithmetic-test n)
  (let loop ((i 0) (sum 0))
    (if (< i n)
        (loop (+ i 1) (+ sum (* i i)))
        sum)))

(time (arithmetic-test 100000))
"#;

        if let Some(result) = self.run_single_benchmark(impl_config, "arithmetic_intensive", arithmetic_code) {
            results.push(result);
        }

        // Floating point benchmark
        let float_code = r#"
(define (float-test n)
  (let loop ((i 0.0) (sum 0.0))
    (if (< i n)
        (loop (+ i 1.0) (+ sum (sqrt (* i i))))
        sum)))

(time (float-test 50000.0))
"#;

        if let Some(result) = self.run_single_benchmark(impl_config, "floating_point", float_code) {
            results.push(result);
        }

        results
    }

    /// Run list operation benchmarks
    fn run_list_benchmarks(&self, impl_config: &SchemeImplementation) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // List creation and traversal
        let list_code = r#"
(define (list-test n)
  (let ((lst (let loop ((i 0) (acc '()))
               (if (< i n)
                   (loop (+ i 1) (cons i acc))
                   acc))))
    (length lst)))

(time (list-test 50000))
"#;

        if let Some(result) = self.run_single_benchmark(impl_config, "list_operations", list_code) {
            results.push(result);
        }

        // List mapping
        let map_code = r#"
(define (map-test n)
  (let ((lst (let loop ((i 0) (acc '()))
               (if (< i n)
                   (loop (+ i 1) (cons i acc))
                   acc))))
    (length (map (lambda (x) (* x x)) lst))))

(time (map-test 25000))
"#;

        if let Some(result) = self.run_single_benchmark(impl_config, "map_operations", map_code) {
            results.push(result);
        }

        results
    }

    /// Run recursion benchmarks
    fn run_recursion_benchmarks(&self, impl_config: &SchemeImplementation) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // Fibonacci (recursive)
        let fib_code = r#"
(define (fib n)
  (cond ((< n 2) n)
        (else (+ (fib (- n 1)) (fib (- n 2))))))

(time (fib 35))
"#;

        if let Some(result) = self.run_single_benchmark(impl_config, "fibonacci_recursive", fib_code) {
            results.push(result);
        }

        // Tail recursion
        let tail_rec_code = r#"
(define (factorial n)
  (define (fact-iter n acc)
    (if (<= n 1)
        acc
        (fact-iter (- n 1) (* n acc))))
  (fact-iter n 1))

(time (factorial 100000))
"#;

        if let Some(result) = self.run_single_benchmark(impl_config, "tail_recursion", tail_rec_code) {
            results.push(result);
        }

        results
    }

    /// Run allocation-intensive benchmarks
    fn run_allocation_benchmarks(&self, impl_config: &SchemeImplementation) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // Vector allocation and access
        let vector_code = r#"
(define (vector-test n)
  (let ((vec (make-vector n 0)))
    (let loop ((i 0))
      (if (< i n)
          (begin
            (vector-set! vec i (* i i))
            (loop (+ i 1)))
          (vector-ref vec (- n 1))))))

(time (vector-test 100000))
"#;

        if let Some(result) = self.run_single_benchmark(impl_config, "vector_allocation", vector_code) {
            results.push(result);
        }

        results
    }

    /// Run a single benchmark against an implementation
    fn run_single_benchmark(&self, impl_config: &SchemeImplementation, benchmark_name: &str, code: &str) -> Option<BenchmarkResult> {
        let start = Instant::now();

        // Create temporary file with the code
        let temp_file = format!("/tmp/lambdust_bench_{}.scm", benchmark_name);
        if let Err(_) = fs::write(&temp_file, code) {
            return Some(BenchmarkResult {
                implementation: impl_config.name.clone()),
                benchmark: benchmark_name.to_string(),
                execution_time_ns: 0,
                memory_usage_bytes: None,
                success: false,
                error: Some("Failed to write benchmark file".to_string()),
            });
        }

        let mut cmd = Command::new(&impl_config.command);
        cmd.args(&impl_config.args);

        // Handle different implementations differently
        match impl_config.name.as_str() {
            "Lambdust" => {
                cmd.arg(&temp_file);
            }
            _ => {
                cmd.arg(&temp_file);
            }
        }

        let result = cmd
            .output();

        let duration = start.elapsed();

        // Clean up temporary file
        let _ = fs::remove_file(&temp_file);

        match result {
            Ok(output) => {
                let success = output.status.success();
                let error = if !success {
                    Some(String::from_utf8_lossy(&output.stderr).to_string())
                } else {
                    None
                };

                Some(BenchmarkResult {
                    implementation: impl_config.name.clone()),
                    benchmark: benchmark_name.to_string(),
                    execution_time_ns: duration.as_nanos() as u64,
                    memory_usage_bytes: None, // TODO: Implement memory measurement
                    success,
                    error,
                })
            }
            Err(e) => {
                Some(BenchmarkResult {
                    implementation: impl_config.name.clone()),
                    benchmark: benchmark_name.to_string(),
                    execution_time_ns: 0,
                    memory_usage_bytes: None,
                    success: false,
                    error: Some(format!("Failed to execute: {}", e)),
                })
            }
        }
    }

    /// Generate summary statistics
    fn generate_summary(&self, results: &[BenchmarkResult]) -> ComparisonSummary {
        let total_benchmarks = results.len();
        let successful_benchmarks = results.iter().filter(|r| r.success).count();

        // Calculate performance ranking
        let mut impl_stats: std::collections::HashMap<String, Vec<u64>> = std::collections::HashMap::new();
        
        for result in results.iter().filter(|r| r.success) {
            impl_stats.entry(result.implementation.clone())
                .or_insert_with(Vec::new)
                .push(result.execution_time_ns);
        }

        let mut performance_ranking: Vec<PerformanceRank> = impl_stats.into_iter()
            .map(|(impl_name, times)| {
                let avg_time = times.iter().sum::<u64>() / times.len() as u64;
                PerformanceRank {
                    implementation: impl_name,
                    avg_execution_time_ns: avg_time,
                    relative_performance: 1.0, // Will be calculated below
                    fastest_count: 0, // Will be calculated below
                }
            })
            .collect();

        // Sort by average execution time
        performance_ranking.sort_by_key(|rank| rank.avg_execution_time_ns);

        // Calculate relative performance (fastest = 1.0)
        if let Some(fastest_time) = performance_ranking.first().map(|r| r.avg_execution_time_ns) {
            for rank in &mut performance_ranking {
                rank.relative_performance = rank.avg_execution_time_ns as f64 / fastest_time as f64;
            }
        }

        // Count fastest implementations per benchmark
        let unique_benchmarks: std::collections::HashSet<_> = results.iter()
            .map(|r| r.benchmark.clone())
            .collect();

        for benchmark in unique_benchmarks {
            if let Some(fastest_result) = results.iter()
                .filter(|r| r.benchmark == benchmark && r.success)
                .min_by_key(|r| r.execution_time_ns)
            {
                if let Some(rank) = performance_ranking.iter_mut()
                    .find(|r| r.implementation == fastest_result.implementation)
                {
                    rank.fastest_count += 1;
                }
            }
        }

        ComparisonSummary {
            total_benchmarks,
            successful_benchmarks,
            performance_ranking,
        }
    }

    /// Generate a detailed report
    pub fn generate_report(&self, report: &ComparisonReport) -> String {
        let mut output = String::new();
        
        output.push_str("# Scheme Implementation Performance Comparison Report\n\n");
        output.push_str(&format!("Generated: {}\n\n", report.timestamp));

        // Summary section
        output.push_str("## Summary\n\n");
        output.push_str(&format!("- Total benchmarks: {}\n", report.summary.total_benchmarks));
        output.push_str(&format!("- Successful benchmarks: {}\n", report.summary.successful_benchmarks));
        output.push_str(&format!("- Success rate: {:.1}%\n\n", 
            (report.summary.successful_benchmarks as f64 / report.summary.total_benchmarks as f64) * 100.0));

        // Performance ranking
        output.push_str("## Performance Ranking\n\n");
        output.push_str("| Rank | Implementation | Avg Time (ms) | Relative Performance | Fastest Count |\n");
        output.push_str("|------|----------------|---------------|---------------------|---------------|\n");

        for (i, rank) in report.summary.performance_ranking.iter().enumerate() {
            output.push_str(&format!(
                "| {} | {} | {:.2} | {:.2}x | {} |\n",
                i + 1,
                rank.implementation,
                rank.avg_execution_time_ns as f64 / 1_000_000.0,
                rank.relative_performance,
                rank.fastest_count
            ));
        }

        output.push('\n');

        // Detailed results by benchmark
        output.push_str("## Detailed Results\n\n");
        
        let unique_benchmarks: std::collections::HashSet<_> = report.results.iter()
            .map(|r| r.benchmark.clone())
            .collect();

        for benchmark in unique_benchmarks {
            output.push_str(&format!("### {}\n\n", benchmark));
            output.push_str("| Implementation | Time (ms) | Status |\n");
            output.push_str("|----------------|-----------|--------|\n");

            let mut benchmark_results: Vec<_> = report.results.iter()
                .filter(|r| r.benchmark == benchmark)
                .collect();
            benchmark_results.sort_by_key(|r| r.execution_time_ns);

            for result in benchmark_results {
                let status = if result.success { "✅" } else { "❌" };
                let time_ms = if result.success {
                    format!("{:.2}", result.execution_time_ns as f64 / 1_000_000.0)
                } else {
                    "N/A".to_string()
                };
                output.push_str(&format!("| {} | {} | {} |\n", result.implementation, time_ms, status));
            }
            output.push('\n');
        }

        output
    }
}

/// Run the full benchmark comparison suite
pub fn run_scheme_comparison() -> Result<ComparisonReport, Box<dyn std::error::Error>> {
    let mut config = ComparisonConfig::default();
    let mut suite = SchemeBenchmarkSuite::new(config.clone());
    
    println!("Detecting available Scheme implementations...");
    suite.detect_implementations();
    
    // Update config with detected implementations
    config = suite.config.clone());
    
    println!("Running performance comparison...");
    let report = suite.run_comparison();
    
    // Create output directory
    fs::create_dir_all(&config.output_dir)?;
    
    // Save JSON report
    let json_path = format!("{}/scheme_comparison.json", config.output_dir);
    let json_content = serde_json::to_string_pretty(&report)?;
    fs::write(&json_path, json_content)?;
    
    // Save markdown report
    let md_path = format!("{}/scheme_comparison.md", config.output_dir);
    let md_content = suite.generate_report(&report);
    fs::write(&md_path, md_content)?;
    
    println!("Results saved to:");
    println!("  - JSON: {}", json_path);
    println!("  - Markdown: {}", md_path);
    
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = ComparisonConfig::default();
        assert!(!config.implementations.is_empty());
        assert!(config.iterations > 0);
    }

    #[test]
    fn test_benchmark_suite_creation() {
        let config = ComparisonConfig::default();
        let suite = SchemeBenchmarkSuite::new(config);
        assert_eq!(suite.config.iterations, 5);
    }
}