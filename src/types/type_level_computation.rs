//! Type-level computation system for Lambdust.
//!
//! This module implements a type-level computation system that allows:
//! - Type-level functions and their evaluation
//! - Type families and indexed types
//! - Compile-time computation with types
//! - Type-level pattern matching
//! - Integration with dependent types

use super::{Type, TypeVar, Kind, dependent::*};
use crate::diagnostics::{Error, Result, Span};
use std::collections::HashMap;
use std::fmt;

/// Type-level computation expressions.
///
/// These represent computations that happen at the type level,
/// similar to how Term represents value-level computations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeComputation {
    /// Type variable reference
    TypeVar(String),
    
    /// Type constant (base types, constructors)
    TypeConstant(String),
    
    /// Type-level lambda: Λα.T
    TypeLambda {
        param: String,
        param_kind: Kind,
        body: Box<TypeComputation>,
    },
    
    /// Type-level application: F[A]
    TypeApplication {
        function: Box<TypeComputation>,
        argument: Box<TypeComputation>,
    },
    
    /// Type-level conditional: if P then T else U
    TypeConditional {
        condition: Box<TypeComputation>,
        then_type: Box<TypeComputation>,
        else_type: Box<TypeComputation>,
    },
    
    /// Type-level case analysis
    TypeCase {
        scrutinee: Box<TypeComputation>,
        branches: Vec<(TypePattern, TypeComputation)>,
    },
    
    /// Type-level arithmetic (for type-level naturals)
    TypeArithmetic {
        op: TypeArithOp,
        left: Box<TypeComputation>,
        right: Box<TypeComputation>,
    },
    
    /// Type-level natural number
    TypeNat(u64),
    
    /// Type-level boolean
    TypeBool(bool),
    
    /// Type-level list
    TypeList(Vec<TypeComputation>),
    
    /// Type family application
    TypeFamily {
        family: String,
        args: Vec<TypeComputation>,
    },
    
    /// Type-level fixpoint: μα.T
    TypeFixpoint {
        var: String,
        body: Box<TypeComputation>,
    },
}

/// Type-level patterns for case analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypePattern {
    /// Variable pattern (binds type variable)
    Var(String),
    
    /// Constructor pattern
    Constructor {
        name: String,
        args: Vec<TypePattern>,
    },
    
    /// Natural number pattern
    Nat(u64),
    
    /// Boolean pattern
    Bool(bool),
    
    /// List pattern
    List(Vec<TypePattern>),
    
    /// Wildcard pattern
    Wildcard,
}

/// Type-level arithmetic operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Equal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

/// Type family definition.
#[derive(Debug, Clone)]
pub struct TypeFamily {
    /// Family name
    pub name: String,
    
    /// Parameter kinds
    pub params: Vec<(String, Kind)>,
    
    /// Result kind
    pub result_kind: Kind,
    
    /// Family equations (instances)
    pub equations: Vec<TypeFamilyEquation>,
    
    /// Whether this family is injective
    pub injective: bool,
}

/// Type family equation: F(patterns) = result
#[derive(Debug, Clone)]
pub struct TypeFamilyEquation {
    /// Left-hand side patterns
    pub lhs_patterns: Vec<TypePattern>,
    
    /// Right-hand side result
    pub rhs: TypeComputation,
    
    /// Guards (optional conditions)
    pub guards: Vec<TypeComputation>,
}

/// Type-level computation evaluator.
pub struct TypeLevelEvaluator {
    /// Type families
    families: HashMap<String, TypeFamily>,
    
    /// Type variable bindings
    bindings: HashMap<String, TypeComputation>,
    
    /// Evaluation depth (to prevent infinite recursion)
    depth: u32,
    
    /// Maximum evaluation depth
    max_depth: u32,
}

impl TypeLevelEvaluator {
    /// Create a new evaluator.
    pub fn new() -> Self {
        let mut evaluator = Self {
            families: HashMap::new(),
            bindings: HashMap::new(),
            depth: 0,
            max_depth: 1000,
        };
        evaluator.register_builtin_families();
        evaluator
    }
    
