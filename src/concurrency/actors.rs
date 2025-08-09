//! Actor model implementation with fault tolerance and supervision.
//!
//! This module provides a comprehensive actor system with lightweight
//! processes, message passing, supervisor hierarchies, and fault tolerance.

use crate::eval::Value;
use crate::diagnostics::{Error, Result, error::helpers};
use super::ConcurrencyError;
use std::sync::{Arc, Mutex as StdMutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tokio::sync::{mpsc, oneshot};
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for actors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActorId(pub u64);

impl Default for ActorId {
    fn default() -> Self {
        Self::new()
    }
}

impl ActorId {
    /// Creates a new unique actor ID.
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    /// Gets the numeric ID.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ActorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "actor-{}", self.0)
    }
}

/// Message sent between actors.
#[derive(Debug)]
pub struct Message {
    /// Sender of the message
    pub sender: Option<ActorId>,
    /// Message payload
    pub payload: Value,
    /// Message timestamp
    pub timestamp: Instant,
    /// Optional reply channel
    pub reply_to: Option<oneshot::Sender<Value>>,
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender,
            payload: self.payload.clone(),
            timestamp: self.timestamp,
            reply_to: None, // Can't clone the sender, so we set it to None
        }
    }
}

impl Message {
    /// Creates a new message.
    pub fn new(sender: Option<ActorId>, payload: Value) -> Self {
        Self {
            sender,
            payload,
            timestamp: Instant::now(),
            reply_to: None,
        }
    }

    /// Creates a new message with a reply channel.
    pub fn with_reply(sender: Option<ActorId>, payload: Value, reply_to: oneshot::Sender<Value>) -> Self {
        Self {
            sender,
            payload,
            timestamp: Instant::now(),
            reply_to: Some(reply_to),
        }
    }

    /// Sends a reply if a reply channel exists.
    pub fn reply(self, response: Value) -> Result<()> {
        if let Some(reply_to) = self.reply_to {
            reply_to.send(response)
                .map_err(|_| helpers::runtime_error_simple("Failed to send reply"))
        } else {
            Err(helpers::runtime_error_simple("No reply channel available"))
        }
    }
}

/// Actor reference for sending messages.
#[derive(Debug, Clone)]
pub struct ActorRef {
    id: ActorId,
    sender: mpsc::UnboundedSender<Message>,
    system: Arc<ActorSystem>,
}

impl ActorRef {
    /// Gets the actor ID.
    pub fn id(&self) -> ActorId {
        self.id
    }

    /// Sends a message to the actor.
    pub fn tell(&self, message: Value) -> Result<()> {
        let msg = Message::new(None, message);
        self.sender.send(msg)
            .map_err(|_| ConcurrencyError::ActorNotFound(self.id.to_string()).boxed())
    }

    /// Sends a message from another actor.
    pub fn tell_from(&self, sender: ActorId, message: Value) -> Result<()> {
        let msg = Message::new(Some(sender), message);
        self.sender.send(msg)
            .map_err(|_| ConcurrencyError::ActorNotFound(self.id.to_string()).boxed())
    }

    /// Sends a message and waits for a reply.
    pub async fn ask(&self, message: Value, timeout: Duration) -> Result<Value> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let msg = Message::with_reply(None, message, reply_tx);
        
        self.sender.send(msg)
            .map_err(|_| ConcurrencyError::ActorNotFound(self.id.to_string()).boxed())?;

        tokio::time::timeout(timeout, reply_rx)
            .await
            .map_err(|_| ConcurrencyError::Timeout.boxed())?
            .map_err(|_| Error::runtime_error("Actor failed to reply".to_string(), None).boxed())
    }

    /// Checks if the actor is still alive.
    pub fn is_alive(&self) -> bool {
        !self.sender.is_closed()
    }

    /// Stops the actor.
    pub fn stop(&self) -> Result<()> {
        self.tell(Value::symbol_from_str("$stop"))
    }
}

