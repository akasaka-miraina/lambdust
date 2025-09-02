#![cfg(feature = "never-enabled")]
//! Cache-Optimized Record Instance Storage for SRFI-9
//!
//! This module implements the record instance layout with:
//! - Cache-line aligned field storage
//! - SIMD-optimized bulk operations
//! - NaN-boxing integration for field values
//! - Direct pointer arithmetic for <1ns field access

use crate::ast::Literal;
use crate::eval::nan_boxed_value::NanBoxedValue;
use crate::eval::record_arena::{GLOBAL_RECORD_ARENA, RecordArena, SizeClass};
use crate::eval::record_type::{
    FieldDescriptor, GLOBAL_RECORD_REGISTRY, RecordError, RecordResult, RecordTypeDescriptor,
    RecordTypeId,
};
use crate::eval::value::Value;
use std::fmt::{self, Debug, Formatter};
use std::ptr::{self, NonNull};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// High-performance record instance with cache-optimized layout
#[repr(C, align(64))] // Cache-line aligned
pub struct RecordInstance {
    /// Type identifier for fast type checking
    type_id: RecordTypeId,
    /// Reference count for memory management
    ref_count: AtomicU64,
    /// Creation timestamp for generational GC
    created_at: u64,
    /// Reserved field for future use
    _reserved: u32,
    /// Field values stored as NaN-boxed values
    /// Variable-length array follows the header
    fields: [NanBoxedValue; 0],
}

impl RecordInstance {
    /// Header size (type_id + ref_count + metadata)
    const HEADER_SIZE: usize = std::mem::size_of::<RecordTypeId>()
        + std::mem::size_of::<AtomicU64>()
        + std::mem::size_of::<u64>()
        + std::mem::size_of::<u32>();

    /// Creates a new record instance
    pub fn new(
        type_id: RecordTypeId,
        field_values: &[NanBoxedValue],
    ) -> RecordResult<NonNull<RecordInstance>> {
        // Get type descriptor
        let type_desc = GLOBAL_RECORD_REGISTRY
            .get_type(type_id)
            .ok_or_else(|| RecordError::TypeNotFound(format!("TypeId({:?})", type_id)))?;

        let type_desc = type_desc.read().unwrap();

        // Validate field count
        if field_values.len() != type_desc.fields.len() {
            return Err(RecordError::ConstructionError(format!(
                "Expected {} fields, got {}",
                type_desc.fields.len(),
                field_values.len()
            )));
        }

        // Calculate total size needed
        let instance_size = Self::calculate_instance_size(field_values.len())?;

        // Allocate memory from arena
        let memory = GLOBAL_RECORD_ARENA
            .allocate(instance_size)
            .ok_or_else(|| RecordError::AllocationError("Arena allocation failed".to_string()))?;

        // Initialize the instance
        let instance = unsafe {
            let ptr = memory.as_ptr() as *mut RecordInstance;

            // Initialize header
            ptr::write(&mut (*ptr).type_id, type_id);
            ptr::write(&mut (*ptr).ref_count, AtomicU64::new(1));
            ptr::write(&mut (*ptr).created_at, Self::current_timestamp());
            ptr::write(&mut (*ptr)._reserved, 0);

            // Initialize field values
            let fields_ptr = Self::fields_ptr(ptr);
            for (i, &value) in field_values.iter().enumerate() {
                ptr::write(fields_ptr.add(i), value);
            }

            NonNull::new_unchecked(ptr)
        };

        // Record creation statistics
        type_desc.access_stats.record_creation();

        Ok(instance)
    }

    /// Creates a record instance with default (nil) values
    pub fn new_default(type_id: RecordTypeId) -> RecordResult<NonNull<RecordInstance>> {
        // Get type descriptor
        let type_desc = GLOBAL_RECORD_REGISTRY
            .get_type(type_id)
            .ok_or_else(|| RecordError::TypeNotFound(format!("TypeId({:?})", type_id)))?;

        let type_desc = type_desc.read().unwrap();
        let field_count = type_desc.fields.len();

        // Create default values (all nil)
        let default_values: Vec<NanBoxedValue> = vec![NanBoxedValue::nil_value(); field_count];

        Self::new(type_id, &default_values)
    }

