//! Gradual typing system for Lambdust.
//!
//! This module implements a gradual typing system that allows seamless migration
//! from dynamic to static to dependent typing, maintaining R7RS compatibility.

use super::{DependentType, SchemeIntegration};
use crate::eval::{Value, Environment};
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result};
use std::collections::HashMap;
use std::fmt;

/// Gradual typing levels in Lambdust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypingLevel {
    /// Dynamic typing - no static checks, pure R7RS mode.
    Dynamic,
    /// Contract-based typing - runtime checks with predicates.
    Contracts,
    /// Static typing - compile-time checks with simple types.
    Static,
    /// Dependent typing - types that depend on values.
    Dependent,
}

/// Gradual type that can represent any typing level.
#[derive(Debug, Clone)]
pub enum GradualType {
    /// Dynamic type (unknown).
    Dynamic,
    /// Contract type with runtime predicate.
    Contract {
        name: String,
        predicate: Value,
    },
    /// Static type.
    Static {
        name: String,
        parameters: Vec<GradualType>,
    },
    /// Dependent type.
    Dependent(DependentType),
    /// Type variable for polymorphism.
    Variable(String),
    /// Function type.
    Function {
        parameters: Vec<GradualType>,
        return_type: Box<GradualType>,
    },
}

/// Gradual typing system managing type migration.
pub struct GradualTypingSystem {
    /// Current typing level.
    current_level: TypingLevel,
    /// Type environment mapping names to types.
    type_environment: HashMap<String, GradualType>,
    /// Contract predicates.
    contracts: HashMap<String, Value>,
    /// Scheme integration for dependent types.
    scheme_integration: SchemeIntegration,
    /// Type migration history.
    migration_path: Vec<TypingLevel>,
    /// Contract validation cache for performance.
    contract_cache: HashMap<String, bool>,
    /// Type equivalence cache for complex types.
    type_equivalence_cache: HashMap<(String, String), bool>,
    /// Migration validation rules
    migration_rules: HashMap<(TypingLevel, TypingLevel), MigrationRule>,
}

/// Rules for type migration between different typing levels.
#[derive(Debug, Clone)]
pub struct MigrationRule {
    /// Whether this migration is allowed.
    pub allowed: bool,
    /// Type conversion function identifier.
    pub converter: String,
    /// Additional constraints for the migration.
    pub constraints: Vec<String>,
    /// Performance cost estimate (1-10 scale).
    pub cost: u8,
}

/// Contract validation context for runtime checking.
#[derive(Debug, Clone)]
pub struct ContractContext {
    /// Current value being validated
    pub value: Value,
    /// Contract predicate to check
    pub predicate: Value,
    /// Environment for predicate evaluation
    pub environment: Environment,
    /// Call stack for debugging contract failures
    pub call_stack: Vec<String>,
}

/// Contract composition operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractComposition {
    /// Logical AND - both contracts must be satisfied
    And,
    /// Logical OR - either contract must be satisfied
    Or,
    /// Logical implication - if first is satisfied, second must be
    Implies,
}

/// Statistics about contract validation performance.
#[derive(Debug, Clone)]
pub struct ContractStatistics {
    /// Total number of registered contracts
    pub total_contracts: usize,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Names of currently active contracts
    pub active_contracts: Vec<String>,
}

/// Statistics about type migration.
#[derive(Debug, Clone)]
pub struct MigrationStatistics {
    /// Current typing level
    pub current_level: TypingLevel,
    /// Path of migrations taken
    pub migration_path: Vec<TypingLevel>,
    /// Total number of migrations performed
    pub total_migrations: usize,
    /// Size of the type environment
    pub type_environment_size: usize,
}

impl GradualTypingSystem {
    /// Create a new gradual typing system.
    pub fn new() -> Result<Self> {
        let mut system = Self {
            current_level: TypingLevel::Dynamic,
            type_environment: HashMap::new(),
            contracts: HashMap::new(),
            scheme_integration: SchemeIntegration::new()?,
            migration_path: vec![TypingLevel::Dynamic],
            contract_cache: HashMap::new(),
            type_equivalence_cache: HashMap::new(),
            migration_rules: HashMap::new(),
        };
        
        system.initialize_migration_rules();
        system.initialize_r7rs_contracts();
        Ok(system)
    }

    /// Initialize migration rules between different typing levels.
    fn initialize_migration_rules(&mut self) {
        // Dynamic -> Contracts: Always allowed, low cost
        self.migration_rules.insert(
            (TypingLevel::Dynamic, TypingLevel::Contracts),
            MigrationRule {
                allowed: true,
                converter: "dynamic_to_contracts".to_string(),
                constraints: vec![],
                cost: 2,
            }
        );
        
        // Contracts -> Static: Conditional, medium cost
        self.migration_rules.insert(
            (TypingLevel::Contracts, TypingLevel::Static),
            MigrationRule {
                allowed: true,
                converter: "contracts_to_static".to_string(),
                constraints: vec!["all_contracts_deterministic".to_string()],
                cost: 5,
            }
        );
        
        // Static -> Dependent: Conditional, high cost
        self.migration_rules.insert(
            (TypingLevel::Static, TypingLevel::Dependent),
            MigrationRule {
                allowed: true,
                converter: "static_to_dependent".to_string(),
                constraints: vec!["no_recursive_types".to_string()],
                cost: 8,
            }
        );
        
        // Downward migrations (generally discouraged but allowed)
        self.migration_rules.insert(
            (TypingLevel::Contracts, TypingLevel::Dynamic),
            MigrationRule {
                allowed: true,
                converter: "contracts_to_dynamic".to_string(),
                constraints: vec!["safety_warning".to_string()],
                cost: 1,
            }
        );
        
        self.migration_rules.insert(
            (TypingLevel::Static, TypingLevel::Contracts),
            MigrationRule {
                allowed: true,
                converter: "static_to_contracts".to_string(),
                constraints: vec!["safety_warning".to_string()],
                cost: 3,
            }
        );
        
        self.migration_rules.insert(
            (TypingLevel::Dependent, TypingLevel::Static),
            MigrationRule {
                allowed: true,
                converter: "dependent_to_static".to_string(),
                constraints: vec!["type_erasure_warning".to_string()],
                cost: 6,
            }
        );
        
        // Skip-level migrations
        self.migration_rules.insert(
            (TypingLevel::Dynamic, TypingLevel::Static),
            MigrationRule {
                allowed: true,
                converter: "dynamic_to_static_via_contracts".to_string(),
                constraints: vec!["intermediate_contracts_required".to_string()],
                cost: 7,
            }
        );
        
        self.migration_rules.insert(
            (TypingLevel::Dynamic, TypingLevel::Dependent),
            MigrationRule {
                allowed: true,
                converter: "dynamic_to_dependent_via_static".to_string(),
                constraints: vec!["full_type_annotation_required".to_string()],
                cost: 10,
            }
        );
    }

