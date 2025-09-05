//! Macro prescan system for detecting and pre-registering define-syntax forms.
//!
//! This module implements the prescan phase that walks through the AST before
//! macro expansion to detect and pre-register all define-syntax forms. This
//! ensures that macros are available during the expansion phase.

use crate::ast::{Expr, Program};
use crate::diagnostics::{Error, Result, Spanned};
use crate::macro_system::{MacroExpander, parse_syntax_rules, syntax_rules_to_macro_transformer};
use crate::eval::Environment;
use std::rc::Rc;

/// Information about a detected macro definition.
#[derive(Debug, Clone)]
pub struct MacroDefinition {
    /// Name of the macro
    pub name: String,
    /// The syntax-rules transformer expression
    pub transformer: Spanned<Expr>,
    /// Span information for error reporting
    pub span: crate::diagnostics::Span,
}

/// Results of the prescan phase.
#[derive(Debug, Default)]
pub struct PrescanResults {
    /// Detected macro definitions
    pub macro_definitions: Vec<MacroDefinition>,
    /// Number of expressions processed
    pub expressions_processed: usize,
    /// Number of define-syntax forms found
    pub define_syntax_count: usize,
}

/// Prescans an AST to detect and collect all define-syntax forms.
pub fn prescan_ast(program: &Program) -> Result<PrescanResults> {
    let mut scanner = AstScanner::new();
    scanner.scan_program(program)?;
    Ok(scanner.results)
}

/// Pre-registers detected macros in a macro expander.
pub fn register_prescanned_macros(
    expander: &mut MacroExpander, 
    results: &PrescanResults
) -> Result<usize> {
    let mut registered_count = 0;
    
    for macro_def in &results.macro_definitions {
        println!("DEBUG: Pre-registering macro: {}", macro_def.name);
        
        // Create a temporary environment for parsing
        let temp_env = Rc::new(Environment::new(None, 0));
        
        // Parse the syntax-rules transformer
        match parse_syntax_rules(&macro_def.transformer, temp_env) {
            Ok(syntax_rules_transformer) => {
                // Use the new multi-rule syntax-rules support
                if syntax_rules_transformer.rules.len() > 1 {
                    println!("DEBUG: Registering multi-rule syntax-rules macro: {} ({} rules)", 
                             macro_def.name, syntax_rules_transformer.rules.len());
                    expander.define_syntax_rules(macro_def.name.clone(), syntax_rules_transformer);
                } else {
                    // Single rule - use the traditional approach
                    println!("DEBUG: Registering single-rule macro: {}", macro_def.name);
                    let macro_transformer = syntax_rules_to_macro_transformer(syntax_rules_transformer);
                    expander.define_macro(macro_def.name.clone(), macro_transformer);
                }
                registered_count += 1;
                println!("DEBUG: Successfully registered macro: {}", macro_def.name);
            }
            Err(e) => {
                println!("DEBUG: Failed to parse syntax-rules for {}: {:?}", macro_def.name, e);
                return Err(e);
            }
        }
    }
    
    println!("DEBUG: Pre-registered {} macros", registered_count);
    Ok(registered_count)
}

/// AST scanner that walks through expressions to find define-syntax forms.
struct AstScanner {
    results: PrescanResults,
}

impl AstScanner {
    /// Creates a new AST scanner.
    fn new() -> Self {
        Self {
            results: PrescanResults::default(),
        }
    }
    
    /// Scans a complete program.
    fn scan_program(&mut self, program: &Program) -> Result<()> {
        for expr in &program.expressions {
            self.scan_expression(expr)?;
        }
        Ok(())
    }
    
