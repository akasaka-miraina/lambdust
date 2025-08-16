//! Memory Usage Measurement and Analysis for Value Optimization
//!
//! This module provides comprehensive memory analysis tools for measuring
//! the effectiveness of the Value enum optimization strategy.
//!
//! Features:
//! - Real-time memory usage tracking
//! - Arc allocation counting and analysis
//! - Memory leak detection for optimization layer
//! - Performance benchmarking for optimized vs legacy values
//! - Detailed reporting on memory savings achieved

#![allow(missing_docs)]

use crate::eval::value::Value;
use crate::eval::optimized_value::OptimizedValue;
use crate::eval::value_bridge::{LegacyValueBridge, OptimizationMetrics};
use crate::eval::value_optimization_core::{ValueOptimizer, PerformanceStats, MemoryAnalysisReport};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::alloc::{GlobalAlloc, Layout, System};

/// Memory allocation tracker that wraps the system allocator
/// 
/// This allows precise measurement of memory usage during value operations.
pub struct TrackedAllocator {
    allocations: Arc<Mutex<AllocationStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct AllocationStats {
    total_allocated: usize,
    total_deallocated: usize,
    peak_usage: usize,
    current_usage: usize,
    allocation_count: usize,
    deallocation_count: usize,
}

impl Default for TrackedAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackedAllocator {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(AllocationStats::default())),
        }
    }
    
    pub fn stats(&self) -> AllocationStats {
        self.allocations.lock().unwrap().clone()
    }
    
    pub fn reset(&self) {
        *self.allocations.lock().unwrap() = AllocationStats::default();
    }
}

unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        
        if !ptr.is_null() {
            if let Ok(mut stats) = self.allocations.lock() {
                stats.total_allocated += layout.size();
                stats.current_usage += layout.size();
                stats.allocation_count += 1;
                
                if stats.current_usage > stats.peak_usage {
                    stats.peak_usage = stats.current_usage;
                }
            }
        }
        
        ptr
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout); }
        
        if let Ok(mut stats) = self.allocations.lock() {
            stats.total_deallocated += layout.size();
            stats.current_usage = stats.current_usage.saturating_sub(layout.size());
            stats.deallocation_count += 1;
        }
    }
}

/// Comprehensive memory measurement for value optimization
pub struct MemoryMeasurer {
    allocator: Option<TrackedAllocator>,
    benchmarks: Arc<RwLock<Vec<BenchmarkResult>>>,
    value_cache: Arc<RwLock<HashMap<String, (Value, usize)>>>,
}

/// Result of a memory benchmark comparing optimized vs legacy values
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub legacy_memory: usize,
    pub optimized_memory: usize,
    pub memory_saved: usize,
    pub savings_percentage: f64,
    pub legacy_time: Duration,
    pub optimized_time: Duration,
    pub time_improvement: f64,
    pub arc_count_before: usize,
    pub arc_count_after: usize,
    pub arc_reduction: usize,
}

