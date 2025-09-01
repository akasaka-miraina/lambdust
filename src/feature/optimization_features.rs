#![allow(missing_docs)]//! Feature Flag System for Phase 8 Gradual Optimization Deployment
//!
//! This module provides a comprehensive feature flag system that allows
//! the gradual deployment of Phase 8 optimizations with safe fallback
//! mechanisms and runtime performance monitoring.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::collections::HashMap;
use parking_lot::RwLock;

/// Feature flag definitions for Phase 8 optimizations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationFeature {
    /// Use NaN-boxed values for immediate types
    NanBoxedValues,
    
    /// Use string interning for symbols and strings
    StringInterning,
    
    /// Use zero-copy parsing infrastructure
    ZeroCopyParsing,
    
    /// Use arena allocation for large objects
    ArenaAllocation,
    
    /// Use SIMD-optimized operations where available
    SimdOptimizations,
    
    /// Use lock-free concurrent data structures
    LockFreeConcurrency,
    
    /// Use memory-mapped symbol tables
    MemoryMappedSymbols,
    
    /// Use compact AST representation
    CompactAst,
    
    /// Use unified value system
    UnifiedValueSystem,
    
    /// Enable performance monitoring and telemetry
    PerformanceMonitoring,
}

/// Feature flag management system
pub struct OptimizationFlags {
    /// Atomic flags for quick runtime checks
    flags: HashMap<OptimizationFeature, AtomicBool>,
    
    /// Usage statistics for each feature
    usage_stats: RwLock<HashMap<OptimizationFeature, FeatureStats>>,
    
    /// Global feature configuration
    config: RwLock<OptimizationConfig>,
}

/// Statistics tracked for each feature
#[derive(Debug, Default)]
pub struct FeatureStats {
    /// Number of times this feature was used
    usage_count: AtomicUsize,
    
    /// Number of successful operations
    success_count: AtomicUsize,
    
    /// Number of failed operations (fallback triggered)
    fallback_count: AtomicUsize,
    
    /// Total time spent in feature (microseconds)
    total_time_micros: AtomicUsize,
}

/// Global feature configuration
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Whether to enable gradual rollout
    gradual_rollout: bool,
    
    /// Percentage of operations to use new features (0-100)
    rollout_percentage: u8,
    
    /// Whether to enable automatic fallback on errors
    auto_fallback: bool,
    
    /// Maximum allowed failure rate before disabling feature (0.0-1.0)
    max_failure_rate: f64,
    
    /// Minimum sample size before failure rate is considered
    min_sample_size: usize,
}

impl OptimizationFlags {
    /// Create a new feature flag system with default configuration
    pub fn new() -> Self {
        let mut flags = HashMap::new();
        
        // Initialize all features as disabled by default
        for feature in OptimizationFeature::all() {
            flags.insert(feature, AtomicBool::new(false));
        }
        
        Self {
            flags,
            usage_stats: RwLock::new(HashMap::new()),
            config: RwLock::new(OptimizationConfig::default()),
        }
    }
    
    /// Check if a feature is enabled
    pub fn is_enabled(&self, feature: OptimizationFeature) -> bool {
        if let Some(flag) = self.flags.get(&feature) {
            let base_enabled = flag.load(Ordering::Relaxed);
            
            if !base_enabled {
                return false;
            }
            
            // Apply gradual rollout logic
            let config = self.config.read();
            if config.gradual_rollout {
                self.should_use_feature_gradual(&config)
            } else {
                true
            }
        } else {
            false
        }
    }
    
    /// Enable a specific feature
    pub fn enable(&self, feature: OptimizationFeature) {
        if let Some(flag) = self.flags.get(&feature) {
            flag.store(true, Ordering::Relaxed);
        }
        
        // Initialize stats if not present
        let mut stats = self.usage_stats.write();
        stats.entry(feature).or_insert_with(FeatureStats::default);
    }
    
