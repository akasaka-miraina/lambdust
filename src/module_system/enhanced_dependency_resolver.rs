//! Enhanced dependency resolution with advanced circular dependency detection.
//!
//! This module provides comprehensive dependency analysis including:
//! - Advanced circular dependency detection with detailed path reporting
//! - Transitive dependency analysis
//! - Dependency optimization strategies
//! - Hot-reload dependency tracking

use super::{Module, ModuleError, ModuleId};
use crate::diagnostics::{Error, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

/// Enhanced dependency resolver with advanced analysis capabilities.
#[derive(Debug)]
pub struct EnhancedDependencyResolver {
    /// Cache of resolved dependency graphs
    dependency_cache: HashMap<ModuleId, DependencyGraph>,
    /// Performance tracking
    resolution_stats: ResolutionStatistics,
    /// Circular dependency detection configuration
    detection_config: CircularDependencyConfig,
}

/// Detailed dependency graph for a module.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Root module ID
    pub root: ModuleId,
    /// Direct dependencies
    pub direct_dependencies: Vec<ModuleId>,
    /// All transitive dependencies in topological order
    pub transitive_dependencies: Vec<ModuleId>,
    /// Dependency levels (distance from root)
    pub dependency_levels: HashMap<ModuleId, usize>,
    /// Optional cycles detected during analysis
    pub detected_cycles: Vec<DependencyCycle>,
    /// Resolution timestamp
    pub resolved_at: std::time::SystemTime,
}

/// Represents a detected circular dependency with detailed path information.
#[derive(Debug, Clone)]
pub struct DependencyCycle {
    /// The cycle path (modules forming the cycle)
    pub cycle_path: Vec<ModuleId>,
    /// The type of cycle detected
    pub cycle_type: CycleType,
    /// Impact assessment of the cycle
    pub impact: CycleImpact,
    /// Suggested resolution strategies
    pub resolution_suggestions: Vec<String>,
}

/// Types of dependency cycles.
#[derive(Debug, Clone, PartialEq)]
pub enum CycleType {
    /// Simple direct cycle (A -> B -> A)
    Direct,
    /// Complex multi-module cycle (A -> B -> C -> D -> A)
    Complex,
    /// Self-dependency (A -> A)
    SelfDependency,
}

/// Assessment of cycle impact on the system.
#[derive(Debug, Clone, PartialEq)]
pub enum CycleImpact {
    /// High impact - prevents proper initialization
    Critical,
    /// Medium impact - may cause runtime issues
    Moderate,
    /// Low impact - manageable with careful ordering
    Low,
}

/// Configuration for circular dependency detection.
#[derive(Debug, Clone)]
pub struct CircularDependencyConfig {
    /// Maximum search depth for cycle detection
    pub max_search_depth: usize,
    /// Whether to enable advanced cycle analysis
    pub enable_advanced_analysis: bool,
    /// Whether to suggest resolution strategies
    pub suggest_resolutions: bool,
    /// Timeout for dependency resolution
    pub resolution_timeout: Duration,
}

impl Default for CircularDependencyConfig {
    fn default() -> Self {
        Self {
            max_search_depth: 1000,
            enable_advanced_analysis: true,
            suggest_resolutions: true,
            resolution_timeout: Duration::from_secs(60),
        }
    }
}

/// Performance statistics for dependency resolution.
#[derive(Debug, Default, Clone)]
pub struct ResolutionStatistics {
    /// Total resolutions performed
    pub total_resolutions: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Cycles detected
    pub cycles_detected: u64,
    /// Total resolution time
    pub total_resolution_time: Duration,
    /// Average resolution time
    pub average_resolution_time: Duration,
}

impl EnhancedDependencyResolver {
    /// Creates a new enhanced dependency resolver.
    pub fn new() -> Self {
        Self::with_config(CircularDependencyConfig::default())
    }

    /// Creates a resolver with custom configuration.
    pub fn with_config(config: CircularDependencyConfig) -> Self {
        Self {
            dependency_cache: HashMap::new(),
            resolution_stats: ResolutionStatistics::default(),
            detection_config: config,
        }
    }

