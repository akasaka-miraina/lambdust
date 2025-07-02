//! Macro system implementation for Scheme

use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use std::collections::HashMap;

/// Macro transformer type
pub type MacroTransformer = fn(&[Expr]) -> Result<Expr>;

/// Macro definition
#[derive(Debug, Clone)]
pub struct Macro {
    /// Macro name
    pub name: String,
    /// Macro transformer function
    pub transformer: MacroTransformer,
    /// Whether this is a syntax-rules macro
    pub is_syntax_rules: bool,
}

/// Pattern for syntax-rules
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Literal pattern (must match exactly)
    Literal(String),
    /// Variable pattern (binds to any expression)
    Variable(String),
    /// List pattern
    List(Vec<Pattern>),
    /// Ellipsis pattern (matches zero or more)
    Ellipsis(Box<Pattern>),
    /// Dotted pattern
    Dotted(Vec<Pattern>, Box<Pattern>),
    /// Nested ellipsis pattern (SRFI 46 extension)
    NestedEllipsis(Box<Pattern>, usize), // pattern with nesting level
    /// Vector pattern (SRFI 46 extension)
    Vector(Vec<Pattern>),
}

/// Template for syntax-rules
#[derive(Debug, Clone, PartialEq)]
pub enum Template {
    /// Literal template
    Literal(String),
    /// Variable reference
    Variable(String),
    /// List template
    List(Vec<Template>),
    /// Ellipsis template (expands pattern variables)
    Ellipsis(Box<Template>),
    /// Dotted template
    Dotted(Vec<Template>, Box<Template>),
    /// Nested ellipsis template (SRFI 46 extension)
    NestedEllipsis(Box<Template>, usize), // template with nesting level
    /// Vector template (SRFI 46 extension)
    Vector(Vec<Template>),
}

/// Syntax rule (pattern -> template)
///
/// Represents a single transformation rule in a syntax-rules macro definition.
/// Each rule consists of a pattern that matches input expressions and a template
/// that specifies how to transform the matched input.
#[derive(Debug, Clone)]
pub struct SyntaxRule {
    /// The pattern to match against input expressions
    pub pattern: Pattern,
    /// The template for generating the output expression
    pub template: Template,
}

/// Macro expansion context
#[derive(Debug, Clone)]
pub struct MacroExpander {
    /// Built-in macros
    macros: HashMap<String, Macro>,
}

impl MacroExpander {
    /// Create a new macro expander
    pub fn new() -> Self {
        let mut expander = MacroExpander {
            macros: HashMap::new(),
        };

        // Add built-in macros
        expander.add_builtin_macros();
        expander
    }

    /// Add built-in macros
    fn add_builtin_macros(&mut self) {
        // let macro
        self.macros.insert(
            "let".to_string(),
            Macro {
                name: "let".to_string(),
                transformer: expand_let,
                is_syntax_rules: false,
            },
        );

        // let* macro
        self.macros.insert(
            "let*".to_string(),
            Macro {
                name: "let*".to_string(),
                transformer: expand_let_star,
                is_syntax_rules: false,
            },
        );

        // letrec macro
        self.macros.insert(
            "letrec".to_string(),
            Macro {
                name: "letrec".to_string(),
                transformer: expand_letrec,
                is_syntax_rules: false,
            },
        );

        // cond macro
        self.macros.insert(
            "cond".to_string(),
            Macro {
                name: "cond".to_string(),
                transformer: expand_cond,
                is_syntax_rules: false,
            },
        );

        // case macro
        self.macros.insert(
            "case".to_string(),
            Macro {
                name: "case".to_string(),
                transformer: expand_case,
                is_syntax_rules: false,
            },
        );

        // when macro
        self.macros.insert(
            "when".to_string(),
            Macro {
                name: "when".to_string(),
                transformer: expand_when,
                is_syntax_rules: false,
            },
        );

        // unless macro
        self.macros.insert(
            "unless".to_string(),
            Macro {
                name: "unless".to_string(),
                transformer: expand_unless,
                is_syntax_rules: false,
            },
        );

        // define-record-type macro (SRFI 9)
        self.macros.insert(
            "define-record-type".to_string(),
            Macro {
                name: "define-record-type".to_string(),
                transformer: expand_define_record_type,
                is_syntax_rules: false,
            },
        );
    }