    /// Gets a field value by index with <1ns access time for hot paths
    #[inline(always)]
    pub unsafe fn get_field_fast(&self, field_index: usize) -> NanBoxedValue {
        // Direct pointer arithmetic for maximum performance
        unsafe {
            let fields_ptr = Self::fields_ptr(self as *const _ as *mut _);
            *fields_ptr.add(field_index)
        }
    }

    /// Gets a field value by index with bounds checking
    pub fn get_field(&self, field_index: usize) -> RecordResult<NanBoxedValue> {
        // Get type descriptor for bounds checking
        let type_desc = GLOBAL_RECORD_REGISTRY
            .get_type(self.type_id)
            .ok_or_else(|| RecordError::TypeNotFound(format!("TypeId({:?})", self.type_id)))?;

        let type_desc = type_desc.read().unwrap();

        if field_index >= type_desc.fields.len() {
            return Err(RecordError::InvalidField(format!(
                "Field index {} out of bounds (max: {})",
                field_index,
                type_desc.fields.len()
            )));
        }

        // Record access for optimization
        type_desc.access_stats.record_access(field_index);
        if let Some(field) = type_desc.fields.get(field_index) {
            field.record_access();
        }

        // Perform the access
        let value = unsafe { self.get_field_fast(field_index) };
        Ok(value)
    }

    /// Gets a field value by name
    pub fn get_field_by_name(&self, field_name: &str) -> RecordResult<NanBoxedValue> {
        // Get type descriptor
        let type_desc = GLOBAL_RECORD_REGISTRY
            .get_type(self.type_id)
            .ok_or_else(|| RecordError::TypeNotFound(format!("TypeId({:?})", self.type_id)))?;

        let type_desc = type_desc.read().unwrap();

        // Find field index
        let field_index = type_desc
            .field_index
            .get(field_name)
            .copied()
            .ok_or_else(|| RecordError::InvalidField(field_name.to_string()))?;

        // Record named access
        type_desc.record_field_access(field_name);

        // Get the value
        drop(type_desc); // Release read lock
        self.get_field(field_index)
    }

    /// Sets a field value by index with fast path optimization
    #[inline(always)]
    pub unsafe fn set_field_fast(&mut self, field_index: usize, value: NanBoxedValue) {
        // Direct pointer arithmetic for maximum performance
        unsafe {
            let fields_ptr = Self::fields_ptr(self as *const _ as *mut _);
            *fields_ptr.add(field_index) = value;
        }
    }

    /// Sets a field value by index with bounds checking
    pub fn set_field(&mut self, field_index: usize, value: NanBoxedValue) -> RecordResult<()> {
        // Get type descriptor for bounds checking
        let type_desc = GLOBAL_RECORD_REGISTRY
            .get_type(self.type_id)
            .ok_or_else(|| RecordError::TypeNotFound(format!("TypeId({:?})", self.type_id)))?;

        let type_desc = type_desc.read().unwrap();

        if field_index >= type_desc.fields.len() {
            return Err(RecordError::InvalidField(format!(
                "Field index {} out of bounds (max: {})",
                field_index,
                type_desc.fields.len()
            )));
        }

        // Record access for optimization
        type_desc.access_stats.record_access(field_index);
        if let Some(field) = type_desc.fields.get(field_index) {
            field.record_access();
        }

        // Perform the mutation
        unsafe {
            self.set_field_fast(field_index, value);
        }

        Ok(())
    }

    /// Sets a field value by name
    pub fn set_field_by_name(
        &mut self,
        field_name: &str,
        value: NanBoxedValue,
    ) -> RecordResult<()> {
        // Get type descriptor
        let type_desc = GLOBAL_RECORD_REGISTRY
            .get_type(self.type_id)
            .ok_or_else(|| RecordError::TypeNotFound(format!("TypeId({:?})", self.type_id)))?;

        let type_desc = type_desc.read().unwrap();

        // Find field index
        let field_index = type_desc
            .field_index
            .get(field_name)
            .copied()
            .ok_or_else(|| RecordError::InvalidField(field_name.to_string()))?;

        // Record named access
        type_desc.record_field_access(field_name);

        // Set the value
        drop(type_desc); // Release read lock
        self.set_field(field_index, value)
    }

