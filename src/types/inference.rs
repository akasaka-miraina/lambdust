//! Hindley-Milner type inference for Lambdust.
//!
//! This module implements Algorithm W for type inference, supporting:
//! - Let-polymorphism
//! - Type classes and constraints
//! - Gradual typing integration
//! - Effect inference and tracking

#![allow(missing_docs)]

use super::{
    Type, TypeVar, TypeScheme, TypeEnv, TypeConstraint, 
    ConstraintSolver, Substitution, Effect
};
use crate::ast::{Expr, Literal, Formals};
use crate::diagnostics::{Error, Result, Span, Spanned};
use std::collections::{HashMap, HashSet};

pub type ParameterTypes = (Vec<Type>, Vec<(String, TypeScheme)>);

/// Type inference engine implementing Algorithm W.
#[derive(Debug)]
pub struct TypeInference {
    /// Type environment
    env: TypeEnv,
    /// Accumulated constraints
    constraints: Vec<TypeConstraint>,
    /// Supply of fresh type variables
    var_supply: u64,
    /// Current substitution
    substitution: Substitution,
    /// Effect inference context
    #[allow(dead_code)]
    effect_context: EffectInferenceContext,
}

/// Context for effect inference.
#[derive(Debug, Clone)]
pub struct EffectInferenceContext {
    /// Currently inferred effects for expressions
    expr_effects: HashMap<ExprId, Vec<Effect>>,
    /// Effect variables for inference
    #[allow(dead_code)]
    effect_vars: HashMap<String, EffectVar>,
    /// Effect constraints
    effect_constraints: Vec<EffectConstraint>,
    /// Supply of fresh effect variables
    effect_var_supply: u64,
}

/// Unique identifier for expressions during type inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(u64);

/// Effect variable for effect inference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectVar {
    /// Unique identifier
    id: u64,
    /// Optional name for debugging
    name: Option<String>,
}

/// Constraint on effects during inference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectConstraint {
    /// Effect must be equal to another effect
    Equal(Effect, Effect),
    /// Effect must be a subeffect of another
    Subeffect(Effect, Effect),
    /// Effect must be one of a set of effects
    OneOf(Effect, Vec<Effect>),
    /// Effect must include all given effects
    Includes(Effect, Vec<Effect>),
}

/// Result of type inference.
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Inferred type
    pub type_: Type,
    /// Substitution generated
    pub substitution: Substitution,
    /// Constraints generated
    pub constraints: Vec<TypeConstraint>,
}

