//! Effect event system for tracking and monitoring.

use crate::effects::Effect;
use std::thread::ThreadId;
use std::time::SystemTime;

/// An event in the effect system.
#[derive(Debug, Clone)]
pub struct EffectEvent {
    /// Thread that produced this event
    pub thread_id: ThreadId,
    /// Timestamp of the event
    pub timestamp: SystemTime,
    /// Type of effect event
    pub event_type: EffectEventType,
    /// Associated effect
    pub effect: Effect,
    /// Optional additional context
    pub context: Option<String>,
    /// Sequence number for ordering
    pub sequence: u64,
    /// Dependencies on other effects
    pub dependencies: Vec<u64>,
}

/// Types of effect events.
#[derive(Debug, Clone)]
pub enum EffectEventType {
    /// Effect was activated
    Activated,
    /// Effect was deactivated
    Deactivated,
    /// Effect produced a result
    Produced,
    /// Effect was handled
    Handled,
    /// Effect caused an error
    Error(String),
    /// Effect is waiting for coordination
    WaitingForCoordination,
    /// Effect coordination completed
    CoordinationCompleted,
    /// Effect was rolled back due to transaction failure
    RolledBack,
}