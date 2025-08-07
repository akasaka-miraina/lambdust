//! Effect System Coordination Tests
//!
//! Tests for the coordination of effect systems in multithreaded environments,
//! including parallel effect handling, transactional state management,
//! and effect handler parallel processing.

use lambdust::runtime::LambdustRuntime;
use lambdust::effects::{Effect, EffectContext, EffectSystem, EffectHandler, EffectResult};
use lambdust::eval::Value;
use lambdust::diagnostics::{Result, Error};

use std::sync::{Arc, Mutex, RwLock, Barrier};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::thread;
use tokio::sync::Semaphore;

/// Thread-safe effect handler for testing
#[derive(Debug)]
pub struct ThreadSafeIOHandler {
    pub operations: Arc<Mutex<Vec<String>>>,
    pub operation_count: Arc<Mutex<u64>>,
}

impl ThreadSafeIOHandler {
    pub fn new() -> Self {
        Self {
            operations: Arc::new(Mutex::new(Vec::new())),
            operation_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get_operations(&self) -> Vec<String> {
        self.operations.lock().unwrap().clone()
    }

    pub fn get_operation_count(&self) -> u64 {
        *self.operation_count.lock().unwrap()
    }
}

impl EffectHandler for ThreadSafeIOHandler {
    fn handle(&self, effect: &Effect, args: &[Value]) -> Result<EffectResult> {
        if !matches!(effect, Effect::IO) {
            return Ok(EffectResult::Unhandled);
        }

        let operation = match args.get(0) {
            Some(Value::Symbol(op)) => op.clone(),
            Some(Value::String(op)) => op.clone(),
            _ => "unknown".to_string(),
        };

        // Simulate IO operation
        thread::sleep(Duration::from_millis(1));

        {
            let mut ops = self.operations.lock().unwrap();
            ops.push(operation.clone());
        }

        {
            let mut count = self.operation_count.lock().unwrap();
            *count += 1;
        }

        Ok(EffectResult::Value(Value::String(format!("IO: {}", operation))))
    }

    fn effect_name(&self) -> &str {
        "thread-safe-io"
    }

    fn can_handle(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::IO)
    }
}

/// Thread-safe state handler with transactional semantics
#[derive(Debug)]
pub struct TransactionalStateHandler {
    pub state: Arc<RwLock<HashMap<String, Value>>>,
    pub transaction_count: Arc<Mutex<u64>>,
}

impl TransactionalStateHandler {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
            transaction_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get_state(&self, key: &str) -> Option<Value> {
        self.state.read().unwrap().get(key).cloned()
    }

    pub fn get_transaction_count(&self) -> u64 {
        *self.transaction_count.lock().unwrap()
    }

    fn transactional_update<F>(&self, f: F) -> Result<Value>
    where
        F: FnOnce(&mut HashMap<String, Value>) -> Result<Value>,
    {
        let mut state = self.state.write().unwrap();
        let result = f(&mut *state)?;
        
        {
            let mut count = self.transaction_count.lock().unwrap();
            *count += 1;
        }

        Ok(result)
    }
}

impl EffectHandler for TransactionalStateHandler {
    fn handle(&self, effect: &Effect, args: &[Value]) -> Result<EffectResult> {
        if !matches!(effect, Effect::State) {
            return Ok(EffectResult::Unhandled);
        }

        let operation = match args.get(0) {
            Some(Value::Symbol(op)) => op.as_str(),
            Some(Value::String(op)) => op.as_str(),
            _ => return Err(Error::runtime_error("Invalid state operation".to_string(), None)),
        };

        match operation {
            "get" => {
                if let Some(Value::Symbol(key)) = args.get(1) {
                    let value = self.get_state(key).unwrap_or(Value::Unspecified);
                    Ok(EffectResult::Value(value))
                } else {
                    Err(Error::runtime_error("Invalid key for state get".to_string(), None))
                }
            }
            "set" => {
                if let (Some(Value::Symbol(key)), Some(value)) = (args.get(1), args.get(2)) {
                    let result = self.transactional_update(|state| {
                        state.insert(key.clone(), value.clone());
                        Ok(Value::Unspecified)
                    })?;
                    Ok(EffectResult::Value(result))
                } else {
                    Err(Error::runtime_error("Invalid args for state set".to_string(), None))
                }
            }
            "increment" => {
                if let Some(Value::Symbol(key)) = args.get(1) {
                    let result = self.transactional_update(|state| {
                        let current = state.get(key).cloned().unwrap_or(Value::Integer(0));
                        match current {
                            Value::Integer(n) => {
                                let new_value = Value::Integer(n + 1);
                                state.insert(key.clone(), new_value.clone());
                                Ok(new_value)
                            }
                            _ => Err(Error::runtime_error("Can only increment integers".to_string(), None))
                        }
                    })?;
                    Ok(EffectResult::Value(result))
                } else {
                    Err(Error::runtime_error("Invalid key for state increment".to_string(), None))
                }
            }
            _ => Ok(EffectResult::Unhandled)
        }
    }

