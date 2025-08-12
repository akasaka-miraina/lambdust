//! Benchmarks for environment lookup optimization.
//!
//! This module provides comprehensive benchmarks to validate the performance
//! improvements of the CachedEnvironment over the traditional Environment.
//! It tests various scenarios including deep nesting, frequent lookups,
//! cache behavior, and mathematical correctness.

use crate::eval::{Environment, CachedEnvironment, Value};
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Configuration for benchmark parameters.
#[derive(Clone)]
pub struct BenchmarkConfig {
    /// Depth of environment chain to create
    pub chain_depth: usize,
    /// Number of variables per environment level  
    pub variables_per_level: usize,
    /// Number of lookup iterations to perform
    pub lookup_iterations: usize,
    /// Cache size for CachedEnvironment
    pub cache_size: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            chain_depth: 50,
            variables_per_level: 10,
            lookup_iterations: 1000,
            cache_size: 256,
        }
    }
}

/// Results from a benchmark run.
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    /// Time taken by traditional Environment lookup
    pub traditional_time: Duration,
    /// Time taken by CachedEnvironment lookup
    pub cached_time: Duration,
    /// Performance improvement ratio (traditional / cached)
    pub speedup_ratio: f64,
    /// Cache hit ratio percentage
    pub cache_hit_ratio: f64,
    /// Number of lookups performed
    pub total_lookups: usize,
    /// Average lookup time in nanoseconds (traditional)
    pub avg_traditional_ns: f64,
    /// Average lookup time in nanoseconds (cached)
    pub avg_cached_ns: f64,
}

impl BenchmarkResults {
    /// Creates a new BenchmarkResults from timing data.
    pub fn new(
        traditional_time: Duration,
        cached_time: Duration, 
        cache_hit_ratio: f64,
        total_lookups: usize
    ) -> Self {
        let speedup_ratio = if cached_time.as_nanos() > 0 {
            traditional_time.as_nanos() as f64 / cached_time.as_nanos() as f64
        } else {
            f64::INFINITY
        };
        
        Self {
            traditional_time,
            cached_time,
            speedup_ratio,
            cache_hit_ratio,
            total_lookups,
            avg_traditional_ns: traditional_time.as_nanos() as f64 / total_lookups as f64,
            avg_cached_ns: cached_time.as_nanos() as f64 / total_lookups as f64,
        }
    }
    
    /// Pretty prints the benchmark results.
    pub fn print_results(&self) {
        println!("\n=== Environment Optimization Benchmark Results ===");
        println!("Total lookups: {}", self.total_lookups);
        println!("Traditional Environment: {:?} ({:.2} ns/lookup)", 
                 self.traditional_time, self.avg_traditional_ns);
        println!("Cached Environment: {:?} ({:.2} ns/lookup)", 
                 self.cached_time, self.avg_cached_ns);
        println!("Speedup: {:.2}x", self.speedup_ratio);
        println!("Cache hit ratio: {:.1}%", self.cache_hit_ratio);
        
        if self.speedup_ratio >= 5.0 {
            println!("✅ Excellent performance improvement!");
        } else if self.speedup_ratio >= 2.0 {
            println!("✅ Good performance improvement!");
        } else if self.speedup_ratio >= 1.5 {
            println!("⚠️  Moderate performance improvement");
        } else {
            println!("❌ Performance improvement below expectations");
        }
    }
}

/// Creates a deep environment chain for testing.
/// 
/// Structure: Global -> Level1 -> Level2 -> ... -> LevelN
/// Each level defines variables like: var0_L, var1_L, ..., varN_L
/// where L is the level number.
pub fn create_environment_chain(config: &BenchmarkConfig) -> Rc<Environment> {
    let mut env = Rc::new(Environment::new(None, 0));
    
    // Global level variables
    for i in 0..config.variables_per_level {
        env.define(format!("global_var{i}"), Value::integer(i as i64));
    }
    
    // Create nested levels
    for level in 1..=config.chain_depth {
        env = env.extend((level as u32).into());
        
        // Add variables at this level
        for i in 0..config.variables_per_level {
            env.define(
                format!("var{i}_{level}"), 
                Value::integer((level * 100 + i) as i64)
            );
        }
    }
    
    env
}