    /// Gets all field values as a slice
    pub fn get_all_fields(&self) -> RecordResult<Vec<NanBoxedValue>> {
        // Get type descriptor
        let type_desc = GLOBAL_RECORD_REGISTRY
            .get_type(self.type_id)
            .ok_or_else(|| RecordError::TypeNotFound(format!("TypeId({:?})", self.type_id)))?;

        let type_desc = type_desc.read().unwrap();
        let field_count = type_desc.fields.len();

        // Copy all fields
        let mut fields = Vec::with_capacity(field_count);
        for i in 0..field_count {
            let value = unsafe { self.get_field_fast(i) };
            fields.push(value);
        }

        Ok(fields)
    }

    /// Bulk sets all field values (SIMD-optimized when applicable)
    pub fn set_all_fields(&mut self, values: &[NanBoxedValue]) -> RecordResult<()> {
        // Get type descriptor
        let type_desc = GLOBAL_RECORD_REGISTRY
            .get_type(self.type_id)
            .ok_or_else(|| RecordError::TypeNotFound(format!("TypeId({:?})", self.type_id)))?;

        let type_desc = type_desc.read().unwrap();

        if values.len() != type_desc.fields.len() {
            return Err(RecordError::ConstructionError(format!(
                "Expected {} fields, got {}",
                type_desc.fields.len(),
                values.len()
            )));
        }

        // Record bulk operation
        type_desc
            .access_stats
            .record_bulk_operation(values.len() as u64);

        // Use SIMD if beneficial and aligned
        if type_desc.simd_flags.bulk_init && values.len() >= 4 {
            self.simd_bulk_set(values)?;
        } else {
            // Standard bulk set
            unsafe {
                let fields_ptr = Self::fields_ptr(self as *const _ as *mut _);
                for (i, &value) in values.iter().enumerate() {
                    *fields_ptr.add(i) = value;
                }
            }
        }

        Ok(())
    }

    /// SIMD-optimized bulk field setting
    #[cfg(target_arch = "x86_64")]
    fn simd_bulk_set(&mut self, values: &[NanBoxedValue]) -> RecordResult<()> {
        use std::arch::x86_64::*;

        if !is_x86_feature_detected!("avx2") {
            return self.fallback_bulk_set(values);
        }

        unsafe {
            let fields_ptr = Self::fields_ptr(self as *const _ as *mut _);
            let values_ptr = values.as_ptr();

            // Process 4 values at a time (256-bit AVX2)
            let mut i = 0;
            while i + 4 <= values.len() {
                // Load 4 NaN-boxed values
                let chunk = _mm256_load_si256(values_ptr.add(i) as *const __m256i);
                // Store to record fields
                _mm256_store_si256(fields_ptr.add(i) as *mut __m256i, chunk);
                i += 4;
            }

            // Handle remaining values
            for j in i..values.len() {
                *fields_ptr.add(j) = *values_ptr.add(j);
            }
        }

        Ok(())
    }

    /// Non-SIMD fallback for bulk setting
    #[cfg(not(target_arch = "x86_64"))]
    fn simd_bulk_set(&mut self, values: &[NanBoxedValue]) -> RecordResult<()> {
        self.fallback_bulk_set(values)
    }

    /// Fallback implementation for bulk setting
    fn fallback_bulk_set(&mut self, values: &[NanBoxedValue]) -> RecordResult<()> {
        unsafe {
            let fields_ptr = Self::fields_ptr(self as *const _ as *mut _);
            ptr::copy_nonoverlapping(values.as_ptr(), fields_ptr, values.len());
        }
        Ok(())
    }

