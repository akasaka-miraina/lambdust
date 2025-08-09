//! FFI-specific memory management system.
//!
//! This module provides safe and efficient memory management for FFI operations,
//! including allocation tracking, leak prevention, garbage collection integration,
//! and proper memory alignment handling.

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::fmt;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};

use crate::diagnostics::Error;
use crate::ffi::c_types::CType;

/// Errors that can occur during FFI memory operations
#[derive(Debug, Clone)]
pub enum MemoryError {
    /// Allocation failed
    AllocationFailed {
        size: usize,
        alignment: usize,
    },
    /// Invalid pointer or null pointer access
    InvalidPointer(*const u8),
    /// Double free detected
    DoubleFree(*const u8),
    /// Memory leak detected
    LeakDetected {
        ptr: *const u8,
        size: usize,
        allocated_at: SystemTime,
    },
    /// Buffer overflow protection
    BufferOverflow {
        ptr: *const u8,
        size: usize,
        accessed_size: usize,
    },
    /// Memory alignment error
    AlignmentError {
        ptr: *const u8,
        required_alignment: usize,
        actual_alignment: usize,
    },
    /// Memory pool exhausted
    PoolExhausted {
        pool_name: String,
        requested_size: usize,
    },
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::AllocationFailed { size, alignment } => {
                write!(f, "Memory allocation failed: {size} bytes with {alignment} alignment")
            }
            MemoryError::InvalidPointer(ptr) => {
                write!(f, "Invalid pointer: {ptr:p}")
            }
            MemoryError::DoubleFree(ptr) => {
                write!(f, "Double free detected for pointer: {ptr:p}")
            }
            MemoryError::LeakDetected { ptr, size, allocated_at } => {
                write!(f, "Memory leak detected: {ptr:p} ({size} bytes, allocated at {allocated_at:?})")
            }
            MemoryError::BufferOverflow { ptr, size, accessed_size } => {
                write!(f, "Buffer overflow: pointer {ptr:p}, buffer size {size}, accessed size {accessed_size}")
            }
            MemoryError::AlignmentError { ptr, required_alignment, actual_alignment } => {
                write!(f, "Memory alignment error: pointer {ptr:p}, required {required_alignment}, actual {actual_alignment}")
            }
            MemoryError::PoolExhausted { pool_name, requested_size } => {
                write!(f, "Memory pool '{pool_name}' exhausted, requested {requested_size} bytes")
            }
        }
    }
}

impl std::error::Error for MemoryError {}

impl From<MemoryError> for Error {
    fn from(memory_error: MemoryError) -> Self {
        Error::runtime_error(memory_error.to_string(), None)
    }
}

/// Memory allocation metadata
#[derive(Debug, Clone)]
pub struct AllocationInfo {
    /// Pointer to allocated memory
    pub ptr: NonNull<u8>,
    /// Size of allocation in bytes
    pub size: usize,
    /// Memory layout used for allocation
    pub layout: Layout,
    /// When this memory was allocated
    pub allocated_at: SystemTime,
    /// Reference count for shared ownership
    pub ref_count: Arc<Mutex<usize>>,
    /// Optional type information
    pub c_type: Option<CType>,
    /// Debug information
    pub debug_info: Option<String>,
}

impl AllocationInfo {
    /// Create new allocation info
    pub fn new(ptr: NonNull<u8>, layout: Layout, c_type: Option<CType>) -> Self {
        Self {
            ptr,
            size: layout.size(),
            layout,
            allocated_at: SystemTime::now(),
            ref_count: Arc::new(Mutex::new(1)),
            c_type,
            debug_info: None,
        }
    }

    /// Increment reference count
    pub fn add_ref(&self) {
        let mut count = self.ref_count.lock().unwrap();
        *count += 1;
    }

    /// Decrement reference count and return new count
    pub fn release_ref(&self) -> usize {
        let mut count = self.ref_count.lock().unwrap();
        if *count > 0 {
            *count -= 1;
        }
        *count
    }

    /// Get current reference count
    pub fn ref_count(&self) -> usize {
        *self.ref_count.lock().unwrap()
    }

    /// Check if pointer is aligned correctly
    pub fn is_aligned(&self) -> bool {
        let addr = self.ptr.as_ptr() as usize;
        addr % self.layout.align() == 0
    }
}

