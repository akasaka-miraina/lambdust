//! SRFI-21 procedure implementations
//!
//! This module implements all SRFI-21 procedures as Scheme-callable functions
//! that wrap the underlying threading infrastructure.

use super::srfi21_threading::{
    Srfi21Thread, Srfi21Mutex, Srfi21CondVar, Srfi21Time, Srfi21Error,
    ThreadId, ThreadState, ThreadPriority, ThreadRegistry, THREAD_REGISTRY,
    WaitingThread, DynamicEnvironment, ThreadExecutionModel, 
    PriorityInheritanceGraph,
};
use crate::diagnostics::{Error, Result};
use crate::eval::{Value, Evaluator, Environment};
use crate::utils::SymbolId;
use num_traits::cast::ToPrimitive;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use std::thread;
use std::rc::Rc;

/// Create a new thread (make-thread thunk [name])
pub fn make_thread(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            format!("make-thread expects 1 or 2 arguments, got {}", args.len()),
            None
        )));
    }

    let thunk = args[0].clone();
    let name = if args.len() > 1 {
        match &args[1] {
            Value::String(s) => Some(s.to_string()),
            Value::Symbol(s) => Some(format!("{:?}", s)),
            _ => return Err(Box::new(Error::type_error("make-thread", "string or symbol", &args[1]))),
        }
    } else {
        None
    };

    // Create new thread with default priority (128) and no quantum limit
    let thread = Arc::new(Srfi21Thread::new(name, 128, None));
    
    // Store the thunk for later execution when thread is started
    thread.specific_storage.write().unwrap()
        .insert("__thunk__".to_string(), thunk);

    // Register thread
    THREAD_REGISTRY.register_thread(thread.clone())?;

    Ok(Value::Thread(thread))
}

/// Start a thread (thread-start! thread [scheduler])
pub fn thread_start(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            format!("thread-start! expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let thread = match &args[0] {
        Value::Thread(t) => t.clone(),
        _ => return Err(Box::new(Error::type_error("thread-start!", "thread", &args[0]))),
    };

    // Check thread state
    if thread.state() != ThreadState::New {
        return Err(Srfi21Error::ThreadAlreadyStarted(thread.id()).into());
    }

    // Get the thunk to execute
    let thunk = thread.specific_storage.read().unwrap()
        .get("__thunk__")
        .cloned()
        .ok_or_else(|| Error::runtime_error("Thread has no thunk to execute".to_string(), None))?;

    // Set thread state to runnable
    thread.set_state(ThreadState::Runnable);

    // Set current thread in registry when starting
    let thread_id = thread.id();
    let thread_clone = thread.clone();
    let terminate_flag = thread.terminate_flag.clone();

    // Spawn the actual OS thread
    let handle = thread::spawn(move || -> Result<Value> {
        // Set this as current thread
        THREAD_REGISTRY.set_current_thread(thread_id);

        // Execute the thunk
        let result = match &thunk {
            Value::Procedure(proc) => {
                // Call the procedure with no arguments
                let env = Environment::new();
                let mut evaluator = Evaluator::new();
                evaluator.call_procedure(proc.clone(), &[], &env)
            }
            Value::Closure(_) => {
                // Call the closure with no arguments
                let env = Environment::new();
                let mut evaluator = Evaluator::new();
                evaluator.eval(&Value::List(Rc::new(vec![thunk])), &env)
            }
            _ => Err(Box::new(Error::type_error("thread-start!", "procedure", &thunk))),
        };

        // Check if termination was requested
        if terminate_flag.load(std::sync::atomic::Ordering::Acquire) {
            return Err(Srfi21Error::ThreadTerminated(thread_id).into());
        }

        // Set final state based on result
        match &result {
            Ok(_) => thread_clone.set_state(ThreadState::Terminated),
            Err(_) => thread_clone.set_state(ThreadState::Killed),
        }

        result
    });

    // Update thread model with the handle
    match &thread.model {
        ThreadExecutionModel::NativeThread(_) => {
            // This is a bit tricky - we need to update the model
            // In a real implementation, we'd need interior mutability here
            // For now, we'll store the handle in thread-specific storage
            thread.specific_storage.write().unwrap()
                .insert("__handle__".to_string(), Value::String("native_thread_handle".to_string()));
        }
        #[cfg(feature = "async-runtime")]
        _ => {
            // For async tasks, we'd create tokio tasks here
            // Implementation would depend on the chosen executor model
        }
    }

    Ok(args[0].clone())
}

