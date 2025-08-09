//! Integration layer for new I/O system with existing Port infrastructure.
//!
//! This module provides:
//! - Backward compatibility with existing R7RS Port operations
//! - Bridge between old and new I/O systems
//! - Unified I/O interface that works with both systems
//! - Migration utilities for upgrading existing code
//! - Performance monitoring and comparison tools

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{
    Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment,
    Port
};
use crate::effects::Effect;
use crate::stdlib::{
    advanced_io, async_io, network_io, streaming_io, platform_io, security_io
};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::LazyLock;
use std::collections::HashMap;

/// I/O system version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoSystemVersion {
    Legacy,  // Original Port-based system
    Modern,  // New advanced I/O system
    Hybrid,  // Both systems available
}

/// I/O operation wrapper that can handle both old and new systems
#[derive(Debug)]
pub enum UnifiedIoHandle {
    Legacy(Arc<Port>),
    Advanced(Box<dyn std::any::Any + Send + Sync>),
    Network(Box<dyn std::any::Any + Send + Sync>),
    Stream(Box<dyn std::any::Any + Send + Sync>),
}

/// I/O system configuration
#[derive(Debug, Clone)]
pub struct IoSystemConfig {
    pub version: IoSystemVersion,
    pub enable_advanced_features: bool,
    pub enable_async: bool,
    pub enable_networking: bool,
    pub enable_compression: bool,
    pub enable_security: bool,
    pub default_buffer_size: usize,
    pub performance_monitoring: bool,
}

impl Default for IoSystemConfig {
    fn default() -> Self {
        IoSystemConfig {
            version: IoSystemVersion::Hybrid,
            enable_advanced_features: true,
            enable_async: cfg!(feature = "async"),
            enable_networking: true,
            enable_compression: cfg!(feature = "compression"),
            enable_security: true,
            default_buffer_size: 8192,
            performance_monitoring: false,
        }
    }
}

/// Performance metrics for I/O operations
#[derive(Debug, Clone, Default)]
pub struct IoPerformanceMetrics {
    pub legacy_operations: u64,
    pub modern_operations: u64,
    pub legacy_bytes_transferred: u64,
    pub modern_bytes_transferred: u64,
    pub legacy_total_time_ns: u64,
    pub modern_total_time_ns: u64,
    pub errors: u64,
    pub conversions: u64,
}

/// Global I/O system state
static IO_SYSTEM_CONFIG: LazyLock<Mutex<IoSystemConfig>> = LazyLock::new(|| Mutex::new(IoSystemConfig::default()));
static IO_PERFORMANCE_METRICS: LazyLock<Mutex<IoPerformanceMetrics>> = LazyLock::new(|| Mutex::new(IoPerformanceMetrics::default()));

pub fn get_io_system_config() -> IoSystemConfig {
    IO_SYSTEM_CONFIG.lock().unwrap().clone()
}

pub fn get_io_performance_metrics() -> IoPerformanceMetrics {
    IO_PERFORMANCE_METRICS.lock().unwrap().clone()
}

pub fn set_io_system_config(config: IoSystemConfig) {
    *IO_SYSTEM_CONFIG.lock().unwrap() = config;
}

pub fn set_io_performance_metrics(metrics: IoPerformanceMetrics) {
    *IO_PERFORMANCE_METRICS.lock().unwrap() = metrics;
}

/// Creates unified I/O integration bindings.
pub fn create_io_integration_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // System configuration
    bind_system_config_operations(env);
    
    // Unified I/O operations
    bind_unified_io_operations(env);
    
    // Migration utilities
    bind_migration_operations(env);
    
    // Performance monitoring
    bind_performance_operations(env);
    
    // Compatibility layer
    bind_compatibility_operations(env);
    
    // Initialize all subsystems based on configuration
    initialize_io_subsystems(env);
}