    /// Resolves dependencies for a module with enhanced analysis.
    pub fn resolve_dependencies_enhanced(
        &mut self,
        module: Module,
    ) -> Result<(Module, DependencyGraph)> {
        let start_time = Instant::now();
        let module_id = module.id.clone();

        // Check cache first
        if let Some(cached_graph) = self.dependency_cache.get(&module_id) {
            self.resolution_stats.cache_hits += 1;
            return Ok((module, cached_graph.clone()));
        }

        self.resolution_stats.cache_misses += 1;
        self.resolution_stats.total_resolutions += 1;

        // Build dependency graph with cycle detection
        let dependency_graph = self.build_dependency_graph(&module)?;

        // Cache the result
        self.dependency_cache
            .insert(module_id, dependency_graph.clone());

        // Update statistics
        let resolution_time = start_time.elapsed();
        self.resolution_stats.total_resolution_time += resolution_time;
        self.resolution_stats.average_resolution_time = self.resolution_stats.total_resolution_time
            / self.resolution_stats.total_resolutions as u32;

        Ok((module, dependency_graph))
    }

    /// Builds a comprehensive dependency graph for a module.
    fn build_dependency_graph(&mut self, module: &Module) -> Result<DependencyGraph> {
        let mut graph = DependencyGraph {
            root: module.id.clone(),
            direct_dependencies: module.dependencies.clone(),
            transitive_dependencies: Vec::new(),
            dependency_levels: HashMap::new(),
            detected_cycles: Vec::new(),
            resolved_at: std::time::SystemTime::now(),
        };

        // Perform topological sort with cycle detection
        let (topological_order, cycles) =
            self.topological_sort_with_cycle_detection(&graph.root, &module.dependencies)?;

        graph.transitive_dependencies = topological_order;
        graph.detected_cycles = cycles;

        // Update cycle statistics
        if !graph.detected_cycles.is_empty() {
            self.resolution_stats.cycles_detected += graph.detected_cycles.len() as u64;
        }

        // Compute dependency levels
        graph.dependency_levels = self.compute_dependency_levels(&graph)?;

        // Generate resolution suggestions if cycles are detected
        if self.detection_config.suggest_resolutions {
            for cycle in &mut graph.detected_cycles {
                cycle.resolution_suggestions = self.generate_resolution_suggestions(cycle);
            }
        }

        Ok(graph)
    }

    /// Performs topological sort with comprehensive cycle detection.
    fn topological_sort_with_cycle_detection(
        &self,
        root: &ModuleId,
        dependencies: &[ModuleId],
    ) -> Result<(Vec<ModuleId>, Vec<DependencyCycle>)> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        let mut topological_order = Vec::new();
        let mut detected_cycles = Vec::new();
        let mut current_path = Vec::new();

        // Start DFS from the root
        self.dfs_with_cycle_detection(
            root,
            dependencies,
            &mut visited,
            &mut recursion_stack,
            &mut topological_order,
            &mut detected_cycles,
            &mut current_path,
            0,
        )?;

