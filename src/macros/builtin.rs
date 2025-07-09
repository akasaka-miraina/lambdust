//! Built-in macro transformers for Scheme
//!
//! This module contains implementations of built-in macros such as let, cond, case,
//! and define-record-type. These macros are implemented as transformer functions
//! that take argument expressions and return expanded expressions.

use crate::ast::{Expr, Literal};
use crate::error::{LambdustError, Result};
use crate::lexer::SchemeNumber;

/// Expand let macro: (let ((var val) ...) body ...) -> ((lambda (var ...) body ...) val ...)
pub fn expand_let(args: &[Expr]) -> Result<Expr> {
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
pub fn expand_let_star(args: &[Expr]) -> Result<Expr> {
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
pub fn expand_letrec(args: &[Expr]) -> Result<Expr> {
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
                        undefined_values.push(Expr::Variable("#f".to_string()));
                        // Use #f as undefined
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
pub fn expand_cond(args: &[Expr]) -> Result<Expr> {
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
pub fn expand_case(args: &[Expr]) -> Result<Expr> {
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
pub fn expand_when(args: &[Expr]) -> Result<Expr> {
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
pub fn expand_unless(args: &[Expr]) -> Result<Expr> {
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
pub fn expand_define_record_type(operands: &[Expr]) -> Result<Expr> {
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
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(i as i64))),
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
                        Expr::Literal(Literal::Number(SchemeNumber::Integer(i as i64))),
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
