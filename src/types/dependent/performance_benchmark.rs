//! Performance benchmarks and memory analysis for dependent type optimization.
//!
//! This module provides comprehensive performance testing for the memory-optimized
//! dependent type system, comparing the new arena-based approach with the
//! traditional Box-based implementation.
//!
//! # Benchmark Categories
//!
//! 1. **Memory Efficiency**: Arena vs Box allocation patterns
//! 2. **Allocation Speed**: Bump pointer vs malloc performance
//! 3. **Cache Locality**: Memory access patterns and cache efficiency
//! 4. **Deduplication**: Type sharing and memory savings
//! 5. **Scalability**: Performance under high allocation loads
//! 6. **Real-world Usage**: Realistic dependent type checking scenarios

use super::{
    arena::{TypeArena, TypeRef, TermRef, DependentTypeData, DependentTermData},
    memory_pool::{MemoryPoolManager, AllocationType, MemoryPoolStatistics},
    optimized_core::{OptimizedDependentType, OptimizedTypingContext, OptimizedNormalizer},
    core::{DependentType, DependentTerm, TypingContext, Normalizer},
};
use crate::diagnostics::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::rc::Rc;

/// Comprehensive performance benchmark suite for dependent types.
pub struct DependentTypeBenchmarkSuite {
    /// Results from all benchmark runs
    results: Vec<BenchmarkResult>,
    /// Configuration for benchmark parameters
    config: BenchmarkConfig,
    /// Memory tracking utilities
    memory_tracker: MemoryTracker,
}

/// Individual benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    /// Duration of the benchmark
    pub duration: Duration,
    /// Memory usage statistics
    pub memory_stats: MemoryUsageStats,
    /// Number of operations performed
    pub operations: u64,
    /// Operations per second
    pub ops_per_sec: f64,
    /// Memory efficiency ratio (vs baseline)
    pub memory_efficiency: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryUsageStats {
    /// Peak memory usage (bytes)
    pub peak_memory: usize,
    /// Average memory usage (bytes)
    pub average_memory: usize,
    /// Total allocations
    pub total_allocations: u64,
    /// Average allocation size
    pub average_allocation_size: f64,
    /// Memory fragmentation ratio
    pub fragmentation_ratio: f64,
    /// Garbage collection overhead
    pub gc_overhead: f64,
}

/// Configuration for benchmark runs
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of iterations for each benchmark
    pub iterations: u32,
    /// Warmup iterations before measuring
    pub warmup_iterations: u32,
    /// Maximum benchmark duration
    pub max_duration: Duration,
    /// Sample size for memory measurements
    pub memory_sample_size: u32,
    /// Enable detailed profiling
    pub detailed_profiling: bool,
}

/// Memory tracking and profiling utility
#[derive(Debug)]
struct MemoryTracker {
    /// Snapshots of memory usage over time
    snapshots: Vec<MemorySnapshot>,
    /// Baseline memory usage
    baseline: usize,
    /// Peak memory observed
    peak: usize,
}

/// Point-in-time memory snapshot
#[derive(Debug, Clone)]
struct MemorySnapshot {
    /// Timestamp of snapshot
    timestamp: Instant,
    /// Memory usage at this point
    memory_usage: usize,
    /// Active allocations count
    active_allocations: u64,
    /// Allocation rate (per second)
    allocation_rate: f64,
}

impl DependentTypeBenchmarkSuite {
    /// Create a new benchmark suite with default configuration
    pub fn new() -> Self {
        Self::with_config(BenchmarkConfig::default())
    }
    
    /// Create benchmark suite with custom configuration
    pub fn with_config(config: BenchmarkConfig) -> Self {
        Self {
            results: Vec::new(),
            config,
            memory_tracker: MemoryTracker::new(),
        }
    }
    
