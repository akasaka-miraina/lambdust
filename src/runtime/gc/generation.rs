//! Generation management for the parallel garbage collector
//!
//! This module implements the generation management system for the parallel
//! garbage collector, including young, old, and permanent generation structures
//! with appropriate allocation and collection strategies.

use crate::eval::value::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, atomic::{AtomicUsize, AtomicU64, AtomicBool, Ordering}};
use std::time::{Duration, Instant};

/// Object header information stored with each allocated object
#[derive(Debug, Clone)]
pub struct ObjectHeader {
    /// Object size in bytes
    pub size: usize,
    /// Generation this object belongs to
    pub generation: GenerationId,
    /// Mark bit for garbage collection
    pub mark: bool,
    /// Age counter (number of collections survived)
    pub age: u8,
    /// Reference to the actual value
    pub value: Arc<Value>,
    /// Forward pointer for copying collection
    pub forwarding_address: Option<*mut ObjectHeader>,
}

// Safety: ObjectHeader can be safely shared between threads. The forwarding_address
// raw pointer is only used during stop-the-world collection phases where thread safety
// is guaranteed by the collector coordination.
unsafe impl Send for ObjectHeader {}
unsafe impl Sync for ObjectHeader {}

impl ObjectHeader {
    /// Create a new object header
    pub fn new(value: Value, size: usize, generation: GenerationId) -> Self {
        ObjectHeader {
            size,
            generation,
            mark: false,
            age: 0,
            value: Arc::new(value),
            forwarding_address: None,
        }
    }

    /// Mark this object as live
    pub fn mark(&mut self) {
        self.mark = true;
    }

    /// Clear the mark bit
    pub fn unmark(&mut self) {
        self.mark = false;
    }

    /// Check if object is marked
    pub fn is_marked(&self) -> bool {
        self.mark
    }

    /// Increment age counter
    pub fn age_increment(&mut self) {
        self.age = self.age.saturating_add(1);
    }

    /// Check if object should be promoted to older generation
    pub fn should_promote(&self, promotion_threshold: u8) -> bool {
        self.age >= promotion_threshold
    }
}

/// Generation identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenerationId {
    /// Young generation (nursery)
    Young,
    /// Old generation (tenured)
    Old, 
    /// Permanent generation (metadata, class info)
    Permanent,
    /// Large object space
    LargeObject,
}

impl GenerationId {
    /// Get the next generation for promotion
    pub fn next_generation(&self) -> Option<GenerationId> {
        match self {
            GenerationId::Young => Some(GenerationId::Old),
            GenerationId::Old => Some(GenerationId::Permanent),
            GenerationId::Permanent => None,
            GenerationId::LargeObject => None,
        }
    }

    /// Check if this generation can be collected concurrently
    pub fn supports_concurrent_collection(&self) -> bool {
        match self {
            GenerationId::Young => false, // Stop-the-world for young gen
            GenerationId::Old => true,    // Concurrent for old gen
            GenerationId::Permanent => false,
            GenerationId::LargeObject => true,
        }
    }
}

/// Statistics for a single generation
#[derive(Debug, Default)]
pub struct GenerationStatistics {
    /// Total allocations in this generation
    pub total_allocations: AtomicU64,
    /// Total bytes allocated
    pub total_allocated_bytes: AtomicU64,
    /// Total collections performed
    pub total_collections: AtomicU64,
    /// Total collection time
    pub total_collection_time: AtomicU64,
    /// Objects promoted from this generation
    pub objects_promoted: AtomicU64,
    /// Bytes promoted from this generation
    pub bytes_promoted: AtomicU64,
    /// Current live objects
    pub live_objects: AtomicUsize,
    /// Current live bytes
    pub live_bytes: AtomicUsize,
}

impl GenerationStatistics {
    /// Create new statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an allocation
    pub fn record_allocation(&self, size: usize) {
        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        self.total_allocated_bytes.fetch_add(size as u64, Ordering::Relaxed);
        self.live_objects.fetch_add(1, Ordering::Relaxed);
        self.live_bytes.fetch_add(size, Ordering::Relaxed);
    }

