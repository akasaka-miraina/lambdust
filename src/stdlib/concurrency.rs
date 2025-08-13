#![allow(unused_variables)]
//! Scheme API bindings for the concurrency system.
//!
//! This module provides Scheme-friendly wrappers around the concurrency
//! primitives, making them accessible from Scheme code with idiomatic APIs.

// The entire concurrency stdlib module is only available with async-runtime feature
#[cfg(feature = "async-runtime")]
mod concurrency_impl {
    use crate::eval::{Value, ThreadSafeEnvironment, PrimitiveProcedure, PrimitiveImpl};
    use crate::diagnostics::{Error, Result};
    use crate::effects::Effect;
    use crate::concurrency::{
        futures::{Future, Promise, FutureOps},
        channels::{Channel, ChannelConfig, ChannelType},
        parallel::{ParallelOps, ParallelConfig},
        Mutex, SemaphoreSync, AtomicCounter,
        actors::{global_actor_system, EchoActor},
        scheduler::{submit_task, submit_priority_task, Priority},
        distributed::DistributedNode,
    };
    use std::sync::Arc;
    use std::time::Duration;

/// Registers all concurrency primitives with the environment.
pub fn populate_environment(env: &ThreadSafeEnvironment) {
    // Future/Promise operations
    register_future_operations(env);
    
    // Channel operations
    register_channel_operations(env);
    
    // Parallel computation operations
    register_parallel_operations(env);
    
    // Synchronization primitives
    register_sync_operations(env);
    
    // Actor system operations
    register_actor_operations(env);
    
    // Scheduler operations
    register_scheduler_operations(env);
    
    // Distributed computing operations
    register_distributed_operations(env);
}

/// Registers future/promise operations.
fn register_future_operations(env: &ThreadSafeEnvironment) {
    // (future-resolved value) - Create a resolved future
    env.define("future-resolved".to_string(), Value::Primitive(Arc::new(
        crate::eval::value::PrimitiveProcedure {
            name: "future-resolved".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: crate::eval::value::PrimitiveImpl::RustFn(|args| {
                if args.len() != 1 {
                    return Err(Box::new(Error::runtime_error("future-resolved expects 1 argument".to_string(), None)));
                }
                let future = Future::resolved(args[0].clone());
                Ok(Value::Future(Arc::new(future)))
            }),
            effects: vec![crate::effects::Effect::State],
        }
    )));

    // (future-rejected error) - Create a rejected future
    env.define("future-rejected".to_string(), Value::Primitive(Arc::new(
        PrimitiveProcedure {
            name: "future-rejected".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| {
                if args.len() != 1 {
                    return Err(Box::new(Error::runtime_error("future-rejected expects 1 argument".to_string(), None)));
                }
                let error = Error::runtime_error(args[0].to_string(), None);
                let future = Future::rejected(error);
                Ok(Value::Future(Arc::new(future)))
            }),
            effects: vec![Effect::State],
        }
    )));

    // (promise) - Create a new promise
    env.define("promise".to_string(), Value::Primitive(Arc::new(
        PrimitiveProcedure {
            name: "promise".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(|args| {
                if !args.is_empty() {
                    return Err(Box::new(Error::runtime_error("promise expects no arguments".to_string(), None)));
                }
                let promise = Promise::new();
                // Convert the promise to a future for consistency with the concurrency model
                let future = Future::from_promise(promise);
                Ok(Value::Future(Arc::new(future)))
            }),
            effects: vec![Effect::State],
        }
    )));

    // (future-delay duration value) - Create a delayed future
    env.define("future-delay".to_string(), Value::Primitive(Arc::new(
        PrimitiveProcedure {
            name: "future-delay".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(primitive_future_delay),
            effects: vec![Effect::State],
        }
    )));

    // (future-all futures) - Wait for all futures to complete
    env.define("future-all".to_string(), Value::Primitive(Arc::new(
        PrimitiveProcedure {
            name: "future-all".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_future_all),
            effects: vec![Effect::State],
        }
    )));

    // (future-race futures) - Race multiple futures
    env.define("future-race".to_string(), Value::Primitive(Arc::new(
        PrimitiveProcedure {
            name: "future-race".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_future_race),
            effects: vec![Effect::State],
        }
    )));
}

