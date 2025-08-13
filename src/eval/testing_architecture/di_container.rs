use std::collections::HashMap;
use std::sync::Arc;

/// Dependency injection container for the monadic evaluator.
///
/// This container manages all dependencies and allows for easy substitution
/// of mock implementations during testing.
pub struct DIContainer {
    /// Registered dependencies by type name
    dependencies: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    
    /// Singleton instances
    singletons: HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
    
    /// Factory functions for creating instances
    factories: HashMap<String, Box<dyn Fn() -> Box<dyn std::any::Any + Send + Sync> + Send + Sync>>,
    
    /// Configuration for the container
    config: super::DIConfiguration,
}

impl std::fmt::Debug for DIContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DIContainer {{ dependencies: <{}>, singletons: <{}>, factories: <{}>, config: {:?} }}",
               self.dependencies.len(),
               self.singletons.len(), 
               self.factories.len(),
               self.config)
    }
}

impl Default for DIContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl DIContainer {
    /// Create a new DI container
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            singletons: HashMap::new(),
            factories: HashMap::new(),
            config: super::DIConfiguration::default(),
        }
    }
    
    /// Register a dependency instance
    pub fn register<T: Send + Sync + 'static>(&mut self, name: &str, instance: T) {
        self.dependencies.insert(
            name.to_string(),
            Box::new(instance)
        );
    }
    
    /// Register a singleton dependency
    pub fn register_singleton<T: Send + Sync + 'static>(&mut self, name: &str, instance: T) {
        self.singletons.insert(
            name.to_string(),
            Arc::new(instance)
        );
    }
    
    /// Register a factory function
    pub fn register_factory<T: Send + Sync + 'static, F>(&mut self, name: &str, factory: F)
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.factories.insert(
            name.to_string(),
            Box::new(move || Box::new(factory()))
        );
    }
    
    /// Resolve a dependency by name
    pub fn resolve<T: Send + Sync + 'static>(&self, name: &str) -> Option<&T> {
        // Try singleton first
        if let Some(singleton) = self.singletons.get(name) {
            return singleton.downcast_ref::<T>();
        }
        
        // Try regular dependency
        if let Some(dependency) = self.dependencies.get(name) {
            return dependency.downcast_ref::<T>();
        }
        
        None
    }
    
    /// Create instance using factory
    pub fn create<T: Send + Sync + 'static>(&self, name: &str) -> Option<T> {
        if let Some(factory) = self.factories.get(name) {
            let instance = factory();
            return instance.downcast::<T>().ok().map(|boxed| *boxed);
        }
        None
    }
    
    /// Check if a dependency is registered
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.contains_key(name) ||
        self.singletons.contains_key(name) ||
        self.factories.contains_key(name)
    }
    
    /// List all registered dependencies
    pub fn list_dependencies(&self) -> Vec<String> {
        let mut deps: Vec<String> = self.dependencies.keys().cloned().collect();
        deps.extend(self.singletons.keys().cloned());
        deps.extend(self.factories.keys().cloned());
        deps.sort();
        deps.dedup();
        deps
    }
}