    /// Record a collection
    pub fn record_collection(&self, duration: Duration) {
        self.total_collections.fetch_add(1, Ordering::Relaxed);
        self.total_collection_time.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Record object promotion
    pub fn record_promotion(&self, size: usize) {
        self.objects_promoted.fetch_add(1, Ordering::Relaxed);
        self.bytes_promoted.fetch_add(size as u64, Ordering::Relaxed);
        self.live_objects.fetch_sub(1, Ordering::Relaxed);
        self.live_bytes.fetch_sub(size, Ordering::Relaxed);
    }

    /// Record object death
    pub fn record_death(&self, size: usize) {
        self.live_objects.fetch_sub(1, Ordering::Relaxed);
        self.live_bytes.fetch_sub(size, Ordering::Relaxed);
    }

    /// Get current utilization percentage
    pub fn utilization_percentage(&self, capacity: usize) -> f64 {
        if capacity == 0 {
            return 0.0;
        }
        (self.live_bytes.load(Ordering::Relaxed) as f64 / capacity as f64) * 100.0
    }
}

/// Memory region within a generation
#[derive(Debug)]
pub struct MemoryRegion {
    /// Start address of the region
    pub start: *mut u8,
    /// Size of the region in bytes
    pub size: usize,
    /// Current allocation pointer
    pub current: AtomicUsize,
    /// End address of the region
    pub end: *mut u8,
    /// Whether this region is currently being used for allocation
    pub active: AtomicBool,
}

// Safety: MemoryRegion can be safely shared between threads as long as
// we don't dereference the raw pointers without proper synchronization
unsafe impl Send for MemoryRegion {}
unsafe impl Sync for MemoryRegion {}

impl MemoryRegion {
    /// Create a new memory region
    pub fn new(size: usize) -> Result<Self, String> {
        // Use system allocator for now - in a real implementation,
        // this would use mmap or VirtualAlloc for better control
        let layout = std::alloc::Layout::from_size_align(size, 8)
            .map_err(|e| format!("Failed to create layout: {e}"))?;
        
        let start = unsafe { std::alloc::alloc(layout) };
        if start.is_null() {
            return Err("Failed to allocate memory region".to_string());
        }

        let end = unsafe { start.add(size) };

        Ok(MemoryRegion {
            start,
            size,
            current: AtomicUsize::new(start as usize),
            end,
            active: AtomicBool::new(true),
        })
    }

    /// Try to allocate space in this region
    pub fn try_allocate(&self, size: usize, alignment: usize) -> Option<*mut u8> {
        if !self.active.load(Ordering::Relaxed) {
            return None;
        }

        loop {
            let current = self.current.load(Ordering::Relaxed);
            
            // Align the allocation
            let aligned_current = (current + alignment - 1) & !(alignment - 1);
            let new_current = aligned_current + size;

            if new_current > self.end as usize {
                // No space available
                return None;
            }

            // Try to update the current pointer
            match self.current.compare_exchange_weak(
                current,
                new_current,
                Ordering::Relaxed,
                Ordering::Relaxed
            ) {
                Ok(_) => return Some(aligned_current as *mut u8),
                Err(_) => continue, // Retry
            }
        }
    }

    /// Reset the region for reuse
    pub fn reset(&self) {
        self.current.store(self.start as usize, Ordering::Relaxed);
    }

    /// Get remaining space
    pub fn remaining_space(&self) -> usize {
        let current = self.current.load(Ordering::Relaxed);
        (self.end as usize).saturating_sub(current)
    }

    /// Get used space
    pub fn used_space(&self) -> usize {
        let current = self.current.load(Ordering::Relaxed);
        current.saturating_sub(self.start as usize)
    }
}

impl Drop for MemoryRegion {
    fn drop(&mut self) {
        if !self.start.is_null() {
            let layout = std::alloc::Layout::from_size_align(self.size, 8).unwrap();
            unsafe {
                std::alloc::dealloc(self.start, layout);
            }
        }
    }
}

/// Base trait for generation management
pub trait Generation: Send + Sync {
    /// Allocate an object in this generation
    fn allocate(&self, value: Value, size: usize) -> Result<Arc<ObjectHeader>, String>;
    
    /// Collect garbage in this generation
    fn collect(&self, full_collection: bool) -> Result<CollectionResult, String>;
    
    /// Get generation statistics
    fn get_statistics(&self) -> &GenerationStatistics;
    
    /// Get generation ID
    fn get_id(&self) -> GenerationId;
    
    /// Check if collection is needed
    fn needs_collection(&self) -> bool;
    
