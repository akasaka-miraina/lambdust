use crate::ast::Program;
use crate::diagnostics::Result;
use crate::eval::{Evaluator, Value};
use crate::module_system::{ModuleSystem, ImportSpec};
use super::{BootstrapIntegration, BootstrapIntegrationConfig, BootstrapMode};
use std::collections::HashMap;

/// Legacy single-threaded runtime for the Lambdust language.
#[derive(Debug)]
pub struct Runtime {
    evaluator: Evaluator,
    module_system: ModuleSystem,
}

impl Runtime {
    /// Creates a new single-threaded runtime using the bootstrap system.
    pub fn new() -> Self {
        Self::with_bootstrap_config(BootstrapIntegrationConfig::default())
            .expect("Failed to create runtime with default bootstrap")
    }

    /// Creates a runtime with custom bootstrap configuration.
    pub fn with_bootstrap_config(config: BootstrapIntegrationConfig) -> Result<Self> {
        // Create bootstrap integration
        let mut bootstrap = BootstrapIntegration::with_config(config)?;
        
        // Run bootstrap process (this will populate the thread-local global environment)
        let _global_env_manager = bootstrap.bootstrap()?;
        
        // Create evaluator (it will use the populated global environment)
        let evaluator = Evaluator::new();
        
        // Create module system
        let module_system = ModuleSystem::new().map_err(|e| {
            crate::diagnostics::Error::runtime_error(
                format!("Failed to create module system: {e}"),
                None,
            )
        })?;
        
        Ok(Self {
            evaluator,
            module_system,
        })
    }

    /// Creates a runtime in fallback mode (legacy Rust stdlib).
    pub fn with_fallback() -> Result<Self> {
        let config = BootstrapIntegrationConfig {
            mode: BootstrapMode::Fallback,
            verbose: false,
            ..Default::default()
        };
        Self::with_bootstrap_config(config)
    }

    /// Evaluates a program using single-threaded evaluation.
    pub fn eval(&mut self, program: Program) -> Result<Value> {
        self.evaluator.eval_program(&program)
    }

    /// Expands macros in a program.
    pub fn expand_macros(&self, program: Program) -> Result<Program> {
        // Placeholder - return unchanged for now
        Ok(program)
    }

    /// Type checks a program.
    pub fn type_check(&self, program: Program) -> Result<Program> {
        // Placeholder - return unchanged for now
        Ok(program)
    }

    /// Imports a module into the runtime.
    pub fn import_module(&mut self, import_spec: ImportSpec) -> Result<HashMap<String, Value>> {
        self.module_system.resolve_import(&import_spec)
    }
    
    /// Gets a reference to the evaluator.
    pub fn evaluator(&self) -> &Evaluator {
        &self.evaluator
    }
    
    /// Gets a mutable reference to the evaluator.
    pub fn evaluator_mut(&mut self) -> &mut Evaluator {
        &mut self.evaluator
    }
    
    /// Gets a reference to the module system.
    pub fn module_system(&self) -> &ModuleSystem {
        &self.module_system
    }
    
    /// Gets a mutable reference to the module system.
    pub fn module_system_mut(&mut self) -> &mut ModuleSystem {
        &mut self.module_system
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}