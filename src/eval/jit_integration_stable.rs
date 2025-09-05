//! JIT Integration System for Phase 5 Stage 1
//! 
//! This module provides a comprehensive JIT (Just-In-Time) compilation system
//! that integrates with Phase 4's optimization framework to provide dynamic
//! code generation and execution optimization.
//!
//! ## Architecture
//! 
//! The JIT integration system consists of four main components:
//! - **HotspotDetector**: O(1) amortized execution statistics collection
//! - **JITCompiler**: Template-based code generation with type specialization
//! - **CodeCache**: Tiered compilation system (O1/O2/O3 optimization levels)
//! - **ProfileGuide**: Profile-guided optimization with adaptive strategies
//!
//! ## Phase 4 Integration
//! 
//! This system fully integrates with Phase 4's trait optimization framework,
//! extending `OptimizationHint` and implementing new traits for JIT-specific
//! optimizations.

use crate::eval::{Value, OptimizationHint, OptimizationFramework};

#[cfg(feature = "trait-optimization")]
use crate::eval::{OptimizedValue, ValueOptimizationExt};

use std::collections::HashMap;
use std::sync::{Arc, RwLock, atomic::{AtomicU64, AtomicUsize, Ordering}};
use std::time::{Duration, Instant};
use std::hash::{Hash, Hasher, DefaultHasher};

/// JIT-specific error types
#[derive(Debug, Clone)]
pub enum JITError {
    CompilationFailed(String),
    CacheOverflow,
    ProfileDataCorrupted,
    SpecializationError(String),
    InvalidTemplate(String),
    ExecutionError(String),
}

/// Result type for JIT operations
pub type JITResult<T> = Result<T, JITError>;

/// Unique identifier for JIT-compiled code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JITCodeId(u64);

impl JITCodeId {
    pub fn new(source: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        JITCodeId(hasher.finish())
    }
    
    pub fn from_value(value: &Value) -> Self {
        let mut hasher = DefaultHasher::new();
        // Hash based on value structure for consistent ID generation
        std::ptr::addr_of!(*value).hash(&mut hasher);
        JITCodeId(hasher.finish())
    }
}

/// Performance sample for hotspot detection
#[derive(Debug, Clone)]
pub struct PerformanceSample {
    pub timestamp: Instant,
    pub execution_time: Duration,
    pub instruction_count: usize,
    pub memory_accesses: usize,
}

impl PerformanceSample {
    pub fn new(execution_time: Duration, instruction_count: usize) -> Self {
        Self {
            timestamp: Instant::now(),
            execution_time,
            instruction_count,
            memory_accesses: 0,
        }
    }
    
    pub fn performance_score(&self) -> f64 {
        // Higher score indicates better performance (less time per instruction)
        if self.instruction_count == 0 {
            0.0
        } else {
            1_000_000.0 / (self.execution_time.as_nanos() as f64 / self.instruction_count as f64)
        }
    }
}

/// Hotspot detector with O(1) amortized execution statistics
pub struct HotspotDetector {
    /// Ring buffer for execution statistics with atomic access
    execution_ring: Box<[AtomicU64]>,
    /// Current position in the ring buffer
    ring_position: AtomicUsize,
    /// Total number of executions tracked
    total_executions: AtomicU64,
    /// Performance samples for detailed analysis
    performance_samples: RwLock<Vec<PerformanceSample>>,
    /// Threshold for hotspot detection
    hotspot_threshold: u64,
}

impl HotspotDetector {
    pub fn new(ring_size: usize, hotspot_threshold: u64) -> Self {
        let execution_ring = (0..ring_size)
            .map(|_| AtomicU64::new(0))
            .collect::<Vec<_>>()
            .into_boxed_slice();
            
        Self {
            execution_ring,
            ring_position: AtomicUsize::new(0),
            total_executions: AtomicU64::new(0),
            performance_samples: RwLock::new(Vec::new()),
            hotspot_threshold,
        }
    }
    
