//! Comprehensive library system integration.
//!
//! This module provides the complete, production-ready library system that integrates
//! all components following domain-driven design principles:
//! - Enhanced module system with runtime integration
//! - Dynamic loading with hot-reload support
//! - R7RS compliance validation
//! - Comprehensive error handling and diagnostics
//! - Performance monitoring and optimization

use super::{
    Module, ModuleError, ModuleId,
    dynamic_loader::{DynamicLibraryLoader, FileMonitorConfig, HotReloadManager},
    enhanced_dependency_resolver::EnhancedDependencyResolver,
    enhanced_module_system::{AutoLoadingConfig, EnhancedModuleSystem},
    r7rs_compliance::{ComplianceConfig, R7RSLibrarySystem},
    runtime_integration::{ImportSpecResolver, LibraryInstance, LibraryInstantiator},
};
use crate::ast::{Expr, Spanned};
use crate::diagnostics::{Error, Result, Span};
use crate::eval::Value;
use crate::runtime::GlobalEnvironmentManager;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Comprehensive library system providing complete functionality.
#[derive(Debug)]
pub struct ComprehensiveLibrarySystem {
    /// Core enhanced module system
    core_system: EnhancedModuleSystem,
    /// Dynamic loading system
    dynamic_loader: DynamicLibraryLoader,
    /// Hot-reload manager
    hot_reload_manager: Option<HotReloadManager>,
    /// R7RS compliance validator
    r7rs_system: R7RSLibrarySystem,
    /// Enhanced dependency resolver
    dependency_resolver: EnhancedDependencyResolver,
    /// System configuration
    config: LibrarySystemConfig,
    /// Performance monitoring
    performance_monitor: PerformanceMonitor,
    /// Error context for enhanced diagnostics
    error_context: ErrorContext,
}

/// Configuration for the comprehensive library system.
#[derive(Debug, Clone)]
pub struct LibrarySystemConfig {
    /// Auto-loading configuration
    pub auto_loading: AutoLoadingConfig,
    /// File monitoring configuration
    pub file_monitoring: FileMonitorConfig,
    /// R7RS compliance configuration
    pub r7rs_compliance: ComplianceConfig,
    /// Whether to enable hot-reload support
    pub enable_hot_reload: bool,
    /// Whether to enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Whether to enable verbose diagnostics
    pub verbose_diagnostics: bool,
    /// Maximum library loading timeout
    pub loading_timeout: Duration,
}

impl Default for LibrarySystemConfig {
    fn default() -> Self {
        Self {
            auto_loading: AutoLoadingConfig::default(),
            file_monitoring: FileMonitorConfig::default(),
            r7rs_compliance: ComplianceConfig::default(),
            enable_hot_reload: true,
            enable_performance_monitoring: true,
            verbose_diagnostics: false,
            loading_timeout: Duration::from_secs(60),
        }
    }
}

/// Performance monitoring for the library system.
#[derive(Debug, Default)]
pub struct PerformanceMonitor {
    /// Library loading times
    loading_times: HashMap<ModuleId, Duration>,
    /// Total number of operations
    total_operations: u64,
    /// Total time spent in operations
    total_time: Duration,
    /// Cache hit ratio
    cache_hits: u64,
    /// Cache misses
    cache_misses: u64,
    /// Hot-reload statistics
    hot_reload_count: u64,
    /// Error counts by type
    error_counts: HashMap<String, u64>,
}

/// Error context for enhanced diagnostics.
#[derive(Debug, Default)]
pub struct ErrorContext {
    /// Current operation being performed
    current_operation: Option<String>,
    /// Stack of operations
    operation_stack: Vec<String>,
    /// Additional context information
    context_info: HashMap<String, String>,
}

