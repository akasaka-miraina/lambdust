//! Cycle detection algorithms for dependency graphs

use crate::parser::dependency_analyzer::DependencyGraph;
use crate::parser::cycle_detector::{Cycle, CycleType};
use std::collections::{HashMap, HashSet};

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
    #[must_use] pub fn new(graph: DependencyGraph) -> Self {
        Self { graph }
    }

    /// Detect all cycles in the dependency graph
    #[must_use] pub fn detect_cycles(&self) -> CycleDetectionResult {
        let mut result = CycleDetectionResult {
            cycles: Vec::new(),
            strongly_connected_components: Vec::new(),
        };

        // Find strongly connected components using Tarjan's algorithm
        let components = self.find_strongly_connected_components();
        result.strongly_connected_components.clone_from(&components);

        // Convert SCCs to cycles
        result.cycles.extend(components.into_iter().filter_map(|scc| {
            match scc.len() {
                n if n > 1 => Some(Cycle {
                    cycle_type: if n == 2 { CycleType::Direct } else { CycleType::Indirect },
                    nodes: scc
                }),
                1 if self.graph.has_direct_dependency(&scc[0], &scc[0]) => Some(Cycle {
                    nodes: scc,
                    cycle_type: CycleType::SelfReference
                }),
                _ => None
            }
        }));

        result
    }

    /// Find strongly connected components using Tarjan's algorithm
    fn find_strongly_connected_components(&self) -> Vec<Vec<String>> {
        let mut tarjan = TarjanSCC::new(&self.graph);
        tarjan.find_sccs()
    }

    /// Check if a specific path forms a cycle
    #[must_use] pub fn has_cycle_in_path(&self, path: &[String]) -> bool {
        if path.len() < 2 {
            return false;
        }

        let first = &path[0];
        let last = &path[path.len() - 1];

        // Check if the last node depends on the first
        self.graph.has_direct_dependency(last, first)
    }

    /// Find the shortest cycle containing a given node
    #[must_use] pub fn find_shortest_cycle(&self, start_node: &str) -> Option<Vec<String>> {
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
                    self.lowlinks
                        .insert(node.to_string(), current_lowlink.min(successor_lowlink));
                } else if self.on_stack.contains(successor) {
                    // Successor is in stack and hence in the current SCC
                    let successor_index = self.indices[successor];
                    let current_lowlink = self.lowlinks[node];
                    self.lowlinks
                        .insert(node.to_string(), current_lowlink.min(successor_index));
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
    #[must_use] pub fn has_cycles(&self) -> bool {
        !self.cycles.is_empty()
    }

    /// Get cycles of a specific type
    #[must_use] pub fn get_cycles_of_type(&self, cycle_type: &CycleType) -> Vec<&Cycle> {
        self.cycles
            .iter()
            .filter(|cycle| cycle.cycle_type == *cycle_type)
            .collect()
    }

    /// Get the total number of cycles
    #[must_use] pub fn cycle_count(&self) -> usize {
        self.cycles.len()
    }

    /// Get all nodes involved in cycles
    #[must_use] pub fn get_cyclic_nodes(&self) -> HashSet<String> {
        let mut nodes = HashSet::new();
        for cycle in &self.cycles {
            for node in &cycle.nodes {
                nodes.insert(node.clone());
            }
        }
        nodes
    }
}

