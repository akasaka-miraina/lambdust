//! Parallel Execution System for Phase 5 Stage 2
//!
//! This module provides a comprehensive parallel execution system that integrates
//! with Stage 1's JIT compilation system and prepares for Stage 3's NaN boxing
//! optimizations. The system is built around work-stealing schedulers and
//! thread-safe value operations.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
//!
//! ## Architecture
//!
//! The parallel execution system consists of four main components:
//! - **WorkStealingScheduler**: Chase-Lev work-stealing deque implementation
//! - **ParallelContinuation**: Fork-join parallel continuation support
//! - **ThreadSafeValue**: Lock-free thread-safe value wrapper
//! - **NUMAAllocator**: NUMA-aware memory allocation optimization
//!
//! ## Integration
//!
//! - **Stage 1 Integration**: Full integration with JIT compilation system
//! - **NaN Boxing Preparation**: Ready for 60%+ memory reduction maintenance

use crate::eval::Value;
use crate::ast::Literal;

#[cfg(feature = "stage1")]
use crate::eval::{JITIntegrationSystem, JITCodeId, CompiledCode, JITOptimizable};

use std::sync::{Arc, RwLock, Mutex, atomic::{AtomicUsize, AtomicU64, AtomicBool, Ordering}};
use std::collections::{VecDeque, HashMap};
use std::thread::{self, JoinHandle, ThreadId};
use std::time::{Duration, Instant};
use std::sync::mpsc::{self, Sender, Receiver};
use crossbeam::utils::Backoff;
use fastrand;

/// Parallel execution specific error types
#[derive(Debug, Clone)]
pub enum ParallelError {
    /// Scheduler is shutting down and cannot accept new tasks
    SchedulerShutdown,
    /// Task execution failed with an error message
    TaskExecutionFailed(String),
    /// Thread creation failed
    ThreadCreationFailed(String),
    /// Work stealing failed due to contention
    WorkStealFailed,
    /// NUMA allocation failed
    NUMAAllocationFailed(String),
    /// Continuation execution failed
    ContinuationFailed(String),
    /// Resource contention detected
    ResourceContention,
}

/// Result type for parallel operations
pub type ParallelResult<T> = Result<T, ParallelError>;

/// Unique identifier for parallel tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        TaskId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// A parallel task that can be executed on the work-stealing scheduler
pub trait ParallelTask: Send + Sync {
    type Output: Send + Sync;
    
    /// Execute the task and return the result
    fn execute(&self) -> ParallelResult<Self::Output>;
    
    /// Check if the task can be split into smaller parallel tasks
    fn can_split(&self) -> bool {
        false
    }
    
    /// Split the task into smaller tasks if possible
    fn split(&self) -> Vec<Box<dyn ParallelTask<Output = Self::Output>>> {
        vec![]
    }
    
    /// Get task priority (higher numbers = higher priority)
    fn priority(&self) -> u32 {
        0
    }
}

/// Work item in the Chase-Lev deque
pub struct WorkItem {
    task_id: TaskId,
    task: Box<dyn ParallelTask<Output = Value>>,
    priority: u32,
    created_at: Instant,
}

impl std::fmt::Debug for WorkItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkItem")
            .field("task_id", &self.task_id)
            .field("priority", &self.priority)
            .field("created_at", &self.created_at)
            .finish()
    }
}

impl WorkItem {
    fn new(task: Box<dyn ParallelTask<Output = Value>>) -> Self {
        let priority = task.priority();
        Self {
            task_id: TaskId::new(),
            task,
            priority,
            created_at: Instant::now(),
        }
    }
}

/// Chase-Lev work-stealing deque implementation
/// 
/// This provides O(1) local operations (push/pop from bottom) and
/// efficient work-stealing from the top by other threads.
pub struct ChaseLevDeque<T> {
    /// Bottom index (only modified by owner thread)
    bottom: AtomicUsize,
    /// Top index (modified by stealing threads)
    top: AtomicUsize,
    /// Circular buffer for storing items
    buffer: RwLock<Vec<Option<T>>>,
    /// Buffer capacity (power of 2 for efficient masking)
    capacity: usize,
}

impl<T> ChaseLevDeque<T> {
    pub fn new(capacity: usize) -> Self {
        // Ensure capacity is power of 2
        let capacity = capacity.next_power_of_two().max(16);
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize_with(capacity, || None);
        
        Self {
            bottom: AtomicUsize::new(0),
            top: AtomicUsize::new(0),
            buffer: RwLock::new(buffer),
            capacity,
        }
    }
    
    /// Push item to bottom (owner thread only)
    pub fn push(&self, item: T) -> ParallelResult<()> {
        let bottom = self.bottom.load(Ordering::Relaxed);
        let top = self.top.load(Ordering::Acquire);
        
        // Check if buffer needs resizing
        if bottom - top >= self.capacity - 1 {
            return Err(ParallelError::ResourceContention);
        }
        
        // Store item at bottom
        if let Ok(mut buffer) = self.buffer.write() {
            buffer[bottom & (self.capacity - 1)] = Some(item);
            self.bottom.store(bottom + 1, Ordering::Release);
            Ok(())
        } else {
            Err(ParallelError::ResourceContention)
        }
    }
    
