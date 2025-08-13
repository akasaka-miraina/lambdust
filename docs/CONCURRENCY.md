# Lambdust Concurrency Guide

This document provides comprehensive documentation of Lambdust's concurrency model, which combines multiple paradigms to provide safe, efficient, and expressive parallel programming capabilities.

## Table of Contents

1. [Concurrency Overview](#concurrency-overview)
2. [Actor System](#actor-system)
3. [Parallel Evaluation](#parallel-evaluation)
4. [Synchronization Primitives](#synchronization-primitives)
5. [Software Transactional Memory](#software-transactional-memory)
6. [Lock-Free Programming](#lock-free-programming)
7. [Async/Await Integration](#asyncawait-integration)
8. [Best Practices](#best-practices)

## Concurrency Overview

Lambdust provides a comprehensive concurrency system that supports multiple programming models:

### **Concurrency Architecture**

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Actor Model   │    │  Parallel Eval  │    │      STM        │
│                 │    │                 │    │                 │
│ • Message Pass  │    │ • Work Stealing │    │ • Transactions  │
│ • Isolation     │    │ • Fork/Join     │    │ • Retry Logic   │
│ • Supervision   │    │ • Data Parallel │    │ • Composability │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                        │                        │
         └────────────────────────┼────────────────────────┘
                                  │
                        ┌─────────────────┐
                        │  Thread Pool    │
                        │   Scheduler     │
                        │                 │
                        │ • Work Stealing │
                        │ • Load Balance  │
                        │ • NUMA Aware    │
                        └─────────────────┘
                                  │
                        ┌─────────────────┐
                        │  Lock-Free      │
                        │  Primitives     │
                        │                 │
                        │ • Atomic Ops    │
                        │ • CAS Loops     │
                        │ • Memory Order  │
                        └─────────────────┘
```

### **Concurrency Features**

- **Actor Model**: Isolated actors communicating via message passing
- **Parallel Evaluation**: Multi-threaded expression evaluation
- **Work-Stealing Scheduler**: Efficient load balancing across threads
- **Software Transactional Memory**: Composable atomic transactions
- **Lock-Free Data Structures**: High-performance concurrent containers
- **Async/Await**: Integration with Rust's async ecosystem
- **Effect System Integration**: Effect-aware concurrency

## Actor System

### **Actor Model Implementation** (`src/concurrency/actors.rs`)

Lambdust implements a full-featured actor system with supervision, mailboxes, and lifecycle management:

```rust
/// Actor system managing actor lifecycle and communication
pub struct ActorSystem {
    actors: Arc<RwLock<HashMap<ActorId, ActorRef>>>,
    supervisor: Arc<Supervisor>,
    mailbox_factory: Arc<dyn MailboxFactory>,
    scheduler: Arc<ActorScheduler>,
}

/// Reference to a running actor
pub struct ActorRef {
    id: ActorId,
    mailbox: Arc<dyn Mailbox>,
    system: Weak<ActorSystem>,
}

/// Actor behavior definition
pub trait Actor: Send + 'static {
    type Message: Send + 'static;
    type State: Send + 'static;
    
    /// Handle incoming message
    fn receive(&mut self, msg: Self::Message, state: &mut Self::State) -> ActorResult;
    
    /// Initialize actor state
    fn initialize(&mut self) -> Self::State;
    
    /// Handle actor lifecycle events
    fn on_start(&mut self, _state: &mut Self::State) {}
    fn on_stop(&mut self, _state: &mut Self::State) {}
}
```

### **Creating and Using Actors**

#### **Basic Actor Definition**
```scheme
;; Define a counter actor
(define counter-actor
  (actor
    ;; Initial state
    [(initial-state 0)]
    
    ;; Message handlers
    [(increment n)
     (set! state (+ state n))
     state]
    
    [(decrement n)
     (set! state (- state n))
     state]
    
    [(get)
     state]))

;; Spawn the actor
(define counter (spawn counter-actor))

;; Send messages
(send! counter '(increment 5))
(send! counter '(increment 3))
(define current-value (ask counter '(get)))  ; => 8
```

#### **Supervisor Trees**
```scheme
;; Define a supervisor strategy
(define worker-supervisor
  (supervisor
    [(strategy 'one-for-one)]
    [(max-restarts 5)]
    [(restart-window 60)] ; seconds
    
    ;; Child specifications
    [(child 'worker-1 worker-actor)]
    [(child 'worker-2 worker-actor)]
    [(child 'worker-3 worker-actor)]))

;; Start supervised actors
(define supervisor-ref (start-supervisor worker-supervisor))
```

### **Actor Mailboxes and Scheduling**

#### **Mailbox Types**
```rust
pub enum MailboxType {
    Unbounded,                    // Unlimited message queue
    Bounded(usize),              // Fixed-size message queue  
    Priority(Box<dyn Comparator>), // Priority-ordered messages
    Stash,                       // Support for message stashing
}

pub struct ActorMailbox {
    queue: Arc<LockFreeQueue<Message>>,
    mailbox_type: MailboxType,
    stash: Option<Vec<Message>>,
}
```

#### **Message Priority**
```scheme
;; Define priority messages
(define-message urgent-message 
  #:priority 10
  #:data data)

(define-message normal-message
  #:priority 5  
  #:data data)

;; High priority messages are processed first
(send! actor (urgent-message "important data"))
(send! actor (normal-message "regular data"))
```

### **Actor Patterns**

#### **Request-Response Pattern**
```scheme
(define server-actor
  (actor
    [(initial-state (make-hash-table))]
    
    [(store key value sender)
     (hash-table-set! state key value)
     (send! sender '(ok stored))]
    
    [(retrieve key sender)
     (let ((value (hash-table-ref state key #f)))
       (if value
           (send! sender `(ok ,value))
           (send! sender '(error not-found))))]))

;; Usage
(define server (spawn server-actor))
(send! server `(store "key1" "value1" ,(self)))
(define response (receive))  ; Wait for response
```

#### **Worker Pool Pattern**
```scheme
(define worker-pool
  (actor
    [(initial-state (make-work-queue))]
    
    [(add-work work)
     (queue-push! state work)
     (notify-workers)]
    
    [(request-work sender)
     (let ((work (queue-pop! state)))
       (if work
           (send! sender `(work ,work))
           (send! sender '(no-work))))]))
```

## Parallel Evaluation

### **Multi-threaded Runtime** (`src/runtime/lambdust_runtime.rs`)

The parallel evaluation system enables automatic parallelization of Scheme expressions:

```rust
pub struct LambdustRuntime {
    evaluators: Vec<EvaluatorHandle>,
    work_scheduler: WorkStealingScheduler,
    effect_coordinator: EffectCoordinator,
    thread_pool: ThreadPool,
}

impl LambdustRuntime {
    /// Evaluates expressions in parallel
    pub async fn eval_parallel(&self, exprs: Vec<(Expr, Option<Span>)>) -> ParallelResult {
        let futures: Vec<_> = exprs.into_iter()
            .map(|(expr, span)| self.eval_async(expr, span))
            .collect();
        
        let results = futures::future::try_join_all(futures).await?;
        
        ParallelResult {
            results,
            metrics: self.collect_metrics(),
        }
    }
}
```

### **Work-Stealing Scheduler** (`src/concurrency/scheduler.rs`)

```rust
pub struct WorkStealingScheduler {
    queues: Vec<WorkQueue>,
    stealers: Vec<Stealer>,
    threads: Vec<JoinHandle<()>>,
    metrics: Arc<SchedulerMetrics>,
}

impl WorkStealingScheduler {
    /// Schedules work across available threads
    pub fn schedule<F>(&self, task: F) -> TaskHandle
    where
        F: FnOnce() -> Result<Value> + Send + 'static,
    {
        let task_id = TaskId::new();
        let work_item = WorkItem::new(task_id, task);
        
        // Find least loaded queue
        let queue_idx = self.select_queue();
        self.queues[queue_idx].push(work_item);
        
        TaskHandle::new(task_id)
    }
    
    fn select_queue(&self) -> usize {
        self.queues
            .iter()
            .enumerate()
            .min_by_key(|(_, queue)| queue.len())
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }
}
```

### **Parallel Programming Patterns**

#### **Data Parallelism**
```scheme
;; Parallel map - automatically distributed across threads
(define numbers (range 1 1000000))
(define squares (pmap square numbers))

;; Parallel reduce
(define sum (preduce + 0 numbers))

;; Parallel filter
(define evens (pfilter even? numbers))
```

#### **Task Parallelism**
```scheme
;; Parallel execution of independent tasks
(define result
  (plet ((a (expensive-computation-1))
         (b (expensive-computation-2))  
         (c (expensive-computation-3)))
    (combine-results a b c)))

;; Future-based parallelism
(define future-a (future (expensive-computation-1)))
(define future-b (future (expensive-computation-2)))
(define result (+ (force future-a) (force future-b)))
```

#### **Pipeline Parallelism**
```scheme
;; Multi-stage processing pipeline
(define pipeline
  (make-pipeline
    (stage "input" read-data)
    (stage "process" process-data)
    (stage "output" write-data)))

(pipeline-start pipeline input-source)
```

## Synchronization Primitives

### **Thread-Safe Primitives** (`src/concurrency/sync.rs`)

```rust
/// Mutex with built-in deadlock detection
pub struct Mutex<T> {
    inner: parking_lot::Mutex<T>,
    id: MutexId,
    owner_tracker: Arc<OwnerTracker>,
}

/// Read-write lock with reader preference
pub struct RwLock<T> {
    inner: parking_lot::RwLock<T>,
    metrics: Arc<RwLockMetrics>,
}

/// Condition variable for thread coordination  
pub struct CondVar {
    inner: parking_lot::Condvar,
    waiters: AtomicUsize,
}

/// Barrier for synchronizing multiple threads
pub struct Barrier {
    inner: std::sync::Barrier,
    completion_handler: Option<Box<dyn Fn() + Send + Sync>>,
}

/// Semaphore for resource counting
pub struct Semaphore {
    permits: AtomicIsize,
    waiters: Mutex<VecDeque<Waker>>,
}
```

### **Scheme-Level Synchronization**

#### **Mutexes and Condition Variables**
```scheme
;; Create synchronization primitives
(define mutex (make-mutex))
(define condition (make-condition-variable))
(define shared-data (make-vector 10 0))

;; Producer thread
(define producer
  (thread
    (lambda ()
      (do ((i 0 (+ i 1)))
          ((= i 100))
        (mutex-lock! mutex)
        (vector-set! shared-data 0 i)
        (condition-notify! condition)
        (mutex-unlock! mutex)
        (sleep 0.01)))))

;; Consumer thread
(define consumer
  (thread
    (lambda ()
      (mutex-lock! mutex)
      (let loop ()
        (condition-wait! condition mutex)
        (let ((value (vector-ref shared-data 0)))
          (display value)
          (newline)
          (if (< value 99)
              (loop)))))))
```

#### **Barriers**
```scheme
;; Coordinate multiple threads at checkpoints
(define barrier (make-barrier 4))

(define worker-thread
  (lambda (id)
    (thread
      (lambda ()
        ;; Phase 1 work
        (printf "Thread ~a: Phase 1 complete~n" id)
        (barrier-wait! barrier)
        
        ;; Phase 2 work  
        (printf "Thread ~a: Phase 2 complete~n" id)
        (barrier-wait! barrier)
        
        ;; Phase 3 work
        (printf "Thread ~a: All done~n" id)))))

;; Start worker threads
(map (lambda (i) (worker-thread i)) '(1 2 3 4))
```

#### **Semaphores**
```scheme
;; Resource pool management
(define resource-semaphore (make-semaphore 3)) ; 3 resources available

(define use-resource
  (lambda (id)
    (thread
      (lambda ()
        (semaphore-acquire! resource-semaphore)
        (printf "Thread ~a: Got resource~n" id)
        (sleep 1) ; Use resource
        (printf "Thread ~a: Releasing resource~n" id)
        (semaphore-release! resource-semaphore)))))
```

## Software Transactional Memory

### **STM Implementation** (`src/concurrency/stm.rs`)

Lambdust provides Software Transactional Memory for composable atomic operations:

```rust
/// Transactional reference
pub struct TVar<T> {
    value: Arc<RwLock<T>>,
    version: AtomicU64,
    watchers: Arc<RwLock<Vec<Weak<Transaction>>>>,
}

/// Transaction context
pub struct Transaction {
    id: TransactionId,
    read_set: HashMap<TVarId, u64>,
    write_set: HashMap<TVarId, Box<dyn Any + Send>>,
    status: AtomicTransactionStatus,
}

/// STM operations
impl<T: Clone + Send + 'static> TVar<T> {
    /// Read within a transaction
    pub fn read(&self, txn: &mut Transaction) -> T;
    
    /// Write within a transaction
    pub fn write(&self, txn: &mut Transaction, value: T);
}

/// Execute atomic transaction
pub fn atomic<F, R>(f: F) -> R
where
    F: Fn(&mut Transaction) -> R + Send + Sync,
    R: Send,
{
    let mut attempts = 0;
    loop {
        let mut txn = Transaction::new();
        let result = f(&mut txn);
        
        match txn.commit() {
            Ok(()) => return result,
            Err(ConflictError) => {
                attempts += 1;
                if attempts > MAX_RETRIES {
                    panic!("Transaction failed after {} attempts", MAX_RETRIES);
                }
                // Exponential backoff
                std::thread::sleep(Duration::from_millis(1 << attempts));
            }
        }
    }
}
```

### **STM Usage Patterns**

#### **Bank Account Transfer**
```scheme
;; Define transactional variables
(define account-a (make-tvar 1000))
(define account-b (make-tvar 500))

;; Atomic transfer function
(define (transfer from to amount)
  (atomic
    (lambda ()
      (let ((from-balance (tvar-read from))
            (to-balance (tvar-read to)))
        (if (>= from-balance amount)
            (begin
              (tvar-write! from (- from-balance amount))
              (tvar-write! to (+ to-balance amount))
              #t)
            #f)))))

;; Concurrent transfers
(parallel-do
  (transfer account-a account-b 200)
  (transfer account-b account-a 100)
  (transfer account-a account-b 50))
```

#### **Transactional Data Structures**
```scheme
;; Transactional stack
(define-record-type tstack
  (make-tstack-internal head)
  tstack?
  (head tstack-head))

(define (make-tstack)
  (make-tstack-internal (make-tvar '())))

(define (tstack-push! stack item)
  (atomic
    (lambda ()
      (let* ((head (tstack-head stack))
             (current (tvar-read head)))
        (tvar-write! head (cons item current))))))

(define (tstack-pop! stack)
  (atomic
    (lambda ()
      (let* ((head (tstack-head stack))
             (current (tvar-read head)))
        (if (null? current)
            #f
            (begin
              (tvar-write! head (cdr current))
              (car current)))))))
```

## Lock-Free Programming

### **Lock-Free Data Structures** (`src/concurrency/lockfree_queue.rs`)

```rust
/// Lock-free queue using Michael & Scott algorithm
pub struct LockFreeQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

struct Node<T> {
    data: Option<T>,
    next: AtomicPtr<Node<T>>,
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> Self {
        let dummy = Box::into_raw(Box::new(Node {
            data: None,
            next: AtomicPtr::new(std::ptr::null_mut()),
        }));
        
        LockFreeQueue {
            head: AtomicPtr::new(dummy),
            tail: AtomicPtr::new(dummy),
        }
    }
    
    pub fn enqueue(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data: Some(data),
            next: AtomicPtr::new(std::ptr::null_mut()),
        }));
        
        loop {
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*tail).next.load(Ordering::Acquire) };
            
            if tail == self.tail.load(Ordering::Acquire) {
                if next.is_null() {
                    if unsafe { (*tail).next.compare_exchange_weak(
                        next,
                        new_node,
                        Ordering::Release,
                        Ordering::Relaxed
                    ).is_ok() } {
                        break;
                    }
                } else {
                    self.tail.compare_exchange_weak(
                        tail,
                        next,
                        Ordering::Release,
                        Ordering::Relaxed
                    ).ok();
                }
            }
        }
        
        self.tail.compare_exchange(
            self.tail.load(Ordering::Acquire),
            new_node,
            Ordering::Release,
            Ordering::Relaxed
        ).ok();
    }
}
```

### **Atomic Operations** (`src/concurrency/atomic_primitives.rs`)

```rust
/// Atomic counter with CAS operations
pub struct AtomicCounter {
    value: AtomicI64,
    metrics: Arc<CounterMetrics>,
}

impl AtomicCounter {
    pub fn increment(&self) -> i64 {
        self.value.fetch_add(1, Ordering::AcqRel)
    }
    
    pub fn compare_and_swap(&self, current: i64, new: i64) -> Result<i64, i64> {
        self.value.compare_exchange(
            current,
            new,
            Ordering::AcqRel,
            Ordering::Acquire
        )
    }
    
    pub fn fetch_update<F>(&self, f: F) -> Result<i64, i64>
    where
        F: FnMut(i64) -> Option<i64>,
    {
        self.value.fetch_update(Ordering::AcqRel, Ordering::Acquire, f)
    }
}
```

### **Lock-Free Usage in Scheme**

```scheme
;; Atomic counter operations
(define counter (make-atomic-counter 0))

;; Multiple threads can safely increment
(parallel-map
  (lambda (_) (atomic-increment! counter))
  (range 1 1000))

;; Compare-and-swap operations
(define (atomic-max! counter new-value)
  (let loop ((current (atomic-load counter)))
    (if (> new-value current)
        (let ((result (atomic-compare-and-swap! counter current new-value)))
          (if (not (= result current))
              (loop result) ; Retry with new current value
              new-value))
        current)))
```

## Async/Await Integration

### **Async Runtime Integration** (`src/concurrency/futures.rs`)

```rust
/// Future-based computation
pub struct LambdustFuture<T> {
    inner: Pin<Box<dyn Future<Output = Result<T>> + Send>>,
    context: EffectContext,
}

/// Async evaluator
pub struct AsyncEvaluator {
    runtime: tokio::runtime::Runtime,
    evaluator: Arc<Mutex<Evaluator>>,
}

impl AsyncEvaluator {
    pub async fn eval_async(&self, expr: &Spanned<Expr>) -> Result<Value> {
        let evaluator = Arc::clone(&self.evaluator);
        let expr = expr.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut eval = evaluator.lock().unwrap();
            eval.eval(&expr)
        }).await?
    }
}
```

### **Async Scheme Programming**

```scheme
;; Async function definition
(define (async-fetch url)
  (async
    (lambda ()
      (let ((response (http-get url)))
        (json-parse (response-body response))))))

;; Await results
(define (fetch-multiple urls)
  (async
    (lambda ()
      (let ((futures (map async-fetch urls)))
        (await-all futures)))))

;; Async/await with error handling
(define (safe-async-operation)
  (async
    (lambda ()
      (try
        (let ((result (await (risky-async-operation))))
          (process-result result))
        (catch error
          (log-error error)
          #f)))))
```

## Best Practices

### **1. Choosing Concurrency Models**

#### **Use Actors When:**
- ✅ You need isolated state management
- ✅ Error isolation and fault tolerance are important
- ✅ You have long-running stateful components
- ✅ Message-driven architecture fits your domain

#### **Use Parallel Evaluation When:**
- ✅ You have CPU-intensive computations
- ✅ Operations are independent and parallelizable
- ✅ You want automatic load balancing
- ✅ Data parallelism is applicable

#### **Use STM When:**
- ✅ You need composable atomic operations
- ✅ Transactions involve multiple memory locations
- ✅ Retry logic is acceptable for your use case
- ✅ Lock-free coordination is desired

### **2. Performance Considerations**

#### **Actor Performance Tips**
```scheme
;; ✅ Good: Batch related messages
(send! actor (batch-update updates))

;; ❌ Bad: Many small messages
(map (lambda (update) (send! actor update)) updates)

;; ✅ Good: Use appropriate mailbox types
(define high-throughput-actor
  (actor #:mailbox 'bounded-1000
    ...))
```

#### **Parallel Evaluation Tips**
```scheme
;; ✅ Good: Sufficient work per task
(pmap expensive-computation large-dataset)

;; ❌ Bad: Too fine-grained parallelism
(pmap (lambda (x) (+ x 1)) tiny-list)

;; ✅ Good: Use chunking for small items
(pmap (lambda (chunk) (map simple-op chunk))
      (chunk-list items 100))
```

### **3. Avoiding Common Pitfalls**

#### **Race Conditions**
```scheme
;; ❌ Bad: Race condition
(define counter 0)
(parallel-do
  (set! counter (+ counter 1))  ; Not atomic
  (set! counter (+ counter 1)))

;; ✅ Good: Use atomic operations
(define counter (make-atomic-counter 0))
(parallel-do
  (atomic-increment! counter)
  (atomic-increment! counter))
```

#### **Deadlocks**
```scheme
;; ❌ Bad: Potential deadlock
(define (transfer-with-locks a b amount)
  (mutex-lock! (account-mutex a))
  (mutex-lock! (account-mutex b))  ; Lock order may vary
  (transfer-amount a b amount)
  (mutex-unlock! (account-mutex b))
  (mutex-unlock! (account-mutex a)))

;; ✅ Good: Consistent lock ordering
(define (transfer-safe a b amount)
  (let ((first (if (< (account-id a) (account-id b)) a b))
        (second (if (< (account-id a) (account-id b)) b a)))
    (mutex-lock! (account-mutex first))
    (mutex-lock! (account-mutex second))
    (transfer-amount a b amount)
    (mutex-unlock! (account-mutex second))
    (mutex-unlock! (account-mutex first))))
```

#### **Memory Leaks in Concurrent Code**
```scheme
;; ❌ Bad: Unbounded mailbox growth
(define busy-actor
  (actor
    (let loop ()
      (receive msg
        ; Process message very slowly
        (expensive-operation msg)
        (loop)))))

;; ✅ Good: Bounded mailbox with backpressure
(define controlled-actor
  (actor #:mailbox 'bounded-100
         #:backpressure 'block
    (let loop ()
      (receive msg
        (expensive-operation msg)
        (loop)))))
```

---

This concurrency guide provides a comprehensive foundation for building concurrent Scheme applications with Lambdust. The combination of actors, parallel evaluation, STM, and lock-free programming enables building robust, high-performance concurrent systems while maintaining the elegant simplicity of Scheme.