/// Actor behavior trait.
#[async_trait::async_trait]
pub trait Actor: Send + 'static {
    /// Called when the actor receives a message.
    async fn receive(&mut self, message: Message, ctx: &mut ActorContext) -> Result<()>;

    /// Called when the actor starts.
    async fn pre_start(&mut self, _ctx: &mut ActorContext) -> Result<()> {
        Ok(())
    }

    /// Called when the actor stops.
    async fn post_stop(&mut self, _ctx: &mut ActorContext) -> Result<()> {
        Ok(())
    }

    /// Called when the actor restarts after a failure.
    async fn pre_restart(&mut self, _ctx: &mut ActorContext, _error: &Error) -> Result<()> {
        Ok(())
    }

    /// Called after the actor restarts.
    async fn post_restart(&mut self, _ctx: &mut ActorContext) -> Result<()> {
        Ok(())
    }
}

/// Context provided to actors during message processing.
pub struct ActorContext {
    id: ActorId,
    sender: mpsc::UnboundedSender<Message>,
    system: Arc<ActorSystem>,
    parent: Option<ActorRef>,
    children: HashMap<ActorId, ActorRef>,
}

impl ActorContext {
    /// Gets the actor's ID.
    pub fn id(&self) -> ActorId {
        self.id
    }

    /// Gets a reference to self.
    pub fn actor_ref(&self) -> ActorRef {
        ActorRef {
            id: self.id,
            sender: self.sender.clone(),
            system: self.system.clone(),
        }
    }

    /// Gets the parent actor reference.
    pub fn parent(&self) -> Option<&ActorRef> {
        self.parent.as_ref()
    }

    /// Spawns a child actor.
    pub async fn spawn_child<A: Actor>(&mut self, actor: A, name: Option<String>) -> Result<ActorRef> {
        let actor_ref = self.system.spawn_actor(actor, name, Some(self.actor_ref())).await?;
        self.children.insert(actor_ref.id(), actor_ref.clone());
        Ok(actor_ref)
    }

    /// Stops a child actor.
    pub fn stop_child(&mut self, child_id: ActorId) -> Result<()> {
        if let Some(child) = self.children.remove(&child_id) {
            child.stop()
        } else {
            Err(helpers::runtime_error_simple("Child actor not found"))
        }
    }

    /// Gets all child actors.
    pub fn children(&self) -> &HashMap<ActorId, ActorRef> {
        &self.children
    }

    /// Sends a message to another actor.
    pub fn tell(&self, target: &ActorRef, message: Value) -> Result<()> {
        target.tell_from(self.id, message)
    }

    /// Sends a message and waits for a reply.
    pub async fn ask(&self, target: &ActorRef, message: Value, timeout: Duration) -> Result<Value> {
        target.ask(message, timeout).await
    }

    /// Stops the current actor.
    pub fn stop(&self) -> Result<()> {
        self.actor_ref().stop()
    }
}

/// Supervision strategy for handling child actor failures.
#[derive(Debug, Clone)]
pub enum SupervisionStrategy {
    /// Restart the failed actor
    Restart,
    /// Stop the failed actor
    Stop,
    /// Escalate the failure to the parent
    Escalate,
    /// Resume the actor (ignore the failure)
    Resume,
}

/// Actor system configuration.
#[derive(Debug, Clone)]
pub struct ActorSystemConfig {
    /// Default supervision strategy
    pub default_supervision_strategy: SupervisionStrategy,
    /// Maximum number of restarts allowed
    pub max_restarts: usize,
    /// Time window for restart counting
    pub restart_window: Duration,
    /// Default message timeout
    pub default_timeout: Duration,
}

impl Default for ActorSystemConfig {
    fn default() -> Self {
        Self {
            default_supervision_strategy: SupervisionStrategy::Restart,
            max_restarts: 10,
            restart_window: Duration::from_secs(60),
            default_timeout: Duration::from_secs(5),
        }
    }
}

