//! Typed Special Forms - Extended syntax with type annotations
//! Support for optional type annotations in lambda and define expressions

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
use crate::type_system::polynomial_types::PolynomialType;
use crate::value::{Procedure, Value};
use std::rc::Rc;

/// Parameter with optional type annotation
#[derive(Debug, Clone)]
pub struct TypedParameter {
    /// Parameter name
    pub name: String,
    /// Optional type annotation
    pub type_annotation: Option<PolynomialType>,
}

/// Lambda expression with optional type annotations
#[derive(Debug, Clone)]
pub struct TypedLambda {
    /// Parameters with optional type annotations
    pub params: Vec<TypedParameter>,
    /// Optional return type annotation
    pub return_type: Option<PolynomialType>,
    /// Lambda body expressions
    pub body: Vec<Expr>,
    /// Whether this lambda is variadic
    pub variadic: bool,
}

/// Define expression with optional type annotations
#[derive(Debug, Clone)]
pub struct TypedDefine {
    /// Variable name
    pub name: String,
    /// Optional type annotation for the variable
    pub type_annotation: Option<PolynomialType>,
    /// Value expression
    pub value: Expr,
}

/// Function definition with optional type annotations
#[derive(Debug, Clone)]
pub struct TypedFunctionDefine {
    /// Function name
    pub name: String,
    /// Parameters with optional type annotations
    pub params: Vec<TypedParameter>,
    /// Optional return type annotation
    pub return_type: Option<PolynomialType>,
    /// Function body expressions
    pub body: Vec<Expr>,
    /// Whether this function is variadic
    pub variadic: bool,
}

impl Evaluator {
    /// Evaluate typed lambda expression
    /// Syntax: (lambda ((x : Int) (y : String)) : Bool body...)
    /// Or:     (lambda (x y) body...)  (without type annotations)
    pub fn eval_typed_lambda(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        let typed_lambda = self.parse_typed_lambda(operands)?;
        
        // For now, we'll create a regular lambda and attach type information
        // In the future, this would integrate with the type checker
        let params: Vec<String> = typed_lambda.params.iter()
            .map(|p| p.name.clone())
            .collect();
        
        let lambda = Procedure::Lambda {
            params,
            body: typed_lambda.body,
            closure: env,
            variadic: typed_lambda.variadic,
        };

        // TODO: Store type annotations for type checking
        self.apply_continuation(cont, Value::Procedure(lambda))
    }

    /// Evaluate typed define expression
    /// Syntax: (define x : Int value)
    /// Or:     (define (foo (x : Int) (y : String)) : Bool body...)
    /// Or:     (define x value)  (without type annotations)
    pub fn eval_typed_define(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        match &operands[0] {
            // Variable definition with optional type: (define x : Int value) or (define x value)
            Expr::Variable(_) => {
                let typed_define = self.parse_typed_variable_define(operands)?;
                
                let define_cont = Continuation::Define {
                    variable: typed_define.name,
                    env: Rc::clone(&env),
                    parent: Box::new(cont),
                };

                // TODO: Type check the value expression against the annotation
                self.eval(typed_define.value, env, define_cont)
            }
            // Function definition with optional types: (define (foo (x : Int)) : Bool body...)
            Expr::List(_) => {
                let typed_func_define = self.parse_typed_function_define(operands)?;
                
                // Transform to typed lambda
                let params: Vec<String> = typed_func_define.params.iter()
                    .map(|p| p.name.clone())
                    .collect();
                
                let lambda = Procedure::Lambda {
                    params,
                    body: typed_func_define.body,
                    closure: Rc::clone(&env),
                    variadic: typed_func_define.variadic,
                };

                // Define the function
                env.define(typed_func_define.name, Value::Procedure(lambda));
                
                // TODO: Store type annotations for type checking
                self.apply_continuation(cont, Value::Undefined)
            }
            _ => Err(LambdustError::syntax_error(
                "define: first argument must be a variable or function definition".to_string(),
            ))
        }
    }

