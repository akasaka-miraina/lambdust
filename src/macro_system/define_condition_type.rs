// SRFI-35 define-condition-type macro implementation
//
// This module implements the define-condition-type macro which is central to SRFI-35.
// The macro expands condition type definitions into the necessary runtime structure
// registrations and procedure definitions.

use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error as DiagnosticError, Result, Span, Spanned};
use crate::eval::Environment;
use crate::utils::SymbolId;
use std::collections::HashMap;
use std::sync::Arc;

/// Macro transformer for define-condition-type
pub struct DefineConditionTypeMacro {
    /// Symbol interner for creating symbols
    interner: Arc<crate::utils::string_interner::StringInterner>,
}

impl DefineConditionTypeMacro {
    /// Creates a new define-condition-type macro transformer
    pub fn new(interner: Arc<crate::utils::string_interner::StringInterner>) -> Self {
        Self { interner }
    }

    /// Transforms a define-condition-type form into the expansion
    /// 
    /// Syntax: (define-condition-type <condition-type> <supertype> <constructor> <predicate>
    ///           (<field> <accessor>) ...)
    ///
    /// Expands to:
    /// 1. Registration of the condition type in the global registry
    /// 2. Definition of the constructor procedure
    /// 3. Definition of the predicate procedure 
    /// 4. Definition of field accessor procedures
    pub fn transform(&self, args: &[Spanned<Expr>], span: Span) -> Result<Spanned<Expr>> {
        if args.len() < 4 {
            return Err(Box::new(DiagnosticError::syntax_error(
                "define-condition-type requires at least 4 arguments: type, supertype, constructor, predicate".to_string(),
                Some(span),
            )));
        }

        // Parse the arguments
        let condition_type = self.extract_symbol(&args[0])?;
        let supertype = self.extract_symbol(&args[1])?;
        let constructor = self.extract_symbol(&args[2])?;
        let predicate = self.extract_symbol(&args[3])?;

        // Parse field specifications: (field accessor) pairs
        let mut field_specs = Vec::new();
        for arg in &args[4..] {
            let (field, accessor) = self.extract_field_spec(arg)?;
            field_specs.push((field, accessor));
        }

        // Generate the expansion
        self.generate_expansion(
            condition_type,
            supertype,
            constructor,
            predicate,
            field_specs,
            span,
        )
    }

    /// Extracts a symbol from an expression
    fn extract_symbol(&self, expr: &Spanned<Expr>) -> Result<String> {
        match &expr.inner {
            Expr::Symbol(symbol_str) => {
                Ok(symbol_str.clone())
            }
            _ => Err(Box::new(DiagnosticError::syntax_error(
                format!("Expected symbol, got {:?}", expr.inner),
                Some(expr.span),
            ))),
        }
    }

    /// Extracts a field specification (field accessor) pair
    fn extract_field_spec(&self, expr: &Spanned<Expr>) -> Result<(String, String)> {
        match &expr.inner {
            Expr::List(items) => {
                if items.len() != 2 {
                    return Err(Box::new(DiagnosticError::syntax_error(
                        "Field specification must be (field accessor)".to_string(),
                        Some(expr.span),
                    )));
                }

                let field = self.extract_symbol(&items[0])?;
                let accessor = self.extract_symbol(&items[1])?;
                Ok((field, accessor))
            }
            _ => Err(Box::new(DiagnosticError::syntax_error(
                "Field specification must be a list (field accessor)".to_string(),
                Some(expr.span),
            ))),
        }
    }

    /// Generates the macro expansion
    fn generate_expansion(
        &self,
        condition_type: String,
        supertype: String,
        constructor: String,
        predicate: String,
        field_specs: Vec<(String, String)>,
        span: Span,
    ) -> Result<Spanned<Expr>> {
        let mut expansions = Vec::new();

        // 1. Register the condition type in the global registry
        let registration_form = self.generate_type_registration(
            &condition_type,
            &supertype,
            &constructor,
            &predicate,
            &field_specs,
            span,
        )?;
        expansions.push(registration_form);

        // 2. Define the constructor procedure
        let constructor_form = self.generate_constructor(
            &condition_type,
            &constructor,
            &field_specs,
            span,
        )?;
        expansions.push(constructor_form);

        // 3. Define the predicate procedure
        let predicate_form = self.generate_predicate(
            &condition_type,
            &predicate,
            span,
        )?;
        expansions.push(predicate_form);

        // 4. Define field accessor procedures
        for (field, accessor) in &field_specs {
            let accessor_form = self.generate_field_accessor(
                &condition_type,
                field,
                accessor,
                span,
            )?;
            expansions.push(accessor_form);
        }

        // Wrap all expansions in a begin form
        let begin_expr = Expr::List(
            std::iter::once(self.make_symbol("begin", span))
                .chain(expansions)
                .collect()
        );

        Ok(Spanned::new(begin_expr, span))
    }

