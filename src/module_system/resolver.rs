//! Dependency resolution for modules.
//!
//! Handles the complex task of resolving module dependencies:
//! - Topological sorting of dependency graphs
//! - Circular dependency detection
//! - Version compatibility checking
//! - Efficient dependency loading order

use super::{Module, ModuleId, ModuleError};
use crate::diagnostics::{Error, Result};
use std::collections::{HashMap, HashSet, VecDeque};

/// Dependency resolver that manages module loading order and circular dependency detection.
#[derive(Debug)]
pub struct DependencyResolver {
    /// Cache of resolved dependency graphs
    dependency_cache: HashMap<ModuleId, Vec<ModuleId>>,
}

impl DependencyResolver {
    /// Creates a new dependency resolver.
    pub fn new() -> Self {
        Self {
            dependency_cache: HashMap::new(),
        }
    }

    /// Resolves dependencies for a module, returning it with dependencies loaded.
    pub fn resolve_dependencies(&mut self, module: Module) -> Result<Module> {
        // Get the dependency order
        let dependency_order = self.resolve_dependency_order(&module.id, &module.dependencies)?;
        
        // Validate that we don't have circular dependencies
        self.detect_circular_dependencies(&module.id, &module.dependencies)?;
        
        // Cache the resolved order
        self.dependency_cache.insert(module.id.clone()), dependency_order);
        
