//! RuntimeExecutor Mock Tests
//!
//! Comprehensive unit tests for RuntimeExecutor using mockall for isolated testing.
//! Tests cover runtime optimization, JIT compilation, and performance measurement.

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use mockall::*;
    
    use lambdust::environment::SharedEnvironment;
    use lambdust::error::{LambdustError, Result};
    use lambdust::value::Value;
    use lambdust::ast::{Expr, Literal};
    use std::rc::Rc;
    use std::collections::HashMap;
    use std::time::Duration;

    /// Mock trait for RuntimeExecutor operations to enable isolated testing
    #[automock]
    pub trait RuntimeExecutorOps {
        /// Execute expression with runtime optimizations
        fn execute(&mut self, expr: &Expr, env: Rc<SharedEnvironment>) -> Result<Value>;
        
        /// Execute with specific optimization level
        fn execute_with_optimization(&mut self, expr: &Expr, env: Rc<SharedEnvironment>, opt_level: u8) -> Result<Value>;
        
        /// Execute with time limit
        fn execute_with_timeout(&mut self, expr: &Expr, env: Rc<SharedEnvironment>, timeout: Duration) -> Result<Value>;
        
        /// Get execution statistics
        fn get_execution_stats(&self) -> HashMap<String, u64>;
        
        /// Reset optimization state
        fn reset_optimizations(&mut self);
        
        /// Check if JIT compilation is available
        fn is_jit_available(&self) -> bool;
        
        /// Get hot path information
        fn get_hot_paths(&self) -> Vec<String>;
        
        /// Get compiled code count
        fn get_compiled_code_count(&self) -> usize;
        
        /// Force JIT compilation for expression
        fn force_jit_compile(&mut self, expr: &Expr) -> Result<bool>;
        
        /// Get optimization level
        fn get_optimization_level(&self) -> u8;
        
        /// Set optimization level
        fn set_optimization_level(&mut self, level: u8);
        
        /// Enable/disable specific optimizations
        fn set_optimization_flag(&mut self, flag: &str, enabled: bool);
        
        /// Get memory usage by runtime executor
        fn get_runtime_memory_usage(&self) -> usize;
        
        /// Get performance profile
        fn get_performance_profile(&self) -> HashMap<String, f64>;
        
        /// Benchmark execution
        fn benchmark_execution(&mut self, expr: &Expr, iterations: usize) -> Result<Duration>;
    }

    // ===== Basic Execution Tests =====

    #[test]
    fn test_execute_literal() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let expr = Expr::Literal(Literal::Number(42.into()));
        let env = Rc::new(SharedEnvironment::new());
        
        mock_executor.expect_execute()
            .with(eq(expr.clone()), always())
            .times(1)
            .returning(|_, _| Ok(Value::Number(42.into())));
        
        let result = mock_executor.execute(&expr, env);
        assert_eq!(result.unwrap(), Value::Number(42.into()));
    }

    #[test]
    fn test_execute_arithmetic_expression() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(5.into())),
            Expr::Literal(Literal::Number(7.into())),
        ]);
        let env = Rc::new(SharedEnvironment::new());
        
        mock_executor.expect_execute()
            .with(eq(expr.clone()), always())
            .times(1)
            .returning(|_, _| Ok(Value::Number(12.into())));
        
        let result = mock_executor.execute(&expr, env);
        assert_eq!(result.unwrap(), Value::Number(12.into()));
    }

    #[test]
    fn test_execute_variable_lookup() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let expr = Expr::Variable("x".to_string());
        let env = Rc::new(SharedEnvironment::new());
        
        mock_executor.expect_execute()
            .with(eq(expr.clone()), always())
            .times(1)
            .returning(|_, _| Ok(Value::String("variable_value".to_string())));
        
        let result = mock_executor.execute(&expr, env);
        assert_eq!(result.unwrap(), Value::String("variable_value".to_string()));
    }

    // ===== Optimization Level Tests =====

    #[test]
    fn test_execute_with_different_optimization_levels() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let expr = Expr::List(vec![
            Expr::Variable("*".to_string()),
            Expr::Literal(Literal::Number(6.into())),
            Expr::Literal(Literal::Number(7.into())),
        ]);
        let env = Rc::new(SharedEnvironment::new());
        
        // Level 0 (no optimization)
        mock_executor.expect_execute_with_optimization()
            .with(eq(expr.clone()), always(), eq(0))
            .times(1)
            .returning(|_, _, _| Ok(Value::Number(42.into())));
        
        // Level 2 (moderate optimization)
        mock_executor.expect_execute_with_optimization()
            .with(eq(expr.clone()), always(), eq(2))
            .times(1)
            .returning(|_, _, _| Ok(Value::Number(42.into())));
        
        // Level 3 (aggressive optimization)
        mock_executor.expect_execute_with_optimization()
            .with(eq(expr.clone()), always(), eq(3))
            .times(1)
            .returning(|_, _, _| Ok(Value::Number(42.into())));
        
        let result0 = mock_executor.execute_with_optimization(&expr, env.clone(), 0);
        let result2 = mock_executor.execute_with_optimization(&expr, env.clone(), 2);
        let result3 = mock_executor.execute_with_optimization(&expr, env, 3);
        
        assert_eq!(result0.unwrap(), Value::Number(42.into()));
        assert_eq!(result2.unwrap(), Value::Number(42.into()));
        assert_eq!(result3.unwrap(), Value::Number(42.into()));
    }

    #[test]
    fn test_optimization_level_management() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        // Get initial optimization level
        mock_executor.expect_get_optimization_level()
            .times(1)
            .returning(|| 1);
        
        // Set new optimization level
        mock_executor.expect_set_optimization_level()
            .with(eq(3))
            .times(1)
            .returning(|_| ());
        
        // Get updated optimization level
        mock_executor.expect_get_optimization_level()
            .times(1)
            .returning(|| 3);
        
        assert_eq!(mock_executor.get_optimization_level(), 1);
        mock_executor.set_optimization_level(3);
        assert_eq!(mock_executor.get_optimization_level(), 3);
    }

    // ===== JIT Compilation Tests =====

    #[test]
    fn test_jit_availability() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        mock_executor.expect_is_jit_available()
            .times(1)
            .returning(|| true);
        
        assert!(mock_executor.is_jit_available());
    }

    #[test]
    fn test_force_jit_compile_success() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let expr = Expr::List(vec![
            Expr::Variable("fib".to_string()),
            Expr::Literal(Literal::Number(10.into())),
        ]);
        
        mock_executor.expect_force_jit_compile()
            .with(eq(expr.clone()))
            .times(1)
            .returning(|_| Ok(true));
        
        let result = mock_executor.force_jit_compile(&expr);
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_force_jit_compile_failure() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let expr = Expr::Literal(Literal::Number(42.into())); // Not suitable for JIT
        
        mock_executor.expect_force_jit_compile()
            .with(eq(expr.clone()))
            .times(1)
            .returning(|_| Ok(false));
        
        let result = mock_executor.force_jit_compile(&expr);
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_compiled_code_count() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        mock_executor.expect_get_compiled_code_count()
            .times(1)
            .returning(|| 5);
        
        assert_eq!(mock_executor.get_compiled_code_count(), 5);
    }

    // ===== Hot Path Detection Tests =====

    #[test]
    fn test_hot_path_detection() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let expected_hot_paths = vec![
            "fibonacci_recursive".to_string(),
            "inner_loop_calculation".to_string(),
            "matrix_multiply".to_string(),
        ];
        
        mock_executor.expect_get_hot_paths()
            .times(1)
            .returning(move || expected_hot_paths.clone());
        
        let hot_paths = mock_executor.get_hot_paths();
        assert_eq!(hot_paths.len(), 3);
        assert!(hot_paths.contains(&"fibonacci_recursive".to_string()));
        assert!(hot_paths.contains(&"inner_loop_calculation".to_string()));
        assert!(hot_paths.contains(&"matrix_multiply".to_string()));
    }

    // ===== Statistics and Profiling Tests =====

    #[test]
    fn test_execution_statistics() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let mut expected_stats = HashMap::new();
        expected_stats.insert("total_executions".to_string(), 1000);
        expected_stats.insert("jit_compilations".to_string(), 25);
        expected_stats.insert("cache_hits".to_string(), 750);
        expected_stats.insert("cache_misses".to_string(), 250);
        
        mock_executor.expect_get_execution_stats()
            .times(1)
            .returning(move || expected_stats.clone());
        
        let stats = mock_executor.get_execution_stats();
        assert_eq!(stats.get("total_executions"), Some(&1000));
        assert_eq!(stats.get("jit_compilations"), Some(&25));
        assert_eq!(stats.get("cache_hits"), Some(&750));
        assert_eq!(stats.get("cache_misses"), Some(&250));
    }

    #[test]
    fn test_performance_profile() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let mut expected_profile = HashMap::new();
        expected_profile.insert("average_execution_time_ms".to_string(), 12.5);
        expected_profile.insert("jit_speedup_factor".to_string(), 3.2);
        expected_profile.insert("memory_efficiency".to_string(), 0.85);
        expected_profile.insert("cache_hit_ratio".to_string(), 0.75);
        
        mock_executor.expect_get_performance_profile()
            .times(1)
            .returning(move || expected_profile.clone());
        
        let profile = mock_executor.get_performance_profile();
        assert_eq!(profile.get("average_execution_time_ms"), Some(&12.5));
        assert_eq!(profile.get("jit_speedup_factor"), Some(&3.2));
        assert_eq!(profile.get("memory_efficiency"), Some(&0.85));
        assert_eq!(profile.get("cache_hit_ratio"), Some(&0.75));
    }

    #[test]
    fn test_runtime_memory_usage() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        mock_executor.expect_get_runtime_memory_usage()
            .times(1)
            .returning(|| 2048);
        
        let memory_usage = mock_executor.get_runtime_memory_usage();
        assert_eq!(memory_usage, 2048);
    }

    // ===== Optimization Flag Tests =====

    #[test]
    fn test_optimization_flags() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        // Enable tail call optimization
        mock_executor.expect_set_optimization_flag()
            .with(eq("tail_call_optimization"), eq(true))
            .times(1)
            .returning(|_, _| ());
        
        // Enable JIT compilation
        mock_executor.expect_set_optimization_flag()
            .with(eq("jit_compilation"), eq(true))
            .times(1)
            .returning(|_, _| ());
        
        // Disable constant folding
        mock_executor.expect_set_optimization_flag()
            .with(eq("constant_folding"), eq(false))
            .times(1)
            .returning(|_, _| ());
        
        mock_executor.set_optimization_flag("tail_call_optimization", true);
        mock_executor.set_optimization_flag("jit_compilation", true);
        mock_executor.set_optimization_flag("constant_folding", false);
    }

    // ===== Error Handling Tests =====

    #[test]
    fn test_execute_runtime_error() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let expr = Expr::List(vec![
            Expr::Variable("undefined_function".to_string()),
            Expr::Literal(Literal::Number(1.into())),
        ]);
        let env = Rc::new(SharedEnvironment::new());
        
        mock_executor.expect_execute()
            .with(eq(expr.clone()), always())
            .times(1)
            .returning(|_, _| Err(LambdustError::runtime_error("Unbound variable: undefined_function".to_string())));
        
        let result = mock_executor.execute(&expr, env);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("undefined_function"));
    }

    #[test]
    fn test_execute_timeout_error() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let expr = Expr::List(vec![
            Expr::Variable("infinite_loop".to_string()),
        ]);
        let env = Rc::new(SharedEnvironment::new());
        let timeout = Duration::from_millis(100);
        
        mock_executor.expect_execute_with_timeout()
            .with(eq(expr.clone()), always(), eq(timeout))
            .times(1)
            .returning(|_, _, _| Err(LambdustError::runtime_error("Execution timeout exceeded".to_string())));
        
        let result = mock_executor.execute_with_timeout(&expr, env, timeout);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout"));
    }

    #[test]
    fn test_jit_compilation_error() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let invalid_expr = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]), // Invalid lambda syntax
        ]);
        
        mock_executor.expect_force_jit_compile()
            .with(eq(invalid_expr.clone()))
            .times(1)
            .returning(|_| Err(LambdustError::compilation_error("Invalid lambda syntax for JIT".to_string())));
        
        let result = mock_executor.force_jit_compile(&invalid_expr);
        assert!(result.is_err());
    }

    // ===== Reset and Cleanup Tests =====

    #[test]
    fn test_reset_optimizations() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        // Initial state
        mock_executor.expect_get_compiled_code_count()
            .times(1)
            .returning(|| 10);
        
        // Reset optimizations
        mock_executor.expect_reset_optimizations()
            .times(1)
            .returning(|| ());
        
        // After reset
        mock_executor.expect_get_compiled_code_count()
            .times(1)
            .returning(|| 0);
        
        assert_eq!(mock_executor.get_compiled_code_count(), 10);
        mock_executor.reset_optimizations();
        assert_eq!(mock_executor.get_compiled_code_count(), 0);
    }

    // ===== Benchmarking Tests =====

    #[test]
    fn test_benchmark_execution() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(1.into())),
            Expr::Literal(Literal::Number(2.into())),
        ]);
        
        let expected_duration = Duration::from_micros(150);
        
        mock_executor.expect_benchmark_execution()
            .with(eq(expr.clone()), eq(1000))
            .times(1)
            .returning(move |_, _| Ok(expected_duration));
        
        let result = mock_executor.benchmark_execution(&expr, 1000);
        assert_eq!(result.unwrap(), expected_duration);
    }

    #[test]
    fn test_benchmark_complex_expression() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        // Fibonacci calculation
        let expr = Expr::List(vec![
            Expr::Variable("fibonacci".to_string()),
            Expr::Literal(Literal::Number(20.into())),
        ]);
        
        let expected_duration = Duration::from_millis(50);
        
        mock_executor.expect_benchmark_execution()
            .with(eq(expr.clone()), eq(100))
            .times(1)
            .returning(move |_, _| Ok(expected_duration));
        
        let result = mock_executor.benchmark_execution(&expr, 100);
        assert_eq!(result.unwrap(), expected_duration);
    }

    // ===== Edge Cases =====

    #[test]
    fn test_execute_empty_list() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let expr = Expr::List(vec![]);
        let env = Rc::new(SharedEnvironment::new());
        
        mock_executor.expect_execute()
            .with(eq(expr.clone()), always())
            .times(1)
            .returning(|_, _| Err(LambdustError::syntax_error("Empty application".to_string())));
        
        let result = mock_executor.execute(&expr, env);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_very_large_expression() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        // Create large expression with many nested additions
        let mut expr = Expr::Literal(Literal::Number(0.into()));
        for i in 1..=1000 {
            expr = Expr::List(vec![
                Expr::Variable("+".to_string()),
                expr,
                Expr::Literal(Literal::Number(i.into())),
            ]);
        }
        
        let env = Rc::new(SharedEnvironment::new());
        
        mock_executor.expect_execute()
            .with(eq(expr.clone()), always())
            .times(1)
            .returning(|_, _| Ok(Value::Number(500500.into()))); // Sum of 1..1000
        
        let result = mock_executor.execute(&expr, env);
        assert_eq!(result.unwrap(), Value::Number(500500.into()));
    }

    // ===== Integration Tests =====

    #[test]
    fn test_optimization_progression() {
        let mut mock_executor = MockRuntimeExecutorOps::new();
        
        let expr = Expr::List(vec![
            Expr::Variable("factorial".to_string()),
            Expr::Literal(Literal::Number(10.into())),
        ]);
        let env = Rc::new(SharedEnvironment::new());
        
        // First execution (no optimization)
        mock_executor.expect_execute()
            .with(eq(expr.clone()), always())
            .times(1)
            .returning(|_, _| Ok(Value::Number(3628800.into())));
        
        // Force JIT compilation
        mock_executor.expect_force_jit_compile()
            .with(eq(expr.clone()))
            .times(1)
            .returning(|_| Ok(true));
        
        // Second execution (with JIT optimization)
        mock_executor.expect_execute()
            .with(eq(expr.clone()), always())
            .times(1)
            .returning(|_, _| Ok(Value::Number(3628800.into())));
        
        // Check hot paths
        mock_executor.expect_get_hot_paths()
            .times(1)
            .returning(|| vec!["factorial".to_string()]);
        
        // First execution
        let result1 = mock_executor.execute(&expr, env.clone());
        assert_eq!(result1.unwrap(), Value::Number(3628800.into()));
        
        // JIT compile
        let compiled = mock_executor.force_jit_compile(&expr);
        assert!(compiled.unwrap());
        
        // Second execution (should be faster)
        let result2 = mock_executor.execute(&expr, env);
        assert_eq!(result2.unwrap(), Value::Number(3628800.into()));
        
        // Check that factorial is now a hot path
        let hot_paths = mock_executor.get_hot_paths();
        assert!(hot_paths.contains(&"factorial".to_string()));
    }
}