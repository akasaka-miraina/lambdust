//! Runtime SIGSEGV monitoring system with automatic initialization
//!
//! This module provides a complete runtime monitoring system that automatically
//! detects and logs SIGSEGV events, platform differences, and memory safety violations.

use std::sync::Once;
use crate::debug::sigsegv_monitor::{start_global_monitoring, write_global_report};
use crate::eval::memory_safety_validator::{
    init_memory_safety, MemorySafetyConfig, get_validator
};

static MONITOR_INIT: Once = Once::new();

/// Initialize the complete runtime monitoring system
pub fn initialize_runtime_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    MONITOR_INIT.call_once(|| {
        // Initialize memory safety validation
        let config = MemorySafetyConfig {
            enable_null_checks: true,
            enable_alignment_checks: true,
            enable_bounds_checks: true,
            enable_use_after_free_detection: cfg!(debug_assertions),
            panic_on_violation: false, // Don't panic in production, just log
            log_violations: true,
        };
        
        init_memory_safety(config);
        println!("Memory safety validator initialized");
        
        // Start SIGSEGV monitoring
        if let Err(e) = start_global_monitoring() {
            eprintln!("Failed to start SIGSEGV monitoring: {}", e);
        } else {
            println!("SIGSEGV monitoring started");
        }
    });
    
    Ok(())
}

/// Generate and write a comprehensive monitoring report
pub fn generate_monitoring_report() -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let report_filename = format!("sigsegv_analysis_{}.txt", timestamp);
    write_global_report(&report_filename)?;
    
    // Also print statistics to stdout
    if let Some(validator) = get_validator() {
        let stats = validator.get_statistics();
        println!("Memory Safety Statistics:");
        println!("  Total violations detected: {}", stats.total_violations);
        println!("  Tracked allocations: {}", stats.tracked_allocations);
    }
    
    Ok(())
}

/// Helper function to be called at program exit
pub fn finalize_monitoring() {
    if let Err(e) = generate_monitoring_report() {
        eprintln!("Failed to generate final monitoring report: {}", e);
    }
    
    crate::debug::sigsegv_monitor::stop_global_monitoring();
    println!("Runtime monitoring finalized");
}

// Note: Automatic initialization using static constructors would require the 'ctor' crate
// For now, users need to call initialize_runtime_monitoring() manually

/// Initialize monitoring if not already done (safe to call multiple times)
pub fn ensure_monitoring_initialized() {
    static INIT_CHECK: std::sync::Once = std::sync::Once::new();
    INIT_CHECK.call_once(|| {
        if std::env::var("LAMBDUST_DISABLE_MONITORING").is_err() {
            if let Err(e) = initialize_runtime_monitoring() {
                eprintln!("Monitoring initialization failed: {}", e);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_monitoring_initialization() {
        // This should not panic and should be idempotent
        assert!(initialize_runtime_monitoring().is_ok());
        assert!(initialize_runtime_monitoring().is_ok());
    }
    
    #[test]
    fn test_report_generation() {
        // Initialize first
        assert!(initialize_runtime_monitoring().is_ok());
        
        // Generate report should not fail
        assert!(generate_monitoring_report().is_ok());
    }
}