//! Security and sandboxing features for I/O operations.
//!
//! This module provides:
//! - File access control and permission management
//! - Resource limits (file descriptors, bandwidth, disk space)
//! - Chroot/jail support for process isolation
//! - I/O operation auditing and logging
//! - Secure file handling with validation

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{
    Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment
};
use crate::effects::Effect;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::sync::LazyLock;

#[cfg(all(unix, feature = "advanced-io"))]
use nix::unistd::{chroot, chdir};
#[cfg(unix)]
/// Security policy for I/O operations
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub allowed_paths: HashSet<PathBuf>,
    pub forbidden_paths: HashSet<PathBuf>,
    pub max_file_size: Option<u64>,
    pub max_bandwidth: Option<u64>, // bytes per second
    pub max_open_files: Option<usize>,
    pub audit_enabled: bool,
    pub strict_mode: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        SecurityPolicy {
            allowed_paths: HashSet::new(),
            forbidden_paths: HashSet::new(),
            max_file_size: Some(100 * 1024 * 1024), // 100MB default
            max_bandwidth: Some(10 * 1024 * 1024), // 10MB/s default
            max_open_files: Some(1024),
            audit_enabled: true,
            strict_mode: false,
        }
    }
}

/// Resource usage tracking
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub open_files: usize,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub operations_count: u64,
    pub last_reset: Instant,
    pub bandwidth_window: Duration,
    pub recent_transfers: Vec<(Instant, u64)>, // (timestamp, bytes)
}

