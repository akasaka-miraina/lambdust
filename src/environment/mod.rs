//! Environment and variable binding management
//!
//! This module provides both traditional and copy-on-write (COW) environment
//! implementations for efficient memory usage and variable scoping.

pub mod cow;
pub mod traditional;

// Re-export the traditional Environment for backward compatibility
pub use traditional::Environment;

// Export COW environment types for Phase 4 optimization
pub use cow::{EnvironmentStrategy, SharedEnvironment};

use crate::error::Result;
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// Factory for creating environments with different strategies
pub struct EnvironmentFactory;

impl EnvironmentFactory {
    /// Create a new environment using the traditional strategy
    pub fn new_traditional() -> Environment {
        Environment::new()
    }

    /// Create a new environment using the COW strategy
    pub fn new_shared() -> SharedEnvironment {
        SharedEnvironment::new()
    }

    /// Create environment with parent using traditional strategy
    pub fn with_parent_traditional(parent: Rc<Environment>) -> Environment {
        Environment::with_parent(parent)
    }

    /// Create environment with parent using COW strategy
    pub fn with_parent_shared(parent: Rc<SharedEnvironment>) -> SharedEnvironment {
        SharedEnvironment::with_parent(parent)
    }

    /// Create environment with initial bindings using traditional strategy
    pub fn with_bindings_traditional(bindings: HashMap<String, Value>) -> Environment {
        Environment::with_bindings(bindings)
    }

    /// Create environment with initial bindings using COW strategy
    pub fn with_bindings_shared(bindings: HashMap<String, Value>) -> SharedEnvironment {
        SharedEnvironment::with_bindings(bindings)
    }
}

/// Trait for unified environment operations
/// This allows code to work with both traditional and COW environments
pub trait EnvironmentOps {
    /// Define a variable in the environment
    fn define(&mut self, name: String, value: Value);

    /// Set a variable (must already exist)
    fn set(&mut self, name: &str, value: Value) -> Result<()>;

    /// Get a variable value
    fn get(&self, name: &str) -> Option<Value>;

    /// Check if variable exists
    fn exists(&self, name: &str) -> bool;

    /// Get environment depth
    fn depth(&self) -> usize;
}

impl EnvironmentOps for Environment {
    fn define(&mut self, name: String, value: Value) {
        Environment::define(self, name, value);
    }

    fn set(&mut self, name: &str, value: Value) -> Result<()> {
        Environment::set(self, name, value)
    }

    fn get(&self, name: &str) -> Option<Value> {
        Environment::get(self, name)
    }

    fn exists(&self, name: &str) -> bool {
        Environment::exists(self, name)
    }

    fn depth(&self) -> usize {
        Environment::depth(self)
    }
}

impl EnvironmentOps for SharedEnvironment {
    fn define(&mut self, name: String, value: Value) {
        SharedEnvironment::define(self, name, value);
    }

    fn set(&mut self, name: &str, value: Value) -> Result<()> {
        SharedEnvironment::set(self, name, value)
    }

    fn get(&self, name: &str) -> Option<Value> {
        SharedEnvironment::get(self, name)
    }

    fn exists(&self, name: &str) -> bool {
        SharedEnvironment::exists(self, name)
    }

    fn depth(&self) -> usize {
        SharedEnvironment::depth(self)
    }
}

/// Environment performance benchmark utilities
#[cfg(test)]
pub mod benchmarks {
    use super::*;
    use std::time::Instant;

    /// Benchmark environment creation
    pub fn benchmark_environment_creation(iterations: usize) -> (u64, u64) {
        // Traditional environment benchmark
        let start = Instant::now();
        for _ in 0..iterations {
            let _env = EnvironmentFactory::new_traditional();
        }
        let traditional_time = start.elapsed().as_nanos() as u64;

        // COW environment benchmark
        let start = Instant::now();
        for _ in 0..iterations {
            let _env = EnvironmentFactory::new_shared();
        }
        let cow_time = start.elapsed().as_nanos() as u64;

        (traditional_time, cow_time)
    }

    /// Benchmark environment extension
    pub fn benchmark_environment_extension(iterations: usize) -> (u64, u64) {
        use crate::lexer::SchemeNumber;
        let bindings = vec![
            ("x".to_string(), Value::Number(SchemeNumber::Integer(1))),
            ("y".to_string(), Value::Number(SchemeNumber::Integer(2))),
            ("z".to_string(), Value::Number(SchemeNumber::Integer(3))),
        ];

        // Traditional environment benchmark
        let start = Instant::now();
        for _ in 0..iterations {
            let env = EnvironmentFactory::new_traditional();
            for (name, value) in &bindings {
                env.define(name.clone(), value.clone());
            }
        }
        let traditional_time = start.elapsed().as_nanos() as u64;

        // COW environment benchmark
        let start = Instant::now();
        for _ in 0..iterations {
            let mut env = EnvironmentFactory::new_shared();
            for (name, value) in &bindings {
                env.define(name.clone(), value.clone());
            }
        }
        let cow_time = start.elapsed().as_nanos() as u64;

        (traditional_time, cow_time)
    }

    /// Benchmark variable lookup
    pub fn benchmark_variable_lookup(iterations: usize) -> (u64, u64) {
        // Setup environments with some bindings
        let traditional_env = EnvironmentFactory::new_traditional();
        let mut cow_env = EnvironmentFactory::new_shared();

        use crate::lexer::SchemeNumber;
        for i in 0..10 {
            let name = format!("var{}", i);
            let value = Value::Number(SchemeNumber::Integer(i as i64));
            traditional_env.define(name.clone(), value.clone());
            cow_env.define(name, value);
        }

        // Traditional environment benchmark
        let start = Instant::now();
        for _ in 0..iterations {
            for i in 0..10 {
                let name = format!("var{}", i);
                let _ = traditional_env.get(&name);
            }
        }
        let traditional_time = start.elapsed().as_nanos() as u64;

        // COW environment benchmark
        let start = Instant::now();
        for _ in 0..iterations {
            for i in 0..10 {
                let name = format!("var{}", i);
                let _ = cow_env.get(&name);
            }
        }
        let cow_time = start.elapsed().as_nanos() as u64;

        (traditional_time, cow_time)
    }
}
