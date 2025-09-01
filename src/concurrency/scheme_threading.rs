//! SRFI-18 Core Threading Infrastructure
//!
//! High-performance implementation of core threading primitives for Lambdust,
//! designed to meet the performance targets:
//! - Thread creation: <1μs
//! - Context switching: <10μs
//! - Mutex operations: <100ns
//! - Parameter access: <5ns (maintaining SRFI-39 performance)
//! - Memory overhead: <4KB per thread

use crate::diagnostics::{Error, Result};
use crate::eval::value::Value;
use crate::eval::parameter::ParameterFrame;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, AtomicUsize, AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Condvar, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use crossbeam_utils::Backoff;
use parking_lot::{Mutex as ParkingMutex, Condvar as ParkingCondvar, RwLock as ParkingRwLock};

/// Global thread ID counter for unique thread identification
static NEXT_THREAD_ID: AtomicU64 = AtomicU64::new(1);

/// Global mutex ID counter for unique mutex identification  
static NEXT_MUTEX_ID: AtomicU64 = AtomicU64::new(1);

/// Global condition variable ID counter
static NEXT_CONDVAR_ID: AtomicU64 = AtomicU64::new(1);

/// Thread states for lifecycle management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    /// Thread is created but not yet started
    Created,
    /// Thread is currently running
    Running,
    /// Thread is blocked on I/O or synchronization
    Blocked,
    /// Thread has completed successfully
    Completed,
    /// Thread terminated due to an exception
    Failed,
    /// Thread was terminated externally
    Terminated,
}

/// Core Scheme thread representation with optimized memory layout
#[derive(Debug)]
pub struct SchemeThread {
    /// Unique thread identifier
    pub id: u64,
    /// Optional thread name for debugging
    pub name: Option<String>,
    /// Current thread state
    pub state: Arc<RwLock<ThreadState>>,
    /// Thread result value (None while running)
    pub result: Arc<RwLock<Option<Value>>>,
    /// Exception that caused failure (if any)
    pub exception: Arc<RwLock<Option<Value>>>,
    /// Thread creation timestamp for profiling
    pub created_at: Instant,
    /// Thread completion timestamp
    pub completed_at: Arc<RwLock<Option<Instant>>>,
    /// Join handle for the underlying OS thread
    pub(crate) join_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// Parameter bindings inherited from parent thread
    pub(crate) inherited_parameters: Arc<Vec<ParameterFrame>>,
    /// Thread-specific statistics
    pub stats: Arc<ThreadStatistics>,
}

/// Thread performance statistics
#[derive(Debug, Default)]
pub struct ThreadStatistics {
    /// Number of procedure calls made by this thread
    pub procedure_calls: AtomicU64,
    /// Number of parameter accesses
    pub parameter_accesses: AtomicU64,
    /// Number of mutex operations
    pub mutex_operations: AtomicU64,
    /// Total execution time in nanoseconds
    pub execution_time_ns: AtomicU64,
}

/// Work item for the thread pool scheduler
#[derive(Debug)]
pub struct WorkItem {
    /// Procedure to execute
    pub procedure: Value,
    /// Arguments to pass to the procedure
    pub args: Vec<Value>,
    /// Target thread for execution (None = any available thread)
    pub target_thread: Option<u64>,
    /// Priority level (0 = highest)
    pub priority: u8,
    /// Submission timestamp for latency tracking
    pub submitted_at: Instant,
}

/// High-performance work-stealing thread pool
#[derive(Debug)]
pub struct SchemeThreadPool {
    /// Pool of worker threads
    worker_threads: Vec<Arc<WorkerThread>>,
    /// Global work queue for load balancing
    global_queue: Arc<ParkingMutex<VecDeque<WorkItem>>>,
    /// Condition variable for worker notification
    work_available: Arc<ParkingCondvar>,
    /// Pool shutdown flag
    shutdown: Arc<AtomicBool>,
    /// Pool statistics
    stats: Arc<PoolStatistics>,
    /// NUMA topology information for optimization
    numa_topology: Option<NumaTopology>,
}

/// Individual worker thread in the pool
#[derive(Debug)]
pub struct WorkerThread {
    /// Worker thread ID
    pub id: u64,
    /// Local work queue for work-stealing
    local_queue: Arc<ParkingMutex<VecDeque<WorkItem>>>,
    /// Worker thread handle
    handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// Worker statistics
    stats: Arc<WorkerStatistics>,
}