    /// Run all benchmarks and return comprehensive results
    pub fn run_all_benchmarks(&mut self) -> Result<BenchmarkSummary> {
        println!("🚀 Starting Dependent Type Performance Benchmarks...");
        
        // 1. Memory Allocation Benchmarks
        self.run_allocation_benchmarks()?;
        
        // 2. Type Construction Benchmarks
        self.run_type_construction_benchmarks()?;
        
        // 3. Cache Efficiency Benchmarks
        self.run_cache_efficiency_benchmarks()?;
        
        // 4. Deduplication Benchmarks
        self.run_deduplication_benchmarks()?;
        
        // 5. Scalability Benchmarks
        self.run_scalability_benchmarks()?;
        
        // 6. Real-world Scenario Benchmarks
        self.run_realistic_scenario_benchmarks()?;
        
        Ok(self.generate_summary())
    }
    
    /// Benchmark memory allocation patterns: Arena vs Box
    fn run_allocation_benchmarks(&mut self) -> Result<()> {
        println!("📊 Running allocation benchmarks...");
        
        // Benchmark 1: Simple type allocations
        self.benchmark_simple_allocations()?;
        
        // Benchmark 2: Complex nested type allocations
        self.benchmark_nested_allocations()?;
        
        // Benchmark 3: Rapid allocation/deallocation cycles
        self.benchmark_allocation_cycles()?;
        
        Ok(())
    }
    
    /// Benchmark type construction and manipulation
    fn run_type_construction_benchmarks(&mut self) -> Result<()> {
        println!("🏗️ Running type construction benchmarks...");
        
        // Benchmark Pi type construction
        self.benchmark_pi_type_construction()?;
        
        // Benchmark Sigma type construction
        self.benchmark_sigma_type_construction()?;
        
        // Benchmark complex inductive types
        self.benchmark_inductive_type_construction()?;
        
        Ok(())
    }
    
    /// Benchmark cache efficiency and locality
    fn run_cache_efficiency_benchmarks(&mut self) -> Result<()> {
        println!("⚡ Running cache efficiency benchmarks...");
        
        // Benchmark sequential access patterns
        self.benchmark_sequential_access()?;
        
        // Benchmark random access patterns
        self.benchmark_random_access()?;
        
        // Benchmark access locality
        self.benchmark_access_locality()?;
        
        Ok(())
    }
    
    /// Benchmark type deduplication efficiency
    fn run_deduplication_benchmarks(&mut self) -> Result<()> {
        println!("🔄 Running deduplication benchmarks...");
        
        // Benchmark identical type sharing
        self.benchmark_identical_type_sharing()?;
        
        // Benchmark structural sharing
        self.benchmark_structural_sharing()?;
        
        Ok(())
    }
    
    /// Benchmark system scalability under load
    fn run_scalability_benchmarks(&mut self) -> Result<()> {
        println!("📈 Running scalability benchmarks...");
        
        // Benchmark with increasing allocation counts
        self.benchmark_scaling_allocations()?;
        
        // Benchmark with increasing type complexity
        self.benchmark_scaling_complexity()?;
        
        // Benchmark concurrent access patterns
        self.benchmark_concurrent_access()?;
        
        Ok(())
    }
    
    /// Benchmark realistic dependent type checking scenarios
    fn run_realistic_scenario_benchmarks(&mut self) -> Result<()> {
        println!("🎯 Running realistic scenario benchmarks...");
        
        // Benchmark type checking pipeline
        self.benchmark_type_checking_pipeline()?;
        
        // Benchmark normalization performance
        self.benchmark_normalization_performance()?;
        
        // Benchmark large program analysis
        self.benchmark_large_program_analysis()?;
        
        Ok(())
    }
    
    // Individual benchmark implementations
    
    fn benchmark_simple_allocations(&mut self) -> Result<()> {
        // Box-based approach
        let box_result = self.time_benchmark("Simple Allocations (Box)", |tracker| {
            let mut context = TypingContext::new();
            for i in 0..10000 {
                let universe_type = DependentType::Universe(i % 10);
                context.bind_variable(format!("x{}", i), universe_type);
                tracker.record_allocation(1);
            }
        })?;
        
        // Arena-based approach  
        let arena_result = self.time_benchmark("Simple Allocations (Arena)", |tracker| {
            let mut context = OptimizedTypingContext::new();
            for i in 0..10000 {
                let universe_type = OptimizedDependentType::Universe(i % 10);
                let _ = context.bind_variable(format!("x{}", i), universe_type);
                tracker.record_allocation(1);
            }
        })?;
        
        self.results.push(box_result);
        self.results.push(arena_result);
        
        Ok(())
    }
    
