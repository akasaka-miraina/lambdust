//! Runtime system for the Lambdust language.
//!
//! This module provides both single-threaded and multithreaded runtime systems
//! for evaluating Lambdust programs. The main entry points are:
//!
//! - `Runtime`: Legacy single-threaded runtime
//! - `LambdustRuntime`: New multithreaded runtime with parallel evaluation

mod evaluator;
mod thread_pool;
mod global_env;
mod effect_coordinator;
mod io_coordinator;
mod error_propagation;
mod bootstrap;
mod bootstrap_integration;
mod library_path_resolver;
mod primitive_bridge; // Re-enabled for containers integration
// mod integration_example; // Temporarily disabled due to compilation issues

#[cfg(test)]
mod tests;

#[cfg(test)]
mod bootstrap_integration_tests;

// Individual structure modules
mod runtime;
mod lambdust_runtime;
mod evaluator_handle;
mod evaluator_message;
mod parallel_result;

pub use evaluator::MultithreadedEvaluator;
pub use thread_pool::ThreadPool;
pub use global_env::GlobalEnvironmentManager;
pub use effect_coordinator::EffectCoordinator;
pub use io_coordinator::IOCoordinator;
pub use error_propagation::ErrorPropagationCoordinator;
pub use bootstrap::{BootstrapSystem, BootstrapStatistics, MinimalPrimitivesRegistry};
pub use crate::module_system::BootstrapConfig;
pub use bootstrap_integration::{BootstrapIntegration, BootstrapIntegrationConfig, BootstrapMetrics, BootstrapMode};
pub use library_path_resolver::{LibraryPathResolver, LibraryPathConfig, LibraryValidationReport};
// Note: integration_example has compilation issues - temporarily disabled
pub use primitive_bridge::{MinimalPrimitiveRegistry, MinimalPrimitive, MinimalPrimitiveCategory};
// pub use integration_example::{IntegrationExample, IntegrationMetrics, run_integration_example};

// Re-export individual structures
pub use runtime::*;
pub use lambdust_runtime::*;
pub use evaluator_handle::*;
pub use evaluator_message::*;
pub use parallel_result::*;

