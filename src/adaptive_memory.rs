//! Adaptive memory management strategy
//!
//! This module implements adaptive memory management that adjusts allocation
//! strategies based on runtime evaluation patterns and memory pressure.

use crate::memory_pool::{ContinuationPoolStats, PoolStats};
#[cfg(feature = "debug-tracing")]
use crate::stack_monitor::{OptimizationRecommendation, StackStatistics};

#[cfg(not(feature = "debug-tracing"))]
mod stub_types {
    #[derive(Debug, Clone, PartialEq)]
    pub enum OptimizationRecommendation {
        MemoryCompression,
        ContinuationInlining,
        ForceGarbageCollection,
        TailCallOptimization,
    }
    
    #[derive(Debug, Clone)]
    pub struct StackStatistics {
        pub stack_depth: usize,
        pub max_stack_depth: usize,
        pub continuation_count: usize,
        pub stack_memory_usage: usize,
        pub total_memory_estimate: usize,
        pub current_depth: usize,
        pub max_depth: usize,
        pub total_frames: usize,
        pub optimizations_applied: usize,
        pub average_frame_time: std::time::Duration,
        pub optimizable_frames: usize,
    }
    
    impl Default for StackStatistics {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl StackStatistics {
        pub fn new() -> Self {
            Self {
                stack_depth: 0,
                max_stack_depth: 0,
                continuation_count: 0,
                stack_memory_usage: 0,
                total_memory_estimate: 0,
                current_depth: 0,
                max_depth: 0,
                total_frames: 0,
                optimizations_applied: 0,
                average_frame_time: std::time::Duration::from_millis(0),
                optimizable_frames: 0,
            }
        }
    }
}

#[cfg(not(feature = "debug-tracing"))]
use stub_types::{OptimizationRecommendation, StackStatistics};

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Memory pressure levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryPressure {
    /// Low memory usage, can allocate freely
    Low,
    /// Moderate memory usage, start optimizing
    Moderate,
    /// High memory usage, aggressive optimization needed
    High,
    /// Critical memory usage, emergency measures
    Critical,
}

/// Memory allocation strategies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AllocationStrategy {
    /// Standard allocation with pools
    Standard,
    /// Aggressive pooling and reuse
    Aggressive,
    /// Conservative allocation, prefer stack
    Conservative,
    /// Emergency mode, minimal allocations
    Emergency,
}

/// Adaptive memory manager
pub struct AdaptiveMemoryManager {
    /// Current memory pressure level
    pressure_level: MemoryPressure,
    /// Current allocation strategy
    strategy: AllocationStrategy,
    /// Memory usage history for trend analysis
    usage_history: VecDeque<MemorySnapshot>,
    /// Configuration parameters
    config: MemoryConfig,
    /// Statistics about memory management decisions
    decisions: MemoryDecisionStats,
}

/// Snapshot of memory usage at a point in time
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// Timestamp of snapshot
    pub timestamp: Instant,
    /// Total estimated memory usage
    pub total_memory: usize,
    /// Pool statistics
    pub pool_stats: PoolStats,
    /// Continuation pool statistics  
    pub continuation_stats: ContinuationPoolStats,
    /// Stack statistics
    pub stack_stats: StackStatistics,
}

/// Configuration for adaptive memory management
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Memory usage threshold for moderate pressure (bytes)
    pub moderate_threshold: usize,
    /// Memory usage threshold for high pressure (bytes)
    pub high_threshold: usize,
    /// Memory usage threshold for critical pressure (bytes)
    pub critical_threshold: usize,
    /// History length for trend analysis
    pub history_length: usize,
    /// Minimum time between strategy changes
    pub strategy_change_cooldown: Duration,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            moderate_threshold: 50_000_000,  // 50MB
            high_threshold: 100_000_000,     // 100MB
            critical_threshold: 200_000_000, // 200MB
            history_length: 100,
            strategy_change_cooldown: Duration::from_secs(1),
        }
    }
}