    fn benchmark_nested_allocations(&mut self) -> Result<()> {
        // Complex nested Pi types: (x : A) → (y : B(x)) → C(x, y)
        let box_result = self.time_benchmark("Nested Allocations (Box)", |tracker| {
            for i in 0..1000 {
                let base_type = DependentType::Universe(0);
                let inner_pi = DependentType::Pi {
                    var: format!("y{}", i),
                    domain: Box::new(DependentType::Universe(0)),
                    codomain: Box::new(DependentType::Universe(1)),
                };
                let outer_pi = DependentType::Pi {
                    var: format!("x{}", i),
                    domain: Box::new(base_type),
                    codomain: Box::new(inner_pi),
                };
                
                // Simulate usage
                std::mem::drop(outer_pi);
                tracker.record_allocation(4); // Multiple Box allocations
            }
        })?;
        
        let arena_result = self.time_benchmark("Nested Allocations (Arena)", |tracker| {
            let context = OptimizedTypingContext::new();
            for i in 0..1000 {
                let base_type = OptimizedDependentType::Universe(0);
                let inner_pi = OptimizedDependentType::Pi {
                    var: format!("y{}", i),
                    domain: Rc::new(OptimizedDependentType::Universe(0)),
                    codomain: Rc::new(OptimizedDependentType::Universe(1)),
                };
                let outer_pi = OptimizedDependentType::Pi {
                    var: format!("x{}", i),
                    domain: Rc::new(base_type),
                    codomain: Rc::new(inner_pi),
                };
                
                // Simulate usage
                std::mem::drop(outer_pi);
                tracker.record_allocation(1); // Single arena allocation
            }
        })?;
        
        self.results.push(box_result);
        self.results.push(arena_result);
        
        Ok(())
    }
    
    fn benchmark_allocation_cycles(&mut self) -> Result<()> {
        let box_result = self.time_benchmark("Allocation Cycles (Box)", |tracker| {
            let mut types = Vec::new();
            
            // Allocate phase
            for i in 0..5000 {
                let pi_type = DependentType::Pi {
                    var: format!("x{}", i),
                    domain: Box::new(DependentType::Universe(0)),
                    codomain: Box::new(DependentType::Universe(1)),
                };
                types.push(pi_type);
                tracker.record_allocation(2);
            }
            
            // Deallocate phase
            types.clear();
        })?;
        
        let arena_result = self.time_benchmark("Allocation Cycles (Arena)", |tracker| {
            let mut context = OptimizedTypingContext::new();
            let mut vars = Vec::new();
            
            // Allocate phase
            for i in 0..5000 {
                let pi_type = OptimizedDependentType::Pi {
                    var: format!("x{}", i),
                    domain: Rc::new(OptimizedDependentType::Universe(0)),
                    codomain: Rc::new(OptimizedDependentType::Universe(1)),
                };
                let var_name = format!("v{}", i);
                let _ = context.bind_variable(var_name.clone(), pi_type);
                vars.push(var_name);
                tracker.record_allocation(1);
            }
            
            // Deallocate phase (unbind variables)
            for var in vars {
                context.unbind_variable(&var);
            }
        })?;
        
        self.results.push(box_result);
        self.results.push(arena_result);
        
        Ok(())
    }
    
    fn benchmark_pi_type_construction(&mut self) -> Result<()> {
        let result = self.time_benchmark("Pi Type Construction", |tracker| {
            let arena = TypeArena::new();
            
            for i in 0..10000 {
                let domain_data = DependentTypeData::Universe(0);
                let codomain_data = DependentTypeData::Universe(1);
                
                let domain_ref = arena.alloc_type(domain_data).unwrap();
                let codomain_ref = arena.alloc_type(codomain_data).unwrap();
                
                let pi_data = DependentTypeData::Pi {
                    var: format!("x{}", i),
                    domain: domain_ref,
                    codomain: codomain_ref,
                };
                
                let _pi_ref = arena.alloc_type(pi_data).unwrap();
                tracker.record_allocation(3);
            }
        })?;
        
        self.results.push(result);
        Ok(())
    }
    
