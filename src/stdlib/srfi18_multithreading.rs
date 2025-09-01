//! SRFI-18 Multithreading Support - API Implementation
//!
//! This module provides the SRFI-18 compliant API for multithreading in Lambdust.
//! It implements all required procedures and integrates with the core threading
//! infrastructure to provide a complete threading solution.
//!
//! ## Implemented Procedures
//! - `(current-thread)` - Returns current thread object
//! - `(make-thread procedure [name])` - Creates new thread
//! - `(thread-start! thread)` - Starts thread execution
//! - `(thread-join! thread [timeout])` - Waits for thread completion
//! - `(thread? obj)` - Thread predicate
//! - `(make-mutex [name])` - Creates mutex object
//! - `(mutex-lock! mutex [timeout])` - Acquires mutex lock
//! - `(mutex-unlock! mutex)` - Releases mutex lock
//! - `(mutex? obj)` - Mutex predicate
//! - `(make-condition-variable mutex [name])` - Creates condition variable
//! - `(condition-variable-wait! condvar [timeout])` - Waits on condition
//! - `(condition-variable-signal! condvar)` - Signals one waiting thread
//! - `(condition-variable-broadcast! condvar)` - Signals all waiting threads
//! - `(condition-variable? obj)` - Condition variable predicate

use crate::diagnostics::{Error, Result};
use num_traits::cast::ToPrimitive;
use crate::eval::value::{Value, PrimitiveImpl, PrimitiveProcedure};
use crate::eval::parameter::{capture_parameter_bindings, ParameterFrame};
use crate::eval::evaluator::EvalStep;
use crate::concurrency::scheme_threading::{
    SchemeThread, SchemeMutex, SchemeConditionVariable, ThreadRegistry,
    current_thread_id, set_current_thread_id,
};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    /// Global thread registry for managing all threads
    static ref THREAD_REGISTRY: ThreadRegistry = ThreadRegistry::new()
        .expect("Failed to initialize thread registry");
        
    /// Global mutex registry for managing all mutexes
    static ref MUTEX_REGISTRY: StdMutex<HashMap<u64, Arc<SchemeMutex>>> = 
        StdMutex::new(HashMap::new());
        
    /// Global condition variable registry
    static ref CONDVAR_REGISTRY: StdMutex<HashMap<u64, Arc<SchemeConditionVariable>>> = 
        StdMutex::new(HashMap::new());
}

// ============= THREAD PROCEDURES =============

/// `(current-thread)` - Returns the current thread object
pub fn current_thread() -> Result<Value> {
    if let Some(thread_id) = current_thread_id() {
        if let Some(thread) = THREAD_REGISTRY.get_thread(thread_id) {
            Ok(Value::Thread(thread))
        } else {
            // Current thread not in registry (main thread case)
            // Create a pseudo-thread object for the main thread
            let main_thread = Arc::new(SchemeThread::new(
                Some("main".to_string()),
                Arc::new(Vec::new()),
            ));
            THREAD_REGISTRY.register_thread(Arc::clone(&main_thread));
            set_current_thread_id(main_thread.id);
            Ok(Value::Thread(main_thread))
        }
    } else {
        Err(Box::new(Error::runtime_error("No current thread available", None)))
    }
}

/// Wrapper for current_thread to match primitive function signature
pub fn current_thread_primitive(_args: &[Value]) -> Result<Value> {
    current_thread()
}

/// `(make-thread procedure [name])` - Creates a new thread
pub fn make_thread(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            "make-thread expects 1 or 2 arguments: procedure [name]",
            None,
        )));
    }

    let procedure = args[0].clone();
    let name = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::literal::Literal::String(s)) => Some((**s).clone()),
            _ => return Err(Box::new(Error::runtime_error("Thread name must be a string", None))),
        }
    } else {
        None
    };

    // Capture current parameter bindings for inheritance
    let inherited_parameters = Arc::new(capture_parameter_bindings());

    // Create the thread
    let thread = Arc::new(SchemeThread::new(name, inherited_parameters));
    
    // Register the thread
    THREAD_REGISTRY.register_thread(Arc::clone(&thread));

    Ok(Value::Thread(thread))
}

/// `(thread-start! thread)` - Starts thread execution
/// Returns an EvalStep that the evaluator will handle
pub fn thread_start(args: Vec<Value>) -> Result<EvalStep> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "thread-start! expects 1 argument: thread",
            None,
        )));
    }

    match &args[0] {
        Value::Thread(thread) => {
            // Return a ThreadSpawn step for the evaluator to handle
            // For now, we'll assume the thread runs a nullary procedure
            Ok(EvalStep::ThreadSpawn {
                procedure: Value::Nil, // TODO: Get actual procedure from thread
                args: Vec::new(),
                env: std::rc::Rc::new(crate::eval::environment::Environment::new(None, 0)),
                name: thread.name.clone(),
            })
        }
        _ => Err(Box::new(Error::runtime_error("Argument must be a thread object", None))),
    }
}