impl Default for ResourceUsage {
    fn default() -> Self {
        ResourceUsage {
            open_files: 0,
            bytes_read: 0,
            bytes_written: 0,
            operations_count: 0,
            last_reset: Instant::now(),
            bandwidth_window: Duration::from_secs(1),
            recent_transfers: Vec::new(),
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: Instant,
    pub operation: String,
    pub path: Option<PathBuf>,
    pub user_data: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Security manager for I/O operations
#[derive(Debug, Default)]
pub struct SecurityManager {
    pub policy: Arc<RwLock<SecurityPolicy>>,
    pub usage: Arc<Mutex<ResourceUsage>>,
    pub audit_log: Arc<Mutex<Vec<AuditEntry>>>,
    pub sandbox_active: bool,
    pub chroot_path: Option<PathBuf>,
}

impl SecurityManager {
    pub fn new() -> Self {
        SecurityManager {
            policy: Arc::new(RwLock::new(SecurityPolicy::default())),
            usage: Arc::new(Mutex::new(ResourceUsage::default())),
            audit_log: Arc::new(Mutex::new(Vec::new())),
            sandbox_active: false,
            chroot_path: None,
        }
    }
    
    pub fn with_policy(self, policy: SecurityPolicy) -> Self {
        *self.policy.write().unwrap() = policy;
        self
    }
    
    pub fn check_path_access(&self, path: &Path, operation: &str) -> Result<()> {
        let policy = self.policy.read().unwrap();
        
        // Normalize path
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                if policy.strict_mode {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        format!("Cannot access non-existent path: {}", path.display()),
                        None,
                    )));
                } else {
                    path.to_path_buf()
                }
            }
        };
        
        // Check forbidden paths first
        for forbidden in &policy.forbidden_paths {
            if canonical_path.starts_with(forbidden) {
                self.log_audit_entry(AuditEntry {
                    timestamp: Instant::now(),
                    operation: operation.to_string(),
                    path: Some(canonical_path.clone()),
                    user_data: None,
                    success: false,
                    error_message: Some("Path is forbidden".to_string()),
                });
                
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("Access denied to forbidden path: {}", canonical_path.display()),
                    None,
                )));
            }
        }
        
        // Check allowed paths
        if !policy.allowed_paths.is_empty() {
            let mut allowed = false;
            for allowed_path in &policy.allowed_paths {
                if canonical_path.starts_with(allowed_path) {
                    allowed = true;
                    break;
                }
            }
            
            if !allowed {
                self.log_audit_entry(AuditEntry {
                    timestamp: Instant::now(),
                    operation: operation.to_string(),
                    path: Some(canonical_path.clone()),
                    user_data: None,
                    success: false,
                    error_message: Some("Path not in allowed list".to_string()),
                });
                
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("Access denied to path not in allowed list: {}", canonical_path.display()),
                    None,
                )));
            }
        }
        
        Ok(())
    }
    
    pub fn check_file_size_limit(&self, size: u64) -> Result<()> {
        let policy = self.policy.read().unwrap();
        
        if let Some(max_size) = policy.max_file_size {
            if size > max_size {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("File size {size} exceeds limit {max_size}"),
                    None,
                )));
            }
        }
        
        Ok(())
    }
    
    pub fn check_bandwidth_limit(&self, bytes: u64) -> Result<()> {
        let policy = self.policy.read().unwrap();
        
        if let Some(max_bandwidth) = policy.max_bandwidth {
            let mut usage = self.usage.lock().unwrap();
            let now = Instant::now();
            
            // Clean up old entries
            let bandwidth_window = usage.bandwidth_window;
            usage.recent_transfers.retain(|(timestamp, _)| {
                now.duration_since(*timestamp) <= bandwidth_window
            });
            
            // Calculate current bandwidth usage
            let current_usage: u64 = usage.recent_transfers.iter()
                .map(|(_, bytes)| *bytes)
                .sum();
            
            if current_usage + bytes > max_bandwidth {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("Bandwidth limit exceeded: {current_usage} + {bytes} > {max_bandwidth}"),
                    None,
                )));
            }
            
            // Record this transfer
            usage.recent_transfers.push((now, bytes));
        }
        
        Ok(())
    }
    
    pub fn check_open_file_limit(&self) -> Result<()> {
        let policy = self.policy.read().unwrap();
        let usage = self.usage.lock().unwrap();
        
        if let Some(max_files) = policy.max_open_files {
            if usage.open_files >= max_files {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("Open file limit exceeded: {open_files} >= {max_files}", open_files = usage.open_files),
                    None,
                )));
            }
        }
        
        Ok(())
    }
    
    pub fn track_file_opened(&self) {
        let mut usage = self.usage.lock().unwrap();
        usage.open_files += 1;
        usage.operations_count += 1;
    }
    
    pub fn track_file_closed(&self) {
        let mut usage = self.usage.lock().unwrap();
        if usage.open_files > 0 {
            usage.open_files -= 1;
        }
    }
    
    pub fn track_bytes_read(&self, bytes: u64) {
        let mut usage = self.usage.lock().unwrap();
        usage.bytes_read += bytes;
        usage.operations_count += 1;
    }
    
    pub fn track_bytes_written(&self, bytes: u64) {
        let mut usage = self.usage.lock().unwrap();
        usage.bytes_written += bytes;
        usage.operations_count += 1;
    }
    
    fn log_audit_entry(&self, entry: AuditEntry) {
        let policy = self.policy.read().unwrap();
        if policy.audit_enabled {
            let mut audit_log = self.audit_log.lock().unwrap();
            audit_log.push(entry);
            
            // Limit audit log size
            if audit_log.len() > 10000 {
                audit_log.drain(0..1000);
            }
        }
    }
    
    pub fn enable_sandbox(&mut self, chroot_path: Option<PathBuf>) -> Result<()> {
        #[cfg(all(unix, feature = "advanced-io"))]
        {
            if let Some(ref path) = chroot_path {
                // Change to chroot directory first
                chdir(path).map_err(|e| {
                    DiagnosticError::runtime_error(
                        format!("Cannot change to chroot directory '{}': {e}", path.display()),
                        None,
                    )
                })?;
                
                // Apply chroot
                chroot(path).map_err(|e| {
                    DiagnosticError::runtime_error(
                        format!("Cannot apply chroot to '{}': {e}", path.display()),
                        None,
                    )
                })?;
                
                self.chroot_path = Some(path.clone());
            }
            
            self.sandbox_active = true;
            Ok(())
        }
        
        // Fallback for Unix systems without advanced-io feature - sandbox is not available
        #[cfg(all(unix, not(feature = "advanced-io")))]
        {
            if chroot_path.is_some() {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "Sandboxing with chroot requires 'advanced-io' feature".to_string(),
                    None,
                )));
            }
            self.sandbox_active = false;
            Ok(())
        }
        
        #[cfg(not(unix))]
        {
            if chroot_path.is_some() {
                return Err(DiagnosticError::runtime_error(
                    "Chroot sandboxing not supported on this platform".to_string(),
                    None,
                ));
            }
            
            // Enable basic sandboxing without chroot
            self.sandbox_active = true;
            Ok(())
        }
    }
}

