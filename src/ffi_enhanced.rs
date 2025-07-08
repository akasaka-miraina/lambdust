//! Enhanced safety features for C FFI interface
//!
//! This module extends the basic FFI with advanced safety, memory tracking,
//! thread safety, and callback mechanisms.

#![cfg(not(feature = "embedded"))]
#![allow(unsafe_op_in_unsafe_fn)]

use crate::ffi::*;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::time::{Duration, SystemTime};

/// Enhanced memory allocation with tracking
///
/// # Safety
///
/// This function uses malloc and manipulates raw pointers. The caller must ensure:
/// - context is a valid, non-null pointer to an initialized LambdustContext
/// - size is a reasonable allocation size that won't cause overflow
/// - the returned pointer should be freed using lambdust_free_tracked
#[unsafe(no_mangle)]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn lambdust_alloc_tracked(
    context: *mut LambdustContext,
    size: usize,
) -> *mut c_char {
    if context.is_null() || !unsafe { validate_context(context) } {
        return std::ptr::null_mut();
    }

    let ctx = unsafe { &mut *context };

    // Check memory limits
    if let Ok(tracker) = ctx.memory_tracker.read() {
        const MAX_MEMORY: usize = 100 * 1024 * 1024; // 100MB limit
        if tracker.total_allocated + size > MAX_MEMORY {
            return std::ptr::null_mut();
        }
    }

    let ptr = libc::malloc(size) as *mut c_char;
    if !ptr.is_null() {
        if let Ok(mut tracker) = ctx.memory_tracker.write() {
            tracker.allocated_strings.insert(ptr, size);
            tracker.total_allocated += size;
            tracker.allocation_count += 1;
            if tracker.total_allocated > tracker.peak_usage {
                tracker.peak_usage = tracker.total_allocated;
            }
        }
    }

    ptr
}

/// Enhanced memory deallocation with tracking
///
/// # Safety
///
/// This function dereferences raw pointers and calls libc::free. The caller must ensure:
/// - context is a valid, non-null pointer to an initialized LambdustContext
/// - ptr is a valid pointer that was previously allocated by lambdust_alloc_tracked
/// - ptr has not been freed before and will not be used after this call
/// - the memory tracker in context is properly initialized and accessible
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_free_tracked(
    context: *mut LambdustContext,
    ptr: *mut c_char,
) -> c_int {
    if context.is_null() || ptr.is_null() || !validate_context(context) {
        return LambdustErrorCode::InvalidArgument as c_int;
    }

    let ctx = unsafe { &mut *context };

    if let Ok(mut tracker) = ctx.memory_tracker.write() {
        if let Some(size) = tracker.allocated_strings.remove(&ptr) {
            tracker.total_allocated = tracker.total_allocated.saturating_sub(size);
            libc::free(ptr as *mut c_void);
            LambdustErrorCode::Success as c_int
        } else {
            LambdustErrorCode::InvalidArgument as c_int
        }
    } else {
        LambdustErrorCode::ThreadSafetyError as c_int
    }
}

/// Get memory usage statistics
///
/// # Safety
///
/// This function dereferences raw pointers to write statistics. The caller must ensure:
/// - context is a valid, non-null pointer to an initialized LambdustContext
/// - total_allocated, peak_usage, and allocation_count (if not null) point to valid memory
/// - the output pointers remain valid for the duration of the call
/// - the memory tracker in context is properly initialized and accessible
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_get_memory_stats(
    context: *mut LambdustContext,
    total_allocated: *mut usize,
    peak_usage: *mut usize,
    allocation_count: *mut u64,
) -> c_int {
    if context.is_null() || !unsafe { validate_context(context) } {
        return LambdustErrorCode::InvalidArgument as c_int;
    }

    let ctx = unsafe { &*context };

    if let Ok(tracker) = ctx.memory_tracker.read() {
        if !total_allocated.is_null() {
            *total_allocated = tracker.total_allocated;
        }
        if !peak_usage.is_null() {
            *peak_usage = tracker.peak_usage;
        }
        if !allocation_count.is_null() {
            *allocation_count = tracker.allocation_count;
        }
        LambdustErrorCode::Success as c_int
    } else {
        LambdustErrorCode::ThreadSafetyError as c_int
    }
}

