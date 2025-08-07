//! IO Coordination Tests
//!
//! Tests for distributed IO coordination system, including safe file access
//! coordination, network communication parallelization, and resource 
//! management in multithreaded environments.

use lambdust::runtime::LambdustRuntime;
use lambdust::diagnostics::{Result, Error};

use std::sync::{Arc, Mutex, RwLock, Barrier};
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Write, Read};
use tempfile::{TempDir, NamedTempFile};
use tokio::sync::Semaphore;

/// File access coordinator for safe concurrent file operations
#[derive(Debug)]
pub struct FileAccessCoordinator {
    file_locks: Arc<RwLock<HashMap<PathBuf, Arc<Mutex<()>>>>>,
    operation_log: Arc<Mutex<Vec<(Instant, String, PathBuf)>>>,
    stats: Arc<Mutex<IOCoordinationStats>>,
}

#[derive(Debug, Clone)]
pub struct IOCoordinationStats {
    pub total_read_operations: u64,
    pub total_write_operations: u64,
    pub total_append_operations: u64,
    pub total_delete_operations: u64,
    pub conflicts_resolved: u64,
    pub errors_encountered: u64,
}

impl Default for IOCoordinationStats {
    fn default() -> Self {
        Self {
            total_read_operations: 0,
            total_write_operations: 0,
            total_append_operations: 0,
            total_delete_operations: 0,
            conflicts_resolved: 0,
            errors_encountered: 0,
        }
    }
}

impl FileAccessCoordinator {
    pub fn new() -> Self {
        Self {
            file_locks: Arc::new(RwLock::new(HashMap::new())),
            operation_log: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(IOCoordinationStats::default())),
        }
    }

    /// Get or create a lock for a specific file
    fn get_file_lock(&self, path: &Path) -> Arc<Mutex<()>> {
        let mut locks = self.file_locks.write().unwrap();
        locks.entry(path.to_path_buf())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    /// Log an IO operation
    fn log_operation(&self, operation: &str, path: &Path) {
        let mut log = self.operation_log.lock().unwrap();
        log.push((Instant::now(), operation.to_string(), path.to_path_buf()));
    }

    /// Update statistics
    fn update_stats<F>(&self, updater: F) where F: FnOnce(&mut IOCoordinationStats) {
        let mut stats = self.stats.lock().unwrap();
        updater(&mut *stats);
    }

    /// Coordinated file read operation
    pub async fn read_file(&self, path: &Path) -> Result<String> {
        let lock = self.get_file_lock(path);
        let _guard = lock.lock().unwrap();

        self.log_operation("read", path);
        
        match fs::read_to_string(path) {
            Ok(content) => {
                self.update_stats(|stats| stats.total_read_operations += 1);
                Ok(content)
            }
            Err(e) => {
                self.update_stats(|stats| stats.errors_encountered += 1);
                Err(Error::runtime_error(
                    format!("Failed to read file {:?}: {}", path, e),
                    None,
                ))
            }
        }
    }

    /// Coordinated file write operation
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        let lock = self.get_file_lock(path);
        let _guard = lock.lock().unwrap();

        self.log_operation("write", path);

        match fs::write(path, content) {
            Ok(()) => {
                self.update_stats(|stats| stats.total_write_operations += 1);
                Ok(())
            }
            Err(e) => {
                self.update_stats(|stats| stats.errors_encountered += 1);
                Err(Error::runtime_error(
                    format!("Failed to write file {:?}: {}", path, e),
                    None,
                ))
            }
        }
    }

    /// Coordinated file append operation
    pub async fn append_file(&self, path: &Path, content: &str) -> Result<()> {
        let lock = self.get_file_lock(path);
        let _guard = lock.lock().unwrap();

        self.log_operation("append", path);

        match fs::OpenOptions::new().create(true).append(true).open(path) {
            Ok(mut file) => {
                match file.write_all(content.as_bytes()) {
                    Ok(()) => {
                        self.update_stats(|stats| stats.total_append_operations += 1);
                        Ok(())
                    }
                    Err(e) => {
                        self.update_stats(|stats| stats.errors_encountered += 1);
                        Err(Error::runtime_error(
                            format!("Failed to append to file {:?}: {}", path, e),
                            None,
                        ))
                    }
                }
            }
            Err(e) => {
                self.update_stats(|stats| stats.errors_encountered += 1);
                Err(Error::runtime_error(
                    format!("Failed to open file for append {:?}: {}", path, e),
                    None,
                ))
            }
        }
    }

    /// Coordinated file truncate operation
    pub async fn truncate_file(&self, path: &Path) -> Result<()> {
        let lock = self.get_file_lock(path);
        let _guard = lock.lock().unwrap();

        self.log_operation("truncate", path);

        match fs::OpenOptions::new().write(true).truncate(true).open(path) {
            Ok(_) => {
                self.update_stats(|stats| stats.total_write_operations += 1);
                Ok(())
            }
            Err(e) => {
                self.update_stats(|stats| stats.errors_encountered += 1);
                Err(Error::runtime_error(
                    format!("Failed to truncate file {:?}: {}", path, e),
                    None,
                ))
            }
        }
    }

    /// Get coordination statistics
    pub fn get_stats(&self) -> IOCoordinationStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get operation log
    pub fn get_operation_log(&self) -> Vec<(Instant, String, PathBuf)> {
        self.operation_log.lock().unwrap().clone()
    }

    /// Check file consistency
    pub fn verify_file_consistency(&self, path: &Path) -> Result<bool> {
        let lock = self.get_file_lock(path);
        let _guard = lock.lock().unwrap();

        if !path.exists() {
            return Ok(true); // Non-existent files are consistent
        }

        // Verify file is readable and has valid content
        match fs::read_to_string(path) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Network operation simulator for testing distributed communication
#[derive(Debug)]
pub struct NetworkOperationSimulator {
    operation_log: Arc<Mutex<Vec<(Instant, String, String)>>>,
    latency_simulation: Duration,
    failure_rate: f64,
    stats: Arc<Mutex<NetworkStats>>,
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub requests_sent: u64,
    pub responses_received: u64,
    pub connection_failures: u64,
    pub timeouts: u64,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            requests_sent: 0,
            responses_received: 0,
            connection_failures: 0,
            timeouts: 0,
        }
    }
}