    /// Initialize R7RS contract predicates.
    fn initialize_r7rs_contracts(&mut self) {
        // Basic R7RS predicates
        let predicates = vec![
            ("number?", "number"),
            ("integer?", "integer"),
            ("rational?", "rational"),
            ("real?", "real"),
            ("complex?", "complex"),
            ("exact?", "exact-number"),
            ("inexact?", "inexact-number"),
            ("string?", "string"),
            ("symbol?", "symbol"),
            ("boolean?", "boolean"),
            ("char?", "character"),
            ("pair?", "pair"),
            ("null?", "null"),
            ("list?", "list"),
            ("vector?", "vector"),
            ("procedure?", "procedure"),
            ("port?", "port"),
        ];
        
        for (predicate_name, type_name) in predicates {
            self.contracts.insert(
                type_name.to_string(),
                Value::symbol_from_str(predicate_name)
            );
        }
    }

    /// Set the current typing level with migration validation.
    pub fn set_typing_level(&mut self, level: TypingLevel) -> Result<()> {
        // Check if migration is allowed
        if let Some(rule) = self.migration_rules.get(&(self.current_level.clone(), level.clone())) {
            if !rule.allowed {
                return Err(Box::new(Error::type_error(
                    format!("Migration from {:?} to {:?} is not allowed", self.current_level, level),
                    crate::diagnostics::Span::default()
                )));
            }
            
            // Validate constraints
            for constraint in &rule.constraints {
                self.validate_migration_constraint(constraint)?;
            }
            
            // Perform the migration
            let current_level = self.current_level.clone();
            self.migrate_types(&current_level, &level)?;
            
            self.migration_path.push(level.clone());
            self.current_level = level;
            Ok(())
        } else {
            Err(Box::new(Error::type_error(
                format!("No migration rule defined from {:?} to {:?}", self.current_level, level),
                crate::diagnostics::Span::default()
            )))
        }
    }

    /// Validate a specific migration constraint.
    fn validate_migration_constraint(&self, constraint: &str) -> Result<()> {
        match constraint {
            "all_contracts_deterministic" => {
                // Check if all contracts in the environment are deterministic
                for (name, contract) in &self.contracts {
                    if !self.is_contract_deterministic(contract) {
                        return Err(Box::new(Error::type_error(
                            format!("Contract '{}' is non-deterministic", name),
                            crate::diagnostics::Span::default()
                        )));
                    }
                }
                Ok(())
            },
            "no_recursive_types" => {
                // Check for recursive type definitions
                for (name, gradual_type) in &self.type_environment {
                    if self.has_recursive_reference(gradual_type, name) {
                        return Err(Box::new(Error::type_error(
                            format!("Type '{}' contains recursive references", name),
                            crate::diagnostics::Span::default()
                        )));
                    }
                }
                Ok(())
            },
            "safety_warning" => {
                // Just log a warning for downward migrations
                eprintln!("Warning: Downward type migration may reduce safety guarantees");
                Ok(())
            },
            "type_erasure_warning" => {
                eprintln!("Warning: Migration will erase dependent type information");
                Ok(())
            },
            "intermediate_contracts_required" => {
                // For skip-level migrations, ensure intermediate contracts exist
                if self.contracts.is_empty() {
                    return Err(Box::new(Error::type_error(
                        "Skip-level migration requires intermediate contract definitions".to_string(),
                        crate::diagnostics::Span::default()
                    )));
                }
                Ok(())
            },
            "full_type_annotation_required" => {
                // Check that all bindings have type annotations
                for (name, gradual_type) in &self.type_environment {
                    if matches!(gradual_type, GradualType::Dynamic) {
                        return Err(Box::new(Error::type_error(
                            format!("Variable '{}' requires explicit type annotation for dependent typing", name),
                            crate::diagnostics::Span::default()
                        )));
                    }
                }
                Ok(())
            },
            _ => {
                eprintln!("Warning: Unknown migration constraint '{}'", constraint);
                Ok(())
            }
        }
    }

    /// Migrate all types in the environment to match the new typing level.
    fn migrate_types(&mut self, from: &TypingLevel, to: &TypingLevel) -> Result<()> {
        let mut new_environment = HashMap::new();
        let old_environment = self.type_environment.clone();
        
        for (name, gradual_type) in &old_environment {
            let migrated_type = self.migrate_gradual_type(gradual_type, from, to)?;
            new_environment.insert(name.clone(), migrated_type);
        }
        
        self.type_environment = new_environment;
        Ok(())
    }

