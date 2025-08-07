//! Concurrency and Parallelism Tests Module
//!
//! This module contains specialized tests for Lambdust's concurrent
//! and parallel execution capabilities.

pub mod actor_system_tests;
pub mod effect_coordination_tests;
pub mod generation_management_tests;
pub mod io_coordination_tests;
pub mod performance_scalability_tests;
pub mod fault_tolerance_tests;
pub mod cross_platform_tests;

// Re-export common testing utilities
pub use crate::multithreaded_integration_tests::{
    MultiThreadTestConfig, MultiThreadTestResult, MultiThreadTestFramework, PerformanceMetrics
};