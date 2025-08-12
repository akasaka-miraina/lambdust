//! Allocation pathways with generation-aware allocation strategies
//!
//! This module implements the core allocation subsystem for the parallel
//! garbage collector, providing thread-local allocation buffers (TLABs),
//! allocation sampling, and generation-aware allocation strategies.

use crate::eval::value::Value;
use crate::runtime::gc::generation::{GenerationManager, ObjectHeader, GenerationId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex, atomic::{AtomicUsize, AtomicU64, AtomicBool, Ordering}};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

/// Thread-Local Allocation Buffer (TLAB) for fast allocation
#[derive(Debug)]
pub struct Tlab {
    /// Start of the allocation buffer
    start: *mut u8,
    /// Current allocation pointer
    current: AtomicUsize,
    /// End of the allocation buffer
    end: *mut u8,
    /// Thread ID that owns this TLAB
    owner_thread: ThreadId,
    /// Size of the buffer in bytes
    size: usize,
    /// Number of objects allocated in this TLAB
    objects_allocated: AtomicUsize,
    /// Bytes allocated in this TLAB
    bytes_allocated: AtomicUsize,
    /// Whether this TLAB is currently active
    active: AtomicBool,
}

// SAFETY: Tlab is designed to be thread-safe using atomic operations.
// The raw pointers are properly managed and only accessed through atomic operations.
// The allocation buffer is owned by the TLAB and is not shared directly between threads.
unsafe impl Send for Tlab {}
unsafe impl Sync for Tlab {}

impl Tlab {
    /// Create a new TLAB
    pub fn new(size: usize, owner_thread: ThreadId) -> Result<Self, String> {
        let layout = std::alloc::Layout::from_size_align(size, 8)
            .map_err(|e| format!("Failed to create TLAB layout: {e}"))?;
        
        let start = unsafe { std::alloc::alloc(layout) };
        if start.is_null() {
            return Err("Failed to allocate TLAB memory".to_string());
        }

        let end = unsafe { start.add(size) };

        Ok(Tlab {
            start,
            current: AtomicUsize::new(start as usize),
            end,
            owner_thread,
            size,
            objects_allocated: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
            active: AtomicBool::new(true),
        })
    }

    /// Try to allocate space in this TLAB
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
                // No space available in TLAB
                return None;
            }

            // Try to update the current pointer
            match self.current.compare_exchange_weak(
                current,
                new_current,
                Ordering::Relaxed,
                Ordering::Relaxed
            ) {
                Ok(_) => {
                    self.objects_allocated.fetch_add(1, Ordering::Relaxed);
                    self.bytes_allocated.fetch_add(size, Ordering::Relaxed);
                    return Some(aligned_current as *mut u8);
                }
                Err(_) => continue, // Retry
            }
        }
    }

    /// Get remaining space in this TLAB
    pub fn remaining_space(&self) -> usize {
        let current = self.current.load(Ordering::Relaxed);
        (self.end as usize).saturating_sub(current)
    }

    /// Get used space in this TLAB
    pub fn used_space(&self) -> usize {
        self.bytes_allocated.load(Ordering::Relaxed)
    }

    /// Get utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.size == 0 {
            0.0
        } else {
            self.used_space() as f64 / self.size as f64 * 100.0
        }
    }

    /// Retire this TLAB (mark as inactive)
    pub fn retire(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    /// Check if this TLAB belongs to the current thread
    pub fn belongs_to_thread(&self, thread_id: ThreadId) -> bool {
        self.owner_thread == thread_id
    }
}

impl Drop for Tlab {
    fn drop(&mut self) {
        if !self.start.is_null() {
            let layout = std::alloc::Layout::from_size_align(self.size, 8).unwrap();
            unsafe {
                std::alloc::dealloc(self.start, layout);
            }
        }
    }
}