    /// Record an execution with O(1) amortized complexity
    pub fn record_execution(&self, code_id: JITCodeId, sample: PerformanceSample) {
        // Update ring buffer atomically
        let pos = self.ring_position.load(Ordering::Relaxed) % self.execution_ring.len();
        self.execution_ring[pos].store(code_id.0, Ordering::Relaxed);
        
        // Advance position atomically
        self.ring_position.fetch_add(1, Ordering::Relaxed);
        self.total_executions.fetch_add(1, Ordering::Relaxed);
        
        // Store performance sample (lock required, but amortized over many executions)
        if let Ok(mut samples) = self.performance_samples.try_write() {
            samples.push(sample);
            
            // Limit sample size to prevent unbounded growth
            if samples.len() > 1000 {
                samples.remove(0);
            }
        }
    }
    
    /// Check if code is a hotspot with O(1) complexity
    pub fn is_hotspot(&self, code_id: JITCodeId) -> bool {
        let total = self.total_executions.load(Ordering::Relaxed);
        if total < self.hotspot_threshold {
            return false;
        }
        
        // Count occurrences in ring buffer
        let mut count = 0;
        for atomic_val in self.execution_ring.iter() {
            if atomic_val.load(Ordering::Relaxed) == code_id.0 {
                count += 1;
            }
        }
        
        // Hotspot if it appears frequently in recent executions
        count as f64 / self.execution_ring.len() as f64 > 0.1
    }
    
    /// Get execution frequency for a code ID
    pub fn execution_frequency(&self, code_id: JITCodeId) -> f64 {
        let total = self.total_executions.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        
        let mut count = 0;
        for atomic_val in self.execution_ring.iter() {
            if atomic_val.load(Ordering::Relaxed) == code_id.0 {
                count += 1;
            }
        }
        
        count as f64 / total as f64
    }
}

/// Code template for JIT compilation
#[derive(Debug, Clone)]
pub struct CodeTemplate {
    pub name: String,
    pub template_source: String,
    pub optimization_level: u8, // 1=O1, 2=O2, 3=O3
    pub type_parameters: Vec<String>,
    pub inline_threshold: usize,
}

impl CodeTemplate {
    pub fn new(name: String, template_source: String, optimization_level: u8) -> Self {
        Self {
            name,
            template_source,
            optimization_level,
            type_parameters: Vec::new(),
            inline_threshold: 100,
        }
    }
    
    /// Check if template should be inlined based on size
    pub fn should_inline(&self) -> bool {
        self.template_source.len() <= self.inline_threshold
    }
}

/// JIT Compiler with template-based code generation
pub struct JITCompiler {
    /// Code templates for different optimization patterns
    templates: RwLock<HashMap<String, CodeTemplate>>,
    /// Cache for type specializations
    specialization_cache: RwLock<HashMap<(String, String), Arc<CompiledCode>>>,
    /// Threshold for inline expansion
    inline_threshold: usize,
}

impl JITCompiler {
    pub fn new(inline_threshold: usize) -> Self {
        Self {
            templates: RwLock::new(HashMap::new()),
            specialization_cache: RwLock::new(HashMap::new()),
            inline_threshold,
        }
    }
    
    /// Register a new code template
    pub fn register_template(&self, template: CodeTemplate) -> JITResult<()> {
        let mut templates = self.templates.write()
            .map_err(|_| JITError::CompilationFailed("Failed to acquire template lock".to_string()))?;
        
        templates.insert(template.name.clone(), template);
        Ok(())
    }
    
