//! Advanced work-stealing scheduler for high-performance task execution.
//!
//! This module provides a sophisticated scheduler that can handle
//! different types of tasks with varying priorities and execution requirements.

use crate::eval::Value;
use crate::diagnostics::{Error, Result};
use super::ConcurrencyError;
use std::sync::{Arc, Mutex};
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicU64, Ordering as AtomicOrdering};
use crossbeam::deque::{Injector, Stealer, Worker};
use crossbeam::queue::SegQueue;
use num_cpus;

/// Task priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Low priority tasks
    Low = 0,
    /// Normal priority tasks
    Normal = 1,
    /// High priority tasks
    High = 2,
    /// Critical priority tasks
    Critical = 3,
}

/// Task execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// CPU-intensive task
    Compute,
    /// I/O-bound task
    Io,
    /// Real-time task with strict timing requirements
    RealTime,
    /// Background task (low priority)
    Background,
}

/// A scheduled task with metadata.
pub struct Task {
    id: TaskId,
    priority: Priority,
    mode: ExecutionMode,
    work: Box<dyn FnOnce() -> Result<Value> + Send + 'static>,
    created_at: Instant,
    deadline: Option<Instant>,
    estimated_duration: Option<Duration>,
}

impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("priority", &self.priority)
            .field("mode", &self.mode)
            .field("work", &"<closure>")
            .field("created_at", &self.created_at)
            .field("deadline", &self.deadline)
            .field("estimated_duration", &self.estimated_duration)
            .finish()
    }
}

impl Task {
    /// Creates a new task.
    pub fn new<F>(work: F) -> Self
    where
        F: FnOnce() -> Result<Value> + Send + 'static,
    {
        Self {
            id: TaskId::new(),
            priority: Priority::Normal,
            mode: ExecutionMode::Compute,
            work: Box::new(work),
            created_at: Instant::now(),
            deadline: None,
            estimated_duration: None,
        }
    }

    /// Sets the task priority.
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the execution mode.
    pub fn with_mode(mut self, mode: ExecutionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets a deadline for the task.
    pub fn with_deadline(mut self, deadline: Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Sets an estimated duration for the task.
    pub fn with_estimated_duration(mut self, duration: Duration) -> Self {
        self.estimated_duration = Some(duration);
        self
    }

    /// Gets the task ID.
    pub fn id(&self) -> TaskId {
        self.id
    }

    /// Gets the task priority.
    pub fn priority(&self) -> Priority {
        self.priority
    }

    /// Gets the execution mode.
    pub fn mode(&self) -> ExecutionMode {
        self.mode
    }

    /// Checks if the task has expired its deadline.
    pub fn is_expired(&self) -> bool {
        self.deadline
            .map(|deadline| Instant::now() > deadline)
            .unwrap_or(false)
    }

    /// Executes the task.
    pub fn execute(self) -> Result<Value> {
        (self.work)()
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority tasks come first
        self.priority.cmp(&other.priority).reverse()
            .then_with(|| {
                // Earlier deadlines come first
                match (self.deadline, other.deadline) {
                    (Some(a), Some(b)) => a.cmp(&b),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => self.created_at.cmp(&other.created_at),
                }
            })
    }
}

/// Unique task identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskId {
    /// Creates a new unique task ID.
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, AtomicOrdering::SeqCst))
    }

    /// Gets the numeric ID.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "task-{}", self.0)
    }
}

/// Work-stealing scheduler configuration.
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Number of worker threads
    pub num_workers: usize,
    /// Number of I/O threads
    pub num_io_threads: usize,
    /// Enable work stealing
    pub work_stealing: bool,
    /// Task queue capacity per worker
    pub queue_capacity: usize,
    /// Global queue capacity
    pub global_queue_capacity: usize,
    /// Task timeout (None for no timeout)
    pub task_timeout: Option<Duration>,
    /// Enable task profiling
    pub profiling_enabled: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            num_workers: num_cpus::get(),
            num_io_threads: num_cpus::get().min(4),
            work_stealing: true,
            queue_capacity: 1024,
            global_queue_capacity: 10000,
            task_timeout: Some(Duration::from_secs(30)),
            profiling_enabled: false,
        }
    }
}

/// Advanced work-stealing scheduler.
pub struct WorkStealingScheduler {
    config: SchedulerConfig,
    