/// Get current thread (current-thread)
pub fn current_thread(_args: &[Value]) -> Result<Value> {
    let thread_id = THREAD_REGISTRY.current_thread_id()
        .ok_or_else(|| Error::runtime_error("No current thread".to_string(), None))?;

    let thread = THREAD_REGISTRY.get_thread(thread_id)
        .ok_or_else(|| Srfi21Error::ThreadNotFound(thread_id))?;

    Ok(Value::Thread(thread))
}

/// Yield current thread (thread-yield! [scheduler])
pub fn thread_yield(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(Box::new(Error::runtime_error(
            format!("thread-yield! expects 0 or 1 arguments, got {}", args.len()),
            None
        ));
    }

    // Yield to the OS scheduler
    thread::yield_now();

    Ok(Value::Unspecified)
}

/// Terminate a thread (thread-terminate! thread)
pub fn thread_terminate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("thread-terminate! expects 1 argument, got {}", args.len()),
            None
        ));
    }

    let thread = match &args[0] {
        Value::Thread(t) => t.clone(),
        _ => return Err(Box::new(Error::type_error("thread-terminate!", "thread", &args[0])),
    };

    // Request termination
    thread.request_termination();
    thread.set_state(ThreadState::Killed);

    // Unregister from global registry
    THREAD_REGISTRY.unregister_thread(thread.id())?;

    Ok(Value::Unspecified)
}

/// Sleep for specified time (thread-sleep! timeout)
pub fn thread_sleep(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("thread-sleep! expects 1 argument, got {}", args.len()),
            None
        ));
    }

    let timeout_secs = match &args[0] {
        Value::number(n) => n.to_f64().ok_or_else(|| {
            Error::type_error("thread-sleep!", "number", &args[0])
        })?,
        _ => return Err(Box::new(Error::type_error("thread-sleep!", "number", &args[0])),
    };

    if timeout_secs < 0.0 {
        return Err(Box::new(Error::runtime_error("Sleep timeout cannot be negative".to_string(), None));
    }

    let duration = Duration::from_secs_f64(timeout_secs);
    thread::sleep(duration);

    Ok(Value::Unspecified)
}

/// Join a thread and get its result (thread-join! thread [timeout [timeout-val]])
pub fn thread_join(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(Error::runtime_error(
            format!("thread-join! expects 1 to 3 arguments, got {}", args.len()),
            None
        ));
    }

    let thread = match &args[0] {
        Value::Thread(t) => t.clone(),
        _ => return Err(Box::new(Error::type_error("thread-join!", "thread", &args[0])),
    };

    let timeout = if args.len() > 1 {
        match &args[1] {
            Value::number(n) => {
                let timeout_secs = n.to_f64().ok_or_else(|| {
                    Error::type_error("thread-join!", "number", &args[1])
                })?;
                if timeout_secs < 0.0 {
                    return Err(Box::new(Error::runtime_error("Timeout cannot be negative".to_string(), None));
                }
                Some(Duration::from_secs_f64(timeout_secs)))
            }
            _ => return Err(Box::new(Error::type_error("thread-join!", "number", &args[1])),
        }
    } else {
        None
    };

    let timeout_val = if args.len() > 2 {
        Some(args[2].clone()))
    } else {
        None
    };

    // In a real implementation, we'd wait for the thread to complete
    // For now, we'll check the thread state and return appropriately
    match thread.state() {
        ThreadState::New | ThreadState::Runnable => {
            // Thread still running - in real implementation would wait
            if timeout.is_some() && timeout_val.is_some() {
                Ok(timeout_val.unwrap()))
            } else {
                Err(Box::new(Error::runtime_error("Thread still running".to_string(), None)))
            }
        }
        ThreadState::Blocked => {
            // Thread blocked - in real implementation would wait
            if timeout.is_some() && timeout_val.is_some() {
                Ok(timeout_val.unwrap()))
            } else {
                Err(Box::new(Error::runtime_error("Thread blocked".to_string(), None)))
            }
        }
        ThreadState::Terminated => {
            // Thread completed successfully - return its result
            // In real implementation, would get actual result from thread
            Ok(Value::Unspecified)
        }
        ThreadState::Killed => {
            // Thread was terminated
            Err(Srfi21Error::ThreadTerminated(thread.id()).into()))
        }
    }
}

