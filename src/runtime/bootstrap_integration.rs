//! Bootstrap integration for the main Lambdust binary.
//!
//! This module provides the integration layer between the main binary and the bootstrap system,
//! including performance measurement, error handling, and graceful degradation.

use super::{BootstrapSystem, GlobalEnvironmentManager, LibraryPathResolver, LibraryPathConfig};
use crate::module_system::BootstrapConfig;
use crate::diagnostics::{Result, Error};
use crate::stdlib::StandardLibrary;
use std::sync::Arc;
use std::time::{Instant, Duration};
use std::path::PathBuf;

/// Performance metrics for bootstrap process.
#[derive(Debug, Clone)]
pub struct BootstrapMetrics {
    /// Total startup time
    pub total_startup_time: Duration,
    /// Time spent in minimal primitives initialization
    pub primitives_init_time: Duration,
    /// Time spent in Scheme library loading
    pub scheme_loading_time: Duration,
    /// Time spent in fallback standard library population
    pub fallback_stdlib_time: Duration,
    /// Memory usage after bootstrap (estimated)
    pub memory_usage_bytes: usize,
    /// Number of Scheme libraries successfully loaded
    pub scheme_libraries_loaded: usize,
    /// Number of minimal primitives registered
    pub minimal_primitives_count: usize,
    /// Whether fallback to Rust stdlib was required
    pub used_fallback: bool,
    /// Bootstrap mode used
    pub bootstrap_mode: BootstrapMode,
}

/// Bootstrap mode selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapMode {
    /// Full bootstrap with Scheme libraries
    Full,
    /// Minimal bootstrap with essential primitives only
    Minimal,
    /// Fallback to legacy Rust standard library
    Fallback,
}

/// Bootstrap configuration for main binary integration.
#[derive(Debug, Clone)]
pub struct BootstrapIntegrationConfig {
    /// Bootstrap mode to use
    pub mode: BootstrapMode,
    /// Whether to enable verbose startup reporting
    pub verbose: bool,
    /// Maximum time to wait for bootstrap
    pub timeout: Duration,
    /// Search paths for Scheme libraries
    pub library_paths: Vec<PathBuf>,
    /// Whether to enable development mode features
    pub development_mode: bool,
    /// Whether to use lazy loading
    pub lazy_loading: bool,
}

/// Main bootstrap integration that coordinates the startup process.
#[derive(Debug)]
pub struct BootstrapIntegration {
    /// Integration configuration
    config: BootstrapIntegrationConfig,
    /// Performance metrics
    metrics: BootstrapMetrics,
    /// Bootstrap system instance
    bootstrap_system: Option<BootstrapSystem>,
    /// Global environment manager
    global_env: Option<Arc<GlobalEnvironmentManager>>,
}