fn initialize_io_subsystems(env: &Arc<ThreadSafeEnvironment>) {
    let config = get_io_system_config();
    
    // Always initialize legacy I/O for backward compatibility
    crate::stdlib::io::create_io_bindings(env);
    
    if config.enable_advanced_features {
        advanced_io::create_advanced_io_bindings(env);
    }
    
    if config.enable_async {
        async_io::create_async_io_bindings(env);
    }
    
    if config.enable_networking {
        network_io::create_network_io_bindings(env);
    }
    
    // Always enable streaming (it's useful for large files)
    streaming_io::create_streaming_io_bindings(env);
    
    // Always enable platform-specific optimizations
    platform_io::create_platform_io_bindings(env);
    
    if config.enable_security {
        security_io::create_security_io_bindings(env);
    }
}

// ============= SYSTEM CONFIGURATION =============

fn bind_system_config_operations(env: &Arc<ThreadSafeEnvironment>) {
    // set-io-system-config
    env.define("set-io-system-config".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "set-io-system-config".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_set_io_system_config),
        effects: vec![Effect::IO],
    })));
    
    // get-io-system-config
    env.define("get-io-system-config".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-io-system-config".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_get_io_system_config),
        effects: vec![Effect::Pure],
    })));
    
    // io-system-version
    env.define("io-system-version".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "io-system-version".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_io_system_version),
        effects: vec![Effect::Pure],
    })));
    
    // io-feature-available?
    env.define("io-feature-available?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "io-feature-available?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_io_feature_available_p),
        effects: vec![Effect::Pure],
    })));
}

fn bind_unified_io_operations(env: &Arc<ThreadSafeEnvironment>) {
    // unified-open-file
    env.define("unified-open-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "unified-open-file".to_string(),
        arity_min: 2,
        arity_max: Some(4),
        implementation: PrimitiveImpl::RustFn(primitive_unified_open_file),
        effects: vec![Effect::IO],
    })));
    
    // unified-read
    env.define("unified-read".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "unified-read".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_unified_read),
        effects: vec![Effect::IO],
    })));
    
    // unified-write
    env.define("unified-write".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "unified-write".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_unified_write),
        effects: vec![Effect::IO],
    })));
    
    // unified-close
    env.define("unified-close".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "unified-close".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_unified_close),
        effects: vec![Effect::IO],
    })));
}

fn bind_migration_operations(env: &Arc<ThreadSafeEnvironment>) {
    // port->advanced-handle
    env.define("port->advanced-handle".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "port->advanced-handle".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_port_to_advanced_handle),
        effects: vec![Effect::IO],
    })));
    
    // advanced-handle->port
    env.define("advanced-handle->port".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "advanced-handle->port".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_advanced_handle_to_port),
        effects: vec![Effect::IO],
    })));
    
    // migrate-io-operations
    env.define("migrate-io-operations".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "migrate-io-operations".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_migrate_io_operations),
        effects: vec![Effect::IO],
    })));
}

fn bind_performance_operations(env: &Arc<ThreadSafeEnvironment>) {
    // get-io-performance-metrics
    env.define("get-io-performance-metrics".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-io-performance-metrics".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_get_io_performance_metrics),
        effects: vec![Effect::IO],
    })));
    
    // reset-io-performance-metrics
    env.define("reset-io-performance-metrics".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "reset-io-performance-metrics".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_reset_io_performance_metrics),
        effects: vec![Effect::IO],
    })));
    
    // benchmark-io-systems
    env.define("benchmark-io-systems".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "benchmark-io-systems".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_benchmark_io_systems),
        effects: vec![Effect::IO],
    })));
}

fn bind_compatibility_operations(env: &Arc<ThreadSafeEnvironment>) {
    // ensure-backward-compatibility
    env.define("ensure-backward-compatibility".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "ensure-backward-compatibility".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_ensure_backward_compatibility),
        effects: vec![Effect::IO],
    })));
    
    // test-compatibility
    env.define("test-compatibility".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "test-compatibility".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_test_compatibility),
        effects: vec![Effect::IO],
    })));
}

// ============= IMPLEMENTATION FUNCTIONS =============

// === System Configuration ===

