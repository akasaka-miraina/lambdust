//! Generational garbage collection system for Lambdust.
//!
//! This module implements a generational garbage collector that reduces GC overhead
//! by focusing collection efforts on recently allocated objects, which are more
//! likely to become garbage quickly (generational hypothesis).

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, Weak};
use std::time::Instant;

/// Generation identifier for GC objects.
pub type GenerationId = u32;

/// The youngest generation (objects are allocated here).
pub const NURSERY_GENERATION: GenerationId = 0;

/// Maximum number of generations before objects are considered long-lived.
pub const MAX_GENERATIONS: GenerationId = 3;

/// Trait for objects that can be garbage collected.
pub trait GcObject: Send + Sync {
    /// Returns the generation this object belongs to.
    fn generation(&self) -> GenerationId;
    
    /// Sets the generation for this object.
    fn set_generation(&mut self, generation: GenerationId);
    
    /// Returns all objects this object references.
    fn references(&self) -> Vec<GcPtr>;
    
    /// Marks this object as reachable during GC.
    fn mark(&self);
    
    /// Checks if this object is marked as reachable.
    fn is_marked(&self) -> bool;
    
    /// Clears the mark for this object.
    fn clear_mark(&self);
    
    /// Returns the approximate size of this object in bytes.
    fn size_hint(&self) -> usize;
}

/// A garbage-collected pointer to an object.
#[derive(Clone)]
pub struct GcPtr {
    inner: Arc<dyn GcObject>,
    id: ObjectId,
}

/// Unique identifier for GC objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(u64);

/// Weak reference to a GC object for breaking cycles.
#[derive(Clone)]
pub struct WeakGcPtr {
    inner: Weak<dyn GcObject>,
    id: ObjectId,
}

/// Statistics about a garbage collection cycle.
#[derive(Debug, Clone)]
pub struct GcStats {
    /// Generation that was collected
    pub generation: GenerationId,
    /// Number of objects before collection
    pub objects_before: usize,
    /// Number of objects after collection  
    pub objects_after: usize,
    /// Objects promoted to next generation
    pub objects_promoted: usize,
    /// Time spent in GC (microseconds)
    pub collection_time_us: u64,
    /// Memory freed (estimated bytes)
    pub memory_freed: usize,
    /// Memory still in use (estimated bytes)
    pub memory_in_use: usize,
}

/// Configuration for the garbage collector.
#[derive(Debug, Clone)]
pub struct GcConfig {
    /// Nursery size threshold for triggering collection (bytes)
    pub nursery_threshold: usize,
    /// Generation 1 threshold for triggering collection (bytes)
    pub gen1_threshold: usize,
    /// Generation 2 threshold for triggering collection (bytes)
    pub gen2_threshold: usize,
    /// Maximum objects to promote per collection
    pub max_promotions: usize,
    /// Enable concurrent collection (experimental)
    pub concurrent_collection: bool,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            nursery_threshold: 1024 * 1024,      // 1MB
            gen1_threshold: 8 * 1024 * 1024,     // 8MB
            gen2_threshold: 32 * 1024 * 1024,    // 32MB
            max_promotions: 1000,
            concurrent_collection: false,
        }
    }
}

/// Generational garbage collector.
pub struct GenerationalGc {
    /// Objects in each generation
    generations: Vec<RwLock<HashSet<ObjectId>>>,
    /// Object registry mapping IDs to objects
    objects: RwLock<HashMap<ObjectId, GcPtr>>,
    /// Weak references to prevent collection
    weak_refs: RwLock<HashMap<ObjectId, Vec<WeakGcPtr>>>,
    /// Root objects that are always reachable
    roots: RwLock<HashSet<ObjectId>>,
    /// Configuration
    config: GcConfig,
    /// Next object ID to assign
    next_id: std::sync::atomic::AtomicU64,
    /// Collection statistics
    stats: RwLock<Vec<GcStats>>,
    /// Memory usage by generation (estimated)
    generation_sizes: RwLock<Vec<usize>>,
}