    /// Check if an expression is a macro call
    pub fn is_macro_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => match &exprs[0] {
                Expr::Variable(name) => self.macros.contains_key(name),
                _ => false,
            },
            _ => false,
        }
    }

    /// Expand a macro call
    pub fn expand_macro(&self, expr: Expr) -> Result<Expr> {
        match &expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                match &exprs[0] {
                    Expr::Variable(name) => {
                        if let Some(macro_def) = self.macros.get(name) {
                            let args = &exprs[1..];
                            (macro_def.transformer)(args)
                        } else {
                            Ok(expr) // Not a macro
                        }
                    }
                    _ => Ok(expr), // Not a macro
                }
            }
            _ => Ok(expr), // Not a macro
        }
    }

    /// Recursively expand all macros in an expression
    pub fn expand_all(&self, expr: Expr) -> Result<Expr> {
        match expr {
            Expr::List(exprs) => {
                // First expand any macro calls
                let expanded = if self.is_macro_call(&Expr::List(exprs.clone())) {
                    self.expand_macro(Expr::List(exprs))?
                } else {
                    Expr::List(exprs)
                };

                // Then recursively expand subexpressions
                match expanded {
                    Expr::List(exprs) => {
                        let mut expanded_exprs = Vec::new();
                        for expr in exprs {
                            expanded_exprs.push(self.expand_all(expr)?);
                        }
                        Ok(Expr::List(expanded_exprs))
                    }
                    other => self.expand_all(other),
                }
            }
            Expr::Quote(expr) => Ok(Expr::Quote(expr)), // Don't expand inside quotes
            Expr::Quasiquote(expr) => {
                // Handle quasiquote expansion specially
                Ok(Expr::Quasiquote(Box::new(self.expand_all(*expr)?)))
            }
            Expr::Unquote(expr) => Ok(Expr::Unquote(Box::new(self.expand_all(*expr)?))),
            Expr::UnquoteSplicing(expr) => {
                Ok(Expr::UnquoteSplicing(Box::new(self.expand_all(*expr)?)))
            }
            Expr::DottedList(exprs, tail) => {
                let mut expanded_exprs = Vec::new();
                for expr in exprs {
                    expanded_exprs.push(self.expand_all(expr)?);
                }
                let expanded_tail = self.expand_all(*tail)?;
                Ok(Expr::DottedList(expanded_exprs, Box::new(expanded_tail)))
            }
            other => Ok(other), // Literals and variables don't need expansion
        }
    }

    /// Define a new macro
    pub fn define_macro(&mut self, name: String, transformer: MacroTransformer) {
        self.macros.insert(
            name.clone(),
            Macro {
                name,
                transformer,
                is_syntax_rules: false,
            },
        );
    }

    /// SRFI 46: Check if pattern has nested ellipsis
    #[allow(dead_code)]
    fn has_nested_ellipsis(&self, pattern: &Pattern) -> bool {
        match pattern {
            Pattern::NestedEllipsis(_, _) => true,
            Pattern::List(patterns) => patterns.iter().any(|p| self.has_nested_ellipsis(p)),
            Pattern::Vector(patterns) => patterns.iter().any(|p| self.has_nested_ellipsis(p)),
            Pattern::Ellipsis(inner) => self.has_nested_ellipsis(inner),
            Pattern::Dotted(patterns, tail) => {
                patterns.iter().any(|p| self.has_nested_ellipsis(p)) || self.has_nested_ellipsis(tail)
            }
            _ => false,
        }
    }

    /// SRFI 46: Count ellipsis nesting level
    fn count_ellipsis_level(&self, expr: &Expr) -> usize {
        match expr {
            Expr::Variable(name) if name == "..." => 1,
            Expr::List(exprs) => {
                exprs.iter().map(|e| self.count_ellipsis_level(e)).max().unwrap_or(0)
            }
            _ => 0,
        }
    }

    /// SRFI 46: Parse pattern with SRFI 46 extensions
    pub fn parse_pattern_srfi46(&self, expr: &Expr) -> Result<Pattern> {
        match expr {
            Expr::Variable(name) => {
                if name == "..." {
                    return Err(LambdustError::syntax_error(
                        "syntax-rules: unexpected ellipsis".to_string(),
                    ));
                }
                Ok(Pattern::Variable(name.clone()))
            }
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    return Ok(Pattern::List(vec![]));
                }

                let mut patterns = Vec::new();
                let mut i = 0;

                while i < exprs.len() {
                    if let Expr::Variable(name) = &exprs[i] {
                        if name == "..." {
                            if patterns.is_empty() {
                                return Err(LambdustError::syntax_error(
                                    "syntax-rules: ellipsis without preceding pattern".to_string(),
                                ));
                            }

                            // Check for nested ellipsis (SRFI 46)
                            let ellipsis_count = if i + 1 < exprs.len() {
                                self.count_ellipsis_level(&exprs[i + 1])
                            } else {
                                0
                            };

                            let last_pattern = patterns.pop().unwrap();
                            if ellipsis_count > 0 {
                                patterns.push(Pattern::NestedEllipsis(Box::new(last_pattern), ellipsis_count + 1));
                                i += ellipsis_count + 1; // Skip the additional ellipses
                            } else {
                                patterns.push(Pattern::Ellipsis(Box::new(last_pattern)));
                                i += 1;
                            }
                            continue;
                        }
                    }

                    patterns.push(self.parse_pattern_srfi46(&exprs[i])?);
                    i += 1;
                }

                Ok(Pattern::List(patterns))
            }
            Expr::Literal(lit) => {
                Ok(Pattern::Literal(format!("{lit:?}")))
            }
            _ => Err(LambdustError::syntax_error(
                "syntax-rules: invalid pattern".to_string(),
            )),
        }
    }

    /// SRFI 46: Parse template with SRFI 46 extensions
    pub fn parse_template_srfi46(&self, expr: &Expr) -> Result<Template> {
        match expr {
            Expr::Variable(name) => {
                if name == "..." {
                    return Err(LambdustError::syntax_error(
                        "syntax-rules: unexpected ellipsis in template".to_string(),
                    ));
                }
                Ok(Template::Variable(name.clone()))
            }
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    return Ok(Template::List(vec![]));
                }

                let mut templates = Vec::new();
                let mut i = 0;

                while i < exprs.len() {
                    if let Expr::Variable(name) = &exprs[i] {
                        if name == "..." {
                            if templates.is_empty() {
                                return Err(LambdustError::syntax_error(
                                    "syntax-rules: ellipsis without preceding template".to_string(),
                                ));
                            }

                            // Check for nested ellipsis (SRFI 46)
                            let ellipsis_count = if i + 1 < exprs.len() {
                                self.count_ellipsis_level(&exprs[i + 1])
                            } else {
                                0
                            };

                            let last_template = templates.pop().unwrap();
                            if ellipsis_count > 0 {
                                templates.push(Template::NestedEllipsis(Box::new(last_template), ellipsis_count + 1));
                                i += ellipsis_count + 1;
                            } else {
                                templates.push(Template::Ellipsis(Box::new(last_template)));
                                i += 1;
                            }
                            continue;
                        }
                    }

                    templates.push(self.parse_template_srfi46(&exprs[i])?);
                    i += 1;
                }

                Ok(Template::List(templates))
            }
            Expr::Literal(lit) => {
                Ok(Template::Literal(format!("{lit:?}")))
            }
            _ => Err(LambdustError::syntax_error(
                "syntax-rules: invalid template".to_string(),
            )),
        }
    }
}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

