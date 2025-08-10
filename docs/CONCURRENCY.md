# Concurrency Guide

Lambdust provides a comprehensive concurrency model combining actor systems, parallel evaluation, and advanced synchronization primitives. This guide covers the complete concurrency infrastructure and usage patterns.

## Concurrency Overview

Lambdust's concurrency system includes:

- **Actor Model**: Message-passing concurrency with isolated state
- **Parallel Evaluation**: Automatic parallelization of computations  
- **Advanced Synchronization**: Thread-safe primitives with performance optimization
- **Software Transactional Memory**: Composable atomic operations
- **Distributed Computing**: Network-transparent actor communication
- **Effect-Aware Concurrency**: Integration with the algebraic effect system

## Actor Model

### Basic Actor System

```rust
use lambdust::concurrency::actors::{Actor, ActorSystem, Message};

// Define an actor
#[derive(Debug)]
struct CounterActor {
    count: i64,
    name: String,
}

#[derive(Debug, Clone)]
enum CounterMessage {
    Increment(i64),
    Decrement(i64),
    GetCount(tokio::sync::oneshot::Sender<i64>),
    Reset,
}

impl Actor for CounterActor {
    type Message = CounterMessage;
    
    async fn handle(&mut self, message: Self::Message) -> Result<(), Box<dyn std::error::Error>> {
        match message {
            CounterMessage::Increment(delta) => {
                self.count += delta;
                println!("[{}] Incremented by {}, count: {}", self.name, delta, self.count);
            }
            CounterMessage::Decrement(delta) => {
                self.count -= delta;
                println!("[{}] Decremented by {}, count: {}", self.name, delta, self.count);
            }
            CounterMessage::GetCount(sender) => {
                let _ = sender.send(self.count);
            }
            CounterMessage::Reset => {
                println!("[{}] Reset from {} to 0", self.name, self.count);
                self.count = 0;
            }
        }
        Ok(())
    }
}

// Use the actor system
async fn actor_example() -> Result<(), Box<dyn std::error::Error>> {
    let system = ActorSystem::new();
    
    // Spawn actors
    let counter1 = system.spawn(CounterActor { count: 0, name: "Counter1".to_string() }).await?;
    let counter2 = system.spawn(CounterActor { count: 100, name: "Counter2".to_string() }).await?;
    
    // Send messages
    counter1.send(CounterMessage::Increment(5)).await?;
    counter2.send(CounterMessage::Decrement(10)).await?;
    
    // Query state
    let (tx, rx) = tokio::sync::oneshot::channel();
    counter1.send(CounterMessage::GetCount(tx)).await?;
    let count = rx.await?;
    println!("Counter1 final count: {}", count);
    
    Ok(())
}
```

### Scheme Actor Interface

```scheme
;; Define actor in Scheme
(define-actor bank-account
  (state (balance 0)
         (account-id "")
         (transaction-log '()))
  
  ;; Handle deposit message
  (handle (deposit amount)
    (when (> amount 0)
      (set! balance (+ balance amount))
      (set! transaction-log 
            (cons (make-transaction 'deposit amount (current-time))
                  transaction-log))
      (log-info (format "Deposited ~a, new balance: ~a" amount balance)))
    balance)
  
  ;; Handle withdrawal message
  (handle (withdraw amount)
    (cond [(> amount balance)
           (log-warn (format "Insufficient funds: ~a > ~a" amount balance))
           (error 'insufficient-funds)]
          [(> amount 0)
           (set! balance (- balance amount))
           (set! transaction-log
                 (cons (make-transaction 'withdraw amount (current-time))
                       transaction-log))
           (log-info (format "Withdrew ~a, new balance: ~a" amount balance))
           balance]
          [else
           (error 'invalid-amount)]))
  
  ;; Handle balance query
  (handle (get-balance)
    balance)
  
  ;; Handle transaction history
  (handle (get-transactions)
    (reverse transaction-log)))

;; Use the actor
(define account (spawn-actor bank-account 
                             #:initial-state '((account-id "ACC001"))))

;; Send messages to actor
(send-message account (deposit 1000))     ;; Returns: 1000
(send-message account (withdraw 250))     ;; Returns: 750  
(send-message account (get-balance))      ;; Returns: 750
```

### Actor Supervision