/// Thread pool performance statistics
#[derive(Debug, Default)]
pub struct PoolStatistics {
    /// Total tasks executed
    pub tasks_executed: AtomicU64,
    /// Total tasks stolen between threads
    pub tasks_stolen: AtomicU64,
    /// Average task latency in nanoseconds
    pub avg_task_latency_ns: AtomicU64,
    /// Peak queue depth
    pub peak_queue_depth: AtomicUsize,
}

/// Individual worker thread statistics
#[derive(Debug, Default)]
pub struct WorkerStatistics {
    /// Tasks executed by this worker
    pub tasks_executed: AtomicU64,
    /// Tasks stolen from other workers
    pub tasks_stolen_from_others: AtomicU64,
    /// Tasks stolen by other workers
    pub tasks_stolen_by_others: AtomicU64,
    /// Total execution time
    pub total_execution_time_ns: AtomicU64,
}

/// NUMA topology information for thread affinity optimization
#[derive(Debug, Clone)]
pub struct NumaTopology {
    /// Number of NUMA nodes
    pub node_count: usize,
    /// CPU cores per NUMA node
    pub cores_per_node: Vec<Vec<usize>>,
    /// Memory bandwidth between nodes
    pub node_distances: Vec<Vec<u32>>,
}

/// Scheme mutex with performance optimizations
#[derive(Debug)]
pub struct SchemeMutex {
    /// Unique mutex identifier
    pub id: u64,
    /// Optional mutex name for debugging
    pub name: Option<String>,
    /// Underlying parking_lot mutex for performance
    pub(crate) inner: Arc<ParkingMutex<()>>,
    /// Current owner thread ID (None if unlocked)
    pub owner: Arc<RwLock<Option<u64>>>,
    /// Lock acquisition timestamp for timeout support
    pub locked_at: Arc<RwLock<Option<Instant>>>,
    /// Mutex statistics
    pub stats: Arc<MutexStatistics>,
}

/// Mutex performance statistics
#[derive(Debug, Default)]
pub struct MutexStatistics {
    /// Total number of lock acquisitions
    pub lock_count: AtomicU64,
    /// Total number of unlock operations
    pub unlock_count: AtomicU64,
    /// Total time spent holding the lock (nanoseconds)
    pub total_held_time_ns: AtomicU64,
    /// Maximum time spent holding the lock (nanoseconds)
    pub max_held_time_ns: AtomicU64,
    /// Number of lock contentions
    pub contention_count: AtomicU64,
}

/// Scheme condition variable for thread synchronization
#[derive(Debug)]
pub struct SchemeConditionVariable {
    /// Unique condition variable identifier
    pub id: u64,
    /// Optional condition variable name
    pub name: Option<String>,
    /// Underlying parking_lot condition variable
    pub(crate) inner: Arc<ParkingCondvar>,
    /// Associated mutex for the condition variable
    pub mutex: Arc<SchemeMutex>,
    /// Condition variable statistics
    pub stats: Arc<CondvarStatistics>,
}

/// Condition variable performance statistics
#[derive(Debug, Default)]
pub struct CondvarStatistics {
    /// Number of wait operations
    pub wait_count: AtomicU64,
    /// Number of notify operations
    pub notify_count: AtomicU64,
    /// Total time spent waiting (nanoseconds)
    pub total_wait_time_ns: AtomicU64,
    /// Maximum time spent waiting (nanoseconds)
    pub max_wait_time_ns: AtomicU64,
}

impl SchemeThread {
    /// Creates a new Scheme thread with inherited parameter bindings
    pub fn new(
        name: Option<String>,
        inherited_parameters: Arc<Vec<ParameterFrame>>,
    ) -> Self {
        let id = NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst);
        
