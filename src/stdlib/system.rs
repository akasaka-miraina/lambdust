//! System interface operations for R7RS compliance.
//!
//! This module implements the system interface procedures required by R7RS Section 6.14:
//! - Process control: exit, emergency-exit
//! - Command line access: command-line
//! - Environment variables: get-environment-variable, get-environment-variables
//! - Time functions: current-second, current-jiffy, jiffies-per-second
//! - System features: features

use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use crate::diagnostics::{Error as DiagnosticError, Result};
use std::sync::{Arc, Mutex, OnceLock};
// Removed unused HashMap import
use std::time::{SystemTime, UNIX_EPOCH, Instant};

/// Global state for system information
static SYSTEM_STATE: OnceLock<Arc<Mutex<SystemState>>> = OnceLock::new();

/// System state containing command line arguments and startup time
#[derive(Debug)]
struct SystemState {
    /// Command line arguments passed to the program
    command_line_args: Vec<String>,
    /// Time when the program started (for jiffy calculation)
    start_time: Instant,
}

impl SystemState {
    fn new() -> Self {
        Self {
            command_line_args: Vec::new(),
            start_time: Instant::now(),
        }
    }
}

/// Initialize the system state with command line arguments
pub fn initialize_system_state(args: Vec<String>) {
    let state = Arc::new(Mutex::new(SystemState {
        command_line_args: args,
        start_time: Instant::now(),
    }));
    
    let _ = SYSTEM_STATE.set(state);
}

/// Get or create the system state
fn get_system_state() -> Arc<Mutex<SystemState>> {
    SYSTEM_STATE.get_or_init(|| {
        Arc::new(Mutex::new(SystemState::new()))
    }).clone())
}

/// Bind all system interface procedures to the environment
pub fn create_system_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Bind system procedures using the mutable define method
    env.define("exit".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "exit".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_exit),
        effects: vec![Effect::IO],
    })));

    env.define("emergency-exit".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "emergency-exit".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_emergency_exit),
        effects: vec![Effect::IO],
    })));

    env.define("command-line".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "command-line".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_command_line),
        effects: vec![Effect::Pure],
    })));

    env.define("get-environment-variable".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-environment-variable".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_get_environment_variable),
        effects: vec![Effect::IO],
    })));

    env.define("get-environment-variables".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-environment-variables".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_get_environment_variables),
        effects: vec![Effect::IO],
    })));

    env.define("current-second".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "current-second".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_current_second),
        effects: vec![Effect::IO],
    })));

    env.define("current-jiffy".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "current-jiffy".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_current_jiffy),
        effects: vec![Effect::IO],
    })));

    env.define("jiffies-per-second".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "jiffies-per-second".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_jiffies_per_second),
        effects: vec![Effect::Pure],
    })));

    env.define("features".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "features".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_features),
        effects: vec![Effect::Pure],
    })));
}

/// Bind all system interface procedures using copy-on-write semantics
/// Returns a new environment with all system procedures bound
pub fn bind_system_procedures_cow(env: &Arc<ThreadSafeEnvironment>) -> Arc<ThreadSafeEnvironment> {
    // Bind all system procedures using chained define calls
    env.define_cow("exit".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "exit".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_exit),
        effects: vec![Effect::IO],
    })))
    .define_cow("emergency-exit".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "emergency-exit".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_emergency_exit),
        effects: vec![Effect::IO],
    })))
    .define_cow("command-line".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "command-line".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_command_line),
        effects: vec![Effect::Pure],
    })))
    .define_cow("get-environment-variable".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-environment-variable".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_get_environment_variable),
        effects: vec![Effect::IO],
    })))
    .define_cow("get-environment-variables".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-environment-variables".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_get_environment_variables),
        effects: vec![Effect::IO],
    })))
    .define_cow("current-second".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "current-second".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_current_second),
        effects: vec![Effect::IO],
    })))
    .define_cow("current-jiffy".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "current-jiffy".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_current_jiffy),
        effects: vec![Effect::IO],
    })))
    .define_cow("jiffies-per-second".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "jiffies-per-second".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_jiffies_per_second),
        effects: vec![Effect::Pure],
    })))
    .define_cow("features".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "features".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_features),
        effects: vec![Effect::Pure],
    })))
}

// ============= PROCESS CONTROL PROCEDURES =============