    /// Parse typed lambda expression
    fn parse_typed_lambda(&self, operands: &[Expr]) -> Result<TypedLambda> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "lambda: requires at least 2 arguments (params and body)".to_string(),
            ));
        }

        let (params, variadic) = self.parse_typed_parameters(&operands[0])?;
        
        // Check for return type annotation
        let (return_type, body_start) = if operands.len() >= 3 {
            if let Expr::Variable(colon) = &operands[1] {
                if colon == ":" {
                    // Parse return type
                    let return_type = self.parse_type_annotation(&operands[2])?;
                    (Some(return_type), 3)
                } else {
                    (None, 1)
                }
            } else {
                (None, 1)
            }
        } else {
            (None, 1)
        };

        let body = operands[body_start..].to_vec();

        Ok(TypedLambda {
            params,
            return_type,
            body,
            variadic,
        })
    }

    /// Parse typed parameters
    /// Supports: (x y z), ((x : Int) (y : String) z), x (variadic)
    fn parse_typed_parameters(&self, params_expr: &Expr) -> Result<(Vec<TypedParameter>, bool)> {
        match params_expr {
            // Parameter list: (param1 param2 ...)
            Expr::List(params) => {
                let mut typed_params = Vec::new();
                
                for param in params {
                    match param {
                        // Simple parameter: x
                        Expr::Variable(name) => {
                            typed_params.push(TypedParameter {
                                name: name.clone(),
                                type_annotation: None,
                            });
                        }
                        // Typed parameter: (x : Type)
                        Expr::List(param_list) => {
                            if param_list.len() == 3 {
                                if let (Expr::Variable(name), Expr::Variable(colon), type_expr) = 
                                    (&param_list[0], &param_list[1], &param_list[2]) {
                                    if colon == ":" {
                                        let type_annotation = self.parse_type_annotation(type_expr)?;
                                        typed_params.push(TypedParameter {
                                            name: name.clone(),
                                            type_annotation: Some(type_annotation),
                                        });
                                        continue;
                                    }
                                }
                            }
                            return Err(LambdustError::syntax_error(
                                "lambda: invalid typed parameter syntax, expected (name : type)".to_string(),
                            ));
                        }
                        _ => {
                            return Err(LambdustError::syntax_error(
                                "lambda: parameter must be a symbol or (name : type)".to_string(),
                            ));
                        }
                    }
                }
                
                Ok((typed_params, false))
            }
            // Single variadic parameter: x
            Expr::Variable(name) => {
                Ok((vec![TypedParameter {
                    name: name.clone(),
                    type_annotation: None,
                }], true))
            }
            _ => Err(LambdustError::syntax_error(
                "lambda: invalid parameter list".to_string(),
            ))
        }
    }

    /// Parse variable define with optional type annotation
    /// Supports: (define x value) or (define x : Type value)
    fn parse_typed_variable_define(&self, operands: &[Expr]) -> Result<TypedDefine> {
        if let Expr::Variable(name) = &operands[0] {
            if operands.len() == 2 {
                // Simple define: (define x value)
                Ok(TypedDefine {
                    name: name.clone(),
                    type_annotation: None,
                    value: operands[1].clone(),
                })
            } else if operands.len() == 4 {
                // Typed define: (define x : Type value)
                if let Expr::Variable(colon) = &operands[1] {
                    if colon == ":" {
                        let type_annotation = self.parse_type_annotation(&operands[2])?;
                        Ok(TypedDefine {
                            name: name.clone(),
                            type_annotation: Some(type_annotation),
                            value: operands[3].clone(),
                        })
                    } else {
                        Err(LambdustError::syntax_error(
                            "define: expected ':' for type annotation".to_string(),
                        ))
                    }
                } else {
                    Err(LambdustError::syntax_error(
                        "define: invalid type annotation syntax".to_string(),
                    ))
                }
            } else {
                Err(LambdustError::arity_error(2, operands.len()))
            }
        } else {
            Err(LambdustError::syntax_error(
                "define: first argument must be a variable".to_string(),
            ))
        }
    }

    /// Parse function define with optional type annotations
    /// Supports: (define (foo x y) body...) or (define (foo (x : Int) (y : String)) : Bool body...)
    fn parse_typed_function_define(&self, operands: &[Expr]) -> Result<TypedFunctionDefine> {
        if let Expr::List(def_list) = &operands[0] {
            if def_list.is_empty() {
                return Err(LambdustError::syntax_error(
                    "define: empty function definition".to_string(),
                ));
            }

            let function_name = match &def_list[0] {
                Expr::Variable(name) => name.clone(),
                _ => {
                    return Err(LambdustError::syntax_error(
                        "define: function name must be a symbol".to_string(),
                    ));
                }
            };

            // Parse parameters
            let param_list = Expr::List(def_list[1..].to_vec());
            let (params, variadic) = self.parse_typed_parameters(&param_list)?;

            // Check for return type annotation
            let (return_type, body_start) = if operands.len() >= 3 {
                if let Expr::Variable(colon) = &operands[1] {
                    if colon == ":" {
                        let return_type = self.parse_type_annotation(&operands[2])?;
                        (Some(return_type), 3)
                    } else {
                        (None, 1)
                    }
                } else {
                    (None, 1)
                }
            } else {
                (None, 1)
            };

            let body = operands[body_start..].to_vec();

            Ok(TypedFunctionDefine {
                name: function_name,
                params,
                return_type,
                body,
                variadic,
            })
        } else {
            Err(LambdustError::syntax_error(
                "define: invalid function definition".to_string(),
            ))
        }
    }

    /// Parse type annotation from expression
    /// This is a simplified parser for basic types
    fn parse_type_annotation(&self, expr: &Expr) -> Result<PolynomialType> {
        match expr {
            Expr::Variable(type_name) => {
                match type_name.as_str() {
                    "Int" | "Integer" => Ok(PolynomialType::Base(
                        crate::type_system::polynomial_types::BaseType::Integer
                    )),
                    "Real" | "Float" => Ok(PolynomialType::Base(
                        crate::type_system::polynomial_types::BaseType::Real
                    )),
                    "Bool" | "Boolean" => Ok(PolynomialType::Base(
                        crate::type_system::polynomial_types::BaseType::Boolean
                    )),
                    "String" => Ok(PolynomialType::Base(
                        crate::type_system::polynomial_types::BaseType::String
                    )),
                    "Char" | "Character" => Ok(PolynomialType::Base(
                        crate::type_system::polynomial_types::BaseType::Character
                    )),
                    "Symbol" => Ok(PolynomialType::Base(
                        crate::type_system::polynomial_types::BaseType::Symbol
                    )),
                    _ => {
                        // Treat as type variable for now
                        Ok(PolynomialType::Variable {
                            name: type_name.clone(),
                            level: crate::type_system::polynomial_types::UniverseLevel::new(0),
                        })
                    }
                }
            }
            Expr::List(type_expr) => {
                if type_expr.len() >= 2 {
                    match type_expr[0].as_symbol() {
                        Some("List") => {
                            let element_type = self.parse_type_annotation(&type_expr[1])?;
                            Ok(PolynomialType::List {
                                element_type: Box::new(element_type),
                            })
                        }
                        Some("->") if type_expr.len() == 3 => {
                            // Function type: (-> Input Output)
                            let input_type = self.parse_type_annotation(&type_expr[1])?;
                            let output_type = self.parse_type_annotation(&type_expr[2])?;
                            Ok(PolynomialType::Function {
                                input: Box::new(input_type),
                                output: Box::new(output_type),
                            })
                        }
                        Some("*") if type_expr.len() == 3 => {
                            // Product type: (* A B)
                            let left_type = self.parse_type_annotation(&type_expr[1])?;
                            let right_type = self.parse_type_annotation(&type_expr[2])?;
                            Ok(PolynomialType::Product {
                                left: Box::new(left_type),
                                right: Box::new(right_type),
                            })
                        }
                        Some("+") if type_expr.len() == 3 => {
                            // Sum type: (+ A B)
                            let left_type = self.parse_type_annotation(&type_expr[1])?;
                            let right_type = self.parse_type_annotation(&type_expr[2])?;
                            Ok(PolynomialType::Sum {
                                left: Box::new(left_type),
                                right: Box::new(right_type),
                            })
                        }
                        _ => {
                            Err(LambdustError::syntax_error(
                                format!("Unknown type constructor: {:?}", type_expr[0])
                            ))
                        }
                    }
                } else {
                    Err(LambdustError::syntax_error(
                        "Invalid type expression".to_string()
                    ))
                }
            }
            _ => {
                Err(LambdustError::syntax_error(
                    "Type annotation must be a symbol or type expression".to_string()
                ))
            }
        }
    }

    /// Check if expression uses typed syntax
    pub fn is_typed_lambda(&self, operands: &[Expr]) -> bool {
        if operands.is_empty() {
            return false;
        }

        // Check if parameters contain type annotations
        if let Expr::List(params) = &operands[0] {
            for param in params {
                if let Expr::List(param_list) = param {
                    if param_list.len() == 3 {
                        if let Expr::Variable(colon) = &param_list[1] {
                            if colon == ":" {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        // Check for return type annotation
        if operands.len() >= 3 {
            if let Expr::Variable(colon) = &operands[1] {
                if colon == ":" {
                    return true;
                }
            }
        }

        false
    }

    /// Check if define expression uses typed syntax
    pub fn is_typed_define(&self, operands: &[Expr]) -> bool {
        if operands.len() < 2 {
            return false;
        }

        match &operands[0] {
            // Variable definition with type: (define x : Type value)
            Expr::Variable(_) => {
                operands.len() == 4 && 
                matches!(&operands[1], Expr::Variable(colon) if colon == ":")
            }
            // Function definition with typed parameters or return type
            Expr::List(def_list) => {
                // Check for typed parameters
                if def_list.len() > 1 {
                    let param_list = Expr::List(def_list[1..].to_vec());
                    if self.is_typed_lambda(&[param_list]) {
                        return true;
                    }
                }

                // Check for return type annotation
                if operands.len() >= 3 {
                    if let Expr::Variable(colon) = &operands[1] {
                        if colon == ":" {
                            return true;
                        }
                    }
                }

                false
            }
            _ => false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::lexer::SchemeNumber;
    // use crate::environment::Environment; // Currently unused

    #[test]
    fn test_parse_simple_typed_lambda() {
        let evaluator = Evaluator::new();
        
        // (lambda ((x : Int) (y : String)) body)
        let operands = vec![
            Expr::List(vec![
                Expr::List(vec![
                    Expr::Variable("x".to_string()),
                    Expr::Variable(":".to_string()),
                    Expr::Variable("Int".to_string()),
                ]),
                Expr::List(vec![
                    Expr::Variable("y".to_string()),
                    Expr::Variable(":".to_string()),
                    Expr::Variable("String".to_string()),
                ]),
            ]),
            Expr::Variable("body".to_string()),
        ];

        let result = evaluator.parse_typed_lambda(&operands);
        assert!(result.is_ok());

        let typed_lambda = result.unwrap();
        assert_eq!(typed_lambda.params.len(), 2);
        assert_eq!(typed_lambda.params[0].name, "x");
        assert!(typed_lambda.params[0].type_annotation.is_some());
        assert_eq!(typed_lambda.params[1].name, "y");
        assert!(typed_lambda.params[1].type_annotation.is_some());
        assert!(!typed_lambda.variadic);
    }

    #[test]
    fn test_parse_typed_lambda_with_return_type() {
        let evaluator = Evaluator::new();
        
        // (lambda ((x : Int)) : Bool body)
        let operands = vec![
            Expr::List(vec![
                Expr::List(vec![
                    Expr::Variable("x".to_string()),
                    Expr::Variable(":".to_string()),
                    Expr::Variable("Int".to_string()),
                ]),
            ]),
            Expr::Variable(":".to_string()),
            Expr::Variable("Bool".to_string()),
            Expr::Variable("body".to_string()),
        ];

        let result = evaluator.parse_typed_lambda(&operands);
        assert!(result.is_ok());

        let typed_lambda = result.unwrap();
        assert_eq!(typed_lambda.params.len(), 1);
        assert!(typed_lambda.return_type.is_some());
    }

    #[test]
    fn test_parse_mixed_typed_parameters() {
        let evaluator = Evaluator::new();
        
        // (lambda ((x : Int) y (z : String)) body)
        let operands = vec![
            Expr::List(vec![
                Expr::List(vec![
                    Expr::Variable("x".to_string()),
                    Expr::Variable(":".to_string()),
                    Expr::Variable("Int".to_string()),
                ]),
                Expr::Variable("y".to_string()), // No type annotation
                Expr::List(vec![
                    Expr::Variable("z".to_string()),
                    Expr::Variable(":".to_string()),
                    Expr::Variable("String".to_string()),
                ]),
            ]),
            Expr::Variable("body".to_string()),
        ];

        let result = evaluator.parse_typed_lambda(&operands);
        assert!(result.is_ok());

        let typed_lambda = result.unwrap();
        assert_eq!(typed_lambda.params.len(), 3);
        assert!(typed_lambda.params[0].type_annotation.is_some()); // x has type
        assert!(typed_lambda.params[1].type_annotation.is_none());  // y has no type
        assert!(typed_lambda.params[2].type_annotation.is_some()); // z has type
    }

    #[test]
    fn test_parse_typed_variable_define() {
        let evaluator = Evaluator::new();
        
        // (define x : Int 42)
        let operands = vec![
            Expr::Variable("x".to_string()),
            Expr::Variable(":".to_string()),
            Expr::Variable("Int".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ];

        let result = evaluator.parse_typed_variable_define(&operands);
        assert!(result.is_ok());

        let typed_define = result.unwrap();
        assert_eq!(typed_define.name, "x");
        assert!(typed_define.type_annotation.is_some());
    }

    #[test]
    fn test_is_typed_lambda() {
        let evaluator = Evaluator::new();
        
        // Typed lambda: (lambda ((x : Int)) body)
        let typed_operands = vec![
            Expr::List(vec![
                Expr::List(vec![
                    Expr::Variable("x".to_string()),
                    Expr::Variable(":".to_string()),
                    Expr::Variable("Int".to_string()),
                ]),
            ]),
            Expr::Variable("body".to_string()),
        ];
        assert!(evaluator.is_typed_lambda(&typed_operands));

        // Untyped lambda: (lambda (x) body)
        let untyped_operands = vec![
            Expr::List(vec![Expr::Variable("x".to_string())]),
            Expr::Variable("body".to_string()),
        ];
        assert!(!evaluator.is_typed_lambda(&untyped_operands));
    }

    #[test]
    fn test_is_typed_define() {
        let evaluator = Evaluator::new();
        
        // Typed variable define: (define x : Int 42)
        let typed_var_operands = vec![
            Expr::Variable("x".to_string()),
            Expr::Variable(":".to_string()),
            Expr::Variable("Int".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ];
        assert!(evaluator.is_typed_define(&typed_var_operands));

        // Untyped variable define: (define x 42)
        let untyped_var_operands = vec![
            Expr::Variable("x".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ];
        assert!(!evaluator.is_typed_define(&untyped_var_operands));
    }

    #[test]
    fn test_parse_function_type_annotation() {
        let evaluator = Evaluator::new();
        
        // Function type: (-> Int Bool)
        let func_type_expr = Expr::List(vec![
            Expr::Variable("->".to_string()),
            Expr::Variable("Int".to_string()),
            Expr::Variable("Bool".to_string()),
        ]);

        let result = evaluator.parse_type_annotation(&func_type_expr);
        assert!(result.is_ok());
        
        if let PolynomialType::Function { .. } = result.unwrap() {
            // Success
        } else {
            panic!("Expected function type");
        }
    }

    #[test]
    fn test_parse_list_type_annotation() {
        let evaluator = Evaluator::new();
        
        // List type: (List Int)
        let list_type_expr = Expr::List(vec![
            Expr::Variable("List".to_string()),
            Expr::Variable("Int".to_string()),
        ]);

        let result = evaluator.parse_type_annotation(&list_type_expr);
        assert!(result.is_ok());
        
        if let PolynomialType::List { .. } = result.unwrap() {
            // Success
        } else {
            panic!("Expected list type");
        }
    }
}