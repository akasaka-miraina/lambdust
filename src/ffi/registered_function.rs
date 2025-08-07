use super::FfiFunction;
use std::sync::Arc;
use std::fmt;

/// A registered FFI function.
#[derive(Clone)]
pub struct RegisteredFunction {
    /// The function implementation
    pub function: Arc<dyn FfiFunction>,
    /// When this function was registered
    pub registered_at: std::time::SystemTime,
}

impl fmt::Debug for RegisteredFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegisteredFunction")
            .field("function", &self.function.signature().name)
            .field("registered_at", &self.registered_at)
            .finish()
    }
}