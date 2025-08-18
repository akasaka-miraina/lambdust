//! Contract combinators for building complex contracts.
//!
//! This module implements the core combinators that allow building
//! sophisticated contracts from simpler components. It includes:
//!
//! - Function contracts (-> and ->i)
//! - Logical combinators (and/c, or/c, not/c)
//! - Structural combinators (listof, vectorof, etc.)
//! - Comparison contracts (</c, >/c, etc.)
//! - Higher-order contract utilities

use crate::contracts::{
    ast::{ContractExpr, ComparisonOp, DependentBinding, FunctionCase},
    blame::{BlameInfo, BlameTarget, BlameBoundary, BoundaryType},
    predicates::{ContractPredicate, PredicateRegistry},
    ContractError, ContractResult,
};
use crate::eval::Value;
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Span, Spanned};
use std::collections::HashMap;
use std::sync::Arc;

/// Contract combinator evaluation context.
#[derive(Debug, Clone)]
pub struct CombinatorContext {
    /// Predicate registry for looking up predicates
    pub predicates: PredicateRegistry,
    /// Current blame information
    pub blame: BlameInfo,
    /// Maximum recursion depth for recursive contracts
    pub max_depth: usize,
    /// Current recursion depth
    pub current_depth: usize,
    /// Environment for evaluating expressions
    pub environment: HashMap<String, Value>,
}

impl CombinatorContext {
    /// Creates a new combinator context.
    pub fn new(predicates: PredicateRegistry, blame: BlameInfo) -> Self {
        Self {
            predicates,
            blame,
            max_depth: 100,
            current_depth: 0,
            environment: HashMap::new(),
        }
    }

    /// Creates a context with custom depth limit.
    pub fn with_depth_limit(
        predicates: PredicateRegistry,
        blame: BlameInfo,
        max_depth: usize,
    ) -> Self {
        Self {
            predicates,
            blame,
            max_depth,
            current_depth: 0,
            environment: HashMap::new(),
        }
    }

    /// Enters a recursive context (increments depth).
    pub fn enter_recursion(&mut self) -> ContractResult<()> {
        if self.current_depth >= self.max_depth {
            return Err(ContractError::RuntimeError {
                message: format!("Maximum recursion depth {} exceeded", self.max_depth),
                location: self.blame.boundary.location,
            }.into());
        }
        self.current_depth += 1;
        Ok(())
    }

    /// Exits a recursive context (decrements depth).
    pub fn exit_recursion(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
    }

    /// Binds a variable in the environment.
    pub fn bind_variable(&mut self, name: String, value: Value) {
        self.environment.insert(name, value);
    }

    /// Looks up a variable in the environment.
    pub fn lookup_variable(&self, name: &str) -> Option<&Value> {
        self.environment.get(name)
    }
}

/// Contract combinator evaluator.
pub struct ContractCombinators {
    /// Predicate registry
    predicates: PredicateRegistry,
}

impl std::fmt::Debug for ContractCombinators {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContractCombinators")
            .field("predicates", &"<predicate_registry>")
            .finish()
    }
}

impl ContractCombinators {
    /// Creates a new contract combinators evaluator.
    pub fn new(predicates: PredicateRegistry) -> Self {
        Self { predicates }
    }