/// Registers channel operations.
fn register_channel_operations(env: &ThreadSafeEnvironment) {
    // (make-channel capacity) - Create a bounded channel
    env.define("make-channel".to_string(), Value::Primitive(Arc::new(
        crate::eval::value::PrimitiveProcedure {
            name: "make-channel".to_string(),
            arity_min: 0,
            arity_max: Some(1),
            implementation: crate::eval::value::PrimitiveImpl::RustFn(|args| {
            let config = if args.is_empty() {
                ChannelConfig::default()
            } else if args.len() == 1 {
                let capacity = args[0].as_number()
                    .ok_or_else(|| Error::runtime_error("Capacity must be a number".to_string(), None))?;
                ChannelConfig {
                    buffer_size: Some(capacity as usize),
                    channel_type: ChannelType::MpscBounded,
                    backpressure: true,
                }
            } else {
                return Err(Box::new(Error::runtime_error("make-channel expects 0 or 1 arguments".to_string(), None)));
            };
            
            let channel = Channel::new(config)
                .map_err(|e| Error::runtime_error(format!("Failed to create channel: {e}"), None))?;
            Ok(Value::Channel(Arc::new(channel)))
            }),
            effects: vec![crate::effects::Effect::State],
        }
    )));

    // (make-unbounded-channel) - Create an unbounded channel
    env.define("make-unbounded-channel".to_string(), Value::Primitive(Arc::new(
        PrimitiveProcedure {
            name: "make-unbounded-channel".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(primitive_make_unbounded_channel),
            effects: vec![Effect::State],
        }
    )));

    // (make-broadcast-channel capacity) - Create a broadcast channel
    env.define("make-broadcast-channel".to_string(), Value::Primitive(Arc::new(
        PrimitiveProcedure {
            name: "make-broadcast-channel".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_make_broadcast_channel),
            effects: vec![Effect::State],
        }
    )));

    // (channel-send! channel value) - Send a value to a channel
    env.define("channel-send!".to_string(), Value::Primitive(Arc::new(
        PrimitiveProcedure {
            name: "channel-send!".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(primitive_channel_send),
            effects: vec![Effect::State],
        }
    )));

    // (channel-recv! channel) - Receive a value from a channel
    env.define("channel-recv!".to_string(), Value::Primitive(Arc::new(
        PrimitiveProcedure {
            name: "channel-recv!".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_channel_recv),
            effects: vec![Effect::State],
        }
    )));
}

/// Helper function to create a primitive procedure
fn create_primitive(
    name: &str, 
    arity_min: usize, 
    arity_max: Option<usize>, 
    implementation: fn(&[Value]) -> Result<Value>
) -> Value {
    Value::Primitive(Arc::new(PrimitiveProcedure {
        name: name.to_string(),
        arity_min,
        arity_max,
        implementation: PrimitiveImpl::RustFn(implementation),
        effects: Vec::new(),
    }))
}

