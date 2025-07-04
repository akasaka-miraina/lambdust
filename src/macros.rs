//! Macro system implementation for Scheme

use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use std::collections::HashMap;

/// Macro transformer type
pub type MacroTransformer = fn(&[Expr]) -> Result<Expr>;

/// Macro definition
#[derive(Debug, Clone)]
pub enum Macro {
    /// Built-in macro with transformer function
    Builtin {
        /// Name of the builtin macro
        name: String,
        /// Transformer function for the macro
        transformer: MacroTransformer,
    },
    /// Syntax-rules macro with pattern/template rules
    SyntaxRules {
        /// Name of the syntax-rules macro
        name: String,
        /// Transformer implementing syntax-rules pattern matching
        transformer: SyntaxRulesTransformer,
    },
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

/// Variable bindings from pattern matching
pub type VariableBindings = HashMap<String, BindingValue>;

/// Value bound to a pattern variable
#[derive(Debug, Clone, PartialEq)]
pub enum BindingValue {
    /// Single expression binding
    Single(Expr),
    /// Multiple expressions (from ellipsis)
    Multiple(Vec<Expr>),
    /// Nested bindings (from nested ellipsis)
    Nested(Vec<BindingValue>),
}

/// Syntax-rules transformer for generic macro definitions
#[derive(Debug, Clone)]
pub struct SyntaxRulesTransformer {
    /// List of literals (identifiers that must match exactly)
    pub literals: Vec<String>,
    /// List of transformation rules
    pub rules: Vec<SyntaxRule>,
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

    /// Define a new syntax-rules macro
    pub fn define_syntax_rules_macro(
        &mut self,
        name: String,
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
    ) {
        let transformer = SyntaxRulesTransformer::new(literals, rules);
        self.macros
            .insert(name.clone(), Macro::SyntaxRules { name, transformer });
    }

