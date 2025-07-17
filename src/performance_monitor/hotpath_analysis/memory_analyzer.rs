//! Memory Access Analysis Module
//!
//! このモジュールはメモリアクセスパターン解析機能を実装します。
//! キャッシュシミュレーション、ローカリティ解析、GC影響解析を含みます。

use crate::ast::Expr;
use crate::error::Result;
use super::core_types::{
    MemoryAccessPattern, MemoryLocation, MemoryAccessType, StridePattern,
    CacheSimulator, MemoryLocalityAnalyzer, GCImpactAnalyzer,
};
use std::collections::HashMap;
use std::time::Instant;

/// Memory access pattern analysis
#[derive(Debug)]
pub struct MemoryAccessAnalyzer {
    /// Memory access patterns
    pub access_patterns: HashMap<String, MemoryAccessPattern>,
    
    /// Cache behavior simulation
    pub cache_simulator: CacheSimulator,
    
    /// Memory locality analysis
    pub locality_analyzer: MemoryLocalityAnalyzer,
    
    /// Garbage collection impact analysis
    pub gc_impact_analyzer: GCImpactAnalyzer,
}

impl MemoryAccessAnalyzer {
    #[must_use] 
    /// Create a new memory access analyzer with default configuration
    pub fn new() -> Self { 
        Self { 
            access_patterns: HashMap::new(), 
            cache_simulator: CacheSimulator, 
            locality_analyzer: MemoryLocalityAnalyzer, 
            gc_impact_analyzer: GCImpactAnalyzer,
        } 
    }
    
    /// Analyze memory access patterns for an expression
    /// 
    /// Tracks memory usage patterns and updates efficiency metrics
    /// for performance optimization analysis.
    pub fn analyze_access_pattern(&mut self, expr_hash: &str, _expr: &Expr, memory_usage: usize) -> Result<()> {
        let pattern = self.access_patterns.entry(expr_hash.to_string()).or_insert_with(|| {
            MemoryAccessPattern {
                read_locations: Vec::new(),
                write_locations: Vec::new(),
                stride_pattern: StridePattern::Sequential,
                cache_miss_rate: 0.0,
                bandwidth_utilization: 0.0,
            }
        });
        
        // Simulate memory access
        pattern.read_locations.push(MemoryLocation {
            address: memory_usage, // Simplified - would use actual memory addresses
            timestamp: Instant::now(),
            access_type: MemoryAccessType::SequentialRead,
            size: memory_usage,
        });
        
        // Update stride pattern based on access history
        Self::update_stride_pattern_static(pattern);
        
        // Update cache statistics
        Self::update_cache_statistics_static(pattern, memory_usage);
        
        Ok(())
    }
    
    /// Update stride pattern based on access history
    fn update_stride_pattern_static(pattern: &mut MemoryAccessPattern) {
        if pattern.read_locations.len() >= 2 {
            let recent_accesses = &pattern.read_locations[pattern.read_locations.len().saturating_sub(10)..];
            let strides: Vec<isize> = recent_accesses.windows(2)
                .map(|window| window[1].address as isize - window[0].address as isize)
                .collect();
            
            if strides.len() >= 2 {
                // Check if all strides are the same
                if strides.windows(2).all(|w| w[0] == w[1]) {
                    if strides[0] == 1 {
                        pattern.stride_pattern = StridePattern::Sequential;
                    } else {
                        pattern.stride_pattern = StridePattern::FixedStride { stride: strides[0] };
                    }
                } else {
                    // Check if pattern is irregular but repeating
                    if strides.len() >= 4 && strides[0..2] == strides[2..4] {
                        pattern.stride_pattern = StridePattern::Irregular { 
                            pattern: strides[0..2].to_vec() 
                        };
                    } else {
                        pattern.stride_pattern = StridePattern::Random;
                    }
                }
            }
        }
    }
    
