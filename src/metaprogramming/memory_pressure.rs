//! Memory pressure monitoring and alerting system.
//!
//! This module provides facilities for monitoring memory pressure levels
//! and triggering appropriate responses when memory usage becomes critical.

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

/// Memory pressure monitor.
#[derive(Debug)]
pub struct MemoryPressureMonitor {
    /// Current pressure level
    pub pressure_level: MemoryPressureLevel,
    /// Pressure history
    pressure_history: VecDeque<MemoryPressurePoint>,
    /// Warning thresholds
    warning_thresholds: HashMap<MemoryPressureLevel, usize>,
}

/// Memory pressure level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryPressureLevel {
    /// Low memory pressure.
    Low,
    /// Medium memory pressure.
    Medium,
    /// High memory pressure.
    High,
    /// Critical memory pressure.
    Critical,
}

/// Point in memory pressure history.
#[derive(Debug, Clone)]
pub struct MemoryPressurePoint {
    /// Timestamp
    pub timestamp: Instant,
    /// Pressure level
    pub level: MemoryPressureLevel,
    /// Memory usage at this point
    pub usage: usize,
}

impl Default for MemoryPressureMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryPressureMonitor {
    /// Creates a new memory pressure monitor with default thresholds.
    pub fn new() -> Self {
        let mut warning_thresholds = HashMap::new();
        warning_thresholds.insert(MemoryPressureLevel::Medium, 1024 * 1024 * 100); // 100MB
        warning_thresholds.insert(MemoryPressureLevel::High, 1024 * 1024 * 500);   // 500MB
        warning_thresholds.insert(MemoryPressureLevel::Critical, 1024 * 1024 * 1000); // 1GB

        Self {
            pressure_level: MemoryPressureLevel::Low,
            pressure_history: VecDeque::new(),
            warning_thresholds,
        }
    }

    /// Creates a monitor with custom thresholds.
    pub fn with_thresholds(thresholds: HashMap<MemoryPressureLevel, usize>) -> Self {
        Self {
            pressure_level: MemoryPressureLevel::Low,
            pressure_history: VecDeque::new(),
            warning_thresholds: thresholds,
        }
    }

    /// Updates the pressure level based on current usage.
    pub fn update_pressure(&mut self, current_usage: usize) -> MemoryPressureLevel {
        let new_level = self.calculate_pressure_level(current_usage);
        
        if new_level != self.pressure_level {
            // Record pressure change
            let point = MemoryPressurePoint {
                timestamp: Instant::now(),
                level: new_level,
                usage: current_usage,
            };
            
            // Keep history bounded
            if self.pressure_history.len() >= 1000 {
                self.pressure_history.pop_front();
            }
            self.pressure_history.push_back(point);
            
            self.pressure_level = new_level;
        }
        
        new_level
    }

    /// Calculates pressure level based on current usage.
    pub fn calculate_pressure_level(&self, usage: usize) -> MemoryPressureLevel {
        if let Some(&critical_threshold) = self.warning_thresholds.get(&MemoryPressureLevel::Critical) {
            if usage >= critical_threshold {
                return MemoryPressureLevel::Critical;
            }
        }
        
        if let Some(&high_threshold) = self.warning_thresholds.get(&MemoryPressureLevel::High) {
            if usage >= high_threshold {
                return MemoryPressureLevel::High;
            }
        }
        
        if let Some(&medium_threshold) = self.warning_thresholds.get(&MemoryPressureLevel::Medium) {
            if usage >= medium_threshold {
                return MemoryPressureLevel::Medium;
            }
        }
        
        MemoryPressureLevel::Low
    }

    /// Gets the current pressure level.
    pub fn current_level(&self) -> MemoryPressureLevel {
        self.pressure_level
    }

    /// Sets a threshold for a pressure level.
    pub fn set_threshold(&mut self, level: MemoryPressureLevel, threshold: usize) {
        self.warning_thresholds.insert(level, threshold);
    }

    /// Gets a threshold for a pressure level.
    pub fn get_threshold(&self, level: MemoryPressureLevel) -> Option<usize> {
        self.warning_thresholds.get(&level).copied()
    }

    /// Gets the pressure history.
    pub fn get_history(&self) -> &VecDeque<MemoryPressurePoint> {
        &self.pressure_history
    }

    /// Gets recent pressure points.
    pub fn get_recent_history(&self, count: usize) -> Vec<&MemoryPressurePoint> {
        self.pressure_history.iter().rev().take(count).collect()
    }

    /// Clears the pressure history.
    pub fn clear_history(&mut self) {
        self.pressure_history.clear();
    }

    /// Checks if pressure has been consistently high.
    pub fn is_pressure_sustained(&self, level: MemoryPressureLevel, duration: std::time::Duration) -> bool {
        let threshold_time = Instant::now() - duration;
        
        self.pressure_history.iter()
            .filter(|p| p.timestamp >= threshold_time)
            .all(|p| p.level >= level)
    }

