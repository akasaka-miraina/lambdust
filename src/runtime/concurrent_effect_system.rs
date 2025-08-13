//! Concurrent effect system for coordinating effects across threads.

use crate::effects::Effect;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::ThreadId;
use std::time::{SystemTime, Duration};
use super::effect_transaction::{EffectTransaction, TransactionState};
use super::effect_dependency_graph::EffectDependencyGraph;

/// Concurrent effect system for coordinating effects across threads.
#[derive(Debug)]
pub struct ConcurrentEffectSystem {
    /// Active effect transactions
    active_transactions: Arc<RwLock<HashMap<u64, EffectTransaction>>>,
    /// Transaction sequence counter
    transaction_sequence: AtomicU64,
    /// Effect dependency graph
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    dependency_graph: Arc<RwLock<EffectDependencyGraph>>,
}

impl ConcurrentEffectSystem {
    /// Creates a new concurrent effect system.
    pub fn new() -> Self {
        Self {
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            transaction_sequence: AtomicU64::new(0),
            dependency_graph: Arc::new(RwLock::new(EffectDependencyGraph::default())),
        }
    }
}

impl Default for ConcurrentEffectSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ConcurrentEffectSystem {
    /// Starts a new effect transaction.
    pub fn start_transaction(
        &self,
        initiator: ThreadId,
        participants: Vec<ThreadId>,
        effects: Vec<Effect>,
    ) -> Result<u64, String> {
        let id = self.transaction_sequence.fetch_add(1, Ordering::SeqCst);
        
        let transaction = EffectTransaction {
            id,
            initiator_thread: initiator,
            participating_threads: participants,
            effects,
            state: TransactionState::Preparing,
            created_at: SystemTime::now(),
            timeout: Duration::from_secs(30),
        };
        
        let mut transactions = self.active_transactions.write().unwrap();
        transactions.insert(id, transaction);
        
        Ok(id)
    }
    
    /// Commits a transaction.
    pub fn commit_transaction(&self, transaction_id: u64) -> Result<(), String> {
        let mut transactions = self.active_transactions.write().unwrap();
        
        if let Some(transaction) = transactions.get_mut(&transaction_id) {
            transaction.state = TransactionState::Committed;
            Ok(())
        } else {
            Err(format!("Transaction {transaction_id} not found"))
        }
    }
    
    /// Aborts a transaction.
    pub fn abort_transaction(&self, transaction_id: u64) -> Result<(), String> {
        let mut transactions = self.active_transactions.write().unwrap();
        
        if let Some(transaction) = transactions.get_mut(&transaction_id) {
            transaction.state = TransactionState::Aborted;
            Ok(())
        } else {
            Err(format!("Transaction {transaction_id} not found"))
        }
    }
    
    /// Waits for coordination completion.
    pub fn wait_for_coordination_completion(
        &self,
        transaction_id: u64,
        timeout: Duration,
    ) -> Result<bool, String> {
        let start_time = SystemTime::now();
        
        loop {
            {
                let transactions = self.active_transactions.read().unwrap();
                if let Some(transaction) = transactions.get(&transaction_id) {
                    match transaction.state {
                        TransactionState::Committed => return Ok(true),
                        TransactionState::Aborted => return Ok(false),
                        _ => {
                            // Continue waiting
                        }
                    }
                } else {
                    return Err(format!("Transaction {transaction_id} not found"));
                }
            }
            
            if start_time.elapsed().unwrap_or(Duration::from_secs(0)) > timeout {
                return Err("Coordination timeout".to_string());
            }
            
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    
    /// Cleans up transactions for a thread.
    pub fn cleanup_thread_transactions(&self, thread_id: ThreadId) {
        let mut transactions = self.active_transactions.write().unwrap();
        transactions.retain(|_, transaction| {
            transaction.initiator_thread != thread_id 
                && !transaction.participating_threads.contains(&thread_id)
        });
    }
}