/// Global security manager instance
static SECURITY_MANAGER: LazyLock<Mutex<SecurityManager>> = LazyLock::new(|| Mutex::new(SecurityManager::new()));

pub fn get_security_manager() -> &'static Mutex<SecurityManager> {
    &SECURITY_MANAGER
}

/// Creates security and sandboxing operation bindings.
pub fn create_security_io_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Security policy management
    bind_security_policy_operations(env);
    
    // Resource management
    bind_resource_management_operations(env);
    
    // Sandboxing operations
    bind_sandbox_operations(env);
    
    // Auditing operations
    bind_audit_operations(env);
    
    // Secure file operations
    bind_secure_file_operations(env);
}

// ============= SECURITY POLICY OPERATIONS =============

fn bind_security_policy_operations(env: &Arc<ThreadSafeEnvironment>) {
    // set-security-policy
    env.define("set-security-policy".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "set-security-policy".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_set_security_policy),
        effects: vec![Effect::IO],
    })));
    
    // get-security-policy
    env.define("get-security-policy".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-security-policy".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_get_security_policy),
        effects: vec![Effect::IO],
    })));
    
    // add-allowed-path
    env.define("add-allowed-path".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "add-allowed-path".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_add_allowed_path),
        effects: vec![Effect::IO],
    })));
    
    // add-forbidden-path
    env.define("add-forbidden-path".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "add-forbidden-path".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_add_forbidden_path),
        effects: vec![Effect::IO],
    })));
    
    // check-path-access
    env.define("check-path-access".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "check-path-access".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_check_path_access),
        effects: vec![Effect::IO],
    })));
}

fn bind_resource_management_operations(env: &Arc<ThreadSafeEnvironment>) {
    // set-resource-limits
    env.define("set-resource-limits".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "set-resource-limits".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_set_resource_limits),
        effects: vec![Effect::IO],
    })));
    
    // get-resource-usage
    env.define("get-resource-usage".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-resource-usage".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_get_resource_usage),
        effects: vec![Effect::IO],
    })));
    
    // reset-resource-counters
    env.define("reset-resource-counters".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "reset-resource-counters".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_reset_resource_counters),
        effects: vec![Effect::IO],
    })));
}

fn bind_sandbox_operations(env: &Arc<ThreadSafeEnvironment>) {
    // enable-sandbox
    env.define("enable-sandbox".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "enable-sandbox".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_enable_sandbox),
        effects: vec![Effect::IO],
    })));
    
    // sandbox-active?
    env.define("sandbox-active?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "sandbox-active?".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_sandbox_active_p),
        effects: vec![Effect::IO],
    })));
    
    // create-secure-environment
    env.define("create-secure-environment".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "create-secure-environment".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_create_secure_environment),
        effects: vec![Effect::IO],
    })));
}

fn bind_audit_operations(env: &Arc<ThreadSafeEnvironment>) {
    // enable-audit-logging
    env.define("enable-audit-logging".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "enable-audit-logging".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_enable_audit_logging),
        effects: vec![Effect::IO],
    })));
    
    // get-audit-log
    env.define("get-audit-log".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-audit-log".to_string(),
        arity_min: 0,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_get_audit_log),
        effects: vec![Effect::IO],
    })));
    
    // clear-audit-log
    env.define("clear-audit-log".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "clear-audit-log".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_clear_audit_log),
        effects: vec![Effect::IO],
    })));
}

fn bind_secure_file_operations(env: &Arc<ThreadSafeEnvironment>) {
    // secure-file-read
    env.define("secure-file-read".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "secure-file-read".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_secure_file_read),
        effects: vec![Effect::IO],
    })));
    
    // secure-file-write
    env.define("secure-file-write".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "secure-file-write".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_secure_file_write),
        effects: vec![Effect::IO],
    })));
    
    // validate-file-path
    env.define("validate-file-path".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "validate-file-path".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_validate_file_path),
        effects: vec![Effect::Pure],
    })));
}

// ============= IMPLEMENTATION FUNCTIONS =============

// === Security Policy Operations ===

