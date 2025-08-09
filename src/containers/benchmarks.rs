//! Benchmark suite for container performance testing.
//!
//! This module provides comprehensive benchmarks for all container implementations
//! to measure and compare performance characteristics.

use crate::eval::value::Value;
use super::*;
use std::time::{Duration, Instant};

/// Benchmark result for a single operation
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the operation being benchmarked
    pub operation: String,
    /// Type of container being benchmarked
    pub container_type: String,
    /// Size of the dataset used in the benchmark
    pub size: usize,
    /// Time taken to complete the operation
    pub duration: Duration,
    /// Number of operations completed per second
    pub operations_per_second: f64,
    /// Optional memory usage measurement in bytes
    pub memory_usage: Option<usize>,
}

impl BenchmarkResult {
    /// Creates a new benchmark result with calculated operations per second.
    pub fn new(
        operation: impl Into<String>,
        container_type: impl Into<String>,
        size: usize,
        duration: Duration,
    ) -> Self {
        let ops_per_sec = if duration.as_secs_f64() > 0.0 {
            size as f64 / duration.as_secs_f64()
        } else {
            f64::INFINITY
        };

        Self {
            operation: operation.into(),
            container_type: container_type.into(),
            size,
            duration,
            operations_per_second: ops_per_sec,
            memory_usage: None,
        }
    }

    /// Sets the memory usage for this benchmark result.
    pub fn with_memory_usage(mut self, memory_usage: usize) -> Self {
        self.memory_usage = Some(memory_usage);
        self
    }
}

impl std::fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}: {} items in {:?} ({:.2} ops/sec)",
            self.container_type,
            self.operation,
            self.size,
            self.duration,
            self.operations_per_second
        )?;

        if let Some(memory) = self.memory_usage {
            write!(f, ", {memory} bytes")?;
        }

        Ok(())
    }
}

/// Comprehensive benchmark suite for all container types
pub struct ContainerBenchmarks {
    sizes: Vec<usize>,
    iterations: usize,
}

impl ContainerBenchmarks {
    /// Creates a new benchmark suite
    pub fn new() -> Self {
        Self {
            sizes: vec![100, 1000, 10000, 100000],
            iterations: 3,
        }
    }

    /// Creates a benchmark suite with custom parameters
    pub fn with_params(sizes: Vec<usize>, iterations: usize) -> Self {
        Self { sizes, iterations }
    }

    /// Runs all benchmarks and returns results
    pub fn run_all(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        for &size in &self.sizes {
            results.extend(self.benchmark_hash_tables(size));
            results.extend(self.benchmark_ideques(size));
            results.extend(self.benchmark_priority_queues(size));
            results.extend(self.benchmark_ordered_sets(size));
            results.extend(self.benchmark_list_queues(size));
            results.extend(self.benchmark_random_access_lists(size));
        }

        results
    }

    /// Benchmarks hash table implementations
    pub fn benchmark_hash_tables(&self, size: usize) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // Benchmark ThreadSafeHashTable
        {
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let table = ThreadSafeHashTable::new();
                let start = Instant::now();

                for i in 0..size {
                    table.insert(Value::number(i as f64), Value::string(format!("value{i}")));
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("insert", "ThreadSafeHashTable", size, avg_duration));

            // Benchmark lookups
            let table = ThreadSafeHashTable::new();
            for i in 0..size {
                table.insert(Value::number(i as f64), Value::string(format!("value{i}")));
            }

            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let start = Instant::now();

                for i in 0..size {
                    let _ = table.get(&Value::number(i as f64));
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("lookup", "ThreadSafeHashTable", size, avg_duration));
        }

        // Compare with standard HashMap
        {
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let mut table = std::collections::HashMap::new();
                let start = Instant::now();

                for i in 0..size {
                    table.insert(i, format!("value{i}"));
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("insert", "std::HashMap", size, avg_duration));
        }