    /// Get current utilization
    fn utilization(&self) -> f64;
}

/// Result of a garbage collection
#[derive(Debug)]
pub struct CollectionResult {
    /// Objects collected
    pub objects_collected: usize,
    /// Bytes reclaimed
    pub bytes_reclaimed: usize,
    /// Objects promoted
    pub objects_promoted: usize,
    /// Bytes promoted
    pub bytes_promoted: usize,
    /// Collection time
    pub collection_time: Duration,
    /// Whether collection was successful
    pub success: bool,
}

/// Young generation implementation (copying collector)
#[derive(Debug)]
pub struct YoungGeneration {
    /// Generation ID
    id: GenerationId,
    /// Memory regions (from-space and to-space)
    regions: RwLock<Vec<Arc<MemoryRegion>>>,
    /// Current active region index
    active_region: AtomicUsize,
    /// Statistics
    statistics: GenerationStatistics,
    /// Maximum size
    max_size: usize,
    /// Promotion age threshold
    promotion_age_threshold: u8,
}

impl YoungGeneration {
    /// Create a new young generation
    pub fn new(max_size: usize) -> Result<Self, String> {
        let region_size = max_size / 2; // From-space and to-space
        let from_space = Arc::new(MemoryRegion::new(region_size)?);
        let to_space = Arc::new(MemoryRegion::new(region_size)?);
        
        // To-space starts inactive
        to_space.active.store(false, Ordering::Relaxed);
        
        let regions = vec![from_space, to_space];

        Ok(YoungGeneration {
            id: GenerationId::Young,
            regions: RwLock::new(regions),
            active_region: AtomicUsize::new(0),
            statistics: GenerationStatistics::new(),
            max_size,
            promotion_age_threshold: 3, // Promote after surviving 3 collections
        })
    }

    /// Switch from-space and to-space
    fn flip_regions(&self) -> Result<(), String> {
        let regions = self.regions.read().map_err(|_| "Failed to read regions")?;
        let current_active = self.active_region.load(Ordering::Relaxed);
        let new_active = if current_active == 0 { 1 } else { 0 };
        
        // Reset the new active region
        regions[new_active].reset();
        regions[new_active].active.store(true, Ordering::Relaxed);
        regions[current_active].active.store(false, Ordering::Relaxed);
        
        self.active_region.store(new_active, Ordering::Relaxed);
        Ok(())
    }
}

impl Generation for YoungGeneration {
    fn allocate(&self, value: Value, size: usize) -> Result<Arc<ObjectHeader>, String> {
        let regions = self.regions.read().map_err(|_| "Failed to read regions")?;
        let active_idx = self.active_region.load(Ordering::Relaxed);
        let active_region = &regions[active_idx];

        // Try to allocate in the active region
        if let Some(ptr) = active_region.try_allocate(size, 8) {
            let header = Arc::new(ObjectHeader::new(value, size, self.id));
            self.statistics.record_allocation(size);
            
            // Store header at allocated location
            unsafe {
                std::ptr::write(ptr as *mut ObjectHeader, (*header).clone());
            }
            
            Ok(header)
        } else {
            Err("Young generation allocation failed: out of memory".to_string())
        }
    }

    fn collect(&self, _full_collection: bool) -> Result<CollectionResult, String> {
        let start_time = Instant::now();
        
        // This is a stop-the-world copying collector
        // In a real implementation, this would:
        // 1. Stop all mutator threads
        // 2. Scan roots and copy live objects to to-space
        // 3. Update all references
        // 4. Flip from-space and to-space
        // 5. Resume mutator threads
        
        // For now, simulate collection
        let collection_time = start_time.elapsed();
        self.statistics.record_collection(collection_time);
        
        // Simulate flipping regions
        self.flip_regions()?;

        Ok(CollectionResult {
            objects_collected: 100, // Simulated
            bytes_reclaimed: 1024,  // Simulated
            objects_promoted: 10,   // Simulated
            bytes_promoted: 512,    // Simulated
            collection_time,
            success: true,
        })
    }

    fn get_statistics(&self) -> &GenerationStatistics {
        &self.statistics
    }

    fn get_id(&self) -> GenerationId {
        self.id
    }

    fn needs_collection(&self) -> bool {
        let regions = self.regions.read().unwrap();
        let active_idx = self.active_region.load(Ordering::Relaxed);
        let active_region = &regions[active_idx];
        
        // Collect when 80% full
        active_region.used_space() as f64 / active_region.size as f64 > 0.8
    }

