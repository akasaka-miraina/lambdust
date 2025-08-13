//! Core environment management for metaprogramming operations.
//!
//! This module provides the primary structures for managing environments
//! in dynamic metaprogramming contexts.

use crate::eval::{Value, Environment};
use crate::diagnostics::{Error, Result};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::SystemTime;

/// Environment manipulation system.
#[derive(Debug)]
pub struct EnvironmentManipulator {
    /// Active environments being tracked
    environments: HashMap<String, EnvironmentHandle>,
    /// Environment hierarchy
    hierarchy: super::environment_hierarchy::EnvironmentHierarchy,
    /// Environment snapshots for rollback
    snapshots: HashMap<String, super::environment_tracking::EnvironmentSnapshot>,
    /// Change tracking
    change_tracker: super::environment_tracking::ChangeTracker,
}

/// Handle to an environment with metadata.
#[derive(Debug, Clone)]
pub struct EnvironmentHandle {
    /// Environment reference
    pub environment: Rc<Environment>,
    /// Environment metadata
    pub metadata: EnvironmentMetadata,
    /// Creation time
    pub created_at: SystemTime,
    /// Last access time
    pub last_accessed: SystemTime,
}

/// Metadata about an environment.
#[derive(Debug, Clone)]
pub struct EnvironmentMetadata {
    /// Environment name/identifier
    pub name: String,
    /// Environment type
    pub env_type: EnvironmentType,
    /// Parent environment (if any)
    pub parent: Option<String>,
    /// Child environments
    pub children: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Custom properties
    pub properties: HashMap<String, Value>,
}

/// Type of environment.
#[derive(Debug, Clone, PartialEq)]
pub enum EnvironmentType {
    /// Global/top-level environment
    Global,
    /// Module environment
    Module,
    /// Function/procedure environment
    Function,
    /// Local scope environment
    Local,
    /// Macro expansion environment
    Macro,
    /// Sandbox environment
    Sandbox,
    /// REPL environment
    Repl,
    /// Custom environment type
    Custom(String),
}

impl EnvironmentManipulator {
    /// Install primitives for environment manipulation.
    pub fn install_primitives(&self, _env: &Rc<Environment>) -> Result<()> {
        // Environment manipulation primitives would be installed here
        Ok(())
    }
    
    /// Creates a new environment manipulator.
    pub fn new() -> Self {
        Self {
            environments: HashMap::new(),
            hierarchy: super::environment_hierarchy::EnvironmentHierarchy::new(),
            snapshots: HashMap::new(),
            change_tracker: super::environment_tracking::ChangeTracker::new(1000),
        }
    }

    /// Registers an environment for tracking.
    pub fn register_environment(
        &mut self,
        name: String,
        environment: Rc<Environment>,
        env_type: EnvironmentType,
    ) -> Result<()> {
        let metadata = EnvironmentMetadata {
            name: name.clone(),
            env_type,
            parent: None,
            children: Vec::new(),
            tags: Vec::new(),
            properties: HashMap::new(),
        };

        let handle = EnvironmentHandle {
            environment,
            metadata,
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
        };

        self.environments.insert(name.clone(), handle);
        self.hierarchy.add_root(name.clone());

        // Track creation change
        self.change_tracker.track_change(name, super::environment_tracking::EnvironmentChange {
            change_type: super::environment_tracking::ChangeType::Create,
            variable: "".to_string(),
            old_value: None,
            new_value: None,
            timestamp: SystemTime::now(),
        });

        Ok(())
    }