    // Worker threads for CPU-bound tasks
    workers: Vec<WorkerThread>,
    worker_handles: Vec<thread::JoinHandle<()>>,
    
    // I/O thread pool
    io_pool: tokio::runtime::Runtime,
    
    // Global task queues
    global_queue: Arc<Injector<Task>>,
    priority_queue: Arc<Mutex<BinaryHeap<Task>>>,
    io_queue: Arc<SegQueue<Task>>,
    
    // Scheduling state
    running: Arc<AtomicBool>,
    active_tasks: Arc<AtomicUsize>,
    completed_tasks: Arc<AtomicUsize>,
    failed_tasks: Arc<AtomicUsize>,
    
    // Task results
    results: Arc<Mutex<std::collections::HashMap<TaskId, Result<Value>>>>,
    
    // Profiling data
    profiler: Option<TaskProfiler>,
}

/// Individual worker thread.
struct WorkerThread {
    id: usize,
    local_queue: Worker<Task>,
    stealer: Stealer<Task>,
}

/// Task execution profiler.
#[derive(Debug)]
struct TaskProfiler {
    task_metrics: Mutex<std::collections::HashMap<TaskId, TaskMetrics>>,
}

#[derive(Debug, Clone)]
struct TaskMetrics {
    created_at: Instant,
    started_at: Option<Instant>,
    completed_at: Option<Instant>,
    worker_id: Option<usize>,
    execution_time: Option<Duration>,
    queue_time: Option<Duration>,
}

/// Worker thread context parameters.
#[allow(dead_code)]
struct WorkerContext {
    global_queue: Arc<Injector<Task>>,
    priority_queue: Arc<Mutex<BinaryHeap<Task>>>,
    stealers: Vec<Stealer<Task>>,
    running: Arc<AtomicBool>,
    active_tasks: Arc<AtomicUsize>,
    completed_tasks: Arc<AtomicUsize>,
    failed_tasks: Arc<AtomicUsize>,
    results: Arc<Mutex<std::collections::HashMap<TaskId, Result<Value>>>>,
    profiler: Option<Arc<Mutex<std::collections::HashMap<TaskId, TaskMetrics>>>>,
}

impl WorkStealingScheduler {
    /// Creates a new work-stealing scheduler.
    pub fn new(config: SchedulerConfig) -> Result<Self> {
        let mut workers = Vec::new();
        let mut stealers = Vec::new();

        // Create worker threads
        for i in 0..config.num_workers {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            workers.push(WorkerThread {
                id: i,
                stealer: worker.stealer(),
                local_queue: worker,
            });
        }

        // Create I/O runtime
        let io_pool = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(config.num_io_threads)
            .enable_all()
            .build()
            .map_err(|e| Error::runtime_error(format!("Failed to create I/O runtime: {e}"), None))?;

        let profiler = if config.profiling_enabled {
            Some(TaskProfiler {
                task_metrics: Mutex::new(std::collections::HashMap::new()),
            })
        } else {
            None
        };

        Ok(Self {
            config,
            workers,
            worker_handles: Vec::new(),
            io_pool,
            global_queue: Arc::new(Injector::new()),
            priority_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            io_queue: Arc::new(SegQueue::new()),
            running: Arc::new(AtomicBool::new(false)),
            active_tasks: Arc::new(AtomicUsize::new(0)),
            completed_tasks: Arc::new(AtomicUsize::new(0)),
            failed_tasks: Arc::new(AtomicUsize::new(0)),
            results: Arc::new(Mutex::new(std::collections::HashMap::new())),
            profiler,
        })
    }

    /// Starts the scheduler.
    pub fn start(&mut self) -> Result<()> {
        if self.running.load(AtomicOrdering::SeqCst) {
            return Err(Box::new(Error::runtime_error("Scheduler already running".to_string(), None)));
        }

        self.running.store(true, AtomicOrdering::SeqCst);

        // Start worker threads
        let stealers: Vec<_> = self.workers.iter().map(|w| w.stealer.clone()).collect();
        
        for (i, worker) in self.workers.drain(..).enumerate() {
            let global_queue = self.global_queue.clone();
            let priority_queue = self.priority_queue.clone();
            let stealers = stealers.clone();
            let running = self.running.clone();
            let active_tasks = self.active_tasks.clone();
            let completed_tasks = self.completed_tasks.clone();
            let failed_tasks = self.failed_tasks.clone();
            let results = self.results.clone();
            let profiler = self.profiler.as_ref().map(|p| Arc::new(Mutex::new(p.task_metrics.lock().unwrap().clone())));

            let handle = thread::Builder::new()
                .name(format!("worker-{i}"))
                .spawn(move || {
                    Self::worker_loop(
                        worker,
                        WorkerContext {
                            global_queue,
                            priority_queue,
                            stealers,
                            running,
                            active_tasks,
                            completed_tasks,
                            failed_tasks,
                            results,
                            profiler,
                        }
                    );
                })
                .map_err(|e| Error::runtime_error(format!("Failed to start worker thread: {e}"), None))?;
            
            self.worker_handles.push(handle);
        }

        Ok(())
    }

