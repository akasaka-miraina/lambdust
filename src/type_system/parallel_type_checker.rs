//! Parallel Type Checking System
//! This module implements parallel type checking to challenge GHC's compilation speed
//! while maintaining the same level of type safety.

use super::polynomial_types::{PolynomialType, UniverseLevel};
use super::type_checker::TypeChecker;
use super::type_inference::TypeInference;
use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Type checking task for parallel processing
#[derive(Debug, Clone)]
pub struct TypeCheckTask {
    /// Unique identifier for the task
    pub id: TaskId,
    /// Expression to type check
    pub expr: Expr,
    /// Context dependencies (other tasks this depends on)
    pub dependencies: Vec<TaskId>,
    /// Priority level (higher = more urgent)
    pub priority: Priority,
    /// Module or scope identifier
    pub scope: String,
}

/// Task identifier
pub type TaskId = usize;

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Low priority - can wait
    Low = 1,
    /// Normal priority - standard processing
    Normal = 2,
    /// High priority - expedite processing
    High = 3,
    /// Critical priority - process immediately
    Critical = 4,
}

/// Result of a type checking task
#[derive(Debug, Clone)]
pub struct TypeCheckResult {
    /// Task identifier
    pub task_id: TaskId,
    /// Inferred type
    pub inferred_type: PolynomialType,
    /// Type checking duration
    pub duration: Duration,
    /// Any warnings generated
    pub warnings: Vec<String>,
}

/// Type checking error with context
#[derive(Debug, Clone)]
pub struct TypeCheckError {
    /// Task identifier
    pub task_id: TaskId,
    /// The actual error
    pub error: LambdustError,
    /// Context where error occurred
    pub context: String,
}

/// Dependency graph for task scheduling
#[derive(Debug)]
pub struct DependencyGraph {
    /// Tasks and their dependencies
    tasks: HashMap<TaskId, TypeCheckTask>,
    /// Adjacency list representation
    dependencies: HashMap<TaskId, Vec<TaskId>>,
    /// Reverse dependencies (who depends on this task)
    dependents: HashMap<TaskId, Vec<TaskId>>,
    /// Tasks ready to execute (no outstanding dependencies)
    ready_queue: VecDeque<TaskId>,
}

