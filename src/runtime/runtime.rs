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
        
        // Run bootstrap process and get the bootstrapped global environment
        let global_env_manager = bootstrap.bootstrap()?;
        
        // Create evaluator with default global environment
        let mut evaluator = Evaluator::new();
        
        // Copy all bindings from bootstrap environment to evaluator's environment
        Self::merge_bootstrap_environment(&mut evaluator, &global_env_manager)?;
        
        // Force populate essential primitives one more time after evaluator creation
        // to ensure they are not overridden during the evaluator initialization
        use crate::eval::environment::global_environment;
        let final_env = global_environment();
        Self::populate_essential_primitives(&final_env)?;
        
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
    pub fn expand_macros(&mut self, program: Program) -> Result<Program> {
        // Use the evaluator's macro expander to expand macros in the program
        self.evaluator.macro_expander_mut().expand_program(&program)
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

    /// Merges the bootstrap environment into the evaluator's global environment.
    fn merge_bootstrap_environment(
        evaluator: &mut Evaluator, 
        global_env_manager: &super::GlobalEnvironmentManager
    ) -> Result<()> {
        use crate::eval::environment::global_environment;
        
        // Get the evaluator's global environment
        let evaluator_env = global_environment();
        
        // Get the bootstrap environment
        let bootstrap_env = global_env_manager.root_environment();
        
        // Force populate essential primitives after stdlib has been loaded
        // This ensures our correct implementations override any problematic stdlib versions
        Self::populate_essential_primitives(&evaluator_env)?;
        
        Ok(())
    }
    
    /// Populates essential primitives that might be missing from the default environment.
    fn populate_essential_primitives(env: &std::rc::Rc<crate::eval::Environment>) -> Result<()> {
        use crate::eval::Value;
        use crate::eval::value::{PrimitiveProcedure, PrimitiveImpl};
        use crate::effects::Effect;
        use std::sync::Arc;
        
        // Force define cons primitive (override any existing definition)
        env.define("cons".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "cons".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(Self::primitive_cons),
            effects: vec![Effect::Pure],
        })));
        
        // Force define car primitive (override any existing definition)
        env.define("car".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "car".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(Self::primitive_car),
            effects: vec![Effect::Pure],
        })));
        
        // Force define cdr primitive (override any existing definition)
        env.define("cdr".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "cdr".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(Self::primitive_cdr),
            effects: vec![Effect::Pure],
        })));
        
        println!("DEBUG: Force defined car, cdr, cons primitives in runtime");
        
        Ok(())
    }
    
    /// cons primitive implementation
    fn primitive_cons(args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            return Err(Box::new(crate::diagnostics::Error::runtime_error(
                format!("cons expects 2 arguments, got {}", args.len()),
                None,
            )));
        }
        Ok(Value::pair(args[0].clone(), args[1].clone()))
    }
    
    /// car primitive implementation
    fn primitive_car(args: &[Value]) -> Result<Value> {
        println!("DEBUG: primitive_car called with {} args", args.len());
        if !args.is_empty() {
            println!("DEBUG: first arg is: {:?}", args[0]);
        }
        
        if args.len() != 1 {
            return Err(Box::new(crate::diagnostics::Error::runtime_error(
                format!("car expects 1 argument, got {}", args.len()),
                None,
            )));
        }
        
        match &args[0] {
            Value::Pair(car, _) => {
                let result = (**car).clone();
                println!("DEBUG: primitive_car returning: {result:?}");
                Ok(result)
            }
            _ => {
                println!("DEBUG: primitive_car error - not a pair: {:?}", args[0]);
                Err(Box::new(crate::diagnostics::Error::runtime_error(
                    "car requires a pair".to_string(),
                    None,
                )))
            }
        }
    }
    
    /// cdr primitive implementation
    fn primitive_cdr(args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(Box::new(crate::diagnostics::Error::runtime_error(
                format!("cdr expects 1 argument, got {}", args.len()),
                None,
            )));
        }
        
        match &args[0] {
            Value::Pair(_, cdr) => Ok((**cdr).clone()),
            _ => Err(Box::new(crate::diagnostics::Error::runtime_error(
                "cdr requires a pair".to_string(),
                None,
            ))),
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}