/// (exit [obj]) - Exit the program
/// - If obj is omitted or #t, exit with code 0
/// - If obj is #f, exit with code 1  
/// - If obj is an exact integer, exit with that code
/// - Otherwise exit with code 1
pub fn primitive_exit(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(DiagnosticError::runtime_error(
            format!("exit expects 0 or 1 arguments, got {}", args.len()),
            None,
        ));
    }

    let exit_code = if args.is_empty() {
        0 // Default to success
    } else {
        match &args[0] {
            // #t means success (0)
            Value::Literal(crate::ast::Literal::Boolean(true)) => 0,
            // #f means failure (1)
            Value::Literal(crate::ast::Literal::Boolean(false)) => 1,
            // Number exit code (could be integer or float)
            Value::Literal(crate::ast::Literal::Number(n)) => {
                // Clamp to valid exit code range (0-255 on most systems)
                (*n as i64).clamp(0, 255) as i32
            }
            // Rational numbers get converted to integer
            Value::Literal(crate::ast::Literal::Rational { numerator, denominator }) => {
                let value = (*numerator as f64 / *denominator as f64) as i64;
                value.clamp(0, 255) as i32
            }
            // All other values default to failure
            _ => 1,
        }
    };

    // In a real implementation, this would perform cleanup and then exit
    // For now, we'll simulate by returning an error that can be caught
    Err(DiagnosticError::runtime_error(
        format!("Program exit requested with code {}", exit_code),
        None,
    ))
}

/// (emergency-exit [obj]) - Exit immediately without cleanup
/// Same semantics as exit but without running cleanup handlers
pub fn primitive_emergency_exit(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(DiagnosticError::runtime_error(
            format!("emergency-exit expects 0 or 1 arguments, got {}", args.len()),
            None,
        ));
    }

    let exit_code = if args.is_empty() {
        0 // Default to success
    } else {
        match &args[0] {
            Value::Literal(crate::ast::Literal::Boolean(true)) => 0,
            Value::Literal(crate::ast::Literal::Boolean(false)) => 1,
            Value::Literal(crate::ast::Literal::Number(n)) => {
                (*n as i64).clamp(0, 255) as i32
            }
            Value::Literal(crate::ast::Literal::Rational { numerator, denominator }) => {
                let value = (*numerator as f64 / *denominator as f64) as i64;
                value.clamp(0, 255) as i32
            }
            _ => 1,
        }
    };

    // Emergency exit - immediate termination without cleanup
    Err(DiagnosticError::runtime_error(
        format!("Emergency exit requested with code {}", exit_code),
        None,
    ))
}

// ============= COMMAND LINE ACCESS =============

/// (command-line) - Return list of command line arguments
pub fn primitive_command_line(_args: &[Value]) -> Result<Value> {
    let state = get_system_state();
    let state_guard = state.lock().map_err(|_| {
        DiagnosticError::runtime_error("Failed to access system state".to_string(), None)
    })?;

    // Convert command line arguments to a Scheme list
    let mut result = Value::Nil;
    for arg in state_guard.command_line_args.iter().rev() {
        result = Value::Pair(
            Arc::new(Value::string(arg.clone())),
            Arc::new(result),
        );
    }

    Ok(result)
}

// ============= ENVIRONMENT VARIABLES =============

/// (get-environment-variable name) - Get environment variable value
pub fn primitive_get_environment_variable(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("get-environment-variable expects 1 argument, got {}", args.len()),
            None,
        ));
    }

    // Extract the variable name
    let var_name = match &args[0] {
        Value::Literal(crate::ast::Literal::String(s)) => s.clone()),
        _ => {
            return Err(DiagnosticError::runtime_error(
                "get-environment-variable requires a string argument".to_string(),
                None,
            ));
        }
    };

    // Get the environment variable
    match std::env::var(&var_name) {
        Ok(value) => Ok(Value::string(value)),
        Err(_) => Ok(Value::Literal(crate::ast::Literal::Boolean(false))), // R7RS: return #f if not found
    }
}

