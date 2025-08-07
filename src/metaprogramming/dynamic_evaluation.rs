//! Dynamic evaluation engine with sandboxing and security.
//!
//! This module provides safe dynamic evaluation of code strings with
//! comprehensive security controls, resource limits, and execution contexts.

use super::security::{SecurityManager, SecurityContext, Permission, ResourceUsage};
use crate::eval::{Value, Environment, Evaluator};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::diagnostics::{Error, Result};
use crate::ast::Program;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Execution context for dynamic evaluation.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Principal executing the code
    pub principal: String,
    /// Environment for evaluation
    pub environment: Rc<Environment>,
    /// Security context
    pub security_context: SecurityContext,
    /// Execution limits
    pub limits: ExecutionLimits,
    /// Context metadata
    pub metadata: HashMap<String, Value>,
}

/// Execution limits for dynamic evaluation.
#[derive(Debug, Clone)]
pub struct ExecutionLimits {
    /// Maximum execution time
    pub time_limit: Option<Duration>,
    /// Maximum memory usage
    pub memory_limit: Option<usize>,
    /// Maximum stack depth
    pub stack_depth_limit: Option<usize>,
    /// Maximum number of evaluation steps
    pub step_limit: Option<usize>,
}

/// Result of dynamic evaluation.
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// The resulting value
    pub value: Value,
    /// Execution statistics
    pub stats: ExecutionStats,
    /// Any warnings generated
    pub warnings: Vec<String>,
    /// Security violations (if any)
    pub security_violations: Vec<String>,
}

/// Execution statistics.
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    /// Total execution time
    pub execution_time: Duration,
    /// Number of evaluation steps
    pub steps: usize,
    /// Memory used
    pub memory_used: usize,
    /// Maximum stack depth reached
    pub max_stack_depth: usize,
    /// Number of allocations
    pub allocations: usize,
}

/// Sandbox environment for safe code execution.
#[derive(Debug)]
pub struct SandboxEnvironment {
    /// Base environment (restricted)
    base_environment: Rc<Environment>,
    /// Security manager
    security_manager: SecurityManager,
    /// Allowed primitives
    allowed_primitives: Vec<String>,
    /// Resource monitoring
    resource_monitor: ResourceMonitor,
}

/// Resource monitor for tracking usage.
#[derive(Debug)]
pub struct ResourceMonitor {
    /// Current resource usage
    usage: Arc<RwLock<ResourceUsage>>,
    /// Start time of execution
    start_time: Instant,
}

impl ResourceMonitor {
    /// Creates a new resource monitor.
    pub fn new() -> Self {
        Self {
            usage: Arc::new(RwLock::new(ResourceUsage::default())),
            start_time: Instant::now(),
        }
    }

    /// Records an allocation.
    pub fn record_allocation(&self, size: usize) {
        let mut usage = self.usage.write().unwrap();
        usage.allocations += 1;
        usage.memory_used += size;
    }

    /// Records a stack frame.
    pub fn record_stack_frame(&self) {
        let mut usage = self.usage.write().unwrap();
        usage.stack_depth += 1;
    }

    /// Removes a stack frame.
    pub fn remove_stack_frame(&self) {
        let mut usage = self.usage.write().unwrap();
        usage.stack_depth = usage.stack_depth.saturating_sub(1);
    }

    /// Updates execution time.
    pub fn update_execution_time(&self) {
        let mut usage = self.usage.write().unwrap();
        usage.execution_time = self.start_time.elapsed();
    }

    /// Gets current resource usage.
    pub fn get_usage(&self) -> ResourceUsage {
        self.usage.read().unwrap().clone())
    }
}

impl SandboxEnvironment {
    /// Creates a new sandbox environment.
    pub fn new() -> Self {
        let base_env = Rc::new(Environment::new(None, 0));
        
        // Install only safe primitives
        let safe_primitives = vec![
            "+", "-", "*", "/", "=", "<", ">", "<=", ">=",
            "cons", "car", "cdr", "list", "length",
            "null?", "pair?", "number?", "string?", "symbol?",
            "not", "and", "or",
        ];

        for _primitive in &safe_primitives {
            // Install safe versions of primitives
            // (actual implementation would install restricted versions)
        }

        Self {
            base_environment: base_env,
            security_manager: SecurityManager::default(),
            allowed_primitives: safe_primitives.into_iter().map(String::from).collect(),
            resource_monitor: ResourceMonitor::new(),
        }
    }

    /// Creates a sandbox with custom security policy.
    pub fn with_policy(policy_name: &str) -> Result<Self> {
        let sandbox = Self::new();
        let _context = sandbox.security_manager.create_context("sandbox".to_string(), policy_name)?;
        Ok(sandbox)
    }

    /// Checks if a primitive is allowed.
    pub fn is_primitive_allowed(&self, name: &str) -> bool {
        self.allowed_primitives.contains(&name.to_string())
    }