impl ComprehensiveLibrarySystem {
    /// Creates a new comprehensive library system with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(LibrarySystemConfig::default())
    }

    /// Creates a comprehensive library system with custom configuration.
    pub fn with_config(config: LibrarySystemConfig) -> Result<Self> {
        let core_system = EnhancedModuleSystem::with_config(config.auto_loading.clone())?;
        let dynamic_loader = DynamicLibraryLoader::with_config(config.file_monitoring.clone());
        let r7rs_system = R7RSLibrarySystem::with_config(config.r7rs_compliance.clone());
        let dependency_resolver = EnhancedDependencyResolver::new();

        let hot_reload_manager = if config.enable_hot_reload {
            Some(HotReloadManager::new(DynamicLibraryLoader::with_config(
                config.file_monitoring.clone(),
            )))
        } else {
            None
        };

        Ok(Self {
            core_system,
            dynamic_loader,
            hot_reload_manager,
            r7rs_system,
            dependency_resolver,
            config,
            performance_monitor: PerformanceMonitor::default(),
            error_context: ErrorContext::default(),
        })
    }

    /// Loads and instantiates a library with comprehensive error handling.
    pub fn load_library(&mut self, module_id: &ModuleId) -> Result<Arc<LibraryInstance>> {
        let start_time = Instant::now();
        self.error_context.push_operation(&format!(
            "load_library: {}",
            super::format_module_id(module_id)
        ));

        let result = self.load_library_impl(module_id);

        let duration = start_time.elapsed();
        self.performance_monitor
            .record_operation("load_library", duration, result.is_ok());

        if let Ok(ref instance) = result {
            self.performance_monitor
                .loading_times
                .insert(module_id.clone(), duration);
        }

        self.error_context.pop_operation();
        result
    }

    /// Core implementation of library loading.
    fn load_library_impl(&mut self, module_id: &ModuleId) -> Result<Arc<LibraryInstance>> {
        // Validate the module ID format
        self.validate_module_id(module_id)?;

        // Load the module using the core system
        let module = self.core_system.load_module(module_id)?;

        // Validate R7RS compliance if required
        if self.config.r7rs_compliance.strict_compliance {
            let compliance_report = self.r7rs_system.validate_library_compliance(&module)?;
            if !compliance_report.is_compliant {
                return Err(Box::new(Error::from(ModuleError::CompilationError(
                    format!(
                        "Library fails R7RS compliance: {}",
                        compliance_report.summary()
                    ),
                ))));
            }
        }

        // Perform enhanced dependency analysis
        let (resolved_module, dependency_graph) = self
            .dependency_resolver
            .resolve_dependencies_enhanced((*module).clone())?;

        // Check for circular dependencies
        if !dependency_graph.detected_cycles.is_empty() {
            let cycle_descriptions: Vec<String> = dependency_graph
                .detected_cycles
                .iter()
                .map(|cycle| {
                    format!(
                        "{} ({})",
                        cycle
                            .cycle_path
                            .iter()
                            .map(super::format_module_id)
                            .collect::<Vec<_>>()
                            .join(" -> "),
                        cycle.impact_description()
                    )
                })
                .collect();

            return Err(Box::new(Error::from(ModuleError::CircularDependency(
                dependency_graph.detected_cycles[0].cycle_path.clone(),
            ))));
        }

        // Instantiate the library
        let library_instance = self.core_system.instantiate_library(module_id)?;

        // Set up dynamic loading if hot-reload is enabled
        if self.config.enable_hot_reload {
            let _dynamic_instance = self.dynamic_loader.load_library_dynamic(
                resolved_module,
                self.core_system.instantiator(),
                true,
            )?;
        }

        Ok(library_instance)
    }

    /// Processes an import specification with comprehensive error handling.
    pub fn process_import(
        &mut self,
        import_expr: &Spanned<Expr>,
    ) -> Result<HashMap<String, Value>> {
        let start_time = Instant::now();
        self.error_context.push_operation("process_import");

        let result = self.process_import_impl(import_expr);

        let duration = start_time.elapsed();
        self.performance_monitor
            .record_operation("process_import", duration, result.is_ok());

        self.error_context.pop_operation();
        result
    }

    /// Core implementation of import processing.
    fn process_import_impl(
        &mut self,
        import_expr: &Spanned<Expr>,
    ) -> Result<HashMap<String, Value>> {
        // Resolve the import specification
        let import_resolution = ImportSpecResolver::resolve_import_spec(import_expr)?;

        // Use the enhanced module system to resolve imports
        self.core_system.resolve_import_from_expr(import_expr)
    }

    /// Processes an export specification with comprehensive validation.
    pub fn process_export(
        &self,
        export_expr: &Spanned<Expr>,
        library_env: &HashMap<String, Value>,
    ) -> Result<Vec<super::r7rs_compliance::ExportBinding>> {
        self.r7rs_system
            .process_export_spec(export_expr, library_env)
    }

    /// Enables hot-reload support for development.
    pub fn enable_hot_reload(&mut self) -> Result<()> {
        if let Some(ref mut manager) = self.hot_reload_manager {
            manager.start()?;
            self.core_system.enable_hot_reload()?;
        }
        Ok(())
    }

    /// Disables hot-reload support.
    pub fn disable_hot_reload(&mut self) {
        if let Some(ref mut manager) = self.hot_reload_manager {
            manager.stop();
        }
    }

    /// Checks for file modifications and triggers reloads.
    pub fn check_for_updates(&mut self) -> Result<Vec<ModuleId>> {
        if let Some(ref mut manager) = self.hot_reload_manager {
            let events = manager.process_events();
            let mut reloaded = Vec::new();

            for event in events {
                match event {
                    super::dynamic_loader::ReloadEvent::LibraryReloaded { module_id, .. } => {
                        reloaded.push(module_id);
                    }
                    super::dynamic_loader::ReloadEvent::ReloadFailed { module_id, error } => {
                        eprintln!(
                            "Hot-reload failed for {}: {}",
                            super::format_module_id(&module_id),
                            error
                        );
                    }
                    _ => {}
                }
            }

            // Also check for file modifications
            let modifications = manager
                .loader()
                .check_for_modifications(self.core_system.instantiator())?;
            reloaded.extend(modifications);

            Ok(reloaded)
        } else {
            Ok(Vec::new())
        }
    }

    /// Gets comprehensive system status.
    pub fn get_system_status(&self) -> SystemStatus {
        let validation_report = self.core_system.validate_system().unwrap_or_else(|_| {
            super::enhanced_module_system::SystemValidationReport {
                library_validation: crate::runtime::LibraryValidationReport {
                    primary_lib_dir_valid: false,
                    found_search_paths: Vec::new(),
                    missing_critical_subdirs: Vec::new(),
                    found_library_files: HashMap::new(),
                    recommendations: Vec::new(),
                },
                instantiation_stats: super::runtime_integration::InstantiationStatistics {
                    instantiated_libraries: 0,
                    max_instantiation_depth: 0,
                    cached_bindings: 0,
                },
                cached_modules: 0,
                available_modules: 0,
            }
        });

        let dynamic_stats = self.dynamic_loader.get_statistics();
        let dependency_stats = self.dependency_resolver.get_statistics();

        SystemStatus {
            core_system_health: validation_report.is_healthy(),
            total_libraries: validation_report.available_modules,
            instantiated_libraries: validation_report.instantiation_stats.instantiated_libraries,
            cached_modules: validation_report.cached_modules,
            hot_reload_enabled: self.config.enable_hot_reload,
            active_dynamic_libraries: dynamic_stats.total_active_libraries,
            dependency_resolution_stats: dependency_stats.clone(),
            performance_stats: self.performance_monitor.get_summary(),
            r7rs_compliance_enabled: self.config.r7rs_compliance.strict_compliance,
        }
    }

    /// Validates a module ID according to system rules.
    fn validate_module_id(&self, module_id: &ModuleId) -> Result<()> {
        if module_id.components.is_empty() {
            return Err(Box::new(Error::from(ModuleError::InvalidDefinition(
                "Module ID cannot have empty components".to_string(),
            ))));
        }

        for component in &module_id.components {
            if component.is_empty() {
                return Err(Box::new(Error::from(ModuleError::InvalidDefinition(
                    "Module ID components cannot be empty".to_string(),
                ))));
            }
        }

        Ok(())
    }

    /// Gets performance statistics.
    pub fn get_performance_stats(&self) -> &PerformanceMonitor {
        &self.performance_monitor
    }

    /// Gets dependency resolution statistics.
    pub fn get_dependency_stats(
        &self,
    ) -> &super::enhanced_dependency_resolver::ResolutionStatistics {
        self.dependency_resolver.get_statistics()
    }

    /// Clears all caches for a fresh start.
    pub fn clear_all_caches(&mut self) {
        self.core_system.clear_all_caches();
        self.dependency_resolver.clear_cache();
        self.performance_monitor = PerformanceMonitor::default();
    }

    /// Lists all available libraries.
    pub fn list_libraries(&self) -> Vec<ModuleId> {
        self.core_system.list_modules()
    }

    /// Lists all instantiated libraries.
    pub fn list_instantiated_libraries(&self) -> Vec<ModuleId> {
        self.core_system.list_instantiated_libraries()
    }

    /// Gets the library path resolver for configuration.
    pub fn library_resolver(&self) -> &crate::runtime::LibraryPathResolver {
        self.core_system.library_resolver()
    }
}

