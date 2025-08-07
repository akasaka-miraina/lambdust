//! Typed channels for message passing between concurrent tasks.
//!
//! This module provides CSP-style channels with support for
//! bounded/unbounded queues, select operations, and backpressure control.

use crate::eval::Value;
use crate::diagnostics::{Error, Result};
use super::{ConcurrencyError, futures::Future};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, broadcast, watch};
use tokio::time::timeout;
use futures::FutureExt;

/// A typed channel for sending and receiving values.
#[derive(Debug, Clone)]
pub struct Channel {
    sender: ChannelSender,
    receiver: Arc<tokio::sync::Mutex<ChannelReceiver>>,
}

/// Sender half of a channel.
#[derive(Debug, Clone)]
pub struct ChannelSender {
    inner: SenderInner,
}

/// Receiver half of a channel.
#[derive(Debug)]
pub struct ChannelReceiver {
    inner: ReceiverInner,
}

#[derive(Debug, Clone)]
enum SenderInner {
    Bounded(mpsc::Sender<Value>),
    Unbounded(mpsc::UnboundedSender<Value>),
    Broadcast(broadcast::Sender<Value>),
    Watch(watch::Sender<Value>),
}

#[derive(Debug)]
enum ReceiverInner {
    Bounded(mpsc::Receiver<Value>),
    Unbounded(mpsc::UnboundedReceiver<Value>),
    Broadcast(broadcast::Receiver<Value>),
    Watch(watch::Receiver<Value>),
}

/// Channel configuration options.
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    /// Buffer size for bounded channels (None for unbounded)
    pub buffer_size: Option<usize>,
    /// Channel type
    pub channel_type: ChannelType,
    /// Enable backpressure control
    pub backpressure: bool,
}

/// Types of channels available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    /// Multiple producer, single consumer with bounded buffer
    MpscBounded,
    /// Multiple producer, single consumer with unbounded buffer
    MpscUnbounded,
    /// Multiple producer, multiple consumer broadcast
    Broadcast,
    /// Single producer, multiple consumer watch channel
    Watch,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            buffer_size: Some(100),
            channel_type: ChannelType::MpscBounded,
            backpressure: true,
        }
    }
}

impl Channel {
    /// Creates a new channel with the given configuration.
    pub fn new(config: ChannelConfig) -> Result<Self> {
        match config.channel_type {
            ChannelType::MpscBounded => {
                let buffer_size = config.buffer_size
                    .ok_or_else(|| Error::runtime_error("Buffer size required for bounded channel".to_string(), None))?;
                let (tx, rx) = mpsc::channel(buffer_size);
                Ok(Self {
                    sender: ChannelSender { inner: SenderInner::Bounded(tx) },
                    receiver: Arc::new(tokio::sync::Mutex::new(ChannelReceiver { inner: ReceiverInner::Bounded(rx) })),
                })
            }
            ChannelType::MpscUnbounded => {
                let (tx, rx) = mpsc::unbounded_channel();
                Ok(Self {
                    sender: ChannelSender { inner: SenderInner::Unbounded(tx) },
                    receiver: Arc::new(tokio::sync::Mutex::new(ChannelReceiver { inner: ReceiverInner::Unbounded(rx) })),
                })
            }
            ChannelType::Broadcast => {
                let capacity = config.buffer_size.unwrap_or(1000);
                let (tx, rx) = broadcast::channel(capacity);
                Ok(Self {
                    sender: ChannelSender { inner: SenderInner::Broadcast(tx) },
                    receiver: Arc::new(tokio::sync::Mutex::new(ChannelReceiver { inner: ReceiverInner::Broadcast(rx) })),
                })
            }
            ChannelType::Watch => {
                let (tx, rx) = watch::channel(Value::Unspecified);
                Ok(Self {
                    sender: ChannelSender { inner: SenderInner::Watch(tx) },
                    receiver: Arc::new(tokio::sync::Mutex::new(ChannelReceiver { inner: ReceiverInner::Watch(rx) })),
                })
            }
        }
    }