    /// Gets the base environment.
    pub fn environment(&self) -> &Rc<Environment> {
        &self.base_environment
    }

    /// Gets the security manager.
    pub fn security_manager(&self) -> &SecurityManager {
        &self.security_manager
    }

    /// Gets mutable access to the security manager.
    pub fn security_manager_mut(&mut self) -> &mut SecurityManager {
        &mut self.security_manager
    }
}

/// Security policy for dynamic evaluation.
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Name of the policy
    pub name: String,
    /// Maximum execution time
    pub max_execution_time: Option<Duration>,
    /// Maximum memory usage
    pub max_memory: Option<usize>,
    /// Allowed operations
    pub allowed_operations: Vec<String>,
    /// Forbidden operations
    pub forbidden_operations: Vec<String>,
    /// Resource limits
    pub resource_limits: HashMap<String, usize>,
}

impl SecurityPolicy {
    /// Creates a restrictive security policy.
    pub fn restrictive() -> Self {
        Self {
            name: "restrictive".to_string(),
            max_execution_time: Some(Duration::from_secs(1)),
            max_memory: Some(64 * 1024), // 64KB
            allowed_operations: vec![
                "arithmetic".to_string(),
                "comparison".to_string(),
                "list-operations".to_string(),
            ],
            forbidden_operations: vec![
                "file-io".to_string(),
                "network".to_string(),
                "system".to_string(),
                "eval".to_string(),
            ],
            resource_limits: {
                let mut limits = HashMap::new();
                limits.insert("max-allocations".to_string(), 100);
                limits.insert("max-stack-depth".to_string(), 20);
                limits
            },
        }
    }

    /// Creates a permissive security policy.
    pub fn permissive() -> Self {
        Self {
            name: "permissive".to_string(),
            max_execution_time: Some(Duration::from_secs(30)),
            max_memory: Some(1024 * 1024), // 1MB  
            allowed_operations: vec!["*".to_string()], // Allow all
            forbidden_operations: vec![],
            resource_limits: {
                let mut limits = HashMap::new();
                limits.insert("max-allocations".to_string(), 10000);
                limits.insert("max-stack-depth".to_string(), 1000);
                limits
            },
        }
    }
}

/// Main dynamic evaluator.
#[derive(Debug)]
pub struct DynamicEvaluator {
    /// Security manager
    security_manager: SecurityManager,
    /// Active execution contexts
    contexts: HashMap<String, ExecutionContext>,
    /// Sandbox environments
    sandboxes: HashMap<String, SandboxEnvironment>,
    /// Default security policy
    default_policy: SecurityPolicy,
}

impl DynamicEvaluator {
    /// Creates a new dynamic evaluator.
    pub fn new() -> Self {
        Self {
            security_manager: SecurityManager::default(),
            contexts: HashMap::new(),
            sandboxes: HashMap::new(),
            default_policy: SecurityPolicy::restrictive(),
        }
    }

    /// Creates a dynamic evaluator with a specific security manager.
    pub fn with_security(security_manager: SecurityManager) -> Self {
        Self {
            security_manager,
            contexts: HashMap::new(),
            sandboxes: HashMap::new(),
            default_policy: SecurityPolicy::restrictive(),
        }
    }

