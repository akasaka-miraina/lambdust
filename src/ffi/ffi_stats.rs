/// Statistics about FFI usage.
#[derive(Debug, Default, Clone)]
pub struct FfiStats {
    /// Total number of FFI calls made
    pub total_calls: u64,
    /// Number of successful calls
    pub successful_calls: u64,
    /// Number of failed calls
    pub failed_calls: u64,
    /// Number of registered functions
    pub registered_functions: usize,
}