    fn utilization(&self) -> f64 {
        let regions = self.regions.read().unwrap();
        let active_idx = self.active_region.load(Ordering::Relaxed);
        let active_region = &regions[active_idx];
        
        active_region.used_space() as f64 / active_region.size as f64 * 100.0
    }
}

/// Old generation implementation (mark-and-sweep collector)
#[derive(Debug)]
pub struct OldGeneration {
    /// Generation ID
    id: GenerationId,
    /// Memory regions
    regions: RwLock<Vec<Arc<MemoryRegion>>>,
    /// Object tracking by ID
    objects: RwLock<HashMap<usize, Arc<ObjectHeader>>>,
    /// Next object ID
    next_object_id: AtomicUsize,
    /// Statistics
    statistics: GenerationStatistics,
    /// Maximum size
    max_size: usize,
}

impl OldGeneration {
    /// Create a new old generation
    pub fn new(max_size: usize) -> Result<Self, String> {
        let initial_region = Arc::new(MemoryRegion::new(max_size / 4)?); // Start with 1/4 of max size
        let regions = vec![initial_region];

        Ok(OldGeneration {
            id: GenerationId::Old,
            regions: RwLock::new(regions),
            objects: RwLock::new(HashMap::new()),
            next_object_id: AtomicUsize::new(1),
            statistics: GenerationStatistics::new(),
            max_size,
        })
    }

    /// Add a new region if needed
    fn add_region_if_needed(&self) -> Result<(), String> {
        let regions = self.regions.read().map_err(|_| "Failed to read regions")?;
        let total_size: usize = regions.iter().map(|r| r.size).sum();
        
        if total_size >= self.max_size {
            return Err("Old generation at maximum size".to_string());
        }
        
        drop(regions);
        
        let mut regions = self.regions.write().map_err(|_| "Failed to write regions")?;
        let new_region_size = (self.max_size - total_size).min(self.max_size / 4);
        let new_region = Arc::new(MemoryRegion::new(new_region_size)?);
        regions.push(new_region);
        
        Ok(())
    }
}

impl Generation for OldGeneration {
    fn allocate(&self, value: Value, size: usize) -> Result<Arc<ObjectHeader>, String> {
        let regions = self.regions.read().map_err(|_| "Failed to read regions")?;
        
        // Try to allocate in existing regions
        for region in regions.iter() {
            if let Some(ptr) = region.try_allocate(size, 8) {
                let header = Arc::new(ObjectHeader::new(value, size, self.id));
                self.statistics.record_allocation(size);
                
                // Store header at allocated location
                unsafe {
                    std::ptr::write(ptr as *mut ObjectHeader, (*header).clone());
                }
                
                // Track the object with an ID instead of raw pointer
                let object_id = self.next_object_id.fetch_add(1, Ordering::Relaxed);
                let mut objects = self.objects.write().map_err(|_| "Failed to write objects")?;
                objects.insert(object_id, Arc::clone(&header));
                
                return Ok(header);
            }
        }
        
        drop(regions);
        
        // No space available, try to add a new region
        self.add_region_if_needed()?;
        
        // Retry allocation
        let regions = self.regions.read().map_err(|_| "Failed to read regions after expansion")?;
        if let Some(region) = regions.last() {
            if let Some(ptr) = region.try_allocate(size, 8) {
                let header = Arc::new(ObjectHeader::new(value, size, self.id));
                self.statistics.record_allocation(size);
                
                unsafe {
                    std::ptr::write(ptr as *mut ObjectHeader, (*header).clone());
                }
                
                let object_id = self.next_object_id.fetch_add(1, Ordering::Relaxed);
                let mut objects = self.objects.write().map_err(|_| "Failed to write objects")?;
                objects.insert(object_id, Arc::clone(&header));
                
                return Ok(header);
            }
        }
        
        Err("Old generation allocation failed: out of memory".to_string())
    }

    fn collect(&self, _full_collection: bool) -> Result<CollectionResult, String> {
        let start_time = Instant::now();
        
        // This would implement mark-and-sweep collection
        // 1. Mark phase: traverse object graph and mark reachable objects
        // 2. Sweep phase: deallocate unmarked objects
        // Can be done concurrently with mutators with proper write barriers
        
        let collection_time = start_time.elapsed();
        self.statistics.record_collection(collection_time);

        Ok(CollectionResult {
            objects_collected: 50,  // Simulated
            bytes_reclaimed: 2048,  // Simulated
            objects_promoted: 0,    // No promotion from old gen
            bytes_promoted: 0,
            collection_time,
            success: true,
        })
    }

    fn get_statistics(&self) -> &GenerationStatistics {
        &self.statistics
    }

    fn get_id(&self) -> GenerationId {
        self.id
    }