    /// Generates the condition type registration form
    fn generate_type_registration(
        &self,
        condition_type: &str,
        supertype: &str,
        constructor: &str,
        predicate: &str,
        field_specs: &[(String, String)],
        span: Span,
    ) -> Result<Spanned<Expr>> {
        // (%register-condition-type "type-name" "supertype-name" "constructor-name" 
        //                          "predicate-name" '(("field1" "accessor1") ...))
        
        let field_list_items: Vec<Spanned<Expr>> = field_specs
            .iter()
            .map(|(field, accessor)| {
                let field_list = vec![
                    self.make_string(field, span),
                    self.make_string(accessor, span),
                ];
                Spanned::new(Expr::List(field_list), span)
            })
            .collect();

        let field_list = Spanned::new(
            Expr::List(
                std::iter::once(self.make_symbol("quote", span))
                    .chain(std::iter::once(Spanned::new(Expr::List(field_list_items), span)))
                    .collect()
            ),
            span
        );

        let registration_args = vec![
            self.make_symbol("%register-condition-type", span),
            self.make_string(condition_type, span),
            self.make_string(supertype, span),
            self.make_string(constructor, span),
            self.make_string(predicate, span),
            field_list,
        ];

        Ok(Spanned::new(Expr::List(registration_args), span))
    }

    /// Generates the constructor procedure definition
    fn generate_constructor(
        &self,
        condition_type: &str,
        constructor: &str,
        field_specs: &[(String, String)],
        span: Span,
    ) -> Result<Spanned<Expr>> {
        // (define (constructor arg1 arg2 ...)
        //   (%make-condition "type-name" (list (cons 'field1 arg1) (cons 'field2 arg2) ...)))

        let param_names: Vec<String> = field_specs
            .iter()
            .enumerate()
            .map(|(i, (field, _))| format!("{}-arg", field))
            .collect();

        let params: Vec<Spanned<Expr>> = param_names
            .iter()
            .map(|name| self.make_symbol(name, span))
            .collect();

        let field_bindings: Vec<Spanned<Expr>> = field_specs
            .iter()
            .zip(&param_names)
            .map(|((field, _), param)| {
                let cons_args = vec![
                    Spanned::new(
                        Expr::List(vec![
                            self.make_symbol("quote", span),
                            self.make_symbol(field, span),
                        ]),
                        span
                    ),
                    self.make_symbol(param, span),
                ];
                Spanned::new(Expr::List(vec![
                    self.make_symbol("cons", span),
                    cons_args[0].clone(),
                    cons_args[1].clone(),
                ]), span)
            })
            .collect();

        let field_list = Spanned::new(
            Expr::List(
                std::iter::once(self.make_symbol("list", span))
                    .chain(field_bindings)
                    .collect()
            ),
            span
        );

        let body = vec![
            Spanned::new(
                Expr::List(vec![
                    self.make_symbol("%make-condition", span),
                    self.make_string(condition_type, span),
                    field_list,
                ]),
                span
            )
        ];

        let procedure_header = vec![
            self.make_symbol(constructor, span)
        ];
        let procedure_header = procedure_header.into_iter().chain(params).collect();

        let define_form = vec![
            self.make_symbol("define", span),
            Spanned::new(Expr::List(procedure_header), span),
        ];
        let define_form = define_form.into_iter().chain(body).collect();

        Ok(Spanned::new(Expr::List(define_form), span))
    }

    /// Generates the predicate procedure definition
    fn generate_predicate(
        &self,
        condition_type: &str,
        predicate: &str,
        span: Span,
    ) -> Result<Spanned<Expr>> {
        // (define (predicate obj)
        //   (and (condition? obj)
        //        (condition-has-type? obj (%get-condition-type "type-name"))))

        let param = self.make_symbol("obj", span);
        
        let condition_check = Spanned::new(
            Expr::List(vec![
                self.make_symbol("condition?", span),
                param.clone(),
            ]),
            span
        );

        let type_check = Spanned::new(
            Expr::List(vec![
                self.make_symbol("condition-has-type?", span),
                param.clone(),
                Spanned::new(
                    Expr::List(vec![
                        self.make_symbol("%get-condition-type", span),
                        self.make_string(condition_type, span),
                    ]),
                    span
                ),
            ]),
            span
        );

        let body = vec![
            Spanned::new(
                Expr::List(vec![
                    self.make_symbol("and", span),
                    condition_check,
                    type_check,
                ]),
                span
            )
        ];

        let procedure_header = vec![
            self.make_symbol(predicate, span),
            param,
        ];

        let define_form = vec![
            self.make_symbol("define", span),
            Spanned::new(Expr::List(procedure_header), span),
        ];
        let define_form = define_form.into_iter().chain(body).collect();

        Ok(Spanned::new(Expr::List(define_form), span))
    }