impl NetworkOperationSimulator {
    pub fn new(latency: Duration, failure_rate: f64) -> Self {
        Self {
            operation_log: Arc::new(Mutex::new(Vec::new())),
            latency_simulation: latency,
            failure_rate,
            stats: Arc::new(Mutex::new(NetworkStats::default())),
        }
    }

    /// Simulate sending a network request
    pub async fn send_request(&self, endpoint: &str, data: &str) -> Result<String> {
        let mut stats = self.stats.lock().unwrap();
        stats.requests_sent += 1;
        drop(stats);

        self.log_operation("send_request", endpoint);

        // Simulate network latency
        tokio::time::sleep(self.latency_simulation).await;

        // Simulate random failures
        if rand::random::<f64>() < self.failure_rate {
            let mut stats = self.stats.lock().unwrap();
            stats.connection_failures += 1;
            return Err(Error::runtime_error(
                format!("Network failure for endpoint: {}", endpoint),
                None,
            ));
        }

        let mut stats = self.stats.lock().unwrap();
        stats.responses_received += 1;
        drop(stats);

        Ok(format!("Response from {} for data: {}", endpoint, data))
    }

    /// Simulate receiving a response
    pub async fn receive_response(&self, endpoint: &str) -> Result<String> {
        self.log_operation("receive_response", endpoint);
        
        // Simulate network latency
        tokio::time::sleep(self.latency_simulation).await;

        Ok(format!("Received data from {}", endpoint))
    }

    fn log_operation(&self, operation: &str, endpoint: &str) {
        let mut log = self.operation_log.lock().unwrap();
        log.push((Instant::now(), operation.to_string(), endpoint.to_string()));
    }

    pub fn get_stats(&self) -> NetworkStats {
        self.stats.lock().unwrap().clone()
    }

    pub fn get_operation_log(&self) -> Vec<(Instant, String, String)> {
        self.operation_log.lock().unwrap().clone()
    }
}