    /// Scans a single expression recursively.
    fn scan_expression(&mut self, expr: &Spanned<Expr>) -> Result<()> {
        self.results.expressions_processed += 1;
        
        // Debug: Log every expression type being processed
        eprintln!("DEBUG: Prescan processing expression: {:?} (span: {:?})", 
                 std::mem::discriminant(&expr.inner), expr.span);
        
        match &expr.inner {
            Expr::DefineSyntax { name, transformer } => {
                // Found a macro definition!
                eprintln!("DEBUG: Prescan detected define-syntax: {}", name);
                
                self.results.macro_definitions.push(MacroDefinition {
                    name: name.clone(),
                    transformer: (**transformer).clone(),
                    span: expr.span,
                });
                self.results.define_syntax_count += 1;
                
                // Don't recursively scan the transformer expression to avoid infinite loops
                // The transformer will be parsed during macro registration
            }
            
            // Recursively scan compound expressions
            Expr::Application { operator, operands } => {
                self.scan_expression(operator)?;
                for operand in operands {
                    self.scan_expression(operand)?;
                }
            }
            
            Expr::Lambda { body, .. } => {
                for body_expr in body {
                    self.scan_expression(body_expr)?;
                }
            }
            
            Expr::If { test, consequent, alternative } => {
                self.scan_expression(test)?;
                self.scan_expression(consequent)?;
                if let Some(alt) = alternative {
                    self.scan_expression(alt)?;
                }
            }
            
            Expr::Let { bindings, body, .. } => {
                for binding in bindings {
                    self.scan_expression(&binding.value)?;
                }
                for body_expr in body {
                    self.scan_expression(body_expr)?;
                }
            }
            
            Expr::Begin(exprs) => {
                eprintln!("DEBUG: Prescan entering Begin with {} expressions", exprs.len());
                for expr in exprs {
                    self.scan_expression(expr)?;
                }
            }
            
            Expr::Define { value, .. } => {
                self.scan_expression(value)?;
            }
            
            Expr::Set { value, .. } => {
                self.scan_expression(value)?;
            }
            
            Expr::And(exprs) | Expr::Or(exprs) => {
                for expr in exprs {
                    self.scan_expression(expr)?;
                }
            }
            
            Expr::Cond(clauses) => {
                for clause in clauses {
                    self.scan_expression(&clause.test)?;
                    for body_expr in &clause.body {
                        self.scan_expression(body_expr)?;
                    }
                }
            }
            
            Expr::DefineLibrary { body, .. } => {
                eprintln!("DEBUG: Prescan entering DefineLibrary with {} body expressions", body.len());
                for (i, body_expr) in body.iter().enumerate() {
                    eprintln!("DEBUG: DefineLibrary body[{}]: {:?}", i, std::mem::discriminant(&body_expr.inner));
                    self.scan_expression(body_expr)?;
                }
            }
            
            // Leaf nodes - no further scanning needed
            Expr::Literal(_) |
            Expr::Identifier(_) |
            Expr::SyntaxRules { .. } |
            Expr::Quote(_) |
            Expr::Quasiquote(_) |
            Expr::Unquote(_) |
            Expr::UnquoteSplicing(_) => {
                // No further scanning needed for literals and leaf nodes
            }
            
            // Handle other expression types that might contain nested expressions
            _ => {
                eprintln!("DEBUG: Prescan - unhandled expression type: {:?}", 
                         std::mem::discriminant(&expr.inner));
                // For any unhandled expression types, we'll skip for now
                // In a complete implementation, we'd handle all expression types
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::lexer::Lexer;
    
    #[test]
    fn test_prescan_simple_define_syntax() {
        let source = r#"
            (define-syntax test-macro
              (syntax-rules ()
                ((_ x) (list x))))
        "#;
        
        let mut lexer = Lexer::new(source, Some("test"));
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        let results = prescan_ast(&program).unwrap();
        
        assert_eq!(results.define_syntax_count, 1);
        assert_eq!(results.macro_definitions.len(), 1);
        assert_eq!(results.macro_definitions[0].name, "test-macro");
    }
    
    #[test]
    fn test_prescan_nested_define_syntax() {
        let source = r#"
            (begin
              (define x 42)
              (define-syntax nested-macro
                (syntax-rules ()
                  ((_ a b) (+ a b))))
              (define y 24))
        "#;
        
        let mut lexer = Lexer::new(source, Some("test"));
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        let results = prescan_ast(&program).unwrap();
        
        assert_eq!(results.define_syntax_count, 1);
        assert_eq!(results.macro_definitions[0].name, "nested-macro");
        assert!(results.expressions_processed > 1);
    }
}