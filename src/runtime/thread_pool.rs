//! Thread pool implementation for multithreaded Scheme evaluation.
//!
//! This module provides a work-stealing thread pool specifically designed
//! for Scheme evaluation with proper isolation and communication between
//! evaluator threads.

use super::{EvaluatorMessage, EvaluatorHandle, GlobalEnvironmentManager, EffectCoordinator};
use super::evaluator::EvaluatorWorker;
use crate::diagnostics::Result;
use crossbeam::channel::{self, Sender, Receiver};
use std::sync::{Arc, RwLock};
use std::thread::{self, ThreadId, JoinHandle};
use std::time::{Duration, Instant};

/// Thread pool for managing multiple Scheme evaluator threads.
///
/// This thread pool provides work-stealing capabilities and manages
/// the lifecycle of evaluator threads. Each thread maintains its own
/// evaluator instance with thread-local state.
#[derive(Debug)]
pub struct ThreadPool {
    /// Number of worker threads
    size: usize,
    /// Global work queue for distributing evaluation tasks
    work_queue: Arc<crossbeam::queue::SegQueue<EvaluatorMessage>>,
    /// Worker thread handles
    workers: Vec<WorkerThread>,
    /// Global environment manager
    #[allow(dead_code)] // Part of Stage 3 advanced threading infrastructure
    global_env: Arc<GlobalEnvironmentManager>,
    /// Effect coordinator
    #[allow(dead_code)] // Part of Stage 3 advanced threading infrastructure
    effect_coordinator: Arc<EffectCoordinator>,
    /// Thread pool statistics
    stats: Arc<RwLock<ThreadPoolStats>>,
    /// Shutdown signal
    shutdown_signal: Arc<std::sync::atomic::AtomicBool>,
}

/// A worker thread in the thread pool.
#[derive(Debug)]
pub struct WorkerThread {
    /// Unique worker ID
    pub id: u64,
    /// Thread handle
    pub handle: Option<JoinHandle<Result<()>>>,
    /// Thread ID
    pub thread_id: Option<ThreadId>,
    /// Message sender for this worker
    pub sender: Sender<EvaluatorMessage>,
    /// Worker statistics
    #[allow(dead_code)] // Part of Stage 3 monitoring infrastructure
    pub stats: WorkerStats,
}

/// Statistics for the entire thread pool.
#[derive(Debug, Clone)]
pub struct ThreadPoolStats {
    /// Total tasks submitted
    pub total_tasks_submitted: u64,
    /// Total tasks completed
    pub total_tasks_completed: u64,
    /// Total tasks failed
    pub total_tasks_failed: u64,
    /// Average task completion time
    pub average_task_time: Duration,
    /// Number of active workers
    pub active_workers: usize,
    /// Pool creation time
    pub created_at: Instant,
}

/// Statistics for an individual worker thread.
#[derive(Debug, Clone)]
pub struct WorkerStats {
    /// Tasks processed by this worker
    #[allow(dead_code)] // Part of Stage 3 monitoring infrastructure
    pub tasks_processed: u64,
    /// Tasks failed by this worker
    #[allow(dead_code)] // Part of Stage 3 monitoring infrastructure
    pub tasks_failed: u64,
    /// Total processing time
    #[allow(dead_code)] // Part of Stage 3 monitoring infrastructure
    pub total_processing_time: Duration,
    /// Worker creation time
    #[allow(dead_code)] // Part of Stage 3 monitoring infrastructure
    pub created_at: Instant,
    /// Last task completion time
    #[allow(dead_code)] // Part of Stage 3 monitoring infrastructure
    pub last_task_completed: Option<Instant>,
}

impl Default for ThreadPoolStats {
    fn default() -> Self {
        Self {
            total_tasks_submitted: 0,
            total_tasks_completed: 0,
            total_tasks_failed: 0,
            average_task_time: Duration::new(0, 0),
            active_workers: 0,
            created_at: Instant::now(),
        }
    }
}

impl Default for WorkerStats {
    fn default() -> Self {
        Self {
            tasks_processed: 0,
            tasks_failed: 0,
            total_processing_time: Duration::new(0, 0),
            created_at: Instant::now(),
            last_task_completed: None,
        }
    }
}

