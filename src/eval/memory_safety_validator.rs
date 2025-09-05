//! Memory safety validation system for OptimizedValue and other critical components
//!
//! This module provides comprehensive validation for memory operations, null pointer
//! checks, alignment verification, and cross-platform safety guarantees.

use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;

use crate::debug::sigsegv_monitor::{self, record_global_event};
use crate::debug::CrashInfo;

/// Memory safety validation configuration
pub struct MemorySafetyConfig {
    /// Enable null pointer dereference checks
    pub enable_null_checks: bool,
    /// Enable memory alignment checks
    pub enable_alignment_checks: bool,
    /// Enable buffer bounds checks
    pub enable_bounds_checks: bool,
    /// Enable use-after-free detection
    pub enable_use_after_free_detection: bool,
    /// Panic on memory safety violations
    pub panic_on_violation: bool,
    /// Log memory safety violations
    pub log_violations: bool,
}

impl Default for MemorySafetyConfig {
    fn default() -> Self {
        Self {
            enable_null_checks: true,
            enable_alignment_checks: true,
            enable_bounds_checks: true,
            enable_use_after_free_detection: cfg!(debug_assertions),
            panic_on_violation: cfg!(debug_assertions),
            log_violations: true,
        }
    }
}

/// Global memory safety validator
static mut VALIDATOR: Option<MemorySafetyValidator> = None;
static INIT_ONCE: std::sync::Once = std::sync::Once::new();

/// Memory safety violation types
#[derive(Debug, Clone, PartialEq)]
pub enum SafetyViolation {
    /// Null pointer dereference detected
    NullPointerDereference {
        /// Memory address that was accessed
        address: usize,
        /// Function where violation occurred
        function: &'static str,
        /// Line number where violation occurred
        line: u32,
    },
    /// Unaligned memory access detected
    UnalignedAccess {
        /// Memory address that was accessed
        address: usize,
        /// Required memory alignment
        required_alignment: usize,
        /// Actual memory alignment
        actual_alignment: usize,
        /// Function where violation occurred
        function: &'static str,
        /// Line number where violation occurred
        line: u32,
    },
    /// Use after free detected
    UseAfterFree {
        /// Memory address that was accessed
        address: usize,
        /// Original size of the allocation
        original_size: usize,
        /// Function where violation occurred
        function: &'static str,
        /// Line number where violation occurred
        line: u32,
    },
    /// Buffer overflow detected
    BufferOverflow {
        /// Memory address that was accessed
        address: usize,
        /// Start address of the buffer
        buffer_start: usize,
        /// Size of the buffer
        buffer_size: usize,
        /// Offset of the access from buffer start
        access_offset: isize,
        /// Function where violation occurred
        function: &'static str,
        /// Line number where violation occurred
        line: u32,
    },
    /// Double free detected
    DoubleFree {
        /// Memory address that was freed
        address: usize,
        /// Function where violation occurred
        function: &'static str,
        /// Line number where violation occurred
        line: u32,
    },
}

/// Memory safety validator
pub struct MemorySafetyValidator {
    config: MemorySafetyConfig,
    violation_count: AtomicUsize,
    allocation_tracker: std::sync::Mutex<HashMap<usize, AllocationRecord>>,
}

#[derive(Debug, Clone)]
struct AllocationRecord {
    size: usize,
    alignment: usize,
    allocated_at: u64,
    freed_at: Option<u64>,
    allocation_source: String,
}

