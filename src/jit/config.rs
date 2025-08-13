//! JIT compilation configuration and settings
//!
//! This module provides comprehensive configuration options for the JIT compiler,
//! allowing fine-tuning of compilation strategies, optimization levels, and
//! performance trade-offs based on specific use cases.

use crate::jit::hotspot_detector::HotspotConfig;
use crate::jit::compilation_tiers::TierConfig;
use std::collections::HashMap;
use std::time::Duration;

/// Main JIT compiler configuration
#[derive(Debug, Clone)]
pub struct JitConfig {
    /// Enable JIT compilation globally
    pub enabled: bool,
    /// Hotspot detection configuration
    pub hotspot_config: HotspotConfig,
    /// Tier management configuration
    pub tier_config: TierConfig,
    /// Code cache configuration
    pub cache_config: CacheConfig,
    /// Optimization pipeline configuration
    pub optimization_config: OptimizationConfig,
    /// Profile-guided optimization configuration
    pub pgo_config: ProfileGuidedOptimizerConfig,
    /// Target-specific features configuration
    pub target_config: TargetConfig,
    /// Memory management configuration
    pub memory_config: MemoryConfig,
    /// Security configuration
    pub security_config: SecurityConfig,
}

impl Default for JitConfig {
    fn default() -> Self {
        JitConfig {
            enabled: true,
            hotspot_config: HotspotConfig::default(),
            tier_config: TierConfig::default(),
            cache_config: CacheConfig::default(),
            optimization_config: OptimizationConfig::default(),
            pgo_config: ProfileGuidedOptimizerConfig::default(),
            target_config: TargetConfig::default(),
            memory_config: MemoryConfig::default(),
            security_config: SecurityConfig::default(),
        }
    }
}

impl JitConfig {
    /// Creates a new JIT configuration optimized for development
    pub fn development() -> Self {
        JitConfig {
            enabled: true,
            hotspot_config: HotspotConfig {
                min_frequency: 5.0,
                min_total_time: Duration::from_millis(20),
                min_execution_count: 50,
                complexity_threshold: 3.0,
                stability_window: Duration::from_secs(1),
                max_tracked_functions: 500,
            },
            tier_config: TierConfig {
                auto_tier_up: true,
                tier_up_execution_threshold: 25,
                tier_up_time_threshold: Duration::from_millis(200),
                tier_up_benefit_threshold: 1.5,
                max_compilation_time: HashMap::new(),
                enable_speculative_compilation: false,
                enable_deoptimization: true,
            },
            cache_config: CacheConfig {
                max_entries: 1000,
                max_memory_usage: 64 * 1024 * 1024, // 64MB
                eviction_policy: EvictionPolicy::LRU,
                enable_persistent_cache: false,
                cache_directory: None,
            },
            optimization_config: OptimizationConfig::development(),
            pgo_config: ProfileGuidedOptimizerConfig::development(),
            target_config: TargetConfig::default(),
            memory_config: MemoryConfig::default(),
            security_config: SecurityConfig::default(),
        }
    }

    /// Creates a new JIT configuration optimized for production
    pub fn production() -> Self {
        JitConfig {
            enabled: true,
            hotspot_config: HotspotConfig {
                min_frequency: 20.0,
                min_total_time: Duration::from_millis(100),
                min_execution_count: 200,
                complexity_threshold: 10.0,
                stability_window: Duration::from_secs(5),
                max_tracked_functions: 2000,
            },
            tier_config: TierConfig {
                auto_tier_up: true,
                tier_up_execution_threshold: 100,
                tier_up_time_threshold: Duration::from_millis(50),
                tier_up_benefit_threshold: 3.0,
                max_compilation_time: HashMap::new(),
                enable_speculative_compilation: true,
                enable_deoptimization: true,
            },
            cache_config: CacheConfig {
                max_entries: 5000,
                max_memory_usage: 256 * 1024 * 1024, // 256MB
                eviction_policy: EvictionPolicy::AdaptiveLRU,
                enable_persistent_cache: true,
                cache_directory: Some("/tmp/lambdust-jit-cache".to_string()),
            },
            optimization_config: OptimizationConfig::production(),
            pgo_config: ProfileGuidedOptimizerConfig::production(),
            target_config: TargetConfig::default(),
            memory_config: MemoryConfig::default(),
            security_config: SecurityConfig::default(),
        }
    }