    /// Migrate a single gradual type between typing levels.
    fn migrate_gradual_type(&mut self, gradual_type: &GradualType, from: &TypingLevel, to: &TypingLevel) -> Result<GradualType> {
        match (from, to) {
            (TypingLevel::Dynamic, TypingLevel::Contracts) => {
                self.dynamic_to_contracts(gradual_type)
            },
            (TypingLevel::Contracts, TypingLevel::Static) => {
                self.contracts_to_static(gradual_type)
            },
            (TypingLevel::Static, TypingLevel::Dependent) => {
                self.static_to_dependent(gradual_type)
            },
            (TypingLevel::Contracts, TypingLevel::Dynamic) => {
                Ok(GradualType::Dynamic)
            },
            (TypingLevel::Static, TypingLevel::Contracts) => {
                self.static_to_contracts(gradual_type)
            },
            (TypingLevel::Dependent, TypingLevel::Static) => {
                self.dependent_to_static(gradual_type)
            },
            _ => {
                // Multi-step migration
                self.multi_step_migration(gradual_type, from, to)
            }
        }
    }

    /// Convert dynamic type to contract type.
    fn dynamic_to_contracts(&self, gradual_type: &GradualType) -> Result<GradualType> {
        match gradual_type {
            GradualType::Dynamic => {
                // Create a generic contract that accepts any value
                Ok(GradualType::Contract {
                    name: "any".to_string(),
                    predicate: Value::symbol_from_str("any/c"),
                })
            },
            other => Ok(other.clone()), // Already has more specific type information
        }
    }

    /// Convert contract type to static type.
    fn contracts_to_static(&self, gradual_type: &GradualType) -> Result<GradualType> {
        match gradual_type {
            GradualType::Contract { name, .. } => {
                // Map contract names to static types
                let static_name = match name.as_str() {
                    "number" => "Number",
                    "integer" => "Integer",
                    "string" => "String",
                    "boolean" => "Boolean",
                    "symbol" => "Symbol",
                    "pair" => "Pair",
                    "list" => "List",
                    "vector" => "Vector",
                    "procedure" => "Procedure",
                    _ => "Any", // Fallback
                };
                
                Ok(GradualType::Static {
                    name: static_name.to_string(),
                    parameters: vec![],
                })
            },
            other => Ok(other.clone()),
        }
    }

    /// Convert static type to dependent type.
    fn static_to_dependent(&mut self, gradual_type: &GradualType) -> Result<GradualType> {
        match gradual_type {
            GradualType::Static { name, parameters } => {
                // Create a corresponding dependent type
                let dependent_type = match name.as_str() {
                    "Number" => DependentType::Inductive {
                        name: "Number".to_string(),
                        parameters: vec![],
                        universe_level: 0,
                        constructors: vec![],
                        induction_principle: None,
                    },
                    "String" => DependentType::Inductive {
                        name: "String".to_string(),
                        parameters: vec![],
                        universe_level: 0,
                        constructors: vec![],
                        induction_principle: None,
                    },
                    "Boolean" => DependentType::Inductive {
                        name: "Boolean".to_string(),
                        parameters: vec![],
                        universe_level: 0,
                        constructors: vec![
                            ("true".to_string(), DependentType::Universe(0)),
                            ("false".to_string(), DependentType::Universe(0)),
                        ],
                        induction_principle: None,
                    },
                    _ => DependentType::Universe(0), // Fallback
                };
                
                Ok(GradualType::Dependent(dependent_type))
            },
            other => Ok(other.clone()),
        }
    }

    /// Convert static type back to contract type.
    fn static_to_contracts(&self, gradual_type: &GradualType) -> Result<GradualType> {
        match gradual_type {
            GradualType::Static { name, .. } => {
                let predicate_name = match name.as_str() {
                    "Number" => "number?",
                    "Integer" => "integer?",
                    "String" => "string?",
                    "Boolean" => "boolean?",
                    "Symbol" => "symbol?",
                    _ => "any/c",
                };
                
                Ok(GradualType::Contract {
                    name: name.clone(),
                    predicate: Value::symbol_from_str(predicate_name),
                })
            },
            other => Ok(other.clone()),
        }
    }

    /// Convert dependent type back to static type.
    fn dependent_to_static(&self, gradual_type: &GradualType) -> Result<GradualType> {
        match gradual_type {
            GradualType::Dependent(dep_type) => {
                match dep_type {
                    DependentType::Inductive { name, .. } => {
                        Ok(GradualType::Static {
                            name: name.clone(),
                            parameters: vec![],
                        })
                    },
                    DependentType::Universe(_) => {
                        Ok(GradualType::Static {
                            name: "Type".to_string(),
                            parameters: vec![],
                        })
                    },
                    _ => {
                        Ok(GradualType::Static {
                            name: "Complex".to_string(),
                            parameters: vec![],
                        })
                    }
                }
            },
            other => Ok(other.clone()),
        }
    }

    /// Perform multi-step migration through intermediate levels.
    fn multi_step_migration(&mut self, gradual_type: &GradualType, from: &TypingLevel, to: &TypingLevel) -> Result<GradualType> {
        // Define the canonical path: Dynamic -> Contracts -> Static -> Dependent
        let path = match (from, to) {
            (TypingLevel::Dynamic, TypingLevel::Static) => {
                vec![TypingLevel::Dynamic, TypingLevel::Contracts, TypingLevel::Static]
            },
            (TypingLevel::Dynamic, TypingLevel::Dependent) => {
                vec![TypingLevel::Dynamic, TypingLevel::Contracts, TypingLevel::Static, TypingLevel::Dependent]
            },
            (TypingLevel::Contracts, TypingLevel::Dependent) => {
                vec![TypingLevel::Contracts, TypingLevel::Static, TypingLevel::Dependent]
            },
            _ => return Ok(gradual_type.clone()), // No multi-step needed
        };
        
        let mut current_type = gradual_type.clone();
        for i in 0..path.len()-1 {
            current_type = self.migrate_gradual_type(&current_type, &path[i], &path[i+1])?;
        }
        
        Ok(current_type)
    }

