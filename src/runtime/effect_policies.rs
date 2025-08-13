//! Effect coordination policies and configuration.

use std::time::Duration;

/// Policies for effect coordination.
#[derive(Debug)]
pub struct EffectPolicies {
    /// Whether to track effect history
    pub track_history: bool,
    /// Maximum size of effect history
    pub max_history_size: usize,
    /// Whether to allow cross-thread effect coordination
    pub allow_cross_thread_coordination: bool,
    /// Timeout for effect coordination operations
    pub coordination_timeout: Duration,
    /// Whether to enforce strict effect ordering
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub enforce_strict_ordering: bool,
    /// Whether to enable effect isolation
    pub enable_effect_isolation: bool,
    /// Maximum concurrent effects per thread
    pub max_concurrent_effects_per_thread: usize,
    /// Whether to enable automatic effect rollback on errors
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub enable_automatic_rollback: bool,
}

impl Default for EffectPolicies {
    fn default() -> Self {
        Self {
            track_history: true,
            max_history_size: 10000,
            allow_cross_thread_coordination: true,
            coordination_timeout: Duration::from_secs(5),
            enforce_strict_ordering: true,
            enable_effect_isolation: true,
            max_concurrent_effects_per_thread: 100,
            enable_automatic_rollback: true,
        }
    }
}

impl EffectPolicies {
    /// Creates new effect policies with history tracking disabled.
    pub fn no_history() -> Self {
        Self {
            track_history: false,
            max_history_size: 0,
            allow_cross_thread_coordination: true,
            coordination_timeout: Duration::from_secs(5),
            enforce_strict_ordering: true,
            enable_effect_isolation: true,
            max_concurrent_effects_per_thread: 100,
            enable_automatic_rollback: true,
        }
    }

    /// Creates new effect policies with minimal overhead.
    pub fn minimal() -> Self {
        Self {
            track_history: false,
            max_history_size: 0,
            allow_cross_thread_coordination: false,
            coordination_timeout: Duration::from_millis(100),
            enforce_strict_ordering: false,
            enable_effect_isolation: false,
            max_concurrent_effects_per_thread: 10,
            enable_automatic_rollback: false,
        }
    }
    
    /// Creates effect policies optimized for high concurrency.
    pub fn high_concurrency() -> Self {
        Self {
            track_history: false,
            max_history_size: 1000,
            allow_cross_thread_coordination: true,
            coordination_timeout: Duration::from_millis(500),
            enforce_strict_ordering: false,
            enable_effect_isolation: true,
            max_concurrent_effects_per_thread: 1000,
            enable_automatic_rollback: true,
        }
    }
}