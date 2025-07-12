//! Mdo Notation for Monads in Lambdust
//! Scheme version of Haskell's do notation with HoTT type class integration
//! Uses 'mdo' keyword to avoid conflict with Scheme's native 'do' loops

use crate::ast::Expr;
use crate::error::LambdustError;
use crate::macros::MacroExpander;
use std::collections::HashMap;

/// Do notation macro expander
#[derive(Debug)]
pub struct DoNotationExpander {
    /// Available monad instances
    monad_instances: HashMap<String, MonadInstance>,
    /// Macro expander for syntax transformation
    #[allow(dead_code)]
    expander: MacroExpander,
}

/// Monad instance information
#[derive(Debug, Clone)]
pub struct MonadInstance {
    /// Monad name (List, Maybe, etc.)
    pub name: String,
    /// Return function implementation
    pub return_fn: String,
    /// Bind function implementation
    pub bind_fn: String,
    /// Type constructor
    pub type_constructor: String,
}

/// Do notation binding
#[derive(Debug, Clone)]
pub enum DoBinding {
    /// Variable binding: x <- computation
    Bind {
        /// Variable name being bound
        var: String,
        /// Monadic computation expression
        computation: Expr,
    },
    /// Let binding: let x = value
    Let {
        /// Variable name being bound
        var: String,
        /// Value expression (non-monadic)
        value: Expr,
    },
    /// Pure computation (final expression)
    Pure(Expr),
}

/// Do block representation
#[derive(Debug, Clone)]
pub struct DoBlock {
    /// Sequence of bindings
    pub bindings: Vec<DoBinding>,
    /// Final expression
    pub result: Expr,
}

impl DoNotationExpander {
    /// Create new do notation expander
    pub fn new() -> Self {
        let mut expander = Self {
            monad_instances: HashMap::new(),
            expander: MacroExpander::new(),
        };
        
        // Register standard monad instances
        expander.register_standard_monads();
        
        expander
    }
    
    /// Register a monad instance
    pub fn register_monad(&mut self, instance: MonadInstance) {
        self.monad_instances.insert(instance.name.clone(), instance);
    }
    
    /// Expand mdo notation to monadic operations
    pub fn expand_mdo(&self, mdo_block: DoBlock) -> Result<Expr, LambdustError> {
        if mdo_block.bindings.is_empty() {
            return Ok(mdo_block.result);
        }
        
        // Desugar mdo notation from right to left
        let mut result = mdo_block.result;
        
        for binding in mdo_block.bindings.into_iter().rev() {
            result = match binding {
                DoBinding::Bind { var, computation } => {
                    // x <- m ==> m >>= (\x -> rest)
                    self.create_bind_expression(computation, var, result)?
                }
                DoBinding::Let { var, value } => {
                    // let x = v ==> (\x -> rest) v
                    self.create_let_expression(var, value, result)?
                }
                DoBinding::Pure(expr) => {
                    // Simple expression, sequence with >>
                    self.create_sequence_expression(expr, result)?
                }
            };
        }
        
        Ok(result)
    }
    
    /// Parse mdo syntax from S-expression
    pub fn parse_mdo_syntax(&self, expr: &Expr) -> Result<DoBlock, LambdustError> {
        match expr {
            Expr::List(elements) => {
                if elements.is_empty() {
                    return Err(LambdustError::syntax_error("Empty mdo block"));
                }
                
                // First element should be 'mdo
                if let Expr::Variable(sym) = &elements[0] {
                    if sym != "mdo" {
                        return Err(LambdustError::syntax_error("Expected 'mdo' keyword"));
                    }
                } else {
                    return Err(LambdustError::syntax_error("Expected 'mdo' keyword"));
                }
                
                let mut bindings = Vec::new();
                let binding_exprs = &elements[1..elements.len()-1];
                let final_expr = elements.last()
                    .ok_or_else(|| LambdustError::syntax_error("Mdo block must have final expression"))?;
                
                // Parse bindings
                for binding_expr in binding_exprs {
                    let binding = self.parse_binding(binding_expr)?;
                    bindings.push(binding);
                }
                
                Ok(DoBlock {
                    bindings,
                    result: final_expr.clone(),
                })
            }
            _ => Err(LambdustError::syntax_error("Mdo notation must be a list"))
        }
    }
    