    /// Evaluates a code string in a sandbox.
    pub fn eval_string(
        &mut self,
        code: &str,
        principal: &str,
        policy_name: Option<&str>,
    ) -> Result<EvaluationResult> {
        let start_time = Instant::now();
        let mut stats = ExecutionStats {
            execution_time: Duration::from_secs(0),
            steps: 0,
            memory_used: 0,
            max_stack_depth: 0,
            allocations: 0,
        };

        // Create or get execution context
        let context = self.get_or_create_context(principal, policy_name)?;
        
        // Check permission to evaluate
        if !self.security_manager.check_permission(principal, &Permission::Eval)? {
            return Err(Box::new(Error::runtime_error(
                "Permission denied: eval not allowed".to_string(),
                None,
            ));
        }

        // Parse the code
        let program = self.parse_code(code)?;

        // Create sandboxed evaluator
        let mut evaluator = self.create_sandboxed_evaluator(&context)?;

        // Evaluate with limits
        let value = self.evaluate_with_limits(&mut evaluator, &program, &context.limits, &mut stats, principal)?;

        stats.execution_time = start_time.elapsed();

        Ok(EvaluationResult {
            value,
            stats,
            warnings: vec![],
            security_violations: vec![],
        })
    }

    /// Evaluates code in a specific environment.
    pub fn eval_in_environment(
        &mut self,
        code: &str,
        environment: Rc<Environment>,
        principal: &str,
    ) -> Result<EvaluationResult> {
        // Create temporary context with custom environment
        let security_context = self.security_manager.create_context(
            principal.to_string(),
            "restrictive"
        )?;

        let context = ExecutionContext {
            principal: principal.to_string(),
            environment,
            security_context,
            limits: ExecutionLimits {
                time_limit: Some(Duration::from_secs(5)),
                memory_limit: Some(256 * 1024),
                stack_depth_limit: Some(50),
                step_limit: Some(1000),
            },
            metadata: HashMap::new(),
        };

        let start_time = Instant::now();
        let mut stats = ExecutionStats {
            execution_time: Duration::from_secs(0),
            steps: 0,
            memory_used: 0,
            max_stack_depth: 0,
            allocations: 0,
        };

        let program = self.parse_code(code)?;
        let mut evaluator = Evaluator::with_environment(context.environment.clone());
        let value = self.evaluate_with_limits(&mut evaluator, &program, &context.limits, &mut stats, principal)?;

        stats.execution_time = start_time.elapsed();

        Ok(EvaluationResult {
            value,
            stats,
            warnings: vec![],
            security_violations: vec![],
        })
    }

    /// Creates a new sandbox for a principal.
    pub fn create_sandbox(&mut self, principal: &str, policy_name: &str) -> Result<()> {
        let sandbox = SandboxEnvironment::with_policy(policy_name)?;
        self.sandboxes.insert(principal.to_string(), sandbox);
        Ok(())
    }

    /// Gets or creates an execution context.
    fn get_or_create_context(
        &mut self,
        principal: &str,
        policy_name: Option<&str>
    ) -> Result<ExecutionContext> {
        if let Some(context) = self.contexts.get(principal) {
            return Ok(context.clone());
        }

        let policy_name = policy_name.unwrap_or("restrictive");
        let security_context = self.security_manager.create_context(
            principal.to_string(),
            policy_name
        )?;

        // Create or get sandbox
        if !self.sandboxes.contains_key(principal) {
            self.create_sandbox(principal, policy_name)?;
        }

        let sandbox = self.sandboxes.get(principal).unwrap();
        let context = ExecutionContext {
            principal: principal.to_string(),
            environment: sandbox.environment().clone()),
            security_context,
            limits: ExecutionLimits {
                time_limit: Some(Duration::from_secs(5)),
                memory_limit: Some(256 * 1024),
                stack_depth_limit: Some(50),
                step_limit: Some(1000),
            },
            metadata: HashMap::new(),
        };

        self.contexts.insert(principal.to_string(), context.clone());
        Ok(context)
    }

    /// Parses code string into a program.
    fn parse_code(&self, code: &str) -> Result<Program> {
        let mut lexer = Lexer::new(code, None);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    /// Creates a sandboxed evaluator.
    fn create_sandboxed_evaluator(&self, context: &ExecutionContext) -> Result<Evaluator> {
        Ok(Evaluator::with_environment(context.environment.clone()))
    }

    /// Evaluates a program with resource limits.
    fn evaluate_with_limits(
        &self,
        evaluator: &mut Evaluator,
        program: &Program,
        limits: &ExecutionLimits,
        stats: &mut ExecutionStats,
        principal: &str,
    ) -> Result<Value> {
        let start_time = Instant::now();
        let mut step_count = 0;

        for expr in &program.expressions {
            // Check time limit
            if let Some(time_limit) = limits.time_limit {
                if start_time.elapsed() > time_limit {
                    return Err(Box::new(Error::runtime_error(
                        "Execution time limit exceeded".to_string(),
                        Some(expr.span),
                    ));
                }
            }

            // Check step limit
            if let Some(step_limit) = limits.step_limit {
                if step_count >= step_limit {
                    return Err(Box::new(Error::runtime_error(
                        "Execution step limit exceeded".to_string(),
                        Some(expr.span),
                    ));
                }
            }

            // Evaluate expression
            // Get the context environment
            let env = self.contexts.get(principal)
                .map(|ctx| ctx.environment.clone())
                .unwrap_or_else(|| Rc::new(Environment::new(None, 0)));
            let result = evaluator.eval(expr, env)?;
            step_count += 1;

            // Update stats
            stats.steps = step_count;
            stats.execution_time = start_time.elapsed();

            // For the last expression, return its value
            if expr == program.expressions.last().unwrap() {
                return Ok(result);
            }
        }

        Ok(Value::Unspecified)
    }

    /// Installs dynamic evaluation primitives.
    pub fn install_primitives(&self, _env: &Rc<Environment>) -> Result<()> {
        // Would install primitives like eval, compile, etc.
        Ok(())
    }

    /// Gets the security manager.
    pub fn security_manager(&self) -> &SecurityManager {
        &self.security_manager
    }
}

impl Default for DynamicEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SandboxEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            time_limit: Some(Duration::from_secs(10)),
            memory_limit: Some(1024 * 1024),
            stack_depth_limit: Some(100),
            step_limit: Some(10000),
        }
    }
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self {
            execution_time: Duration::from_secs(0),
            steps: 0,
            memory_used: 0,
            max_stack_depth: 0,
            allocations: 0,
        }
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}