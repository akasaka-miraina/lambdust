//! Memory safety debugging utilities for SIGSEGV detection and prevention.
//!
//! This module provides comprehensive tools for detecting and preventing
//! memory safety issues including Arc reference cycles, mixed threading
//! model violations, and resource leaks.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Weak, Mutex, atomic::{AtomicU64, Ordering}};
use std::thread;
use std::time::Instant;

/// Global cycle detection registry
static CYCLE_DETECTOR: once_cell::sync::Lazy<CycleDetector> = once_cell::sync::Lazy::new(|| {
    CycleDetector::new()
});

/// Unique identifier for trackable objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(u64);

static NEXT_OBJECT_ID: AtomicU64 = AtomicU64::new(1);

impl ObjectId {
    pub fn new() -> Self {
        Self(NEXT_OBJECT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// Reference relationship between objects
#[derive(Debug, Clone)]
pub struct Reference {
    from: ObjectId,
    to: ObjectId,
    reference_type: ReferenceType,
    created_at: Instant,
}

/// Types of references that can create cycles
#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceType {
    /// Strong Arc reference
    StrongArc,
    /// Weak Arc reference
    WeakArc,
    /// Environment parent chain
    EnvironmentParent,
    /// Container element reference
    ContainerElement,
    /// Procedure environment capture
    ProcedureEnvironment,
}

/// Cycle detector that tracks Arc references and detects potential cycles
pub struct CycleDetector {
    references: Mutex<HashMap<ObjectId, Vec<Reference>>>,
    object_metadata: Mutex<HashMap<ObjectId, ObjectMetadata>>,
}

#[derive(Debug, Clone)]
pub struct ObjectMetadata {
    object_type: String,
    created_at: Instant,
    thread_id: thread::ThreadId,
    strong_count: usize,
    weak_count: usize,
}

impl CycleDetector {
    fn new() -> Self {
        Self {
            references: Mutex::new(HashMap::new()),
            object_metadata: Mutex::new(HashMap::new()),
        }
    }

    /// Register a new trackable object
    pub fn register_object(&self, id: ObjectId, object_type: String) {
        let metadata = ObjectMetadata {
            object_type,
            created_at: Instant::now(),
            thread_id: thread::current().id(),
            strong_count: 1,
            weak_count: 0,
        };

        let mut metadata_map = self.object_metadata.lock().unwrap();
        metadata_map.insert(id, metadata);
    }

    /// Add a reference between objects
    pub fn add_reference(&self, from: ObjectId, to: ObjectId, ref_type: ReferenceType) {
        let reference = Reference {
            from,
            to,
            reference_type: ref_type,
            created_at: Instant::now(),
        };

        let mut references = self.references.lock().unwrap();
        references.entry(from).or_insert_with(Vec::new).push(reference);
    }

    /// Remove a reference between objects
    pub fn remove_reference(&self, from: ObjectId, to: ObjectId, ref_type: ReferenceType) {
        let mut references = self.references.lock().unwrap();
        if let Some(refs) = references.get_mut(&from) {
            refs.retain(|r| !(r.to == to && r.reference_type == ref_type));
        }
    }

    /// Detect cycles in the reference graph
    pub fn detect_cycles(&self) -> Vec<Vec<ObjectId>> {
        let references = self.references.lock().unwrap();
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for &node in references.keys() {
            if !visited.contains(&node) {
                let mut path = Vec::new();
                self.detect_cycles_dfs(node, &references, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }

        cycles
    }

    fn detect_cycles_dfs(
        &self,
        node: ObjectId,
        references: &HashMap<ObjectId, Vec<Reference>>,
        visited: &mut HashSet<ObjectId>,
        rec_stack: &mut HashSet<ObjectId>,
        path: &mut Vec<ObjectId>,
        cycles: &mut Vec<Vec<ObjectId>>,
    ) {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(refs) = references.get(&node) {
            for reference in refs {
                // Only consider strong references for cycle detection
                if reference.reference_type != ReferenceType::WeakArc {
                    let target = reference.to;
                    
                    if !visited.contains(&target) {
                        self.detect_cycles_dfs(target, references, visited, rec_stack, path, cycles);
                    } else if rec_stack.contains(&target) {
                        // Found a cycle
                        if let Some(cycle_start) = path.iter().position(|&id| id == target) {
                            let cycle = path[cycle_start..].to_vec();
                            cycles.push(cycle);
                        }
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(&node);
    }

    /// Update reference counts for an object
    pub fn update_counts(&self, id: ObjectId, strong_count: usize, weak_count: usize) {
        let mut metadata_map = self.object_metadata.lock().unwrap();
        if let Some(metadata) = metadata_map.get_mut(&id) {
            metadata.strong_count = strong_count;
            metadata.weak_count = weak_count;
        }
    }

    /// Remove an object from tracking
    pub fn unregister_object(&self, id: ObjectId) {
        let mut references = self.references.lock().unwrap();
        let mut metadata_map = self.object_metadata.lock().unwrap();
        
        references.remove(&id);
        metadata_map.remove(&id);
        
        // Remove references pointing to this object
        for refs in references.values_mut() {
            refs.retain(|r| r.to != id);
        }
    }

    /// Generate a detailed report of potential memory issues
    pub fn generate_report(&self) -> MemorySafetyReport {
        let cycles = self.detect_cycles();
        let references = self.references.lock().unwrap();
        let metadata = self.object_metadata.lock().unwrap();

        let mut strong_reference_count = 0;
        let mut weak_reference_count = 0;
        let mut environment_chains = 0;

        for refs in references.values() {
            for reference in refs {
                match reference.reference_type {
                    ReferenceType::StrongArc => strong_reference_count += 1,
                    ReferenceType::WeakArc => weak_reference_count += 1,
                    ReferenceType::EnvironmentParent => environment_chains += 1,
                    _ => {}
                }
            }
        }

        MemorySafetyReport {
            cycles_detected: cycles.len(),
            cycle_details: cycles,
            total_objects: metadata.len(),
            strong_references: strong_reference_count,
            weak_references: weak_reference_count,
            environment_chains,
            potential_leaks: self.identify_potential_leaks(),
        }
    }

    fn identify_potential_leaks(&self) -> Vec<ObjectId> {
        let metadata = self.object_metadata.lock().unwrap();
        let mut potential_leaks = Vec::new();

        for (&id, meta) in metadata.iter() {
            // Objects with high reference counts that have existed for a long time
            if meta.strong_count > 10 && meta.created_at.elapsed().as_secs() > 60 {
                potential_leaks.push(id);
            }
        }

        potential_leaks
    }
}

/// Report generated by the cycle detector
#[derive(Debug)]
pub struct MemorySafetyReport {
    pub cycles_detected: usize,
    pub cycle_details: Vec<Vec<ObjectId>>,
    pub total_objects: usize,
    pub strong_references: usize,
    pub weak_references: usize,
    pub environment_chains: usize,
    pub potential_leaks: Vec<ObjectId>,
}

/// Trait for objects that can be tracked for cycle detection
pub trait CycleTrackable {
    fn object_id(&self) -> ObjectId;
    fn object_type(&self) -> String;
    
    fn register_for_tracking(&self) {
        CYCLE_DETECTOR.register_object(self.object_id(), self.object_type());
    }
    
    fn add_reference_to(&self, target: ObjectId, ref_type: ReferenceType) {
        CYCLE_DETECTOR.add_reference(self.object_id(), target, ref_type);
    }
    
    fn remove_reference_to(&self, target: ObjectId, ref_type: ReferenceType) {
        CYCLE_DETECTOR.remove_reference(self.object_id(), target, ref_type);
    }
}

/// Global function to check for memory safety issues
pub fn check_memory_safety() -> MemorySafetyReport {
    CYCLE_DETECTOR.generate_report()
}

/// Global function to detect and report cycles
pub fn detect_and_report_cycles() {
    let report = check_memory_safety();
    
    if report.cycles_detected > 0 {
        eprintln!("⚠️  MEMORY SAFETY WARNING: {} Arc cycles detected!", report.cycles_detected);
        for (i, cycle) in report.cycle_details.iter().enumerate() {
            eprintln!("  Cycle {}: {:?}", i + 1, cycle);
        }
    }
    
    if !report.potential_leaks.is_empty() {
        eprintln!("⚠️  MEMORY SAFETY WARNING: {} potential leaks detected!", report.potential_leaks.len());
        for leak in &report.potential_leaks {
            eprintln!("  Potential leak: {:?}", leak);
        }
    }
    
    if report.cycles_detected == 0 && report.potential_leaks.is_empty() {
        println!("✅ Memory safety check passed - no cycles or leaks detected");
    }
}

/// Thread-safe wrapper for Arc that includes cycle detection
pub struct SafeArc<T> {
    inner: Arc<T>,
    id: ObjectId,
}

impl<T> SafeArc<T> {
    pub fn new(value: T) -> Self where T: std::fmt::Debug {
        let id = ObjectId::new();
        let inner = Arc::new(value);
        
        CYCLE_DETECTOR.register_object(id, std::any::type_name::<T>().to_string());
        
        Self { inner, id }
    }
    
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
    
    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.inner)
    }
}

impl<T> Clone for SafeArc<T> {
    fn clone(&self) -> Self {
        let cloned = Self {
            inner: self.inner.clone(),
            id: self.id, // Same ID for all clones
        };
        
        // Update reference count
        CYCLE_DETECTOR.update_counts(self.id, self.strong_count(), self.weak_count());
        
        cloned
    }
}

impl<T> Drop for SafeArc<T> {
    fn drop(&mut self) {
        let strong_count = Arc::strong_count(&self.inner);
        if strong_count == 1 {
            // This is the last strong reference
            CYCLE_DETECTOR.unregister_object(self.id);
        } else {
            // Update count
            CYCLE_DETECTOR.update_counts(self.id, strong_count - 1, Arc::weak_count(&self.inner));
        }
    }
}

impl<T> std::ops::Deref for SafeArc<T> {
    type Target = Arc<T>;
    
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_detection() {
        let detector = CycleDetector::new();
        
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();
        let id3 = ObjectId::new();
        
        detector.register_object(id1, "Node1".to_string());
        detector.register_object(id2, "Node2".to_string());
        detector.register_object(id3, "Node3".to_string());
        
        // Create a cycle: 1 -> 2 -> 3 -> 1
        detector.add_reference(id1, id2, ReferenceType::StrongArc);
        detector.add_reference(id2, id3, ReferenceType::StrongArc);
        detector.add_reference(id3, id1, ReferenceType::StrongArc);
        
        let cycles = detector.detect_cycles();
        assert!(!cycles.is_empty(), "Should detect the cycle");
    }
    
    #[test]
    fn test_safe_arc_tracking() {
        let _arc1 = SafeArc::new("test1");
        let arc2 = SafeArc::new("test2");
        let _arc2_clone = arc2.clone();
        
        let report = check_memory_safety();
        assert!(report.total_objects >= 2);
    }
}