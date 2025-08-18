//! Runtime contract system integration and coordination.
//!
//! This module provides the main runtime integration point for the contract
//! system, coordinating between different components and providing a unified
//! interface for contract operations during program execution.

use crate::contracts::{
    ContractSystem, ContractConfig, ContractError, ContractResult,
    ast::ContractExpr,
    blame::{BlameInfo, BlameTarget, BlameBoundary, BoundaryType, BlameTracker},
    compiler::{CompiledContract, CompilationContext, OptimizationLevel},
    enforcement::{ContractEnforcement, PerformanceStats},
    evaluator::ContractEvaluator,
    predicates::PredicateRegistry,
};
use crate::eval::Value;
use crate::diagnostics::{Span, Spanned};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};

/// Runtime contract coordinator that manages all contract operations.
#[derive(Debug)]
pub struct ContractRuntime {
    /// Main contract system
    system: Arc<RwLock<ContractSystem>>,
    /// Contract evaluator for integration with main evaluator
    evaluator: Arc<RwLock<ContractEvaluator>>,
    /// Performance monitoring
    performance_monitor: Arc<Mutex<RuntimePerformanceMonitor>>,
    /// Runtime configuration
    config: ContractConfig,
    /// Global contract registry
    contract_registry: Arc<RwLock<ContractRegistry>>,
}

/// Registry for named contracts and contract templates.
#[derive(Debug)]
pub struct ContractRegistry {
    /// Named contracts
    named_contracts: HashMap<String, Arc<CompiledContract>>,
    /// Contract templates for parametric contracts
    templates: HashMap<String, ContractTemplate>,
    /// Contract aliases
    aliases: HashMap<String, String>,
}

/// Template for parametric contracts.
#[derive(Debug, Clone)]
pub struct ContractTemplate {
    /// Template name
    pub name: String,
    /// Parameter names
    pub parameters: Vec<String>,
    /// Template contract expression
    pub template: ContractExpr,
    /// Metadata
    pub metadata: TemplateMetadata,
}

/// Metadata for contract templates.
#[derive(Debug, Clone)]
pub struct TemplateMetadata {
    /// Documentation string
    pub documentation: String,
    /// Creation timestamp
    pub created_at: std::time::SystemTime,
    /// Usage count
    pub usage_count: usize,
}

/// Performance monitor for runtime contract operations.
#[derive(Debug)]
pub struct RuntimePerformanceMonitor {
    /// Contract compilation statistics
    pub compilation_stats: CompilationStats,
    /// Contract checking statistics
    pub checking_stats: CheckingStats,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Overall runtime statistics
    pub runtime_stats: RuntimeStats,
}

/// Statistics about contract compilation.
#[derive(Debug, Clone)]
pub struct CompilationStats {
    /// Total contracts compiled
    pub total_compiled: u64,
    /// Total compilation time
    pub total_compilation_time: std::time::Duration,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Average compilation time
    pub average_compilation_time: std::time::Duration,
}

/// Statistics about contract checking.
#[derive(Debug, Clone)]
pub struct CheckingStats {
    /// Total checks performed
    pub total_checks: u64,
    /// Total checking time
    pub total_checking_time: std::time::Duration,
    /// Success rate
    pub success_rate: f64,
    /// Average checking time
    pub average_checking_time: std::time::Duration,
}

/// Statistics about memory usage.
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Compiled contracts memory usage
    pub compiled_contracts_memory: usize,
    /// Wrapped values memory usage
    pub wrapped_values_memory: usize,
    /// Cache memory usage
    pub cache_memory: usize,
    /// Total memory usage
    pub total_memory: usize,
}

/// Overall runtime statistics.
#[derive(Debug, Clone)]
pub struct RuntimeStats {
    /// Runtime startup time
    pub startup_time: std::time::Duration,
    /// Total runtime uptime
    pub uptime: std::time::Duration,
    /// Number of active contracts
    pub active_contracts: usize,
    /// Number of contract violations
    pub total_violations: u64,
}