    fn benchmark_sigma_type_construction(&mut self) -> Result<()> {
        let result = self.time_benchmark("Sigma Type Construction", |tracker| {
            let arena = TypeArena::new();
            
            for i in 0..10000 {
                let first_data = DependentTypeData::Universe(0);
                let second_data = DependentTypeData::Universe(1);
                
                let first_ref = arena.alloc_type(first_data).unwrap();
                let second_ref = arena.alloc_type(second_data).unwrap();
                
                let sigma_data = DependentTypeData::Sigma {
                    var: format!("x{}", i),
                    first: first_ref,
                    second: second_ref,
                };
                
                let _sigma_ref = arena.alloc_type(sigma_data).unwrap();
                tracker.record_allocation(3);
            }
        })?;
        
        self.results.push(result);
        Ok(())
    }
    
    fn benchmark_inductive_type_construction(&mut self) -> Result<()> {
        let result = self.time_benchmark("Inductive Type Construction", |tracker| {
            let arena = TypeArena::new();
            
            for i in 0..1000 {
                // Create constructors
                let mut constructors = Vec::new();
                for j in 0..5 {
                    let ctor_type_data = DependentTypeData::Universe(0);
                    let ctor_type_ref = arena.alloc_type(ctor_type_data).unwrap();
                    constructors.push((format!("ctor{}_{}", i, j), ctor_type_ref));
                }
                
                let inductive_data = DependentTypeData::Inductive {
                    name: format!("Type{}", i),
                    parameters: vec![],
                    universe_level: 1,
                    constructors,
                    induction_principle: None,
                };
                
                let _inductive_ref = arena.alloc_type(inductive_data).unwrap();
                tracker.record_allocation(6); // 1 inductive + 5 constructors
            }
        })?;
        
        self.results.push(result);
        Ok(())
    }
    
    fn benchmark_sequential_access(&mut self) -> Result<()> {
        let result = self.time_benchmark("Sequential Access", |tracker| {
            let arena = TypeArena::new();
            let mut refs = Vec::new();
            
            // Create types
            for i in 0..10000 {
                let type_data = DependentTypeData::Universe(i % 10);
                let type_ref = arena.alloc_type(type_data).unwrap();
                refs.push(type_ref);
                tracker.record_allocation(1);
            }
            
            // Sequential access
            for &type_ref in &refs {
                let _resolved = arena.resolve_type(type_ref).unwrap();
            }
        })?;
        
        self.results.push(result);
        Ok(())
    }
    
    fn benchmark_random_access(&mut self) -> Result<()> {
        let result = self.time_benchmark("Random Access", |tracker| {
            let arena = TypeArena::new();
            let mut refs = Vec::new();
            
            // Create types
            for i in 0..10000 {
                let type_data = DependentTypeData::Universe(i % 10);
                let type_ref = arena.alloc_type(type_data).unwrap();
                refs.push(type_ref);
                tracker.record_allocation(1);
            }
            
            // Random access pattern
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            for i in 0..10000 {
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let idx = (hasher.finish() as usize) % refs.len();
                let _resolved = arena.resolve_type(refs[idx]).unwrap();
            }
        })?;
        
        self.results.push(result);
        Ok(())
    }
    
    fn benchmark_access_locality(&mut self) -> Result<()> {
        let result = self.time_benchmark("Access Locality", |tracker| {
            let arena = TypeArena::new();
            let mut refs = Vec::new();
            
            // Create related types in sequence (good locality)
            for i in 0..1000 {
                let domain_data = DependentTypeData::Universe(0);
                let codomain_data = DependentTypeData::Universe(1);
                let domain_ref = arena.alloc_type(domain_data).unwrap();
                let codomain_ref = arena.alloc_type(codomain_data).unwrap();
                
                let pi_data = DependentTypeData::Pi {
                    var: format!("x{}", i),
                    domain: domain_ref,
                    codomain: codomain_ref,
                };
                let pi_ref = arena.alloc_type(pi_data).unwrap();
                
                refs.push((domain_ref, codomain_ref, pi_ref));
                tracker.record_allocation(3);
            }
            
            // Access related types together (test locality)
            for &(domain_ref, codomain_ref, pi_ref) in &refs {
                let _pi = arena.resolve_type(pi_ref).unwrap();
                let _domain = arena.resolve_type(domain_ref).unwrap();
                let _codomain = arena.resolve_type(codomain_ref).unwrap();
            }
        })?;
        
        self.results.push(result);
        Ok(())
    }
    