    /// Register built-in type families.
    fn register_builtin_families(&mut self) {
        // Register built-in families like Add, Mul, If, etc.
        
        // Type-level addition: Add n m
        let add_family = TypeFamily {
            name: "Add".to_string(),
            params: vec![
                ("n".to_string(), Kind::Type),
                ("m".to_string(), Kind::Type),
            ],
            result_kind: Kind::Type,
            equations: vec![
                // Add 0 m = m
                TypeFamilyEquation {
                    lhs_patterns: vec![
                        TypePattern::Nat(0),
                        TypePattern::Var("m".to_string()),
                    ],
                    rhs: TypeComputation::TypeVar("m".to_string()),
                    guards: vec![],
                },
                // Add (S n) m = S (Add n m)
                // This would be more complex in practice
            ],
            injective: true,
        };
        self.families.insert("Add".to_string(), add_family);
        
        // Type-level conditional: If b t e
        let if_family = TypeFamily {
            name: "If".to_string(),
            params: vec![
                ("b".to_string(), Kind::Type),
                ("t".to_string(), Kind::Type),
                ("e".to_string(), Kind::Type),
            ],
            result_kind: Kind::Type,
            equations: vec![
                // If True t e = t
                TypeFamilyEquation {
                    lhs_patterns: vec![
                        TypePattern::Bool(true),
                        TypePattern::Var("t".to_string()),
                        TypePattern::Var("e".to_string()),
                    ],
                    rhs: TypeComputation::TypeVar("t".to_string()),
                    guards: vec![],
                },
                // If False t e = e
                TypeFamilyEquation {
                    lhs_patterns: vec![
                        TypePattern::Bool(false),
                        TypePattern::Var("t".to_string()),
                        TypePattern::Var("e".to_string()),
                    ],
                    rhs: TypeComputation::TypeVar("e".to_string()),
                    guards: vec![],
                },
            ],
            injective: false,
        };
        self.families.insert("If".to_string(), if_family);
    }
    
    /// Evaluate a type-level computation.
    pub fn evaluate(&mut self, computation: &TypeComputation) -> Result<TypeComputation> {
        if self.depth >= self.max_depth {
            return Err(Box::new(Error::type_error(
                "Type-level computation depth exceeded".to_string(),
                Span::new(0, 0)
            )));
        }
        
        self.depth += 1;
        let result = self.evaluate_inner(computation);
        self.depth -= 1;
        result
    }
    
    /// Internal evaluation implementation.
    fn evaluate_inner(&mut self, computation: &TypeComputation) -> Result<TypeComputation> {
        match computation {
            TypeComputation::TypeVar(name) => {
                if let Some(binding) = self.bindings.get(name).cloned() {
                    self.evaluate(&binding)
                } else {
                    Ok(computation.clone())
                }
            }
            
            TypeComputation::TypeConstant(_) => Ok(computation.clone()),
            
            TypeComputation::TypeLambda { .. } => {
                // Type lambdas are values
                Ok(computation.clone())
            }
            
            TypeComputation::TypeApplication { function, argument } => {
                let eval_func = self.evaluate(function)?;
                let eval_arg = self.evaluate(argument)?;
                self.apply_type_function(&eval_func, &eval_arg)
            }
            
            TypeComputation::TypeConditional { condition, then_type, else_type } => {
                let eval_cond = self.evaluate(condition)?;
                match eval_cond {
                    TypeComputation::TypeBool(true) => self.evaluate(then_type),
                    TypeComputation::TypeBool(false) => self.evaluate(else_type),
                    _ => {
                        // Can't evaluate condition - return unevaluated
                        Ok(TypeComputation::TypeConditional {
                            condition: Box::new(eval_cond),
                            then_type: then_type.clone(),
                            else_type: else_type.clone(),
                        })
                    }
                }
            }
            
            TypeComputation::TypeArithmetic { op, left, right } => {
                let eval_left = self.evaluate(left)?;
                let eval_right = self.evaluate(right)?;
                self.apply_arithmetic_op(*op, &eval_left, &eval_right)
            }
            
            TypeComputation::TypeFamily { family, args } => {
                let eval_args: Result<Vec<TypeComputation>> = args.iter()
                    .map(|arg| self.evaluate(arg))
                    .collect();
                let eval_args = eval_args?;
                self.apply_type_family(family, &eval_args)
            }
            
            TypeComputation::TypeCase { scrutinee, branches } => {
                let eval_scrutinee = self.evaluate(scrutinee)?;
                self.apply_type_case(&eval_scrutinee, branches)
            }
            
            TypeComputation::TypeFixpoint { var, body } => {
                // μα.T - substitute α with (μα.T) in T
                let fixpoint = computation.clone();
                let mut substituted_body = body.clone();
                self.substitute_type_var(&mut substituted_body, var, &fixpoint);
                self.evaluate(&substituted_body)
            }
            
            // Literals evaluate to themselves
            TypeComputation::TypeNat(_) |
            TypeComputation::TypeBool(_) |
            TypeComputation::TypeList(_) => Ok(computation.clone()),
        }
    }
    