/// Create a mutex (make-mutex [name])
pub fn make_mutex(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(Box::new(Error::runtime_error(
            format!("make-mutex expects 0 or 1 arguments, got {}", args.len()),
            None
        ));
    }

    let name = if args.len() == 1 {
        match &args[0] {
            Value::String(s) => Some(s.to_string()),
            Value::Symbol(s) => Some(format!("{:?}", s)),
            _ => return Err(Box::new(Error::type_error("make-mutex", "string or symbol", &args[0])),
        }
    } else {
        None
    };

    let mutex = Srfi21Mutex::new(name, true); // Enable priority inheritance
    Ok(Value::Mutex(Arc::new(mutex))))
}

/// Lock a mutex (mutex-lock! mutex [timeout [thread]])
pub fn mutex_lock(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(Error::runtime_error(
            format!("mutex-lock! expects 1 to 3 arguments, got {}", args.len()),
            None
        ));
    }

    let mutex = match &args[0] {
        Value::Mutex(m) => m.clone(),
        _ => return Err(Box::new(Error::type_error("mutex-lock!", "mutex", &args[0])),
    };

    let timeout = if args.len() > 1 {
        match &args[1] {
            Value::number(n) => {
                let timeout_secs = n.to_f64().ok_or_else(|| {
                    Error::type_error("mutex-lock!", "number", &args[1])
                })?;
                if timeout_secs < 0.0 {
                    return Err(Box::new(Error::runtime_error("Timeout cannot be negative".to_string(), None));
                }
                Some(Duration::from_secs_f64(timeout_secs)))
            }
            Value::Boolean(false) => None, // #f means no timeout
            _ => return Err(Box::new(Error::type_error("mutex-lock!", "number or #f", &args[1])),
        }
    } else {
        None
    };

    let thread = if args.len() > 2 {
        match &args[2] {
            Value::Thread(t) => Some(t.clone()),
            _ => return Err(Box::new(Error::type_error("mutex-lock!", "thread", &args[2])),
        }
    } else {
        // Get current thread
        let thread_id = THREAD_REGISTRY.current_thread_id()
            .ok_or_else(|| Error::runtime_error("No current thread".to_string(), None))?;
        THREAD_REGISTRY.get_thread(thread_id)
    };

    // Attempt to acquire the lock
    // In a real implementation, this would handle priority inheritance
    let _guard = if let Some(timeout_duration) = timeout {
        // Try to lock with timeout (simplified - real implementation would be more complex)
        mutex.inner.try_lock().map_err(|_| Srfi21Error::MutexTimeout)?
    } else {
        // Block until lock is acquired
        mutex.inner.lock().map_err(|_| {
            Error::runtime_error("Mutex lock failed".to_string(), None)
        })?
    };

    // Set mutex owner
    if let Some(ref t) = thread {
        *mutex.owner.write().unwrap() = Some(t.id());
    }

    Ok(Value::Boolean(true)))
}

/// Unlock a mutex (mutex-unlock! mutex [condvar [timeout]])
pub fn mutex_unlock(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(Error::runtime_error(
            format!("mutex-unlock! expects 1 to 3 arguments, got {}", args.len()),
            None
        ));
    }

    let mutex = match &args[0] {
        Value::Mutex(m) => m.clone(),
        _ => return Err(Box::new(Error::type_error("mutex-unlock!", "mutex", &args[0])),
    };

    let condvar = if args.len() > 1 {
        match &args[1] {
            Value::ConditionVariable(cv) => Some(cv.clone()),
            _ => return Err(Box::new(Error::type_error("mutex-unlock!", "condition-variable", &args[1])),
        }
    } else {
        None
    };

    let timeout = if args.len() > 2 {
        match &args[2] {
            Value::number(n) => {
                let timeout_secs = n.to_f64().ok_or_else(|| {
                    Error::type_error("mutex-unlock!", "number", &args[2])
                })?;
                if timeout_secs < 0.0 {
                    return Err(Box::new(Error::runtime_error("Timeout cannot be negative".to_string(), None));
                }
                Some(Duration::from_secs_f64(timeout_secs)))
            }
            _ => return Err(Box::new(Error::type_error("mutex-unlock!", "number", &args[2])),
        }
    } else {
        None
    };

    // Clear mutex owner
    *mutex.owner.write().unwrap() = None;

    // If condvar is provided, wait on it after unlocking
    if let Some(cv) = condvar {
        if let Some(timeout_duration) = timeout {
            // Wait with timeout
            let _result = cv.inner.wait_timeout_while(
                mutex.inner.lock().unwrap(),
                timeout_duration,
                |_| true // Always wait
            ).map_err(|_| Srfi21Error::CondVarTimeout)?;
        } else {
            // Wait indefinitely
            let _result = cv.inner.wait_while(
                mutex.inner.lock().unwrap(),
                |_| true // Always wait
            ).map_err(|_| Error::runtime_error("Condition variable wait failed".to_string(), None))?;
        }
    }

    // Note: In real implementation, mutex would be automatically unlocked
    // when the lock guard goes out of scope

    Ok(Value::Boolean(true)))
}

/// Get mutex state information (mutex-state mutex)
pub fn mutex_state(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("mutex-state expects 1 argument, got {}", args.len()),
            None
        ));
    }

    let mutex = match &args[0] {
        Value::Mutex(m) => m.clone(),
        _ => return Err(Box::new(Error::type_error("mutex-state", "mutex", &args[0])),
    };

    // Return mutex owner or #f if not owned
    match mutex.owner() {
        Some(thread_id) => {
            if let Some(thread) = THREAD_REGISTRY.get_thread(thread_id) {
                Ok(Value::Thread(thread)))
            } else {
                Ok(Value::Boolean(false)))
            }
        }
        None => Ok(Value::Symbol(SymbolId::new("not-owned"))),
    }
}