impl BootstrapIntegration {
    /// Creates a new bootstrap integration with default configuration.
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: BootstrapIntegrationConfig::default(),
            metrics: BootstrapMetrics::default(),
            bootstrap_system: None,
            global_env: None,
        })
    }

    /// Creates a bootstrap integration with custom configuration.
    pub fn with_config(config: BootstrapIntegrationConfig) -> Result<Self> {
        Ok(Self {
            config,
            metrics: BootstrapMetrics::default(),
            bootstrap_system: None,
            global_env: None,
        })
    }

    /// Runs the complete bootstrap process with error handling and performance measurement.
    pub fn bootstrap(&mut self) -> Result<Arc<GlobalEnvironmentManager>> {
        let total_start = Instant::now();
        self.metrics.bootstrap_mode = self.config.mode;

        if self.config.verbose {
            println!("Lambdust bootstrap starting in {:?} mode...", self.config.mode);
        }

        // Attempt bootstrap based on configured mode
        let result = match self.config.mode {
            BootstrapMode::Full => self.attempt_full_bootstrap(),
            BootstrapMode::Minimal => self.attempt_minimal_bootstrap(),
            BootstrapMode::Fallback => self.fallback_to_rust_stdlib(),
        };

        // Handle bootstrap result with fallback logic
        let global_env = match result {
            Ok(env) => env,
            Err(e) => {
                if self.config.verbose {
                    eprintln!("Bootstrap failed: {e}. Falling back to Rust stdlib...");
                }
                self.metrics.used_fallback = true;
                self.fallback_to_rust_stdlib()?
            }
        };

        self.metrics.total_startup_time = total_start.elapsed();
        self.global_env = Some(global_env.clone());

        if self.config.verbose {
            self.report_bootstrap_metrics();
        }

        Ok(global_env)
    }

    /// Attempts full bootstrap with Scheme libraries.
    fn attempt_full_bootstrap(&mut self) -> Result<Arc<GlobalEnvironmentManager>> {
        let bootstrap_config = self.create_bootstrap_config();
        let mut bootstrap_system = BootstrapSystem::with_config(bootstrap_config)?;

        if self.config.development_mode {
            bootstrap_system.enable_development_mode();
        }

        let primitives_start = Instant::now();
        let global_env = bootstrap_system.bootstrap()?;
        self.metrics.primitives_init_time = primitives_start.elapsed();

        // Extract statistics from bootstrap system
        let stats = bootstrap_system.statistics();
        self.metrics.scheme_loading_time = stats.libraries_load_time;
        self.metrics.scheme_libraries_loaded = stats.libraries_count;
        self.metrics.minimal_primitives_count = stats.primitives_count;
        self.metrics.memory_usage_bytes = stats.memory_usage_bytes;

        self.bootstrap_system = Some(bootstrap_system);
        Ok(global_env)
    }

    /// Attempts minimal bootstrap with essential primitives only.
    fn attempt_minimal_bootstrap(&mut self) -> Result<Arc<GlobalEnvironmentManager>> {
        let bootstrap_config = BootstrapConfig::minimal_config();
        let mut bootstrap_system = BootstrapSystem::with_config(bootstrap_config)?;

        let primitives_start = Instant::now();
        let global_env = bootstrap_system.bootstrap()?;
        self.metrics.primitives_init_time = primitives_start.elapsed();

        // Extract statistics
        let stats = bootstrap_system.statistics();
        self.metrics.minimal_primitives_count = stats.primitives_count;
        self.metrics.memory_usage_bytes = stats.memory_usage_bytes;

        self.bootstrap_system = Some(bootstrap_system);
        Ok(global_env)
    }

    /// Falls back to the legacy Rust standard library population.
    fn fallback_to_rust_stdlib(&mut self) -> Result<Arc<GlobalEnvironmentManager>> {
        let fallback_start = Instant::now();
        
        // Create a GlobalEnvironmentManager for tracking purposes
        let global_env_manager = Arc::new(GlobalEnvironmentManager::new());
        
        // Get the thread-local global environment and populate it
        let thread_local_env = crate::eval::environment::global_environment();
        let thread_safe_env = thread_local_env.to_thread_safe();
        
        let stdlib = StandardLibrary::new();
        stdlib.populate_environment(&thread_safe_env);
        
        self.metrics.fallback_stdlib_time = fallback_start.elapsed();
        self.metrics.used_fallback = true;
        
        // Estimate primitives count from stdlib
        self.metrics.minimal_primitives_count = stdlib.builtins().len();
        
        Ok(global_env_manager)
    }

    /// Creates bootstrap configuration from integration settings.
    fn create_bootstrap_config(&self) -> BootstrapConfig {
        // Add custom search paths
        // Note: This would need to be implemented in BootstrapConfig
        // config.add_search_paths(&self.config.library_paths);
        
        if self.config.lazy_loading {
            BootstrapConfig::lazy_config()
        } else {
            BootstrapConfig::new_default()
        }
    }

    /// Reports bootstrap performance metrics to the user.
    fn report_bootstrap_metrics(&self) {
        let metrics = &self.metrics;
        
        println!("Bootstrap completed successfully:");
        println!("• Total startup time: {:?}", metrics.total_startup_time);
        println!("• Primitives loaded: {}", metrics.minimal_primitives_count);
        
        if metrics.scheme_libraries_loaded > 0 {
            println!("• Scheme libraries loaded: {}", metrics.scheme_libraries_loaded);
            println!("• Library loading time: {:?}", metrics.scheme_loading_time);
        }
        
        if metrics.used_fallback {
            println!("• Used fallback Rust stdlib: {:?}", metrics.fallback_stdlib_time);
        }
        
        println!("• Estimated memory usage: {} KB", metrics.memory_usage_bytes / 1024);
    }

    /// Gets the bootstrap metrics.
    pub fn metrics(&self) -> &BootstrapMetrics {
        &self.metrics
    }

    /// Gets the global environment manager.
    pub fn global_environment(&self) -> Option<Arc<GlobalEnvironmentManager>> {
        self.global_env.clone()
    }

    /// Gets the bootstrap system instance.
    pub fn bootstrap_system(&self) -> Option<&BootstrapSystem> {
        self.bootstrap_system.as_ref()
    }

    /// Checks if stdlib directory exists and contains expected files using library path resolver.
    pub fn verify_stdlib_directory() -> Result<Vec<PathBuf>> {
        match LibraryPathResolver::new() {
            Ok(resolver) => {
                let validation = resolver.validate_library_setup()?;
                if validation.is_usable() {
                    Ok(validation.found_search_paths)
                } else {
                    Err(Box::new(Error::io_error(format!(
                        "Library setup validation failed. Issues found: {}",
                        validation.missing_critical_subdirs.join(", ")
                    ))))
                }
            }
            Err(e) => {
                // Fallback to legacy method if resolver creation fails
                let mut found_paths = Vec::new();
                
                // Check relative path from current directory
                let current_stdlib = std::env::current_dir()
                    .map_err(|err| Error::io_error(format!("Failed to get current directory: {err}")))?
                    .join("stdlib");
                if current_stdlib.exists() && current_stdlib.is_dir() {
                    found_paths.push(current_stdlib);
                }

                // Check relative path from executable
                if let Ok(exe_path) = std::env::current_exe() {
                    if let Some(exe_dir) = exe_path.parent() {
                        let exe_stdlib = exe_dir.join("stdlib");
                        if exe_stdlib.exists() && exe_stdlib.is_dir() {
                            found_paths.push(exe_stdlib);
                        }
                    }
                }

                if found_paths.is_empty() {
                    Err(Box::new(Error::io_error(format!(
                        "No stdlib directory found and library resolver failed: {e}. Scheme libraries unavailable."
                    ))))
                } else {
                    Ok(found_paths)
                }
            }
        }
    }

    /// Determines the best bootstrap mode based on available resources.
    pub fn determine_bootstrap_mode() -> BootstrapMode {
        Self::determine_bootstrap_mode_with_config(LibraryPathConfig::default())
    }

    /// Determines bootstrap mode using a specific library path configuration.
    pub fn determine_bootstrap_mode_with_config(lib_config: LibraryPathConfig) -> BootstrapMode {
        match LibraryPathResolver::with_config(lib_config) {
            Ok(resolver) => {
                match resolver.validate_library_setup() {
                    Ok(validation) => {
                        if validation.is_usable() {
                            BootstrapMode::Full
                        } else {
                            BootstrapMode::Minimal
                        }
                    }
                    Err(_) => BootstrapMode::Minimal
                }
            }
            Err(_) => {
                // Fallback to legacy verification
                if Self::verify_stdlib_directory().is_ok() {
                    BootstrapMode::Full
                } else {
                    BootstrapMode::Minimal
                }
            }
        }
    }
}