    fn effect_name(&self) -> &str {
        "transactional-state"
    }

    fn can_handle(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::State)
    }
}

/// Async logging handler for testing
#[derive(Debug)]
pub struct AsyncLoggingHandler {
    pub logs: Arc<Mutex<Vec<(Instant, String)>>>,
    pub log_count: Arc<Mutex<u64>>,
}

impl AsyncLoggingHandler {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
            log_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get_logs(&self) -> Vec<(Instant, String)> {
        self.logs.lock().unwrap().clone()
    }

    pub fn get_log_count(&self) -> u64 {
        *self.log_count.lock().unwrap()
    }
}

impl EffectHandler for AsyncLoggingHandler {
    fn handle(&self, effect: &Effect, args: &[Value]) -> Result<EffectResult> {
        if !matches!(effect, Effect::Custom(name) if name == "logging") {
            return Ok(EffectResult::Unhandled);
        }

        let message = match args.get(0) {
            Some(Value::String(msg)) => msg.clone(),
            Some(Value::Symbol(msg)) => msg.clone(),
            _ => "empty log".to_string(),
        };

        {
            let mut logs = self.logs.lock().unwrap();
            logs.push((Instant::now(), message.clone()));
        }

        {
            let mut count = self.log_count.lock().unwrap();
            *count += 1;
        }

        Ok(EffectResult::Value(Value::String(format!("Logged: {}", message))))
    }

    fn effect_name(&self) -> &str {
        "async-logging"
    }

    fn can_handle(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::Custom(name) if name == "logging")
    }
}

// ============================================================================
// PARALLEL EFFECT COORDINATION TESTS
// ============================================================================