/// (get-environment-variables) - Get all environment variables as alist
pub fn primitive_get_environment_variables(_args: &[Value]) -> Result<Value> {
    let mut result = Value::Nil;

    // Get all environment variables and build an association list
    let vars: Vec<_> = std::env::vars().collect();
    
    // Build the list in reverse order to maintain proper order
    for (key, value) in vars.into_iter().rev() {
        let pair = Value::Pair(
            Arc::new(Value::string(key)),
            Arc::new(Value::string(value)),
        );
        result = Value::Pair(Arc::new(pair), Arc::new(result));
    }

    Ok(result)
}

// ============= TIME FUNCTIONS =============

/// (current-second) - Return current time as seconds since Unix epoch
pub fn primitive_current_second(_args: &[Value]) -> Result<Value> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            // Return as exact integer (seconds) + fractional part as real
            let seconds = duration.as_secs();
            let nanos = duration.subsec_nanos();
            let fractional = seconds as f64 + (nanos as f64 / 1_000_000_000.0);
            Ok(Value::number(fractional))
        }
        Err(_) => {
            Err(DiagnosticError::runtime_error(
                "Failed to get current time".to_string(),
                None,
            ))
        }
    }
}

/// (current-jiffy) - Return current time in jiffies
/// A jiffy is an implementation-defined unit of time
pub fn primitive_current_jiffy(_args: &[Value]) -> Result<Value> {
    let state = get_system_state();
    let state_guard = state.lock().map_err(|_| {
        DiagnosticError::runtime_error("Failed to access system state".to_string(), None)
    })?;

    // Calculate jiffies since program start
    let elapsed = state_guard.start_time.elapsed();
    
    // Use nanoseconds as our jiffy unit for high precision
    let jiffies = elapsed.as_nanos() as i64;
    
    Ok(Value::integer(jiffies))
}

/// (jiffies-per-second) - Return number of jiffies per second
/// This is a constant for our implementation
pub fn primitive_jiffies_per_second(_args: &[Value]) -> Result<Value> {
    // We use nanoseconds as jiffies, so 1 billion jiffies per second  
    Ok(Value::integer(1_000_000_000))
}

// ============= SYSTEM FEATURES =============