    /// Parse a single binding
    fn parse_binding(&self, expr: &Expr) -> Result<DoBinding, LambdustError> {
        match expr {
            Expr::List(elements) => {
                if elements.len() == 3 {
                    // Check for <- binding
                    if let (Expr::Variable(var), Expr::Variable(arrow), computation) = (&elements[0], &elements[1], &elements[2]) {
                        if arrow == "<-" {
                            return Ok(DoBinding::Bind {
                                var: var.clone(),
                                computation: computation.clone(),
                            });
                        }
                    }
                    
                    // Check for let binding
                    if let (Expr::Variable(let_kw), Expr::Variable(var), value) = (&elements[0], &elements[1], &elements[2]) {
                        if let_kw == "let" {
                            return Ok(DoBinding::Let {
                                var: var.clone(),
                                value: value.clone(),
                            });
                        }
                    }
                }
                
                // If not a binding, treat as pure computation
                Ok(DoBinding::Pure(expr.clone()))
            }
            _ => Ok(DoBinding::Pure(expr.clone()))
        }
    }
    
    /// Create bind expression: m >>= (\x -> rest)
    fn create_bind_expression(&self, computation: Expr, var: String, rest: Expr) -> Result<Expr, LambdustError> {
        // Create lambda: (\x -> rest)
        let lambda = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![Expr::Variable(var)]),
            rest,
        ]);
        
        // Create bind: computation >>= lambda
        Ok(Expr::List(vec![
            Expr::Variable(">>=".to_string()),
            computation,
            lambda,
        ]))
    }
    
    /// Create let expression: (\x -> rest) value
    fn create_let_expression(&self, var: String, value: Expr, rest: Expr) -> Result<Expr, LambdustError> {
        // Create lambda application: ((\x -> rest) value)
        let lambda = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![Expr::Variable(var)]),
            rest,
        ]);
        
        Ok(Expr::List(vec![lambda, value]))
    }
    
    /// Create sequence expression: expr >> rest
    fn create_sequence_expression(&self, expr: Expr, rest: Expr) -> Result<Expr, LambdustError> {
        // Create sequence: expr >> rest
        Ok(Expr::List(vec![
            Expr::Variable(">>".to_string()),
            expr,
            rest,
        ]))
    }
    
    /// Register standard monad instances
    fn register_standard_monads(&mut self) {
        // List monad
        self.register_monad(MonadInstance {
            name: "List".to_string(),
            return_fn: "list".to_string(),
            bind_fn: "list-bind".to_string(),
            type_constructor: "List".to_string(),
        });
        
        // Maybe monad
        self.register_monad(MonadInstance {
            name: "Maybe".to_string(),
            return_fn: "just".to_string(),
            bind_fn: "maybe-bind".to_string(),
            type_constructor: "Maybe".to_string(),
        });
        
        // IO monad
        self.register_monad(MonadInstance {
            name: "IO".to_string(),
            return_fn: "return-io".to_string(),
            bind_fn: "bind-io".to_string(),
            type_constructor: "IO".to_string(),
        });
        
        // State monad
        self.register_monad(MonadInstance {
            name: "State".to_string(),
            return_fn: "return-state".to_string(),
            bind_fn: "bind-state".to_string(),
            type_constructor: "State".to_string(),
        });
    }
    
    /// Get monad instance by name
    pub fn get_monad(&self, name: &str) -> Option<&MonadInstance> {
        self.monad_instances.get(name)
    }
    
    /// Check if expression is mdo notation
    pub fn is_mdo_notation(&self, expr: &Expr) -> bool {
        match expr {
            Expr::List(elements) => {
                !elements.is_empty() && 
                matches!(&elements[0], Expr::Variable(sym) if sym == "mdo")
            }
            _ => false
        }
    }
    
    /// Transform mdo notation for specific monad
    pub fn transform_for_monad(&self, mdo_block: DoBlock, monad_name: &str) -> Result<Expr, LambdustError> {
        let monad = self.get_monad(monad_name)
            .ok_or_else(|| LambdustError::type_error(format!("Unknown monad: {}", monad_name)))?;
        
        // Replace generic >>= and return with monad-specific ones
        let mut expanded = self.expand_mdo(mdo_block)?;
        self.replace_monad_operations(&mut expanded, monad);
        
        Ok(expanded)
    }
    
    /// Replace generic monad operations with specific implementations
    fn replace_monad_operations(&self, expr: &mut Expr, monad: &MonadInstance) {
        match expr {
            Expr::List(elements) => {
                // First recursively process all elements
                for element in &mut *elements {
                    self.replace_monad_operations(element, monad);
                }
                
                // Replace >>= with monad-specific bind
                if !elements.is_empty() {
                    if let Expr::Variable(op) = &elements[0] {
                        if op == ">>=" {
                            elements[0] = Expr::Variable(monad.bind_fn.clone());
                        } else if op == "return" {
                            elements[0] = Expr::Variable(monad.return_fn.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl Default for DoNotationExpander {
    fn default() -> Self {
        Self::new()
    }
}

/// Scheme macro for mdo notation
pub fn register_mdo_macro() -> Result<(), LambdustError> {
    // This would integrate with the macro system
    // For now, return Ok as placeholder
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;

    #[test]
    fn test_do_notation_expander_creation() {
        let expander = DoNotationExpander::new();
        assert!(expander.get_monad("List").is_some());
        assert!(expander.get_monad("Maybe").is_some());
    }

    #[test]
    fn test_parse_simple_mdo_block() {
        let expander = DoNotationExpander::new();
        
        // (mdo [x <- (list 1 2 3)] [y <- (list 4 5 6)] (+ x y))
        let mdo_expr = Expr::List(vec![
            Expr::Variable("mdo".to_string()),
            Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Variable("<-".to_string()),
                Expr::List(vec![
                    Expr::Variable("list".to_string()),
                    Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(1))),
                    Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(2))),
                    Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(3))),
                ]),
            ]),
            Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Variable("x".to_string()),
                Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(10))),
            ]),
        ]);
        
        let result = expander.parse_mdo_syntax(&mdo_expr);
        assert!(result.is_ok());
        
        let mdo_block = result.unwrap();
        assert_eq!(mdo_block.bindings.len(), 1);
        
        match &mdo_block.bindings[0] {
            DoBinding::Bind { var, .. } => assert_eq!(var, "x"),
            _ => panic!("Expected bind"),
        }
    }

    #[test]
    fn test_expand_simple_mdo() {
        let expander = DoNotationExpander::new();
        
        let mdo_block = DoBlock {
            bindings: vec![
                DoBinding::Bind {
                    var: "x".to_string(),
                    computation: Expr::List(vec![
                        Expr::Variable("list".to_string()),
                        Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(1))),
                    ]),
                }
            ],
            result: Expr::Variable("x".to_string()),
        };
        
        let result = expander.expand_mdo(mdo_block);
        assert!(result.is_ok());
        
        // Should generate bind expression
        let expansion_result = result.unwrap();
        match expansion_result {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 3);
                if let Expr::Variable(op) = &elements[0] {
                    assert_eq!(op, ">>=");
                }
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_monad_instance_registration() {
        let mut expander = DoNotationExpander::new();
        
        let custom_monad = MonadInstance {
            name: "Custom".to_string(),
            return_fn: "custom-return".to_string(),
            bind_fn: "custom-bind".to_string(),
            type_constructor: "Custom".to_string(),
        };
        
        expander.register_monad(custom_monad);
        assert!(expander.get_monad("Custom").is_some());
    }

    #[test]
    fn test_is_mdo_notation() {
        let expander = DoNotationExpander::new();
        
        let mdo_expr = Expr::List(vec![
            Expr::Variable("mdo".to_string()),
            Expr::Variable("x".to_string()),
        ]);
        
        assert!(expander.is_mdo_notation(&mdo_expr));
        
        let not_mdo_expr = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::Variable("x".to_string()),
        ]);
        
        assert!(!expander.is_mdo_notation(&not_mdo_expr));
    }

    #[test]
    fn test_multiple_bindings() {
        let notation_expander = DoNotationExpander::new();
        
        let mdo_block = DoBlock {
            bindings: vec![
                DoBinding::Bind {
                    var: "x".to_string(),
                    computation: Expr::List(vec![
                        Expr::Variable("list".to_string()),
                        Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(1))),
                    ]),
                },
                DoBinding::Bind {
                    var: "y".to_string(),
                    computation: Expr::List(vec![
                        Expr::Variable("list".to_string()),
                        Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(2))),
                    ]),
                }
            ],
            result: Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Variable("x".to_string()),
                Expr::Variable("y".to_string()),
            ]),
        };
        
        let result = notation_expander.expand_mdo(mdo_block);
        assert!(result.is_ok());
        
        // Should generate nested bind expressions
        let nested_expanded = result.unwrap();
        match nested_expanded {
            Expr::List(elements) => {
                // Outer bind
                assert_eq!(elements.len(), 3);
                if let Expr::Variable(op) = &elements[0] {
                    assert_eq!(op, ">>=");
                }
                
                // Inner should also be a bind
                if let Expr::List(lambda_elements) = &elements[2] {
                    if lambda_elements.len() >= 3 {
                        if let Expr::List(body_elements) = &lambda_elements[2] {
                            if let Expr::Variable(inner_op) = &body_elements[0] {
                                assert_eq!(inner_op, ">>=");
                            }
                        }
                    }
                }
            }
            _ => panic!("Expected list expression"),
        }
    }
}