    /// Creates a new JIT configuration optimized for benchmarking
    pub fn benchmarking() -> Self {
        JitConfig {
            enabled: true,
            hotspot_config: HotspotConfig {
                min_frequency: 50.0,
                min_total_time: Duration::from_millis(200),
                min_execution_count: 500,
                complexity_threshold: 20.0,
                stability_window: Duration::from_secs(10),
                max_tracked_functions: 10000,
            },
            tier_config: TierConfig {
                auto_tier_up: true,
                tier_up_execution_threshold: 200,
                tier_up_time_threshold: Duration::from_millis(10),
                tier_up_benefit_threshold: 5.0,
                max_compilation_time: HashMap::new(),
                enable_speculative_compilation: true,
                enable_deoptimization: false,
            },
            cache_config: CacheConfig {
                max_entries: 20000,
                max_memory_usage: 1024 * 1024 * 1024, // 1GB
                eviction_policy: EvictionPolicy::AdaptiveLRU,
                enable_persistent_cache: true,
                cache_directory: Some("/tmp/lambdust-jit-benchmark-cache".to_string()),
            },
            optimization_config: OptimizationConfig::aggressive(),
            pgo_config: ProfileGuidedOptimizerConfig::aggressive(),
            target_config: TargetConfig::default(),
            memory_config: MemoryConfig::default(),
            security_config: SecurityConfig::default(),
        }
    }

    /// Validates the configuration for consistency
    pub fn validate(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(()); // No validation needed for disabled JIT
        }

        // Validate hotspot config
        if self.hotspot_config.min_frequency <= 0.0 {
            return Err("Hotspot min_frequency must be positive".to_string());
        }

        if self.hotspot_config.min_execution_count == 0 {
            return Err("Hotspot min_execution_count must be positive".to_string());
        }

        // Validate tier config
        if self.tier_config.tier_up_execution_threshold == 0 {
            return Err("Tier tier_up_execution_threshold must be positive".to_string());
        }

        if self.tier_config.tier_up_benefit_threshold <= 1.0 {
            return Err("Tier tier_up_benefit_threshold must be greater than 1.0".to_string());
        }

        // Validate cache config
        if self.cache_config.max_entries == 0 {
            return Err("Cache max_entries must be positive".to_string());
        }

        if self.cache_config.max_memory_usage == 0 {
            return Err("Cache max_memory_usage must be positive".to_string());
        }

        Ok(())
    }

    /// Converts JitConfig to CodegenConfig for code generation
    pub fn to_codegen_config(&self) -> crate::jit::code_generator::CodegenConfig {
        use crate::jit::code_generator::{CodegenConfig, TargetFeatures, OptimizationLevel};
        
        CodegenConfig {
            target_features: TargetFeatures::detect(),
            optimization_level: if self.optimization_config.enable_advanced_optimizations {
                OptimizationLevel::Aggressive
            } else if self.optimization_config.enable_basic_optimizations {
                OptimizationLevel::Balanced
            } else {
                OptimizationLevel::None
            },
            debug_info: false, // TODO: could be configurable
            bounds_checking: true, // TODO: could be configurable
            overflow_checking: true, // TODO: could be configurable
            simd_optimizations: self.optimization_config.enable_advanced_optimizations,
        }
    }
}

/// Code cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of cached code entries
    pub max_entries: usize,
    /// Maximum memory usage for cached code (bytes)
    pub max_memory_usage: usize,
    /// Cache eviction policy
    pub eviction_policy: EvictionPolicy,
    /// Enable persistent cache across program runs
    pub enable_persistent_cache: bool,
    /// Directory for persistent cache files
    pub cache_directory: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            max_entries: 2000,
            max_memory_usage: 128 * 1024 * 1024, // 128MB
            eviction_policy: EvictionPolicy::LRU,
            enable_persistent_cache: false,
            cache_directory: None,
        }
    }
}

impl CacheConfig {
    /// Converts to code_cache::CacheConfig
    pub fn to_code_cache_config(&self) -> crate::jit::code_cache::CacheConfig {
        use std::time::Duration;
        crate::jit::code_cache::CacheConfig {
            max_memory_bytes: self.max_memory_usage,
            max_entries: self.max_entries,
            memory_pressure_threshold: 0.8, // reasonable default
            enable_lru_eviction: matches!(self.eviction_policy, EvictionPolicy::LRU | EvictionPolicy::AdaptiveLRU),
            enable_compaction: true,
            compaction_interval: Duration::from_secs(60),
            execution_based_retention: true,
            min_execution_count_for_retention: 5,
        }
    }
}

/// Cache eviction policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// Adaptive LRU with frequency consideration
    AdaptiveLRU,
    /// First In First Out
    FIFO,
    /// Random eviction
    Random,
}

impl std::fmt::Display for EvictionPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvictionPolicy::LRU => write!(f, "lru"),
            EvictionPolicy::LFU => write!(f, "lfu"),
            EvictionPolicy::AdaptiveLRU => write!(f, "adaptive_lru"),
            EvictionPolicy::FIFO => write!(f, "fifo"),
            EvictionPolicy::Random => write!(f, "random"),
        }
    }
}