    /// Creates a bounded MPSC channel.
    pub fn bounded(buffer_size: usize) -> Result<Self> {
        Self::new(ChannelConfig {
            buffer_size: Some(buffer_size),
            channel_type: ChannelType::MpscBounded,
            backpressure: true,
        })
    }

    /// Creates an unbounded MPSC channel.
    pub fn unbounded() -> Result<Self> {
        Self::new(ChannelConfig {
            buffer_size: None,
            channel_type: ChannelType::MpscUnbounded,
            backpressure: false,
        })
    }

    /// Creates a broadcast channel.
    pub fn broadcast(capacity: usize) -> Result<Self> {
        Self::new(ChannelConfig {
            buffer_size: Some(capacity),
            channel_type: ChannelType::Broadcast,
            backpressure: false,
        })
    }

    /// Creates a watch channel.
    pub fn watch() -> Result<Self> {
        Self::new(ChannelConfig {
            buffer_size: None,
            channel_type: ChannelType::Watch,
            backpressure: false,
        })
    }

    /// Gets the sender for this channel.
    pub fn sender(&self) -> ChannelSender {
        self.sender.clone())
    }

    /// Gets the receiver for this channel.
    pub fn receiver(&self) -> Arc<tokio::sync::Mutex<ChannelReceiver>> {
        self.receiver.clone())
    }

    /// Creates multiple senders for this channel.
    pub fn senders(&self, count: usize) -> Vec<ChannelSender> {
        (0..count).map(|_| self.sender.clone()).collect()
    }

    /// Creates a receiver subscription for broadcast channels.
    pub fn subscribe(&self) -> Result<ChannelReceiver> {
        match &self.sender.inner {
            SenderInner::Broadcast(tx) => {
                Ok(ChannelReceiver { inner: ReceiverInner::Broadcast(tx.subscribe()) })
            }
            SenderInner::Watch(tx) => {
                Ok(ChannelReceiver { inner: ReceiverInner::Watch(tx.subscribe()) })
            }
            _ => Err(Box::new(Error::runtime_error("Channel type does not support subscriptions".to_string(), None)),
        }
    }
}

impl ChannelSender {
    /// Sends a value through the channel.
    pub async fn send(&self, value: Value) -> Result<()> {
        match &self.inner {
            SenderInner::Bounded(tx) => {
                tx.send(value).await
                    .map_err(|_| ConcurrencyError::ChannelClosed.boxed())
            }
            SenderInner::Unbounded(tx) => {
                tx.send(value)
                    .map_err(|_| ConcurrencyError::ChannelClosed.boxed())
            }
            SenderInner::Broadcast(tx) => {
                tx.send(value)
                    .map(|_| ())
                    .map_err(|_| ConcurrencyError::ChannelClosed.boxed())
            }
            SenderInner::Watch(tx) => {
                tx.send(value)
                    .map_err(|_| ConcurrencyError::ChannelClosed.boxed())
            }
        }
    }

    /// Attempts to send a value without blocking.
    pub fn try_send(&self, value: Value) -> Result<()> {
        match &self.inner {
            SenderInner::Bounded(tx) => {
                tx.try_send(value)
                    .map_err(|e| match e {
                        mpsc::error::TrySendError::Closed(_) => ConcurrencyError::ChannelClosed.into(),
                        mpsc::error::TrySendError::Full(_) => Error::runtime_error("Channel full".to_string(), None),
                    })
            }
            SenderInner::Unbounded(tx) => {
                tx.send(value)
                    .map_err(|_| ConcurrencyError::ChannelClosed.boxed())
            }
            SenderInner::Broadcast(tx) => {
                tx.send(value)
                    .map(|_| ())
                    .map_err(|_| ConcurrencyError::ChannelClosed.boxed())
            }
            SenderInner::Watch(tx) => {
                tx.send(value)
                    .map_err(|_| ConcurrencyError::ChannelClosed.boxed())
            }
        }
    }