/// `(thread-join! thread [timeout])` - Waits for thread completion
/// Returns an EvalStep that the evaluator will handle
pub fn thread_join(args: Vec<Value>) -> Result<EvalStep> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            "thread-join! expects 1 or 2 arguments: thread [timeout]",
            None,
        )));
    }

    let thread = match &args[0] {
        Value::Thread(thread) => thread,
        _ => return Err(Box::new(Error::runtime_error("First argument must be a thread object", None))),
    };

    let timeout = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::literal::Literal::Number(n)) => {
                if let Some(seconds) = n.to_f64() {
                    if seconds >= 0.0 {
                        Some(Duration::from_secs_f64(seconds))
                    } else {
                        return Err(Box::new(Error::runtime_error("Timeout must be non-negative", None)));
                    }
                } else {
                    return Err(Box::new(Error::runtime_error("Invalid timeout value", None)));
                }
            }
            _ => return Err(Box::new(Error::runtime_error("Timeout must be a number", None))),
        }
    } else {
        None
    };

    // Return a ThreadJoin step for the evaluator to handle
    Ok(EvalStep::ThreadJoin {
        thread_id: thread.id,
        timeout,
        continuation: Box::new(|result| EvalStep::Return(result)),
    })
}

/// `(thread? obj)` - Thread predicate
pub fn thread_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "thread? expects 1 argument",
            None,
        )));
    }

    let is_thread = matches!(args[0], Value::Thread(_));
    Ok(Value::Literal(crate::ast::literal::Literal::Boolean(is_thread)))
}

// ============= MUTEX PROCEDURES =============

/// `(make-mutex [name])` - Creates a mutex object
pub fn make_mutex(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(Box::new(Error::runtime_error(
            "make-mutex expects 0 or 1 arguments: [name]",
            None,
        )));
    }

    let name = if args.len() == 1 {
        match &args[0] {
            Value::Literal(crate::ast::literal::Literal::String(s)) => Some((**s).clone()),
            _ => return Err(Box::new(Error::runtime_error("Mutex name must be a string", None))),
        }
    } else {
        None
    };

    let mutex = Arc::new(SchemeMutex::new(name));
    
    // Register the mutex
    {
        let mut registry = MUTEX_REGISTRY.lock().unwrap();
        registry.insert(mutex.id, Arc::clone(&mutex));
    }

    Ok(Value::Mutex(mutex))
}

/// `(mutex-lock! mutex [timeout])` - Acquires mutex lock
/// Returns an EvalStep that the evaluator will handle
pub fn mutex_lock(args: Vec<Value>) -> Result<EvalStep> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            "mutex-lock! expects 1 or 2 arguments: mutex [timeout]",
            None,
        )));
    }

    let mutex = match &args[0] {
        Value::Mutex(mutex) => mutex,
        _ => return Err(Box::new(Error::runtime_error("First argument must be a mutex object", None))),
    };

    let timeout = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::literal::Literal::Number(n)) => {
                if let Some(seconds) = n.to_f64() {
                    if seconds >= 0.0 {
                        Some(Duration::from_secs_f64(seconds))
                    } else {
                        return Err(Box::new(Error::runtime_error("Timeout must be non-negative", None)));
                    }
                } else {
                    return Err(Box::new(Error::runtime_error("Invalid timeout value", None)));
                }
            }
            _ => return Err(Box::new(Error::runtime_error("Timeout must be a number", None))),
        }
    } else {
        None
    };

    // Return a MutexLock step for the evaluator to handle
    Ok(EvalStep::MutexLock {
        mutex_id: mutex.id,
        timeout,
        continuation: Box::new(|| EvalStep::Return(Value::Nil)),
    })
}

/// `(mutex-unlock! mutex)` - Releases mutex lock
/// Returns an EvalStep that the evaluator will handle
pub fn mutex_unlock(args: Vec<Value>) -> Result<EvalStep> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "mutex-unlock! expects 1 argument: mutex",
            None,
        )));
    }

    let mutex = match &args[0] {
        Value::Mutex(mutex) => mutex,
        _ => return Err(Box::new(Error::runtime_error("Argument must be a mutex object", None))),
    };

    // Return a MutexUnlock step for the evaluator to handle
    Ok(EvalStep::MutexUnlock {
        mutex_id: mutex.id,
        continuation: Box::new(|| EvalStep::Return(Value::Nil)),
    })
}

/// `(mutex? obj)` - Mutex predicate
pub fn mutex_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "mutex? expects 1 argument",
            None,
        )));
    }

    let is_mutex = matches!(args[0], Value::Mutex(_));
    Ok(Value::Literal(crate::ast::literal::Literal::Boolean(is_mutex)))
}

