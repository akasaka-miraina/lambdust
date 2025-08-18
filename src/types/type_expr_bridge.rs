//! Bridge between parsed type expressions (TypeExpr) and semantic types (Type).
//!
//! This module provides conversion functionality from AST type expressions
//! to the semantic type system representations used by the type checker.

use crate::ast::{TypeExpr, TypeConstraint as AstConstraint, VariantCase};
use crate::diagnostics::{Error, Result, Spanned};
use crate::types::{Type, TypeVar, Constraint, Row, Effect, Kind};
use std::collections::HashMap;

/// Context for type expression evaluation, tracking type variables and constraints.
#[derive(Debug, Clone)]
pub struct TypeExprContext {
    /// Type variable bindings
    type_vars: HashMap<String, TypeVar>,
    /// Next type variable ID
    next_var_id: usize,
}

impl TypeExprContext {
    /// Creates a new empty context.
    pub fn new() -> Self {
        Self {
            type_vars: HashMap::new(),
            next_var_id: 0,
        }
    }
    
    /// Creates a type variable for the given name, reusing existing ones.
    pub fn get_or_create_type_var(&mut self, name: &str) -> TypeVar {
        if let Some(var) = self.type_vars.get(name) {
            var.clone()
        } else {
            let var = TypeVar::with_name(name);
            self.type_vars.insert(name.to_string(), var.clone());
            var
        }
    }
    
    /// Creates a fresh anonymous type variable.
    pub fn fresh_type_var(&mut self) -> TypeVar {
        let var = TypeVar::new();
        self.next_var_id += 1;
        var
    }
    
    /// Enters a new scope for type variables (for forall/exists binding).
    pub fn enter_scope(&self) -> Self {
        Self {
            type_vars: self.type_vars.clone(),
            next_var_id: self.next_var_id,
        }
    }
    
    /// Bind type variables in the current scope.
    pub fn bind_vars(&mut self, names: &[String]) -> Vec<TypeVar> {
        names.iter()
            .map(|name| self.get_or_create_type_var(name))
            .collect()
    }
}

impl Default for TypeExprContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluates a type expression into a semantic type.
pub fn evaluate_type_expr(
    type_expr: &Spanned<TypeExpr>,
    context: &mut TypeExprContext,
) -> Result<Type> {
    evaluate_type_expr_inner(&type_expr.inner, context)
}