/// FFI memory manager
#[derive(Debug)]
pub struct FfiMemoryManager {
    /// Active allocations tracking
    allocations: RwLock<HashMap<*const u8, AllocationInfo>>,
    /// Memory statistics
    stats: RwLock<MemoryStats>,
    /// Configuration
    config: RwLock<MemoryConfig>,
    /// Memory pools for different sizes
    pools: RwLock<HashMap<String, MemoryPool>>,
}

/// Memory usage statistics
#[derive(Debug, Default, Clone)]
pub struct MemoryStats {
    /// Total bytes allocated
    pub total_allocated: usize,
    /// Total bytes freed
    pub total_freed: usize,
    /// Current bytes in use
    pub current_usage: usize,
    /// Peak memory usage
    pub peak_usage: usize,
    /// Number of active allocations
    pub active_allocations: usize,
    /// Number of allocation operations
    pub allocation_count: u64,
    /// Number of deallocation operations
    pub deallocation_count: u64,
    /// Number of leaks detected
    pub leaks_detected: u64,
    /// Number of double frees prevented
    pub double_frees_prevented: u64,
}

/// Memory manager configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Enable leak detection
    pub leak_detection: bool,
    /// Enable double-free protection
    pub double_free_protection: bool,
    /// Enable buffer overflow protection
    pub buffer_overflow_protection: bool,
    /// Maximum memory usage (0 = unlimited)
    pub max_memory_usage: usize,
    /// Enable memory pools
    pub use_memory_pools: bool,
    /// Pool sizes to pre-allocate
    pub pool_sizes: Vec<usize>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            leak_detection: true,
            double_free_protection: true,
            buffer_overflow_protection: true,
            max_memory_usage: 0, // Unlimited
            use_memory_pools: true,
            pool_sizes: vec![32, 64, 128, 256, 512, 1024, 2048, 4096],
        }
    }
}

impl Default for FfiMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FfiMemoryManager {
    /// Create a new FFI memory manager
    pub fn new() -> Self {
        Self {
            allocations: RwLock::new(HashMap::new()),
            stats: RwLock::new(MemoryStats::default()),
            config: RwLock::new(MemoryConfig::default()),
            pools: RwLock::new(HashMap::new()),
        }
    }

    /// Configure the memory manager
    pub fn configure(&self, config: MemoryConfig) {
        let mut current_config = self.config.write().unwrap();
        *current_config = config.clone();
        
        // Initialize memory pools if enabled
        if config.use_memory_pools {
            let mut pools = self.pools.write().unwrap();
            for &size in &config.pool_sizes {
                let pool_name = format!("pool_{size}");
                pools.insert(pool_name.clone(), MemoryPool::new(pool_name, size, 16)); // 16 blocks per pool
            }
        }
    }

    /// Allocate memory with FFI-safe alignment
    pub fn allocate(&self, size: usize, c_type: Option<CType>) -> std::result::Result<NonNull<u8>, MemoryError> {
        let config = self.config.read().unwrap();
        
        // Check memory limit
        if config.max_memory_usage > 0 {
            let stats = self.stats.read().unwrap();
            if stats.current_usage + size > config.max_memory_usage {
                return Err(MemoryError::AllocationFailed { size, alignment: 1 });
            }
        }

        // Determine alignment based on type
        let alignment = if let Some(ref c_type) = c_type {
            c_type.alignment()
        } else {
            // Default to pointer alignment
            std::mem::align_of::<*const u8>()
        };

        // Try to use memory pool first
        if config.use_memory_pools {
            if let Some(ptr) = self.try_allocate_from_pool(size) {
                self.track_allocation(ptr, size, alignment, c_type)?;
                return Ok(ptr);
            }
        }

        // Fall back to system allocation
        let layout = Layout::from_size_align(size, alignment)
            .map_err(|_| MemoryError::AllocationFailed { size, alignment })?;

        let ptr = unsafe {
            let raw_ptr = alloc(layout);
            if raw_ptr.is_null() {
                return Err(MemoryError::AllocationFailed { size, alignment });
            }
            NonNull::new_unchecked(raw_ptr)
        };

        self.track_allocation(ptr, size, alignment, c_type)?;
        Ok(ptr)
    }

