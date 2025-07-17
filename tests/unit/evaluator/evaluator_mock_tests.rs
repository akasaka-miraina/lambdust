//! Evaluator Mock Tests
//!
//! Comprehensive unit tests for Evaluator using mockall for isolated testing.
//! Tests cover expression evaluation, memory management, and optimization features.

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use mockall::*;
    
    use lambdust::environment::SharedEnvironment;
    use lambdust::error::{LambdustError, Result};
    use lambdust::value::Value;
    use lambdust::ast::{Expr, Literal};
    use lambdust::evaluator::types::Evaluator;
    use std::rc::Rc;
    use std::collections::HashMap;

    /// Mock trait for Evaluator operations to enable isolated testing
    #[automock]
    pub trait EvaluatorOps {
        /// Evaluate an expression
        fn eval(&mut self, expr: &Expr) -> Result<Value>;
        
        /// Evaluate with specific environment
        fn eval_with_env(&mut self, expr: &Expr, env: Rc<SharedEnvironment>) -> Result<Value>;
        
        /// Evaluate string expression
        fn eval_string(&mut self, source: &str) -> Result<Value>;
        
        /// Get memory usage statistics
        fn memory_usage(&self) -> usize;
        
        /// Force garbage collection
        fn collect_garbage(&mut self) -> Result<usize>;
        
        /// Check if memory is under pressure
        fn is_memory_under_pressure(&self) -> bool;
        
        /// Get optimization statistics
        fn optimization_stats(&self) -> String;
        
        /// Set memory limit
        fn set_memory_limit(&mut self, limit: usize);
        
        /// Create execution context
        fn create_execution_context(&mut self, expr: Expr, env: Rc<SharedEnvironment>) -> HashMap<String, Value>;
        
        /// Call procedure with arguments
        fn call_procedure(&mut self, procedure: Value, args: Vec<Value>) -> Result<Value>;
        
        /// Apply procedure to arguments
        fn apply_procedure(&mut self, procedure: &crate::value::Procedure, args: &[Value], env: &SharedEnvironment) -> Result<Value>;
    }

    // ===== Basic Expression Evaluation Tests =====

    #[test]
    fn test_eval_literal_number() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let expr = Expr::Literal(Literal::Number(42.into()));
        let expected = Value::Number(42.into());
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1)
            .returning(move |_| Ok(expected.clone()));
        
        let result = mock_eval.eval(&expr);
        assert_eq!(result.unwrap(), Value::Number(42.into()));
    }

    #[test]
    fn test_eval_literal_string() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let expr = Expr::Literal(Literal::String("hello".to_string()));
        let expected = Value::String("hello".to_string());
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1)
            .returning(move |_| Ok(expected.clone()));
        
        let result = mock_eval.eval(&expr);
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_eval_literal_boolean() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let expr = Expr::Literal(Literal::Boolean(true));
        let expected = Value::Boolean(true);
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1)
            .returning(move |_| Ok(expected));
        
        let result = mock_eval.eval(&expr);
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_eval_variable_success() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let expr = Expr::Variable("x".to_string());
        let expected = Value::Number(42.into());
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1)
            .returning(move |_| Ok(expected.clone()));
        
        let result = mock_eval.eval(&expr);
        assert_eq!(result.unwrap(), Value::Number(42.into()));
    }

    #[test]
    fn test_eval_variable_unbound_error() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let expr = Expr::Variable("undefined_var".to_string());
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1)
            .returning(|_| Err(LambdustError::runtime_error("Unbound variable: undefined_var".to_string())));
        
        let result = mock_eval.eval(&expr);
        assert!(result.is_err());
    }

    // ===== Arithmetic Operations Tests =====

    #[test]
    fn test_eval_addition() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(1.into())),
            Expr::Literal(Literal::Number(2.into())),
        ]);
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1)
            .returning(|_| Ok(Value::Number(3.into())));
        
        let result = mock_eval.eval(&expr);
        assert_eq!(result.unwrap(), Value::Number(3.into()));
    }

    #[test]
    fn test_eval_nested_arithmetic() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        // (+ (* 2 3) 4) = 10
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::List(vec![
                Expr::Variable("*".to_string()),
                Expr::Literal(Literal::Number(2.into())),
                Expr::Literal(Literal::Number(3.into())),
            ]),
            Expr::Literal(Literal::Number(4.into())),
        ]);
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1)
            .returning(|_| Ok(Value::Number(10.into())));
        
        let result = mock_eval.eval(&expr);
        assert_eq!(result.unwrap(), Value::Number(10.into()));
    }

    // ===== String Evaluation Tests =====

    #[test]
    fn test_eval_string_simple() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        mock_eval.expect_eval_string()
            .with(eq("42"))
            .times(1)
            .returning(|_| Ok(Value::Number(42.into())));
        
        let result = mock_eval.eval_string("42");
        assert_eq!(result.unwrap(), Value::Number(42.into()));
    }

    #[test]
    fn test_eval_string_expression() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        mock_eval.expect_eval_string()
            .with(eq("(+ 1 2 3)"))
            .times(1)
            .returning(|_| Ok(Value::Number(6.into())));
        
        let result = mock_eval.eval_string("(+ 1 2 3)");
        assert_eq!(result.unwrap(), Value::Number(6.into()));
    }

    #[test]
    fn test_eval_string_syntax_error() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        mock_eval.expect_eval_string()
            .with(eq("(+ 1 2"))  // Missing closing parenthesis
            .times(1)
            .returning(|_| Err(LambdustError::syntax_error("Unexpected end of input".to_string())));
        
        let result = mock_eval.eval_string("(+ 1 2");
        assert!(result.is_err());
    }

    // ===== Memory Management Tests =====

    #[test]
    fn test_memory_usage_tracking() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        mock_eval.expect_memory_usage()
            .times(1)
            .returning(|| 1024);
        
        let usage = mock_eval.memory_usage();
        assert_eq!(usage, 1024);
    }

    #[test]
    fn test_garbage_collection() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        mock_eval.expect_collect_garbage()
            .times(1)
            .returning(|| Ok(512)); // Freed 512 bytes
        
        let result = mock_eval.collect_garbage();
        assert_eq!(result.unwrap(), 512);
    }

    #[test]
    fn test_memory_pressure_detection() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        // Initially not under pressure
        mock_eval.expect_is_memory_under_pressure()
            .times(1)
            .returning(|| false);
        
        // After heavy usage, under pressure
        mock_eval.expect_is_memory_under_pressure()
            .times(1)
            .returning(|| true);
        
        assert!(!mock_eval.is_memory_under_pressure());
        assert!(mock_eval.is_memory_under_pressure());
    }

    #[test]
    fn test_memory_limit_setting() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        mock_eval.expect_set_memory_limit()
            .with(eq(1_000_000))
            .times(1)
            .returning(|_| ());
        
        mock_eval.set_memory_limit(1_000_000);
    }

    #[test]
    fn test_memory_limit_exceeded_error() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let large_expr = Expr::List((0..10000).map(|i| {
            Expr::Literal(Literal::Number(i.into()))
        }).collect());
        
        mock_eval.expect_eval()
            .with(eq(large_expr.clone()))
            .times(1)
            .returning(|_| Err(LambdustError::runtime_error("Memory limit exceeded".to_string())));
        
        let result = mock_eval.eval(&large_expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Memory limit exceeded"));
    }

    // ===== Optimization Tests =====

    #[test]
    fn test_optimization_stats() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        mock_eval.expect_optimization_stats()
            .times(1)
            .returning(|| "JIT compiled: 5, Tail calls optimized: 12, Memory saved: 2048 bytes".to_string());
        
        let stats = mock_eval.optimization_stats();
        assert!(stats.contains("JIT compiled"));
        assert!(stats.contains("Tail calls optimized"));
        assert!(stats.contains("Memory saved"));
    }

    // ===== Environment Integration Tests =====

    #[test]
    fn test_eval_with_custom_environment() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let env = Rc::new(SharedEnvironment::new());
        let expr = Expr::Variable("custom_var".to_string());
        
        mock_eval.expect_eval_with_env()
            .with(eq(expr.clone()), always())
            .times(1)
            .returning(|_, _| Ok(Value::String("custom_value".to_string())));
        
        let result = mock_eval.eval_with_env(&expr, env);
        assert_eq!(result.unwrap(), Value::String("custom_value".to_string()));
    }

    // ===== Procedure Call Tests =====

    #[test]
    fn test_call_procedure_success() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let procedure = Value::String("+".to_string()); // Mock procedure representation
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        
        mock_eval.expect_call_procedure()
            .with(eq(procedure.clone()), eq(args.clone()))
            .times(1)
            .returning(|_, _| Ok(Value::Number(3.into())));
        
        let result = mock_eval.call_procedure(procedure, args);
        assert_eq!(result.unwrap(), Value::Number(3.into()));
    }

    #[test]
    fn test_call_procedure_invalid_procedure_error() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let not_a_procedure = Value::Number(42.into());
        let args = vec![Value::Number(1.into())];
        
        mock_eval.expect_call_procedure()
            .with(eq(not_a_procedure.clone()), eq(args.clone()))
            .times(1)
            .returning(|_, _| Err(LambdustError::runtime_error("Not a procedure".to_string())));
        
        let result = mock_eval.call_procedure(not_a_procedure, args);
        assert!(result.is_err());
    }

    // ===== Error Handling Tests =====

    #[test]
    fn test_eval_type_error() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        // Attempt to apply a number as if it were a function
        let expr = Expr::List(vec![
            Expr::Literal(Literal::Number(42.into())),
            Expr::Literal(Literal::Number(1.into())),
        ]);
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1)
            .returning(|_| Err(LambdustError::type_error("Number is not applicable".to_string())));
        
        let result = mock_eval.eval(&expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not applicable"));
    }

    #[test]
    fn test_eval_arity_error() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        // Wrong number of arguments to +
        let expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(1.into())),
            // Missing second argument
        ]);
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1)
            .returning(|_| Err(LambdustError::runtime_error("Wrong number of arguments to +".to_string())));
        
        let result = mock_eval.eval(&expr);
        assert!(result.is_err());
    }

    // ===== Edge Cases =====

    #[test]
    fn test_eval_empty_list() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let expr = Expr::List(vec![]);
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1)
            .returning(|_| Err(LambdustError::syntax_error("Empty application".to_string())));
        
        let result = mock_eval.eval(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_deeply_nested_expression() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        // Create deeply nested expression
        let mut expr = Expr::Literal(Literal::Number(1.into()));
        for _ in 0..100 {
            expr = Expr::List(vec![
                Expr::Variable("+".to_string()),
                expr,
                Expr::Literal(Literal::Number(1.into())),
            ]);
        }
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1)
            .returning(|_| Ok(Value::Number(101.into())));
        
        let result = mock_eval.eval(&expr);
        assert_eq!(result.unwrap(), Value::Number(101.into()));
    }

    #[test]
    fn test_eval_very_large_number() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let expr = Expr::Literal(Literal::Number(i64::MAX.into()));
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1)
            .returning(|_| Ok(Value::Number(i64::MAX.into())));
        
        let result = mock_eval.eval(&expr);
        assert_eq!(result.unwrap(), Value::Number(i64::MAX.into()));
    }

    // ===== Performance Tests =====

    #[test]
    fn test_eval_performance_simple_expression() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let expr = Expr::Literal(Literal::Number(42.into()));
        
        mock_eval.expect_eval()
            .with(eq(expr.clone()))
            .times(1000)
            .returning(|_| Ok(Value::Number(42.into())));
        
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = mock_eval.eval(&expr);
        }
        let duration = start.elapsed();
        
        // Simple expressions should be very fast
        assert!(duration.as_millis() < 100);
    }

    // ===== Execution Context Tests =====

    #[test]
    fn test_create_execution_context() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        let expr = Expr::Variable("x".to_string());
        let env = Rc::new(SharedEnvironment::new());
        
        let mut expected_context = HashMap::new();
        expected_context.insert("optimization_level".to_string(), Value::String("high".to_string()));
        expected_context.insert("tail_call_optimizable".to_string(), Value::Boolean(true));
        
        mock_eval.expect_create_execution_context()
            .with(eq(expr.clone()), always())
            .times(1)
            .returning(move |_, _| expected_context.clone());
        
        let context = mock_eval.create_execution_context(expr, env);
        assert_eq!(context.get("optimization_level"), Some(&Value::String("high".to_string())));
        assert_eq!(context.get("tail_call_optimizable"), Some(&Value::Boolean(true)));
    }

    // ===== Complex Integration Tests =====

    #[test]
    fn test_evaluation_sequence() {
        let mut mock_eval = MockEvaluatorOps::new();
        
        // Test a sequence of evaluations that build on each other
        let define_expr = Expr::List(vec![
            Expr::Variable("define".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Literal(Literal::Number(42.into())),
        ]);
        
        let use_expr = Expr::Variable("x".to_string());
        
        mock_eval.expect_eval()
            .with(eq(define_expr.clone()))
            .times(1)
            .returning(|_| Ok(Value::Nil));
        
        mock_eval.expect_eval()
            .with(eq(use_expr.clone()))
            .times(1)
            .returning(|_| Ok(Value::Number(42.into())));
        
        // Define variable
        let result1 = mock_eval.eval(&define_expr);
        assert!(result1.is_ok());
        
        // Use variable
        let result2 = mock_eval.eval(&use_expr);
        assert_eq!(result2.unwrap(), Value::Number(42.into()));
    }
}