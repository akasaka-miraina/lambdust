//! Special Forms Parser Implementation for Lambdust
//!
//! This module implements parsing for all R7RS special forms, which are the
//! fundamental syntactic constructs that cannot be implemented as ordinary
//! procedures. Special forms have special evaluation rules and are processed
//! directly by the evaluator rather than being macro-expanded.
//!
//! # R7RS Special Forms
//!
//! The R7RS standard defines exactly 10 special forms (Section 4.1):
//!
//! 1. **`quote`** - Literal data quotation  
//! 2. **`lambda`** - Procedure creation
//! 3. **`if`** - Conditional evaluation
//! 4. **`set!`** - Variable assignment
//! 5. **`define`** - Variable definition
//! 6. **`define-syntax`** - Syntax definition
//! 7. **`let-syntax`** - Local syntax binding
//! 8. **`letrec-syntax`** - Recursive syntax binding
//! 9. **`syntax-rules`** - Pattern-based macro definition
//! 10. **`include`** - File inclusion (implementation-dependent)
//!
//! Additionally, Lambdust supports these R7RS-compatible extensions:
//!
//! - **`call-with-current-continuation`** (call/cc) - Continuation capture
//! - **`parameterize`** - Dynamic parameter binding
//! - **`guard`** - Exception handling
//!
//! # Parsing Strategy
//!
//! Special form parsing uses recursive descent with the following principles:
//!
//! ## Syntactic Validation
//!
//! Each special form has precise syntactic requirements defined in R7RS.
//! The parser enforces these requirements and provides detailed error messages
//! for violations.
//!
//! ## Context-Sensitive Parsing
//!
//! Some special forms (like `define`) have multiple syntactic variants:
//! - `(define <variable> <expression>)`
//! - `(define (<variable> <formals>) <body>)`
//!
//! The parser handles these variants through careful lookahead and dispatching.
//!
//! ## Error Recovery
//!
//! The parser provides comprehensive error recovery with suggestions for
//! common mistakes, helping developers understand R7RS syntax requirements.
//!
//! # Implementation Details
//!
//! ## Grammar Productions
//!
//! Each method implements specific R7RS grammar productions:
//!
//! ```bnf
//! <definition> := (define <variable> <expression>)
//!               | (define (<variable> <def formals>) <body>)
//!               | (define-syntax <keyword> <transformer spec>)
//!               
//! <expression> := <variable>
//!               | <literal>
//!               | <procedure call>
//!               | <lambda expression>
//!               | <conditional>
//!               | <assignment>
//!               | <derived expression>
//!               | <macro use>
//!               | <macro block>
//! ```
//!
//! ## Type Safety
//!
//! All parsing methods maintain Rust's type safety while building AST nodes.
//! Span information is preserved for all expressions to enable precise
//! error reporting and IDE integration.
//!
//! # Error Handling
//!
//! The parser uses Lambdust's structured error system with:
//!
//! - **Precise Source Locations**: All errors include exact span information
//! - **Contextual Messages**: Error messages explain what was expected
//! - **Recovery Suggestions**: Common fixes are suggested when possible
//! - **R7RS References**: Errors reference relevant R7RS sections when helpful

use super::Parser;
use crate::ast::{
    Binding, CaseClause, CaseLambdaClause, CondClause, CutArgument, Expr, Formals, GuardClause,
    KeywordParam, ParameterBinding, TypeExpr, TypedParam, TypedParameterKind,
};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::lexer::TokenKind;
use std::collections::HashMap;