impl GenerationalGc {
    /// Creates a new generational garbage collector.
    pub fn new(config: GcConfig) -> Self {
        let mut generations = Vec::new();
        let mut generation_sizes = Vec::new();
        
        for _ in 0..=MAX_GENERATIONS {
            generations.push(RwLock::new(HashSet::new()));
            generation_sizes.push(0);
        }
        
        Self {
            generations,
            objects: RwLock::new(HashMap::new()),
            weak_refs: RwLock::new(HashMap::new()),
            roots: RwLock::new(HashSet::new()),
            config,
            next_id: std::sync::atomic::AtomicU64::new(1),
            stats: RwLock::new(Vec::new()),
            generation_sizes: RwLock::new(generation_sizes),
        }
    }
    
    /// Allocates a new object in the nursery generation.
    pub fn alloc<T: GcObject + 'static>(&self, mut obj: T) -> GcPtr {
        let id = ObjectId(self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
        obj.set_generation(NURSERY_GENERATION);
        
        let gc_ptr = GcPtr {
            inner: Arc::new(obj),
            id,
        };
        
        // Add to nursery generation
        if let Ok(mut nursery) = self.generations[NURSERY_GENERATION as usize].write() {
            nursery.insert(id);
        }
        
        // Add to object registry
        if let Ok(mut objects) = self.objects.write() {
            objects.insert(id, gc_ptr.clone());
        }
        
        // Update generation size
        if let Ok(mut sizes) = self.generation_sizes.write() {
            sizes[NURSERY_GENERATION as usize] += gc_ptr.inner.size_hint();
        }
        
        // Check if collection is needed
        self.maybe_collect();
        
        gc_ptr
    }
    
    /// Adds a root object that should never be collected.
    pub fn add_root(&self, ptr: &GcPtr) {
        if let Ok(mut roots) = self.roots.write() {
            roots.insert(ptr.id);
        }
    }
    
    /// Removes a root object.
    pub fn remove_root(&self, ptr: &GcPtr) {
        if let Ok(mut roots) = self.roots.write() {
            roots.remove(&ptr.id);
        }
    }
    
    /// Creates a weak reference to an object.
    pub fn downgrade(&self, ptr: &GcPtr) -> WeakGcPtr {
        let weak_ptr = WeakGcPtr {
            inner: Arc::downgrade(&ptr.inner),
            id: ptr.id,
        };
        
        // Track weak reference
        if let Ok(mut weak_refs) = self.weak_refs.write() {
            weak_refs.entry(ptr.id).or_default().push(weak_ptr.clone());
        }
        
        weak_ptr
    }
    
    /// Triggers garbage collection if thresholds are exceeded.
    pub fn maybe_collect(&self) {
        if let Ok(sizes) = self.generation_sizes.read() {
            // Check nursery threshold
            if sizes[NURSERY_GENERATION as usize] > self.config.nursery_threshold {
                self.collect_generation(NURSERY_GENERATION);
                return;
            }
            
            // Check older generation thresholds
            if sizes[1] > self.config.gen1_threshold {
                self.collect_generation(1);
                return;
            }
            
            if sizes[2] > self.config.gen2_threshold {
                self.collect_generation(2);
            }
        }
    }
    
    /// Forces garbage collection of a specific generation.
    pub fn collect_generation(&self, generation: GenerationId) {
        let start_time = Instant::now();
        
        // Get current object counts
        let objects_before = if let Ok(generation_objects) = self.generations[generation as usize].read() {
            generation_objects.len()
        } else {
            return;
        };
        
        // Mark phase - start from roots
        self.mark_from_roots();
        
        // Mark objects referenced by older generations (if collecting nursery/gen1)
        if generation < MAX_GENERATIONS {
            self.mark_from_older_generations(generation);
        }
        
        // Sweep phase - collect unmarked objects in this generation
        let (objects_collected, memory_freed, objects_promoted) = self.sweep_generation(generation);
        
        // Update statistics
        let collection_time_us = start_time.elapsed().as_micros() as u64;
        let objects_after = objects_before - objects_collected;
        
        let memory_in_use = if let Ok(sizes) = self.generation_sizes.read() {
            sizes.iter().sum()
        } else {
            0
        };
        
        let stats = GcStats {
            generation,
            objects_before,
            objects_after,
            objects_promoted,
            collection_time_us,
            memory_freed,
            memory_in_use,
        };
        
        if let Ok(mut all_stats) = self.stats.write() {
            all_stats.push(stats);
            
            // Keep only last 100 collections for statistics
            if all_stats.len() > 100 {
                all_stats.remove(0);
            }
        }
    }
    