/// Registers parallel computation operations.
fn register_parallel_operations(env: &ThreadSafeEnvironment) {
    // (par-map proc list) - Parallel map
    fn par_map_impl(args: &[Value]) -> Result<Value> {
            if args.len() != 2 {
                return Err(Box::new(Error::runtime_error("par-map expects 2 arguments".to_string(), None)));
            }
            
            let proc = args[0].clone();
            let list = &args[1];
            
            // Convert Scheme list to Vec<Value>
            let mut values = Vec::new();
            let mut current = list;
            loop {
                match current {
                    Value::Pair(car, cdr) => {
                        values.push(car.as_ref().clone());
                        current = cdr;
                    }
                    Value::Nil => break,
                    _ => {
                        values.push(current.clone());
                        break;
                    }
                }
            }
            
            let ops = ParallelOps::new(ParallelConfig::default());
            let future = ops.par_map(values, move |value| {
                // In a real implementation, you'd call the Scheme procedure here
                // For now, just return the value unchanged
                Ok(value)
            });
            
            Ok(Value::Future(Arc::new(future)))
    }
    
    env.define("par-map".to_string(), create_primitive("par-map", 2, Some(2), par_map_impl));

    // (par-filter pred list) - Parallel filter
    fn par_filter_impl(args: &[Value]) -> Result<Value> {
            if args.len() != 2 {
                return Err(Box::new(Error::runtime_error("par-filter expects 2 arguments".to_string(), None)));
            }
            
            let pred = args[0].clone();
            let list = &args[1];
            
            // Convert Scheme list to Vec<Value>
            let mut values = Vec::new();
            let mut current = list;
            loop {
                match current {
                    Value::Pair(car, cdr) => {
                        values.push(car.as_ref().clone());
                        current = cdr;
                    }
                    Value::Nil => break,
                    _ => {
                        values.push(current.clone());
                        break;
                    }
                }
            }
            
            let ops = ParallelOps::new(ParallelConfig::default());
            let future = ops.par_filter(values, move |_value| {
                // In a real implementation, you'd call the Scheme predicate here
                Ok(true) // Placeholder
            });
            
            Ok(Value::Future(Arc::new(future)))
    }
    
    env.define("par-filter".to_string(), create_primitive("par-filter", 2, Some(2), par_filter_impl));

    // (par-reduce proc identity list) - Parallel reduce
    fn par_reduce_impl(args: &[Value]) -> Result<Value> {
            if args.len() != 3 {
                return Err(Box::new(Error::runtime_error("par-reduce expects 3 arguments".to_string(), None)));
            }
            
            let proc = args[0].clone();
            let identity = args[1].clone();
            let list = &args[2];
            
            // Convert Scheme list to Vec<Value>
            let mut values = Vec::new();
            let mut current = list;
            loop {
                match current {
                    Value::Pair(car, cdr) => {
                        values.push(car.as_ref().clone());
                        current = cdr;
                    }
                    Value::Nil => break,
                    _ => {
                        values.push(current.clone());
                        break;
                    }
                }
            }
            
            let ops = ParallelOps::new(ParallelConfig::default());
            let future = ops.par_reduce(values, identity, move |a, b| {
                // In a real implementation, you'd call the Scheme procedure here
                Ok(a) // Placeholder
            });
            
            Ok(Value::Future(Arc::new(future)))
    }
    
    env.define("par-reduce".to_string(), create_primitive("par-reduce", 3, Some(3), par_reduce_impl));
}

/// Registers synchronization operations.
fn register_sync_operations(env: &ThreadSafeEnvironment) {
    // (make-mutex value) - Create a mutex
    fn make_mutex_impl(args: &[Value]) -> Result<Value> {
            let value = if args.is_empty() {
                Value::Nil
            } else if args.len() == 1 {
                args[0].clone()
            } else {
                return Err(Box::new(Error::runtime_error("make-mutex expects 0 or 1 arguments".to_string(), None)));
            };
            
            let mutex = Mutex::new(value);
            Ok(Value::Mutex(Arc::new(mutex)))
    }
    
    env.define("make-mutex".to_string(), create_primitive("make-mutex", 0, Some(1), make_mutex_impl));

    // (make-semaphore permits) - Create a semaphore
    fn make_semaphore_impl(args: &[Value]) -> Result<Value> {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error("make-semaphore expects 1 argument".to_string(), None)));
            }
            
            let permits = args[0].as_number()
                .ok_or_else(|| Error::runtime_error("Permits must be a number".to_string(), None))?;
            
            let semaphore = SemaphoreSync::new(permits as usize);
            Ok(Value::Semaphore(Arc::new(semaphore)))
    }
    
    env.define("make-semaphore".to_string(), create_primitive("make-semaphore", 1, Some(1), make_semaphore_impl));

    // (make-atomic-counter initial) - Create an atomic counter
    fn make_atomic_counter_impl(args: &[Value]) -> Result<Value> {
            let initial = if args.is_empty() {
                0
            } else if args.len() == 1 {
                args[0].as_number()
                    .ok_or_else(|| Error::runtime_error("Initial value must be a number".to_string(), None))? as i64
            } else {
                return Err(Box::new(Error::runtime_error("make-atomic-counter expects 0 or 1 arguments".to_string(), None)));
            };
            
            let counter = AtomicCounter::new(initial);
            Ok(Value::AtomicCounter(Arc::new(counter)))
    }
    
    env.define("make-atomic-counter".to_string(), create_primitive("make-atomic-counter", 0, Some(1), make_atomic_counter_impl));

    // (atomic-counter-increment! counter) - Increment atomic counter
    fn atomic_counter_increment_impl(args: &[Value]) -> Result<Value> {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error("atomic-counter-increment! expects 1 argument".to_string(), None)));
            }
            
            if let Value::AtomicCounter(counter) = &args[0] {
                let new_value = counter.increment();
                Ok(Value::integer(new_value))
            } else {
                Err(Box::new(Error::runtime_error("Argument must be an atomic counter".to_string(), None)))
            }
    }
    
    env.define("atomic-counter-increment!".to_string(), create_primitive("atomic-counter-increment!", 1, Some(1), atomic_counter_increment_impl));

    // (atomic-counter-get counter) - Get atomic counter value
    fn atomic_counter_get_impl(args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(Box::new(Error::runtime_error("atomic-counter-get expects 1 argument".to_string(), None)));
        }
        
        if let Value::AtomicCounter(counter) = &args[0] {
            let value = counter.get();
            Ok(Value::integer(value))
        } else {
            Err(Box::new(Error::runtime_error("Argument must be an atomic counter".to_string(), None)))
        }
    }
    
    env.define("atomic-counter-get".to_string(), create_primitive("atomic-counter-get", 1, Some(1), atomic_counter_get_impl));
}

