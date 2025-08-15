//! Integration of dependent types with Scheme syntax and semantics.
//!
//! This module provides:
//! - S-expression syntax for dependent types
//! - R7RS compatibility layer
//! - Smooth migration from dynamic to dependent typing
//! - Scheme-friendly type annotation syntax
//! - Integration with existing Scheme primitives

use super::{Type, TypeVar, dependent_bridge::*, type_level_computation::*};
use crate::diagnostics::{Error, Result, Span};
use crate::ast::{Program, Literal};
use crate::eval::Value;
use crate::utils::SymbolId;
use std::collections::HashMap;
use std::fmt;

/// Scheme-syntax dependent type annotations.
///
/// These provide a Scheme-friendly way to write dependent types
/// using familiar S-expression syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemeDependentType {
    /// (Pi (x A) B) - dependent function type
    Pi {
        var: String,
        domain: Box<SchemeDependentType>,
        codomain: Box<SchemeDependentType>,
    },
    
    /// (Sigma (x A) B) - dependent pair type
    Sigma {
        var: String,
        first: Box<SchemeDependentType>,
        second: Box<SchemeDependentType>,
    },
    
    /// (Vec n A) - vector of length n with elements of type A
    Vector {
        length: SchemeTypeTerm,
        element_type: Box<SchemeDependentType>,
    },
    
    /// (List A) - homogeneous list
    List(Box<SchemeDependentType>),
    
    /// (Refinement (x A) P) - refinement type {x : A | P(x)}
    Refinement {
        var: String,
        base_type: Box<SchemeDependentType>,
        predicate: SchemeTypeTerm,
    },
    
    /// (Indexed family args...) - indexed type family application
    Indexed {
        family: String,
        args: Vec<SchemeTypeTerm>,
    },
    
    /// Base types
    Number,
    String,
    Symbol,
    Boolean,
    Char,
    
    /// Type variables
    Var(String),
    
    /// Dynamic type
    Dynamic,
}

/// Scheme-syntax term expressions that can appear in types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemeTypeTerm {
    /// Variable
    Var(String),
    
    /// Number literal
    Number(i64),
    
    /// Boolean literal
    Bool(bool),
    
    /// String literal
    String(String),
    
    /// Function application (f arg1 arg2 ...)
    App {
        function: Box<SchemeTypeTerm>,
        args: Vec<SchemeTypeTerm>,
    },
    
    /// Lambda expression (lambda (x) body)
    Lambda {
        params: Vec<String>,
        body: Box<SchemeTypeTerm>,
    },
    
    /// Conditional (if cond then else)
    If {
        condition: Box<SchemeTypeTerm>,
        then_branch: Box<SchemeTypeTerm>,
        else_branch: Box<SchemeTypeTerm>,
    },
    
    /// Length function (length lst)
    Length(Box<SchemeTypeTerm>),
    
    /// Arithmetic operations
    Add(Box<SchemeTypeTerm>, Box<SchemeTypeTerm>),
    Sub(Box<SchemeTypeTerm>, Box<SchemeTypeTerm>),
    Mul(Box<SchemeTypeTerm>, Box<SchemeTypeTerm>),
    
    /// Comparisons
    Equal(Box<SchemeTypeTerm>, Box<SchemeTypeTerm>),
    Less(Box<SchemeTypeTerm>, Box<SchemeTypeTerm>),
    Greater(Box<SchemeTypeTerm>, Box<SchemeTypeTerm>),
}

/// Parser for Scheme-syntax dependent types.
pub struct SchemeDependentParser {
    /// Current parsing context
    context: HashMap<String, SchemeDependentType>,
}