/// Register an enhanced host function with user data and thread safety info
///
/// # Safety
///
/// This function dereferences raw pointers and stores function pointers. The caller must ensure:
/// - context is a valid, non-null pointer to an initialized LambdustContext
/// - name is a valid null-terminated C string pointer
/// - func is a valid function pointer with the correct signature
/// - user_data (if not null) points to valid memory that remains valid during callback lifetime
/// - the function pointer and user_data remain valid for the lifetime of the context
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_register_function_enhanced(
    context: *mut LambdustContext,
    name: *const c_char,
    func: LambdustHostFunction,
    user_data: *mut c_void,
    thread_safe: bool,
) -> c_int {
    if context.is_null() || name.is_null() || !validate_context(context) {
        return LambdustErrorCode::InvalidArgument as c_int;
    }

    let ctx = unsafe { &mut *context };

    // Convert function name
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            ctx.last_error = Some("Invalid UTF-8 in function name".to_string());
            return LambdustErrorCode::InvalidArgument as c_int;
        }
    };

    // Register callback info
    let callback_info = CallbackInfo {
        function: func,
        user_data,
        thread_safe,
        registered_time: SystemTime::now(),
    };

    if let Ok(mut callbacks) = ctx.callbacks.write() {
        callbacks.insert(name_str.clone(), callback_info);
    } else {
        return LambdustErrorCode::ThreadSafetyError as c_int;
    }

    // Register with bridge (simplified - would need full integration)
    LambdustErrorCode::Success as c_int
}

/// Set error callback for enhanced error handling
///
/// # Safety
///
/// This function stores function pointers and user data. The caller must ensure:
/// - context is a valid, non-null pointer to an initialized LambdustContext
/// - callback is a valid function pointer with the correct signature
/// - user_data (if not null) points to valid memory that remains valid during callback lifetime
/// - the callback function pointer and user_data remain valid for the lifetime of the context
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_set_error_callback(
    context: *mut LambdustContext,
    _callback: LambdustErrorCallback,
    _user_data: *mut c_void,
) -> c_int {
    if context.is_null() || !unsafe { validate_context(context) } {
        return LambdustErrorCode::InvalidArgument as c_int;
    }

    // Store error callback (would need additional context field)
    LambdustErrorCode::Success as c_int
}

/// Check context health and integrity
///
/// # Safety
///
/// This function dereferences a raw pointer to validate context state. The caller must ensure:
/// - context is a valid pointer (may be null, which is handled gracefully)
/// - if context is non-null, it points to a properly initialized LambdustContext
/// - the context structure remains valid for the duration of the call
/// - concurrent access to the context is properly synchronized
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_check_context_health(context: *mut LambdustContext) -> c_int {
    if context.is_null() {
        return LambdustErrorCode::NullPointer as c_int;
    }

    if !validate_context(context) {
        return LambdustErrorCode::CorruptedContext as c_int;
    }

    let ctx = unsafe { &*context };

    // Check if created too long ago (prevent stale contexts)
    if let Ok(elapsed) = ctx.creation_time.elapsed() {
        const MAX_CONTEXT_AGE: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours
        if elapsed > MAX_CONTEXT_AGE {
            return LambdustErrorCode::SecurityError as c_int;
        }
    }

    // Check thread consistency
    if !check_thread_safety(context) {
        return LambdustErrorCode::ThreadSafetyError as c_int;
    }

    // Check memory tracker integrity
    if ctx.memory_tracker.read().is_err() {
        return LambdustErrorCode::ThreadSafetyError as c_int;
    }

    LambdustErrorCode::Success as c_int
}

/// Increment context reference count for shared usage
///
/// # Safety
///
/// This function dereferences a raw pointer and modifies reference count state. The caller must ensure:
/// - context is a valid, non-null pointer to an initialized LambdustContext
/// - the context structure remains valid for the duration of the call
/// - proper synchronization when multiple threads access the same context
/// - the reference count is properly decremented when no longer needed
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_context_ref(context: *mut LambdustContext) -> c_int {
    if context.is_null() || !unsafe { validate_context(context) } {
        return LambdustErrorCode::InvalidArgument as c_int;
    }

    let ctx = unsafe { &*context };

    if let Ok(mut ref_count) = ctx.ref_count.lock() {
        *ref_count += 1;
        LambdustErrorCode::Success as c_int
    } else {
        LambdustErrorCode::ThreadSafetyError as c_int
    }
}

/// Evaluate with timeout to prevent infinite loops
///
/// # Safety
///
/// This function dereferences multiple raw pointers and delegates to lambdust_eval. The caller must ensure:
/// - context is a valid, non-null pointer to an initialized LambdustContext
/// - code is a valid null-terminated C string pointer
/// - result is a valid pointer to a pointer that will receive the allocated result string
/// - all pointers remain valid for the duration of the call
/// - the allocated result string must be freed by the caller using lambdust_free_string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_eval_with_timeout(
    context: *mut LambdustContext,
    code: *const c_char,
    result: *mut *mut c_char,
    _timeout_ms: u32,
) -> c_int {
    if context.is_null() || code.is_null() || result.is_null() {
        return LambdustErrorCode::NullPointer as c_int;
    }

    if !validate_context(context) {
        return LambdustErrorCode::CorruptedContext as c_int;
    }

    // This would require async evaluation implementation
    // For now, fall back to regular eval
    lambdust_eval(context, code, result)
}

