//! Call Graph Analysis Module
//!
//! このモジュールはコールグラフの構築・解析機能を実装します。
//! 強結合成分分析、クリティカルパス検出を含みます。

use crate::error::Result;
use super::core_types::{
    StronglyConnectedComponentAnalyzer, CriticalPathAnalyzer,
};
use std::collections::{HashMap, HashSet};

/// Call graph construction and analysis
#[derive(Debug)]
pub struct CallGraphAnalyzer {
    /// Call graph edges (caller -> callees)
    pub call_graph: HashMap<String, HashSet<String>>,
    
    /// Reverse call graph (callee -> callers)
    pub reverse_call_graph: HashMap<String, HashSet<String>>,
    
    /// Call frequency weights
    pub call_weights: HashMap<(String, String), u64>,
    
    /// Strongly connected components
    pub scc_analyzer: StronglyConnectedComponentAnalyzer,
    
    /// Critical path analysis
    pub critical_path_analyzer: CriticalPathAnalyzer,
}

/// Call graph information for an expression
#[derive(Debug, Clone)]
pub struct CallGraphInfo {
    /// Direct callers
    pub callers: HashSet<String>,
    
    /// Direct callees
    pub callees: HashSet<String>,
    
    /// Call frequency from each caller
    pub caller_frequencies: HashMap<String, u64>,
    
    /// Is part of strongly connected component
    pub in_scc: bool,
    
    /// Critical path involvement
    pub on_critical_path: bool,
}

/// Call graph complexity metrics
#[derive(Debug, Clone)]
pub struct CallGraphComplexity {
    /// Number of nodes
    pub node_count: usize,
    
    /// Number of edges
    pub edge_count: usize,
    
    /// Strongly connected components count
    pub scc_count: usize,
    
    /// Maximum call depth
    pub max_depth: usize,
    
    /// Average out-degree
    pub avg_out_degree: f64,
}

impl CallGraphAnalyzer {
    #[must_use] 
    pub fn new() -> Self { 
        Self { 
            call_graph: HashMap::new(), 
            reverse_call_graph: HashMap::new(), 
            call_weights: HashMap::new(), 
            scc_analyzer: StronglyConnectedComponentAnalyzer, 
            critical_path_analyzer: CriticalPathAnalyzer,
        } 
    }
    
    pub fn record_call(&mut self, caller: &str, target: &str) -> Result<()> {
        self.call_graph.entry(caller.to_string()).or_insert_with(HashSet::new).insert(target.to_string());
        self.reverse_call_graph.entry(target.to_string()).or_insert_with(HashSet::new).insert(caller.to_string());
        *self.call_weights.entry((caller.to_string(), target.to_string())).or_insert(0) += 1;
        Ok(())
    }
    
    pub fn get_call_info(&self, expr_hash: &str) -> Option<CallGraphInfo> {
        let incoming_calls = self.reverse_call_graph.get(expr_hash)?.clone();
        let outgoing_calls = self.call_graph.get(expr_hash)?.clone();
        let caller_frequencies = incoming_calls.iter()
            .filter_map(|caller| {
                self.call_weights.get(&(caller.clone(), expr_hash.to_string()))
                    .map(|&freq| (caller.clone(), freq))
            })
            .collect();
        
        Some(CallGraphInfo {
            callers: incoming_calls,
            callees: outgoing_calls,
            caller_frequencies,
            in_scc: false, // Placeholder - would be computed by SCC analyzer
            on_critical_path: false, // Placeholder - would be computed by critical path analyzer
        })
    }
    
    pub fn calculate_complexity(&self) -> CallGraphComplexity {
        CallGraphComplexity {
            node_count: self.call_graph.len(),
            edge_count: self.call_weights.len(),
            scc_count: 0, // Placeholder - would be computed by SCC analyzer
            max_depth: self.calculate_max_depth(),
            avg_out_degree: if self.call_graph.is_empty() { 
                0.0 
            } else { 
                self.call_graph.values().map(|callees| callees.len()).sum::<usize>() as f64 / self.call_graph.len() as f64 
            },
        }
    }
    