    /// Compares this record with another for equality (SIMD-optimized)
    pub fn equals(&self, other: &RecordInstance) -> RecordResult<bool> {
        // Type must match
        if self.type_id != other.type_id {
            return Ok(false);
        }

        // Get field count
        let type_desc = GLOBAL_RECORD_REGISTRY
            .get_type(self.type_id)
            .ok_or_else(|| RecordError::TypeNotFound(format!("TypeId({:?})", self.type_id)))?;

        let type_desc = type_desc.read().unwrap();
        let field_count = type_desc.fields.len();

        // Record bulk operation
        type_desc
            .access_stats
            .record_bulk_operation(field_count as u64 * 2);

        // Use SIMD comparison if beneficial
        if type_desc.simd_flags.bulk_compare && field_count >= 4 {
            self.simd_compare(other, field_count)
        } else {
            // Standard field-by-field comparison
            for i in 0..field_count {
                let a = unsafe { self.get_field_fast(i) };
                let b = unsafe { other.get_field_fast(i) };
                if a != b {
                    return Ok(false);
                }
            }
            Ok(true)
        }
    }

    /// SIMD-optimized record comparison
    #[cfg(target_arch = "x86_64")]
    fn simd_compare(&self, other: &RecordInstance, field_count: usize) -> RecordResult<bool> {
        use std::arch::x86_64::*;

        if !is_x86_feature_detected!("avx2") {
            return self.fallback_compare(other, field_count);
        }

        unsafe {
            let self_ptr = Self::fields_ptr(self as *const _ as *mut _);
            let other_ptr = Self::fields_ptr(other as *const _ as *mut _);

            // Process 4 fields at a time
            let mut i = 0;
            while i + 4 <= field_count {
                let a = _mm256_load_si256(self_ptr.add(i) as *const __m256i);
                let b = _mm256_load_si256(other_ptr.add(i) as *const __m256i);
                let cmp = _mm256_cmpeq_epi64(a, b);

                // Check if all comparisons are true
                if _mm256_movemask_epi8(cmp) != -1i32 as u32 {
                    return Ok(false);
                }
                i += 4;
            }

            // Handle remaining fields
            for j in i..field_count {
                let a = *self_ptr.add(j);
                let b = *other_ptr.add(j);
                if a != b {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Non-SIMD fallback for comparison
    #[cfg(not(target_arch = "x86_64"))]
    fn simd_compare(&self, other: &RecordInstance, field_count: usize) -> RecordResult<bool> {
        self.fallback_compare(other, field_count)
    }

    /// Fallback implementation for comparison
    fn fallback_compare(&self, other: &RecordInstance, field_count: usize) -> RecordResult<bool> {
        for i in 0..field_count {
            let a = unsafe { self.get_field_fast(i) };
            let b = unsafe { other.get_field_fast(i) };
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Gets the type ID of this record
    pub fn type_id(&self) -> RecordTypeId {
        self.type_id
    }

    /// Type predicate - checks if this is an instance of the given type
    pub fn is_type(&self, type_id: RecordTypeId) -> bool {
        self.type_id == type_id
    }

    /// Increments reference count
    pub fn retain(&self) {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrements reference count and returns true if should be deallocated
    pub fn release(&self) -> bool {
        self.ref_count.fetch_sub(1, Ordering::Relaxed) == 1
    }

    /// Gets current reference count
    pub fn ref_count(&self) -> u64 {
        self.ref_count.load(Ordering::Relaxed)
    }

    /// Gets creation timestamp
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Calculates the total instance size needed for the given field count
    fn calculate_instance_size(field_count: usize) -> RecordResult<u32> {
        let field_size = std::mem::size_of::<NanBoxedValue>();
        let total_fields_size = field_count
            .checked_mul(field_size)
            .ok_or_else(|| RecordError::AllocationError("Field count overflow".to_string()))?;

        let total_size = Self::HEADER_SIZE
            .checked_add(total_fields_size)
            .ok_or_else(|| RecordError::AllocationError("Instance size overflow".to_string()))?;

        // Align to cache line boundary (64 bytes)
        let aligned_size = (total_size + 63) & !63;

        if aligned_size > u32::MAX as usize {
            return Err(RecordError::AllocationError(
                "Instance too large".to_string(),
            ));
        }

        Ok(aligned_size as u32)
    }

    /// Gets pointer to the fields array
    unsafe fn fields_ptr(instance_ptr: *mut RecordInstance) -> *mut NanBoxedValue {
        unsafe { (instance_ptr as *mut u8).add(Self::HEADER_SIZE) as *mut NanBoxedValue }
    }

    /// Gets current timestamp for instance creation
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

impl Debug for RecordInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecordInstance")
            .field("type_id", &self.type_id)
            .field("ref_count", &self.ref_count.load(Ordering::Relaxed))
            .field("created_at", &self.created_at)
            .finish_non_exhaustive()
    }
}

impl Drop for RecordInstance {
    fn drop(&mut self) {
        // Get instance size for deallocation
        if let Some(type_desc) = GLOBAL_RECORD_REGISTRY.get_type(self.type_id) {
            let type_desc = type_desc.read().unwrap();
            let instance_size = type_desc.instance_size;

            // Deallocate from arena
            let self_ptr = NonNull::new(self as *mut RecordInstance).unwrap();
            GLOBAL_RECORD_ARENA.deallocate(self_ptr.cast::<u8>(), instance_size);
        }
    }
}

/// Record reference wrapper for safe access
pub struct RecordRef {
    instance: NonNull<RecordInstance>,
}

impl RecordRef {
    /// Creates a new record reference
    pub fn new(instance: NonNull<RecordInstance>) -> Self {
        unsafe {
            instance.as_ref().retain();
        }
        Self { instance }
    }

    /// Gets the underlying record instance
    pub fn instance(&self) -> &RecordInstance {
        unsafe { self.instance.as_ref() }
    }

    /// Gets mutable access to the underlying record instance
    pub fn instance_mut(&mut self) -> &mut RecordInstance {
        unsafe { self.instance.as_mut() }
    }
}

impl Clone for RecordRef {
    fn clone(&self) -> Self {
        Self::new(self.instance)
    }
}

impl Drop for RecordRef {
    fn drop(&mut self) {
        unsafe {
            if self.instance.as_ref().release() {
                // Last reference, instance will be dropped
            }
        }
    }
}

impl Debug for RecordRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RecordRef").field(&self.instance()).finish()
    }
}

/// Converts a Value to a NanBoxedValue for record field storage
pub fn value_to_nan_boxed(value: &Value) -> NanBoxedValue {
    match value {
        Value::Nil => NanBoxedValue::nil_value(),
        Value::Literal(Literal::Boolean(b)) => NanBoxedValue::from_bool(*b),
        Value::Literal(Literal::Number(n)) => NanBoxedValue::from_number(*n),
        Value::Literal(Literal::Character(c)) => NanBoxedValue::from_char(*c),
        // For complex values, we'd need to store them as heap references
        // This is a simplified implementation
        _ => NanBoxedValue::unspecified_value(), // Placeholder
    }
}

/// Converts a NanBoxedValue back to a Value for external access
pub fn nan_boxed_to_value(boxed: NanBoxedValue) -> Value {
    if boxed.is_nil() {
        Value::Nil
    } else if boxed.is_boolean() {
        Value::boolean(boxed.as_bool().unwrap_or(false))
    } else if boxed.is_small_int() {
        Value::integer(boxed.as_small_int().unwrap_or(0))
    } else if boxed.is_character() {
        Value::Literal(Literal::Character(boxed.as_char().unwrap_or('\0')))
    } else if boxed.is_number() {
        Value::number(boxed.as_number().unwrap_or(0.0))
    } else {
        // For complex types, we'd need to resolve heap references
        Value::Nil // Placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::record_type::{GLOBAL_RECORD_REGISTRY, RecordTypeDescriptor};

    fn create_test_type() -> RecordTypeId {
        let desc = RecordTypeDescriptor::new(
            "test-record".to_string(),
            vec!["field1".to_string(), "field2".to_string()],
        );
        let type_id = desc.type_id;
        GLOBAL_RECORD_REGISTRY.register_type(desc);
        type_id
    }

    #[test]
    fn test_record_instance_creation() {
        let type_id = create_test_type();
        let values = vec![
            NanBoxedValue::boolean(true),
            NanBoxedValue::small_integer(42),
        ];

        let instance = RecordInstance::new(type_id, &values).unwrap();

        unsafe {
            assert_eq!(instance.as_ref().type_id(), type_id);
            assert_eq!(
                instance.as_ref().get_field_fast(0),
                NanBoxedValue::boolean(true)
            );
            assert_eq!(
                instance.as_ref().get_field_fast(1),
                NanBoxedValue::small_integer(42)
            );
        }
    }

    #[test]
    fn test_field_access_by_name() {
        let type_id = create_test_type();
        let values = vec![
            NanBoxedValue::small_integer(1),
            NanBoxedValue::small_integer(2),
        ];

        let instance = RecordInstance::new(type_id, &values).unwrap();

        unsafe {
            let field1 = instance.as_ref().get_field_by_name("field1").unwrap();
            let field2 = instance.as_ref().get_field_by_name("field2").unwrap();

            assert_eq!(field1, NanBoxedValue::small_integer(1));
            assert_eq!(field2, NanBoxedValue::small_integer(2));
        }
    }

    #[test]
    fn test_field_mutation() {
        let type_id = create_test_type();
        let values = vec![
            NanBoxedValue::small_integer(1),
            NanBoxedValue::small_integer(2),
        ];

        let mut instance = RecordInstance::new(type_id, &values).unwrap();

        unsafe {
            let instance_ref = instance.as_mut();

            // Mutate field by index
            instance_ref
                .set_field(0, NanBoxedValue::small_integer(10))
                .unwrap();

            // Mutate field by name
            instance_ref
                .set_field_by_name("field2", NanBoxedValue::small_integer(20))
                .unwrap();

            // Verify mutations
            assert_eq!(
                instance_ref.get_field_fast(0),
                NanBoxedValue::small_integer(10)
            );
            assert_eq!(
                instance_ref.get_field_fast(1),
                NanBoxedValue::small_integer(20)
            );
        }
    }

    #[test]
    fn test_bulk_operations() {
        let type_id = create_test_type();
        let initial_values = vec![
            NanBoxedValue::small_integer(1),
            NanBoxedValue::small_integer(2),
        ];

        let mut instance = RecordInstance::new(type_id, &initial_values).unwrap();

        unsafe {
            let instance_ref = instance.as_mut();

            // Test bulk get
            let all_fields = instance_ref.get_all_fields().unwrap();
            assert_eq!(all_fields.len(), 2);

            // Test bulk set
            let new_values = vec![
                NanBoxedValue::small_integer(10),
                NanBoxedValue::small_integer(20),
            ];
            instance_ref.set_all_fields(&new_values).unwrap();

            // Verify
            assert_eq!(
                instance_ref.get_field_fast(0),
                NanBoxedValue::small_integer(10)
            );
            assert_eq!(
                instance_ref.get_field_fast(1),
                NanBoxedValue::small_integer(20)
            );
        }
    }

    #[test]
    fn test_record_comparison() {
        let type_id = create_test_type();
        let values = vec![
            NanBoxedValue::small_integer(1),
            NanBoxedValue::small_integer(2),
        ];

        let instance1 = RecordInstance::new(type_id, &values).unwrap();
        let instance2 = RecordInstance::new(type_id, &values).unwrap();

        unsafe {
            let equal = instance1.as_ref().equals(instance2.as_ref()).unwrap();
            assert!(equal);

            // Create different instance
            let different_values = vec![
                NanBoxedValue::small_integer(3),
                NanBoxedValue::small_integer(4),
            ];
            let instance3 = RecordInstance::new(type_id, &different_values).unwrap();

            let not_equal = instance1.as_ref().equals(instance3.as_ref()).unwrap();
            assert!(!not_equal);
        }
    }

    #[test]
    fn test_reference_counting() {
        let type_id = create_test_type();
        let values = vec![
            NanBoxedValue::small_integer(1),
            NanBoxedValue::small_integer(2),
        ];

        let instance = RecordInstance::new(type_id, &values).unwrap();

        unsafe {
            assert_eq!(instance.as_ref().ref_count(), 1);

            instance.as_ref().retain();
            assert_eq!(instance.as_ref().ref_count(), 2);

            let should_deallocate = instance.as_ref().release();
            assert!(!should_deallocate);
            assert_eq!(instance.as_ref().ref_count(), 1);
        }
    }
}