pub fn primitive_set_security_policy(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("set-security-policy expects 1 argument, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    // Extract policy from hashtable
    match &args[0] {
        Value::Hashtable(policy_table) => {
            let table = policy_table.read().unwrap();
            let mut policy = SecurityPolicy::default();
            
            // Parse policy settings
            if let Some(Value::Literal(crate::ast::Literal::Boolean(strict))) = 
                table.get(&Value::Symbol(crate::utils::intern_symbol("strict-mode"))) {
                policy.strict_mode = *strict;
            }
            
            if let Some(Value::Literal(crate::ast::Literal::Boolean(audit))) = 
                table.get(&Value::Symbol(crate::utils::intern_symbol("audit-enabled"))) {
                policy.audit_enabled = *audit;
            }
            
            if let Some(max_size_val) = table.get(&Value::Symbol(crate::utils::intern_symbol("max-file-size"))) {
                if let Some(size) = extract_optional_integer(max_size_val) {
                    policy.max_file_size = Some(size as u64);
                }
            }
            
            if let Some(max_bandwidth_val) = table.get(&Value::Symbol(crate::utils::intern_symbol("max-bandwidth"))) {
                if let Some(bandwidth) = extract_optional_integer(max_bandwidth_val) {
                    policy.max_bandwidth = Some(bandwidth as u64);
                }
            }
            
            if let Some(max_files_val) = table.get(&Value::Symbol(crate::utils::intern_symbol("max-open-files"))) {
                if let Some(files) = extract_optional_integer(max_files_val) {
                    policy.max_open_files = Some(files as usize);
                }
            }
            
            // Update security manager
            let security_manager = get_security_manager();
            let manager = security_manager.lock().unwrap();
            *manager.policy.write().unwrap() = policy;
            
            Ok(Value::Unspecified)
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "set-security-policy requires hashtable argument".to_string(),
            None,
        ))),
    }
}

pub fn primitive_get_security_policy(_args: &[Value]) -> Result<Value> {
    let security_manager = get_security_manager();
    let manager = security_manager.lock().unwrap();
    let policy = manager.policy.read().unwrap();
    
    #[allow(clippy::mutable_key_type)]
    let mut result = HashMap::new();
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("strict-mode")),
        Value::boolean(policy.strict_mode)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("audit-enabled")),
        Value::boolean(policy.audit_enabled)
    );
    
    if let Some(max_size) = policy.max_file_size {
        result.insert(
            Value::Symbol(crate::utils::intern_symbol("max-file-size")),
            Value::integer(max_size as i64)
        );
    }
    
    if let Some(max_bandwidth) = policy.max_bandwidth {
        result.insert(
            Value::Symbol(crate::utils::intern_symbol("max-bandwidth")),
            Value::integer(max_bandwidth as i64)
        );
    }
    
    if let Some(max_files) = policy.max_open_files {
        result.insert(
            Value::Symbol(crate::utils::intern_symbol("max-open-files")),
            Value::integer(max_files as i64)
        );
    }
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("allowed-paths")),
        {
            let paths: Vec<Value> = policy.allowed_paths.iter()
                .map(|p| Value::string(p.to_string_lossy().to_string()))
                .collect();
            list_to_value(paths)
        }
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("forbidden-paths")),
        {
            let paths: Vec<Value> = policy.forbidden_paths.iter()
                .map(|p| Value::string(p.to_string_lossy().to_string()))
                .collect();
            list_to_value(paths)
        }
    );
    
    Ok(Value::Hashtable(Arc::new(std::sync::RwLock::new(result))))
}

