//! Parallel computation primitives with work-stealing scheduler.
//!
//! This module provides high-performance parallel versions of common
//! functional programming patterns like map, filter, and reduce.

use crate::eval::Value;
use crate::diagnostics::{Error, Result};
use super::{ConcurrencyError, futures::Future};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use crossbeam::deque::{Injector, Stealer, Worker};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use num_cpus;

/// Parallel computation configuration.
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of worker threads (None for CPU count)
    pub num_threads: Option<usize>,
    /// Chunk size for work distribution
    pub chunk_size: usize,
    /// Enable work stealing
    pub work_stealing: bool,
    /// CPU affinity settings
    pub cpu_affinity: Option<Vec<usize>>,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            num_threads: None,
            chunk_size: 1000,
            work_stealing: true,
            cpu_affinity: None,
        }
    }
}

/// Work-stealing scheduler for parallel tasks.
pub struct WorkStealingScheduler {
    injector: Arc<Injector<Task>>,
    stealers: Vec<Stealer<Task>>,
    workers: Vec<Worker<Task>>,
    num_threads: usize,
    active_tasks: Arc<AtomicUsize>,
}

/// A task in the work-stealing scheduler.
type Task = Box<dyn FnOnce() -> Result<Value> + Send + 'static>;

impl WorkStealingScheduler {
    /// Creates a new work-stealing scheduler.
    pub fn new(config: ParallelConfig) -> Self {
        let num_threads = config.num_threads.unwrap_or_else(num_cpus::get);
        let injector = Arc::new(Injector::new());
        let mut workers = Vec::new();
        let mut stealers = Vec::new();

        for _ in 0..num_threads {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            workers.push(worker);
        }

        Self {
            injector,
            stealers,
            workers,
            num_threads,
            active_tasks: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Submits a task to the scheduler.
    pub fn submit<F>(&self, task: F) -> Result<()>
    where
        F: FnOnce() -> Result<Value> + Send + 'static,
    {
        self.injector.push(Box::new(task));
        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// Runs the scheduler until all tasks are completed.
    pub fn run_to_completion(self) -> Result<()> {
        let stealers = self.stealers.clone());
        let injector = self.injector.clone());
        let active_tasks = self.active_tasks.clone());

        thread::scope(|s| {
            for (i, worker) in self.workers.into_iter().enumerate() {
                let stealers = stealers.clone());
                let injector = injector.clone());
                let active_tasks = active_tasks.clone());
                
                s.spawn(move || {
                    loop {
                        // Try to get a task from local queue first
                        if let Some(task) = worker.pop() {
                            let _ = task(); // Execute task
                            active_tasks.fetch_sub(1, Ordering::SeqCst);
                            continue;
                        }

                        // Try to steal from global injector
                        match injector.steal() {
                            crossbeam::deque::Steal::Success(task) => {
                                let _ = task(); // Execute task
                                active_tasks.fetch_sub(1, Ordering::SeqCst);
                                continue;
                            }
                            _ => {} // Empty or Retry - continue to next iteration
                        }

                        // Try to steal from other workers
                        let mut found_work = false;
                        for (j, stealer) in stealers.iter().enumerate() {
                            if i != j {
                                match stealer.steal() {
                                    crossbeam::deque::Steal::Success(task) => {
                                        let _ = task(); // Execute task
                                        active_tasks.fetch_sub(1, Ordering::SeqCst);
                                        found_work = true;
                                        break;
                                    }
                                    _ => {} // Empty or Retry - continue to next stealer
                                }
                            }
                        }

                        if !found_work {
                            // No work available, check if we should exit
                            if active_tasks.load(Ordering::SeqCst) == 0 {
                                break;
                            }
                            // Yield to other threads
                            thread::yield_now();
                        }
                    }
                });
            }
        });

        Ok(())
    }
}

/// Parallel computation operations.
pub struct ParallelOps {
    scheduler: WorkStealingScheduler,
    config: ParallelConfig,
}

impl ParallelOps {
    /// Creates a new parallel operations instance.
    pub fn new(config: ParallelConfig) -> Self {
        Self {
            scheduler: WorkStealingScheduler::new(config.clone()),
            config,
        }
    }

    /// Parallel map operation.
    pub fn par_map<F>(&self, values: Vec<Value>, f: F) -> Future
    where
        F: Fn(Value) -> Result<Value> + Send + Sync + 'static,
    {
        let f = Arc::new(f);
        let _results = Arc::new(Mutex::new(Vec::<Result<Value>>::with_capacity(values.len())));
        let chunk_size = self.config.chunk_size;

        Future::new(async move {
            if values.is_empty() {
                return Ok(Value::Nil);
            }

            // Use rayon for simplicity and performance
            let par_results: std::result::Result<Vec<_>, Error> = values
                .into_par_iter()
                .with_min_len(chunk_size)
                .map(|value| f(value))
                .collect();

            match par_results {
                Ok(results) => {
                    let mut list = Value::Nil;
                    for value in results.into_iter().rev() {
                        list = Value::pair(value, list);
                    }
                    Ok(list)
                }
                Err(error) => Err(error),
            }
        })
    }

