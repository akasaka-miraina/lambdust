use crate::eval::continuation_domain::ContinuationId;

/// Repository call tracking for assertions
#[derive(Debug, Clone)]
pub enum RepositoryCall {
    /// Store a continuation with given ID
    Store(ContinuationId),
    /// Find a continuation by ID
    Find(ContinuationId),
    /// Remove a continuation by ID
    Remove(ContinuationId),
    /// List all stored continuations
    List,
    /// Garbage collect continuations older than given generation
    GarbageCollect(u64),
}