/// Statistics about memory management decisions
#[derive(Debug, Clone)]
pub struct MemoryDecisionStats {
    /// Number of strategy changes
    pub strategy_changes: usize,
    /// Number of pressure level changes
    pub pressure_changes: usize,
    /// Time spent in each strategy
    pub strategy_durations: Vec<(AllocationStrategy, Duration)>,
    /// Last strategy change timestamp
    pub last_strategy_change: Option<Instant>,
}

impl AdaptiveMemoryManager {
    /// Create a new adaptive memory manager
    pub fn new() -> Self {
        Self::with_config(MemoryConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: MemoryConfig) -> Self {
        Self {
            pressure_level: MemoryPressure::Low,
            strategy: AllocationStrategy::Standard,
            usage_history: VecDeque::with_capacity(config.history_length),
            config,
            decisions: MemoryDecisionStats {
                strategy_changes: 0,
                pressure_changes: 0,
                strategy_durations: Vec::new(),
                last_strategy_change: None,
            },
        }
    }

    /// Update memory manager with current usage data
    pub fn update(
        &mut self,
        pool_stats: PoolStats,
        continuation_stats: ContinuationPoolStats,
        stack_stats: StackStatistics,
    ) {
        let snapshot = MemorySnapshot {
            timestamp: Instant::now(),
            total_memory: self.estimate_total_memory(
                &pool_stats,
                &continuation_stats,
                &stack_stats,
            ),
            pool_stats,
            continuation_stats,
            stack_stats,
        };

        // Add to history
        self.usage_history.push_back(snapshot.clone());
        if self.usage_history.len() > self.config.history_length {
            self.usage_history.pop_front();
        }

        // Update pressure level
        self.update_pressure_level(&snapshot);

        // Update allocation strategy
        self.update_allocation_strategy(&snapshot);
    }

    /// Estimate total memory usage from various sources
    pub fn estimate_total_memory(
        &self,
        pool_stats: &PoolStats,
        continuation_stats: &ContinuationPoolStats,
        stack_stats: &StackStatistics,
    ) -> usize {
        // Estimate memory from various sources
        let pool_memory = pool_stats.values_in_recycle_pool * 64 // Estimate 64 bytes per value
            + pool_stats.small_integers_cached * 32 // Estimate 32 bytes per integer
            + pool_stats.symbols_interned * 50; // Estimate 50 bytes per symbol

        let continuation_memory = continuation_stats.identity_pooled * 128 // Estimate 128 bytes per continuation
            + continuation_stats.total_created * 64; // Additional overhead

        let stack_memory = stack_stats.total_memory_estimate;

        pool_memory + continuation_memory + stack_memory
    }

    /// Update memory pressure level based on current usage
    fn update_pressure_level(&mut self, snapshot: &MemorySnapshot) {
        let old_pressure = self.pressure_level;

        self.pressure_level = if snapshot.total_memory >= self.config.critical_threshold {
            MemoryPressure::Critical
        } else if snapshot.total_memory >= self.config.high_threshold {
            MemoryPressure::High
        } else if snapshot.total_memory >= self.config.moderate_threshold {
            MemoryPressure::Moderate
        } else {
            MemoryPressure::Low
        };

        if old_pressure != self.pressure_level {
            self.decisions.pressure_changes += 1;
        }
    }

    /// Update allocation strategy based on memory pressure and trends
    fn update_allocation_strategy(&mut self, _snapshot: &MemorySnapshot) {
        let old_strategy = self.strategy;
        let now = Instant::now();

        // Check cooldown period
        if let Some(last_change) = self.decisions.last_strategy_change {
            if now.duration_since(last_change) < self.config.strategy_change_cooldown {
                return; // Still in cooldown
            }
        }

        let new_strategy = match self.pressure_level {
            MemoryPressure::Low => {
                // Low pressure: use standard allocation
                AllocationStrategy::Standard
            }
            MemoryPressure::Moderate => {
                // Moderate pressure: start optimizing
                if self.is_memory_trending_up() {
                    AllocationStrategy::Aggressive
                } else {
                    AllocationStrategy::Standard
                }
            }
            MemoryPressure::High => {
                // High pressure: aggressive optimization
                AllocationStrategy::Aggressive
            }
            MemoryPressure::Critical => {
                // Critical pressure: emergency measures
                AllocationStrategy::Emergency
            }
        };

        if new_strategy != old_strategy {
            // Record strategy duration
            if let Some(last_change) = self.decisions.last_strategy_change {
                let duration = now.duration_since(last_change);
                self.decisions
                    .strategy_durations
                    .push((old_strategy, duration));
            }

            self.strategy = new_strategy;
            self.decisions.strategy_changes += 1;
            self.decisions.last_strategy_change = Some(now);
        }
    }

    /// Check if memory usage is trending upward
    fn is_memory_trending_up(&self) -> bool {
        if self.usage_history.len() < 3 {
            return false;
        }

        let start_index = self.usage_history.len() - 3;
        let recent: Vec<&MemorySnapshot> = self.usage_history.iter().skip(start_index).collect();
        let mut increases = 0;

        for window in recent.windows(2) {
            if window[1].total_memory > window[0].total_memory {
                increases += 1;
            }
        }

        increases >= 2 // At least 2 out of 3 increases
    }

    /// Get optimization recommendations based on current state
    pub fn get_optimization_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        match self.pressure_level {
            MemoryPressure::Low => {
                // No urgent optimizations needed
            }
            MemoryPressure::Moderate => {
                recommendations.push(OptimizationRecommendation::MemoryCompression);
            }
            MemoryPressure::High => {
                recommendations.push(OptimizationRecommendation::MemoryCompression);
                recommendations.push(OptimizationRecommendation::ContinuationInlining);
            }
            MemoryPressure::Critical => {
                recommendations.push(OptimizationRecommendation::ForceGarbageCollection);
                recommendations.push(OptimizationRecommendation::MemoryCompression);
                recommendations.push(OptimizationRecommendation::ContinuationInlining);
                recommendations.push(OptimizationRecommendation::TailCallOptimization);
            }
        }

        recommendations
    }