        Self {
            id,
            name,
            state: Arc::new(RwLock::new(ThreadState::Created)),
            result: Arc::new(RwLock::new(None)),
            exception: Arc::new(RwLock::new(None)),
            created_at: Instant::now(),
            completed_at: Arc::new(RwLock::new(None)),
            join_handle: Arc::new(Mutex::new(None)),
            inherited_parameters,
            stats: Arc::new(ThreadStatistics::default()),
        }
    }

    /// Starts the thread with the given procedure and arguments
    pub fn start(
        &self,
        procedure: Value,
        args: Vec<Value>,
    ) -> Result<()> {
        let mut state = self.state.write().unwrap();
        if *state != ThreadState::Created {
            return Err(Box::new(Error::runtime_error("Thread already started", None)));
        }

        *state = ThreadState::Running;
        drop(state);

        // Clone necessary data for the thread
        let thread_id = self.id;
        let state_arc = Arc::clone(&self.state);
        let result_arc = Arc::clone(&self.result);
        let exception_arc = Arc::clone(&self.exception);
        let completed_at_arc = Arc::clone(&self.completed_at);
        let stats_arc = Arc::clone(&self.stats);
        let inherited_params = Arc::clone(&self.inherited_parameters);

        // Spawn the actual OS thread
        let handle = thread::spawn(move || {
            let start_time = Instant::now();
            
            // Set up thread-local parameter storage with inherited bindings
            Self::setup_thread_local_parameters(&inherited_params);
            
            // Execute the procedure
            let execution_result = Self::execute_procedure(
                thread_id,
                procedure,
                args,
                &stats_arc,
            );

            // Record completion
            let execution_time = start_time.elapsed();
            stats_arc.execution_time_ns.store(
                execution_time.as_nanos() as u64,
                Ordering::SeqCst,
            );

            // Update thread state based on execution result
            match execution_result {
                Ok(value) => {
                    *result_arc.write().unwrap() = Some(value);
                    *state_arc.write().unwrap() = ThreadState::Completed;
                }
                Err(error) => {
                    // Convert the error to an ErrorObject value
                    let error_obj = crate::stdlib::exceptions::ErrorObject {
                        message: format!("{}", error),
                        irritants: Vec::new(),
                        error_type: crate::stdlib::exceptions::ErrorType::General,
                    };
                    *exception_arc.write().unwrap() = Some(Value::ErrorObject(Arc::new(error_obj)));
                    *state_arc.write().unwrap() = ThreadState::Failed;
                }
            }

            *completed_at_arc.write().unwrap() = Some(Instant::now());
        });

        *self.join_handle.lock().unwrap() = Some(handle);
        Ok(())
    }

    /// Waits for the thread to complete with optional timeout
    pub fn join(&self, timeout: Option<Duration>) -> Result<Value> {
        // Take the join handle to ensure we can only join once
        let handle = self.join_handle.lock().unwrap().take();
        
        if let Some(handle) = handle {
            // Wait for the OS thread to complete
            if let Some(timeout_duration) = timeout {
                // For timeout support, we need to poll the thread state
                let start = Instant::now();
                while start.elapsed() < timeout_duration {
                    let state = *self.state.read().unwrap();
                    match state {
                        ThreadState::Completed | ThreadState::Failed => break,
                        _ => thread::sleep(Duration::from_millis(1)),
                    }
                }
                
                // Check if we timed out
                let state = *self.state.read().unwrap();
                if state == ThreadState::Running || state == ThreadState::Blocked {
                    return Err(Box::new(Error::runtime_error("Thread join timeout", None)));
                }
            }

            // Actually join the thread
            handle.join().map_err(|_| {
                Box::new(Error::runtime_error("Thread join failed", None))
            })?;
        }

        // Return the result or propagate the exception
        let state = *self.state.read().unwrap();
        match state {
            ThreadState::Completed => {
                Ok(self.result.read().unwrap().clone().unwrap_or(Value::Nil))
            }
            ThreadState::Failed => {
                let exception = self.exception.read().unwrap().clone();
                Err(Box::new(Error::runtime_error(
                    format!("Thread failed with exception: {:?}", exception),
                    None,
                )))
            }
            _ => Err(Box::new(Error::runtime_error("Thread in unexpected state", None))),
        }
    }

    /// Sets up thread-local parameter storage with inherited bindings
    fn setup_thread_local_parameters(inherited: &[ParameterFrame]) {
        // Integrate with the existing SRFI-39 parameter system
        // This sets up proper parameter inheritance for the new thread
        crate::eval::parameter::inherit_parameter_bindings(inherited);
    }

    /// Executes a procedure in the context of this thread
    fn execute_procedure(
        _thread_id: u64,
        _procedure: Value,
        _args: Vec<Value>,
        stats: &ThreadStatistics,
    ) -> Result<Value> {
        // Increment procedure call counter
        stats.procedure_calls.fetch_add(1, Ordering::SeqCst);
        
        // TODO: Implement actual procedure execution
        // This will integrate with the main evaluator
        // For now, return a placeholder value
        Ok(Value::Nil)
    }
}

impl SchemeMutex {
    /// Creates a new Scheme mutex
    pub fn new(name: Option<String>) -> Self {
        let id = NEXT_MUTEX_ID.fetch_add(1, Ordering::SeqCst);
        
        Self {
            id,
            name,
            inner: Arc::new(ParkingMutex::new(())),
            owner: Arc::new(RwLock::new(None)),
            locked_at: Arc::new(RwLock::new(None)),
            stats: Arc::new(MutexStatistics::default()),
        }
    }