impl DependencyGraph {
    /// Create new dependency graph
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
            ready_queue: VecDeque::new(),
        }
    }

    /// Add a task to the graph
    pub fn add_task(&mut self, task: TypeCheckTask) {
        let task_id = task.id;
        
        // Add to dependencies
        self.dependencies.insert(task_id, task.dependencies.clone());
        
        // Update reverse dependencies
        for &dep_id in &task.dependencies {
            self.dependents.entry(dep_id)
                .or_insert_with(Vec::new)
                .push(task_id);
        }
        
        // If no dependencies, add to ready queue
        if task.dependencies.is_empty() {
            self.ready_queue.push_back(task_id);
        }
        
        self.tasks.insert(task_id, task);
    }

    /// Get next ready task (highest priority first)
    pub fn get_ready_task(&mut self) -> Option<TypeCheckTask> {
        // Sort ready queue by priority
        let mut ready_tasks: Vec<_> = self.ready_queue.drain(..).collect();
        ready_tasks.sort_by_key(|&task_id| {
            self.tasks.get(&task_id)
                .map(|task| std::cmp::Reverse(task.priority))
                .unwrap_or(std::cmp::Reverse(Priority::Low))
        });

        if let Some(task_id) = ready_tasks.first() {
            let task_id = *task_id;
            // Remove this task and re-add the rest
            ready_tasks.remove(0);
            self.ready_queue.extend(ready_tasks);
            
            self.tasks.remove(&task_id)
        } else {
            None
        }
    }

    /// Mark task as completed and update dependencies
    pub fn complete_task(&mut self, task_id: TaskId) {
        if let Some(dependents) = self.dependents.remove(&task_id) {
            for dependent_id in dependents {
                // Remove this dependency from the dependent task
                if let Some(deps) = self.dependencies.get_mut(&dependent_id) {
                    deps.retain(|&id| id != task_id);
                    
                    // If no more dependencies, add to ready queue
                    if deps.is_empty() {
                        self.ready_queue.push_back(dependent_id);
                    }
                }
            }
        }
    }

    /// Check for cycles in the dependency graph
    pub fn has_cycles(&self) -> bool {
        let mut visited = HashMap::new();
        let mut rec_stack = HashMap::new();

        for &task_id in self.tasks.keys() {
            if self.has_cycle_util(task_id, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        false
    }

    fn has_cycle_util(
        &self,
        task_id: TaskId,
        visited: &mut HashMap<TaskId, bool>,
        rec_stack: &mut HashMap<TaskId, bool>,
    ) -> bool {
        visited.insert(task_id, true);
        rec_stack.insert(task_id, true);

        if let Some(deps) = self.dependencies.get(&task_id) {
            for &dep_id in deps {
                if !visited.get(&dep_id).unwrap_or(&false) {
                    if self.has_cycle_util(dep_id, visited, rec_stack) {
                        return true;
                    }
                } else if *rec_stack.get(&dep_id).unwrap_or(&false) {
                    return true;
                }
            }
        }

        rec_stack.insert(task_id, false);
        false
    }
}

/// Worker thread for parallel type checking
pub struct TypeCheckWorker {
    /// Worker ID
    id: usize,
    /// Type checker instance
    type_checker: TypeChecker,
    /// Type inference engine
    type_inference: TypeInference,
    /// Shared type environment
    type_env: Arc<RwLock<HashMap<String, PolynomialType>>>,
}

impl TypeCheckWorker {
    /// Create new worker
    pub fn new(
        id: usize,
        type_env: Arc<RwLock<HashMap<String, PolynomialType>>>,
    ) -> Self {
        Self {
            id,
            type_checker: TypeChecker::new(),
            type_inference: TypeInference::new(),
            type_env,
        }
    }

    /// Process a type checking task
    pub fn process_task(&mut self, task: TypeCheckTask) -> std::result::Result<TypeCheckResult, TypeCheckError> {
        let start_time = Instant::now();

        let result = self.check_expression(&task.expr, &task.scope);

        match result {
            Ok(inferred_type) => {
                let duration = start_time.elapsed();
                Ok(TypeCheckResult {
                    task_id: task.id,
                    inferred_type,
                    duration,
                    warnings: vec![], // TODO: Collect warnings during type checking
                })
            }
            Err(error) => {
                Err(TypeCheckError {
                    task_id: task.id,
                    error,
                    context: task.scope,
                })
            }
        }
    }

    /// Check a single expression
    fn check_expression(&mut self, expr: &Expr, _scope: &str) -> Result<PolynomialType> {
        // Convert AST expression to Value for type inference
        // This is a simplified conversion - in a full implementation,
        // we'd have a more sophisticated AST -> Value conversion
        match expr {
            Expr::Literal(literal) => {
                let value = match literal {
                    crate::ast::Literal::Boolean(b) => crate::value::Value::Boolean(*b),
                    crate::ast::Literal::Number(n) => crate::value::Value::Number(n.clone()),
                    crate::ast::Literal::String(s) => crate::value::Value::new_string(s.clone()),
                    crate::ast::Literal::Character(c) => crate::value::Value::new_character(*c),
                    crate::ast::Literal::Nil => crate::value::Value::Nil,
                };
                self.type_inference.infer(&value)
            }
            Expr::Variable(name) => {
                // Look up in shared type environment
                let type_env = self.type_env.read().unwrap();
                if let Some(typ) = type_env.get(name) {
                    Ok(typ.clone())
                } else {
                    // Generate fresh type variable
                    Ok(PolynomialType::Variable {
                        name: name.clone(),
                        level: UniverseLevel::new(0),
                    })
                }
            }
            Expr::List(elements) => {
                if elements.is_empty() {
                    return Ok(PolynomialType::List {
                        element_type: Box::new(PolynomialType::Variable {
                            name: "α".to_string(),
                            level: UniverseLevel::new(0),
                        }),
                    });
                }

                // For function applications, check operator and operands
                let _operator_type = self.check_expression(&elements[0], _scope)?;
                
                // For now, return a generic type
                // TODO: Implement proper function application type checking
                Ok(PolynomialType::Variable {
                    name: format!("τ{}", self.id),
                    level: UniverseLevel::new(0),
                })
            }
            _ => {
                // For other expression types, return a fresh type variable
                Ok(PolynomialType::Variable {
                    name: format!("τ{}", self.id),
                    level: UniverseLevel::new(0),
                })
            }
        }
    }
    
    /// Check type equivalence using the type checker
    pub fn check_type_equivalence(&self, type1: &PolynomialType, type2: &PolynomialType) -> Result<bool> {
        self.type_checker.equivalent(type1, type2)
    }
    
    /// Get worker ID
    pub fn worker_id(&self) -> usize {
        self.id
    }
}

/// Parallel type checking coordinator
#[derive(Debug)]
pub struct ParallelTypeChecker {
    /// Number of worker threads
    worker_count: usize,
    /// Shared type environment
    type_env: Arc<RwLock<HashMap<String, PolynomialType>>>,
    /// Task ID counter
    next_task_id: Arc<Mutex<TaskId>>,
    /// Performance metrics
    metrics: Arc<Mutex<ParallelTypeCheckMetrics>>,
}

/// Performance metrics for parallel type checking
#[derive(Debug, Clone)]
pub struct ParallelTypeCheckMetrics {
    /// Total tasks processed
    pub total_tasks: usize,
    /// Total processing time
    pub total_time: Duration,
    /// Average time per task
    pub avg_time_per_task: Duration,
    /// Number of workers used
    pub workers_used: usize,
    /// Speedup factor compared to sequential processing
    pub speedup_factor: f64,
    /// Efficiency (speedup / workers_used)
    pub efficiency: f64,
}

impl ParallelTypeChecker {
    /// Create new parallel type checker
    pub fn new(worker_count: usize) -> Self {
        Self {
            worker_count,
            type_env: Arc::new(RwLock::new(HashMap::new())),
            next_task_id: Arc::new(Mutex::new(0)),
            metrics: Arc::new(Mutex::new(ParallelTypeCheckMetrics {
                total_tasks: 0,
                total_time: Duration::ZERO,
                avg_time_per_task: Duration::ZERO,
                workers_used: worker_count,
                speedup_factor: 1.0,
                efficiency: 1.0 / worker_count as f64,
            })),
        }
    }

    /// Generate next task ID
    fn next_task_id(&self) -> TaskId {
        let mut counter = self.next_task_id.lock().unwrap();
        let id = *counter;
        *counter += 1;
        id
    }

    /// Type check multiple expressions in parallel
    pub fn type_check_parallel(
        &self,
        expressions: Vec<(Expr, String)>, // (expression, scope)
    ) -> Result<Vec<TypeCheckResult>> {
        let start_time = Instant::now();

        // Create tasks
        let mut tasks = Vec::new();
        for (expr, scope) in expressions {
            let task = TypeCheckTask {
                id: self.next_task_id(),
                expr,
                dependencies: vec![], // TODO: Analyze dependencies
                priority: Priority::Normal,
                scope,
            };
            tasks.push(task);
        }

        // Check for empty input
        if tasks.is_empty() {
            return Ok(vec![]);
        }

        // Build dependency graph
        let mut graph = DependencyGraph::new();
        for task in tasks {
            graph.add_task(task);
        }

        // Check for cycles
        if graph.has_cycles() {
            return Err(LambdustError::type_error(
                "Circular dependencies detected in type checking".to_string()
            ));
        }

        // Create channels for communication
        let (task_sender, task_receiver) = mpsc::channel::<TypeCheckTask>();
        let (result_sender, result_receiver) = mpsc::channel::<std::result::Result<TypeCheckResult, TypeCheckError>>();

        // Spawn worker threads
        let mut workers = Vec::new();
        let task_receiver = Arc::new(Mutex::new(task_receiver));
        
        for worker_id in 0..self.worker_count {
            let task_rx = Arc::clone(&task_receiver);
            let result_tx = result_sender.clone();
            let type_env = Arc::clone(&self.type_env);

            let worker = thread::spawn(move || {
                let mut worker = TypeCheckWorker::new(worker_id, type_env);
                
                loop {
                    let task = {
                        let rx = task_rx.lock().unwrap();
                        rx.recv()
                    };
                    
                    match task {
                        Ok(task) => {
                            let result = worker.process_task(task);
                            if result_tx.send(result).is_err() {
                                break; // Main thread dropped receiver
                            }
                        }
                        Err(_) => break, // Channel closed
                    }
                }
            });
            workers.push(worker);
        }

        // Coordinate task execution
        let graph = Arc::new(Mutex::new(graph));
        let task_sender = Arc::new(Mutex::new(task_sender));
        
        // Task dispatcher thread
        let dispatcher_graph = Arc::clone(&graph);
        let dispatcher_sender = Arc::clone(&task_sender);
        let dispatcher = thread::spawn(move || {
            loop {
                let ready_task = {
                    let mut g = dispatcher_graph.lock().unwrap();
                    g.get_ready_task()
                };

                if let Some(task) = ready_task {
                    let sender = dispatcher_sender.lock().unwrap();
                    if sender.send(task).is_err() {
                        break; // Workers are done
                    }
                } else {
                    // No more ready tasks, check if we're done
                    let g = dispatcher_graph.lock().unwrap();
                    if g.tasks.is_empty() && g.ready_queue.is_empty() {
                        break; // All tasks completed
                    }
                    // Wait a bit before checking again
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });

        // Collect results
        let mut results = Vec::new();
        let mut completed_tasks = 0;
        let total_tasks = {
            let g = graph.lock().unwrap();
            g.tasks.len() + g.ready_queue.len()
        };

        while completed_tasks < total_tasks {
            match result_receiver.recv() {
                Ok(Ok(result)) => {
                    let task_id = result.task_id;
                    results.push(result);
                    
                    // Mark task as completed in graph
                    {
                        let mut g = graph.lock().unwrap();
                        g.complete_task(task_id);
                    }
                    
                    completed_tasks += 1;
                }
                Ok(Err(error)) => {
                    return Err(error.error);
                }
                Err(_) => {
                    return Err(LambdustError::type_error(
                        "Worker communication failed".to_string()
                    ));
                }
            }
        }

        // Clean up
        drop(task_sender);
        dispatcher.join().unwrap();
        
        for worker in workers {
            worker.join().unwrap();
        }

        // Update metrics
        let total_time = start_time.elapsed();
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_tasks = total_tasks;
        metrics.total_time = total_time;
        metrics.avg_time_per_task = total_time / total_tasks as u32;

        // Calculate speedup (this would require sequential baseline)
        // For now, estimate based on parallel efficiency
        let sequential_time_estimate = results.iter()
            .map(|r| r.duration)
            .sum::<Duration>();
        
        if sequential_time_estimate > Duration::ZERO {
            metrics.speedup_factor = sequential_time_estimate.as_secs_f64() / total_time.as_secs_f64();
        }
        metrics.efficiency = metrics.speedup_factor / self.worker_count as f64;

        // Sort results by task ID to maintain order
        results.sort_by_key(|r| r.task_id);

        Ok(results)
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> ParallelTypeCheckMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Add type binding to shared environment
    pub fn add_type_binding(&self, name: String, typ: PolynomialType) {
        let mut env = self.type_env.write().unwrap();
        env.insert(name, typ);
    }

    /// Get type from shared environment
    pub fn get_type(&self, name: &str) -> Option<PolynomialType> {
        let env = self.type_env.read().unwrap();
        env.get(name).cloned()
    }

    /// Type check with automatic parallelization decision
    pub fn type_check_auto(
        &self,
        expressions: Vec<(Expr, String)>,
    ) -> Result<Vec<TypeCheckResult>> {
        // Decision heuristic: use parallel processing for larger workloads
        if expressions.len() >= 4 && self.worker_count > 1 {
            self.type_check_parallel(expressions)
        } else {
            // Fall back to sequential for small workloads
            self.type_check_sequential(expressions)
        }
    }

    /// Sequential type checking for comparison
    pub fn type_check_sequential(
        &self,
        expressions: Vec<(Expr, String)>,
    ) -> Result<Vec<TypeCheckResult>> {
        let mut worker = TypeCheckWorker::new(0, Arc::clone(&self.type_env));
        let mut results = Vec::new();

        for (i, (expr, scope)) in expressions.into_iter().enumerate() {
            let task = TypeCheckTask {
                id: i,
                expr,
                dependencies: vec![],
                priority: Priority::Normal,
                scope,
            };

            match worker.process_task(task) {
                Ok(result) => results.push(result),
                Err(error) => return Err(error.error),
            }
        }

        Ok(results)
    }
}

impl Default for ParallelTypeChecker {
    fn default() -> Self {
        let worker_count = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(8); // Cap at 8 workers for now
        
        Self::new(worker_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();

        // Task 1: no dependencies
        let task1 = TypeCheckTask {
            id: 1,
            expr: Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            dependencies: vec![],
            priority: Priority::Normal,
            scope: "test".to_string(),
        };
        graph.add_task(task1);

        // Task 2: depends on task 1
        let task2 = TypeCheckTask {
            id: 2,
            expr: Expr::Variable("x".to_string()),
            dependencies: vec![1],
            priority: Priority::High,
            scope: "test".to_string(),
        };
        graph.add_task(task2);

        // Should get task 1 first (no dependencies)
        let ready_task = graph.get_ready_task();
        assert!(ready_task.is_some());
        assert_eq!(ready_task.unwrap().id, 1);

        // Complete task 1
        graph.complete_task(1);

        // Now task 2 should be ready
        let ready_task = graph.get_ready_task();
        assert!(ready_task.is_some());
        assert_eq!(ready_task.unwrap().id, 2);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();

        // Create circular dependency: 1 -> 2 -> 1
        let task1 = TypeCheckTask {
            id: 1,
            expr: Expr::Variable("x".to_string()),
            dependencies: vec![2],
            priority: Priority::Normal,
            scope: "test".to_string(),
        };
        let task2 = TypeCheckTask {
            id: 2,
            expr: Expr::Variable("y".to_string()),
            dependencies: vec![1],
            priority: Priority::Normal,
            scope: "test".to_string(),
        };

        graph.add_task(task1);
        graph.add_task(task2);

        assert!(graph.has_cycles());
    }

    #[test]
    fn test_parallel_type_checker_creation() {
        let checker = ParallelTypeChecker::new(4);
        assert_eq!(checker.worker_count, 4);
    }

    #[test]
    fn test_simple_parallel_type_checking() {
        let checker = ParallelTypeChecker::new(2);

        let expressions = vec![
            (Expr::Literal(Literal::Number(SchemeNumber::Integer(42))), "test".to_string()),
            (Expr::Literal(Literal::Boolean(true)), "test".to_string()),
            (Expr::Variable("x".to_string()), "test".to_string()),
        ];

        let results = checker.type_check_parallel(expressions);
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.len(), 3);

        // Check that results are in order
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.task_id, i);
        }
    }

    #[test]
    fn test_auto_parallelization_decision() {
        let checker = ParallelTypeChecker::new(4);

        // Small workload - should use sequential
        let small_expressions = vec![
            (Expr::Literal(Literal::Number(SchemeNumber::Integer(1))), "test".to_string()),
            (Expr::Literal(Literal::Number(SchemeNumber::Integer(2))), "test".to_string()),
        ];

        let results = checker.type_check_auto(small_expressions);
        assert!(results.is_ok());

        // Large workload - should use parallel
        let large_expressions = vec![
            (Expr::Literal(Literal::Number(SchemeNumber::Integer(1))), "test".to_string()),
            (Expr::Literal(Literal::Number(SchemeNumber::Integer(2))), "test".to_string()),
            (Expr::Literal(Literal::Number(SchemeNumber::Integer(3))), "test".to_string()),
            (Expr::Literal(Literal::Number(SchemeNumber::Integer(4))), "test".to_string()),
            (Expr::Literal(Literal::Number(SchemeNumber::Integer(5))), "test".to_string()),
        ];

        let results = checker.type_check_auto(large_expressions);
        assert!(results.is_ok());
    }

    #[test]
    fn test_type_environment_sharing() {
        let checker = ParallelTypeChecker::new(2);

        // Add a type binding
        checker.add_type_binding(
            "test_var".to_string(),
            PolynomialType::Base(crate::type_system::polynomial_types::BaseType::Integer)
        );

        // Check it can be retrieved
        let retrieved = checker.get_type("test_var");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_performance_metrics() {
        let checker = ParallelTypeChecker::new(2);

        let expressions = vec![
            (Expr::Literal(Literal::Number(SchemeNumber::Integer(42))), "test".to_string()),
        ];

        let _results = checker.type_check_parallel(expressions);

        let metrics = checker.get_metrics();
        assert!(metrics.total_tasks > 0);
        assert!(metrics.total_time > Duration::ZERO);
    }

    #[test]
    fn test_priority_ordering() {
        let mut graph = DependencyGraph::new();

        // Add tasks with different priorities
        let low_task = TypeCheckTask {
            id: 1,
            expr: Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            dependencies: vec![],
            priority: Priority::Low,
            scope: "test".to_string(),
        };

        let high_task = TypeCheckTask {
            id: 2,
            expr: Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            dependencies: vec![],
            priority: Priority::High,
            scope: "test".to_string(),
        };

        graph.add_task(low_task);
        graph.add_task(high_task);

        // Should get high priority task first
        let ready_task = graph.get_ready_task();
        assert!(ready_task.is_some());
        assert_eq!(ready_task.unwrap().id, 2); // High priority task
    }
}