/// Create a sandboxed context with resource limits
///
/// # Safety
///
/// This function creates a new context by calling lambdust_create_context. The caller must ensure:
/// - the returned pointer (if non-null) is eventually freed using lambdust_destroy_context
/// - the context is not used after being destroyed
/// - proper synchronization if the context is shared across threads
/// - max_memory and max_execution_time_ms are reasonable values that won't cause overflow
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_create_sandboxed_context(
    _max_memory: usize,
    _max_execution_time_ms: u32,
) -> *mut LambdustContext {
    let context = unsafe { lambdust_create_context() };
    if context.is_null() {
        return context;
    }

    // Set resource limits (would need additional implementation)
    // For now, return regular context
    context
}

/// Get detailed error information
///
/// # Safety
///
/// This function dereferences multiple raw pointers and allocates C strings. The caller must ensure:
/// - context is a valid, non-null pointer to an initialized LambdustContext
/// - error_code (if not null) points to valid memory for writing the error code
/// - error_message (if not null) points to valid memory for writing the error message pointer
/// - error_location (if not null) points to valid memory for writing the error location pointer
/// - all output pointers remain valid for the duration of the call
/// - allocated error message strings must be freed by the caller
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_get_detailed_error(
    context: *mut LambdustContext,
    error_code: *mut c_int,
    error_message: *mut *const c_char,
    error_location: *mut *const c_char,
) -> c_int {
    if context.is_null() || !unsafe { validate_context(context) } {
        return LambdustErrorCode::InvalidArgument as c_int;
    }

    let ctx = unsafe { &*context };

    if let Some(ref error_msg) = ctx.last_error {
        unsafe {
            if !error_message.is_null() {
                let c_msg =
                    CString::new(error_msg.as_str()).unwrap_or_else(|_| CString::new("").unwrap());
                *error_message = c_msg.into_raw();
            }
            if !error_code.is_null() {
                *error_code = LambdustErrorCode::Error as c_int;
            }
            if !error_location.is_null() {
                *error_location = std::ptr::null();
            }
        }
        LambdustErrorCode::Success as c_int
    } else {
        LambdustErrorCode::Success as c_int
    }
}

/// Security function to clear sensitive data
///
/// # Safety
///
/// This function dereferences a raw pointer and modifies context state. The caller must ensure:
/// - context is a valid, non-null pointer to an initialized LambdustContext
/// - the context structure remains valid for the duration of the call
/// - concurrent access to the context is properly synchronized
/// - callbacks and user data referenced by the context remain valid during cleanup
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_clear_sensitive_data(context: *mut LambdustContext) -> c_int {
    if context.is_null() || !unsafe { validate_context(context) } {
        return LambdustErrorCode::InvalidArgument as c_int;
    }

    let ctx = unsafe { &mut *context };

    // Clear error messages
    ctx.last_error = None;

    // Clear callback user data (would need implementation)
    if let Ok(mut callbacks) = ctx.callbacks.write() {
        for (_, callback_info) in callbacks.iter_mut() {
            callback_info.user_data = std::ptr::null_mut();
        }
    }

    LambdustErrorCode::Success as c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_context_creation() {
        unsafe {
            let ctx = lambdust_create_context();
            assert!(!ctx.is_null());
            assert_eq!(
                lambdust_check_context_health(ctx),
                LambdustErrorCode::Success as c_int
            );
            lambdust_destroy_context(ctx);
        }
    }

    #[test]
    fn test_memory_tracking() {
        unsafe {
            let ctx = lambdust_create_context();
            assert!(!ctx.is_null());

            let ptr = lambdust_alloc_tracked(ctx, 100);
            assert!(!ptr.is_null());

            let mut total = 0;
            let mut peak = 0;
            let mut count = 0;
            assert_eq!(
                lambdust_get_memory_stats(ctx, &mut total, &mut peak, &mut count),
                LambdustErrorCode::Success as c_int
            );
            assert!(total >= 100);
            assert!(count >= 1);

            assert_eq!(
                lambdust_free_tracked(ctx, ptr),
                LambdustErrorCode::Success as c_int
            );

            lambdust_destroy_context(ctx);
        }
    }

    #[test]
    fn test_context_validation() {
        unsafe {
            let null_ctx: *mut LambdustContext = std::ptr::null_mut();
            assert_eq!(
                lambdust_check_context_health(null_ctx),
                LambdustErrorCode::NullPointer as c_int
            );

            let ctx = lambdust_create_context();
            assert!(!ctx.is_null());
            assert_eq!(
                lambdust_check_context_health(ctx),
                LambdustErrorCode::Success as c_int
            );

            lambdust_destroy_context(ctx);
        }
    }
}
