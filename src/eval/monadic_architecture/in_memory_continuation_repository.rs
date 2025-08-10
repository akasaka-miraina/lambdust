//! In-memory implementation of continuation repository

use crate::eval::continuation_domain::{CapturedContinuation, ContinuationId};
use crate::diagnostics::{Result, Error};
use std::collections::HashMap;

use super::{
    continuation_repository::ContinuationRepository,
    repository_configuration::RepositoryConfiguration,
};

/// In-memory implementation of continuation repository
#[derive(Debug)]
pub struct InMemoryContinuationRepository {
    /// Storage for continuations
    continuations: HashMap<ContinuationId, CapturedContinuation>,
    
    /// Configuration
    config: RepositoryConfiguration,
}

impl ContinuationRepository for InMemoryContinuationRepository {
    fn store(&mut self, continuation: CapturedContinuation) -> Result<ContinuationId> {
        let id = continuation.id;
        
        // Check if we're at capacity
        if self.continuations.len() >= self.config.max_continuations {
            if self.config.auto_gc_enabled {
                self.garbage_collect(0)?; // Force GC
            } else {
                return Err(Box::new(Error::runtime_error(
                    "Continuation repository at capacity".to_string(),
                    None,
                )));
            }
        }
        
        self.continuations.insert(id, continuation);
        Ok(id)
    }
    
    fn find_by_id(&self, id: ContinuationId) -> Option<CapturedContinuation> {
        self.continuations.get(&id).cloned()
    }
    
    fn remove(&mut self, id: ContinuationId) -> Result<()> {
        self.continuations.remove(&id);
        Ok(())
    }
    
    fn list_all(&self) -> Vec<ContinuationId> {
        self.continuations.keys().copied().collect()
    }
    
    fn garbage_collect(&mut self, current_generation: u64) -> Result<usize> {
        let initial_count = self.continuations.len();
        
        // Remove continuations that are too old or already invoked
        self.continuations.retain(|_id, cont| {
            !cont.is_invoked && 
            (current_generation.saturating_sub(cont.metadata.generation) <= self.config.gc_threshold)
        });
        
        let collected = initial_count - self.continuations.len();
        Ok(collected)
    }
}