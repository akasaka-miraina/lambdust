//! Contract evaluation integration with the main evaluator.
//!
//! This module provides integration between the contract system and the
//! main expression evaluator, handling define/contract forms and contract
//! checking during evaluation.

use crate::ast::{Expr, Formals};
use crate::contracts::{
    ContractConfig, ContractError, ContractResult, ContractSystem,
    ast::ContractExpr,
    blame::{BlameBoundary, BlameInfo, BlameTarget, BoundaryType},
    compiler::CompilationContext,
};
use crate::diagnostics::{Span, Spanned};
use crate::eval::{Environment, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Contract evaluator that integrates with the main evaluation engine.
#[derive(Debug)]
pub struct ContractEvaluator {
    /// Contract system
    contract_system: ContractSystem,
    /// Active contract contexts
    active_contexts: Vec<BlameInfo>,
}

impl ContractEvaluator {
    /// Creates a new contract evaluator.
    pub fn new(config: ContractConfig) -> Self {
        Self {
            contract_system: ContractSystem::with_config(config),
            active_contexts: Vec::new(),
        }
    }

    /// Evaluates a define/contract expression.
    pub fn eval_define_contract(
        &mut self,
        name: &str,
        formals: &Option<Formals>,
        contract: &Spanned<ContractExpr>,
        body: &[Spanned<Expr>],
        env: Arc<Environment>,
    ) -> ContractResult<Value> {
        // Create blame information for this contract
        let blame = BlameInfo::function_definition(
            name.to_string(),
            None, // TODO: Get module name from environment
            contract.span,
            format!("{}", contract.inner),
        );

        // Compile the contract
        let compilation_context =
            CompilationContext::new(self.contract_system.predicates.clone(), blame.clone());

        let compiled_contract = self
            .contract_system
            .compile_contract(&contract.inner, &compilation_context)?;

        // Create the function value
        // TODO: This would need integration with the main evaluator
        // For now, just return a placeholder
        let function_value = Value::Unspecified; // Placeholder

        // Wrap the function with contract checking
        let wrapped_function =
            self.contract_system
                .wrap_with_contract(function_value, compiled_contract, blame)?;

        Ok(wrapped_function)
    }

    /// Checks a value against a contract during evaluation.
    pub fn check_value_contract(
        &mut self,
        value: &Value,
        contract: &ContractExpr,
        blame: &BlameInfo,
    ) -> ContractResult<()> {
        let compilation_context =
            CompilationContext::new(self.contract_system.predicates.clone(), blame.clone());

        let compiled_contract = self
            .contract_system
            .compile_contract(contract, &compilation_context)?;

        self.contract_system
            .check_contract(value, &compiled_contract, blame)
    }

    /// Enters a contract context.
    pub fn enter_contract_context(&mut self, blame: BlameInfo) {
        self.active_contexts.push(blame);
    }

    /// Exits the current contract context.
    pub fn exit_contract_context(&mut self) -> Option<BlameInfo> {
        self.active_contexts.pop()
    }

    /// Gets the current contract context.
    pub fn current_context(&self) -> Option<&BlameInfo> {
        self.active_contexts.last()
    }

    /// Gets performance statistics.
    pub fn performance_stats(&self) -> crate::contracts::enforcement::PerformanceStats {
        self.contract_system.enforcement.performance_stats()
    }

    /// Updates the contract system configuration.
    pub fn update_config(&mut self, config: ContractConfig) {
        self.contract_system.update_config(config);
    }

    /// Clears contract caches.
    pub fn clear_caches(&mut self) {
        self.contract_system.clear_cache();
    }
}

/// Extension trait for the main evaluator to support contracts.
pub trait EvaluatorContractExtensions {
    /// Evaluates a define/contract expression.
    fn eval_define_contract(
        &mut self,
        name: &str,
        formals: &Option<Formals>,
        contract: &Spanned<ContractExpr>,
        body: &[Spanned<Expr>],
        env: Arc<Environment>,
    ) -> ContractResult<Value>;

    /// Checks if a value satisfies a contract.
    fn check_contract(
        &mut self,
        value: &Value,
        contract: &ContractExpr,
        location: Span,
    ) -> ContractResult<()>;

    /// Wraps a value with contract checking.
    fn wrap_with_contract(
        &mut self,
        value: Value,
        contract: &ContractExpr,
        location: Span,
    ) -> ContractResult<Value>;
}

// Note: In a real implementation, this would be implemented for the main Evaluator type
// For now, it's just a trait definition showing the intended interface

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::ast::ContractExpr;
    use crate::diagnostics::Span;

    #[test]
    fn test_contract_evaluator_creation() {
        let config = ContractConfig::default();
        let evaluator = ContractEvaluator::new(config);

        assert!(evaluator.active_contexts.is_empty());
    }

    #[test]
    fn test_contract_context_management() {
        let config = ContractConfig::default();
        let mut evaluator = ContractEvaluator::new(config);

        assert!(evaluator.current_context().is_none());

        let blame = BlameInfo::function_definition(
            "test".to_string(),
            None,
            Span::new(0, 10),
            "number?".to_string(),
        );

        evaluator.enter_contract_context(blame.clone());
        assert!(evaluator.current_context().is_some());

        let popped = evaluator.exit_contract_context();
        assert!(popped.is_some());
        assert!(evaluator.current_context().is_none());
    }

    #[test]
    fn test_value_contract_checking() {
        let config = ContractConfig::default();
        let mut evaluator = ContractEvaluator::new(config);

        let contract = ContractExpr::Predicate {
            name: "number?".to_string(),
            location: Span::new(0, 10),
        };

        let blame = BlameInfo::function_definition(
            "test".to_string(),
            None,
            Span::new(0, 10),
            "number?".to_string(),
        );

        let number_value = Value::Literal(crate::ast::Literal::Number(42.0));
        let result = evaluator.check_value_contract(&number_value, &contract, &blame);
        assert!(result.is_ok());

        let string_value =
            Value::Literal(crate::ast::Literal::String(Box::new("hello".to_string())));
        let result = evaluator.check_value_contract(&string_value, &contract, &blame);
        assert!(result.is_err());
    }
}
