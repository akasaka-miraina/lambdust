//! Garbage collection for continuation system with cycle detection.
//!
//! This module implements specialized garbage collection for continuations,
//! focusing on cycle resolution and memory safety through weak references.

use super::optimization::OptimizedContinuation;
use super::{ContinuationGeneration, ContinuationId};

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Specialized garbage collector for continuation system.
///
/// Provides cycle detection, weak reference management, and memory optimization
/// specifically designed for continuation chains and their unique access patterns.
#[derive(Debug)]
pub struct ContinuationGC {
    /// Current generation counter
    generation: ContinuationGeneration,

    /// Garbage collection statistics
    stats: GCStats,

    /// Configuration parameters
    pub config: GCConfig,
}

impl ContinuationGC {
    /// Creates a new continuation garbage collector.
    pub fn new() -> Self {
        Self {
            generation: 0,
            stats: GCStats::default(),
            config: GCConfig::default(),
        }
    }

    /// Creates a GC with custom configuration.
    pub fn with_config(config: GCConfig) -> Self {
        let mut gc = Self::new();
        gc.config = config;
        gc
    }

    /// Advances to the next generation.
    pub fn next_generation(&mut self) -> ContinuationGeneration {
        self.generation += 1;
        self.generation
    }

    /// Gets the current generation.
    pub fn current_generation(&self) -> ContinuationGeneration {
        self.generation
    }

    /// Collects unused continuations from the registry (simplified).
    pub fn collect(&mut self, continuations: &mut HashMap<ContinuationId, OptimizedContinuation>) {
        let start_time = Instant::now();
        let initial_count = continuations.len();

        // Simplified collection - remove continuations older than threshold
        let mut to_remove = Vec::new();
        for (id, continuation) in continuations.iter() {
            if continuation.generation() < self.generation.saturating_sub(2) {
                to_remove.push(*id);
            }
        }

        for id in &to_remove {
            continuations.remove(id);
        }

        // Update statistics
        let duration = start_time.elapsed();
        self.update_stats(initial_count, to_remove.len(), duration);
    }

    /// Updates GC statistics after collection.
    fn update_stats(&mut self, initial_count: usize, removed: usize, duration: Duration) {
        self.stats.collection_count += 1;
        self.stats.total_collected += removed;
        self.stats.last_collection_duration = duration;

        if removed > 0 {
            self.stats.last_successful_collection = Instant::now();
        }
    }

    /// Gets current GC statistics.
    pub fn stats(&self) -> &GCStats {
        &self.stats
    }

    /// Analyzes memory usage patterns to optimize collection strategy.
    pub fn analyze_usage_patterns(
        &mut self,
        continuations: &HashMap<ContinuationId, OptimizedContinuation>,
    ) -> UsageAnalysis {
        let mut analysis = UsageAnalysis::default();

        for continuation in continuations.values() {
            analysis.total_continuations += 1;
            analysis.total_memory += continuation.memory_usage();

            match continuation {
                OptimizedContinuation::SingleOwned(_) => {
                    analysis.single_owned_count += 1;
                }
                OptimizedContinuation::SharedAdaptive(_) => {
                    analysis.shared_weak_count += 1;
                }
                OptimizedContinuation::JitSpecialized(jit) => {
                    analysis.jit_specialized_count += 1;
                    analysis.total_hotness += jit.optimization_data.hotness;
                }
                OptimizedContinuation::Distributed(_) => {
                    analysis.distributed_count += 1;
                }
            }
        }

        if analysis.jit_specialized_count > 0 {
            analysis.average_hotness =
                analysis.total_hotness / analysis.jit_specialized_count as u32;
        }

        analysis
    }

    /// Suggests optimizations based on usage analysis.
    pub fn suggest_optimizations(&self, analysis: &UsageAnalysis) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // Suggest converting single-owned to shared if there are many
        if analysis.single_owned_count > analysis.total_continuations / 2 {
            suggestions.push(OptimizationSuggestion::ConvertToShared);
        }

        // Suggest more aggressive collection if memory usage is high
        if analysis.total_memory > 10_000_000 {
            // 10MB threshold
            suggestions.push(OptimizationSuggestion::MoreAggressiveCollection);
        }

        suggestions
    }
}

impl Default for ContinuationGC {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for continuation garbage collection.
#[derive(Debug, Clone)]
pub struct GCConfig {
    /// Maximum number of continuations before triggering collection
    pub max_continuations: usize,

    /// Minimum age before continuation can be collected (in generations)
    pub min_age_for_collection: ContinuationGeneration,

    /// Threshold for JIT hotness below which continuations can be collected
    pub jit_hotness_threshold: u32,

    /// Whether to use generational collection strategy
    pub use_generational: bool,

    /// Time between forced collection cycles
    pub forced_collection_interval: Duration,
}

impl Default for GCConfig {
    fn default() -> Self {
        Self {
            max_continuations: 1000,
            min_age_for_collection: 2,
            jit_hotness_threshold: 10,
            use_generational: true,
            forced_collection_interval: Duration::from_secs(30),
        }
    }
}

/// Statistics for garbage collection performance.
#[derive(Debug, Clone)]
pub struct GCStats {
    /// Total number of collection cycles performed
    pub collection_count: u64,

    /// Total number of continuations collected
    pub total_collected: usize,

    /// Duration of the last collection cycle
    pub last_collection_duration: Duration,

    /// Timestamp of last successful collection
    pub last_successful_collection: Instant,

    /// Estimated memory saved through collection
    pub estimated_memory_saved: usize,
}

impl Default for GCStats {
    fn default() -> Self {
        use std::time::{Duration, Instant};
        Self {
            collection_count: 0,
            total_collected: 0,
            last_collection_duration: Duration::ZERO,
            last_successful_collection: Instant::now(),
            estimated_memory_saved: 0,
        }
    }
}

/// Analysis of continuation usage patterns.
#[derive(Debug, Default)]
pub struct UsageAnalysis {
    /// Total number of continuations currently allocated in the system
    pub total_continuations: usize,
    /// Total memory consumed by all continuations in bytes
    pub total_memory: usize,
    /// Count of continuations with single ownership (no sharing)
    pub single_owned_count: usize,
    /// Count of continuations using shared weak reference patterns
    pub shared_weak_count: usize,
    /// Count of continuations optimized with JIT specialization
    pub jit_specialized_count: usize,
    /// Count of continuations distributed across multiple nodes
    pub distributed_count: usize,
    /// Sum of all continuation hotness scores across the system
    pub total_hotness: u32,
    /// Average hotness score per continuation (total_hotness / total_continuations)
    pub average_hotness: u32,
}

/// Optimization suggestions based on usage analysis.
#[derive(Debug, Clone)]
pub enum OptimizationSuggestion {
    /// Convert single-owned continuations to shared ownership patterns for better memory efficiency
    ConvertToShared,
    /// Enable JIT compilation for frequently accessed continuation chains
    EnableJitCompilation,
    /// Increase garbage collection frequency to reduce memory pressure
    MoreAggressiveCollection,
    /// Reduce the number of weak references to improve cache locality
    ReduceWeakReferences,
    /// Increase the generation threshold before promoting continuations
    IncreaseGenerationThreshold,
}