    /// Parallel filter operation.
    pub fn par_filter<F>(&self, values: Vec<Value>, predicate: F) -> Future
    where
        F: Fn(&Value) -> Result<bool> + Send + Sync + 'static,
    {
        let predicate = Arc::new(predicate);
        let chunk_size = self.config.chunk_size;

        Future::new(async move {
            if values.is_empty() {
                return Ok(Value::Nil);
            }

            let filtered: std::result::Result<Vec<_>, Error> = values
                .into_par_iter()
                .with_min_len(chunk_size)
                .filter_map(|value| {
                    match predicate(&value) {
                        Ok(true) => Some(Ok(value)),
                        Ok(false) => None,
                        Err(e) => Some(Err(e)),
                    }
                })
                .collect();

            match filtered {
                Ok(results) => {
                    let mut list = Value::Nil;
                    for value in results.into_iter().rev() {
                        list = Value::pair(value, list);
                    }
                    Ok(list)
                }
                Err(error) => Err(error),
            }
        })
    }

    /// Parallel reduce operation.
    pub fn par_reduce<F>(&self, values: Vec<Value>, identity: Value, f: F) -> Future
    where
        F: Fn(Value, Value) -> Result<Value> + Send + Sync + 'static,
    {
        let f = Arc::new(f);
        let chunk_size = self.config.chunk_size;

        Future::new(async move {
            if values.is_empty() {
                return Ok(identity);
            }

            // Convert values to Results and reduce
            let results: Vec<Result<Value>> = values
                .into_par_iter()
                .with_min_len(chunk_size)
                .map(|v| Ok(v))
                .collect();

            // Sequential reduce to handle errors properly
            let mut acc = identity;
            for result in results {
                match result {
                    Ok(val) => {
                        acc = f(acc, val)?;
                    }
                    Err(e) => return Err(e),
                }
            }

            Ok(acc)
        })
    }

    /// Parallel fold operation with chunking.
    pub fn par_fold<F>(&self, values: Vec<Value>, identity: Value, f: F) -> Future
    where
        F: Fn(Value, Value) -> Result<Value> + Send + Sync + 'static,
    {
        let f = Arc::new(f);
        let chunk_size = self.config.chunk_size;

        Future::new(async move {
            if values.is_empty() {
                return Ok(identity);
            }

            let result = values
                .into_par_iter()
                .with_min_len(chunk_size)
                .try_fold(|| identity.clone()), |acc, value| f(acc, value))
                .try_reduce(|| identity.clone()), |a, b| f(a, b))?;

            Ok(result)
        })
    }

    /// Parallel for-each operation.
    pub fn par_for_each<F>(&self, values: Vec<Value>, f: F) -> Future
    where
        F: Fn(Value) -> Result<()> + Send + Sync + 'static,
    {
        let f = Arc::new(f);
        let chunk_size = self.config.chunk_size;

        Future::new(async move {
            let result: std::result::Result<(), Error> = values
                .into_par_iter()
                .with_min_len(chunk_size)
                .map(|value| f(value))
                .collect();

            result?;
            Ok(Value::Unspecified)
        })
    }

    /// Parallel partition operation.
    pub fn par_partition<F>(&self, values: Vec<Value>, predicate: F) -> Future
    where
        F: Fn(&Value) -> Result<bool> + Send + Sync + 'static,
    {
        let predicate = Arc::new(predicate);
        let chunk_size = self.config.chunk_size;

        Future::new(async move {
            if values.is_empty() {
                return Ok(Value::from_vec(vec![Value::Nil, Value::Nil]));
            }

            let (trues, falses): (Vec<_>, Vec<_>) = values
                .into_par_iter()
                .with_min_len(chunk_size)
                .map(|value| {
                    let matches = predicate(&value)?;
                    Ok((value, matches))
                })
                .collect::<std::result::Result<Vec<_>, Error>>()?
                .into_iter()
                .partition(|(_, matches)| *matches);

            let true_list = trues.into_iter()
                .map(|(value, _)| value)
                .collect::<Vec<_>>();
            let false_list = falses.into_iter()
                .map(|(value, _)| value)
                .collect::<Vec<_>>();

            Ok(Value::from_vec(vec![
                Value::from_vec(true_list),
                Value::from_vec(false_list),
            ]))
        })
    }

    /// Parallel find operation.
    pub fn par_find<F>(&self, values: Vec<Value>, predicate: F) -> Future
    where
        F: Fn(&Value) -> Result<bool> + Send + Sync + 'static,
    {
        let predicate = Arc::new(predicate);
        let chunk_size = self.config.chunk_size;

        Future::new(async move {
            let result = values
                .into_par_iter()
                .with_min_len(chunk_size)
                .find_first(|value| {
                    predicate(value).unwrap_or(false)
                });

            Ok(result.unwrap_or(Value::Nil))
        })
    }

    /// Parallel any operation.
    pub fn par_any<F>(&self, values: Vec<Value>, predicate: F) -> Future
    where
        F: Fn(&Value) -> Result<bool> + Send + Sync + 'static,
    {
        let predicate = Arc::new(predicate);
        let chunk_size = self.config.chunk_size;

        Future::new(async move {
            let result = values
                .into_par_iter()
                .with_min_len(chunk_size)
                .any(|value| {
                    predicate(&value).unwrap_or(false)
                });

            Ok(Value::boolean(result))
        })
    }

    /// Parallel all operation.
    pub fn par_all<F>(&self, values: Vec<Value>, predicate: F) -> Future
    where
        F: Fn(&Value) -> Result<bool> + Send + Sync + 'static,
    {
        let predicate = Arc::new(predicate);
        let chunk_size = self.config.chunk_size;

        Future::new(async move {
            let result = values
                .into_par_iter()
                .with_min_len(chunk_size)
                .all(|value| {
                    predicate(&value).unwrap_or(false)
                });

            Ok(Value::boolean(result))
        })
    }

    /// Parallel sort operation.
    pub fn par_sort<F>(&self, mut values: Vec<Value>, compare: F) -> Future
    where
        F: Fn(&Value, &Value) -> Result<std::cmp::Ordering> + Send + Sync + 'static,
    {
        let compare = Arc::new(compare);

        Future::new(async move {
            values.par_sort_by(|a, b| {
                compare(a, b).unwrap_or(std::cmp::Ordering::Equal)
            });

            Ok(Value::from_vec(values))
        })
    }
}

