//! Cycle detection algorithms for dependency graphs

use crate::parser::dependency_analyzer::DependencyGraph;
use std::collections::{HashMap, HashSet};

/// Represents a cycle in the dependency graph
#[derive(Debug, Clone, PartialEq)]
pub struct Cycle {
    /// Names of variables/functions in the cycle
    pub nodes: Vec<String>,
    /// Type of cycle
    pub cycle_type: CycleType,
}

/// Type of cycle detected
#[derive(Debug, Clone, PartialEq)]
pub enum CycleType {
    /// Simple self-reference (a depends on a)
    SelfReference,
    /// Direct cycle (a depends on b, b depends on a)
    Direct,
    /// Indirect cycle (a → b → c → a)
    Indirect,
}

/// Result of cycle detection
#[derive(Debug, Clone)]
pub struct CycleDetectionResult {
    /// List of cycles found
    pub cycles: Vec<Cycle>,
    /// Strongly connected components
    pub strongly_connected_components: Vec<Vec<String>>,
}

/// Cycle detector using various algorithms
pub struct CycleDetector {
    /// The dependency graph to analyze
    graph: DependencyGraph,
}

impl CycleDetector {
    /// Create a new cycle detector
    pub fn new(graph: DependencyGraph) -> Self {
        Self { graph }
    }

    /// Detect all cycles in the dependency graph
    pub fn detect_cycles(&self) -> CycleDetectionResult {
        let mut result = CycleDetectionResult {
            cycles: Vec::new(),
            strongly_connected_components: Vec::new(),
        };

        // Find strongly connected components using Tarjan's algorithm
        let sccs = self.find_strongly_connected_components();
        result.strongly_connected_components = sccs.clone();

        // Convert SCCs to cycles
        for scc in sccs {
            if scc.len() > 1 {
                // Multi-node cycle
                let cycle = Cycle {
                    cycle_type: if scc.len() == 2 {
                        CycleType::Direct
                    } else {
                        CycleType::Indirect
                    },
                    nodes: scc,
                };
                result.cycles.push(cycle);
            } else if scc.len() == 1 {
                // Check for self-reference
                let node = &scc[0];
                if self.graph.has_direct_dependency(node, node) {
                    let cycle = Cycle {
                        nodes: scc,
                        cycle_type: CycleType::SelfReference,
                    };
                    result.cycles.push(cycle);
                }
            }
        }

        result
    }

    /// Find strongly connected components using Tarjan's algorithm
    fn find_strongly_connected_components(&self) -> Vec<Vec<String>> {
        let mut tarjan = TarjanSCC::new(&self.graph);
        tarjan.find_sccs()
    }

    /// Check if a specific path forms a cycle
    pub fn has_cycle_in_path(&self, path: &[String]) -> bool {
        if path.len() < 2 {
            return false;
        }

        let first = &path[0];
        let last = &path[path.len() - 1];

        // Check if the last node depends on the first
        self.graph.has_direct_dependency(last, first)
    }

    /// Find the shortest cycle containing a given node
    pub fn find_shortest_cycle(&self, start_node: &str) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        
        self.dfs_find_cycle(start_node, start_node, &mut visited, &mut path)
    }

    /// DFS helper for finding cycles
    fn dfs_find_cycle(
        &self,
        current: &str,
        target: &str,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if visited.contains(current) {
            return None;
        }

        visited.insert(current.to_string());
        path.push(current.to_string());

        if let Some(dependencies) = self.graph.get_edges().get(current) {
            for dep in dependencies {
                if dep == target && path.len() > 1 {
                    // Found a cycle back to the target
                    let mut cycle_path = path.clone();
                    cycle_path.push(dep.clone());
                    return Some(cycle_path);
                }

                if let Some(cycle) = self.dfs_find_cycle(dep, target, visited, path) {
                    return Some(cycle);
                }
            }
        }

        path.pop();
        visited.remove(current);
        None
    }
}

/// Tarjan's strongly connected components algorithm
struct TarjanSCC<'a> {
    graph: &'a DependencyGraph,
    index: usize,
    stack: Vec<String>,
    indices: HashMap<String, usize>,
    lowlinks: HashMap<String, usize>,
    on_stack: HashSet<String>,
    sccs: Vec<Vec<String>>,
}

impl<'a> TarjanSCC<'a> {
    fn new(graph: &'a DependencyGraph) -> Self {
        Self {
            graph,
            index: 0,
            stack: Vec::new(),
            indices: HashMap::new(),
            lowlinks: HashMap::new(),
            on_stack: HashSet::new(),
            sccs: Vec::new(),
        }
    }

    fn find_sccs(&mut self) -> Vec<Vec<String>> {
        let nodes: Vec<String> = self.graph.get_nodes().keys().cloned().collect();
        
        for node in nodes {
            if !self.indices.contains_key(&node) {
                self.strongconnect(&node);
            }
        }

        self.sccs.clone()
    }