impl TypeInference {
    /// Creates a new type inference engine.
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
            constraints: Vec::new(),
            var_supply: 0,
            substitution: Substitution::empty(),
            effect_context: EffectInferenceContext::new(),
        }
    }
    
    /// Creates an inference engine with a given environment.
    pub fn with_env(env: TypeEnv) -> Self {
        Self {
            env,
            constraints: Vec::new(),
            var_supply: 0,
            substitution: Substitution::empty(),
            effect_context: EffectInferenceContext::new(),
        }
    }
    
    /// Infers the type of an expression.
    pub fn infer(&mut self, expr: &Spanned<Expr>) -> Result<InferenceResult> {
        let type_ = self.infer_expr(expr)?;
        
        // Solve accumulated constraints
        let solver = ConstraintSolver::with_constraints(self.constraints.clone());
        let solver_result = solver.solve();
        
        if !solver_result.errors.is_empty() {
            return Err(solver_result.errors.into_iter().next().unwrap());
        }
        
        // Apply final substitution
        let final_type = solver_result.substitution.apply_to_type(&type_);
        let final_substitution = self.substitution.compose(&solver_result.substitution);
        
        Ok(InferenceResult {
            type_: final_type,
            substitution: final_substitution,
            constraints: solver_result.unresolved,
        })
    }
    
    /// Infers the type of an expression.
    fn infer_expr(&mut self, expr: &Spanned<Expr>) -> Result<Type> {
        match &expr.inner {
            // Literals
            Expr::Literal(lit) => self.infer_literal(lit),
            
            // Identifiers
            Expr::Identifier(name) => self.infer_identifier(name, expr.span),
            
            // Keywords
            Expr::Keyword(_) => Ok(Type::Symbol), // Keywords are symbols
            
            // Lambda expressions
            Expr::Lambda { formals, body, metadata } => {
                self.infer_lambda(formals, body, metadata, expr.span)
            }
            
            // Function application
            Expr::Application { operator, operands } => {
                self.infer_application(operator, operands, expr.span)
            }
            
            // Conditional expressions
            Expr::If { test, consequent, alternative } => {
                self.infer_if(test, consequent, alternative.as_deref(), expr.span)
            }
            
            // Variable definition
            Expr::Define { name, value, metadata } => {
                self.infer_define(name, value, metadata, expr.span)
            }
            
            // Assignment
            Expr::Set { name, value } => {
                self.infer_set(name, value, expr.span)
            }
            
            // Type annotation
            Expr::TypeAnnotation { expr: inner_expr, type_expr } => {
                self.infer_type_annotation(inner_expr, type_expr, expr.span)
            }
            
            // Quote
            Expr::Quote(_quoted) => {
                // Quoted expressions have dynamic type in gradual typing
                Ok(Type::Dynamic)
            }
            
            // Pairs
            Expr::Pair { car, cdr } => {
                let car_type = self.infer_expr(car)?;
                let cdr_type = self.infer_expr(cdr)?;
                Ok(Type::pair(car_type, cdr_type))
            }
            
            // Begin (sequence)
            Expr::Begin(exprs) => {
                if exprs.is_empty() {
                    Ok(Type::Unit)
                } else {
                    // Type of sequence is type of last expression
                    for expr in &exprs[..exprs.len() - 1] {
                        self.infer_expr(expr)?; // Infer for side effects
                    }
                    self.infer_expr(&exprs[exprs.len() - 1])
                }
            }
            
            // Let bindings
            Expr::Let { bindings, body } => {
                self.infer_let(bindings, body, expr.span)
            }
            
            Expr::LetStar { bindings, body } => {
                self.infer_let_star(bindings, body, expr.span)
            }
            
            Expr::LetRec { bindings, body } => {
                self.infer_letrec(bindings, body, expr.span)
            }
            
            // Conditional clauses
            Expr::Cond(clauses) => {
                self.infer_cond(clauses, expr.span)
            }
            
            // Logical operations
            Expr::And(exprs) => {
                for expr in exprs {
                    let _expr_type = self.infer_expr(expr)?;
                    // And expressions can have any type
                }
                Ok(Type::Boolean)
            }
            
            Expr::Or(exprs) => {
                for expr in exprs {
                    let _expr_type = self.infer_expr(expr)?;
                    // Or expressions can have any type
                }
                Ok(Type::Boolean)
            }
            
            // Unimplemented
            _ => {
                // For unimplemented forms, return dynamic type
                Ok(Type::Dynamic)
            }
        }
    }
    
    /// Infers the type of a literal.
    fn infer_literal(&mut self, literal: &Literal) -> Result<Type> {
        match literal {
            Literal::Number(_) => Ok(Type::Number),
            Literal::Rational { .. } => Ok(Type::Number),
            Literal::Complex { .. } => Ok(Type::Number),
            Literal::String(_) => Ok(Type::String),
            Literal::Character(_) => Ok(Type::Char),
            Literal::Boolean(_) => Ok(Type::Boolean),
            Literal::Bytevector(_) => Ok(Type::Bytevector),
            Literal::Nil => Ok(Type::list(Type::Dynamic)), // Empty list can be any list type
            Literal::Unspecified => Ok(Type::Unit),
        }
    }
    
    /// Infers the type of an identifier.
    fn infer_identifier(&mut self, name: &str, span: Span) -> Result<Type> {
        if let Some(scheme) = self.env.lookup(name).clone())() {
            // Instantiate the type scheme with fresh variables
            Ok(self.instantiate_scheme(&scheme))
        } else {
            Err(Box::new(Error::type_error(
                format!("Unbound variable: {name}"),
                span,
            ))
        }
    }
    
    /// Infers the type of a lambda expression.
    fn infer_lambda(
        &mut self,
        formals: &Formals,
        body: &[Spanned<Expr>],
        _metadata: &HashMap<String, Spanned<Expr>>,
        span: Span,
    ) -> Result<Type> {
        if body.is_empty() {
            return Err(Box::new(Error::type_error(
                "Lambda body cannot be empty",
                span,
            ));
        }
        
        // Create fresh type variables for parameters
        let (param_types, param_bindings) = self.create_parameter_types(formals)?;
        
        // Extend environment with parameter bindings
        let old_env = self.env.clone());
        for (name, type_scheme) in param_bindings {
            self.env.bind(name, type_scheme);
        }
        
        // Infer body type
        let body_type = self.infer_sequence(body)?;
        
        // Restore environment
        self.env = old_env;
        
        Ok(Type::function(param_types, body_type))
    }
    
    /// Infers the type of a function application.
    fn infer_application(
        &mut self,
        operator: &Spanned<Expr>,
        operands: &[Spanned<Expr>],
        span: Span,
    ) -> Result<Type> {
        // Infer operator type
        let operator_type = self.infer_expr(operator)?;
        
        // Infer operand types
        let mut operand_types = Vec::new();
        for operand in operands {
            operand_types.push(self.infer_expr(operand)?);
        }
        
        // Create fresh type variable for result
        let result_type = self.fresh_type_var();
        
        // Create function type constraint
        let expected_func_type = Type::function(operand_types, result_type.clone());
        
        self.add_constraint(TypeConstraint::equal(
            operator_type,
            expected_func_type,
            Some(span),
            "function application",
        ));
        
        Ok(result_type)
    }
    
    /// Infers the type of an if expression.
    fn infer_if(
        &mut self,
        test: &Spanned<Expr>,
        consequent: &Spanned<Expr>,
        alternative: Option<&Spanned<Expr>>,
        span: Span,
    ) -> Result<Type> {
        // Test expression should be boolean (but we allow any type in gradual typing)
        let _test_type = self.infer_expr(test)?;
        
        // Infer consequent type
        let consequent_type = self.infer_expr(consequent)?;
        
        // Infer alternative type or use unit
        let alternative_type = if let Some(alt) = alternative {
            self.infer_expr(alt)?
        } else {
            Type::Unit
        };
        
        // Consequent and alternative must have the same type
        self.add_constraint(TypeConstraint::equal(
            consequent_type.clone()),
            alternative_type,
            Some(span),
            "if expression branches",
        ));
        
        Ok(consequent_type)
    }
    
    /// Infers the type of a define expression.
    fn infer_define(
        &mut self,
        name: &str,
        value: &Spanned<Expr>,
        _metadata: &HashMap<String, Spanned<Expr>>,
        span: Span,
    ) -> Result<Type> {
        // Check for explicit type annotation
        if let Some(type_expr) = _metadata.get("type") {
            let annotated_type = self.parse_type_expression(type_expr)?;
            let value_type = self.infer_expr(value)?;
            
            // Value must match annotation
            self.add_constraint(TypeConstraint::equal(
                value_type,
                annotated_type.clone()),
                Some(span),
                "type annotation",
            ));
            
            // Generalize and add to environment
            let scheme = self.generalize(&annotated_type);
            self.env.bind(name.to_string(), scheme);
            
            Ok(Type::Unit)
        } else {
            // Infer value type
            let value_type = self.infer_expr(value)?;
            
            // Generalize and add to environment
            let scheme = self.generalize(&value_type);
            self.env.bind(name.to_string(), scheme);
            
            Ok(Type::Unit)
        }
    }
    
    /// Infers the type of a set! expression.
    fn infer_set(
        &mut self,
        name: &str,
        value: &Spanned<Expr>,
        span: Span,
    ) -> Result<Type> {
        // Variable must exist
        let var_type = self.infer_identifier(name, span)?;
        let value_type = self.infer_expr(value)?;
        
        // Value must match variable type
        self.add_constraint(TypeConstraint::equal(
            var_type,
            value_type,
            Some(span),
            "assignment",
        ));
        
        Ok(Type::Unit)
    }
    
    /// Infers the type of a type annotation.
    fn infer_type_annotation(
        &mut self,
        expr: &Spanned<Expr>,
        type_expr: &Spanned<Expr>,
        span: Span,
    ) -> Result<Type> {
        let annotated_type = self.parse_type_expression(type_expr)?;
        let expr_type = self.infer_expr(expr)?;
        
        // Expression must match annotation
        self.add_constraint(TypeConstraint::equal(
            expr_type,
            annotated_type.clone()),
            Some(span),
            "type annotation",
        ));
        
        Ok(annotated_type)
    }
    
    /// Infers the type of a let expression.
    fn infer_let(
        &mut self,
        bindings: &[crate::ast::Binding],
        body: &[Spanned<Expr>],
        _span: Span,
    ) -> Result<Type> {
        let old_env = self.env.clone());
        
        // Infer binding types and extend environment
        for binding in bindings {
            let binding_type = self.infer_expr(&binding.value)?;
            let scheme = self.generalize(&binding_type);
            self.env.bind(binding.name.clone()), scheme);
        }
        
        // Infer body type
        let body_type = self.infer_sequence(body)?;
        
        // Restore environment
        self.env = old_env;
        
        Ok(body_type)
    }
    
    /// Infers the type of a let* expression.
    fn infer_let_star(
        &mut self,
        bindings: &[crate::ast::Binding],
        body: &[Spanned<Expr>],
        _span: Span,
    ) -> Result<Type> {
        let old_env = self.env.clone());
        
        // Process bindings sequentially
        for binding in bindings {
            let binding_type = self.infer_expr(&binding.value)?;
            let scheme = self.generalize(&binding_type);
            self.env.bind(binding.name.clone()), scheme);
        }
        
        // Infer body type
        let body_type = self.infer_sequence(body)?;
        
        // Restore environment
        self.env = old_env;
        
        Ok(body_type)
    }
    
    /// Infers the type of a letrec expression.
    fn infer_letrec(
        &mut self,
        bindings: &[crate::ast::Binding],
        body: &[Spanned<Expr>],
        span: Span,
    ) -> Result<Type> {
        let old_env = self.env.clone());
        
        // Create fresh type variables for all bindings first
        let mut binding_types = Vec::new();
        for binding in bindings {
            let binding_type = self.fresh_type_var();
            binding_types.push(binding_type.clone());
            let scheme = TypeScheme::monomorphic(binding_type);
            self.env.bind(binding.name.clone()), scheme);
        }
        
        // Now infer actual types and add constraints
        for (binding, expected_type) in bindings.iter().zip(binding_types.iter()) {
            let actual_type = self.infer_expr(&binding.value)?;
            self.add_constraint(TypeConstraint::equal(
                actual_type,
                expected_type.clone()),
                Some(span),
                "letrec binding",
            ));
        }
        
        // Infer body type
        let body_type = self.infer_sequence(body)?;
        
        // Restore environment
        self.env = old_env;
        
        Ok(body_type)
    }
    
    /// Infers the type of a cond expression.
    fn infer_cond(
        &mut self,
        clauses: &[crate::ast::CondClause],
        span: Span,
    ) -> Result<Type> {
        if clauses.is_empty() {
            return Ok(Type::Unit);
        }
        
        let mut result_type: Option<Type> = None;
        
        for clause in clauses {
            // Infer test type (can be any type)
            self.infer_expr(&clause.test)?;
            
            // Infer body type
            let clause_type = self.infer_sequence(&clause.body)?;
            
            if let Some(ref expected) = result_type {
                // All clauses must have the same type
                self.add_constraint(TypeConstraint::equal(
                    clause_type,
                    expected.clone()),
                    Some(span),
                    "cond clause",
                ));
            } else {
                result_type = Some(clause_type);
            }
        }
        
        Ok(result_type.unwrap_or(Type::Unit))
    }
    
    /// Infers the type of a sequence of expressions.
    fn infer_sequence(&mut self, exprs: &[Spanned<Expr>]) -> Result<Type> {
        if exprs.is_empty() {
            return Ok(Type::Unit);
        }
        
        // Infer all expressions, return type of last
        for expr in &exprs[..exprs.len() - 1] {
            self.infer_expr(expr)?;
        }
        
        self.infer_expr(&exprs[exprs.len() - 1])
    }
    
    /// Creates fresh type variables for formal parameters.
    fn create_parameter_types(&mut self, formals: &Formals) -> Result<ParameterTypes> {
        let mut param_types = Vec::new();
        let mut bindings = Vec::new();
        
        match formals {
            Formals::Fixed(params) => {
                for param in params {
                    let param_type = self.fresh_type_var();
                    param_types.push(param_type.clone());
                    bindings.push((param.clone()), TypeScheme::monomorphic(param_type)));
                }
            }
            Formals::Variable(param) => {
                // Variable parameters get list type
                let element_type = self.fresh_type_var();
                let list_type = Type::list(element_type);
                bindings.push((param.clone()), TypeScheme::monomorphic(list_type)));
            }
            Formals::Mixed { fixed, rest } => {
                // Fixed parameters
                for param in fixed {
                    let param_type = self.fresh_type_var();
                    param_types.push(param_type.clone());
                    bindings.push((param.clone()), TypeScheme::monomorphic(param_type)));
                }
                
                // Rest parameter gets list type
                let element_type = self.fresh_type_var();
                let list_type = Type::list(element_type);
                bindings.push((rest.clone()), TypeScheme::monomorphic(list_type)));
            }
            Formals::Keyword { fixed, rest, keywords } => {
                // Fixed parameters
                for param in fixed {
                    let param_type = self.fresh_type_var();
                    param_types.push(param_type.clone());
                    bindings.push((param.clone()), TypeScheme::monomorphic(param_type)));
                }
                
                // Keyword parameters
                for kw_param in keywords {
                    let param_type = self.fresh_type_var();
                    bindings.push((kw_param.name.clone()), TypeScheme::monomorphic(param_type)));
                }
                
                // Rest parameter if present
                if let Some(rest) = rest {
                    let element_type = self.fresh_type_var();
                    let list_type = Type::list(element_type);
                    bindings.push((rest.clone()), TypeScheme::monomorphic(list_type)));
                }
            }
        }
        
        Ok((param_types, bindings))
    }
    
    /// Parses a type expression into a Type.
    fn parse_type_expression(&mut self, type_expr: &Spanned<Expr>) -> Result<Type> {
        match &type_expr.inner {
            Expr::Identifier(name) => {
                match name.as_str() {
                    "Number" => Ok(Type::Number),
                    "String" => Ok(Type::String),
                    "Symbol" => Ok(Type::Symbol),
                    "Boolean" => Ok(Type::Boolean),
                    "Char" => Ok(Type::Char),
                    "Dynamic" => Ok(Type::Dynamic),
                    _ => {
                        // Look up type constructor
                        if let Some(constructor) = self.env.constructors.get(name) {
                            Ok(Type::Constructor {
                                name: constructor.name.clone()),
                                kind: constructor.kind.clone()),
                            })
                        } else {
                            // Create type variable
                            Ok(Type::named_var(name))
                        }
                    }
                }
            }
            Expr::Application { operator, operands } => {
                let constructor = self.parse_type_expression(operator)?;
                
                if operands.len() == 1 {
                    let argument = self.parse_type_expression(&operands[0])?;
                    Ok(Type::Application {
                        constructor: Box::new(constructor),
                        argument: Box::new(argument),
                    })
                } else {
                    // Multiple arguments - create nested applications
                    let mut result = constructor;
                    for operand in operands {
                        let argument = self.parse_type_expression(operand)?;
                        result = Type::Application {
                            constructor: Box::new(result),
                            argument: Box::new(argument),
                        };
                    }
                    Ok(result)
                }
            }
            _ => Err(Box::new(Error::type_error(
                "Invalid type expression".to_string(),
                type_expr.span,
            )),
        }
    }
    
    /// Generalizes a type into a type scheme.
    fn generalize(&self, type_: &Type) -> TypeScheme {
        let env_vars = self.env_free_vars();
        let type_vars = type_.free_vars();
        
        // Variables to generalize are those in the type but not in the environment
        let generalized_vars: Vec<_> = type_vars
            .difference(&env_vars)
            .clone())()
            .collect();
        
        TypeScheme::polymorphic(generalized_vars, Vec::new(), type_.clone())
    }
    
    /// Instantiates a type scheme with fresh type variables.
    fn instantiate_scheme(&mut self, scheme: &TypeScheme) -> Type {
        if scheme.vars.is_empty() {
            return scheme.type_.clone());
        }
        
        // Create fresh variables for each quantified variable
        let fresh_mapping: HashMap<TypeVar, Type> = scheme.vars
            .iter()
            .map(|var| (var.clone()), self.fresh_type_var()))
            .collect();
        
        // Apply substitution
        let subst = Substitution::from_mappings(fresh_mapping.into_iter().collect());
        subst.apply_to_type(&scheme.type_)
    }
    
    /// Gets all free type variables in the environment.
    fn env_free_vars(&self) -> HashSet<TypeVar> {
        let mut vars = HashSet::new();
        for scheme in self.env.bindings.values() {
            // Only count free variables (not quantified ones)
            let mut type_vars = scheme.type_.free_vars();
            for quantified in &scheme.vars {
                type_vars.remove(quantified);
            }
            vars.extend(type_vars);
        }
        vars
    }
    
    /// Creates a fresh type variable.
    fn fresh_type_var(&mut self) -> Type {
        let var = TypeVar {
            id: self.var_supply,
            name: None,
        };
        self.var_supply += 1;
        Type::Variable(var)
    }
    
    /// Adds a constraint.
    fn add_constraint(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
    }
}

