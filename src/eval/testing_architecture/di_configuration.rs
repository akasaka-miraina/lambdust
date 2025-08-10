/// Configuration for dependency injection
#[derive(Debug, Clone)]
pub struct DIConfiguration {
    /// Whether to enable automatic dependency resolution
    pub auto_resolve: bool,
    
    /// Whether to cache resolved instances
    pub enable_caching: bool,
    
    /// Whether to enable circular dependency detection
    pub detect_circular_deps: bool,
    
    /// Maximum resolution depth
    pub max_resolution_depth: usize,
}

impl Default for DIConfiguration {
    fn default() -> Self {
        Self {
            auto_resolve: true,
            enable_caching: true,
            detect_circular_deps: true,
            max_resolution_depth: 10,
        }
    }
}