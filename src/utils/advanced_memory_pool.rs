//! Advanced memory pool implementation for high-performance object allocation
//!
//! This module provides sophisticated memory pooling strategies to minimize
//! allocation overhead and fragmentation, with specialized pools for different
//! object types commonly used in the Scheme interpreter.

use std::alloc::{Layout, GlobalAlloc, System};
use std::ptr::{NonNull, null_mut};
use std::sync::{Mutex, Arc, atomic::{AtomicUsize, AtomicPtr, Ordering}};
use std::collections::HashMap;
use std::mem::{size_of, align_of};
use std::thread;

/// Thread-safe wrapper for NonNull pointers from memory pool
/// 
/// This wrapper implements Send because the underlying pointers are managed
/// by thread-safe memory pool mechanisms and are guaranteed to be valid
/// across thread boundaries within the pool's lifetime.
#[derive(Debug)]
pub struct ThreadSafePtr(NonNull<u8>);

impl ThreadSafePtr {
    /// Creates a new thread-safe pointer wrapper
    pub fn new(ptr: NonNull<u8>) -> Self {
        Self(ptr)
    }
    
    /// Extracts the underlying NonNull pointer
    pub fn into_inner(self) -> NonNull<u8> {
        self.0
    }
    
    /// Gets a reference to the underlying pointer
    pub fn as_ptr(&self) -> NonNull<u8> {
        self.0
    }
}

// Safety: ThreadSafePtr can be sent between threads because the underlying
// NonNull<u8> pointers are allocated and managed by thread-safe memory pool
// operations that ensure validity across thread boundaries.
unsafe impl Send for ThreadSafePtr {}
unsafe impl Sync for ThreadSafePtr {}

/// Configuration for memory pools
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Initial number of objects to pre-allocate
    pub initial_size: usize,
    /// Maximum number of objects in the pool
    pub max_size: usize,
    /// Whether to grow the pool when exhausted
    pub grow_on_exhaustion: bool,
    /// Growth factor when expanding the pool
    pub growth_factor: f32,
    /// Whether to enable pool statistics
    pub enable_stats: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            initial_size: 100,
            max_size: 10000,
            grow_on_exhaustion: true,
            growth_factor: 1.5,
            enable_stats: true,
        }
    }
}

/// Statistics for a memory pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total allocations from this pool
    pub total_allocations: usize,
    /// Total deallocations to this pool
    pub total_deallocations: usize,
    /// Current number of allocated objects
    pub current_allocated: usize,
    /// Current number of free objects in pool
    pub current_free: usize,
    /// Peak number of allocated objects
    pub peak_allocated: usize,
    /// Number of times pool was grown
    pub growth_count: usize,
    /// Number of allocations that missed the pool (went to system allocator)
    pub pool_misses: usize,
    /// Pool efficiency as percentage (pool hits / total allocations)
    pub efficiency: f64,
}

impl PoolStats {
    fn new() -> Self {
        Self {
            total_allocations: 0,
            total_deallocations: 0,
            current_allocated: 0,
            current_free: 0,
            peak_allocated: 0,
            growth_count: 0,
            pool_misses: 0,
            efficiency: 0.0,
        }
    }
    
    fn update_efficiency(&mut self) {
        if self.total_allocations > 0 {
            let pool_hits = self.total_allocations - self.pool_misses;
            self.efficiency = (pool_hits as f64 / self.total_allocations as f64) * 100.0;
        }
    }
}

/// Block in the free list
#[repr(C)]
struct FreeBlock {
    next: *mut FreeBlock,
}

/// A memory pool for objects of a specific size
pub struct MemoryPool {
    /// Configuration for this pool
    config: PoolConfig,
    /// Size of objects allocated from this pool
    object_size: usize,
    /// Alignment requirement for objects
    object_align: usize,
    /// Head of the free list
    free_list: AtomicPtr<FreeBlock>,
    /// Statistics for this pool
    stats: Mutex<PoolStats>,
    /// Allocated chunks for cleanup
    chunks: Mutex<Vec<NonNull<u8>>>,
}