impl ContractRuntime {
    /// Creates a new contract runtime with default configuration.
    pub fn new() -> Self {
        Self::with_config(ContractConfig::default())
    }

    /// Creates a new contract runtime with custom configuration.
    pub fn with_config(config: ContractConfig) -> Self {
        let system = Arc::new(RwLock::new(ContractSystem::with_config(config.clone())));
        let evaluator = Arc::new(RwLock::new(ContractEvaluator::new(config.clone())));
        let performance_monitor = Arc::new(Mutex::new(RuntimePerformanceMonitor::new()));
        let contract_registry = Arc::new(RwLock::new(ContractRegistry::new()));

        Self {
            system,
            evaluator,
            performance_monitor,
            config,
            contract_registry,
        }
    }

    /// Compiles and registers a named contract.
    pub fn register_contract(
        &self,
        name: String,
        contract: ContractExpr,
        blame: BlameInfo,
    ) -> ContractResult<()> {
        let start_time = std::time::Instant::now();

        let compilation_context = CompilationContext::new(
            self.system.read().unwrap().predicates.clone(),
            blame,
        );

        let compiled = {
            let mut system = self.system.write().unwrap();
            system.compile_contract(&contract, &compilation_context)?
        };

        {
            let mut registry = self.contract_registry.write().unwrap();
            registry.named_contracts.insert(name, compiled);
        }

        // Update performance statistics
        {
            let mut monitor = self.performance_monitor.lock().unwrap();
            monitor.record_compilation(start_time.elapsed());
        }

        Ok(())
    }

    /// Looks up a named contract.
    pub fn lookup_contract(&self, name: &str) -> Option<Arc<CompiledContract>> {
        let registry = self.contract_registry.read().unwrap();
        registry.named_contracts.get(name).cloned()
    }

    /// Registers a contract template.
    pub fn register_template(
        &self,
        name: String,
        parameters: Vec<String>,
        template: ContractExpr,
        documentation: String,
    ) -> ContractResult<()> {
        let template = ContractTemplate {
            name: name.clone(),
            parameters,
            template,
            metadata: TemplateMetadata {
                documentation,
                created_at: std::time::SystemTime::now(),
                usage_count: 0,
            },
        };

        let mut registry = self.contract_registry.write().unwrap();
        registry.templates.insert(name, template);

        Ok(())
    }

    /// Instantiates a contract template.
    pub fn instantiate_template(
        &self,
        name: &str,
        arguments: Vec<ContractExpr>,
        blame: BlameInfo,
    ) -> ContractResult<Arc<CompiledContract>> {
        let template = {
            let mut registry = self.contract_registry.write().unwrap();
            let template = registry.templates.get_mut(name)
                .ok_or_else(|| ContractError::RuntimeError {
                    message: format!("Unknown contract template: {name}"),
                    location: blame.boundary.location,
                })?;
            
            template.metadata.usage_count += 1;
            template.clone()
        };

        if arguments.len() != template.parameters.len() {
            return Err(ContractError::RuntimeError {
                message: format!(
                    "Template {} expects {} arguments, got {}",
                    name,
                    template.parameters.len(),
                    arguments.len()
                ),
                location: blame.boundary.location,
            }.into());
        }

        // TODO: Substitute parameters in template with arguments
        // For now, just compile the template as-is
        let compilation_context = CompilationContext::new(
            self.system.read().unwrap().predicates.clone(),
            blame,
        );

        let mut system = self.system.write().unwrap();
        system.compile_contract(&template.template, &compilation_context)
    }