        Ok((topological_order, detected_cycles))
    }

    /// Depth-first search with enhanced cycle detection.
    #[allow(clippy::too_many_arguments)]
    fn dfs_with_cycle_detection(
        &self,
        current: &ModuleId,
        all_dependencies: &[ModuleId],
        visited: &mut HashSet<ModuleId>,
        recursion_stack: &mut HashSet<ModuleId>,
        topological_order: &mut Vec<ModuleId>,
        detected_cycles: &mut Vec<DependencyCycle>,
        current_path: &mut Vec<ModuleId>,
        depth: usize,
    ) -> Result<()> {
        // Check maximum search depth
        if depth > self.detection_config.max_search_depth {
            return Err(Box::new(Error::from(ModuleError::CircularDependency(
                current_path.clone(),
            ))));
        }

        visited.insert(current.clone());
        recursion_stack.insert(current.clone());
        current_path.push(current.clone());

        // Check dependencies of current module
        for dep in all_dependencies {
            if dep == current {
                // Self-dependency
                let cycle = DependencyCycle {
                    cycle_path: vec![current.clone(), current.clone()],
                    cycle_type: CycleType::SelfDependency,
                    impact: CycleImpact::Critical,
                    resolution_suggestions: Vec::new(),
                };
                detected_cycles.push(cycle);
                continue;
            }

            if !visited.contains(dep) {
                // Recurse on unvisited dependency
                self.dfs_with_cycle_detection(
                    dep,
                    all_dependencies,
                    visited,
                    recursion_stack,
                    topological_order,
                    detected_cycles,
                    current_path,
                    depth + 1,
                )?;
            } else if recursion_stack.contains(dep) {
                // Found a cycle
                let cycle_start_index = current_path.iter().position(|id| id == dep).unwrap_or(0);
                let cycle_path: Vec<ModuleId> = current_path[cycle_start_index..]
                    .iter()
                    .cloned()
                    .chain(std::iter::once(dep.clone()))
                    .collect();

                let cycle_type = if cycle_path.len() == 2 {
                    CycleType::Direct
                } else {
                    CycleType::Complex
                };

                let impact = self.assess_cycle_impact(&cycle_path);

                let cycle = DependencyCycle {
                    cycle_path,
                    cycle_type,
                    impact,
                    resolution_suggestions: Vec::new(),
                };

                detected_cycles.push(cycle);
            }
        }

        recursion_stack.remove(current);
        current_path.pop();

        // Add to topological order
        if !topological_order.contains(current) {
            topological_order.push(current.clone());
        }

        Ok(())
    }

    /// Assesses the impact of a detected cycle.
    fn assess_cycle_impact(&self, cycle_path: &[ModuleId]) -> CycleImpact {
        match cycle_path.len() {
            2 => CycleImpact::Critical,     // Direct cycles are always critical
            3..=5 => CycleImpact::Moderate, // Small cycles are moderate
            _ => CycleImpact::Low,          // Large cycles might be manageable
        }
    }

    /// Computes dependency levels (distance from root).
    fn compute_dependency_levels(
        &self,
        graph: &DependencyGraph,
    ) -> Result<HashMap<ModuleId, usize>> {
        let mut levels = HashMap::new();
        let mut queue = VecDeque::new();

        // Root is at level 0
        levels.insert(graph.root.clone(), 0);
        queue.push_back((graph.root.clone(), 0));

        while let Some((current_id, current_level)) = queue.pop_front() {
            for dep_id in &graph.direct_dependencies {
                if !levels.contains_key(dep_id) {
                    let dep_level = current_level + 1;
                    levels.insert(dep_id.clone(), dep_level);
                    queue.push_back((dep_id.clone(), dep_level));
                }
            }
        }

        Ok(levels)
    }

    /// Generates resolution suggestions for a cycle.
    fn generate_resolution_suggestions(&self, cycle: &DependencyCycle) -> Vec<String> {
        let mut suggestions = Vec::new();

        match cycle.cycle_type {
            CycleType::SelfDependency => {
                suggestions.push(
                    "Remove self-dependency - modules should not depend on themselves".to_string(),
                );
            }
            CycleType::Direct => {
                suggestions
                    .push("Introduce an intermediate module to break the direct cycle".to_string());
                suggestions
                    .push("Extract common functionality into a shared base module".to_string());
            }
            CycleType::Complex => {
                suggestions.push("Refactor modules to reduce coupling".to_string());
                suggestions.push("Use dependency inversion to break the cycle".to_string());
                suggestions.push(
                    "Consider splitting large modules into smaller, focused modules".to_string(),
                );
            }
        }

        match cycle.impact {
            CycleImpact::Critical => {
                suggestions.push(
                    "This cycle must be resolved before the module can be loaded".to_string(),
                );
            }
            CycleImpact::Moderate => {
                suggestions.push(
                    "Consider resolving this cycle to improve system reliability".to_string(),
                );
            }
            CycleImpact::Low => {
                suggestions.push(
                    "This cycle may be acceptable with careful initialization order".to_string(),
                );
            }
        }

        suggestions
    }

    /// Gets the dependency graph for a module (if cached).
    pub fn get_dependency_graph(&self, id: &ModuleId) -> Option<&DependencyGraph> {
        self.dependency_cache.get(id)
    }

    /// Gets performance statistics.
    pub fn get_statistics(&self) -> &ResolutionStatistics {
        &self.resolution_stats
    }

    /// Clears the dependency cache.
    pub fn clear_cache(&mut self) {
        self.dependency_cache.clear();
    }

    /// Validates a complete module dependency graph.
    pub fn validate_module_graph(
        &self,
        modules: &HashMap<ModuleId, Module>,
    ) -> Vec<GraphValidationError> {
        let mut errors = Vec::new();

        for (module_id, module) in modules {
            // Check for missing dependencies
            for dep_id in &module.dependencies {
                if !modules.contains_key(dep_id) {
                    errors.push(GraphValidationError::MissingDependency {
                        module: module_id.clone(),
                        dependency: dep_id.clone(),
                    });
                }
            }

            // Check for self-dependencies
            if module.dependencies.contains(module_id) {
                errors.push(GraphValidationError::SelfDependency(module_id.clone()));
            }
        }

        // Check for cycles in the complete graph
        if let Ok((_, cycles)) = self.analyze_complete_graph(modules) {
            for cycle in cycles {
                errors.push(GraphValidationError::CircularDependency(cycle));
            }
        }

        errors
    }

    /// Analyzes the complete module graph for global issues.
    fn analyze_complete_graph(
        &self,
        modules: &HashMap<ModuleId, Module>,
    ) -> Result<(Vec<ModuleId>, Vec<DependencyCycle>)> {
        let mut all_modules = Vec::new();
        let mut detected_cycles = Vec::new();
        let mut visited = HashSet::new();

        for module_id in modules.keys() {
            if !visited.contains(module_id) {
                // Perform analysis starting from this module
                let mut recursion_stack = HashSet::new();
                let mut current_path = Vec::new();

                self.analyze_module_subgraph(
                    module_id,
                    modules,
                    &mut visited,
                    &mut recursion_stack,
                    &mut current_path,
                    &mut detected_cycles,
                    0,
                )?;

                all_modules.push(module_id.clone());
            }
        }

        Ok((all_modules, detected_cycles))
    }

    /// Analyzes a subgraph starting from a specific module.
    #[allow(clippy::too_many_arguments)]
    fn analyze_module_subgraph(
        &self,
        current: &ModuleId,
        modules: &HashMap<ModuleId, Module>,
        visited: &mut HashSet<ModuleId>,
        recursion_stack: &mut HashSet<ModuleId>,
        current_path: &mut Vec<ModuleId>,
        detected_cycles: &mut Vec<DependencyCycle>,
        depth: usize,
    ) -> Result<()> {
        if depth > self.detection_config.max_search_depth {
            return Ok(()); // Stop recursion at max depth
        }

        visited.insert(current.clone());
        recursion_stack.insert(current.clone());
        current_path.push(current.clone());

        if let Some(current_module) = modules.get(current) {
            for dep_id in &current_module.dependencies {
                if !visited.contains(dep_id) {
                    self.analyze_module_subgraph(
                        dep_id,
                        modules,
                        visited,
                        recursion_stack,
                        current_path,
                        detected_cycles,
                        depth + 1,
                    )?;
                } else if recursion_stack.contains(dep_id) {
                    // Found cycle
                    let cycle_start = current_path.iter().position(|id| id == dep_id).unwrap_or(0);
                    let cycle_path: Vec<ModuleId> = current_path[cycle_start..]
                        .iter()
                        .cloned()
                        .chain(std::iter::once(dep_id.clone()))
                        .collect();

                    let cycle_type = if cycle_path.len() == 2 {
                        CycleType::Direct
                    } else {
                        CycleType::Complex
                    };

                    let cycle = DependencyCycle {
                        cycle_path,
                        cycle_type,
                        impact: CycleImpact::Moderate, // Default for complete graph analysis
                        resolution_suggestions: Vec::new(),
                    };

                    detected_cycles.push(cycle);
                }
            }
        }

        recursion_stack.remove(current);
        current_path.pop();

        Ok(())
    }
}

