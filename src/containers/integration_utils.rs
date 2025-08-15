//! Integration utilities for container context optimization.
//!
//! This module provides utilities for integrating optimized containers
//! with the existing Lambdust value system and runtime infrastructure.

use crate::containers::context_optimization::{
    ArenaVector, ArenaHashTable, ContainerContext, OptimizationPriority, AccessPattern
};
use crate::eval::value::Value;
use crate::eval::arena_integration::{ValueLifetime, ArenaAllocator};
use crate::diagnostics::{Error, Result, Span};
use std::sync::Arc;
use std::collections::HashMap;

/// Container factory for creating optimized containers based on usage patterns
pub struct OptimizedContainerFactory {
    allocator: Arc<ArenaAllocator>,
    default_contexts: HashMap<ContainerType, ContainerContext>,
}

/// Container type enumeration for factory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerType {
    /// Dynamic array/vector
    Vector,
    /// Hash table/map
    HashTable,
    /// List/sequence
    List,
    /// Set/collection
    Set,
    /// Queue/deque
    Queue,
}

/// Usage pattern analyzer for automatic optimization
pub struct UsagePatternAnalyzer {
    /// Operation counters
    read_operations: u64,
    write_operations: u64,
    sequential_accesses: u64,
    random_accesses: u64,
    /// Size tracking
    max_size: usize,
    avg_size: f64,
    size_samples: u32,
}

/// Container migration utilities
pub struct ContainerMigrator {
    allocator: Arc<ArenaAllocator>,
}

impl OptimizedContainerFactory {
    /// Create a new container factory
    pub fn new() -> Self {
        let allocator = Arc::new(ArenaAllocator::new());
        let mut default_contexts = HashMap::new();
        
        // Setup default contexts for different container types
        default_contexts.insert(ContainerType::Vector, ContainerContext {
            lifetime: ValueLifetime::Call,
            expected_size: Some(32),
            access_pattern: AccessPattern::Sequential,
            sharing_expected: false,
            optimization_priority: OptimizationPriority::Balanced,
            name: Some("vector".to_string()),
        });
        
        default_contexts.insert(ContainerType::HashTable, ContainerContext {
            lifetime: ValueLifetime::Call,
            expected_size: Some(16),
            access_pattern: AccessPattern::Random,
            sharing_expected: true,
            optimization_priority: OptimizationPriority::Balanced,
            name: Some("hash-table".to_string()),
        });
        
        Self {
            allocator,
            default_contexts,
        }
    }
    
    /// Create factory with custom allocator
    pub fn with_allocator(allocator: Arc<ArenaAllocator>) -> Self {
        let mut factory = Self::new();
        factory.allocator = allocator;
        factory
    }
    
    /// Create optimized vector with usage hints
    pub fn create_vector(&self, usage_hint: Option<VectorUsageHint>) -> ArenaVector {
        let context = if let Some(hint) = usage_hint {
            self.create_vector_context(hint)
        } else {
            self.default_contexts[&ContainerType::Vector].clone()
        };
        
        ArenaVector::with_context(context)
    }
    
    /// Create optimized hash table with usage hints
    pub fn create_hash_table(&self, usage_hint: Option<HashTableUsageHint>) -> ArenaHashTable {
        let context = if let Some(hint) = usage_hint {
            self.create_hash_table_context(hint)
        } else {
            self.default_contexts[&ContainerType::HashTable].clone()
        };
        
        ArenaHashTable::with_context(context)
    }
    
    /// Create optimized vector from existing Value vector
    pub fn vector_from_values(&self, values: Vec<Value>, usage_hint: Option<VectorUsageHint>) -> Result<ArenaVector> {
        let mut optimized_vector = self.create_vector(usage_hint);
        
        for value in values {
            optimized_vector.push(value)?;
        }
        
        Ok(optimized_vector)
    }
    
    /// Create optimized hash table from existing key-value pairs
    pub fn hash_table_from_pairs(&self, pairs: Vec<(Value, Value)>, usage_hint: Option<HashTableUsageHint>) -> Result<ArenaHashTable> {
        let mut optimized_table = self.create_hash_table(usage_hint);
        
        for (key, value) in pairs {
            optimized_table.insert(key, value)?;
        }
        
        Ok(optimized_table)
    }
    
    /// Update default context for a container type
    pub fn set_default_context(&mut self, container_type: ContainerType, context: ContainerContext) {
        self.default_contexts.insert(container_type, context);
    }
    