    fn strongconnect(&mut self, node: &str) {
        // Set the depth index for this node
        self.indices.insert(node.to_string(), self.index);
        self.lowlinks.insert(node.to_string(), self.index);
        self.index += 1;
        self.stack.push(node.to_string());
        self.on_stack.insert(node.to_string());

        // Consider successors of node
        if let Some(successors) = self.graph.get_edges().get(node) {
            for successor in successors {
                if !self.indices.contains_key(successor) {
                    // Successor has not yet been visited; recurse on it
                    self.strongconnect(successor);
                    let successor_lowlink = self.lowlinks[successor];
                    let current_lowlink = self.lowlinks[node];
                    self.lowlinks.insert(
                        node.to_string(),
                        current_lowlink.min(successor_lowlink),
                    );
                } else if self.on_stack.contains(successor) {
                    // Successor is in stack and hence in the current SCC
                    let successor_index = self.indices[successor];
                    let current_lowlink = self.lowlinks[node];
                    self.lowlinks.insert(
                        node.to_string(),
                        current_lowlink.min(successor_index),
                    );
                }
            }
        }

        // If node is a root node, pop the stack and print an SCC
        if self.lowlinks[node] == self.indices[node] {
            let mut scc = Vec::new();
            loop {
                let w = self.stack.pop().expect("Stack should not be empty");
                self.on_stack.remove(&w);
                scc.push(w.clone());
                if w == node {
                    break;
                }
            }
            self.sccs.push(scc);
        }
    }
}

/// Utility functions for cycle analysis
impl CycleDetectionResult {
    /// Check if any cycles were found
    pub fn has_cycles(&self) -> bool {
        !self.cycles.is_empty()
    }

    /// Get cycles of a specific type
    pub fn get_cycles_of_type(&self, cycle_type: CycleType) -> Vec<&Cycle> {
        self.cycles
            .iter()
            .filter(|cycle| cycle.cycle_type == cycle_type)
            .collect()
    }

    /// Get the total number of cycles
    pub fn cycle_count(&self) -> usize {
        self.cycles.len()
    }

    /// Get all nodes involved in cycles
    pub fn get_cyclic_nodes(&self) -> HashSet<String> {
        let mut nodes = HashSet::new();
        for cycle in &self.cycles {
            for node in &cycle.nodes {
                nodes.insert(node.clone());
            }
        }
        nodes
    }
}

impl Cycle {
    /// Get a human-readable representation of the cycle
    pub fn to_string(&self) -> String {
        match self.cycle_type {
            CycleType::SelfReference => {
                format!("{} → {}", self.nodes[0], self.nodes[0])
            }
            CycleType::Direct => {
                format!("{} → {}", self.nodes.join(" → "), self.nodes[0])
            }
            CycleType::Indirect => {
                format!("{} → {}", self.nodes.join(" → "), self.nodes[0])
            }
        }
    }

    /// Get the length of the cycle
    pub fn length(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::dependency_analyzer::{DependencyGraph, DependencyNode, DependencyType};
    use crate::ast::{Expr, Literal};

    #[test]
    fn test_self_reference_cycle() {
        let mut graph = DependencyGraph::new();
        
        // Create a self-referencing node: (define x x)
        let node = DependencyNode {
            name: "x".to_string(),
            expr: Expr::Variable("x".to_string()),
            depends_on: vec!["x".to_string()],
            dependency_type: DependencyType::Variable,
        };
        graph.add_node(node);
        
        let detector = CycleDetector::new(graph);
        let result = detector.detect_cycles();
        
        assert!(result.has_cycles());
        assert_eq!(result.cycle_count(), 1);
        
        let cycle = &result.cycles[0];
        assert_eq!(cycle.cycle_type, CycleType::SelfReference);
        assert_eq!(cycle.nodes, vec!["x".to_string()]);
    }

    #[test]
    fn test_direct_cycle() {
        let mut graph = DependencyGraph::new();
        
        // Create a direct cycle: x → y → x
        let node_x = DependencyNode {
            name: "x".to_string(),
            expr: Expr::Variable("y".to_string()),
            depends_on: vec!["y".to_string()],
            dependency_type: DependencyType::Variable,
        };
        let node_y = DependencyNode {
            name: "y".to_string(),
            expr: Expr::Variable("x".to_string()),
            depends_on: vec!["x".to_string()],
            dependency_type: DependencyType::Variable,
        };
        
        graph.add_node(node_x);
        graph.add_node(node_y);
        
        let detector = CycleDetector::new(graph);
        let result = detector.detect_cycles();
        
        assert!(result.has_cycles());
        assert_eq!(result.cycle_count(), 1);
        
        let cycle = &result.cycles[0];
        assert_eq!(cycle.cycle_type, CycleType::Direct);
        assert_eq!(cycle.length(), 2);
        
        let cyclic_nodes = result.get_cyclic_nodes();
        assert!(cyclic_nodes.contains("x"));
        assert!(cyclic_nodes.contains("y"));
    }

    #[test]
    fn test_indirect_cycle() {
        let mut graph = DependencyGraph::new();
        
        // Create an indirect cycle: a → b → c → a
        let node_a = DependencyNode {
            name: "a".to_string(),
            expr: Expr::Variable("b".to_string()),
            depends_on: vec!["b".to_string()],
            dependency_type: DependencyType::Variable,
        };
        let node_b = DependencyNode {
            name: "b".to_string(),
            expr: Expr::Variable("c".to_string()),
            depends_on: vec!["c".to_string()],
            dependency_type: DependencyType::Variable,
        };
        let node_c = DependencyNode {
            name: "c".to_string(),
            expr: Expr::Variable("a".to_string()),
            depends_on: vec!["a".to_string()],
            dependency_type: DependencyType::Variable,
        };
        
        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);
        
        let detector = CycleDetector::new(graph);
        let result = detector.detect_cycles();
        
        assert!(result.has_cycles());
        assert_eq!(result.cycle_count(), 1);
        
        let cycle = &result.cycles[0];
        assert_eq!(cycle.cycle_type, CycleType::Indirect);
        assert_eq!(cycle.length(), 3);
        
        let cyclic_nodes = result.get_cyclic_nodes();
        assert!(cyclic_nodes.contains("a"));
        assert!(cyclic_nodes.contains("b"));
        assert!(cyclic_nodes.contains("c"));
    }