    /// Stops the scheduler.
    pub fn stop(&mut self) -> Result<()> {
        self.running.store(false, AtomicOrdering::SeqCst);

        // Wait for worker threads to finish
        for handle in self.worker_handles.drain(..) {
            handle.join()
                .map_err(|_| Error::runtime_error("Failed to join worker thread".to_string(), None))?;
        }

        // Shutdown I/O runtime
        let io_pool = std::mem::replace(&mut self.io_pool, tokio::runtime::Runtime::new().unwrap());
        io_pool.shutdown_background();

        Ok(())
    }

    /// Submits a task for execution.
    pub fn submit(&self, task: Task) -> TaskId {
        let task_id = task.id();
        
        // Record task creation if profiling is enabled
        if let Some(ref profiler) = self.profiler {
            let metrics = TaskMetrics {
                created_at: Instant::now(),
                started_at: None,
                completed_at: None,
                worker_id: None,
                execution_time: None,
                queue_time: None,
            };
            profiler.task_metrics.lock().unwrap().insert(task_id, metrics);
        }

        match task.mode() {
            ExecutionMode::Io => {
                self.io_queue.push(task);
            }
            ExecutionMode::RealTime | ExecutionMode::Compute if task.priority() >= Priority::High => {
                self.priority_queue.lock().unwrap().push(task);
            }
            _ => {
                self.global_queue.push(task);
            }
        }

        self.active_tasks.fetch_add(1, AtomicOrdering::SeqCst);
        task_id
    }

    /// Gets the result of a completed task.
    pub fn get_result(&self, task_id: TaskId) -> Option<Result<Value>> {
        self.results.lock().unwrap().remove(&task_id)
    }

