//! Security system for metaprogramming operations.
//!
//! This module provides comprehensive security controls for dynamic evaluation,
//! including permission systems, access controls, and resource limits.

use crate::eval::Value;
use crate::diagnostics::{Error, Result};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Security permission types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Allow dynamic evaluation
    Eval,
    /// Allow compilation
    Compile,
    /// Allow file system access
    FileSystem { 
        /// File path for access.
        path: String, 
        /// Allow read access.
        read: bool, 
        /// Allow write access.
        write: bool 
    },
    /// Allow network access
    Network { 
        /// Host name or IP address.
        host: String, 
        /// Optional port number.
        port: Option<u16> 
    },
    /// Allow environment manipulation
    Environment { 
        /// Allow reading environment.
        read: bool, 
        /// Allow writing environment.
        write: bool 
    },
    /// Allow reflection operations
    Reflection,
    /// Allow FFI calls
    Ffi,
    /// Allow module operations
    Module { 
        /// Allow loading modules.
        load: bool, 
        /// Allow unloading modules.
        unload: bool 
    },
    /// Allow memory management
    Memory { 
        /// Allow memory allocation.
        allocate: bool, 
        /// Allow garbage collection.
        gc: bool 
    },
    /// Allow system calls
    System,
    /// Custom permission
    Custom(String),
}

/// Security policy that defines allowed operations.
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Name of the policy
    pub name: String,
    /// Allowed permissions
    pub allowed_permissions: HashSet<Permission>,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Execution time limit
    pub time_limit: Option<Duration>,
    /// Memory limit
    pub memory_limit: Option<usize>,
    /// Stack depth limit
    pub stack_depth_limit: Option<usize>,
    /// Custom restrictions
    pub custom_restrictions: HashMap<String, Value>,
}

/// Resource limits for sandboxed execution.
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct ResourceLimits {
    /// Maximum number of allocations
    pub max_allocations: Option<usize>,
    /// Maximum memory usage
    pub max_memory: Option<usize>,
    /// Maximum execution time
    pub max_execution_time: Option<Duration>,
    /// Maximum stack depth
    pub max_stack_depth: Option<usize>,
    /// Maximum number of file operations
    pub max_file_operations: Option<usize>,
    /// Maximum number of network operations
    pub max_network_operations: Option<usize>,
}

/// Access control entry.
#[derive(Debug, Clone)]
pub struct AccessControlEntry {
    /// Principal (user, role, etc.)
    pub principal: String,
    /// Resource being accessed
    pub resource: String,
    /// Operation being performed
    pub operation: String,
    /// Whether access is allowed
    pub allowed: bool,
    /// Conditions for access
    pub conditions: Vec<AccessCondition>,
}

/// Condition for access control.
#[derive(Debug, Clone)]
pub enum AccessCondition {
    /// Time-based condition
    TimeRange { 
        /// Start time of the valid range.
        start: Instant, 
        /// End time of the valid range.
        end: Instant 
    },
    /// Resource-based condition
    ResourceLimit { 
        /// The resource being limited.
        resource: String, 
        /// The limit value for the resource.
        limit: usize 
    },
    /// Context-based condition
    Context { 
        /// Context key.
        key: String, 
        /// Context value.
        value: String 
    },
    /// Custom condition
    Custom(String),
}

/// Security context for operations.
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Current principal
    pub principal: String,
    /// Active security policy
    pub policy: SecurityPolicy,
    /// Current permissions
    pub permissions: HashSet<Permission>,
    /// Resource usage tracking
    pub resource_usage: ResourceUsage,
    /// Execution start time
    pub start_time: Instant,
}

/// Resource usage tracking.
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Number of allocations made
    pub allocations: usize,
    /// Memory currently used
    pub memory_used: usize,
    /// Current stack depth
    pub stack_depth: usize,
    /// Number of file operations
    pub file_operations: usize,
    /// Number of network operations
    pub network_operations: usize,
    /// Execution time so far
    pub execution_time: Duration,
}

