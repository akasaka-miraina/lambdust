//! Comprehensive tests for the concurrency system.

use super::*;
use crate::eval::Value;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_future_resolved() {
    let value = Value::integer(42);
    let future = futures::Future::resolved(value.clone());
    
    assert!(future.is_completed());
    assert!(future.is_resolved());
    assert!(!future.is_rejected());
    
    let result = future.await_result().await.unwrap();
    assert_eq!(result.as_integer().unwrap(), 42);
}

#[tokio::test]
async fn test_future_rejected() {
    let error = crate::diagnostics::Error::runtime_error("test error".to_string(), None);
    let future = futures::Future::rejected(error);
    
    assert!(future.is_completed());
    assert!(!future.is_resolved());
    assert!(future.is_rejected());
    
    let result = future.await_result().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_promise() {
    let promise = futures::Promise::new();
    let future = promise.future();
    
    assert!(!future.is_completed());
    assert!(promise.is_pending());
    
    let value = Value::integer(123);
    promise.resolve(value.clone()).unwrap();
    
    let result = future.await_result().await.unwrap();
    assert_eq!(result.as_integer().unwrap(), 123);
}

#[tokio::test]
async fn test_future_map() {
    let value = Value::integer(10);
    let future = futures::Future::resolved(value);
    
    let mapped_future = future.map(|v| {
        let n = v.as_integer().unwrap();
        Ok(Value::integer(n * 2))
    });
    
    let result = mapped_future.await_result().await.unwrap();
    assert_eq!(result.as_integer().unwrap(), 20);
}

#[tokio::test]
async fn test_future_delay() {
    let value = Value::integer(456);
    let duration = Duration::from_millis(100);
    let future = futures::FutureOps::delay_value(duration, value.clone());
    
    let start = std::time::Instant::now();
    let result = future.await_result().await.unwrap();
    let elapsed = start.elapsed();
    
    assert_eq!(result.as_integer().unwrap(), 456);
    assert!(elapsed >= duration);
}

#[tokio::test]
async fn test_future_all() {
    let futures = vec![
        futures::Future::resolved(Value::integer(1)),
        futures::Future::resolved(Value::integer(2)),
        futures::Future::resolved(Value::integer(3)),
    ];
    
    let all_future = futures::FutureOps::all(futures);
    let result = all_future.await_result().await.unwrap();
    
    // Result should be a list of values
    assert!(matches!(result, Value::Pair(_, _)));
}

#[tokio::test]
async fn test_future_race() {
    let futures = vec![
        futures::FutureOps::delay_value(Duration::from_millis(200), Value::integer(1)),
        futures::FutureOps::delay_value(Duration::from_millis(100), Value::integer(2)),
        futures::FutureOps::delay_value(Duration::from_millis(300), Value::integer(3)),
    ];
    
    let race_future = futures::FutureOps::race(futures);
    let result = race_future.await_result().await.unwrap();
    
    // Should return the fastest (value 2)
    assert_eq!(result.as_integer().unwrap(), 2);
}

#[tokio::test]
async fn test_channel_bounded() {
    let channel = channels::Channel::bounded(3).unwrap();
    let sender = channel.sender();
    let receiver = channel.receiver();
    
    // Send some values
    sender.send(Value::integer(1)).await.unwrap();
    sender.send(Value::integer(2)).await.unwrap();
    sender.send(Value::integer(3)).await.unwrap();
    
    // Receive values
    let mut rx = receiver.lock().await;
    let val1 = rx.recv().await.unwrap();
    let val2 = rx.recv().await.unwrap();
    let val3 = rx.recv().await.unwrap();
    
    assert_eq!(val1.as_integer().unwrap(), 1);
    assert_eq!(val2.as_integer().unwrap(), 2);
    assert_eq!(val3.as_integer().unwrap(), 3);
}

#[tokio::test]
async fn test_channel_unbounded() {
    let channel = channels::Channel::unbounded().unwrap();
    let sender = channel.sender();
    let receiver = channel.receiver();
    
    // Send many values
    for i in 0..1000 {
        sender.send(Value::integer(i)).await.unwrap();
    }
    
    // Receive values
    let mut rx = receiver.lock().await;
    for i in 0..1000 {
        let val = rx.recv().await.unwrap();
        assert_eq!(val.as_integer().unwrap(), i);
    }
}

#[tokio::test]
async fn test_channel_try_send_recv() {
    let channel = channels::Channel::bounded(1).unwrap();
    let sender = channel.sender();
    let receiver = channel.receiver();
    
    // Send one value
    sender.try_send(Value::integer(42)).unwrap();
    
    // Channel should be full now
    let result = sender.try_send(Value::integer(43));
    assert!(result.is_err());
    
    // Receive the value
    let mut rx = receiver.lock().await;
    let val = rx.try_recv().unwrap();
    assert_eq!(val.as_integer().unwrap(), 42);
    
    // Channel should be empty now
    let result = rx.try_recv();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_select_operation() {
    let ch1 = channels::Channel::bounded(1).unwrap();
    let ch2 = channels::Channel::bounded(1).unwrap();
    
    // Send to ch2 after a delay
    let sender2 = ch2.sender();
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        sender2.send(Value::integer(200)).await.unwrap();
    });
    
    let select = channels::Select::new()
        .recv(1, ch1.receiver())
        .recv(2, ch2.receiver())
        .timeout(3, Duration::from_millis(200));
    
    let result = select.execute().await.unwrap();
    
    // Should receive from ch2 (id=2)
    if let Value::Pair(op, rest) = result {
        assert_eq!(op.to_string(), "recv");
        if let Value::Pair(id, _) = rest.as_ref() {
            assert_eq!(id.as_integer().unwrap(), 2);
        }
    }
}

