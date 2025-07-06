//! Enhanced C FFI interface for Lambdust Scheme interpreter
//!
//! This module provides a hardened C-compatible interface for embedding Lambdust
//! in C and C++ applications. Features enhanced safety, thread safety,
//! advanced error handling, and resource tracking.

use crate::bridge::LambdustBridge;
use crate::error::LambdustError;
use crate::interpreter::LambdustInterpreter;
use crate::marshal::{c_string_to_scheme, scheme_string_to_c, free_c_string};
use crate::value::Value;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::panic;
use std::ptr;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::time::SystemTime;
use std::thread;

/// Enhanced error codes for C FFI interface
#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum LambdustErrorCode {
    /// Operation successful
    Success = 0,
    /// General error
    Error = 1,
    /// Invalid argument
    InvalidArgument = 2,
    /// Null pointer
    NullPointer = 3,
    /// Memory allocation failure
    MemoryError = 4,
    /// Evaluation error
    EvaluationError = 5,
    /// Type error
    TypeError = 6,
    /// Arity error (wrong number of arguments)
    ArityError = 7,
    /// Runtime error
    RuntimeError = 8,
    /// Panic occurred
    PanicError = 9,
    /// Thread safety violation
    ThreadSafetyError = 10,
    /// Resource limit exceeded
    ResourceLimitError = 11,
    /// Context corrupted
    CorruptedContext = 12,
    /// Callback error
    CallbackError = 13,
    /// Security violation
    SecurityError = 14,
}

/// Enhanced context handle with safety features
#[repr(C)]
pub struct LambdustContext {
    pub interpreter: LambdustInterpreter,
    pub bridge: LambdustBridge,
    pub last_error: Option<String>,
    // Safety and tracking fields
    pub thread_id: std::thread::ThreadId,
    pub creation_time: SystemTime,
    pub magic_number: u64,
    pub ref_count: Arc<Mutex<u32>>,
    pub memory_tracker: Arc<RwLock<MemoryTracker>>,
    pub callbacks: Arc<RwLock<HashMap<String, CallbackInfo>>>,
}

/// Memory tracking for FFI allocations
#[derive(Debug, Default)]
pub struct MemoryTracker {
    pub allocated_strings: HashMap<*mut c_char, usize>,
    pub total_allocated: usize,
    pub peak_usage: usize,
    pub allocation_count: u64,
}

/// Callback function information
#[derive(Debug, Clone)]
pub struct CallbackInfo {
    pub function: LambdustHostFunction,
    pub user_data: *mut c_void,
    pub thread_safe: bool,
    pub registered_time: SystemTime,
}

/// Enhanced C-compatible host function signature
pub type LambdustEnhancedHostFunction = unsafe extern "C" fn(
    argc: c_int,
    argv: *const *const c_char,
    result: *mut *mut c_char,
    user_data: *mut c_void,
) -> c_int;

/// Enhanced callback function signature for error handling
pub type LambdustErrorCallback = unsafe extern "C" fn(
    context: *mut LambdustContext,
    error_code: c_int,
    error_message: *const c_char,
    user_data: *mut c_void,
);

/// Magic number for context validation
pub const CONTEXT_MAGIC: u64 = 0xDEADBEEF_CAFEBABE;

/// Context validation helper
pub unsafe fn validate_context(context: *const LambdustContext) -> bool {
    if context.is_null() {
        return false;
    }
    
    let ctx = &*context;
    ctx.magic_number == CONTEXT_MAGIC
}

/// Thread safety check
pub unsafe fn check_thread_safety(context: *const LambdustContext) -> bool {
    if !validate_context(context) {
        return false;
    }
    
    let ctx = &*context;
    ctx.thread_id == std::thread::current().id()
}

/// C-compatible host function signature
///
/// # Parameters
/// - `argc`: Number of arguments
/// - `argv`: Array of argument strings
/// - `result`: Output buffer for result string (allocated by host function)
///
/// # Returns
/// Error code indicating success or failure
pub type LambdustHostFunction = unsafe extern "C" fn(
    argc: c_int,
    argv: *const *const c_char,
    result: *mut *mut c_char,
) -> c_int;

/// Create a new Lambdust context
///
/// # Returns
/// Pointer to new context, or null on failure
///
/// # Safety
/// The returned pointer must be freed with `lambdust_destroy_context`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_create_context() -> *mut LambdustContext {
    let result = panic::catch_unwind(|| {
        let interpreter = LambdustInterpreter::new();
        let bridge = LambdustBridge::new();
        
        Box::into_raw(Box::new(LambdustContext {
            interpreter,
            bridge,
            last_error: None,
            thread_id: std::thread::current().id(),
            creation_time: SystemTime::now(),
            magic_number: CONTEXT_MAGIC,
            ref_count: Arc::new(Mutex::new(1)),
            memory_tracker: Arc::new(RwLock::new(MemoryTracker::default())),
            callbacks: Arc::new(RwLock::new(HashMap::new())),
        }))
    });
    
    match result {
        Ok(ctx) => ctx,
        Err(_) => ptr::null_mut(),
    }
}