/// Graph validation errors.
#[derive(Debug, Clone)]
pub enum GraphValidationError {
    /// Missing dependency
    MissingDependency {
        /// The module that has the missing dependency
        module: ModuleId,
        /// The dependency that is missing
        dependency: ModuleId,
    },
    /// Self-dependency
    SelfDependency(ModuleId),
    /// Circular dependency
    CircularDependency(DependencyCycle),
}

impl std::fmt::Display for GraphValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphValidationError::MissingDependency { module, dependency } => {
                write!(
                    f,
                    "Module {} depends on missing module {}",
                    super::format_module_id(module),
                    super::format_module_id(dependency)
                )
            }
            GraphValidationError::SelfDependency(module) => {
                write!(
                    f,
                    "Module {} depends on itself",
                    super::format_module_id(module)
                )
            }
            GraphValidationError::CircularDependency(cycle) => {
                let cycle_str = cycle
                    .cycle_path
                    .iter()
                    .map(super::format_module_id)
                    .collect::<Vec<_>>()
                    .join(" -> ");
                write!(
                    f,
                    "Circular dependency: {} ({})",
                    cycle_str,
                    cycle.impact_description()
                )
            }
        }
    }
}

impl DependencyCycle {
    /// Gets a description of the cycle impact.
    pub fn impact_description(&self) -> &'static str {
        match self.impact {
            CycleImpact::Critical => "Critical",
            CycleImpact::Moderate => "Moderate",
            CycleImpact::Low => "Low",
        }
    }

    /// Gets the length of the cycle.
    pub fn cycle_length(&self) -> usize {
        self.cycle_path.len().saturating_sub(1) // Subtract 1 for the repeated node
    }
}