    /// Disable a specific feature
    pub fn disable(&self, feature: OptimizationFeature) {
        if let Some(flag) = self.flags.get(&feature) {
            flag.store(false, Ordering::Relaxed);
        }
    }
    
    /// Record successful usage of a feature
    pub fn record_success(&self, feature: OptimizationFeature, duration_micros: u64) {
        let stats = self.usage_stats.read();
        if let Some(feature_stats) = stats.get(&feature) {
            feature_stats.usage_count.fetch_add(1, Ordering::Relaxed);
            feature_stats.success_count.fetch_add(1, Ordering::Relaxed);
            feature_stats.total_time_micros.fetch_add(duration_micros as usize, Ordering::Relaxed);
        }
    }
    
    /// Record failed usage of a feature (triggered fallback)
    pub fn record_fallback(&self, feature: OptimizationFeature) {
        let stats = self.usage_stats.read();
        if let Some(feature_stats) = stats.get(&feature) {
            feature_stats.usage_count.fetch_add(1, Ordering::Relaxed);
            feature_stats.fallback_count.fetch_add(1, Ordering::Relaxed);
        }
        
        // Check if we should auto-disable due to high failure rate
        self.check_auto_disable(feature);
    }
    
    /// Get statistics for a specific feature
    pub fn get_stats(&self, feature: OptimizationFeature) -> Option<FeatureStatsSnapshot> {
        let stats = self.usage_stats.read();
        stats.get(&feature).map(|fs| fs.snapshot())
    }
    
    /// Get statistics for all features
    pub fn get_all_stats(&self) -> HashMap<OptimizationFeature, FeatureStatsSnapshot> {
        let stats = self.usage_stats.read();
        stats.iter()
            .map(|(feature, stats)| (*feature, stats.snapshot()))
            .collect()
    }
    
    /// Configure gradual rollout settings
    pub fn configure_rollout(&self, percentage: u8, auto_fallback: bool) {
        let mut config = self.config.write();
        config.gradual_rollout = true;
        config.rollout_percentage = percentage.min(100);
        config.auto_fallback = auto_fallback;
    }
    
    /// Disable gradual rollout (full deployment)
    pub fn disable_rollout(&self) {
        let mut config = self.config.write();
        config.gradual_rollout = false;
    }
    
    /// Internal: Check if feature should be used based on gradual rollout
    fn should_use_feature_gradual(&self, config: &OptimizationConfig) -> bool {
        // Simple hash-based selection for consistent behavior
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        std::thread::current().id().hash(&mut hasher);
        let hash = hasher.finish();
        
        (hash % 100) < config.rollout_percentage as u64
    }
    
    /// Internal: Check if feature should be auto-disabled due to failures
    fn check_auto_disable(&self, feature: OptimizationFeature) {
        let config = self.config.read();
        if !config.auto_fallback {
            return;
        }
        
        let stats = self.usage_stats.read();
        if let Some(feature_stats) = stats.get(&feature) {
            let usage = feature_stats.usage_count.load(Ordering::Relaxed);
            let failures = feature_stats.fallback_count.load(Ordering::Relaxed);
            
            if usage >= config.min_sample_size {
                let failure_rate = failures as f64 / usage as f64;
                if failure_rate > config.max_failure_rate {
                    drop(stats);
                    drop(config);
                    self.disable(feature);
                    eprintln!("Auto-disabled feature {:?} due to high failure rate: {:.2}%", 
                             feature, failure_rate * 100.0);
                }
            }
        }
    }
}

