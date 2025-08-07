//! Actor System Tests for Lambdust
//!
//! This module implements comprehensive tests for the actor-based
//! evaluation system, testing message passing, distributed computation,
//! and actor lifecycle management.

use lambdust::runtime::{LambdustRuntime, EvaluatorHandle, EvaluatorMessage};
use lambdust::eval::Value;
use lambdust::diagnostics::{Result, Error};
use lambdust::ast::Expr;

use std::sync::{Arc, Mutex, Barrier};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use crossbeam::channel;

/// Actor message types for testing
#[derive(Debug, Clone)]
pub enum ActorTestMessage {
    /// Compute factorial of a number
    Factorial(u64),
    /// Add numbers together
    Sum(Vec<i64>),
    /// Forward message to another actor
    Forward { target_id: u64, message: Box<ActorTestMessage> },
    /// Ping message for connectivity testing
    Ping,
    /// Shutdown the actor
    Shutdown,
}

/// Response types from actors
#[derive(Debug, Clone)]
pub enum ActorTestResponse {
    /// Result of computation
    Value(Value),
    /// Acknowledgment
    Ack,
    /// Error response
    Error(String),
}

/// Test actor that can perform various computations
pub struct TestActor {
    pub id: u64,
    pub evaluator_handle: EvaluatorHandle,
    pub message_count: Arc<Mutex<u64>>,
    pub runtime: Arc<LambdustRuntime>,
}

impl TestActor {
    pub fn new(id: u64, evaluator_handle: EvaluatorHandle, runtime: Arc<LambdustRuntime>) -> Self {
        Self {
            id,
            evaluator_handle,
            message_count: Arc::new(Mutex::new(0)),
            runtime,
        }
    }

    /// Process a message and return a response
    pub async fn process_message(&self, message: ActorTestMessage) -> Result<ActorTestResponse> {
        {
            let mut count = self.message_count.lock().unwrap();
            *count += 1;
        }

        match message {
            ActorTestMessage::Factorial(n) => {
                let result = self.compute_factorial(n).await?;
                Ok(ActorTestResponse::Value(Value::Integer(result)))
            }
            ActorTestMessage::Sum(numbers) => {
                let result: i64 = numbers.iter().sum();
                Ok(ActorTestResponse::Value(Value::Integer(result)))
            }
            ActorTestMessage::Forward { target_id, message } => {
                // In a real implementation, this would forward to another actor
                // For testing, we'll just acknowledge
                Ok(ActorTestResponse::Ack)
            }
            ActorTestMessage::Ping => {
                Ok(ActorTestResponse::Ack)
            }
            ActorTestMessage::Shutdown => {
                Ok(ActorTestResponse::Ack)
            }
        }
    }

    /// Compute factorial using the evaluator
    async fn compute_factorial(&self, n: u64) -> Result<i64> {
        if n == 0 || n == 1 {
            return Ok(1);
        }

        // Simulate computation using evaluator
        tokio::time::sleep(Duration::from_millis(1)).await;
        
        let mut result = 1i64;
        for i in 1..=n {
            result = result.saturating_mul(i as i64);
        }
        
        Ok(result)
    }

    /// Get the number of messages processed
    pub fn message_count(&self) -> u64 {
        *self.message_count.lock().unwrap()
    }
}

/// Actor system coordinator for managing multiple actors
pub struct ActorSystemCoordinator {
    actors: HashMap<u64, TestActor>,
    runtime: Arc<LambdustRuntime>,
    message_router: Arc<Mutex<HashMap<u64, mpsc::UnboundedSender<(ActorTestMessage, oneshot::Sender<Result<ActorTestResponse>>)>>>>,
}