// Safety: MemoryPool maintains thread-safe access to its internal state
// The NonNull<u8> pointers are only used internally and protected by mutexes
unsafe impl Send for MemoryPool {}
unsafe impl Sync for MemoryPool {}

impl MemoryPool {
    /// Creates a new memory pool for objects of the given size and alignment
    pub fn new(object_size: usize, object_align: usize, config: PoolConfig) -> Result<Self, String> {
        if object_size == 0 {
            return Err("Object size cannot be zero".to_string());
        }
        
        if !object_align.is_power_of_two() {
            return Err("Object alignment must be a power of two".to_string());
        }
        
        let pool = Self {
            config,
            object_size: object_size.max(size_of::<FreeBlock>()),
            object_align: object_align.max(align_of::<FreeBlock>()),
            free_list: AtomicPtr::new(null_mut()),
            stats: Mutex::new(PoolStats::new()),
            chunks: Mutex::new(Vec::new()),
        };
        
        // Pre-allocate initial objects
        if pool.config.initial_size > 0 {
            pool.grow_pool(pool.config.initial_size)?;
        }
        
        Ok(pool)
    }
    
    /// Allocates an object from the pool
    pub fn allocate(&self) -> Option<NonNull<u8>> {
        // Try to get object from free list
        loop {
            let current_head = self.free_list.load(Ordering::Acquire);
            
            if current_head.is_null() {
                // Free list is empty, try to grow the pool
                if self.config.grow_on_exhaustion {
                    let current_size = self.get_current_size();
                    let new_size = (current_size as f32 * self.config.growth_factor) as usize;
                    let growth_size = new_size.saturating_sub(current_size).max(10);
                    
                    if current_size + growth_size <= self.config.max_size {
                        if let Ok(()) = self.grow_pool(growth_size) {
                            continue; // Try allocation again
                        }
                    }
                }
                
                // Pool exhausted or growth failed, update stats
                if let Ok(mut stats) = self.stats.lock() {
                    stats.pool_misses += 1;
                    stats.total_allocations += 1;
                    stats.update_efficiency();
                }
                
                return None; // Let caller fallback to system allocator
            }
            
            // Try to pop from free list
            let next = unsafe { (*current_head).next };
            if self.free_list.compare_exchange_weak(
                current_head,
                next,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                // Successfully allocated from pool
                if let Ok(mut stats) = self.stats.lock() {
                    stats.total_allocations += 1;
                    stats.current_allocated += 1;
                    stats.current_free = stats.current_free.saturating_sub(1);
                    stats.peak_allocated = stats.peak_allocated.max(stats.current_allocated);
                    stats.update_efficiency();
                }
                
                return Some(unsafe { NonNull::new_unchecked(current_head as *mut u8) });
            }
            // CAS failed, retry
        }
    }
    