/// Registers actor system operations.
fn register_actor_operations(env: &ThreadSafeEnvironment) {
    // (spawn-actor behavior) - Spawn a new actor
    fn spawn_actor_impl(args: &[Value]) -> Result<Value> {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error("spawn-actor expects 1 argument".to_string(), None)));
            }
            
            // For simplicity, spawn an echo actor
            let system = global_actor_system();
            let actor = EchoActor;
            
            let future = Future::new(async move {
                let actor_ref = system.spawn_actor(actor, None, None).await?;
                Ok(Value::integer(actor_ref.id().as_u64() as i64))
            });
            
            Ok(Value::Future(Arc::new(future)))
    }
    
    env.define("spawn-actor".to_string(), create_primitive("spawn-actor", 1, Some(1), spawn_actor_impl));

    // (actor-tell actor-id message) - Send a message to an actor
    fn actor_tell_impl(args: &[Value]) -> Result<Value> {
            if args.len() != 2 {
                return Err(Box::new(Error::runtime_error("actor-tell expects 2 arguments".to_string(), None)));
            }
            
            let actor_id = args[0].as_number()
                .ok_or_else(|| Error::runtime_error("Actor ID must be a number".to_string(), None))? as u64;
            let message = args[1].clone();
            
            let system = global_actor_system();
            if let Some(actor_ref) = system.get_actor(crate::concurrency::actors::ActorId(actor_id)) {
                actor_ref.tell(message)
                    .map_err(|e| Error::runtime_error(format!("Failed to send message: {e}"), None))?;
                Ok(Value::Unspecified)
            } else {
                Err(Box::new(Error::runtime_error("Actor not found".to_string(), None)))
            }
    }
    
    env.define("actor-tell".to_string(), create_primitive("actor-tell", 2, Some(2), actor_tell_impl));
}