    /// Generates a field accessor procedure definition
    fn generate_field_accessor(
        &self,
        condition_type: &str,
        field: &str,
        accessor: &str,
        span: Span,
    ) -> Result<Spanned<Expr>> {
        // (define (accessor condition)
        //   (condition-ref condition 'field))

        let param = self.make_symbol("condition", span);
        
        let body = vec![
            Spanned::new(
                Expr::List(vec![
                    self.make_symbol("condition-ref", span),
                    param.clone(),
                    Spanned::new(
                        Expr::List(vec![
                            self.make_symbol("quote", span),
                            self.make_symbol(field, span),
                        ]),
                        span
                    ),
                ]),
                span
            )
        ];

        let procedure_header = vec![
            self.make_symbol(accessor, span),
            param,
        ];

        let define_form = vec![
            self.make_symbol("define", span),
            Spanned::new(Expr::List(procedure_header), span),
        ];
        let define_form = define_form.into_iter().chain(body).collect();

        Ok(Spanned::new(Expr::List(define_form), span))
    }

    /// Helper to create a symbol expression
    fn make_symbol(&self, name: &str, span: Span) -> Spanned<Expr> {
        Spanned::new(Expr::Symbol(name.to_string()), span)
    }

    /// Helper to create a string literal expression
    fn make_string(&self, content: &str, span: Span) -> Spanned<Expr> {
        let literal = Literal::String(Box::new(content.to_string()));
        Spanned::new(Expr::Literal(literal), span)
    }
}

/// Installs the define-condition-type macro in an environment
pub fn install_define_condition_type_macro(_env: &Arc<Environment>) -> Result<()> {
    // TODO: Integrate with the macro expansion system properly
    // This would need to be integrated with the prescan system
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::string_interner::StringInterner;
    
    #[test]
    fn test_simple_condition_type_expansion() {
        let interner = Arc::new(StringInterner::new());
        let transformer = DefineConditionTypeMacro::new(interner.clone());
        
        // Parse (define-condition-type &my-error &error make-my-error my-error?)
        let condition_type = Spanned::new(
            Expr::Symbol(interner.intern("&my-error")),
            Span::default()
        );
        let supertype = Spanned::new(
            Expr::Symbol(interner.intern("&error")),
            Span::default()
        );
        let constructor = Spanned::new(
            Expr::Symbol(interner.intern("make-my-error")),
            Span::default()
        );
        let predicate = Spanned::new(
            Expr::Symbol(interner.intern("my-error?")),
            Span::default()
        );
        
        let args = vec![condition_type, supertype, constructor, predicate];
        let result = transformer.transform(&args, Span::default());
        
        assert!(result.is_ok());
        // Verify the expansion contains the expected forms
        match &result.unwrap().node {
            Expr::List(forms) => {
                assert!(forms.len() >= 4); // begin + registration + constructor + predicate
                // First should be 'begin'
                match &forms[0].node {
                    Expr::Symbol(sym) => {
                        assert_eq!(interner.resolve(*sym), "begin");
                    }
                    _ => panic!("Expected begin symbol"),
                }
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_condition_type_with_fields() {
        let interner = Arc::new(StringInterner::new());
        let transformer = DefineConditionTypeMacro::new(interner.clone());
        
        // Parse (define-condition-type &file-error &error 
        //         make-file-error file-error?
        //         (filename file-error-filename))
        let field_spec = Spanned::new(
            Expr::List(vec![
                Spanned::new(Expr::Symbol(interner.intern("filename")), Span::default()),
                Spanned::new(Expr::Symbol(interner.intern("file-error-filename")), Span::default()),
            ]),
            Span::default()
        );
        
        let args = vec![
            Spanned::new(Expr::Symbol(interner.intern("&file-error")), Span::default()),
            Spanned::new(Expr::Symbol(interner.intern("&error")), Span::default()),
            Spanned::new(Expr::Symbol(interner.intern("make-file-error")), Span::default()),
            Spanned::new(Expr::Symbol(interner.intern("file-error?")), Span::default()),
            field_spec,
        ];
        
        let result = transformer.transform(&args, Span::default());
        assert!(result.is_ok());
        
        // Verify the expansion includes field accessor
        match &result.unwrap().node {
            Expr::List(forms) => {
                assert!(forms.len() >= 5); // begin + registration + constructor + predicate + accessor
            }
            _ => panic!("Expected list expression"),
        }
    }
}