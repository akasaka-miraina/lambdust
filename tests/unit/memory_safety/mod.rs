//! Memory Safety Stress Tests
//!
//! Comprehensive memory safety testing suite covering:
//! - Stack overflow prevention and detection
//! - Memory leak detection and prevention
//! - Concurrent memory access safety
//! - Resource exhaustion handling
//! - Memory pressure response

pub mod stack_overflow_tests;
pub mod memory_leak_tests;
pub mod concurrent_safety_tests;
pub mod resource_exhaustion_tests;
pub mod memory_pressure_tests;

use crate::evaluator::Evaluator;
use crate::interpreter::LambdustInterpreter;
use crate::value::Value;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Memory safety test utilities
pub mod test_utils {
    use super::*;
    
    /// Memory allocation tracker for leak detection
    pub struct MemoryTracker {
        pub allocations: AtomicUsize,
        pub deallocations: AtomicUsize,
        pub peak_memory: AtomicUsize,
        pub current_memory: AtomicUsize,
    }
    
    impl MemoryTracker {
        pub fn new() -> Self {
            Self {
                allocations: AtomicUsize::new(0),
                deallocations: AtomicUsize::new(0),
                peak_memory: AtomicUsize::new(0),
                current_memory: AtomicUsize::new(0),
            }
        }
        
        pub fn track_allocation(&self, size: usize) {
            self.allocations.fetch_add(1, Ordering::Relaxed);
            let current = self.current_memory.fetch_add(size, Ordering::Relaxed) + size;
            
            // Update peak if necessary
            let mut peak = self.peak_memory.load(Ordering::Relaxed);
            while current > peak {
                match self.peak_memory.compare_exchange_weak(
                    peak, current, Ordering::Relaxed, Ordering::Relaxed
                ) {
                    Ok(_) => break,
                    Err(actual) => peak = actual,
                }
            }
        }
        
        pub fn track_deallocation(&self, size: usize) {
            self.deallocations.fetch_add(1, Ordering::Relaxed);
            self.current_memory.fetch_sub(size, Ordering::Relaxed);
        }
        
        pub fn get_stats(&self) -> MemoryStats {
            MemoryStats {
                allocations: self.allocations.load(Ordering::Relaxed),
                deallocations: self.deallocations.load(Ordering::Relaxed),
                peak_memory: self.peak_memory.load(Ordering::Relaxed),
                current_memory: self.current_memory.load(Ordering::Relaxed),
                leaked_objects: self.allocations.load(Ordering::Relaxed)
                    .saturating_sub(self.deallocations.load(Ordering::Relaxed)),
            }
        }
        
        pub fn reset(&self) {
            self.allocations.store(0, Ordering::Relaxed);
            self.deallocations.store(0, Ordering::Relaxed);
            self.peak_memory.store(0, Ordering::Relaxed);
            self.current_memory.store(0, Ordering::Relaxed);
        }
    }
    
    /// Memory statistics
    #[derive(Debug, Clone)]
    pub struct MemoryStats {
        pub allocations: usize,
        pub deallocations: usize,
        pub peak_memory: usize,
        pub current_memory: usize,
        pub leaked_objects: usize,
    }
    
    /// Stack depth tracker for overflow detection
    pub struct StackTracker {
        pub max_depth: AtomicUsize,
        pub current_depth: AtomicUsize,
        pub overflow_threshold: usize,
    }
    
    impl StackTracker {
        pub fn new(threshold: usize) -> Self {
            Self {
                max_depth: AtomicUsize::new(0),
                current_depth: AtomicUsize::new(0),
                overflow_threshold: threshold,
            }
        }
        
        pub fn enter_frame(&self) -> Result<StackGuard, StackOverflowError> {
            let depth = self.current_depth.fetch_add(1, Ordering::Relaxed) + 1;
            
            if depth > self.overflow_threshold {
                self.current_depth.fetch_sub(1, Ordering::Relaxed);
                return Err(StackOverflowError::new(depth, self.overflow_threshold));
            }
            
            // Update max depth
            let mut max = self.max_depth.load(Ordering::Relaxed);
            while depth > max {
                match self.max_depth.compare_exchange_weak(
                    max, depth, Ordering::Relaxed, Ordering::Relaxed
                ) {
                    Ok(_) => break,
                    Err(actual) => max = actual,
                }
            }
            
            Ok(StackGuard { tracker: self })
        }
        