    /// Deallocates an object back to the pool
    pub fn deallocate(&self, ptr: NonNull<u8>) {
        let block = ptr.as_ptr() as *mut FreeBlock;
        
        // Push onto free list
        loop {
            let current_head = self.free_list.load(Ordering::Acquire);
            unsafe { (*block).next = current_head };
            
            if self.free_list.compare_exchange_weak(
                current_head,
                block,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                break;
            }
        }
        
        // Update stats
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_deallocations += 1;
            stats.current_allocated = stats.current_allocated.saturating_sub(1);
            stats.current_free += 1;
        }
    }
    
    /// Grows the pool by allocating more objects
    fn grow_pool(&self, count: usize) -> Result<(), String> {
        if count == 0 {
            return Ok(());
        }
        
        let layout = Layout::from_size_align(self.object_size * count, self.object_align)
            .map_err(|e| format!("Invalid layout: {e:?}"))?;
        
        // Allocate chunk from system
        let chunk_ptr = unsafe { System.alloc(layout) };
        if chunk_ptr.is_null() {
            return Err("System allocation failed".to_string());
        }
        
        let chunk = unsafe { NonNull::new_unchecked(chunk_ptr) };
        
        // Add to chunks list for cleanup
        if let Ok(mut chunks) = self.chunks.lock() {
            chunks.push(chunk);
        }
        
        // Link all objects in the chunk into the free list
        for i in 0..count {
            let object_ptr = unsafe { 
                chunk_ptr.add(i * self.object_size) as *mut FreeBlock 
            };
            
            // Push onto free list
            loop {
                let current_head = self.free_list.load(Ordering::Acquire);
                unsafe { (*object_ptr).next = current_head };
                
                if self.free_list.compare_exchange_weak(
                    current_head,
                    object_ptr,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    break;
                }
            }
        }
        
        // Update stats
        if let Ok(mut stats) = self.stats.lock() {
            stats.current_free += count;
            stats.growth_count += 1;
        }
        
        Ok(())
    }
    
    /// Gets the current total size of the pool
    fn get_current_size(&self) -> usize {
        if let Ok(stats) = self.stats.lock() {
            stats.current_allocated + stats.current_free
        } else {
            0
        }
    }
    
    /// Gets pool statistics
    pub fn get_stats(&self) -> PoolStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            PoolStats::new()
        }
    }
    
    /// Resets pool statistics
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            *stats = PoolStats::new();
        }
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        // Clean up all allocated chunks
        if let Ok(chunks) = self.chunks.lock() {
            for &chunk in chunks.iter() {
                let layout = Layout::from_size_align(
                    self.object_size * self.config.initial_size,
                    self.object_align,
                ).unwrap();
                unsafe {
                    System.dealloc(chunk.as_ptr(), layout);
                }
            }
        }
    }
}

/// Manager for multiple memory pools
pub struct PoolManager {
    /// Pools indexed by (size, alignment)
    pools: Mutex<HashMap<(usize, usize), Arc<MemoryPool>>>,
    /// Configuration for new pools
    default_config: PoolConfig,
}

impl PoolManager {
    /// Creates a new pool manager
    pub fn new(default_config: PoolConfig) -> Self {
        Self {
            pools: Mutex::new(HashMap::new()),
            default_config,
        }
    }
    
    /// Gets or creates a pool for the given size and alignment
    pub fn get_pool(&self, size: usize, align: usize) -> Result<Arc<MemoryPool>, String> {
        let mut pools = self.pools.lock().unwrap();
        
        let key = (size, align);
        if let Some(pool) = pools.get(&key) {
            return Ok(pool.clone());
        }
        
        // Create new pool
        let pool = Arc::new(MemoryPool::new(size, align, self.default_config.clone())?);
        pools.insert(key, pool.clone());
        
        Ok(pool)
    }
    
    /// Allocates an object of the given size and alignment
    pub fn allocate(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        if let Ok(pool) = self.get_pool(size, align) {
            pool.allocate()
        } else {
            None
        }
    }
    
    /// Deallocates an object with the given size and alignment
    pub fn deallocate(&self, ptr: NonNull<u8>, size: usize, align: usize) -> Result<(), String> {
        let pool = self.get_pool(size, align)?;
        pool.deallocate(ptr);
        Ok(())
    }
    
    /// Gets statistics for all pools
    pub fn get_global_stats(&self) -> GlobalPoolStats {
        let pools = self.pools.lock().unwrap();
        
        let mut global_stats = GlobalPoolStats {
            pool_count: pools.len(),
            total_allocations: 0,
            total_deallocations: 0,
            total_pool_misses: 0,
            overall_efficiency: 0.0,
            pool_stats: HashMap::new(),
        };
        
        for (&(size, align), pool) in pools.iter() {
            let stats = pool.get_stats();
            global_stats.total_allocations += stats.total_allocations;
            global_stats.total_deallocations += stats.total_deallocations;
            global_stats.total_pool_misses += stats.pool_misses;
            global_stats.pool_stats.insert((size, align), stats);
        }
        
        if global_stats.total_allocations > 0 {
            let pool_hits = global_stats.total_allocations - global_stats.total_pool_misses;
            global_stats.overall_efficiency = 
                (pool_hits as f64 / global_stats.total_allocations as f64) * 100.0;
        }
        
        global_stats
    }
}