    /// Sends a value with a timeout.
    pub async fn send_timeout(&self, value: Value, duration: Duration) -> Result<()> {
        match timeout(duration, self.send(value)).await {
            Ok(result) => result,
            Err(_) => Err(ConcurrencyError::Timeout.into()),
        }
    }

    /// Checks if the channel is closed.
    pub fn is_closed(&self) -> bool {
        match &self.inner {
            SenderInner::Bounded(tx) => tx.is_closed(),
            SenderInner::Unbounded(tx) => tx.is_closed(),
            SenderInner::Broadcast(tx) => tx.receiver_count() == 0,
            SenderInner::Watch(tx) => tx.receiver_count() == 0,
        }
    }

    /// Gets the number of receivers.
    pub fn receiver_count(&self) -> usize {
        match &self.inner {
            SenderInner::Bounded(_) | SenderInner::Unbounded(_) => 1,
            SenderInner::Broadcast(tx) => tx.receiver_count(),
            SenderInner::Watch(tx) => tx.receiver_count(),
        }
    }
}

impl ChannelReceiver {
    /// Receives a value from the channel.
    pub async fn recv(&mut self) -> Result<Value> {
        match &mut self.inner {
            ReceiverInner::Bounded(rx) => {
                rx.recv().await
                    .ok_or_else(|| ConcurrencyError::ChannelClosed.boxed())
            }
            ReceiverInner::Unbounded(rx) => {
                rx.recv().await
                    .ok_or_else(|| ConcurrencyError::ChannelClosed.boxed())
            }
            ReceiverInner::Broadcast(rx) => {
                rx.recv().await
                    .map_err(|e| match e {
                        broadcast::error::RecvError::Closed => ConcurrencyError::ChannelClosed.into(),
                        broadcast::error::RecvError::Lagged(n) => 
                            Error::runtime_error(format!("Lagged behind by {} messages", n), None).into(),
                    })
            }
            ReceiverInner::Watch(rx) => {
                rx.changed().await
                    .map_err(|_| ConcurrencyError::ChannelClosed.boxed())?;
                Ok(rx.borrow().clone())
            }
        }
    }