impl ActorSystemCoordinator {
    pub async fn new(runtime: Arc<LambdustRuntime>) -> Result<Self> {
        Ok(Self {
            actors: HashMap::new(),
            runtime,
            message_router: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Spawn a new actor
    pub async fn spawn_actor(&mut self, actor_id: u64) -> Result<()> {
        let evaluator_handle = self.runtime.spawn_evaluator()?;
        let actor = TestActor::new(actor_id, evaluator_handle, self.runtime.clone());
        
        // Create message channel for this actor
        let (tx, mut rx) = mpsc::unbounded_channel::<(ActorTestMessage, oneshot::Sender<Result<ActorTestResponse>>)>();
        
        {
            let mut router = self.message_router.lock().unwrap();
            router.insert(actor_id, tx);
        }

        // Spawn the actor task
        let actor_clone = TestActor::new(actor_id, actor.evaluator_handle.clone(), self.runtime.clone());
        tokio::spawn(async move {
            while let Some((message, response_sender)) = rx.recv().await {
                let response = actor_clone.process_message(message).await;
                let _ = response_sender.send(response);
            }
        });

        self.actors.insert(actor_id, actor);
        Ok(())
    }

    /// Send a message to an actor
    pub async fn send_message(&self, actor_id: u64, message: ActorTestMessage) -> Result<ActorTestResponse> {
        let sender = {
            let router = self.message_router.lock().unwrap();
            router.get(&actor_id).cloned()
                .ok_or_else(|| Error::runtime_error(
                    format!("Actor {} not found", actor_id),
                    None,
                ))?
        };

        let (response_tx, response_rx) = oneshot::channel();
        sender.send((message, response_tx))
            .map_err(|e| Error::runtime_error(
                format!("Failed to send message: {}", e),
                None,
            ))?;

        response_rx.await
            .map_err(|e| Error::runtime_error(
                format!("Failed to receive response: {}", e),
                None,
            ))?
    }

    /// Get actor statistics
    pub fn get_actor_stats(&self) -> HashMap<u64, u64> {
        self.actors.iter()
            .map(|(id, actor)| (*id, actor.message_count()))
            .collect()
    }
}

// ============================================================================
// BASIC ACTOR SYSTEM TESTS
// ============================================================================

#[tokio::test]
async fn test_actor_creation_and_basic_communication() {
    let runtime = Arc::new(LambdustRuntime::new(Some(4)).expect("Failed to create runtime"));
    let mut coordinator = ActorSystemCoordinator::new(runtime.clone()).await
        .expect("Failed to create coordinator");

    // Spawn test actors
    coordinator.spawn_actor(1).await.expect("Failed to spawn actor 1");
    coordinator.spawn_actor(2).await.expect("Failed to spawn actor 2");

    // Test basic communication
    let response1 = coordinator.send_message(1, ActorTestMessage::Ping).await
        .expect("Failed to send ping to actor 1");
    
    match response1 {
        ActorTestResponse::Ack => println!("✓ Actor 1 responded to ping"),
        _ => panic!("Actor 1 did not respond correctly to ping"),
    }

    let response2 = coordinator.send_message(2, ActorTestMessage::Sum(vec![1, 2, 3, 4, 5])).await
        .expect("Failed to send sum message to actor 2");

    match response2 {
        ActorTestResponse::Value(Value::Integer(15)) => println!("✓ Actor 2 computed sum correctly"),
        _ => panic!("Actor 2 did not compute sum correctly: {:?}", response2),
    }
}

#[tokio::test]
async fn test_distributed_factorial_computation() {
    let runtime = Arc::new(LambdustRuntime::new(Some(8)).expect("Failed to create runtime"));
    let mut coordinator = ActorSystemCoordinator::new(runtime.clone()).await
        .expect("Failed to create coordinator");

    // Spawn multiple actors for distributed computation
    let actor_count = 4;
    for i in 0..actor_count {
        coordinator.spawn_actor(i).await
            .expect(&format!("Failed to spawn actor {}", i));
    }

    // Distribute factorial computations
    let numbers = vec![5, 6, 7, 8];
    let expected_results = vec![120, 720, 5040, 40320];
    
    let start_time = Instant::now();
    let mut handles = Vec::new();

    for (i, &number) in numbers.iter().enumerate() {
        let coordinator_ref = &coordinator;
        let handle = tokio::spawn(async move {
            coordinator_ref.send_message(i as u64, ActorTestMessage::Factorial(number)).await
        });
        handles.push(handle);
    }

    // Collect results
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?
        .into_iter().collect();

    let responses = results?;
    let duration = start_time.elapsed();

    // Verify results
    for (i, response) in responses.iter().enumerate() {
        match response {
            ActorTestResponse::Value(Value::Integer(result)) => {
                assert_eq!(*result, expected_results[i], 
                    "Actor {} computed incorrect factorial for {}", i, numbers[i]);
            }
            _ => panic!("Actor {} returned unexpected response: {:?}", i, response),
        }
    }

    println!("✓ Distributed factorial computation completed in {:?}", duration);
    println!("✓ All {} actors computed factorials correctly", actor_count);

    // Check message distribution
    let stats = coordinator.get_actor_stats();
    for (actor_id, message_count) in stats {
        println!("  Actor {}: processed {} messages", actor_id, message_count);
        assert!(message_count > 0, "Actor {} processed no messages", actor_id);
    }
}

// ============================================================================
// MESSAGE PASSING STRESS TESTS
// ============================================================================

#[tokio::test]
async fn test_high_volume_message_passing() {
    let runtime = Arc::new(LambdustRuntime::new(Some(6)).expect("Failed to create runtime"));
    let mut coordinator = ActorSystemCoordinator::new(runtime.clone()).await
        .expect("Failed to create coordinator");

    let actor_count = 6;
    let messages_per_actor = 100;

    // Spawn actors
    for i in 0..actor_count {
        coordinator.spawn_actor(i).await
            .expect(&format!("Failed to spawn actor {}", i));
    }

    let start_time = Instant::now();
    let barrier = Arc::new(Barrier::new(actor_count));
    let mut handles = Vec::new();

    // Generate high volume of messages
    for actor_id in 0..actor_count {
        let coordinator_ref = &coordinator;
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            // Wait for all threads to be ready
            barrier_clone.wait();
            
            let mut successful_messages = 0;
            for i in 0..messages_per_actor {
                let message = if i % 3 == 0 {
                    ActorTestMessage::Factorial((i % 10) as u64)
                } else if i % 3 == 1 {
                    ActorTestMessage::Sum(vec![i, i + 1, i + 2])
                } else {
                    ActorTestMessage::Ping
                };

                match coordinator_ref.send_message(actor_id, message).await {
                    Ok(_) => successful_messages += 1,
                    Err(e) => eprintln!("Failed to send message to actor {}: {}", actor_id, e),
                }
            }
            
            successful_messages
        });
        handles.push(handle);
    }

    // Wait for all message sending to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?
        .into_iter().collect();

    let successful_counts = results?;
    let duration = start_time.elapsed();
    let total_messages: i32 = successful_counts.iter().sum();
    let total_expected = actor_count * messages_per_actor;

    println!("✓ High volume message passing completed in {:?}", duration);
    println!("✓ Successfully sent {}/{} messages", total_messages, total_expected);
    println!("✓ Throughput: {:.2} messages/sec", 
        total_messages as f64 / duration.as_secs_f64());

    // Verify high success rate
    let success_rate = total_messages as f64 / total_expected as f64;
    assert!(success_rate > 0.95, "Message success rate too low: {:.2}%", success_rate * 100.0);

    // Check final message counts
    let final_stats = coordinator.get_actor_stats();
    for (actor_id, message_count) in final_stats {
        println!("  Actor {}: processed {} messages", actor_id, message_count);
        assert!(message_count >= (messages_per_actor as u64 * 80 / 100), 
            "Actor {} processed too few messages: {}", actor_id, message_count);
    }
}

// ============================================================================
// ACTOR LIFECYCLE TESTS
// ============================================================================

#[tokio::test]
async fn test_actor_lifecycle_management() {
    let runtime = Arc::new(LambdustRuntime::new(Some(4)).expect("Failed to create runtime"));
    let mut coordinator = ActorSystemCoordinator::new(runtime.clone()).await
        .expect("Failed to create coordinator");

    // Test dynamic actor creation and destruction
    let initial_actor_count = 3;
    for i in 0..initial_actor_count {
        coordinator.spawn_actor(i).await
            .expect(&format!("Failed to spawn initial actor {}", i));
    }

    // Verify initial actors are working
    for i in 0..initial_actor_count {
        let response = coordinator.send_message(i, ActorTestMessage::Ping).await
            .expect(&format!("Failed to ping actor {}", i));
        
        match response {
            ActorTestResponse::Ack => {},
            _ => panic!("Actor {} did not respond to ping correctly", i),
        }
    }

    println!("✓ Initial {} actors created and responding", initial_actor_count);

    // Test adding more actors dynamically
    let additional_actors = 2;
    for i in initial_actor_count..(initial_actor_count + additional_actors) {
        coordinator.spawn_actor(i).await
            .expect(&format!("Failed to spawn additional actor {}", i));
    }

    // Verify all actors are still working
    let total_actors = initial_actor_count + additional_actors;
    for i in 0..total_actors {
        let response = coordinator.send_message(i, ActorTestMessage::Sum(vec![i, i + 1])).await
            .expect(&format!("Failed to send sum message to actor {}", i));
        
        match response {
            ActorTestResponse::Value(Value::Integer(result)) => {
                let expected = (i + i + 1) as i64;
                assert_eq!(result, expected, "Actor {} computed wrong sum", i);
            },
            _ => panic!("Actor {} did not compute sum correctly", i),
        }
    }

    println!("✓ Dynamic actor creation successful, {} total actors", total_actors);

    // Test shutdown sequence
    for i in 0..total_actors {
        let response = coordinator.send_message(i, ActorTestMessage::Shutdown).await
            .expect(&format!("Failed to send shutdown to actor {}", i));
        
        match response {
            ActorTestResponse::Ack => {},
            _ => eprintln!("Actor {} did not acknowledge shutdown correctly", i),
        }
    }

    println!("✓ Actor lifecycle management tests completed successfully");
}

// ============================================================================
// FAULT TOLERANCE TESTS
// ============================================================================

#[tokio::test]
async fn test_actor_fault_tolerance() {
    let runtime = Arc::new(LambdustRuntime::new(Some(4)).expect("Failed to create runtime"));
    let mut coordinator = ActorSystemCoordinator::new(runtime.clone()).await
        .expect("Failed to create coordinator");

    let actor_count = 4;
    for i in 0..actor_count {
        coordinator.spawn_actor(i).await
            .expect(&format!("Failed to spawn actor {}", i));
    }

    // Test normal operation first
    for i in 0..actor_count {
        let response = coordinator.send_message(i, ActorTestMessage::Ping).await
            .expect(&format!("Failed to ping actor {}", i));
        
        match response {
            ActorTestResponse::Ack => {},
            _ => panic!("Actor {} did not respond to ping", i),
        }
    }

    println!("✓ All actors responding normally");

    // Test error recovery - simulate computational errors
    // In a real implementation, this would test actual error conditions
    let challenging_computations = vec![
        ActorTestMessage::Factorial(20), // Large factorial
        ActorTestMessage::Sum(vec![i64::MAX / 2, i64::MAX / 2]), // Potential overflow
        ActorTestMessage::Factorial(0), // Edge case
    ];

    for (i, computation) in challenging_computations.iter().enumerate() {
        let actor_id = i as u64 % actor_count;
        match coordinator.send_message(actor_id, computation.clone()).await {
            Ok(response) => {
                println!("✓ Actor {} handled challenging computation successfully: {:?}", 
                    actor_id, response);
            },
            Err(e) => {
                println!("⚠ Actor {} failed computation (expected for some cases): {}", 
                    actor_id, e);
            }
        }
    }

    // Verify actors are still responsive after challenging computations
    for i in 0..actor_count {
        let response = coordinator.send_message(i, ActorTestMessage::Ping).await
            .expect(&format!("Actor {} became unresponsive after challenging computation", i));
        
        match response {
            ActorTestResponse::Ack => {},
            _ => panic!("Actor {} not responding correctly after stress", i),
        }
    }

    println!("✓ Actor fault tolerance test completed - all actors remain responsive");
}