// Built-in macro transformers

/// Expand let macro: (let ((var val) ...) body ...) -> ((lambda (var ...) body ...) val ...)
fn expand_let(args: &[Expr]) -> Result<Expr> {
    if args.len() < 2 {
        return Err(LambdustError::syntax_error(
            "let: too few arguments".to_string(),
        ));
    }

    let bindings = &args[0];
    let body = &args[1..];

    // Parse bindings
    let binding_list = match bindings {
        Expr::List(bindings) => bindings,
        _ => {
            return Err(LambdustError::syntax_error(
                "let: bindings must be a list".to_string(),
            ));
        }
    };

    let mut vars = Vec::new();
    let mut vals = Vec::new();

    for binding in binding_list {
        match binding {
            Expr::List(parts) if parts.len() == 2 => match &parts[0] {
                Expr::Variable(var) => {
                    vars.push(Expr::Variable(var.clone()));
                    vals.push(parts[1].clone());
                }
                _ => {
                    return Err(LambdustError::syntax_error(
                        "let: binding variable must be a symbol".to_string(),
                    ));
                }
            },
            _ => {
                return Err(LambdustError::syntax_error(
                    "let: each binding must be (var val)".to_string(),
                ));
            }
        }
    }

    // Create lambda expression
    let lambda = Expr::List({
        let mut lambda_expr = vec![Expr::Variable("lambda".to_string()), Expr::List(vars)];
        lambda_expr.extend(body.iter().cloned());
        lambda_expr
    });

    // Create application
    let mut application = vec![lambda];
    application.extend(vals);

    Ok(Expr::List(application))
}