impl ErrorContext {
    /// Pushes an operation onto the context stack.
    fn push_operation(&mut self, operation: &str) {
        self.operation_stack.push(operation.to_string());
        self.current_operation = Some(operation.to_string());
    }

    /// Pops an operation from the context stack.
    fn pop_operation(&mut self) {
        self.operation_stack.pop();
        self.current_operation = self.operation_stack.last().cloned();
    }

    /// Adds context information.
    fn add_context(&mut self, key: String, value: String) {
        self.context_info.insert(key, value);
    }

    /// Gets a formatted error context.
    fn format_context(&self) -> String {
        let mut context = String::new();

        if let Some(ref op) = self.current_operation {
            context.push_str(&format!("Operation: {op}\n"));
        }

        if !self.operation_stack.is_empty() {
            context.push_str(&format!(
                "Call stack: {}\n",
                self.operation_stack.join(" -> ")
            ));
        }

        if !self.context_info.is_empty() {
            context.push_str("Context:\n");
            for (key, value) in &self.context_info {
                context.push_str(&format!("  {key}: {value}\n"));
            }
        }

        context
    }
}

impl PerformanceMonitor {
    /// Records an operation with timing and success information.
    fn record_operation(&mut self, operation: &str, duration: Duration, success: bool) {
        self.total_operations += 1;
        self.total_time += duration;

        if success {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
            *self.error_counts.entry(operation.to_string()).or_insert(0) += 1;
        }
    }

