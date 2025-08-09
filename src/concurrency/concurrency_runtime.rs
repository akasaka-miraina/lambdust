//! Runtime handle for async operations in the concurrency system.

use std::sync::Arc;
use tokio::runtime::Handle;

/// Runtime handle for async operations in the concurrency system.
/// 
/// This provides access to the global tokio runtime for executing
/// asynchronous operations and managing futures.
pub struct ConcurrencyRuntime {
    handle: Handle,
}

impl ConcurrencyRuntime {
    /// Gets or creates the global concurrency runtime.
    pub fn global() -> Arc<Self> {
        use std::sync::OnceLock;
        static RUNTIME: OnceLock<Arc<ConcurrencyRuntime>> = OnceLock::new();
        
        RUNTIME.get_or_init(|| {
            Arc::new(ConcurrencyRuntime {
                handle: tokio::runtime::Handle::current(),
            })
        }).clone()
    }

    /// Gets the tokio runtime handle.
    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    /// Spawns a task on the runtime.
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.handle.spawn(future)
    }

    /// Blocks on a future until completion.
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.handle.block_on(future)
    }
}