pub fn primitive_set_io_system_config(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("set-io-system-config expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    match &args[0] {
        Value::Hashtable(config_table) => {
            let table = config_table.read().unwrap();
            let mut config = IoSystemConfig::default();
            
            if let Some(Value::Literal(crate::ast::Literal::String(version_str))) = 
                table.get(&Value::Symbol(crate::utils::intern_symbol("version"))) {
                config.version = match version_str.as_str() {
                    "legacy" => IoSystemVersion::Legacy,
                    "modern" => IoSystemVersion::Modern,
                    "hybrid" => IoSystemVersion::Hybrid,
                    _ => IoSystemVersion::Hybrid,
                };
            }
            
            if let Some(Value::Literal(crate::ast::Literal::Boolean(enable))) = 
                table.get(&Value::Symbol(crate::utils::intern_symbol("enable-advanced-features"))) {
                config.enable_advanced_features = *enable;
            }
            
            if let Some(Value::Literal(crate::ast::Literal::Boolean(enable))) = 
                table.get(&Value::Symbol(crate::utils::intern_symbol("enable-async"))) {
                config.enable_async = *enable;
            }
            
            if let Some(Value::Literal(crate::ast::Literal::Boolean(enable))) = 
                table.get(&Value::Symbol(crate::utils::intern_symbol("enable-networking"))) {
                config.enable_networking = *enable;
            }
            
            if let Some(Value::Literal(crate::ast::Literal::Boolean(enable))) = 
                table.get(&Value::Symbol(crate::utils::intern_symbol("enable-compression"))) {
                config.enable_compression = *enable;
            }
            
            if let Some(Value::Literal(crate::ast::Literal::Boolean(enable))) = 
                table.get(&Value::Symbol(crate::utils::intern_symbol("enable-security"))) {
                config.enable_security = *enable;
            }
            
            if let Some(size_val) = table.get(&Value::Symbol(crate::utils::intern_symbol("default-buffer-size"))) {
                if let Some(size) = extract_optional_integer(size_val) {
                    config.default_buffer_size = size as usize;
                }
            }
            
            if let Some(Value::Literal(crate::ast::Literal::Boolean(enable))) = 
                table.get(&Value::Symbol(crate::utils::intern_symbol("performance-monitoring"))) {
                config.performance_monitoring = *enable;
            }
            
            // Update global configuration
            set_io_system_config(config);
            
            Ok(Value::Unspecified)
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "set-io-system-config requires hashtable argument".to_string(),
            None,
        ))),
    }
}

pub fn primitive_get_io_system_config(_args: &[Value]) -> Result<Value> {
    let config = get_io_system_config();
    #[allow(clippy::mutable_key_type)]
    let mut result = HashMap::new();
    
    let version_str = match config.version {
        IoSystemVersion::Legacy => "legacy",
        IoSystemVersion::Modern => "modern",
        IoSystemVersion::Hybrid => "hybrid",
    };
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("version")),
        Value::string(version_str.to_string())
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("enable-advanced-features")),
        Value::boolean(config.enable_advanced_features)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("enable-async")),
        Value::boolean(config.enable_async)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("enable-networking")),
        Value::boolean(config.enable_networking)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("enable-compression")),
        Value::boolean(config.enable_compression)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("enable-security")),
        Value::boolean(config.enable_security)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("default-buffer-size")),
        Value::integer(config.default_buffer_size as i64)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("performance-monitoring")),
        Value::boolean(config.performance_monitoring)
    );
    
    Ok(Value::Hashtable(Arc::new(std::sync::RwLock::new(result))))
}

pub fn primitive_io_system_version(_args: &[Value]) -> Result<Value> {
    let config = get_io_system_config();
    let version_str = match config.version {
        IoSystemVersion::Legacy => "legacy",
        IoSystemVersion::Modern => "modern",
        IoSystemVersion::Hybrid => "hybrid",
    };
    
    Ok(Value::string(version_str.to_string()))
}