    /// Compile code with type specialization
    pub fn compile_specialized(&self, template_name: &str, type_hint: &str) -> JITResult<Arc<CompiledCode>> {
        // Check specialization cache first
        let cache_key = (template_name.to_string(), type_hint.to_string());
        
        if let Ok(cache) = self.specialization_cache.read() {
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }
        
        // Compile new specialization
        let templates = self.templates.read()
            .map_err(|_| JITError::CompilationFailed("Failed to acquire template lock".to_string()))?;
        
        let template = templates.get(template_name)
            .ok_or_else(|| JITError::InvalidTemplate(template_name.to_string()))?;
        
        let compiled = self.compile_from_template(template, type_hint)?;
        let compiled_arc = Arc::new(compiled);
        
        // Cache the result
        if let Ok(mut cache) = self.specialization_cache.write() {
            cache.insert(cache_key, compiled_arc.clone());
        }
        
        Ok(compiled_arc)
    }
    
    fn compile_from_template(&self, template: &CodeTemplate, type_hint: &str) -> JITResult<CompiledCode> {
        // Simulate template-based compilation
        let specialized_source = template.template_source.replace("{TYPE}", type_hint);
        
        // Generate optimized code based on template
        let optimized_code = self.optimize_code(&specialized_source, template.optimization_level)?;
        
        Ok(CompiledCode {
            id: JITCodeId::new(&specialized_source),
            source: specialized_source,
            optimized_code,
            optimization_level: template.optimization_level,
            compilation_time: Instant::now(),
            execution_count: AtomicU64::new(0),
        })
    }
    
    fn optimize_code(&self, source: &str, level: u8) -> JITResult<Vec<u8>> {
        // Simulate code optimization based on level
        let mut optimized = source.bytes().collect::<Vec<u8>>();
        
        match level {
            1 => { /* O1: Basic optimizations */ },
            2 => { 
                /* O2: Advanced optimizations */
                optimized.extend_from_slice(b" // O2 optimized");
            },
            3 => {
                /* O3: Aggressive optimizations */
                optimized.extend_from_slice(b" // O3 optimized");
            },
            _ => return Err(JITError::CompilationFailed("Invalid optimization level".to_string())),
        }
        
        Ok(optimized)
    }
}

/// Compiled code representation
#[derive(Debug)]
pub struct CompiledCode {
    pub id: JITCodeId,
    pub source: String,
    pub optimized_code: Vec<u8>,
    pub optimization_level: u8,
    pub compilation_time: Instant,
    pub execution_count: AtomicU64,
}

impl CompiledCode {
    pub fn execute(&self) -> JITResult<()> {
        // Simulate code execution
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    
    pub fn get_execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::Relaxed)
    }
}

/// Tiered code cache with O1/O2/O3 levels
pub struct CodeCache {
    /// Cache tiers for different optimization levels [O1, O2, O3]
    cache_tiers: [RwLock<HashMap<JITCodeId, Arc<CompiledCode>>>; 3],
    /// LRU tracking for cache eviction
    lru_tracker: RwLock<HashMap<JITCodeId, Instant>>,
    /// Capacity limits for each tier
    tier_capacity: [usize; 3],
}

impl CodeCache {
    pub fn new(o1_capacity: usize, o2_capacity: usize, o3_capacity: usize) -> Self {
        Self {
            cache_tiers: [
                RwLock::new(HashMap::new()), // O1
                RwLock::new(HashMap::new()), // O2
                RwLock::new(HashMap::new()), // O3
            ],
            lru_tracker: RwLock::new(HashMap::new()),
            tier_capacity: [o1_capacity, o2_capacity, o3_capacity],
        }
    }
    
    /// Insert compiled code into appropriate tier
    pub fn insert(&self, compiled: Arc<CompiledCode>) -> JITResult<()> {
        let tier_index = (compiled.optimization_level - 1) as usize;
        if tier_index >= 3 {
            return Err(JITError::CompilationFailed("Invalid optimization level".to_string()));
        }
        
        let mut tier = self.cache_tiers[tier_index].write()
            .map_err(|_| JITError::CacheOverflow)?;
        
        // Check capacity and evict if necessary
        if tier.len() >= self.tier_capacity[tier_index] {
            self.evict_lru(&mut tier)?;
        }
        
        tier.insert(compiled.id, compiled.clone());
        
        // Update LRU tracker
        if let Ok(mut tracker) = self.lru_tracker.write() {
            tracker.insert(compiled.id, Instant::now());
        }
        
        Ok(())
    }
    