/// Security manager that enforces policies and access controls.
#[derive(Debug, Clone)]
pub struct SecurityManager {
    /// Security policies
    policies: HashMap<String, SecurityPolicy>,
    /// Access control list
    acl: Vec<AccessControlEntry>,
    /// Current security contexts
    contexts: Arc<RwLock<HashMap<String, SecurityContext>>>,
    /// Default policy
    default_policy: SecurityPolicy,
}

impl SecurityPolicy {
    /// Creates a restrictive security policy.
    pub fn restrictive() -> Self {
        Self {
            name: "restrictive".to_string(),
            allowed_permissions: HashSet::new(),
            resource_limits: ResourceLimits::default(),
            time_limit: Some(Duration::from_secs(30)),
            memory_limit: Some(1024 * 1024), // 1MB
            stack_depth_limit: Some(100),
            custom_restrictions: HashMap::new(),
        }
    }

    /// Creates a permissive security policy.
    pub fn permissive() -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::Eval);
        permissions.insert(Permission::Compile);
        permissions.insert(Permission::Reflection);
        permissions.insert(Permission::Environment { read: true, write: true });
        
        Self {
            name: "permissive".to_string(),
            allowed_permissions: permissions,
            resource_limits: ResourceLimits::default(),
            time_limit: None,
            memory_limit: None,
            stack_depth_limit: None,
            custom_restrictions: HashMap::new(),
        }
    }
}

impl SecurityManager {
    /// Creates a new security manager.
    pub fn new() -> Self {
        let default_policy = SecurityPolicy {
            name: "default".to_string(),
            allowed_permissions: HashSet::new(),
            resource_limits: ResourceLimits::default(),
            time_limit: Some(Duration::from_secs(10)),
            memory_limit: Some(1024 * 1024), // 1MB
            stack_depth_limit: Some(100),
            custom_restrictions: HashMap::new(),
        };

        Self {
            policies: HashMap::new(),
            acl: Vec::new(),
            contexts: Arc::new(RwLock::new(HashMap::new())),
            default_policy,
        }
    }

    /// Installs default security policies.
    pub fn install_default_policies(&mut self) {
        // Restrictive policy for untrusted code
        let mut sandbox_policy = SecurityPolicy {
            name: "sandbox".to_string(),
            allowed_permissions: HashSet::new(),
            resource_limits: ResourceLimits {
                max_allocations: Some(1000),
                max_memory: Some(512 * 1024), // 512KB
                max_execution_time: Some(Duration::from_secs(5)),
                max_stack_depth: Some(50),
                max_file_operations: Some(0), // No file access
                max_network_operations: Some(0), // No network access
            },
            time_limit: Some(Duration::from_secs(5)),
            memory_limit: Some(512 * 1024),
            stack_depth_limit: Some(50),
            custom_restrictions: HashMap::new(),
        };
        sandbox_policy.allowed_permissions.insert(Permission::Eval);
        sandbox_policy.allowed_permissions.insert(Permission::Reflection);

        // Permissive policy for trusted code  
        let mut trusted_policy = SecurityPolicy {
            name: "trusted".to_string(),
            allowed_permissions: HashSet::new(),
            resource_limits: ResourceLimits::default(),
            time_limit: None,
            memory_limit: None,
            stack_depth_limit: None,
            custom_restrictions: HashMap::new(),
        };
        trusted_policy.allowed_permissions.insert(Permission::Eval);
        trusted_policy.allowed_permissions.insert(Permission::Compile);
        trusted_policy.allowed_permissions.insert(Permission::Reflection);
        trusted_policy.allowed_permissions.insert(Permission::Environment { read: true, write: true });
        trusted_policy.allowed_permissions.insert(Permission::Module { load: true, unload: true });

        self.policies.insert("sandbox".to_string(), sandbox_policy);
        self.policies.insert("trusted".to_string(), trusted_policy);
    }

