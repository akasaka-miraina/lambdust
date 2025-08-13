//! Repository trait for managing continuations (interface)

use crate::eval::continuation_domain::{CapturedContinuation, ContinuationId};
use crate::diagnostics::Result;

/// Repository trait for managing continuations (interface)
pub trait ContinuationRepository: std::fmt::Debug {
    /// Store a continuation
    fn store(&mut self, continuation: CapturedContinuation) -> Result<ContinuationId>;
    
    /// Retrieve a continuation by ID
    fn find_by_id(&self, id: ContinuationId) -> Option<CapturedContinuation>;
    
    /// Remove a continuation
    fn remove(&mut self, id: ContinuationId) -> Result<()>;
    
    /// List all continuation IDs
    fn list_all(&self) -> Vec<ContinuationId>;
    
    /// Garbage collect expired continuations
    fn garbage_collect(&mut self, current_generation: u64) -> Result<usize>;
}