impl FeatureStats {
    fn snapshot(&self) -> FeatureStatsSnapshot {
        FeatureStatsSnapshot {
            usage_count: self.usage_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            fallback_count: self.fallback_count.load(Ordering::Relaxed),
            total_time_micros: self.total_time_micros.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of feature statistics (non-atomic)
#[derive(Debug, Clone)]
pub struct FeatureStatsSnapshot {
    pub usage_count: usize,
    pub success_count: usize,
    pub fallback_count: usize,
    pub total_time_micros: usize,
}

impl FeatureStatsSnapshot {
    /// Calculate success rate (0.0-1.0)
    pub fn success_rate(&self) -> f64 {
        if self.usage_count > 0 {
            self.success_count as f64 / self.usage_count as f64
        } else {
            0.0
        }
    }
    
    /// Calculate average execution time in microseconds
    pub fn avg_time_micros(&self) -> f64 {
        if self.success_count > 0 {
            self.total_time_micros as f64 / self.success_count as f64
        } else {
            0.0
        }
    }
    
    /// Calculate fallback rate (0.0-1.0)
    pub fn fallback_rate(&self) -> f64 {
        if self.usage_count > 0 {
            self.fallback_count as f64 / self.usage_count as f64
        } else {
            0.0
        }
    }
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            gradual_rollout: false,
            rollout_percentage: 100,
            auto_fallback: true,
            max_failure_rate: 0.1, // 10% failure rate threshold
            min_sample_size: 100,
        }
    }
}

impl OptimizationFeature {
    /// Get all available features
    pub fn all() -> Vec<Self> {
        vec![
            Self::NanBoxedValues,
            Self::StringInterning,
            Self::ZeroCopyParsing,
            Self::ArenaAllocation,
            Self::SimdOptimizations,
            Self::LockFreeConcurrency,
            Self::MemoryMappedSymbols,
            Self::CompactAst,
            Self::UnifiedValueSystem,
            Self::PerformanceMonitoring,
        ]
    }
    
    /// Get feature name as string
    pub fn name(&self) -> &'static str {
        match self {
            Self::NanBoxedValues => "nan_boxed_values",
            Self::StringInterning => "string_interning",
            Self::ZeroCopyParsing => "zero_copy_parsing",
            Self::ArenaAllocation => "arena_allocation",
            Self::SimdOptimizations => "simd_optimizations",
            Self::LockFreeConcurrency => "lock_free_concurrency",
            Self::MemoryMappedSymbols => "memory_mapped_symbols",
            Self::CompactAst => "compact_ast",
            Self::UnifiedValueSystem => "unified_value_system",
            Self::PerformanceMonitoring => "performance_monitoring",
        }
    }
    
    /// Get feature description
    pub fn description(&self) -> &'static str {
        match self {
            Self::NanBoxedValues => "Use NaN-boxing for immediate value types",
            Self::StringInterning => "Global string interning for symbols and strings",
            Self::ZeroCopyParsing => "Zero-copy parsing with borrowed string slices",
            Self::ArenaAllocation => "Arena allocation for large objects",
            Self::SimdOptimizations => "SIMD-accelerated operations where possible",
            Self::LockFreeConcurrency => "Lock-free concurrent data structures",
            Self::MemoryMappedSymbols => "Memory-mapped symbol tables",
            Self::CompactAst => "Compact AST representation",
            Self::UnifiedValueSystem => "Unified optimized value system",
            Self::PerformanceMonitoring => "Performance monitoring and telemetry",
        }
    }
}

/// Global feature flag instance
static GLOBAL_OPTIMIZATION_FLAGS: std::sync::OnceLock<OptimizationFlags> = std::sync::OnceLock::new();

/// Get the global optimization flags instance
pub fn global_optimization_flags() -> &'static OptimizationFlags {
    GLOBAL_OPTIMIZATION_FLAGS.get_or_init(|| OptimizationFlags::new())
}

/// Convenience macros for feature checks
#[macro_export]
macro_rules! if_optimization_enabled {
    ($feature:expr, $enabled_block:block, $disabled_block:block) => {
        if $crate::feature::optimization_features::global_optimization_flags().is_enabled($feature) {
            $enabled_block
        } else {
            $disabled_block
        }
    };
    ($feature:expr, $enabled_block:block) => {
        if $crate::feature::optimization_features::global_optimization_flags().is_enabled($feature) {
            $enabled_block
        }
    };
}

