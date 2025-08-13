//! Garbage collection policy management and configuration.
//!
//! This module provides various garbage collection strategies and policies
//! for fine-tuned memory management in dynamic evaluation contexts.

use std::time::Duration;

/// Garbage collection policy.
#[derive(Debug, Clone)]
pub struct GcPolicy {
    /// Policy name
    pub name: String,
    /// Trigger threshold (bytes)
    pub threshold: usize,
    /// Generation thresholds
    pub generation_thresholds: Vec<usize>,
    /// Collection frequency
    pub frequency: GcFrequency,
    /// Collection strategy
    pub strategy: GcStrategy,
}

/// Garbage collection frequency.
#[derive(Debug, Clone)]
pub enum GcFrequency {
    /// Manual collection only
    Manual,
    /// Periodic collection
    Periodic(Duration),
    /// Threshold-based collection
    Threshold(usize),
    /// Adaptive frequency
    Adaptive,
}

/// Garbage collection strategy.
#[derive(Debug, Clone, PartialEq)]
pub enum GcStrategy {
    /// Mark and sweep
    MarkAndSweep,
    /// Generational collection
    Generational,
    /// Incremental collection
    Incremental,
    /// Concurrent collection
    Concurrent,
}

impl GcPolicy {
    /// Creates a new GC policy.
    pub fn new(name: String, strategy: GcStrategy) -> Self {
        Self {
            name,
            threshold: 1024 * 1024, // 1MB default
            generation_thresholds: vec![1024 * 64, 1024 * 256, 1024 * 1024], // Default generations
            frequency: GcFrequency::Threshold(1024 * 1024),
            strategy,
        }
    }

    /// Creates a manual collection policy.
    pub fn manual(name: String) -> Self {
        Self {
            name,
            threshold: usize::MAX, // Never trigger automatically
            generation_thresholds: Vec::new(),
            frequency: GcFrequency::Manual,
            strategy: GcStrategy::MarkAndSweep,
        }
    }

    /// Creates a periodic collection policy.
    pub fn periodic(name: String, interval: Duration) -> Self {
        Self {
            name,
            threshold: usize::MAX,
            generation_thresholds: Vec::new(),
            frequency: GcFrequency::Periodic(interval),
            strategy: GcStrategy::MarkAndSweep,
        }
    }

    /// Creates a threshold-based collection policy.
    pub fn threshold(name: String, threshold: usize, strategy: GcStrategy) -> Self {
        Self {
            name,
            threshold,
            generation_thresholds: Vec::new(),
            frequency: GcFrequency::Threshold(threshold),
            strategy,
        }
    }

    /// Creates a generational collection policy.
    pub fn generational(name: String, thresholds: Vec<usize>) -> Self {
        Self {
            name,
            threshold: thresholds.first().copied().unwrap_or(1024 * 64),
            generation_thresholds: thresholds,
            frequency: GcFrequency::Threshold(1024 * 64),
            strategy: GcStrategy::Generational,
        }
    }

    /// Creates an adaptive collection policy.
    pub fn adaptive(name: String) -> Self {
        Self {
            name,
            threshold: 1024 * 256, // Start with 256KB
            generation_thresholds: vec![1024 * 32, 1024 * 128, 1024 * 512],
            frequency: GcFrequency::Adaptive,
            strategy: GcStrategy::Incremental,
        }
    }

    /// Sets the main threshold for collection.
    pub fn with_threshold(mut self, threshold: usize) -> Self {
        self.threshold = threshold;
        self
    }

    /// Sets the collection frequency.
    pub fn with_frequency(mut self, frequency: GcFrequency) -> Self {
        self.frequency = frequency;
        self
    }