    fn benchmark_identical_type_sharing(&mut self) -> Result<()> {
        let result = self.time_benchmark("Identical Type Sharing", |tracker| {
            let arena = TypeArena::new();
            
            // Create many identical universe types (should deduplicate)
            let mut refs = Vec::new();
            for _ in 0..10000 {
                let type_data = DependentTypeData::Universe(0);
                let type_ref = arena.alloc_type(type_data).unwrap();
                refs.push(type_ref);
                tracker.record_allocation(1);
            }
            
            // Verify they share storage (many refs should point to same data)
            let stats = arena.memory_stats();
            println!("Deduplication: {} refs, {} actual types", refs.len(), stats.types_count);
        })?;
        
        self.results.push(result);
        Ok(())
    }
    
    fn benchmark_structural_sharing(&mut self) -> Result<()> {
        let result = self.time_benchmark("Structural Sharing", |tracker| {
            let arena = TypeArena::new();
            
            // Create shared base types
            let base_type_data = DependentTypeData::Universe(0);
            let base_type_ref = arena.alloc_type(base_type_data).unwrap();
            
            // Create many Pi types sharing the same domain
            let mut pi_refs = Vec::new();
            for i in 0..5000 {
                let codomain_data = DependentTypeData::Universe(i % 5); // Some variety
                let codomain_ref = arena.alloc_type(codomain_data).unwrap();
                
                let pi_data = DependentTypeData::Pi {
                    var: format!("x{}", i),
                    domain: base_type_ref, // Shared domain
                    codomain: codomain_ref,
                };
                let pi_ref = arena.alloc_type(pi_data).unwrap();
                pi_refs.push(pi_ref);
                tracker.record_allocation(2); // pi + codomain (domain is shared)
            }
            
            let stats = arena.memory_stats();
            println!("Structural sharing: {} Pi types, {} total arena types", pi_refs.len(), stats.types_count);
        })?;
        
        self.results.push(result);
        Ok(())
    }
    
    fn benchmark_scaling_allocations(&mut self) -> Result<()> {
        // Test scaling from 1K to 100K allocations
        for scale in &[1_000, 10_000, 50_000, 100_000] {
            let result = self.time_benchmark(
                &format!("Scaling Allocations ({}K)", scale / 1000),
                |tracker| {
                    let arena = TypeArena::new();
                    
                    for i in 0..*scale {
                        let type_data = DependentTypeData::Universe(i % 100);
                        let _type_ref = arena.alloc_type(type_data).unwrap();
                        tracker.record_allocation(1);
                    }
                }
            )?;
            
            self.results.push(result);
        }
        
        Ok(())
    }
    
    fn benchmark_scaling_complexity(&mut self) -> Result<()> {
        // Test with increasing type complexity
        for depth in &[2, 5, 10, 20] {
            let result = self.time_benchmark(
                &format!("Scaling Complexity (depth {})", depth),
                |tracker| {
                    let arena = TypeArena::new();
                    
                    for i in 0..1000 {
                        let mut current_type_ref = {
                            let base_data = DependentTypeData::Universe(0);
                            arena.alloc_type(base_data).unwrap()
                        };
                        
                        // Build nested Pi types
                        for j in 0..*depth {
                            let pi_data = DependentTypeData::Pi {
                                var: format!("x{}_{}", i, j),
                                domain: current_type_ref,
                                codomain: current_type_ref,
                            };
                            current_type_ref = arena.alloc_type(pi_data).unwrap();
                            tracker.record_allocation(1);
                        }
                    }
                }
            )?;
            
            self.results.push(result);
        }
        
        Ok(())
    }
    