    /// Evaluates a contract expression against a value.
    pub fn evaluate_contract(
        &self,
        contract: &ContractExpr,
        value: &Value,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        match contract {
            ContractExpr::Predicate { name, location } => {
                self.evaluate_predicate(name, value, *location, context)
            }
            ContractExpr::Flat { check_expr, location } => {
                self.evaluate_flat_contract(check_expr, value, *location, context)
            }
            ContractExpr::Any { .. } => Ok(true),
            ContractExpr::None { .. } => Ok(false),
            ContractExpr::Function { domain, codomain, location } => {
                self.evaluate_function_contract(domain, codomain, value, *location, context)
            }
            ContractExpr::DependentFunction { bindings, codomain, location } => {
                self.evaluate_dependent_function_contract(bindings, codomain, value, *location, context)
            }
            ContractExpr::CaseFunction { cases, location } => {
                self.evaluate_case_function_contract(cases, value, *location, context)
            }
            ContractExpr::And { contracts, location } => {
                self.evaluate_and_combinator(contracts, value, *location, context)
            }
            ContractExpr::Or { contracts, location } => {
                self.evaluate_or_combinator(contracts, value, *location, context)
            }
            ContractExpr::Not { contract, location } => {
                self.evaluate_not_combinator(contract, value, *location, context)
            }
            ContractExpr::OneOf { values, location } => {
                self.evaluate_one_of_combinator(values, value, *location, context)
            }
            ContractExpr::Between { min, max, location } => {
                self.evaluate_between_combinator(min, max, value, *location, context)
            }
            ContractExpr::Comparison { operator, value: comp_value, location } => {
                self.evaluate_comparison_combinator(operator, comp_value, value, *location, context)
            }
            ContractExpr::ListOf { element_contract, location } => {
                self.evaluate_listof_combinator(element_contract, value, *location, context)
            }
            ContractExpr::VectorOf { element_contract, location } => {
                self.evaluate_vectorof_combinator(element_contract, value, *location, context)
            }
            ContractExpr::Hash { key_contract, value_contract, location } => {
                self.evaluate_hash_combinator(key_contract, value_contract, value, *location, context)
            }
            ContractExpr::Tuple { element_contracts, location } => {
                self.evaluate_tuple_combinator(element_contracts, value, *location, context)
            }
            ContractExpr::List { element_contracts, location } => {
                self.evaluate_list_combinator(element_contracts, value, *location, context)
            }
            ContractExpr::Vector { element_contracts, location } => {
                self.evaluate_vector_combinator(element_contracts, value, *location, context)
            }
            ContractExpr::Recursive { name, contract, location } => {
                self.evaluate_recursive_contract(name, contract, value, *location, context)
            }
            ContractExpr::Reference { name, location } => {
                self.evaluate_contract_reference(name, value, *location, context)
            }
            ContractExpr::Parametric { name, parameters, location } => {
                self.evaluate_parametric_contract(name, parameters, value, *location, context)
            }
            ContractExpr::WithMessage { contract, message, location } => {
                self.evaluate_with_message_contract(contract, message, value, *location, context)
            }
            _ => {
                Err(ContractError::RuntimeError {
                    message: format!("Unsupported contract type: {contract:?}"),
                    location: contract.location(),
                }.into())
            }
        }
    }

    // ============= PREDICATE EVALUATION =============

    fn evaluate_predicate(
        &self,
        name: &str,
        value: &Value,
        location: Span,
        context: &CombinatorContext,
    ) -> ContractResult<bool> {
        let predicate = context.predicates.lookup(name)
            .ok_or_else(|| ContractError::RuntimeError {
                message: format!("Unknown predicate: {name}"),
                location,
            })?;

        Ok(predicate.test(value))
    }

    fn evaluate_flat_contract(
        &self,
        _check_expr: &Spanned<Expr>,
        _value: &Value,
        location: Span,
        _context: &CombinatorContext,
    ) -> ContractResult<bool> {
        // TODO: Evaluate the check expression with the value
        // This requires integration with the expression evaluator
        Err(ContractError::RuntimeError {
            message: "Flat contracts not yet implemented".to_string(),
            location,
        }.into())
    }

    // ============= FUNCTION CONTRACT EVALUATION =============