    /// Marks objects reachable from roots.
    fn mark_from_roots(&self) {
        if let (Ok(roots), Ok(objects)) = (self.roots.read(), self.objects.read()) {
            for &root_id in roots.iter() {
                if let Some(root_obj) = objects.get(&root_id) {
                    self.mark_object_and_references(root_obj);
                }
            }
        }
    }
    
    /// Marks objects referenced by older generations.
    fn mark_from_older_generations(&self, generation: GenerationId) {
        if let Ok(objects) = self.objects.read() {
            for gen_idx in (generation + 1)..=MAX_GENERATIONS {
                if let Ok(gen_objects) = self.generations[gen_idx as usize].read() {
                    for &obj_id in gen_objects.iter() {
                        if let Some(obj) = objects.get(&obj_id) {
                            // Mark references that point to younger generations
                            for reference in obj.inner.references() {
                                if let Some(ref_obj) = objects.get(&reference.id) {
                                    if ref_obj.inner.generation() <= generation {
                                        self.mark_object_and_references(ref_obj);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Recursively marks an object and all its references.
    fn mark_object_and_references(&self, obj: &GcPtr) {
        if obj.inner.is_marked() {
            return; // Already marked
        }
        
        obj.inner.mark();
        
        // Mark all referenced objects
        if let Ok(objects) = self.objects.read() {
            for reference in obj.inner.references() {
                if let Some(ref_obj) = objects.get(&reference.id) {
                    self.mark_object_and_references(ref_obj);
                }
            }
        }
    }
    
    /// Sweeps a generation, collecting unmarked objects and promoting survivors.
    fn sweep_generation(&self, generation: GenerationId) -> (usize, usize, usize) {
        let mut objects_collected = 0;
        let mut memory_freed = 0;
        let mut objects_promoted = 0;
        
        let mut to_remove = Vec::new();
        let mut to_promote = Vec::new();
        
        if let (Ok(mut gen_objects), Ok(objects)) = 
            (self.generations[generation as usize].write(), self.objects.read()) {
            
            for &obj_id in gen_objects.iter() {
                if let Some(obj) = objects.get(&obj_id) {
                    if !obj.inner.is_marked() {
                        // Object is unreachable - collect it
                        to_remove.push(obj_id);
                        memory_freed += obj.inner.size_hint();
                        objects_collected += 1;
                    } else {
                        // Object survived - consider promotion
                        if generation < MAX_GENERATIONS && objects_promoted < self.config.max_promotions {
                            to_promote.push(obj_id);
                            objects_promoted += 1;
                        }
                        
                        // Clear mark for next collection
                        obj.inner.clear_mark();
                    }
                }
            }
            
            // Remove collected objects from generation
            for obj_id in &to_remove {
                gen_objects.remove(obj_id);
            }
            
            // Remove promoted objects from current generation
            for obj_id in &to_promote {
                gen_objects.remove(obj_id);
            }
        }
        
        // Promote surviving objects to next generation
        if generation < MAX_GENERATIONS && !to_promote.is_empty() {
            if let (Ok(mut next_gen), Ok(mut objects)) = 
                (self.generations[(generation + 1) as usize].write(), self.objects.write()) {
                
                for obj_id in to_promote {
                    next_gen.insert(obj_id);
                    
                    // Update object's generation
                    if let Some(_obj) = objects.get_mut(&obj_id) {
                        // Note: This requires making the object mutable, which may require
                        // interior mutability in the GcObject implementation
                    }
                }
            }
        }
        
        // Remove collected objects from registry
        if let Ok(mut objects) = self.objects.write() {
            for obj_id in to_remove {
                objects.remove(&obj_id);
            }
        }
        
        // Update generation size
        if let Ok(mut sizes) = self.generation_sizes.write() {
            sizes[generation as usize] = sizes[generation as usize].saturating_sub(memory_freed);
        }
        
        (objects_collected, memory_freed, objects_promoted)
    }
    
    /// Forces a full collection of all generations.
    pub fn collect_all(&self) {
        for generation in 0..=MAX_GENERATIONS {
            self.collect_generation(generation);
        }
    }
    
    /// Returns statistics about recent garbage collections.
    pub fn get_stats(&self) -> Vec<GcStats> {
        if let Ok(stats) = self.stats.read() {
            stats.clone()
        } else {
            Vec::new()
        }
    }
    
    /// Returns current memory usage by generation.
    pub fn memory_usage(&self) -> Vec<usize> {
        if let Ok(sizes) = self.generation_sizes.read() {
            sizes.clone()
        } else {
            vec![0; (MAX_GENERATIONS + 1) as usize]
        }
    }
    
    /// Returns the total number of objects in all generations.
    pub fn object_count(&self) -> usize {
        if let Ok(objects) = self.objects.read() {
            objects.len()
        } else {
            0
        }
    }
    
    /// Returns detailed information about the GC state.
    pub fn debug_info(&self) -> GcDebugInfo {
        let mut generation_counts = Vec::new();
        for generation in &self.generations {
            if let Ok(gen_objects) = generation.read() {
                generation_counts.push(gen_objects.len());
            } else {
                generation_counts.push(0);
            }
        }
        
        let memory_usage = self.memory_usage();
        let total_memory = memory_usage.iter().sum();
        
        let root_count = if let Ok(roots) = self.roots.read() {
            roots.len()
        } else {
            0
        };
        
        let weak_ref_count = if let Ok(weak_refs) = self.weak_refs.read() {
            weak_refs.values().map(|v| v.len()).sum()
        } else {
            0
        };
        
        GcDebugInfo {
            generation_counts,
            memory_usage,
            total_memory,
            root_count,
            weak_ref_count,
            total_objects: self.object_count(),
        }
    }
}

/// Debug information about the garbage collector state.
#[derive(Debug, Clone)]
pub struct GcDebugInfo {
    /// Number of objects in each generation
    pub generation_counts: Vec<usize>,
    /// Memory usage by generation
    pub memory_usage: Vec<usize>,
    /// Total memory in use
    pub total_memory: usize,
    /// Number of root objects
    pub root_count: usize,
    /// Number of weak references
    pub weak_ref_count: usize,
    /// Total number of objects
    pub total_objects: usize,
}

impl GcPtr {
    /// Gets the object ID.
    pub fn id(&self) -> ObjectId {
        self.id
    }
    
    /// Creates a weak reference to this object.
    pub fn downgrade(&self) -> WeakGcPtr {
        WeakGcPtr {
            inner: Arc::downgrade(&self.inner),
            id: self.id,
        }
    }
}

impl std::ops::Deref for GcPtr {
    type Target = dyn GcObject;
    
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl PartialEq for GcPtr {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for GcPtr {}

impl std::hash::Hash for GcPtr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl WeakGcPtr {
    /// Attempts to upgrade this weak reference to a strong reference.
    pub fn upgrade(&self) -> Option<GcPtr> {
        self.inner.upgrade().map(|inner| GcPtr {
            inner,
            id: self.id,
        })
    }
    
    /// Gets the object ID.
    pub fn id(&self) -> ObjectId {
        self.id
    }
}

/// Global garbage collector instance.
static GLOBAL_GC: once_cell::sync::Lazy<GenerationalGc> = 
    once_cell::sync::Lazy::new(|| GenerationalGc::new(GcConfig::default()));

/// Allocates an object using the global garbage collector.
pub fn gc_alloc<T: GcObject + 'static>(obj: T) -> GcPtr {
    GLOBAL_GC.alloc(obj)
}

/// Adds a root to the global garbage collector.
pub fn gc_add_root(ptr: &GcPtr) {
    GLOBAL_GC.add_root(ptr);
}

/// Removes a root from the global garbage collector.
pub fn gc_remove_root(ptr: &GcPtr) {
    GLOBAL_GC.remove_root(ptr);
}

/// Forces garbage collection.
pub fn gc_collect() {
    GLOBAL_GC.collect_all();
}

/// Gets garbage collection statistics.
pub fn gc_stats() -> Vec<GcStats> {
    GLOBAL_GC.get_stats()
}

/// Gets memory usage information.
pub fn gc_memory_usage() -> Vec<usize> {
    GLOBAL_GC.memory_usage()
}

/// Gets debug information about the garbage collector.
pub fn gc_debug_info() -> GcDebugInfo {
    GLOBAL_GC.debug_info()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    
    // Mock GC object for testing
    struct MockObject {
        generation: AtomicU32,
        marked: AtomicBool,
        references: Vec<GcPtr>,
        size: usize,
    }
    
    impl MockObject {
        fn new(size: usize) -> Self {
            Self {
                generation: AtomicU32::new(NURSERY_GENERATION),
                marked: AtomicBool::new(false),
                references: Vec::new(),
                size,
            }
        }
        
        fn with_references(size: usize, references: Vec<GcPtr>) -> Self {
            Self {
                generation: AtomicU32::new(NURSERY_GENERATION),
                marked: AtomicBool::new(false),
                references,
                size,
            }
        }
    }
    
    impl GcObject for MockObject {
        fn generation(&self) -> GenerationId {
            self.generation.load(Ordering::Relaxed)
        }
        
        fn set_generation(&mut self, generation: GenerationId) {
            self.generation.store(generation, Ordering::Relaxed);
        }
        
        fn references(&self) -> Vec<GcPtr> {
            self.references.clone()
        }
        
        fn mark(&self) {
            self.marked.store(true, Ordering::Relaxed);
        }
        
        fn is_marked(&self) -> bool {
            self.marked.load(Ordering::Relaxed)
        }
        
        fn clear_mark(&self) {
            self.marked.store(false, Ordering::Relaxed);
        }
        
        fn size_hint(&self) -> usize {
            self.size
        }
    }
    
    #[test]
    fn test_object_allocation() {
        let gc = GenerationalGc::new(GcConfig::default());
        let obj = gc.alloc(MockObject::new(100));
        
        assert_eq!(obj.generation(), NURSERY_GENERATION);
        assert_eq!(obj.size_hint(), 100);
        assert_eq!(gc.object_count(), 1);
    }
    
    #[test]
    fn test_root_objects() {
        let gc = GenerationalGc::new(GcConfig::default());
        let obj = gc.alloc(MockObject::new(100));
        
        gc.add_root(&obj);
        gc.collect_generation(NURSERY_GENERATION);
        
        // Object should survive collection due to being a root
        assert_eq!(gc.object_count(), 1);
    }
    
    #[test]
    fn test_object_collection() {
        let gc = GenerationalGc::new(GcConfig::default());
        let _obj = gc.alloc(MockObject::new(100));
        
        // Object is not rooted, should be collected
        gc.collect_generation(NURSERY_GENERATION);
        assert_eq!(gc.object_count(), 0);
    }
    
    #[test]
    fn test_weak_references() {
        let gc = GenerationalGc::new(GcConfig::default());
        let obj = gc.alloc(MockObject::new(100));
        let weak_ref = gc.downgrade(&obj);
        
        // Should be able to upgrade while object exists
        assert!(weak_ref.upgrade().is_some());
        
        // Drop strong reference
        drop(obj);
        gc.collect_generation(NURSERY_GENERATION);
        
        // Should not be able to upgrade after collection
        assert!(weak_ref.upgrade().is_none());
    }
    
    #[test]
    fn test_gc_statistics() {
        let gc = GenerationalGc::new(GcConfig::default());
        
        // Allocate some objects
        for i in 0..10 {
            gc.alloc(MockObject::new(i * 10));
        }
        
        // Force collection
        gc.collect_generation(NURSERY_GENERATION);
        
        let stats = gc.get_stats();
        assert!(!stats.is_empty());
        
        let last_stats = &stats[stats.len() - 1];
        assert_eq!(last_stats.generation, NURSERY_GENERATION);
        assert!(last_stats.objects_before > 0);
        assert!(last_stats.collection_time_us > 0);
    }
}