    /// Gets a performance summary.
    fn get_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            total_operations: self.total_operations,
            total_time: self.total_time,
            average_operation_time: if self.total_operations > 0 {
                self.total_time / self.total_operations as u32
            } else {
                Duration::ZERO
            },
            cache_hit_ratio: if self.total_operations > 0 {
                self.cache_hits as f64 / self.total_operations as f64
            } else {
                0.0
            },
            error_rate: if self.total_operations > 0 {
                self.cache_misses as f64 / self.total_operations as f64
            } else {
                0.0
            },
            hot_reload_count: self.hot_reload_count,
        }
    }
}

/// Overall system status information.
#[derive(Debug, Clone)]
pub struct SystemStatus {
    /// Whether the core system is healthy
    pub core_system_health: bool,
    /// Total number of available libraries
    pub total_libraries: usize,
    /// Number of instantiated libraries
    pub instantiated_libraries: usize,
    /// Number of cached modules
    pub cached_modules: usize,
    /// Whether hot-reload is enabled
    pub hot_reload_enabled: bool,
    /// Number of active dynamic libraries
    pub active_dynamic_libraries: usize,
    /// Dependency resolution statistics
    pub dependency_resolution_stats: super::enhanced_dependency_resolver::ResolutionStatistics,
    /// Performance statistics
    pub performance_stats: PerformanceSummary,
    /// Whether R7RS compliance checking is enabled
    pub r7rs_compliance_enabled: bool,
}

/// Performance summary information.
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    /// Total number of operations performed
    pub total_operations: u64,
    /// Total time spent in operations
    pub total_time: Duration,
    /// Average time per operation
    pub average_operation_time: Duration,
    /// Cache hit ratio (0.0 to 1.0)
    pub cache_hit_ratio: f64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Number of hot-reloads performed
    pub hot_reload_count: u64,
}

impl SystemStatus {
    /// Checks if the entire system is healthy.
    pub fn is_healthy(&self) -> bool {
        self.core_system_health &&
        self.error_rate() < 0.1 && // Less than 10% error rate
        self.total_libraries > 0
    }

    /// Gets the error rate from performance stats.
    pub fn error_rate(&self) -> f64 {
        self.performance_stats.error_rate
    }