        results
    }

    /// Benchmarks ideque implementations
    pub fn benchmark_ideques(&self, size: usize) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // Benchmark PersistentIdeque
        {
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let mut ideque = PersistentIdeque::new();
                let start = Instant::now();

                for i in 0..size {
                    ideque = ideque.cons(Value::number(i as f64));
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("cons_front", "PersistentIdeque", size, avg_duration));

            // Benchmark snoc (add to back)
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let mut ideque = PersistentIdeque::new();
                let start = Instant::now();

                for i in 0..size {
                    ideque = ideque.snoc(Value::number(i as f64));
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("snoc_back", "PersistentIdeque", size, avg_duration));
        }

        // Compare with VecDeque
        {
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let mut deque = std::collections::VecDeque::new();
                let start = Instant::now();

                for i in 0..size {
                    deque.push_front(i);
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("push_front", "std::VecDeque", size, avg_duration));
        }

        results
    }

    /// Benchmarks priority queue implementations
    pub fn benchmark_priority_queues(&self, size: usize) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // Benchmark ThreadSafePriorityQueue
        {
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let queue = ThreadSafePriorityQueue::new();
                let start = Instant::now();

                for i in 0..size {
                    queue.insert(Value::string(format!("item{i}")), Value::number(i as f64));
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("insert", "ThreadSafePriorityQueue", size, avg_duration));

            // Benchmark extraction
            let queue = ThreadSafePriorityQueue::new();
            for i in 0..size {
                queue.insert(Value::string(format!("item{i}")), Value::number(i as f64));
            }

            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let queue_clone = queue.clone();
                let start = Instant::now();

                for _ in 0..size {
                    let _ = queue_clone.extract();
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("extract", "ThreadSafePriorityQueue", size, avg_duration));
        }

        // Compare with BinaryHeap
        {
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let mut heap = std::collections::BinaryHeap::new();
                let start = Instant::now();

                for i in 0..size {
                    heap.push(i);
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("push", "std::BinaryHeap", size, avg_duration));
        }

        results
    }

    /// Benchmarks ordered set implementations
    pub fn benchmark_ordered_sets(&self, size: usize) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // Benchmark ThreadSafeOrderedSet
        {
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let set = ThreadSafeOrderedSet::new();
                let start = Instant::now();

                for i in 0..size {
                    set.insert(Value::number(i as f64));
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("insert", "ThreadSafeOrderedSet", size, avg_duration));

            // Benchmark contains
            let set = ThreadSafeOrderedSet::new();
            for i in 0..size {
                set.insert(Value::number(i as f64));
            }

            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let start = Instant::now();

                for i in 0..size {
                    let _ = set.contains(&Value::number(i as f64));
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("contains", "ThreadSafeOrderedSet", size, avg_duration));
        }

        // Compare with BTreeSet
        {
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let mut set = std::collections::BTreeSet::new();
                let start = Instant::now();

                for i in 0..size {
                    set.insert(i);
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("insert", "std::BTreeSet", size, avg_duration));
        }

        results
    }

    /// Benchmarks list queue implementations
    pub fn benchmark_list_queues(&self, size: usize) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // Benchmark ThreadSafeListQueue
        {
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let queue = ThreadSafeListQueue::new();
                let start = Instant::now();

                for i in 0..size {
                    queue.enqueue(Value::number(i as f64));
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("enqueue", "ThreadSafeListQueue", size, avg_duration));

            // Benchmark dequeue
            let queue = ThreadSafeListQueue::new();
            for i in 0..size {
                queue.enqueue(Value::number(i as f64));
            }

            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let queue_clone = queue.clone();
                let start = Instant::now();

                for _ in 0..size {
                    let _ = queue_clone.dequeue();
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("dequeue", "ThreadSafeListQueue", size, avg_duration));
        }

        results
    }

    /// Benchmarks random access list implementations
    pub fn benchmark_random_access_lists(&self, size: usize) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // Benchmark ThreadSafeRandomAccessList
        {
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let list = ThreadSafeRandomAccessList::new();
                let start = Instant::now();

                for i in 0..size {
                    list.push_front(Value::number(i as f64));
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("push_front", "ThreadSafeRandomAccessList", size, avg_duration));

            // Benchmark random access
            let list = ThreadSafeRandomAccessList::new();
            for i in 0..size {
                list.push_front(Value::number(i as f64));
            }

            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let start = Instant::now();

                for i in 0..size {
                    let _ = list.get(i % list.len());
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("random_access", "ThreadSafeRandomAccessList", size, avg_duration));
        }

        // Compare with Vec
        {
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let mut vec = Vec::new();
                let start = Instant::now();

                for i in 0..size {
                    vec.insert(0, i); // Insert at front like push_front
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("insert_front", "std::Vec", size, avg_duration));

            // Benchmark random access for Vec
            let vec: Vec<usize> = (0..size).collect();
            let mut durations = Vec::new();
            for _ in 0..self.iterations {
                let start = Instant::now();

                for i in 0..size {
                    let _ = vec.get(i % vec.len());
                }

                durations.push(start.elapsed());
            }

            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            results.push(BenchmarkResult::new("random_access", "std::Vec", size, avg_duration));
        }

        results
    }

    /// Runs memory usage benchmarks
    pub fn benchmark_memory_usage(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        let size = 10000;

        // Estimate memory usage for each container type
        // Note: These are rough estimates and would need more sophisticated measurement in practice

        // Hash table memory usage
        let table = ThreadSafeHashTable::new();
        for i in 0..size {
            table.insert(Value::number(i as f64), Value::string(format!("value{i}")));
        }
        let estimated_memory = size * (std::mem::size_of::<Value>() * 2 + 64); // Key + Value + overhead
        results.push(
            BenchmarkResult::new("memory", "ThreadSafeHashTable", size, Duration::ZERO)
                .with_memory_usage(estimated_memory)
        );

        // Ideque memory usage
        let mut ideque = PersistentIdeque::new();
        for i in 0..size {
            ideque = ideque.cons(Value::number(i as f64));
        }
        let estimated_memory = size * (std::mem::size_of::<Value>() + 32); // Value + tree node overhead
        results.push(
            BenchmarkResult::new("memory", "PersistentIdeque", size, Duration::ZERO)
                .with_memory_usage(estimated_memory)
        );

        results
    }

    /// Prints benchmark results in a formatted table
    pub fn print_results(&self, results: &[BenchmarkResult]) {
        println!("\n{:=<100}", "");
        println!("{:^100}", "CONTAINER BENCHMARK RESULTS");
        println!("{:=<100}", "");

        // Group results by container type
        let mut by_container: std::collections::HashMap<String, Vec<&BenchmarkResult>> = 
            std::collections::HashMap::new();

        for result in results {
            by_container
                .entry(result.container_type.clone())
                .or_default()
                .push(result);
        }

        for (container_type, container_results) in by_container {
            println!("\n{container_type}");
            println!("{:-<50}", "");

            for result in container_results {
                println!("  {result}");
            }
        }

        println!("\n{:=<100}", "");
    }

    /// Runs performance comparison between container types
    pub fn compare_performance(&self, operation: &str, size: usize) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        match operation {
            "insert" => {
                // Compare insertion performance across container types
                results.extend(self.benchmark_hash_tables(size)
                    .into_iter()
                    .filter(|r| r.operation == "insert"));
                results.extend(self.benchmark_ordered_sets(size)
                    .into_iter()
                    .filter(|r| r.operation == "insert"));
                results.extend(self.benchmark_priority_queues(size)
                    .into_iter()
                    .filter(|r| r.operation == "insert"));
            }
            "lookup" => {
                // Compare lookup performance
                results.extend(self.benchmark_hash_tables(size)
                    .into_iter()
                    .filter(|r| r.operation == "lookup"));
                results.extend(self.benchmark_ordered_sets(size)
                    .into_iter()
                    .filter(|r| r.operation == "contains"));
            }
            _ => {
                eprintln!("Unknown operation: {operation}");
            }
        }

        results.sort_by(|a, b| a.operations_per_second.partial_cmp(&b.operations_per_second).unwrap().reverse());
        results
    }
}

