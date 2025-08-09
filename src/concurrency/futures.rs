//! Future/Promise system with async/await paradigm for Scheme.
//!
//! This module provides a comprehensive Future/Promise implementation
//! that integrates with Rust's async ecosystem while providing
//! Scheme-friendly APIs.

use crate::eval::Value;
use crate::diagnostics::{Error, Result};
use super::ConcurrencyError;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use futures::future::{BoxFuture, FutureExt};

/// A Future represents a computation that will complete in the future.
///
/// This is the main future type exposed to Scheme code.
#[derive(Debug, Clone)]
pub struct Future {
    inner: Arc<Mutex<FutureState>>,
}

/// Internal state of a Future.
enum FutureState {
    /// Future is still pending
    Pending(BoxFuture<'static, Result<Value>>),
    /// Future completed successfully
    Resolved(Value),
    /// Future failed with an error
    Rejected(Box<Error>),
}

impl std::fmt::Debug for FutureState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FutureState::Pending(_) => f.debug_tuple("Pending").field(&"<future>").finish(),
            FutureState::Resolved(value) => f.debug_tuple("Resolved").field(value).finish(),
            FutureState::Rejected(error) => f.debug_tuple("Rejected").field(error).finish(),
        }
    }
}

impl Future {
    /// Creates a new future from an async computation.
    pub fn new<F>(future: F) -> Self
    where
        F: std::future::Future<Output = Result<Value>> + Send + 'static,
    {
        Self {
            inner: Arc::new(Mutex::new(FutureState::Pending(future.boxed()))),
        }
    }

    /// Creates an already resolved future.
    pub fn resolved(value: Value) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FutureState::Resolved(value))),
        }
    }

    /// Creates an already rejected future.
    pub fn rejected(error: Error) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FutureState::Rejected(Box::new(error)))),
        }
    }

    /// Creates a future from a promise.
    pub fn from_promise(promise: Promise) -> Self {
        promise.future
    }

    /// Checks if the future is completed (resolved or rejected).
    pub fn is_completed(&self) -> bool {
        let state = self.inner.lock().unwrap();
        matches!(*state, FutureState::Resolved(_) | FutureState::Rejected(_))
    }

    /// Checks if the future is resolved.
    pub fn is_resolved(&self) -> bool {
        let state = self.inner.lock().unwrap();
        matches!(*state, FutureState::Resolved(_))
    }

    /// Checks if the future is rejected.
    pub fn is_rejected(&self) -> bool {
        let state = self.inner.lock().unwrap();
        matches!(*state, FutureState::Rejected(_))
    }

    /// Waits for the future to complete and returns the result.
    pub async fn await_result(&self) -> Result<Value> {
        // Check if already completed
        {
            let mut state = self.inner.lock().unwrap();
            match &mut *state {
                FutureState::Resolved(value) => return Ok(value.clone()),
                FutureState::Rejected(error) => return Err(error.clone()),
                FutureState::Pending(_) => {}
            }
        }

        // Need to await the future
        let mut future_opt = None;
        {
            let mut state = self.inner.lock().unwrap();
            if let FutureState::Pending(future) = &mut *state {
                // Take ownership of the future
                future_opt = Some(std::mem::replace(future, futures::future::pending().boxed()));
            }
        }

        if let Some(future) = future_opt {
            let result = future.await;
            
            // Update state with result
            {
                let mut state = self.inner.lock().unwrap();
                *state = match &result {
                    Ok(value) => FutureState::Resolved(value.clone()),
                    Err(error) => FutureState::Rejected(error.clone()),
                };
            }
            
            result
        } else {
            // Future was completed while we were waiting
            let state = self.inner.lock().unwrap();
            match &*state {
                FutureState::Resolved(value) => Ok(value.clone()),
                FutureState::Rejected(error) => Err(error.clone()),
                FutureState::Pending(_) => unreachable!(),
            }
        }
    }

    /// Waits for the future with a timeout.
    pub async fn await_timeout(&self, duration: Duration) -> Result<Value> {
        match timeout(duration, self.await_result()).await {
            Ok(result) => result,
            Err(_) => Err(ConcurrencyError::Timeout.into()),
        }
    }

    /// Maps the future's value with a function.
    pub fn map<F>(self, f: F) -> Future
    where
        F: FnOnce(Value) -> Result<Value> + Send + 'static,
    {
        let future = async move {
            let value = self.await_result().await?;
            f(value)
        };
        Future::new(future)
    }

    /// Flat-maps the future with a function that returns another future.
    pub fn flat_map<F>(self, f: F) -> Future
    where
        F: FnOnce(Value) -> Future + Send + 'static,
    {
        let future = async move {
            let value = self.await_result().await?;
            f(value).await_result().await
        };
        Future::new(future)
    }

    /// Chains multiple futures together.
    pub fn then<F>(self, f: F) -> Future
    where
        F: FnOnce(Result<Value>) -> Future + Send + 'static,
    {
        let future = async move {
            let result = self.await_result().await;
            f(result).await_result().await
        };
        Future::new(future)
    }

    /// Handles errors in the future.
    pub fn catch<F>(self, f: F) -> Future
    where
        F: FnOnce(Error) -> Result<Value> + Send + 'static,
    {
        let future = async move {
            match self.await_result().await {
                Ok(value) => Ok(value),
                Err(error) => f(*error),
            }
        };
        Future::new(future)
    }
}