/// Optimization pipeline configuration
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Enable basic optimizations (constant folding, dead code elimination)
    pub enable_basic_optimizations: bool,
    /// Enable Scheme-specific optimizations (tail call, closure optimizations)
    pub enable_scheme_optimizations: bool,
    /// Enable advanced optimizations (inlining, loop optimizations)
    pub enable_advanced_optimizations: bool,
    /// Maximum inline depth
    pub max_inline_depth: u32,
    /// Maximum function size for inlining
    pub max_inline_size: u32,
    /// Enable type-based optimizations
    pub enable_type_optimizations: bool,
    /// Enable SIMD optimizations
    pub enable_simd_optimizations: bool,
    /// Target optimization level (0-3)
    pub optimization_level: u32,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        OptimizationConfig {
            enable_basic_optimizations: true,
            enable_scheme_optimizations: true,
            enable_advanced_optimizations: false,
            max_inline_depth: 3,
            max_inline_size: 100,
            enable_type_optimizations: true,
            enable_simd_optimizations: true,
            optimization_level: 1,
        }
    }
}

impl OptimizationConfig {
    /// Development-optimized configuration (fast compilation)
    pub fn development() -> Self {
        OptimizationConfig {
            enable_basic_optimizations: true,
            enable_scheme_optimizations: true,
            enable_advanced_optimizations: false,
            max_inline_depth: 2,
            max_inline_size: 50,
            enable_type_optimizations: true,
            enable_simd_optimizations: false,
            optimization_level: 0,
        }
    }

    /// Production-optimized configuration (balanced)
    pub fn production() -> Self {
        OptimizationConfig {
            enable_basic_optimizations: true,
            enable_scheme_optimizations: true,
            enable_advanced_optimizations: true,
            max_inline_depth: 4,
            max_inline_size: 150,
            enable_type_optimizations: true,
            enable_simd_optimizations: true,
            optimization_level: 2,
        }
    }

    /// Aggressive optimization configuration (maximum performance)
    pub fn aggressive() -> Self {
        OptimizationConfig {
            enable_basic_optimizations: true,
            enable_scheme_optimizations: true,
            enable_advanced_optimizations: true,
            max_inline_depth: 6,
            max_inline_size: 300,
            enable_type_optimizations: true,
            enable_simd_optimizations: true,
            optimization_level: 3,
        }
    }
}

/// Profile-guided optimization configuration
#[derive(Debug, Clone)]
pub struct ProfileGuidedOptimizerConfig {
    /// Enable profile-guided optimization
    pub enabled: bool,
    /// Profile collection duration before optimization
    pub profile_collection_duration: Duration,
    /// Minimum executions for profile-based decisions
    pub min_profile_executions: u64,
    /// Enable type profile collection
    pub enable_type_profiling: bool,
    /// Enable branch profiling
    pub enable_branch_profiling: bool,
    /// Enable memory access profiling
    pub enable_memory_profiling: bool,
    /// Profile data retention time
    pub profile_retention_time: Duration,
}