/// Create a condition variable (make-condition-variable [name])
pub fn make_condition_variable(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(Box::new(Error::runtime_error(
            format!("make-condition-variable expects 0 or 1 arguments, got {}", args.len()),
            None
        ));
    }

    let name = if args.len() == 1 {
        match &args[0] {
            Value::String(s) => Some(s.to_string()),
            Value::Symbol(s) => Some(format!("{:?}", s)),
            _ => return Err(Box::new(Error::type_error("make-condition-variable", "string or symbol", &args[0])),
        }
    } else {
        None
    };

    let condvar = Srfi21CondVar::new(name);
    Ok(Value::ConditionVariable(Arc::new(condvar))))
}

/// Signal a condition variable (condition-variable-signal! condvar)
pub fn condition_variable_signal(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("condition-variable-signal! expects 1 argument, got {}", args.len()),
            None
        ));
    }

    let condvar = match &args[0] {
        Value::ConditionVariable(cv) => cv.clone(),
        _ => return Err(Box::new(Error::type_error("condition-variable-signal!", "condition-variable", &args[0])),
    };

    // Notify one waiting thread (highest priority first)
    condvar.inner.notify_one();

    // In a real implementation, we'd handle priority-based notification
    // using the priority_queue to select which thread to wake

    Ok(Value::Unspecified)
}

/// Broadcast to all waiting threads (condition-variable-broadcast! condvar)
pub fn condition_variable_broadcast(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("condition-variable-broadcast! expects 1 argument, got {}", args.len()),
            None
        ));
    }

    let condvar = match &args[0] {
        Value::ConditionVariable(cv) => cv.clone(),
        _ => return Err(Box::new(Error::type_error("condition-variable-broadcast!", "condition-variable", &args[0])),
    };

    // Notify all waiting threads
    condvar.inner.notify_all();

    Ok(Value::Unspecified)
}

/// Get current time (current-time)
pub fn current_time(_args: &[Value]) -> Result<Value> {
    let time = Srfi21Time::now();
    Ok(Value::Time21(time)))
}

