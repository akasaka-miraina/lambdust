//! Effect sandbox configuration and management.

use crate::effects::Effect;
use std::thread::ThreadId;
use std::time::{SystemTime, Duration};

/// Configuration for an effect sandbox.
#[derive(Debug, Clone)]
pub struct EffectSandboxConfig {
    /// Maximum number of effects allowed in sandbox
    pub max_effects: usize,
    /// Timeout for sandbox operations
    pub timeout: Duration,
    /// Allowed effect types in sandbox
    pub allowed_effects: Vec<Effect>,
    /// Whether sandbox should auto-cleanup on completion
    pub auto_cleanup: bool,
    /// Resource limits for sandbox
    pub resource_limits: SandboxResourceLimits,
}

/// Resource limits for a sandbox.
#[derive(Debug, Clone)]
pub struct SandboxResourceLimits {
    /// Maximum memory usage (bytes)
    pub max_memory: Option<usize>,
    /// Maximum execution time
    pub max_execution_time: Option<Duration>,
    /// Maximum file operations
    pub max_file_operations: Option<usize>,
    /// Maximum network operations
    pub max_network_operations: Option<usize>,
}

/// Handle to an effect sandbox.
#[derive(Debug)]
pub struct EffectSandboxHandle {
    /// Unique sandbox ID
    pub id: u64,
    /// Thread this sandbox belongs to
    pub thread_id: ThreadId,
    /// Sandbox configuration
    pub config: EffectSandboxConfig,
    /// When sandbox was created
    pub created_at: SystemTime,
    /// Weak reference to coordinator (forward declaration to avoid circular dependency)
    pub coordinator: std::sync::Weak<()>,
}

/// Statistics for a specific sandbox.
#[derive(Debug, Clone)]
pub struct SandboxStatistics {
    /// Sandbox ID
    pub id: u64,
    /// Associated thread
    pub thread_id: ThreadId,
    /// How long sandbox has been active
    pub uptime: Duration,
    /// Number of effects executed
    pub effect_count: usize,
    /// Current resource usage
    pub resource_usage: ResourceUsage,
}

/// Resource usage information.
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// Memory usage in bytes
    pub memory_bytes: usize,
    /// Execution time
    pub execution_time: Duration,
    /// Number of file operations
    pub file_operations: usize,
    /// Number of network operations
    pub network_operations: usize,
}

impl Default for EffectSandboxConfig {
    fn default() -> Self {
        Self {
            max_effects: 100,
            timeout: Duration::from_secs(30),
            allowed_effects: vec![Effect::Pure],
            auto_cleanup: true,
            resource_limits: SandboxResourceLimits::default(),
        }
    }
}

impl Default for SandboxResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: Some(64 * 1024 * 1024), // 64MB
            max_execution_time: Some(Duration::from_secs(10)),
            max_file_operations: Some(100),
            max_network_operations: Some(10),
        }
    }
}

impl EffectSandboxHandle {
    /// Destroys the sandbox and cleans up resources.
    pub fn destroy(self) -> Result<(), String> {
        // Simplified implementation to avoid circular dependency
        // Full implementation would coordinate with EffectCoordinator
        Ok(())
    }
    
    /// Checks if the sandbox is still valid.
    pub fn is_valid(&self) -> bool {
        // TODO: Fix weak reference issue 
        // self.coordinator.strong_count() > 0
        true
    }
    
    /// Gets sandbox statistics.
    pub fn get_statistics(&self) -> SandboxStatistics {
        SandboxStatistics {
            id: self.id,
            thread_id: self.thread_id,
            uptime: SystemTime::now().duration_since(self.created_at).unwrap_or_default(),
            effect_count: 0, // Would be populated in real implementation
            resource_usage: ResourceUsage::default(),
        }
    }
}