pub fn primitive_add_allowed_path(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("add-allowed-path expects 1 argument, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "add-allowed-path")?;
    let path_buf = PathBuf::from(path);
    
    let security_manager = get_security_manager();
    let manager = security_manager.lock().unwrap();
    let mut policy = manager.policy.write().unwrap();
    policy.allowed_paths.insert(path_buf);
    
    Ok(Value::Unspecified)
}

pub fn primitive_add_forbidden_path(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("add-forbidden-path expects 1 argument, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "add-forbidden-path")?;
    let path_buf = PathBuf::from(path);
    
    let security_manager = get_security_manager();
    let manager = security_manager.lock().unwrap();
    let mut policy = manager.policy.write().unwrap();
    policy.forbidden_paths.insert(path_buf);
    
    Ok(Value::Unspecified)
}

pub fn primitive_check_path_access(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("check-path-access expects 2 arguments, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "check-path-access")?;
    let operation = extract_string(&args[1], "check-path-access")?;
    
    let security_manager = get_security_manager();
    let manager = security_manager.lock().unwrap();
    
    match manager.check_path_access(Path::new(&path), &operation) {
        Ok(()) => Ok(Value::boolean(true)),
        Err(_) => Ok(Value::boolean(false)),
    }
}

// === Resource Management Operations ===

pub fn primitive_set_resource_limits(_args: &[Value]) -> Result<Value> {
    // TODO: Implement resource limit setting
    Err(Box::new(DiagnosticError::runtime_error(
        "set-resource-limits not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_get_resource_usage(_args: &[Value]) -> Result<Value> {
    let security_manager = get_security_manager();
    let manager = security_manager.lock().unwrap();
    let usage = manager.usage.lock().unwrap();
    
    #[allow(clippy::mutable_key_type)]
    let mut result = HashMap::new();
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("open-files")),
        Value::integer(usage.open_files as i64)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("bytes-read")),
        Value::integer(usage.bytes_read as i64)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("bytes-written")),
        Value::integer(usage.bytes_written as i64)
    );
    
    result.insert(
        Value::Symbol(crate::utils::intern_symbol("operations-count")),
        Value::integer(usage.operations_count as i64)
    );
    
    Ok(Value::Hashtable(Arc::new(std::sync::RwLock::new(result))))
}

pub fn primitive_reset_resource_counters(_args: &[Value]) -> Result<Value> {
    let security_manager = get_security_manager();
    let manager = security_manager.lock().unwrap();
    let mut usage = manager.usage.lock().unwrap();
    
    usage.bytes_read = 0;
    usage.bytes_written = 0;
    usage.operations_count = 0;
    usage.last_reset = Instant::now();
    usage.recent_transfers.clear();
    
    Ok(Value::Unspecified)
}

// === Sandbox Operations ===

pub fn primitive_enable_sandbox(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("enable-sandbox expects 0 or 1 arguments, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let chroot_path = if args.len() == 1 {
        Some(PathBuf::from(extract_string(&args[0], "enable-sandbox")?))
    } else {
        None
    };
    
    let security_manager = get_security_manager();
    let mut manager = security_manager.lock().unwrap();
    
    match manager.enable_sandbox(chroot_path) {
        Ok(()) => Ok(Value::Unspecified),
        Err(e) => Err(e),
    }
}

pub fn primitive_sandbox_active_p(_args: &[Value]) -> Result<Value> {
    let security_manager = get_security_manager();
    let manager = security_manager.lock().unwrap();
    Ok(Value::boolean(manager.sandbox_active))
}

pub fn primitive_create_secure_environment(_args: &[Value]) -> Result<Value> {
    // TODO: Implement secure environment creation
    Err(Box::new(DiagnosticError::runtime_error(
        "create-secure-environment not yet implemented".to_string(),
        None,
    )))
}

// === Audit Operations ===

pub fn primitive_enable_audit_logging(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("enable-audit-logging expects 1 argument, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let enabled = extract_boolean(&args[0], "enable-audit-logging")?;
    
    let security_manager = get_security_manager();
    let manager = security_manager.lock().unwrap();
    let mut policy = manager.policy.write().unwrap();
    policy.audit_enabled = enabled;
    
    Ok(Value::Unspecified)
}

pub fn primitive_get_audit_log(args: &[Value]) -> Result<Value> {
    if args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("get-audit-log expects 0 to 2 arguments, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let limit = if !args.is_empty() {
        Some(extract_integer(&args[0], "get-audit-log")? as usize)
    } else {
        None
    };
    
    let _filter = if args.len() > 1 {
        Some(extract_string(&args[1], "get-audit-log")?)
    } else {
        None
    };
    
    let security_manager = get_security_manager();
    let manager = security_manager.lock().unwrap();
    let audit_log = manager.audit_log.lock().unwrap();
    
    let entries_to_return = if let Some(limit) = limit {
        audit_log.iter().rev().take(limit).collect::<Vec<_>>()
    } else {
        audit_log.iter().collect::<Vec<_>>()
    };
    
    let audit_entries: Vec<Value> = entries_to_return.into_iter().rev().map(|entry| {
        #[allow(clippy::mutable_key_type)]
        let mut entry_map = HashMap::new();
        
        entry_map.insert(
            Value::Symbol(crate::utils::intern_symbol("operation")),
            Value::string(entry.operation.clone())
        );
        
        if let Some(ref path) = entry.path {
            entry_map.insert(
                Value::Symbol(crate::utils::intern_symbol("path")),
                Value::string(path.to_string_lossy().to_string())
            );
        }
        
        entry_map.insert(
            Value::Symbol(crate::utils::intern_symbol("success")),
            Value::boolean(entry.success)
        );
        
        if let Some(ref error) = entry.error_message {
            entry_map.insert(
                Value::Symbol(crate::utils::intern_symbol("error")),
                Value::string(error.clone())
            );
        }
        
        Value::Hashtable(Arc::new(std::sync::RwLock::new(entry_map)))
    }).collect();
    
    Ok(list_to_value(audit_entries))
}

pub fn primitive_clear_audit_log(_args: &[Value]) -> Result<Value> {
    let security_manager = get_security_manager();
    let manager = security_manager.lock().unwrap();
    let mut audit_log = manager.audit_log.lock().unwrap();
    audit_log.clear();
    
    Ok(Value::Unspecified)
}

// === Secure File Operations ===

pub fn primitive_secure_file_read(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("secure-file-read expects 1 or 2 arguments, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "secure-file-read")?;
    let as_binary = if args.len() > 1 {
        extract_boolean(&args[1], "secure-file-read")?
    } else {
        false
    };
    
    let security_manager = get_security_manager();
    let manager = security_manager.lock().unwrap();
    
    // Check path access
    manager.check_path_access(Path::new(&path), "read")?;
    
    // Check file size limit
    match std::fs::metadata(&path) {
        Ok(metadata) => {
            manager.check_file_size_limit(metadata.len())?;
            manager.check_bandwidth_limit(metadata.len())?;
        }
        Err(e) => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("Cannot access file '{path}': {e}"),
                None,
            )));
        }
    }
    
    // Read file
    let result = if as_binary {
        match std::fs::read(&path) {
            Ok(data) => {
                manager.track_bytes_read(data.len() as u64);
                Ok(Value::bytevector(data))
            }
            Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                format!("Cannot read file '{path}': {e}"),
                None,
            ))),
        }
    } else {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                manager.track_bytes_read(content.len() as u64);
                Ok(Value::string(content))
            }
            Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                format!("Cannot read file '{path}': {e}"),
                None,
            ))),
        }
    };
    
    // Log audit entry
    manager.log_audit_entry(AuditEntry {
        timestamp: Instant::now(),
        operation: "read".to_string(),
        path: Some(PathBuf::from(&path)),
        user_data: None,
        success: result.is_ok(),
        error_message: result.as_ref().err().map(|e| e.to_string()),
    });
    
    result
}