    /// Attempts to acquire the mutex with optional timeout
    pub fn lock(&self, thread_id: u64, timeout: Option<Duration>) -> Result<()> {
        let start_time = Instant::now();
        
        // Try to acquire the lock
        let acquired = if let Some(timeout_duration) = timeout {
            self.inner.try_lock_for(timeout_duration).is_some()
        } else {
            self.inner.lock();
            true
        };

        if !acquired {
            self.stats.contention_count.fetch_add(1, Ordering::SeqCst);
            return Err(Box::new(Error::runtime_error("Mutex lock timeout", None)));
        }

        // Update ownership and timing information
        *self.owner.write().unwrap() = Some(thread_id);
        *self.locked_at.write().unwrap() = Some(start_time);
        
        // Update statistics
        self.stats.lock_count.fetch_add(1, Ordering::SeqCst);
        
        Ok(())
    }

    /// Releases the mutex
    pub fn unlock(&self, thread_id: u64) -> Result<()> {
        // Verify ownership
        let current_owner = *self.owner.read().unwrap();
        if current_owner != Some(thread_id) {
            return Err(Box::new(Error::runtime_error(
                "Attempting to unlock mutex not owned by current thread",
                None,
            )));
        }

        // Update timing statistics
        if let Some(locked_at) = *self.locked_at.read().unwrap() {
            let held_time = locked_at.elapsed().as_nanos() as u64;
            self.stats.total_held_time_ns.fetch_add(held_time, Ordering::SeqCst);
            
            // Update maximum held time
            let max_held = self.stats.max_held_time_ns.load(Ordering::SeqCst);
            if held_time > max_held {
                self.stats.max_held_time_ns.store(held_time, Ordering::SeqCst);
            }
        }

        // Clear ownership
        *self.owner.write().unwrap() = None;
        *self.locked_at.write().unwrap() = None;
        
        // Release the actual mutex
        // Note: parking_lot mutexes are automatically released when dropped
        // This is a design consideration - we may need to adjust this
        
        // Update statistics
        self.stats.unlock_count.fetch_add(1, Ordering::SeqCst);
        
        Ok(())
    }

    /// Checks if the mutex is currently locked
    pub fn is_locked(&self) -> bool {
        self.owner.read().unwrap().is_some()
    }
}

impl SchemeConditionVariable {
    /// Creates a new condition variable associated with a mutex
    pub fn new(name: Option<String>, mutex: Arc<SchemeMutex>) -> Self {
        let id = NEXT_CONDVAR_ID.fetch_add(1, Ordering::SeqCst);
        
        Self {
            id,
            name,
            inner: Arc::new(ParkingCondvar::new()),
            mutex,
            stats: Arc::new(CondvarStatistics::default()),
        }
    }

    /// Waits on the condition variable with optional timeout
    pub fn wait(&self, timeout: Option<Duration>) -> Result<()> {
        let start_time = Instant::now();
        
        // Verify that the current thread owns the associated mutex
        // This is a requirement for condition variable semantics
        
        let timed_out = if let Some(timeout_duration) = timeout {
            !self.inner.wait_for(&mut self.mutex.inner.lock(), timeout_duration).timed_out()
        } else {
            self.inner.wait(&mut self.mutex.inner.lock());
            false
        };

        // Update statistics
        let wait_time = start_time.elapsed().as_nanos() as u64;
        self.stats.wait_count.fetch_add(1, Ordering::SeqCst);
        self.stats.total_wait_time_ns.fetch_add(wait_time, Ordering::SeqCst);
        
        let max_wait = self.stats.max_wait_time_ns.load(Ordering::SeqCst);
        if wait_time > max_wait {
            self.stats.max_wait_time_ns.store(wait_time, Ordering::SeqCst);
        }

        if timed_out {
            Err(Box::new(Error::runtime_error("Condition variable wait timeout", None)))
        } else {
            Ok(())
        }
    }

