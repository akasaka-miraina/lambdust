//! Advanced concurrency tests for Lambdust
//! 
//! This module tests the advanced concurrency features including:
//! - Actor system with message passing and supervision
//! - Futures, promises, and async/await patterns
//! - STM (Software Transactional Memory) operations
//! - Work-stealing scheduler and parallel operations

use lambdust::{
    concurrency::*,
    eval::Value,
    ast::Literal,
    diagnostics::Result,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_future_creation_and_completion() {
    // Test future creation
    let (future, completer) = Future::new();
    
    assert!(!future.is_completed());
    
    // Complete the future
    let value = Value::Literal(Literal::number(42.0));
    completer.complete(value.clone());
    
    // Wait for completion
    let result = future.await;
    assert!(result.is_ok());
    
    if let Ok(Value::Literal(Literal::Number(n))) = result {
        assert_eq!(n, 42.0);
    }
}

#[tokio::test]
async fn test_promise_lazy_evaluation() {
    // Create a promise with lazy computation
    let promise = Promise::new(|| {
        Value::Literal(Literal::number(100.0))
    });
    
    // Force the promise
    let result = promise.force().await;
    
    if let Value::Literal(Literal::Number(n)) = result {
        assert_eq!(n, 100.0);
    }
    
    // Subsequent forces should return the same cached result
    let result2 = promise.force().await;
    assert_eq!(result, result2);
}

#[tokio::test]
async fn test_channel_communication() {
    // Create a typed channel
    let (sender, mut receiver) = Channel::<Value>::unbounded();
    
    // Send values
    let values = vec![
        Value::Literal(Literal::number(1.0)),
        Value::Literal(Literal::number(2.0)),
        Value::Literal(Literal::number(3.0)),
    ];
    
    for value in values.iter() {
        sender.send(value.clone()).unwrap();
    }
    
    // Receive values
    let mut received = Vec::new();
    for _ in 0..3 {
        if let Some(value) = receiver.recv().await {
            received.push(value);
        }
    }
    
    assert_eq!(received.len(), 3);
    for (i, value) in received.iter().enumerate() {
        if let Value::Literal(Literal::Number(n)) = value {
            assert_eq!(*n, (i + 1) as f64);
        }
    }
}

#[tokio::test]
async fn test_select_operations() {
    let (sender1, mut receiver1) = Channel::<Value>::unbounded();
    let (sender2, mut receiver2) = Channel::<Value>::unbounded();
    
    // Send to first channel
    sender1.send(Value::Literal(Literal::string("first".to_string()))).unwrap();
    
    // Select between channels
    let selected = select! {
        value = receiver1.recv() => ("first", value),
        value = receiver2.recv() => ("second", value),
    };
    
    assert_eq!(selected.0, "first");
    if let Some(Value::Literal(Literal::String(s))) = selected.1 {
        assert_eq!(s, "first");
    }
}

#[tokio::test]
async fn test_parallel_computation() {
    // Test parallel map operation
    let values: Vec<Value> = (1..=10)
        .map(|i| Value::Literal(Literal::number(i as f64)))
        .collect();
    
    let square = |x: &Value| -> Value {
        if let Value::Literal(Literal::Number(n)) = x {
            Value::Literal(Literal::number(n * n))
        } else {
            x.clone()
        }
    };
    
    let results = ParallelIterator::from_vec(values)
        .map(square)
        .collect()
        .await;
    
    assert_eq!(results.len(), 10);
    
    // Check that values were squared
    for (i, result) in results.iter().enumerate() {
        if let Value::Literal(Literal::Number(n)) = result {
            let expected = ((i + 1) * (i + 1)) as f64;
            assert_eq!(*n, expected);
        }
    }
}

#[tokio::test]
async fn test_actor_system() {
    initialize().unwrap();
    
    // Define a simple actor behavior
    #[derive(Debug, Clone)]
    struct CounterActor {
        count: Arc<std::sync::Mutex<i64>>,
    }
    
    impl Actor for CounterActor {
        type Message = CounterMessage;
        
        fn receive(&mut self, message: Self::Message, context: &ActorContext) -> Result<()> {
            match message {
                CounterMessage::Increment => {
                    if let Ok(mut count) = self.count.lock() {
                        *count += 1;
                    }
                }
                CounterMessage::GetCount(reply_to) => {
                    if let Ok(count) = self.count.lock() {
                        let _ = reply_to.send(CounterMessage::Count(*count));
                    }
                }
                _ => {}
            }
            Ok(())
        }
    }
    
    #[derive(Debug, Clone)]
    enum CounterMessage {
        Increment,
        GetCount(tokio::sync::oneshot::Sender<CounterMessage>),
        Count(i64),
    }
    
    // Create and start the actor
    let count = Arc::new(std::sync::Mutex::new(0i64));
    let actor = CounterActor { count: count.clone() };
    let actor_ref = ActorSystem::spawn(actor).await.unwrap();
    
    // Send increment messages
    for _ in 0..5 {
        actor_ref.send(CounterMessage::Increment).await.unwrap();
    }
    
    // Get the count
    let (sender, receiver) = tokio::sync::oneshot::channel();
    actor_ref.send(CounterMessage::GetCount(sender)).await.unwrap();
    
    let response = timeout(Duration::from_secs(1), receiver).await.unwrap().unwrap();
    if let CounterMessage::Count(n) = response {
        assert_eq!(n, 5);
    }
    
    shutdown().await.unwrap();
}

#[tokio::test]
async fn test_actor_supervision() {
    initialize().unwrap();
    
    // Create a supervisor with restart strategy
    let supervisor = Supervisor::new(SupervisionStrategy::RestartOne);
    
    // Define a failing actor
    #[derive(Debug, Clone)]
    struct FailingActor {
        should_fail: bool,
    }
    
    impl Actor for FailingActor {
        type Message = String;
        
        fn receive(&mut self, message: Self::Message, _context: &ActorContext) -> Result<()> {
            if self.should_fail && message == "fail" {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Actor failure".to_string(),
                    None,
                ));
            }
            Ok(())
        }
    }
    
    let failing_actor = FailingActor { should_fail: true };
    let actor_ref = supervisor.supervise(failing_actor).await.unwrap();
    
    // Send a failing message - actor should be restarted
    let result = actor_ref.send("fail".to_string()).await;
    // The actor should handle the failure through supervision
    
    shutdown().await.unwrap();
}

