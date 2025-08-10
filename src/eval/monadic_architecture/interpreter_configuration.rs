//! Configuration for effect interpreter

/// Configuration for effect interpreter
#[derive(Debug, Clone)]
pub struct InterpreterConfiguration {
    /// Whether to enable async IO operations
    pub enable_async_io: bool,
    
    /// Timeout for IO operations
    pub io_timeout_ms: u64,
    
    /// Maximum concurrent IO operations
    pub max_concurrent_io: usize,
}

impl Default for InterpreterConfiguration {
    fn default() -> Self {
        Self {
            enable_async_io: true,
            io_timeout_ms: 1000,
            max_concurrent_io: 10,
        }
    }
}