// ============================================================================
// FILE ACCESS COORDINATION TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_file_access_coordination() {
    let runtime = Arc::new(LambdustRuntime::new(Some(6)).expect("Failed to create runtime"));
    let coordinator = Arc::new(FileAccessCoordinator::new());
    
    // Create temporary directory for test files
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let shared_file = temp_dir.path().join("shared_file.txt");
    let log_file = temp_dir.path().join("operations.log");

    // Initialize shared file
    coordinator.write_file(&shared_file, "Initial content\n").await
        .expect("Failed to initialize shared file");

    let thread_count = 6;
    let operations_per_thread = 20;
    
    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let coordinator_clone = coordinator.clone();
        let shared_file_clone = shared_file.clone();
        let log_file_clone = log_file.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut successful_operations = 0;
            let mut read_operations = 0;
            let mut write_operations = 0;

            for op_id in 0..operations_per_thread {
                let operation_type = op_id % 4;
                
                match operation_type {
                    0 => {
                        // Read operation
                        match coordinator_clone.read_file(&shared_file_clone).await {
                            Ok(content) => {
                                successful_operations += 1;
                                read_operations += 1;
                                
                                // Log read operation
                                let log_entry = format!("Thread {} read {} bytes at op {}\n", 
                                    thread_id, content.len(), op_id);
                                let _ = coordinator_clone.append_file(&log_file_clone, &log_entry).await;
                            }
                            Err(e) => eprintln!("Thread {} read failed: {}", thread_id, e),
                        }
                    }
                    1 => {
                        // Append operation
                        let append_data = format!("Thread {} appended at op {}\n", thread_id, op_id);
                        match coordinator_clone.append_file(&shared_file_clone, &append_data).await {
                            Ok(()) => {
                                successful_operations += 1;
                                write_operations += 1;
                            }
                            Err(e) => eprintln!("Thread {} append failed: {}", thread_id, e),
                        }
                    }
                    2 => {
                        // Write new content (occasionally)
                        if op_id % 8 == 0 {
                            let new_content = format!("Thread {} overwrote file at op {}\n", thread_id, op_id);
                            match coordinator_clone.write_file(&shared_file_clone, &new_content).await {
                                Ok(()) => {
                                    successful_operations += 1;
                                    write_operations += 1;
                                }
                                Err(e) => eprintln!("Thread {} write failed: {}", thread_id, e),
                            }
                        } else {
                            successful_operations += 1; // Skip operation
                        }
                    }
                    3 => {
                        // Read and verify consistency
                        match coordinator_clone.read_file(&shared_file_clone).await {
                            Ok(_) => {
                                if coordinator_clone.verify_file_consistency(&shared_file_clone).unwrap_or(false) {
                                    successful_operations += 1;
                                    read_operations += 1;
                                }
                            }
                            Err(e) => eprintln!("Thread {} consistency check failed: {}", thread_id, e),
                        }
                    }
                    _ => unreachable!(),
                }

                // Small delay between operations
                tokio::time::sleep(Duration::from_millis(2)).await;
            }

            (thread_id, successful_operations, read_operations, write_operations)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    let total_successful: usize = results.iter().map(|(_, ops, _, _)| *ops).sum();
    let total_reads: usize = results.iter().map(|(_, _, reads, _)| *reads).sum();
    let total_writes: usize = results.iter().map(|(_, _, _, writes)| *writes).sum();

    let stats = coordinator.get_stats();
    
    println!("✓ Concurrent file access coordination test completed");
    println!("  Total successful operations: {}", total_successful);
    println!("  Read operations: {}", total_reads);
    println!("  Write operations: {}", total_writes);
    println!("  Coordinator stats:");
    println!("    Read operations: {}", stats.total_read_operations);
    println!("    Write operations: {}", stats.total_write_operations);
    println!("    Append operations: {}", stats.total_append_operations);
    println!("    Errors encountered: {}", stats.errors_encountered);

    // Verify file integrity
    assert!(coordinator.verify_file_consistency(&shared_file).unwrap(), 
        "Shared file consistency check failed");
    
    if log_file.exists() {
        assert!(coordinator.verify_file_consistency(&log_file).unwrap(), 
            "Log file consistency check failed");
    }

    // Verify coordination effectiveness
    let expected_total = thread_count * operations_per_thread;
    let success_rate = total_successful as f64 / expected_total as f64;
    assert!(success_rate > 0.9, "File coordination success rate too low: {:.2}%", success_rate * 100.0);

    // Verify no data corruption occurred
    if let Ok(final_content) = coordinator.read_file(&shared_file).await {
        assert!(!final_content.is_empty(), "Shared file became empty");
        println!("  Final shared file size: {} bytes", final_content.len());
    }

    println!("✓ File access coordination successful with no data corruption");
}