impl Default for ContainerBenchmarks {
    fn default() -> Self {
        Self::new()
    }
}

/// Runs a quick benchmark suite for demonstration
pub fn run_quick_benchmark() {
    println!("Running container benchmarks...");
    
    let benchmarks = ContainerBenchmarks::with_params(vec![1000, 10000], 2);
    let results = benchmarks.run_all();
    
    benchmarks.print_results(&results);
    
    // Print top performers
    println!("\nTop performers for insertion (10000 items):");
    let insertion_results = benchmarks.compare_performance("insert", 10000);
    for (i, result) in insertion_results.iter().take(5).enumerate() {
        println!("  {}. {} - {:.2} ops/sec", i + 1, result.container_type, result.operations_per_second);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_creation() {
        let benchmarks = ContainerBenchmarks::new();
        assert!(!benchmarks.sizes.is_empty());
        assert!(benchmarks.iterations > 0);
    }

    #[test]
    fn test_benchmark_result() {
        let result = BenchmarkResult::new(
            "test_op",
            "TestContainer",
            1000,
            Duration::from_millis(100),
        );

        assert_eq!(result.operation, "test_op");
        assert_eq!(result.container_type, "TestContainer");
        assert_eq!(result.size, 1000);
        assert!(result.operations_per_second > 0.0);
    }

    #[test]
    fn test_hash_table_benchmark() {
        let benchmarks = ContainerBenchmarks::with_params(vec![100], 1);
        let results = benchmarks.benchmark_hash_tables(100);
        
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.operation == "insert"));
        assert!(results.iter().any(|r| r.operation == "lookup"));
    }

    #[test] 
    fn test_performance_comparison() {
        let benchmarks = ContainerBenchmarks::with_params(vec![100], 1);
        let results = benchmarks.compare_performance("insert", 100);
        
        assert!(!results.is_empty());
        // Results should be sorted by performance (descending)
        if results.len() > 1 {
            assert!(results[0].operations_per_second >= results[1].operations_per_second);
        }
    }
}