/// TLAB manager that handles allocation and management of TLABs
#[derive(Debug)]
pub struct TlabManager {
    /// TLABs by thread ID
    tlabs: Arc<RwLock<HashMap<ThreadId, Arc<Tlab>>>>,
    /// Default TLAB size
    default_tlab_size: usize,
    /// Maximum TLAB size
    max_tlab_size: usize,
    /// TLAB allocation statistics
    statistics: TlabStatistics,
    /// Whether adaptive TLAB sizing is enabled
    adaptive_sizing: bool,
}

/// TLAB allocation statistics
#[derive(Debug, Default)]
pub struct TlabStatistics {
    /// Total TLABs created
    pub tlabs_created: AtomicU64,
    /// Total TLABs retired
    pub tlabs_retired: AtomicU64,
    /// Total bytes allocated through TLABs
    pub tlab_allocated_bytes: AtomicU64,
    /// Total waste in retired TLABs
    pub tlab_waste_bytes: AtomicU64,
    /// Average TLAB utilization
    pub avg_utilization: AtomicU64, // Stored as fixed-point (multiply by 100)
}

impl TlabStatistics {
    /// Create new TLAB statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record TLAB creation
    pub fn record_tlab_creation(&self, size: usize) {
        self.tlabs_created.fetch_add(1, Ordering::Relaxed);
        self.tlab_allocated_bytes.fetch_add(size as u64, Ordering::Relaxed);
    }

    /// Record TLAB retirement
    pub fn record_tlab_retirement(&self, used: usize, total: usize) {
        self.tlabs_retired.fetch_add(1, Ordering::Relaxed);
        let waste = total.saturating_sub(used);
        self.tlab_waste_bytes.fetch_add(waste as u64, Ordering::Relaxed);
        
        // Update running average utilization
        let utilization = if total > 0 { (used as f64 / total as f64 * 10000.0) as u64 } else { 0 };
        let retired_count = self.tlabs_retired.load(Ordering::Relaxed);
        let current_avg = self.avg_utilization.load(Ordering::Relaxed);
        let new_avg = (current_avg * (retired_count - 1) + utilization) / retired_count;
        self.avg_utilization.store(new_avg, Ordering::Relaxed);
    }

    /// Get average utilization as percentage
    pub fn average_utilization(&self) -> f64 {
        self.avg_utilization.load(Ordering::Relaxed) as f64 / 100.0
    }

    /// Get waste percentage
    pub fn waste_percentage(&self) -> f64 {
        let allocated = self.tlab_allocated_bytes.load(Ordering::Relaxed);
        let waste = self.tlab_waste_bytes.load(Ordering::Relaxed);
        if allocated > 0 {
            waste as f64 / allocated as f64 * 100.0
        } else {
            0.0
        }
    }
}

impl TlabManager {
    /// Create a new TLAB manager
    pub fn new(default_tlab_size: usize, max_tlab_size: usize) -> Self {
        TlabManager {
            tlabs: Arc::new(RwLock::new(HashMap::new())),
            default_tlab_size,
            max_tlab_size,
            statistics: TlabStatistics::new(),
            adaptive_sizing: true,
        }
    }

    /// Get or create a TLAB for the current thread
    pub fn get_tlab(&self) -> Result<Arc<Tlab>, String> {
        let thread_id = thread::current().id();
        
        // First try to get existing TLAB
        {
            let tlabs = self.tlabs.read().map_err(|_| "Failed to read TLABs")?;
            if let Some(tlab) = tlabs.get(&thread_id) {
                if tlab.active.load(Ordering::Relaxed) && tlab.remaining_space() > 0 {
                    return Ok(Arc::clone(tlab));
                }
            }
        }

        // Need to create a new TLAB
        self.create_tlab_for_thread(thread_id)
    }