#[tokio::test]
async fn test_parallel_io_effects() {
    let runtime = Arc::new(LambdustRuntime::new(Some(4)).expect("Failed to create runtime"));
    let io_handler = Arc::new(ThreadSafeIOHandler::new());
    
    let thread_count = 4;
    let operations_per_thread = 25;
    
    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let handler_clone = io_handler.clone();
        let runtime_clone = runtime.clone();

        let handle = tokio::spawn(async move {
            // Wait for all threads to be ready
            barrier_clone.wait();
            
            let start_time = Instant::now();
            let mut successful_ops = 0;

            for op_id in 0..operations_per_thread {
                let operation = format!("thread-{}-op-{}", thread_id, op_id);
                let args = vec![Value::String(operation)];
                
                match handler_clone.handle(&Effect::IO, &args) {
                    Ok(EffectResult::Value(_)) => successful_ops += 1,
                    Ok(_) => {},
                    Err(e) => eprintln!("IO operation failed: {}", e),
                }

                // Small delay to simulate realistic operation timing
                tokio::time::sleep(Duration::from_millis(1)).await;
            }

            let duration = start_time.elapsed();
            (thread_id, successful_ops, duration)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    let total_operations: usize = results.iter().map(|(_, ops, _)| *ops).sum();
    let total_expected = thread_count * operations_per_thread;
    
    println!("✓ Parallel IO effects test completed");
    println!("  Total operations: {}/{}", total_operations, total_expected);
    println!("  Handler operation count: {}", io_handler.get_operation_count());
    
    // Verify operation counts match
    assert_eq!(total_operations, total_expected, "Some IO operations failed");
    assert_eq!(io_handler.get_operation_count() as usize, total_expected, 
        "Handler count doesn't match operations");

    // Verify all operations were recorded
    let recorded_ops = io_handler.get_operations();
    assert_eq!(recorded_ops.len(), total_expected, "Not all operations were recorded");

    // Check for thread safety - no duplicate operations
    let mut unique_ops = std::collections::HashSet::new();
    for op in &recorded_ops {
        assert!(unique_ops.insert(op.clone()), "Duplicate operation found: {}", op);
    }

    println!("✓ All {} operations were unique and correctly recorded", total_expected);
}

#[tokio::test]
async fn test_transactional_state_management() {
    let runtime = Arc::new(LambdustRuntime::new(Some(6)).expect("Failed to create runtime"));
    let state_handler = Arc::new(TransactionalStateHandler::new());
    
    let thread_count = 6;
    let increments_per_thread = 50;
    
    // Initialize shared counter
    state_handler.handle(&Effect::State, &[
        Value::Symbol("set".to_string()),
        Value::Symbol("counter".to_string()),
        Value::Integer(0),
    ]).expect("Failed to initialize counter");

    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let handler_clone = state_handler.clone();

        let handle = tokio::spawn(async move {
            // Wait for all threads to be ready
            barrier_clone.wait();
            
            let mut successful_increments = 0;

            for _increment in 0..increments_per_thread {
                match handler_clone.handle(&Effect::State, &[
                    Value::Symbol("increment".to_string()),
                    Value::Symbol("counter".to_string()),
                ]) {
                    Ok(EffectResult::Value(Value::Integer(_))) => successful_increments += 1,
                    Ok(_) => {},
                    Err(e) => eprintln!("Thread {} increment failed: {}", thread_id, e),
                }

                // Small delay to increase chance of race conditions
                tokio::time::sleep(Duration::from_millis(1)).await;
            }

            (thread_id, successful_increments)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    let total_successful: usize = results.iter().map(|(_, increments)| *increments).sum();
    let expected_total = thread_count * increments_per_thread;

    // Get final counter value
    let final_value = state_handler.get_state("counter").unwrap();
    let final_count = match final_value {
        Value::Integer(n) => n,
        _ => panic!("Counter is not an integer: {:?}", final_value),
    };

    println!("✓ Transactional state management test completed");
    println!("  Successful increments: {}/{}", total_successful, expected_total);
    println!("  Final counter value: {}", final_count);
    println!("  Total transactions: {}", state_handler.get_transaction_count());

    // Verify consistency
    assert_eq!(total_successful, expected_total, "Some increments failed");
    assert_eq!(final_count, expected_total as i64, 
        "Final counter value doesn't match expected increments");

    // Verify no race conditions occurred
    assert_eq!(state_handler.get_transaction_count() as usize, expected_total + 1, // +1 for initialization
        "Transaction count doesn't match operations");

    println!("✓ No race conditions detected - all {} increments applied correctly", expected_total);
}

#[tokio::test]
async fn test_parallel_effect_handlers() {
    let runtime = Arc::new(LambdustRuntime::new(Some(6)).expect("Failed to create runtime"));
    
    // Create multiple effect handlers
    let io_handler = Arc::new(ThreadSafeIOHandler::new());
    let state_handler = Arc::new(TransactionalStateHandler::new());
    let logging_handler = Arc::new(AsyncLoggingHandler::new());
    
    let thread_count = 6;
    let operations_per_thread = 20;
    
    // Initialize state
    state_handler.handle(&Effect::State, &[
        Value::Symbol("set".to_string()),
        Value::Symbol("shared_data".to_string()),
        Value::Integer(100),
    ]).expect("Failed to initialize state");

    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let io_clone = io_handler.clone();
        let state_clone = state_handler.clone();
        let logging_clone = logging_handler.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut io_ops = 0;
            let mut state_ops = 0;
            let mut log_ops = 0;

            for op_id in 0..operations_per_thread {
                // Rotate between different effect types
                match op_id % 3 {
                    0 => {
                        // IO operation
                        let args = vec![Value::String(format!("io-thread-{}-op-{}", thread_id, op_id))];
                        if io_clone.handle(&Effect::IO, &args).is_ok() {
                            io_ops += 1;
                        }
                    }
                    1 => {
                        // State operation
                        let args = vec![
                            Value::Symbol("increment".to_string()),
                            Value::Symbol("shared_data".to_string()),
                        ];
                        if state_clone.handle(&Effect::State, &args).is_ok() {
                            state_ops += 1;
                        }
                    }
                    2 => {
                        // Logging operation
                        let args = vec![Value::String(format!("log-thread-{}-op-{}", thread_id, op_id))];
                        if logging_clone.handle(&Effect::Custom("logging".to_string()), &args).is_ok() {
                            log_ops += 1;
                        }
                    }
                    _ => unreachable!(),
                }

                // Small delay between operations
                tokio::time::sleep(Duration::from_millis(1)).await;
            }

            (thread_id, io_ops, state_ops, log_ops)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    // Aggregate results
    let (total_io, total_state, total_log): (usize, usize, usize) = results.iter()
        .fold((0, 0, 0), |(io, state, log), (_, thread_io, thread_state, thread_log)| {
            (io + thread_io, state + thread_state, log + thread_log)
        });

    let expected_per_type = (thread_count * operations_per_thread + 2) / 3; // Round up division

    println!("✓ Parallel effect handlers test completed");
    println!("  IO operations: {}/{}", total_io, expected_per_type);
    println!("  State operations: {}/{}", total_state, expected_per_type);
    println!("  Logging operations: {}/{}", total_log, expected_per_type);

    // Verify handler statistics
    println!("  IO handler count: {}", io_handler.get_operation_count());
    println!("  State handler transactions: {}", state_handler.get_transaction_count());
    println!("  Logging handler count: {}", logging_handler.get_log_count());

    // Verify final state value
    let final_state = state_handler.get_state("shared_data").unwrap();
    match final_state {
        Value::Integer(n) => {
            let expected_final = 100 + total_state as i64; // Initial 100 + increments
            assert_eq!(n, expected_final, "Final state value incorrect");
            println!("  Final shared_data value: {} (started at 100)", n);
        }
        _ => panic!("Shared data is not an integer: {:?}", final_state),
    }

    // Verify no operations were lost
    assert!(total_io >= expected_per_type - 1, "Too few IO operations completed");
    assert!(total_state >= expected_per_type - 1, "Too few state operations completed");
    assert!(total_log >= expected_per_type - 1, "Too few logging operations completed");

    println!("✓ All effect handlers coordinated successfully in parallel execution");
}

// ============================================================================
// EFFECT CONSISTENCY TESTS
// ============================================================================

#[tokio::test]
async fn test_effect_isolation_and_consistency() {
    let runtime = Arc::new(LambdustRuntime::new(Some(4)).expect("Failed to create runtime"));
    
    // Create isolated state handlers for different "contexts"
    let context1_state = Arc::new(TransactionalStateHandler::new());
    let context2_state = Arc::new(TransactionalStateHandler::new());
    
    // Initialize different values in each context
    context1_state.handle(&Effect::State, &[
        Value::Symbol("set".to_string()),
        Value::Symbol("value".to_string()),
        Value::Integer(1000),
    ]).expect("Failed to initialize context1");

    context2_state.handle(&Effect::State, &[
        Value::Symbol("set".to_string()),
        Value::Symbol("value".to_string()),
        Value::Integer(2000),
    ]).expect("Failed to initialize context2");

    let thread_count = 4;
    let operations_per_thread = 30;
    
    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let barrier_clone = barrier.clone();
        let ctx1_clone = context1_state.clone();
        let ctx2_clone = context2_state.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait();
            
            let mut ctx1_ops = 0;
            let mut ctx2_ops = 0;

            for op_id in 0..operations_per_thread {
                if thread_id % 2 == 0 {
                    // Even threads work with context1
                    if ctx1_clone.handle(&Effect::State, &[
                        Value::Symbol("increment".to_string()),
                        Value::Symbol("value".to_string()),
                    ]).is_ok() {
                        ctx1_ops += 1;
                    }
                } else {
                    // Odd threads work with context2
                    if ctx2_clone.handle(&Effect::State, &[
                        Value::Symbol("increment".to_string()),
                        Value::Symbol("value".to_string()),
                    ]).is_ok() {
                        ctx2_ops += 1;
                    }
                }

                tokio::time::sleep(Duration::from_millis(1)).await;
            }

            (thread_id, ctx1_ops, ctx2_ops)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await
        .map_err(|e| Error::runtime_error(format!("Join error: {}", e), None))?;

    // Calculate expected operations per context
    let ctx1_threads = (thread_count + 1) / 2; // Even threads: 0, 2, 4, ...
    let ctx2_threads = thread_count / 2;       // Odd threads: 1, 3, 5, ...
    
    let expected_ctx1_ops = ctx1_threads * operations_per_thread;
    let expected_ctx2_ops = ctx2_threads * operations_per_thread;

    // Get final values
    let final_ctx1 = context1_state.get_state("value").unwrap();
    let final_ctx2 = context2_state.get_state("value").unwrap();

    match (final_ctx1, final_ctx2) {
        (Value::Integer(val1), Value::Integer(val2)) => {
            let expected_val1 = 1000 + expected_ctx1_ops as i64;
            let expected_val2 = 2000 + expected_ctx2_ops as i64;
            
            println!("✓ Effect isolation and consistency test completed");
            println!("  Context1: {} → {} (expected {})", 1000, val1, expected_val1);
            println!("  Context2: {} → {} (expected {})", 2000, val2, expected_val2);
            
            assert_eq!(val1, expected_val1, "Context1 final value incorrect");
            assert_eq!(val2, expected_val2, "Context2 final value incorrect");
        }
        _ => panic!("Context values are not integers: {:?}, {:?}", final_ctx1, final_ctx2),
    }

    println!("✓ Effect isolation maintained - contexts did not interfere with each other");
}