impl SchemeDependentParser {
    /// Create a new parser.
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
        }
    }
    
    /// Parse a dependent type from S-expression representation.
    ///
    /// Examples:
    /// - `Number` -> base type
    /// - `(Pi (n Number) (Vec n Number))` -> dependent function
    /// - `(Refinement (x Number) (> x 0))` -> positive numbers
    /// - `(Vec (length xs) Number)` -> vector with length depending on xs
    pub fn parse_type(&mut self, expr: &Value) -> Result<SchemeDependentType> {
        match expr {
            // Base types as symbols
            Value::Symbol(sym) => match sym.to_string().as_str() {
                "Number" => Ok(SchemeDependentType::Number),
                "String" => Ok(SchemeDependentType::String),
                "Symbol" => Ok(SchemeDependentType::Symbol),
                "Boolean" => Ok(SchemeDependentType::Boolean),
                "Char" => Ok(SchemeDependentType::Char),
                "Dynamic" => Ok(SchemeDependentType::Dynamic),
                name => Ok(SchemeDependentType::Var(name.to_string())),
            },
            
            // Compound types as lists
            Value::Pair(head, tail) => {
                if let Value::Symbol(constructor) = head.as_ref() {
                    let args = self.collect_list_elements(tail)?;
                    self.parse_compound_type(&constructor.to_string(), &args)
                } else {
                    Err(Box::new(Error::type_error(
                        "Expected type constructor symbol".to_string(),
                        Span::new(0, 0)
                    )))
                }
            }
            
            _ => Err(Box::new(Error::type_error(
                format!("Cannot parse type from: {expr:?}"),
                Span::new(0, 0)
            ))),
        }
    }
    
    /// Parse compound type constructions.
    fn parse_compound_type(
        &mut self,
        constructor: &str,
        args: &[Value],
    ) -> Result<SchemeDependentType> {
        match constructor {
            "Pi" => {
                if args.len() != 2 {
                    return Err(Box::new(Error::type_error(
                        "Pi type requires 2 arguments: (Pi (var domain) codomain)".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let (var, domain) = self.parse_binding(&args[0])?;
                let codomain = self.parse_type(&args[1])?;
                
                Ok(SchemeDependentType::Pi {
                    var,
                    domain: Box::new(domain),
                    codomain: Box::new(codomain),
                })
            }
            
            "Sigma" => {
                if args.len() != 2 {
                    return Err(Box::new(Error::type_error(
                        "Sigma type requires 2 arguments: (Sigma (var first) second)".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let (var, first) = self.parse_binding(&args[0])?;
                let second = self.parse_type(&args[1])?;
                
                Ok(SchemeDependentType::Sigma {
                    var,
                    first: Box::new(first),
                    second: Box::new(second),
                })
            }
            
            "Vec" => {
                if args.len() != 2 {
                    return Err(Box::new(Error::type_error(
                        "Vec type requires 2 arguments: (Vec length element-type)".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let length = self.parse_term(&args[0])?;
                let element_type = self.parse_type(&args[1])?;
                
                Ok(SchemeDependentType::Vector {
                    length,
                    element_type: Box::new(element_type),
                })
            }
            
            "List" => {
                if args.len() != 1 {
                    return Err(Box::new(Error::type_error(
                        "List type requires 1 argument: (List element-type)".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let element_type = self.parse_type(&args[0])?;
                Ok(SchemeDependentType::List(Box::new(element_type)))
            }
            
            "Refinement" => {
                if args.len() != 2 {
                    return Err(Box::new(Error::type_error(
                        "Refinement type requires 2 arguments: (Refinement (var base-type) predicate)".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let (var, base_type) = self.parse_binding(&args[0])?;
                let predicate = self.parse_term(&args[1])?;
                
                Ok(SchemeDependentType::Refinement {
                    var,
                    base_type: Box::new(base_type),
                    predicate,
                })
            }
            
            // Type family application
            family_name => {
                let parsed_args = args.iter()
                    .map(|arg| self.parse_term(arg))
                    .collect::<Result<Vec<_>>>()?;
                
                Ok(SchemeDependentType::Indexed {
                    family: family_name.to_string(),
                    args: parsed_args,
                })
            }
        }
    }
    
    /// Parse a variable binding (var type).
    fn parse_binding(&mut self, expr: &Value) -> Result<(String, SchemeDependentType)> {
        match expr {
            Value::Pair(var_val, type_val) => {
                if let Value::Symbol(var_name) = var_val.as_ref() {
                    if let Value::Pair(type_head, _) = type_val.as_ref() {
                        let type_ = self.parse_type(type_val)?;
                        Ok((var_name.to_string(), type_))
                    } else if let Value::Symbol(_) = type_val.as_ref() {
                        let type_ = self.parse_type(type_val)?;
                        Ok((var_name.to_string(), type_))
                    } else {
                        Err(Box::new(Error::type_error(
                            "Expected type in binding".to_string(),
                            Span::new(0, 0)
                        )))
                    }
                } else {
                    Err(Box::new(Error::type_error(
                        "Expected variable name in binding".to_string(),
                        Span::new(0, 0)
                    )))
                }
            }
            _ => Err(Box::new(Error::type_error(
                "Expected pair for binding".to_string(),
                Span::new(0, 0)
            ))),
        }
    }
    
    /// Parse a term expression.
    fn parse_term(&mut self, expr: &Value) -> Result<SchemeTypeTerm> {
        match expr {
            Value::Symbol(name) => Ok(SchemeTypeTerm::Var(name.to_string())),
            
            Value::Literal(Literal::Number(n)) => Ok(SchemeTypeTerm::Number(*n as i64)),
            
            Value::Literal(Literal::Boolean(b)) => Ok(SchemeTypeTerm::Bool(*b)),
            
            Value::Literal(Literal::String(s)) => Ok(SchemeTypeTerm::String(s.clone())),
            
            Value::Pair(head, tail) => {
                if let Value::Symbol(op) = head.as_ref() {
                    let args = self.collect_list_elements(tail)?;
                    self.parse_term_application(&op.to_string(), &args)
                } else {
                    Err(Box::new(Error::type_error(
                        "Expected operator symbol in term".to_string(),
                        Span::new(0, 0)
                    )))
                }
            }
            
            _ => Err(Box::new(Error::type_error(
                format!("Cannot parse term from: {expr:?}"),
                Span::new(0, 0)
            ))),
        }
    }
    
    /// Parse term applications.
    fn parse_term_application(
        &mut self,
        op: &str,
        args: &[Value],
    ) -> Result<SchemeTypeTerm> {
        match op {
            "lambda" => {
                if args.len() != 2 {
                    return Err(Box::new(Error::type_error(
                        "lambda requires 2 arguments: (lambda (params...) body)".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let params = self.parse_parameter_list(&args[0])?;
                let body = self.parse_term(&args[1])?;
                
                Ok(SchemeTypeTerm::Lambda {
                    params,
                    body: Box::new(body),
                })
            }
            
            "if" => {
                if args.len() != 3 {
                    return Err(Box::new(Error::type_error(
                        "if requires 3 arguments: (if condition then else)".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let condition = self.parse_term(&args[0])?;
                let then_branch = self.parse_term(&args[1])?;
                let else_branch = self.parse_term(&args[2])?;
                
                Ok(SchemeTypeTerm::If {
                    condition: Box::new(condition),
                    then_branch: Box::new(then_branch),
                    else_branch: Box::new(else_branch),
                })
            }
            
            "length" => {
                if args.len() != 1 {
                    return Err(Box::new(Error::type_error(
                        "length requires 1 argument".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let arg = self.parse_term(&args[0])?;
                Ok(SchemeTypeTerm::Length(Box::new(arg)))
            }
            
            "+" => {
                if args.len() != 2 {
                    return Err(Box::new(Error::type_error(
                        "+ requires 2 arguments".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let left = self.parse_term(&args[0])?;
                let right = self.parse_term(&args[1])?;
                Ok(SchemeTypeTerm::Add(Box::new(left), Box::new(right)))
            }
            
            "-" => {
                if args.len() != 2 {
                    return Err(Box::new(Error::type_error(
                        "- requires 2 arguments".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let left = self.parse_term(&args[0])?;
                let right = self.parse_term(&args[1])?;
                Ok(SchemeTypeTerm::Sub(Box::new(left), Box::new(right)))
            }
            
            "*" => {
                if args.len() != 2 {
                    return Err(Box::new(Error::type_error(
                        "* requires 2 arguments".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let left = self.parse_term(&args[0])?;
                let right = self.parse_term(&args[1])?;
                Ok(SchemeTypeTerm::Mul(Box::new(left), Box::new(right)))
            }
            
            "=" => {
                if args.len() != 2 {
                    return Err(Box::new(Error::type_error(
                        "= requires 2 arguments".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let left = self.parse_term(&args[0])?;
                let right = self.parse_term(&args[1])?;
                Ok(SchemeTypeTerm::Equal(Box::new(left), Box::new(right)))
            }
            
            "<" => {
                if args.len() != 2 {
                    return Err(Box::new(Error::type_error(
                        "< requires 2 arguments".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let left = self.parse_term(&args[0])?;
                let right = self.parse_term(&args[1])?;
                Ok(SchemeTypeTerm::Less(Box::new(left), Box::new(right)))
            }
            
            ">" => {
                if args.len() != 2 {
                    return Err(Box::new(Error::type_error(
                        "> requires 2 arguments".to_string(),
                        Span::new(0, 0)
                    )));
                }
                
                let left = self.parse_term(&args[0])?;
                let right = self.parse_term(&args[1])?;
                Ok(SchemeTypeTerm::Greater(Box::new(left), Box::new(right)))
            }
            
            // General function application
            _ => {
                let function = SchemeTypeTerm::Var(op.to_string());
                let parsed_args = args.iter()
                    .map(|arg| self.parse_term(arg))
                    .collect::<Result<Vec<_>>>()?;
                
                Ok(SchemeTypeTerm::App {
                    function: Box::new(function),
                    args: parsed_args,
                })
            }
        }
    }
    
    /// Parse parameter list for lambda.
    fn parse_parameter_list(&self, expr: &Value) -> Result<Vec<String>> {
        let elements = self.collect_list_elements(expr)?;
        let mut params = Vec::new();
        
        for element in elements {
            if let Value::Symbol(param_name) = element {
                params.push(param_name.to_string());
            } else {
                return Err(Box::new(Error::type_error(
                    "Expected symbol in parameter list".to_string(),
                    Span::new(0, 0)
                )));
            }
        }
        
        Ok(params)
    }
    
    /// Collect elements from a Scheme list.
    fn collect_list_elements(&self, expr: &Value) -> Result<Vec<Value>> {
        let mut elements = Vec::new();
        let mut current = expr;
        
        loop {
            match current {
                Value::Nil => break,
                Value::Pair(head, tail) => {
                    elements.push((**head).clone());
                    current = tail;
                }
                _ => {
                    // Improper list - treat last element as final
                    elements.push(current.clone());
                    break;
                }
            }
        }
        
        Ok(elements)
    }
}

/// Convert Scheme-syntax dependent types to internal representation.
impl From<SchemeDependentType> for Type {
    fn from(scheme_type: SchemeDependentType) -> Self {
        match scheme_type {
            SchemeDependentType::Number => Type::Number,
            SchemeDependentType::String => Type::String,
            SchemeDependentType::Symbol => Type::Symbol,
            SchemeDependentType::Boolean => Type::Boolean,
            SchemeDependentType::Char => Type::Char,
            SchemeDependentType::Dynamic => Type::Dynamic,
            SchemeDependentType::Var(name) => Type::Variable(TypeVar::with_name(name)),
            
            SchemeDependentType::Pi { domain, codomain, .. } => {
                Type::function(vec![(*domain).into()], (*codomain).into())
            }
            
            SchemeDependentType::Sigma { first, .. } => {
                // Simplified - would need more sophisticated conversion
                Type::pair((*first).into(), Type::Dynamic)
            }
            
            SchemeDependentType::Vector { element_type, .. } => {
                Type::vector((*element_type).into())
            }
            
            SchemeDependentType::List(element_type) => {
                Type::list((*element_type).into())
            }
            
            SchemeDependentType::Refinement { base_type, .. } => {
                // For now, just use base type
                (*base_type).into()
            }
            
            SchemeDependentType::Indexed { .. } => {
                // Would need type family resolution
                Type::Dynamic
            }
        }
    }
}

/// R7RS compatibility layer for dependent types.
pub struct R7RSDependent {
    /// Parser for Scheme syntax
    parser: SchemeDependentParser,
    
    /// Type checker integration
    checker: DependentTypeChecker,
}

impl R7RSDependent {
    /// Create new R7RS dependent type integration.
    pub fn new() -> Self {
        Self {
            parser: SchemeDependentParser::new(),
            checker: DependentTypeChecker::new(),
        }
    }
    
    /// Check a Scheme expression with dependent type annotations.
    ///
    /// Example usage:
    /// ```scheme
    /// (: positive-length (Pi (xs (List Number)) (Refinement (n Number) (> n 0))))
    /// (define (positive-length xs)
    ///   (length xs))
    /// ```
    pub fn check_annotated_expression(
        &mut self,
        expr: &Value,
        annotation: Option<&Value>,
    ) -> Result<Type> {
        if let Some(type_expr) = annotation {
            let expected_type = self.parser.parse_type(type_expr)?;
            let internal_type = Type::from(expected_type);
            
            // Would integrate with main type checker
            Ok(internal_type)
        } else {
            // No annotation - use gradual typing
            Ok(Type::Dynamic)
        }
    }
    
    /// Provide type inference for Scheme expressions using dependent types.
    #[allow(clippy::only_used_in_recursion)]
    pub fn infer_dependent_type(&mut self, expr: &Value) -> Result<Type> {
        // Simplified inference
        match expr {
            Value::Literal(Literal::Number(_)) => Ok(Type::Number),
            Value::Literal(Literal::String(_)) => Ok(Type::String),
            Value::Literal(Literal::Boolean(_)) => Ok(Type::Boolean),
            Value::Symbol(_) => Ok(Type::Symbol),
            Value::Nil => Ok(Type::list(Type::Dynamic)),
            Value::Pair(head, tail) => {
                let head_type = self.infer_dependent_type(head)?;
                let tail_type = self.infer_dependent_type(tail)?;
                
                // Try to infer homogeneous list type
                match tail_type {
                    Type::List(elem_type) if *elem_type == head_type => {
                        Ok(Type::list(head_type))
                    }
                    _ if matches!(tail_type, Type::List(_)) => Ok(Type::list(head_type)),
                    _ => Ok(Type::pair(head_type, tail_type)),
                }
            }
            _ => Ok(Type::Dynamic),
        }
    }
}

impl Default for SchemeDependentParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for R7RSDependent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Value;

    #[test]
    fn test_basic_type_parsing() {
        let mut parser = SchemeDependentParser::new();
        
        let number_type = Value::Symbol(SymbolId::from("Number"));
        let parsed = parser.parse_type(&number_type).unwrap();
        assert_eq!(parsed, SchemeDependentType::Number);
    }
    
    #[test]
    fn test_pi_type_parsing() {
        let mut parser = SchemeDependentParser::new();
        
        // (Pi (n Number) String) - function from Number to String  
        let pi_expr = Value::Pair(
            Box::new(Value::Symbol(SymbolId::from("Pi"))),
            Box::new(Value::Pair(
                Box::new(Value::Pair(
                    Box::new(Value::Symbol(SymbolId::from("n"))),
                    Box::new(Value::Symbol(SymbolId::from("Number")))
                )),
                Box::new(Value::Pair(
                    Box::new(Value::Symbol(SymbolId::from("String"))),
                    Box::new(Value::Nil)
                ))
            ))
        );
        
        let parsed = parser.parse_type(&pi_expr).unwrap();
        match parsed {
            SchemeDependentType::Pi { var, domain, codomain } => {
                assert_eq!(var, "n");
                assert_eq!(*domain, SchemeDependentType::Number);
                assert_eq!(*codomain, SchemeDependentType::String);
            }
            _ => panic!("Expected Pi type"),
        }
    }
    
    #[test]
    fn test_vector_type_parsing() {
        let mut parser = SchemeDependentParser::new();
        
        // (Vec 5 Number) - vector of 5 numbers
        let vec_expr = Value::Pair(
            Box::new(Value::Symbol(SymbolId::from("Vec"))),
            Box::new(Value::Pair(
                Box::new(Value::Literal(Literal::Number(5.0))),
                Box::new(Value::Pair(
                    Box::new(Value::Symbol(SymbolId::from("Number"))),
                    Box::new(Value::Nil)
                ))
            ))
        );
        
        let parsed = parser.parse_type(&vec_expr).unwrap();
        match parsed {
            SchemeDependentType::Vector { length, element_type } => {
                assert_eq!(length, SchemeTypeTerm::Number(5));
                assert_eq!(*element_type, SchemeDependentType::Number);
            }
            _ => panic!("Expected Vector type"),
        }
    }
    
    #[test]
    fn test_type_conversion() {
        let scheme_type = SchemeDependentType::Pi {
            var: "x".to_string(),
            domain: Box::new(SchemeDependentType::Number),
            codomain: Box::new(SchemeDependentType::String),
        };
        
        let internal_type = Type::from(scheme_type);
        match internal_type {
            Type::Function { params, return_type } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], Type::Number);
                assert_eq!(*return_type, Type::String);
            }
            _ => panic!("Expected function type"),
        }
    }
}