impl Parser {
    /// Parse quote special form: `(quote <datum>)`.
    ///
    /// Quote prevents evaluation of its argument and returns the datum literally.
    /// This is fundamental to Lisp's homoiconicity and metaprogramming capabilities.
    ///
    /// # R7RS Specification
    ///
    /// From R7RS Section 4.1.2: "Literal expressions":
    /// > `(quote <datum>)` evaluates to `<datum>`. `<datum>` can be any external  
    /// > representation of a Scheme object. This notation is used to include literal
    /// > constants in Scheme code.
    ///
    /// # Syntax
    ///
    /// ```scheme
    /// (quote datum)     ; Full form
    /// 'datum            ; Abbreviated form (handled by lexer)
    /// ```
    ///
    /// # Examples
    ///
    /// ```scheme
    /// (quote a)         ; => a
    /// (quote (+ 1 2))   ; => (+ 1 2)  [not evaluated]
    /// (quote "hello")   ; => "hello"
    /// '(a b c)          ; => (a b c)
    /// ```
    ///
    /// # Grammar Production
    ///
    /// ```bnf
    /// <quotation> := (quote <datum>) | '<datum>
    /// <datum> := <simple datum> | <compound datum>
    /// ```
    ///
    /// # Arguments
    ///
    /// - `start_span`: Source location of the opening parenthesis
    ///
    /// # Returns
    ///
    /// `Result<Spanned<Expr>>` containing the quote expression or a parse error
    ///
    /// # Errors
    ///
    /// - Missing datum argument: `(quote)`
    /// - Multiple arguments: `(quote a b)`
    /// - Missing closing parenthesis
    pub fn parse_quote_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let expr = self.parse_expression()?;

        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after quote",
        )?;
        let end_span = self.current_span();
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::Quote(Box::new(expr)), span))
    }

    /// Parse lambda special form: `(lambda <formals> <body>)`.
    ///
    /// Lambda expressions create anonymous procedures with lexical scoping.
    /// They are the foundation of functional programming in Scheme and support
    /// multiple parameter patterns for maximum flexibility.
    ///
    /// # R7RS Specification
    ///
    /// From R7RS Section 4.1.4: "Lambda expressions":
    /// > A lambda expression evaluates to a procedure. The environment in  
    /// > effect when the lambda expression was evaluated is remembered as part
    /// > of the procedure. When the procedure is later called with some actual
    /// > arguments, the environment in which the lambda expression was evaluated
    /// > will be extended by binding the variables in the formal argument list
    /// > to fresh locations, and the corresponding actual argument values will
    /// > be stored in those locations.
    ///
    /// # Syntax Variants
    ///
    /// ```scheme
    /// (lambda (var ...) body1 body2 ...)           ; Fixed parameters
    /// (lambda var body1 body2 ...)                 ; Variable parameters  
    /// (lambda (var1 ... varn . varn+1) body ...)   ; Mixed parameters
    /// ```
    ///
    /// # Parameter Patterns
    ///
    /// ## Fixed Parameters: `(lambda (x y z) ...)`
    /// - Procedure must be called with exactly the specified number of arguments
    /// - Each parameter is bound to the corresponding argument
    ///
    /// ## Variable Parameters: `(lambda args ...)`  
    /// - Procedure accepts any number of arguments
    /// - All arguments are collected into a list bound to the parameter
    ///
    /// ## Mixed Parameters: `(lambda (x y . rest) ...)`
    /// - First parameters are bound to individual arguments
    /// - Remaining arguments are collected into a list bound to the rest parameter
    ///
    /// # Lambdust Extensions
    ///
    /// Lambdust extends lambda with optional features:
    /// - **Type annotations**: `(lambda ((x : Int) (y : String)) ...)`
    /// - **Return types**: `(lambda (x y) : String ...)`
    /// - **Metadata**: `(lambda (x y) #:pure #:inline ...)`
    ///
    /// # Examples
    ///
    /// ```scheme
    /// (lambda (x) (* x x))                    ; Square function
    /// (lambda (x y) (+ x y))                  ; Addition
    /// (lambda args (apply + args))            ; Variable arguments
    /// (lambda (first . rest) (cons first rest))  ; Mixed parameters
    /// ```
    ///
    /// # Arguments
    ///
    /// - `start_span`: Source location of the opening parenthesis
    ///
    /// # Returns
    ///
    /// `Result<Spanned<Expr>>` containing the lambda expression or a parse error
    ///
    /// # Errors
    ///
    /// - Invalid formal parameter syntax
    /// - Empty lambda body: `(lambda (x))`
    /// - Type annotation syntax errors
    pub fn parse_lambda_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let formals = self.parse_formals()?;

        // Check for return type annotation
        let return_type = if self.check(&TokenKind::Colon) {
            self.advance(); // consume ':'
            Some(self.parse_type_expression()?)
        } else {
            None
        };

        let metadata = self.parse_metadata_exprs()?;

        let mut body = Vec::new();
        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            body.push(self.parse_expression()?);
        }

        if body.is_empty() {
            return Err(Box::new(Error::parse_error(
                "Lambda body cannot be empty",
                self.current_span(),
            )));
        }

        let end_span = self.current_span();
        self.advance(); // consume ')'
        let span = start_span.combine(end_span);

        Ok(Spanned::new(
            Expr::Lambda {
                formals,
                return_type,
                metadata,
                body,
            },
            span,
        ))
    }

    /// Parse if special form: `(if <test> <consequent> <alternative>)`.
    ///
    /// The fundamental conditional construct in Scheme. If evaluates the test
    /// expression and then evaluates either the consequent or alternative
    /// based on the truth value of the test.
    ///
    /// # R7RS Specification
    ///
    /// From R7RS Section 4.1.5: "Conditionals":
    /// > An if expression is evaluated as follows: first, `<test>` is evaluated.
    /// > If it yields a true value, then `<consequent>` is evaluated and its
    /// > values are returned. Otherwise `<alternative>` is evaluated and its
    /// > values are returned. If `<test>` yields a false value and no
    /// > `<alternative>` is specified, then the result of the expression is
    /// > unspecified.
    ///
    /// # Truth Values in Scheme
    ///
    /// - **False**: Only `#f` is false in Scheme
    /// - **True**: All other values are true, including:
    ///   - `#t` (explicit true)
    ///   - `0` (zero is true, unlike C)
    ///   - `""` (empty string is true)
    ///   - `'()` (empty list is true)
    ///
    /// # Syntax Variants
    ///
    /// ```scheme
    /// (if test consequent alternative)  ; Full form
    /// (if test consequent)              ; No alternative (result unspecified if false)
    /// ```
    ///
    /// # Examples
    ///
    /// ```scheme
    /// (if (> 3 2) 'yes 'no)            ; => yes
    /// (if (> 2 3) 'yes 'no)            ; => no
    /// (if #t (+ 1 2) (+ 3 4))          ; => 3
    /// (if #f 'true)                    ; => unspecified
    /// (if '() 'empty-list-is-true)     ; => empty-list-is-true
    /// ```
    ///
    /// # Tail Context
    ///
    /// The consequent and alternative of if expressions are in tail context,
    /// meaning they preserve proper tail recursion when the if expression
    /// itself is in tail context.
    ///
    /// # Arguments
    ///
    /// - `start_span`: Source location of the opening parenthesis
    ///
    /// # Returns
    ///
    /// `Result<Spanned<Expr>>` containing the if expression or a parse error
    ///
    /// # Errors
    ///
    /// - Missing test expression: `(if)`
    /// - Missing consequent: `(if test)`
    /// - Too many expressions: `(if test conseq alt extra)`
    pub fn parse_if_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let test = Box::new(self.parse_expression()?);
        let consequent = Box::new(self.parse_expression()?);

        let alternative = if !self.check(&TokenKind::RightParen) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after if",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(
            Expr::If {
                test,
                consequent,
                alternative,
            },
            span,
        ))
    }

    /// Parse define special form with multiple syntax variants.
    ///
    /// Define creates variable bindings in the current environment. It supports
    /// both simple variable definitions and function definition syntax sugar.
    ///
    /// # R7RS Specification
    ///
    /// From R7RS Section 4.1.6: "Definitions":
    /// > Definitions are valid only at the top level of a `<program>` and at  
    /// > the beginning of a `<body>`. A definition should have one of the forms:
    /// > - `(define <variable> <expression>)`
    /// > - `(define (<variable> <def formals>) <body>)`
    /// > - `(define (<variable> . <formal>) <body>)`
    ///
    /// # Syntax Variants
    ///
    /// ## Simple Variable Definition
    /// ```scheme
    /// (define variable expression)
    /// (define pi 3.14159)
    /// (define greeting "Hello, World!")
    /// ```
    ///
    /// ## Function Definition (Syntactic Sugar)
    /// ```scheme
    /// (define (function-name param1 param2) body ...)
    /// (define (square x) (* x x))
    /// (define (add x y) (+ x y))
    /// ```
    ///
    /// ## Variable Argument Function  
    /// ```scheme
    /// (define (function-name . args) body ...)
    /// (define (sum . numbers) (apply + numbers))
    /// ```
    ///
    /// ## Mixed Parameter Function
    /// ```scheme
    /// (define (function-name fixed1 fixed2 . rest) body ...)
    /// (define (list-operation op list . extra) (apply op list extra))
    /// ```
    ///
    /// # Lambdust Extensions
    ///
    /// Lambdust extends define with:
    /// - **Type annotations**: `(define (f (x : Int)) : String ...)`
    /// - **Metadata**: `(define f value #:pure #:inline)`
    /// - **Contract integration**: Works with define/contract
    ///
    /// # Desugaring
    ///
    /// Function definition syntax is desugared into variable definition + lambda:
    /// ```scheme
    /// (define (f x y) (+ x y))
    /// ;; Desugars to:
    /// (define f (lambda (x y) (+ x y)))
    /// ```
    ///
    /// # Scoping Rules
    ///
    /// - **Top-level**: Creates global bindings
    /// - **Body context**: Creates local bindings (letrec* semantics)
    /// - **Internal definitions**: Must precede expressions in body
    ///
    /// # Examples
    ///
    /// ```scheme
    /// ;; Variable definitions
    /// (define x 42)
    /// (define message "Hello")
    ///
    /// ;; Function definitions  
    /// (define (factorial n)
    ///   (if (= n 0) 1 (* n (factorial (- n 1)))))
    ///   
    /// (define (make-adder n)
    ///   (lambda (x) (+ x n)))
    /// ```
    ///
    /// # Arguments
    ///
    /// - `start_span`: Source location of the opening parenthesis
    ///
    /// # Returns
    ///
    /// `Result<Spanned<Expr>>` containing the define expression or a parse error
    ///
    /// # Errors
    ///
    /// - Missing variable name
    /// - Invalid function syntax: `(define ())`
    /// - Empty function body: `(define (f x))`
    /// - Type annotation syntax errors
    pub fn parse_define_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        self.with_context("define form", |parser| {
            let first = parser.parse_expression()?;

            match first.inner {
                Expr::Identifier(name) => {
                    // Check for type annotation: (define (x : Type) value)
                    if parser.check(&TokenKind::Colon) {
                        parser.advance(); // consume ':'
                        let type_annotation = parser.parse_type_expression()?;
                        let value = Box::new(parser.parse_expression()?);

                        let end_span = parser.current_span();
                        parser.consume(
                            &TokenKind::RightParen,
                            "Expected closing parenthesis after define",
                        )?;
                        let span = start_span.combine(end_span);

                        // Store type annotation in metadata for now
                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "type".to_string(),
                            Spanned::new(
                                Expr::TypeAnnotation {
                                    expr: Box::new(Spanned::new(
                                        Expr::Identifier(name.clone()),
                                        first.span,
                                    )),
                                    type_expr: Box::new(Spanned::new(
                                        // Convert TypeExpr to Expr for storage
                                        // This is a temporary solution
                                        Expr::Identifier("TypeAnnotation".to_string()),
                                        type_annotation.span,
                                    )),
                                },
                                span,
                            ),
                        );

                        return Ok(Spanned::new(
                            Expr::Define {
                                name,
                                value,
                                return_type: Some(type_annotation),
                                metadata,
                            },
                            span,
                        ));
                    }

                    // (define identifier expression)
                    let metadata = parser.parse_metadata_exprs()?;
                    let value = Box::new(parser.parse_expression()?);

                    let end_span = parser.current_span();
                    parser.consume(
                        &TokenKind::RightParen,
                        "Expected closing parenthesis after define",
                    )?;
                    let span = start_span.combine(end_span);

                    Ok(Spanned::new(
                        Expr::Define {
                            name,
                            value,
                            return_type: None,
                            metadata,
                        },
                        span,
                    ))
                }
                Expr::Application { operator, operands } => {
                    // (define (identifier formals) [: return-type] body)
                    // This is syntactic sugar for (define identifier (lambda formals [: return-type] body))
                    if let Expr::Identifier(name) = &operator.inner {
                        // Convert operands to formals
                        let formals = parser.operands_to_formals(operands)?;

                        // Check for return type annotation
                        let return_type = if parser.check(&TokenKind::Colon) {
                            parser.advance(); // consume ':'
                            Some(parser.parse_type_expression()?)
                        } else {
                            None
                        };

                        let metadata = parser.parse_metadata_exprs()?;

                        // Parse the body
                        let body = parser.parse_body()?;

                        let end_span = parser.current_span();
                        parser.consume(
                            &TokenKind::RightParen,
                            "Expected closing parenthesis after define",
                        )?;
                        let span = start_span.combine(end_span);

                        // Create a lambda expression as the value
                        let lambda_span = operator.span.combine(end_span);
                        let lambda_expr = Spanned::new(
                            Expr::Lambda {
                                formals,
                                return_type: return_type.clone(),
                                metadata: HashMap::new(),
                                body,
                            },
                            lambda_span,
                        );

                        Ok(Spanned::new(
                            Expr::Define {
                                name: name.clone(),
                                value: Box::new(lambda_expr),
                                return_type,
                                metadata,
                            },
                            span,
                        ))
                    } else {
                        Err(Box::new(Error::parse_error(
                            "Expected identifier as function name in define",
                            operator.span,
                        )))
                    }
                }
                _ => Err(Box::new(Error::parse_error(
                    "Expected identifier or function signature in define",
                    first.span,
                ))),
            }
        })
    }

    /// Parses a set! form: `(set! identifier expression)`
    pub fn parse_set_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        self.with_context("set! form", |parser| {
            let name_expr = parser.parse_expression()?;
            let name = match name_expr.inner {
                Expr::Identifier(name) => name,
                _ => {
                    return Err(Box::new(Error::parse_error(
                        "Expected identifier in set!",
                        name_expr.span,
                    )));
                }
            };

            let value = Box::new(parser.parse_expression()?);

            let end_span = parser.current_span();
            parser.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis after set!",
            )?;
            let span = start_span.combine(end_span);

            Ok(Spanned::new(Expr::Set { name, value }, span))
        })
    }

    /// Parses a define-syntax form: `(define-syntax identifier transformer)`
    pub fn parse_define_syntax_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        self.with_context("define-syntax form", |parser| {
            let name_expr = parser.parse_expression()?;
            let name = match name_expr.inner {
                Expr::Identifier(name) => name,
                _ => {
                    return Err(Box::new(Error::parse_error(
                        "Expected identifier in define-syntax",
                        name_expr.span,
                    )));
                }
            };

            let transformer = Box::new(parser.parse_expression()?);

            let end_span = parser.current_span();
            parser.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis after define-syntax",
            )?;
            let span = start_span.combine(end_span);

            Ok(Spanned::new(Expr::DefineSyntax { name, transformer }, span))
        })
    }

    /// Parses a syntax-rules form: (syntax-rules (literal ...) (pattern template) ...)
    pub fn parse_syntax_rules_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        self.with_context("syntax-rules form", |parser| {
            // Parse literals list
            parser.consume(
                &TokenKind::LeftParen,
                "Expected opening parenthesis for literals list",
            )?;
            parser.skip_whitespace();

            let mut literals = Vec::new();
            while !parser.check(&TokenKind::RightParen) && !parser.is_at_end() {
                if !parser.check(&TokenKind::Identifier) {
                    return Err(Box::new(Error::parse_error(
                        "Expected identifier in literals list",
                        parser.current_span(),
                    )));
                }

                literals.push(parser.current_token().text().to_string());
                parser.advance();
                parser.skip_whitespace();
            }

            parser.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis for literals list",
            )?;
            parser.skip_whitespace();

            // Parse syntax rules
            let mut rules = Vec::new();
            while !parser.check(&TokenKind::RightParen) && !parser.is_at_end() {
                // Each rule is (pattern template)
                parser.consume(
                    &TokenKind::LeftParen,
                    "Expected opening parenthesis for syntax rule",
                )?;
                parser.skip_whitespace();

                // Parse pattern
                let pattern = parser.parse_expression()?;
                parser.skip_whitespace();

                // Parse template
                let template = parser.parse_expression()?;
                parser.skip_whitespace();

                parser.consume(
                    &TokenKind::RightParen,
                    "Expected closing parenthesis for syntax rule",
                )?;
                parser.skip_whitespace();

                rules.push((pattern, template))
            }

            if rules.is_empty() {
                return Err(Box::new(Error::parse_error(
                    "syntax-rules must have at least one rule",
                    parser.current_span(),
                )));
            }

            let end_span = parser.current_span();
            parser.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis after syntax-rules",
            )?;
            let span = start_span.combine(end_span);

            // Create a SyntaxRules expression
            Ok(Spanned::new(Expr::SyntaxRules { literals, rules }, span))
        })
    }

    /// Parses a call/cc form: `(call-with-current-continuation procedure)`
    pub fn parse_call_cc_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let expr = Box::new(self.parse_expression()?);

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after call/cc",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::CallCC(expr), span))
    }

    /// Parses a primitive form: `(primitive symbol arguments*)`
    pub fn parse_primitive_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let name_expr = self.parse_expression()?;
        let name = match name_expr.inner {
            Expr::Identifier(name) => name,
            Expr::Literal(crate::ast::Literal::String(name)) => *name,
            _ => {
                return Err(crate::diagnostics::Error::parse_error(
                    "Expected symbol or string in primitive",
                    name_expr.span,
                )
                .boxed());
            }
        };

        let mut args = Vec::new();
        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            args.push(self.parse_expression()?);
        }

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after primitive",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::Primitive { name, args }, span))
    }

    /// Parses a type annotation form: `(:: expression type)`
    pub fn parse_type_annotation_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let expr = Box::new(self.parse_expression()?);
        let type_expr = Box::new(self.parse_expression()?);

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after type annotation",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::TypeAnnotation { expr, type_expr }, span))
    }

    // Derived forms

    /// Parses a begin form: `(begin expressions+)`
    pub fn parse_begin_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let mut exprs = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            exprs.push(self.parse_expression()?);
        }

        if exprs.is_empty() {
            return Err(crate::diagnostics::Error::parse_error(
                "Begin form cannot be empty",
                self.current_span(),
            )
            .into());
        }

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after begin",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::Begin(exprs), span))
    }

    /// Parses a let form: `(let (bindings*) body)`
    pub fn parse_let_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let bindings = self.parse_bindings()?;
        let body = self.parse_body()?;

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after let",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::Let { bindings, body }, span))
    }

    /// Parses a let* form: `(let* (bindings*) body)`
    pub fn parse_let_star_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let bindings = self.parse_bindings()?;
        let body = self.parse_body()?;

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after let*",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::LetStar { bindings, body }, span))
    }

    /// Parses a letrec form: `(letrec (bindings*) body)`
    pub fn parse_letrec_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let bindings = self.parse_bindings()?;
        let body = self.parse_body()?;

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after letrec",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::LetRec { bindings, body }, span))
    }

    /// Parses a rec form: `(rec <variable> <expression>)`.
    ///
    /// The rec special form provides syntactic sugar for simple recursive definitions.
    /// It is more concise than letrec for cases where you're defining a single 
    /// recursive binding and immediately returning it.
    ///
    /// # SRFI-31 Specification
    ///
    /// From SRFI-31: "A special form `rec` for recursive evaluation":
    /// > `(rec <variable> <expression>)` is equivalent to
    /// > `(letrec ((<variable> <expression>)) <variable>)`
    ///
    /// # Syntax
    ///
    /// ```scheme
    /// (rec variable expression)
    /// ```
    ///
    /// # Examples
    ///
    /// ```scheme
    /// ;; Factorial using rec
    /// (define factorial
    ///   (rec f (lambda (n)
    ///            (if (= n 0) 1 (* n (f (- n 1)))))))
    /// 
    /// ;; Fibonacci using rec  
    /// (define fibonacci
    ///   (rec fib (lambda (n)
    ///              (if (<= n 1) n (+ (fib (- n 1)) (fib (- n 2)))))))
    ///
    /// ;; List length using rec
    /// (define length
    ///   (rec len (lambda (lst)
    ///              (if (null? lst) 0 (+ 1 (len (cdr lst)))))))
    /// ```
    ///
    /// # Implementation Strategy
    ///
    /// This method performs immediate transformation during parsing:
    /// 1. Parse the variable name (must be identifier)
    /// 2. Parse the expression 
    /// 3. Create equivalent letrec structure: `(letrec ((var expr)) var)`
    /// 4. Return the desugared letrec expression
    ///
    /// This approach leverages existing letrec infrastructure for:
    /// - Recursive binding semantics
    /// - Scope management
    /// - Optimization opportunities
    /// - Error handling consistency
    ///
    /// # Arguments
    ///
    /// - `start_span`: Source location of the opening parenthesis
    ///
    /// # Returns
    ///
    /// `Result<Spanned<Expr>>` containing the desugared letrec expression or a parse error
    ///
    /// # Errors
    ///
    /// - Missing variable name: `(rec)`
    /// - Invalid variable (non-identifier): `(rec 123 expr)` or `(rec "string" expr)`
    /// - Missing expression: `(rec var)`
    /// - Extra arguments: `(rec var expr extra)`
    /// - Malformed syntax
    pub fn parse_rec_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        self.with_context("rec form", |parser| {
            // Parse variable name - must be identifier
            let name_expr = parser.parse_expression()?;
            let variable_name = match name_expr.inner {
                Expr::Identifier(name) => name,
                _ => {
                    return Err(Box::new(Error::parse_error(
                        "Expected identifier as first argument to rec",
                        name_expr.span,
                    )));
                }
            };

            // Parse expression
            let expression = parser.parse_expression()?;

            // Check for extra arguments (rec should have exactly 2 arguments)
            if !parser.check(&TokenKind::RightParen) {
                return Err(Box::new(Error::parse_error(
                    "rec form takes exactly two arguments: variable and expression",
                    parser.current_span(),
                )));
            }

            let end_span = parser.current_span();
            parser.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis after rec",
            )?;
            let span = start_span.combine(end_span);

            // Transform (rec var expr) to (letrec ((var expr)) var)
            // Create binding: (var expr)
            use crate::ast::Binding;
            let binding = Binding {
                name: variable_name.clone(),
                value: expression,
            };

            // Create body: var (return the variable itself)
            let body = vec![Spanned::new(
                Expr::Identifier(variable_name),
                name_expr.span,
            )];

            // Create letrec expression
            Ok(Spanned::new(
                Expr::LetRec {
                    bindings: vec![binding],
                    body,
                },
                span,
            ))
        })
    }

    /// Parses a cond form: `(cond clauses+)`
    pub fn parse_cond_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let mut clauses = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            // Each clause is (test body...) or (else body...)
            self.consume(
                &TokenKind::LeftParen,
                "Expected opening parenthesis for cond clause",
            )?;
            self.skip_whitespace();

            // Parse test expression
            let test = self.parse_expression()?;
            self.skip_whitespace();

            // Parse body expressions
            let mut body = Vec::new();
            while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                body.push(self.parse_expression()?);
                self.skip_whitespace();
            }

            if body.is_empty() {
                return Err(Box::new(Error::parse_error(
                    "Cond clause must have at least one body expression",
                    self.current_span(),
                )));
            }

            self.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis for cond clause",
            )?;
            self.skip_whitespace();

            clauses.push(CondClause { test, body });
        }

        if clauses.is_empty() {
            return Err(Box::new(Error::parse_error(
                "Cond form must have at least one clause",
                self.current_span(),
            )));
        }

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after cond",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::Cond(clauses), span))
    }

    /// Parses a case form: `(case expression clauses+)`
    pub fn parse_case_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let expr = Box::new(self.parse_expression()?);
        self.skip_whitespace();

        let mut clauses = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            // Each clause is ((value1 value2 ...) body...) or (else body...)
            self.consume(
                &TokenKind::LeftParen,
                "Expected opening parenthesis for case clause",
            )?;
            self.skip_whitespace();

            // Parse values list or else
            let values =
                if self.check(&TokenKind::Identifier) && self.current_token().text() == "else" {
                    // else clause - no values
                    self.advance(); // consume 'else'
                    Vec::new()
                } else {
                    // Parse list of values
                    self.consume(
                        &TokenKind::LeftParen,
                        "Expected opening parenthesis for case values",
                    )?;
                    self.skip_whitespace();

                    let mut values = Vec::new();
                    while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                        values.push(self.parse_expression()?);
                        self.skip_whitespace();
                    }

                    self.consume(
                        &TokenKind::RightParen,
                        "Expected closing parenthesis for case values",
                    )?;
                    values
                };

            self.skip_whitespace();

            // Parse body expressions
            let mut body = Vec::new();
            while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                body.push(self.parse_expression()?);
                self.skip_whitespace();
            }

            if body.is_empty() {
                return Err(Box::new(Error::parse_error(
                    "Case clause must have at least one body expression",
                    self.current_span(),
                )));
            }

            self.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis for case clause",
            )?;
            self.skip_whitespace();

            clauses.push(CaseClause { values, body });
        }

        if clauses.is_empty() {
            return Err(Box::new(Error::parse_error(
                "Case form must have at least one clause",
                self.current_span(),
            )));
        }

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after case",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::Case { expr, clauses }, span))
    }

    /// Parses an and form: `(and expressions*)`
    pub fn parse_and_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let mut exprs = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            exprs.push(self.parse_expression()?);
        }

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after and",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::And(exprs), span))
    }

    /// Parses an or form: `(or expressions*)`
    pub fn parse_or_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let mut exprs = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            exprs.push(self.parse_expression()?);
        }

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after or",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::Or(exprs), span))
    }

    /// Parses a when form: `(when test expressions+)`
    pub fn parse_when_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let test = Box::new(self.parse_expression()?);
        let body = self.parse_body()?;

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after when",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::When { test, body }, span))
    }

    /// Parses an unless form: `(unless test expressions+)`
    pub fn parse_unless_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let test = Box::new(self.parse_expression()?);
        let body = self.parse_body()?;

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after unless",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::Unless { test, body }, span))
    }

    /// Parses a parameterize form: `(parameterize ((parameter value) ...) body)`
    pub fn parse_parameterize_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        self.with_context("parameterize form", |parser| {
            let bindings = parser.parse_parameter_bindings()?;
            let body = parser.parse_body()?;

            let end_span = parser.current_span();
            parser.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis after parameterize",
            )?;
            let span = start_span.combine(end_span);

            Ok(Spanned::new(Expr::Parameterize { bindings, body }, span))
        })
    }

    /// Parses a guard form: `(guard (variable clauses*) body)`
    pub fn parse_guard_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        self.with_context("guard form", |parser| {
            // Parse (variable clauses...)
            parser.consume(
                &TokenKind::LeftParen,
                "Expected opening parenthesis for guard variable and clauses",
            )?;
            parser.skip_whitespace();

            // Parse variable
            if !parser.check(&TokenKind::Identifier) {
                return Err(Box::new(Error::parse_error(
                    "Expected identifier for guard variable",
                    parser.current_span(),
                )));
            }

            let variable = parser.current_token().text().to_string();
            parser.advance();
            parser.skip_whitespace();

            // Parse clauses
            let mut clauses = Vec::new();
            while !parser.check(&TokenKind::RightParen) && !parser.is_at_end() {
                // Each clause is (test body...) or (test => proc) or (else body...)
                parser.consume(
                    &TokenKind::LeftParen,
                    "Expected opening parenthesis for guard clause",
                )?;
                parser.skip_whitespace();

                // Parse test expression
                let test = parser.parse_expression()?;
                parser.skip_whitespace();

                // Check for => clause
                let mut arrow = None;
                let mut body = Vec::new();

                if parser.check(&TokenKind::Identifier) && parser.current_token().text() == "=>" {
                    // => clause: (test => proc)
                    parser.advance(); // consume '=>'
                    parser.skip_whitespace();
                    arrow = Some(parser.parse_expression()?);
                } else {
                    // Regular clause: (test body...)
                    while !parser.check(&TokenKind::RightParen) && !parser.is_at_end() {
                        body.push(parser.parse_expression()?);
                        parser.skip_whitespace();
                    }

                    if body.is_empty() && arrow.is_none() {
                        return Err(Box::new(Error::parse_error(
                            "Guard clause must have at least one body expression or => clause",
                            parser.current_span(),
                        )));
                    }
                }

                parser.consume(
                    &TokenKind::RightParen,
                    "Expected closing parenthesis for guard clause",
                )?;
                parser.skip_whitespace();

                clauses.push(GuardClause { test, body, arrow });
            }

            if clauses.is_empty() {
                return Err(Box::new(Error::parse_error(
                    "Guard form must have at least one clause",
                    parser.current_span(),
                )));
            }

            parser.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis for guard clauses",
            )?;
            parser.skip_whitespace();

            // Parse body
            let body = parser.parse_body()?;

            let end_span = parser.current_span();
            parser.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis after guard",
            )?;
            let span = start_span.combine(end_span);

            Ok(Spanned::new(
                Expr::Guard {
                    variable,
                    clauses,
                    body,
                },
                span,
            ))
        })
    }

    /// Parses a case-lambda form: `(case-lambda [: return-type] (formals1 body1...) (formals2 body2...) ...)`
    pub fn parse_case_lambda_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        self.with_context("case-lambda form", |parser| {
            // Check for return type annotation
            let return_type = if parser.check(&TokenKind::Colon) {
                parser.advance(); // consume ':'
                Some(parser.parse_type_expression()?)
            } else {
                None
            };

            let metadata = HashMap::new(); // For now, no metadata support in case-lambda
            let mut clauses = Vec::new();

            while !parser.check(&TokenKind::RightParen) && !parser.is_at_end() {
                // Each clause is (formals body...)
                parser.skip_whitespace();
                parser.consume(
                    &TokenKind::LeftParen,
                    "Expected opening parenthesis for case-lambda clause",
                )?;
                parser.skip_whitespace();

                // Parse formals
                let formals = parser.parse_formals()?;
                parser.skip_whitespace();

                // Parse body expressions
                let mut body = Vec::new();
                while !parser.check(&TokenKind::RightParen) && !parser.is_at_end() {
                    body.push(parser.parse_expression()?);
                    parser.skip_whitespace();
                }

                if body.is_empty() {
                    return Err(Box::new(Error::parse_error(
                        "Case-lambda clause must have at least one body expression",
                        parser.current_span(),
                    )));
                }

                parser.consume(
                    &TokenKind::RightParen,
                    "Expected closing parenthesis for case-lambda clause",
                )?;
                parser.skip_whitespace();

                clauses.push(CaseLambdaClause { formals, body });
            }

            if clauses.is_empty() {
                return Err(Box::new(Error::parse_error(
                    "Case-lambda form must have at least one clause",
                    parser.current_span(),
                )));
            }

            let end_span = parser.current_span();
            parser.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis after case-lambda",
            )?;
            let span = start_span.combine(end_span);

            Ok(Spanned::new(
                Expr::CaseLambda {
                    clauses,
                    return_type,
                    metadata,
                },
                span,
            ))
        })
    }

    // Helper methods

    /// Parses formal parameters for lambda expressions.
    ///
    /// Supports all R7RS parameter patterns plus Lambdust typed parameter extensions:
    /// 1. Single identifier: x (variable arity)
    /// 2. List of identifiers: (x y z) (fixed arity)
    /// 3. Dotted pair: (x y . z) (mixed: fixed + rest)
    /// 4. Keyword parameters: (x y #:key default #:key2 default2)
    /// 5. Mixed keyword: (x #:key default . rest)
    /// 6. Typed parameters: ((x : Type) (y : Type)) (typed fixed arity)
    /// 7. Typed variable: (x : Type) (typed variable arity)
    /// 8. Typed mixed: ((x : Type) (y : Type) . (rest : RestType))
    pub fn parse_formals(&mut self) -> Result<Formals> {
        // First, check for typed parameter patterns using efficient LL(2) lookahead
        if let Some(typed_kind) = self.is_typed_parameter_kind() {
            return match typed_kind {
                TypedParameterKind::Single => {
                    // Single typed parameter: (param : Type)
                    let typed_param = self.parse_single_typed_formal()?;
                    Ok(Formals::TypedVariable(typed_param))
                }
                TypedParameterKind::List => {
                    // List of typed parameters: ((param1 : Type1) (param2 : Type2) ...)
                    self.advance(); // consume '('
                    self.skip_whitespace();
                    self.parse_typed_formals()
                }
                TypedParameterKind::Mixed => {
                    // Mixed typed parameters: ((param1 : Type1) ... . (rest : RestType))
                    self.advance(); // consume '('
                    self.skip_whitespace();
                    self.parse_typed_formals()
                }
            };
        }

        if self.check(&TokenKind::LeftParen) {
            self.advance(); // consume '('
            self.skip_whitespace();

            // Check if this is a typed parameter list by looking ahead (legacy support)
            let is_typed = self.is_typed_parameter_list();

            if is_typed {
                return self.parse_typed_formals();
            }

            let mut fixed = Vec::new();
            let mut rest = None;
            let mut keywords = Vec::new();
            let mut seen_keyword = false;

            while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                if self.check(&TokenKind::Dot) {
                    // Dotted pair syntax: (x y . rest)
                    self.advance(); // consume '.'
                    self.skip_whitespace();

                    if !self.check(&TokenKind::Identifier) {
                        return Err(Box::new(Error::parse_error(
                            "Expected identifier after dot in formals",
                            self.current_span(),
                        )));
                    }

                    let rest_name = self.current_token().text().to_string();
                    self.advance();
                    rest = Some(rest_name);
                    break;
                } else if self.check(&TokenKind::Keyword) {
                    // Keyword parameter: #:key [default]
                    seen_keyword = true;
                    let keyword_token = self.current_token();
                    let keyword_name = keyword_token
                        .text()
                        .strip_prefix("#:")
                        .unwrap_or(keyword_token.text())
                        .to_string();
                    self.advance();
                    self.skip_whitespace();

                    // Check for optional default value
                    let default = if !self.check(&TokenKind::RightParen)
                        && !self.check(&TokenKind::Keyword)
                    {
                        Some(self.parse_expression()?)
                    } else {
                        None
                    };

                    keywords.push(KeywordParam {
                        name: keyword_name,
                        default,
                    });
                } else if self.check(&TokenKind::Identifier) {
                    if seen_keyword {
                        return Err(Box::new(Error::parse_error(
                            "Cannot mix positional and keyword parameters",
                            self.current_span(),
                        )));
                    }

                    let param_name = self.current_token().text().to_string();
                    self.advance();
                    fixed.push(param_name);
                } else {
                    return Err(Box::new(Error::parse_error(
                        "Expected identifier or keyword in formals",
                        self.current_span(),
                    )));
                }

                self.skip_whitespace();
            }

            if !self.check(&TokenKind::RightParen) {
                return Err(Box::new(Error::parse_error(
                    "Expected closing parenthesis in formals",
                    self.current_span(),
                )));
            }
            self.advance(); // consume ')'

            // Determine the type of formals based on what we parsed
            let formals = if !keywords.is_empty() {
                Formals::Keyword {
                    fixed,
                    rest,
                    keywords,
                }
            } else if let Some(rest_param) = rest {
                if fixed.is_empty() {
                    // Just a rest parameter - convert to Variable
                    Formals::Variable(rest_param)
                } else {
                    Formals::Mixed {
                        fixed,
                        rest: rest_param,
                    }
                }
            } else {
                Formals::Fixed(fixed)
            };

            // Validate the formals before returning
            Parser::validate_formals(&formals, self.current_span())?;
            Ok(formals)
        } else if self.check(&TokenKind::Identifier) {
            // Could be a single identifier or start of typed variable
            let name = self.current_token().text().to_string();
            let name_span = self.current_span();
            self.advance();

            // Check if this is a typed variable: param : Type
            if self.check(&TokenKind::Colon) {
                self.advance(); // consume ':'
                let type_annotation = self.parse_type_expression()?;
                let typed_param = TypedParam::new(name, type_annotation);
                return Ok(Formals::TypedVariable(typed_param));
            }

            // Single identifier - variable arity
            // Validate the identifier
            Parser::validate_identifier(&name, name_span)?;

            Ok(Formals::Variable(name))
        } else {
            Err(Box::new(Error::parse_error(
                format!(
                    "Expected formals list or identifier, found {}",
                    self.current_token_text()
                ),
                self.current_span(),
            )))
        }
    }

    /// Parses parameter bindings for parameterize forms.
    fn parse_parameter_bindings(&mut self) -> Result<Vec<ParameterBinding>> {
        self.consume(
            &TokenKind::LeftParen,
            "Expected opening parenthesis for parameter bindings",
        )?;
        self.skip_whitespace();

        let mut bindings = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            // Each binding is (parameter value)
            self.consume(
                &TokenKind::LeftParen,
                "Expected opening parenthesis for parameter binding",
            )?;
            self.skip_whitespace();

            // Parse parameter expression
            let parameter = self.parse_expression()?;
            self.skip_whitespace();

            // Parse value expression
            let value = self.parse_expression()?;
            self.skip_whitespace();

            self.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis for parameter binding",
            )?;
            self.skip_whitespace();

            bindings.push(ParameterBinding { parameter, value });
        }

        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis for parameter bindings",
        )?;
        Ok(bindings)
    }

    /// Parses variable bindings for let forms.
    fn parse_bindings(&mut self) -> Result<Vec<Binding>> {
        self.consume(
            &TokenKind::LeftParen,
            "Expected opening parenthesis for bindings",
        )?;
        self.skip_whitespace();

        let mut bindings = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            // Each binding is (identifier expression)
            self.consume(
                &TokenKind::LeftParen,
                "Expected opening parenthesis for binding",
            )?;
            self.skip_whitespace();

            // Parse identifier
            if !self.check(&TokenKind::Identifier) {
                return Err(Box::new(Error::parse_error(
                    "Expected identifier in binding",
                    self.current_span(),
                )));
            }

            let name = self.current_token().text().to_string();
            self.advance();
            self.skip_whitespace();

            // Parse value expression
            let value = self.parse_expression()?;
            self.skip_whitespace();

            self.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis for binding",
            )?;
            self.skip_whitespace();

            bindings.push(Binding { name, value });
        }

        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis for bindings",
        )?;
        Ok(bindings)
    }

    /// Parses a body (sequence of expressions).
    pub fn parse_body(&mut self) -> Result<Vec<Spanned<Expr>>> {
        let mut body = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            body.push(self.parse_expression()?);
        }

        if body.is_empty() {
            return Err(Box::new(Error::parse_error(
                "Body cannot be empty",
                self.current_span(),
            )));
        }

        Ok(body)
    }

    /// Converts a list of operand expressions to formal parameters.
    /// Used for function definition syntax: (define (f x y) body)
    pub fn operands_to_formals(&self, operands: Vec<Spanned<Expr>>) -> Result<Formals> {
        let mut fixed_params = Vec::new();
        let mut rest_param = None;
        let mut _found_dot = false;

        for (i, operand) in operands.iter().enumerate() {
            match &operand.inner {
                Expr::Identifier(name) => {
                    if name == "." {
                        if _found_dot {
                            return Err(Box::new(Error::parse_error(
                                "Multiple dots in formal parameter list",
                                operand.span,
                            )));
                        }
                        if i == operands.len() - 1 {
                            return Err(Box::new(Error::parse_error(
                                "Expected rest parameter after dot",
                                operand.span,
                            )));
                        }
                        if i == operands.len() - 2 {
                            // Next parameter should be the rest parameter
                            if let Expr::Identifier(rest_name) = &operands[i + 1].inner {
                                rest_param = Some(rest_name.clone());
                                _found_dot = true;
                                break;
                            } else {
                                return Err(Box::new(Error::parse_error(
                                    "Expected identifier as rest parameter",
                                    operands[i + 1].span,
                                )));
                            }
                        } else {
                            return Err(Box::new(Error::parse_error(
                                "Dot must be followed by exactly one rest parameter",
                                operand.span,
                            )));
                        }
                    } else if !_found_dot {
                        fixed_params.push(name.clone());
                    }
                }
                _ => {
                    return Err(Box::new(Error::parse_error(
                        "Expected identifier in formal parameter list",
                        operand.span,
                    )));
                }
            }
        }

        let formals = if let Some(rest) = rest_param {
            Formals::Mixed {
                fixed: fixed_params,
                rest,
            }
        } else {
            Formals::Fixed(fixed_params)
        };

        // Validate the formals
        Parser::validate_formals(
            &formals,
            operands.first().map(|e| e.span).unwrap_or(Span::new(0, 0)),
        )?;

        Ok(formals)
    }

    /// Parses an import form: `(import import-spec+)`
    pub fn parse_import_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        let mut import_specs = Vec::new();

        // Parse one or more import specifications
        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            import_specs.push(self.parse_expression()?);
            self.skip_whitespace();
        }

        if import_specs.is_empty() {
            return Err(Box::new(Error::parse_error(
                "Import form requires at least one import specification",
                self.current_span(),
            )));
        }

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after import",
        )?;
        let span = start_span.combine(end_span);

        Ok(Spanned::new(Expr::Import { import_specs }, span))
    }

    /// Parses a define-library form: `(define-library name library-declaration*)`
    pub fn parse_define_library_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        self.with_context("define-library form", |parser| {
            // Parse library name (a list of identifiers/numbers)
            parser.consume(
                &TokenKind::LeftParen,
                "Expected opening parenthesis for library name",
            )?;
            parser.skip_whitespace();

            let mut name_parts = Vec::new();
            while !parser.check(&TokenKind::RightParen) && !parser.is_at_end() {
                if parser.check(&TokenKind::Identifier) || parser.check(&TokenKind::IntegerNumber) {
                    let name = parser.current_token().text().to_string();
                    name_parts.push(name);
                    parser.advance();
                } else {
                    return Err(Box::new(Error::parse_error(
                        "Library name parts must be identifiers or numbers",
                        parser.current_span(),
                    )));
                }
                parser.skip_whitespace();
            }

            if name_parts.is_empty() {
                return Err(Box::new(Error::parse_error(
                    "Library name cannot be empty",
                    parser.current_span(),
                )));
            }

            parser.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis for library name",
            )?;
            parser.skip_whitespace();

            // Parse library declarations
            let mut imports = Vec::new();
            let mut exports = Vec::new();
            let mut body = Vec::new();

            while !parser.check(&TokenKind::RightParen) && !parser.is_at_end() {
                // Parse each library declaration
                if parser.check(&TokenKind::LeftParen) {
                    let declaration_start_span = parser.current_span();
                    parser.advance(); // consume '('
                    parser.skip_whitespace();

                    if parser.check(&TokenKind::Identifier) {
                        let keyword = parser.current_token().text().to_string();
                        parser.advance();
                        parser.skip_whitespace();

                        match keyword.as_str() {
                            "import" => {
                                // Parse import declaration - get all import specs
                                while !parser.check(&TokenKind::RightParen) && !parser.is_at_end() {
                                    imports.push(parser.parse_expression()?);
                                    parser.skip_whitespace();
                                }
                                parser.consume(
                                    &TokenKind::RightParen,
                                    "Expected closing parenthesis for import declaration",
                                )?;
                            }
                            "export" => {
                                // Parse export declaration - get all export specs
                                while !parser.check(&TokenKind::RightParen) && !parser.is_at_end() {
                                    exports.push(parser.parse_expression()?);
                                    parser.skip_whitespace();
                                }
                                parser.consume(
                                    &TokenKind::RightParen,
                                    "Expected closing parenthesis for export declaration",
                                )?;
                            }
                            _ => {
                                // Parse as a general library form (begin, include, etc.)
                                // Collect remaining content as body expressions
                                let mut content = Vec::new();
                                while !parser.check(&TokenKind::RightParen) && !parser.is_at_end() {
                                    content.push(parser.parse_expression()?);
                                    parser.skip_whitespace();
                                }
                                parser.consume(
                                    &TokenKind::RightParen,
                                    "Expected closing parenthesis for library declaration",
                                )?;

                                // Create a compound expression for this declaration
                                let declaration_expr = Expr::List({
                                    let mut decl = vec![Spanned::new(
                                        Expr::Identifier(keyword),
                                        declaration_start_span,
                                    )];
                                    decl.extend(content);
                                    decl
                                });
                                body.push(Spanned::new(declaration_expr, declaration_start_span))
                            }
                        }
                    } else {
                        return Err(Box::new(Error::parse_error(
                            "Expected identifier in library declaration",
                            parser.current_span(),
                        )));
                    }
                } else {
                    return Err(Box::new(Error::parse_error(
                        "Expected library declaration (list starting with identifier)",
                        parser.current_span(),
                    )));
                }
                parser.skip_whitespace();
            }

            let end_span = parser.current_span();
            parser.consume(
                &TokenKind::RightParen,
                "Expected closing parenthesis after define-library",
            )?;
            let span = start_span.combine(end_span);

            Ok(Spanned::new(
                Expr::DefineLibrary {
                    name: name_parts,
                    imports,
                    exports,
                    body,
                },
                span,
            ))
        })
    }

    /// Checks if the current position is the start of a typed parameter list.
    /// This does a limited lookahead to detect patterns like ((x : Type) ...)
    fn is_typed_parameter_list(&mut self) -> bool {
        // Simple heuristic: if we see a '(' followed by identifier, colon, we assume typed
        if self.check(&TokenKind::LeftParen) {
            // Save current position for backtracking
            let saved_pos = self.position();

            self.advance(); // consume '('
            self.skip_whitespace();

            let is_typed = self.check(&TokenKind::Identifier) && {
                self.advance(); // consume identifier
                self.check(&TokenKind::Colon)
            };

            // Restore position
            self.position = saved_pos;
            is_typed
        } else {
            false
        }
    }

    /// Efficiently determines the type of typed parameter using LL(2) lookahead.
    /// This method implements cs-architect optimization strategy for minimal backtracking.
    ///
    /// Patterns detected:
    /// - `(param : Type)` → TypedParameterKind::Single
    /// - `((param1 : Type1) (param2 : Type2) ...)` → TypedParameterKind::List
    /// - `((param1 : Type1) ... . (rest : RestType))` → TypedParameterKind::Mixed
    #[inline]
    fn is_typed_parameter_kind(&mut self) -> Option<TypedParameterKind> {
        let saved_pos = self.position(); // Zero-cost position save using Copy trait

        // Fast path: Check for single typed parameter (param : Type)
        if self.check(&TokenKind::Identifier) {
            self.advance(); // consume identifier
            if self.check(&TokenKind::Colon) {
                // Restore position - this is a single typed parameter
                self.position = saved_pos;
                return Some(TypedParameterKind::Single);
            }
            // Restore and continue checking
            self.position = saved_pos;
        }

        // Check for list of typed parameters: ((param : Type) ...)
        if self.check(&TokenKind::LeftParen) {
            self.advance(); // consume first '('
            self.skip_whitespace();

            // Check if first element is typed parameter (param : Type)
            if self.check(&TokenKind::LeftParen) {
                self.advance(); // consume inner '('
                self.skip_whitespace();

                if self.check(&TokenKind::Identifier) {
                    self.advance(); // consume identifier
                    self.skip_whitespace();

                    if self.check(&TokenKind::Colon) {
                        // This looks like ((param : Type) ...
                        // Quick check for mixed pattern: look for dot
                        self.position = saved_pos;
                        return Some(if self.has_dot_in_typed_list() {
                            TypedParameterKind::Mixed
                        } else {
                            TypedParameterKind::List
                        });
                    }
                }
            }
        }

        // Restore position and return None if no typed parameter pattern found
        self.position = saved_pos;
        None
    }

    /// Helper method to detect dot in typed parameter lists for mixed pattern detection.
    /// Uses efficient scanning without full parsing.
    #[inline]
    fn has_dot_in_typed_list(&mut self) -> bool {
        let saved_pos = self.position();
        let mut paren_depth = 0;
        let mut found_dot = false;

        // Scan ahead efficiently to find dot at correct nesting level
        while !self.is_at_end() {
            match self.current_token().kind {
                TokenKind::LeftParen => paren_depth += 1,
                TokenKind::RightParen => {
                    paren_depth -= 1;
                    if paren_depth < 0 {
                        break; // End of typed parameter list
                    }
                }
                TokenKind::Dot if paren_depth == 0 => {
                    found_dot = true;
                    break;
                }
                _ => {}
            }
            self.advance();
        }

        // Restore position
        self.position = saved_pos;
        found_dot
    }

    /// Parses typed formal parameters: ((x : Type) (y : Type) ...)
    fn parse_typed_formals(&mut self) -> Result<Formals> {
        let mut typed_params = Vec::new();
        let mut rest_param = None;

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            if self.check(&TokenKind::Dot) {
                // Dotted pair syntax: ((x : Type) . (rest : RestType))
                self.advance(); // consume '.'
                self.skip_whitespace();

                if !self.check(&TokenKind::LeftParen) {
                    return Err(Box::new(Error::parse_error(
                        "Expected typed parameter after dot in typed formals",
                        self.current_span(),
                    )));
                }

                let typed_param = self.parse_single_typed_parameter()?;
                rest_param = Some(typed_param);
                break;
            }

            // Parse typed parameter: (name : Type)
            let typed_param = self.parse_single_typed_parameter()?;
            typed_params.push(typed_param);

            self.skip_whitespace();
        }

        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis in typed formals",
        )?;

        if let Some(rest) = rest_param {
            Ok(Formals::TypedMixed {
                fixed: typed_params,
                rest,
            })
        } else {
            Ok(Formals::Typed(typed_params))
        }
    }

    /// Parses a single typed parameter: (name : Type)
    fn parse_single_typed_parameter(&mut self) -> Result<TypedParam> {
        self.consume(
            &TokenKind::LeftParen,
            "Expected opening parenthesis for typed parameter",
        )?;
        self.skip_whitespace();

        if !self.check(&TokenKind::Identifier) {
            return Err(Box::new(Error::parse_error(
                "Expected parameter name in typed parameter",
                self.current_span(),
            )));
        }

        let name = self.current_token_text();
        self.advance();

        self.consume(
            &TokenKind::Colon,
            "Expected ':' after parameter name in typed parameter",
        )?;
        let type_annotation = self.parse_type_expression()?;

        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis for typed parameter",
        )?;

        Ok(TypedParam::new(name, type_annotation))
    }

    /// Parses a single typed formal parameter with enhanced error recovery.
    /// This method handles the Lambda syntax extension: (param : Type)
    ///
    /// Implementation follows rust-expert-programmer idioms:
    /// - Zero-allocation error handling with Result<T>
    /// - Efficient string handling without unnecessary clones
    /// - #[inline] optimization for hot path
    #[inline]
    pub fn parse_single_typed_formal(&mut self) -> Result<TypedParam> {
        self.with_context("typed formal parameter", |parser| {
            // Handle both parenthesized and non-parenthesized forms
            let needs_paren = parser.check(&TokenKind::LeftParen);

            if needs_paren {
                parser.advance(); // consume '('
                parser.skip_whitespace();
            }

            // Parse parameter name with enhanced validation
            if !parser.check(&TokenKind::Identifier) {
                return Err(Box::new(Error::parse_error(
                    "Expected parameter name in typed formal parameter",
                    parser.current_span(),
                )));
            }

            let name = parser.current_token_text();
            let name_span = parser.current_span();
            parser.advance();

            // Validate identifier follows Scheme naming conventions
            Parser::validate_identifier(&name, name_span)?;

            // Consume colon with whitespace handling
            parser.skip_whitespace();
            if !parser.check(&TokenKind::Colon) {
                return Err(Box::new(Error::parse_error(
                    "Expected ':' after parameter name in typed formal parameter",
                    parser.current_span(),
                )));
            }
            parser.advance(); // consume ':'
            parser.skip_whitespace();

            // Parse type expression with full error context
            let type_annotation = parser.parse_type_expression().map_err(|e| {
                Box::new(Error::parse_error(
                    format!("Failed to parse type annotation for parameter '{name}': {e}"),
                    parser.current_span(),
                ))
            })?;

            if needs_paren {
                parser.skip_whitespace();
                parser.consume(
                    &TokenKind::RightParen,
                    "Expected closing parenthesis for typed formal parameter",
                )?;
            }

            Ok(TypedParam::new(name, type_annotation))
        })
    }

    /// Parses a cut form: `(cut <procedure> <slot-or-expr>*)`
    ///
    /// Cut forms create specialized procedures with placeholder slots:
    /// - `<>` represents a single argument placeholder
    /// - `<...>` represents a rest argument placeholder (must be last)
    /// - Other expressions are evaluated when the procedure is called
    ///
    /// # SRFI-26 Specification
    ///
    /// Examples:
    /// - `(cut + <> 5)` => `(lambda (x) (+ x 5))`
    /// - `(cut list 1 <> 3 <> 5)` => `(lambda (x y) (list 1 x 3 y 5))`
    /// - `(cut list <> <...>)` => `(lambda (x . rest) (apply list x rest))`
    pub fn parse_cut_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        // Parse procedure
        let procedure = self.parse_expression()?;
        
        // Parse arguments (slot placeholders and expressions)
        let mut arguments = Vec::new();
        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            let arg_expr = self.parse_expression()?;
            
            // Convert placeholder identifiers to CutArgument types
            let cut_arg = match &arg_expr.inner {
                Expr::Identifier(name) if name == "<>" => CutArgument::slot(),
                Expr::Identifier(name) if name == "<...>" => CutArgument::rest_slot(),
                _ => CutArgument::expression(arg_expr),
            };
            
            arguments.push(cut_arg);
        }

        // Validate arguments according to SRFI-26 rules
        self.validate_cut_arguments(&arguments)?;

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after cut",
        )?;
        let span = start_span.combine(end_span);

        // Use optimized expansion instead of storing raw AST
        crate::macro_system::srfi26_expansion::expand_cut_optimized(&procedure, &arguments, span)
    }

    /// Parses a cute form: `(cute <procedure> <slot-or-expr>*)`
    ///
    /// Cute forms are like cut, but non-slot expressions are evaluated immediately:
    /// - `<>` represents a single argument placeholder
    /// - `<...>` represents a rest argument placeholder (must be last)
    /// - Other expressions are evaluated when the cute form is evaluated
    ///
    /// # SRFI-26 Specification
    ///
    /// Examples:
    /// - `(cute cons <> (expensive-computation))` evaluates `expensive-computation` once
    /// - `(cut cons <> (expensive-computation))` evaluates it each time the lambda is called
    pub fn parse_cute_form(&mut self, start_span: Span) -> Result<Spanned<Expr>> {
        // Parse procedure
        let procedure = self.parse_expression()?;
        
        // Parse arguments (slot placeholders and expressions)
        let mut arguments = Vec::new();
        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            let arg_expr = self.parse_expression()?;
            
            // Convert placeholder identifiers to CutArgument types
            let cut_arg = match &arg_expr.inner {
                Expr::Identifier(name) if name == "<>" => CutArgument::slot(),
                Expr::Identifier(name) if name == "<...>" => CutArgument::rest_slot(),
                _ => CutArgument::expression(arg_expr),
            };
            
            arguments.push(cut_arg);
        }

        // Validate arguments according to SRFI-26 rules
        self.validate_cut_arguments(&arguments)?;

        let end_span = self.current_span();
        self.consume(
            &TokenKind::RightParen,
            "Expected closing parenthesis after cute",
        )?;
        let span = start_span.combine(end_span);

        // Use optimized expansion instead of storing raw AST
        crate::macro_system::srfi26_expansion::expand_cute_optimized(&procedure, &arguments, span)
    }

    /// Validates cut/cute arguments according to SRFI-26 rules
    fn validate_cut_arguments(&self, arguments: &[CutArgument]) -> Result<()> {
        let mut rest_slot_position = None;
        
        // Find rest slot and check for validity
        for (i, arg) in arguments.iter().enumerate() {
            if let CutArgument::RestSlot = arg {
                if rest_slot_position.is_some() {
                    return Err(Box::new(Error::parse_error(
                        "Only one rest slot (<...>) allowed per cut/cute expression",
                        Span::new(0, 0), // TODO: Use actual span
                    )));
                }
                rest_slot_position = Some(i);
            }
        }

        // Rest slot must be last argument if present
        if let Some(pos) = rest_slot_position {
            if pos != arguments.len() - 1 {
                return Err(Box::new(Error::parse_error(
                    "Rest slot (<...>) must be the last argument",
                    Span::new(0, 0), // TODO: Use actual span
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Formals, TypedParam, TypedParameterKind};
    use crate::diagnostics::Span;
    use crate::lexer::{Lexer, TokenKind};

    /// Helper function to create a parser from source code
    fn create_parser(source: &str) -> Parser {
        let mut lexer = Lexer::new(source, None);
        let tokens = lexer.tokenize().expect("Tokenization should succeed");
        Parser::new(tokens)
    }

    /// Helper function to create a typed parameter for testing
    fn create_typed_param(name: &str, type_str: &str) -> TypedParam {
        let source = format!("({} : {})", name, type_str);
        let mut parser = create_parser(&source);
        parser
            .parse_single_typed_formal()
            .expect("Should parse typed parameter")
    }

    #[test]
    fn test_typed_parameter_kind_detection_single() {
        // Test single typed parameter detection: (x : Int)
        let mut parser = create_parser("x : Int");
        let kind = parser.is_typed_parameter_kind();
        assert_eq!(kind, Some(TypedParameterKind::Single));

        // Ensure position is restored
        assert_eq!(parser.position(), 0);
    }

    #[test]
    fn test_typed_parameter_kind_detection_list() {
        // Test typed parameter list detection: ((x : Int) (y : String))
        let mut parser = create_parser("((x : Int) (y : String))");
        let kind = parser.is_typed_parameter_kind();
        assert_eq!(kind, Some(TypedParameterKind::List));

        // Ensure position is restored
        assert_eq!(parser.position(), 0);
    }

    #[test]
    fn test_typed_parameter_kind_detection_mixed() {
        // Test mixed typed parameter detection: ((x : Int) . (rest : List))
        let mut parser = create_parser("((x : Int) . (rest : List))");
        let kind = parser.is_typed_parameter_kind();
        assert_eq!(kind, Some(TypedParameterKind::Mixed));

        // Ensure position is restored
        assert_eq!(parser.position(), 0);
    }

    #[test]
    fn test_typed_parameter_kind_detection_none() {
        // Test no typed parameter: (x y z)
        let mut parser = create_parser("(x y z)");
        let kind = parser.is_typed_parameter_kind();
        assert_eq!(kind, None);

        // Ensure position is restored
        assert_eq!(parser.position(), 0);
    }

    #[test]
    fn test_parse_single_typed_formal_simple() {
        let mut parser = create_parser("(x : Int)");
        let typed_param = parser
            .parse_single_typed_formal()
            .expect("Should parse typed parameter");

        assert_eq!(typed_param.name, "x");
        // Type checking would require more complex setup
    }

    #[test]
    fn test_parse_single_typed_formal_complex_type() {
        let mut parser = create_parser("(callback : (Int -> String))");
        let typed_param = parser
            .parse_single_typed_formal()
            .expect("Should parse complex typed parameter");

        assert_eq!(typed_param.name, "callback");
        // Complex type should be parsed correctly
    }

    #[test]
    fn test_parse_single_typed_formal_no_parens() {
        let mut parser = create_parser("x : Int");
        let typed_param = parser
            .parse_single_typed_formal()
            .expect("Should parse typed parameter without parentheses");

        assert_eq!(typed_param.name, "x");
    }

    #[test]
    fn test_parse_formals_typed_single() {
        let mut parser = create_parser("x : Int");
        let formals = parser
            .parse_formals()
            .expect("Should parse single typed formal");

        match formals {
            Formals::TypedVariable(param) => {
                assert_eq!(param.name, "x");
            }
            _ => panic!("Expected TypedVariable, got {:?}", formals),
        }
    }

    #[test]
    fn test_parse_formals_typed_list() {
        let mut parser = create_parser("((x : Int) (y : String))");
        let formals = parser
            .parse_formals()
            .expect("Should parse typed formal list");

        match formals {
            Formals::Typed(params) => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "x");
                assert_eq!(params[1].name, "y");
            }
            _ => panic!("Expected Typed, got {:?}", formals),
        }
    }

    #[test]
    fn test_parse_formals_typed_mixed() {
        let mut parser = create_parser("((x : Int) (y : String) . (rest : List))");
        let formals = parser
            .parse_formals()
            .expect("Should parse typed mixed formals");

        match formals {
            Formals::TypedMixed { fixed, rest } => {
                assert_eq!(fixed.len(), 2);
                assert_eq!(fixed[0].name, "x");
                assert_eq!(fixed[1].name, "y");
                assert_eq!(rest.name, "rest");
            }
            _ => panic!("Expected TypedMixed, got {:?}", formals),
        }
    }

    #[test]
    fn test_parse_formals_backward_compatibility() {
        // Test that regular (non-typed) formals still work
        let mut parser = create_parser("(x y z)");
        let formals = parser
            .parse_formals()
            .expect("Should parse regular formals");

        match formals {
            Formals::Fixed(params) => {
                assert_eq!(params.len(), 3);
                assert_eq!(params[0], "x");
                assert_eq!(params[1], "y");
                assert_eq!(params[2], "z");
            }
            _ => panic!("Expected Fixed, got {:?}", formals),
        }
    }

    #[test]
    fn test_parse_lambda_form_with_typed_params() {
        let mut parser = create_parser("(lambda ((x : Int) (y : String)) (+ x (string-length y)))");
        let start_span = Span::new(0, 0);

        // Skip the lambda token
        parser.advance(); // skip '('
        parser.advance(); // skip 'lambda'

        let lambda_expr = parser
            .parse_lambda_form(start_span)
            .expect("Should parse lambda with typed parameters");

        // Verify the lambda expression structure
        assert!(matches!(lambda_expr.inner, Expr::Lambda { .. }));
    }

    #[test]
    fn test_error_handling_invalid_typed_parameter() {
        // Test error handling for invalid typed parameter syntax
        let mut parser = create_parser("(x Int)"); // Missing colon
        let result = parser.parse_single_typed_formal();

        assert!(
            result.is_err(),
            "Should fail to parse invalid typed parameter"
        );
    }

    #[test]
    fn test_error_handling_unterminated_typed_parameter() {
        // Test error handling for unterminated typed parameter
        let mut parser = create_parser("(x : Int"); // Missing closing paren
        let result = parser.parse_single_typed_formal();

        assert!(
            result.is_err(),
            "Should fail to parse unterminated typed parameter"
        );
    }

    #[test]
    fn test_position_restoration_after_failed_lookahead() {
        // Test that parser position is correctly restored after failed lookahead
        let mut parser = create_parser("(regular params)");
        let initial_pos = parser.position();

        let kind = parser.is_typed_parameter_kind();
        assert_eq!(kind, None);
        assert_eq!(
            parser.position(),
            initial_pos,
            "Position should be restored after failed lookahead"
        );
    }

    #[test]
    fn test_has_dot_in_typed_list_detection() {
        // Test dot detection in typed parameter lists
        let mut parser = create_parser("((x : Int) . (rest : List))");
        parser.advance(); // consume first '('

        let has_dot = parser.has_dot_in_typed_list();
        assert!(has_dot, "Should detect dot in typed parameter list");

        // Test without dot
        let mut parser2 = create_parser("((x : Int) (y : String))");
        parser2.advance(); // consume first '('

        let has_dot2 = parser2.has_dot_in_typed_list();
        assert!(!has_dot2, "Should not detect dot when none present");
    }

    #[test]
    fn test_performance_typed_parameter_parsing() {
        // Test performance with large typed parameter lists
        let mut params = Vec::new();
        for i in 0..100 {
            params.push(format!("(param{} : Int)", i));
        }
        let large_typed_list = format!("({})", params.join(" "));

        let start_time = std::time::Instant::now();
        let mut parser = create_parser(&large_typed_list);
        let formals = parser
            .parse_formals()
            .expect("Should parse large typed parameter list");
        let duration = start_time.elapsed();

        match formals {
            Formals::Typed(params) => {
                assert_eq!(params.len(), 100);
            }
            _ => panic!("Expected Typed formals"),
        }

        // Should complete in reasonable time (adjust threshold as needed)
        assert!(
            duration.as_millis() < 50,
            "Large typed parameter parsing took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_r7rs_compliance_mixed_syntax() {
        // Test that R7RS standard syntax still works alongside typed extensions
        let test_cases = vec![
            ("x", "single identifier"),
            ("(x y z)", "fixed parameters"),
            ("(x y . rest)", "mixed parameters"),
            ("(x : Int)", "single typed parameter"),
            ("((x : Int) (y : String))", "typed parameter list"),
            ("((x : Int) . (rest : List))", "typed mixed parameters"),
        ];

        for (input, description) in test_cases {
            let mut parser = create_parser(input);
            let result = parser.parse_formals();
            assert!(
                result.is_ok(),
                "Failed to parse {} ({}): {:?}",
                description,
                input,
                result.err()
            );
        }
    }
}