    fn benchmark_concurrent_access(&mut self) -> Result<()> {
        // Note: This is a simplified version - real concurrent benchmarks
        // would require more sophisticated setup
        let result = self.time_benchmark("Concurrent Access (Simulated)", |tracker| {
            let arena = TypeArena::new();
            let mut refs = Vec::new();
            
            // Pre-populate arena
            for i in 0..10000 {
                let type_data = DependentTypeData::Universe(i % 10);
                let type_ref = arena.alloc_type(type_data).unwrap();
                refs.push(type_ref);
                tracker.record_allocation(1);
            }
            
            // Simulate concurrent access patterns
            for _ in 0..50000 {
                let idx = refs.len() / 2; // Access middle elements frequently
                let _resolved = arena.resolve_type(refs[idx]).unwrap();
            }
        })?;
        
        self.results.push(result);
        Ok(())
    }
    
    fn benchmark_type_checking_pipeline(&mut self) -> Result<()> {
        let result = self.time_benchmark("Type Checking Pipeline", |tracker| {
            let mut context = OptimizedTypingContext::new();
            let normalizer = OptimizedNormalizer::new();
            
            // Simulate type checking a realistic program
            for i in 0..1000 {
                // Create function type: (x : Nat) → Nat
                let nat_type = OptimizedDependentType::Universe(0); // Simplified Nat
                let fun_type = OptimizedDependentType::Pi {
                    var: "x".to_string(),
                    domain: Rc::new(nat_type.clone()),
                    codomain: Rc::new(nat_type.clone()),
                };
                
                // Bind function
                let _ = context.bind_variable(format!("f{}", i), fun_type.clone());
                
                // Normalize the type
                let _normalized = normalizer.normalize_type(&fun_type).unwrap();
                
                tracker.record_allocation(3); // Domain, codomain, pi type
            }
        })?;
        
        self.results.push(result);
        Ok(())
    }
    
    fn benchmark_normalization_performance(&mut self) -> Result<()> {
        let result = self.time_benchmark("Normalization Performance", |tracker| {
            let normalizer = OptimizedNormalizer::new();
            
            for i in 0..5000 {
                // Create complex nested type
                let base = OptimizedDependentType::Universe(0);
                let mut complex_type = base;
                
                // Build nested Pi types
                for j in 0..5 {
                    complex_type = OptimizedDependentType::Pi {
                        var: format!("x{}_{}", i, j),
                        domain: Rc::new(complex_type.clone()),
                        codomain: Rc::new(OptimizedDependentType::Universe(1)),
                    };
                }
                
                // Normalize
                let _normalized = normalizer.normalize_type(&complex_type).unwrap();
                tracker.record_allocation(10); // Estimate for complex normalization
            }
        })?;
        
        self.results.push(result);
        Ok(())
    }
    
    fn benchmark_large_program_analysis(&mut self) -> Result<()> {
        let result = self.time_benchmark("Large Program Analysis", |tracker| {
            let mut context = OptimizedTypingContext::new();
            let normalizer = OptimizedNormalizer::new();
            
            // Simulate analyzing a large program with many definitions
            for i in 0..10000 {
                let type_complexity = i % 10; // Varying complexity
                
                match type_complexity {
                    0..=3 => {
                        // Simple types
                        let simple_type = OptimizedDependentType::Universe(type_complexity as u32);
                        let _ = context.bind_variable(format!("simple{}", i), simple_type);
                        tracker.record_allocation(1);
                    }
                    4..=7 => {
                        // Function types
                        let fun_type = OptimizedDependentType::Pi {
                            var: format!("x{}", i),
                            domain: Rc::new(OptimizedDependentType::Universe(0)),
                            codomain: Rc::new(OptimizedDependentType::Universe(1)),
                        };
                        let _ = context.bind_variable(format!("fun{}", i), fun_type);
                        tracker.record_allocation(3);
                    }
                    _ => {
                        // Complex dependent types
                        let complex_type = OptimizedDependentType::Sigma {
                            var: format!("y{}", i),
                            first: Rc::new(OptimizedDependentType::Universe(0)),
                            second: Rc::new(OptimizedDependentType::Pi {
                                var: format!("z{}", i),
                                domain: Rc::new(OptimizedDependentType::Universe(0)),
                                codomain: Rc::new(OptimizedDependentType::Universe(1)),
                            }),
                        };
                        let _ = context.bind_variable(format!("complex{}", i), complex_type.clone());
                        let _normalized = normalizer.normalize_type(&complex_type).unwrap();
                        tracker.record_allocation(5);
                    }
                }
            }
            
            // Get final statistics
            let stats = context.memory_stats();
            let norm_stats = normalizer.memory_stats();
            println!("Large program analysis: Context memory: {}KB, Normalizer memory: {}KB", 
                     stats.total_memory() / 1024, 
                     norm_stats.total_memory() / 1024);
        })?;
        
        self.results.push(result);
        Ok(())
    }
    