    /// Calculate maximum call depth using DFS
    fn calculate_max_depth(&self) -> usize {
        let mut max_depth = 0;
        let mut visited = HashSet::new();
        
        for node in self.call_graph.keys() {
            if !visited.contains(node) {
                max_depth = max_depth.max(self.dfs_depth(node, &mut visited, &mut HashSet::new()));
            }
        }
        
        max_depth
    }
    
    /// Depth-first search to calculate depth
    fn dfs_depth(&self, node: &str, visited: &mut HashSet<String>, current_path: &mut HashSet<String>) -> usize {
        if current_path.contains(node) {
            // Cycle detected, return 0 to avoid infinite recursion
            return 0;
        }
        
        visited.insert(node.to_string());
        current_path.insert(node.to_string());
        
        let mut max_child_depth = 0;
        if let Some(callees) = self.call_graph.get(node) {
            for callee in callees {
                if !visited.contains(callee) {
                    max_child_depth = max_child_depth.max(self.dfs_depth(callee, visited, current_path));
                }
            }
        }
        
        current_path.remove(node);
        1 + max_child_depth
    }
    
    /// Get nodes that are hotspots (have many incoming calls)
    pub fn get_call_hotspots(&self, threshold: u64) -> Vec<CallHotspot> {
        let mut hotspots = Vec::new();
        
        for (node, callers) in &self.reverse_call_graph {
            let total_calls: u64 = callers.iter()
                .filter_map(|caller| self.call_weights.get(&(caller.clone(), node.clone())))
                .sum();
            
            if total_calls >= threshold {
                hotspots.push(CallHotspot {
                    node: node.clone(),
                    total_incoming_calls: total_calls,
                    caller_count: callers.len(),
                    average_calls_per_caller: total_calls as f64 / callers.len() as f64,
                });
            }
        }
        
        // Sort by total incoming calls (descending)
        hotspots.sort_by(|a, b| b.total_incoming_calls.cmp(&a.total_incoming_calls));
        hotspots
    }
    
    /// Find strongly connected components (simplified implementation)
    pub fn find_strongly_connected_components(&self) -> Vec<Vec<String>> {
        // Placeholder implementation - would use Tarjan's algorithm
        let mut components = Vec::new();
        let mut visited = HashSet::new();
        
        for node in self.call_graph.keys() {
            if !visited.contains(node) {
                let mut component = Vec::new();
                self.dfs_component(node, &mut visited, &mut component);
                if component.len() > 1 {
                    components.push(component);
                }
            }
        }
        
        components
    }
    
    /// DFS for component detection (simplified)
    fn dfs_component(&self, node: &str, visited: &mut HashSet<String>, component: &mut Vec<String>) {
        visited.insert(node.to_string());
        component.push(node.to_string());
        
        if let Some(callees) = self.call_graph.get(node) {
            for callee in callees {
                if !visited.contains(callee) {
                    self.dfs_component(callee, visited, component);
                }
            }
        }
    }
    
    /// Get call path from source to target
    pub fn find_call_path(&self, source: &str, target: &str) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        
        if self.dfs_path_search(source, target, &mut visited, &mut path) {
            Some(path)
        } else {
            None
        }
    }
    
    /// DFS for path finding
    fn dfs_path_search(&self, current: &str, target: &str, visited: &mut HashSet<String>, path: &mut Vec<String>) -> bool {
        visited.insert(current.to_string());
        path.push(current.to_string());
        
        if current == target {
            return true;
        }
        
        if let Some(callees) = self.call_graph.get(current) {
            for callee in callees {
                if !visited.contains(callee) && self.dfs_path_search(callee, target, visited, path) {
                    return true;
                }
            }
        }
        
        path.pop();
        false
    }
}

/// Call hotspot information
#[derive(Debug, Clone)]
pub struct CallHotspot {
    /// Node identifier
    pub node: String,
    
    /// Total incoming calls
    pub total_incoming_calls: u64,
    
    /// Number of different callers
    pub caller_count: usize,
    
    /// Average calls per caller
    pub average_calls_per_caller: f64,
}

impl Default for CallGraphAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}