impl Default for EnhancedDependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_system::{ModuleMetadata, ModuleNamespace, ModuleSource};

    fn create_test_module(name: &str, deps: Vec<&str>) -> Module {
        Module {
            id: ModuleId {
                components: vec![name.to_string()],
                namespace: ModuleNamespace::User,
            },
            exports: HashMap::new(),
            dependencies: deps
                .into_iter()
                .map(|dep| ModuleId {
                    components: vec![dep.to_string()],
                    namespace: ModuleNamespace::User,
                })
                .collect(),
            source: Some(ModuleSource::Builtin),
            metadata: ModuleMetadata::default(),
        }
    }

    #[test]
    fn test_enhanced_resolver_creation() {
        let resolver = EnhancedDependencyResolver::new();
        assert_eq!(resolver.get_statistics().total_resolutions, 0);
    }

    #[test]
    fn test_simple_dependency_resolution() {
        let mut resolver = EnhancedDependencyResolver::new();
        let module = create_test_module("test", vec![]);

        let result = resolver.resolve_dependencies_enhanced(module);
        assert!(result.is_ok());

        let (_, graph) = result.unwrap();
        assert_eq!(graph.root.components[0], "test");
        assert!(graph.detected_cycles.is_empty());
    }

    #[test]
    fn test_self_dependency_detection() {
        let mut resolver = EnhancedDependencyResolver::new();
        let module = create_test_module("self_dep", vec!["self_dep"]);

        let result = resolver.resolve_dependencies_enhanced(module);
        assert!(result.is_ok());

        let (_, graph) = result.unwrap();
        assert!(!graph.detected_cycles.is_empty());
        assert_eq!(
            graph.detected_cycles[0].cycle_type,
            CycleType::SelfDependency
        );
    }

    #[test]
    fn test_cycle_impact_assessment() {
        let resolver = EnhancedDependencyResolver::new();

        // Test direct cycle impact
        let direct_cycle = vec![
            ModuleId {
                components: vec!["a".to_string()],
                namespace: ModuleNamespace::User,
            },
            ModuleId {
                components: vec!["b".to_string()],
                namespace: ModuleNamespace::User,
            },
        ];
        assert_eq!(
            resolver.assess_cycle_impact(&direct_cycle),
            CycleImpact::Critical
        );

        // Test complex cycle impact
        let complex_cycle = vec![
            ModuleId {
                components: vec!["a".to_string()],
                namespace: ModuleNamespace::User,
            },
            ModuleId {
                components: vec!["b".to_string()],
                namespace: ModuleNamespace::User,
            },
            ModuleId {
                components: vec!["c".to_string()],
                namespace: ModuleNamespace::User,
            },
            ModuleId {
                components: vec!["d".to_string()],
                namespace: ModuleNamespace::User,
            },
        ];
        assert_eq!(
            resolver.assess_cycle_impact(&complex_cycle),
            CycleImpact::Moderate
        );
    }

    #[test]
    fn test_statistics_tracking() {
        let mut resolver = EnhancedDependencyResolver::new();
        let module1 = create_test_module("test1", vec![]);
        let module2 = create_test_module("test2", vec![]);

        resolver.resolve_dependencies_enhanced(module1).unwrap();
        resolver.resolve_dependencies_enhanced(module2).unwrap();

        let stats = resolver.get_statistics();
        assert_eq!(stats.total_resolutions, 2);
        assert_eq!(stats.cache_misses, 2);
    }

    #[test]
    fn test_cache_functionality() {
        let mut resolver = EnhancedDependencyResolver::new();
        let module = create_test_module("cached", vec![]);
        let module_id = module.id.clone();

        // First resolution should be a cache miss
        resolver
            .resolve_dependencies_enhanced(module.clone())
            .unwrap();
        assert_eq!(resolver.get_statistics().cache_misses, 1);

        // Check if graph is cached
        let cached_graph = resolver.get_dependency_graph(&module_id);
        assert!(cached_graph.is_some());

        // Second resolution should be a cache hit
        resolver.resolve_dependencies_enhanced(module).unwrap();
        assert_eq!(resolver.get_statistics().cache_hits, 1);
    }
}