    /// Creates a new security context.
    pub fn create_context(&self, principal: String, policy_name: &str) -> Result<SecurityContext> {
        let policy = self.policies.get(policy_name)
            .unwrap_or(&self.default_policy)
            .clone();

        let context = SecurityContext {
            principal: principal.clone(),
            permissions: policy.allowed_permissions.clone(),
            policy,
            resource_usage: ResourceUsage::default(),
            start_time: Instant::now(),
        };

        let mut contexts = self.contexts.write().unwrap();
        contexts.insert(principal.clone(), context.clone());

        Ok(context)
    }

    /// Checks if a permission is allowed for a principal.
    pub fn check_permission(&self, principal: &str, permission: &Permission) -> Result<bool> {
        let contexts = self.contexts.read().unwrap();
        if let Some(context) = contexts.get(principal) {
            Ok(context.permissions.contains(permission))
        } else {
            Ok(false)
        }
    }

    /// Checks access control for a specific operation.
    pub fn check_access(&self, principal: &str, resource: &str, operation: &str) -> Result<bool> {
        for entry in &self.acl {
            if entry.principal == principal && entry.resource == resource && entry.operation == operation
                && self.evaluate_conditions(&entry.conditions)? {
                    return Ok(entry.allowed);
                }
        }
        Ok(false) // Default deny
    }

    /// Updates resource usage for a context.
    pub fn update_resource_usage<F>(&self, principal: &str, updater: F) -> Result<()>
    where
        F: FnOnce(&mut ResourceUsage),
    {
        let mut contexts = self.contexts.write().unwrap();
        if let Some(context) = contexts.get_mut(principal) {
            updater(&mut context.resource_usage);
            self.check_resource_limits(context)?;
        }
        Ok(())
    }

    /// Checks if resource limits are exceeded.
    fn check_resource_limits(&self, context: &SecurityContext) -> Result<()> {
        let limits = &context.policy.resource_limits;
        let usage = &context.resource_usage;

        if let Some(max_allocations) = limits.max_allocations {
            if usage.allocations > max_allocations {
                return Err(Box::new(Error::runtime_error(
                    format!("Allocation limit exceeded: {} > {}", usage.allocations, max_allocations),
                    None,
                )));
            }
        }

        if let Some(max_memory) = limits.max_memory {
            if usage.memory_used > max_memory {
                return Err(Box::new(Error::runtime_error(
                    format!("Memory limit exceeded: {} > {}", usage.memory_used, max_memory),
                    None,
                )));
            }
        }

        if let Some(max_time) = limits.max_execution_time {
            if usage.execution_time > max_time {
                return Err(Box::new(Error::runtime_error(
                    format!("Execution time limit exceeded: {:?} > {:?}", usage.execution_time, max_time),
                    None,
                )));
            }
        }

        if let Some(max_stack) = limits.max_stack_depth {
            if usage.stack_depth > max_stack {
                return Err(Box::new(Error::runtime_error(
                    format!("Stack depth limit exceeded: {} > {}", usage.stack_depth, max_stack),
                    None,
                )));
            }
        }

        Ok(())
    }

    /// Evaluates access conditions.
    fn evaluate_conditions(&self, conditions: &[AccessCondition]) -> Result<bool> {
        for condition in conditions {
            match condition {
                AccessCondition::TimeRange { start, end } => {
                    let now = Instant::now();
                    if now < *start || now > *end {
                        return Ok(false);
                    }
                }
                AccessCondition::ResourceLimit { limit, .. } => {
                    // Would need to check actual resource usage
                    if *limit == 0 {
                        return Ok(false);
                    }
                }
                _ => {
                    // Other conditions not implemented
                }
            }
        }
        Ok(true)
    }

    /// Adds an access control entry.
    pub fn add_access_control_entry(&mut self, entry: AccessControlEntry) {
        self.acl.push(entry);
    }

    /// Adds a security policy.
    pub fn add_policy(&mut self, policy: SecurityPolicy) {
        self.policies.insert(policy.name.clone(), policy);
    }