/// Convert time to seconds (time->seconds time)
pub fn time_to_seconds(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("time->seconds expects 1 argument, got {}", args.len()),
            None
        ));
    }

    let time = match &args[0] {
        Value::Time21(t) => *t,
        _ => return Err(Box::new(Error::type_error("time->seconds", "time", &args[0])),
    };

    Ok(Value::number(time.to_seconds().into())))
}

/// Convert seconds to time (seconds->time seconds)
pub fn seconds_to_time(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("seconds->time expects 1 argument, got {}", args.len()),
            None
        ));
    }

    let seconds = match &args[0] {
        Value::number(n) => n.to_f64().ok_or_else(|| {
            Error::type_error("seconds->time", "number", &args[0])
        })?,
        _ => return Err(Box::new(Error::type_error("seconds->time", "number", &args[0])),
    };

    if seconds < 0.0 {
        return Err(Box::new(Error::runtime_error("Time cannot be negative".to_string(), None));
    }

    let time = Srfi21Time::from_seconds(seconds);
    Ok(Value::Time21(time)))
}

/// Thread predicates

/// Check if value is a thread (thread? obj)
pub fn is_thread(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("thread? expects 1 argument, got {}", args.len()),
            None
        ));
    }

    match &args[0] {
        Value::Thread(_) => Ok(Value::Boolean(true)),
        _ => Ok(Value::Boolean(false)),
    }
}

/// Check if value is a mutex (mutex? obj)
pub fn is_mutex(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("mutex? expects 1 argument, got {}", args.len()),
            None
        ));
    }

    match &args[0] {
        Value::Mutex(_) => Ok(Value::Boolean(true)),
        _ => Ok(Value::Boolean(false)),
    }
}

/// Check if value is a condition variable (condition-variable? obj)
pub fn is_condition_variable(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("condition-variable? expects 1 argument, got {}", args.len()),
            None
        ));
    }

    match &args[0] {
        Value::ConditionVariable(_) => Ok(Value::Boolean(true)),
        _ => Ok(Value::Boolean(false)),
    }
}

/// Get thread name (thread-name thread)
pub fn thread_name(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("thread-name expects 1 argument, got {}", args.len()),
            None
        ));
    }

    let thread = match &args[0] {
        Value::Thread(t) => t.clone(),
        _ => return Err(Box::new(Error::type_error("thread-name", "thread", &args[0])),
    };

    match thread.name() {
        Some(name) => Ok(Value::String(name.to_string())),
        None => Ok(Value::Boolean(false)),
    }
}

/// Get thread specific storage (thread-specific thread)
pub fn thread_specific(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("thread-specific expects 1 argument, got {}", args.len()),
            None
        ));
    }

    let thread = match &args[0] {
        Value::Thread(t) => t.clone(),
        _ => return Err(Box::new(Error::type_error("thread-specific", "thread", &args[0])),
    };

    // Return thread-specific data (simplified - would normally return the stored value)
    match thread.specific_storage.read().unwrap().get("__specific__") {
        Some(value) => Ok(value.clone()),
        None => Ok(Value::Boolean(false)),
    }
}

/// Set thread specific storage (thread-specific-set! thread obj)
pub fn thread_specific_set(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("thread-specific-set! expects 2 arguments, got {}", args.len()),
            None
        ));
    }

    let thread = match &args[0] {
        Value::Thread(t) => t.clone(),
        _ => return Err(Box::new(Error::type_error("thread-specific-set!", "thread", &args[0])),
    };

    let value = args[1].clone();

    // Store the value in thread-specific storage
    thread.specific_storage.write().unwrap()
        .insert("__specific__".to_string(), value);

    Ok(Value::Unspecified)
}

/// Check if thread terminated (thread-terminated? thread)
pub fn is_thread_terminated(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("thread-terminated? expects 1 argument, got {}", args.len()),
            None
        ));
    }

    let thread = match &args[0] {
        Value::Thread(t) => t.clone(),
        _ => return Err(Box::new(Error::type_error("thread-terminated?", "thread", &args[0])),
    };

    let terminated = matches!(thread.state(), ThreadState::Terminated | ThreadState::Killed);
    Ok(Value::Boolean(terminated)))
}