/// Global statistics for all memory pools
#[derive(Debug, Clone)]
pub struct GlobalPoolStats {
    /// Number of active pools
    pub pool_count: usize,
    /// Total allocations across all pools
    pub total_allocations: usize,
    /// Total deallocations across all pools
    pub total_deallocations: usize,
    /// Total pool misses (fallbacks to system allocator)
    pub total_pool_misses: usize,
    /// Overall efficiency across all pools
    pub overall_efficiency: f64,
    /// Stats for individual pools
    pub pool_stats: HashMap<(usize, usize), PoolStats>,
}

impl GlobalPoolStats {
    /// Formats the statistics as a human-readable report
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Memory Pool Statistics ===\n");
        report.push_str(&format!("Active Pools: {}\n", self.pool_count));
        report.push_str(&format!("Total Allocations: {}\n", self.total_allocations));
        report.push_str(&format!("Total Deallocations: {}\n", self.total_deallocations));
        report.push_str(&format!("Pool Misses: {}\n", self.total_pool_misses));
        report.push_str(&format!("Overall Efficiency: {:.1}%\n", self.overall_efficiency));
        report.push_str("\n=== Individual Pool Stats ===\n");
        
        let mut pools: Vec<_> = self.pool_stats.iter().collect();
        pools.sort_by_key(|(_, stats)| std::cmp::Reverse(stats.total_allocations));
        
        for ((size, align), stats) in pools.iter().take(10) {
            report.push_str(&format!(
                "Pool {}B/{}: {} allocs, {:.1}% efficiency\n",
                size, align, stats.total_allocations, stats.efficiency
            ));
        }
        
        report
    }
    
    /// Gets the overall efficiency score
    pub fn overall_efficiency(&self) -> f64 {
        self.overall_efficiency
    }
}

// Specialized pools for common Scheme object types

/// Memory pool specifically for Scheme cons cells
pub struct ConsPool {
    pool: Arc<MemoryPool>,
}

impl ConsPool {
    /// Creates a new cons cell memory pool.
    pub fn new() -> Result<Self, String> {
        let config = PoolConfig {
            initial_size: 1000,
            max_size: 100000,
            growth_factor: 2.0,
            ..Default::default()
        };
        
        let pool = MemoryPool::new(
            size_of::<ConsCellRepr>(),
            align_of::<ConsCellRepr>(),
            config,
        )?;
        
        Ok(Self { pool: Arc::new(pool) })
    }
    
    /// Allocates memory for a cons cell.
    pub fn allocate_cons(&self) -> Option<NonNull<ConsCellRepr>> {
        self.pool.allocate().map(|ptr| ptr.cast())
    }
    
    /// Deallocates memory for a cons cell.
    pub fn deallocate_cons(&self, ptr: NonNull<ConsCellRepr>) {
        self.pool.deallocate(ptr.cast());
    }
}

/// Memory pool for small objects (symbols, numbers, etc.)
pub struct SmallObjectPool {
    pool: Arc<MemoryPool>,
}

impl SmallObjectPool {
    /// Creates a new small object memory pool.
    pub fn new() -> Result<Self, String> {
        let config = PoolConfig {
            initial_size: 500,
            max_size: 50000,
            growth_factor: 1.8,
            ..Default::default()
        };
        
        let pool = MemoryPool::new(32, 8, config)?; // 32 bytes, 8-byte aligned
        
        Ok(Self { pool: Arc::new(pool) })
    }
    
    /// Allocates memory for a small object.
    pub fn allocate(&self) -> Option<NonNull<u8>> {
        self.pool.allocate()
    }
    
    /// Deallocates memory for a small object.
    pub fn deallocate(&self, ptr: NonNull<u8>) {
        self.pool.deallocate(ptr);
    }
}

// Representation types for Scheme objects
/// Represents a cons cell in memory with car and cdr pointers.
#[repr(C)]
pub struct ConsCellRepr {
    car: *mut u8, // Pointer to car value
    cdr: *mut u8, // Pointer to cdr value
}