/// Internal implementation of type expression evaluation.
fn evaluate_type_expr_inner(
    type_expr: &TypeExpr,
    context: &mut TypeExprContext,
) -> Result<Type> {
    match type_expr {
        TypeExpr::Identifier(name) => {
            // Map common type names to built-in types
            match name.as_str() {
                "Number" | "Integer" | "Real" | "Complex" | "Rational" => Ok(Type::Number),
                "String" => Ok(Type::String),
                "Symbol" => Ok(Type::Symbol),
                "Boolean" | "Bool" => Ok(Type::Boolean),
                "Char" | "Character" => Ok(Type::Char),
                "Bytevector" => Ok(Type::Bytevector),
                "Unit" | "()" => Ok(Type::Unit),
                "Dynamic" | "*" => Ok(Type::Dynamic),
                "Unknown" | "?" => Ok(Type::Unknown),
                _ => {
                    // Unknown identifier - treat as type constructor with kind *
                    Ok(Type::Constructor {
                        name: name.clone(),
                        kind: Kind::Type,
                    })
                }
            }
        }
        
        TypeExpr::Variable(name) => {
            let var = context.get_or_create_type_var(name);
            Ok(Type::Variable(var))
        }
        
        TypeExpr::Function { params, return_type } => {
            let param_types = params.iter()
                .map(|param| evaluate_type_expr(param, context))
                .collect::<Result<Vec<_>>>()?;
            let ret_type = evaluate_type_expr(return_type, context)?;
            
            Ok(Type::Function {
                params: param_types,
                return_type: Box::new(ret_type),
            })
        }
        
        TypeExpr::Pair { first, second } => {
            let first_type = evaluate_type_expr(first, context)?;
            let second_type = evaluate_type_expr(second, context)?;
            Ok(Type::Pair(Box::new(first_type), Box::new(second_type)))
        }
        
        TypeExpr::List { element_type } => {
            let elem_type = evaluate_type_expr(element_type, context)?;
            Ok(Type::List(Box::new(elem_type)))
        }
        
        TypeExpr::Vector { element_type } => {
            let elem_type = evaluate_type_expr(element_type, context)?;
            Ok(Type::Vector(Box::new(elem_type)))
        }
        
        TypeExpr::Forall { vars, body } => {
            let mut new_context = context.enter_scope();
            let type_vars = new_context.bind_vars(vars);
            let body_type = evaluate_type_expr(body, &mut new_context)?;
            
            Ok(Type::Forall {
                vars: type_vars,
                body: Box::new(body_type),
            })
        }
        
        TypeExpr::Exists { vars, body } => {
            let mut new_context = context.enter_scope();
            let type_vars = new_context.bind_vars(vars);
            let body_type = evaluate_type_expr(body, &mut new_context)?;
            
            Ok(Type::Exists {
                vars: type_vars,
                body: Box::new(body_type),
            })
        }
        
        TypeExpr::Application { constructor, argument } => {
            let constructor_type = evaluate_type_expr(constructor, context)?;
            let argument_type = evaluate_type_expr(argument, context)?;
            
            Ok(Type::Application {
                constructor: Box::new(constructor_type),
                argument: Box::new(argument_type),
            })
        }
        
        TypeExpr::Parametric { name, args } => {
            // Handle common parametric types
            match (name.as_str(), args.len()) {
                ("List", 1) => {
                    let elem_type = evaluate_type_expr(&args[0], context)?;
                    Ok(Type::List(Box::new(elem_type)))
                }
                ("Vector", 1) => {
                    let elem_type = evaluate_type_expr(&args[0], context)?;
                    Ok(Type::Vector(Box::new(elem_type)))
                }
                ("Pair", 2) => {
                    let first_type = evaluate_type_expr(&args[0], context)?;
                    let second_type = evaluate_type_expr(&args[1], context)?;
                    Ok(Type::Pair(Box::new(first_type), Box::new(second_type)))
                }
                _ => {
                    // Generic parametric type - build nested applications
                    let constructor = Type::Constructor {
                        name: name.clone(),
                        kind: Kind::Type, // Simplified - should compute proper kind
                    };
                    
                    args.iter().try_fold(constructor, |acc, arg| {
                        let arg_type = evaluate_type_expr(arg, context)?;
                        Ok(Type::Application {
                            constructor: Box::new(acc),
                            argument: Box::new(arg_type),
                        })
                    })
                }
            }
        }
        
        TypeExpr::Constrained { constraints, type_expr } => {
            let base_type = evaluate_type_expr(type_expr, context)?;
            let evaluated_constraints = constraints.iter()
                .map(|c| evaluate_constraint(c, context))
                .collect::<Result<Vec<_>>>()?;
            
            Ok(Type::Constrained {
                constraints: evaluated_constraints,
                type_: Box::new(base_type),
            })
        }
        
        TypeExpr::Record { fields, rest } => {
            let mut field_map = HashMap::new();
            
            for (name, type_expr) in fields {
                let field_type = evaluate_type_expr(type_expr, context)?;
                field_map.insert(name.clone(), field_type);
            }
            
            let rest_var = rest.as_ref().map(|name| context.get_or_create_type_var(name));
            
            Ok(Type::Record(Row {
                fields: field_map,
                rest: rest_var,
            }))
        }
        
        TypeExpr::Variant { cases } => {
            let mut field_map = HashMap::new();
            
            for case in cases {
                let case_type = if let Some(payload) = &case.payload {
                    evaluate_type_expr(payload, context)?
                } else {
                    Type::Unit
                };
                field_map.insert(case.constructor.clone(), case_type);
            }
            
            Ok(Type::Variant(Row {
                fields: field_map,
                rest: None,
            }))
        }
        
        TypeExpr::Recursive { var, body } => {
            let type_var = context.get_or_create_type_var(var);
            let body_type = evaluate_type_expr(body, context)?;
            
            Ok(Type::Recursive {
                var: type_var,
                body: Box::new(body_type),
            })
        }
        
        TypeExpr::Effectful { input, effects, output } => {
            let input_type = evaluate_type_expr(input, context)?;
            let output_type = evaluate_type_expr(output, context)?;
            
            let effect_types = effects.iter()
                .map(|effect_name| match effect_name.as_str() {
                    "IO" => Effect::IO,
                    "Error" => Effect::Error,
                    "Pure" => Effect::Pure,
                    _ => Effect::Custom(effect_name.clone()),
                })
                .collect();
            
            Ok(Type::Effectful {
                input: Box::new(input_type),
                effects: effect_types,
                output: Box::new(output_type),
            })
        }
        
        TypeExpr::Dynamic => Ok(Type::Dynamic),
        TypeExpr::Unknown => Ok(Type::Unknown),
        
        TypeExpr::Parenthesized(inner) => {
            evaluate_type_expr(inner, context)
        }
        
        TypeExpr::Kinded { type_expr, kind: _ } => {
            // For now, ignore kind annotations and just evaluate the type
            // TODO: Implement proper kind checking
            evaluate_type_expr(type_expr, context)
        }
    }
}