    /// Apply a type-level function to an argument.
    fn apply_type_function(
        &mut self,
        function: &TypeComputation,
        argument: &TypeComputation,
    ) -> Result<TypeComputation> {
        match function {
            TypeComputation::TypeLambda { param, body, .. } => {
                // Substitute parameter with argument in body
                let mut substituted_body = (**body).clone();
                self.substitute_type_var(&mut substituted_body, param, argument);
                self.evaluate(&substituted_body)
            }
            _ => {
                // Not a lambda - return application
                Ok(TypeComputation::TypeApplication {
                    function: Box::new(function.clone()),
                    argument: Box::new(argument.clone()),
                })
            }
        }
    }
    
    /// Apply arithmetic operation.
    fn apply_arithmetic_op(
        &mut self,
        op: TypeArithOp,
        left: &TypeComputation,
        right: &TypeComputation,
    ) -> Result<TypeComputation> {
        match (left, right) {
            (TypeComputation::TypeNat(n1), TypeComputation::TypeNat(n2)) => {
                let result = match op {
                    TypeArithOp::Add => n1 + n2,
                    TypeArithOp::Sub => n1.saturating_sub(*n2),
                    TypeArithOp::Mul => n1 * n2,
                    TypeArithOp::Div => {
                        if *n2 == 0 {
                            return Err(Box::new(Error::type_error(
                                "Division by zero in type-level computation".to_string(),
                                Span::new(0, 0)
                            )));
                        }
                        n1 / n2
                    },
                    TypeArithOp::Mod => {
                        if *n2 == 0 {
                            return Err(Box::new(Error::type_error(
                                "Modulo by zero in type-level computation".to_string(),
                                Span::new(0, 0)
                            )));
                        }
                        n1 % n2
                    },
                    TypeArithOp::Equal => return Ok(TypeComputation::TypeBool(n1 == n2)),
                    TypeArithOp::Less => return Ok(TypeComputation::TypeBool(n1 < n2)),
                    TypeArithOp::Greater => return Ok(TypeComputation::TypeBool(n1 > n2)),
                    TypeArithOp::LessEqual => return Ok(TypeComputation::TypeBool(n1 <= n2)),
                    TypeArithOp::GreaterEqual => return Ok(TypeComputation::TypeBool(n1 >= n2)),
                };
                Ok(TypeComputation::TypeNat(result))
            }
            _ => {
                // Can't compute - return unevaluated
                Ok(TypeComputation::TypeArithmetic {
                    op,
                    left: Box::new(left.clone()),
                    right: Box::new(right.clone()),
                })
            }
        }
    }
    
    /// Apply type family.
    fn apply_type_family(
        &mut self,
        family_name: &str,
        args: &[TypeComputation],
    ) -> Result<TypeComputation> {
        if let Some(family) = self.families.get(family_name).cloned() {
            // Try to match against family equations
            for equation in &family.equations {
                if let Some(bindings) = self.match_patterns(&equation.lhs_patterns, args) {
                    // Check guards
                    let guards_satisfied = {
                        let mut all_satisfied = true;
                        for guard in &equation.guards {
                            let mut guard_with_bindings = guard.clone();
                            for (var, val) in &bindings {
                                self.substitute_type_var(&mut guard_with_bindings, var, val);
                            }
                            match self.evaluate(&guard_with_bindings) {
                                Ok(TypeComputation::TypeBool(true)) => {},
                                _ => {
                                    all_satisfied = false;
                                    break;
                                }
                            }
                        }
                        all_satisfied
                    };
                    
                    if guards_satisfied {
                        // Apply bindings to RHS
                        let mut result = equation.rhs.clone();
                        for (var, val) in bindings {
                            self.substitute_type_var(&mut result, &var, &val);
                        }
                        return self.evaluate(&result);
                    }
                }
            }
            
            // No equation matched - return unevaluated
            Ok(TypeComputation::TypeFamily {
                family: family_name.to_string(),
                args: args.to_vec(),
            })
        } else {
            Err(Box::new(Error::type_error(
                format!("Unknown type family: {family_name}"),
                Span::new(0, 0)
            )))
        }
    }
    