    /// Get current default context for a container type
    pub fn get_default_context(&self, container_type: ContainerType) -> Option<&ContainerContext> {
        self.default_contexts.get(&container_type)
    }
    
    /// Create vector context from usage hint
    fn create_vector_context(&self, hint: VectorUsageHint) -> ContainerContext {
        ContainerContext {
            lifetime: hint.lifetime,
            expected_size: hint.expected_size,
            access_pattern: hint.access_pattern,
            sharing_expected: hint.sharing_expected,
            optimization_priority: hint.optimization_priority,
            name: hint.name,
        }
    }
    
    /// Create hash table context from usage hint
    fn create_hash_table_context(&self, hint: HashTableUsageHint) -> ContainerContext {
        ContainerContext {
            lifetime: hint.lifetime,
            expected_size: hint.expected_size,
            access_pattern: hint.access_pattern,
            sharing_expected: hint.sharing_expected,
            optimization_priority: hint.optimization_priority,
            name: hint.name,
        }
    }
}

/// Usage hint for vector creation
#[derive(Debug, Clone)]
pub struct VectorUsageHint {
    pub lifetime: ValueLifetime,
    pub expected_size: Option<usize>,
    pub access_pattern: AccessPattern,
    pub sharing_expected: bool,
    pub optimization_priority: OptimizationPriority,
    pub name: Option<String>,
}

/// Usage hint for hash table creation
#[derive(Debug, Clone)]
pub struct HashTableUsageHint {
    pub lifetime: ValueLifetime,
    pub expected_size: Option<usize>,
    pub access_pattern: AccessPattern,
    pub sharing_expected: bool,
    pub optimization_priority: OptimizationPriority,
    pub name: Option<String>,
}

impl UsagePatternAnalyzer {
    /// Create a new usage pattern analyzer
    pub fn new() -> Self {
        Self {
            read_operations: 0,
            write_operations: 0,
            sequential_accesses: 0,
            random_accesses: 0,
            max_size: 0,
            avg_size: 0.0,
            size_samples: 0,
        }
    }
    
    /// Record a read operation
    pub fn record_read(&mut self, is_sequential: bool) {
        self.read_operations += 1;
        if is_sequential {
            self.sequential_accesses += 1;
        } else {
            self.random_accesses += 1;
        }
    }
    
    /// Record a write operation
    pub fn record_write(&mut self, is_sequential: bool) {
        self.write_operations += 1;
        if is_sequential {
            self.sequential_accesses += 1;
        } else {
            self.random_accesses += 1;
        }
    }
    
    /// Record container size
    pub fn record_size(&mut self, size: usize) {
        self.max_size = self.max_size.max(size);
        self.avg_size = (self.avg_size * self.size_samples as f64 + size as f64) / (self.size_samples + 1) as f64;
        self.size_samples += 1;
    }
    
    /// Analyze patterns and generate optimization recommendation
    pub fn analyze(&self) -> UsageAnalysis {
        let total_operations = self.read_operations + self.write_operations;
        let total_accesses = self.sequential_accesses + self.random_accesses;
        
        let read_ratio = if total_operations > 0 {
            self.read_operations as f64 / total_operations as f64
        } else {
            0.5
        };
        
        let sequential_ratio = if total_accesses > 0 {
            self.sequential_accesses as f64 / total_accesses as f64
        } else {
            0.5
        };
        
        let access_pattern = if sequential_ratio > 0.8 {
            AccessPattern::Sequential
        } else if sequential_ratio < 0.3 {
            AccessPattern::Random
        } else if read_ratio > 0.8 {
            AccessPattern::ReadHeavy
        } else if read_ratio < 0.3 {
            AccessPattern::WriteHeavy
        } else {
            AccessPattern::Mixed
        };
        
        let optimization_priority = if total_operations > 1000 {
            OptimizationPriority::Aggressive
        } else if total_operations > 100 {
            OptimizationPriority::Balanced
        } else {
            OptimizationPriority::Minimal
        };
        
        let recommended_context = ContainerContext {
            lifetime: ValueLifetime::Call, // Default
            expected_size: if self.avg_size > 0.0 {
                Some((self.avg_size * 1.5) as usize) // 50% buffer
            } else {
                None
            },
            access_pattern,
            sharing_expected: read_ratio > 0.6, // High read ratio suggests sharing
            optimization_priority,
            name: None,
        };
        
        UsageAnalysis {
            total_operations,
            read_ratio,
            sequential_ratio,
            max_size: self.max_size,
            avg_size: self.avg_size,
            recommended_context,
        }
    }
    