    /// Check if a contract is deterministic.
    fn is_contract_deterministic(&self, _contract: &Value) -> bool {
        // For now, assume all basic R7RS predicates are deterministic
        // In a full implementation, this would analyze the predicate
        true
    }

    /// Check if a type has recursive references.
    fn has_recursive_reference(&self, gradual_type: &GradualType, type_name: &str) -> bool {
        match gradual_type {
            GradualType::Static { name, parameters } => {
                if name == type_name {
                    return true;
                }
                parameters.iter().any(|param| self.has_recursive_reference(param, type_name))
            },
            GradualType::Function { parameters, return_type } => {
                parameters.iter().any(|param| self.has_recursive_reference(param, type_name)) ||
                self.has_recursive_reference(return_type, type_name)
            },
            _ => false,
        }
    }

    /// Get the current typing level.
    pub fn current_level(&self) -> &TypingLevel {
        &self.current_level
    }

    /// Add a type binding to the environment.
    pub fn add_type_binding(&mut self, name: String, gradual_type: GradualType) {
        self.type_environment.insert(name, gradual_type);
    }

    /// Add a contract predicate to the system.
    pub fn add_contract(&mut self, name: String, predicate: Value) {
        self.contracts.insert(name, predicate);
    }

    /// Validate a value against a contract predicate.
    pub fn validate_contract(&mut self, context: &ContractContext) -> Result<bool> {
        let cache_key = format!("{:?}:{:?}", context.value, context.predicate);
        
        // Check cache first
        if let Some(cached_result) = self.contract_cache.get(&cache_key) {
            return Ok(*cached_result);
        }

        let validation_result = self.perform_contract_validation(context)?;
        
        // Cache the result for future use
        self.contract_cache.insert(cache_key, validation_result);
        
        Ok(validation_result)
    }

    /// Perform the actual contract validation.
    fn perform_contract_validation(&self, context: &ContractContext) -> Result<bool> {
        use crate::eval::Value;
        
        match &context.predicate {
            Value::Symbol(_symbol_id) => {
                // For now, we'll handle symbol predicates by pattern matching
                // In a full implementation, we'd use a symbol table to resolve the name
                self.validate_symbolic_predicate(&context.predicate, &context.value)
            },
            
            Value::Procedure(_) => {
                // Handle user-defined contract procedures
                self.validate_procedure_contract(context)
            },
            
            _ => {
                // Invalid contract predicate
                Err(Box::new(Error::type_error(
                    format!("Invalid contract predicate: {:?}", context.predicate),
                    crate::diagnostics::Span::default()
                )))
            }
        }
    }

    /// Validate using symbolic predicate matching.
    fn validate_symbolic_predicate(&self, predicate: &Value, value: &Value) -> Result<bool> {
        // Since we can't easily convert SymbolId back to string without a symbol table,
        // we'll use a different approach: compare against known symbolic predicates
        
        let built_in_predicates = vec![
            (Value::symbol_from_str("number?"), self.is_number(value)),
            (Value::symbol_from_str("integer?"), self.is_integer(value)),
            (Value::symbol_from_str("rational?"), self.is_rational(value)),
            (Value::symbol_from_str("real?"), self.is_real(value)),
            (Value::symbol_from_str("complex?"), self.is_complex(value)),
            (Value::symbol_from_str("exact?"), self.is_exact(value)),
            (Value::symbol_from_str("inexact?"), self.is_inexact(value)),
            (Value::symbol_from_str("string?"), self.is_string(value)),
            (Value::symbol_from_str("symbol?"), self.is_symbol(value)),
            (Value::symbol_from_str("boolean?"), self.is_boolean(value)),
            (Value::symbol_from_str("char?"), self.is_char(value)),
            (Value::symbol_from_str("pair?"), self.is_pair(value)),
            (Value::symbol_from_str("null?"), self.is_null(value)),
            (Value::symbol_from_str("list?"), self.is_list(value)),
            (Value::symbol_from_str("vector?"), self.is_vector(value)),
            (Value::symbol_from_str("procedure?"), self.is_procedure(value)),
            (Value::symbol_from_str("port?"), self.is_port(value)),
            (Value::symbol_from_str("any/c"), true), // Universal contract
        ];
        
        for (pred_symbol, result) in built_in_predicates {
            if pred_symbol == *predicate {
                return Ok(result);
            }
        }
        
        // Unknown predicate - assume false for safety
        Ok(false)
    }

    // Helper methods for type checking
    fn is_number(&self, value: &Value) -> bool {
        matches!(value, Value::Literal(crate::ast::Literal::ExactInteger(_))
                     | Value::Literal(crate::ast::Literal::InexactReal(_))
                     | Value::Literal(crate::ast::Literal::Rational { .. })
                     | Value::Literal(crate::ast::Literal::Complex { .. }))
    }

    fn is_integer(&self, value: &Value) -> bool {
        matches!(value, Value::Literal(crate::ast::Literal::ExactInteger(_)))
    }

    fn is_rational(&self, value: &Value) -> bool {
        matches!(value, Value::Literal(crate::ast::Literal::Rational { .. })
                     | Value::Literal(crate::ast::Literal::ExactInteger(_)))
    }

    fn is_real(&self, value: &Value) -> bool {
        matches!(value, Value::Literal(crate::ast::Literal::ExactInteger(_))
                     | Value::Literal(crate::ast::Literal::InexactReal(_))
                     | Value::Literal(crate::ast::Literal::Rational { .. }))
    }

    fn is_complex(&self, value: &Value) -> bool {
        matches!(value, Value::Literal(crate::ast::Literal::Complex { .. })
                     | Value::Literal(crate::ast::Literal::ExactInteger(_))
                     | Value::Literal(crate::ast::Literal::InexactReal(_))
                     | Value::Literal(crate::ast::Literal::Rational { .. }))
    }