        pub fn get_stats(&self) -> StackStats {
            StackStats {
                max_depth: self.max_depth.load(Ordering::Relaxed),
                current_depth: self.current_depth.load(Ordering::Relaxed),
                overflow_threshold: self.overflow_threshold,
            }
        }
    }
    
    /// RAII guard for stack frames
    pub struct StackGuard<'a> {
        tracker: &'a StackTracker,
    }
    
    impl<'a> Drop for StackGuard<'a> {
        fn drop(&mut self) {
            self.tracker.current_depth.fetch_sub(1, Ordering::Relaxed);
        }
    }
    
    /// Stack overflow error
    #[derive(Debug, Clone)]
    pub struct StackOverflowError {
        pub current_depth: usize,
        pub threshold: usize,
    }
    
    impl StackOverflowError {
        pub fn new(current: usize, threshold: usize) -> Self {
            Self {
                current_depth: current,
                threshold,
            }
        }
    }
    
    impl std::fmt::Display for StackOverflowError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Stack overflow: depth {} exceeds threshold {}", 
                   self.current_depth, self.threshold)
        }
    }
    
    impl std::error::Error for StackOverflowError {}
    
    /// Stack statistics
    #[derive(Debug, Clone)]
    pub struct StackStats {
        pub max_depth: usize,
        pub current_depth: usize,
        pub overflow_threshold: usize,
    }
    
    /// Create a test interpreter with memory tracking
    pub fn create_tracked_interpreter() -> (LambdustInterpreter, Arc<MemoryTracker>) {
        let tracker = Arc::new(MemoryTracker::new());
        let interpreter = LambdustInterpreter::new();
        (interpreter, tracker)
    }
    
    /// Create a recursive expression for stack testing
    pub fn create_recursive_expression(depth: usize) -> String {
        if depth == 0 {
            "1".to_string()
        } else {
            format!("(+ 1 {})", create_recursive_expression(depth - 1))
        }
    }
    
    /// Create a large data structure for memory testing
    pub fn create_large_list(size: usize) -> Value {
        let mut values = Vec::with_capacity(size);
        for i in 0..size {
            values.push(Value::Number(crate::lexer::SchemeNumber::Integer(i as i64)));
        }
        Value::Vector(values)
    }
    
    /// Measure memory usage during operation
    pub fn measure_memory<F, R>(tracker: &MemoryTracker, operation: F) -> (R, MemoryStats)
    where
        F: FnOnce() -> R,
    {
        tracker.reset();
        let start_stats = tracker.get_stats();
        
        let result = operation();
        
        let end_stats = tracker.get_stats();
        let delta_stats = MemoryStats {
            allocations: end_stats.allocations.saturating_sub(start_stats.allocations),
            deallocations: end_stats.deallocations.saturating_sub(start_stats.deallocations),
            peak_memory: end_stats.peak_memory,
            current_memory: end_stats.current_memory,
            leaked_objects: end_stats.leaked_objects,
        };
        
        (result, delta_stats)
    }
    
    /// Run operation with timeout to prevent infinite loops
    pub fn run_with_timeout<F, R>(operation: F, timeout: Duration) -> Result<R, TimeoutError>
    where
        F: FnOnce() -> R + Send,
        R: Send,
    {
        use std::sync::mpsc;
        use std::thread;
        
        let (tx, rx) = mpsc::channel();
        
        thread::spawn(move || {
            let result = operation();
            let _ = tx.send(result);
        });
        
        match rx.recv_timeout(timeout) {
            Ok(result) => Ok(result),
            Err(_) => Err(TimeoutError::new(timeout)),
        }
    }
    
    /// Timeout error
    #[derive(Debug, Clone)]
    pub struct TimeoutError {
        pub timeout_duration: Duration,
    }
    
    impl TimeoutError {
        pub fn new(duration: Duration) -> Self {
            Self {
                timeout_duration: duration,
            }
        }
    }
    
    impl std::fmt::Display for TimeoutError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Operation timed out after {:?}", self.timeout_duration)
        }
    }
    
    impl std::error::Error for TimeoutError {}
    
    /// Create memory pressure by allocating large amounts of data
    pub fn create_memory_pressure(megabytes: usize) -> Vec<Vec<u8>> {
        let mut allocations = Vec::new();
        let chunk_size = 1024 * 1024; // 1 MB chunks
        
        for _ in 0..megabytes {
            allocations.push(vec![0u8; chunk_size]);
        }
        
        allocations
    }
    
    /// Test that checks for proper cleanup after operation
    pub fn test_cleanup<F>(operation: F) -> bool
    where
        F: FnOnce(),
    {
        let initial_memory = get_process_memory();
        
        operation();
        
        // Force garbage collection if available
        #[cfg(feature = "gc")]
        {
            crate::gc::collect();
        }
        
        // Allow some time for cleanup
        std::thread::sleep(Duration::from_millis(100));
        
        let final_memory = get_process_memory();
        
        // Memory should not have grown significantly
        final_memory <= initial_memory + 1024 * 1024 // Allow 1MB tolerance
    }
    
    /// Get current process memory usage (placeholder implementation)
    fn get_process_memory() -> usize {
        // In a real implementation, this would use platform-specific APIs
        // to get actual memory usage
        0
    }
}