impl EffectInferenceContext {
    /// Creates a new effect inference context.
    pub fn new() -> Self {
        Self {
            expr_effects: HashMap::new(),
            effect_vars: HashMap::new(),
            effect_constraints: Vec::new(),
            effect_var_supply: 0,
        }
    }
    
    /// Creates a fresh effect variable.
    pub fn fresh_effect_var(&mut self, name: Option<String>) -> EffectVar {
        let id = self.effect_var_supply;
        self.effect_var_supply += 1;
        EffectVar { id, name }
    }
    
    /// Records the effects for an expression.
    pub fn record_effects(&mut self, expr_id: ExprId, effects: Vec<Effect>) {
        self.expr_effects.insert(expr_id, effects);
    }
    
    /// Gets the effects for an expression.
    pub fn get_effects(&self, expr_id: ExprId) -> Option<&Vec<Effect>> {
        self.expr_effects.get(&expr_id)
    }
    
    /// Adds an effect constraint.
    pub fn add_constraint(&mut self, constraint: EffectConstraint) {
        self.effect_constraints.push(constraint);
    }
    
    /// Gets all effect constraints.
    pub fn constraints(&self) -> &[EffectConstraint] {
        &self.effect_constraints
    }
}

impl EffectVar {
    /// Creates a new effect variable.
    pub fn new(id: u64) -> Self {
        Self { id, name: None }
    }
    