impl MemorySafetyValidator {
    /// Create a new memory safety validator with the given configuration
    pub fn new(config: MemorySafetyConfig) -> Self {
        Self {
            config,
            violation_count: AtomicUsize::new(0),
            allocation_tracker: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Validate a pointer before dereferencing
    pub fn validate_ptr_deref<T>(&self, ptr: *const T, function: &'static str, line: u32) -> Result<(), SafetyViolation> {
        // Null pointer check
        if self.config.enable_null_checks && ptr.is_null() {
            let violation = SafetyViolation::NullPointerDereference {
                address: ptr as *const u8 as usize,
                function,
                line,
            };
            self.handle_violation(violation.clone())?;
            return Err(violation);
        }

        // Alignment check
        if self.config.enable_alignment_checks {
            let required_alignment = mem::align_of::<T>();
            let address = ptr as *const u8 as usize;
            let actual_alignment = if address == 0 { 0 } else { address & !(address - 1) };
            
            if actual_alignment < required_alignment {
                let violation = SafetyViolation::UnalignedAccess {
                    address,
                    required_alignment,
                    actual_alignment,
                    function,
                    line,
                };
                self.handle_violation(violation.clone())?;
                return Err(violation);
            }
        }

        // Use-after-free check
        if self.config.enable_use_after_free_detection {
            if let Ok(tracker) = self.allocation_tracker.lock() {
                let address = ptr as usize;
                if let Some(record) = tracker.get(&address) {
                    if record.freed_at.is_some() {
                        let violation = SafetyViolation::UseAfterFree {
                            address,
                            original_size: record.size,
                            function,
                            line,
                        };
                        self.handle_violation(violation.clone())?;
                        return Err(violation);
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate array/buffer access
    pub fn validate_bounds_access(
        &self, 
        ptr: *const u8, 
        buffer_start: *const u8, 
        buffer_size: usize,
        access_offset: isize,
        function: &'static str, 
        line: u32
    ) -> Result<(), SafetyViolation> {
        if !self.config.enable_bounds_checks {
            return Ok(());
        }

        let access_addr = unsafe { ptr.offset(access_offset) } as usize;
        let buffer_start_addr = buffer_start as usize;
        let buffer_end_addr = buffer_start_addr + buffer_size;

        if access_addr < buffer_start_addr || access_addr >= buffer_end_addr {
            let violation = SafetyViolation::BufferOverflow {
                address: access_addr,
                buffer_start: buffer_start_addr,
                buffer_size,
                access_offset,
                function,
                line,
            };
            self.handle_violation(violation.clone())?;
            return Err(violation);
        }

        Ok(())
    }

    /// Record an allocation
    pub fn record_allocation(&self, ptr: *const u8, size: usize, alignment: usize, source: String) {
        if let Ok(mut tracker) = self.allocation_tracker.lock() {
            let address = ptr as usize;
            tracker.insert(address, AllocationRecord {
                size,
                alignment,
                allocated_at: current_timestamp(),
                freed_at: None,
                allocation_source: source,
            });
        }
    }

    /// Record a deallocation
    pub fn record_deallocation(&self, ptr: *const u8, function: &'static str, line: u32) -> Result<(), SafetyViolation> {
        if let Ok(mut tracker) = self.allocation_tracker.lock() {
            let address = ptr as usize;
            
            match tracker.get_mut(&address) {
                Some(record) => {
                    if record.freed_at.is_some() {
                        // Double free detected
                        let violation = SafetyViolation::DoubleFree {
                            address,
                            function,
                            line,
                        };
                        self.handle_violation(violation.clone())?;
                        return Err(violation);
                    }
                    record.freed_at = Some(current_timestamp());
                }
                None => {
                    // Free of untracked pointer - might be a stack allocation or external
                    // For now, we'll allow this but could add stricter checking
                }
            }
        }
        
        Ok(())
    }

    /// Handle a safety violation
    fn handle_violation(&self, violation: SafetyViolation) -> Result<(), SafetyViolation> {
        self.violation_count.fetch_add(1, Ordering::SeqCst);

        if self.config.log_violations {
            eprintln!("Memory Safety Violation: {:?}", violation);
            
            // Create a synthetic crash info for the SIGSEGV monitor
            let crash_info = create_crash_info_from_violation(&violation);
            record_global_event(crash_info);
        }

        if self.config.panic_on_violation {
            match violation {
                SafetyViolation::NullPointerDereference { address, function, line } => {
                    panic!("Null pointer dereference at 0x{:016x} in {}:{}", address, function, line);
                }
                SafetyViolation::UnalignedAccess { address, required_alignment, actual_alignment, function, line } => {
                    panic!("Unaligned access at 0x{:016x} (required: {}, actual: {}) in {}:{}", 
                          address, required_alignment, actual_alignment, function, line);
                }
                SafetyViolation::UseAfterFree { address, original_size, function, line } => {
                    panic!("Use after free at 0x{:016x} (size: {}) in {}:{}", 
                          address, original_size, function, line);
                }
                SafetyViolation::BufferOverflow { address, buffer_start, buffer_size, access_offset, function, line } => {
                    panic!("Buffer overflow at 0x{:016x} (buffer: 0x{:016x}+{}, offset: {}) in {}:{}", 
                          address, buffer_start, buffer_size, access_offset, function, line);
                }
                SafetyViolation::DoubleFree { address, function, line } => {
                    panic!("Double free at 0x{:016x} in {}:{}", address, function, line);
                }
            }
        }

        Err(violation)
    }

    /// Get violation statistics
    pub fn get_statistics(&self) -> ValidationStatistics {
        ValidationStatistics {
            total_violations: self.violation_count.load(Ordering::SeqCst),
            tracked_allocations: self.allocation_tracker.lock()
                .map(|tracker| tracker.len())
                .unwrap_or(0),
        }
    }
}

/// Validation statistics
#[derive(Debug, Clone)]
pub struct ValidationStatistics {
    /// Total number of memory safety violations detected
    pub total_violations: usize,
    /// Number of active tracked allocations
    pub tracked_allocations: usize,
}

/// Initialize the global memory safety validator
pub fn init_memory_safety(config: MemorySafetyConfig) {
    INIT_ONCE.call_once(|| {
        unsafe {
            VALIDATOR = Some(MemorySafetyValidator::new(config));
        }
    });
}

/// Get the global validator
pub fn get_validator() -> Option<&'static MemorySafetyValidator> {
    #[allow(static_mut_refs)]
    unsafe { VALIDATOR.as_ref() }
}

/// Convenience functions for validation

/// Validate pointer dereference with location info
#[macro_export]
macro_rules! safe_deref {
    ($ptr:expr) => {{
        if let Some(validator) = $crate::eval::memory_safety_validator::get_validator() {
            validator.validate_ptr_deref($ptr, file!(), line!())?;
        }
        unsafe { &*$ptr }
    }};
}

/// Validate mutable pointer dereference
#[macro_export]
macro_rules! safe_deref_mut {
    ($ptr:expr) => {{
        if let Some(validator) = $crate::eval::memory_safety_validator::get_validator() {
            validator.validate_ptr_deref($ptr, file!(), line!())?;
        }
        unsafe { &mut *$ptr }
    }};
}

/// Validate pointer cast with type safety
#[macro_export]
macro_rules! safe_cast_deref {
    ($ptr:expr, $ty:ty) => {{
        let typed_ptr = $ptr as *const $ty;
        if let Some(validator) = $crate::eval::memory_safety_validator::get_validator() {
            validator.validate_ptr_deref(typed_ptr, file!(), line!())?;
        }
        unsafe { &*typed_ptr }
    }};
}

/// Record allocation for tracking
pub fn record_allocation<T>(ptr: *const T, source: String) {
    if let Some(validator) = get_validator() {
        validator.record_allocation(
            ptr as *const u8,
            mem::size_of::<T>(),
            mem::align_of::<T>(),
            source,
        );
    }
}

/// Record deallocation for tracking
pub fn record_deallocation<T>(ptr: *const T) -> Result<(), SafetyViolation> {
    if let Some(validator) = get_validator() {
        validator.record_deallocation(ptr as *const u8, file!(), line!())
    } else {
        Ok(())
    }
}

/// Platform-specific memory safety functions

/// Check if address is properly aligned for the target platform
pub fn is_platform_aligned(address: usize, size: usize) -> bool {
    match size {
        1 => true, // byte alignment always OK
        2 => address % 2 == 0,
        4 => address % 4 == 0,
        8 => address % 8 == 0,
        16 => address % 16 == 0,
        _ => {
            // For larger sizes, check if it's aligned to the platform's preferred alignment
            #[cfg(target_arch = "aarch64")]
            let preferred_alignment = 16;
            #[cfg(target_arch = "x86_64")]
            let preferred_alignment = 8;
            #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
            let preferred_alignment = mem::align_of::<usize>();
            
            address % preferred_alignment == 0
        }
    }
}

/// Validate SIMD alignment requirements
pub fn validate_simd_alignment(address: usize) -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        // ARM NEON requires 16-byte alignment
        address % 16 == 0
    }
    #[cfg(target_arch = "x86_64")]
    {
        // SSE requires 16-byte, AVX requires 32-byte alignment
        if is_x86_feature_detected!("avx") {
            address % 32 == 0
        } else {
            address % 16 == 0
        }
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        // Conservative: assume 16-byte alignment requirement
        address % 16 == 0
    }
}

/// Helper functions

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn create_crash_info_from_violation(violation: &SafetyViolation) -> CrashInfo {
    use crate::debug::MemoryInfo;
    
    let (signal, fault_address) = match violation {
        SafetyViolation::NullPointerDereference { address, .. } => (11, Some(*address)), // SIGSEGV
        SafetyViolation::UnalignedAccess { address, .. } => (7, Some(*address)), // SIGBUS
        SafetyViolation::UseAfterFree { address, .. } => (11, Some(*address)), // SIGSEGV
        SafetyViolation::BufferOverflow { address, .. } => (11, Some(*address)), // SIGSEGV
        SafetyViolation::DoubleFree { address, .. } => (6, Some(*address)), // SIGABRT
    };

    CrashInfo {
        signal,
        signal_name: match signal {
            6 => "SIGABRT",
            7 => "SIGBUS", 
            11 => "SIGSEGV",
            _ => "UNKNOWN",
        },
        timestamp: current_timestamp(),
        process_id: std::process::id(),
        thread_id: get_thread_id(),
        fault_address,
        instruction_pointer: None,
        stack_pointer: None,
        backtrace: vec![format!("Memory safety violation: {:?}", violation)],
        register_dump: std::collections::HashMap::new(),
        memory_info: MemoryInfo {
            heap_size: 0,
            stack_size: 0,
            virtual_memory: 0,
            physical_memory: 0,
        },
        platform_info: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        test_context: Some(format!("memory_safety_validation")),
    }
}

fn get_thread_id() -> u64 {
    // Simple thread ID - using a hash since as_u64() is unstable
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_null_pointer_detection() {
        let config = MemorySafetyConfig::default();
        let validator = MemorySafetyValidator::new(config);
        
        let null_ptr: *const i32 = ptr::null();
        let result = validator.validate_ptr_deref(null_ptr, "test", 0);
        
        match result {
            Err(SafetyViolation::NullPointerDereference { address, .. }) => {
                assert_eq!(address, 0);
            }
            _ => panic!("Expected null pointer violation"),
        }
    }
    
    #[test]
    fn test_alignment_detection() {
        let config = MemorySafetyConfig::default();
        let validator = MemorySafetyValidator::new(config);
        
        // Create a misaligned pointer (odd address for a type requiring even alignment)
        let misaligned_ptr = 0x1001 as *const u64; // Misaligned for 8-byte type
        let result = validator.validate_ptr_deref(misaligned_ptr, "test", 0);
        
        match result {
            Err(SafetyViolation::UnalignedAccess { required_alignment, .. }) => {
                assert_eq!(required_alignment, mem::align_of::<u64>());
            }
            _ => panic!("Expected alignment violation"),
        }
    }
    
    #[test]
    fn test_platform_alignment() {
        assert!(is_platform_aligned(0x1000, 8)); // Well-aligned
        assert!(!is_platform_aligned(0x1001, 8)); // Misaligned for 8-byte
        assert!(is_platform_aligned(0x1010, 16)); // 16-byte aligned
    }
    
    #[test] 
    fn test_simd_alignment() {
        assert!(validate_simd_alignment(0x1000)); // 16-byte aligned
        assert!(!validate_simd_alignment(0x1008)); // Only 8-byte aligned
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx") {
                assert!(validate_simd_alignment(0x1020)); // 32-byte aligned
                assert!(!validate_simd_alignment(0x1010)); // Only 16-byte aligned
            }
        }
    }
}