    /// Gets pressure statistics.
    pub fn get_statistics(&self) -> PressureStatistics {
        let mut stats = PressureStatistics {
            low_count: 0,
            medium_count: 0,
            high_count: 0,
            critical_count: 0,
            total_points: self.pressure_history.len(),
            average_usage: 0.0,
        };

        if stats.total_points == 0 {
            return stats;
        }

        let mut total_usage = 0;
        
        for point in &self.pressure_history {
            total_usage += point.usage;
            
            match point.level {
                MemoryPressureLevel::Low => stats.low_count += 1,
                MemoryPressureLevel::Medium => stats.medium_count += 1,
                MemoryPressureLevel::High => stats.high_count += 1,
                MemoryPressureLevel::Critical => stats.critical_count += 1,
            }
        }
        
        stats.average_usage = total_usage as f64 / stats.total_points as f64;
        stats
    }
}

/// Pressure monitoring statistics.
#[derive(Debug, Clone)]
pub struct PressureStatistics {
    /// Number of low pressure measurements
    pub low_count: usize,
    /// Number of medium pressure measurements
    pub medium_count: usize,
    /// Number of high pressure measurements
    pub high_count: usize,
    /// Number of critical pressure measurements
    pub critical_count: usize,
    /// Total pressure points across all measurements
    pub total_points: usize,
    /// Average memory usage across all measurements
    pub average_usage: f64,
}

impl MemoryPressureLevel {
    /// Gets the numeric priority of this pressure level.
    pub fn priority(&self) -> u8 {
        match self {
            MemoryPressureLevel::Low => 0,
            MemoryPressureLevel::Medium => 1,
            MemoryPressureLevel::High => 2,
            MemoryPressureLevel::Critical => 3,
        }
    }

    /// Gets the name of this pressure level.
    pub fn name(&self) -> &'static str {
        match self {
            MemoryPressureLevel::Low => "Low",
            MemoryPressureLevel::Medium => "Medium",
            MemoryPressureLevel::High => "High",
            MemoryPressureLevel::Critical => "Critical",
        }
    }

    /// Checks if this level indicates memory stress.
    pub fn is_stressed(&self) -> bool {
        matches!(self, MemoryPressureLevel::High | MemoryPressureLevel::Critical)
    }

    /// Checks if this level requires immediate action.
    pub fn requires_action(&self) -> bool {
        matches!(self, MemoryPressureLevel::Critical)
    }

    /// Gets all pressure levels in order.
    pub fn all_levels() -> &'static [MemoryPressureLevel] {
        &[
            MemoryPressureLevel::Low,
            MemoryPressureLevel::Medium,
            MemoryPressureLevel::High,
            MemoryPressureLevel::Critical,
        ]
    }
}

impl PartialOrd for MemoryPressureLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MemoryPressureLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&other.priority())
    }
}

impl MemoryPressurePoint {
    /// Creates a new pressure point.
    pub fn new(level: MemoryPressureLevel, usage: usize) -> Self {
        Self {
            timestamp: Instant::now(),
            level,
            usage,
        }
    }

    /// Gets the age of this pressure point.
    pub fn age(&self) -> std::time::Duration {
        Instant::now() - self.timestamp
    }

    /// Gets usage in MB.
    pub fn usage_mb(&self) -> f64 {
        self.usage as f64 / (1024.0 * 1024.0)
    }

    /// Gets usage in KB.
    pub fn usage_kb(&self) -> f64 {
        self.usage as f64 / 1024.0
    }
}

impl PressureStatistics {
    /// Gets the percentage of time at each pressure level.
    pub fn level_percentages(&self) -> HashMap<MemoryPressureLevel, f64> {
        if self.total_points == 0 {
            return HashMap::new();
        }

        let total = self.total_points as f64;
        let mut percentages = HashMap::new();
        
        percentages.insert(MemoryPressureLevel::Low, self.low_count as f64 / total * 100.0);
        percentages.insert(MemoryPressureLevel::Medium, self.medium_count as f64 / total * 100.0);
        percentages.insert(MemoryPressureLevel::High, self.high_count as f64 / total * 100.0);
        percentages.insert(MemoryPressureLevel::Critical, self.critical_count as f64 / total * 100.0);
        
        percentages
    }

    /// Gets the percentage of time under stress.
    pub fn stress_percentage(&self) -> f64 {
        if self.total_points == 0 {
            return 0.0;
        }

        let stressed_count = self.high_count + self.critical_count;
        stressed_count as f64 / self.total_points as f64 * 100.0
    }

    /// Gets average usage in MB.
    pub fn average_usage_mb(&self) -> f64 {
        self.average_usage / (1024.0 * 1024.0)
    }
}