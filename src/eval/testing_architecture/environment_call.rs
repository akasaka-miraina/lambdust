/// Environment operation call tracking
#[derive(Debug, Clone)]
pub enum EnvironmentCall {
    /// Create a new environment with optional parent ID
    Create(Option<u64>),
    /// Clone an existing environment by ID
    Clone(u64),
    /// Extend an environment with new bindings by ID
    Extend(u64),
    /// Look up a variable in an environment
    Lookup(u64, String),
    /// Update a variable binding in an environment
    Update(u64, String),
}