pub fn primitive_secure_file_write(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("secure-file-write expects 2 or 3 arguments, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "secure-file-write")?;
    let append = if args.len() > 2 {
        extract_boolean(&args[2], "secure-file-write")?
    } else {
        false
    };
    
    let (data, data_len) = match &args[1] {
        Value::Literal(crate::ast::Literal::String(s)) => (s.as_bytes().to_vec(), s.len()),
        Value::Literal(crate::ast::Literal::Bytevector(bv)) => (bv.clone(), bv.len()),
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                "secure-file-write requires string or bytevector data".to_string(),
                None,
            )));
        }
    };
    
    let security_manager = get_security_manager();
    let manager = security_manager.lock().unwrap();
    
    // Check path access
    manager.check_path_access(Path::new(&path), "write")?;
    
    // Check file size and bandwidth limits
    manager.check_file_size_limit(data_len as u64)?;
    manager.check_bandwidth_limit(data_len as u64)?;
    
    // Write file
    let write_result = if append {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .and_then(|mut file| {
                use std::io::Write;
                file.write_all(&data)
            })
    } else {
        std::fs::write(&path, &data)
    };
    
    let result = match write_result {
        Ok(()) => {
            manager.track_bytes_written(data_len as u64);
            Ok(Value::Unspecified)
        }
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Cannot write to file '{path}': {e}"),
            None,
        ))),
    };
    
    // Log audit entry
    manager.log_audit_entry(AuditEntry {
        timestamp: Instant::now(),
        operation: if append { "append" } else { "write" }.to_string(),
        path: Some(PathBuf::from(&path)),
        user_data: Some(format!("{data_len} bytes")),
        success: result.is_ok(),
        error_message: result.as_ref().err().map(|e| e.to_string()),
    });
    
    result
}