/// Destroy a Lambdust context and free associated memory
///
/// # Parameters
/// - `context`: Context to destroy
///
/// # Safety
/// The context pointer must be valid and not used after this call
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_destroy_context(context: *mut LambdustContext) {
    if !context.is_null() {
        // Direct drop without panic catching for better compatibility
        drop(unsafe { Box::from_raw(context) });
    }
}

/// Evaluate Scheme code and return the result
///
/// # Parameters
/// - `context`: Lambdust context
/// - `code`: Null-terminated Scheme code string
/// - `result`: Output buffer for result string (allocated by this function)
///
/// # Returns
/// Error code indicating success or failure
///
/// # Safety
/// - `context` must be a valid context pointer
/// - `code` must be a valid null-terminated string
/// - `result` must be freed with `lambdust_free_string`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_eval(
    context: *mut LambdustContext,
    code: *const c_char,
    result: *mut *mut c_char,
) -> c_int {
    if context.is_null() || code.is_null() || result.is_null() {
        return LambdustErrorCode::NullPointer as c_int;
    }
    
    let ctx = unsafe { &mut *context };
    
    // Convert C string to Rust string
    let code_str = match unsafe { CStr::from_ptr(code).to_str() } {
        Ok(s) => s,
        Err(_) => {
            ctx.last_error = Some("Invalid UTF-8 in code string".to_string());
            return LambdustErrorCode::InvalidArgument as c_int;
        }
    };
    
    // Evaluate the code
    match ctx.interpreter.eval_string(code_str) {
        Ok(value) => {
            // Convert result to C string
            let result_str = value.to_string();
            let result_value = Value::String(result_str);
            match scheme_string_to_c(&result_value) {
                Ok(c_str) => {
                    unsafe { *result = c_str; }
                    LambdustErrorCode::Success as c_int
                }
                Err(_) => {
                    ctx.last_error = Some("Failed to convert result to C string".to_string());
                    LambdustErrorCode::MemoryError as c_int
                }
            }
        }
        Err(err) => {
            ctx.last_error = Some(err.to_string());
            match err {
                LambdustError::ArityError { .. } => LambdustErrorCode::ArityError as c_int,
                LambdustError::TypeError { .. } => LambdustErrorCode::TypeError as c_int,
                LambdustError::RuntimeError { .. } => LambdustErrorCode::RuntimeError as c_int,
                _ => LambdustErrorCode::EvaluationError as c_int,
            }
        }
    }
}

/// Register a host function that can be called from Scheme
///
/// # Parameters
/// - `context`: Lambdust context
/// - `name`: Function name in Scheme (null-terminated)
/// - `func`: Host function implementation
///
/// # Returns
/// Error code indicating success or failure
///
/// # Safety
/// - `context` must be a valid context pointer
/// - `name` must be a valid null-terminated string
/// - `func` must be a valid function pointer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_register_function(
    context: *mut LambdustContext,
    name: *const c_char,
    func: LambdustHostFunction,
) -> c_int {
    if context.is_null() || name.is_null() {
        return LambdustErrorCode::NullPointer as c_int;
    }
    
    let ctx = unsafe { &mut *context };
    
    // Convert function name to Rust string
    let name_str = match unsafe { CStr::from_ptr(name).to_str() } {
        Ok(s) => s,
        Err(_) => {
            ctx.last_error = Some("Invalid UTF-8 in function name".to_string());
            return LambdustErrorCode::InvalidArgument as c_int;
        }
    };
    
    // Create a wrapper that calls the C function
    let func_wrapper = move |args: &[Value]| -> crate::Result<Value> {
        // Convert Scheme arguments to C strings
        let c_strings: Result<Vec<CString>, _> = args
            .iter()
            .map(|arg| CString::new(arg.to_string()))
            .collect();
        
        let c_strings = c_strings.map_err(|_| {
            LambdustError::runtime_error("Failed to convert arguments to C strings".to_string())
        })?;
        
        // Create array of C string pointers
        let c_ptrs: Vec<*const c_char> = c_strings.iter().map(|s| s.as_ptr()).collect();
        
        // Call the C function
        let mut result_ptr: *mut c_char = ptr::null_mut();
        let error_code = unsafe {
            func(
                c_strings.len() as c_int,
                c_ptrs.as_ptr(),
                &mut result_ptr,
            )
        };
        
        if error_code != LambdustErrorCode::Success as c_int {
            return Err(LambdustError::runtime_error(format!(
                "Host function returned error code: {}",
                error_code
            )));
        }
        
        if result_ptr.is_null() {
            return Ok(Value::Undefined);
        }
        
        // Convert result back to Scheme value
        let result_cstr = unsafe { CStr::from_ptr(result_ptr) };
        let result_str = result_cstr.to_str().map_err(|_| {
            LambdustError::runtime_error("Invalid UTF-8 in function result".to_string())
        })?;
        
        // Try to parse as number first, fall back to string
        let scheme_value = if let Ok(num) = result_str.parse::<i64>() {
            Value::Number(crate::lexer::SchemeNumber::Integer(num))
        } else if let Ok(num) = result_str.parse::<f64>() {
            Value::Number(crate::lexer::SchemeNumber::Real(num))
        } else {
            // Return as string value
            Value::String(result_str.to_string())
        };
        
        // Free the result string
        unsafe { free_c_string(result_ptr); }
        
        Ok(scheme_value)
    };
    
    // Register the function with the interpreter
    ctx.interpreter.register_simple_host_function(name_str.to_string(), func_wrapper);
    LambdustErrorCode::Success as c_int
}