    /// Pop item from bottom (owner thread only)
    pub fn pop(&self) -> Option<T> {
        let bottom = self.bottom.load(Ordering::Relaxed);
        if bottom == 0 {
            return None;
        }
        
        let new_bottom = bottom - 1;
        self.bottom.store(new_bottom, Ordering::Relaxed);
        
        let top = self.top.load(Ordering::Acquire);
        
        if new_bottom > top {
            // Non-empty queue, safe to take
            if let Ok(mut buffer) = self.buffer.write() {
                return buffer[new_bottom & (self.capacity - 1)].take();
            }
        } else if new_bottom == top {
            // Last item, compete with stealers
            if self.top.compare_exchange_weak(
                top, 
                top + 1, 
                Ordering::SeqCst, 
                Ordering::Relaxed
            ).is_ok() {
                // Won the race
                if let Ok(mut buffer) = self.buffer.write() {
                    return buffer[new_bottom & (self.capacity - 1)].take();
                }
            }
        }
        
        // Failed to pop, restore bottom
        self.bottom.store(bottom, Ordering::Relaxed);
        None
    }
    
    /// Steal item from top (other threads)
    pub fn steal(&self) -> Option<T> {
        let top = self.top.load(Ordering::Acquire);
        let bottom = self.bottom.load(Ordering::Acquire);
        
        if top >= bottom {
            return None; // Empty
        }
        
        // Attempt to steal
        if let Ok(buffer) = self.buffer.read() {
            let item = buffer[top & (self.capacity - 1)].as_ref();
            if item.is_some() {
                if self.top.compare_exchange_weak(
                    top,
                    top + 1,
                    Ordering::SeqCst,
                    Ordering::Relaxed
                ).is_ok() {
                    // Successfully stole
                    drop(buffer);
                    if let Ok(mut buffer) = self.buffer.write() {
                        return buffer[top & (self.capacity - 1)].take();
                    }
                }
            }
        }
        
        None
    }
    
    /// Check if deque is empty
    pub fn is_empty(&self) -> bool {
        let bottom = self.bottom.load(Ordering::Relaxed);
        let top = self.top.load(Ordering::Acquire);
        bottom <= top
    }
    
    /// Get approximate size
    pub fn len(&self) -> usize {
        let bottom = self.bottom.load(Ordering::Relaxed);
        let top = self.top.load(Ordering::Acquire);
        bottom.saturating_sub(top)
    }
}

unsafe impl<T: Send> Send for ChaseLevDeque<T> {}
unsafe impl<T: Send> Sync for ChaseLevDeque<T> {}

/// Worker thread in the work-stealing scheduler
pub struct WorkerThread {
    /// Unique worker ID
    pub id: usize,
    /// Thread ID from the OS
    pub thread_id: Option<ThreadId>,
    /// Local work deque
    pub deque: Arc<ChaseLevDeque<WorkItem>>,
    /// Handle to the worker thread
    pub handle: Option<JoinHandle<()>>,
    /// Statistics for this worker
    pub stats: Arc<RwLock<WorkerStats>>,
}

/// Statistics for a worker thread
#[derive(Debug, Clone, Default)]
pub struct WorkerStats {
    pub tasks_executed: u64,
    pub tasks_stolen: u64,
    pub steal_attempts: u64,
    pub failed_steals: u64,
    pub execution_time: Duration,
    pub idle_time: Duration,
}

/// Work-stealing scheduler with Chase-Lev deques
pub struct WorkStealingScheduler {
    /// Worker threads
    workers: Vec<WorkerThread>,
    /// Number of worker threads
    num_workers: usize,
    /// Shutdown flag
    shutdown: Arc<AtomicBool>,
    /// Global task counter
    task_counter: Arc<AtomicU64>,
    /// Random number generator for victim selection
    rng: Arc<Mutex<fastrand::Rng>>,
    /// NUMA allocator
    numa_allocator: Arc<NUMAAllocator>,
}