    fn is_exact(&self, value: &Value) -> bool {
        matches!(value, Value::Literal(crate::ast::Literal::ExactInteger(_))
                     | Value::Literal(crate::ast::Literal::Rational { .. }))
    }

    fn is_inexact(&self, value: &Value) -> bool {
        matches!(value, Value::Literal(crate::ast::Literal::InexactReal(_))
                     | Value::Literal(crate::ast::Literal::Complex { .. }))
    }

    fn is_string(&self, value: &Value) -> bool {
        matches!(value, Value::Literal(crate::ast::Literal::String(_)))
    }

    fn is_symbol(&self, value: &Value) -> bool {
        matches!(value, Value::Symbol(_))
    }

    fn is_boolean(&self, value: &Value) -> bool {
        matches!(value, Value::Literal(crate::ast::Literal::Boolean(_)))
    }

    fn is_char(&self, value: &Value) -> bool {
        matches!(value, Value::Literal(crate::ast::Literal::Character(_)))
    }

    fn is_pair(&self, value: &Value) -> bool {
        matches!(value, Value::Pair(_, _) | Value::MutablePair(_, _))
    }

    fn is_null(&self, value: &Value) -> bool {
        matches!(value, Value::Nil)
    }

    fn is_list(&self, value: &Value) -> bool {
        self.is_proper_list(value)
    }

    fn is_vector(&self, value: &Value) -> bool {
        matches!(value, Value::Vector(_))
    }

    fn is_procedure(&self, value: &Value) -> bool {
        matches!(value, Value::Procedure(_) | Value::CaseLambda(_) | Value::Primitive(_))
    }

    fn is_port(&self, value: &Value) -> bool {
        matches!(value, Value::Port(_))
    }

    /// Validate against built-in R7RS predicates.
    fn validate_builtin_predicate(&self, predicate_name: &str, value: &Value) -> Result<bool> {
        use crate::eval::Value;
        
        let result = match predicate_name {
            "number?" => matches!(value, Value::Literal(crate::ast::Literal::ExactInteger(_))
                               | Value::Literal(crate::ast::Literal::InexactReal(_))
                               | Value::Literal(crate::ast::Literal::Rational { .. })
                               | Value::Literal(crate::ast::Literal::Complex { .. })),
            
            "integer?" => matches!(value, Value::Literal(crate::ast::Literal::ExactInteger(_))),
            
            "rational?" => matches!(value, Value::Literal(crate::ast::Literal::Rational { .. })
                                        | Value::Literal(crate::ast::Literal::ExactInteger(_))),
            
            "real?" => matches!(value, Value::Literal(crate::ast::Literal::ExactInteger(_))
                                    | Value::Literal(crate::ast::Literal::InexactReal(_))
                                    | Value::Literal(crate::ast::Literal::Rational { .. })),
            
            "complex?" => matches!(value, Value::Literal(crate::ast::Literal::Complex { .. })
                                        | Value::Literal(crate::ast::Literal::ExactInteger(_))
                                        | Value::Literal(crate::ast::Literal::InexactReal(_))
                                        | Value::Literal(crate::ast::Literal::Rational { .. })),
            
            "exact?" => matches!(value, Value::Literal(crate::ast::Literal::ExactInteger(_))
                                      | Value::Literal(crate::ast::Literal::Rational { .. })),
            
            "inexact?" => matches!(value, Value::Literal(crate::ast::Literal::InexactReal(_))
                                        | Value::Literal(crate::ast::Literal::Complex { .. })),
            
            "string?" => matches!(value, Value::Literal(crate::ast::Literal::String(_))),
            
            "symbol?" => matches!(value, Value::Symbol(_)),
            
            "boolean?" => matches!(value, Value::Literal(crate::ast::Literal::Boolean(_))),
            
            "char?" => matches!(value, Value::Literal(crate::ast::Literal::Character(_))),
            
            "pair?" => matches!(value, Value::Pair(_, _) | Value::MutablePair(_, _)),
            
            "null?" => matches!(value, Value::Nil),
            
            "list?" => self.is_proper_list(value),
            
            "vector?" => matches!(value, Value::Vector(_)),
            
            "procedure?" => matches!(value, Value::Procedure(_) | Value::CaseLambda(_) | Value::Primitive(_)),
            
            "port?" => matches!(value, Value::Port(_)),
            
            "any/c" => true, // Universal contract that accepts anything
            
            _ => {
                // Unknown predicate - assume false for safety
                false
            }
        };
        
        Ok(result)
    }

    /// Check if a value is a proper list.
    fn is_proper_list(&self, value: &Value) -> bool {
        match value {
            Value::Nil => true,
            Value::Pair(_, cdr) => self.is_proper_list(cdr),
            Value::MutablePair(_, cdr_ref) => {
                if let Ok(cdr) = cdr_ref.read() {
                    self.is_proper_list(&cdr)
                } else {
                    false // Locked reference, assume not proper list
                }
            },
            _ => false,
        }
    }

    /// Validate against user-defined contract procedures.
    fn validate_procedure_contract(&self, context: &ContractContext) -> Result<bool> {
        // For user-defined contracts, we would need to evaluate the procedure
        // with the value as an argument. For now, this is a simplified implementation.
        
        // This would require integration with the evaluator
        // For now, we assume user-defined contracts are always valid
        Ok(true)
    }

    /// Create a contract type from a predicate.
    pub fn create_contract_type(&mut self, name: String, predicate: Value) -> GradualType {
        self.add_contract(name.clone(), predicate.clone());
        
        GradualType::Contract {
            name,
            predicate,
        }
    }