    /// Gets a formatted status summary.
    pub fn summary(&self) -> String {
        format!(
            "Comprehensive Library System Status:\n\
             • Overall Health: {}\n\
             • Total Libraries: {}\n\
             • Instantiated Libraries: {}\n\
             • Cached Modules: {}\n\
             • Hot-Reload: {}\n\
             • Dynamic Libraries: {}\n\
             • Cache Hit Ratio: {:.1}%\n\
             • Error Rate: {:.1}%\n\
             • Average Operation Time: {:.2}ms\n\
             • R7RS Compliance: {}",
            if self.is_healthy() {
                "Healthy"
            } else {
                "Issues Detected"
            },
            self.total_libraries,
            self.instantiated_libraries,
            self.cached_modules,
            if self.hot_reload_enabled {
                "Enabled"
            } else {
                "Disabled"
            },
            self.active_dynamic_libraries,
            self.performance_stats.cache_hit_ratio * 100.0,
            self.performance_stats.error_rate * 100.0,
            self.performance_stats.average_operation_time.as_secs_f64() * 1000.0,
            if self.r7rs_compliance_enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        )
    }
}

impl Default for ComprehensiveLibrarySystem {
    fn default() -> Self {
        Self::new().expect("Failed to create comprehensive library system")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_system::{ModuleMetadata, ModuleNamespace, ModuleSource};

    fn create_test_module(name: &str) -> Module {
        Module {
            id: ModuleId {
                components: vec![name.to_string()],
                namespace: ModuleNamespace::User,
            },
            exports: HashMap::new(),
            dependencies: Vec::new(),
            source: Some(ModuleSource::Builtin),
            metadata: ModuleMetadata::default(),
        }
    }

    #[test]
    fn test_comprehensive_system_creation() {
        let system = ComprehensiveLibrarySystem::new();
        assert!(system.is_ok());
    }

    #[test]
    fn test_system_configuration() {
        let config = LibrarySystemConfig {
            enable_hot_reload: false,
            enable_performance_monitoring: true,
            verbose_diagnostics: true,
            ..Default::default()
        };

        let system = ComprehensiveLibrarySystem::with_config(config);
        assert!(system.is_ok());

        let system = system.unwrap();
        assert!(!system.config.enable_hot_reload);
        assert!(system.config.enable_performance_monitoring);
    }

    #[test]
    fn test_system_status() {
        let system = ComprehensiveLibrarySystem::new().unwrap();
        let status = system.get_system_status();

        // Basic sanity checks
        // Verify status is valid (total_libraries is unsigned, so always >= 0)
        assert!(status.instantiated_libraries <= status.total_libraries);
        assert!(status.performance_stats.cache_hit_ratio >= 0.0);
        assert!(status.performance_stats.cache_hit_ratio <= 1.0);

        let summary = status.summary();
        assert!(summary.contains("Comprehensive Library System Status"));
    }

    #[test]
    fn test_module_id_validation() {
        let system = ComprehensiveLibrarySystem::new().unwrap();

        // Valid module ID
        let valid_id = ModuleId {
            components: vec!["test".to_string(), "module".to_string()],
            namespace: ModuleNamespace::User,
        };
        assert!(system.validate_module_id(&valid_id).is_ok());

        // Invalid module ID (empty components)
        let invalid_id = ModuleId {
            components: vec![],
            namespace: ModuleNamespace::User,
        };
        assert!(system.validate_module_id(&invalid_id).is_err());
    }

    #[test]
    fn test_performance_monitoring() {
        let mut monitor = PerformanceMonitor::default();

        monitor.record_operation("test_op", Duration::from_millis(100), true);
        monitor.record_operation("test_op", Duration::from_millis(200), false);

        let summary = monitor.get_summary();
        assert_eq!(summary.total_operations, 2);
        assert_eq!(summary.cache_hit_ratio, 0.5);
        assert_eq!(summary.error_rate, 0.5);
    }

    #[test]
    fn test_error_context() {
        let mut context = ErrorContext::default();

        context.push_operation("test_operation");
        context.add_context("module_id".to_string(), "test-module".to_string());

        let formatted = context.format_context();
        assert!(formatted.contains("Operation: test_operation"));
        assert!(formatted.contains("module_id: test-module"));

        context.pop_operation();
        assert!(context.current_operation.is_none());
    }
}