    /// Update cache simulation statistics
    fn update_cache_statistics_static(pattern: &mut MemoryAccessPattern, memory_usage: usize) {
        // Simplified cache miss calculation
        // In a real implementation, this would simulate cache behavior
        pattern.cache_miss_rate = match memory_usage {
            0..=64 => 0.01,      // Very likely to be in L1 cache
            65..=4096 => 0.05,   // Likely to be in L2 cache
            4097..=65536 => 0.15, // May be in L3 cache
            _ => 0.3,            // Likely cache miss
        };
        
        // Bandwidth utilization estimation
        pattern.bandwidth_utilization = match pattern.stride_pattern {
            StridePattern::Sequential => 0.9,
            StridePattern::FixedStride { stride } if stride.abs() <= 8 => 0.7,
            StridePattern::FixedStride { .. } => 0.4,
            StridePattern::Irregular { .. } => 0.3,
            StridePattern::Random => 0.1,
        };
    }
    
    /// Calculate overall memory efficiency score
    /// 
    /// Returns a score between 0.0 and 1.0 indicating memory efficiency
    /// based on access patterns and allocation behavior.
    #[must_use] pub fn calculate_efficiency_score(&self) -> f64 {
        if self.access_patterns.is_empty() {
            return 1.0;
        }
        
        let total_cache_miss_rate: f64 = self.access_patterns.values()
            .map(|pattern| pattern.cache_miss_rate)
            .sum();
        
        let total_bandwidth_utilization: f64 = self.access_patterns.values()
            .map(|pattern| pattern.bandwidth_utilization)
            .sum();
        
        let avg_cache_hit_rate = 1.0 - (total_cache_miss_rate / self.access_patterns.len() as f64);
        let avg_bandwidth_utilization = total_bandwidth_utilization / self.access_patterns.len() as f64;
        