    /// Free allocated memory
    pub fn deallocate(&self, ptr: NonNull<u8>) -> std::result::Result<(), MemoryError> {
        let config = self.config.read().unwrap();
        
        // Check for double free
        if config.double_free_protection {
            let allocations = self.allocations.read().unwrap();
            if !allocations.contains_key(&(ptr.as_ptr() as *const u8)) {
                let mut stats = self.stats.write().unwrap();
                stats.double_frees_prevented += 1;
                return Err(MemoryError::DoubleFree(ptr.as_ptr()));
            }
        }

        // Get allocation info
        let allocation_info = {
            let mut allocations = self.allocations.write().unwrap();
            allocations.remove(&(ptr.as_ptr() as *const u8))
        };

        if let Some(info) = allocation_info {
            // Check if this should go back to a pool
            if config.use_memory_pools
                && self.try_return_to_pool(ptr, info.size) {
                    self.update_deallocation_stats(info.size);
                    return Ok(());
                }

            // System deallocation
            unsafe {
                dealloc(ptr.as_ptr(), info.layout);
            }

            self.update_deallocation_stats(info.size);
            Ok(())
        } else {
            Err(MemoryError::InvalidPointer(ptr.as_ptr()))
        }
    }

    /// Track an allocation
    fn track_allocation(&self, ptr: NonNull<u8>, size: usize, alignment: usize, c_type: Option<CType>) 
        -> std::result::Result<(), MemoryError> {
        
        let layout = Layout::from_size_align(size, alignment)
            .map_err(|_| MemoryError::AllocationFailed { size, alignment })?;
        
        let info = AllocationInfo::new(ptr, layout, c_type);
        
        // Check alignment
        if !info.is_aligned() {
            return Err(MemoryError::AlignmentError {
                ptr: ptr.as_ptr(),
                required_alignment: alignment,
                actual_alignment: ptr.as_ptr() as usize % alignment,
            });
        }

        // Track the allocation
        {
            let mut allocations = self.allocations.write().unwrap();
            allocations.insert(ptr.as_ptr(), info);
        }

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_allocated += size;
            stats.current_usage += size;
            stats.active_allocations += 1;
            stats.allocation_count += 1;
            
            if stats.current_usage > stats.peak_usage {
                stats.peak_usage = stats.current_usage;
            }
        }

