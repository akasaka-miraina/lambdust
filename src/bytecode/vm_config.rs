//! Configuration for the virtual machine.

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
            initial_stack_size: 1024,
            max_stack_size: 1024 * 1024, // 1M stack slots
            gc_enabled: true,
            gc_threshold: 1000,
            profiling_enabled: false,
            debug_mode: false,
        }
    }
}