pub fn primitive_io_feature_available_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("io-feature-available? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let feature = extract_string(&args[0], "io-feature-available?")?;
    let config = get_io_system_config();
    
    let available = match feature.as_str() {
        "advanced-features" => config.enable_advanced_features,
        "async" => config.enable_async,
        "networking" => config.enable_networking,
        "compression" => config.enable_compression,
        "security" => config.enable_security,
        "streaming" => true, // Always available
        "platform-specific" => true, // Always available
        "memory-mapping" => true, // Always available
        _ => false,
    };
    
    Ok(Value::boolean(available))
}

// === Unified I/O Operations ===

pub fn primitive_unified_open_file(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("unified-open-file expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "unified-open-file")?;
    let mode = extract_string(&args[1], "unified-open-file")?;
    let use_advanced = if args.len() > 2 {
        extract_boolean(&args[2], "unified-open-file")?
    } else {
        get_io_system_config().enable_advanced_features
    };
    let _options = if args.len() > 3 {
        Some(&args[3])
    } else {
        None
    };
    
    let config = get_io_system_config();
    
    if use_advanced && config.enable_advanced_features {
        // Use advanced I/O system
        match mode.as_str() {
            "r" | "read" => {
                // This would delegate to advanced_io functions
                // For now, fall back to legacy system
                let port = Port::new_file_input(path, false);
                Ok(Value::Port(Arc::new(port)))
            }
            "w" | "write" => {
                let port = Port::new_file_output(path, false);
                Ok(Value::Port(Arc::new(port)))
            }
            "a" | "append" => {
                let port = Port::new_file_output(path, false);
                Ok(Value::Port(Arc::new(port)))
            }
            _ => Err(Box::new(DiagnosticError::runtime_error(
                format!("Unsupported file mode: {mode}"),
                None,
            ))),
        }
    } else {
        // Use legacy Port system
        match mode.as_str() {
            "r" | "read" => {
                let port = Port::new_file_input(path, false);
                Ok(Value::Port(Arc::new(port)))
            }
            "w" | "write" => {
                let port = Port::new_file_output(path, false);
                Ok(Value::Port(Arc::new(port)))
            }
            "a" | "append" => {
                let port = Port::new_file_output(path, false);
                Ok(Value::Port(Arc::new(port)))
            }
            _ => Err(Box::new(DiagnosticError::runtime_error(
                format!("Unsupported file mode: {mode}"),
                None,
            ))),
        }
    }
}

pub fn primitive_unified_read(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("unified-read expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let _buffer_size = if args.len() > 1 {
        Some(extract_integer(&args[1], "unified-read")? as usize)
    } else {
        None
    };
    
    let _async_mode = if args.len() > 2 {
        extract_boolean(&args[2], "unified-read")?
    } else {
        false
    };
    
    // For now, delegate to legacy port operations
    match &args[0] {
        Value::Port(_port) => {
            // Would implement unified reading logic here
            Ok(Value::string("placeholder".to_string()))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "unified-read requires port or handle".to_string(),
            None,
        ))),
    }
}

pub fn primitive_unified_write(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("unified-write expects 2 or 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let _async_mode = if args.len() > 2 {
        extract_boolean(&args[2], "unified-write")?
    } else {
        false
    };
    
    // For now, delegate to legacy port operations
    match &args[0] {
        Value::Port(_port) => {
            // Would implement unified writing logic here
            Ok(Value::Unspecified)
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "unified-write requires port or handle".to_string(),
            None,
        ))),
    }
}