impl ThreadPool {
    /// Creates a new thread pool with the specified number of threads.
    ///
    /// # Arguments
    /// * `size` - Number of evaluator threads to create
    /// * `global_env` - Shared global environment manager
    /// * `effect_coordinator` - Shared effect coordinator
    pub fn new(
        size: usize,
        global_env: Arc<GlobalEnvironmentManager>,
        effect_coordinator: Arc<EffectCoordinator>,
    ) -> Result<Self> {
        if size == 0 {
            return Err(crate::diagnostics::Error::runtime_error(
                "Thread pool size must be greater than 0".to_string(),
                None,
            ).boxed());
        }

        let work_queue = Arc::new(crossbeam::queue::SegQueue::new());
        let shutdown_signal = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stats = Arc::new(RwLock::new(ThreadPoolStats {
            created_at: Instant::now(),
            ..Default::default()
        }));

        let mut workers = Vec::with_capacity(size);

        // Create worker threads
        for worker_id in 0..size {
            let worker = Self::create_worker(
                worker_id as u64,
                global_env.clone(),
                effect_coordinator.clone(),
                work_queue.clone(),
                shutdown_signal.clone(),
                stats.clone(),
            )?;
            workers.push(worker);
        }

        // Update active worker count
        {
            let mut pool_stats = stats.write().unwrap();
            pool_stats.active_workers = workers.len();
        }

        Ok(Self {
            size,
            work_queue,
            workers,
            global_env,
            effect_coordinator,
            stats,
            shutdown_signal,
        })
    }

    /// Creates a single worker thread.
    fn create_worker(
        worker_id: u64,
        global_env: Arc<GlobalEnvironmentManager>,
        effect_coordinator: Arc<EffectCoordinator>,
        work_queue: Arc<crossbeam::queue::SegQueue<EvaluatorMessage>>,
        shutdown_signal: Arc<std::sync::atomic::AtomicBool>,
        pool_stats: Arc<RwLock<ThreadPoolStats>>,
    ) -> Result<WorkerThread> {
        let (worker_sender, worker_receiver) = channel::unbounded();
        
        // Clone what we need for the thread
        let worker_global_env = global_env.clone();
        let worker_effect_coordinator = effect_coordinator.clone();
        let worker_work_queue = work_queue.clone();
        let worker_shutdown_signal = shutdown_signal.clone();
        let worker_pool_stats = pool_stats.clone();
        let _worker_sender_clone = worker_sender.clone();

        let handle = thread::spawn(move || {
            Self::worker_thread_main(
                worker_id,
                worker_global_env,
                worker_effect_coordinator,
                worker_receiver,
                worker_work_queue,
                worker_shutdown_signal,
                worker_pool_stats,
            )
        });

        let thread_id = handle.thread().id();

        Ok(WorkerThread {
            id: worker_id,
            handle: Some(handle),
            thread_id: Some(thread_id),
            sender: worker_sender,
            stats: WorkerStats {
                created_at: Instant::now(),
                ..Default::default()
            },
        })
    }

    /// Main function for worker threads.
    fn worker_thread_main(
        worker_id: u64,
        global_env: Arc<GlobalEnvironmentManager>,
        effect_coordinator: Arc<EffectCoordinator>,
        worker_receiver: Receiver<EvaluatorMessage>,
        work_queue: Arc<crossbeam::queue::SegQueue<EvaluatorMessage>>,
        shutdown_signal: Arc<std::sync::atomic::AtomicBool>,
        pool_stats: Arc<RwLock<ThreadPoolStats>>,
    ) -> Result<()> {
        let thread_id = thread::current().id();
        
        // Register this thread with the effect coordinator
        effect_coordinator.register_thread(thread_id);
        
        // Create local evaluator
        let (evaluator_worker, _) = EvaluatorWorker::new(
            worker_id,
            global_env.clone(),
            effect_coordinator.clone(),
        );

        // Main worker loop
        loop {
            // Check for shutdown
            if shutdown_signal.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            // Try to get work from local queue first
            let message = if let Ok(msg) = worker_receiver.try_recv() {
                Some(msg)
            } else {
                // Try to steal work from global queue
                work_queue.pop()
            };

            if let Some(msg) = message {
                let start_time = Instant::now();
                
                // Process the message
                let result = Self::process_worker_message(msg, &evaluator_worker);
                
                let elapsed = start_time.elapsed();
                
                // Update statistics
                Self::update_worker_stats(&pool_stats, elapsed, result.is_ok());
            } else {
                // No work available, sleep briefly to avoid busy waiting
                thread::sleep(Duration::from_millis(1));
            }
        }

        // Unregister from effect coordinator
        effect_coordinator.unregister_thread(thread_id);
        
        Ok(())
    }