    /// Gets a security context for a principal.
    pub fn get_context(&self, principal: &str) -> Option<SecurityContext> {
        let contexts = self.contexts.read().unwrap();
        contexts.get(principal).cloned()
    }

    /// Removes a security context.
    pub fn remove_context(&self, principal: &str) {
        let mut contexts = self.contexts.write().unwrap();
        contexts.remove(principal);
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        let mut manager = Self::new();
        manager.install_default_policies();
        manager
    }
}


impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            allocations: 0,
            memory_used: 0,
            stack_depth: 0,
            file_operations: 0,
            network_operations: 0,
            execution_time: Duration::from_secs(0),
        }
    }
}

/// Permission system for fine-grained access control.
#[derive(Debug)]
pub struct PermissionSystem {
    /// Granted permissions by principal
    granted_permissions: HashMap<String, HashSet<Permission>>,
    /// Permission hierarchy
    permission_hierarchy: HashMap<Permission, Vec<Permission>>,
}

impl PermissionSystem {
    /// Creates a new permission system.
    pub fn new() -> Self {
        Self {
            granted_permissions: HashMap::new(),
            permission_hierarchy: HashMap::new(),
        }
    }

    /// Grants a permission to a principal.
    pub fn grant_permission(&mut self, principal: String, permission: Permission) {
        self.granted_permissions
            .entry(principal)
            .or_default()
            .insert(permission);
    }

    /// Revokes a permission from a principal.
    pub fn revoke_permission(&mut self, principal: &str, permission: &Permission) {
        if let Some(permissions) = self.granted_permissions.get_mut(principal) {
            permissions.remove(permission);
        }
    }

    /// Checks if a principal has a permission.
    pub fn has_permission(&self, principal: &str, permission: &Permission) -> bool {
        if let Some(permissions) = self.granted_permissions.get(principal) {
            if permissions.contains(permission) {
                return true;
            }
            
            // Check permission hierarchy
            for granted in permissions {
                if self.implies_permission(granted, permission) {
                    return true;
                }
            }
        }
        false
    }

    /// Checks if one permission implies another.
    fn implies_permission(&self, granted: &Permission, required: &Permission) -> bool {
        if let Some(implied) = self.permission_hierarchy.get(granted) {
            implied.contains(required)
        } else {
            false
        }
    }

    /// Adds a permission implication.
    pub fn add_permission_implication(&mut self, parent: Permission, child: Permission) {
        self.permission_hierarchy
            .entry(parent)
            .or_default()
            .push(child);
    }
}

/// Access control system for resource protection.
#[derive(Debug)]
pub struct AccessControl {
    /// Security manager
    security_manager: SecurityManager,
    /// Permission system
    permission_system: PermissionSystem,
}

impl AccessControl {
    /// Creates a new access control system.
    pub fn new() -> Self {
        Self {
            security_manager: SecurityManager::default(),
            permission_system: PermissionSystem::new(),
        }
    }

    /// Checks if an operation is allowed.
    pub fn check_operation(
        &self,
        principal: &str,
        resource: &str,
        operation: &str,
        permission: &Permission,
    ) -> Result<bool> {
        // Check permission first
        if !self.permission_system.has_permission(principal, permission) {
            return Ok(false);
        }

        // Check access control
        self.security_manager.check_access(principal, resource, operation)
    }

    /// Gets the security manager.
    pub fn security_manager(&self) -> &SecurityManager {
        &self.security_manager
    }

    /// Gets the permission system.
    pub fn permission_system(&self) -> &PermissionSystem {
        &self.permission_system
    }

    /// Gets mutable access to the security manager.
    pub fn security_manager_mut(&mut self) -> &mut SecurityManager {
        &mut self.security_manager
    }

    /// Gets mutable access to the permission system.
    pub fn permission_system_mut(&mut self) -> &mut PermissionSystem {
        &mut self.permission_system
    }
}

impl Default for AccessControl {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PermissionSystem {
    fn default() -> Self {
        Self::new()
    }
}