impl Default for BootstrapMetrics {
    fn default() -> Self {
        Self {
            total_startup_time: Duration::ZERO,
            primitives_init_time: Duration::ZERO,
            scheme_loading_time: Duration::ZERO,
            fallback_stdlib_time: Duration::ZERO,
            memory_usage_bytes: 0,
            scheme_libraries_loaded: 0,
            minimal_primitives_count: 0,
            used_fallback: false,
            bootstrap_mode: BootstrapMode::Minimal,
        }
    }
}

impl Default for BootstrapIntegrationConfig {
    fn default() -> Self {
        Self {
            mode: BootstrapIntegration::determine_bootstrap_mode(),
            verbose: false,
            timeout: Duration::from_secs(30),
            library_paths: Vec::new(),
            development_mode: false,
            lazy_loading: false,
        }
    }
}

impl Default for BootstrapIntegration {
    fn default() -> Self {
        Self::new().expect("Failed to create default bootstrap integration")
    }
}

// Extension trait for BootstrapConfig to support integration features
impl BootstrapConfig {
    /// Creates a configuration optimized for lazy loading.
    pub fn lazy_config() -> Self {
        let mut config = Self::new_default();
        config.lazy_loading = true;
        config
    }

    /// Creates a minimal configuration with only essential primitives.
    pub fn minimal_config() -> Self {
        Self {
            essential_primitives: vec![
                "+".to_string(), "-".to_string(), "*".to_string(),
                "cons".to_string(), "car".to_string(), "cdr".to_string(),
                "=".to_string(), "<".to_string(),
                "null?".to_string(), "pair?".to_string(),
                "display".to_string(), "error".to_string(),
            ],
            core_libraries: Vec::new(), // No core libraries in minimal mode
            load_order: Vec::new(),
            lazy_loading: true,
            bootstrap_timeout: Duration::from_secs(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_integration_creation() {
        let integration = BootstrapIntegration::new();
        assert!(integration.is_ok());
    }

    #[test]
    fn test_bootstrap_config_modes() {
        let default_config = BootstrapIntegrationConfig::default();
        // Should automatically determine mode
        assert!(matches!(default_config.mode, BootstrapMode::Full | BootstrapMode::Minimal));

        let minimal_bootstrap_config = BootstrapConfig::minimal_config();
        assert!(minimal_bootstrap_config.lazy_loading);
        assert!(minimal_bootstrap_config.core_libraries.is_empty());
        assert!(!minimal_bootstrap_config.essential_primitives.is_empty());
    }

    #[test]
    fn test_stdlib_directory_detection() {
        // This test will depend on the actual directory structure
        // For now, just test that it doesn't panic
        let _result = BootstrapIntegration::verify_stdlib_directory();
    }

    #[test]
    fn test_bootstrap_mode_determination() {
        let mode = BootstrapIntegration::determine_bootstrap_mode();
        assert!(matches!(mode, BootstrapMode::Full | BootstrapMode::Minimal));
    }

    #[test]
    fn test_metrics_defaults() {
        let metrics = BootstrapMetrics::default();
        assert_eq!(metrics.total_startup_time, Duration::ZERO);
        assert_eq!(metrics.scheme_libraries_loaded, 0);
        assert!(!metrics.used_fallback);
    }
}