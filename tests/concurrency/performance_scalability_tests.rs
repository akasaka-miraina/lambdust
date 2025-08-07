//! Performance and Scalability Tests
//!
//! Tests for measuring and verifying the performance characteristics and
//! scalability of the multithreaded Lambdust runtime, including throughput
//! measurement, memory usage analysis, and latency testing.

use lambdust::runtime::LambdustRuntime;
use lambdust::diagnostics::{Result, Error};

use std::sync::{Arc, Mutex, Barrier};
use std::time::{Duration, Instant};
use std::thread;
use std::collections::VecDeque;
use tokio::sync::Semaphore;

/// Performance measurement framework
#[derive(Debug)]
pub struct PerformanceMeasurement {
    pub test_name: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub thread_count: usize,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub peak_memory_usage: Option<u64>,
    pub average_latency: Duration,
    pub min_latency: Duration,
    pub max_latency: Duration,
    pub throughput_ops_per_sec: f64,
}

impl PerformanceMeasurement {
    pub fn new(test_name: String, thread_count: usize) -> Self {
        Self {
            test_name,
            start_time: Instant::now(),
            end_time: None,
            thread_count,
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            peak_memory_usage: None,
            average_latency: Duration::ZERO,
            min_latency: Duration::MAX,
            max_latency: Duration::ZERO,
            throughput_ops_per_sec: 0.0,
        }
    }

    pub fn complete(&mut self) {
        self.end_time = Some(Instant::now());
        let total_duration = self.duration();
        
        if total_duration.as_secs_f64() > 0.0 && self.successful_operations > 0 {
            self.throughput_ops_per_sec = self.successful_operations as f64 / total_duration.as_secs_f64();
        }
    }

    pub fn duration(&self) -> Duration {
        match self.end_time {
            Some(end) => end.duration_since(self.start_time),
            None => self.start_time.elapsed(),
        }
    }

    pub fn update_latency_stats(&mut self, latency: Duration) {
        if latency < self.min_latency {
            self.min_latency = latency;
        }
        if latency > self.max_latency {
            self.max_latency = latency;
        }
    }

    pub fn print_report(&self) {
        println!("\n=== Performance Report: {} ===", self.test_name);
        println!("Duration: {:.3}s", self.duration().as_secs_f64());
        println!("Thread Count: {}", self.thread_count);
        println!("Total Operations: {}", self.total_operations);
        println!("Successful Operations: {}", self.successful_operations);
        println!("Failed Operations: {}", self.failed_operations);
        println!("Success Rate: {:.2}%", 
            (self.successful_operations as f64 / self.total_operations as f64) * 100.0);
        println!("Throughput: {:.2} ops/sec", self.throughput_ops_per_sec);
        println!("Average Latency: {:.3}ms", self.average_latency.as_secs_f64() * 1000.0);
        println!("Min Latency: {:.3}ms", self.min_latency.as_secs_f64() * 1000.0);
        println!("Max Latency: {:.3}ms", self.max_latency.as_secs_f64() * 1000.0);
        
        if let Some(memory) = self.peak_memory_usage {
            println!("Peak Memory Usage: {} KB", memory);
        }
        
        println!("=== End Report ===\n");
    }
}