```scheme
;; Supervisor actor for fault tolerance
(define-supervisor-actor banking-supervisor
  (supervision-strategy 'one-for-one)  ;; Restart failed child only
  (max-restarts 5)
  (restart-window 60) ;; 5 restarts in 60 seconds max
  
  (supervise account-actors
    (lambda (child-spec)
      (spawn-actor bank-account 
                   #:name (child-spec-name child-spec)
                   #:initial-state (child-spec-state child-spec))))
  
  ;; Handle child failure
  (on-child-failure (child-ref reason)
    (log-error (format "Account actor ~a failed: ~a" 
                      (actor-name child-ref) reason))
    ;; Custom restart logic
    (restart-child child-ref)))

;; Create supervised actor system
(define supervisor (spawn-supervisor banking-supervisor))
(define account1 (supervisor-spawn-child supervisor 
                                        'account-1 
                                        '((account-id "ACC001"))))
```

## Parallel Evaluation

### Automatic Parallelization

```scheme
;; Parallel map - automatically distributes work
(define (parallel-processing data)
  (parallel-map 
    (lambda (item)
      (expensive-computation item))
    data))

;; Parallel fold with custom combiner
(define (parallel-sum numbers)
  (parallel-fold + 0 numbers))

;; Parallel filter
(define (parallel-find-primes numbers)
  (parallel-filter prime? numbers))

;; Control parallelism level
(parameterize ([max-parallelism 4])
  (parallel-map computation large-dataset))
```

### Futures and Promises

```scheme
;; Create futures for asynchronous computation
(define future1 (future (expensive-computation-1)))
(define future2 (future (expensive-computation-2)))
(define future3 (future (expensive-computation-3)))

;; Combine future results
(define combined-result
  (future
    (+ (force future1)
       (force future2)
       (force future3))))

;; Promise-based coordination
(define promise1 (make-promise))
(define promise2 (make-promise))

;; Producer thread
(spawn
  (lambda ()
    (let ([result (compute-data)])
      (deliver-promise promise1 result))))

;; Consumer thread
(spawn
  (lambda ()
    (let ([data (promise-value promise1)])
      (let ([processed (process-data data)])
        (deliver-promise promise2 processed)))))

;; Wait for final result
(define final-result (promise-value promise2))
```

### Work-Stealing Parallelism

```rust
use lambdust::concurrency::parallel::{WorkStealingPool, Task};

// Create work-stealing thread pool
let pool = WorkStealingPool::new()
    .with_thread_count(num_cpus::get())
    .with_queue_size(10000)
    .with_work_stealing_strategy(WorkStealingStrategy::Randomized);

// Submit parallel tasks
let tasks: Vec<Task<i32>> = data.iter().map(|item| {
    pool.submit(move || expensive_computation(*item))
}).collect();

// Collect results when ready
let results: Vec<i32> = tasks.into_iter()
    .map(|task| task.join().unwrap())
    .collect();
```

## Synchronization Primitives

### Thread-Safe Data Structures

```rust
use lambdust::concurrency::sync::{
    Mutex, RwLock, Semaphore, Barrier, 
    AtomicRef, LockFreeQueue, AtomicCounter
};

// Mutex for exclusive access
let shared_data = Mutex::new(HashMap::new());
{
    let mut guard = shared_data.lock().await;
    guard.insert("key".to_string(), "value".to_string());
}

// RwLock for multiple readers, single writer
let config = RwLock::new(ApplicationConfig::default());
{
    let read_guard = config.read().await;
    println!("Current setting: {}", read_guard.get_setting("key"));
}
{
    let mut write_guard = config.write().await;
    write_guard.update_setting("key", "new_value");
}

// Semaphore for resource control
let resource_pool = Semaphore::new(5); // Allow 5 concurrent users
let permit = resource_pool.acquire().await?;
// Use resource
drop(permit); // Release resource

// Barrier for synchronization
let barrier = Barrier::new(4); // Wait for 4 threads
barrier.wait().await;

// Lock-free data structures
let queue = LockFreeQueue::new();
queue.push(item).await?;
let item = queue.pop().await?;

// Atomic operations
let counter = AtomicCounter::new(0);
let old_value = counter.fetch_add(1, Ordering::SeqCst);
```

### Scheme Synchronization Interface

