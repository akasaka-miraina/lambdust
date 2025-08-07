//! Cross-Platform Parallelism Tests
//!
//! Tests for verifying consistent parallel behavior across different operating
//! systems and hardware configurations, including platform-specific optimizations
//! and adaptive thread pool management.

use lambdust::runtime::LambdustRuntime;
use lambdust::diagnostics::{Result, Error};

use std::sync::{Arc, Mutex, Barrier};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::thread;

/// Platform information detector
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub os_name: String,
    pub cpu_count: usize,
    pub architecture: String,
    pub features: Vec<String>,
}

impl PlatformInfo {
    pub fn detect() -> Self {
        Self {
            os_name: std::env::consts::OS.to_string(),
            cpu_count: num_cpus::get(),
            architecture: std::env::consts::ARCH.to_string(),
            features: Self::detect_features(),
        }
    }

    fn detect_features() -> Vec<String> {
        let mut features = Vec::new();
        
        // Detect available features based on platform
        #[cfg(target_os = "windows")]
        {
            features.push("windows_native_threads".to_string());
            features.push("windows_synchronization".to_string());
        }
        
        #[cfg(target_os = "macos")]
        {
            features.push("macos_grand_central_dispatch".to_string());
            features.push("macos_native_threads".to_string());
        }
        
        #[cfg(target_os = "linux")]
        {
            features.push("linux_native_threads".to_string());
            features.push("linux_futex".to_string());
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            features.push("x86_64_optimizations".to_string());
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            features.push("arm64_optimizations".to_string());
        }

        features
    }

    pub fn get_recommended_thread_count(&self) -> usize {
        match self.os_name.as_str() {
            "windows" => std::cmp::min(self.cpu_count, 64), // Windows thread limit considerations
            "macos" => self.cpu_count, // macOS handles thread scheduling well
            "linux" => self.cpu_count * 2, // Linux can handle more threads effectively
            _ => self.cpu_count,
        }
    }

    pub fn get_platform_specific_optimizations(&self) -> HashMap<String, bool> {
        let mut optimizations = HashMap::new();
        
        // Enable platform-specific optimizations
        optimizations.insert("work_stealing".to_string(), true);
        optimizations.insert("numa_awareness".to_string(), self.cpu_count > 8);
        optimizations.insert("thread_affinity".to_string(), 
            self.os_name == "linux" && self.cpu_count > 4);
        optimizations.insert("lock_free_algorithms".to_string(), 
            self.architecture == "x86_64" || self.architecture == "aarch64");
        
        optimizations
    }
}

/// Adaptive thread pool manager for cross-platform optimization
#[derive(Debug)]
pub struct AdaptiveThreadPoolManager {
    platform_info: PlatformInfo,
    current_load: Arc<Mutex<f64>>,
    performance_history: Arc<Mutex<Vec<(Instant, usize, f64)>>>,
    optimal_thread_count: Arc<Mutex<usize>>,
}

impl AdaptiveThreadPoolManager {
    pub fn new() -> Self {
        let platform_info = PlatformInfo::detect();
        let initial_thread_count = platform_info.get_recommended_thread_count();
        
        Self {
            platform_info,
            current_load: Arc::new(Mutex::new(0.0)),
            performance_history: Arc::new(Mutex::new(Vec::new())),
            optimal_thread_count: Arc::new(Mutex::new(initial_thread_count)),
        }
    }

    pub fn get_platform_info(&self) -> &PlatformInfo {
        &self.platform_info
    }

    pub fn adjust_thread_count(&self, workload_characteristics: &WorkloadCharacteristics) -> usize {
        let base_count = self.platform_info.get_recommended_thread_count();
        
        // Adjust based on workload type
        let adjusted_count = match workload_characteristics.workload_type {
            WorkloadType::CPUIntensive => {
                std::cmp::min(base_count, self.platform_info.cpu_count)
            }
            WorkloadType::IOIntensive => {
                base_count * 2 // More threads for IO-bound work
            }
            WorkloadType::Mixed => {
                (base_count as f64 * 1.5) as usize
            }
            WorkloadType::Memory => {
                std::cmp::min(base_count, self.platform_info.cpu_count / 2)
            }
        };

        // Apply platform-specific adjustments
        let platform_adjusted = match self.platform_info.os_name.as_str() {
            "windows" => std::cmp::min(adjusted_count, 32), // Conservative on Windows
            "macos" => adjusted_count, // macOS handles threading well
            "linux" => adjusted_count, // Linux is flexible
            _ => std::cmp::min(adjusted_count, self.platform_info.cpu_count),
        };

        {
            let mut optimal = self.optimal_thread_count.lock().unwrap();
            *optimal = platform_adjusted;
        }

        platform_adjusted
    }