    /// Attempts to receive a value without blocking.
    pub fn try_recv(&mut self) -> Result<Value> {
        match &mut self.inner {
            ReceiverInner::Bounded(rx) => {
                rx.try_recv()
                    .map_err(|e| match e {
                        mpsc::error::TryRecvError::Empty => Error::runtime_error("Channel empty".to_string(), None),
                        mpsc::error::TryRecvError::Disconnected => ConcurrencyError::ChannelClosed.into(),
                    })
            }
            ReceiverInner::Unbounded(rx) => {
                rx.try_recv()
                    .map_err(|e| match e {
                        mpsc::error::TryRecvError::Empty => Error::runtime_error("Channel empty".to_string(), None),
                        mpsc::error::TryRecvError::Disconnected => ConcurrencyError::ChannelClosed.into(),
                    })
            }
            ReceiverInner::Broadcast(rx) => {
                rx.try_recv()
                    .map_err(|e| match e {
                        broadcast::error::TryRecvError::Empty => Error::runtime_error("Channel empty".to_string(), None),
                        broadcast::error::TryRecvError::Closed => ConcurrencyError::ChannelClosed.into(),
                        broadcast::error::TryRecvError::Lagged(n) => 
                            Error::runtime_error(format!("Lagged behind by {} messages", n), None),
                    })
            }
            ReceiverInner::Watch(rx) => {
                match rx.has_changed() {
                    Ok(true) => Ok(rx.borrow_and_update().clone()),
                    Ok(false) => Err(Box::new(Error::runtime_error("No new value available".to_string(), None)),
                    Err(e) => Err(Box::new(Error::runtime_error(format!("Watch receiver error: {}", e), None)),
                }
            }
        }
    }

    /// Receives a value with a timeout.
    pub async fn recv_timeout(&mut self, duration: Duration) -> Result<Value> {
        match timeout(duration, self.recv()).await {
            Ok(result) => result,
            Err(_) => Err(ConcurrencyError::Timeout.into()),
        }
    }
}

/// Select operation for non-deterministic choice between channels.
pub struct Select {
    futures: Vec<SelectBranch>,
}

struct SelectBranch {
    future: Future,
    id: usize,
}

impl Select {
    /// Creates a new select operation.
    pub fn new() -> Self {
        Self {
            futures: Vec::new(),
        }
    }

    /// Adds a receive operation to the select.
    pub fn recv(mut self, id: usize, receiver: Arc<tokio::sync::Mutex<ChannelReceiver>>) -> Self {
        let future = Future::new(async move {
            let mut rx = receiver.lock().await;
            let value = rx.recv().await?;
            Ok(Value::from_vec(vec![
                Value::symbol_from_str("recv"),
                Value::integer(id as i64),
                value,
            ]))
        });
        
        self.futures.push(SelectBranch { future, id });
        self
    }

    /// Adds a send operation to the select.
    pub fn send(mut self, id: usize, sender: ChannelSender, value: Value) -> Self {
        let future = Future::new(async move {
            sender.send(value).await?;
            Ok(Value::from_vec(vec![
                Value::symbol_from_str("send"),
                Value::integer(id as i64),
                Value::Unspecified,
            ]))
        });
        
        self.futures.push(SelectBranch { future, id });
        self
    }

    /// Adds a timeout to the select.
    pub fn timeout(mut self, id: usize, duration: Duration) -> Self {
        let future = Future::new(async move {
            tokio::time::sleep(duration).await;
            Ok(Value::from_vec(vec![
                Value::symbol_from_str("timeout"),
                Value::integer(id as i64),
                Value::Unspecified,
            ]))
        });
        
        self.futures.push(SelectBranch { future, id });
        self
    }

    /// Executes the select operation.
    pub async fn execute(self) -> Result<Value> {
        if self.futures.is_empty() {
            return Err(Box::new(Error::runtime_error("No operations in select".to_string(), None).into())
        }

        let futures: Vec<_> = self.futures.into_iter()
            .map(|branch| async move { branch.future.await_result().await }.boxed())
            .collect();
        
        futures::future::select_all(futures).await.0
    }
}

impl Default for Select {
    fn default() -> Self {
        Self::new()
    }
}


/// Channel utilities and convenience functions.
pub struct ChannelOps;

impl ChannelOps {
    /// Creates a channel from a Scheme specification.
    pub fn from_spec(_spec: Value) -> Result<Channel> {
        // Parse channel specification from Scheme values
        // This would be implemented based on the specific API design
        let config = ChannelConfig::default();
        Channel::new(config)
    }

    /// Creates a pipeline of channels connected by transformations.
    pub fn pipeline(stages: Vec<Box<dyn Fn(Value) -> Result<Value> + Send + Sync>>) -> Result<(ChannelSender, Arc<tokio::sync::Mutex<ChannelReceiver>>)> {
        if stages.is_empty() {
            return Err(Box::new(Error::runtime_error("Empty pipeline".to_string(), None));
        }

        let first_channel = Channel::unbounded()?;
        let mut current_receiver = first_channel.receiver();
        let stages_len = stages.len();
        
        for (i, stage) in stages.into_iter().enumerate() {
            if i == stages_len - 1 {
                // Last stage - return the current receiver
                break;
            }
            
            let next_channel = Channel::unbounded()?;
            let next_sender = next_channel.sender();
            let next_receiver = next_channel.receiver();
            
            // Spawn a task to process this stage
            tokio::spawn(async move {
                loop {
                    let mut rx = current_receiver.lock().await;
                    match rx.recv().await {
                        Ok(value) => {
                            match stage(value) {
                                Ok(transformed) => {
                                    if let Err(_) = next_sender.send(transformed).await {
                                        break; // Next stage closed
                                    }
                                }
                                Err(_) => break, // Error in transformation
                            }
                        }
                        Err(_) => break, // Input closed
                    }
                }
            });
            
            current_receiver = next_receiver;
        }
        
        Ok((first_channel.sender(), current_receiver))
    }
}