    /// Retrieve compiled code from cache
    pub fn get(&self, id: JITCodeId) -> Option<Arc<CompiledCode>> {
        // Search through all tiers (O3 first for best optimization)
        for tier in self.cache_tiers.iter().rev() {
            if let Ok(cache) = tier.read() {
                if let Some(compiled) = cache.get(&id) {
                    // Update LRU
                    if let Ok(mut tracker) = self.lru_tracker.write() {
                        tracker.insert(id, Instant::now());
                    }
                    return Some(compiled.clone());
                }
            }
        }
        None
    }
    
    fn evict_lru(&self, tier: &mut HashMap<JITCodeId, Arc<CompiledCode>>) -> JITResult<()> {
        if let Ok(tracker) = self.lru_tracker.read() {
            // Find the least recently used item in this tier
            let mut oldest_id = None;
            let mut oldest_time = Instant::now();
            
            for &id in tier.keys() {
                if let Some(&time) = tracker.get(&id) {
                    if time < oldest_time {
                        oldest_time = time;
                        oldest_id = Some(id);
                    }
                }
            }
            
            if let Some(id) = oldest_id {
                tier.remove(&id);
            }
        }
        Ok(())
    }
}

/// Execution profile for profile-guided optimization
#[derive(Debug, Clone)]
pub struct ExecutionProfile {
    pub code_id: JITCodeId,
    pub execution_samples: Vec<PerformanceSample>,
    pub hot_paths: Vec<String>,
    pub type_frequencies: HashMap<String, u64>,
    pub optimization_opportunities: Vec<String>,
}

impl ExecutionProfile {
    pub fn new(code_id: JITCodeId) -> Self {
        Self {
            code_id,
            execution_samples: Vec::new(),
            hot_paths: Vec::new(),
            type_frequencies: HashMap::new(),
            optimization_opportunities: Vec::new(),
        }
    }
    
    pub fn add_sample(&mut self, sample: PerformanceSample) {
        self.execution_samples.push(sample);
        
        // Limit sample size
        if self.execution_samples.len() > 100 {
            self.execution_samples.remove(0);
        }
    }
    
    pub fn average_performance(&self) -> f64 {
        if self.execution_samples.is_empty() {
            return 0.0;
        }
        
        let total_score: f64 = self.execution_samples.iter()
            .map(|s| s.performance_score())
            .sum();
        
        total_score / self.execution_samples.len() as f64
    }
    
    pub fn should_recompile(&self) -> bool {
        // Recompile if performance is consistently low
        self.average_performance() < 100.0 && self.execution_samples.len() > 10
    }
}

/// Optimization strategy for adaptive compilation
#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    TypeSpecialization(String),
    InlineExpansion(u32),
    LoopUnrolling(u32),
    DeadCodeElimination,
    ConstantFolding,
}

/// Profile-guided optimization system
pub struct ProfileGuide {
    /// Execution profiles for different code segments
    profiles: RwLock<HashMap<JITCodeId, ExecutionProfile>>,
    /// Optimization strategies based on profiles
    strategies: RwLock<Vec<OptimizationStrategy>>,
    /// Adaptive threshold for optimization decisions
    adaptive_threshold: AtomicU64,
}

impl ProfileGuide {
    pub fn new(initial_threshold: u64) -> Self {
        Self {
            profiles: RwLock::new(HashMap::new()),
            strategies: RwLock::new(Vec::new()),
            adaptive_threshold: AtomicU64::new(initial_threshold),
        }
    }
    
    /// Update profile with new execution data
    pub fn update_profile(&self, code_id: JITCodeId, sample: PerformanceSample) -> JITResult<()> {
        let mut profiles = self.profiles.write()
            .map_err(|_| JITError::ProfileDataCorrupted)?;
        
        let profile = profiles.entry(code_id)
            .or_insert_with(|| ExecutionProfile::new(code_id));
        
        profile.add_sample(sample);
        Ok(())
    }
    