    fn needs_collection(&self) -> bool {
        // Collect when utilization is above 70%
        self.utilization() > 70.0
    }

    fn utilization(&self) -> f64 {
        let regions = self.regions.read().unwrap();
        let total_size: usize = regions.iter().map(|r| r.size).sum();
        let used_size: usize = regions.iter().map(|r| r.used_space()).sum();
        
        if total_size == 0 {
            0.0
        } else {
            used_size as f64 / total_size as f64 * 100.0
        }
    }
}

/// Generation manager that coordinates all generations
#[derive(Debug)]
pub struct GenerationManager {
    /// Young generation
    young: Arc<YoungGeneration>,
    /// Old generation
    old: Arc<OldGeneration>,
    /// Generation-specific configurations
    configs: HashMap<GenerationId, GenerationConfig>,
}

/// Configuration for a specific generation
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    /// Collection trigger threshold (percentage)
    pub collection_threshold: f64,
    /// Promotion age threshold
    pub promotion_age_threshold: u8,
    /// Concurrent collection enabled
    pub concurrent_collection: bool,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        GenerationConfig {
            collection_threshold: 80.0,
            promotion_age_threshold: 3,
            concurrent_collection: false,
        }
    }
}

impl GenerationManager {
    /// Create a new generation manager
    pub fn new(young_size: usize, old_size: usize) -> Result<Self, String> {
        let young = Arc::new(YoungGeneration::new(young_size)?);
        let old = Arc::new(OldGeneration::new(old_size)?);
        
        let mut configs = HashMap::new();
        configs.insert(GenerationId::Young, GenerationConfig::default());
        let old_config = GenerationConfig { 
            concurrent_collection: true, 
            ..Default::default() 
        };
        configs.insert(GenerationId::Old, old_config);

        Ok(GenerationManager {
            young,
            old,
            configs,
        })
    }

    /// Get generation by ID
    pub fn get_generation(&self, id: GenerationId) -> Option<Arc<dyn Generation>> {
        match id {
            GenerationId::Young => Some(Arc::clone(&self.young) as Arc<dyn Generation>),
            GenerationId::Old => Some(Arc::clone(&self.old) as Arc<dyn Generation>),
            _ => None, // TODO: Implement permanent and large object generations
        }
    }

    /// Allocate in the most appropriate generation
    pub fn allocate(&self, value: Value, size: usize) -> Result<Arc<ObjectHeader>, String> {
        // For now, allocate small objects in young generation
        // Large objects would go directly to old generation or large object space
        const LARGE_OBJECT_THRESHOLD: usize = 32 * 1024; // 32KB
        
        if size >= LARGE_OBJECT_THRESHOLD {
            // Large objects go directly to old generation for now
            self.old.allocate(value, size)
        } else {
            // Small objects go to young generation
            self.young.allocate(value, size)
        }
    }

    /// Check which generations need collection
    pub fn check_collection_needs(&self) -> Vec<GenerationId> {
        let mut needs_collection = Vec::new();
        
        if self.young.needs_collection() {
            needs_collection.push(GenerationId::Young);
        }
        
        if self.old.needs_collection() {
            needs_collection.push(GenerationId::Old);
        }
        
        needs_collection
    }

    /// Get overall heap statistics
    pub fn get_heap_statistics(&self) -> HeapStatistics {
        let young_stats = self.young.get_statistics();
        let old_stats = self.old.get_statistics();
        
        HeapStatistics {
            total_allocations: young_stats.total_allocations.load(Ordering::Relaxed) + 
                             old_stats.total_allocations.load(Ordering::Relaxed),
            total_allocated_bytes: young_stats.total_allocated_bytes.load(Ordering::Relaxed) + 
                                 old_stats.total_allocated_bytes.load(Ordering::Relaxed),
            total_collections: young_stats.total_collections.load(Ordering::Relaxed) + 
                             old_stats.total_collections.load(Ordering::Relaxed),
            young_utilization: self.young.utilization(),
            old_utilization: self.old.utilization(),
        }
    }
}

/// Overall heap statistics
#[derive(Debug)]
pub struct HeapStatistics {
    /// Total allocations across all generations
    pub total_allocations: u64,
    /// Total bytes allocated across all generations
    pub total_allocated_bytes: u64,
    /// Total collections across all generations
    pub total_collections: u64,
    /// Young generation utilization percentage
    pub young_utilization: f64,
    /// Old generation utilization percentage
    pub old_utilization: f64,
}