    /// Apply type-level case analysis.
    fn apply_type_case(
        &mut self,
        scrutinee: &TypeComputation,
        branches: &[(TypePattern, TypeComputation)],
    ) -> Result<TypeComputation> {
        for (pattern, result) in branches {
            if let Some(bindings) = self.match_pattern(pattern, scrutinee) {
                let mut result_with_bindings = result.clone();
                for (var, val) in bindings {
                    self.substitute_type_var(&mut result_with_bindings, &var, &val);
                }
                return self.evaluate(&result_with_bindings);
            }
        }
        
        // No pattern matched
        Err(Box::new(Error::type_error(
            "No pattern matched in type case".to_string(),
            Span::new(0, 0)
        )))
    }
    
    /// Match patterns against arguments.
    fn match_patterns(
        &self,
        patterns: &[TypePattern],
        args: &[TypeComputation],
    ) -> Option<HashMap<String, TypeComputation>> {
        if patterns.len() != args.len() {
            return None;
        }
        
        let mut bindings = HashMap::new();
        for (pattern, arg) in patterns.iter().zip(args.iter()) {
            if let Some(pattern_bindings) = self.match_pattern(pattern, arg) {
                bindings.extend(pattern_bindings);
            } else {
                return None;
            }
        }
        Some(bindings)
    }
    
    /// Match a single pattern against a computation.
    fn match_pattern(
        &self,
        pattern: &TypePattern,
        computation: &TypeComputation,
    ) -> Option<HashMap<String, TypeComputation>> {
        match (pattern, computation) {
            (TypePattern::Var(name), comp) => {
                let mut bindings = HashMap::new();
                bindings.insert(name.clone(), comp.clone());
                Some(bindings)
            }
            
            (TypePattern::Nat(n1), TypeComputation::TypeNat(n2)) => {
                if n1 == n2 { Some(HashMap::new()) } else { None }
            }
            
            (TypePattern::Bool(b1), TypeComputation::TypeBool(b2)) => {
                if b1 == b2 { Some(HashMap::new()) } else { None }
            }
            
            (TypePattern::Wildcard, _) => Some(HashMap::new()),
            
            (TypePattern::Constructor { name: pname, args: pargs },
             TypeComputation::TypeFamily { family, args }) => {
                if pname == family && pargs.len() == args.len() {
                    self.match_patterns(pargs, args)
                } else {
                    None
                }
            }
            
            _ => None,
        }
    }
    
    /// Substitute type variable in a computation.
    #[allow(clippy::only_used_in_recursion)]
    fn substitute_type_var(
        &self,
        computation: &mut TypeComputation,
        var: &str,
        replacement: &TypeComputation,
    ) {
        match computation {
            TypeComputation::TypeVar(name) if name == var => {
                *computation = replacement.clone();
            }
            
            TypeComputation::TypeLambda { param, body, .. } if param != var => {
                self.substitute_type_var(body, var, replacement);
            }
            
            TypeComputation::TypeApplication { function, argument } => {
                self.substitute_type_var(function, var, replacement);
                self.substitute_type_var(argument, var, replacement);
            }
            
            TypeComputation::TypeConditional { condition, then_type, else_type } => {
                self.substitute_type_var(condition, var, replacement);
                self.substitute_type_var(then_type, var, replacement);
                self.substitute_type_var(else_type, var, replacement);
            }
            
            TypeComputation::TypeArithmetic { left, right, .. } => {
                self.substitute_type_var(left, var, replacement);
                self.substitute_type_var(right, var, replacement);
            }
            
            TypeComputation::TypeFamily { args, .. } => {
                for arg in args {
                    self.substitute_type_var(arg, var, replacement);
                }
            }
            
            _ => {} // Other cases don't contain variables
        }
    }
    