```scheme
;; Mutex usage in Scheme
(define shared-counter (make-mutex 0))

(define (increment-shared-counter)
  (with-mutex shared-counter
    (lambda (current)
      (+ current 1))))

;; Multiple threads incrementing safely
(parallel-eval
  (spawn (lambda () (for-each (lambda (_) (increment-shared-counter)) 
                             (range 0 1000))))
  (spawn (lambda () (for-each (lambda (_) (increment-shared-counter)) 
                             (range 0 1000))))
  (spawn (lambda () (for-each (lambda (_) (increment-shared-counter)) 
                             (range 0 1000)))))

;; Final result should be 3000
(display (format "Final counter value: ~a" (mutex-value shared-counter)))

;; Condition variables for coordination
(define buffer (make-bounded-buffer 10))
(define buffer-not-empty (make-condition-variable))
(define buffer-not-full (make-condition-variable))

;; Producer
(define (producer items)
  (for-each
    (lambda (item)
      (with-mutex buffer
        (lambda ()
          (when (buffer-full? buffer)
            (condition-wait buffer-not-full))
          (buffer-put! buffer item)
          (condition-signal buffer-not-empty))))
    items))

;; Consumer
(define (consumer)
  (with-mutex buffer
    (lambda ()
      (when (buffer-empty? buffer)
        (condition-wait buffer-not-empty))
      (let ([item (buffer-get! buffer)])
        (condition-signal buffer-not-full)
        item))))
```

## Software Transactional Memory

### STM Basics

```scheme
;; Define transactional variables
(define account1-balance (make-tvar 1000))
(define account2-balance (make-tvar 500))
(define transaction-log (make-tvar '()))

;; Atomic money transfer
(define (transfer from-account to-account amount)
  (atomic
    (let ([from-balance (tvar-read from-account)]
          [to-balance (tvar-read to-account)])
      (if (>= from-balance amount)
          (begin
            (tvar-write! from-account (- from-balance amount))
            (tvar-write! to-account (+ to-balance amount))
            (tvar-write! transaction-log 
                        (cons (make-transaction from-account to-account amount)
                              (tvar-read transaction-log)))
            #t) ;; Success
          #f)))) ;; Insufficient funds

;; Concurrent transfers - all atomic
(parallel-eval
  (spawn (lambda () (transfer account1-balance account2-balance 200)))
  (spawn (lambda () (transfer account2-balance account1-balance 100)))
  (spawn (lambda () (transfer account1-balance account2-balance 50))))

;; Balances remain consistent despite concurrency
(display (format "Account 1: ~a, Account 2: ~a" 
                (tvar-read account1-balance)
                (tvar-read account2-balance)))
```

### Composable Transactions

```scheme
;; Composable transactional operations
(define (withdraw account amount)
  (atomic
    (let ([balance (tvar-read account)])
      (if (>= balance amount)
          (begin
            (tvar-write! account (- balance amount))
            amount)
          (retry))))) ;; Retry until sufficient funds

(define (deposit account amount)
  (atomic
    (let ([balance (tvar-read account)])
      (tvar-write! account (+ balance amount))
      (+ balance amount))))

;; Compose transactions
(define (transfer-with-fee from to amount fee-account fee)
  (atomic
    (withdraw from amount)
    (deposit to (- amount fee))
    (deposit fee-account fee)))

;; Alternative composition using orElse
(define (try-transfer-from-either account1 account2 target amount)
  (or-else
    (atomic (transfer account1 target amount))
    (atomic (transfer account2 target amount))))
```

## Distributed Concurrency

### Remote Actors

```scheme
;; Distributed actor system
(define-distributed-actor distributed-counter
  (state (count 0)
         (node-id (get-node-id)))
  
  (handle (increment)
    (set! count (+ count 1))
    (broadcast-to-replicas `(sync-count ,count ,node-id))
    count)
  
  (handle (sync-count remote-count remote-node)
    (when (> remote-count count)
      (log-info (format "Syncing count from node ~a: ~a" remote-node remote-count))
      (set! count remote-count)))
  
  (handle (get-count)
    count))