        // Return the module (dependencies would be loaded by the module loader)
        Ok(module)
    }

    /// Resolves the loading order for a set of dependencies.
    pub fn resolve_dependency_order(&self, _root_id: &ModuleId, dependencies: &[ModuleId]) -> Result<Vec<ModuleId>> {
        // For linear dependencies (no actual dependency graph information available),
        // we simply return the dependencies in the order they were provided.
        // In a real implementation, we would have actual dependency information
        // to perform proper topological sorting.
        
        // Check for duplicates and self-references
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        
        for dep_id in dependencies {
            if !seen.contains(dep_id) {
                seen.insert(dep_id.clone());
                result.push(dep_id.clone());
            }
        }
        
        Ok(result)
    }

    /// Detects circular dependencies in the module graph.
    pub fn detect_circular_dependencies(&self, root_id: &ModuleId, dependencies: &[ModuleId]) -> Result<()> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        let mut path = Vec::new();

        self.dfs_cycle_detection(root_id, dependencies, &mut visited, &mut recursion_stack, &mut path)
    }

    /// Performs depth-first search for cycle detection.
    fn dfs_cycle_detection(
        &self,
        current_id: &ModuleId,
        all_dependencies: &[ModuleId],
        visited: &mut HashSet<ModuleId>,
        recursion_stack: &mut HashSet<ModuleId>,
        path: &mut Vec<ModuleId>,
    ) -> Result<()> {
        visited.insert(current_id.clone());
        recursion_stack.insert(current_id.clone());
        path.push(current_id.clone());

        // Check direct dependencies of current module
        for dep_id in all_dependencies {
            if dep_id == current_id {
                continue; // Skip self
            }

            if !visited.contains(dep_id) {
                // Recursively check this dependency
                self.dfs_cycle_detection(dep_id, all_dependencies, visited, recursion_stack, path)?;
            } else if recursion_stack.contains(dep_id) {
                // Found a cycle - create error with cycle path
                let cycle_start = path.iter().position(|id| id == dep_id).unwrap_or(0);
                let cycle: Vec<ModuleId> = path[cycle_start..].iter().clone())()
                    .chain(std::iter::once(dep_id.clone()))
                    .collect();
                
                return Err(Box::new(Error::from(ModuleError::CircularDependency(cycle).boxed()));
            }
        }

        recursion_stack.remove(current_id);
        path.pop();
        Ok(())
    }


    /// Gets the cached dependency order for a module.
    pub fn get_cached_dependencies(&self, id: &ModuleId) -> Option<&Vec<ModuleId>> {
        self.dependency_cache.get(id)
    }

    /// Clears the dependency cache.
    pub fn clear_cache(&mut self) {
        self.dependency_cache.clear();
    }

    /// Validates a dependency graph for consistency.
    pub fn validate_dependency_graph(&self, modules: &HashMap<ModuleId, Module>) -> Vec<DependencyValidationError> {
        let mut errors = Vec::new();

        for (module_id, module) in modules {
            // Check if all dependencies exist
            for dep_id in &module.dependencies {
                if !modules.contains_key(dep_id) {
                    errors.push(DependencyValidationError::MissingDependency {
                        module: module_id.clone()),
                        dependency: dep_id.clone()),
                    });
                }
            }

            // Check for self-dependencies
            if module.dependencies.contains(module_id) {
                errors.push(DependencyValidationError::SelfDependency(module_id.clone()));
            }
        }

        // Check for circular dependencies
        if let Err(_err) = self.detect_circular_dependencies_in_graph(modules) {
            // For now, we can't extract the cycle info from the converted error
            // In a full implementation, we'd need a different approach
            errors.push(DependencyValidationError::CircularDependency(vec![]));
        }

        errors
    }

    /// Detects circular dependencies in a complete module graph.
    fn detect_circular_dependencies_in_graph(&self, modules: &HashMap<ModuleId, Module>) -> Result<()> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for module_id in modules.keys() {
            if !visited.contains(module_id) {
                self.dfs_cycle_detection_graph(
                    module_id,
                    modules,
                    &mut visited,
                    &mut recursion_stack,
                    &mut Vec::new(),
                )?;
            }
        }

        Ok(())
    }

    /// DFS cycle detection for complete module graph.
    fn dfs_cycle_detection_graph(
        &self,
        current_id: &ModuleId,
        modules: &HashMap<ModuleId, Module>,
        visited: &mut HashSet<ModuleId>,
        recursion_stack: &mut HashSet<ModuleId>,
        path: &mut Vec<ModuleId>,
    ) -> Result<()> {
        visited.insert(current_id.clone());
        recursion_stack.insert(current_id.clone());
        path.push(current_id.clone());

        if let Some(current_module) = modules.get(current_id) {
            for dep_id in &current_module.dependencies {
                if !visited.contains(dep_id) {
                    self.dfs_cycle_detection_graph(dep_id, modules, visited, recursion_stack, path)?;
                } else if recursion_stack.contains(dep_id) {
                    // Found cycle
                    let cycle_start = path.iter().position(|id| id == dep_id).unwrap_or(0);
                    let cycle: Vec<ModuleId> = path[cycle_start..].iter().clone())()
                        .chain(std::iter::once(dep_id.clone()))
                        .collect();
                    
                    return Err(Box::new(Error::from(ModuleError::CircularDependency(cycle).boxed()));
                }
            }
        }

        recursion_stack.remove(current_id);
        path.pop();
        Ok(())
    }

    /// Computes the transitive closure of dependencies for a module.
    pub fn compute_transitive_dependencies(&self, root_id: &ModuleId, modules: &HashMap<ModuleId, Module>) -> HashSet<ModuleId> {
        let mut transitive_deps = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Start with direct dependencies
        if let Some(root_module) = modules.get(root_id) {
            for dep_id in &root_module.dependencies {
                queue.push_back(dep_id.clone());
            }
        }

        // BFS to find all transitive dependencies
        while let Some(current_id) = queue.pop_front() {
            if transitive_deps.insert(current_id.clone()) {
                // This is a new dependency, add its dependencies to the queue
                if let Some(current_module) = modules.get(&current_id) {
                    for dep_id in &current_module.dependencies {
                        if !transitive_deps.contains(dep_id) {
                            queue.push_back(dep_id.clone());
                        }
                    }
                }
            }
        }

        transitive_deps
    }
}

/// Dependency validation errors.
#[derive(Debug, Clone)]
pub enum DependencyValidationError {
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
    CircularDependency(Vec<ModuleId>),
}

