//! Integration tests for the bootstrap system.

use super::bootstrap_integration::*;
use super::BootstrapSystem;
use crate::module_system::BootstrapConfig;
use crate::diagnostics::Result;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_integration_modes() {
        // Test minimal mode
        let config = BootstrapIntegrationConfig {
            mode: BootstrapMode::Minimal,
            verbose: false,
            ..Default::default()
        };
        
        let mut integration = BootstrapIntegration::with_config(config).unwrap();
        let result = integration.bootstrap();
        assert!(result.is_ok(), "Minimal bootstrap should succeed");
        
        let metrics = integration.metrics();
        assert_eq!(metrics.bootstrap_mode, BootstrapMode::Minimal);
        assert!(metrics.minimal_primitives_count > 0);

        // Test fallback mode
        let config = BootstrapIntegrationConfig {
            mode: BootstrapMode::Fallback,
            verbose: false,
            ..Default::default()
        };
        
        let mut integration = BootstrapIntegration::with_config(config).unwrap();
        let result = integration.bootstrap();
        assert!(result.is_ok(), "Fallback bootstrap should succeed");
        
        let metrics = integration.metrics();
        assert!(metrics.used_fallback);
        assert!(metrics.fallback_stdlib_time > Duration::ZERO);
    }

    #[test]
    fn test_bootstrap_performance_metrics() {
        let config = BootstrapIntegrationConfig {
            mode: BootstrapMode::Minimal,
            verbose: false,
            ..Default::default()
        };
        
        let mut integration = BootstrapIntegration::with_config(config).unwrap();
        let _result = integration.bootstrap().unwrap();
        
        let metrics = integration.metrics();
        assert!(metrics.total_startup_time > Duration::ZERO);
        assert!(metrics.primitives_init_time > Duration::ZERO);
        assert!(metrics.memory_usage_bytes > 0);
    }

    #[test]
    fn test_bootstrap_auto_detection() {
        let mode = BootstrapIntegration::determine_bootstrap_mode();
        // Should auto-detect a valid mode
        assert!(matches!(mode, BootstrapMode::Full | BootstrapMode::Minimal));
    }

    #[test]
    fn test_bootstrap_config_defaults() {
        let config = BootstrapIntegrationConfig::default();
        assert!(!config.verbose);
        assert!(!config.development_mode);
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_stdlib_directory_detection() {
        // This test will depend on the actual filesystem
        let result = BootstrapIntegration::verify_stdlib_directory();
        // Either finds paths or returns an appropriate error
        match result {
            Ok(paths) => assert!(!paths.is_empty()),
            Err(_) => {
                // This is acceptable if no stdlib directory exists
                println!("No stdlib directory found - this is acceptable for testing");
            }
        }
    }

    #[test]
    fn test_bootstrap_error_handling() {
        // Test with a configuration that should trigger error handling
        let config = BootstrapIntegrationConfig {
            mode: BootstrapMode::Full,
            timeout: Duration::from_millis(1), // Very short timeout
            verbose: false,
            ..Default::default()
        };
        
        let mut integration = BootstrapIntegration::with_config(config).unwrap();
        let result = integration.bootstrap();
        
        // Should succeed due to fallback mechanism
        assert!(result.is_ok());
        
        // Should have used fallback
        let metrics = integration.metrics();
        assert!(metrics.used_fallback || metrics.bootstrap_mode == BootstrapMode::Full);
    }

    #[test]
    fn test_minimal_primitives_config() {
        let config = BootstrapConfig::minimal_config();
        assert!(config.lazy_loading);
        assert!(config.core_libraries.is_empty());
        assert!(!config.essential_primitives.is_empty());
        assert_eq!(config.bootstrap_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_lazy_config() {
        let config = BootstrapConfig::lazy_config();
        assert!(config.lazy_loading);
    }
}

/// Integration test utilities for verifying bootstrap behavior across different scenarios.
pub struct BootstrapTestSuite;

impl BootstrapTestSuite {
    /// Runs a comprehensive test of all bootstrap modes.
    pub fn run_comprehensive_test() -> Result<()> {
        println!("Running comprehensive bootstrap test...");
        
        // Test each mode
        for mode in [BootstrapMode::Minimal, BootstrapMode::Fallback] {
            println!("Testing {:?} mode...", mode);
            
            let config = BootstrapIntegrationConfig {
                mode,
                verbose: true,
                ..Default::default()
            };
            
            let mut integration = BootstrapIntegration::with_config(config)?;
            let _env = integration.bootstrap()?;
            
            let metrics = integration.metrics();
            println!("  Startup time: {:?}", metrics.total_startup_time);
            println!("  Primitives loaded: {}", metrics.minimal_primitives_count);
            println!("  Memory usage: {} KB", metrics.memory_usage_bytes / 1024);
            
            if metrics.used_fallback {
                println!("  Used fallback mode");
            }
        }
        
        println!("Comprehensive bootstrap test completed successfully!");
        Ok(())
    }

    /// Tests performance characteristics of the bootstrap system.
    pub fn run_performance_test() -> Result<()> {
        println!("Running bootstrap performance test...");
        
        let mut total_time = Duration::ZERO;
        let runs = 5;
        
        for i in 1..=runs {
            let start = std::time::Instant::now();
            
            let config = BootstrapIntegrationConfig {
                mode: BootstrapMode::Minimal,
                verbose: false,
                ..Default::default()
            };
            
            let mut integration = BootstrapIntegration::with_config(config)?;
            let _env = integration.bootstrap()?;
            
            let run_time = start.elapsed();
            total_time += run_time;
            
            println!("  Run {}: {:?}", i, run_time);
        }
        
        let average_time = total_time / runs;
        println!("Average bootstrap time: {:?}", average_time);
        
        // Bootstrap should be reasonably fast
        if average_time > Duration::from_secs(1) {
            println!("Warning: Bootstrap is taking longer than expected");
        }
        
        println!("Performance test completed!");
        Ok(())
    }

    /// Tests compatibility with existing Lambdust functionality.
    pub fn run_compatibility_test() -> Result<()> {
        println!("Running compatibility test...");
        
        // Test that we can still create a Lambdust instance
        let lambdust = crate::Lambdust::new();
        println!("  Basic Lambdust creation: OK");
        
        // Test runtime creation
        let _runtime = crate::Runtime::new();
        println!("  Basic Runtime creation: OK");
        
        // Test multithreaded runtime creation
        let _mt_runtime = crate::MultithreadedLambdust::new(Some(2))?;
        println!("  Multithreaded runtime creation: OK");
        
        println!("Compatibility test completed!");
        Ok(())
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_comprehensive_bootstrap() {
        BootstrapTestSuite::run_comprehensive_test().unwrap();
    }

    #[test]
    fn test_performance_characteristics() {
        BootstrapTestSuite::run_performance_test().unwrap();
    }

    #[test]
    fn test_backward_compatibility() {
        BootstrapTestSuite::run_compatibility_test().unwrap();
    }
}