/// Thread pool for CPU-intensive tasks.
pub struct ThreadPool {
    pool: rayon::ThreadPool,
}

impl ThreadPool {
    /// Creates a new thread pool with the given configuration.
    pub fn new(config: ParallelConfig) -> Result<Self> {
        let num_threads = config.num_threads.unwrap_or_else(num_cpus::get);
        
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .map_err(|e| Error::runtime_error(format!("Failed to create thread pool: {}", e), None))?;

        Ok(Self { pool })
    }

    /// Executes a task on the thread pool.
    pub fn execute<F, R>(&self, task: F) -> Future
    where
        F: FnOnce() -> Result<R> + Send + 'static,
        R: Into<Value> + Send + 'static,
    {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        
        self.pool.spawn(move || {
            let result = task().map(|r| r.into())
            let _ = sender.send(result);
        });

        Future::new(async move {
            receiver.await
                .map_err(|_| ConcurrencyError::Cancelled.boxed())?
        })
    }

    /// Executes multiple tasks in parallel.
    pub fn execute_all<F, R>(&self, tasks: Vec<F>) -> Future
    where
        F: FnOnce() -> Result<R> + Send + 'static,
        R: Into<Value> + Send + 'static,
    {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let num_tasks = tasks.len();
        let results = Arc::new(Mutex::new(Vec::with_capacity(num_tasks)));
        let counter = Arc::new(AtomicUsize::new(0));
        let sender = Arc::new(Mutex::new(Some(sender)));

        for (i, task) in tasks.into_iter().enumerate() {
            let results = results.clone());
            let counter = counter.clone());
            let sender = sender.clone());

            self.pool.spawn(move || {
                let result = task().map(|r| r.into())
                
                {
                    let mut results = results.lock().unwrap();
                    if results.len() <= i {
                        results.resize(i + 1, Value::Nil);
                    }
                    results[i] = result.unwrap_or(Value::Nil);
                }

                let completed = counter.fetch_add(1, Ordering::SeqCst) + 1;
                if completed == num_tasks {
                    if let Some(sender) = sender.lock().unwrap().take() {
                        let final_results = results.lock().unwrap().clone());
                        let _ = sender.send(Ok(Value::from_vec(final_results)));
                    }
                }
            });
        }

        Future::new(async move {
            receiver.await
                .map_err(|_| ConcurrencyError::Cancelled.boxed())?
        })
    }
}

/// CPU affinity utilities.
pub struct CpuAffinity;

impl CpuAffinity {
    /// Sets CPU affinity for the current thread.
    #[cfg(target_os = "linux")]
    pub fn set_affinity(cpu_ids: &[usize]) -> Result<()> {
        use std::mem;
        
        let mut cpu_set: libc::cpu_set_t = unsafe { mem::zeroed() };
        
        for &cpu_id in cpu_ids {
            unsafe {
                libc::CPU_SET(cpu_id, &mut cpu_set);
            }
        }
        
        let result = unsafe {
            libc::sched_setaffinity(0, mem::size_of::<libc::cpu_set_t>(), &cpu_set)
        };
        
        if result != 0 {
            Err(Box::new(Error::runtime_error("Failed to set CPU affinity".to_string(), None))
        } else {
            Ok(())
        }
    }

    /// Sets CPU affinity for the current thread (no-op on non-Linux systems).
    #[cfg(not(target_os = "linux"))]
    pub fn set_affinity(_cpu_ids: &[usize]) -> Result<()> {
        // CPU affinity is not supported on this platform
        Ok(())
    }

    /// Gets the number of available CPUs.
    pub fn cpu_count() -> usize {
        num_cpus::get()
    }

    /// Gets the optimal number of threads for CPU-bound tasks.
    pub fn optimal_thread_count() -> usize {
        num_cpus::get()
    }
}

