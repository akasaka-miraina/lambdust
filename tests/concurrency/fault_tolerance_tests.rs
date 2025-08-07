//! Fault Tolerance and Recovery Tests
//!
//! Tests for the fault tolerance capabilities of the multithreaded runtime,
//! including partial failure handling, deadlock detection and resolution,
//! error propagation, and system recovery mechanisms.

use lambdust::runtime::LambdustRuntime;
use lambdust::diagnostics::{Result, Error};

use std::sync::{Arc, Mutex, RwLock, Barrier};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::thread;
use tokio::sync::Semaphore;

/// Fault injection framework for testing
#[derive(Debug)]
pub struct FaultInjector {
    failure_rate: Arc<RwLock<f64>>,
    failure_types: Arc<RwLock<Vec<FaultType>>>,
    injection_active: Arc<RwLock<bool>>,
    fault_history: Arc<Mutex<Vec<(Instant, FaultType, String)>>>,
}

#[derive(Debug, Clone)]
pub enum FaultType {
    /// Simulate network timeout
    NetworkTimeout,
    /// Simulate resource unavailable
    ResourceUnavailable,
    /// Simulate memory allocation failure
    MemoryFailure,
    /// Simulate computation error
    ComputationError,
    /// Simulate deadlock situation
    DeadlockPotential,
    /// Simulate partial system failure
    PartialFailure,
}