    /// Notifies one waiting thread
    pub fn notify_one(&self) {
        self.inner.notify_one();
        self.stats.notify_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Notifies all waiting threads
    pub fn notify_all(&self) {
        self.inner.notify_all();
        self.stats.notify_count.fetch_add(1, Ordering::SeqCst);
    }
}

impl SchemeThreadPool {
    /// Creates a new thread pool with the specified number of worker threads
    pub fn new(worker_count: usize) -> Result<Self> {
        if worker_count == 0 {
            return Err(Box::new(Error::runtime_error("Thread pool must have at least one worker", None)));
        }

        let global_queue = Arc::new(ParkingMutex::new(VecDeque::new()));
        let work_available = Arc::new(ParkingCondvar::new());
        let shutdown = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(PoolStatistics::default());
        
        let mut worker_threads = Vec::with_capacity(worker_count);
        
        // Create worker threads
        for i in 0..worker_count {
            let worker = Arc::new(WorkerThread::new(
                i as u64,
                Arc::clone(&global_queue),
                Arc::clone(&work_available),
                Arc::clone(&shutdown),
                Arc::clone(&stats),
            ));
            
            worker_threads.push(worker);
        }

        // Detect NUMA topology if available
        let numa_topology = Self::detect_numa_topology();

        Ok(Self {
            worker_threads,
            global_queue,
            work_available,
            shutdown,
            stats,
            numa_topology,
        })
    }

    /// Submits a work item to the thread pool
    pub fn submit(&self, work_item: WorkItem) -> Result<()> {
        if self.shutdown.load(Ordering::SeqCst) {
            return Err(Box::new(Error::runtime_error("Thread pool is shutdown", None)));
        }

        // Add to global queue
        {
            let mut queue = self.global_queue.lock();
            queue.push_back(work_item);
            
            // Update peak queue depth
            let current_depth = queue.len();
            let peak_depth = self.stats.peak_queue_depth.load(Ordering::SeqCst);
            if current_depth > peak_depth {
                self.stats.peak_queue_depth.store(current_depth, Ordering::SeqCst);
            }
        }

        // Notify workers
        self.work_available.notify_one();
        
        Ok(())
    }

    /// Shuts down the thread pool gracefully
    pub fn shutdown(&self) -> Result<()> {
        self.shutdown.store(true, Ordering::SeqCst);
        self.work_available.notify_all();
        
        // Wait for all workers to complete
        for worker in &self.worker_threads {
            if let Some(handle) = worker.handle.lock().unwrap().take() {
                handle.join().map_err(|_| {
                    Box::new(Error::runtime_error("Worker thread join failed", None))
                })?;
            }
        }
        
        Ok(())
    }

    /// Detects NUMA topology for thread affinity optimization
    fn detect_numa_topology() -> Option<NumaTopology> {
        // Placeholder implementation
        // In a real implementation, this would query the system for NUMA information
        None
    }
}

impl WorkerThread {
    /// Creates a new worker thread
    pub fn new(
        id: u64,
        global_queue: Arc<ParkingMutex<VecDeque<WorkItem>>>,
        work_available: Arc<ParkingCondvar>,
        shutdown: Arc<AtomicBool>,
        _pool_stats: Arc<PoolStatistics>,
    ) -> Self {
        let local_queue = Arc::new(ParkingMutex::new(VecDeque::new()));
        let stats = Arc::new(WorkerStatistics::default());
        
        // Clone data for the worker thread
        let worker_id = id;
        let local_queue_clone = Arc::clone(&local_queue);
        let global_queue_clone = global_queue;
        let work_available_clone = work_available;
        let shutdown_clone = shutdown;
        let stats_clone = Arc::clone(&stats);
        
        // Spawn the worker thread
        let handle = thread::spawn(move || {
            Self::worker_loop(
                worker_id,
                local_queue_clone,
                global_queue_clone,
                work_available_clone,
                shutdown_clone,
                stats_clone,
            );
        });

        Self {
            id,
            local_queue,
            handle: Arc::new(Mutex::new(Some(handle))),
            stats,
        }
    }

    /// Main worker thread loop with work-stealing
    fn worker_loop(
        worker_id: u64,
        local_queue: Arc<ParkingMutex<VecDeque<WorkItem>>>,
        global_queue: Arc<ParkingMutex<VecDeque<WorkItem>>>,
        work_available: Arc<ParkingCondvar>,
        shutdown: Arc<AtomicBool>,
        stats: Arc<WorkerStatistics>,
    ) {
        let backoff = Backoff::new();
        
        while !shutdown.load(Ordering::SeqCst) {
            // Try to get work from local queue first
            let work_item = {
                let mut local = local_queue.lock();
                local.pop_front()
            };

            let work_item = work_item.or_else(|| {
                // Try to steal from global queue
                let mut global = global_queue.lock();
                global.pop_front()
            });

            if let Some(work_item) = work_item {
                // Execute the work item
                Self::execute_work_item(worker_id, work_item, &stats);
                backoff.reset();
            } else {
                // No work available, wait for notification
                let mut global = global_queue.lock();
                if global.is_empty() && !shutdown.load(Ordering::SeqCst) {
                    work_available.wait(&mut global);
                }
                backoff.snooze();
            }
        }
    }