/// Promise is the writable side of a Future.
///
/// It allows you to resolve or reject a future manually.
#[derive(Debug)]
pub struct Promise {
    sender: Option<tokio::sync::oneshot::Sender<Result<Value>>>,
    future: Future,
}

impl Promise {
    /// Creates a new promise/future pair.
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        
        let future = Future::new(async move {
            match receiver.await {
                Ok(result) => result,
                Err(_) => Err(ConcurrencyError::Cancelled.into()),
            }
        });

        Self {
            sender: Some(sender),
            future,
        }
    }

    /// Gets the associated future.
    pub fn future(&self) -> Future {
        self.future.clone()
    }

    /// Resolves the promise with a value.
    pub fn resolve(mut self, value: Value) -> Result<()> {
        if let Some(sender) = self.sender.take() {
            sender.send(Ok(value)).map_err(|_| ConcurrencyError::Cancelled.into())
        } else {
            Err(Error::runtime_error("Promise already completed".to_string(), None).into())
        }
    }

    /// Rejects the promise with an error.
    pub fn reject(mut self, error: Error) -> Result<()> {
        if let Some(sender) = self.sender.take() {
            sender.send(Err(error.into())).map_err(|_| ConcurrencyError::Cancelled.into())
        } else {
            Err(Error::runtime_error("Promise already completed".to_string(), None).into())
        }
    }

    /// Checks if the promise is still pending.
    pub fn is_pending(&self) -> bool {
        self.sender.is_some()
    }
}

impl Default for Promise {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for working with futures.
pub struct FutureOps;

impl FutureOps {
    /// Creates a future that resolves after a delay.
    pub fn delay(duration: Duration) -> Future {
        Future::new(async move {
            sleep(duration).await;
            Ok(Value::Unspecified)
        })
    }

    /// Creates a future that resolves with a value after a delay.
    pub fn delay_value(duration: Duration, value: Value) -> Future {
        Future::new(async move {
            sleep(duration).await;
            Ok(value)
        })
    }

    /// Races multiple futures, returning the first one to complete.
    pub fn race(futures: Vec<Future>) -> Future {
        if futures.is_empty() {
            return Future::rejected(Error::runtime_error("No futures to race".to_string(), None));
        }

        Future::new(async move {
            let futures: Vec<_> = futures.into_iter()
                .map(|f| Box::pin(async move { f.await_result().await }))
                .collect();
            
            futures::future::select_all(futures).await.0
        })
    }

    /// Waits for all futures to complete, collecting results.
    pub fn all(futures: Vec<Future>) -> Future {
        Future::new(async move {
            let mut results = Vec::new();
            
            for future in futures {
                results.push(future.await_result().await?);
            }
            
            // Convert to Scheme list
            let mut list = Value::Nil;
            for value in results.into_iter().rev() {
                list = Value::pair(value, list);
            }
            
            Ok(list)
        })
    }

    /// Waits for all futures to settle (complete or fail).
    pub fn all_settled(futures: Vec<Future>) -> Future {
        Future::new(async move {
            let mut results = Vec::new();
            
            for future in futures {
                match future.await_result().await {
                    Ok(value) => {
                        let result = vec![
                            Value::symbol_from_str("fulfilled"),
                            value,
                        ];
                        results.push(Value::from_vec(result));
                    }
                    Err(error) => {
                        let result = vec![
                            Value::symbol_from_str("rejected"),
                            Value::string(error.to_string()),
                        ];
                        results.push(Value::from_vec(result));
                    }
                }
            }
            
            // Convert to Scheme list
            let mut list = Value::Nil;
            for value in results.into_iter().rev() {
                list = Value::pair(value, list);
            }
            
            Ok(list)
        })
    }

    /// Retries a future-producing function with exponential backoff.
    pub fn retry<F, Fut>(f: F, max_attempts: usize, initial_delay: Duration) -> Future
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<Value>> + Send + 'static,
    {
        Future::new(async move {
            let mut attempt = 0;
            let mut delay = initial_delay;
            
            loop {
                attempt += 1;
                
                match f().await {
                    Ok(value) => return Ok(value),
                    Err(error) => {
                        if attempt >= max_attempts {
                            return Err(error);
                        }
                        
                        sleep(delay).await;
                        delay *= 2; // Exponential backoff
                    }
                }
            }
        })
    }
}

/// Extension trait for Value to support future operations.
pub trait ValueFutureExt {
    /// Converts a value to a resolved future.
    fn to_future(self) -> Future;
}

impl ValueFutureExt for Value {
    fn to_future(self) -> Future {
        Future::resolved(self)
    }
}

impl ValueFutureExt for Result<Value> {
    fn to_future(self) -> Future {
        match self {
            Ok(value) => Future::resolved(value),
            Err(error) => Future::rejected(*error),
        }
    }
}

/// Helper trait for creating futures from async functions.
pub trait IntoFuture<T> {
    /// Converts self into a Future.
    fn into_future(self) -> Future;
}

impl<F, Fut> IntoFuture<F> for F
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<Value>> + Send + 'static,
{
    fn into_future(self) -> Future {
        Future::new(self())
    }
}