    /// Get allocation parameters for current strategy
    pub fn allocation_parameters(&self) -> AllocationParameters {
        match self.strategy {
            AllocationStrategy::Standard => AllocationParameters {
                pool_size_multiplier: 1.0,
                aggressive_recycling: false,
                prefer_stack_allocation: false,
                gc_frequency_multiplier: 1.0,
            },
            AllocationStrategy::Aggressive => AllocationParameters {
                pool_size_multiplier: 1.5,
                aggressive_recycling: true,
                prefer_stack_allocation: true,
                gc_frequency_multiplier: 2.0,
            },
            AllocationStrategy::Conservative => AllocationParameters {
                pool_size_multiplier: 0.8,
                aggressive_recycling: false,
                prefer_stack_allocation: true,
                gc_frequency_multiplier: 0.5,
            },
            AllocationStrategy::Emergency => AllocationParameters {
                pool_size_multiplier: 0.5,
                aggressive_recycling: true,
                prefer_stack_allocation: true,
                gc_frequency_multiplier: 4.0,
            },
        }
    }

    /// Get current state information
    pub fn state_info(&self) -> AdaptiveMemoryState {
        AdaptiveMemoryState {
            pressure_level: self.pressure_level,
            strategy: self.strategy,
            history_length: self.usage_history.len(),
            total_memory: self
                .usage_history
                .back()
                .map(|s| s.total_memory)
                .unwrap_or(0),
            decisions: self.decisions.clone(),
        }
    }
}

impl Default for AdaptiveMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameters for memory allocation under current strategy
#[derive(Debug, Clone)]
pub struct AllocationParameters {
    /// Multiplier for pool sizes
    pub pool_size_multiplier: f64,
    /// Whether to aggressively recycle objects
    pub aggressive_recycling: bool,
    /// Whether to prefer stack allocation over heap
    pub prefer_stack_allocation: bool,
    /// Multiplier for garbage collection frequency
    pub gc_frequency_multiplier: f64,
}

