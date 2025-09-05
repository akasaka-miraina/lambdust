//! Comprehensive SIGSEGV debugging infrastructure for cross-platform analysis
//!
//! This module provides tools for:
//! - Signal handler with detailed backtrace capture
//! - Memory safety debugging and leak detection  
//! - Cross-platform crash analysis
//! - Test isolation and crash monitoring
//! - CI/CD integration for crash reporting

#![allow(missing_docs)]

pub mod signal_handler;
pub mod memory_tracker;
pub mod platform_detector;
pub mod test_harness;
pub mod crash_analyzer;
pub mod ci_reporter;
pub mod sigsegv_monitor;
pub mod runtime_monitor;
pub mod platform_tests;

pub use signal_handler::{SigsegvHandler, CrashInfo, MemoryInfo, install_signal_handlers, set_test_context};
pub use memory_tracker::{MemoryTracker, AllocationInfo, LeakReport};
pub use platform_detector::{PlatformInfo, detect_platform, compare_platforms};
pub use test_harness::{SafeTestHarness, TestResult, IsolatedTest, TestSummary};
pub use crash_analyzer::{CrashAnalyzer, CrashPattern, FixSuggestion, analyze_crashes};
pub use ci_reporter::{CiReporter, CrashReport, upload_crash_artifacts};
pub use sigsegv_monitor::{
    SigsegvMonitor, SigsegvEvent, PlatformInfo as MonitorPlatformInfo,
    start_global_monitoring, stop_global_monitoring, record_global_event,
    generate_global_report, write_global_report
};
pub use runtime_monitor::{initialize_runtime_monitoring, generate_monitoring_report, finalize_monitoring};
pub use platform_tests::{
    run_comprehensive_platform_tests, PlatformTestResults, TestStatus,
    generate_platform_test_report, write_platform_test_report
};

/// Initialize the complete debugging infrastructure
pub fn initialize_debug_infrastructure() -> Result<(), Box<dyn std::error::Error>> {
    // Install signal handlers for SIGSEGV, SIGABRT, etc.
    install_signal_handlers()?;
    
    // Initialize memory tracking
    MemoryTracker::initialize()?;
    
    // Detect platform information
    let platform = detect_platform();
    println!("Debug infrastructure initialized for: {}", platform);
    
    Ok(())
}

/// Clean up debugging infrastructure  
pub fn cleanup_debug_infrastructure() {
    MemoryTracker::cleanup();
    println!("Debug infrastructure cleaned up");
}