    /// Reset analyzer state
    pub fn reset(&mut self) {
        self.read_operations = 0;
        self.write_operations = 0;
        self.sequential_accesses = 0;
        self.random_accesses = 0;
        self.max_size = 0;
        self.avg_size = 0.0;
        self.size_samples = 0;
    }
}

/// Usage analysis results
#[derive(Debug, Clone)]
pub struct UsageAnalysis {
    pub total_operations: u64,
    pub read_ratio: f64,
    pub sequential_ratio: f64,
    pub max_size: usize,
    pub avg_size: f64,
    pub recommended_context: ContainerContext,
}

impl ContainerMigrator {
    /// Create a new container migrator
    pub fn new(allocator: Arc<ArenaAllocator>) -> Self {
        Self { allocator }
    }
    
    /// Migrate standard Vec<Value> to ArenaVector
    pub fn migrate_vector(&self, vec: Vec<Value>, context: ContainerContext) -> Result<ArenaVector> {
        let mut arena_vec = ArenaVector::with_context(context);
        
        for value in vec {
            arena_vec.push(value)?;
        }
        
        Ok(arena_vec)
    }
    
    /// Migrate standard HashMap to ArenaHashTable
    pub fn migrate_hash_map(&self, map: HashMap<Value, Value>, context: ContainerContext) -> Result<ArenaHashTable> {
        let mut arena_table = ArenaHashTable::with_context(context);
        
        for (key, value) in map {
            arena_table.insert(key, value)?;
        }
        
        Ok(arena_table)
    }
    
    /// Migrate ArenaVector back to standard Vec<Value>
    pub fn migrate_vector_back(&self, arena_vec: ArenaVector) -> Result<Vec<Value>> {
        arena_vec.to_standard_vector()
    }
    
    /// Batch migrate multiple containers
    pub fn batch_migrate_vectors(&self, vecs: Vec<Vec<Value>>, context: ContainerContext) -> Result<Vec<ArenaVector>> {
        let mut results = Vec::with_capacity(vecs.len());
        
        for vec in vecs {
            results.push(self.migrate_vector(vec, context.clone())?);
        }
        
        Ok(results)
    }
}

/// Container optimization advisor
pub struct OptimizationAdvisor {
    analyzers: HashMap<String, UsagePatternAnalyzer>,
}

impl OptimizationAdvisor {
    /// Create a new optimization advisor
    pub fn new() -> Self {
        Self {
            analyzers: HashMap::new(),
        }
    }
    
    /// Get or create analyzer for a container
    pub fn get_analyzer(&mut self, container_name: &str) -> &mut UsagePatternAnalyzer {
        self.analyzers.entry(container_name.to_string())
            .or_insert_with(UsagePatternAnalyzer::new)
    }
    
    /// Generate recommendations for all tracked containers
    pub fn generate_recommendations(&self) -> HashMap<String, UsageAnalysis> {
        self.analyzers.iter()
            .map(|(name, analyzer)| (name.clone(), analyzer.analyze()))
            .collect()
    }
    
    /// Generate optimization report
    pub fn optimization_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Container Optimization Report\n");
        report.push_str("============================\n\n");
        
        let recommendations = self.generate_recommendations();
        
        for (name, analysis) in recommendations {
            report.push_str(&format!("Container: {}\n", name));
            report.push_str(&format!("  Total operations: {}\n", analysis.total_operations));
            report.push_str(&format!("  Read ratio: {:.1}%\n", analysis.read_ratio * 100.0));
            report.push_str(&format!("  Sequential ratio: {:.1}%\n", analysis.sequential_ratio * 100.0));
            report.push_str(&format!("  Max size: {}\n", analysis.max_size));
            report.push_str(&format!("  Avg size: {:.1}\n", analysis.avg_size));
            report.push_str(&format!("  Recommended pattern: {:?}\n", analysis.recommended_context.access_pattern));
            report.push_str(&format!("  Recommended priority: {:?}\n", analysis.recommended_context.optimization_priority));
            report.push_str("\n");
        }
        
        report
    }
    
    /// Clear all analyzers
    pub fn clear(&mut self) {
        self.analyzers.clear();
    }
}

impl Default for OptimizedContainerFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for UsagePatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OptimizationAdvisor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for VectorUsageHint {
    fn default() -> Self {
        Self {
            lifetime: ValueLifetime::Call,
            expected_size: None,
            access_pattern: AccessPattern::Mixed,
            sharing_expected: false,
            optimization_priority: OptimizationPriority::Balanced,
            name: None,
        }
    }
}