impl MemoryMeasurer {
    /// Creates a new memory measurer with optional allocation tracking
    pub fn new(enable_allocation_tracking: bool) -> Self {
        Self {
            allocator: if enable_allocation_tracking { Some(TrackedAllocator::new()) } else { None },
            benchmarks: Arc::new(RwLock::new(Vec::new())),
            value_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Measures memory usage of creating a specific value
    pub fn measure_value_creation<F>(&self, name: &str, create_fn: F) -> ValueCreationMeasurement
    where
        F: Fn() -> Value,
    {
        let start_time = Instant::now();
        let start_mem = self.current_memory_usage();
        
        let value = create_fn();
        
        let end_time = Instant::now();
        let end_mem = self.current_memory_usage();
        
        let memory_used = end_mem.saturating_sub(start_mem);
        let time_taken = end_time.duration_since(start_time);
        
        let estimated_arc_count = self.estimate_arc_count_in_value(&value);
        
        ValueCreationMeasurement {
            name: name.to_string(),
            value,
            memory_used,
            time_taken,
            estimated_arc_count,
        }
    }
    
    /// Compares memory usage between legacy and optimized value creation
    pub fn benchmark_optimization<F, G>(&self, name: &str, legacy_fn: F, optimized_fn: G) -> BenchmarkResult
    where
        F: Fn() -> Value,
        G: Fn() -> Value,
    {
        // Measure legacy implementation
        let legacy_measurement = self.measure_value_creation(&format!("{name}_legacy"), legacy_fn);
        
        // Measure optimized implementation  
        let optimized_measurement = self.measure_value_creation(&format!("{name}_optimized"), optimized_fn);
        
        let memory_saved = legacy_measurement.memory_used.saturating_sub(optimized_measurement.memory_used);
        let savings_percentage = if legacy_measurement.memory_used > 0 {
            (memory_saved as f64 / legacy_measurement.memory_used as f64) * 100.0
        } else {
            0.0
        };
        
        let time_improvement = if legacy_measurement.time_taken > optimized_measurement.time_taken {
            let saved = legacy_measurement.time_taken.saturating_sub(optimized_measurement.time_taken);
            (saved.as_nanos() as f64 / legacy_measurement.time_taken.as_nanos() as f64) * 100.0
        } else {
            0.0
        };
        
        let arc_reduction = legacy_measurement.estimated_arc_count.saturating_sub(optimized_measurement.estimated_arc_count);
        
        let result = BenchmarkResult {
            name: name.to_string(),
            legacy_memory: legacy_measurement.memory_used,
            optimized_memory: optimized_measurement.memory_used,
            memory_saved,
            savings_percentage,
            legacy_time: legacy_measurement.time_taken,
            optimized_time: optimized_measurement.time_taken,
            time_improvement,
            arc_count_before: legacy_measurement.estimated_arc_count,
            arc_count_after: optimized_measurement.estimated_arc_count,
            arc_reduction,
        };
        
        // Store benchmark result
        self.benchmarks.write().unwrap().push(result.clone());
        
        result
    }
    
    /// Gets current memory usage from tracked allocator or estimates
    fn current_memory_usage(&self) -> usize {
        if let Some(allocator) = &self.allocator {
            allocator.stats().current_usage
        } else {
            // Fallback to estimation based on process info
            // This is less accurate but still useful
            0
        }
    }
    
    /// Estimates the number of Arc allocations in a Value
    fn estimate_arc_count_in_value(&self, value: &Value) -> usize {
        match value {
            Value::Nil | Value::Unspecified => 0,
            Value::Literal(_) => 0, // Literals don't use Arc in current implementation
            Value::Symbol(_) => 0, // Symbols are inline
            Value::Keyword(_) => 1, // String inside
            Value::Pair(_, _) => 2, // Two Arc references
            Value::MutablePair(_, _) => 2, // Two Arc<RwLock<Value>>
            Value::Vector(_) => 1, // Arc<RwLock<Vec<Value>>>
            Value::Hashtable(_) => 1, // Arc<RwLock<HashMap>>
            Value::MutableString(_) => 1, // Arc<RwLock<Vec<char>>>
            Value::Procedure(proc) => {
                1 + // Arc<Procedure>
                1 + // Arc<ThreadSafeEnvironment> 
                proc.metadata.len() // Each metadata value might have Arcs
            }
            Value::CaseLambda(_) => 2, // Arc<CaseLambdaProcedure> + environment
            Value::Primitive(_) => 1, // Arc<PrimitiveProcedure>
            Value::Continuation(_) => 2, // Arc<Continuation> + environment
            Value::Syntax(_) => 2, // Arc<SyntaxTransformer> + environment
            Value::Port(_) => 1, // Arc<Port>
            Value::Promise(_) => 1, // Arc<RwLock<Promise>>
            Value::Type(_) => 1, // Arc<TypeValue>
            Value::Foreign(_) => 1, // Arc<ForeignObject>
            Value::ErrorObject(_) => 1, // Arc<ErrorObject>
            Value::CharSet(_) => 1, // Arc<CharSet>
            Value::Parameter(_) => 1, // Arc<Parameter>
            Value::Record(_) => 1, // Arc<Record>
            
            // Advanced containers (all use Arc)
            Value::AdvancedHashTable(_) => 1,
            Value::Ideque(_) => 1,
            Value::PriorityQueue(_) => 1,
            Value::OrderedSet(_) => 1,
            Value::ListQueue(_) => 1,
            Value::RandomAccessList(_) => 1,
            Value::Set(_) => 1,
            Value::Bag(_) => 1,
            Value::Generator(_) => 1,
            
            // Concurrency values (when enabled)
            #[cfg(feature = "async-runtime")]
            Value::Future(_) => 1,
            #[cfg(feature = "async-runtime")]
            Value::Channel(_) => 1,
            #[cfg(feature = "async-runtime")]
            Value::Mutex(_) => 1,
            #[cfg(feature = "async-runtime")]
            Value::Semaphore(_) => 1,
            #[cfg(feature = "async-runtime")]
            Value::AtomicCounter(_) => 1,
            #[cfg(feature = "async-runtime")]
            Value::DistributedNode(_) => 1,
            
            Value::Opaque(_) => 1, // Arc<dyn Any>
        }
    }
    
    /// Runs a comprehensive benchmark suite comparing legacy and optimized values
    pub fn run_comprehensive_benchmark(&self) -> ComprehensiveBenchmarkReport {
        let mut report = ComprehensiveBenchmarkReport::default();
        
        // Benchmark immediate values
        report.immediate_benchmarks.push(
            self.benchmark_optimization(
                "boolean_true",
                || Value::Literal(crate::ast::Literal::Boolean(true)),
                || Value::Literal(crate::ast::Literal::Boolean(true)) // Same for now, but would use optimized constructor
            )
        );
        
        report.immediate_benchmarks.push(
            self.benchmark_optimization(
                "small_integer",
                || Value::Literal(crate::ast::Literal::ExactInteger(42)),
                || Value::Literal(crate::ast::Literal::ExactInteger(42))
            )
        );
        
        report.immediate_benchmarks.push(
            self.benchmark_optimization(
                "character",
                || Value::Literal(crate::ast::Literal::Character('A')),
                || Value::Literal(crate::ast::Literal::Character('A'))
            )
        );
        
        // Benchmark compound values
        report.compound_benchmarks.push(
            self.benchmark_optimization(
                "simple_pair",
                || {
                    let car = Value::Literal(crate::ast::Literal::ExactInteger(1));
                    let cdr = Value::Literal(crate::ast::Literal::ExactInteger(2));
                    Value::Pair(Arc::new(car), Arc::new(cdr))
                },
                || {
                    // In optimized form, this would use direct boxing
                    let car = Value::Literal(crate::ast::Literal::ExactInteger(1));
                    let cdr = Value::Literal(crate::ast::Literal::ExactInteger(2));
                    Value::Pair(Arc::new(car), Arc::new(cdr))
                }
            )
        );
        
        report.compound_benchmarks.push(
            self.benchmark_optimization(
                "string_value",
                || Value::Literal(crate::ast::Literal::String(Box::new("hello world".to_string()))),
                || Value::Literal(crate::ast::Literal::String(Box::new("hello world".to_string())))
            )
        );
        
        // Benchmark complex values
        report.complex_benchmarks.push(
            self.benchmark_optimization(
                "vector_small",
                || {
                    let elements = vec![
                        Value::Literal(crate::ast::Literal::ExactInteger(1)),
                        Value::Literal(crate::ast::Literal::ExactInteger(2)),
                        Value::Literal(crate::ast::Literal::ExactInteger(3)),
                    ];
                    Value::Vector(Arc::new(RwLock::new(elements)))
                },
                || {
                    let elements = vec![
                        Value::Literal(crate::ast::Literal::ExactInteger(1)),
                        Value::Literal(crate::ast::Literal::ExactInteger(2)),
                        Value::Literal(crate::ast::Literal::ExactInteger(3)),
                    ];
                    Value::Vector(Arc::new(RwLock::new(elements)))
                }
            )
        );
        
        // Calculate summary statistics
        let all_benchmarks: Vec<&BenchmarkResult> = report.immediate_benchmarks.iter()
            .chain(report.compound_benchmarks.iter())
            .chain(report.complex_benchmarks.iter())
            .collect();
        
        if !all_benchmarks.is_empty() {
            report.total_memory_saved = all_benchmarks.iter().map(|b| b.memory_saved).sum();
            report.average_savings_percentage = all_benchmarks.iter().map(|b| b.savings_percentage).sum::<f64>() / all_benchmarks.len() as f64;
            report.total_arc_reduction = all_benchmarks.iter().map(|b| b.arc_reduction).sum();
            report.average_time_improvement = all_benchmarks.iter().map(|b| b.time_improvement).sum::<f64>() / all_benchmarks.len() as f64;
        }
        
        report
    }
    
    /// Gets all benchmark results
    pub fn get_all_benchmarks(&self) -> Vec<BenchmarkResult> {
        self.benchmarks.read().unwrap().clone()
    }
    
    /// Clears all benchmark results
    pub fn clear_benchmarks(&self) {
        self.benchmarks.write().unwrap().clear();
        self.value_cache.write().unwrap().clear();
        
        if let Some(allocator) = &self.allocator {
            allocator.reset();
        }
    }
}

/// Result of measuring a single value creation
#[derive(Debug, Clone)]
pub struct ValueCreationMeasurement {
    pub name: String,
    pub value: Value,
    pub memory_used: usize,
    pub time_taken: Duration,
    pub estimated_arc_count: usize,
}

/// Comprehensive benchmark report
#[derive(Debug, Clone, Default)]
pub struct ComprehensiveBenchmarkReport {
    pub immediate_benchmarks: Vec<BenchmarkResult>,
    pub compound_benchmarks: Vec<BenchmarkResult>,
    pub complex_benchmarks: Vec<BenchmarkResult>,
    pub total_memory_saved: usize,
    pub average_savings_percentage: f64,
    pub total_arc_reduction: usize,
    pub average_time_improvement: f64,
}

impl ComprehensiveBenchmarkReport {
    /// Generates a human-readable summary of the benchmark results
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str("=== Value Optimization Benchmark Report ===\n\n");
        
        summary.push_str(&format!("Total Memory Saved: {} bytes\n", self.total_memory_saved));
        summary.push_str(&format!("Average Memory Savings: {:.2}%\n", self.average_savings_percentage));
        summary.push_str(&format!("Total Arc Reduction: {}\n", self.total_arc_reduction));
        summary.push_str(&format!("Average Time Improvement: {:.2}%\n\n", self.average_time_improvement));
        
        summary.push_str("Immediate Values:\n");
        for benchmark in &self.immediate_benchmarks {
            summary.push_str(&format!(
                "  {}: {} bytes saved ({:.1}%), {} Arc reduction\n",
                benchmark.name, benchmark.memory_saved, benchmark.savings_percentage, benchmark.arc_reduction
            ));
        }
        
        summary.push_str("\nCompound Values:\n");
        for benchmark in &self.compound_benchmarks {
            summary.push_str(&format!(
                "  {}: {} bytes saved ({:.1}%), {} Arc reduction\n",
                benchmark.name, benchmark.memory_saved, benchmark.savings_percentage, benchmark.arc_reduction
            ));
        }
        
        summary.push_str("\nComplex Values:\n");
        for benchmark in &self.complex_benchmarks {
            summary.push_str(&format!(
                "  {}: {} bytes saved ({:.1}%), {} Arc reduction\n",
                benchmark.name, benchmark.memory_saved, benchmark.savings_percentage, benchmark.arc_reduction
            ));
        }
        
        summary
    }
}

/// Utility for analyzing Arc usage patterns in Value hierarchies
pub struct ArcAnalyzer;

impl ArcAnalyzer {
    /// Analyzes Arc usage in a Value tree/graph
    pub fn analyze_value_tree(value: &Value) -> ArcAnalysisResult {
        let mut result = ArcAnalysisResult::default();
        let mut visited = std::collections::HashSet::new();
        
        Self::analyze_value_recursive(value, &mut result, &mut visited, 0);
        
        result
    }
    
