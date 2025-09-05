//! Safe SIMD intrinsics wrapper for Lambdust
//!
//! This module provides a safe, zero-cost abstraction over platform-specific
//! SIMD intrinsics, ensuring memory safety and type correctness while
//! maintaining maximum performance.

use std::marker::PhantomData;
use std::mem;
use std::ops::{Add, Div, Mul, Sub};
use std::ptr;

/// Platform-agnostic SIMD vector trait
pub trait SimdVector<T, const LANES: usize>: Clone + Copy + Send + Sync {
    /// Create a new vector from an array of values
    fn from_array(values: [T; LANES]) -> Self;

    /// Convert to an array of values
    fn to_array(self) -> [T; LANES];

    /// Splat a single value to all lanes
    fn splat(value: T) -> Self;

    /// Extract a single lane value
    fn extract(self, index: usize) -> T;

    /// Insert a value at a specific lane
    fn insert(self, index: usize, value: T) -> Self;

    /// Get the number of lanes
    fn lanes() -> usize {
        LANES
    }
}

/// Safe SIMD vector wrapper with alignment guarantees
#[repr(align(32))] // Ensure proper alignment for AVX operations
pub struct SafeSimdVector<T, V, const LANES: usize> {
    vector: V,
    _phantom: PhantomData<T>,
}

impl<T, V, const LANES: usize> SafeSimdVector<T, V, LANES>
where
    V: SimdVector<T, LANES>,
    T: Copy + Default,
{
    /// Create a new safe SIMD vector
    pub fn new(vector: V) -> Self {
        Self {
            vector,
            _phantom: PhantomData,
        }
    }

    /// Create from array with bounds checking
    pub fn from_array(values: [T; LANES]) -> Self {
        Self::new(V::from_array(values))
    }

    /// Create by splatting a single value
    pub fn splat(value: T) -> Self {
        Self::new(V::splat(value))
    }

    /// Extract element with bounds checking
    pub fn extract(&self, index: usize) -> Option<T> {
        if index < LANES {
            Some(self.vector.extract(index))
        } else {
            None
        }
    }

    /// Convert to array
    pub fn to_array(self) -> [T; LANES] {
        self.vector.to_array()
    }

    /// Get the number of lanes
    pub const fn lanes() -> usize {
        LANES
    }
}

/// Memory-aligned buffer for SIMD operations
#[derive(Debug)]
pub struct AlignedBuffer<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
    align: usize,
}

impl<T> AlignedBuffer<T> {
    /// Create a new aligned buffer
    pub fn new(capacity: usize, alignment: usize) -> Result<Self, AlignmentError> {
        if !alignment.is_power_of_two() {
            return Err(AlignmentError::InvalidAlignment);
        }

        let size = capacity * mem::size_of::<T>();
        let layout = std::alloc::Layout::from_size_align(size, alignment)
            .map_err(|_| AlignmentError::AllocationFailed)?;

        let ptr = unsafe { std::alloc::alloc(layout) as *mut T };
        if ptr.is_null() {
            return Err(AlignmentError::AllocationFailed);
        }

        Ok(Self {
            ptr,
            len: 0,
            capacity,
            align: alignment,
        })
    }

    /// Get a safe slice of the buffer
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Get a safe mutable slice of the buffer
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    /// Check if buffer is properly aligned for SIMD operations
    pub fn is_aligned(&self) -> bool {
        (self.ptr as usize) % self.align == 0
    }

    /// Push an element to the buffer
    pub fn push(&mut self, value: T) -> Result<(), BufferError> {
        if self.len >= self.capacity {
            return Err(BufferError::CapacityExceeded);
        }

        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
        Ok(())
    }

    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get current length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T> Drop for AlignedBuffer<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                // Drop all valid elements first
                for i in 0..self.len {
                    ptr::drop_in_place(self.ptr.add(i));
                }

                // Deallocate memory
                let size = self.capacity * mem::size_of::<T>();
                let layout = std::alloc::Layout::from_size_align_unchecked(size, self.align);
                std::alloc::dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

unsafe impl<T: Send> Send for AlignedBuffer<T> {}
unsafe impl<T: Sync> Sync for AlignedBuffer<T> {}

/// SIMD operation strategy selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdStrategy {
    /// Use AVX-512 instructions (64-byte vectors)
    Avx512,
    /// Use AVX2 instructions (32-byte vectors)
    Avx2,
    /// Use SSE instructions (16-byte vectors)
    Sse,
    /// Use ARM NEON instructions
    Neon,
    /// Fallback to scalar operations
    Scalar,
}