    fn evaluate_function_contract(
        &self,
        _domain: &[Spanned<ContractExpr>],
        _codomain: &Spanned<ContractExpr>,
        value: &Value,
        location: Span,
        _context: &CombinatorContext,
    ) -> ContractResult<bool> {
        // Check if value is a procedure
        match value {
            Value::Procedure(_) | Value::CaseLambda(_) | Value::Primitive(_) => {
                // Function contracts require wrapping the procedure with runtime checks
                // For now, just check that it's a procedure
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn evaluate_dependent_function_contract(
        &self,
        _bindings: &[DependentBinding],
        _codomain: &Spanned<ContractExpr>,
        value: &Value,
        location: Span,
        context: &CombinatorContext,
    ) -> ContractResult<bool> {
        // Similar to function contracts but with dependent types
        self.evaluate_function_contract(&[], &dummy_contract(), value, location, context)
    }

    fn evaluate_case_function_contract(
        &self,
        _cases: &[FunctionCase],
        value: &Value,
        location: Span,
        context: &CombinatorContext,
    ) -> ContractResult<bool> {
        // Case function contracts handle multiple arities
        self.evaluate_function_contract(&[], &dummy_contract(), value, location, context)
    }

    // ============= LOGICAL COMBINATORS =============

    fn evaluate_and_combinator(
        &self,
        contracts: &[Spanned<ContractExpr>],
        value: &Value,
        _location: Span,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        for contract in contracts {
            if !self.evaluate_contract(&contract.inner, value, context)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn evaluate_or_combinator(
        &self,
        contracts: &[Spanned<ContractExpr>],
        value: &Value,
        _location: Span,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        for contract in contracts {
            if self.evaluate_contract(&contract.inner, value, context)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn evaluate_not_combinator(
        &self,
        contract: &Spanned<ContractExpr>,
        value: &Value,
        _location: Span,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        Ok(!self.evaluate_contract(&contract.inner, value, context)?)
    }

    // ============= VALUE COMBINATORS =============

    fn evaluate_one_of_combinator(
        &self,
        values: &[Box<Spanned<Expr>>],
        value: &Value,
        _location: Span,
        _context: &CombinatorContext,
    ) -> ContractResult<bool> {
        // TODO: Evaluate expressions and compare with value
        // For now, simplified implementation
        for _allowed_value in values {
            // If we could evaluate the expression and it equals value, return true
        }
        Ok(false)
    }

    fn evaluate_between_combinator(
        &self,
        _min: &Spanned<Expr>,
        _max: &Spanned<Expr>,
        value: &Value,
        location: Span,
        _context: &CombinatorContext,
    ) -> ContractResult<bool> {
        // Check if value is between min and max
        match value {
            Value::Literal(Literal::Number(n)) => {
                // TODO: Evaluate min and max expressions
                // For now, just check if it's a number
                Ok(true)
            }
            Value::Literal(Literal::Integer(_)) => Ok(true),
            _ => Ok(false),
        }
    }

    fn evaluate_comparison_combinator(
        &self,
        operator: &ComparisonOp,
        comp_value: &Spanned<Expr>,
        value: &Value,
        location: Span,
        context: &CombinatorContext,
    ) -> ContractResult<bool> {
        // TODO: Evaluate comp_value expression and compare with value using operator
        match (operator, value) {
            (ComparisonOp::Equal, _) => {
                // TODO: Implement equality comparison
                Ok(false)
            }
            (ComparisonOp::LessThan, Value::Literal(Literal::Number(_))) => {
                // TODO: Implement numeric comparison
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    // ============= STRUCTURAL COMBINATORS =============

    fn evaluate_listof_combinator(
        &self,
        element_contract: &Spanned<ContractExpr>,
        value: &Value,
        _location: Span,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        match value {
            Value::Nil => Ok(true), // Empty list satisfies listof
            Value::Pair(car, cdr) => {
                // Check car against element contract
                if !self.evaluate_contract(&element_contract.inner, car, context)? {
                    return Ok(false);
                }
                // Recursively check cdr
                self.evaluate_listof_combinator(element_contract, cdr, _location, context)
            }
            _ => Ok(false), // Not a list
        }
    }

    fn evaluate_vectorof_combinator(
        &self,
        element_contract: &Spanned<ContractExpr>,
        value: &Value,
        location: Span,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        match value {
            Value::Vector(vec) => {
                if let Ok(vector) = vec.read() {
                    for element in vector.iter() {
                        if !self.evaluate_contract(&element_contract.inner, element, context)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Err(ContractError::RuntimeError {
                        message: "Failed to read vector".to_string(),
                        location,
                    }.into())
                }
            }
            _ => Ok(false), // Not a vector
        }
    }

    fn evaluate_hash_combinator(
        &self,
        key_contract: &Spanned<ContractExpr>,
        value_contract: &Spanned<ContractExpr>,
        value: &Value,
        location: Span,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        match value {
            Value::Hashtable(hash) => {
                if let Ok(hashtable) = hash.read() {
                    for (key, val) in hashtable.iter() {
                        if !self.evaluate_contract(&key_contract.inner, key, context)? {
                            return Ok(false);
                        }
                        if !self.evaluate_contract(&value_contract.inner, val, context)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Err(ContractError::RuntimeError {
                        message: "Failed to read hashtable".to_string(),
                        location,
                    }.into())
                }
            }
            _ => Ok(false), // Not a hashtable
        }
    }

    fn evaluate_tuple_combinator(
        &self,
        element_contracts: &[Spanned<ContractExpr>],
        value: &Value,
        _location: Span,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        // Check if value is a list with exactly the right number of elements
        let elements = self.extract_list_elements(value)?;
        if elements.len() != element_contracts.len() {
            return Ok(false);
        }

        for (element, contract) in elements.iter().zip(element_contracts.iter()) {
            if !self.evaluate_contract(&contract.inner, element, context)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn evaluate_list_combinator(
        &self,
        element_contracts: &[Spanned<ContractExpr>],
        value: &Value,
        location: Span,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        // Same as tuple combinator
        self.evaluate_tuple_combinator(element_contracts, value, location, context)
    }

    fn evaluate_vector_combinator(
        &self,
        element_contracts: &[Spanned<ContractExpr>],
        value: &Value,
        location: Span,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        match value {
            Value::Vector(vec) => {
                if let Ok(vector) = vec.read() {
                    if vector.len() != element_contracts.len() {
                        return Ok(false);
                    }

                    for (element, contract) in vector.iter().zip(element_contracts.iter()) {
                        if !self.evaluate_contract(&contract.inner, element, context)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Err(ContractError::RuntimeError {
                        message: "Failed to read vector".to_string(),
                        location,
                    }.into())
                }
            }
            _ => Ok(false), // Not a vector
        }
    }

    // ============= ADVANCED COMBINATORS =============

    fn evaluate_recursive_contract(
        &self,
        name: &str,
        contract: &Spanned<ContractExpr>,
        value: &Value,
        location: Span,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        context.enter_recursion()?;
        
        // Bind the recursive name to the contract
        // TODO: Implement proper recursive contract handling
        let result = self.evaluate_contract(&contract.inner, value, context);
        
        context.exit_recursion();
        result
    }

    fn evaluate_contract_reference(
        &self,
        name: &str,
        value: &Value,
        location: Span,
        context: &CombinatorContext,
    ) -> ContractResult<bool> {
        // Look up the referenced contract
        // TODO: Implement contract reference resolution
        Err(ContractError::RuntimeError {
            message: format!("Contract reference not implemented: {name}"),
            location,
        }.into())
    }

    fn evaluate_parametric_contract(
        &self,
        name: &str,
        parameters: &[Spanned<ContractExpr>],
        value: &Value,
        location: Span,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        // Look up the parametric contract and instantiate it with parameters
        // TODO: Implement parametric contract handling
        Err(ContractError::RuntimeError {
            message: format!("Parametric contract not implemented: {name}"),
            location,
        }.into())
    }

    fn evaluate_with_message_contract(
        &self,
        contract: &Spanned<ContractExpr>,
        message: &str,
        value: &Value,
        location: Span,
        context: &mut CombinatorContext,
    ) -> ContractResult<bool> {
        // Evaluate the underlying contract, but use custom message on failure
        let result = self.evaluate_contract(&contract.inner, value, context);
        match result {
            Ok(false) => {
                // Contract failed, but we want to use the custom message
                // For now, just return false - the custom message would be used
                // in the actual error reporting
                Ok(false)
            }
            other => other,
        }
    }

    // ============= UTILITY METHODS =============

    /// Extracts elements from a list value.
    fn extract_list_elements(&self, value: &Value) -> ContractResult<Vec<Value>> {
        let mut elements = Vec::new();
        let mut current = value;

        loop {
            match current {
                Value::Nil => break,
                Value::Pair(car, cdr) => {
                    elements.push((**car).clone());
                    current = cdr;
                }
                _ => {
                    return Err(ContractError::RuntimeError {
                        message: "Not a proper list".to_string(),
                        location: Span::new(0, 0), // TODO: Better error location
                    }.into());
                }
            }
        }

        Ok(elements)
    }
}

/// Creates a dummy contract for testing.
fn dummy_contract() -> Spanned<ContractExpr> {
    Spanned::new(
        ContractExpr::Any { location: Span::new(0, 0) },
        Span::new(0, 0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{
        predicates::PredicateRegistry,
        blame::{BlameInfo, BlameTarget, BlameBoundary, BoundaryType},
    };
    use crate::eval::Value;
    use crate::ast::Literal;
    use crate::diagnostics::Span;
    use std::sync::Arc;

    fn create_test_context() -> CombinatorContext {
        let predicates = PredicateRegistry::new();
        let blame = BlameInfo {
            positive: BlameTarget::System {
                component: "test".to_string(),
                description: "test context".to_string(),
            },
            negative: BlameTarget::System {
                component: "test".to_string(),
                description: "test context".to_string(),
            },
            boundary: BlameBoundary {
                boundary_type: BoundaryType::ExplicitContract,
                contract: "test".to_string(),
                location: Span::new(0, 0),
                context: HashMap::new(),
            },
            call_stack: Vec::new(),
            id: 1,
            parent: None,
        };
        
        CombinatorContext::new(predicates, blame)
    }

    #[test]
    fn test_predicate_evaluation() {
        let predicates = PredicateRegistry::new();
        let combinators = ContractCombinators::new(predicates);
        let mut context = create_test_context();

        let number_contract = ContractExpr::Predicate {
            name: "number?".to_string(),
            location: Span::new(0, 0),
        };

        let number_value = Value::Literal(Literal::Number(42.0));
        let string_value = Value::Literal(Literal::String("hello".to_string()));

        assert!(combinators.evaluate_contract(&number_contract, &number_value, &mut context).unwrap());
        assert!(!combinators.evaluate_contract(&number_contract, &string_value, &mut context).unwrap());
    }

    #[test]
    fn test_and_combinator() {
        let predicates = PredicateRegistry::new();
        let combinators = ContractCombinators::new(predicates);
        let mut context = create_test_context();

        let number_contract = Spanned::new(
            ContractExpr::Predicate {
                name: "number?".to_string(),
                location: Span::new(0, 0),
            },
            Span::new(0, 0),
        );

        let positive_contract = Spanned::new(
            ContractExpr::Predicate {
                name: "positive?".to_string(),
                location: Span::new(0, 0),
            },
            Span::new(0, 0),
        );

        let and_contract = ContractExpr::And {
            contracts: vec![number_contract, positive_contract],
            location: Span::new(0, 0),
        };

        let positive_number = Value::Literal(Literal::Number(42.0));
        let negative_number = Value::Literal(Literal::Number(-5.0));
        let string_value = Value::Literal(Literal::String("hello".to_string()));

        assert!(combinators.evaluate_contract(&and_contract, &positive_number, &mut context).unwrap());
        assert!(!combinators.evaluate_contract(&and_contract, &negative_number, &mut context).unwrap());
        assert!(!combinators.evaluate_contract(&and_contract, &string_value, &mut context).unwrap());
    }

    #[test]
    fn test_or_combinator() {
        let predicates = PredicateRegistry::new();
        let combinators = ContractCombinators::new(predicates);
        let mut context = create_test_context();

        let number_contract = Spanned::new(
            ContractExpr::Predicate {
                name: "number?".to_string(),
                location: Span::new(0, 0),
            },
            Span::new(0, 0),
        );

        let string_contract = Spanned::new(
            ContractExpr::Predicate {
                name: "string?".to_string(),
                location: Span::new(0, 0),
            },
            Span::new(0, 0),
        );

        let or_contract = ContractExpr::Or {
            contracts: vec![number_contract, string_contract],
            location: Span::new(0, 0),
        };

        let number_value = Value::Literal(Literal::Number(42.0));
        let string_value = Value::Literal(Literal::String("hello".to_string()));
        let boolean_value = Value::Literal(Literal::Boolean(true));

        assert!(combinators.evaluate_contract(&or_contract, &number_value, &mut context).unwrap());
        assert!(combinators.evaluate_contract(&or_contract, &string_value, &mut context).unwrap());
        assert!(!combinators.evaluate_contract(&or_contract, &boolean_value, &mut context).unwrap());
    }

    #[test]
    fn test_not_combinator() {
        let predicates = PredicateRegistry::new();
        let combinators = ContractCombinators::new(predicates);
        let mut context = create_test_context();

        let number_contract = Spanned::new(
            ContractExpr::Predicate {
                name: "number?".to_string(),
                location: Span::new(0, 0),
            },
            Span::new(0, 0),
        );

        let not_contract = ContractExpr::Not {
            contract: Box::new(number_contract),
            location: Span::new(0, 0),
        };

        let number_value = Value::Literal(Literal::Number(42.0));
        let string_value = Value::Literal(Literal::String("hello".to_string()));

        assert!(!combinators.evaluate_contract(&not_contract, &number_value, &mut context).unwrap());
        assert!(combinators.evaluate_contract(&not_contract, &string_value, &mut context).unwrap());
    }

    #[test]
    fn test_listof_combinator() {
        let predicates = PredicateRegistry::new();
        let combinators = ContractCombinators::new(predicates);
        let mut context = create_test_context();

        let number_contract = Spanned::new(
            ContractExpr::Predicate {
                name: "number?".to_string(),
                location: Span::new(0, 0),
            },
            Span::new(0, 0),
        );

        let listof_contract = ContractExpr::ListOf {
            element_contract: Box::new(number_contract),
            location: Span::new(0, 0),
        };

        // Create a list of numbers: (1 2 3)
        let num1 = Arc::new(Value::Literal(Literal::Number(1.0)));
        let num2 = Arc::new(Value::Literal(Literal::Number(2.0)));
        let num3 = Arc::new(Value::Literal(Literal::Number(3.0)));
        
        let list_123 = Value::Pair(
            num1,
            Arc::new(Value::Pair(
                num2,
                Arc::new(Value::Pair(num3, Arc::new(Value::Nil))),
            )),
        );

        // Create a list with mixed types: (1 "hello" 3)
        let mixed_list = Value::Pair(
            Arc::new(Value::Literal(Literal::Number(1.0))),
            Arc::new(Value::Pair(
                Arc::new(Value::Literal(Literal::String("hello".to_string()))),
                Arc::new(Value::Pair(
                    Arc::new(Value::Literal(Literal::Number(3.0))),
                    Arc::new(Value::Nil),
                )),
            )),
        );

        let empty_list = Value::Nil;

        assert!(combinators.evaluate_contract(&listof_contract, &list_123, &mut context).unwrap());
        assert!(!combinators.evaluate_contract(&listof_contract, &mixed_list, &mut context).unwrap());
        assert!(combinators.evaluate_contract(&listof_contract, &empty_list, &mut context).unwrap());
    }

    #[test]
    fn test_recursion_depth_limit() {
        let mut context = create_test_context();
        context.max_depth = 2;

        // Test entering recursion
        assert!(context.enter_recursion().is_ok());
        assert_eq!(context.current_depth, 1);

        assert!(context.enter_recursion().is_ok());
        assert_eq!(context.current_depth, 2);

        // Should fail on third level
        assert!(context.enter_recursion().is_err());

        // Test exiting recursion
        context.exit_recursion();
        assert_eq!(context.current_depth, 1);

        context.exit_recursion();
        assert_eq!(context.current_depth, 0);
    }
}