        Ok(())
    }

    /// Update deallocation statistics
    fn update_deallocation_stats(&self, size: usize) {
        let mut stats = self.stats.write().unwrap();
        stats.total_freed += size;
        stats.current_usage = stats.current_usage.saturating_sub(size);
        stats.active_allocations = stats.active_allocations.saturating_sub(1);
        stats.deallocation_count += 1;
    }

    /// Try to allocate from a memory pool
    fn try_allocate_from_pool(&self, size: usize) -> Option<NonNull<u8>> {
        let pools = self.pools.read().unwrap();
        
        // Find the smallest pool that can accommodate the request
        for &pool_size in [32, 64, 128, 256, 512, 1024, 2048, 4096].iter() {
            if size <= pool_size {
                let pool_name = format!("pool_{pool_size}");
                if let Some(pool) = pools.get(&pool_name) {
                    if let Some(ptr) = pool.allocate() {
                        return Some(ptr);
                    }
                }
            }
        }
        
        None
    }

    /// Try to return memory to a pool
    fn try_return_to_pool(&self, ptr: NonNull<u8>, size: usize) -> bool {
        let pools = self.pools.read().unwrap();
        
        // Find the appropriate pool
        for &pool_size in [32, 64, 128, 256, 512, 1024, 2048, 4096].iter() {
            if size <= pool_size {
                let pool_name = format!("pool_{pool_size}");
                if let Some(pool) = pools.get(&pool_name) {
                    return pool.deallocate(ptr);
                }
            }
        }
        
        false
    }

    /// Check for memory leaks
    pub fn check_leaks(&self) -> Vec<MemoryError> {
        let config = self.config.read().unwrap();
        if !config.leak_detection {
            return vec![];
        }

        let allocations = self.allocations.read().unwrap();
        let now = SystemTime::now();
        let mut leaks = Vec::new();

        for (ptr, info) in allocations.iter() {
            // Consider memory leaked if it's been allocated for more than 5 minutes
            // and has no references
            if info.ref_count() == 0 {
                if let Ok(duration) = now.duration_since(info.allocated_at) {
                    if duration > Duration::from_secs(300) { // 5 minutes
                        leaks.push(MemoryError::LeakDetected {
                            ptr: *ptr,
                            size: info.size,
                            allocated_at: info.allocated_at,
                        });
                    }
                }
            }
        }

        // Update leak statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.leaks_detected += leaks.len() as u64;
        }

        leaks
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        self.stats.read().unwrap().clone()
    }

    /// Get allocation info for a pointer
    pub fn get_allocation_info(&self, ptr: *const u8) -> Option<AllocationInfo> {
        let allocations = self.allocations.read().unwrap();
        allocations.get(&ptr).cloned()
    }

    /// List all active allocations
    pub fn list_allocations(&self) -> Vec<(*const u8, AllocationInfo)> {
        let allocations = self.allocations.read().unwrap();
        allocations.iter().map(|(&ptr, info)| (ptr, info.clone())).collect()
    }

    /// Force cleanup of all allocations
    pub fn cleanup_all(&self) -> std::result::Result<(), Vec<MemoryError>> {
        let allocation_ptrs: Vec<*const u8> = {
            let allocations = self.allocations.read().unwrap();
            allocations.keys().cloned().collect()
        };

        let mut errors = Vec::new();
        for ptr in allocation_ptrs {
            if let Some(non_null_ptr) = NonNull::new(ptr as *mut u8) {
                if let Err(e) = self.deallocate(non_null_ptr) {
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Memory pool for efficient allocation of fixed-size blocks
#[derive(Debug)]
pub struct MemoryPool {
    /// Pool name
    name: String,
    /// Block size
    block_size: usize,
    /// Total blocks
    total_blocks: usize,
    /// Free blocks
    free_blocks: Mutex<Vec<NonNull<u8>>>,
    /// Pool memory
    pool_memory: NonNull<u8>,
    /// Pool layout
    layout: Layout,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(name: String, block_size: usize, block_count: usize) -> Self {
        let total_size = block_size * block_count;
        let alignment = std::mem::align_of::<*const u8>();
        let layout = Layout::from_size_align(total_size, alignment).unwrap();

        let pool_memory = unsafe {
            let raw_ptr = alloc(layout);
            assert!(!raw_ptr.is_null(), "Failed to allocate memory pool");
            NonNull::new_unchecked(raw_ptr)
        };

        // Initialize free block list
        let mut free_blocks = Vec::with_capacity(block_count);
        for i in 0..block_count {
            unsafe {
                let block_ptr = pool_memory.as_ptr().add(i * block_size);
                free_blocks.push(NonNull::new_unchecked(block_ptr));
            }
        }

        Self {
            name,
            block_size,
            total_blocks: block_count,
            free_blocks: Mutex::new(free_blocks),
            pool_memory,
            layout,
        }
    }

    /// Allocate a block from this pool
    pub fn allocate(&self) -> Option<NonNull<u8>> {
        let mut free_blocks = self.free_blocks.lock().unwrap();
        free_blocks.pop()
    }

    /// Return a block to this pool
    pub fn deallocate(&self, ptr: NonNull<u8>) -> bool {
        // Check if pointer belongs to this pool
        let pool_start = self.pool_memory.as_ptr() as usize;
        let pool_end = pool_start + self.layout.size();
        let ptr_addr = ptr.as_ptr() as usize;

        if ptr_addr >= pool_start && ptr_addr < pool_end {
            // Check if pointer is properly aligned to block boundary
            let offset = ptr_addr - pool_start;
            if offset % self.block_size == 0 {
                let mut free_blocks = self.free_blocks.lock().unwrap();
                free_blocks.push(ptr);
                return true;
            }
        }

        false
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let free_blocks = self.free_blocks.lock().unwrap();
        PoolStats {
            name: self.name.clone(),
            block_size: self.block_size,
            total_blocks: self.total_blocks,
            free_blocks: free_blocks.len(),
            used_blocks: self.total_blocks - free_blocks.len(),
        }
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.pool_memory.as_ptr(), self.layout);
        }
    }
}

/// Memory pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub name: String,
    pub block_size: usize,
    pub total_blocks: usize,
    pub free_blocks: usize,
    pub used_blocks: usize,
}

lazy_static::lazy_static! {
    /// Global FFI memory manager instance
    pub static ref GLOBAL_FFI_MEMORY_MANAGER: FfiMemoryManager = FfiMemoryManager::new();
}

/// Convenience functions for global memory manager
pub fn ffi_allocate(size: usize, c_type: Option<CType>) -> std::result::Result<NonNull<u8>, MemoryError> {
    GLOBAL_FFI_MEMORY_MANAGER.allocate(size, c_type)
}

pub fn ffi_deallocate(ptr: NonNull<u8>) -> std::result::Result<(), MemoryError> {
    GLOBAL_FFI_MEMORY_MANAGER.deallocate(ptr)
}

pub fn ffi_check_leaks() -> Vec<MemoryError> {
    GLOBAL_FFI_MEMORY_MANAGER.check_leaks()
}

pub fn ffi_memory_stats() -> MemoryStats {
    GLOBAL_FFI_MEMORY_MANAGER.stats()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_manager_creation() {
        let manager = FfiMemoryManager::new();
        let stats = manager.stats();
        assert_eq!(stats.current_usage, 0);
        assert_eq!(stats.active_allocations, 0);
    }

    #[test]
    fn test_basic_allocation() {
        let manager = FfiMemoryManager::new();
        let ptr = manager.allocate(64, None).unwrap();
        assert!(!ptr.as_ptr().is_null());
        
        let stats = manager.stats();
        assert_eq!(stats.active_allocations, 1);
        assert!(stats.current_usage >= 64);
        
        manager.deallocate(ptr).unwrap();
        
        let stats = manager.stats();
        assert_eq!(stats.active_allocations, 0);
    }

    #[test]
    fn test_double_free_protection() {
        let manager = FfiMemoryManager::new();
        let ptr = manager.allocate(64, None).unwrap();
        
        // First free should succeed
        manager.deallocate(ptr).unwrap();
        
        // Second free should fail
        let result = manager.deallocate(ptr);
        assert!(matches!(result, Err(MemoryError::DoubleFree(_))));
    }

    #[test]
    fn test_memory_pool() {
        let pool = MemoryPool::new("test_pool".to_string(), 64, 4);
        
        let ptr1 = pool.allocate().unwrap();
        let ptr2 = pool.allocate().unwrap();
        
        let stats = pool.stats();
        assert_eq!(stats.used_blocks, 2);
        assert_eq!(stats.free_blocks, 2);
        
        assert!(pool.deallocate(ptr1));
        assert!(pool.deallocate(ptr2));
        
        let stats = pool.stats();
        assert_eq!(stats.used_blocks, 0);
        assert_eq!(stats.free_blocks, 4);
    }

    #[test]
    fn test_allocation_info() {
        let manager = FfiMemoryManager::new();
        let ptr = manager.allocate(128, Some(CType::Int32)).unwrap();
        
        let info = manager.get_allocation_info(ptr.as_ptr()).unwrap();
        assert_eq!(info.size, 128);
        assert_eq!(info.c_type, Some(CType::Int32));
        assert!(info.is_aligned());
        
        manager.deallocate(ptr).unwrap();
    }

    #[test]
    fn test_memory_configuration() {
        let manager = FfiMemoryManager::new();
        let config = MemoryConfig {
            max_memory_usage: 1024,
            use_memory_pools: false,
            ..Default::default()
        };
        
        manager.configure(config);
        
        // Should be able to allocate within limit
        let ptr1 = manager.allocate(512, None).unwrap();
        
        // Should fail to allocate beyond limit
        let result = manager.allocate(600, None);
        assert!(matches!(result, Err(MemoryError::AllocationFailed { .. })));
        
        manager.deallocate(ptr1).unwrap();
    }
}

// Safety: FfiMemoryManager uses appropriate synchronization primitives for thread safety
unsafe impl Send for FfiMemoryManager {}
unsafe impl Sync for FfiMemoryManager {}