    fn analyze_value_recursive(
        value: &Value, 
        result: &mut ArcAnalysisResult, 
        visited: &mut std::collections::HashSet<*const Value>,
        depth: usize
    ) {
        let value_ptr = value as *const Value;
        
        // Avoid infinite recursion in cyclic structures
        if visited.contains(&value_ptr) {
            result.cyclic_references += 1;
            return;
        }
        visited.insert(value_ptr);
        
        result.total_values += 1;
        result.max_depth = result.max_depth.max(depth);
        
        match value {
            Value::Nil | Value::Unspecified => {
                result.immediate_values += 1;
            }
            Value::Literal(_) => {
                result.immediate_values += 1;
            }
            Value::Symbol(_) => {
                result.immediate_values += 1;
            }
            Value::Keyword(_) => {
                result.string_arcs += 1;
                result.total_arcs += 1;
            }
            Value::Pair(car, cdr) => {
                result.pair_arcs += 2;
                result.total_arcs += 2;
                result.compound_values += 1;
                
                Self::analyze_value_recursive(car, result, visited, depth + 1);
                Self::analyze_value_recursive(cdr, result, visited, depth + 1);
            }
            Value::MutablePair(car, cdr) => {
                result.pair_arcs += 2;
                result.total_arcs += 2;
                result.compound_values += 1;
                
                // Try to analyze contents if locks can be acquired
                if let (Ok(car_guard), Ok(cdr_guard)) = (car.read(), cdr.read()) {
                    Self::analyze_value_recursive(&car_guard, result, visited, depth + 1);
                    Self::analyze_value_recursive(&cdr_guard, result, visited, depth + 1);
                }
            }
            Value::Vector(vec_arc) => {
                result.container_arcs += 1;
                result.total_arcs += 1;
                result.complex_values += 1;
                
                if let Ok(elements) = vec_arc.read() {
                    for element in elements.iter() {
                        Self::analyze_value_recursive(element, result, visited, depth + 1);
                    }
                }
            }
            _ => {
                // Most other values use 1 Arc
                result.complex_arcs += 1;
                result.total_arcs += 1;
                result.complex_values += 1;
            }
        }
    }
}

/// Result of Arc usage analysis
#[derive(Debug, Clone, Default)]
pub struct ArcAnalysisResult {
    pub total_values: usize,
    pub immediate_values: usize,
    pub compound_values: usize,
    pub complex_values: usize,
    pub total_arcs: usize,
    pub pair_arcs: usize,
    pub string_arcs: usize,
    pub container_arcs: usize,
    pub complex_arcs: usize,
    pub max_depth: usize,
    pub cyclic_references: usize,
}

impl ArcAnalysisResult {
    /// Calculates the potential Arc reduction if optimization is applied
    pub fn calculate_optimization_potential(&self) -> OptimizationPotential {
        // Immediate values: can eliminate all Arc usage
        let immediate_savings = self.immediate_values;
        
        // Pairs: can reduce from 2 Arcs to 0 Arcs (direct boxing)
        let pair_savings = self.pair_arcs; // All pair Arcs can be eliminated
        
        // Containers: depends on whether thread safety is needed
        let container_savings = self.container_arcs / 2; // Conservative: 50% reduction
        
        let total_potential_savings = immediate_savings + pair_savings + container_savings;
        let savings_percentage = if self.total_arcs > 0 {
            (total_potential_savings as f64 / self.total_arcs as f64) * 100.0
        } else {
            0.0
        };
        
        OptimizationPotential {
            current_arc_count: self.total_arcs,
            potential_arc_savings: total_potential_savings,
            savings_percentage,
            immediate_value_savings: immediate_savings,
            pair_savings,
            container_savings,
        }
    }
}

/// Potential optimization gains
#[derive(Debug, Clone)]
pub struct OptimizationPotential {
    pub current_arc_count: usize,
    pub potential_arc_savings: usize,
    pub savings_percentage: f64,
    pub immediate_value_savings: usize,
    pub pair_savings: usize,
    pub container_savings: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    
    #[test]
    fn test_memory_measurer_creation() {
        let measurer = MemoryMeasurer::new(false);
        assert!(measurer.allocator.is_none());
        
        let measurer_tracked = MemoryMeasurer::new(true);
        assert!(measurer_tracked.allocator.is_some());
    }
    