    /// Creates a child environment.
    pub fn create_child_environment(
        &mut self,
        parent_name: &str,
        child_name: String,
        env_type: EnvironmentType,
    ) -> Result<Rc<Environment>> {
        let parent_env = self.get_environment(parent_name)?;
        let child_env = Rc::new(Environment::new(Some(parent_env.clone()), 0));

        let metadata = EnvironmentMetadata {
            name: child_name.clone(),
            env_type,
            parent: Some(parent_name.to_string()),
            children: Vec::new(),
            tags: Vec::new(),
            properties: HashMap::new(),
        };

        // Update parent's children list
        if let Some(parent_handle) = self.environments.get_mut(parent_name) {
            parent_handle.metadata.children.push(child_name.clone());
        }

        let handle = EnvironmentHandle {
            environment: child_env.clone(),
            metadata,
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
        };

        self.environments.insert(child_name.clone(), handle);
        self.hierarchy.add_child(parent_name.to_string(), child_name.clone());

        Ok(child_env)
    }

    /// Gets an environment by name.
    pub fn get_environment(&mut self, name: &str) -> Result<Rc<Environment>> {
        let handle = self.environments.get_mut(name)
            .ok_or_else(|| Error::runtime_error(
                format!("Environment '{name}' not found"),
                None,
            ))?;

        handle.last_accessed = SystemTime::now();
        Ok(handle.environment.clone())
    }

    /// Gets environment metadata.
    pub fn get_metadata(&self, name: &str) -> Option<&EnvironmentMetadata> {
        self.environments.get(name).map(|h| &h.metadata)
    }

    /// Creates a snapshot of an environment.
    pub fn create_snapshot(&mut self, env_name: &str, snapshot_id: String) -> Result<()> {
        let env = self.get_environment(env_name)?;
        let bindings = env.get_all_bindings();

        let snapshot = super::environment_tracking::EnvironmentSnapshot {
            id: snapshot_id.clone(),
            environment_name: env_name.to_string(),
            bindings,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };

        self.snapshots.insert(snapshot_id, snapshot);
        Ok(())
    }

    /// Restores an environment from a snapshot.
    pub fn restore_from_snapshot(&mut self, snapshot_id: &str) -> Result<()> {
        let snapshot = self.snapshots.get(snapshot_id)
            .ok_or_else(|| Error::runtime_error(
                format!("Snapshot '{snapshot_id}' not found"),
                None,
            ))?
            .clone();

        let env = self.get_environment(&snapshot.environment_name)?;

        // Clear current bindings and restore snapshot bindings
        env.clear_all_bindings();
        for (name, value) in snapshot.bindings {
            env.define(name, value);
        }

        Ok(())
    }

    /// Gets all changes for an environment.
    pub fn get_changes(&self, env_name: &str) -> Vec<&super::environment_tracking::EnvironmentChange> {
        self.change_tracker.changes.get(env_name)
            .map(|changes| changes.iter().collect())
            .unwrap_or_default()
    }

    /// Removes an environment.
    pub fn remove_environment(&mut self, name: &str) -> Result<()> {
        if let Some(_handle) = self.environments.remove(name) {
            // Remove from hierarchy
            self.hierarchy.remove_node(name);

            // Track destruction change
            self.change_tracker.track_change(name.to_string(), super::environment_tracking::EnvironmentChange {
                change_type: super::environment_tracking::ChangeType::Destroy,
                variable: "".to_string(),
                old_value: None,
                new_value: None,
                timestamp: SystemTime::now(),
            });

            Ok(())
        } else {
            Err(Box::new(Error::runtime_error(
                format!("Environment '{name}' not found"),
                None,
            )))
        }
    }
}

// Extension trait for Environment to add manipulation methods
/// Extension trait for Environment to add manipulation methods.
pub trait EnvironmentExt {
    /// Clears all bindings from the environment.
    fn clear_all_bindings(&self);
    /// Gets all bindings from the environment as a HashMap.
    fn get_all_bindings(&self) -> HashMap<String, Value>;
}

impl EnvironmentExt for Environment {
    fn clear_all_bindings(&self) {
        // Implementation would clear all bindings in the environment
        // This is a placeholder
    }

    fn get_all_bindings(&self) -> HashMap<String, Value> {
        // Implementation would return all current bindings
        // This is a placeholder
        HashMap::new()
    }
}

// Default implementations
impl Default for EnvironmentManipulator {
    fn default() -> Self {
        Self::new()
    }
}