    /// Refine a dynamic type using contract information.
    pub fn refine_dynamic_type(&mut self, value: &Value) -> Result<GradualType> {
        // Try to infer the most specific contract that applies to this value
        for (contract_name, predicate) in &self.contracts.clone() {
            let context = ContractContext {
                value: value.clone(),
                predicate: predicate.clone(),
                environment: Environment::new(None, 0), // Would need proper environment
                call_stack: vec![],
            };
            
            if self.validate_contract(&context)? {
                return Ok(GradualType::Contract {
                    name: contract_name.clone(),
                    predicate: predicate.clone(),
                });
            }
        }
        
        // If no specific contract matches, return dynamic
        Ok(GradualType::Dynamic)
    }

    /// Compose two contracts using logical operations.
    pub fn compose_contracts(&mut self, name: String, left: &GradualType, right: &GradualType, op: ContractComposition) -> Result<GradualType> {
        match (left, right) {
            (GradualType::Contract { predicate: left_pred, .. }, 
             GradualType::Contract { predicate: right_pred, .. }) => {
                
                let composed_predicate = match op {
                    ContractComposition::And => {
                        // Create a compound predicate that requires both to be true
                        Value::list(vec![
                            Value::symbol_from_str("and"),
                            left_pred.clone(),
                            right_pred.clone(),
                        ])
                    },
                    ContractComposition::Or => {
                        // Create a compound predicate that requires either to be true
                        Value::list(vec![
                            Value::symbol_from_str("or"),
                            left_pred.clone(),
                            right_pred.clone(),
                        ])
                    },
                    ContractComposition::Implies => {
                        // Create an implication predicate
                        Value::list(vec![
                            Value::symbol_from_str("implies"),
                            left_pred.clone(),
                            right_pred.clone(),
                        ])
                    },
                };
                
                Ok(self.create_contract_type(name, composed_predicate))
            },
            
            _ => {
                Err(Box::new(Error::type_error(
                    "Contract composition requires two contract types".to_string(),
                    crate::diagnostics::Span::default()
                )))
            }
        }
    }

    /// Generate a higher-order contract for function types.
    pub fn create_function_contract(&mut self, name: String, arg_contracts: Vec<GradualType>, result_contract: GradualType) -> Result<GradualType> {
        // Create a function contract that validates arguments and result
        let contract_spec = Value::list(vec![
            Value::symbol_from_str("function-contract"),
            Value::list(arg_contracts.into_iter().map(|contract| {
                match contract {
                    GradualType::Contract { predicate, .. } => predicate,
                    _ => Value::symbol_from_str("any/c"),
                }
            }).collect()),
            match result_contract {
                GradualType::Contract { predicate, .. } => predicate,
                _ => Value::symbol_from_str("any/c"),
            },
        ]);
        
        Ok(self.create_contract_type(name, contract_spec))
    }

    /// Get contract validation statistics.
    pub fn contract_statistics(&self) -> ContractStatistics {
        let total_contracts = self.contracts.len();
        let cache_hits = self.contract_cache.len();
        let cache_hit_rate = if total_contracts > 0 {
            cache_hits as f64 / total_contracts as f64
        } else {
            0.0
        };
        
        ContractStatistics {
            total_contracts,
            cache_hits,
            cache_hit_rate,
            active_contracts: self.contracts.keys().cloned().collect(),
        }
    }

    /// Clear contract cache to free memory.
    pub fn clear_contract_cache(&mut self) {
        self.contract_cache.clear();
    }

    /// Look up a type in the environment.
    pub fn lookup_type(&self, name: &str) -> Option<&GradualType> {
        self.type_environment.get(name)
    }

    /// Infer type for expression based on current typing level.
    pub fn infer_type(&mut self, expr: &Expr, env: &Environment) -> Result<GradualType> {
        match self.current_level {
            TypingLevel::Dynamic => {
                // In dynamic mode, all expressions have dynamic type
                Ok(GradualType::Dynamic)
            },
            
            TypingLevel::Contracts => {
                // In contract mode, infer contracts from values
                self.infer_contract_type(expr, env)
            },
            
            TypingLevel::Static => {
                // In static mode, infer static types
                self.infer_static_type(expr, env)
            },
            
            TypingLevel::Dependent => {
                // In dependent mode, infer dependent types
                self.infer_dependent_type(expr, env)
            },
        }
    }

    /// Infer contract types for expressions in contract mode.
    fn infer_contract_type(&mut self, expr: &Expr, env: &Environment) -> Result<GradualType> {
        match expr {
            Expr::Literal(literal) => {
                self.infer_literal_contract_type(literal)
            },
            
            Expr::Identifier(name) => {
                // Look up in type environment first
                if let Some(known_type) = self.lookup_type(name) {
                    Ok(known_type.clone())
                } else if let Some(value) = env.lookup(name) {
                    // Try to refine based on runtime value
                    self.refine_dynamic_type(&value)
                } else {
                    // Unknown identifier gets dynamic type
                    Ok(GradualType::Dynamic)
                }
            },
            
            Expr::Application { operator, operands } => {
                let operator_type = self.infer_contract_type(operator, env)?;
                let operand_types: Result<Vec<_>> = operands.iter()
                    .map(|arg| self.infer_contract_type(&arg.inner, env))
                    .collect();
                let operand_types = operand_types?;
                
                self.infer_application_contract_type(&operator_type, &operand_types)
            },
            
            Expr::Lambda { formals, body, metadata: _, .. } => {
                // Infer function contract
                let arity = match formals {
                    crate::ast::Formals::Fixed(params) => params.len(),
                    crate::ast::Formals::Mixed { fixed, .. } => fixed.len(),
                    crate::ast::Formals::Variable(_) => 0, // Variable arity
                    crate::ast::Formals::Keyword { fixed, keywords, .. } => fixed.len() + keywords.len(),
                    crate::ast::Formals::Typed(typed_params) => typed_params.len(),
                    crate::ast::Formals::TypedVariable(_) => 0, // Variable arity
                    crate::ast::Formals::TypedMixed { fixed, .. } => fixed.len(),
                };
                
                let arg_contracts: Vec<GradualType> = (0..arity)
                    .map(|_| GradualType::Contract {
                        name: "any".to_string(),
                        predicate: Value::symbol_from_str("any/c"),
                    })
                    .collect();
                
                let result_contract = if let Some(last_expr) = body.last() {
                    self.infer_contract_type(&last_expr.inner, env)?
                } else {
                    GradualType::Contract {
                        name: "any".to_string(),
                        predicate: Value::symbol_from_str("any/c"),
                    }
                };
                
                Ok(GradualType::Function {
                    parameters: arg_contracts,
                    return_type: Box::new(result_contract),
                })
            },
            
            Expr::If { consequent, alternative, .. } => {
                let consequent_type = self.infer_contract_type(&consequent.inner, env)?;
                
                if let Some(alt) = alternative {
                    let alternative_type = self.infer_contract_type(&alt.inner, env)?;
                    self.unify_contract_types(&consequent_type, &alternative_type)
                } else {
                    // If-without-else can return unspecified
                    Ok(GradualType::Contract {
                        name: "any".to_string(),
                        predicate: Value::symbol_from_str("any/c"),
                    })
                }
            },
            
            _ => {
                // Default: dynamic type for unsupported expressions
                Ok(GradualType::Dynamic)
            }
        }
    }