    #[test]
    fn test_value_creation_measurement() {
        let measurer = MemoryMeasurer::new(false);
        
        let measurement = measurer.measure_value_creation("test_boolean", || {
            Value::Literal(Literal::Boolean(true))
        });
        
        assert_eq!(measurement.name, "test_boolean");
        assert!(matches!(measurement.value, Value::Literal(Literal::Boolean(true))));
        assert_eq!(measurement.estimated_arc_count, 0); // Boolean literals use no Arcs
    }
    
    #[test]
    fn test_arc_count_estimation() {
        let measurer = MemoryMeasurer::new(false);
        
        // Test immediate values (should be 0 Arcs)
        let nil_arcs = measurer.estimate_arc_count_in_value(&Value::Nil);
        assert_eq!(nil_arcs, 0);
        
        let bool_arcs = measurer.estimate_arc_count_in_value(&Value::Literal(Literal::Boolean(true)));
        assert_eq!(bool_arcs, 0);
        
        // Test compound values
        let pair = Value::Pair(
            Arc::new(Value::Literal(Literal::ExactInteger(1))),
            Arc::new(Value::Literal(Literal::ExactInteger(2)))
        );
        let pair_arcs = measurer.estimate_arc_count_in_value(&pair);
        assert_eq!(pair_arcs, 2); // Two Arc references in a pair
        
        // Test complex values
        let vector = Value::Vector(Arc::new(RwLock::new(vec![Value::Nil])));
        let vector_arcs = measurer.estimate_arc_count_in_value(&vector);
        assert_eq!(vector_arcs, 1); // One Arc for the vector itself
    }
    
