//! Environment change tracking and snapshot management.
//!
//! This module provides functionality for tracking changes to environments
//! and creating/restoring snapshots for rollback operations.

use crate::eval::Value;
use std::collections::HashMap;
use std::time::SystemTime;

/// Snapshot of an environment for rollback.
#[derive(Debug, Clone)]
pub struct EnvironmentSnapshot {
    /// Snapshot identifier
    pub id: String,
    /// Environment name
    pub environment_name: String,
    /// Bindings at snapshot time
    pub bindings: HashMap<String, Value>,
    /// Snapshot timestamp
    pub timestamp: SystemTime,
    /// Snapshot metadata
    pub metadata: HashMap<String, Value>,
}

/// Change tracking for environments.
#[derive(Debug)]
pub struct ChangeTracker {
    /// Changes by environment
    pub changes: HashMap<String, Vec<EnvironmentChange>>,
    /// Maximum changes to track per environment
    max_changes: usize,
}

/// A change to an environment.
#[derive(Debug, Clone)]
pub struct EnvironmentChange {
    /// Type of change
    pub change_type: ChangeType,
    /// Variable name affected
    pub variable: String,
    /// Old value (if any)
    pub old_value: Option<Value>,
    /// New value (if any)
    pub new_value: Option<Value>,
    /// Timestamp of change
    pub timestamp: SystemTime,
}

/// Type of environment change.
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    /// Variable defined
    Define,
    /// Variable updated
    Update,
    /// Variable removed
    Remove,
    /// Environment created
    Create,
    /// Environment destroyed
    Destroy,
}

impl EnvironmentSnapshot {
    /// Creates a new environment snapshot.
    pub fn new(
        id: String,
        environment_name: String,
        bindings: HashMap<String, Value>,
    ) -> Self {
        Self {
            id,
            environment_name,
            bindings,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    /// Gets the age of this snapshot.
    pub fn age(&self) -> Option<std::time::Duration> {
        SystemTime::now().duration_since(self.timestamp).ok()
    }

    /// Adds metadata to the snapshot.
    pub fn add_metadata(&mut self, key: String, value: Value) {
        self.metadata.insert(key, value);
    }

    /// Gets metadata from the snapshot.
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }
}

impl ChangeTracker {
    /// Creates a new change tracker with maximum change limit.
    pub fn new(max_changes: usize) -> Self {
        Self {
            changes: HashMap::new(),
            max_changes,
        }
    }

    /// Tracks a change for a given environment.
    pub fn track_change(&mut self, env_name: String, change: EnvironmentChange) {
        let changes = self.changes.entry(env_name).or_default();
        
        if changes.len() >= self.max_changes {
            changes.remove(0);
        }
        
        changes.push(change);
    }

    /// Gets all changes for an environment.
    pub fn get_changes(&self, env_name: &str) -> Option<&Vec<EnvironmentChange>> {
        self.changes.get(env_name)
    }

    /// Gets the most recent change for an environment.
    pub fn get_latest_change(&self, env_name: &str) -> Option<&EnvironmentChange> {
        self.changes.get(env_name)?.last()
    }

    /// Gets changes of a specific type for an environment.
    pub fn get_changes_by_type(&self, env_name: &str, change_type: ChangeType) -> Vec<&EnvironmentChange> {
        self.changes.get(env_name)
            .map(|changes| changes.iter().filter(|c| c.change_type == change_type).collect())
            .unwrap_or_default()
    }

    /// Clears all changes for an environment.
    pub fn clear_changes(&mut self, env_name: &str) {
        self.changes.remove(env_name);
    }

    /// Gets the total number of changes across all environments.
    pub fn total_changes(&self) -> usize {
        self.changes.values().map(|v| v.len()).sum()
    }

    /// Gets environments with changes.
    pub fn environments_with_changes(&self) -> Vec<&String> {
        self.changes.keys().collect()
    }
}

impl EnvironmentChange {
    /// Creates a new variable definition change.
    pub fn define(variable: String, new_value: Value) -> Self {
        Self {
            change_type: ChangeType::Define,
            variable,
            old_value: None,
            new_value: Some(new_value),
            timestamp: SystemTime::now(),
        }
    }

    /// Creates a new variable update change.
    pub fn update(variable: String, old_value: Value, new_value: Value) -> Self {
        Self {
            change_type: ChangeType::Update,
            variable,
            old_value: Some(old_value),
            new_value: Some(new_value),
            timestamp: SystemTime::now(),
        }
    }

    /// Creates a new variable removal change.
    pub fn remove(variable: String, old_value: Value) -> Self {
        Self {
            change_type: ChangeType::Remove,
            variable,
            old_value: Some(old_value),
            new_value: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Creates a new environment creation change.
    pub fn create_environment() -> Self {
        Self {
            change_type: ChangeType::Create,
            variable: "".to_string(),
            old_value: None,
            new_value: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Creates a new environment destruction change.
    pub fn destroy_environment() -> Self {
        Self {
            change_type: ChangeType::Destroy,
            variable: "".to_string(),
            old_value: None,
            new_value: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Gets the age of this change.
    pub fn age(&self) -> Option<std::time::Duration> {
        SystemTime::now().duration_since(self.timestamp).ok()
    }

    /// Checks if this change affects a specific variable.
    pub fn affects_variable(&self, var_name: &str) -> bool {
        self.variable == var_name
    }
}