#[tokio::test]
async fn test_parallel_map() {
    let config = parallel::ParallelConfig::default();
    let ops = parallel::ParallelOps::new(config);
    
    let values = vec![
        Value::integer(1),
        Value::integer(2),
        Value::integer(3),
        Value::integer(4),
        Value::integer(5),
    ];
    
    let future = ops.par_map(values, |v| {
        let n = v.as_integer().unwrap();
        Ok(Value::integer(n * 2))
    });
    
    let result = future.await_result().await.unwrap();
    
    // Result should be a list of doubled values
    let mut current = &result;
    let mut expected = vec![2, 4, 6, 8, 10];
    let mut i = 0;
    
    loop {
        match current {
            Value::Pair(car, cdr) => {
                assert_eq!(car.as_integer().unwrap(), expected[i]);
                i += 1;
                current = cdr;
            }
            Value::Nil => break,
            _ => panic!("Expected proper list"),
        }
    }
    
    assert_eq!(i, expected.len());
}

#[tokio::test]
async fn test_parallel_filter() {
    let config = parallel::ParallelConfig::default();
    let ops = parallel::ParallelOps::new(config);
    
    let values = vec![
        Value::integer(1),
        Value::integer(2),
        Value::integer(3),
        Value::integer(4),
        Value::integer(5),
    ];
    
    let future = ops.par_filter(values, |v| {
        let n = v.as_integer().unwrap();
        Ok(n % 2 == 0) // Keep even numbers
    });
    
    let result = future.await_result().await.unwrap();
    
    // Result should contain only even numbers (2, 4)
    let mut current = &result;
    let mut count = 0;
    
    loop {
        match current {
            Value::Pair(car, cdr) => {
                let n = car.as_integer().unwrap();
                assert!(n % 2 == 0);
                count += 1;
                current = cdr;
            }
            Value::Nil => break,
            _ => panic!("Expected proper list"),
        }
    }
    
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_parallel_reduce() {
    let config = parallel::ParallelConfig::default();
    let ops = parallel::ParallelOps::new(config);
    
    let values = vec![
        Value::integer(1),
        Value::integer(2),
        Value::integer(3),
        Value::integer(4),
        Value::integer(5),
    ];
    
    let future = ops.par_reduce(values, Value::integer(0), |a, b| {
        let x = a.as_integer().unwrap();
        let y = b.as_integer().unwrap();
        Ok(Value::integer(x + y))
    });
    
    let result = future.await_result().await.unwrap();
    assert_eq!(result.as_integer().unwrap(), 15); // 1+2+3+4+5
}

#[tokio::test]
async fn test_mutex() {
    let mutex = sync::Mutex::new(Value::integer(0));
    
    let mut handles = Vec::new();
    
    for _ in 0..10 {
        let mutex_clone = mutex.clone();
        let handle = tokio::spawn(async move {
            let mut guard = mutex_clone.lock().await;
            let current = guard.get().as_integer().unwrap();
            guard.set(Value::integer(current + 1));
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let guard = mutex.lock().await;
    assert_eq!(guard.get().as_integer().unwrap(), 10);
}

#[tokio::test]
async fn test_rwlock() {
    let rwlock = sync::RwLock::new(Value::integer(42));
    
    // Multiple readers
    let mut read_handles = Vec::new();
    for _ in 0..5 {
        let rwlock_clone = rwlock.clone();
        let handle = tokio::spawn(async move {
            let guard = rwlock_clone.read().await;
            assert_eq!(guard.get().as_integer().unwrap(), 42);
        });
        read_handles.push(handle);
    }
    
    for handle in read_handles {
        handle.await.unwrap();
    }
    
    // Single writer
    {
        let mut guard = rwlock.write().await;
        guard.set(Value::integer(100));
    }
    
    // Verify write
    let guard = rwlock.read().await;
    assert_eq!(guard.get().as_integer().unwrap(), 100);
}

#[tokio::test]
async fn test_semaphore() {
    let semaphore = sync::SemaphoreSync::new(3);
    
    // Acquire all permits
    let permit1 = semaphore.acquire().await.unwrap();
    let permit2 = semaphore.acquire().await.unwrap();
    let permit3 = semaphore.acquire().await.unwrap();
    
    assert_eq!(semaphore.available_permits(), 0);
    
    // Try to acquire one more (should fail)
    let result = semaphore.try_acquire();
    assert!(result.is_err());
    
    // Release permits by dropping them
    drop(permit1);
    drop(permit2);
    
    assert_eq!(semaphore.available_permits(), 2);
    
    // Should be able to acquire again
    let _permit4 = semaphore.try_acquire().unwrap();
    assert_eq!(semaphore.available_permits(), 1);
}

#[tokio::test]
async fn test_condition_variable() {
    let condvar = sync::CondVar::new();
    let condvar_clone = condvar.clone();
    
    let handle = tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        condvar_clone.notify_one();
    });
    
    // Wait for notification
    condvar.wait().await;
    
    handle.await.unwrap();
}

#[tokio::test]
async fn test_barrier() {
    let barrier = sync::Barrier::new(3);
    let mut handles = Vec::new();
    
    for i in 0..3 {
        let barrier_clone = barrier.clone();
        let handle = tokio::spawn(async move {
            sleep(Duration::from_millis(i * 50)).await;
            let result = barrier_clone.wait().await;
            if i == 2 {
                assert!(result.is_leader);
            } else {
                assert!(!result.is_leader);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_atomic_counter() {
    let counter = sync::AtomicCounter::new(0);
    let mut handles = Vec::new();
    
    for _ in 0..100 {
        let counter_clone = counter.clone();
        let handle = tokio::spawn(async move {
            counter_clone.increment();
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    assert_eq!(counter.get(), 100);
}

#[tokio::test]
async fn test_atomic_flag() {
    let flag = sync::AtomicFlag::new(false);
    
    assert!(!flag.get());
    
    let old_value = flag.set_true();
    assert!(!old_value);
    assert!(flag.get());
    
    let old_value = flag.set_false();
    assert!(old_value);
    assert!(!flag.get());
}

#[tokio::test]
async fn test_lock_free_queue() {
    let queue = sync::LockFreeQueue::new();
    
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
    
    queue.push(Value::integer(1));
    queue.push(Value::integer(2));
    queue.push(Value::integer(3));
    
    assert!(!queue.is_empty());
    assert_eq!(queue.len(), 3);
    
    let val1 = queue.pop().unwrap();
    let val2 = queue.pop().unwrap();
    let val3 = queue.pop().unwrap();
    let val4 = queue.pop();
    
    assert_eq!(val1.as_integer().unwrap(), 1);
    assert_eq!(val2.as_integer().unwrap(), 2);
    assert_eq!(val3.as_integer().unwrap(), 3);
    assert!(val4.is_none());
    
    assert!(queue.is_empty());
}

#[tokio::test]
async fn test_bounded_lock_free_queue() {
    let queue = sync::BoundedLockFreeQueue::new(2);
    
    assert_eq!(queue.capacity(), 2);
    assert!(queue.is_empty());
    assert!(!queue.is_full());
    
    queue.push(Value::integer(1)).unwrap();
    queue.push(Value::integer(2)).unwrap();
    
    assert!(queue.is_full());
    assert_eq!(queue.len(), 2);
    
    // Should fail to push another item
    let result = queue.push(Value::integer(3));
    assert!(result.is_err());
    
    let val1 = queue.pop().unwrap();
    assert_eq!(val1.as_integer().unwrap(), 1);
    
    assert!(!queue.is_full());
    assert_eq!(queue.len(), 1);
    
    // Should be able to push again
    queue.push(Value::integer(3)).unwrap();
    assert_eq!(queue.len(), 2);
}

// Integration tests
#[tokio::test]
async fn test_producer_consumer_pattern() {
    let channel = channels::Channel::bounded(10).unwrap();
    let sender = channel.sender();
    let receiver = channel.receiver();
    
    // Producer
    let producer_sender = sender.clone();
    let producer = tokio::spawn(async move {
        for i in 0..100 {
            producer_sender.send(Value::integer(i)).await.unwrap();
        }
    });
    
    // Consumer
    let consumer = tokio::spawn(async move {
        let mut sum = 0i64;
        let mut rx = receiver.lock().await;
        for _ in 0..100 {
            let val = rx.recv().await.unwrap();
            sum += val.as_integer().unwrap();
        }
        sum
    });
    
    let (_, sum) = tokio::join!(producer, consumer);
    let sum = sum.unwrap();
    
    // Sum of 0..99 = 99 * 100 / 2 = 4950
    assert_eq!(sum, 4950);
}

#[tokio::test]
async fn test_work_distribution() {
    let config = parallel::ParallelConfig {
        num_threads: Some(4),
        chunk_size: 25,
        work_stealing: true,
        cpu_affinity: None,
    };
    
    let ops = parallel::ParallelOps::new(config);
    
    // Create a large dataset
    let values: Vec<Value> = (0..1000).map(Value::integer).collect();
    
    let future = ops.par_map(values, |v| {
        let n = v.as_integer().unwrap();
        // Simulate some work
        std::thread::sleep(Duration::from_micros(10));
        Ok(Value::integer(n * n))
    });
    
    let start = std::time::Instant::now();
    let result = future.await_result().await.unwrap();
    let elapsed = start.elapsed();
    
    // Verify result length and some values
    let mut count = 0;
    let mut current = &result;
    loop {
        match current {
            Value::Pair(car, cdr) => {
                let expected = count * count;
                assert_eq!(car.as_integer().unwrap(), expected);
                count += 1;
                current = cdr;
            }
            Value::Nil => break,
            _ => panic!("Expected proper list"),
        }
    }
    
    assert_eq!(count, 1000);
    
    // Should be faster than sequential execution
    println!("Parallel execution took: {:?}", elapsed);
    assert!(elapsed < Duration::from_millis(500)); // Should be much faster than 10ms * 1000
}

// Helper functions for tests

// Benchmark tests (would normally be in benches/)
#[tokio::test]
async fn bench_channel_throughput() {
    let channel = channels::Channel::unbounded().unwrap();
    let sender = channel.sender();
    let receiver = channel.receiver();
    
    let num_messages = 100000;
    
    let producer = tokio::spawn(async move {
        let start = std::time::Instant::now();
        for i in 0..num_messages {
            sender.send(Value::integer(i)).await.unwrap();
        }
        start.elapsed()
    });
    
    let consumer = tokio::spawn(async move {
        let start = std::time::Instant::now();
        let mut rx = receiver.lock().await;
        for _ in 0..num_messages {
            let _val = rx.recv().await.unwrap();
        }
        start.elapsed()
    });
    
    let (producer_time, consumer_time) = tokio::join!(producer, consumer);
    let producer_time = producer_time.unwrap();
    let consumer_time = consumer_time.unwrap();
    
    println!("Producer time: {:?}", producer_time);
    println!("Consumer time: {:?}", consumer_time);
    println!("Messages per second: {}", num_messages as f64 / consumer_time.as_secs_f64());
}

#[tokio::test]
async fn bench_future_creation() {
    let num_futures = 10000;
    
    let start = std::time::Instant::now();
    let mut futures = Vec::new();
    
    for i in 0..num_futures {
        let future = futures::Future::resolved(Value::integer(i));
        futures.push(future);
    }
    
    let creation_time = start.elapsed();
    
    let start = std::time::Instant::now();
    for future in futures {
        let _result = future.await_result().await.unwrap();
    }
    let await_time = start.elapsed();
    
    println!("Future creation time: {:?}", creation_time);
    println!("Future await time: {:?}", await_time);
    println!("Futures per second (creation): {}", num_futures as f64 / creation_time.as_secs_f64());
    println!("Futures per second (await): {}", num_futures as f64 / await_time.as_secs_f64());
}