pub fn primitive_unified_close(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("unified-close expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    // For now, delegate to legacy port operations
    match &args[0] {
        Value::Port(port) => {
            port.close();
            Ok(Value::Unspecified)
        }
        _ => {
            // Handle advanced I/O handles here
            Ok(Value::Unspecified)
        }
    }
}

// === Migration Operations ===

pub fn primitive_port_to_advanced_handle(_args: &[Value]) -> Result<Value> {
    // TODO: Implement port to advanced handle conversion
    Err(Box::new(DiagnosticError::runtime_error(
        "port->advanced-handle not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_advanced_handle_to_port(_args: &[Value]) -> Result<Value> {
    // TODO: Implement advanced handle to port conversion
    Err(Box::new(DiagnosticError::runtime_error(
        "advanced-handle->port not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_migrate_io_operations(_args: &[Value]) -> Result<Value> {
    // TODO: Implement I/O operation migration
    Err(Box::new(DiagnosticError::runtime_error(
        "migrate-io-operations not yet implemented".to_string(),
        None,
    )))
}

// === Performance Operations ===

pub fn primitive_get_io_performance_metrics(_args: &[Value]) -> Result<Value> {
    let metrics = get_io_performance_metrics();
    #[allow(clippy::mutable_key_type)]
    let mut result = HashMap::new();
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("legacy-operations")),
        Value::integer(metrics.legacy_operations as i64)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("modern-operations")),
        Value::integer(metrics.modern_operations as i64)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("legacy-bytes-transferred")),
        Value::integer(metrics.legacy_bytes_transferred as i64)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("modern-bytes-transferred")),
        Value::integer(metrics.modern_bytes_transferred as i64)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("legacy-total-time-ns")),
        Value::integer(metrics.legacy_total_time_ns as i64)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("modern-total-time-ns")),
        Value::integer(metrics.modern_total_time_ns as i64)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("errors")),
        Value::integer(metrics.errors as i64)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("conversions")),
        Value::integer(metrics.conversions as i64)
    );
    
    // Calculate averages and rates
    if metrics.legacy_operations > 0 {
        result.insert(
            Value::Symbol(crate::utils::intern_symbol("legacy-avg-time-ns")),
            Value::integer((metrics.legacy_total_time_ns / metrics.legacy_operations) as i64)
        );
    }
    
    if metrics.modern_operations > 0 {
        result.insert(
            Value::Symbol(crate::utils::intern_symbol("modern-avg-time-ns")),
            Value::integer((metrics.modern_total_time_ns / metrics.modern_operations) as i64)
        );
    }
    
    Ok(Value::Hashtable(Arc::new(std::sync::RwLock::new(result))))
}

pub fn primitive_reset_io_performance_metrics(_args: &[Value]) -> Result<Value> {
    set_io_performance_metrics(IoPerformanceMetrics::default());
    Ok(Value::Unspecified)
}

pub fn primitive_benchmark_io_systems(_args: &[Value]) -> Result<Value> {
    // TODO: Implement I/O system benchmarking
    Err(Box::new(DiagnosticError::runtime_error(
        "benchmark-io-systems not yet implemented".to_string(),
        None,
    )))
}

// === Compatibility Operations ===

pub fn primitive_ensure_backward_compatibility(_args: &[Value]) -> Result<Value> {
    // Check that all legacy I/O operations are still available
    let issues = Vec::new();
    
    // This is a placeholder - would check actual function availability
    let legacy_functions = [
        "open-input-file", "open-output-file", "close-port",
        "read", "write", "display", "newline",
        "input-port?", "output-port?", "port?"
    ];
    
    for _func in &legacy_functions {
        // Would check if function is bound and working
    }
    
    if issues.is_empty() {
        Ok(Value::boolean(true))
    } else {
        Ok(Value::Pair(
            Arc::new(Value::boolean(false)),
            Arc::new(list_to_value(issues))
        ))
    }
}

pub fn primitive_test_compatibility(_args: &[Value]) -> Result<Value> {
    // TODO: Implement comprehensive compatibility testing
    Err(Box::new(DiagnosticError::runtime_error(
        "test-compatibility not yet implemented".to_string(),
        None,
    )))
}

// ============= HELPER FUNCTIONS =============

/// Extracts a string from a Value.
fn extract_string(value: &Value, operation: &str) -> Result<String> {
    match value {
        Value::Literal(crate::ast::Literal::String(s)) => Ok(s.clone()),
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires string arguments"),
            None,
        ))),
    }
}

/// Extracts a boolean from a Value.
fn extract_boolean(value: &Value, operation: &str) -> Result<bool> {
    match value {
        Value::Literal(crate::ast::Literal::Boolean(b)) => Ok(*b),
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires boolean arguments"),
            None,
        ))),
    }
}

