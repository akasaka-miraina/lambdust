//! Abstract Syntax Tree nodes for contracts.
//!
//! This module defines the AST representation for contract expressions,
//! including predicates, combinators, and function contracts.

use crate::ast::{Expr, Formals};
use crate::diagnostics::{Span, Spanned};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Contract expression AST node.
///
/// Represents all types of contracts that can be written in the contract language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContractExpr {
    // ============= BASIC CONTRACTS =============
    
    /// Simple predicate contract (e.g., number?, string?)
    Predicate {
        /// Name of the predicate function
        name: String,
        /// Source location of this contract
        location: Span,
    },
    
    /// Flat contract with custom check function
    Flat {
        /// Expression that performs the contract check
        check_expr: Box<Spanned<Expr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Any contract (matches any value)
    Any {
        /// Source location of this contract
        location: Span,
    },
    
    /// None contract (matches no value)
    None {
        /// Source location of this contract
        location: Span,
    },
    
    // ============= CONTRACT COMBINATORS =============
    
    /// Function contract: (-> domain ... codomain)
    Function {
        /// Contracts for function arguments (domain)
        domain: Vec<Spanned<ContractExpr>>,
        /// Contract for function return value (codomain)
        codomain: Box<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Dependent function contract: (->i ([x pred] ...) codomain)
    DependentFunction {
        /// Dependent parameter bindings
        bindings: Vec<DependentBinding>,
        /// Contract for function return value
        codomain: Box<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Case function contract: (->* (domain ... codomain) ...)
    CaseFunction {
        /// Function cases with different arities
        cases: Vec<FunctionCase>,
        /// Source location of this contract
        location: Span,
    },
    
    /// And combinator: (and/c contract ...)
    And {
        /// Contracts that must all be satisfied
        contracts: Vec<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Or combinator: (or/c contract ...)
    Or {
        /// Contracts where at least one must be satisfied
        contracts: Vec<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Not combinator: (not/c contract)
    Not {
        /// Contract to negate
        contract: Box<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// One-of combinator: (one-of/c value ...)
    OneOf {
        /// Values that are acceptable
        values: Vec<Box<Spanned<Expr>>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Between combinator: (between/c min max)
    Between {
        /// Minimum acceptable value
        min: Box<Spanned<Expr>>,
        /// Maximum acceptable value
        max: Box<Spanned<Expr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Comparison contract: (</c expr), (>/c expr), etc.
    Comparison {
        /// Comparison operator to use
        operator: ComparisonOp,
        /// Value to compare against
        value: Box<Spanned<Expr>>,
        /// Source location of this contract
        location: Span,
    },
    
    // ============= STRUCTURAL CONTRACTS =============
    
    /// List contract: (listof contract)
    ListOf {
        /// Contract for list elements
        element_contract: Box<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Vector contract: (vectorof contract)
    VectorOf {
        /// Contract for vector elements
        element_contract: Box<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Hash contract: (hash/c key-contract value-contract)
    Hash {
        /// Contract for hash table keys
        key_contract: Box<Spanned<ContractExpr>>,
        /// Contract for hash table values
        value_contract: Box<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Tuple contract: (tuple/c contract ...)
    Tuple {
        /// Contracts for each tuple element
        element_contracts: Vec<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// List contract with specific length: (list/c contract ...)
    List {
        /// Contracts for each list element
        element_contracts: Vec<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Vector contract with specific length: (vector/c contract ...)
    Vector {
        /// Contracts for each vector element
        element_contracts: Vec<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    // ============= OBJECT CONTRACTS =============
    
    /// Object contract with method contracts
    Object {
        /// Method contracts for the object
        methods: HashMap<String, Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Class contract for object-oriented features
    Class {
        /// Optional superclass name
        superclass: Option<String>,
        /// Method contracts for the class
        methods: HashMap<String, Spanned<ContractExpr>>,
        /// Field contracts for the class
        fields: HashMap<String, Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    // ============= ADVANCED CONTRACTS =============
    
    /// Recursive contract: (rec/c name contract)
    Recursive {
        /// Name of the recursive contract
        name: String,
        /// Body contract that may reference the name
        contract: Box<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Contract reference (for recursive contracts)
    Reference {
        /// Name of the referenced contract
        name: String,
        /// Source location of this contract
        location: Span,
    },
    
    /// Parametric contract: (contract/c param ...)
    Parametric {
        /// Name of the parametric contract
        name: String,
        /// Contract parameters
        parameters: Vec<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
    
    /// Blame boundary: (blame/c contract blame-info)
    Blame {
        /// Contract with blame boundary
        contract: Box<Spanned<ContractExpr>>,
        /// Positive blame party
        positive_blame: String,
        /// Negative blame party
        negative_blame: String,
        /// Source location of this contract
        location: Span,
    },
    
    /// Contract with custom violation message
    WithMessage {
        /// Contract to wrap with custom message
        contract: Box<Spanned<ContractExpr>>,
        /// Custom violation message
        message: String,
        /// Source location of this contract
        location: Span,
    },
    
    // ============= SYNTAX HELPERS =============
    
    /// Contract variable (for parametric contracts)
    Variable {
        /// Variable name for parametric contracts
        name: String,
        /// Source location of this contract
        location: Span,
    },
    
    /// Contract application: (contract arg ...)
    Application {
        /// Contract to apply
        contract: Box<Spanned<ContractExpr>>,
        /// Arguments to apply to the contract
        arguments: Vec<Spanned<ContractExpr>>,
        /// Source location of this contract
        location: Span,
    },
}

/// Dependent binding in dependent function contracts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependentBinding {
    /// Parameter name
    pub name: String,
    /// Contract for the parameter
    pub contract: Spanned<ContractExpr>,
    /// Optional dependency expression
    pub dependency: Option<Box<Spanned<Expr>>>,
    /// Binding location
    pub location: Span,
}

/// Function case in case function contracts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCase {
    /// Domain contracts
    pub domain: Vec<Spanned<ContractExpr>>,
    /// Codomain contract
    pub codomain: Spanned<ContractExpr>,
    /// Case location
    pub location: Span,
}

/// Comparison operators for comparison contracts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOp {
    /// Less than comparison (</c)
    LessThan,
    /// Less than or equal comparison (<=/c)
    LessThanEqual,
    /// Greater than comparison (>/c)
    GreaterThan,
    /// Greater than or equal comparison (>=/c)
    GreaterThanEqual,
    /// Equal comparison (=/c)
    Equal,
    /// Not equal comparison (!=/c)
    NotEqual,
}

impl ContractExpr {
    /// Gets the location span for this contract expression.
    pub fn location(&self) -> Span {
        match self {
            ContractExpr::Predicate { location, .. } => *location,
            ContractExpr::Flat { location, .. } => *location,
            ContractExpr::Any { location } => *location,
            ContractExpr::None { location } => *location,
            ContractExpr::Function { location, .. } => *location,
            ContractExpr::DependentFunction { location, .. } => *location,
            ContractExpr::CaseFunction { location, .. } => *location,
            ContractExpr::And { location, .. } => *location,
            ContractExpr::Or { location, .. } => *location,
            ContractExpr::Not { location, .. } => *location,
            ContractExpr::OneOf { location, .. } => *location,
            ContractExpr::Between { location, .. } => *location,
            ContractExpr::Comparison { location, .. } => *location,
            ContractExpr::ListOf { location, .. } => *location,
            ContractExpr::VectorOf { location, .. } => *location,
            ContractExpr::Hash { location, .. } => *location,
            ContractExpr::Tuple { location, .. } => *location,
            ContractExpr::List { location, .. } => *location,
            ContractExpr::Vector { location, .. } => *location,
            ContractExpr::Object { location, .. } => *location,
            ContractExpr::Class { location, .. } => *location,
            ContractExpr::Recursive { location, .. } => *location,
            ContractExpr::Reference { location, .. } => *location,
            ContractExpr::Parametric { location, .. } => *location,
            ContractExpr::Blame { location, .. } => *location,
            ContractExpr::WithMessage { location, .. } => *location,
            ContractExpr::Variable { location, .. } => *location,
            ContractExpr::Application { location, .. } => *location,
        }
    }
    
    /// Returns true if this contract is a basic predicate.
    pub fn is_predicate(&self) -> bool {
        matches!(self, ContractExpr::Predicate { .. })
    }
    
    /// Returns true if this contract is a function contract.
    pub fn is_function_contract(&self) -> bool {
        matches!(
            self,
            ContractExpr::Function { .. } |
            ContractExpr::DependentFunction { .. } |
            ContractExpr::CaseFunction { .. }
        )
    }
    
    /// Returns true if this contract is a combinator.
    pub fn is_combinator(&self) -> bool {
        matches!(
            self,
            ContractExpr::And { .. } |
            ContractExpr::Or { .. } |
            ContractExpr::Not { .. } |
            ContractExpr::OneOf { .. } |
            ContractExpr::Between { .. } |
            ContractExpr::Comparison { .. }
        )
    }
    
    /// Returns true if this contract is structural.
    pub fn is_structural(&self) -> bool {
        matches!(
            self,
            ContractExpr::ListOf { .. } |
            ContractExpr::VectorOf { .. } |
            ContractExpr::Hash { .. } |
            ContractExpr::Tuple { .. } |
            ContractExpr::List { .. } |
            ContractExpr::Vector { .. }
        )
    }
    
    /// Gets the predicate name if this is a predicate contract.
    pub fn as_predicate(&self) -> Option<&str> {
        match self {
            ContractExpr::Predicate { name, .. } => Some(name),
            _ => None,
        }
    }
    
    /// Creates a predicate contract.
    pub fn predicate(name: impl Into<String>, location: Span) -> Self {
        ContractExpr::Predicate {
            name: name.into(),
            location,
        }
    }
    
    /// Creates a function contract.
    pub fn function(
        domain: Vec<Spanned<ContractExpr>>,
        codomain: Spanned<ContractExpr>,
        location: Span,
    ) -> Self {
        ContractExpr::Function {
            domain,
            codomain: Box::new(codomain),
            location,
        }
    }
    
    /// Creates an and combinator.
    pub fn and(contracts: Vec<Spanned<ContractExpr>>, location: Span) -> Self {
        ContractExpr::And { contracts, location }
    }
    
    /// Creates an or combinator.
    pub fn or(contracts: Vec<Spanned<ContractExpr>>, location: Span) -> Self {
        ContractExpr::Or { contracts, location }
    }
    
    /// Creates a not combinator.
    pub fn not(contract: Spanned<ContractExpr>, location: Span) -> Self {
        ContractExpr::Not {
            contract: Box::new(contract),
            location,
        }
    }
    
    /// Creates a listof contract.
    pub fn listof(element_contract: Spanned<ContractExpr>, location: Span) -> Self {
        ContractExpr::ListOf {
            element_contract: Box::new(element_contract),
            location,
        }
    }
    
    /// Creates an any contract.
    pub fn any(location: Span) -> Self {
        ContractExpr::Any { location }
    }
}

impl DependentBinding {
    /// Creates a new dependent binding.
    pub fn new(
        name: impl Into<String>,
        contract: Spanned<ContractExpr>,
        dependency: Option<Box<Spanned<Expr>>>,
        location: Span,
    ) -> Self {
        Self {
            name: name.into(),
            contract,
            dependency,
            location,
        }
    }
}

impl FunctionCase {
    /// Creates a new function case.
    pub fn new(
        domain: Vec<Spanned<ContractExpr>>,
        codomain: Spanned<ContractExpr>,
        location: Span,
    ) -> Self {
        Self {
            domain,
            codomain,
            location,
        }
    }
}

impl fmt::Display for ContractExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractExpr::Predicate { name, .. } => write!(f, "{name}"),
            ContractExpr::Flat { check_expr, .. } => write!(f, "(flat/c {})", check_expr.inner),
            ContractExpr::Any { .. } => write!(f, "any/c"),
            ContractExpr::None { .. } => write!(f, "none/c"),
            ContractExpr::Function { domain, codomain, .. } => {
                write!(f, "(->")?;
                for contract in domain {
                    write!(f, " {}", contract.inner)?;
                }
                write!(f, " {})", codomain.inner)
            }
            ContractExpr::DependentFunction { bindings, codomain, .. } => {
                write!(f, "(->i (")?;
                for (i, binding) in bindings.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "[{} {}]", binding.name, binding.contract.inner)?;
                }
                write!(f, ") {})", codomain.inner)
            }
            ContractExpr::CaseFunction { cases, .. } => {
                write!(f, "(->*")?;
                for case in cases {
                    write!(f, " (")?;
                    for (i, domain) in case.domain.iter().enumerate() {
                        if i > 0 { write!(f, " ")?; }
                        write!(f, "{}", domain.inner)?;
                    }
                    write!(f, " {})", case.codomain.inner)?;
                }
                write!(f, ")")
            }
            ContractExpr::And { contracts, .. } => {
                write!(f, "(and/c")?;
                for contract in contracts {
                    write!(f, " {}", contract.inner)?;
                }
                write!(f, ")")
            }
            ContractExpr::Or { contracts, .. } => {
                write!(f, "(or/c")?;
                for contract in contracts {
                    write!(f, " {}", contract.inner)?;
                }
                write!(f, ")")
            }
            ContractExpr::Not { contract, .. } => {
                write!(f, "(not/c {})", contract.inner)
            }
            ContractExpr::OneOf { values, .. } => {
                write!(f, "(one-of/c")?;
                for value in values {
                    write!(f, " {}", value.inner)?;
                }
                write!(f, ")")
            }
            ContractExpr::Between { min, max, .. } => {
                write!(f, "(between/c {} {})", min.inner, max.inner)
            }
            ContractExpr::Comparison { operator, value, .. } => {
                let op_str = match operator {
                    ComparisonOp::LessThan => "</c",
                    ComparisonOp::LessThanEqual => "<=/c",
                    ComparisonOp::GreaterThan => ">/c",
                    ComparisonOp::GreaterThanEqual => ">=/c",
                    ComparisonOp::Equal => "=/c",
                    ComparisonOp::NotEqual => "!=/c",
                };
                write!(f, "({} {})", op_str, value.inner)
            }
            ContractExpr::ListOf { element_contract, .. } => {
                write!(f, "(listof {})", element_contract.inner)
            }
            ContractExpr::VectorOf { element_contract, .. } => {
                write!(f, "(vectorof {})", element_contract.inner)
            }
            ContractExpr::Hash { key_contract, value_contract, .. } => {
                write!(f, "(hash/c {} {})", key_contract.inner, value_contract.inner)
            }
            ContractExpr::Tuple { element_contracts, .. } => {
                write!(f, "(tuple/c")?;
                for contract in element_contracts {
                    write!(f, " {}", contract.inner)?;
                }
                write!(f, ")")
            }
            ContractExpr::List { element_contracts, .. } => {
                write!(f, "(list/c")?;
                for contract in element_contracts {
                    write!(f, " {}", contract.inner)?;
                }
                write!(f, ")")
            }
            ContractExpr::Vector { element_contracts, .. } => {
                write!(f, "(vector/c")?;
                for contract in element_contracts {
                    write!(f, " {}", contract.inner)?;
                }
                write!(f, ")")
            }
            ContractExpr::Object { methods, .. } => {
                write!(f, "(object/c")?;
                for (name, contract) in methods {
                    write!(f, " [{} {}]", name, contract.inner)?;
                }
                write!(f, ")")
            }
            ContractExpr::Class { methods, fields, .. } => {
                write!(f, "(class/c")?;
                for (name, contract) in methods {
                    write!(f, " [method {} {}]", name, contract.inner)?;
                }
                for (name, contract) in fields {
                    write!(f, " [field {} {}]", name, contract.inner)?;
                }
                write!(f, ")")
            }
            ContractExpr::Recursive { name, contract, .. } => {
                write!(f, "(rec/c {} {})", name, contract.inner)
            }
            ContractExpr::Reference { name, .. } => {
                write!(f, "{name}")
            }
            ContractExpr::Parametric { name, parameters, .. } => {
                write!(f, "({name}")?;
                for param in parameters {
                    write!(f, " {}", param.inner)?;
                }
                write!(f, ")")
            }
            ContractExpr::Blame { contract, positive_blame, negative_blame, .. } => {
                write!(f, "(blame/c {} {} {})", contract.inner, positive_blame, negative_blame)
            }
            ContractExpr::WithMessage { contract, message, .. } => {
                write!(f, "(with-message/c {} \"{}\")", contract.inner, message)
            }
            ContractExpr::Variable { name, .. } => write!(f, "{name}"),
            ContractExpr::Application { contract, arguments, .. } => {
                write!(f, "({}", contract.inner)?;
                for arg in arguments {
                    write!(f, " {}", arg.inner)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for ComparisonOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOp::LessThan => write!(f, "<"),
            ComparisonOp::LessThanEqual => write!(f, "<="),
            ComparisonOp::GreaterThan => write!(f, ">"),
            ComparisonOp::GreaterThanEqual => write!(f, ">="),
            ComparisonOp::Equal => write!(f, "="),
            ComparisonOp::NotEqual => write!(f, "!="),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    fn dummy_span() -> Span {
        Span::new(0, 1)
    }

    fn spanned_contract(expr: ContractExpr) -> Spanned<ContractExpr> {
        Spanned::new(expr, dummy_span())
    }

    #[test]
    fn test_contract_creation() {
        let span = dummy_span();
        let number_contract = ContractExpr::predicate("number?", span);
        assert!(number_contract.is_predicate());
        assert_eq!(number_contract.as_predicate(), Some("number?"));
    }

    #[test]
    fn test_function_contract_display() {
        let span = dummy_span();
        let number_contract = spanned_contract(ContractExpr::predicate("number?", span));
        let string_contract = spanned_contract(ContractExpr::predicate("string?", span));
        
        let func_contract = ContractExpr::function(
            vec![number_contract.clone(), number_contract.clone()],
            string_contract,
            span,
        );
        
        assert!(func_contract.is_function_contract());
        assert_eq!(format!("{func_contract}"), "(-> number? number? string?)");
    }

    #[test]
    fn test_combinator_contracts() {
        let span = dummy_span();
        let number_contract = spanned_contract(ContractExpr::predicate("number?", span));
        let string_contract = spanned_contract(ContractExpr::predicate("string?", span));
        
        let and_contract = ContractExpr::and(vec![number_contract.clone(), string_contract.clone()], span);
        assert!(and_contract.is_combinator());
        assert_eq!(format!("{and_contract}"), "(and/c number? string?)");
        
        let or_contract = ContractExpr::or(vec![number_contract.clone(), string_contract.clone()], span);
        assert!(or_contract.is_combinator());
        assert_eq!(format!("{or_contract}"), "(or/c number? string?)");
        
        let not_contract = ContractExpr::not(number_contract, span);
        assert!(not_contract.is_combinator());
        assert_eq!(format!("{not_contract}"), "(not/c number?)");
    }

    #[test]
    fn test_structural_contracts() {
        let span = dummy_span();
        let number_contract = spanned_contract(ContractExpr::predicate("number?", span));
        
        let listof_contract = ContractExpr::listof(number_contract, span);
        assert!(listof_contract.is_structural());
        assert_eq!(format!("{listof_contract}"), "(listof number?)");
    }
}