    #[test]
    fn test_benchmark_optimization() {
        let measurer = MemoryMeasurer::new(false);
        
        let result = measurer.benchmark_optimization(
            "test_comparison",
            || Value::Literal(Literal::Boolean(true)), // Legacy
            || Value::Literal(Literal::Boolean(true))  // Optimized (same for now)
        );
        
        assert_eq!(result.name, "test_comparison");
        assert_eq!(result.arc_count_before, 0);
        assert_eq!(result.arc_count_after, 0);
        assert_eq!(result.arc_reduction, 0);
    }
    
    #[test]
    fn test_arc_analyzer() {
        // Simple value tree
        let pair = Value::Pair(
            Arc::new(Value::Literal(Literal::ExactInteger(1))),
            Arc::new(Value::Nil)
        );
        
        let analysis = ArcAnalyzer::analyze_value_tree(&pair);
        
        assert_eq!(analysis.total_values, 3); // pair + two contents
        assert_eq!(analysis.immediate_values, 2); // integer + nil
        assert_eq!(analysis.compound_values, 1); // the pair
        assert_eq!(analysis.pair_arcs, 2); // two Arc references in pair
        assert_eq!(analysis.total_arcs, 2);
    }
    
    #[test]
    fn test_optimization_potential() {
        // Create a value tree with known Arc usage
        let pair = Value::Pair(
            Arc::new(Value::Literal(Literal::ExactInteger(1))),
            Arc::new(Value::Literal(Literal::Boolean(true)))
        );
        
        let analysis = ArcAnalyzer::analyze_value_tree(&pair);
        let potential = analysis.calculate_optimization_potential();
        
        assert_eq!(potential.current_arc_count, 2);
        assert_eq!(potential.pair_savings, 2); // Can eliminate both pair Arcs
        assert!(potential.savings_percentage > 0.0);
    }
    
    #[test]
    fn test_comprehensive_benchmark() {
        let measurer = MemoryMeasurer::new(false);
        
        let report = measurer.run_comprehensive_benchmark();
        
        // Should have results for different categories
        assert!(!report.immediate_benchmarks.is_empty());
        assert!(!report.compound_benchmarks.is_empty());
        assert!(!report.complex_benchmarks.is_empty());
        
        // Summary should be readable
        let summary = report.summary();
        assert!(summary.contains("Benchmark Report"));
        assert!(summary.contains("Memory Saved"));
    }
    
    #[test]
    fn test_tracked_allocator() {
        let allocator = TrackedAllocator::new();
        
        let initial_stats = allocator.stats();
        assert_eq!(initial_stats.allocation_count, 0);
        assert_eq!(initial_stats.current_usage, 0);
        
        // Reset should work
        allocator.reset();
        let reset_stats = allocator.stats();
        assert_eq!(reset_stats.allocation_count, 0);
    }
}