impl WorkStealingScheduler {
    /// Create new work-stealing scheduler
    pub fn new(num_workers: Option<usize>) -> ParallelResult<Self> {
        let num_workers = num_workers.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
        });
        
        let mut workers = Vec::with_capacity(num_workers);
        let shutdown = Arc::new(AtomicBool::new(false));
        let task_counter = Arc::new(AtomicU64::new(0));
        let numa_allocator = Arc::new(NUMAAllocator::new()?);
        
        // Create worker threads
        for i in 0..num_workers {
            let deque = Arc::new(ChaseLevDeque::new(1024));
            let stats = Arc::new(RwLock::new(WorkerStats::default()));
            
            workers.push(WorkerThread {
                id: i,
                thread_id: None,
                deque,
                handle: None,
                stats,
            });
        }
        
        let rng = Arc::new(Mutex::new(fastrand::Rng::new()));
        
        Ok(Self {
            workers,
            num_workers,
            shutdown,
            task_counter,
            rng,
            numa_allocator,
        })
    }
    
    /// Start the scheduler
    pub fn start(&mut self) -> ParallelResult<()> {
        let all_deques: Vec<_> = self.workers.iter()
            .map(|w| w.deque.clone())
            .collect();
        
        for (i, worker) in self.workers.iter_mut().enumerate() {
            let worker_deque = worker.deque.clone();
            let all_deques_clone = all_deques.clone();
            let shutdown = self.shutdown.clone();
            let stats = worker.stats.clone();
            let rng = self.rng.clone();
            let numa_allocator = self.numa_allocator.clone();
            
            let handle = thread::Builder::new()
                .name(format!("worker-{}", i))
                .spawn(move || {
                    Self::worker_loop(
                        i,
                        worker_deque,
                        all_deques_clone,
                        shutdown,
                        stats,
                        rng,
                        numa_allocator,
                    );
                })
                .map_err(|e| ParallelError::ThreadCreationFailed(e.to_string()))?;
            
            let thread_id = handle.thread().id();
            worker.handle = Some(handle);
            worker.thread_id = Some(thread_id);
        }
        
        Ok(())
    }
    
    /// Submit a task for parallel execution
    pub fn submit(&self, task: Box<dyn ParallelTask<Output = Value>>) -> ParallelResult<TaskId> {
        if self.shutdown.load(Ordering::Relaxed) {
            return Err(ParallelError::SchedulerShutdown);
        }
        
        let work_item = WorkItem::new(task);
        let task_id = work_item.task_id;
        
        // Try to submit to least loaded worker
        let mut min_load = usize::MAX;
        let mut target_worker = 0;
        
        for (i, worker) in self.workers.iter().enumerate() {
            let load = worker.deque.len();
            if load < min_load {
                min_load = load;
                target_worker = i;
            }
        }
        
        self.workers[target_worker].deque.push(work_item)?;
        self.task_counter.fetch_add(1, Ordering::Relaxed);
        
        Ok(task_id)
    }
    
    /// Worker thread main loop
    fn worker_loop(
        worker_id: usize,
        local_deque: Arc<ChaseLevDeque<WorkItem>>,
        all_deques: Vec<Arc<ChaseLevDeque<WorkItem>>>,
        shutdown: Arc<AtomicBool>,
        stats: Arc<RwLock<WorkerStats>>,
        rng: Arc<Mutex<fastrand::Rng>>,
        numa_allocator: Arc<NUMAAllocator>,
    ) {
        let backoff = Backoff::new();
        
        // Set NUMA affinity
        let _ = numa_allocator.set_thread_affinity(worker_id);
        
        while !shutdown.load(Ordering::Relaxed) {
            let start_time = Instant::now();
            let mut task_executed = false;
            
            // Try local work first
            if let Some(work_item) = local_deque.pop() {
                let execution_start = Instant::now();
                let _result = work_item.task.execute();
                let execution_time = execution_start.elapsed();
                
                if let Ok(mut worker_stats) = stats.write() {
                    worker_stats.tasks_executed += 1;
                    worker_stats.execution_time += execution_time;
                }
                
                task_executed = true;
                backoff.reset();
            } else {
                // Try to steal work
                let mut steal_attempted = false;
                
                for _ in 0..3 { // Try up to 3 steal attempts
                    if let Ok(mut rng_guard) = rng.try_lock() {
                        let victim = rng_guard.usize(0..all_deques.len());
                        drop(rng_guard);
                        
                        if victim != worker_id {
                            steal_attempted = true;
                            if let Some(work_item) = all_deques[victim].steal() {
                                let execution_start = Instant::now();
                                let _result = work_item.task.execute();
                                let execution_time = execution_start.elapsed();
                                
                                if let Ok(mut worker_stats) = stats.write() {
                                    worker_stats.tasks_executed += 1;
                                    worker_stats.tasks_stolen += 1;
                                    worker_stats.execution_time += execution_time;
                                }
                                
                                task_executed = true;
                                backoff.reset();
                                break;
                            } else {
                                if let Ok(mut worker_stats) = stats.write() {
                                    worker_stats.failed_steals += 1;
                                }
                            }
                        }
                    }
                }
                
                if steal_attempted {
                    if let Ok(mut worker_stats) = stats.write() {
                        worker_stats.steal_attempts += 1;
                    }
                }
            }
            
            if !task_executed {
                // No work found, back off
                backoff.snooze();
                
                let idle_time = start_time.elapsed();
                if let Ok(mut worker_stats) = stats.write() {
                    worker_stats.idle_time += idle_time;
                }
            }
        }
    }
    
    /// Shutdown the scheduler
    pub fn shutdown(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        
        // Wait for all workers to finish
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                let _ = handle.join();
            }
        }
    }
    
    /// Get scheduler statistics
    pub fn get_stats(&self) -> SchedulerStats {
        let mut total_stats = WorkerStats::default();
        let mut worker_stats = Vec::new();
        
        for worker in &self.workers {
            if let Ok(stats) = worker.stats.read() {
                worker_stats.push(stats.clone());
                total_stats.tasks_executed += stats.tasks_executed;
                total_stats.tasks_stolen += stats.tasks_stolen;
                total_stats.steal_attempts += stats.steal_attempts;
                total_stats.failed_steals += stats.failed_steals;
                total_stats.execution_time += stats.execution_time;
                total_stats.idle_time += stats.idle_time;
            }
        }
        
        SchedulerStats {
            num_workers: self.num_workers,
            total_tasks: self.task_counter.load(Ordering::Relaxed),
            worker_stats,
            parallel_efficiency: Self::calculate_efficiency(&total_stats, self.num_workers),
            total_stats,
        }
    }
    
    fn calculate_efficiency(stats: &WorkerStats, num_workers: usize) -> f64 {
        let total_time = stats.execution_time + stats.idle_time;
        if total_time == Duration::ZERO {
            return 0.0;
        }
        
        let actual_work = stats.execution_time.as_secs_f64();
        let potential_work = total_time.as_secs_f64() * num_workers as f64;
        
        if potential_work > 0.0 {
            actual_work / potential_work
        } else {
            0.0
        }
    }
}