/// Generates a realistic lookup pattern for benchmarking.
/// 
/// Simulates realistic variable access patterns:
/// - 70% access to recent/local variables (shallow in chain)
/// - 20% access to middle-level variables  
/// - 10% access to global/deep variables
pub fn generate_lookup_pattern(config: &BenchmarkConfig) -> Vec<String> {
    let mut patterns = Vec::new();
    
    for _ in 0..config.lookup_iterations {
        let rand_val = (patterns.len() * 73 + 17) % 100; // Pseudo-random
        
        let var_name = if rand_val < 70 {
            // Recent variables (top 20% of chain)
            let level = config.chain_depth - (rand_val % (config.chain_depth / 5));
            let var_idx = rand_val % config.variables_per_level;
            format!("var{var_idx}_{level}")
        } else if rand_val < 90 {
            // Middle variables
            let level = config.chain_depth / 2 + (rand_val % (config.chain_depth / 4));
            let var_idx = rand_val % config.variables_per_level;
            format!("var{var_idx}_{level}")
        } else {
            // Global variables (deepest in chain)
            let var_idx = rand_val % config.variables_per_level;
            format!("global_var{var_idx}")
        };
        
        patterns.push(var_name);
    }
    
    patterns
}

/// Benchmarks traditional Environment lookup performance.
pub fn benchmark_traditional_environment(
    env: &Rc<Environment>,
    lookup_patterns: &[String]
) -> Duration {
    let start = Instant::now();
    
    for pattern in lookup_patterns {
        let _value = env.lookup(pattern);
    }
    
    start.elapsed()
}

/// Benchmarks CachedEnvironment lookup performance.
pub fn benchmark_cached_environment(
    env: &Rc<Environment>,
    lookup_patterns: &[String],
    cache_size: usize
) -> (Duration, f64) {
    let cached_env = CachedEnvironment::with_cache_size(env.clone(), cache_size);
    
    let start = Instant::now();
    
    for pattern in lookup_patterns {
        let _value = cached_env.lookup(pattern);
    }
    
    let elapsed = start.elapsed();
    let hit_ratio = cached_env.cache_hit_ratio();
    
    (elapsed, hit_ratio)
}

/// Runs a comprehensive benchmark comparing both environments.
pub fn run_comprehensive_benchmark(config: BenchmarkConfig) -> BenchmarkResults {
    println!("Setting up environment chain (depth: {}, vars/level: {})...", 
             config.chain_depth, config.variables_per_level);
    
    // Setup
    let env = create_environment_chain(&config);
    let lookup_patterns = generate_lookup_pattern(&config);
    
    println!("Running traditional Environment benchmark...");
    let traditional_time = benchmark_traditional_environment(&env, &lookup_patterns);
    
    println!("Running CachedEnvironment benchmark...");
    let (cached_time, cache_hit_ratio) = benchmark_cached_environment(
        &env, &lookup_patterns, config.cache_size
    );
    
    BenchmarkResults::new(
        traditional_time,
        cached_time, 
        cache_hit_ratio,
        config.lookup_iterations
    )
}

/// Tests mathematical correctness by comparing results.
pub fn verify_correctness(config: &BenchmarkConfig) -> bool {
    let env = create_environment_chain(config);
    let cached_env = CachedEnvironment::new(env.clone());
    
    // Test a comprehensive set of variable lookups
    let test_cases = vec![
        format!("global_var0"),
        format!("var0_1"),
        format!("var5_{}", config.chain_depth),
        format!("var{}_10", config.variables_per_level - 1),
        format!("nonexistent_variable"),
    ];
    
    for test_case in &test_cases {
        let traditional_result = env.lookup(test_case);
        let cached_result = cached_env.lookup(test_case);
        
        if traditional_result != cached_result {
            println!("❌ Correctness error for '{test_case}': traditional={traditional_result:?}, cached={cached_result:?}");
            return false;
        }
    }
    
    // Test variable shadowing correctness
    let outer = Rc::new(Environment::new(None, 0));
    outer.define("x".to_string(), Value::integer(1));
    
    let middle = outer.extend(1);
    middle.define("x".to_string(), Value::integer(10));
    middle.define("y".to_string(), Value::integer(2));
    
    let inner = middle.extend(2);
    inner.define("x".to_string(), Value::integer(100));
    
    let cached_inner = CachedEnvironment::new(inner.clone());
    
    // Verify shadowing behavior
    if inner.lookup("x") != cached_inner.lookup("x") ||
       inner.lookup("y") != cached_inner.lookup("y") {
        println!("❌ Shadowing behavior mismatch");
        return false;
    }
    
    println!("✅ Mathematical correctness verified");
    true
}