/// Current state of adaptive memory manager
#[derive(Debug, Clone)]
pub struct AdaptiveMemoryState {
    /// Current memory pressure level
    pub pressure_level: MemoryPressure,
    /// Current allocation strategy
    pub strategy: AllocationStrategy,
    /// Number of snapshots in history
    pub history_length: usize,
    /// Current total memory usage estimate
    pub total_memory: usize,
    /// Decision statistics
    pub decisions: MemoryDecisionStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_stats() -> (PoolStats, ContinuationPoolStats, StackStatistics) {
        let pool_stats = PoolStats {
            small_integers_cached: 256,
            values_in_recycle_pool: 100,
            symbols_interned: 50,
            total_interned_storage: 50,
        };

        let continuation_stats = ContinuationPoolStats {
            identity_pooled: 50,
            total_recycled: 100,
            total_created: 200,
            recycle_rate: 50.0,
        };

        let stack_stats = StackStatistics {
            current_depth: 10,
            max_depth: 20,
            total_frames: 1000,
            optimizations_applied: 5,
            average_frame_time: Duration::from_millis(1),
            total_memory_estimate: 5000,
            optimizable_frames: 8,
        };

        (pool_stats, continuation_stats, stack_stats)
    }

    #[test]
    fn test_adaptive_memory_basic_operations() {
        let mut manager = AdaptiveMemoryManager::new();
        let (pool_stats, continuation_stats, stack_stats) = create_test_stats();

        // Initial state should be low pressure
        assert_eq!(manager.pressure_level, MemoryPressure::Low);
        assert_eq!(manager.strategy, AllocationStrategy::Standard);

        // Update with normal usage
        manager.update(pool_stats, continuation_stats, stack_stats);

        // Should still be low pressure for these stats
        assert_eq!(manager.pressure_level, MemoryPressure::Low);
    }

    #[test]
    fn test_memory_pressure_escalation() {
        let mut manager = AdaptiveMemoryManager::with_config(MemoryConfig {
            moderate_threshold: 1000, // Very low thresholds for testing
            high_threshold: 2000,
            critical_threshold: 3000,
            ..Default::default()
        });

        let (pool_stats, continuation_stats, mut stack_stats) = create_test_stats();

        // Set a very high stack memory to ensure critical pressure
        stack_stats.total_memory_estimate = 10000; // Much higher than critical threshold

        manager.update(pool_stats, continuation_stats, stack_stats);

        assert_eq!(manager.pressure_level, MemoryPressure::Critical);
        assert_eq!(manager.strategy, AllocationStrategy::Emergency);
    }

    #[test]
    fn test_optimization_recommendations() {
        let mut manager = AdaptiveMemoryManager::with_config(MemoryConfig {
            critical_threshold: 1000, // Low threshold for testing
            ..Default::default()
        });

        let (pool_stats, continuation_stats, mut stack_stats) = create_test_stats();
        stack_stats.total_memory_estimate = 1500; // Critical pressure

        manager.update(pool_stats, continuation_stats, stack_stats);

        let recommendations = manager.get_optimization_recommendations();
        assert!(recommendations.contains(&OptimizationRecommendation::ForceGarbageCollection));
        assert!(recommendations.contains(&OptimizationRecommendation::MemoryCompression));
    }

    #[test]
    fn test_allocation_parameters() {
        let manager = AdaptiveMemoryManager::new();

        let params = manager.allocation_parameters();
        assert_eq!(params.pool_size_multiplier, 1.0); // Standard strategy
        assert!(!params.aggressive_recycling);
    }

    #[test]
    fn test_memory_trending() {
        let mut manager = AdaptiveMemoryManager::new();
        let (pool_stats, continuation_stats, mut stack_stats) = create_test_stats();

        // Add samples with increasing memory usage
        for i in 1..=5 {
            stack_stats.total_memory_estimate = i * 1000;
            manager.update(
                pool_stats.clone(),
                continuation_stats.clone(),
                stack_stats.clone(),
            );
        }

        // Should detect upward trend
        assert!(manager.is_memory_trending_up());
    }
}