pub fn primitive_validate_file_path(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("validate-file-path expects 1 or 2 arguments, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "validate-file-path")?;
    let strict = if args.len() > 1 {
        extract_boolean(&args[1], "validate-file-path")?
    } else {
        true
    };
    
    // Basic path validation
    let path_obj = Path::new(&path);
    
    // Check for dangerous patterns
    let path_str = path_obj.to_string_lossy();
    
    // Check for path traversal attempts
    if path_str.contains("..") {
        return Ok(Value::boolean(false));
    }
    
    // Check for absolute paths in strict mode
    if strict && path_obj.is_absolute() {
        return Ok(Value::boolean(false));
    }
    
    // Check for special characters
    if path_str.contains('\0') {
        return Ok(Value::boolean(false));
    }
    
    // Additional platform-specific checks
    #[cfg(windows)]
    {
        // Check for Windows reserved names
        let filename = path_obj.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        
        let reserved_names = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", 
                             "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", 
                             "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
        
        if reserved_names.iter().any(|&name| filename.eq_ignore_ascii_case(name)) {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
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
    fn test_security_policy() {
        let mut policy_map = HashMap::new();
        policy_map.insert(
            Value::Symbol(crate::utils::intern_symbol("strict-mode")),
            Value::boolean(true)
        );
        policy_map.insert(
            Value::Symbol(crate::utils::intern_symbol("audit-enabled")),
            Value::boolean(true)
        );
        policy_map.insert(
            Value::Symbol(crate::utils::intern_symbol("max-file-size")),
            Value::integer(1024 * 1024) // 1MB
        );
        
        let args = vec![Value::Hashtable(Arc::new(std::sync::RwLock::new(policy_map)))];
        let result = primitive_set_security_policy(&args);
        assert!(result.is_ok());
        
        // Test getting the policy
        let get_result = primitive_get_security_policy(&[]);
        assert!(get_result.is_ok());
    }
    
    #[test]
    fn test_path_validation() {
        // Valid path
        let args = vec![Value::string("valid/path.txt".to_string())];
        let result = primitive_validate_file_path(&args);
        assert!(result.is_ok());
        if let Ok(Value::Literal(crate::ast::Literal::Boolean(valid))) = result {
            assert!(valid);
        }
        
        // Invalid path with traversal
        let args = vec![Value::string("../../../etc/passwd".to_string())];
        let result = primitive_validate_file_path(&args);
        assert!(result.is_ok());
        if let Ok(Value::Literal(crate::ast::Literal::Boolean(valid))) = result {
            assert!(!valid);
        }
    }
    
    #[test]
    fn test_resource_tracking() {
        let security_manager = get_security_manager();
        let manager = security_manager.lock().unwrap();
        
        // Test tracking
        manager.track_file_opened();
        manager.track_bytes_read(1024);
        manager.track_bytes_written(512);
        
        let usage = manager.usage.lock().unwrap();
        assert_eq!(usage.open_files, 1);
        assert_eq!(usage.bytes_read, 1024);
        assert_eq!(usage.bytes_written, 512);
        assert_eq!(usage.operations_count, 3);
    }
    
    #[test]
    fn test_secure_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("secure_test.txt");
        let file_path = test_file.to_string_lossy().to_string();
        
        // Add allowed path
        let args = vec![Value::string(temp_dir.path().to_string_lossy().to_string())];
        let result = primitive_add_allowed_path(&args);
        assert!(result.is_ok());
        
        // Test secure write
        let write_args = vec![
            Value::string(file_path.clone()),
            Value::string("Hello, secure world!".to_string()),
        ];
        let result = primitive_secure_file_write(&write_args);
        assert!(result.is_ok());
        
        // Test secure read
        let read_args = vec![Value::string(file_path)];
        let result = primitive_secure_file_read(&read_args);
        assert!(result.is_ok());
        
        if let Ok(Value::Literal(crate::ast::Literal::String(content))) = result {
            assert_eq!(content, "Hello, secure world!");
        } else {
            panic!("Expected string result");
        }
    }
}