/// Actor system for managing actors.
pub struct ActorSystem {
    config: ActorSystemConfig,
    actors: StdMutex<HashMap<ActorId, ActorInfo>>,
    root_guardian: Option<ActorRef>,
}

impl std::fmt::Debug for ActorSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorSystem")
            .field("config", &self.config)
            .field("actors", &"<HashMap>")
            .field("root_guardian", &self.root_guardian.as_ref().map(|_| "<ActorRef>"))
            .finish()
    }
}

#[derive(Debug)]
struct ActorInfo {
    actor_ref: ActorRef,
    join_handle: JoinHandle<()>,
    parent: Option<ActorId>,
    children: Vec<ActorId>,
    restart_count: usize,
    last_restart: Option<Instant>,
}

impl ActorSystem {
    /// Creates a new actor system.
    pub fn new(config: ActorSystemConfig) -> Arc<Self> {
        Arc::new(Self {
            config,
            actors: StdMutex::new(HashMap::new()),
            root_guardian: None,
        })
    }

    /// Creates a new actor system with default configuration.
    pub fn new_default() -> Arc<Self> {
        Self::new(ActorSystemConfig::default())
    }

    /// Spawns a new actor.
    pub async fn spawn_actor<A: Actor>(
        self: &Arc<Self>,
        mut actor: A,
        _name: Option<String>,
        parent: Option<ActorRef>,
    ) -> Result<ActorRef> {
        let id = ActorId::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        let actor_ref = ActorRef {
            id,
            sender: tx,
            system: self.clone(),
        };

        let mut ctx = ActorContext {
            id,
            sender: actor_ref.sender.clone(),
            system: self.clone(),
            parent: parent.clone(),
            children: HashMap::new(),
        };

        // Start the actor
        actor.pre_start(&mut ctx).await?;

        // Spawn the actor task
        let system = self.clone();
        let join_handle = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                // Check for system messages
                if let Value::Symbol(sym) = &message.payload {
                    if sym.to_string() == "$stop" {
                        break;
                    }
                }

                // Process the message
                if let Err(error) = actor.receive(message, &mut ctx).await {
                    // Handle actor failure
                    system.handle_actor_failure(id, error).await;
                    break;
                }
            }

            // Cleanup
            let _ = actor.post_stop(&mut ctx).await;
        });

        // Register the actor
        let actor_info = ActorInfo {
            actor_ref: actor_ref.clone(),
            join_handle,
            parent: parent.map(|p| p.id()),
            children: Vec::new(),
            restart_count: 0,
            last_restart: None,
        };

        {
            let mut actors = self.actors.lock().unwrap();
            actors.insert(id, actor_info);
        }

        Ok(actor_ref)
    }

    /// Gets an actor reference by ID.
    pub fn get_actor(&self, id: ActorId) -> Option<ActorRef> {
        let actors = self.actors.lock().unwrap();
        actors.get(&id).map(|info| info.actor_ref.clone())
    }

    /// Stops an actor.
    pub async fn stop_actor(&self, id: ActorId) -> Result<()> {
        let actor_info = {
            let mut actors = self.actors.lock().unwrap();
            actors.remove(&id)
        };

        if let Some(info) = actor_info {
            info.actor_ref.stop()?;
            info.join_handle.await
                .map_err(|e| Error::runtime_error(format!("Failed to stop actor: {e}"), None))?;
        }

        Ok(())
    }

    /// Handles actor failure according to supervision strategy.
    async fn handle_actor_failure(&self, actor_id: ActorId, error: Box<Error>) {
        let strategy = self.config.default_supervision_strategy.clone();
        
        match strategy {
            SupervisionStrategy::Restart => {
                // TODO: Implement actor restart logic
                eprintln!("Actor {actor_id} failed with error: {error}. Restarting...");
            }
            SupervisionStrategy::Stop => {
                let _ = self.stop_actor(actor_id).await;
            }
            SupervisionStrategy::Escalate => {
                // TODO: Escalate to parent
                eprintln!("Actor {actor_id} failed with error: {error}. Escalating...");
            }
            SupervisionStrategy::Resume => {
                // Do nothing - let the actor continue
            }
        }
    }

    /// Shuts down the actor system.
    pub async fn shutdown(&self) -> Result<()> {
        let actor_ids: Vec<ActorId> = {
            let actors = self.actors.lock().unwrap();
            actors.keys().copied().collect()
        };

        for id in actor_ids {
            let _ = self.stop_actor(id).await;
        }

        Ok(())
    }
}