    /// Create a new TLAB for a specific thread
    fn create_tlab_for_thread(&self, thread_id: ThreadId) -> Result<Arc<Tlab>, String> {
        let tlab_size = if self.adaptive_sizing {
            self.calculate_adaptive_tlab_size(thread_id)
        } else {
            self.default_tlab_size
        };

        let tlab = Arc::new(Tlab::new(tlab_size, thread_id)?);
        self.statistics.record_tlab_creation(tlab_size);

        // Retire old TLAB if it exists
        {
            let mut tlabs = self.tlabs.write().map_err(|_| "Failed to write TLABs")?;
            if let Some(old_tlab) = tlabs.insert(thread_id, Arc::clone(&tlab)) {
                old_tlab.retire();
                self.statistics.record_tlab_retirement(old_tlab.used_space(), old_tlab.size);
            }
        }

        Ok(tlab)
    }

    /// Calculate adaptive TLAB size based on allocation patterns
    fn calculate_adaptive_tlab_size(&self, _thread_id: ThreadId) -> usize {
        // For now, use a simple heuristic based on average utilization
        let avg_utilization = self.statistics.average_utilization();
        
        if avg_utilization > 80.0 {
            // High utilization, increase TLAB size
            (self.default_tlab_size * 2).min(self.max_tlab_size)
        } else if avg_utilization < 40.0 {
            // Low utilization, decrease TLAB size
            (self.default_tlab_size / 2).max(4096) // Minimum 4KB
        } else {
            // Medium utilization, keep default
            self.default_tlab_size
        }
    }

    /// Retire TLAB for a specific thread
    pub fn retire_tlab(&self, thread_id: ThreadId) -> Result<(), String> {
        let mut tlabs = self.tlabs.write().map_err(|_| "Failed to write TLABs")?;
        if let Some(tlab) = tlabs.remove(&thread_id) {
            tlab.retire();
            self.statistics.record_tlab_retirement(tlab.used_space(), tlab.size);
        }
        Ok(())
    }

    /// Get TLAB statistics
    pub fn get_statistics(&self) -> &TlabStatistics {
        &self.statistics
    }

    /// Cleanup TLABs for dead threads
    pub fn cleanup_dead_threads(&self) -> Result<(), String> {
        let mut tlabs = self.tlabs.write().map_err(|_| "Failed to write TLABs")?;
        let mut to_remove = Vec::new();

        // Identify TLABs for threads that are no longer active
        // In a real implementation, this would check if threads are still alive
        for (thread_id, tlab) in tlabs.iter() {
            if !tlab.active.load(Ordering::Relaxed) {
                to_remove.push(*thread_id);
            }
        }

        // Remove inactive TLABs
        for thread_id in to_remove {
            if let Some(tlab) = tlabs.remove(&thread_id) {
                self.statistics.record_tlab_retirement(tlab.used_space(), tlab.size);
            }
        }

        Ok(())
    }
}

/// Allocation sampling for tracking allocation patterns
#[derive(Debug)]
pub struct AllocationSampler {
    /// Sample every N-th allocation
    sample_rate: usize,
    /// Current allocation count
    allocation_count: AtomicUsize,
    /// Allocation samples
    samples: Arc<Mutex<Vec<AllocationSample>>>,
    /// Maximum number of samples to keep
    max_samples: usize,
}

/// A single allocation sample
#[derive(Debug, Clone)]
pub struct AllocationSample {
    /// Size of the allocation
    pub size: usize,
    /// Thread that performed the allocation
    pub thread_id: ThreadId,
    /// Timestamp of allocation
    pub timestamp: Instant,
    /// Generation where object was allocated
    pub generation: GenerationId,
    /// Allocation site (simplified)
    pub allocation_site: String,
}

impl AllocationSampler {
    /// Create a new allocation sampler
    pub fn new(sample_rate: usize, max_samples: usize) -> Self {
        AllocationSampler {
            sample_rate,
            allocation_count: AtomicUsize::new(0),
            samples: Arc::new(Mutex::new(Vec::new())),
            max_samples,
        }
    }

