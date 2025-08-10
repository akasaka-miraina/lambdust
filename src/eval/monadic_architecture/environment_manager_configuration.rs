//! Configuration for environment manager

/// Configuration for environment manager
#[derive(Debug, Clone)]
pub struct EnvironmentManagerConfiguration {
    /// Whether to enable environment caching
    pub enable_caching: bool,
    
    /// Maximum cache size
    pub max_cache_size: usize,
    
    /// Whether to enable copy-on-write optimization
    pub enable_cow: bool,
}

impl Default for EnvironmentManagerConfiguration {
    fn default() -> Self {
        Self {
            enable_caching: true,
            max_cache_size: 1000,
            enable_cow: true,
        }
    }
}