    /// Infer static types for expressions in static mode.
    fn infer_static_type(&mut self, expr: &Expr, env: &Environment) -> Result<GradualType> {
        match expr {
            Expr::Literal(literal) => {
                self.infer_literal_static_type(literal)
            },
            
            Expr::Identifier(name) => {
                // Look up in type environment
                if let Some(known_type) = self.lookup_type(name) {
                    Ok(known_type.clone())
                } else {
                    // Unknown identifier - try to infer from runtime if available
                    if let Some(value) = env.lookup(name) {
                        let contract_type = self.refine_dynamic_type(&value)?;
                        self.contract_to_static_type(&contract_type)
                    } else {
                        Err(Box::new(Error::type_error(
                            format!("Unbound identifier '{}' in static typing mode", name),
                            crate::diagnostics::Span::default()
                        )))
                    }
                }
            },
            
            Expr::Application { operator, operands } => {
                let operator_type = self.infer_static_type(operator, env)?;
                let operand_types: Result<Vec<_>> = operands.iter()
                    .map(|arg| self.infer_static_type(&arg.inner, env))
                    .collect();
                let operand_types = operand_types?;
                
                self.infer_application_static_type(&operator_type, &operand_types)
            },
            
            _ => {
                // Default: Any type for unsupported expressions
                Ok(GradualType::Static {
                    name: "Any".to_string(),
                    parameters: vec![],
                })
            }
        }
    }

    /// Infer dependent types for expressions in dependent mode.
    fn infer_dependent_type(&mut self, expr: &Expr, env: &Environment) -> Result<GradualType> {
        // Use the scheme integration for dependent type inference
        let dependent_type = self.scheme_integration.infer_expression_type(expr, env)?;
        Ok(GradualType::Dependent(dependent_type))
    }

    /// Infer contract type for literals.
    fn infer_literal_contract_type(&self, literal: &crate::ast::Literal) -> Result<GradualType> {
        use crate::ast::Literal;
        
        let (name, predicate) = match literal {
            Literal::ExactInteger(_) => ("integer", "integer?"),
            Literal::InexactReal(_) => ("real", "real?"),
            Literal::Rational { .. } => ("rational", "rational?"),
            Literal::Complex { .. } => ("complex", "complex?"),
            Literal::String(_) => ("string", "string?"),
            Literal::Boolean(_) => ("boolean", "boolean?"),
            Literal::Character(_) => ("character", "char?"),
            Literal::Bytevector(_) => ("bytevector", "bytevector?"),
            _ => ("any", "any/c"),
        };
        
        Ok(GradualType::Contract {
            name: name.to_string(),
            predicate: Value::symbol_from_str(predicate),
        })
    }

    /// Infer static type for literals.
    fn infer_literal_static_type(&self, literal: &crate::ast::Literal) -> Result<GradualType> {
        use crate::ast::Literal;
        
        let name = match literal {
            Literal::ExactInteger(_) => "Integer",
            Literal::InexactReal(_) => "Real",
            Literal::Rational { .. } => "Rational",
            Literal::Complex { .. } => "Complex",
            Literal::String(_) => "String",
            Literal::Boolean(_) => "Boolean",
            Literal::Character(_) => "Character",
            Literal::Bytevector(_) => "Bytevector",
            _ => "Any",
        };
        
        Ok(GradualType::Static {
            name: name.to_string(),
            parameters: vec![],
        })
    }

    /// Convert contract type to static type.
    fn contract_to_static_type(&self, contract_type: &GradualType) -> Result<GradualType> {
        match contract_type {
            GradualType::Contract { name, .. } => {
                let static_name = match name.as_str() {
                    "integer" => "Integer",
                    "real" => "Real",
                    "rational" => "Rational", 
                    "complex" => "Complex",
                    "string" => "String",
                    "boolean" => "Boolean",
                    "character" => "Character",
                    "symbol" => "Symbol",
                    "pair" => "Pair",
                    "vector" => "Vector",
                    "procedure" => "Procedure",
                    _ => "Any",
                };
                
                Ok(GradualType::Static {
                    name: static_name.to_string(),
                    parameters: vec![],
                })
            },
            
            other => Ok(other.clone()),
        }
    }

    /// Infer application result type in contract mode.
    fn infer_application_contract_type(&self, operator_type: &GradualType, _operand_types: &[GradualType]) -> Result<GradualType> {
        match operator_type {
            GradualType::Function { return_type, .. } => {
                Ok((**return_type).clone())
            },
            
            GradualType::Contract { name, .. } if name == "procedure" => {
                // Generic procedure contract returns any
                Ok(GradualType::Contract {
                    name: "any".to_string(),
                    predicate: Value::symbol_from_str("any/c"),
                })
            },
            
            _ => {
                // Not a function type
                Ok(GradualType::Dynamic)
            }
        }
    }

