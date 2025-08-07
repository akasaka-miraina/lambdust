//! Central metaprogramming system that coordinates all subsystems.

use std::rc::Rc;
use crate::eval::Environment;
use crate::diagnostics::Result;
use super::{
    ReflectionSystem, CodeGenerator, DynamicEvaluator, ProceduralMacro,
    StaticAnalyzer, EnvironmentManipulator, SecurityManager
};

/// Central metaprogramming system that coordinates all subsystems.
#[derive(Debug)]
pub struct MetaprogrammingSystem {
    /// Reflection system for runtime introspection
    reflection: ReflectionSystem,
    /// Code generation system
    code_generation: CodeGenerator,
    /// Dynamic evaluation engine
    dynamic_evaluation: DynamicEvaluator,
    /// Advanced macro system
    advanced_macros: ProceduralMacro,
    /// Program analysis tools
    program_analysis: StaticAnalyzer,
    /// Environment manipulation system
    environment_manipulation: EnvironmentManipulator,
    /// Security system
    security: SecurityManager,
}

impl MetaprogrammingSystem {
    /// Creates a new metaprogramming system.
    pub fn new() -> Self {
        let security = SecurityManager::new();
        
        Self {
            reflection: ReflectionSystem::new(),
            code_generation: CodeGenerator::new(),
            dynamic_evaluation: DynamicEvaluator::with_security(security.clone()),
            advanced_macros: ProceduralMacro::new(),
            program_analysis: StaticAnalyzer::new(),
            environment_manipulation: EnvironmentManipulator::new(),
            security,
        }
    }

    /// Creates a metaprogramming system with default security policies.
    pub fn with_security() -> Self {
        let mut system = Self::new();
        system.security.install_default_policies();
        system
    }

    /// Gets the reflection system.
    pub fn reflection(&self) -> &ReflectionSystem {
        &self.reflection
    }

    /// Gets the code generation system.
    pub fn code_generation(&self) -> &CodeGenerator {
        &self.code_generation
    }

    /// Gets the dynamic evaluation engine.
    pub fn dynamic_evaluation(&self) -> &DynamicEvaluator {
        &self.dynamic_evaluation
    }

    /// Gets the advanced macro system.
    pub fn advanced_macros(&self) -> &ProceduralMacro {
        &self.advanced_macros
    }

    /// Gets the program analysis tools.
    pub fn program_analysis(&self) -> &StaticAnalyzer {
        &self.program_analysis
    }

    /// Gets the environment manipulation system.
    pub fn environment_manipulation(&self) -> &EnvironmentManipulator {
        &self.environment_manipulation
    }

    /// Gets the security system.
    pub fn security(&self) -> &SecurityManager {
        &self.security
    }

    /// Installs metaprogramming primitives into an environment.
    pub fn install_primitives(&self, env: &Rc<Environment>) -> Result<()> {
        self.reflection.install_primitives(env)?;
        self.code_generation.install_primitives(env)?;
        self.dynamic_evaluation.install_primitives(env)?;
        self.advanced_macros.install_primitives(env)?;
        self.program_analysis.install_primitives(env)?;
        self.environment_manipulation.install_primitives(env)?;
        Ok(())
    }
}

impl Default for MetaprogrammingSystem {
    fn default() -> Self {
        Self::with_security()
    }
}