/// Registers scheduler operations.
fn register_scheduler_operations(env: &ThreadSafeEnvironment) {
    // (submit-task thunk) - Submit a task to the scheduler
    fn submit_task_impl(args: &[Value]) -> Result<Value> {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error("submit-task expects 1 argument".to_string(), None)));
            }
            
            let thunk = args[0].clone();
            
            let task_id = submit_task(move || {
                // In a real implementation, you'd call the Scheme thunk here
                Ok(Value::Unspecified)
            }).map_err(|e| Error::runtime_error(format!("Failed to submit task: {e}"), None))?;
            
            Ok(Value::integer(task_id.as_u64() as i64))
    }
    
    env.define("submit-task".to_string(), create_primitive("submit-task", 1, Some(1), submit_task_impl));

    // (submit-priority-task thunk priority) - Submit a priority task
    fn submit_priority_task_impl(args: &[Value]) -> Result<Value> {
            if args.len() != 2 {
                return Err(Box::new(Error::runtime_error("submit-priority-task expects 2 arguments".to_string(), None)));
            }
            
            let thunk = args[0].clone();
            let priority_num = args[1].as_number()
                .ok_or_else(|| Error::runtime_error("Priority must be a number".to_string(), None))? as u8;
            
            let priority = match priority_num {
                0 => Priority::Low,
                1 => Priority::Normal,
                2 => Priority::High,
                3 => Priority::Critical,
                _ => return Err(Box::new(Error::runtime_error("Priority must be 0-3".to_string(), None))),
            };
            
            let task_id = submit_priority_task(move || {
                // In a real implementation, you'd call the Scheme thunk here
                Ok(Value::Unspecified)
            }, priority).map_err(|e| Error::runtime_error(format!("Failed to submit task: {e}"), None))?;
            
            Ok(Value::integer(task_id.as_u64() as i64))
    }
    
    env.define("submit-priority-task".to_string(), create_primitive("submit-priority-task", 2, Some(2), submit_priority_task_impl));
}

/// Registers distributed computing operations.
fn register_distributed_operations(env: &ThreadSafeEnvironment) {
    // (make-distributed-node) - Create a distributed node
    fn make_distributed_node_impl(args: &[Value]) -> Result<Value> {
            if !args.is_empty() {
                return Err(Box::new(Error::runtime_error("make-distributed-node expects no arguments".to_string(), None)));
            }
            
            let node = DistributedNode::new();
            Ok(Value::DistributedNode(Arc::new(node)))
    }
    
    env.define("make-distributed-node".to_string(), create_primitive("make-distributed-node", 0, Some(0), make_distributed_node_impl));

    // Additional distributed operations would be implemented here...
}

// Value extensions for concurrency types would be in value.rs, not here

// Add new Value variants for concurrency types (this would go in value.rs)
// Future(Arc<Future>),
// Promise(Arc<std::sync::RwLock<Promise>>),
// Channel(Arc<Channel>),
// Mutex(Arc<Mutex>),
// Semaphore(Arc<SemaphoreSync>),
// AtomicCounter(Arc<AtomicCounter>),
// DistributedNode(Arc<std::sync::Mutex<DistributedNode>>),

/// Helper macros for concurrency operations.
#[macro_use]
pub mod macros {
    /// Converts a Scheme list to a Rust Vec.
    #[macro_export]
    macro_rules! scheme_list_to_vec {
        ($list:expr) => {{
            let mut values = Vec::new();
            let mut current = $list;
            loop {
                match current {
                    Value::Pair(car, cdr) => {
                        values.push(car.as_ref().clone());
                        current = cdr;
                    }
                    Value::Nil => break,
                    _ => {
                        values.push(current.clone());
                        break;
                    }
                }
            }
            values
        }};
    }

    /// Converts a Rust Vec to a Scheme list.
    #[macro_export]
    macro_rules! vec_to_scheme_list {
        ($vec:expr) => {{
            let mut list = Value::Nil;
            for value in $vec.into_iter().rev() {
                list = Value::pair(value, list);
            }
            list
        }};
    }
}

// ============= PRIMITIVE FUNCTION IMPLEMENTATIONS =============

/// Implementation of (future-delay duration value)
fn primitive_future_delay(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error("future-delay expects 2 arguments".to_string(), None)));
    }
    
    let duration_ms = args[0].as_number()
        .ok_or_else(|| Error::runtime_error("Duration must be a number".to_string(), None))?;
    let value = args[1].clone();
    let duration = Duration::from_millis(duration_ms as u64);
    
    let future = FutureOps::delay_value(duration, value);
    Ok(Value::Future(Arc::new(future)))
}