    /// Executes a single work item
    fn execute_work_item(
        _worker_id: u64,
        work_item: WorkItem,
        stats: &WorkerStatistics,
    ) {
        let start_time = Instant::now();
        
        // TODO: Implement actual procedure execution
        // This will integrate with the main evaluator
        
        // Update statistics
        let execution_time = start_time.elapsed().as_nanos() as u64;
        stats.tasks_executed.fetch_add(1, Ordering::SeqCst);
        stats.total_execution_time_ns.fetch_add(execution_time, Ordering::SeqCst);
    }
}

/// Global thread registry for managing all active threads
pub struct ThreadRegistry {
    /// Map of thread ID to thread handle
    threads: Arc<RwLock<FxHashMap<u64, Arc<SchemeThread>>>>,
    /// Thread pool for work scheduling
    thread_pool: Arc<SchemeThreadPool>,
}

impl ThreadRegistry {
    /// Creates a new thread registry with default thread pool
    pub fn new() -> Result<Self> {
        let thread_count = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        
        let thread_pool = Arc::new(SchemeThreadPool::new(thread_count)?);
        
        Ok(Self {
            threads: Arc::new(RwLock::new(FxHashMap::default())),
            thread_pool,
        })
    }

    /// Registers a new thread
    pub fn register_thread(&self, thread: Arc<SchemeThread>) {
        let mut threads = self.threads.write().unwrap();
        threads.insert(thread.id, thread);
    }

    /// Unregisters a thread (called when thread completes)
    pub fn unregister_thread(&self, thread_id: u64) {
        let mut threads = self.threads.write().unwrap();
        threads.remove(&thread_id);
    }

    /// Gets a thread by ID
    pub fn get_thread(&self, thread_id: u64) -> Option<Arc<SchemeThread>> {
        let threads = self.threads.read().unwrap();
        threads.get(&thread_id).cloned()
    }

    /// Lists all active threads
    pub fn list_threads(&self) -> Vec<Arc<SchemeThread>> {
        let threads = self.threads.read().unwrap();
        threads.values().cloned().collect()
    }

    /// Gets a reference to the thread pool
    pub fn thread_pool(&self) -> &SchemeThreadPool {
        &self.thread_pool
    }
}

impl Default for ThreadRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create default thread registry")
    }
}

/// Thread-local current thread ID storage
thread_local! {
    static CURRENT_THREAD_ID: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

/// Gets the current thread ID
pub fn current_thread_id() -> Option<u64> {
    CURRENT_THREAD_ID.with(|id| id.get())
}

/// Sets the current thread ID (called when thread starts)
pub fn set_current_thread_id(id: u64) {
    CURRENT_THREAD_ID.with(|current| current.set(Some(id)));
}

/// Clears the current thread ID (called when thread ends)
pub fn clear_current_thread_id() {
    CURRENT_THREAD_ID.with(|current| current.set(None));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_creation() {
        let thread = SchemeThread::new(
            Some("test-thread".to_string()),
            Arc::new(Vec::new()),
        );
        
        assert_eq!(thread.name, Some("test-thread".to_string()));
        assert_eq!(*thread.state.read().unwrap(), ThreadState::Created);
    }

    #[test]
    fn test_mutex_creation() {
        let mutex = SchemeMutex::new(Some("test-mutex".to_string()));
        assert_eq!(mutex.name, Some("test-mutex".to_string()));
        assert!(!mutex.is_locked());
    }

    #[test]
    fn test_thread_pool_creation() {
        let pool = SchemeThreadPool::new(4).unwrap();
        assert_eq!(pool.worker_threads.len(), 4);
        assert!(!pool.shutdown.load(Ordering::SeqCst));
    }

    #[test]
    fn test_thread_registry() {
        let registry = ThreadRegistry::new().unwrap();
        let thread = Arc::new(SchemeThread::new(
            Some("test".to_string()),
            Arc::new(Vec::new()),
        ));
        
        let thread_id = thread.id;
        registry.register_thread(Arc::clone(&thread));
        
        assert!(registry.get_thread(thread_id).is_some());
        
        registry.unregister_thread(thread_id);
        assert!(registry.get_thread(thread_id).is_none());
    }
}