/// Runs performance regression tests to ensure optimization targets are met.
pub fn run_performance_tests() {
    let configs = [
        // Shallow environments
        BenchmarkConfig { chain_depth: 5, variables_per_level: 5, ..Default::default() },
        // Medium environments  
        BenchmarkConfig { chain_depth: 25, variables_per_level: 8, ..Default::default() },
        // Deep environments (target scenario)
        BenchmarkConfig { chain_depth: 100, variables_per_level: 10, ..Default::default() },
        // Very deep environments
        BenchmarkConfig { chain_depth: 200, variables_per_level: 15, ..Default::default() },
    ];
    
    for (i, config) in configs.iter().enumerate() {
        println!("\n--- Test Case {} ---", i + 1);
        
        // Verify correctness first
        if !verify_correctness(config) {
            println!("❌ Correctness verification failed for test case {}", i + 1);
            continue;
        }
        
        // Run performance benchmark
        let results = run_comprehensive_benchmark(config.clone());
        results.print_results();
        
        // Check if performance targets are met
        let target_speedup = match config.chain_depth {
            0..=10 => 1.5,   // Shallow: modest improvement expected
            11..=50 => 3.0,  // Medium: good improvement expected
            51..=150 => 8.0, // Deep: significant improvement expected
            _ => 15.0,       // Very deep: excellent improvement expected
        };
        
        if results.speedup_ratio >= target_speedup {
            println!("✅ Performance target achieved ({:.1}x >= {:.1}x)", 
                     results.speedup_ratio, target_speedup);
        } else {
            println!("⚠️  Performance target not met ({:.1}x < {:.1}x)", 
                     results.speedup_ratio, target_speedup);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_environment_chain_creation() {
        let config = BenchmarkConfig {
            chain_depth: 10,
            variables_per_level: 3,
            ..Default::default()
        };
        
        let env = create_environment_chain(&config);
        
        // Test that variables exist at expected locations
        assert!(env.lookup("global_var0").is_some());
        assert!(env.lookup("var0_5").is_some());
        assert!(env.lookup("var2_10").is_some());
        assert!(env.lookup("nonexistent").is_none());
    }
    
    #[test]
    fn test_lookup_pattern_generation() {
        let config = BenchmarkConfig {
            chain_depth: 10,
            variables_per_level: 5,
            lookup_iterations: 100,
            ..Default::default()
        };
        
        let patterns = generate_lookup_pattern(&config);
        assert_eq!(patterns.len(), config.lookup_iterations);
        
        // Check that patterns contain expected variable names
        let has_global = patterns.iter().any(|p| p.starts_with("global_"));
        let has_numbered = patterns.iter().any(|p| p.contains("_"));
        
        assert!(has_global);
        assert!(has_numbered);
    }
    
    #[test]
    fn test_benchmark_correctness() {
        let config = BenchmarkConfig {
            chain_depth: 5,
            variables_per_level: 3,
            lookup_iterations: 10,
            ..Default::default()
        };
        
        assert!(verify_correctness(&config));
    }
    
    #[test] 
    fn test_small_scale_benchmark() {
        let config = BenchmarkConfig {
            chain_depth: 3,
            variables_per_level: 2,
            lookup_iterations: 20,
            cache_size: 16,
        };
        
        let results = run_comprehensive_benchmark(config);
        
        // Basic sanity checks
        assert!(results.speedup_ratio > 0.0);
        assert!(results.cache_hit_ratio >= 0.0 && results.cache_hit_ratio <= 100.0);
        assert_eq!(results.total_lookups, 20);
    }
}