/// Expand let* macro: (let* ((var val) ...) body ...) -> nested lets
fn expand_let_star(args: &[Expr]) -> Result<Expr> {
    if args.len() < 2 {
        return Err(LambdustError::syntax_error(
            "let*: too few arguments".to_string(),
        ));
    }

    let bindings = &args[0];
    let body = &args[1..];

    let binding_list = match bindings {
        Expr::List(bindings) => bindings,
        _ => {
            return Err(LambdustError::syntax_error(
                "let*: bindings must be a list".to_string(),
            ));
        }
    };

    if binding_list.is_empty() {
        // No bindings, just return begin
        return Ok(Expr::List({
            let mut begin_expr = vec![Expr::Variable("begin".to_string())];
            begin_expr.extend(body.iter().cloned());
            begin_expr
        }));
    }

    // Create nested let expressions
    let mut result = Expr::List({
        let mut begin_expr = vec![Expr::Variable("begin".to_string())];
        begin_expr.extend(body.iter().cloned());
        begin_expr
    });

    for binding in binding_list.iter().rev() {
        result = Expr::List(vec![
            Expr::Variable("let".to_string()),
            Expr::List(vec![binding.clone()]),
            result,
        ]);
    }

    Ok(result)
}

/// Expand letrec macro: (letrec ((var val) ...) body ...) ->
/// ((lambda (var ...) (set! var val) ... body ...) #f ...)
fn expand_letrec(args: &[Expr]) -> Result<Expr> {
    if args.len() < 2 {
        return Err(LambdustError::syntax_error(
            "letrec: too few arguments".to_string(),
        ));
    }

    let bindings = &args[0];
    let body = &args[1..];

    let binding_list = match bindings {
        Expr::List(bindings) => bindings,
        _ => {
            return Err(LambdustError::syntax_error(
                "letrec: bindings must be a list".to_string(),
            ));
        }
    };

    let mut vars = Vec::new();
    let mut assignments = Vec::new();
    let mut undefined_vals = Vec::new();

    for binding in binding_list {
        match binding {
            Expr::List(parts) if parts.len() == 2 => {
                match &parts[0] {
                    Expr::Variable(var) => {
                        vars.push(Expr::Variable(var.clone()));
                        assignments.push(Expr::List(vec![
                            Expr::Variable("set!".to_string()),
                            Expr::Variable(var.clone()),
                            parts[1].clone(),
                        ]));
                        undefined_vals.push(Expr::Variable("#f".to_string())); // Use #f as undefined
                    }
                    _ => {
                        return Err(LambdustError::syntax_error(
                            "letrec: binding variable must be a symbol".to_string(),
                        ));
                    }
                }
            }
            _ => {
                return Err(LambdustError::syntax_error(
                    "letrec: each binding must be (var val)".to_string(),
                ));
            }
        }
    }

    // Create lambda expression with assignments
    let mut lambda_body = assignments;
    lambda_body.extend(body.iter().cloned());

    let lambda = Expr::List({
        let mut lambda_expr = vec![Expr::Variable("lambda".to_string()), Expr::List(vars)];
        lambda_expr.extend(lambda_body);
        lambda_expr
    });

    // Create application with undefined values
    let mut application = vec![lambda];
    application.extend(undefined_vals);

    Ok(Expr::List(application))
}

/// Expand cond macro
fn expand_cond(args: &[Expr]) -> Result<Expr> {
    if args.is_empty() {
        return Ok(Expr::Variable("#f".to_string())); // No clauses
    }

    expand_cond_clauses(args)
}