/// Evaluates a type constraint.
fn evaluate_constraint(
    constraint: &AstConstraint,
    context: &mut TypeExprContext,
) -> Result<Constraint> {
    let type_ = evaluate_type_expr(&constraint.type_expr, context)?;
    Ok(Constraint {
        class: constraint.class.clone(),
        type_,
    })
}

/// Convenience function to evaluate a type expression with a fresh context.
pub fn evaluate_type_expr_simple(type_expr: &Spanned<TypeExpr>) -> Result<Type> {
    let mut context = TypeExprContext::new();
    evaluate_type_expr(type_expr, &mut context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::TypeExpr;
    use crate::diagnostics::Span;

    fn dummy_span() -> Span {
        Span::new(0, 1)
    }

    fn spanned_type(expr: TypeExpr) -> Spanned<TypeExpr> {
        Spanned::new(expr, dummy_span())
    }

    #[test]
    fn test_basic_type_evaluation() {
        let int_expr = spanned_type(TypeExpr::Identifier("Integer".to_string()));
        let result = evaluate_type_expr_simple(&int_expr).unwrap();
        assert!(matches!(result, Type::Number));
        
        let string_expr = spanned_type(TypeExpr::Identifier("String".to_string()));
        let result = evaluate_type_expr_simple(&string_expr).unwrap();
        assert!(matches!(result, Type::String));
    }

    #[test]
    fn test_type_variable_evaluation() {
        let var_expr = spanned_type(TypeExpr::Variable("a".to_string()));
        let result = evaluate_type_expr_simple(&var_expr).unwrap();
        assert!(matches!(result, Type::Variable(_)));
    }

    #[test]
    fn test_function_type_evaluation() {
        let int_type = spanned_type(TypeExpr::Identifier("Integer".to_string()));
        let string_type = spanned_type(TypeExpr::Identifier("String".to_string()));
        let func_expr = spanned_type(TypeExpr::Function {
            params: vec![int_type],
            return_type: Box::new(string_type),
        });
        
        let result = evaluate_type_expr_simple(&func_expr).unwrap();
        assert!(matches!(result, Type::Function { .. }));
    }

    #[test]
    fn test_list_type_evaluation() {
        let int_type = spanned_type(TypeExpr::Identifier("Integer".to_string()));
        let list_expr = spanned_type(TypeExpr::List {
            element_type: Box::new(int_type),
        });
        
        let result = evaluate_type_expr_simple(&list_expr).unwrap();
        assert!(matches!(result, Type::List(_)));
    }

    #[test]
    fn test_parametric_type_evaluation() {
        let int_type = spanned_type(TypeExpr::Identifier("Integer".to_string()));
        let list_expr = spanned_type(TypeExpr::Parametric {
            name: "List".to_string(),
            args: vec![int_type],
        });
        
        let result = evaluate_type_expr_simple(&list_expr).unwrap();
        assert!(matches!(result, Type::List(_)));
    }

    #[test]
    fn test_forall_type_evaluation() {
        let var_a = spanned_type(TypeExpr::Variable("a".to_string()));
        let forall_expr = spanned_type(TypeExpr::Forall {
            vars: vec!["a".to_string()],
            body: Box::new(var_a),
        });
        
        let result = evaluate_type_expr_simple(&forall_expr).unwrap();
        assert!(matches!(result, Type::Forall { .. }));
    }
}