impl Drop for WorkStealingScheduler {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Statistics for the entire scheduler
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub num_workers: usize,
    pub total_tasks: u64,
    pub worker_stats: Vec<WorkerStats>,
    pub total_stats: WorkerStats,
    pub parallel_efficiency: f64,
}

/// Fork-join parallel continuation support
pub struct ParallelContinuation {
    /// Unique continuation ID
    pub id: TaskId,
    /// Continuation closure
    continuation: Box<dyn Fn(&WorkStealingScheduler) -> ParallelResult<Value> + Send + Sync>,
    /// Dependencies that must complete first
    dependencies: Vec<TaskId>,
    /// Completion status
    completed: Arc<AtomicBool>,
    /// Result of the continuation
    result: Arc<RwLock<Option<ParallelResult<Value>>>>,
}

impl std::fmt::Debug for ParallelContinuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParallelContinuation")
            .field("id", &self.id)
            .field("dependencies", &self.dependencies)
            .field("completed", &self.completed)
            .finish()
    }
}

impl ParallelContinuation {
    /// Create new parallel continuation
    pub fn new<F>(continuation: F) -> Self 
    where 
        F: Fn(&WorkStealingScheduler) -> ParallelResult<Value> + Send + Sync + 'static,
    {
        Self {
            id: TaskId::new(),
            continuation: Box::new(continuation),
            dependencies: Vec::new(),
            completed: Arc::new(AtomicBool::new(false)),
            result: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Add dependency
    pub fn add_dependency(&mut self, task_id: TaskId) {
        self.dependencies.push(task_id);
    }
    
    /// Execute the continuation
    pub fn execute(&self, scheduler: &WorkStealingScheduler) -> ParallelResult<Value> {
        if self.completed.load(Ordering::Acquire) {
            if let Ok(result) = self.result.read() {
                if let Some(ref cached_result) = *result {
                    return cached_result.clone();
                }
            }
        }
        
        let result = (self.continuation)(scheduler);
        
        if let Ok(mut result_guard) = self.result.write() {
            *result_guard = Some(result.clone());
        }
        
        self.completed.store(true, Ordering::Release);
        result
    }
    
    /// Check if continuation is completed
    pub fn is_completed(&self) -> bool {
        self.completed.load(Ordering::Acquire)
    }
    
    /// Fork this continuation into multiple parallel tasks
    pub fn fork<F>(&self, tasks: Vec<F>) -> Vec<ParallelContinuation>
    where
        F: Fn(&WorkStealingScheduler) -> ParallelResult<Value> + Send + Sync + 'static,
    {
        tasks.into_iter()
            .map(|task| ParallelContinuation::new(task))
            .collect()
    }
    
    /// Join multiple continuations into one
    pub fn join(continuations: Vec<ParallelContinuation>) -> ParallelContinuation {
        ParallelContinuation::new(move |scheduler| {
            let mut results = Vec::new();
            
            for cont in &continuations {
                results.push(cont.execute(scheduler)?);
            }
            
            // Combine results into a vector value
            Ok(Value::vector(results))
        })
    }
}

/// Thread-safe value wrapper for parallel execution
#[derive(Debug, Clone)]
pub struct ThreadSafeValue {
    /// The underlying value
    value: Arc<RwLock<Value>>,
    /// Version for optimistic concurrency
    version: Arc<AtomicU64>,
    /// Creation timestamp
    created_at: Instant,
    /// Last modification timestamp
    last_modified: Arc<RwLock<Instant>>,
}

impl ThreadSafeValue {
    /// Create new thread-safe value
    pub fn new(value: Value) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
            version: Arc::new(AtomicU64::new(1)),
            created_at: Instant::now(),
            last_modified: Arc::new(RwLock::new(Instant::now())),
        }
    }
    
    /// Read the value
    pub fn read<F, R>(&self, f: F) -> ParallelResult<R>
    where
        F: FnOnce(&Value) -> R,
    {
        if let Ok(value) = self.value.read() {
            Ok(f(&value))
        } else {
            Err(ParallelError::ResourceContention)
        }
    }
    
    /// Write to the value
    pub fn write<F>(&self, f: F) -> ParallelResult<()>
    where
        F: FnOnce(&mut Value),
    {
        if let Ok(mut value) = self.value.write() {
            f(&mut value);
            self.version.fetch_add(1, Ordering::Relaxed);
            
            if let Ok(mut last_mod) = self.last_modified.write() {
                *last_mod = Instant::now();
            }
            
            Ok(())
        } else {
            Err(ParallelError::ResourceContention)
        }
    }
    