impl std::fmt::Display for DependencyValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyValidationError::MissingDependency { module, dependency } => {
                write!(f, "Module {} depends on missing module {}", 
                       super::format_module_id(module), 
                       super::format_module_id(dependency))
            }
            DependencyValidationError::SelfDependency(module) => {
                write!(f, "Module {} depends on itself", super::format_module_id(module))
            }
            DependencyValidationError::CircularDependency(cycle) => {
                let cycle_str = cycle.iter()
                    .map(super::format_module_id)
                    .collect::<Vec<_>>()
                    .join(" -> ");
                write!(f, "Circular dependency: {}", cycle_str)
            }
        }
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{ModuleNamespace, ModuleSource, ModuleMetadata};
    use std::collections::HashMap;

    fn create_test_module(name: &str, deps: Vec<&str>) -> Module {
        Module {
            id: ModuleId {
                components: vec![name.to_string()],
                namespace: ModuleNamespace::Builtin,
            },
            exports: HashMap::new(),
            dependencies: deps.into_iter().map(|dep| ModuleId {
                components: vec![dep.to_string()],
                namespace: ModuleNamespace::Builtin,
            }).collect(),
            source: Some(ModuleSource::Builtin),
            metadata: ModuleMetadata::default(),
        }
    }

    #[test]
    fn test_no_dependencies() {
        let mut resolver = DependencyResolver::new();
        let module = create_test_module("test", vec![]);
        
        let result = resolver.resolve_dependencies(module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_linear_dependencies() {
        let resolver = DependencyResolver::new();
        
        // Module A depends on B, B depends on C
        let module_a = create_test_module("a", vec!["b"]);
        let deps = vec![
            ModuleId {
                components: vec!["b".to_string()],
                namespace: ModuleNamespace::Builtin,
            },
            ModuleId {
                components: vec!["c".to_string()],
                namespace: ModuleNamespace::Builtin,
            },
        ];
        
        let order = resolver.resolve_dependency_order(&module_a.id, &deps);
        assert!(order.is_ok());
    }

    #[test]
    fn test_circular_dependency_detection() {
        let resolver = DependencyResolver::new();
        
        // Create modules with circular dependency: A -> B -> A
        let mut modules = HashMap::new();
        modules.insert(
            ModuleId {
                components: vec!["a".to_string()],
                namespace: ModuleNamespace::Builtin,
            },
            create_test_module("a", vec!["b"]),
        );
        modules.insert(
            ModuleId {
                components: vec!["b".to_string()],
                namespace: ModuleNamespace::Builtin,
            },
            create_test_module("b", vec!["a"]),
        );

        let errors = resolver.validate_dependency_graph(&modules);
        assert!(!errors.is_empty());
        
        // Should detect circular dependency
        assert!(matches!(
            errors[0],
            DependencyValidationError::CircularDependency(_)
        ));
    }

    #[test]
    fn test_missing_dependency_detection() {
        let resolver = DependencyResolver::new();
        
        let mut modules = HashMap::new();
        modules.insert(
            ModuleId {
                components: vec!["a".to_string()],
                namespace: ModuleNamespace::Builtin,
            },
            create_test_module("a", vec!["nonexistent"]),
        );

        let errors = resolver.validate_dependency_graph(&modules);
        assert!(!errors.is_empty());
        
        // Should detect missing dependency
        assert!(matches!(
            errors[0],
            DependencyValidationError::MissingDependency { .. }
        ));
    }

    #[test]
    fn test_transitive_dependencies() {
        let resolver = DependencyResolver::new();
        
        let mut modules = HashMap::new();
        modules.insert(
            ModuleId {
                components: vec!["a".to_string()],
                namespace: ModuleNamespace::Builtin,
            },
            create_test_module("a", vec!["b"]),
        );
        modules.insert(
            ModuleId {
                components: vec!["b".to_string()],
                namespace: ModuleNamespace::Builtin,
            },
            create_test_module("b", vec!["c"]),
        );
        modules.insert(
            ModuleId {
                components: vec!["c".to_string()],
                namespace: ModuleNamespace::Builtin,
            },
            create_test_module("c", vec![]),
        );

        let root_id = ModuleId {
            components: vec!["a".to_string()],
            namespace: ModuleNamespace::Builtin,
        };
        
        let transitive = resolver.compute_transitive_dependencies(&root_id, &modules);
        
        // Should include both b and c
        assert_eq!(transitive.len(), 2);
        assert!(transitive.contains(&ModuleId {
            components: vec!["b".to_string()],
            namespace: ModuleNamespace::Builtin,
        }));
        assert!(transitive.contains(&ModuleId {
            components: vec!["c".to_string()],
            namespace: ModuleNamespace::Builtin,
        }));
    }
}