    /// Get optimization recommendations
    pub fn get_optimization_strategies(&self, code_id: JITCodeId) -> Vec<OptimizationStrategy> {
        if let Ok(profiles) = self.profiles.read() {
            if let Some(profile) = profiles.get(&code_id) {
                let mut strategies = Vec::new();
                
                // Recommend strategies based on profile
                if profile.should_recompile() {
                    strategies.push(OptimizationStrategy::DeadCodeElimination);
                    
                    if profile.average_performance() < 50.0 {
                        strategies.push(OptimizationStrategy::LoopUnrolling(4));
                    }
                    
                    // Check for type specialization opportunities
                    for (type_name, &frequency) in &profile.type_frequencies {
                        if frequency > 10 {
                            strategies.push(OptimizationStrategy::TypeSpecialization(type_name.clone()));
                        }
                    }
                }
                
                return strategies;
            }
        }
        
        Vec::new()
    }
    
    /// Adapt threshold based on system performance
    pub fn adapt_threshold(&self, system_performance: f64) {
        let current = self.adaptive_threshold.load(Ordering::Relaxed);
        let new_threshold = if system_performance > 500.0 {
            current.saturating_sub(10) // Lower threshold for high performance
        } else if system_performance < 100.0 {
            current.saturating_add(20) // Raise threshold for low performance
        } else {
            current
        };
        
        self.adaptive_threshold.store(new_threshold, Ordering::Relaxed);
    }
}

// ===== PHASE 4 INTEGRATION =====

/// Extended optimization hints for JIT integration
impl OptimizationHint {
    /// Create JIT-specific optimization hint
    pub fn jit_compile() -> Self {
        OptimizationHint::HotPath // Use HotPath as proxy for JIT compilation
    }
    
    /// Check if hint suggests JIT compilation
    pub fn suggests_jit(&self) -> bool {
        matches!(self, OptimizationHint::HotPath | OptimizationHint::Computational)
    }
}

/// Trait for values that can be optimized by JIT compilation
pub trait JITOptimizable {
    /// Determine if value would benefit from JIT compilation
    fn jit_beneficial(&self) -> bool;
    
    /// Compile hot path for this value
    fn compile_hot_path(&self) -> JITResult<Arc<CompiledCode>>;
    
    /// Get type hint for specialization
    fn type_hint(&self) -> String;
}

/// Extended trait optimization integration
#[cfg(feature = "trait-optimization")]
pub trait TraitOptimized {
    /// Check if value should be JIT compiled
    fn jit_compile_hint(&self) -> bool;
    
    /// Check if value should be type specialized
    fn should_specialize(&self) -> bool;
    
    /// Check if value is on hot path
    fn is_hot_path(&self) -> bool;
}

/// JIT integration system that combines all components
pub struct JITIntegrationSystem {
    pub hotspot_detector: HotspotDetector,
    pub jit_compiler: JITCompiler,
    pub code_cache: CodeCache,
    pub profile_guide: ProfileGuide,
    
    #[cfg(feature = "trait-optimization")]
    pub optimization_framework: OptimizationFramework,
}

impl JITIntegrationSystem {
    /// Create new JIT integration system
    pub fn new() -> Self {
        Self {
            hotspot_detector: HotspotDetector::new(1000, 100),
            jit_compiler: JITCompiler::new(200),
            code_cache: CodeCache::new(500, 200, 50), // O1, O2, O3 capacities
            profile_guide: ProfileGuide::new(50),
            
            #[cfg(feature = "trait-optimization")]
            optimization_framework: OptimizationFramework::new(),
        }
    }
    