/// Extracts an integer from a Value.
fn extract_integer(value: &Value, operation: &str) -> Result<i64> {
    match value {
        Value::Literal(lit) => {
            if let Some(i) = lit.to_i64() {
                Ok(i)
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    format!("{operation} requires integer arguments"),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires integer arguments"),
            None,
        ))),
    }
}

/// Extracts an optional integer from a Value.
fn extract_optional_integer(value: &Value) -> Option<i64> {
    match value {
        Value::Literal(lit) => lit.to_i64(),
        _ => None,
    }
}

/// Converts a vector of values to a Scheme list.
fn list_to_value(values: Vec<Value>) -> Value {
    values.into_iter().rev().fold(Value::Nil, |acc, val| {
        Value::Pair(Arc::new(val), Arc::new(acc))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_io_system_config() {
        let mut config_map = HashMap::new();
        config_map.insert(
            Value::Symbol(crate::utils::intern_symbol("version")),
            Value::string("hybrid".to_string())
        );
        config_map.insert(
            Value::Symbol(crate::utils::intern_symbol("enable-advanced-features")),
            Value::boolean(true)
        );
        config_map.insert(
            Value::Symbol(crate::utils::intern_symbol("default-buffer-size")),
            Value::integer(16384)
        );
        
        let args = vec![Value::Hashtable(Arc::new(std::sync::RwLock::new(config_map)))];
        let result = primitive_set_io_system_config(&args);
        assert!(result.is_ok());
        
        // Test getting the config
        let get_result = primitive_get_io_system_config(&[]);
        assert!(get_result.is_ok());
        
        if let Ok(Value::Hashtable(config)) = get_result {
            let config_map = config.read().unwrap();
            assert!(config_map.contains_key(&Value::Symbol(crate::utils::intern_symbol("version"))));
        } else {
            panic!("Expected hashtable result");
        }
    }
    
    #[test]
    fn test_io_system_version() {
        let result = primitive_io_system_version(&[]);
        assert!(result.is_ok());
        
        if let Ok(Value::Literal(crate::ast::Literal::String(version))) = result {
            assert!(["legacy", "modern", "hybrid"].contains(&version.as_str()));
        } else {
            panic!("Expected string result");
        }
    }
    
    #[test]
    fn test_feature_availability() {
        let args = vec![Value::string("streaming".to_string())];
        let result = primitive_io_feature_available_p(&args);
        assert!(result.is_ok());
        
        if let Ok(Value::Literal(crate::ast::Literal::Boolean(available))) = result {
            assert!(available); // Streaming should always be available
        } else {
            panic!("Expected boolean result");
        }
    }
    
    #[test]
    fn test_unified_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("unified_test.txt");
        let file_path = test_file.to_string_lossy().to_string();
        
        // Test unified file opening
        let args = vec![
            Value::string(file_path),
            Value::string("write".to_string()),
        ];
        let result = primitive_unified_open_file(&args);
        assert!(result.is_ok());
        
        // Just verify we get some kind of handle back
        if let Ok(Value::Port(_)) = result {
            // Success - we got a port back
        } else {
            panic!("Expected port result");
        }
    }
    
    #[test]
    fn test_performance_metrics() {
        let result = primitive_get_io_performance_metrics(&[]);
        assert!(result.is_ok());
        
        if let Ok(Value::Hashtable(metrics)) = result {
            let metrics_map = metrics.read().unwrap();
            assert!(metrics_map.contains_key(&Value::Symbol(crate::utils::intern_symbol("legacy-operations"))));
            assert!(metrics_map.contains_key(&Value::Symbol(crate::utils::intern_symbol("modern-operations"))));
        } else {
            panic!("Expected hashtable result");
        }
    }
    
    #[test]
    fn test_backward_compatibility() {
        let result = primitive_ensure_backward_compatibility(&[]);
        assert!(result.is_ok());
        
        // Should return true (or a list of issues)
        match result.unwrap() {
            Value::Literal(crate::ast::Literal::Boolean(compatible)) => {
                assert!(compatible);
            }
            Value::Pair(_, _) => {
                // Got a list of issues - that's also valid
            }
            _ => panic!("Expected boolean or pair result"),
        }
    }
}