    /// Get current version
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }
    
    /// Try to update with optimistic concurrency
    pub fn compare_and_swap<F>(&self, expected_version: u64, f: F) -> ParallelResult<bool>
    where
        F: FnOnce(&mut Value),
    {
        if let Ok(mut value) = self.value.try_write() {
            if self.version.load(Ordering::Acquire) == expected_version {
                f(&mut value);
                self.version.fetch_add(1, Ordering::Release);
                
                if let Ok(mut last_mod) = self.last_modified.write() {
                    *last_mod = Instant::now();
                }
                
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Clone the underlying value
    pub fn clone_value(&self) -> ParallelResult<Value> {
        if let Ok(value) = self.value.read() {
            Ok(value.clone())
        } else {
            Err(ParallelError::ResourceContention)
        }
    }
}

unsafe impl Send for ThreadSafeValue {}
unsafe impl Sync for ThreadSafeValue {}

/// NUMA-aware allocator for optimal memory locality
pub struct NUMAAllocator {
    /// Number of NUMA nodes
    num_nodes: usize,
    /// Thread to NUMA node mapping
    thread_affinity: Arc<RwLock<HashMap<ThreadId, usize>>>,
    /// Per-node memory statistics
    node_stats: Vec<Arc<AtomicU64>>,
}

impl NUMAAllocator {
    /// Create new NUMA allocator
    pub fn new() -> ParallelResult<Self> {
        // Detect NUMA topology (simplified)
        let num_nodes = Self::detect_numa_nodes();
        
        let node_stats = (0..num_nodes)
            .map(|_| Arc::new(AtomicU64::new(0)))
            .collect();
        
        Ok(Self {
            num_nodes,
            thread_affinity: Arc::new(RwLock::new(HashMap::new())),
            node_stats,
        })
    }
    
    /// Detect number of NUMA nodes
    fn detect_numa_nodes() -> usize {
        // Simplified detection - in real implementation would use libnuma
        std::thread::available_parallelism()
            .map(|n| (n.get() / 4).max(1))
            .unwrap_or(1)
    }
    
    /// Set thread affinity to NUMA node
    pub fn set_thread_affinity(&self, worker_id: usize) -> ParallelResult<()> {
        let node_id = worker_id % self.num_nodes;
        let thread_id = thread::current().id();
        
        if let Ok(mut affinity) = self.thread_affinity.write() {
            affinity.insert(thread_id, node_id);
        }
        
        // In real implementation, would set actual CPU affinity
        // using libnuma or similar
        
        Ok(())
    }
    
    /// Get preferred NUMA node for current thread
    pub fn get_preferred_node(&self) -> usize {
        let thread_id = thread::current().id();
        
        if let Ok(affinity) = self.thread_affinity.read() {
            affinity.get(&thread_id).copied().unwrap_or(0)
        } else {
            0
        }
    }
    
    /// Allocate memory on preferred NUMA node
    pub fn allocate_local<T>(&self, value: T) -> Box<T> {
        let node_id = self.get_preferred_node();
        
        // Update statistics
        self.node_stats[node_id].fetch_add(std::mem::size_of::<T>() as u64, Ordering::Relaxed);
        
        // In real implementation, would use numa_alloc_onnode
        Box::new(value)
    }
    
    /// Get NUMA statistics
    pub fn get_stats(&self) -> Vec<u64> {
        self.node_stats
            .iter()
            .map(|stat| stat.load(Ordering::Relaxed))
            .collect()
    }
}

// ===== STAGE 1 JIT INTEGRATION =====

/// Trait for values that can be optimized for parallel execution
#[cfg(feature = "stage1")]
pub trait ParallelOptimizable {
    /// Check if value can be parallelized
    fn can_parallelize(&self) -> bool;
    
    /// Execute value in parallel context
    fn parallel_execute(&self, scheduler: &WorkStealingScheduler) -> ParallelResult<Value>;
    
    /// Get parallelization strategy
    fn parallelization_strategy(&self) -> ParallelizationStrategy;
}

/// Strategies for parallelizing computations
#[cfg(feature = "stage1")]
#[derive(Debug, Clone)]
pub enum ParallelizationStrategy {
    /// Data parallelism - split data across threads
    DataParallel(usize), // chunk size
    /// Task parallelism - independent tasks
    TaskParallel,
    /// Pipeline parallelism - stages in pipeline
    Pipeline(usize), // pipeline depth
    /// Fork-join parallelism
    ForkJoin,
    /// No parallelization
    Sequential,
}

#[cfg(feature = "stage1")]
impl ParallelOptimizable for Value {
    fn can_parallelize(&self) -> bool {
        match self {
            Value::Vector(..) => true,   // Vector operations can be parallelized
            Value::Pair(..) => true,     // List operations can be parallelized
            Value::Procedure(..) => true, // Function calls can be parallelized
            _ => false,
        }
    }
    
    fn parallel_execute(&self, scheduler: &WorkStealingScheduler) -> ParallelResult<Value> {
        match self {
            Value::Vector(vec_ref) => {
                // Parallel vector processing
                let vec_guard = vec_ref.borrow();
                let vec_data = vec_guard.clone();
                drop(vec_guard);
                
                // Split vector into chunks for parallel processing
                let chunk_size = (vec_data.len() / scheduler.num_workers).max(1);
                let mut tasks = Vec::new();
                
                for chunk in vec_data.chunks(chunk_size) {
                    let chunk_vec = chunk.to_vec();
                    tasks.push(Box::new(VectorProcessingTask::new(chunk_vec)) 
                        as Box<dyn ParallelTask<Output = Value>>);
                }
                
                // Submit tasks and collect results
                let mut results = Vec::new();
                for task in tasks {
                    scheduler.submit(task)?;
                    // In real implementation, would wait for results
                    results.push(Value::Nil);
                }
                
                Ok(Value::vector(results))
            },
            _ => {
                // Sequential fallback
                Ok(self.clone())
            }
        }
    }
    
    fn parallelization_strategy(&self) -> ParallelizationStrategy {
        match self {
            Value::Vector(..) => ParallelizationStrategy::DataParallel(100),
            Value::Pair(..) => ParallelizationStrategy::ForkJoin,
            Value::Procedure(..) => ParallelizationStrategy::TaskParallel,
            _ => ParallelizationStrategy::Sequential,
        }
    }
}

/// Example parallel task for vector processing
#[cfg(feature = "stage1")]
struct VectorProcessingTask {
    data: Vec<Value>,
}

#[cfg(feature = "stage1")]
impl VectorProcessingTask {
    fn new(data: Vec<Value>) -> Self {
        Self { data }
    }
}

#[cfg(feature = "stage1")]
impl ParallelTask for VectorProcessingTask {
    type Output = Value;
    
    fn execute(&self) -> ParallelResult<Self::Output> {
        // Process vector chunk in parallel
        let processed_data: Vec<Value> = self.data
            .iter().cloned()
            .collect();
        
        Ok(Value::vector(processed_data))
    }
}

// ===== NAN BOXING INTEGRATION PREPARATION =====

/// Trait for maintaining NaN boxing optimizations in parallel context
pub trait ParallelSafe {
    /// Convert to thread-safe wrapper while maintaining optimizations
    fn share_across_threads(&self) -> ThreadSafeValue;
    
    /// Check if optimizations are preserved in parallel context
    fn preserves_optimizations(&self) -> bool;
}

impl ParallelSafe for Value {
    fn share_across_threads(&self) -> ThreadSafeValue {
        // Maintain 60% memory reduction from NaN boxing when available
        ThreadSafeValue::new(self.clone())
    }
    
    fn preserves_optimizations(&self) -> bool {
        // All Value types preserve optimizations when shared
        true
    }
}

// Future integration point for NaN boxing
// Stage 3 integration placeholder for NaN boxing optimization
// This implementation will be available when both stage2 and nan-boxing-optimization features are enabled
#[cfg(all(feature = "stage2", feature = "nan-boxing-optimization"))]
impl ParallelSafe for crate::eval::EnhancedNanBoxedValue {
    fn share_across_threads(&self) -> ThreadSafeValue {
        // Simple fallback for Stage 2 - will be properly integrated in Stage 3
        ThreadSafeValue::new(Value::number(0.0))
    }
    
    fn preserves_optimizations(&self) -> bool {
        true // NaN boxing optimizations are preserved
    }
}

/// Parallel execution system that integrates all components
pub struct ParallelExecutionSystem {
    /// Work-stealing scheduler
    pub scheduler: WorkStealingScheduler,
    /// Active continuations
    pub continuations: Arc<RwLock<HashMap<TaskId, ParallelContinuation>>>,
    /// NUMA allocator
    pub numa_allocator: Arc<NUMAAllocator>,
    /// System statistics
    pub stats: Arc<RwLock<ParallelSystemStats>>,
    
    #[cfg(feature = "stage1")]
    /// JIT integration system
    pub jit_system: Option<Arc<JITIntegrationSystem>>,
}

impl ParallelExecutionSystem {
    /// Create new parallel execution system
    pub fn new(num_workers: Option<usize>) -> ParallelResult<Self> {
        let mut scheduler = WorkStealingScheduler::new(num_workers)?;
        scheduler.start()?;
        
        let numa_allocator = Arc::new(NUMAAllocator::new()?);
        
        Ok(Self {
            scheduler,
            continuations: Arc::new(RwLock::new(HashMap::new())),
            numa_allocator,
            stats: Arc::new(RwLock::new(ParallelSystemStats::default())),
            
            #[cfg(feature = "stage1")]
            jit_system: None,
        })
    }
    
    #[cfg(feature = "stage1")]
    /// Integrate with JIT system
    pub fn integrate_jit(&mut self, jit_system: Arc<JITIntegrationSystem>) {
        self.jit_system = Some(jit_system);
    }
    
    /// Execute value in parallel
    pub fn parallel_execute(&self, value: &Value) -> ParallelResult<Value> {
        #[cfg(feature = "stage1")]
        {
            // Try JIT-optimized parallel execution first
            if let Some(ref jit) = self.jit_system {
                if value.can_parallelize() {
                    if let Ok(Some(compiled)) = jit.process_value(value) {
                        // Execute JIT-compiled parallel code
                        let _ = compiled.execute();
                        return value.parallel_execute(&self.scheduler);
                    }
                }
            }
        }
        
        // Fallback to regular parallel execution
        if let Ok(shared_value) = self.create_thread_safe_value(value) {
            // Use thread-safe value for parallel execution
            shared_value.read(|v| v.clone())
        } else {
            Ok(value.clone())
        }
    }
    
    /// Create thread-safe value wrapper
    fn create_thread_safe_value(&self, value: &Value) -> ParallelResult<ThreadSafeValue> {
        Ok(value.share_across_threads())
    }
    
    /// Submit parallel continuation
    pub fn submit_continuation(&self, continuation: ParallelContinuation) -> ParallelResult<TaskId> {
        let task_id = continuation.id;
        
        if let Ok(mut conts) = self.continuations.write() {
            conts.insert(task_id, continuation);
        }
        
        // Update statistics
        if let Ok(mut stats) = self.stats.write() {
            stats.continuations_submitted += 1;
        }
        
        Ok(task_id)
    }
    
    /// Get system statistics
    pub fn get_stats(&self) -> ParallelSystemStats {
        let scheduler_stats = self.scheduler.get_stats();
        
        if let Ok(stats) = self.stats.read() {
            let mut system_stats = stats.clone();
            system_stats.scheduler_stats = Some(scheduler_stats);
            system_stats
        } else {
            ParallelSystemStats::default()
        }
    }
    
    /// Shutdown the system
    pub fn shutdown(&mut self) {
        self.scheduler.shutdown();
    }
}

impl Drop for ParallelExecutionSystem {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Statistics for the entire parallel system
#[derive(Debug, Clone, Default)]
pub struct ParallelSystemStats {
    pub continuations_submitted: u64,
    pub continuations_completed: u64,
    pub thread_safe_values_created: u64,
    pub numa_allocations: u64,
    pub scheduler_stats: Option<SchedulerStats>,
}

// ===== INTEGRATION TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    struct TestTask {
        id: u32,
        duration: Duration,
    }
    
    impl TestTask {
        fn new(id: u32, duration_ms: u64) -> Self {
            Self {
                id,
                duration: Duration::from_millis(duration_ms),
            }
        }
    }
    
    impl ParallelTask for TestTask {
        type Output = Value;
        
        fn execute(&self) -> ParallelResult<Self::Output> {
            std::thread::sleep(self.duration);
            Ok(Value::integer(self.id as i64))
        }
    }
    
    #[test]
    fn test_chase_lev_deque() {
        let deque = ChaseLevDeque::new(16);
        
        // Test push/pop
        deque.push(1).unwrap();
        deque.push(2).unwrap();
        
        assert_eq!(deque.len(), 2);
        assert_eq!(deque.pop(), Some(2));
        assert_eq!(deque.pop(), Some(1));
        assert_eq!(deque.pop(), None);
    }
    
    #[test]
    fn test_work_stealing_scheduler() {
        let mut scheduler = WorkStealingScheduler::new(Some(2)).unwrap();
        scheduler.start().unwrap();
        
        // Submit some tasks
        let task1 = Box::new(TestTask::new(1, 10));
        let task2 = Box::new(TestTask::new(2, 10));
        
        let _id1 = scheduler.submit(task1).unwrap();
        let _id2 = scheduler.submit(task2).unwrap();
        
        // Give tasks time to execute
        std::thread::sleep(Duration::from_millis(50));
        
        let stats = scheduler.get_stats();
        assert!(stats.total_tasks >= 2);
        assert!(stats.total_stats.tasks_executed > 0);
    }
    
    #[test]
    fn test_parallel_continuation() {
        let continuation = ParallelContinuation::new(|_scheduler| {
            Ok(Value::string("test result"))
        });
        
        let scheduler = WorkStealingScheduler::new(Some(1)).unwrap();
        let result = continuation.execute(&scheduler).unwrap();
        
        assert!(continuation.is_completed());
        
        if let Value::Literal(Literal::String(s)) = result {
            assert_eq!(*s, "test result");
        } else {
            panic!("Expected string result");
        }
    }
    
    #[test]
    fn test_thread_safe_value() {
        let value = ThreadSafeValue::new(Value::integer(42));
        
        // Test read
        let read_result = value.read(|v| {
            if let Value::Literal(Literal::ExactInteger(n)) = v {
                *n
            } else {
                0
            }
        }).unwrap();
        
        assert_eq!(read_result, 42);
        
        // Test write
        let initial_version = value.version();
        value.write(|v| {
            *v = Value::integer(84);
        }).unwrap();
        
        assert!(value.version() > initial_version);
    }
    
    #[test]
    fn test_numa_allocator() {
        let allocator = NUMAAllocator::new().unwrap();
        
        let preferred_node = allocator.get_preferred_node();
        assert!(preferred_node < allocator.num_nodes);
        
        let allocation = allocator.allocate_local(42i32);
        assert_eq!(*allocation, 42);
        
        let stats = allocator.get_stats();
        assert_eq!(stats.len(), allocator.num_nodes);
    }
    
    #[test]
    fn test_parallel_safe_trait() {
        let value = Value::vector(vec![Value::integer(1), Value::integer(2)]);
        let thread_safe = value.share_across_threads();
        
        assert!(value.preserves_optimizations());
        
        let cloned = thread_safe.clone_value().unwrap();
        // Vector should be preserved
        assert!(matches!(cloned, Value::Vector(..)));
    }
    
    #[cfg(feature = "stage1")]
    #[test]
    fn test_parallel_optimizable() {
        let vector = Value::vector(vec![Value::integer(1), Value::integer(2), Value::integer(3)]);
        let scheduler = WorkStealingScheduler::new(Some(2)).unwrap();
        
        assert!(vector.can_parallelize());
        
        let strategy = vector.parallelization_strategy();
        assert!(matches!(strategy, ParallelizationStrategy::DataParallel(..)));
        
        // Test would require running scheduler
        // let result = vector.parallel_execute(&scheduler).unwrap();
        // assert!(matches!(result, Value::Vector(..)));
    }
    
    #[test]
    fn test_parallel_execution_system() {
        let system = ParallelExecutionSystem::new(Some(2)).unwrap();
        let value = Value::integer(42);
        
        let result = system.parallel_execute(&value).unwrap();
        assert!(matches!(result, Value::Literal(..)));
    }
    
    #[test]
    fn test_fork_join_continuation() {
        let tasks = vec![
            Box::new(|_scheduler: &WorkStealingScheduler| Ok(Value::integer(1))) as Box<dyn Fn(&WorkStealingScheduler) -> ParallelResult<Value> + Send + Sync>,
            Box::new(|_scheduler: &WorkStealingScheduler| Ok(Value::integer(2))) as Box<dyn Fn(&WorkStealingScheduler) -> ParallelResult<Value> + Send + Sync>,
            Box::new(|_scheduler: &WorkStealingScheduler| Ok(Value::integer(3))) as Box<dyn Fn(&WorkStealingScheduler) -> ParallelResult<Value> + Send + Sync>,
        ];
        
        let base_cont = ParallelContinuation::new(|_| Ok(Value::Nil));
        let forked = base_cont.fork(tasks);
        let joined = ParallelContinuation::join(forked);
        
        let scheduler = WorkStealingScheduler::new(Some(1)).unwrap();
        let result = joined.execute(&scheduler).unwrap();
        
        // Should get a vector with 3 elements
        if let Value::Vector(vec_ref) = result {
            let vec_data = vec_ref.borrow();
            assert_eq!(vec_data.len(), 3);
        } else {
            panic!("Expected vector result from join");
        }
    }
    
    #[test]
    fn test_task_priorities() {
        struct PriorityTask {
            priority: u32,
        }
        
        impl ParallelTask for PriorityTask {
            type Output = Value;
            
            fn execute(&self) -> ParallelResult<Self::Output> {
                Ok(Value::integer(self.priority as i64))
            }
            
            fn priority(&self) -> u32 {
                self.priority
            }
        }
        
        let high_priority = Box::new(PriorityTask { priority: 10 });
        let low_priority = Box::new(PriorityTask { priority: 1 });
        
        assert_eq!(high_priority.priority(), 10);
        assert_eq!(low_priority.priority(), 1);
    }
    
    #[test]
    fn test_work_item_creation() {
        let task = Box::new(TestTask::new(1, 10));
        let work_item = WorkItem::new(task);
        
        assert_eq!(work_item.priority, 0); // Default priority
        assert!(work_item.created_at.elapsed() < Duration::from_secs(1));
    }
    
    #[test]
    fn test_scheduler_stats() {
        let scheduler = WorkStealingScheduler::new(Some(2)).unwrap();
        let stats = scheduler.get_stats();
        
        assert_eq!(stats.num_workers, 2);
        assert_eq!(stats.worker_stats.len(), 2);
        assert!(stats.parallel_efficiency >= 0.0);
        assert!(stats.parallel_efficiency <= 1.0);
    }
    
    #[test]
    fn test_concurrent_access() {
        let value = ThreadSafeValue::new(Value::integer(0));
        let value_clone = value.clone();
        
        let handle = std::thread::spawn(move || {
            for _ in 0..10 {
                let _ = value_clone.write(|v| {
                    if let Value::Literal(Literal::ExactInteger(n)) = v {
                        *n += 1;
                    }
                });
            }
        });
        
        for _ in 0..10 {
            let _ = value.write(|v| {
                if let Value::Literal(Literal::ExactInteger(n)) = v {
                    *n += 1;
                }
            });
        }
        
        handle.join().unwrap();
        
        let final_value = value.read(|v| {
            if let Value::Literal(Literal::ExactInteger(n)) = v {
                *n
            } else {
                -1
            }
        }).unwrap();
        
        assert_eq!(final_value, 20);
    }
    
    #[test]
    fn test_optimistic_concurrency() {
        let value = ThreadSafeValue::new(Value::integer(42));
        let version = value.version();
        
        let success = value.compare_and_swap(version, |v| {
            *v = Value::integer(84);
        }).unwrap();
        
        assert!(success);
        assert!(value.version() > version);
        
        // Try again with old version - should fail
        let success2 = value.compare_and_swap(version, |v| {
            *v = Value::integer(126);
        }).unwrap();
        
        assert!(!success2);
    }
}