    /// Define a new type family.
    pub fn define_family(&mut self, family: TypeFamily) {
        self.families.insert(family.name.clone(), family);
    }
    
    /// Bind a type variable.
    pub fn bind_type_var(&mut self, name: String, computation: TypeComputation) {
        self.bindings.insert(name, computation);
    }
}

/// Convert between Type and TypeComputation.
impl From<Type> for TypeComputation {
    fn from(type_: Type) -> Self {
        match type_ {
            Type::Number => TypeComputation::TypeConstant("Number".to_string()),
            Type::String => TypeComputation::TypeConstant("String".to_string()),
            Type::Boolean => TypeComputation::TypeConstant("Boolean".to_string()),
            Type::Dynamic => TypeComputation::TypeConstant("Dynamic".to_string()),
            
            Type::Variable(var) => {
                if let Some(name) = var.name {
                    TypeComputation::TypeVar(name)
                } else {
                    TypeComputation::TypeVar(format!("t{}", var.id))
                }
            }
            
            Type::Function { params, return_type } => {
                // Convert to type-level function application
                let mut result = TypeComputation::from(*return_type);
                for param in params.into_iter().rev() {
                    result = TypeComputation::TypeApplication {
                        function: Box::new(TypeComputation::TypeConstant("->".to_string())),
                        argument: Box::new(TypeComputation::TypeList(vec![
                            TypeComputation::from(param),
                            result,
                        ])),
                    };
                }
                result
            }
            
            _ => TypeComputation::TypeConstant("UnknownType".to_string()),
        }
    }
}

impl Default for TypeLevelEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_level_arithmetic() {
        let mut evaluator = TypeLevelEvaluator::new();
        
        let computation = TypeComputation::TypeArithmetic {
            op: TypeArithOp::Add,
            left: Box::new(TypeComputation::TypeNat(2)),
            right: Box::new(TypeComputation::TypeNat(3)),
        };
        
        let result = evaluator.evaluate(&computation).unwrap();
        assert_eq!(result, TypeComputation::TypeNat(5));
    }
    
    #[test]
    fn test_type_level_conditional() {
        let mut evaluator = TypeLevelEvaluator::new();
        
        let computation = TypeComputation::TypeConditional {
            condition: Box::new(TypeComputation::TypeBool(true)),
            then_type: Box::new(TypeComputation::TypeConstant("String".to_string())),
            else_type: Box::new(TypeComputation::TypeConstant("Number".to_string())),
        };
        
        let result = evaluator.evaluate(&computation).unwrap();
        assert_eq!(result, TypeComputation::TypeConstant("String".to_string()));
    }
    
    #[test]
    fn test_type_lambda_application() {
        let mut evaluator = TypeLevelEvaluator::new();
        
        // Λα. α
        let identity = TypeComputation::TypeLambda {
            param: "α".to_string(),
            param_kind: Kind::Type,
            body: Box::new(TypeComputation::TypeVar("α".to_string())),
        };
        
        let application = TypeComputation::TypeApplication {
            function: Box::new(identity),
            argument: Box::new(TypeComputation::TypeConstant("Number".to_string())),
        };
        
        let result = evaluator.evaluate(&application).unwrap();
        assert_eq!(result, TypeComputation::TypeConstant("Number".to_string()));
    }
    
    #[test]
    fn test_type_family_evaluation() {
        let mut evaluator = TypeLevelEvaluator::new();
        
        // If True String Number should evaluate to String
        let computation = TypeComputation::TypeFamily {
            family: "If".to_string(),
            args: vec![
                TypeComputation::TypeBool(true),
                TypeComputation::TypeConstant("String".to_string()),
                TypeComputation::TypeConstant("Number".to_string()),
            ],
        };
        
        let result = evaluator.evaluate(&computation).unwrap();
        assert_eq!(result, TypeComputation::TypeConstant("String".to_string()));
    }
}