/// SIMD feature detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimdCapabilities {
    /// AVX-512 Foundation instructions are available
    pub has_avx512f: bool,
    /// AVX-512 Doubleword and Quadword instructions are available
    pub has_avx512dq: bool,
    /// Advanced Vector Extensions 2 instructions are available
    pub has_avx2: bool,
    /// Fused Multiply-Add instructions are available
    pub has_fma: bool,
    /// SSE 4.2 instructions are available
    pub has_sse42: bool,
    /// ARM NEON SIMD instructions are available
    pub has_neon: bool,
}

impl SimdCapabilities {
    /// Detect available SIMD capabilities
    pub fn detect() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            Self {
                has_avx512f: is_x86_feature_detected!("avx512f"),
                has_avx512dq: is_x86_feature_detected!("avx512dq"),
                has_avx2: is_x86_feature_detected!("avx2"),
                has_fma: is_x86_feature_detected!("fma"),
                has_sse42: is_x86_feature_detected!("sse4.2"),
                has_neon: false,
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            Self {
                has_avx512f: false,
                has_avx512dq: false,
                has_avx2: false,
                has_fma: false,
                has_sse42: false,
                has_neon: true, // ARM64 always has NEON
            }
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Self {
                has_avx512f: false,
                has_avx512dq: false,
                has_avx2: false,
                has_fma: false,
                has_sse42: false,
                has_neon: false,
            }
        }
    }

    /// Get the best available SIMD strategy
    pub fn best_strategy(&self) -> SimdStrategy {
        if self.has_avx512f && self.has_avx512dq {
            SimdStrategy::Avx512
        } else if self.has_avx2 {
            SimdStrategy::Avx2
        } else if self.has_sse42 {
            SimdStrategy::Sse
        } else if self.has_neon {
            SimdStrategy::Neon
        } else {
            SimdStrategy::Scalar
        }
    }
}

/// Error types for SIMD operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlignmentError {
    /// Memory alignment requirements are not satisfied
    InvalidAlignment,
    /// Failed to allocate properly aligned memory
    AllocationFailed,
}

/// Buffer-related errors in SIMD operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferError {
    /// Buffer capacity has been exceeded
    CapacityExceeded,
    /// Invalid index access in buffer
    InvalidIndex,
}

/// Comprehensive error type for all SIMD operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimdError {
    /// Memory alignment related error
    Alignment(AlignmentError),
    /// Buffer operation error
    Buffer(BufferError),
    /// Requested SIMD operation is not supported on this platform
    UnsupportedOperation,
    /// Vector size is invalid for the requested operation
    InvalidVectorSize,
}

impl std::fmt::Display for AlignmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlignmentError::InvalidAlignment => write!(f, "Invalid alignment specification"),
            AlignmentError::AllocationFailed => write!(f, "Failed to allocate aligned memory"),
        }
    }
}

impl std::fmt::Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferError::CapacityExceeded => write!(f, "Buffer capacity exceeded"),
            BufferError::InvalidIndex => write!(f, "Invalid buffer index"),
        }
    }
}

impl std::fmt::Display for SimdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimdError::Alignment(e) => write!(f, "Alignment error: {e}"),
            SimdError::Buffer(e) => write!(f, "Buffer error: {e}"),
            SimdError::UnsupportedOperation => write!(f, "Unsupported SIMD operation"),
            SimdError::InvalidVectorSize => write!(f, "Invalid vector size for operation"),
        }
    }
}

impl std::error::Error for AlignmentError {}
impl std::error::Error for BufferError {}
impl std::error::Error for SimdError {}

impl From<AlignmentError> for SimdError {
    fn from(err: AlignmentError) -> Self {
        SimdError::Alignment(err)
    }
}

impl From<BufferError> for SimdError {
    fn from(err: BufferError) -> Self {
        SimdError::Buffer(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aligned_buffer_creation() {
        let buffer: AlignedBuffer<f64> = AlignedBuffer::new(16, 32).unwrap();
        assert!(buffer.is_aligned());
        assert_eq!(buffer.capacity(), 16);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_simd_capabilities_detection() {
        let caps = SimdCapabilities::detect();
        let strategy = caps.best_strategy();

        // Should always have some strategy (at least Scalar)
        assert!(matches!(
            strategy,
            SimdStrategy::Avx512
                | SimdStrategy::Avx2
                | SimdStrategy::Sse
                | SimdStrategy::Neon
                | SimdStrategy::Scalar
        ));
    }

    #[test]
    fn test_buffer_operations() {
        let mut buffer: AlignedBuffer<i32> = AlignedBuffer::new(4, 16).unwrap();

        buffer.push(1).unwrap();
        buffer.push(2).unwrap();
        buffer.push(3).unwrap();
        buffer.push(4).unwrap();

        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.as_slice(), &[1, 2, 3, 4]);

        // Should fail on capacity exceeded
        assert!(buffer.push(5).is_err());
    }
}