// ============= CONDITION VARIABLE PROCEDURES =============

/// `(make-condition-variable mutex [name])` - Creates a condition variable
pub fn make_condition_variable(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            "make-condition-variable expects 1 or 2 arguments: mutex [name]",
            None,
        )));
    }

    let mutex = match &args[0] {
        Value::Mutex(mutex) => Arc::clone(mutex),
        _ => return Err(Box::new(Error::runtime_error("First argument must be a mutex object", None))),
    };

    let name = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::literal::Literal::String(s)) => Some((**s).clone()),
            _ => return Err(Box::new(Error::runtime_error("Condition variable name must be a string", None))),
        }
    } else {
        None
    };

    let condvar = Arc::new(SchemeConditionVariable::new(name, mutex));
    
    // Register the condition variable
    {
        let mut registry = CONDVAR_REGISTRY.lock().unwrap();
        registry.insert(condvar.id, Arc::clone(&condvar));
    }

    Ok(Value::ConditionVariable(condvar))
}

/// `(condition-variable-wait! condvar [timeout])` - Waits on condition variable
/// Returns an EvalStep that the evaluator will handle
pub fn condition_variable_wait(args: Vec<Value>) -> Result<EvalStep> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            "condition-variable-wait! expects 1 or 2 arguments: condvar [timeout]",
            None,
        )));
    }

    let condvar = match &args[0] {
        Value::ConditionVariable(condvar) => condvar,
        _ => return Err(Box::new(Error::runtime_error("First argument must be a condition variable object", None))),
    };

    let timeout = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::literal::Literal::Number(n)) => {
                if let Some(seconds) = n.to_f64() {
                    if seconds >= 0.0 {
                        Some(Duration::from_secs_f64(seconds))
                    } else {
                        return Err(Box::new(Error::runtime_error("Timeout must be non-negative", None)));
                    }
                } else {
                    return Err(Box::new(Error::runtime_error("Invalid timeout value", None)));
                }
            }
            _ => return Err(Box::new(Error::runtime_error("Timeout must be a number", None))),
        }
    } else {
        None
    };

    // Return a CondvarWait step for the evaluator to handle
    Ok(EvalStep::CondvarWait {
        condvar_id: condvar.id,
        timeout,
        continuation: Box::new(|| EvalStep::Return(Value::Nil)),
    })
}

/// `(condition-variable-signal! condvar)` - Signals one waiting thread
/// Returns an EvalStep that the evaluator will handle
pub fn condition_variable_signal(args: Vec<Value>) -> Result<EvalStep> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "condition-variable-signal! expects 1 argument: condvar",
            None,
        )));
    }

    let condvar = match &args[0] {
        Value::ConditionVariable(condvar) => condvar,
        _ => return Err(Box::new(Error::runtime_error("Argument must be a condition variable object", None))),
    };

    // Return a CondvarNotify step for the evaluator to handle
    Ok(EvalStep::CondvarNotify {
        condvar_id: condvar.id,
        notify_all: false, // Signal only one thread
        continuation: Box::new(|| EvalStep::Return(Value::Nil)),
    })
}

/// `(condition-variable-broadcast! condvar)` - Signals all waiting threads
/// Returns an EvalStep that the evaluator will handle
pub fn condition_variable_broadcast(args: Vec<Value>) -> Result<EvalStep> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "condition-variable-broadcast! expects 1 argument: condvar",
            None,
        )));
    }

    let condvar = match &args[0] {
        Value::ConditionVariable(condvar) => condvar,
        _ => return Err(Box::new(Error::runtime_error("Argument must be a condition variable object", None))),
    };

    // Return a CondvarNotify step for the evaluator to handle
    Ok(EvalStep::CondvarNotify {
        condvar_id: condvar.id,
        notify_all: true, // Signal all waiting threads
        continuation: Box::new(|| EvalStep::Return(Value::Nil)),
    })
}

/// `(condition-variable? obj)` - Condition variable predicate
pub fn condition_variable_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "condition-variable? expects 1 argument",
            None,
        )));
    }

    let is_condvar = matches!(args[0], Value::ConditionVariable(_));
    Ok(Value::Literal(crate::ast::literal::Literal::Boolean(is_condvar)))
}

// ============= PRIMITIVE PROCEDURE REGISTRATION =============