    /// Maybe sample this allocation
    pub fn maybe_sample(&self, size: usize, generation: GenerationId, allocation_site: String) {
        let count = self.allocation_count.fetch_add(1, Ordering::Relaxed);
        
        if count % self.sample_rate == 0 {
            let sample = AllocationSample {
                size,
                thread_id: thread::current().id(),
                timestamp: Instant::now(),
                generation,
                allocation_site,
            };

            if let Ok(mut samples) = self.samples.lock() {
                samples.push(sample);
                
                // Keep only the most recent samples
                let samples_len = samples.len();
                if samples_len > self.max_samples {
                    samples.drain(0..samples_len - self.max_samples);
                }
            }
        }
    }

    /// Get allocation rate (allocations per second)
    pub fn allocation_rate(&self) -> f64 {
        if let Ok(samples) = self.samples.lock() {
            if samples.len() < 2 {
                return 0.0;
            }

            let oldest = samples.first().unwrap().timestamp;
            let newest = samples.last().unwrap().timestamp;
            let duration = newest.duration_since(oldest);

            if duration.as_secs_f64() > 0.0 {
                (samples.len() * self.sample_rate) as f64 / duration.as_secs_f64()
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Get recent samples
    pub fn get_recent_samples(&self, limit: usize) -> Vec<AllocationSample> {
        if let Ok(samples) = self.samples.lock() {
            if samples.len() <= limit {
                samples.clone()
            } else {
                samples[samples.len() - limit..].to_vec()
            }
        } else {
            Vec::new()
        }
    }

    /// Clear all samples
    pub fn clear_samples(&self) {
        if let Ok(mut samples) = self.samples.lock() {
            samples.clear();
        }
    }
}

/// Main allocation coordinator that manages all allocation strategies
#[derive(Debug)]
pub struct AllocationCoordinator {
    /// Generation manager
    generation_manager: Arc<GenerationManager>,
    /// TLAB manager
    tlab_manager: Arc<TlabManager>,
    /// Allocation sampler
    allocation_sampler: Arc<AllocationSampler>,
    /// Allocation statistics
    statistics: AllocationStatistics,
    /// Large object threshold (bytes)
    large_object_threshold: usize,
}

/// Overall allocation statistics
#[derive(Debug, Default)]
pub struct AllocationStatistics {
    /// Total allocations
    pub total_allocations: AtomicU64,
    /// Total bytes allocated
    pub total_allocated_bytes: AtomicU64,
    /// Young generation allocations
    pub young_allocations: AtomicU64,
    /// Old generation allocations
    pub old_allocations: AtomicU64,
    /// Large object allocations
    pub large_allocations: AtomicU64,
    /// Failed allocations
    pub failed_allocations: AtomicU64,
    /// Average allocation size
    pub avg_allocation_size: AtomicU64,
}

impl AllocationStatistics {
    /// Create new allocation statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an allocation
    pub fn record_allocation(&self, size: usize, generation: GenerationId) {
        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        self.total_allocated_bytes.fetch_add(size as u64, Ordering::Relaxed);

        match generation {
            GenerationId::Young => { self.young_allocations.fetch_add(1, Ordering::Relaxed); },
            GenerationId::Old => { self.old_allocations.fetch_add(1, Ordering::Relaxed); },
            GenerationId::LargeObject => { self.large_allocations.fetch_add(1, Ordering::Relaxed); },
            _ => {}
        }

        // Update average allocation size
        let total = self.total_allocations.load(Ordering::Relaxed);
        let total_bytes = self.total_allocated_bytes.load(Ordering::Relaxed);
        if total > 0 {
            self.avg_allocation_size.store(total_bytes / total, Ordering::Relaxed);
        }
    }

    /// Record a failed allocation
    pub fn record_failed_allocation(&self) {
        self.failed_allocations.fetch_add(1, Ordering::Relaxed);
    }

    /// Get allocation failure rate
    pub fn failure_rate(&self) -> f64 {
        let total = self.total_allocations.load(Ordering::Relaxed);
        let failed = self.failed_allocations.load(Ordering::Relaxed);
        if total > 0 {
            failed as f64 / (total + failed) as f64 * 100.0
        } else {
            0.0
        }
    }
}

impl AllocationCoordinator {
    /// Create a new allocation coordinator
    pub fn new(
        generation_manager: Arc<GenerationManager>,
        default_tlab_size: usize,
        max_tlab_size: usize,
        large_object_threshold: usize,
    ) -> Self {
        let tlab_manager = Arc::new(TlabManager::new(default_tlab_size, max_tlab_size));
        let allocation_sampler = Arc::new(AllocationSampler::new(1000, 10000)); // Sample 1 in 1000, keep 10k samples

        AllocationCoordinator {
            generation_manager,
            tlab_manager,
            allocation_sampler,
            statistics: AllocationStatistics::new(),
            large_object_threshold,
        }
    }

    /// Allocate a new object
    pub fn allocate(&self, value: Value, size: usize) -> Result<Arc<ObjectHeader>, String> {
        let allocation_site = format!("{}:{}", file!(), line!()); // Simplified
        
        // Sample this allocation
        let generation = self.choose_generation(size);
        self.allocation_sampler.maybe_sample(size, generation, allocation_site);

        // Try different allocation strategies based on size
        let result = if size >= self.large_object_threshold {
            // Large object - allocate directly in old generation or large object space
            self.allocate_large_object(value, size)
        } else {
            // Small object - try TLAB allocation first
            self.allocate_small_object(value, size)
        };

        match &result {
            Ok(_) => {
                self.statistics.record_allocation(size, generation);
            }
            Err(_) => {
                self.statistics.record_failed_allocation();
            }
        }

        result
    }

    /// Choose the appropriate generation for an object
    fn choose_generation(&self, size: usize) -> GenerationId {
        if size >= self.large_object_threshold {
            GenerationId::LargeObject
        } else {
            GenerationId::Young // Most objects start in young generation
        }
    }

    /// Allocate a small object (using TLAB if possible)
    fn allocate_small_object(&self, value: Value, size: usize) -> Result<Arc<ObjectHeader>, String> {
        // Try TLAB allocation first
        if let Ok(tlab) = self.tlab_manager.get_tlab() {
            if let Some(ptr) = tlab.try_allocate(size, 8) {
                // Successfully allocated in TLAB
                let header = ObjectHeader::new(value, size, GenerationId::Young);
                
                // Store header at allocated location
                unsafe {
                    std::ptr::write(ptr as *mut ObjectHeader, header.clone());
                }
                
                return Ok(Arc::new(header));
            }
        }

        // TLAB allocation failed, fall back to generation allocation
        self.generation_manager.allocate(value, size)
    }

    /// Allocate a large object
    fn allocate_large_object(&self, value: Value, size: usize) -> Result<Arc<ObjectHeader>, String> {
        // Large objects go directly to old generation for now
        // In a full implementation, they would go to a dedicated large object space
        self.generation_manager.allocate(value, size)
    }

    /// Get current allocation rate (objects per second)
    pub fn allocation_rate(&self) -> f64 {
        self.allocation_sampler.allocation_rate()
    }

    /// Get allocation statistics
    pub fn get_statistics(&self) -> &AllocationStatistics {
        &self.statistics
    }

    /// Get TLAB statistics
    pub fn get_tlab_statistics(&self) -> &TlabStatistics {
        self.tlab_manager.get_statistics()
    }

    /// Cleanup resources (should be called periodically)
    pub fn cleanup(&self) -> Result<(), String> {
        self.tlab_manager.cleanup_dead_threads()
    }

    /// Retire TLAB for current thread
    pub fn retire_current_thread_tlab(&self) -> Result<(), String> {
        let thread_id = thread::current().id();
        self.tlab_manager.retire_tlab(thread_id)
    }
}