/// Example actor implementations.
///
/// A simple echo actor that replies with the same message.
pub struct EchoActor;

#[async_trait::async_trait]
impl Actor for EchoActor {
    async fn receive(&mut self, message: Message, _ctx: &mut ActorContext) -> Result<()> {
        if let Some(reply_to) = message.reply_to {
            let _ = reply_to.send(message.payload);
        }
        Ok(())
    }
}

/// A counter actor that maintains state.
pub struct CounterActor {
    count: i64,
}

impl Default for CounterActor {
    fn default() -> Self {
        Self::new()
    }
}

impl CounterActor {
    /// Creates a new counter actor with count initialized to zero.
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

#[async_trait::async_trait]
impl Actor for CounterActor {
    async fn receive(&mut self, message: Message, _ctx: &mut ActorContext) -> Result<()> {
        match message.payload {
            Value::Symbol(ref sym) if sym.to_string() == "increment" => {
                self.count += 1;
                if let Some(reply_to) = message.reply_to {
                    let _ = reply_to.send(Value::integer(self.count));
                }
            }
            Value::Symbol(ref sym) if sym.to_string() == "decrement" => {
                self.count -= 1;
                if let Some(reply_to) = message.reply_to {
                    let _ = reply_to.send(Value::integer(self.count));
                }
            }
            Value::Symbol(ref sym) if sym.to_string() == "get" => {
                if let Some(reply_to) = message.reply_to {
                    let _ = reply_to.send(Value::integer(self.count));
                }
            }
            _ => {
                return Err(helpers::runtime_error_simple("Unknown message"));
            }
        }
        Ok(())
    }
}

/// A supervisor actor that manages child actors.
pub struct SupervisorActor {
    strategy: SupervisionStrategy,
}

impl SupervisorActor {
    /// Creates a new supervisor actor with the specified supervision strategy.
    pub fn new(strategy: SupervisionStrategy) -> Self {
        Self { strategy }
    }
}

#[async_trait::async_trait]
impl Actor for SupervisorActor {
    async fn receive(&mut self, message: Message, ctx: &mut ActorContext) -> Result<()> {
        match message.payload {
            Value::Symbol(ref sym) if sym.to_string() == "spawn_child" => {
                let child = ctx.spawn_child(EchoActor, None).await?;
                if let Some(reply_to) = message.reply_to {
                    let _ = reply_to.send(Value::integer(child.id().as_u64() as i64));
                }
            }
            _ => {
                // Forward to children or handle supervision logic
            }
        }
        Ok(())
    }
}

/// Global actor system instance.
static GLOBAL_ACTOR_SYSTEM: std::sync::OnceLock<Arc<ActorSystem>> = std::sync::OnceLock::new();

/// Gets the global actor system.
pub fn global_actor_system() -> Arc<ActorSystem> {
    GLOBAL_ACTOR_SYSTEM.get_or_init(ActorSystem::new_default).clone()
}

/// Initializes the actor system.
pub fn initialize() -> Result<()> {
    let _system = global_actor_system();
    Ok(())
}

/// Shuts down the actor system.
pub async fn shutdown() -> Result<()> {
    let system = global_actor_system();
    system.shutdown().await
}

// Helper methods for Value removed - using implementation from value.rs

// Implement necessary traits for SymbolId
impl crate::utils::SymbolId {
    /// Creates a SymbolId from a string (placeholder implementation).
    pub fn from(s: String) -> Self {
        // This is a simplified implementation
        // In practice, you'd use a proper symbol interner
        Self(s.len()) // Placeholder implementation
    }

}