    /// Sets the collection strategy.
    pub fn with_strategy(mut self, strategy: GcStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Sets generation thresholds for generational collection.
    pub fn with_generations(mut self, thresholds: Vec<usize>) -> Self {
        self.generation_thresholds = thresholds;
        self
    }

    /// Checks if this policy should trigger collection based on usage.
    pub fn should_collect(&self, current_usage: usize) -> bool {
        match &self.frequency {
            GcFrequency::Manual => false,
            GcFrequency::Threshold(threshold) => current_usage >= *threshold,
            GcFrequency::Periodic(_) => false, // Time-based, not usage-based
            GcFrequency::Adaptive => current_usage >= self.threshold,
        }
    }

    /// Gets the collection priority based on current conditions.
    pub fn collection_priority(&self, current_usage: usize) -> CollectionPriority {
        if current_usage >= self.threshold * 2 {
            CollectionPriority::Critical
        } else if current_usage >= self.threshold {
            CollectionPriority::High
        } else if current_usage >= self.threshold / 2 {
            CollectionPriority::Medium
        } else {
            CollectionPriority::Low
        }
    }

    /// Checks if this is a generational policy.
    pub fn is_generational(&self) -> bool {
        self.strategy == GcStrategy::Generational && !self.generation_thresholds.is_empty()
    }

    /// Gets the generation for a given object age.
    pub fn generation_for_age(&self, age: Duration) -> usize {
        if !self.is_generational() {
            return 0;
        }

        let age_ms = age.as_millis() as usize;
        
        for (generation, &threshold) in self.generation_thresholds.iter().enumerate() {
            if age_ms < threshold {
                return generation;
            }
        }
        
        self.generation_thresholds.len() // Oldest generation
    }
}

/// Collection priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CollectionPriority {
    /// Low priority - collection can wait
    Low,
    /// Medium priority - collection should happen soon
    Medium,
    /// High priority - collection needed
    High,
    /// Critical priority - immediate collection required
    Critical,
}

impl GcFrequency {
    /// Gets the time interval for periodic collection.
    pub fn interval(&self) -> Option<Duration> {
        match self {
            GcFrequency::Periodic(duration) => Some(*duration),
            _ => None,
        }
    }

    /// Checks if this frequency is manual.
    pub fn is_manual(&self) -> bool {
        matches!(self, GcFrequency::Manual)
    }

    /// Checks if this frequency is adaptive.
    pub fn is_adaptive(&self) -> bool {
        matches!(self, GcFrequency::Adaptive)
    }
}

impl GcStrategy {
    /// Checks if this strategy supports concurrent collection.
    pub fn supports_concurrent(&self) -> bool {
        matches!(self, GcStrategy::Concurrent | GcStrategy::Incremental)
    }

    /// Checks if this strategy is generational.
    pub fn is_generational(&self) -> bool {
        matches!(self, GcStrategy::Generational)
    }

    /// Gets the typical collection pause time for this strategy.
    pub fn typical_pause_time(&self) -> Duration {
        match self {
            GcStrategy::MarkAndSweep => Duration::from_millis(10),
            GcStrategy::Generational => Duration::from_millis(5),
            GcStrategy::Incremental => Duration::from_millis(2),
            GcStrategy::Concurrent => Duration::from_millis(1),
        }
    }

    /// Gets the memory overhead for this strategy.
    pub fn memory_overhead(&self) -> f64 {
        match self {
            GcStrategy::MarkAndSweep => 0.1, // 10%
            GcStrategy::Generational => 0.15, // 15%
            GcStrategy::Incremental => 0.2, // 20%
            GcStrategy::Concurrent => 0.25, // 25%
        }
    }
}

impl Default for GcPolicy {
    fn default() -> Self {
        Self::new("default".to_string(), GcStrategy::MarkAndSweep)
    }
}

impl CollectionPriority {
    /// Converts priority to a numeric score.
    pub fn score(&self) -> u8 {
        match self {
            CollectionPriority::Low => 1,
            CollectionPriority::Medium => 2,
            CollectionPriority::High => 3,
            CollectionPriority::Critical => 4,
        }
    }

    /// Creates priority from a usage ratio.
    pub fn from_usage_ratio(ratio: f64) -> Self {
        if ratio >= 0.9 {
            CollectionPriority::Critical
        } else if ratio >= 0.7 {
            CollectionPriority::High
        } else if ratio >= 0.5 {
            CollectionPriority::Medium
        } else {
            CollectionPriority::Low
        }
    }
}