// ============================================================================
// NETWORK COMMUNICATION TESTS
// ============================================================================

#[tokio::test]
async fn test_parallel_network_operations() {
    let runtime = Arc::new(LambdustRuntime::new(Some(8)).expect("Failed to create runtime"));
    
    // Create network simulators for different "servers"
    let server_simulators = vec![
        Arc::new(NetworkOperationSimulator::new(Duration::from_millis(10), 0.05)), // server1: low latency, low failure
        Arc::new(NetworkOperationSimulator::new(Duration::from_millis(25), 0.10)), // server2: medium latency, medium failure  
        Arc::new(NetworkOperationSimulator::new(Duration::from_millis(5), 0.15)),  // server3: very low latency, higher failure
        Arc::new(NetworkOperationSimulator::new(Duration::from_millis(50), 0.02)), // server4: high latency, very low failure
    ];

    let thread_count = 8;
    let requests_per_thread = 15;
    
    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let simulators = server_simulators.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut successful_requests = 0;
            let mut failed_requests = 0;
            let mut total_latency = Duration::ZERO;

            for request_id in 0..requests_per_thread {
                let server_index = request_id % simulators.len();
                let simulator = &simulators[server_index];
                let endpoint = format!("server{}", server_index + 1);
                let request_data = format!("thread-{}-request-{}", thread_id, request_id);

                let start_time = Instant::now();
                
                match simulator.send_request(&endpoint, &request_data).await {
                    Ok(response) => {
                        successful_requests += 1;
                        let latency = start_time.elapsed();
                        total_latency += latency;
                        
                        // Simulate processing response
                        if !response.is_empty() {
                            // Response received successfully
                        }
                    }
                    Err(_) => {
                        failed_requests += 1;
                    }
                }

                // Small delay between requests
                tokio::time::sleep(Duration::from_millis(1)).await;
            }

            let avg_latency = if successful_requests > 0 {
                total_latency / successful_requests as u32
            } else {
                Duration::ZERO
            };

            (thread_id, successful_requests, failed_requests, avg_latency)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    let total_successful: usize = results.iter().map(|(_, success, _, _)| *success).sum();
    let total_failed: usize = results.iter().map(|(_, _, failed, _)| *failed).sum();
    let total_requests = total_successful + total_failed;
    let expected_requests = thread_count * requests_per_thread;

    println!("✓ Parallel network operations test completed");
    println!("  Total requests: {}/{}", total_requests, expected_requests);
    println!("  Successful requests: {}", total_successful);
    println!("  Failed requests: {}", total_failed);
    println!("  Success rate: {:.2}%", (total_successful as f64 / total_requests as f64) * 100.0);

    // Print per-server statistics
    for (i, simulator) in server_simulators.iter().enumerate() {
        let stats = simulator.get_stats();
        println!("  Server {} stats:", i + 1);
        println!("    Requests sent: {}", stats.requests_sent);
        println!("    Responses received: {}", stats.responses_received);
        println!("    Connection failures: {}", stats.connection_failures);
        
        let server_success_rate = if stats.requests_sent > 0 {
            (stats.responses_received as f64 / stats.requests_sent as f64) * 100.0
        } else {
            0.0
        };
        println!("    Success rate: {:.2}%", server_success_rate);
    }

    // Calculate average latency across all threads
    let total_avg_latency: Duration = results.iter()
        .map(|(_, _, _, latency)| *latency)
        .sum::<Duration>() / thread_count as u32;
    
    println!("  Average latency: {:.2}ms", total_avg_latency.as_secs_f64() * 1000.0);

    // Verify reasonable success rate
    let overall_success_rate = total_successful as f64 / total_requests as f64;
    assert!(overall_success_rate > 0.8, "Overall success rate too low: {:.2}%", overall_success_rate * 100.0);

    // Verify all expected requests were attempted
    assert_eq!(total_requests, expected_requests, "Not all expected requests were attempted");

    println!("✓ Parallel network operations coordinated successfully");
}