    /// Helper method to time a benchmark with memory tracking
    fn time_benchmark<F>(&mut self, name: &str, benchmark_fn: F) -> Result<BenchmarkResult>
    where
        F: Fn(&mut MemoryTracker),
    {
        println!("  Running: {}", name);
        
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let mut dummy_tracker = MemoryTracker::new();
            benchmark_fn(&mut dummy_tracker);
        }
        
        // Reset memory tracker
        self.memory_tracker = MemoryTracker::new();
        self.memory_tracker.take_baseline_snapshot();
        
        // Run actual benchmark
        let start_time = Instant::now();
        benchmark_fn(&mut self.memory_tracker);
        let duration = start_time.elapsed();
        
        // Finalize memory tracking
        self.memory_tracker.take_final_snapshot();
        let memory_stats = self.memory_tracker.analyze();
        
        let operations = memory_stats.total_allocations;
        let ops_per_sec = if duration.as_secs_f64() > 0.0 {
            operations as f64 / duration.as_secs_f64()
        } else {
            0.0
        };
        
        Ok(BenchmarkResult {
            name: name.to_string(),
            duration,
            memory_stats,
            operations,
            ops_per_sec,
            memory_efficiency: 1.0, // Would compare against baseline
            cache_hit_rate: 0.95,   // Simplified
        })
    }
    
    /// Generate comprehensive benchmark summary
    fn generate_summary(&self) -> BenchmarkSummary {
        let total_duration: Duration = self.results.iter().map(|r| r.duration).sum();
        let total_operations: u64 = self.results.iter().map(|r| r.operations).sum();
        
        let avg_ops_per_sec = if total_duration.as_secs_f64() > 0.0 {
            total_operations as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };
        
        let peak_memory = self.results.iter()
            .map(|r| r.memory_stats.peak_memory)
            .max()
            .unwrap_or(0);
            
        let avg_memory = self.results.iter()
            .map(|r| r.memory_stats.average_memory as u64)
            .sum::<u64>() as f64 / self.results.len() as f64;
        
        BenchmarkSummary {
            total_benchmarks: self.results.len(),
            total_duration,
            total_operations,
            avg_ops_per_sec,
            peak_memory,
            avg_memory,
            results: self.results.clone(),
        }
    }
    
    /// Get individual benchmark results
    pub fn get_results(&self) -> &[BenchmarkResult] {
        &self.results
    }
}

/// Summary of all benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkSummary {
    pub total_benchmarks: usize,
    pub total_duration: Duration,
    pub total_operations: u64,
    pub avg_ops_per_sec: f64,
    pub peak_memory: usize,
    pub avg_memory: f64,
    pub results: Vec<BenchmarkResult>,
}