impl Default for HashTableUsageHint {
    fn default() -> Self {
        Self {
            lifetime: ValueLifetime::Call,
            expected_size: None,
            access_pattern: AccessPattern::Random,
            sharing_expected: true,
            optimization_priority: OptimizationPriority::Balanced,
            name: None,
        }
    }
}

/// Convenience functions for creating optimized containers
pub mod convenience {
    use super::*;
    
    /// Create a high-performance vector for temporary use
    pub fn temp_vector() -> ArenaVector {
        let factory = OptimizedContainerFactory::new();
        factory.create_vector(Some(VectorUsageHint {
            lifetime: ValueLifetime::Temporary,
            optimization_priority: OptimizationPriority::Aggressive,
            ..Default::default()
        }))
    }
    
    /// Create a high-performance hash table for temporary use
    pub fn temp_hash_table() -> ArenaHashTable {
        let factory = OptimizedContainerFactory::new();
        factory.create_hash_table(Some(HashTableUsageHint {
            lifetime: ValueLifetime::Temporary,
            optimization_priority: OptimizationPriority::Aggressive,
            ..Default::default()
        }))
    }
    
    /// Create a vector optimized for sequential access
    pub fn sequential_vector(expected_size: Option<usize>) -> ArenaVector {
        let factory = OptimizedContainerFactory::new();
        factory.create_vector(Some(VectorUsageHint {
            access_pattern: AccessPattern::Sequential,
            expected_size,
            optimization_priority: OptimizationPriority::Balanced,
            ..Default::default()
        }))
    }
    
    /// Create a hash table optimized for read-heavy workloads
    pub fn read_heavy_hash_table(expected_size: Option<usize>) -> ArenaHashTable {
        let factory = OptimizedContainerFactory::new();
        factory.create_hash_table(Some(HashTableUsageHint {
            access_pattern: AccessPattern::ReadHeavy,
            expected_size,
            sharing_expected: true,
            optimization_priority: OptimizationPriority::Balanced,
            ..Default::default()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_container_factory() {
        let factory = OptimizedContainerFactory::new();
        
        let vec = factory.create_vector(None);
        assert_eq!(vec.len(), 0);
        
        let table = factory.create_hash_table(None);
        assert_eq!(table.len(), 0);
    }
    
    #[test]
    fn test_usage_pattern_analyzer() {
        let mut analyzer = UsagePatternAnalyzer::new();
        
        // Simulate operations
        analyzer.record_read(true);
        analyzer.record_read(true);
        analyzer.record_write(false);
        analyzer.record_size(100);
        
        let analysis = analyzer.analyze();
        assert_eq!(analysis.total_operations, 3);
        assert!(analysis.read_ratio > 0.6);
        assert_eq!(analysis.max_size, 100);
    }
    
    #[test]
    fn test_container_migrator() {
        let allocator = Arc::new(ArenaAllocator::new());
        let migrator = ContainerMigrator::new(allocator);
        
        let vec = vec![Value::integer(1), Value::integer(2), Value::integer(3)];
        let context = ContainerContext::default();
        
        let arena_vec = migrator.migrate_vector(vec, context).unwrap();
        assert_eq!(arena_vec.len(), 3);
        
        let migrated_back = migrator.migrate_vector_back(arena_vec).unwrap();
        assert_eq!(migrated_back.len(), 3);
    }
    
    #[test]
    fn test_optimization_advisor() {
        let mut advisor = OptimizationAdvisor::new();
        
        let analyzer = advisor.get_analyzer("test-container");
        analyzer.record_read(true);
        analyzer.record_size(50);
        
        let recommendations = advisor.generate_recommendations();
        assert!(recommendations.contains_key("test-container"));
        
        let report = advisor.optimization_report();
        assert!(report.contains("Container: test-container"));
    }
    
    #[test]
    fn test_convenience_functions() {
        let vec = convenience::temp_vector();
        assert_eq!(vec.len(), 0);
        
        let table = convenience::temp_hash_table();
        assert_eq!(table.len(), 0);
        
        let seq_vec = convenience::sequential_vector(Some(100));
        assert_eq!(seq_vec.len(), 0);
        
        let read_table = convenience::read_heavy_hash_table(Some(50));
        assert_eq!(read_table.len(), 0);
    }
}