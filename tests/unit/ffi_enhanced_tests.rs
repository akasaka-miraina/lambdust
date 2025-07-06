//! Enhanced FFI safety tests
//!
//! Tests for advanced safety features in the C FFI interface including
//! memory tracking, thread safety, resource limits, and error handling.

use lambdust::ffi_enhanced::*;
use lambdust::ffi::*;
use std::ptr;

#[test]
fn test_enhanced_context_creation() {
    unsafe {
        let ctx = lambdust_create_context();
        assert!(!ctx.is_null(), "Context creation should succeed");
        
        // Test context health check
        let health = lambdust_check_context_health(ctx);
        assert_eq!(health, LambdustErrorCode::Success as i32);
        
        lambdust_destroy_context(ctx);
    }
}

#[test]
fn test_context_validation() {
    unsafe {
        // Test null context
        let null_ctx: *mut LambdustContext = ptr::null_mut();
        let health = lambdust_check_context_health(null_ctx);
        assert_eq!(health, LambdustErrorCode::NullPointer as i32);
        
        // Test valid context
        let ctx = lambdust_create_context();
        assert!(!ctx.is_null());
        
        let health = lambdust_check_context_health(ctx);
        assert_eq!(health, LambdustErrorCode::Success as i32);
        
        lambdust_destroy_context(ctx);
        
        // Test destroyed context (should detect corruption)
        let health = lambdust_check_context_health(ctx);
        assert_ne!(health, LambdustErrorCode::Success as i32);
    }
}

#[test]
fn test_memory_tracking() {
    unsafe {
        let ctx = lambdust_create_context();
        assert!(!ctx.is_null());
        
        // Get initial memory stats
        let mut initial_total = 0;
        let mut initial_peak = 0;
        let mut initial_count = 0;
        let result = lambdust_get_memory_stats(
            ctx, 
            &mut initial_total, 
            &mut initial_peak, 
            &mut initial_count
        );
        assert_eq!(result, LambdustErrorCode::Success as i32);
        
        // Allocate tracked memory
        let size = 1024;
        let ptr = lambdust_alloc_tracked(ctx, size);
        assert!(!ptr.is_null(), "Tracked allocation should succeed");
        
        // Check memory stats after allocation
        let mut new_total = 0;
        let mut new_peak = 0;
        let mut new_count = 0;
        let result = lambdust_get_memory_stats(
            ctx, 
            &mut new_total, 
            &mut new_peak, 
            &mut new_count
        );
        assert_eq!(result, LambdustErrorCode::Success as i32);
        assert!(new_total >= initial_total + size, "Total memory should increase");
        assert!(new_count > initial_count, "Allocation count should increase");
        assert!(new_peak >= new_total, "Peak should be at least current total");
        
        // Free tracked memory
        let result = lambdust_free_tracked(ctx, ptr);
        assert_eq!(result, LambdustErrorCode::Success as i32);
        
        // Check memory stats after deallocation
        let mut final_total = 0;
        let mut final_peak = 0;
        let mut final_count = 0;
        let result = lambdust_get_memory_stats(
            ctx, 
            &mut final_total, 
            &mut final_peak, 
            &mut final_count
        );
        assert_eq!(result, LambdustErrorCode::Success as i32);
        assert!(final_total < new_total, "Total memory should decrease after free");
        assert_eq!(final_peak, new_peak, "Peak should remain the same");
        
        lambdust_destroy_context(ctx);
    }
}

#[test]
fn test_memory_limits() {
    unsafe {
        let ctx = lambdust_create_context();
        assert!(!ctx.is_null());
        
        // Try to allocate very large amount (should fail due to limits)
        let huge_size = 200 * 1024 * 1024; // 200MB (exceeds 100MB limit)
        let ptr = lambdust_alloc_tracked(ctx, huge_size);
        assert!(ptr.is_null(), "Huge allocation should fail due to limits");
        
        lambdust_destroy_context(ctx);
    }
}

#[test]
fn test_reference_counting() {
    unsafe {
        let ctx = lambdust_create_context();
        assert!(!ctx.is_null());
        
        // Increment reference count
        let result = lambdust_context_ref(ctx);
        assert_eq!(result, LambdustErrorCode::Success as i32);
        
        // First destroy should not actually destroy due to ref count
        lambdust_destroy_context(ctx);
        
        // Context should still be valid
        let health = lambdust_check_context_health(ctx);
        assert_eq!(health, LambdustErrorCode::Success as i32);
        
        // Second destroy should actually destroy
        lambdust_destroy_context(ctx);
    }
}

#[test]
fn test_enhanced_function_registration() {
    unsafe {
        let ctx = lambdust_create_context();
        assert!(!ctx.is_null());
        
        // Test function registration with user data
        extern "C" fn test_enhanced_func(
            _argc: std::os::raw::c_int,
            _argv: *const *const std::os::raw::c_char,
            result: *mut *mut std::os::raw::c_char,
            user_data: *mut std::os::raw::c_void,
        ) -> std::os::raw::c_int {
            // Simple test function that returns user data as string
            let data_value = if user_data.is_null() { 0 } else { 42 };
            let result_str = format!("{}", data_value);
            let c_str = std::ffi::CString::new(result_str).unwrap();
            *result = c_str.into_raw();
            LambdustErrorCode::Success as std::os::raw::c_int
        }
        
        let user_data = 42i32;
        let user_data_ptr = &user_data as *const i32 as *mut std::os::raw::c_void;
        
        let name = std::ffi::CString::new("test-enhanced").unwrap();
        let result = lambdust_register_function_enhanced(
            ctx,
            name.as_ptr(),
            test_enhanced_func,
            user_data_ptr,
            true, // thread safe
        );
        assert_eq!(result, LambdustErrorCode::Success as i32);
        
        lambdust_destroy_context(ctx);
    }
}