fn expand_cond_clauses(clauses: &[Expr]) -> Result<Expr> {
    if clauses.is_empty() {
        return Ok(Expr::Variable("#f".to_string()));
    }

    let clause = &clauses[0];
    let rest = &clauses[1..];

    match clause {
        Expr::List(parts) if !parts.is_empty() => {
            let test = &parts[0];
            let exprs = &parts[1..];

            // Check for else clause
            if let Expr::Variable(name) = test {
                if name == "else" {
                    if !rest.is_empty() {
                        return Err(LambdustError::syntax_error(
                            "cond: else clause must be last".to_string(),
                        ));
                    }
                    return if exprs.is_empty() {
                        Ok(Expr::Variable("#t".to_string()))
                    } else {
                        Ok(Expr::List({
                            let mut begin_expr = vec![Expr::Variable("begin".to_string())];
                            begin_expr.extend(exprs.iter().cloned());
                            begin_expr
                        }))
                    };
                }
            }

            // Regular clause
            let then_expr = if exprs.is_empty() {
                test.clone()
            } else {
                Expr::List({
                    let mut begin_expr = vec![Expr::Variable("begin".to_string())];
                    begin_expr.extend(exprs.iter().cloned());
                    begin_expr
                })
            };

            let else_expr = expand_cond_clauses(rest)?;

            Ok(Expr::List(vec![
                Expr::Variable("if".to_string()),
                test.clone(),
                then_expr,
                else_expr,
            ]))
        }
        _ => Err(LambdustError::syntax_error(
            "cond: clause must be a list".to_string(),
        )),
    }
}

/// Expand case macro
fn expand_case(args: &[Expr]) -> Result<Expr> {
    if args.len() < 2 {
        return Err(LambdustError::syntax_error(
            "case: too few arguments".to_string(),
        ));
    }

    let key = &args[0];
    let clauses = &args[1..];

    // Generate a unique variable name for the key
    let key_var = "__case_key__";

    let cond_clauses = expand_case_clauses(key_var, clauses)?;

    Ok(Expr::List(vec![
        Expr::Variable("let".to_string()),
        Expr::List(vec![Expr::List(vec![
            Expr::Variable(key_var.to_string()),
            key.clone(),
        ])]),
        cond_clauses,
    ]))
}

fn expand_case_clauses(key_var: &str, clauses: &[Expr]) -> Result<Expr> {
    if clauses.is_empty() {
        return Ok(Expr::Variable("#f".to_string()));
    }

    let clause = &clauses[0];
    let rest = &clauses[1..];

    match clause {
        Expr::List(parts) if parts.len() >= 2 => {
            let datum_list = &parts[0];
            let exprs = &parts[1..];

            // Check for else clause
            if let Expr::Variable(name) = datum_list {
                if name == "else" {
                    if !rest.is_empty() {
                        return Err(LambdustError::syntax_error(
                            "case: else clause must be last".to_string(),
                        ));
                    }
                    return Ok(Expr::List({
                        let mut begin_expr = vec![Expr::Variable("begin".to_string())];
                        begin_expr.extend(exprs.iter().cloned());
                        begin_expr
                    }));
                }
            }

            // Create test expression
            let test = match datum_list {
                Expr::List(datums) => {
                    let mut or_expr = vec![Expr::Variable("or".to_string())];
                    for datum in datums {
                        or_expr.push(Expr::List(vec![
                            Expr::Variable("eqv?".to_string()),
                            Expr::Variable(key_var.to_string()),
                            Expr::Quote(Box::new(datum.clone())),
                        ]));
                    }
                    Expr::List(or_expr)
                }
                single_datum => Expr::List(vec![
                    Expr::Variable("eqv?".to_string()),
                    Expr::Variable(key_var.to_string()),
                    Expr::Quote(Box::new(single_datum.clone())),
                ]),
            };

            let then_expr = Expr::List({
                let mut begin_expr = vec![Expr::Variable("begin".to_string())];
                begin_expr.extend(exprs.iter().cloned());
                begin_expr
            });

            let else_expr = expand_case_clauses(key_var, rest)?;

            Ok(Expr::List(vec![
                Expr::Variable("if".to_string()),
                test,
                then_expr,
                else_expr,
            ]))
        }
        _ => Err(LambdustError::syntax_error(
            "case: clause must be a list".to_string(),
        )),
    }
}