    /// Creates a contract alias.
    pub fn create_alias(&self, alias: String, target: String) -> ContractResult<()> {
        let mut registry = self.contract_registry.write().unwrap();
        
        // Check that target exists
        if !registry.named_contracts.contains_key(&target) && 
           !registry.templates.contains_key(&target) {
            return Err(ContractError::RuntimeError {
                message: format!("Cannot create alias for unknown contract: {target}"),
                location: Span::new(0, 0),
            }.into());
        }

        registry.aliases.insert(alias, target);
        Ok(())
    }

    /// Resolves a contract name (following aliases).
    pub fn resolve_name(&self, name: &str) -> String {
        let registry = self.contract_registry.read().unwrap();
        
        let mut current = name.to_string();
        let mut visited = std::collections::HashSet::new();
        
        while let Some(target) = registry.aliases.get(&current) {
            if visited.contains(&current) {
                // Circular alias - return original name
                break;
            }
            visited.insert(current.clone());
            current = target.clone();
        }
        
        current
    }

    /// Checks a value against a named contract.
    pub fn check_named_contract(
        &self,
        value: &Value,
        contract_name: &str,
        blame: &BlameInfo,
    ) -> ContractResult<()> {
        let resolved_name = self.resolve_name(contract_name);
        let contract = self.lookup_contract(&resolved_name)
            .ok_or_else(|| ContractError::RuntimeError {
                message: format!("Unknown contract: {contract_name}"),
                location: blame.boundary.location,
            })?;

        let start_time = std::time::Instant::now();
        let result = {
            let mut system = self.system.write().unwrap();
            system.check_contract(value, &contract, blame)
        };

        // Update performance statistics
        {
            let mut monitor = self.performance_monitor.lock().unwrap();
            monitor.record_check(start_time.elapsed(), result.is_ok());
        }

        result
    }

    /// Wraps a value with a named contract.
    pub fn wrap_with_named_contract(
        &self,
        value: Value,
        contract_name: &str,
        blame: BlameInfo,
    ) -> ContractResult<Value> {
        let resolved_name = self.resolve_name(contract_name);
        let contract = self.lookup_contract(&resolved_name)
            .ok_or_else(|| ContractError::RuntimeError {
                message: format!("Unknown contract: {contract_name}"),
                location: blame.boundary.location,
            })?;

        let mut system = self.system.write().unwrap();
        system.wrap_with_contract(value, contract, blame)
    }

    /// Gets runtime performance statistics.
    pub fn performance_stats(&self) -> RuntimePerformanceStats {
        let monitor = self.performance_monitor.lock().unwrap();
        let enforcement_stats = {
            let evaluator = self.evaluator.read().unwrap();
            evaluator.performance_stats()
        };
        
        RuntimePerformanceStats {
            compilation: monitor.compilation_stats.clone(),
            checking: monitor.checking_stats.clone(),
            memory: monitor.memory_stats.clone(),
            runtime: monitor.runtime_stats.clone(),
            enforcement: enforcement_stats,
        }
    }

    /// Updates the runtime configuration.
    pub fn update_config(&mut self, config: ContractConfig) {
        self.config = config.clone();
        
        {
            let mut system = self.system.write().unwrap();
            system.update_config(config.clone());
        }
        
        {
            let mut evaluator = self.evaluator.write().unwrap();
            evaluator.update_config(config);
        }
    }

    /// Clears all caches.
    pub fn clear_caches(&self) {
        {
            let mut system = self.system.write().unwrap();
            system.clear_cache();
        }
        
        {
            let mut evaluator = self.evaluator.write().unwrap();
            evaluator.clear_caches();
        }
    }

    /// Gets the number of registered contracts.
    pub fn contract_count(&self) -> usize {
        let registry = self.contract_registry.read().unwrap();
        registry.named_contracts.len()
    }

    /// Gets the number of registered templates.
    pub fn template_count(&self) -> usize {
        let registry = self.contract_registry.read().unwrap();
        registry.templates.len()
    }

    /// Lists all registered contract names.
    pub fn list_contracts(&self) -> Vec<String> {
        let registry = self.contract_registry.read().unwrap();
        registry.named_contracts.keys().cloned().collect()
    }