// Global pool manager instance
use std::sync::OnceLock;
static GLOBAL_POOL_MANAGER: OnceLock<PoolManager> = OnceLock::new();

/// Gets the global pool manager
pub fn global_pool_manager() -> &'static PoolManager {
    GLOBAL_POOL_MANAGER.get_or_init(|| {
        PoolManager::new(PoolConfig::default())
    })
}

/// Gets global pool statistics
pub fn comprehensive_pool_stats() -> GlobalPoolStats {
    global_pool_manager().get_global_stats()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_memory_pool_basic_operations() {
        let config = PoolConfig::default();
        let pool = MemoryPool::new(64, 8, config).unwrap();
        
        // Test allocation
        let ptr1 = pool.allocate().unwrap();
        let ptr2 = pool.allocate().unwrap();
        
        assert_ne!(ptr1.as_ptr(), ptr2.as_ptr());
        
        // Test deallocation
        pool.deallocate(ptr1);
        pool.deallocate(ptr2);
        
        let stats = pool.get_stats();
        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.total_deallocations, 2);
    }
    
    #[test]
    fn test_pool_growth() {
        let config = PoolConfig {
            initial_size: 2,
            max_size: 10,
            grow_on_exhaustion: true,
            growth_factor: 2.0,
            enable_stats: true,
        };
        
        let pool = MemoryPool::new(32, 8, config).unwrap();
        
        // Allocate more than initial size to trigger growth
        let mut ptrs = Vec::new();
        for _ in 0..5 {
            if let Some(ptr) = pool.allocate() {
                ptrs.push(ptr);
            }
        }
        
        let stats = pool.get_stats();
        assert!(stats.growth_count > 0);
        
        // Clean up
        for ptr in ptrs {
            pool.deallocate(ptr);
        }
    }
    
    #[test]
    fn test_pool_manager() {
        let manager = PoolManager::new(PoolConfig::default());
        
        // Test getting pools for different sizes
        let pool1 = manager.get_pool(32, 8).unwrap();
        let pool2 = manager.get_pool(64, 8).unwrap();
        let pool1_again = manager.get_pool(32, 8).unwrap();
        
        // Same size/align should return same pool
        assert!(Arc::ptr_eq(&pool1, &pool1_again));
        
        // Different sizes should return different pools
        assert!(!Arc::ptr_eq(&pool1, &pool2));
    }
    
    #[test]
    fn test_concurrent_access() {
        let pool = Arc::new(MemoryPool::new(64, 8, PoolConfig::default()).unwrap());
        
        let handles: Vec<_> = (0..4).map(|_| {
            let pool_clone = pool.clone();
            thread::spawn(move || {
                let mut ptrs = Vec::new();
                
                // Allocate objects
                for _ in 0..100 {
                    if let Some(ptr) = pool_clone.allocate() {
                        ptrs.push(ThreadSafePtr::new(ptr));
                    }
                }
                
                // Deallocate half of them
                for ptr in ptrs.drain(..50) {
                    pool_clone.deallocate(ptr.into_inner());
                }
                
                ptrs
            })
        }).collect();
        
        // Wait for all threads
        let mut total_remaining = 0;
        for handle in handles {
            total_remaining += handle.join().unwrap().len();
        }
        
        let stats = pool.get_stats();
        assert_eq!(stats.total_allocations, 400);
        assert_eq!(stats.total_deallocations, 200);
        assert_eq!(stats.current_allocated, 200);
    }
    
    #[test]
    fn test_specialized_pools() {
        let cons_pool = ConsPool::new().unwrap();
        let small_pool = SmallObjectPool::new().unwrap();
        
        // Test cons pool
        let cons_ptr = cons_pool.allocate_cons().unwrap();
        cons_pool.deallocate_cons(cons_ptr);
        
        // Test small object pool
        let small_ptr = small_pool.allocate().unwrap();
        small_pool.deallocate(small_ptr);
    }
}