#[test]
fn test_stm_transactions() {
    // Test Software Transactional Memory operations
    let var1 = STMVar::new(Value::Literal(Literal::number(10.0)));
    let var2 = STMVar::new(Value::Literal(Literal::number(20.0)));
    
    // Atomic transaction that transfers value between variables
    let transaction = || {
        let val1 = var1.read();
        let val2 = var2.read();
        
        if let (Value::Literal(Literal::Number(n1)), Value::Literal(Literal::Number(n2))) = (&val1, &val2) {
            if *n1 >= 5.0 {
                var1.write(Value::Literal(Literal::number(n1 - 5.0)));
                var2.write(Value::Literal(Literal::number(n2 + 5.0)));
                Ok(())
            } else {
                Err(STMError::Retry)
            }
        } else {
            Err(STMError::InvalidType)
        }
    };
    
    // Execute transaction atomically
    let result = STM::atomically(transaction);
    assert!(result.is_ok());
    
    // Verify final values
    let final1 = var1.read();
    let final2 = var2.read();
    
    if let (Value::Literal(Literal::Number(n1)), Value::Literal(Literal::Number(n2))) = (&final1, &final2) {
        assert_eq!(*n1, 5.0);
        assert_eq!(*n2, 25.0);
    }
}

#[test]
fn test_work_stealing_scheduler() {
    // Initialize the work-stealing scheduler
    let scheduler = WorkStealingScheduler::new(4); // 4 worker threads
    
    // Create tasks with different priorities
    let high_priority_task = Task::new(
        Priority::High,
        Box::new(|| Value::Literal(Literal::number(1.0))),
    );
    
    let normal_priority_task = Task::new(
        Priority::Normal,
        Box::new(|| Value::Literal(Literal::number(2.0))),
    );
    
    let low_priority_task = Task::new(
        Priority::Low,
        Box::new(|| Value::Literal(Literal::number(3.0))),
    );
    
    // Submit tasks
    let handle1 = scheduler.submit(high_priority_task).unwrap();
    let handle2 = scheduler.submit(normal_priority_task).unwrap();
    let handle3 = scheduler.submit(low_priority_task).unwrap();
    
    // Wait for completion
    let result1 = handle1.wait().unwrap();
    let result2 = handle2.wait().unwrap();
    let result3 = handle3.wait().unwrap();
    
    // Verify results
    assert!(matches!(result1, Value::Literal(Literal::Number(n)) if n == 1.0));
    assert!(matches!(result2, Value::Literal(Literal::Number(n)) if n == 2.0));
    assert!(matches!(result3, Value::Literal(Literal::Number(n)) if n == 3.0));
    
    // Check scheduler statistics
    let stats = scheduler.statistics();
    assert!(stats.completed_tasks >= 3);
}

#[tokio::test]
async fn test_async_streams() {
    use futures::StreamExt;
    
    // Create an async stream of values
    let stream = AsyncStream::new(|| async {
        for i in 1..=5 {
            yield Value::Literal(Literal::number(i as f64));
        }
    });
    
    let mut collected = Vec::new();
    let mut stream = stream.enumerate();
    
    while let Some((index, value)) = stream.next().await {
        collected.push((index, value));
    }
    
    assert_eq!(collected.len(), 5);
    
    for (i, (index, value)) in collected.iter().enumerate() {
        assert_eq!(*index, i);
        if let Value::Literal(Literal::Number(n)) = value {
            assert_eq!(*n, (i + 1) as f64);
        }
    }
}

