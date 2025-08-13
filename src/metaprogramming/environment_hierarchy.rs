//! Environment hierarchy tracking and management.
//!
//! This module provides tree-like hierarchy management for environments
//! including parent-child relationships and traversal operations.

use std::collections::HashMap;

/// Environment hierarchy tracking.
#[derive(Debug)]
pub struct EnvironmentHierarchy {
    /// Root environments
    pub roots: Vec<String>,
    /// Parent-child relationships
    relationships: HashMap<String, Vec<String>>,
    /// Reverse lookup (child -> parent)
    parent_lookup: HashMap<String, String>,
}

impl Default for EnvironmentHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentHierarchy {
    /// Creates a new environment hierarchy.
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            relationships: HashMap::new(),
            parent_lookup: HashMap::new(),
        }
    }

    /// Adds a root environment to the hierarchy.
    pub fn add_root(&mut self, name: String) {
        self.roots.push(name);
    }

    /// Adds a child environment under a parent.
    pub fn add_child(&mut self, parent: String, child: String) {
        self.relationships.entry(parent.clone()).or_default().push(child.clone());
        self.parent_lookup.insert(child, parent);
    }

    /// Removes a node from the hierarchy.
    pub fn remove_node(&mut self, name: &str) {
        self.roots.retain(|n| n != name);
        if let Some(children) = self.relationships.remove(name) {
            for child in children {
                self.parent_lookup.remove(&child);
            }
        }
        if let Some(parent) = self.parent_lookup.remove(name) {
            if let Some(siblings) = self.relationships.get_mut(&parent) {
                siblings.retain(|s| s != name);
            }
        }
    }

    /// Gets the parent of an environment.
    pub fn get_parent(&self, name: &str) -> Option<&String> {
        self.parent_lookup.get(name)
    }

    /// Gets the children of an environment.
    pub fn get_children(&self, name: &str) -> Option<&Vec<String>> {
        self.relationships.get(name)
    }

    /// Checks if an environment is a root.
    pub fn is_root(&self, name: &str) -> bool {
        self.roots.contains(&name.to_string())
    }

    /// Gets all descendants of an environment.
    pub fn get_descendants(&self, name: &str) -> Vec<String> {
        let mut descendants = Vec::new();
        self.collect_descendants(name, &mut descendants);
        descendants
    }

    /// Gets all ancestors of an environment.
    pub fn get_ancestors(&self, name: &str) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut current = name;
        
        while let Some(parent) = self.parent_lookup.get(current) {
            ancestors.push(parent.clone());
            current = parent;
        }
        
        ancestors
    }

    fn collect_descendants(&self, name: &str, descendants: &mut Vec<String>) {
        if let Some(children) = self.relationships.get(name) {
            for child in children {
                descendants.push(child.clone());
                self.collect_descendants(child, descendants);
            }
        }
    }
}