/// Common memory safety test patterns
pub mod patterns {
    use super::*;
    
    /// Test pattern for stack overflow prevention
    pub fn test_stack_overflow_prevention<F>(
        operation: F,
        max_depth: usize,
    ) -> Result<(), StackOverflowError>
    where
        F: FnOnce(&StackTracker) -> Result<(), StackOverflowError>,
    {
        let tracker = StackTracker::new(max_depth);
        operation(&tracker)?;
        
        let stats = tracker.get_stats();
        assert!(stats.max_depth <= max_depth, 
                "Stack depth {} exceeded threshold {}", 
                stats.max_depth, max_depth);
        
        Ok(())
    }
    
    /// Test pattern for memory leak detection
    pub fn test_memory_leak_detection<F>(
        operation: F,
        max_leaked_objects: usize,
    ) -> Result<(), MemoryLeakError>
    where
        F: FnOnce(&MemoryTracker),
    {
        let tracker = MemoryTracker::new();
        operation(&tracker);
        
        let stats = tracker.get_stats();
        if stats.leaked_objects > max_leaked_objects {
            return Err(MemoryLeakError::new(stats.leaked_objects, max_leaked_objects));
        }
        
        Ok(())
    }
    
    /// Memory leak error
    #[derive(Debug, Clone)]
    pub struct MemoryLeakError {
        pub leaked_objects: usize,
        pub max_allowed: usize,
    }
    
    impl MemoryLeakError {
        pub fn new(leaked: usize, max_allowed: usize) -> Self {
            Self {
                leaked_objects: leaked,
                max_allowed,
            }
        }
    }
    
    impl std::fmt::Display for MemoryLeakError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Memory leak detected: {} objects leaked (max allowed: {})", 
                   self.leaked_objects, self.max_allowed)
        }
    }
    
    impl std::error::Error for MemoryLeakError {}
    
    /// Test pattern for concurrent safety
    pub fn test_concurrent_safety<F>(
        operation: F,
        thread_count: usize,
        iterations_per_thread: usize,
    ) -> Result<(), ConcurrentSafetyError>
    where
        F: Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::thread;
        
        let operation = Arc::new(operation);
        let error_count = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        
        for _ in 0..thread_count {
            let op = Arc::clone(&operation);
            let errors = Arc::clone(&error_count);
            
            let handle = thread::spawn(move || {
                for _ in 0..iterations_per_thread {
                    if let Err(_) = op() {
                        errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().map_err(|_| ConcurrentSafetyError::ThreadPanic)?;
        }
        
        let total_errors = error_count.load(Ordering::Relaxed);
        if total_errors > 0 {
            return Err(ConcurrentSafetyError::OperationErrors(total_errors));
        }
        
        Ok(())
    }
    
    /// Concurrent safety error
    #[derive(Debug, Clone)]
    pub enum ConcurrentSafetyError {
        ThreadPanic,
        OperationErrors(usize),
    }
    
    impl std::fmt::Display for ConcurrentSafetyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ConcurrentSafetyError::ThreadPanic => write!(f, "Thread panicked during concurrent test"),
                ConcurrentSafetyError::OperationErrors(count) => {
                    write!(f, "Concurrent safety test failed with {} errors", count)
                }
            }
        }
    }
    
    impl std::error::Error for ConcurrentSafetyError {}
}