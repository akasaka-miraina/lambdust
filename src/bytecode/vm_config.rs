//! Configuration for the virtual machine.

/// Default VM configuration constants
pub mod defaults {
    /// Default initial stack size in slots
    pub const INITIAL_STACK_SIZE: usize = 1024;
    /// Default maximum stack size in slots (1M slots)
    pub const MAX_STACK_SIZE: usize = 1024 * 1024;
    /// Default garbage collection threshold
    pub const GC_THRESHOLD: usize = 1000;
}

/// Configuration for the virtual machine.
#[derive(Debug, Clone)]
pub struct VmConfig {
    /// Initial stack size
    pub initial_stack_size: usize,
    /// Maximum stack size before overflow
    pub max_stack_size: usize,
    /// Enable garbage collection during execution
    pub gc_enabled: bool,
    /// GC threshold (allocations before triggering GC)
    pub gc_threshold: usize,
    /// Enable profiling during execution
    pub profiling_enabled: bool,
    /// Enable instruction-level debugging
    pub debug_mode: bool,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            initial_stack_size: defaults::INITIAL_STACK_SIZE,
            max_stack_size: defaults::MAX_STACK_SIZE,
            gc_enabled: true,
            gc_threshold: defaults::GC_THRESHOLD,
            profiling_enabled: false,
            debug_mode: false,
        }
    }
}