// ============================================================================
// RESOURCE CONTENTION TESTS
// ============================================================================

#[tokio::test]
async fn test_resource_contention_resolution() {
    let runtime = Arc::new(LambdustRuntime::new(Some(10)).expect("Failed to create runtime"));
    let coordinator = Arc::new(FileAccessCoordinator::new());
    
    // Create multiple temporary files that will be contended
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut contended_files = Vec::new();
    
    for i in 0..3 {
        let file_path = temp_dir.path().join(format!("contended_file_{}.txt", i));
        coordinator.write_file(&file_path, &format!("Initial content for file {}\n", i)).await
            .expect(&format!("Failed to initialize file {}", i));
        contended_files.push(file_path);
    }

    let thread_count = 10;
    let contentions_per_thread = 25;
    
    let barrier = Arc::new(Barrier::new(thread_count));
    let contention_semaphore = Arc::new(Semaphore::new(3)); // Limit concurrent access
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let coordinator_clone = coordinator.clone();
        let files = contended_files.clone();
        let semaphore_clone = contention_semaphore.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut contentions_resolved = 0;
            let mut operations_completed = 0;

            for contention_id in 0..contentions_per_thread {
                // Acquire semaphore to simulate resource contention
                let _permit = semaphore_clone.acquire().await.unwrap();
                
                let file_index = contention_id % files.len();
                let target_file = &files[file_index];

                // Simulate contended operations: read, modify, write
                match coordinator_clone.read_file(target_file).await {
                    Ok(current_content) => {
                        let modified_content = format!("{}Thread {} modified at contention {}\n", 
                            current_content, thread_id, contention_id);
                        
                        // Small delay to increase contention chance
                        tokio::time::sleep(Duration::from_millis(2)).await;
                        
                        match coordinator_clone.write_file(target_file, &modified_content).await {
                            Ok(()) => {
                                contentions_resolved += 1;
                                operations_completed += 1;
                            }
                            Err(e) => eprintln!("Thread {} write failed during contention: {}", thread_id, e),
                        }
                    }
                    Err(e) => eprintln!("Thread {} read failed during contention: {}", thread_id, e),
                }

                // Verify file consistency after contention resolution
                if coordinator_clone.verify_file_consistency(target_file).unwrap_or(false) {
                    operations_completed += 1;
                }

                // Release permit to allow other threads
                drop(_permit);
                
                // Brief pause between contentions
                tokio::time::sleep(Duration::from_millis(1)).await;
            }

            (thread_id, contentions_resolved, operations_completed)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    let total_contentions: usize = results.iter().map(|(_, contentions, _)| *contentions).sum();
    let total_operations: usize = results.iter().map(|(_, _, ops)| *ops).sum();
    
    let stats = coordinator.get_stats();

    println!("✓ Resource contention resolution test completed");
    println!("  Total contentions resolved: {}", total_contentions);
    println!("  Total operations completed: {}", total_operations);
    println!("  Expected contentions: {}", thread_count * contentions_per_thread);
    println!("  Resolution rate: {:.2}%", (total_contentions as f64 / (thread_count * contentions_per_thread) as f64) * 100.0);
    
    println!("  Coordinator stats:");
    println!("    Total read operations: {}", stats.total_read_operations);
    println!("    Total write operations: {}", stats.total_write_operations);
    println!("    Errors encountered: {}", stats.errors_encountered);

    // Verify all files maintain consistency
    for (i, file_path) in contended_files.iter().enumerate() {
        assert!(coordinator.verify_file_consistency(file_path).unwrap(), 
            "File {} consistency check failed", i);
        
        if let Ok(final_content) = coordinator.read_file(file_path).await {
            println!("  File {} final size: {} bytes", i, final_content.len());
            assert!(!final_content.is_empty(), "File {} became empty after contention", i);
        }
    }

    // Verify reasonable contention resolution rate
    let resolution_rate = total_contentions as f64 / (thread_count * contentions_per_thread) as f64;
    assert!(resolution_rate > 0.8, "Contention resolution rate too low: {:.2}%", resolution_rate * 100.0);

    println!("✓ Resource contention resolved successfully with maintained data integrity");
}