    /// Lists all registered template names.
    pub fn list_templates(&self) -> Vec<String> {
        let registry = self.contract_registry.read().unwrap();
        registry.templates.keys().cloned().collect()
    }
}

/// Combined performance statistics for the runtime.
#[derive(Debug, Clone)]
pub struct RuntimePerformanceStats {
    /// Contract compilation statistics
    pub compilation: CompilationStats,
    /// Contract checking statistics
    pub checking: CheckingStats,
    /// Memory usage statistics
    pub memory: MemoryStats,
    /// Overall runtime statistics
    pub runtime: RuntimeStats,
    /// Contract enforcement performance statistics
    pub enforcement: PerformanceStats,
}

impl ContractRegistry {
    /// Creates a new empty contract registry.
    pub fn new() -> Self {
        Self {
            named_contracts: HashMap::new(),
            templates: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    /// Clears all registered contracts and templates.
    pub fn clear(&mut self) {
        self.named_contracts.clear();
        self.templates.clear();
        self.aliases.clear();
    }
}

impl RuntimePerformanceMonitor {
    /// Creates a new performance monitor.
    pub fn new() -> Self {
        Self {
            compilation_stats: CompilationStats::new(),
            checking_stats: CheckingStats::new(),
            memory_stats: MemoryStats::new(),
            runtime_stats: RuntimeStats::new(),
        }
    }

    /// Records a contract compilation.
    pub fn record_compilation(&mut self, duration: std::time::Duration) {
        self.compilation_stats.total_compiled += 1;
        self.compilation_stats.total_compilation_time += duration;
        self.compilation_stats.average_compilation_time = 
            self.compilation_stats.total_compilation_time / self.compilation_stats.total_compiled as u32;
    }

    /// Records a contract check.
    pub fn record_check(&mut self, duration: std::time::Duration, success: bool) {
        self.checking_stats.total_checks += 1;
        self.checking_stats.total_checking_time += duration;
        self.checking_stats.average_checking_time = 
            self.checking_stats.total_checking_time / self.checking_stats.total_checks as u32;
        
        if success {
            let total_successes = (self.checking_stats.success_rate * (self.checking_stats.total_checks - 1) as f64) + 1.0;
            self.checking_stats.success_rate = total_successes / self.checking_stats.total_checks as f64;
        } else {
            let total_successes = self.checking_stats.success_rate * (self.checking_stats.total_checks - 1) as f64;
            self.checking_stats.success_rate = total_successes / self.checking_stats.total_checks as f64;
        }
    }
}

impl Default for CompilationStats {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilationStats {
    /// Creates a new compilation statistics instance with default values
    pub fn new() -> Self {
        Self {
            total_compiled: 0,
            total_compilation_time: std::time::Duration::new(0, 0),
            cache_hit_rate: 0.0,
            average_compilation_time: std::time::Duration::new(0, 0),
        }
    }
}

impl Default for CheckingStats {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckingStats {
    /// Creates a new checking statistics instance with default values
    pub fn new() -> Self {
        Self {
            total_checks: 0,
            total_checking_time: std::time::Duration::new(0, 0),
            success_rate: 1.0,
            average_checking_time: std::time::Duration::new(0, 0),
        }
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStats {
    /// Creates a new memory statistics instance with default values
    pub fn new() -> Self {
        Self {
            compiled_contracts_memory: 0,
            wrapped_values_memory: 0,
            cache_memory: 0,
            total_memory: 0,
        }
    }
}

impl Default for RuntimeStats {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeStats {
    /// Creates a new runtime statistics instance with default values
    pub fn new() -> Self {
        Self {
            startup_time: std::time::Duration::new(0, 0),
            uptime: std::time::Duration::new(0, 0),
            active_contracts: 0,
            total_violations: 0,
        }
    }
}

impl Default for ContractRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ContractRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RuntimePerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::ast::ContractExpr;
    use crate::diagnostics::Span;

    fn create_test_blame() -> BlameInfo {
        BlameInfo::function_definition(
            "test".to_string(),
            None,
            Span::new(0, 10),
            "test contract".to_string(),
        )
    }

    #[test]
    fn test_contract_runtime_creation() {
        let runtime = ContractRuntime::new();
        assert_eq!(runtime.contract_count(), 0);
        assert_eq!(runtime.template_count(), 0);
    }

    #[test]
    fn test_contract_registration() {
        let runtime = ContractRuntime::new();
        let blame = create_test_blame();
        
        let contract = ContractExpr::Predicate {
            name: "number?".to_string(),
            location: Span::new(0, 10),
        };
        
        let result = runtime.register_contract(
            "my-number-contract".to_string(),
            contract,
            blame,
        );
        
        assert!(result.is_ok());
        assert_eq!(runtime.contract_count(), 1);
        
        let lookup_result = runtime.lookup_contract("my-number-contract");
        assert!(lookup_result.is_some());
    }

    #[test]
    fn test_template_registration() {
        let runtime = ContractRuntime::new();
        
        let template = ContractExpr::Predicate {
            name: "number?".to_string(),
            location: Span::new(0, 10),
        };
        
        let result = runtime.register_template(
            "parameterized-contract".to_string(),
            vec!["T".to_string()],
            template,
            "A parameterized contract template".to_string(),
        );
        
        assert!(result.is_ok());
        assert_eq!(runtime.template_count(), 1);
    }

    #[test]
    fn test_contract_aliases() {
        let runtime = ContractRuntime::new();
        let blame = create_test_blame();
        
        // Register a contract
        let contract = ContractExpr::Predicate {
            name: "number?".to_string(),
            location: Span::new(0, 10),
        };
        
        runtime.register_contract(
            "number-contract".to_string(),
            contract,
            blame,
        ).unwrap();
        
        // Create an alias
        let result = runtime.create_alias(
            "num".to_string(),
            "number-contract".to_string(),
        );
        assert!(result.is_ok());
        
        // Test alias resolution
        assert_eq!(runtime.resolve_name("num"), "number-contract");
        assert_eq!(runtime.resolve_name("number-contract"), "number-contract");
    }

    #[test]
    fn test_contract_lookup() {
        let runtime = ContractRuntime::new();
        let blame = create_test_blame();
        
        let contract = ContractExpr::Predicate {
            name: "string?".to_string(),
            location: Span::new(0, 10),
        };
        
        runtime.register_contract(
            "string-contract".to_string(),
            contract,
            blame,
        ).unwrap();
        
        let found = runtime.lookup_contract("string-contract");
        assert!(found.is_some());
        
        let not_found = runtime.lookup_contract("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_performance_monitoring() {
        let runtime = ContractRuntime::new();
        let stats = runtime.performance_stats();
        
        assert_eq!(stats.compilation.total_compiled, 0);
        assert_eq!(stats.checking.total_checks, 0);
        assert_eq!(stats.runtime.total_violations, 0);
    }

    #[test]
    fn test_cache_clearing() {
        let runtime = ContractRuntime::new();
        
        // This should not panic
        runtime.clear_caches();
    }

    #[test]
    fn test_contract_listing() {
        let runtime = ContractRuntime::new();
        let blame = create_test_blame();
        
        assert!(runtime.list_contracts().is_empty());
        assert!(runtime.list_templates().is_empty());
        
        let contract = ContractExpr::Predicate {
            name: "boolean?".to_string(),
            location: Span::new(0, 10),
        };
        
        runtime.register_contract(
            "bool-contract".to_string(),
            contract,
            blame,
        ).unwrap();
        
        let contracts = runtime.list_contracts();
        assert_eq!(contracts.len(), 1);
        assert!(contracts.contains(&"bool-contract".to_string()));
    }
}