//! Configuration for continuation repository

/// Configuration for continuation repository
#[derive(Debug, Clone)]
pub struct RepositoryConfiguration {
    /// Maximum number of continuations to store
    pub max_continuations: usize,
    
    /// Whether to enable automatic garbage collection
    pub auto_gc_enabled: bool,
    
    /// GC threshold (collect when this many generations old)
    pub gc_threshold: u64,
}

impl Default for RepositoryConfiguration {
    fn default() -> Self {
        Self {
            max_continuations: 1000,
            auto_gc_enabled: true,
            gc_threshold: 10,
        }
    }
}