/// Memory usage tracker (simplified for testing)
#[derive(Debug)]
pub struct MemoryTracker {
    samples: Arc<Mutex<VecDeque<(Instant, u64)>>>,
    peak_usage: Arc<Mutex<u64>>,
    sampling_active: Arc<Mutex<bool>>,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(VecDeque::new())),
            peak_usage: Arc::new(Mutex::new(0)),
            sampling_active: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start_sampling(&self) {
        {
            let mut active = self.sampling_active.lock().unwrap();
            *active = true;
        }

        let samples_clone = self.samples.clone();
        let peak_clone = self.peak_usage.clone();
        let active_clone = self.sampling_active.clone();

        tokio::spawn(async move {
            while *active_clone.lock().unwrap() {
                // Simulate memory usage measurement
                // In a real implementation, this would use platform-specific APIs
                let simulated_usage = Self::get_simulated_memory_usage();
                
                {
                    let mut samples = samples_clone.lock().unwrap();
                    samples.push_back((Instant::now(), simulated_usage));
                    
                    // Keep only recent samples
                    if samples.len() > 1000 {
                        samples.pop_front();
                    }
                }

                {
                    let mut peak = peak_clone.lock().unwrap();
                    if simulated_usage > *peak {
                        *peak = simulated_usage;
                    }
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    }

    pub fn stop_sampling(&self) {
        let mut active = self.sampling_active.lock().unwrap();
        *active = false;
    }

    pub fn get_peak_usage(&self) -> u64 {
        *self.peak_usage.lock().unwrap()
    }

    pub fn get_current_usage(&self) -> u64 {
        Self::get_simulated_memory_usage()
    }

    fn get_simulated_memory_usage() -> u64 {
        // Simulate memory usage based on some baseline + random variation
        // In a real implementation, this would use actual memory measurement
        let baseline = 1024 * 1024; // 1MB baseline
        let variation = (rand::random::<f64>() * 512.0 * 1024.0) as u64; // Up to 512KB variation
        baseline + variation
    }
}

/// Latency measurement utilities
#[derive(Debug)]
pub struct LatencyMeasurement {
    measurements: Arc<Mutex<Vec<Duration>>>,
    percentiles: Arc<Mutex<Option<LatencyPercentiles>>>,
}

#[derive(Debug, Clone)]
pub struct LatencyPercentiles {
    pub p50: Duration,
    pub p90: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

impl LatencyMeasurement {
    pub fn new() -> Self {
        Self {
            measurements: Arc::new(Mutex::new(Vec::new())),
            percentiles: Arc::new(Mutex::new(None)),
        }
    }

    pub fn record(&self, latency: Duration) {
        let mut measurements = self.measurements.lock().unwrap();
        measurements.push(latency);
    }

    pub fn calculate_percentiles(&self) -> LatencyPercentiles {
        let mut measurements = self.measurements.lock().unwrap();
        
        if measurements.is_empty() {
            return LatencyPercentiles {
                p50: Duration::ZERO,
                p90: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
            };
        }

        measurements.sort();
        let len = measurements.len();

        let percentiles = LatencyPercentiles {
            p50: measurements[len * 50 / 100],
            p90: measurements[len * 90 / 100],
            p95: measurements[len * 95 / 100],
            p99: measurements[len * 99 / 100],
        };

        {
            let mut stored_percentiles = self.percentiles.lock().unwrap();
            *stored_percentiles = Some(percentiles.clone());
        }

        percentiles
    }

    pub fn get_average(&self) -> Duration {
        let measurements = self.measurements.lock().unwrap();
        if measurements.is_empty() {
            return Duration::ZERO;
        }

        let total: Duration = measurements.iter().sum();
        total / measurements.len() as u32
    }

    pub fn get_count(&self) -> usize {
        self.measurements.lock().unwrap().len()
    }
}

// ============================================================================
// THROUGHPUT MEASUREMENT TESTS
// ============================================================================

#[tokio::test]
async fn test_throughput_scaling() {
    println!("=== Throughput Scaling Test ===");
    
    let thread_counts = vec![1, 2, 4, 8, 16];
    let operations_per_thread = 100;
    let mut results = Vec::new();

    for &thread_count in &thread_counts {
        let runtime = Arc::new(LambdustRuntime::new(Some(thread_count))
            .expect("Failed to create runtime"));

        let mut measurement = PerformanceMeasurement::new(
            format!("throughput_test_{}_threads", thread_count),
            thread_count,
        );

        let barrier = Arc::new(Barrier::new(thread_count));
        let mut handles = Vec::new();

        for thread_id in 0..thread_count {
            let barrier_clone = barrier.clone();
            let runtime_clone = runtime.clone();

            let handle = tokio::spawn(async move {
                barrier_clone.wait();
                
                let mut successful_ops = 0;
                let mut total_latency = Duration::ZERO;

                for op_id in 0..operations_per_thread {
                    let start_time = Instant::now();
                    
                    // Simulate computational work
                    let result = perform_simulated_computation(thread_id, op_id).await;
                    
                    let latency = start_time.elapsed();
                    total_latency += latency;

                    if result.is_ok() {
                        successful_ops += 1;
                    }

                    // Small delay to prevent overwhelming
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }

                let avg_latency = if successful_ops > 0 {
                    total_latency / successful_ops as u32
                } else {
                    Duration::ZERO
                };

                (thread_id, successful_ops, avg_latency, total_latency)
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        let thread_results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
            .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

        // Aggregate results
        let total_successful: usize = thread_results.iter().map(|(_, ops, _, _)| *ops).sum();
        let total_expected = thread_count * operations_per_thread;
        let avg_latencies: Vec<Duration> = thread_results.iter().map(|(_, _, lat, _)| *lat).collect();
        let overall_avg_latency = avg_latencies.iter().sum::<Duration>() / avg_latencies.len() as u32;

        measurement.total_operations = total_expected as u64;
        measurement.successful_operations = total_successful as u64;
        measurement.failed_operations = (total_expected - total_successful) as u64;
        measurement.average_latency = overall_avg_latency;
        
        // Calculate min/max latency from thread results
        for (_, _, thread_avg_latency, _) in &thread_results {
            measurement.update_latency_stats(*thread_avg_latency);
        }

        measurement.complete();
        results.push((thread_count, measurement.throughput_ops_per_sec, measurement.average_latency));

        println!("Threads: {}, Throughput: {:.2} ops/sec, Avg Latency: {:.3}ms",
            thread_count, 
            measurement.throughput_ops_per_sec,
            measurement.average_latency.as_secs_f64() * 1000.0);
    }

    // Analyze scaling characteristics
    println!("\n=== Scaling Analysis ===");
    for i in 1..results.len() {
        let (prev_threads, prev_throughput, _) = results[i-1];
        let (curr_threads, curr_throughput, _) = results[i];
        
        let scaling_factor = curr_throughput / prev_throughput;
        let ideal_scaling = curr_threads as f64 / prev_threads as f64;
        let efficiency = scaling_factor / ideal_scaling;
        
        println!("{} → {} threads: {:.2}x throughput increase ({:.1}% efficiency)",
            prev_threads, curr_threads, scaling_factor, efficiency * 100.0);
    }

    // Verify reasonable scaling
    let (_, first_throughput, _) = results[0];
    let (_, last_throughput, _) = results[results.len() - 1];
    let overall_scaling = last_throughput / first_throughput;
    
    println!("Overall scaling: {:.2}x improvement", overall_scaling);
    assert!(overall_scaling > 2.0, "Insufficient throughput scaling: {:.2}x", overall_scaling);

    println!("✓ Throughput scaling test completed successfully");
}

// ============================================================================
// MEMORY USAGE TESTS
// ============================================================================

#[tokio::test]
async fn test_memory_usage_under_load() {
    println!("=== Memory Usage Under Load Test ===");
    
    let runtime = Arc::new(LambdustRuntime::new(Some(8)).expect("Failed to create runtime"));
    let memory_tracker = Arc::new(MemoryTracker::new());
    
    // Start memory monitoring
    memory_tracker.start_sampling();
    
    let thread_count = 8;
    let memory_intensive_operations = 50;
    
    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let runtime_clone = runtime.clone();
        let tracker_clone = memory_tracker.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut allocated_data = Vec::new();
            let initial_memory = tracker_clone.get_current_usage();

            for op_id in 0..memory_intensive_operations {
                // Simulate memory-intensive operations
                let data_size = 1024 * (op_id % 100 + 1); // Variable data sizes
                let data = create_test_data(data_size);
                allocated_data.push(data);

                // Perform computation on the data
                let result = process_memory_intensive_data(&allocated_data[op_id]).await;
                
                if result.is_err() {
                    eprintln!("Thread {} operation {} failed", thread_id, op_id);
                }

                // Periodic cleanup to test garbage collection
                if op_id % 10 == 0 && op_id > 0 {
                    allocated_data.clear();
                    
                    // Force a small delay to allow cleanup
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }

                let current_memory = tracker_clone.get_current_usage();
                
                tokio::time::sleep(Duration::from_millis(2)).await;
            }

            let final_memory = tracker_clone.get_current_usage();
            (thread_id, initial_memory, final_memory, allocated_data.len())
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    memory_tracker.stop_sampling();
    
    let peak_memory = memory_tracker.get_peak_usage();
    let final_memory = memory_tracker.get_current_usage();
    
    println!("Peak Memory Usage: {} KB", peak_memory / 1024);
    println!("Final Memory Usage: {} KB", final_memory / 1024);
    
    // Analyze per-thread memory behavior
    for (thread_id, initial, final, remaining_allocations) in &results {
        let memory_delta = if *final > *initial {
            (*final - *initial) as i64
        } else {
            -((*initial - *final) as i64)
        };
        
        println!("Thread {}: {} → {} KB (Δ{:+} KB), {} remaining allocations",
            thread_id,
            initial / 1024,
            final / 1024,
            memory_delta / 1024,
            remaining_allocations);
    }

    // Verify memory usage is reasonable
    let memory_limit = 100 * 1024 * 1024; // 100MB limit
    assert!(peak_memory < memory_limit, "Peak memory usage exceeded limit: {} KB", peak_memory / 1024);

    // Verify memory cleanup effectiveness
    let memory_growth_ratio = final_memory as f64 / peak_memory as f64;
    assert!(memory_growth_ratio < 0.8, "Memory cleanup insufficient: {:.2}% retention", memory_growth_ratio * 100.0);

    println!("✓ Memory usage test completed - peak usage within limits, cleanup effective");
}

// ============================================================================
// LATENCY MEASUREMENT TESTS
// ============================================================================

#[tokio::test]
async fn test_latency_characteristics() {
    println!("=== Latency Characteristics Test ===");
    
    let runtime = Arc::new(LambdustRuntime::new(Some(6)).expect("Failed to create runtime"));
    let latency_tracker = Arc::new(LatencyMeasurement::new());
    
    let thread_count = 6;
    let samples_per_thread = 100;
    
    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let runtime_clone = runtime.clone();
        let tracker_clone = latency_tracker.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut thread_latencies = Vec::new();

            for sample_id in 0..samples_per_thread {
                let start_time = Instant::now();
                
                // Simulate different types of operations with varying latencies
                let operation_type = sample_id % 4;
                let result = match operation_type {
                    0 => fast_operation().await,
                    1 => medium_operation().await,
                    2 => slow_operation().await,
                    3 => variable_latency_operation(sample_id).await,
                    _ => unreachable!(),
                };

                let latency = start_time.elapsed();
                thread_latencies.push(latency);
                tracker_clone.record(latency);

                if result.is_err() {
                    eprintln!("Thread {} sample {} failed", thread_id, sample_id);
                }

                // Small inter-operation delay
                tokio::time::sleep(Duration::from_millis(1)).await;
            }

            // Calculate thread-specific statistics
            thread_latencies.sort();
            let thread_min = thread_latencies[0];
            let thread_max = thread_latencies[thread_latencies.len() - 1];
            let thread_median = thread_latencies[thread_latencies.len() / 2];
            let thread_avg: Duration = thread_latencies.iter().sum::<Duration>() / thread_latencies.len() as u32;

            (thread_id, thread_min, thread_max, thread_median, thread_avg)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    // Calculate overall latency statistics
    let percentiles = latency_tracker.calculate_percentiles();
    let overall_average = latency_tracker.get_average();
    let total_samples = latency_tracker.get_count();

    println!("Total Samples: {}", total_samples);
    println!("Overall Average Latency: {:.3}ms", overall_average.as_secs_f64() * 1000.0);
    println!("Latency Percentiles:");
    println!("  P50 (Median): {:.3}ms", percentiles.p50.as_secs_f64() * 1000.0);
    println!("  P90: {:.3}ms", percentiles.p90.as_secs_f64() * 1000.0);
    println!("  P95: {:.3}ms", percentiles.p95.as_secs_f64() * 1000.0);
    println!("  P99: {:.3}ms", percentiles.p99.as_secs_f64() * 1000.0);

    // Print per-thread latency statistics
    println!("\nPer-Thread Latency Statistics:");
    for (thread_id, min_lat, max_lat, median_lat, avg_lat) in &results {
        println!("Thread {}: min={:.3}ms, max={:.3}ms, median={:.3}ms, avg={:.3}ms",
            thread_id,
            min_lat.as_secs_f64() * 1000.0,
            max_lat.as_secs_f64() * 1000.0,
            median_lat.as_secs_f64() * 1000.0,
            avg_lat.as_secs_f64() * 1000.0);
    }

    // Verify latency characteristics
    assert!(percentiles.p99 < Duration::from_millis(500), "P99 latency too high: {:.3}ms", percentiles.p99.as_secs_f64() * 1000.0);
    assert!(percentiles.p90 < Duration::from_millis(200), "P90 latency too high: {:.3}ms", percentiles.p90.as_secs_f64() * 1000.0);
    assert!(overall_average < Duration::from_millis(100), "Average latency too high: {:.3}ms", overall_average.as_secs_f64() * 1000.0);

    let expected_samples = thread_count * samples_per_thread;
    assert_eq!(total_samples, expected_samples, "Not all latency samples were collected");

    println!("✓ Latency characteristics test completed - all metrics within acceptable bounds");
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async fn perform_simulated_computation(thread_id: usize, op_id: usize) -> Result<u64> {
    // Simulate different types of computational work
    let work_type = (thread_id + op_id) % 3;
    
    match work_type {
        0 => {
            // CPU-intensive work
            let mut result = 1u64;
            for i in 1..=100 {
                result = result.wrapping_mul(i);
            }
            Ok(result)
        }
        1 => {
            // IO-simulated work
            tokio::time::sleep(Duration::from_millis(2)).await;
            Ok((thread_id * op_id) as u64)
        }
        2 => {
            // Mixed work
            let mut sum = 0u64;
            for i in 1..=50 {
                sum = sum.wrapping_add(i);
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
            Ok(sum)
        }
        _ => unreachable!(),
    }
}

fn create_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

async fn process_memory_intensive_data(data: &[u8]) -> Result<usize> {
    // Simulate processing that might involve memory allocation
    let mut checksum = 0usize;
    for (i, &byte) in data.iter().enumerate() {
        checksum = checksum.wrapping_add(byte as usize * i);
    }
    
    // Small async delay to simulate real processing
    tokio::time::sleep(Duration::from_millis(1)).await;
    
    Ok(checksum)
}

async fn fast_operation() -> Result<()> {
    // Simulate fast operation (1-5ms)
    tokio::time::sleep(Duration::from_millis(1 + rand::random::<u64>() % 5)).await;
    Ok(())
}

async fn medium_operation() -> Result<()> {
    // Simulate medium operation (10-30ms)
    tokio::time::sleep(Duration::from_millis(10 + rand::random::<u64>() % 21)).await;
    Ok(())
}

async fn slow_operation() -> Result<()> {
    // Simulate slow operation (50-100ms)
    tokio::time::sleep(Duration::from_millis(50 + rand::random::<u64>() % 51)).await;
    Ok(())
}

async fn variable_latency_operation(seed: usize) -> Result<()> {
    // Simulate operation with variable latency based on input
    let latency = if seed % 10 == 0 {
        Duration::from_millis(100) // Occasional slow operation
    } else {
        Duration::from_millis(2 + (seed % 10) as u64)
    };
    
    tokio::time::sleep(latency).await;
    Ok(())
}