#[test]
fn test_lock_free_data_structures() {
    // Test lock-free stack
    let stack = LockFreeStack::new();
    
    // Push values from multiple threads
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let stack_clone = stack.clone();
            std::thread::spawn(move || {
                stack_clone.push(Value::Literal(Literal::number(i as f64)));
            })
        })
        .collect();
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Pop all values
    let mut popped = Vec::new();
    while let Some(value) = stack.pop() {
        popped.push(value);
    }
    
    assert_eq!(popped.len(), 10);
    
    // Test lock-free queue
    let queue = LockFreeQueue::new();
    
    // Enqueue values
    for i in 1..=5 {
        queue.enqueue(Value::Literal(Literal::number(i as f64)));
    }
    
    // Dequeue and verify order
    for i in 1..=5 {
        if let Some(Value::Literal(Literal::Number(n))) = queue.dequeue() {
            assert_eq!(n, i as f64);
        }
    }
}

#[test]
fn test_thread_pool_execution() {
    let pool = ThreadPool::new(4).unwrap();
    
    // Submit CPU-bound tasks
    let futures: Vec<_> = (1..=10)
        .map(|i| {
            pool.spawn(move || {
                // Simulate work
                std::thread::sleep(Duration::from_millis(10));
                Value::Literal(Literal::number(i as f64 * i as f64))
            })
        })
        .collect();
    
    // Collect results
    let results: Vec<_> = futures.into_iter().map(|f| f.join().unwrap()).collect();
    
    assert_eq!(results.len(), 10);
    
    // Verify computations
    for (i, result) in results.iter().enumerate() {
        if let Value::Literal(Literal::Number(n)) = result {
            let expected = ((i + 1) * (i + 1)) as f64;
            assert_eq!(*n, expected);
        }
    }
}

#[tokio::test]
async fn test_distributed_messaging() {
    // Test basic distributed node communication
    let node1 = DistributedNode::new("node1".to_string(), "127.0.0.1:8001".to_string()).await.unwrap();
    let node2 = DistributedNode::new("node2".to_string(), "127.0.0.1:8002".to_string()).await.unwrap();
    
    // Connect nodes
    node1.connect_to("127.0.0.1:8002").await.unwrap();
    
    // Send message from node1 to node2
    let message = DistributedMessage::new(
        "node1".to_string(),
        "node2".to_string(),
        Value::Literal(Literal::string("Hello distributed world!".to_string())),
    );
    
    node1.send_message(message.clone()).await.unwrap();
    
    // Receive message on node2
    let received = timeout(Duration::from_secs(1), node2.receive_message()).await.unwrap().unwrap();
    
    assert_eq!(received.from, "node1");
    assert_eq!(received.to, "node2");
    
    if let Value::Literal(Literal::String(s)) = received.payload {
        assert_eq!(s, "Hello distributed world!");
    }
    
    // Cleanup
    node1.shutdown().await.unwrap();
    node2.shutdown().await.unwrap();
}

#[test]
fn test_memory_consistency() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let counter = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(std::sync::Barrier::new(4));
    
    // Test memory ordering with multiple threads
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let counter = counter.clone();
            let barrier = barrier.clone();
            
            std::thread::spawn(move || {
                // Synchronize thread start
                barrier.wait();
                
                // Increment counter with different memory orderings
                for _ in 0..1000 {
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify final count
    assert_eq!(counter.load(Ordering::SeqCst), 4000);
}

// Integration test combining multiple concurrency features
#[tokio::test]
async fn test_advanced_concurrency_integration() {
    initialize().unwrap();
    
    // Create a complex concurrent computation using multiple primitives
    let (result_sender, mut result_receiver) = Channel::unbounded();
    
    // Spawn multiple actors that communicate through channels and futures
    let mut actor_handles = Vec::new();
    
    for i in 0..3 {
        let sender = result_sender.clone();
        
        let handle = tokio::spawn(async move {
            // Create a future computation
            let (future, completer) = Future::new();
            
            // Complete the future after some work
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                completer.complete(Value::Literal(Literal::number(i as f64 * 10.0)));
            });
            
            // Wait for the future and send result
            if let Ok(result) = future.await {
                let _ = sender.send(result);
            }
        });
        
        actor_handles.push(handle);
    }
    
    // Collect results
    let mut results = Vec::new();
    for _ in 0..3 {
        if let Some(result) = result_receiver.recv().await {
            results.push(result);
        }
    }
    
    // Wait for all actors to complete
    for handle in actor_handles {
        handle.await.unwrap();
    }
    
    assert_eq!(results.len(), 3);
    
    // Sort results for consistent testing
    results.sort_by(|a, b| {
        match (a, b) {
            (Value::Literal(Literal::Number(n1)), Value::Literal(Literal::Number(n2))) => {
                n1.partial_cmp(n2).unwrap()
            }
            _ => std::cmp::Ordering::Equal,
        }
    });
    
    // Verify results are [0.0, 10.0, 20.0]
    for (i, result) in results.iter().enumerate() {
        if let Value::Literal(Literal::Number(n)) = result {
            assert_eq!(*n, i as f64 * 10.0);
        }
    }
    
    shutdown().await.unwrap();
}