;; Deploy across multiple nodes
(define node1-counter (spawn-distributed-actor 
                        distributed-counter 
                        #:node "node1.cluster.local"))
(define node2-counter (spawn-distributed-actor 
                        distributed-counter 
                        #:node "node2.cluster.local"))

;; Operations work transparently across network
(send-message node1-counter (increment))  ;; Updates on node1
(send-message node2-counter (get-count))  ;; Reads from node2 (synced)
```

### Network-Transparent Communication

```rust
use lambdust::concurrency::distributed::{DistributedNode, ClusterConfig};

// Set up distributed computing cluster
let cluster_config = ClusterConfig {
    node_id: "worker-1".to_string(),
    cluster_peers: vec![
        "worker-2.cluster.local:9000".to_string(),
        "worker-3.cluster.local:9000".to_string(),
    ],
    heartbeat_interval: Duration::from_secs(5),
    failure_detection_timeout: Duration::from_secs(15),
};

let node = DistributedNode::new(cluster_config).await?;

// Spawn distributed computation
let distributed_task = node.spawn_distributed_task(
    "compute-pi",
    |start: i64, end: i64| {
        // Compute π using Monte Carlo method
        monte_carlo_pi_segment(start, end)
    },
    vec![(0, 1000000), (1000000, 2000000), (2000000, 3000000)]
).await?;

// Collect results from all nodes
let results = distributed_task.collect().await?;
let pi_estimate = results.iter().sum::<f64>() / 3.0;
```

## Effect-Aware Concurrency

### Concurrent Effects

```scheme
;; Effects in concurrent context
(define (concurrent-file-processing filenames)
  (with-effect-coordination
    (parallel-map
      (lambda (filename)
        (do [content (read-file filename)]           ;; FileSystem effect
            [_ (log-info (format "Processing ~a" filename))] ;; Logging effect
            [processed (process-content content)]
            [output (string-append filename ".processed")]
            [_ (write-file output processed)]
            (return output)))
      filenames)))

;; Effect isolation between actors
(define-actor file-processor
  (with-effects [FileSystem Logging])  ;; Declare allowed effects
  
  (handle (process-file filename)
    (with-effect-isolation
      (do [content (read-file filename)]
          [result (transform-content content)]
          [_ (log-info (format "Processed ~a: ~a bytes" 
                              filename (string-length result)))]
          (return result)))))
```

### Coordinated Effect Handling

```rust
use lambdust::runtime::effect_coordination::{
    EffectCoordinator, ConcurrentEffectSystem, EffectIsolationLevel
};

// Set up effect coordination for concurrent system
let coordinator = EffectCoordinator::new()
    .with_concurrent_effects(true)
    .with_isolation_level(EffectIsolationLevel::Strict)
    .with_resource_limits(ResourceLimits {
        max_concurrent_effects: 100,
        memory_limit: 512 * 1024 * 1024, // 512MB
        time_limit: Duration::from_secs(60),
    });

// Execute effects across multiple actors with coordination
let concurrent_system = ConcurrentEffectSystem::new(coordinator);
let results = concurrent_system.execute_parallel_effects(
    vec![
        Effect::FileOperation(FileOp::Read("file1.txt".to_string())),
        Effect::FileOperation(FileOp::Read("file2.txt".to_string())),
        Effect::NetworkOperation(NetworkOp::HttpGet("api.example.com".to_string())),
    ]
).await?;
```

## Performance Optimization

### Lock-Free Programming

```rust
use lambdust::concurrency::sync::{AtomicRef, LockFreeQueue, AtomicCounter};
use std::sync::atomic::{AtomicPtr, Ordering};

// Lock-free stack implementation
struct LockFreeStack<T> {
    head: AtomicPtr<Node<T>>,
}

struct Node<T> {
    data: T,
    next: *mut Node<T>,
}

impl<T> LockFreeStack<T> {
    fn new() -> Self {
        Self {
            head: AtomicPtr::new(std::ptr::null_mut()),
        }
    }
    
    fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data,
            next: std::ptr::null_mut(),
        }));
        
        loop {
            let head = self.head.load(Ordering::Acquire);
            unsafe { (*new_node).next = head; }
            
            if self.head.compare_exchange_weak(
                head,
                new_node,
                Ordering::Release,
                Ordering::Relaxed
            ).is_ok() {
                break;
            }
        }
    }
    
    fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            if head.is_null() {
                return None;
            }
            
            let next = unsafe { (*head).next };
            if self.head.compare_exchange_weak(
                head,
                next,
                Ordering::Release,
                Ordering::Relaxed
            ).is_ok() {
                let data = unsafe { Box::from_raw(head).data };
                return Some(data);
            }
        }
    }
}
```

### High-Performance Actor Messaging

```scheme
;; Optimized actor for high-throughput messaging
(define-actor high-throughput-processor
  (state (processed-count 0)
         (batch-buffer '())
         (batch-size 1000))
  
  ;; Batch processing for better throughput
  (handle (process-item item)
    (set! batch-buffer (cons item batch-buffer))
    (when (>= (length batch-buffer) batch-size)
      (let ([batch (reverse batch-buffer)])
        (set! batch-buffer '())
        (set! processed-count (+ processed-count batch-size))
        (process-batch batch))))
  
  ;; Handle backpressure
  (handle (get-queue-size)
    (actor-mailbox-size (self)))
  
  (on-mailbox-full (message sender)
    ;; Apply backpressure by slowing down sender
    (send-message sender (slow-down 100))))  ;; 100ms delay

;; Use with performance monitoring
(define processor (spawn-actor high-throughput-processor 
                              #:mailbox-size 10000
                              #:priority 'high))

;; Monitor performance
(define (monitor-performance actor)
  (let loop ()
    (let ([queue-size (send-message-sync actor (get-queue-size))])
      (when (> queue-size 5000)  ;; High queue size
        (log-warn "High message queue size detected"))
      (thread-sleep 1000)  ;; Check every second
      (loop))))
```

## Debugging and Monitoring

### Concurrency Debugging

```scheme
;; Debug concurrent actors
(define-debug-actor debug-counter
  (enable-message-tracing #t)
  (enable-state-snapshots #t)
  
  (state (count 0))
  
  (handle (increment)
    (debug-trace "Incrementing count from ~a" count)
    (set! count (+ count 1))
    (debug-snapshot 'count count)
    count))

;; Deadlock detection
(with-deadlock-detection
  (with-mutex mutex1
    (lambda ()
      (with-mutex mutex2
        (lambda ()
          (critical-section))))))

;; Race condition detection
(define (test-race-conditions)
  (with-race-detection
    (let ([shared-var 0])
      (parallel-eval
        (spawn (lambda () (set! shared-var (+ shared-var 1))))
        (spawn (lambda () (set! shared-var (+ shared-var 1)))))
      shared-var)))
```

### Performance Monitoring

```rust
use lambdust::concurrency::monitoring::{ConcurrencyMonitor, ActorMetrics};

// Monitor actor system performance
let monitor = ConcurrencyMonitor::new()
    .with_metrics([
        ActorMetrics::MessageThroughput,
        ActorMetrics::MailboxUtilization,
        ActorMetrics::ProcessingLatency,
        ActorMetrics::ErrorRates,
    ])
    .with_sampling_interval(Duration::from_secs(1));

monitor.start().await?;

// Query performance metrics
let metrics = monitor.snapshot().await?;
println!("Messages/sec: {}", metrics.message_throughput);
println!("Avg mailbox utilization: {:.1}%", metrics.avg_mailbox_utilization * 100.0);
println!("P99 latency: {:?}", metrics.p99_latency);
```

## Configuration

### Concurrency System Configuration

```toml
[concurrency]
# Actor system
actor_system_threads = "auto"  # Number of CPU cores
mailbox_default_size = 1000
supervisor_restart_strategy = "one_for_one"
max_restarts = 5
restart_window = "60s"

# Parallel evaluation
parallel_threads = "auto"
work_stealing = true
task_queue_size = 10000
parallel_threshold = 100  # Minimum items for parallelization

# Synchronization
mutex_spin_count = 1000
rwlock_prefer_writers = false
semaphore_fairness = true

# STM configuration
stm_retry_limit = 1000
stm_contention_backoff = "exponential"
stm_gc_threshold = 10000

# Distributed computing
cluster_heartbeat = "5s"
failure_timeout = "15s"
network_buffer_size = "64KB"
compression = "lz4"

# Monitoring
enable_metrics = true
metrics_sampling = "1s"
deadlock_detection = true  # Enable in development
race_detection = false     # Expensive, use for testing
```

This concurrency guide demonstrates Lambdust's comprehensive approach to concurrent and parallel programming, providing both high-level abstractions and low-level control for building efficient, scalable applications.