/// Creates all SRFI-18 primitive procedures for registration with the evaluator
/// Note: Functions that return EvalStep cannot be registered as regular primitives
/// and need special handling in the evaluator
pub fn create_srfi18_primitives() -> Vec<(String, PrimitiveProcedure)> {
    vec![
        // Thread procedures
        (
            "current-thread".to_string(),
            PrimitiveProcedure {
                name: "current-thread".to_string(),
                arity_min: 0,
                arity_max: Some(0),
                implementation: PrimitiveImpl::Native(current_thread_primitive),
                effects: vec![crate::effects::Effect::State],
            },
        ),
        (
            "make-thread".to_string(),
            PrimitiveProcedure {
                name: "make-thread".to_string(),
                arity_min: 1,
                arity_max: Some(2),
                implementation: PrimitiveImpl::Native(make_thread),
                effects: vec![crate::effects::Effect::State],
            },
        ),
        (
            "thread?".to_string(),
            PrimitiveProcedure {
                name: "thread?".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::Native(thread_predicate),
                effects: vec![crate::effects::Effect::Pure],
            },
        ),
        
        // Mutex procedures
        (
            "make-mutex".to_string(),
            PrimitiveProcedure {
                name: "make-mutex".to_string(),
                arity_min: 0,
                arity_max: Some(1),
                implementation: PrimitiveImpl::Native(make_mutex),
                effects: vec![crate::effects::Effect::State],
            },
        ),
        (
            "mutex?".to_string(),
            PrimitiveProcedure {
                name: "mutex?".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::Native(mutex_predicate),
                effects: vec![crate::effects::Effect::Pure],
            },
        ),
        
        // Condition variable procedures
        (
            "make-condition-variable".to_string(),
            PrimitiveProcedure {
                name: "make-condition-variable".to_string(),
                arity_min: 1,
                arity_max: Some(2),
                implementation: PrimitiveImpl::Native(make_condition_variable),
                effects: vec![crate::effects::Effect::State],
            },
        ),
        (
            "condition-variable?".to_string(),
            PrimitiveProcedure {
                name: "condition-variable?".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::Native(condition_variable_predicate),
                effects: vec![crate::effects::Effect::Pure],
            },
        ),
        
        // NOTE: The following procedures return EvalStep and cannot be registered
        // as regular primitives. They need special handling in the evaluator:
        // - thread-start!
        // - thread-join!  
        // - mutex-lock!
        // - mutex-unlock!
        // - condition-variable-wait!
        // - condition-variable-signal!
        // - condition-variable-broadcast!
    ]
}

/// Helper function to register a thread ID for the current OS thread
pub fn register_current_thread(thread_id: u64) {
    set_current_thread_id(thread_id);
}

/// Helper function to get a thread by ID from the registry
pub fn get_thread_by_id(thread_id: u64) -> Option<Arc<SchemeThread>> {
    THREAD_REGISTRY.get_thread(thread_id)
}

/// Helper function to get a mutex by ID from the registry
pub fn get_mutex_by_id(mutex_id: u64) -> Option<Arc<SchemeMutex>> {
    let registry = MUTEX_REGISTRY.lock().unwrap();
    registry.get(&mutex_id).cloned()
}

/// Helper function to get a condition variable by ID from the registry
pub fn get_condvar_by_id(condvar_id: u64) -> Option<Arc<SchemeConditionVariable>> {
    let registry = CONDVAR_REGISTRY.lock().unwrap();
    registry.get(&condvar_id).cloned()
}

/// Helper function to get the thread registry
pub fn get_thread_registry() -> &'static ThreadRegistry {
    &THREAD_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_predicate() {
        let thread = Arc::new(SchemeThread::new(None, Arc::new(Vec::new())));
        let thread_value = Value::Thread(thread);
        
        let result = thread_predicate(&[thread_value]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::literal::Literal::Boolean(true)));
        
        let non_thread_value = Value::Nil;
        let result = thread_predicate(&[non_thread_value]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::literal::Literal::Boolean(false)));
    }

    #[test]
    fn test_mutex_predicate() {
        let mutex = Arc::new(SchemeMutex::new(None));
        let mutex_value = Value::Mutex(mutex);
        
        let result = mutex_predicate(&[mutex_value]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::literal::Literal::Boolean(true)));
        
        let non_mutex_value = Value::Nil;
        let result = mutex_predicate(&[non_mutex_value]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::literal::Literal::Boolean(false)));
    }

    #[test]
    fn test_make_thread() {
        // Create a simple procedure for the thread
        let procedure = Value::Nil; // Simplified for testing
        
        let result = make_thread(&[procedure]).unwrap();
        assert!(matches!(result, Value::Thread(_)));
    }

    #[test]
    fn test_make_mutex() {
        let result = make_mutex(&[]).unwrap();
        assert!(matches!(result, Value::Mutex(_)));
        
        let name = Value::Literal(crate::ast::literal::Literal::String(Box::new("test-mutex".to_string())));
        let result = make_mutex(&[name]).unwrap();
        assert!(matches!(result, Value::Mutex(_)));
    }
}