#[macro_export]
macro_rules! with_optimization_fallback {
    ($feature:expr, $optimized:expr, $fallback:expr) => {{
        if $crate::feature::optimization_features::global_optimization_flags().is_enabled($feature) {
            let start_time = std::time::Instant::now();
            match $optimized {
                Ok(result) => {
                    let duration = start_time.elapsed().as_micros() as u64;
                    $crate::feature::optimization_features::global_optimization_flags().record_success($feature, duration);
                    Ok(result)
                }
                Err(e) => {
                    $crate::feature::optimization_features::global_optimization_flags().record_fallback($feature);
                    $fallback
                }
            }
        } else {
            $fallback
        }
    }};
}

/// Initialize Phase 8 optimization features with safe defaults
pub fn initialize_phase8_optimizations() {
    let flags = global_optimization_flags();
    
    // Enable basic optimizations that are well-tested
    flags.enable(OptimizationFeature::PerformanceMonitoring);
    flags.enable(OptimizationFeature::StringInterning);
    
    // Enable gradual rollout for experimental features
    flags.configure_rollout(10, true); // Start with 10% rollout
    
    // Gradually enable more features
    flags.enable(OptimizationFeature::NanBoxedValues);
    flags.enable(OptimizationFeature::ZeroCopyParsing);
    
    println!("Phase 8 optimization features initialized with gradual rollout");
}

/// Print optimization feature status report
pub fn print_optimization_report() {
    let flags = global_optimization_flags();
    
    println!("\n=== Phase 8 Optimization Feature Report ===");
    for feature in OptimizationFeature::all() {
        let enabled = flags.is_enabled(feature);
        let stats = flags.get_stats(feature);
        
        println!("{}:", feature.name());
        println!("  Enabled: {}", enabled);
        if let Some(stats) = stats {
            println!("  Usage: {} calls", stats.usage_count);
            println!("  Success Rate: {:.1}%", stats.success_rate() * 100.0);
            println!("  Avg Time: {:.1}μs", stats.avg_time_micros());
            if stats.fallback_count > 0 {
                println!("  Fallback Rate: {:.1}%", stats.fallback_rate() * 100.0);
            }
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimization_flag_basic() {
        let flags = OptimizationFlags::new();
        
        // Initially disabled
        assert!(!flags.is_enabled(OptimizationFeature::NanBoxedValues));
        
        // Enable and test
        flags.enable(OptimizationFeature::NanBoxedValues);
        assert!(flags.is_enabled(OptimizationFeature::NanBoxedValues));
        
        // Disable and test
        flags.disable(OptimizationFeature::NanBoxedValues);
        assert!(!flags.is_enabled(OptimizationFeature::NanBoxedValues));
    }
    
    #[test]
    fn test_optimization_statistics() {
        let flags = OptimizationFlags::new();
        flags.enable(OptimizationFeature::StringInterning);
        
        // Record some usage
        flags.record_success(OptimizationFeature::StringInterning, 100);
        flags.record_success(OptimizationFeature::StringInterning, 200);
        flags.record_fallback(OptimizationFeature::StringInterning);
        
        let stats = flags.get_stats(OptimizationFeature::StringInterning).unwrap();
        assert_eq!(stats.usage_count, 3);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.fallback_count, 1);
        assert_eq!(stats.success_rate(), 2.0 / 3.0);
    }
    
    #[test]
    fn test_optimization_gradual_rollout() {
        let flags = OptimizationFlags::new();
        flags.enable(OptimizationFeature::SimdOptimizations);
        flags.configure_rollout(50, false); // 50% rollout
        
        // Should sometimes be enabled, sometimes not
        // (This is probabilistic, but with a reasonable sample should work)
        let mut enabled_count = 0;
        for _ in 0..100 {
            if flags.is_enabled(OptimizationFeature::SimdOptimizations) {
                enabled_count += 1;
            }
        }
        
        // Should be roughly 50% (within reasonable variance)
        assert!(enabled_count > 20 && enabled_count < 80);
    }
}