impl Default for ProfileGuidedOptimizerConfig {
    fn default() -> Self {
        ProfileGuidedOptimizerConfig {
            enabled: true,
            profile_collection_duration: Duration::from_secs(60),
            min_profile_executions: 100,
            enable_type_profiling: true,
            enable_branch_profiling: true,
            enable_memory_profiling: false,
            profile_retention_time: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl ProfileGuidedOptimizerConfig {
    /// Development configuration (lighter profiling)
    pub fn development() -> Self {
        ProfileGuidedOptimizerConfig {
            enabled: true,
            profile_collection_duration: Duration::from_secs(30),
            min_profile_executions: 50,
            enable_type_profiling: true,
            enable_branch_profiling: false,
            enable_memory_profiling: false,
            profile_retention_time: Duration::from_secs(1800), // 30 minutes
        }
    }

    /// Production configuration (balanced profiling)
    pub fn production() -> Self {
        ProfileGuidedOptimizerConfig {
            enabled: true,
            profile_collection_duration: Duration::from_secs(120),
            min_profile_executions: 200,
            enable_type_profiling: true,
            enable_branch_profiling: true,
            enable_memory_profiling: true,
            profile_retention_time: Duration::from_secs(7200), // 2 hours
        }
    }

    /// Aggressive configuration (comprehensive profiling)
    pub fn aggressive() -> Self {
        ProfileGuidedOptimizerConfig {
            enabled: true,
            profile_collection_duration: Duration::from_secs(300),
            min_profile_executions: 500,
            enable_type_profiling: true,
            enable_branch_profiling: true,
            enable_memory_profiling: true,
            profile_retention_time: Duration::from_secs(14400), // 4 hours
        }
    }
}

/// Target-specific configuration
#[derive(Debug, Clone)]
pub struct TargetConfig {
    /// Target CPU features to enable
    pub cpu_features: Vec<String>,
    /// Target architecture optimizations
    pub arch_optimizations: Vec<String>,
    /// Enable vectorization
    pub enable_vectorization: bool,
    /// Vector width preference
    pub preferred_vector_width: Option<u32>,
    /// Enable target-specific intrinsics
    pub enable_intrinsics: bool,
}

impl Default for TargetConfig {
    fn default() -> Self {
        TargetConfig {
            cpu_features: vec![],
            arch_optimizations: vec![],
            enable_vectorization: true,
            preferred_vector_width: None,
            enable_intrinsics: true,
        }
    }
}

/// Memory management configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Initial code buffer size
    pub initial_code_buffer_size: usize,
    /// Maximum code buffer size
    pub max_code_buffer_size: usize,
    /// Enable memory pool for allocations
    pub enable_memory_pool: bool,
    /// Memory pool initial size
    pub memory_pool_initial_size: usize,
    /// Enable garbage collection of unused code
    pub enable_code_gc: bool,
    /// Code GC interval
    pub code_gc_interval: Duration,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        MemoryConfig {
            initial_code_buffer_size: 64 * 1024, // 64KB
            max_code_buffer_size: 16 * 1024 * 1024, // 16MB
            enable_memory_pool: true,
            memory_pool_initial_size: 1024 * 1024, // 1MB
            enable_code_gc: true,
            code_gc_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable code signing verification
    pub enable_code_signing: bool,
    /// Enable execution sandboxing
    pub enable_sandboxing: bool,
    /// Enable stack protection
    pub enable_stack_protection: bool,
    /// Enable control flow integrity
    pub enable_cfi: bool,
    /// Maximum executable memory size
    pub max_executable_memory: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        SecurityConfig {
            enable_code_signing: false,
            enable_sandboxing: false,
            enable_stack_protection: true,
            enable_cfi: false,
            max_executable_memory: 64 * 1024 * 1024, // 64MB
        }
    }
}

/// Compilation strategy selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompilationStrategy {
    /// Lazy compilation - compile on first use
    Lazy,
    /// Eager compilation - compile immediately when hotspot detected
    Eager,
    /// Background compilation - compile in background thread
    Background,
    /// Adaptive - choose strategy based on system load
    #[default]
    Adaptive,
}


impl std::fmt::Display for CompilationStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationStrategy::Lazy => write!(f, "lazy"),
            CompilationStrategy::Eager => write!(f, "eager"),
            CompilationStrategy::Background => write!(f, "background"),
            CompilationStrategy::Adaptive => write!(f, "adaptive"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = JitConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_development_config_validation() {
        let config = JitConfig::development();
        assert!(config.validate().is_ok());
        assert!(config.enabled);
    }

    #[test]
    fn test_production_config_validation() {
        let config = JitConfig::production();
        assert!(config.validate().is_ok());
        assert!(config.enabled);
        assert!(config.cache_config.enable_persistent_cache);
    }

    #[test]
    fn test_benchmarking_config_validation() {
        let config = JitConfig::benchmarking();
        assert!(config.validate().is_ok());
        assert_eq!(config.optimization_config.optimization_level, 3);
    }

    #[test]
    fn test_invalid_config_validation() {
        let mut config = JitConfig::default();
        config.hotspot_config.min_frequency = -1.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_disabled_jit_config() {
        let mut config = JitConfig::default();
        config.enabled = false;
        config.hotspot_config.min_frequency = -1.0; // Invalid, but should pass validation
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_eviction_policy_display() {
        assert_eq!(EvictionPolicy::LRU.to_string(), "lru");
        assert_eq!(EvictionPolicy::AdaptiveLRU.to_string(), "adaptive_lru");
    }

    #[test]
    fn test_compilation_strategy_display() {
        assert_eq!(CompilationStrategy::Lazy.to_string(), "lazy");
        assert_eq!(CompilationStrategy::Adaptive.to_string(), "adaptive");
    }

    #[test]
    fn test_optimization_config_presets() {
        let dev_config = OptimizationConfig::development();
        let prod_config = OptimizationConfig::production();
        let aggressive_config = OptimizationConfig::aggressive();

        assert!(dev_config.optimization_level < prod_config.optimization_level);
        assert!(prod_config.optimization_level < aggressive_config.optimization_level);
        assert!(dev_config.max_inline_size < prod_config.max_inline_size);
        assert!(prod_config.max_inline_size < aggressive_config.max_inline_size);
    }
}