    #[test]
    fn test_no_cycles() {
        let mut graph = DependencyGraph::new();
        
        // Create a non-cyclic dependency: a → b → c
        let node_a = DependencyNode {
            name: "a".to_string(),
            expr: Expr::Variable("b".to_string()),
            depends_on: vec!["b".to_string()],
            dependency_type: DependencyType::Variable,
        };
        let node_b = DependencyNode {
            name: "b".to_string(),
            expr: Expr::Variable("c".to_string()),
            depends_on: vec!["c".to_string()],
            dependency_type: DependencyType::Variable,
        };
        let node_c = DependencyNode {
            name: "c".to_string(),
            expr: Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(42))),
            depends_on: vec![],
            dependency_type: DependencyType::Variable,
        };
        
        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);
        
        let detector = CycleDetector::new(graph);
        let result = detector.detect_cycles();
        
        assert!(!result.has_cycles());
        assert_eq!(result.cycle_count(), 0);
    }

    #[test]
    fn test_multiple_cycles() {
        let mut graph = DependencyGraph::new();
        
        // Create multiple cycles
        // Cycle 1: a → a (self-reference)
        let node_a = DependencyNode {
            name: "a".to_string(),
            expr: Expr::Variable("a".to_string()),
            depends_on: vec!["a".to_string()],
            dependency_type: DependencyType::Variable,
        };
        
        // Cycle 2: b → c → b (direct)
        let node_b = DependencyNode {
            name: "b".to_string(),
            expr: Expr::Variable("c".to_string()),
            depends_on: vec!["c".to_string()],
            dependency_type: DependencyType::Variable,
        };
        let node_c = DependencyNode {
            name: "c".to_string(),
            expr: Expr::Variable("b".to_string()),
            depends_on: vec!["b".to_string()],
            dependency_type: DependencyType::Variable,
        };
        
        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);
        
        let detector = CycleDetector::new(graph);
        let result = detector.detect_cycles();
        
        assert!(result.has_cycles());
        assert_eq!(result.cycle_count(), 2);
        
        let self_refs = result.get_cycles_of_type(CycleType::SelfReference);
        let direct_cycles = result.get_cycles_of_type(CycleType::Direct);
        
        assert_eq!(self_refs.len(), 1);
        assert_eq!(direct_cycles.len(), 1);
    }

    #[test]
    fn test_shortest_cycle_finding() {
        let mut graph = DependencyGraph::new();
        
        // Create a cycle: a → b → c → a
        let node_a = DependencyNode {
            name: "a".to_string(),
            expr: Expr::Variable("b".to_string()),
            depends_on: vec!["b".to_string()],
            dependency_type: DependencyType::Variable,
        };
        let node_b = DependencyNode {
            name: "b".to_string(),
            expr: Expr::Variable("c".to_string()),
            depends_on: vec!["c".to_string()],
            dependency_type: DependencyType::Variable,
        };
        let node_c = DependencyNode {
            name: "c".to_string(),
            expr: Expr::Variable("a".to_string()),
            depends_on: vec!["a".to_string()],
            dependency_type: DependencyType::Variable,
        };
        
        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);
        
        let detector = CycleDetector::new(graph);
        let cycle = detector.find_shortest_cycle("a");
        
        assert!(cycle.is_some());
        let cycle_path = cycle.unwrap();
        assert_eq!(cycle_path.len(), 4); // a → b → c → a
        assert_eq!(cycle_path[0], "a");
        assert_eq!(cycle_path[3], "a");
    }
}