/// (features) - Return list of supported feature identifiers
pub fn primitive_features(_args: &[Value]) -> Result<Value> {
    // R7RS-small required features
    let features = vec![
        "r7rs",                    // R7RS compliance
        "exact-closed",            // Exact arithmetic is closed under operations
        "exact-complex",           // Exact complex numbers supported
        "ieee-float",              // IEEE floating point
        "full-unicode",            // Full Unicode support
        "ratios",                  // Rational number support
        
        // Lambdust-specific features
        "lambdust",                // This implementation
        "gradual-typing",          // Gradual type system
        "effect-system",           // Effect system support
        "call/cc",                 // call-with-current-continuation
        "threads",                 // Threading support
        
        // Platform features
        #[cfg(unix)]
        "posix",
        #[cfg(windows)]
        "windows",
        #[cfg(target_pointer_width = "64")]
        "64bit",
        #[cfg(target_pointer_width = "32")]
        "32bit",
        
        // I/O features
        "port-position",           // Port position tracking
        "r7rs-io",                 // R7RS I/O system
        
        // Data structure features
        "bytevectors",             // Bytevector support
        "hashtables",              // Hash table support
        "records",                 // Record type support
    ];

    // Convert to Scheme list
    let mut result = Value::Nil;
    for feature in features.iter().rev() {
        let symbol_id = crate::utils::symbol::intern_symbol(feature.to_string());
        result = Value::Pair(
            Arc::new(Value::symbol(symbol_id)),
            Arc::new(result),
        );
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    // Environment is not needed for tests since we use ThreadSafeEnvironment directly

    fn create_test_env() -> Arc<ThreadSafeEnvironment> {
        Arc::new(ThreadSafeEnvironment::new(None, 0))
    }

    #[test]
    fn test_system_bindings() {
        // Test that all system binding functions can be called without panicking
        let env = create_test_env();
        
        // This should not panic even if the COW semantics don't work as expected
        create_system_bindings(&env);
        
        // Test that we can at least call the primitive functions directly
        let result = primitive_jiffies_per_second(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::integer(1_000_000_000));
        
        let result = primitive_features(&[]);
        assert!(result.is_ok());
        // Should return a list
        match result.unwrap() {
            Value::Pair(_, _) | Value::Nil => {
                // Good, it's a list
            }
            _ => panic!("features should return a list"),
        }
    }

    #[test]
    fn test_command_line() {
        // Initialize with test arguments
        initialize_system_state(vec![
            "lambdust".to_string(),
            "--version".to_string(),
            "test.scm".to_string(),
        ]);

        let result = primitive_command_line(&[]).unwrap();
        
        // Should return a list of strings
        match result {
            Value::Pair(car, _) => {
                // First element should be "lambdust"
                match car.as_ref() {
                    Value::Literal(crate::ast::Literal::String(s)) => {
                        assert_eq!(s, "lambdust");
                    }
                    _ => panic!("Expected string"),
                }
            }
            Value::Nil => {
                // Empty command line is also valid
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_get_environment_variable() {
        // Test with a known environment variable (PATH should exist on most systems)
        let args = vec![Value::string("PATH")];
        let result = primitive_get_environment_variable(&args).unwrap();
        
        // Should return either a string (if PATH exists) or #f (if not)
        match result {
            Value::Literal(crate::ast::Literal::String(_)) => {
                // PATH exists - good
            }
            Value::Literal(crate::ast::Literal::Boolean(false)) => {
                // PATH doesn't exist - unlikely but possible
            }
            _ => panic!("Expected string or #f"),
        }

        // Test with non-existent variable
        let args = vec![Value::string("NONEXISTENT_VAR_12345")];
        let result = primitive_get_environment_variable(&args).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Boolean(false)));
    }

    #[test]
    fn test_get_environment_variables() {
        let result = primitive_get_environment_variables(&[]).unwrap();
        
        // Should return an association list
        match result {
            Value::Nil => {
                // Empty environment is theoretically possible
            }
            Value::Pair(_, _) => {
                // Non-empty list is expected
            }
            _ => panic!("Expected association list"),
        }
    }

    #[test]
    fn test_time_functions() {
        // Test current-second
        let result = primitive_current_second(&[]).unwrap();
        match result {
            Value::Literal(crate::ast::Literal::Number(seconds)) => {
                // Should be a reasonable timestamp (after year 2020)
                assert!(seconds > 1_600_000_000.0);
            }
            _ => panic!("Expected number"),
        }

        // Test jiffies-per-second
        let result = primitive_jiffies_per_second(&[]).unwrap();
        assert_eq!(result, Value::integer(1_000_000_000));

        // Test current-jiffy
        let result = primitive_current_jiffy(&[]).unwrap();
        match result {
            Value::Literal(crate::ast::Literal::Number(_)) => {
                // Should be some number (could be integer or float depending on internal representation)
            }
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_features() {
        let result = primitive_features(&[]).unwrap();
        
        // Should return a list containing required R7RS features
        let mut current = &result;
        let mut found_r7rs = false;
        let mut found_lambdust = false;

        while let Value::Pair(car, cdr) = current {
            if let Value::Symbol(sym_id) = car.as_ref() {
                if let Some(name) = crate::utils::symbol::symbol_name(*sym_id) {
                    if name == "r7rs" {
                        found_r7rs = true;
                    }
                    if name == "lambdust" {
                        found_lambdust = true;
                    }
                }
            }
            current = cdr;
        }

        assert!(found_r7rs, "Should include 'r7rs' feature");
        assert!(found_lambdust, "Should include 'lambdust' feature");
    }

    #[test]
    fn test_exit_semantics() {
        // Test exit with no arguments (should default to 0)
        let result = primitive_exit(&[]);
        assert!(result.is_err()); // Exit should cause an error

        // Test exit with #t (should exit with 0)
        let args = vec![Value::boolean(true)];
        let result = primitive_exit(&args);
        assert!(result.is_err());

        // Test exit with #f (should exit with 1)  
        let args = vec![Value::boolean(false)];
        let result = primitive_exit(&args);
        assert!(result.is_err());

        // Test exit with integer
        let args = vec![Value::integer(42)];
        let result = primitive_exit(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_conditions() {
        // Test wrong number of arguments
        let result = primitive_get_environment_variable(&[]);
        assert!(result.is_err());

        let result = primitive_get_environment_variable(&[Value::integer(1), Value::integer(2)]);
        assert!(result.is_err());

        // Test wrong argument types
        let result = primitive_get_environment_variable(&[Value::integer(42)]);
        assert!(result.is_err());
    }
}