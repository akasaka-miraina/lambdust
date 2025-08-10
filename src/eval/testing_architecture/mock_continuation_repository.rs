use crate::eval::continuation_domain::{
    CapturedContinuation, ContinuationId, ContinuationRepository,
};
use crate::diagnostics::{Result, Error};
use super::{MockRepositoryBehavior, RepositoryCall};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock continuation repository for testing
#[derive(Debug, Default)]
pub struct MockContinuationRepository {
    /// Mock storage
    storage: Arc<Mutex<HashMap<ContinuationId, CapturedContinuation>>>,
    
    /// Mock behavior configuration
    behavior: MockRepositoryBehavior,
    
    /// Call tracking
    call_log: Arc<Mutex<Vec<RepositoryCall>>>,
}

impl MockContinuationRepository {
    /// Create a new mock repository
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            behavior: MockRepositoryBehavior::default(),
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Create with custom behavior
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn with_behavior(behavior: MockRepositoryBehavior) -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            behavior,
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get call log for assertions
    pub fn call_log(&self) -> Vec<RepositoryCall> {
        self.call_log.lock().unwrap().clone()
    }
    
    /// Clear call log
    pub fn clear_call_log(&self) {
        self.call_log.lock().unwrap().clear();
    }
    
    /// Get number of stored continuations
    pub fn storage_size(&self) -> usize {
        self.storage.lock().unwrap().len()
    }
}

impl ContinuationRepository for MockContinuationRepository {
    fn store(&mut self, continuation: CapturedContinuation) -> Result<ContinuationId> {
        // Log the call
        self.call_log.lock().unwrap().push(RepositoryCall::Store(continuation.id));
        
        // Simulate failure if configured
        if self.behavior.store_should_fail {
            return Err(Box::new(Error::runtime_error(
                "Mock repository configured to fail".to_string(),
                None,
            )));
        }
        
        // Check capacity
        if let Some(max_cap) = self.behavior.max_capacity {
            if self.storage.lock().unwrap().len() >= max_cap {
                return Err(Box::new(Error::runtime_error(
                    "Mock repository at capacity".to_string(),
                    None,
                )));
            }
        }
        
        // Simulate latency
        if self.behavior.simulated_latency_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.behavior.simulated_latency_ms));
        }
        
        let id = continuation.id;
        self.storage.lock().unwrap().insert(id, continuation);
        Ok(id)
    }
    
    fn find_by_id(&self, id: ContinuationId) -> Option<CapturedContinuation> {
        // Log the call
        self.call_log.lock().unwrap().push(RepositoryCall::Find(id));
        
        // Simulate failure if configured
        if self.behavior.find_should_fail {
            return None;
        }
        
        // Simulate latency
        if self.behavior.simulated_latency_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.behavior.simulated_latency_ms));
        }
        
        self.storage.lock().unwrap().get(&id).cloned()
    }
    
    fn remove(&mut self, id: ContinuationId) -> Result<()> {
        self.call_log.lock().unwrap().push(RepositoryCall::Remove(id));
        self.storage.lock().unwrap().remove(&id);
        Ok(())
    }
    
    fn list_all(&self) -> Vec<ContinuationId> {
        self.call_log.lock().unwrap().push(RepositoryCall::List);
        self.storage.lock().unwrap().keys().copied().collect()
    }
    
    fn garbage_collect(&mut self, current_generation: u64) -> Result<usize> {
        self.call_log.lock().unwrap().push(RepositoryCall::GarbageCollect(current_generation));
        
        let initial_size = self.storage.lock().unwrap().len();
        
        // Simple GC simulation - remove old continuations
        self.storage.lock().unwrap().retain(|_id, cont| {
            cont.metadata.generation + 5 > current_generation // Keep recent ones
        });
        
        let final_size = self.storage.lock().unwrap().len();
        Ok(initial_size - final_size)
    }
}