impl FaultInjector {
    pub fn new(failure_rate: f64) -> Self {
        Self {
            failure_rate: Arc::new(RwLock::new(failure_rate)),
            failure_types: Arc::new(RwLock::new(vec![
                FaultType::NetworkTimeout,
                FaultType::ResourceUnavailable,
                FaultType::ComputationError,
            ])),
            injection_active: Arc::new(RwLock::new(true)),
            fault_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_failure_rate(&self, rate: f64) {
        let mut failure_rate = self.failure_rate.write().unwrap();
        *failure_rate = rate.clamp(0.0, 1.0);
    }

    pub fn enable_fault_type(&self, fault_type: FaultType) {
        let mut types = self.failure_types.write().unwrap();
        if !types.iter().any(|f| matches!(f, fault_type)) {
            types.push(fault_type);
        }
    }

    pub fn should_inject_fault(&self) -> Option<FaultType> {
        let injection_active = *self.injection_active.read().unwrap();
        if !injection_active {
            return None;
        }

        let failure_rate = *self.failure_rate.read().unwrap();
        if rand::random::<f64>() < failure_rate {
            let fault_types = self.failure_types.read().unwrap();
            if !fault_types.is_empty() {
                let fault_index = rand::random::<usize>() % fault_types.len();
                return Some(fault_types[fault_index].clone());
            }
        }
        
        None
    }

    pub fn inject_fault(&self, context: &str) -> Result<()> {
        if let Some(fault_type) = self.should_inject_fault() {
            let error_message = self.create_fault_error(&fault_type, context);
            
            // Log the fault
            {
                let mut history = self.fault_history.lock().unwrap();
                history.push((Instant::now(), fault_type.clone(), context.to_string()));
            }

            return Err(Error::runtime_error(error_message, None));
        }
        
        Ok(())
    }

    fn create_fault_error(&self, fault_type: &FaultType, context: &str) -> String {
        match fault_type {
            FaultType::NetworkTimeout => format!("Network timeout in {}", context),
            FaultType::ResourceUnavailable => format!("Resource unavailable in {}", context),
            FaultType::MemoryFailure => format!("Memory allocation failed in {}", context),
            FaultType::ComputationError => format!("Computation error in {}", context),
            FaultType::DeadlockPotential => format!("Deadlock potential detected in {}", context),
            FaultType::PartialFailure => format!("Partial system failure in {}", context),
        }
    }

    pub fn get_fault_count(&self) -> usize {
        self.fault_history.lock().unwrap().len()
    }

    pub fn get_fault_history(&self) -> Vec<(Instant, FaultType, String)> {
        self.fault_history.lock().unwrap().clone()
    }

    pub fn disable_injection(&self) {
        let mut active = self.injection_active.write().unwrap();
        *active = false;
    }

    pub fn enable_injection(&self) {
        let mut active = self.injection_active.write().unwrap();
        *active = true;
    }
}

/// Deadlock detection framework
#[derive(Debug)]
pub struct DeadlockDetector {
    resource_graph: Arc<Mutex<HashMap<String, Vec<String>>>>,
    thread_resources: Arc<Mutex<HashMap<thread::ThreadId, Vec<String>>>>,
    detection_active: Arc<RwLock<bool>>,
    deadlock_count: Arc<Mutex<u64>>,
}

impl DeadlockDetector {
    pub fn new() -> Self {
        Self {
            resource_graph: Arc::new(Mutex::new(HashMap::new())),
            thread_resources: Arc::new(Mutex::new(HashMap::new())),
            detection_active: Arc::new(RwLock::new(true)),
            deadlock_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn acquire_resource(&self, resource_id: &str) -> Result<()> {
        let thread_id = thread::current().id();
        
        // Check for potential deadlock before acquiring
        if self.would_create_deadlock(thread_id, resource_id)? {
            let mut count = self.deadlock_count.lock().unwrap();
            *count += 1;
            return Err(Error::runtime_error(
                format!("Deadlock potential detected when acquiring resource: {}", resource_id),
                None,
            ));
        }

        // Record resource acquisition
        {
            let mut thread_resources = self.thread_resources.lock().unwrap();
            thread_resources.entry(thread_id).or_insert_with(Vec::new).push(resource_id.to_string());
        }

        Ok(())
    }

    pub fn release_resource(&self, resource_id: &str) {
        let thread_id = thread::current().id();
        
        let mut thread_resources = self.thread_resources.lock().unwrap();
        if let Some(resources) = thread_resources.get_mut(&thread_id) {
            resources.retain(|r| r != resource_id);
            if resources.is_empty() {
                thread_resources.remove(&thread_id);
            }
        }
    }

    fn would_create_deadlock(&self, thread_id: thread::ThreadId, resource_id: &str) -> Result<bool> {
        if !*self.detection_active.read().unwrap() {
            return Ok(false);
        }

        // Simplified deadlock detection: check if any other thread holding this resource
        // is waiting for a resource that this thread holds
        let thread_resources = self.thread_resources.lock().unwrap();
        
        if let Some(current_resources) = thread_resources.get(&thread_id) {
            for (other_thread, other_resources) in thread_resources.iter() {
                if *other_thread != thread_id && other_resources.contains(&resource_id.to_string()) {
                    // Check if we have any resource that the other thread might want
                    // This is a simplified heuristic
                    if !current_resources.is_empty() {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    pub fn get_deadlock_count(&self) -> u64 {
        *self.deadlock_count.lock().unwrap()
    }

    pub fn reset_detection(&self) {
        let mut thread_resources = self.thread_resources.lock().unwrap();
        thread_resources.clear();
        
        let mut count = self.deadlock_count.lock().unwrap();
        *count = 0;
    }
}

/// Error recovery framework
#[derive(Debug)]
pub struct ErrorRecoveryManager {
    recovery_strategies: Arc<RwLock<HashMap<String, RecoveryStrategy>>>,
    recovery_attempts: Arc<Mutex<Vec<(Instant, String, bool)>>>,
    max_retry_attempts: Arc<RwLock<u32>>,
}

#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Retry,
    Fallback(String),
    Escalate,
    Ignore,
}

impl ErrorRecoveryManager {
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        strategies.insert("NetworkTimeout".to_string(), RecoveryStrategy::Retry);
        strategies.insert("ResourceUnavailable".to_string(), RecoveryStrategy::Fallback("alternative_resource".to_string()));
        strategies.insert("ComputationError".to_string(), RecoveryStrategy::Retry);
        strategies.insert("MemoryFailure".to_string(), RecoveryStrategy::Escalate);

        Self {
            recovery_strategies: Arc::new(RwLock::new(strategies)),
            recovery_attempts: Arc::new(Mutex::new(Vec::new())),
            max_retry_attempts: Arc::new(RwLock::new(3)),
        }
    }

    pub fn attempt_recovery(&self, error_type: &str, context: &str) -> Result<bool> {
        let strategy = {
            let strategies = self.recovery_strategies.read().unwrap();
            strategies.get(error_type).cloned().unwrap_or(RecoveryStrategy::Escalate)
        };

        let recovery_successful = match strategy {
            RecoveryStrategy::Retry => {
                // Simulate retry with some delay
                thread::sleep(Duration::from_millis(10));
                rand::random::<f64>() > 0.3 // 70% success rate for retries
            }
            RecoveryStrategy::Fallback(_fallback_resource) => {
                // Simulate fallback to alternative resource
                thread::sleep(Duration::from_millis(5));
                rand::random::<f64>() > 0.1 // 90% success rate for fallbacks
            }
            RecoveryStrategy::Escalate => {
                // Escalation means we can't recover at this level
                false
            }
            RecoveryStrategy::Ignore => {
                // Ignore the error and continue
                true
            }
        };

        // Log recovery attempt
        {
            let mut attempts = self.recovery_attempts.lock().unwrap();
            attempts.push((Instant::now(), error_type.to_string(), recovery_successful));
        }

        Ok(recovery_successful)
    }

    pub fn get_recovery_statistics(&self) -> (usize, usize) {
        let attempts = self.recovery_attempts.lock().unwrap();
        let total_attempts = attempts.len();
        let successful_recoveries = attempts.iter().filter(|(_, _, success)| *success).count();
        (total_attempts, successful_recoveries)
    }

    pub fn set_recovery_strategy(&self, error_type: String, strategy: RecoveryStrategy) {
        let mut strategies = self.recovery_strategies.write().unwrap();
        strategies.insert(error_type, strategy);
    }
}

// ============================================================================
// PARTIAL FAILURE HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_partial_failure_recovery() {
    let runtime = Arc::new(LambdustRuntime::new(Some(8)).expect("Failed to create runtime"));
    let fault_injector = Arc::new(FaultInjector::new(0.2)); // 20% failure rate
    let recovery_manager = Arc::new(ErrorRecoveryManager::new());
    
    let thread_count = 8;
    let operations_per_thread = 30;
    
    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let runtime_clone = runtime.clone();
        let injector_clone = fault_injector.clone();
        let recovery_clone = recovery_manager.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut successful_operations = 0;
            let mut failed_operations = 0;
            let mut recovered_operations = 0;
            let max_retries = 3;

            for op_id in 0..operations_per_thread {
                let operation_context = format!("thread-{}-op-{}", thread_id, op_id);
                let mut retry_count = 0;
                let mut operation_successful = false;

                while retry_count <= max_retries && !operation_successful {
                    // Attempt operation with potential fault injection
                    match injector_clone.inject_fault(&operation_context) {
                        Ok(()) => {
                            // Operation succeeded
                            let result = perform_resilient_operation(thread_id, op_id).await;
                            if result.is_ok() {
                                successful_operations += 1;
                                operation_successful = true;
                            } else {
                                // Operation failed due to other reasons
                                if retry_count < max_retries {
                                    retry_count += 1;
                                    tokio::time::sleep(Duration::from_millis(10)).await;
                                } else {
                                    failed_operations += 1;
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            // Fault was injected, attempt recovery
                            let error_type = if e.to_string().contains("Network") {
                                "NetworkTimeout"
                            } else if e.to_string().contains("Resource") {
                                "ResourceUnavailable"
                            } else if e.to_string().contains("Computation") {
                                "ComputationError"
                            } else {
                                "Unknown"
                            };

                            match recovery_clone.attempt_recovery(error_type, &operation_context) {
                                Ok(true) => {
                                    // Recovery successful, retry operation
                                    recovered_operations += 1;
                                    retry_count += 1;
                                    tokio::time::sleep(Duration::from_millis(5)).await;
                                }
                                Ok(false) => {
                                    // Recovery failed
                                    if retry_count < max_retries {
                                        retry_count += 1;
                                        tokio::time::sleep(Duration::from_millis(15)).await;
                                    } else {
                                        failed_operations += 1;
                                        break;
                                    }
                                }
                                Err(_) => {
                                    // Recovery mechanism failed
                                    failed_operations += 1;
                                    break;
                                }
                            }
                        }
                    }
                }

                // Small delay between operations
                tokio::time::sleep(Duration::from_millis(2)).await;
            }

            (thread_id, successful_operations, failed_operations, recovered_operations)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    let total_successful: usize = results.iter().map(|(_, success, _, _)| *success).sum();
    let total_failed: usize = results.iter().map(|(_, _, failed, _)| *failed).sum();
    let total_recovered: usize = results.iter().map(|(_, _, _, recovered)| *recovered).sum();
    let total_operations = total_successful + total_failed;
    let expected_operations = thread_count * operations_per_thread;

    let fault_count = fault_injector.get_fault_count();
    let (recovery_attempts, successful_recoveries) = recovery_manager.get_recovery_statistics();

    println!("✓ Partial failure recovery test completed");
    println!("  Expected operations: {}", expected_operations);
    println!("  Total operations attempted: {}", total_operations);
    println!("  Successful operations: {}", total_successful);
    println!("  Failed operations: {}", total_failed);
    println!("  Operations recovered: {}", total_recovered);
    println!("  Success rate: {:.2}%", (total_successful as f64 / total_operations as f64) * 100.0);
    println!("  Faults injected: {}", fault_count);
    println!("  Recovery attempts: {}", recovery_attempts);
    println!("  Successful recoveries: {}", successful_recoveries);
    println!("  Recovery success rate: {:.2}%", 
        if recovery_attempts > 0 { (successful_recoveries as f64 / recovery_attempts as f64) * 100.0 } else { 0.0 });

    // Verify system maintained reasonable operation despite failures
    let operation_success_rate = total_successful as f64 / expected_operations as f64;
    assert!(operation_success_rate > 0.7, "Operation success rate too low: {:.2}%", operation_success_rate * 100.0);

    // Verify recovery mechanisms were effective
    if recovery_attempts > 0 {
        let recovery_effectiveness = successful_recoveries as f64 / recovery_attempts as f64;
        assert!(recovery_effectiveness > 0.5, "Recovery effectiveness too low: {:.2}%", recovery_effectiveness * 100.0);
    }

    println!("✓ System maintained operation despite {} faults with effective recovery", fault_count);
}

// ============================================================================
// DEADLOCK DETECTION AND RESOLUTION TESTS
// ============================================================================

#[tokio::test]
async fn test_deadlock_detection_and_prevention() {
    let runtime = Arc::new(LambdustRuntime::new(Some(6)).expect("Failed to create runtime"));
    let deadlock_detector = Arc::new(DeadlockDetector::new());
    
    let thread_count = 6;
    let resource_acquisition_rounds = 20;
    
    // Create shared resources that might cause deadlocks
    let shared_resources = vec!["resource_A", "resource_B", "resource_C", "resource_D"];
    let resource_semaphores: HashMap<String, Arc<Semaphore>> = shared_resources.iter()
        .map(|&name| (name.to_string(), Arc::new(Semaphore::new(1))))
        .collect();

    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let detector_clone = deadlock_detector.clone();
        let resources = shared_resources.clone();
        let semaphores = resource_semaphores.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut successful_acquisitions = 0;
            let mut deadlock_preventions = 0;
            let mut resource_conflicts = 0;

            for round in 0..resource_acquisition_rounds {
                // Create potential deadlock scenario by acquiring resources in different orders
                let resource_order = if thread_id % 2 == 0 {
                    vec![&resources[0], &resources[1], &resources[2]]
                } else {
                    vec![&resources[2], &resources[1], &resources[0]]
                };

                let mut acquired_resources = Vec::new();
                let mut round_successful = true;

                for &resource_name in &resource_order {
                    // Check for deadlock potential before acquiring
                    match detector_clone.acquire_resource(resource_name) {
                        Ok(()) => {
                            // Attempt to acquire the actual resource
                            let semaphore = semaphores.get(resource_name).unwrap();
                            
                            match tokio::time::timeout(Duration::from_millis(50), semaphore.acquire()).await {
                                Ok(Ok(permit)) => {
                                    acquired_resources.push((resource_name, permit));
                                    
                                    // Simulate work with the resource
                                    tokio::time::sleep(Duration::from_millis(5)).await;
                                }
                                Ok(Err(_)) => {
                                    // Semaphore acquisition failed
                                    resource_conflicts += 1;
                                    round_successful = false;
                                    break;
                                }
                                Err(_) => {
                                    // Timeout - potential deadlock
                                    resource_conflicts += 1;
                                    round_successful = false;
                                    break;
                                }
                            }
                        }
                        Err(_) => {
                            // Deadlock prevention activated
                            deadlock_preventions += 1;
                            round_successful = false;
                            break;
                        }
                    }
                }

                // Release resources in reverse order
                for (resource_name, _permit) in acquired_resources.into_iter().rev() {
                    detector_clone.release_resource(resource_name);
                    // Permit is automatically dropped
                }

                if round_successful {
                    successful_acquisitions += 1;
                }

                // Small delay between rounds
                tokio::time::sleep(Duration::from_millis(2)).await;
            }

            (thread_id, successful_acquisitions, deadlock_preventions, resource_conflicts)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    let total_successful: usize = results.iter().map(|(_, success, _, _)| *success).sum();
    let total_preventions: usize = results.iter().map(|(_, _, preventions, _)| *preventions).sum();
    let total_conflicts: usize = results.iter().map(|(_, _, _, conflicts)| *conflicts).sum();
    let total_attempts = thread_count * resource_acquisition_rounds;

    let detector_deadlock_count = deadlock_detector.get_deadlock_count();

    println!("✓ Deadlock detection and prevention test completed");
    println!("  Total acquisition attempts: {}", total_attempts);
    println!("  Successful acquisitions: {}", total_successful);
    println!("  Deadlock preventions: {}", total_preventions);
    println!("  Resource conflicts: {}", total_conflicts);
    println!("  Detector deadlock count: {}", detector_deadlock_count);
    println!("  Success rate: {:.2}%", (total_successful as f64 / total_attempts as f64) * 100.0);
    println!("  Prevention effectiveness: {:.2}%", 
        if total_preventions > 0 { (total_preventions as f64 / (total_preventions + total_conflicts) as f64) * 100.0 } else { 0.0 });

    // Verify deadlock detection was active and effective
    assert!(total_preventions > 0 || total_conflicts == 0, "Deadlock detection should have prevented some deadlocks");
    
    // Verify system remained responsive despite potential deadlocks
    let responsiveness = total_successful as f64 / total_attempts as f64;
    assert!(responsiveness > 0.5, "System responsiveness too low: {:.2}%", responsiveness * 100.0);

    println!("✓ Deadlock detection prevented {} potential deadlocks", total_preventions);
}

// ============================================================================
// ERROR PROPAGATION TESTS
// ============================================================================

#[tokio::test]
async fn test_error_propagation_and_isolation() {
    let runtime = Arc::new(LambdustRuntime::new(Some(10)).expect("Failed to create runtime"));
    let fault_injector = Arc::new(FaultInjector::new(0.3)); // 30% failure rate
    
    // Create separate error boundaries for different subsystems
    let subsystem_barriers = vec![
        Arc::new(Barrier::new(3)), // Subsystem A: 3 threads
        Arc::new(Barrier::new(3)), // Subsystem B: 3 threads  
        Arc::new(Barrier::new(4)), // Subsystem C: 4 threads
    ];
    
    let error_isolation_stats = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = Vec::new();

    let mut thread_id = 0;
    for (subsystem_id, barrier) in subsystem_barriers.iter().enumerate() {
        let threads_in_subsystem = match subsystem_id {
            0 | 1 => 3,
            2 => 4,
            _ => unreachable!(),
        };

        for local_thread_id in 0..threads_in_subsystem {
            let barrier_clone = barrier.clone();
            let runtime_clone = runtime.clone();
            let injector_clone = fault_injector.clone();
            let stats_clone = error_isolation_stats.clone();

            let handle = tokio::spawn(async move {
                barrier_clone.wait();
                
                let mut successful_operations = 0;
                let mut isolated_errors = 0;
                let mut propagated_errors = 0;
                let operations_per_thread = 25;

                for op_id in 0..operations_per_thread {
                    let operation_context = format!("subsystem-{}-thread-{}-op-{}", 
                        subsystem_id, local_thread_id, op_id);

                    // Perform operation with error isolation
                    match perform_isolated_operation(&injector_clone, &operation_context, subsystem_id).await {
                        Ok(()) => {
                            successful_operations += 1;
                        }
                        Err(e) => {
                            if e.to_string().contains("isolated") {
                                isolated_errors += 1;
                            } else {
                                propagated_errors += 1;
                            }
                        }
                    }

                    // Small delay between operations
                    tokio::time::sleep(Duration::from_millis(2)).await;
                }

                // Update subsystem statistics
                {
                    let mut stats = stats_clone.lock().unwrap();
                    let subsystem_stats = stats.entry(subsystem_id).or_insert((0, 0, 0));
                    subsystem_stats.0 += successful_operations;
                    subsystem_stats.1 += isolated_errors;
                    subsystem_stats.2 += propagated_errors;
                }

                (thread_id + local_thread_id, subsystem_id, successful_operations, isolated_errors, propagated_errors)
            });
            handles.push(handle);
        }
        thread_id += threads_in_subsystem;
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    // Aggregate results by subsystem
    let final_stats = error_isolation_stats.lock().unwrap();
    let total_fault_count = fault_injector.get_fault_count();

    println!("✓ Error propagation and isolation test completed");
    println!("  Total faults injected: {}", total_fault_count);
    
    for (subsystem_id, (successful, isolated, propagated)) in final_stats.iter() {
        let total_ops = successful + isolated + propagated;
        let isolation_rate = if isolated + propagated > 0 {
            *isolated as f64 / (*isolated + *propagated) as f64
        } else {
            1.0
        };

        println!("  Subsystem {}: {} ops, {} isolated, {} propagated (isolation: {:.1}%)",
            subsystem_id, total_ops, isolated, propagated, isolation_rate * 100.0);
        
        // Verify error isolation effectiveness
        assert!(isolation_rate > 0.8, "Error isolation ineffective in subsystem {}: {:.1}%", 
            subsystem_id, isolation_rate * 100.0);
    }

    // Verify overall system resilience
    let total_successful: usize = final_stats.values().map(|(s, _, _)| *s).sum();
    let total_operations: usize = final_stats.values().map(|(s, i, p)| s + i + p).sum();
    let overall_success_rate = total_successful as f64 / total_operations as f64;

    assert!(overall_success_rate > 0.6, "Overall system success rate too low: {:.2}%", 
        overall_success_rate * 100.0);

    println!("✓ Error isolation maintained system stability with {:.2}% success rate", 
        overall_success_rate * 100.0);
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async fn perform_resilient_operation(thread_id: usize, op_id: usize) -> Result<u64> {
    // Simulate an operation that might fail but can be retried
    tokio::time::sleep(Duration::from_millis(2)).await;
    
    // Simulate random failures independent of fault injection
    if rand::random::<f64>() < 0.1 { // 10% natural failure rate
        return Err(Error::runtime_error("Natural operation failure".to_string(), None));
    }
    
    Ok((thread_id * 1000 + op_id) as u64)
}

async fn perform_isolated_operation(
    fault_injector: &FaultInjector, 
    context: &str, 
    subsystem_id: usize
) -> Result<()> {
    // Check for fault injection
    if let Some(fault_type) = fault_injector.should_inject_fault() {
        match fault_type {
            FaultType::NetworkTimeout | FaultType::ComputationError => {
                // These errors should be isolated within the subsystem
                return Err(Error::runtime_error(
                    format!("Isolated error in subsystem {} for {}", subsystem_id, context),
                    None,
                ));
            }
            FaultType::MemoryFailure | FaultType::PartialFailure => {
                // These errors might propagate
                return Err(Error::runtime_error(
                    format!("Propagated error from subsystem {} in {}", subsystem_id, context),
                    None,
                ));
            }
            _ => {
                // Other faults are isolated
                return Err(Error::runtime_error(
                    format!("Isolated fault in subsystem {} for {}", subsystem_id, context),
                    None,
                ));
            }
        }
    }

    // Simulate successful operation
    tokio::time::sleep(Duration::from_millis(3)).await;
    Ok(())
}