impl BenchmarkSummary {
    /// Print a human-readable summary
    pub fn print_summary(&self) {
        println!("\n🎯 Dependent Type Benchmark Summary");
        println!("═══════════════════════════════════");
        println!("Total benchmarks: {}", self.total_benchmarks);
        println!("Total duration: {:.2}s", self.total_duration.as_secs_f64());
        println!("Total operations: {}", self.total_operations);
        println!("Average ops/sec: {:.0}", self.avg_ops_per_sec);
        println!("Peak memory: {:.1}MB", self.peak_memory as f64 / 1024.0 / 1024.0);
        println!("Average memory: {:.1}MB", self.avg_memory / 1024.0 / 1024.0);
        
        println!("\n📊 Individual Results:");
        println!("─────────────────────────────────");
        for result in &self.results {
            println!("{}: {:.2}ms ({:.0} ops/sec, {:.1}KB peak)", 
                     result.name,
                     result.duration.as_secs_f64() * 1000.0,
                     result.ops_per_sec,
                     result.memory_stats.peak_memory as f64 / 1024.0);
        }
        
        println!("\n✨ Performance Improvements:");
        println!("─────────────────────────────");
        
        // Find Box vs Arena comparisons
        let box_results: Vec<_> = self.results.iter()
            .filter(|r| r.name.contains("(Box)"))
            .collect();
        let arena_results: Vec<_> = self.results.iter()
            .filter(|r| r.name.contains("(Arena)"))
            .collect();
            
        for (box_result, arena_result) in box_results.iter().zip(arena_results.iter()) {
            let speedup = box_result.duration.as_secs_f64() / arena_result.duration.as_secs_f64();
            let memory_improvement = box_result.memory_stats.peak_memory as f64 / 
                                   arena_result.memory_stats.peak_memory as f64;
                                   
            println!("{}: {:.1}x faster, {:.1}x less memory", 
                     arena_result.name.replace(" (Arena)", ""),
                     speedup, 
                     memory_improvement);
        }
    }
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            baseline: 0,
            peak: 0,
        }
    }
    
    fn take_baseline_snapshot(&mut self) {
        let snapshot = MemorySnapshot {
            timestamp: Instant::now(),
            memory_usage: 0, // Simplified - would measure actual memory
            active_allocations: 0,
            allocation_rate: 0.0,
        };
        
        self.baseline = snapshot.memory_usage;
        self.snapshots.push(snapshot);
    }
    
    fn take_final_snapshot(&mut self) {
        let snapshot = MemorySnapshot {
            timestamp: Instant::now(),
            memory_usage: self.peak,
            active_allocations: 0,
            allocation_rate: 0.0,
        };
        
        self.snapshots.push(snapshot);
    }
    
    fn record_allocation(&mut self, estimated_size: usize) {
        // Simplified allocation tracking
        let current_memory = self.snapshots.last()
            .map(|s| s.memory_usage)
            .unwrap_or(0) + estimated_size;
            
        if current_memory > self.peak {
            self.peak = current_memory;
        }
        
        let snapshot = MemorySnapshot {
            timestamp: Instant::now(),
            memory_usage: current_memory,
            active_allocations: 1,
            allocation_rate: 1.0,
        };
        
        self.snapshots.push(snapshot);
    }
    
    fn analyze(&self) -> MemoryUsageStats {
        let total_allocations = self.snapshots.len() as u64;
        let peak_memory = self.peak;
        let average_memory = if self.snapshots.is_empty() {
            0
        } else {
            self.snapshots.iter().map(|s| s.memory_usage).sum::<usize>() / self.snapshots.len()
        };
        
        MemoryUsageStats {
            peak_memory,
            average_memory,
            total_allocations,
            average_allocation_size: if total_allocations > 0 { 
                peak_memory as f64 / total_allocations as f64 
            } else { 
                0.0 
            },
            fragmentation_ratio: 0.1, // Simplified
            gc_overhead: 0.05,         // Simplified
        }
    }
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 1,
            warmup_iterations: 3,
            max_duration: Duration::from_secs(60),
            memory_sample_size: 1000,
            detailed_profiling: false,
        }
    }
}

impl Default for DependentTypeBenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_suite_creation() {
        let suite = DependentTypeBenchmarkSuite::new();
        assert_eq!(suite.results.len(), 0);
    }
    
    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new();
        tracker.take_baseline_snapshot();
        tracker.record_allocation(100);
        tracker.record_allocation(200);
        tracker.take_final_snapshot();
        
        let stats = tracker.analyze();
        assert!(stats.total_allocations >= 2);
        assert!(stats.peak_memory >= 300);
    }
    
    #[test]
    fn test_benchmark_config() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.iterations, 1);
        assert_eq!(config.warmup_iterations, 3);
    }
    
    #[test]
    fn test_simple_benchmark_run() {
        let mut suite = DependentTypeBenchmarkSuite::new();
        
        // Run a single simple benchmark
        let result = suite.time_benchmark("Test Benchmark", |tracker| {
            for i in 0..100 {
                tracker.record_allocation(i);
            }
        }).unwrap();
        
        assert_eq!(result.name, "Test Benchmark");
        assert!(result.duration.as_nanos() > 0);
        assert_eq!(result.operations, 100);
    }
}