        // Weighted combination of cache hit rate and bandwidth utilization
        avg_cache_hit_rate * 0.6 + avg_bandwidth_utilization * 0.4
    }
    
    /// Get memory access pattern for an expression
    #[must_use] pub fn get_access_pattern(&self, expr_hash: &str) -> Option<&MemoryAccessPattern> {
        self.access_patterns.get(expr_hash)
    }
    
    /// Get memory access statistics
    #[must_use] pub fn get_memory_statistics(&self) -> MemoryStatistics {
        let total_reads: usize = self.access_patterns.values()
            .map(|pattern| pattern.read_locations.len())
            .sum();
        
        let total_writes: usize = self.access_patterns.values()
            .map(|pattern| pattern.write_locations.len())
            .sum();
        
        let total_bytes_accessed: usize = self.access_patterns.values()
            .flat_map(|pattern| &pattern.read_locations)
            .map(|location| location.size)
            .sum();
        
        let average_cache_miss_rate = if self.access_patterns.is_empty() {
            0.0
        } else {
            self.access_patterns.values()
                .map(|pattern| pattern.cache_miss_rate)
                .sum::<f64>() / self.access_patterns.len() as f64
        };
        
        MemoryStatistics {
            total_expressions_tracked: self.access_patterns.len(),
            total_reads,
            total_writes,
            total_bytes_accessed,
            average_cache_miss_rate,
            efficiency_score: self.calculate_efficiency_score(),
        }
    }
    
    /// Identify memory hotspots
    #[must_use] pub fn identify_memory_hotspots(&self, threshold: usize) -> Vec<MemoryHotspot> {
        let mut hotspots = Vec::new();
        
        for (expr_hash, pattern) in &self.access_patterns {
            let total_accesses = pattern.read_locations.len() + pattern.write_locations.len();
            let total_bytes: usize = pattern.read_locations.iter()
                .chain(pattern.write_locations.iter())
                .map(|location| location.size)
                .sum();
            
            if total_bytes >= threshold {
                hotspots.push(MemoryHotspot {
                    expression: expr_hash.clone(),
                    total_accesses,
                    total_bytes_accessed: total_bytes,
                    cache_miss_rate: pattern.cache_miss_rate,
                    bandwidth_utilization: pattern.bandwidth_utilization,
                    stride_pattern: pattern.stride_pattern.clone(),
                });
            }
        }
        
        // Sort by total bytes accessed (descending)
        hotspots.sort_by(|a, b| b.total_bytes_accessed.cmp(&a.total_bytes_accessed));
        hotspots
    }
    
    /// Analyze memory locality for an expression
    #[must_use] pub fn analyze_locality(&self, expr_hash: &str) -> Option<LocalityAnalysis> {
        let pattern = self.access_patterns.get(expr_hash)?;
        
        // Temporal locality: how often are the same locations accessed repeatedly
        let temporal_locality = self.calculate_temporal_locality(pattern);
        
        // Spatial locality: how often are nearby locations accessed
        let spatial_locality = self.calculate_spatial_locality(pattern);
        
        Some(LocalityAnalysis {
            temporal_locality,
            spatial_locality,
            overall_locality: f64::midpoint(temporal_locality, spatial_locality),
        })
    }
    
    /// Calculate temporal locality score
    fn calculate_temporal_locality(&self, pattern: &MemoryAccessPattern) -> f64 {
        if pattern.read_locations.len() < 2 {
            return 1.0;
        }
        
        let mut address_last_seen = HashMap::new();
        let mut temporal_distances = Vec::new();
        
        for (index, location) in pattern.read_locations.iter().enumerate() {
            if let Some(last_index) = address_last_seen.get(&location.address) {
                temporal_distances.push(index - last_index);
            }
            address_last_seen.insert(location.address, index);
        }
        
        if temporal_distances.is_empty() {
            1.0
        } else {
            let avg_distance = temporal_distances.iter().sum::<usize>() as f64 / temporal_distances.len() as f64;
            // Lower distance means better temporal locality
            1.0 / (1.0 + avg_distance / 10.0)
        }
    }
    
    /// Calculate spatial locality score
    fn calculate_spatial_locality(&self, pattern: &MemoryAccessPattern) -> f64 {
        if pattern.read_locations.len() < 2 {
            return 1.0;
        }
        
        let mut spatial_distances = Vec::new();
        
        for window in pattern.read_locations.windows(2) {
            let distance = (window[1].address as isize - window[0].address as isize).abs();
            spatial_distances.push(distance as usize);
        }
        
        if spatial_distances.is_empty() {
            1.0
        } else {
            let avg_distance = spatial_distances.iter().sum::<usize>() as f64 / spatial_distances.len() as f64;
            // Lower distance means better spatial locality
            1.0 / (1.0 + avg_distance / 64.0) // 64 bytes is typical cache line size
        }
    }
}

/// Memory access statistics
#[derive(Debug, Clone)]
pub struct MemoryStatistics {
    /// Total number of expressions being tracked
    pub total_expressions_tracked: usize,
    
    /// Total number of read operations
    pub total_reads: usize,
    
    /// Total number of write operations
    pub total_writes: usize,
    
    /// Total bytes accessed
    pub total_bytes_accessed: usize,
    
    /// Average cache miss rate across all expressions
    pub average_cache_miss_rate: f64,
    
    /// Overall memory efficiency score
    pub efficiency_score: f64,
}

/// Memory hotspot information
#[derive(Debug, Clone)]
pub struct MemoryHotspot {
    /// Expression identifier
    pub expression: String,
    
    /// Total number of memory accesses
    pub total_accesses: usize,
    
    /// Total bytes accessed
    pub total_bytes_accessed: usize,
    
    /// Cache miss rate
    pub cache_miss_rate: f64,
    
    /// Memory bandwidth utilization
    pub bandwidth_utilization: f64,
    
    /// Memory access stride pattern
    pub stride_pattern: StridePattern,
}

/// Memory locality analysis
#[derive(Debug, Clone)]
pub struct LocalityAnalysis {
    /// Temporal locality score (0.0-1.0, higher is better)
    pub temporal_locality: f64,
    
    /// Spatial locality score (0.0-1.0, higher is better)
    pub spatial_locality: f64,
    
    /// Overall locality score
    pub overall_locality: f64,
}

impl Default for MemoryAccessAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}