/// Implementation of (future-all futures)
fn primitive_future_all(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("future-all expects 1 argument".to_string(), None)));
    }
    
    let futures_list = &args[0];
    let mut futures = Vec::new();
    
    // Convert Scheme list to Vec<Future>
    let mut current = futures_list;
    loop {
        match current {
            Value::Pair(car, cdr) => {
                if let Value::Future(future) = car.as_ref() {
                    futures.push((**future).clone());
                } else {
                    return Err(Box::new(Error::runtime_error("Expected future in list".to_string(), None)));
                }
                current = cdr;
            }
            Value::Nil => break,
            _ => return Err(Box::new(Error::runtime_error("Expected list of futures".to_string(), None))),
        }
    }
    
    let future = FutureOps::all(futures);
    Ok(Value::Future(Arc::new(future)))
}

/// Implementation of (future-race futures)
fn primitive_future_race(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("future-race expects 1 argument".to_string(), None)));
    }
    
    let futures_list = &args[0];
    let mut futures = Vec::new();
    
    // Convert Scheme list to Vec<Future>
    let mut current = futures_list;
    loop {
        match current {
            Value::Pair(car, cdr) => {
                if let Value::Future(future) = car.as_ref() {
                    futures.push((**future).clone());
                } else {
                    return Err(Box::new(Error::runtime_error("Expected future in list".to_string(), None)));
                }
                current = cdr;
            }
            Value::Nil => break,
            _ => return Err(Box::new(Error::runtime_error("Expected list of futures".to_string(), None))),
        }
    }
    
    let future = FutureOps::race(futures);
    Ok(Value::Future(Arc::new(future)))
}

/// Implementation of (make-unbounded-channel)
fn primitive_make_unbounded_channel(args: &[Value]) -> Result<Value> {
    if !args.is_empty() {
        return Err(Box::new(Error::runtime_error("make-unbounded-channel expects no arguments".to_string(), None)));
    }
    
    let channel = Channel::unbounded()
        .map_err(|e| Error::runtime_error(format!("Failed to create channel: {e}"), None))?;
    Ok(Value::Channel(Arc::new(channel)))
}

/// Implementation of (make-broadcast-channel capacity)
fn primitive_make_broadcast_channel(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("make-broadcast-channel expects 1 argument".to_string(), None)));
    }
    
    let capacity = args[0].as_number()
        .ok_or_else(|| Error::runtime_error("Capacity must be a number".to_string(), None))?;
    
    let channel = Channel::broadcast(capacity as usize)
        .map_err(|e| Error::runtime_error(format!("Failed to create channel: {e}"), None))?;
    Ok(Value::Channel(Arc::new(channel)))
}

/// Implementation of (channel-send! channel value)
fn primitive_channel_send(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error("channel-send! expects 2 arguments".to_string(), None)));
    }
    
    if let Value::Channel(channel) = &args[0] {
        let value = args[1].clone();
        let sender = channel.sender();
        
        let future = Future::new(async move {
            sender.send(value).await.map(|_| Value::Unspecified)
        });
        
        Ok(Value::Future(Arc::new(future)))
    } else {
        Err(Box::new(Error::runtime_error("First argument must be a channel".to_string(), None)))
    }
}

/// Implementation of (channel-recv! channel)
fn primitive_channel_recv(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("channel-recv! expects 1 argument".to_string(), None)));
    }
    
    if let Value::Channel(channel) = &args[0] {
        let receiver = channel.receiver();
        
        let future = Future::new(async move {
            let mut rx = receiver.lock().await;
            rx.recv().await
        });
        
        Ok(Value::Future(Arc::new(future)))
    } else {
        Err(Box::new(Error::runtime_error("Argument must be a channel".to_string(), None)))
    }
}

    /// Concurrency standard library initialization.
    pub fn init_concurrency_stdlib() -> Result<()> {
        // Initialize the concurrency system
        crate::concurrency::initialize()?;
        Ok(())
    }
}

// Re-export functions when async-runtime feature is available
#[cfg(feature = "async-runtime")]
pub use concurrency_impl::*;

// Provide stub implementations when async-runtime is not available
#[cfg(not(feature = "async-runtime"))]
pub fn populate_environment(_env: &crate::eval::ThreadSafeEnvironment) {
    // No-op when async runtime is disabled
}

#[cfg(not(feature = "async-runtime"))]
pub fn init_concurrency_stdlib() -> crate::diagnostics::Result<()> {
    // No-op when async runtime is disabled
    Ok(())
}