    /// Creates a new effect variable with a name.
    pub fn with_name(id: u64, name: String) -> Self {
        Self { id, name: Some(name) }
    }
}

impl ExprId {
    /// Creates a new expression ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for EffectInferenceContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Literal, Expr};
    use crate::diagnostics::{Span, spanned};

    #[test]
    fn test_infer_literal() {
        let mut inference = TypeInference::new();
        
        let number_expr = spanned(Expr::Literal(Literal::Number(42.0)), Span::new(0, 2));
        let result = inference.infer(&number_expr).unwrap();
        assert_eq!(result.type_, Type::Number);
        
        let string_expr = spanned(Expr::Literal(Literal::String("hello".to_string())), Span::new(0, 7));
        let result = inference.infer(&string_expr).unwrap();
        assert_eq!(result.type_, Type::String);
    }

    #[test]
    fn test_infer_lambda() {
        let mut inference = TypeInference::new();
        
        // (lambda (x) x) - identity function
        let lambda_expr = spanned(
            Expr::Lambda {
                formals: Formals::Fixed(vec!["x".to_string()]),
                metadata: HashMap::new(),
                body: vec![spanned(Expr::Identifier("x".to_string()), Span::new(11, 1))],
            },
            Span::new(0, 13)
        );
        
        let result = inference.infer(&lambda_expr).unwrap();
        // Should be a function type with same input and output type
        match result.type_ {
            Type::Function { params, return_type: _ } => {
                assert_eq!(params.len(), 1);
                // In a proper implementation, this would be unified as t -> t
            }
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_infer_application() {
        let mut inference = TypeInference::new();
        
        // Add identity function to environment
        let _id_type = Type::forall(
            vec![TypeVar::with_name("a")],
            Type::function(vec![Type::named_var("a")], Type::named_var("a"))
        );
        inference.env.bind("id".to_string(), TypeScheme::polymorphic(
            vec![TypeVar::with_name("a")],
            Vec::new(),
            Type::function(vec![Type::named_var("a")], Type::named_var("a"))
        ));
        
        // (id 42)
        let app_expr = spanned(
            Expr::Application {
                operator: Box::new(spanned(Expr::Identifier("id".to_string()), Span::new(1, 2))),
                operands: vec![spanned(Expr::Literal(Literal::Number(42.0)), Span::new(4, 2))],
            },
            Span::new(0, 7)
        );
        
        let _result = inference.infer(&app_expr).unwrap();
        // Should infer Number type for the result
        // Note: This requires constraint solving to work properly
    }
}