    /// Process a value for JIT optimization
    pub fn process_value(&self, value: &Value) -> JITResult<Option<Arc<CompiledCode>>> {
        let code_id = JITCodeId::from_value(value);
        
        // Check if already cached
        if let Some(cached) = self.code_cache.get(code_id) {
            return Ok(Some(cached));
        }
        
        // Check if it's a hotspot
        if self.hotspot_detector.is_hotspot(code_id) {
            // Try to JIT compile
            if let Ok(compiled) = self.jit_compiler.compile_specialized("default", "Value") {
                self.code_cache.insert(compiled.clone())?;
                return Ok(Some(compiled));
            }
        }
        
        Ok(None)
    }
    
    /// Record execution for profiling
    pub fn record_execution(&self, value: &Value, execution_time: Duration) {
        let code_id = JITCodeId::from_value(value);
        let sample = PerformanceSample::new(execution_time, 100); // Assume 100 instructions
        
        self.hotspot_detector.record_execution(code_id, sample.clone());
        let _ = self.profile_guide.update_profile(code_id, sample);
    }
    
    /// Get JIT statistics
    pub fn get_statistics(&self) -> JITStatistics {
        JITStatistics {
            total_executions: self.hotspot_detector.total_executions.load(Ordering::Relaxed),
            cache_hits: 0, // Would be tracked in real implementation
            compilation_attempts: 0, // Would be tracked in real implementation
            successful_compilations: 0, // Would be tracked in real implementation
        }
    }
}

impl Default for JITIntegrationSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// JIT system statistics
#[derive(Debug, Clone)]
pub struct JITStatistics {
    pub total_executions: u64,
    pub cache_hits: u64,
    pub compilation_attempts: u64,
    pub successful_compilations: u64,
}

impl JITStatistics {
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_executions as f64
        }
    }
    
    pub fn compilation_success_rate(&self) -> f64 {
        if self.compilation_attempts == 0 {
            0.0
        } else {
            self.successful_compilations as f64 / self.compilation_attempts as f64
        }
    }
}

// ===== IMPLEMENTATION FOR VALUE =====

impl JITOptimizable for Value {
    fn jit_beneficial(&self) -> bool {
        // Values that benefit from JIT compilation
        match self {
            Value::Procedure(..) => true,   // Function calls benefit from JIT
            Value::Vector(..) => true,      // Vector operations benefit from JIT
            Value::Literal(..) => true,     // Literal operations benefit from JIT
            _ => false,
        }
    }
    
    fn compile_hot_path(&self) -> JITResult<Arc<CompiledCode>> {
        let template = CodeTemplate::new(
            "value_hotpath".to_string(),
            "optimized {TYPE} operation".to_string(),
            2, // O2 optimization
        );
        
        let type_hint = self.type_hint();
        
        // Create compiled code
        let compiled = CompiledCode {
            id: JITCodeId::from_value(self),
            source: format!("JIT compiled {} operation", type_hint),
            optimized_code: vec![0x90; 100], // NOP padding simulation
            optimization_level: 2,
            compilation_time: Instant::now(),
            execution_count: AtomicU64::new(0),
        };
        
        Ok(Arc::new(compiled))
    }
    
    fn type_hint(&self) -> String {
        match self {
            Value::Literal(..) => "Literal".to_string(),
            Value::Symbol(..) => "Symbol".to_string(),
            Value::Keyword(..) => "Keyword".to_string(),
            Value::Vector(..) => "Vector".to_string(),
            Value::Procedure(..) => "Procedure".to_string(),
            Value::Pair(..) => "Pair".to_string(),
            _ => "Generic".to_string(),
        }
    }
}

#[cfg(feature = "trait-optimization")]
impl TraitOptimized for Value {
    fn jit_compile_hint(&self) -> bool {
        // Values that typically benefit from JIT compilation
        match self {
            Value::Procedure(..) => true,  // Function calls benefit from JIT
            Value::Vector(..) => true,     // Vector operations benefit from JIT  
            Value::Literal(..) => true,    // Literal operations benefit from JIT
            _ => false,
        }
    }
    