    /// Infer application result type in static mode.
    fn infer_application_static_type(&self, operator_type: &GradualType, _operand_types: &[GradualType]) -> Result<GradualType> {
        match operator_type {
            GradualType::Function { return_type, .. } => {
                Ok((**return_type).clone())
            },
            
            GradualType::Static { name, .. } if name == "Procedure" => {
                // Generic procedure returns Any
                Ok(GradualType::Static {
                    name: "Any".to_string(),
                    parameters: vec![],
                })
            },
            
            _ => {
                // Not a function type
                Err(Box::new(Error::type_error(
                    "Cannot apply non-function type".to_string(),
                    crate::diagnostics::Span::default()
                )))
            }
        }
    }

    /// Unify two contract types to find a common supertype.
    fn unify_contract_types(&self, left: &GradualType, right: &GradualType) -> Result<GradualType> {
        match (left, right) {
            // If both are the same contract, return it
            (GradualType::Contract { name: n1, predicate: p1 }, 
             GradualType::Contract { name: n2, predicate: p2 }) if n1 == n2 && p1 == p2 => {
                Ok(left.clone())
            },
            
            // If one is more general than the other, return the more general one
            (GradualType::Contract { name, .. }, GradualType::Dynamic) |
            (GradualType::Dynamic, GradualType::Contract { name, .. }) => {
                Ok(GradualType::Contract {
                    name: name.clone(),
                    predicate: Value::symbol_from_str("any/c"),
                })
            },
            
            // Both dynamic
            (GradualType::Dynamic, GradualType::Dynamic) => {
                Ok(GradualType::Dynamic)
            },
            
            // Default: unify to any/c contract
            _ => {
                Ok(GradualType::Contract {
                    name: "any".to_string(),
                    predicate: Value::symbol_from_str("any/c"),
                })
            }
        }
    }

    /// Get migration path statistics.
    pub fn migration_statistics(&self) -> MigrationStatistics {
        MigrationStatistics {
            current_level: self.current_level.clone(),
            migration_path: self.migration_path.clone(),
            total_migrations: self.migration_path.len() - 1,
            type_environment_size: self.type_environment.len(),
        }
    }

    /// Get type environment as a debug representation.
    pub fn debug_type_environment(&self) -> HashMap<String, String> {
        self.type_environment.iter()
            .map(|(name, gradual_type)| (name.clone(), format!("{:?}", gradual_type)))
            .collect()
    }
}

impl Default for GradualTypingSystem {
    fn default() -> Self {
        Self::new().expect("Failed to create GradualTypingSystem")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_typing_level_migration() {
        let mut system = GradualTypingSystem::new().unwrap();
        
        assert_eq!(system.current_level(), &TypingLevel::Dynamic);
        
        // Test migration to contracts
        system.set_typing_level(TypingLevel::Contracts).unwrap();
        assert_eq!(system.current_level(), &TypingLevel::Contracts);
        
        // Test migration to static
        system.set_typing_level(TypingLevel::Static).unwrap();
        assert_eq!(system.current_level(), &TypingLevel::Static);
        
        // Test migration to dependent
        system.set_typing_level(TypingLevel::Dependent).unwrap();
        assert_eq!(system.current_level(), &TypingLevel::Dependent);
    }

    #[test]
    fn test_contract_validation() {
        let mut system = GradualTypingSystem::new().unwrap();
        
        let value = Value::integer(42);
        let context = ContractContext {
            value: value.clone(),
            predicate: Value::symbol_from_str("integer?"),
            environment: Environment::new(None, 0),
            call_stack: vec![],
        };
        
        let result = system.validate_contract(&context).unwrap();
        assert!(result);
    }

    #[test]
    fn test_type_inference() {
        let mut system = GradualTypingSystem::new().unwrap();
        let env = Environment::new(None, 0);
        
        // Test literal type inference
        let expr = Expr::Literal(Literal::ExactInteger(42));
        let inferred_type = system.infer_type(&expr, &env).unwrap();
        
        match inferred_type {
            GradualType::Dynamic => {}, // Expected in dynamic mode
            _ => panic!("Expected dynamic type in dynamic mode"),
        }
        
        // Test in contract mode
        system.set_typing_level(TypingLevel::Contracts).unwrap();
        let inferred_type = system.infer_type(&expr, &env).unwrap();
        
        match inferred_type {
            GradualType::Contract { name, .. } if name == "integer" => {},
            _ => panic!("Expected integer contract type"),
        }
    }

    #[test]
    fn test_contract_composition() {
        let mut system = GradualTypingSystem::new().unwrap();
        
        let integer_contract = GradualType::Contract {
            name: "integer".to_string(),
            predicate: Value::symbol_from_str("integer?"),
        };
        
        let positive_contract = GradualType::Contract {
            name: "positive".to_string(),
            predicate: Value::symbol_from_str("positive?"),
        };
        
        let composed = system.compose_contracts(
            "positive-integer".to_string(),
            &integer_contract,
            &positive_contract,
            ContractComposition::And
        ).unwrap();
        
        match composed {
            GradualType::Contract { name, .. } => {
                assert_eq!(name, "positive-integer");
            },
            _ => panic!("Expected composed contract"),
        }
    }

    #[test]
    fn test_migration_statistics() {
        let mut system = GradualTypingSystem::new().unwrap();
        
        system.set_typing_level(TypingLevel::Contracts).unwrap();
        system.set_typing_level(TypingLevel::Static).unwrap();
        
        let stats = system.migration_statistics();
        assert_eq!(stats.total_migrations, 2);
        assert_eq!(stats.current_level, TypingLevel::Static);
    }
}