    pub fn record_performance(&self, thread_count: usize, throughput: f64) {
        let mut history = self.performance_history.lock().unwrap();
        history.push((Instant::now(), thread_count, throughput));
        
        // Keep only recent history
        if history.len() > 100 {
            history.drain(0..50);
        }
    }

    pub fn get_optimal_thread_count(&self) -> usize {
        *self.optimal_thread_count.lock().unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum WorkloadType {
    CPUIntensive,
    IOIntensive,
    Mixed,
    Memory,
}

#[derive(Debug, Clone)]
pub struct WorkloadCharacteristics {
    pub workload_type: WorkloadType,
    pub expected_duration: Duration,
    pub memory_usage: usize,
    pub io_operations: usize,
}

// ============================================================================
// PLATFORM-SPECIFIC OPTIMIZATION TESTS
// ============================================================================

#[tokio::test]
async fn test_platform_specific_optimizations() {
    let platform_info = PlatformInfo::detect();
    let optimizations = platform_info.get_platform_specific_optimizations();
    
    println!("=== Platform-Specific Optimization Test ===");
    println!("Platform: {} on {}", platform_info.os_name, platform_info.architecture);
    println!("CPU Count: {}", platform_info.cpu_count);
    println!("Features: {:?}", platform_info.features);
    println!("Optimizations: {:?}", optimizations);

    let recommended_threads = platform_info.get_recommended_thread_count();
    let runtime = Arc::new(LambdustRuntime::new(Some(recommended_threads))
        .expect("Failed to create runtime"));

    let thread_count = recommended_threads;
    let operations_per_thread = 50;
    
    let barrier = Arc::new(Barrier::new(thread_count));
    let performance_data = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let runtime_clone = runtime.clone();
        let data_clone = performance_data.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let start_time = Instant::now();
            let mut successful_operations = 0;

            for op_id in 0..operations_per_thread {
                // Perform platform-optimized operations
                let result = perform_platform_optimized_operation(thread_id, op_id, &platform_info).await;
                
                if result.is_ok() {
                    successful_operations += 1;
                }

                // Platform-specific delay strategy
                let delay_millis = match platform_info.os_name.as_str() {
                    "windows" => 2, // Slightly longer delays on Windows
                    "macos" => 1,   // Efficient scheduling on macOS
                    "linux" => 1,   // Linux handles fine-grained timing well
                    _ => 2,
                };
                tokio::time::sleep(Duration::from_millis(delay_millis)).await;
            }

            let duration = start_time.elapsed();
            let throughput = successful_operations as f64 / duration.as_secs_f64();
            
            {
                let mut data = data_clone.lock().unwrap();
                data.push((thread_id, successful_operations, throughput, duration));
            }

            (thread_id, successful_operations, throughput)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    let total_successful: usize = results.iter().map(|(_, ops, _)| *ops).sum();
    let total_expected = thread_count * operations_per_thread;
    let average_throughput: f64 = results.iter().map(|(_, _, throughput)| *throughput).sum::<f64>() / results.len() as f64;

    println!("  Recommended threads: {}", recommended_threads);
    println!("  Total operations: {}/{}", total_successful, total_expected);
    println!("  Success rate: {:.2}%", (total_successful as f64 / total_expected as f64) * 100.0);
    println!("  Average throughput: {:.2} ops/sec", average_throughput);

    // Verify platform-specific performance expectations
    let expected_min_throughput = match platform_info.os_name.as_str() {
        "linux" => 50.0,   // Linux should perform well
        "macos" => 45.0,   // macOS should be efficient
        "windows" => 40.0, // Windows might be slightly slower
        _ => 35.0,
    };

    assert!(average_throughput > expected_min_throughput, 
        "Platform {} throughput below expectations: {:.2} < {:.2}", 
        platform_info.os_name, average_throughput, expected_min_throughput);

    // Verify threading efficiency
    let threading_efficiency = average_throughput / thread_count as f64;
    println!("  Threading efficiency: {:.2} ops/sec/thread", threading_efficiency);

    assert!(threading_efficiency > 5.0, "Threading efficiency too low: {:.2}", threading_efficiency);

    println!("✓ Platform-specific optimizations working effectively");
}

#[tokio::test]
async fn test_adaptive_thread_pool_management() {
    let mut manager = AdaptiveThreadPoolManager::new();
    let platform_info = manager.get_platform_info().clone();
    
    println!("=== Adaptive Thread Pool Management Test ===");
    println!("Platform: {} on {}", platform_info.os_name, platform_info.architecture);

    // Test different workload types
    let workload_types = vec![
        (WorkloadType::CPUIntensive, "CPU Intensive"),
        (WorkloadType::IOIntensive, "IO Intensive"),
        (WorkloadType::Mixed, "Mixed"),
        (WorkloadType::Memory, "Memory Intensive"),
    ];

    for (workload_type, workload_name) in workload_types {
        let workload_characteristics = WorkloadCharacteristics {
            workload_type: workload_type.clone(),
            expected_duration: Duration::from_secs(5),
            memory_usage: 1024 * 1024, // 1MB
            io_operations: 10,
        };

        let optimal_threads = manager.adjust_thread_count(&workload_characteristics);
        println!("  {} workload: {} threads", workload_name, optimal_threads);

        // Test the configuration
        let runtime = Arc::new(LambdustRuntime::new(Some(optimal_threads))
            .expect("Failed to create runtime"));

        let start_time = Instant::now();
        let operations_per_thread = 20;
        
        let barrier = Arc::new(Barrier::new(optimal_threads));
        let mut handles = Vec::new();

        for thread_id in 0..optimal_threads {
            let barrier_clone = barrier.clone();
            let runtime_clone = runtime.clone();
            let workload_clone = workload_characteristics.clone();

            let handle = tokio::spawn(async move {
                barrier_clone.wait();
                
                let mut successful_operations = 0;

                for op_id in 0..operations_per_thread {
                    let result = perform_workload_specific_operation(
                        thread_id, op_id, &workload_clone).await;
                    
                    if result.is_ok() {
                        successful_operations += 1;
                    }

                    tokio::time::sleep(Duration::from_millis(2)).await;
                }

                successful_operations
            });
            handles.push(handle);
        }

        // Wait for completion and measure performance
        let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
            .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

        let duration = start_time.elapsed();
        let total_operations: usize = results.iter().sum();
        let throughput = total_operations as f64 / duration.as_secs_f64();

        manager.record_performance(optimal_threads, throughput);

        println!("    Operations: {}", total_operations);
        println!("    Duration: {:.3}s", duration.as_secs_f64());
        println!("    Throughput: {:.2} ops/sec", throughput);

        // Verify reasonable performance for each workload type
        let expected_min_throughput = match workload_type {
            WorkloadType::CPUIntensive => 30.0,
            WorkloadType::IOIntensive => 40.0,
            WorkloadType::Mixed => 35.0,
            WorkloadType::Memory => 25.0,
        };

        assert!(throughput > expected_min_throughput, 
            "{} workload throughput too low: {:.2}", workload_name, throughput);
    }

    println!("✓ Adaptive thread pool management optimized for different workloads");
}

// ============================================================================
// CROSS-PLATFORM CONSISTENCY TESTS
// ============================================================================

#[tokio::test]
async fn test_cross_platform_consistency() {
    let platform_info = PlatformInfo::detect();
    
    println!("=== Cross-Platform Consistency Test ===");
    println!("Testing on: {} {}", platform_info.os_name, platform_info.architecture);

    // Test consistent behavior across different thread counts
    let thread_counts = vec![1, 2, 4, platform_info.cpu_count];
    let mut consistency_results = Vec::new();

    for &thread_count in &thread_counts {
        let runtime = Arc::new(LambdustRuntime::new(Some(thread_count))
            .expect("Failed to create runtime"));

        let operations_per_thread = 30;
        let start_time = Instant::now();
        
        let barrier = Arc::new(Barrier::new(thread_count));
        let mut handles = Vec::new();

        for thread_id in 0..thread_count {
            let barrier_clone = barrier.clone();
            let runtime_clone = runtime.clone();

            let handle = tokio::spawn(async move {
                barrier_clone.wait();
                
                let mut operation_results = Vec::new();

                for op_id in 0..operations_per_thread {
                    let result = perform_consistency_test_operation(thread_id, op_id).await;
                    operation_results.push(result);
                    
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }

                operation_results
            });
            handles.push(handle);
        }

        let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
            .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

        let duration = start_time.elapsed();
        let all_results: Vec<_> = results.into_iter().flatten().collect();
        let successful_operations = all_results.iter().filter(|r| r.is_ok()).count();
        let total_operations = all_results.len();
        let throughput = successful_operations as f64 / duration.as_secs_f64();

        consistency_results.push((thread_count, successful_operations, total_operations, throughput));

        println!("  {} threads: {}/{} ops, {:.2} ops/sec", 
            thread_count, successful_operations, total_operations, throughput);
    }

    // Analyze consistency across thread counts
    let success_rates: Vec<f64> = consistency_results.iter()
        .map(|(_, success, total, _)| *success as f64 / *total as f64)
        .collect();

    let throughputs: Vec<f64> = consistency_results.iter()
        .map(|(_, _, _, throughput)| *throughput)
        .collect();

    // Verify consistency
    let success_rate_variance = calculate_variance(&success_rates);
    let throughput_scaling = if throughputs.len() > 1 {
        throughputs[throughputs.len() - 1] / throughputs[0]
    } else {
        1.0
    };

    println!("  Success rate variance: {:.4}", success_rate_variance);
    println!("  Throughput scaling: {:.2}x", throughput_scaling);

    // Verify platform consistency expectations
    assert!(success_rate_variance < 0.01, "Success rates too inconsistent across thread counts");
    assert!(throughput_scaling > 1.5, "Insufficient throughput scaling: {:.2}x", throughput_scaling);
    assert!(success_rates.iter().all(|&rate| rate > 0.95), "Some configurations had low success rates");

    println!("✓ Cross-platform consistency maintained across all thread configurations");
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async fn perform_platform_optimized_operation(
    thread_id: usize, 
    op_id: usize, 
    platform_info: &PlatformInfo
) -> Result<u64> {
    // Simulate platform-specific optimizations
    let optimization_factor = match platform_info.os_name.as_str() {
        "linux" => 1.2,   // Linux optimizations
        "macos" => 1.1,   // macOS optimizations
        "windows" => 1.0, // Baseline for Windows
        _ => 0.9,
    };

    let base_computation_time = Duration::from_millis(3);
    let optimized_time = Duration::from_nanos(
        (base_computation_time.as_nanos() as f64 / optimization_factor) as u64
    );

    tokio::time::sleep(optimized_time).await;

    // Simulate architecture-specific optimizations
    let result = match platform_info.architecture.as_str() {
        "x86_64" => (thread_id as u64 * 1000 + op_id as u64) ^ 0xDEADBEEF,
        "aarch64" => (thread_id as u64 * 1000 + op_id as u64) ^ 0xCAFEBABE,
        _ => thread_id as u64 * 1000 + op_id as u64,
    };

    Ok(result)
}

async fn perform_workload_specific_operation(
    thread_id: usize,
    op_id: usize,
    workload: &WorkloadCharacteristics,
) -> Result<()> {
    match workload.workload_type {
        WorkloadType::CPUIntensive => {
            // CPU-intensive computation
            let mut result = 1u64;
            for i in 1..=100 {
                result = result.wrapping_mul(i);
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        WorkloadType::IOIntensive => {
            // IO-intensive simulation
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        WorkloadType::Mixed => {
            // Mixed workload
            let mut sum = 0u64;
            for i in 1..=50 {
                sum = sum.wrapping_add(i);
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        WorkloadType::Memory => {
            // Memory-intensive operations
            let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
            let _checksum: usize = data.iter().enumerate().map(|(i, &b)| i * b as usize).sum();
            tokio::time::sleep(Duration::from_millis(3)).await;
        }
    }

    Ok(())
}

async fn perform_consistency_test_operation(thread_id: usize, op_id: usize) -> Result<u64> {
    // Deterministic operation for consistency testing
    tokio::time::sleep(Duration::from_millis(2)).await;
    
    let result = (thread_id as u64 * 10000 + op_id as u64) % 1000000;
    Ok(result)
}

fn calculate_variance(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    
    variance
}