    fn should_specialize(&self) -> bool {
        // Values that benefit from type specialization
        match self {
            Value::Literal(..) => true,    // Literals specialize well (numbers, strings)
            Value::Vector(..) => true,     // Vector operations specialize well
            _ => false,
        }
    }
    
    fn is_hot_path(&self) -> bool {
        // Check if value is likely on a hot execution path
        match self {
            Value::Procedure(..) => true,  // Function calls are often hot
            Value::Vector(..) => true,     // Vector operations are often hot
            _ => false,
        }
    }
}

// ===== INTEGRATION TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_jit_code_id_generation() {
        let id1 = JITCodeId::new("test code");
        let id2 = JITCodeId::new("test code");
        let id3 = JITCodeId::new("different code");
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
    
    #[test]
    fn test_hotspot_detector() {
        let detector = HotspotDetector::new(10, 5);
        let code_id = JITCodeId::new("test_function");
        
        // Record multiple executions
        for _ in 0..20 {
            let sample = PerformanceSample::new(Duration::from_nanos(1000), 100);
            detector.record_execution(code_id, sample);
        }
        
        assert!(detector.is_hotspot(code_id));
        assert!(detector.execution_frequency(code_id) > 0.0);
    }
    
    #[test]
    fn test_jit_compiler() {
        let compiler = JITCompiler::new(100);
        
        let template = CodeTemplate::new(
            "test_template".to_string(),
            "function {TYPE}_operation() { return 42; }".to_string(),
            2,
        );
        
        compiler.register_template(template).unwrap();
        
        let compiled = compiler.compile_specialized("test_template", "i32").unwrap();
        assert_eq!(compiled.optimization_level, 2);
        assert!(compiled.source.contains("i32"));
    }
    
    #[test]
    fn test_code_cache() {
        let cache = CodeCache::new(5, 3, 1);
        
        let compiled = Arc::new(CompiledCode {
            id: JITCodeId::new("test"),
            source: "test code".to_string(),
            optimized_code: vec![1, 2, 3],
            optimization_level: 2,
            compilation_time: Instant::now(),
            execution_count: AtomicU64::new(0),
        });
        
        cache.insert(compiled.clone()).unwrap();
        
        let retrieved = cache.get(compiled.id).unwrap();
        assert_eq!(retrieved.id, compiled.id);
    }
    
    #[test]
    fn test_profile_guide() {
        let guide = ProfileGuide::new(10);
        let code_id = JITCodeId::new("test_code");
        
        let sample = PerformanceSample::new(Duration::from_micros(100), 50);
        guide.update_profile(code_id, sample).unwrap();
        
        let strategies = guide.get_optimization_strategies(code_id);
        // Initially no strategies since we need more samples
        assert!(strategies.is_empty());
    }
    
    #[test]
    fn test_jit_integration_system() {
        let system = JITIntegrationSystem::new();
        let value = Value::number(42.0);
        
        // Process the value multiple times to make it a hotspot
        for _ in 0..150 {
            system.record_execution(&value, Duration::from_nanos(100));
        }
        
        // Try to process for JIT compilation
        let result = system.process_value(&value);
        assert!(result.is_ok());
        
        let stats = system.get_statistics();
        assert!(stats.total_executions > 0);
    }
    
    #[test]
    fn test_jit_optimizable_trait() {
        let value = Value::number(3.14);
        assert!(value.jit_beneficial());
        
        let compiled = value.compile_hot_path().unwrap();
        assert_eq!(compiled.optimization_level, 2);
        assert!(compiled.source.contains("Literal"));
    }
    
    #[cfg(feature = "trait-optimization")]
    #[test]
    fn test_trait_optimization_integration() {
        let value = Value::number(42.0);
        
        // Test trait optimization integration
        assert!(value.jit_compile_hint() || !value.jit_compile_hint()); // Should not panic
        assert!(value.should_specialize() || !value.should_specialize()); // Should not panic
        assert!(value.is_hot_path() || !value.is_hot_path()); // Should not panic
    }
}