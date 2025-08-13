//! Dependency analysis components for program structure analysis.

use crate::ast::Program;
use crate::diagnostics::{Result, Span};
use super::analysis_types::{DefinitionType, DependencyType};
use std::collections::{HashMap, HashSet};

/// Dependency graph representing relationships between definitions.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Nodes in the graph (definitions)
    pub nodes: HashMap<String, DependencyNode>,
    /// Edges representing dependencies
    pub edges: Vec<DependencyEdge>,
    /// Strongly connected components (cycles)
    pub cycles: Vec<Vec<String>>,
}

/// Node in the dependency graph.
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// Name of the definition
    pub name: String,
    /// Type of definition
    pub definition_type: DefinitionType,
    /// Source location
    pub location: Option<Span>,
    /// Dependencies (outgoing edges)
    pub dependencies: HashSet<String>,
    /// Dependents (incoming edges)
    pub dependents: HashSet<String>,
}

/// Edge in the dependency graph.
#[derive(Debug, Clone)]
pub struct DependencyEdge {
    /// Source node
    pub from: String,
    /// Target node
    pub to: String,
    /// Type of dependency
    pub dependency_type: DependencyType,
    /// Source location where dependency occurs
    pub location: Option<Span>,
}

/// Dependency analyzer for program dependencies.
#[derive(Debug)]
pub struct DependencyAnalyzer {
    /// Placeholder for internal state
    _internal: (),
}

impl DependencyGraph {
    /// Creates a new empty dependency graph.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            cycles: Vec::new(),
        }
    }

    /// Adds a new node to the dependency graph.
    pub fn add_node(&mut self, name: String, def_type: DefinitionType, location: Option<Span>) {
        let node = DependencyNode {
            name: name.clone(),
            definition_type: def_type,
            location,
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
        };
        self.nodes.insert(name, node);
    }

    /// Adds a dependency edge to the graph.
    pub fn add_dependency(&mut self, from: String, to: String, dep_type: DependencyType, location: Option<Span>) {
        // Add edge
        self.edges.push(DependencyEdge {
            from: from.clone(),
            to: to.clone(),
            dependency_type: dep_type,
            location,
        });

        // Update node dependencies
        if let Some(from_node) = self.nodes.get_mut(&from) {
            from_node.dependencies.insert(to.clone());
        }
        if let Some(to_node) = self.nodes.get_mut(&to) {
            to_node.dependents.insert(from);
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyAnalyzer {
    /// Creates a new dependency analyzer.
    pub fn new() -> Self {
        Self {
            _internal: (),
        }
    }

    /// Analyzes dependencies in a program.
    pub fn analyze_dependencies(&mut self, _program: &Program) -> Result<DependencyGraph> {
        // Temporary implementation - will be filled in when integrating with StaticAnalyzer
        Ok(DependencyGraph::new())
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}