    /// Waits for a task to complete and returns its result.
    pub async fn wait_for_task(&self, task_id: TaskId, timeout: Option<Duration>) -> Result<Value> {
        let start = Instant::now();
        
        loop {
            if let Some(result) = self.get_result(task_id) {
                return result;
            }

            if let Some(timeout) = timeout {
                if start.elapsed() > timeout {
                    return Err(ConcurrencyError::Timeout.into())
                }
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Gets scheduler statistics.
    pub fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            active_tasks: self.active_tasks.load(AtomicOrdering::SeqCst),
            completed_tasks: self.completed_tasks.load(AtomicOrdering::SeqCst),
            failed_tasks: self.failed_tasks.load(AtomicOrdering::SeqCst),
            global_queue_len: self.global_queue.len(),
            priority_queue_len: self.priority_queue.lock().unwrap().len(),
            io_queue_len: self.io_queue.len(),
        }
    }

    /// Worker thread main loop.
    fn worker_loop(worker: WorkerThread, ctx: WorkerContext) {
        while ctx.running.load(AtomicOrdering::SeqCst) {
            let task = Self::find_task(&worker, &ctx.global_queue, &ctx.priority_queue, &ctx.stealers);
            
            if let Some(task) = task {
                let task_id = task.id();
                let start_time = Instant::now();
                
                // Update profiling metrics
                if let Some(ref profiler) = ctx.profiler {
                    if let Ok(mut metrics) = profiler.lock() {
                        if let Some(task_metrics) = metrics.get_mut(&task_id) {
                            task_metrics.started_at = Some(start_time);
                            task_metrics.worker_id = Some(worker.id);
                            task_metrics.queue_time = Some(start_time - task_metrics.created_at);
                        }
                    }
                }

                // Execute the task
                let result = task.execute();
                let execution_time = start_time.elapsed();
                
                // Store result
                ctx.results.lock().unwrap().insert(task_id, result.clone());
                
                // Update counters
                ctx.active_tasks.fetch_sub(1, AtomicOrdering::SeqCst);
                if result.is_ok() {
                    ctx.completed_tasks.fetch_add(1, AtomicOrdering::SeqCst);
                } else {
                    ctx.failed_tasks.fetch_add(1, AtomicOrdering::SeqCst);
                }
                
                // Update profiling metrics
                if let Some(ref profiler) = ctx.profiler {
                    if let Ok(mut metrics) = profiler.lock() {
                        if let Some(task_metrics) = metrics.get_mut(&task_id) {
                            task_metrics.completed_at = Some(Instant::now());
                            task_metrics.execution_time = Some(execution_time);
                        }
                    }
                }
            } else {
                // No work available, yield
                thread::yield_now();
            }
        }
    }

    /// Finds the next task to execute using work-stealing.
    fn find_task(
        worker: &WorkerThread,
        global_queue: &Injector<Task>,
        priority_queue: &Arc<Mutex<BinaryHeap<Task>>>,
        stealers: &[Stealer<Task>],
    ) -> Option<Task> {
        // Try local queue first
        if let Some(task) = worker.local_queue.pop() {
            return Some(task);
        }

        // Try priority queue
        if let Ok(mut pq) = priority_queue.lock() {
            if let Some(task) = pq.pop() {
                return Some(task);
            }
        }

        // Try global queue
        if let crossbeam::deque::Steal::Success(task) = global_queue.steal() {
            return Some(task);
        }

        // Try stealing from other workers
        for stealer in stealers {
            if let crossbeam::deque::Steal::Success(task) = stealer.steal() {
                return Some(task);
            }
        }

        None
    }
}

/// Scheduler statistics.
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    /// Number of currently executing tasks
    pub active_tasks: usize,
    /// Total number of successfully completed tasks
    pub completed_tasks: usize,
    /// Total number of tasks that failed during execution
    pub failed_tasks: usize,
    /// Current length of the global task queue
    pub global_queue_len: usize,
    /// Current length of the priority task queue
    pub priority_queue_len: usize,
    /// Current length of the IO-bound task queue
    pub io_queue_len: usize,
}

/// Global scheduler instance.
static GLOBAL_SCHEDULER: std::sync::OnceLock<Arc<Mutex<Option<WorkStealingScheduler>>>> = std::sync::OnceLock::new();

/// Gets the global scheduler.
pub fn global_scheduler() -> Arc<Mutex<Option<WorkStealingScheduler>>> {
    GLOBAL_SCHEDULER.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

/// Initializes the global scheduler.
pub fn initialize() -> Result<()> {
    let scheduler_guard = global_scheduler();
    let mut scheduler_opt = scheduler_guard.lock().unwrap();
    
    if scheduler_opt.is_none() {
        let mut scheduler = WorkStealingScheduler::new(SchedulerConfig::default())?;
        scheduler.start()?;
        *scheduler_opt = Some(scheduler);
    }
    
    Ok(())
}

/// Shuts down the global scheduler.
pub async fn shutdown() -> Result<()> {
    let scheduler_guard = global_scheduler();
    let mut scheduler_opt = scheduler_guard.lock().unwrap();
    
    if let Some(mut scheduler) = scheduler_opt.take() {
        scheduler.stop()?;
    }
    
    Ok(())
}

/// Submits a task to the global scheduler.
pub fn submit_task<F>(work: F) -> Result<TaskId>
where
    F: FnOnce() -> Result<Value> + Send + 'static,
{
    let scheduler_guard = global_scheduler();
    let scheduler_opt = scheduler_guard.lock().unwrap();
    
    if let Some(ref scheduler) = *scheduler_opt {
        let task = Task::new(work);
        Ok(scheduler.submit(task))
    } else {
        Err(Box::new(Error::runtime_error("Scheduler not initialized".to_string(), None)))
    }
}

/// Submits a high-priority task to the global scheduler.
pub fn submit_priority_task<F>(work: F, priority: Priority) -> Result<TaskId>
where
    F: FnOnce() -> Result<Value> + Send + 'static,
{
    let scheduler_guard = global_scheduler();
    let scheduler_opt = scheduler_guard.lock().unwrap();
    
    if let Some(ref scheduler) = *scheduler_opt {
        let task = Task::new(work).with_priority(priority);
        Ok(scheduler.submit(task))
    } else {
        Err(Box::new(Error::runtime_error("Scheduler not initialized".to_string(), None)))
    }
}