/// Expand when macro: (when test body ...) -> (if test (begin body ...))
fn expand_when(args: &[Expr]) -> Result<Expr> {
    if args.is_empty() {
        return Err(LambdustError::syntax_error(
            "when: too few arguments".to_string(),
        ));
    }

    let test = &args[0];
    let body = &args[1..];

    if body.is_empty() {
        Ok(Expr::List(vec![
            Expr::Variable("if".to_string()),
            test.clone(),
            Expr::Variable("#f".to_string()),
        ]))
    } else {
        Ok(Expr::List(vec![
            Expr::Variable("if".to_string()),
            test.clone(),
            Expr::List({
                let mut begin_expr = vec![Expr::Variable("begin".to_string())];
                begin_expr.extend(body.iter().cloned());
                begin_expr
            }),
        ]))
    }
}

/// Expand unless macro: (unless test body ...) -> (if (not test) (begin body ...))
fn expand_unless(args: &[Expr]) -> Result<Expr> {
    if args.is_empty() {
        return Err(LambdustError::syntax_error(
            "unless: too few arguments".to_string(),
        ));
    }

    let test = &args[0];
    let body = &args[1..];

    let negated_test = Expr::List(vec![Expr::Variable("not".to_string()), test.clone()]);

    if body.is_empty() {
        Ok(Expr::List(vec![
            Expr::Variable("if".to_string()),
            negated_test,
            Expr::Variable("#f".to_string()),
        ]))
    } else {
        Ok(Expr::List(vec![
            Expr::Variable("if".to_string()),
            negated_test,
            Expr::List({
                let mut begin_expr = vec![Expr::Variable("begin".to_string())];
                begin_expr.extend(body.iter().cloned());
                begin_expr
            }),
        ]))
    }
}

