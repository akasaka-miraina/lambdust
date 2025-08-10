//! Effect transaction system for coordinating complex operations.

use crate::effects::Effect;
use std::thread::ThreadId;
use std::time::{SystemTime, Duration};

/// Transaction for coordinating effects.
#[derive(Debug, Clone)]
pub struct EffectTransaction {
    /// Unique transaction ID
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub id: u64,
    /// Thread that initiated the transaction
    pub initiator_thread: ThreadId,
    /// Participating threads
    pub participating_threads: Vec<ThreadId>,
    /// Effects involved in this transaction
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub effects: Vec<Effect>,
    /// Transaction state
    pub state: TransactionState,
    /// Creation timestamp
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub created_at: SystemTime,
    /// Timeout for this transaction
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub timeout: Duration,
}

/// State of an effect transaction.
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    /// Transaction is being prepared
    Preparing,
    /// Transaction is active
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    Active,
    /// Transaction is committing
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    Committing,
    /// Transaction committed successfully
    Committed,
    /// Transaction is aborting
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    Aborting,
    /// Transaction was aborted
    Aborted,
}