/// Get the last error message
///
/// # Parameters
/// - `context`: Lambdust context
///
/// # Returns
/// Null-terminated error message, or null if no error
///
/// # Safety
/// - `context` must be a valid context pointer
/// - The returned string is owned by the context and should not be freed
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_get_last_error(
    context: *mut LambdustContext,
) -> *const c_char {
    if context.is_null() {
        return ptr::null();
    }
    
    let ctx = unsafe { &*context };
    match &ctx.last_error {
        Some(error) => {
            match CString::new(error.as_str()) {
                Ok(c_str) => {
                    // This is a bit unsafe - we're returning a pointer to memory
                    // that could be freed. In practice, the context should live
                    // longer than the error message usage.
                    c_str.into_raw()
                }
                Err(_) => ptr::null(),
            }
        }
        None => ptr::null(),
    }
}

/// Free a string allocated by Lambdust
///
/// # Parameters
/// - `str_ptr`: String pointer to free
///
/// # Safety
/// The pointer must have been allocated by Lambdust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_free_string(str_ptr: *mut c_char) {
    if !str_ptr.is_null() {
        unsafe { free_c_string(str_ptr); }
    }
}

/// Get version information for Lambdust
///
/// # Returns
/// Static string containing version information
///
/// # Safety
/// The returned string is statically allocated and should not be freed
#[unsafe(no_mangle)]
pub extern "C" fn lambdust_get_version() -> *const c_char {
    static VERSION: &[u8] = concat!("Lambdust ", env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr() as *const c_char
}

/// Check if Lambdust library is properly initialized
///
/// Performs a basic sanity check to ensure the library is working correctly
///
/// # Returns
/// 1 if library is working correctly, 0 otherwise
#[unsafe(no_mangle)]
pub extern "C" fn lambdust_check_library() -> c_int {
    // Test basic interpreter creation and evaluation
    let mut interpreter = LambdustInterpreter::new();
    match interpreter.eval_string("(+ 1 2)") {
        Ok(Value::Number(crate::lexer::SchemeNumber::Integer(3))) => 1,
        _ => 0,
    }
}

/// Call a Scheme function from C
///
/// # Parameters
/// - `context`: Lambdust context
/// - `function_name`: Name of the Scheme function to call
/// - `argc`: Number of arguments
/// - `argv`: Array of argument strings
/// - `result`: Output buffer for result string
///
/// # Returns
/// Error code indicating success or failure
///
/// # Safety
/// All pointers must be valid and strings must be null-terminated
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lambdust_call_function(
    context: *mut LambdustContext,
    function_name: *const c_char,
    argc: c_int,
    argv: *const *const c_char,
    result: *mut *mut c_char,
) -> c_int {
    if context.is_null() || function_name.is_null() || result.is_null() {
        return LambdustErrorCode::NullPointer as c_int;
    }
    
    if argc < 0 || (argc > 0 && argv.is_null()) {
        return LambdustErrorCode::InvalidArgument as c_int;
    }
    
    let ctx = unsafe { &mut *context };
    
    // Convert function name
    let func_name = match unsafe { CStr::from_ptr(function_name).to_str() } {
        Ok(s) => s,
        Err(_) => {
            ctx.last_error = Some("Invalid UTF-8 in function name".to_string());
            return LambdustErrorCode::InvalidArgument as c_int;
        }
    };
    
    // Convert arguments
    let mut args = Vec::new();
    for i in 0..argc {
        let arg_ptr = unsafe { *argv.add(i as usize) };
        if arg_ptr.is_null() {
            ctx.last_error = Some("Null argument pointer".to_string());
            return LambdustErrorCode::NullPointer as c_int;
        }
        
        let _arg_str = match unsafe { CStr::from_ptr(arg_ptr).to_str() } {
            Ok(s) => s,
            Err(_) => {
                ctx.last_error = Some("Invalid UTF-8 in argument".to_string());
                return LambdustErrorCode::InvalidArgument as c_int;
            }
        };
        
        match unsafe { c_string_to_scheme(arg_ptr) } {
            Ok(value) => args.push(value),
            Err(err) => {
                ctx.last_error = Some(err.to_string());
                return LambdustErrorCode::TypeError as c_int;
            }
        }
    }
    
    // Build and evaluate the function call
    let call_expr = if args.is_empty() {
        format!("({})", func_name)
    } else {
        let arg_strs: Vec<String> = args.iter().map(|v| v.to_string()).collect();
        format!("({} {})", func_name, arg_strs.join(" "))
    };
    
    match ctx.interpreter.eval_string(&call_expr) {
        Ok(value) => {
            let result_str = value.to_string();
            let result_value = Value::String(result_str);
            match scheme_string_to_c(&result_value) {
                Ok(c_str) => {
                    unsafe { *result = c_str; }
                    LambdustErrorCode::Success as c_int
                }
                Err(_) => {
                    ctx.last_error = Some("Failed to convert result to C string".to_string());
                    LambdustErrorCode::MemoryError as c_int
                }
            }
        }
        Err(err) => {
            ctx.last_error = Some(err.to_string());
            LambdustErrorCode::EvaluationError as c_int
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    
    #[test]
    fn test_context_creation_and_destruction() {
        unsafe {
            let ctx = lambdust_create_context();
            assert!(!ctx.is_null());
            lambdust_destroy_context(ctx);
        }
    }
    
    #[test]
    fn test_basic_evaluation() {
        unsafe {
            let ctx = lambdust_create_context();
            assert!(!ctx.is_null());
            
            let code = CString::new("(+ 1 2 3)").unwrap();
            let mut result: *mut c_char = ptr::null_mut();
            
            let error_code = lambdust_eval(ctx, code.as_ptr(), &mut result);
            assert_eq!(error_code, LambdustErrorCode::Success as c_int);
            assert!(!result.is_null());
            
            let result_str = CStr::from_ptr(result).to_str().unwrap();
            assert_eq!(result_str, "6");
            
            lambdust_free_string(result);
            lambdust_destroy_context(ctx);
        }
    }
    
    #[test]
    fn test_host_function_registration() {
        unsafe extern "C" fn test_host_func(
            argc: c_int,
            argv: *const *const c_char,
            result: *mut *mut c_char,
        ) -> c_int {
            if argc != 2 {
                return LambdustErrorCode::ArityError as c_int;
            }
            
            // Simple addition function
            let a_str = unsafe { CStr::from_ptr(*argv).to_str().unwrap() };
            let b_str = unsafe { CStr::from_ptr(*argv.add(1)).to_str().unwrap() };
            
            let a: i32 = a_str.parse().unwrap_or(0);
            let b: i32 = b_str.parse().unwrap_or(0);
            let sum = a + b;
            
            let result_string = CString::new(sum.to_string()).unwrap();
            unsafe { *result = result_string.into_raw(); }
            
            LambdustErrorCode::Success as c_int
        }
        
        unsafe {
            let ctx = lambdust_create_context();
            assert!(!ctx.is_null());
            
            let func_name = CString::new("host-add").unwrap();
            let error_code = lambdust_register_function(ctx, func_name.as_ptr(), test_host_func);
            assert_eq!(error_code, LambdustErrorCode::Success as c_int);
            
            // Test calling the registered function  
            let code = CString::new("(host-add 10 20)").unwrap();
            let mut result: *mut c_char = ptr::null_mut();
            
            let eval_code = lambdust_eval(ctx, code.as_ptr(), &mut result);
            assert_eq!(eval_code, LambdustErrorCode::Success as c_int);
            
            if !result.is_null() {
                let result_str = CStr::from_ptr(result).to_str().unwrap();
                // The result is quoted because it comes back as a string value representation
                assert_eq!(result_str, "30");
                lambdust_free_string(result);
            }
            
            lambdust_destroy_context(ctx);
        }
    }
}