/// Expand define-record-type macro (SRFI 9)
/// 
/// Syntax: (define-record-type <type-name>
///           (<constructor> <field-name> ...)
///           <predicate>
///           (<field-name> <accessor> [<modifier>])
///           ...)
/// 
/// Example: (define-record-type point
///            (make-point x y)
///            point?
///            (x point-x set-point-x!)
///            (y point-y set-point-y!))
fn expand_define_record_type(operands: &[Expr]) -> Result<Expr> {
    if operands.len() < 3 {
        return Err(LambdustError::syntax_error(
            "define-record-type: expected at least 3 arguments".to_string(),
        ));
    }

    // Parse type name
    let type_name = match &operands[0] {
        Expr::Variable(name) => name.clone(),
        _ => return Err(LambdustError::syntax_error(
            "define-record-type: type name must be an identifier".to_string(),
        )),
    };

    // Parse constructor specification
    let (constructor_name, field_names) = match &operands[1] {
        Expr::List(exprs) if !exprs.is_empty() => {
            let constructor_name = match &exprs[0] {
                Expr::Variable(name) => name.clone(),
                _ => return Err(LambdustError::syntax_error(
                    "define-record-type: constructor name must be an identifier".to_string(),
                )),
            };
            
            let field_names: Result<Vec<String>> = exprs[1..]
                .iter()
                .map(|expr| match expr {
                    Expr::Variable(name) => Ok(name.clone()),
                    _ => Err(LambdustError::syntax_error(
                        "define-record-type: field names must be identifiers".to_string(),
                    )),
                })
                .collect();
            
            (constructor_name, field_names?)
        },
        _ => return Err(LambdustError::syntax_error(
            "define-record-type: constructor specification must be a list".to_string(),
        )),
    };

    // Parse predicate name
    let predicate_name = match &operands[2] {
        Expr::Variable(name) => name.clone(),
        _ => return Err(LambdustError::syntax_error(
            "define-record-type: predicate name must be an identifier".to_string(),
        )),
    };

    // Parse field specifications
    let mut field_specs = Vec::new();
    for field_spec in &operands[3..] {
        match field_spec {
            Expr::List(exprs) if exprs.len() >= 2 => {
                let field_name = match &exprs[0] {
                    Expr::Variable(name) => name.clone(),
                    _ => return Err(LambdustError::syntax_error(
                        "define-record-type: field name must be an identifier".to_string(),
                    )),
                };
                
                let accessor_name = match &exprs[1] {
                    Expr::Variable(name) => name.clone(),
                    _ => return Err(LambdustError::syntax_error(
                        "define-record-type: accessor name must be an identifier".to_string(),
                    )),
                };
                
                let modifier_name = if exprs.len() >= 3 {
                    match &exprs[2] {
                        Expr::Variable(name) => Some(name.clone()),
                        _ => return Err(LambdustError::syntax_error(
                            "define-record-type: modifier name must be an identifier".to_string(),
                        )),
                    }
                } else {
                    None
                };
                
                field_specs.push((field_name, accessor_name, modifier_name));
            },
            _ => return Err(LambdustError::syntax_error(
                "define-record-type: field specification must be a list".to_string(),
            )),
        }
    }

    // Generate the expanded code
    // This will create:
    // 1. A record type definition 
    // 2. Constructor procedure
    // 3. Predicate procedure
    // 4. Accessor procedures
    // 5. Modifier procedures (if specified)

    let mut definitions = Vec::new();

    // For now, generate a simple begin form with placeholder definitions
    // In a complete implementation, this would generate proper procedures
    // that create and manipulate record instances

    // Generate constructor
    let constructor_args = field_names.iter()
        .map(|name| Expr::Variable(name.clone()))
        .collect();
    
    let constructor_def = Expr::List(vec![
        Expr::Variable("define".to_string()),
        Expr::Variable(constructor_name),
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(constructor_args),
            // TODO: Create actual record construction logic
            Expr::List(vec![
                Expr::Variable("make-record".to_string()),
                Expr::Quote(Box::new(Expr::Variable(type_name.clone()))),
                Expr::List(vec![
                    Expr::Variable("list".to_string())
                ].into_iter().chain(field_names.iter().map(|name| Expr::Variable(name.clone()))).collect()),
            ]),
        ]),
    ]);
    definitions.push(constructor_def);

    // Generate predicate
    let predicate_def = Expr::List(vec![
        Expr::Variable("define".to_string()),
        Expr::Variable(predicate_name),
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![Expr::Variable("obj".to_string())]),
            Expr::List(vec![
                Expr::Variable("record-of-type?".to_string()),
                Expr::Variable("obj".to_string()),
                Expr::Quote(Box::new(Expr::Variable(type_name.clone()))),
            ]),
        ]),
    ]);
    definitions.push(predicate_def);

    // Generate accessors and modifiers
    for (i, (_field_name, accessor_name, modifier_name)) in field_specs.into_iter().enumerate() {
        // Accessor
        let accessor_def = Expr::List(vec![
            Expr::Variable("define".to_string()),
            Expr::Variable(accessor_name),
            Expr::List(vec![
                Expr::Variable("lambda".to_string()),
                Expr::List(vec![Expr::Variable("record".to_string())]),
                Expr::List(vec![
                    Expr::Variable("record-field".to_string()),
                    Expr::Variable("record".to_string()),
                    Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(i as i64))),
                ]),
            ]),
        ]);
        definitions.push(accessor_def);

        // Modifier (if specified)
        if let Some(modifier_name) = modifier_name {
            let modifier_def = Expr::List(vec![
                Expr::Variable("define".to_string()),
                Expr::Variable(modifier_name),
                Expr::List(vec![
                    Expr::Variable("lambda".to_string()),
                    Expr::List(vec![
                        Expr::Variable("record".to_string()),
                        Expr::Variable("value".to_string()),
                    ]),
                    Expr::List(vec![
                        Expr::Variable("record-set-field!".to_string()),
                        Expr::Variable("record".to_string()),
                        Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(i as i64))),
                        Expr::Variable("value".to_string()),
                    ]),
                ]),
            ]);
            definitions.push(modifier_def);
        }
    }

    // Return a begin form with all definitions
    Ok(Expr::List(vec![
        Expr::Variable("begin".to_string())
    ].into_iter().chain(definitions).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn parse_expr(input: &str) -> Expr {
        let tokens = tokenize(input).unwrap();
        parse(tokens).unwrap()
    }

    #[test]
    fn test_expand_let() {
        let expander = MacroExpander::new();
        let expr = parse_expr("(let ((x 1) (y 2)) (+ x y))");
        let expanded = expander.expand_macro(expr).unwrap();

        // Should expand to ((lambda (x y) (+ x y)) 1 2)
        match expanded {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert!(matches!(exprs[0], Expr::List(_))); // lambda expression
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_expand_cond() {
        let expander = MacroExpander::new();
        let expr = parse_expr("(cond ((< x 0) 'negative) ((> x 0) 'positive) (else 'zero))");
        let expanded = expander.expand_macro(expr).unwrap();

        // Should expand to nested if expressions
        match expanded {
            Expr::List(exprs) => {
                assert_eq!(exprs[0], Expr::Variable("if".to_string()));
            }
            _ => panic!("Expected if expression"),
        }
    }

    #[test]
    fn test_expand_when() {
        let expander = MacroExpander::new();
        let expr = parse_expr("(when (> x 0) (display x) (newline))");
        let expanded = expander.expand_macro(expr).unwrap();

        // Should expand to (if (> x 0) (begin (display x) (newline)))
        match expanded {
            Expr::List(exprs) => {
                assert_eq!(exprs[0], Expr::Variable("if".to_string()));
                assert_eq!(exprs.len(), 3);
            }
            _ => panic!("Expected if expression"),
        }
    }

    #[test]
    fn test_is_macro_call() {
        let expander = MacroExpander::new();
        let let_expr = parse_expr("(let ((x 1)) x)");
        let regular_expr = parse_expr("(+ 1 2)");

        assert!(expander.is_macro_call(&let_expr));
        assert!(!expander.is_macro_call(&regular_expr));
    }

    #[test]
    fn test_srfi_46_ellipsis_count() {
        let expander = MacroExpander::new();
        
        // Test ellipsis variable
        let ellipsis_var = Expr::Variable("...".to_string());
        assert_eq!(expander.count_ellipsis_level(&ellipsis_var), 1);
        
        // Test nested ellipsis in list  
        let nested = Expr::List(vec![
            Expr::Variable("...".to_string()),
            Expr::List(vec![Expr::Variable("...".to_string())])
        ]);
        assert_eq!(expander.count_ellipsis_level(&nested), 1);
    }

    #[test]
    fn test_srfi_46_pattern_parsing() {
        let expander = MacroExpander::new();
        
        // Test simple pattern
        let simple = Expr::Variable("x".to_string());
        let pattern = expander.parse_pattern_srfi46(&simple).unwrap();
        assert_eq!(pattern, Pattern::Variable("x".to_string()));
        
        // Test list pattern
        let list = Expr::List(vec![
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
            Expr::Variable("z".to_string()),
        ]);
        let pattern = expander.parse_pattern_srfi46(&list).unwrap();
        assert!(matches!(pattern, Pattern::List(_)));
        
        // Test ellipsis pattern
        let ellipsis = Expr::List(vec![
            Expr::Variable("x".to_string()),
            Expr::Variable("...".to_string()),
        ]);
        let pattern = expander.parse_pattern_srfi46(&ellipsis).unwrap();
        if let Pattern::List(patterns) = pattern {
            assert_eq!(patterns.len(), 1);
            assert!(matches!(patterns[0], Pattern::Ellipsis(_)));
        } else {
            panic!("Expected list pattern with ellipsis");
        }
    }

    #[test] 
    fn test_srfi_46_template_parsing() {
        let expander = MacroExpander::new();
        
        // Test simple template
        let simple = Expr::Variable("x".to_string());
        let template = expander.parse_template_srfi46(&simple).unwrap();
        assert_eq!(template, Template::Variable("x".to_string()));
        
        // Test list template
        let list = Expr::List(vec![
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
            Expr::Variable("z".to_string()),
        ]);
        let template = expander.parse_template_srfi46(&list).unwrap();
        assert!(matches!(template, Template::List(_)));
        
        // Test ellipsis template  
        let ellipsis = Expr::List(vec![
            Expr::Variable("x".to_string()),
            Expr::Variable("...".to_string()),
        ]);
        let template = expander.parse_template_srfi46(&ellipsis).unwrap();
        if let Template::List(templates) = template {
            assert_eq!(templates.len(), 1);
            assert!(matches!(templates[0], Template::Ellipsis(_)));
        } else {
            panic!("Expected list template with ellipsis");
        }
    }
}