    /// Processes a message in a worker thread.
    fn process_worker_message(
        message: EvaluatorMessage,
        _evaluator_worker: &EvaluatorWorker,
    ) -> Result<()> {
        // For now, we'll handle messages directly here
        // In a full implementation, this would delegate to the evaluator worker
        match message {
            EvaluatorMessage::Evaluate { expr: _, span: _, sender } => {
                // Placeholder evaluation - just return unspecified
                let _ = sender.send(Ok(crate::eval::Value::Unspecified));
            }
            EvaluatorMessage::DefineGlobal { name: _, value: _ } => {
                // Placeholder - global definitions would be handled here
            }
            EvaluatorMessage::ImportModule { import_spec: _, sender } => {
                // Placeholder - module import would be handled here
                let _ = sender.send(Ok(std::collections::HashMap::new()));
            }
            EvaluatorMessage::Shutdown => {
                // Worker-level shutdown would be handled here
            }
        }
        
        Ok(())
    }

    /// Updates worker statistics.
    fn update_worker_stats(
        pool_stats: &Arc<RwLock<ThreadPoolStats>>,
        elapsed: Duration,
        success: bool,
    ) {
        let mut stats = pool_stats.write().unwrap();
        
        if success {
            stats.total_tasks_completed += 1;
        } else {
            stats.total_tasks_failed += 1;
        }
        
        // Update average task time (simple moving average)
        let total_completed = stats.total_tasks_completed;
        if total_completed > 0 {
            let current_avg = stats.average_task_time;
            stats.average_task_time = Duration::from_nanos(
                (current_avg.as_nanos() as u64 * (total_completed - 1) + elapsed.as_nanos() as u64) / total_completed
            );
        } else {
            stats.average_task_time = elapsed;
        }
    }

    /// Submits work to the thread pool.
    pub fn submit_work(&self, message: EvaluatorMessage) -> Result<()> {
        if self.shutdown_signal.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(crate::diagnostics::Error::runtime_error(
                "Cannot submit work to shutdown thread pool".to_string(),
                None,
            ).boxed());
        }

        // Add to global work queue for work stealing
        self.work_queue.push(message);
        
        // Update submitted task count
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_tasks_submitted += 1;
        }

        Ok(())
    }

    /// Spawns a new evaluator and returns a handle to it.
    pub fn spawn_evaluator(&self, handle_id: u64) -> Result<EvaluatorHandle> {
        if self.workers.is_empty() {
            return Err(crate::diagnostics::Error::runtime_error(
                "No workers available in thread pool".to_string(),
                None,
            ).boxed());
        }

        // For now, just use the first worker
        // In a full implementation, this would use load balancing
        let worker = &self.workers[0];
        
        Ok(EvaluatorHandle {
            thread_id: worker.thread_id.unwrap_or_else(|| thread::current().id()),
            sender: worker.sender.clone(),
            id: handle_id,
        })
    }

    /// Gets the size of the thread pool.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Gets current thread pool statistics.
    pub fn statistics(&self) -> ThreadPoolStats {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }

    /// Shuts down the thread pool gracefully.
    pub async fn shutdown(mut self) -> Result<()> {
        // Signal shutdown to all workers
        self.shutdown_signal.store(true, std::sync::atomic::Ordering::Relaxed);
        
        // Send shutdown messages to all workers
        for worker in &self.workers {
            let _ = worker.sender.send(EvaluatorMessage::Shutdown);
        }

        // Wait for all worker threads to complete
        let workers = std::mem::take(&mut self.workers);
        for mut worker in workers {
            if let Some(handle) = worker.handle.take() {
                match handle.join() {
                    Ok(result) => {
                        if let Err(e) = result {
                            eprintln!("Worker thread {} finished with error: {}", worker.id, e);
                        }
                    }
                    Err(_) => {
                        eprintln!("Worker thread {} panicked", worker.id);
                    }
                }
            }
        }

        Ok(())
    }

    /// Checks if the thread pool is running.
    pub fn is_running(&self) -> bool {
        !self.shutdown_signal.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Gets the number of active workers.
    pub fn active_worker_count(&self) -> usize {
        let stats = self.stats.read().unwrap();
        stats.active_workers
    }

    /// Gets the current work queue size.
    pub fn work_queue_size(&self) -> usize {
        self.work_queue.len()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Signal shutdown
        self.shutdown_signal.store(true, std::sync::atomic::Ordering::Relaxed);
        
        // Try to join remaining threads
        let workers = std::mem::take(&mut self.workers);
        for mut worker in workers {
            if let Some(handle) = worker.handle.take() {
                let _ = handle.join();
            }
        }
    }
}