    /// Add built-in macros
    fn add_builtin_macros(&mut self) {
        // let macro
        self.macros.insert(
            "let".to_string(),
            Macro::Builtin {
                name: "let".to_string(),
                transformer: expand_let,
            },
        );

        // let* macro
        self.macros.insert(
            "let*".to_string(),
            Macro::Builtin {
                name: "let*".to_string(),
                transformer: expand_let_star,
            },
        );

        // letrec macro
        self.macros.insert(
            "letrec".to_string(),
            Macro::Builtin {
                name: "letrec".to_string(),
                transformer: expand_letrec,
            },
        );

        // cond macro
        self.macros.insert(
            "cond".to_string(),
            Macro::Builtin {
                name: "cond".to_string(),
                transformer: expand_cond,
            },
        );

        // case macro
        self.macros.insert(
            "case".to_string(),
            Macro::Builtin {
                name: "case".to_string(),
                transformer: expand_case,
            },
        );

        // when macro
        self.macros.insert(
            "when".to_string(),
            Macro::Builtin {
                name: "when".to_string(),
                transformer: expand_when,
            },
        );

        // unless macro
        self.macros.insert(
            "unless".to_string(),
            Macro::Builtin {
                name: "unless".to_string(),
                transformer: expand_unless,
            },
        );

        // define-record-type macro (SRFI 9)
        self.macros.insert(
            "define-record-type".to_string(),
            Macro::Builtin {
                name: "define-record-type".to_string(),
                transformer: expand_define_record_type,
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
                            match macro_def {
                                Macro::Builtin { transformer, .. } => {
                                    let args = &exprs[1..];
                                    transformer(args)
                                }
                                Macro::SyntaxRules { transformer, .. } => {
                                    transformer.transform(&expr)
                                }
                            }
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
            Expr::Vector(exprs) => {
                let mut expanded_exprs = Vec::new();
                for expr in exprs {
                    expanded_exprs.push(self.expand_all(expr)?);
                }
                Ok(Expr::Vector(expanded_exprs))
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
        self.macros
            .insert(name.clone(), Macro::Builtin { name, transformer });
    }

    // Removed the `has_nested_ellipsis` method as it was dead code.

    /// SRFI 46: Count ellipsis nesting level
    pub fn count_ellipsis_level(expr: &Expr) -> usize {
        match expr {
            Expr::Variable(name) if name == "..." => 1,
            Expr::List(exprs) => exprs
                .iter()
                .map(Self::count_ellipsis_level)
                .max()
                .unwrap_or(0),
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
                                Self::count_ellipsis_level(&exprs[i + 1])
                            } else {
                                0
                            };

                            let last_pattern = patterns.pop().unwrap();
                            if ellipsis_count > 0 {
                                patterns.push(Pattern::NestedEllipsis(
                                    Box::new(last_pattern),
                                    ellipsis_count + 1,
                                ));
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
            Expr::Literal(lit) => Ok(Pattern::Literal(format!("{lit:?}"))),
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
                                Self::count_ellipsis_level(&exprs[i + 1])
                            } else {
                                0
                            };

                            let last_template = templates.pop().unwrap();
                            if ellipsis_count > 0 {
                                templates.push(Template::NestedEllipsis(
                                    Box::new(last_template),
                                    ellipsis_count + 1,
                                ));
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
            Expr::Literal(lit) => Ok(Template::Literal(format!("{lit:?}"))),
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

/// Public helper function for expanding macros by name
pub fn expand_macro(name: &str, args: &[Expr]) -> Result<Expr> {
    let _expander = MacroExpander::new();
    match name {
        "let" => expand_let(args),
        "let*" => expand_let_star(args),
        "letrec" => expand_letrec(args),
        "case" => expand_case(args),
        "when" => expand_when(args),
        "unless" => expand_unless(args),
        _ => Err(LambdustError::syntax_error(format!(
            "Unknown macro: {name}"
        ))),
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

    let mut variables = Vec::new();
    let mut values = Vec::new();

    for binding in binding_list {
        match binding {
            Expr::List(parts) if parts.len() == 2 => match &parts[0] {
                Expr::Variable(var) => {
                    variables.push(Expr::Variable(var.clone()));
                    values.push(parts[1].clone());
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
        let mut lambda_expr = vec![Expr::Variable("lambda".to_string()), Expr::List(variables)];
        lambda_expr.extend(body.iter().cloned());
        lambda_expr
    });

    // Create application
    let mut application = vec![lambda];
    application.extend(values);

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

    let mut variables = Vec::new();
    let mut assignments = Vec::new();
    let mut undefined_values = Vec::new();

    for binding in binding_list {
        match binding {
            Expr::List(parts) if parts.len() == 2 => {
                match &parts[0] {
                    Expr::Variable(var) => {
                        variables.push(Expr::Variable(var.clone()));
                        assignments.push(Expr::List(vec![
                            Expr::Variable("set!".to_string()),
                            Expr::Variable(var.clone()),
                            parts[1].clone(),
                        ]));
                        undefined_values.push(Expr::Variable("#f".to_string())); // Use #f as undefined
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
        let mut lambda_expr = vec![Expr::Variable("lambda".to_string()), Expr::List(variables)];
        lambda_expr.extend(lambda_body);
        lambda_expr
    });

    // Create application with undefined values
    let mut application = vec![lambda];
    application.extend(undefined_values);

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
        _ => {
            return Err(LambdustError::syntax_error(
                "define-record-type: type name must be an identifier".to_string(),
            ));
        }
    };

    // Parse constructor specification
    let (constructor_name, field_names) = match &operands[1] {
        Expr::List(exprs) if !exprs.is_empty() => {
            let constructor_name = match &exprs[0] {
                Expr::Variable(name) => name.clone(),
                _ => {
                    return Err(LambdustError::syntax_error(
                        "define-record-type: constructor name must be an identifier".to_string(),
                    ));
                }
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
        }
        _ => {
            return Err(LambdustError::syntax_error(
                "define-record-type: constructor specification must be a list".to_string(),
            ));
        }
    };

    // Parse predicate name
    let predicate_name = match &operands[2] {
        Expr::Variable(name) => name.clone(),
        _ => {
            return Err(LambdustError::syntax_error(
                "define-record-type: predicate name must be an identifier".to_string(),
            ));
        }
    };

    // Parse field specifications
    let mut field_specs = Vec::new();
    for field_spec in &operands[3..] {
        match field_spec {
            Expr::List(exprs) if exprs.len() >= 2 => {
                let field_name = match &exprs[0] {
                    Expr::Variable(name) => name.clone(),
                    _ => {
                        return Err(LambdustError::syntax_error(
                            "define-record-type: field name must be an identifier".to_string(),
                        ));
                    }
                };

                let accessor_name = match &exprs[1] {
                    Expr::Variable(name) => name.clone(),
                    _ => {
                        return Err(LambdustError::syntax_error(
                            "define-record-type: accessor name must be an identifier".to_string(),
                        ));
                    }
                };

                let modifier_name = if exprs.len() >= 3 {
                    match &exprs[2] {
                        Expr::Variable(name) => Some(name.clone()),
                        _ => {
                            return Err(LambdustError::syntax_error(
                                "define-record-type: modifier name must be an identifier"
                                    .to_string(),
                            ));
                        }
                    }
                } else {
                    None
                };

                field_specs.push((field_name, accessor_name, modifier_name));
            }
            _ => {
                return Err(LambdustError::syntax_error(
                    "define-record-type: field specification must be a list".to_string(),
                ));
            }
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
    let constructor_args = field_names
        .iter()
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
                Expr::List(
                    vec![Expr::Variable("list".to_string())]
                        .into_iter()
                        .chain(field_names.iter().map(|name| Expr::Variable(name.clone())))
                        .collect(),
                ),
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
                    Expr::Literal(crate::ast::Literal::Number(
                        crate::lexer::SchemeNumber::Integer(i as i64),
                    )),
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
                        Expr::Literal(crate::ast::Literal::Number(
                            crate::lexer::SchemeNumber::Integer(i as i64),
                        )),
                        Expr::Variable("value".to_string()),
                    ]),
                ]),
            ]);
            definitions.push(modifier_def);
        }
    }

    // Return a begin form with all definitions
    Ok(Expr::List(
        vec![Expr::Variable("begin".to_string())]
            .into_iter()
            .chain(definitions)
            .collect(),
    ))
}

impl SyntaxRulesTransformer {
    /// Create a new syntax-rules transformer
    pub fn new(literals: Vec<String>, rules: Vec<SyntaxRule>) -> Self {
        Self { literals, rules }
    }

    /// Transform input expression using syntax-rules
    pub fn transform(&self, expr: &Expr) -> Result<Expr> {
        // Try each rule in order until one matches
        for rule in &self.rules {
            if let Ok(bindings) = self.pattern_match(&rule.pattern, expr) {
                return self.template_expand(&rule.template, &bindings);
            }
        }

        Err(LambdustError::macro_error_old(format!(
            "No syntax-rules pattern matched: {expr:?}"
        )))
    }

    /// Match pattern against expression, returning variable bindings
    fn pattern_match(&self, pattern: &Pattern, expr: &Expr) -> Result<VariableBindings> {
        let mut bindings = HashMap::new();
        self.pattern_match_impl(pattern, expr, &mut bindings)?;
        Ok(bindings)
    }

    /// Implementation of pattern matching
    fn pattern_match_impl(
        &self,
        pattern: &Pattern,
        expr: &Expr,
        bindings: &mut VariableBindings,
    ) -> Result<()> {
        match (pattern, expr) {
            // Literal patterns must match exactly
            (Pattern::Literal(lit), Expr::Variable(var)) => {
                if lit == var || self.literals.contains(lit) {
                    Ok(())
                } else {
                    Err(LambdustError::macro_error_old(format!(
                        "Literal mismatch: expected {lit}, got {var}"
                    )))
                }
            }

            // Variable patterns bind to any expression
            (Pattern::Variable(var), expr) => {
                if self.literals.contains(var) {
                    // Literal variable must match exactly
                    if let Expr::Variable(expr_var) = expr {
                        if var == expr_var {
                            Ok(())
                        } else {
                            Err(LambdustError::macro_error_old(format!(
                                "Literal variable mismatch: expected {var}, got {expr_var}"
                            )))
                        }
                    } else {
                        Err(LambdustError::macro_error_old(format!(
                            "Expected literal {var}, got expression: {expr:?}"
                        )))
                    }
                } else {
                    // Pattern variable binds to expression
                    bindings.insert(var.clone(), BindingValue::Single(expr.clone()));
                    Ok(())
                }
            }

            // List patterns
            (Pattern::List(patterns), Expr::List(exprs)) => {
                self.match_list_patterns(patterns, exprs, bindings)
            }

            // Vector patterns (SRFI 46) - treating as lists for now
            (Pattern::Vector(patterns), Expr::List(exprs)) => {
                self.match_vector_patterns(patterns, exprs, bindings)
            }

            // Dotted patterns
            (Pattern::Dotted(patterns, rest_pattern), Expr::List(exprs)) => {
                self.match_dotted_patterns(patterns, rest_pattern, exprs, bindings)
            }

            // Ellipsis patterns
            (Pattern::Ellipsis(_pattern), _) => {
                // This should be handled by list pattern matching
                Err(LambdustError::macro_error_old(
                    "Ellipsis pattern not in list context".to_string(),
                ))
            }

            // Nested ellipsis patterns (SRFI 46)
            (Pattern::NestedEllipsis(_pattern, _level), _) => {
                // This should be handled by list pattern matching
                Err(LambdustError::macro_error_old(
                    "Nested ellipsis pattern not in list context".to_string(),
                ))
            }

            // Type mismatches
            _ => Err(LambdustError::macro_error_old(format!(
                "Pattern type mismatch: {pattern:?} vs {expr:?}"
            ))),
        }
    }

    /// Match list patterns with ellipsis support
    fn match_list_patterns(
        &self,
        patterns: &[Pattern],
        exprs: &[Expr],
        bindings: &mut VariableBindings,
    ) -> Result<()> {
        let mut pattern_idx = 0;
        let mut expr_idx = 0;

        while pattern_idx < patterns.len() && expr_idx < exprs.len() {
            match &patterns[pattern_idx] {
                Pattern::Ellipsis(ellipsis_pattern) => {
                    // Match zero or more expressions with the ellipsis pattern
                    let mut matched_exprs = Vec::new();

                    // Determine how many expressions to match
                    let remaining_patterns = patterns.len() - pattern_idx - 1;
                    let remaining_exprs = exprs.len() - expr_idx;

                    if remaining_exprs >= remaining_patterns {
                        let ellipsis_count = remaining_exprs - remaining_patterns;

                        for _ in 0..ellipsis_count {
                            matched_exprs.push(exprs[expr_idx].clone());
                            expr_idx += 1;
                        }

                        // Store ellipsis bindings
                        self.store_ellipsis_bindings(ellipsis_pattern, &matched_exprs, bindings)?;
                    }

                    pattern_idx += 1;
                }

                Pattern::NestedEllipsis(ellipsis_pattern, level) => {
                    // Handle nested ellipsis (SRFI 46)
                    self.match_nested_ellipsis(
                        ellipsis_pattern,
                        *level,
                        &exprs[expr_idx..],
                        bindings,
                    )?;
                    // For now, consume all remaining expressions
                    expr_idx = exprs.len();
                    pattern_idx += 1;
                }

                _ => {
                    // Regular pattern matching
                    self.pattern_match_impl(&patterns[pattern_idx], &exprs[expr_idx], bindings)?;
                    pattern_idx += 1;
                    expr_idx += 1;
                }
            }
        }

        // Check if all patterns and expressions were consumed
        if pattern_idx < patterns.len() || expr_idx < exprs.len() {
            Err(LambdustError::macro_error_old(format!(
                "List length mismatch: {} patterns vs {} expressions",
                patterns.len(),
                exprs.len()
            )))
        } else {
            Ok(())
        }
    }

    /// Match vector patterns
    fn match_vector_patterns(
        &self,
        patterns: &[Pattern],
        exprs: &[Expr],
        bindings: &mut VariableBindings,
    ) -> Result<()> {
        // For vectors, we can reuse list pattern matching logic
        self.match_list_patterns(patterns, exprs, bindings)
    }

    /// Match dotted patterns  
    fn match_dotted_patterns(
        &self,
        patterns: &[Pattern],
        rest_pattern: &Pattern,
        exprs: &[Expr],
        bindings: &mut VariableBindings,
    ) -> Result<()> {
        if exprs.len() < patterns.len() {
            return Err(LambdustError::macro_error_old(
                "Not enough expressions for dotted pattern".to_string(),
            ));
        }

        // Match fixed patterns
        for (i, pattern) in patterns.iter().enumerate() {
            self.pattern_match_impl(pattern, &exprs[i], bindings)?;
        }

        // Match rest pattern with remaining expressions
        let rest_exprs = &exprs[patterns.len()..];
        let rest_list = Expr::List(rest_exprs.to_vec());
        self.pattern_match_impl(rest_pattern, &rest_list, bindings)?;

        Ok(())
    }

    /// Store ellipsis bindings for multiple matched expressions
    fn store_ellipsis_bindings(
        &self,
        pattern: &Pattern,
        exprs: &[Expr],
        bindings: &mut VariableBindings,
    ) -> Result<()> {
        match pattern {
            Pattern::Variable(var) => {
                if !self.literals.contains(var) {
                    bindings.insert(var.clone(), BindingValue::Multiple(exprs.to_vec()));
                }
                Ok(())
            }
            Pattern::List(patterns) => {
                // Handle ellipsis within list patterns
                for expr in exprs {
                    if let Expr::List(sub_exprs) = expr {
                        self.match_list_patterns(patterns, sub_exprs, bindings)?;
                    } else {
                        return Err(LambdustError::macro_error_old(
                            "Expected list in ellipsis pattern".to_string(),
                        ));
                    }
                }
                Ok(())
            }
            _ => Err(LambdustError::macro_error_old(format!(
                "Unsupported ellipsis pattern: {pattern:?}"
            ))),
        }
    }

    /// Handle nested ellipsis patterns (SRFI 46)
    fn match_nested_ellipsis(
        &self,
        _pattern: &Pattern,
        _level: usize,
        _exprs: &[Expr],
        _bindings: &mut VariableBindings,
    ) -> Result<()> {
        // Placeholder for nested ellipsis implementation
        // This is a complex feature that requires careful handling of nesting levels
        Ok(())
    }

    /// Expand template using variable bindings
    fn template_expand(&self, template: &Template, bindings: &VariableBindings) -> Result<Expr> {
        match template {
            Template::Literal(lit) => Ok(Expr::Variable(lit.clone())),

            Template::Variable(var) => {
                if let Some(binding) = bindings.get(var) {
                    match binding {
                        BindingValue::Single(expr) => Ok(expr.clone()),
                        BindingValue::Multiple(_exprs) => Err(LambdustError::macro_error_old(
                            format!("Variable {var} bound to multiple values, not single value"),
                        )),
                        BindingValue::Nested(_nested) => Err(LambdustError::macro_error_old(
                            format!("Variable {var} bound to nested values, not single value"),
                        )),
                    }
                } else {
                    // Unbound variable becomes literal
                    Ok(Expr::Variable(var.clone()))
                }
            }

            Template::List(templates) => {
                let mut result_exprs = Vec::new();

                for template in templates {
                    match template {
                        Template::Ellipsis(ellipsis_template) => {
                            let expanded =
                                self.expand_ellipsis_template(ellipsis_template, bindings)?;
                            result_exprs.extend(expanded);
                        }
                        _ => {
                            let expanded = self.template_expand(template, bindings)?;
                            result_exprs.push(expanded);
                        }
                    }
                }

                Ok(Expr::List(result_exprs))
            }

            Template::Vector(templates) => {
                let mut result_exprs = Vec::new();

                for template in templates {
                    let expanded = self.template_expand(template, bindings)?;
                    result_exprs.push(expanded);
                }

                Ok(Expr::List(result_exprs)) // Vector support pending AST update
            }

            Template::Dotted(templates, rest_template) => {
                let mut result_exprs = Vec::new();

                for template in templates {
                    let expanded = self.template_expand(template, bindings)?;
                    result_exprs.push(expanded);
                }

                let rest_expanded = self.template_expand(rest_template, bindings)?;
                // For simplicity, add rest as final element (proper dotted list handling would be more complex)
                result_exprs.push(rest_expanded);

                Ok(Expr::List(result_exprs))
            }

            Template::Ellipsis(_) => Err(LambdustError::macro_error_old(
                "Ellipsis template not in list context".to_string(),
            )),

            Template::NestedEllipsis(_template, _level) => {
                // Placeholder for nested ellipsis expansion
                Ok(Expr::List(Vec::new()))
            }
        }
    }

    /// Expand ellipsis template
    fn expand_ellipsis_template(
        &self,
        template: &Template,
        bindings: &VariableBindings,
    ) -> Result<Vec<Expr>> {
        match template {
            Template::Variable(var) => {
                if let Some(binding) = bindings.get(var) {
                    match binding {
                        BindingValue::Multiple(exprs) => Ok(exprs.clone()),
                        BindingValue::Single(expr) => Ok(vec![expr.clone()]),
                        BindingValue::Nested(_) => Ok(Vec::new()), // Placeholder
                    }
                } else {
                    Ok(Vec::new())
                }
            }

            Template::List(templates) => {
                // For list templates in ellipsis, we need to coordinate expansion
                // This is a simplified implementation
                let mut result = Vec::new();

                // Find the first ellipsis-bound variable to determine iteration count
                let mut max_count = 0;
                for template in templates {
                    if let Template::Variable(var) = template {
                        if let Some(BindingValue::Multiple(exprs)) = bindings.get(var) {
                            max_count = max_count.max(exprs.len());
                        }
                    }
                }

                // Generate expressions for each iteration
                for i in 0..max_count {
                    let mut iter_exprs = Vec::new();

                    for template in templates {
                        match template {
                            Template::Variable(var) => {
                                if let Some(binding) = bindings.get(var) {
                                    match binding {
                                        BindingValue::Multiple(exprs) => {
                                            if i < exprs.len() {
                                                iter_exprs.push(exprs[i].clone());
                                            }
                                        }
                                        BindingValue::Single(expr) => {
                                            iter_exprs.push(expr.clone());
                                        }
                                        BindingValue::Nested(_) => {} // Placeholder
                                    }
                                } else {
                                    iter_exprs.push(Expr::Variable(var.clone()));
                                }
                            }
                            _ => {
                                let expanded = self.template_expand(template, bindings)?;
                                iter_exprs.push(expanded);
                            }
                        }
                    }

                    result.push(Expr::List(iter_exprs));
                }

                Ok(result)
            }

            _ => {
                let expanded = self.template_expand(template, bindings)?;
                Ok(vec![expanded])
            }
        }
    }
}