#[test]
fn test_sandboxed_context() {
    unsafe {
        let max_memory = 10 * 1024 * 1024; // 10MB
        let max_time = 5000; // 5 seconds
        
        let ctx = lambdust_create_sandboxed_context(max_memory, max_time);
        assert!(!ctx.is_null(), "Sandboxed context creation should succeed");
        
        // Test that it's a valid context
        let health = lambdust_check_context_health(ctx);
        assert_eq!(health, LambdustErrorCode::Success as i32);
        
        lambdust_destroy_context(ctx);
    }
}

#[test]
fn test_eval_with_timeout() {
    unsafe {
        let ctx = lambdust_create_context();
        assert!(!ctx.is_null());
        
        // Test simple evaluation with timeout
        let code = std::ffi::CString::new("(+ 1 2 3)").unwrap();
        let mut result: *mut std::os::raw::c_char = ptr::null_mut();
        let timeout_ms = 1000; // 1 second
        
        let error = lambdust_eval_with_timeout(
            ctx,
            code.as_ptr(),
            &mut result as *mut *mut std::os::raw::c_char,
            timeout_ms,
        );
        
        assert_eq!(error, LambdustErrorCode::Success as i32);
        if !result.is_null() {
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "6");
            lambdust_free_string(result);
        }
        
        lambdust_destroy_context(ctx);
    }
}

#[test]
fn test_detailed_error_handling() {
    unsafe {
        let ctx = lambdust_create_context();
        assert!(!ctx.is_null());
        
        // Cause an error by evaluating invalid code
        let invalid_code = std::ffi::CString::new("(+ 1 2").unwrap(); // Missing closing paren
        let mut result: *mut std::os::raw::c_char = ptr::null_mut();
        
        let error = lambdust_eval(
            ctx,
            invalid_code.as_ptr(),
            &mut result as *mut *mut std::os::raw::c_char,
        );
        
        // Should have an error
        assert_ne!(error, LambdustErrorCode::Success as i32);
        
        // Get detailed error information
        let mut error_code = 0;
        let mut error_message: *const std::os::raw::c_char = ptr::null();
        let mut error_location: *const std::os::raw::c_char = ptr::null();
        
        let detail_result = lambdust_get_detailed_error(
            ctx,
            &mut error_code,
            &mut error_message,
            &mut error_location,
        );
        
        assert_eq!(detail_result, LambdustErrorCode::Success as i32);
        assert_ne!(error_code, 0);
        
        if !error_message.is_null() {
            let msg = std::ffi::CStr::from_ptr(error_message);
            assert!(!msg.to_str().unwrap().is_empty());
        }
        
        lambdust_destroy_context(ctx);
    }
}

#[test]
fn test_sensitive_data_clearing() {
    unsafe {
        let ctx = lambdust_create_context();
        assert!(!ctx.is_null());
        
        // Cause an error to populate error information
        let invalid_code = std::ffi::CString::new("(invalid syntax").unwrap();
        let mut result: *mut std::os::raw::c_char = ptr::null_mut();
        
        let _error = lambdust_eval(
            ctx,
            invalid_code.as_ptr(),
            &mut result as *mut *mut std::os::raw::c_char,
        );
        
        // Clear sensitive data
        let clear_result = lambdust_clear_sensitive_data(ctx);
        assert_eq!(clear_result, LambdustErrorCode::Success as i32);
        
        // Error message should be cleared
        let error_msg = lambdust_get_last_error(ctx);
        assert!(error_msg.is_null() || std::ffi::CStr::from_ptr(error_msg).to_str().unwrap().is_empty());
        
        lambdust_destroy_context(ctx);
    }
}

#[test]
fn test_null_pointer_safety() {
    unsafe {
        let null_ctx: *mut LambdustContext = ptr::null_mut();
        
        // All functions should handle null context gracefully
        assert_eq!(
            lambdust_check_context_health(null_ctx),
            LambdustErrorCode::NullPointer as i32
        );
        
        assert_eq!(
            lambdust_context_ref(null_ctx),
            LambdustErrorCode::InvalidArgument as i32
        );
        
        let ptr = lambdust_alloc_tracked(null_ctx, 100);
        assert!(ptr.is_null());
        
        assert_eq!(
            lambdust_free_tracked(null_ctx, ptr::null_mut()),
            LambdustErrorCode::InvalidArgument as i32
        );
        
        assert_eq!(
            lambdust_clear_sensitive_data(null_ctx),
            LambdustErrorCode::InvalidArgument as i32
        );
    }
}

#[test]
fn test_thread_safety_detection() {
    use std::thread;
    use std::sync::Arc;
    
    unsafe {
        let ctx = lambdust_create_context();
        assert!(!ctx.is_null());
        
        // Context should be valid in the creating thread
        assert_eq!(
            lambdust_check_context_health(ctx),
            LambdustErrorCode::Success as i32
        );
        
        // Test thread safety from different thread
        let ctx_ptr = ctx as usize;
        let handle = thread::spawn(move || {
            let ctx = ctx_ptr as *mut LambdustContext;
            unsafe {
                // Should detect thread safety violation
                let health = lambdust_check_context_health(ctx);
                health == LambdustErrorCode::ThreadSafetyError as i32
            }
        });
        
        let thread_safety_detected = handle.join().unwrap();
        assert!(thread_safety_detected, "Thread safety violation should be detected");
        
        lambdust_destroy_context(ctx);
    }
}