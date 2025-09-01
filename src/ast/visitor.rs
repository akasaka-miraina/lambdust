//! AST Visitor Pattern Implementation for Lambdust
//!
//! This module implements the visitor pattern for traversing and transforming
//! Abstract Syntax Trees (ASTs) in Lambdust. The visitor pattern provides a
//! clean separation between AST structure and operations performed on ASTs,
//! enabling extensible tree traversal algorithms.
//!
//! # R7RS Compliance
//!
//! The visitor implementation ensures semantic accuracy with R7RS Scheme
//! specifications by providing specialized visit methods for:
//!
//! - **Special Forms**: The 10 core special forms defined in R7RS Section 4.1
//! - **Derived Forms**: Standard derived expressions like `let`, `cond`, `case`
//! - **Library System**: R7RS library definitions and import specifications
//! - **Syntax System**: Hygenic macro expansion and syntax transformations
//!
//! # Visitor Patterns
//!
//! ## Immutable Visitor (`Visitor`)
//!
//! The immutable visitor trait provides read-only traversal of AST nodes:
//!
//! ```rust,ignore
//! use lambdust::ast::{Visitor, Program};
//!
//! struct MyVisitor;
//! impl Visitor for MyVisitor {
//!     type Output = String;
//!
//!     fn visit_expressions(&mut self, exprs: &[Spanned<Expr>]) -> String {
//!         format!("Found {} expressions", exprs.len())
//!     }
//!
//!     // Implement other required methods...
//! }
//! ```
//!
//! ## Mutable Visitor (`VisitorMut`)
//!
//! The mutable visitor trait enables AST transformations and optimizations:
//!
//! ```rust,ignore
//! use lambdust::ast::{VisitorMut, Program};
//!
//! struct OptimizingVisitor;
//! impl VisitorMut for OptimizingVisitor {
//!     type Output = ();
//!     type Error = Error;
//!
//!     fn visit_expr_mut(&mut self, expr: &mut Spanned<Expr>) -> Result<()> {
//!         // Apply optimizations to the expression
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Traversal Algorithms
//!
//! The visitor implementation uses depth-first traversal with the following
//! algorithmic properties:
//!
//! - **Time Complexity**: O(n) where n is the number of AST nodes
//! - **Space Complexity**: O(d) where d is the maximum tree depth (call stack)
//! - **Memory Safety**: All traversals maintain Rust's ownership invariants
//! - **Tail Call Optimization**: Compatible with TCO for deeply nested structures
//!
//! # Built-in Visitors
//!
//! ## NodeCounter
//!
//! A utility visitor that counts different types of AST nodes:
//!
//! ```rust,ignore
//! let mut counter = NodeCounter::default();
//! counter.visit_program(&program);
//! println!("Found {} lambdas and {} applications",
//!          counter.lambdas, counter.applications);
//! ```

use super::*;
use crate::diagnostics::{Result, Spanned};

/// Core trait for immutable AST traversal following the visitor pattern.
///
/// This trait defines a comprehensive interface for traversing Lambdust ASTs
/// without modifying the tree structure. Each visit method corresponds to a
/// specific AST node type, enabling fine-grained control over traversal behavior.
///
/// # R7RS Semantic Compliance
///
/// The visitor methods are organized according to R7RS Scheme language structure:
///
/// - **Literals**: Self-evaluating expressions (numbers, strings, booleans)
/// - **Identifiers**: Variable and procedure references
/// - **Special Forms**: The 10 fundamental language constructs
/// - **Derived Forms**: Standard macros expanded from special forms
/// - **Library Forms**: Module system constructs
///
/// # Implementation Strategy
///
/// The default `visit_expr` implementation provides a comprehensive dispatch
/// mechanism that routes to specialized methods based on expression type.
/// Implementors typically override specific visit methods rather than the
/// main dispatch logic.
///
/// # Type Parameters
///
/// - `Output`: The result type returned by visit operations
pub trait Visitor {
    /// The type returned by all visit methods.
    ///
    /// Common choices include:
    /// - `()` for side-effect based visitors
    /// - `String` for pretty-printing visitors  
    /// - `bool` for predicate visitors
    /// - Custom types for analysis results
    type Output;

    /// Visit a complete Lambdust program.
    ///
    /// A program consists of a sequence of top-level expressions, which may
    /// include definitions, library declarations, and expressions to evaluate.
    ///
    /// # R7RS Context
    ///
    /// Corresponds to the `<program>` production in R7RS grammar.
    /// The default implementation delegates to `visit_expressions`.
    ///
    /// # Arguments
    ///
    /// - `program`: The program to visit
    fn visit_program(&mut self, program: &Program) -> Self::Output {
        self.visit_expressions(&program.expressions)
    }

    /// Visit a sequence of expressions.
    ///
    /// This is the fundamental operation that handles expression sequences
    /// in various contexts (program bodies, lambda bodies, let bodies, etc.).
    ///
    /// # Implementation Note
    ///
    /// This method must be implemented by all visitors as it defines
    /// how to combine results from multiple expressions.
    ///
    /// # Arguments
    ///
    /// - `expressions`: Sequence of spanned expressions to visit
    fn visit_expressions(&mut self, expressions: &[Spanned<Expr>]) -> Self::Output;

    /// Visit a single expression with automatic dispatch based on expression type.
    ///
    /// This is the core dispatch method that routes expressions to their
    /// appropriate specialized visit methods. The default implementation
    /// provides comprehensive coverage of all Lambdust expression types.
    ///
    /// # R7RS Expression Classification
    ///
    /// The dispatch covers all expression types defined in R7RS:
    ///
    /// - **Literals**: Numbers, strings, booleans, characters
    /// - **Variables**: Identifiers, symbols, keywords  
    /// - **Special Forms**: Core language constructs (R7RS Section 4.1)
    /// - **Derived Forms**: Standard macro expansions (R7RS Section 4.2)
    /// - **Library Forms**: Module system constructs (R7RS Section 5)
    ///
    /// # Arguments
    ///
    /// - `expr`: The spanned expression to visit
    fn visit_expr(&mut self, expr: &Spanned<Expr>) -> Self::Output {
        match &expr.inner {
            Expr::Literal(lit) => self.visit_literal(lit),
            Expr::Identifier(name) => self.visit_identifier(name),
            Expr::Symbol(name) => self.visit_identifier(name), // Handle Symbol same as Identifier
            Expr::Keyword(name) => self.visit_keyword(name),
            Expr::List(elements) => self.visit_list(elements),
            Expr::Quote(expr) => self.visit_quote(expr),
            Expr::Quasiquote(expr) => self.visit_quasiquote(expr),
            Expr::Unquote(expr) => self.visit_unquote(expr),
            Expr::UnquoteSplicing(expr) => self.visit_unquote_splicing(expr),
            Expr::ExternalForm { tag, args } => self.visit_external_form(tag, args),
            Expr::Lambda {
                formals,
                metadata,
                body,
                ..
            } => self.visit_lambda(formals, metadata, body),
            Expr::CaseLambda {
                clauses, metadata, ..
            } => self.visit_case_lambda(clauses, metadata),
            Expr::If {
                test,
                consequent,
                alternative,
            } => self.visit_if(
                test,
                consequent,
                alternative.as_ref().map(|boxed| boxed.as_ref()),
            ),
            Expr::Define {
                name,
                value,
                metadata,
                ..
            } => self.visit_define(name, value, metadata),
            Expr::Set { name, value } => self.visit_set(name, value),
            Expr::DefineSyntax { name, transformer } => self.visit_define_syntax(name, transformer),
            Expr::SyntaxRules { literals, rules } => self.visit_syntax_rules(literals, rules),
            Expr::CallCC(expr) => self.visit_call_cc(expr),
            Expr::Delay { expression } => self.visit_delay(expression),
            Expr::Primitive { name, args } => self.visit_primitive(name, args),
            Expr::TypeAnnotation { expr, type_expr } => self.visit_type_annotation(expr, type_expr),
            Expr::Application { operator, operands } => self.visit_application(operator, operands),
            Expr::Begin(exprs) => self.visit_begin(exprs),
            Expr::Let { bindings, body } => self.visit_let(bindings, body),
            Expr::LetStar { bindings, body } => self.visit_let_star(bindings, body),
            Expr::LetRec { bindings, body } => self.visit_let_rec(bindings, body),
            Expr::Cond(clauses) => self.visit_cond(clauses),
            Expr::Case { expr, clauses } => self.visit_case(expr, clauses),
            Expr::And(exprs) => self.visit_and(exprs),
            Expr::Or(exprs) => self.visit_or(exprs),
            Expr::When { test, body } => self.visit_when(test, body),
            Expr::Unless { test, body } => self.visit_unless(test, body),
            Expr::Pair { car, cdr } => self.visit_pair(car, cdr),
            Expr::Guard {
                variable,
                clauses,
                body,
            } => self.visit_guard(variable, clauses, body),
            Expr::Parameterize { bindings, body } => self.visit_parameterize(bindings, body),
            Expr::Import { import_specs } => self.visit_import(import_specs),
            Expr::DefineLibrary {
                name,
                imports,
                exports,
                body,
            } => self.visit_define_library(name, imports, exports, body),
            Expr::DefineContract { name, contract, .. } => {
                self.visit_define_contract(name, contract)
            }
            Expr::Contract(contract) => self.visit_contract(contract),
            Expr::ContractApplication { contract, expr } => {
                self.visit_contract_application(contract, expr)
            }
            Expr::CondExpand {
                clauses,
                else_clause,
            } => self.visit_cond_expand(clauses, else_clause),
            Expr::Cut {
                procedure,
                arguments,
            } => self.visit_cut(procedure, arguments),
            Expr::Cute {
                procedure,
                arguments,
            } => self.visit_cute(procedure, arguments),
            Expr::Lazy { expression } => self.visit_expr(expression),
            Expr::Eager { expression } => self.visit_expr(expression),
            Expr::AndLetStar { clauses, body } => self.visit_and_let_star(clauses, body),
        }
    }

    // =========================================================================
    // LITERAL AND IDENTIFIER VISIT METHODS
    // =========================================================================

    /// Visit a literal expression.
    ///
    /// Handles all self-evaluating expressions including numbers, strings,
    /// booleans, characters, and bytevectors.
    ///
    /// # R7RS Reference
    ///
    /// Corresponds to R7RS Section 4.1.2 "Literal expressions".
    /// All literal expressions evaluate to themselves.
    ///
    /// # Arguments
    ///
    /// - `literal`: The literal value to visit
    fn visit_literal(&mut self, literal: &Literal) -> Self::Output;

    /// Visit an identifier expression.
    ///
    /// Handles variable references and procedure names. The identifier
    /// is resolved through the lexical environment during evaluation.
    ///
    /// # R7RS Reference
    ///
    /// Corresponds to R7RS Section 4.1.1 "Variable references".
    /// Identifiers are looked up in the current lexical environment.
    ///
    /// # Arguments
    ///
    /// - `name`: The identifier name as a string
    fn visit_identifier(&mut self, name: &str) -> Self::Output;

    /// Visit a keyword expression.
    ///
    /// Keywords are self-evaluating symbols prefixed with `#:` that
    /// are commonly used for named parameters and options.
    ///
    /// # Lambdust Extension
    ///
    /// Keywords are a Lambdust extension for improved ergonomics,
    /// particularly useful in function calls with named parameters.
    ///
    /// # Arguments
    ///
    /// - `name`: The keyword name (without the `#:` prefix)
    fn visit_keyword(&mut self, name: &str) -> Self::Output;

    // =========================================================================
    // QUOTE FAMILY VISIT METHODS - R7RS Section 4.1.2
    // =========================================================================

    /// Visit a quote expression: `(quote <datum>)` or `'<datum>`.
    ///
    /// Quote prevents evaluation of its argument, returning the datum literally.
    /// This is fundamental to Lisp metaprogramming and data manipulation.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.1.2: "(quote <datum>)" returns <datum> without evaluation.
    /// The datum may be any external representation of a Scheme object.
    ///
    /// # Arguments
    ///
    /// - `expr`: The quoted expression
    fn visit_quote(&mut self, expr: &Spanned<Expr>) -> Self::Output;

    /// Visit a quasiquote expression: `(quasiquote <datum>)` or `` `<datum>``.
    ///
    /// Quasiquote is like quote but allows selective evaluation through
    /// unquote and unquote-splicing. Essential for macro systems.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.6: "Quasiquotation" - allows template-based
    /// code generation with selective evaluation.
    ///
    /// # Arguments
    ///
    /// - `expr`: The quasiquoted expression
    fn visit_quasiquote(&mut self, expr: &Spanned<Expr>) -> Self::Output;

    /// Visit an unquote expression: `(unquote <expression>)` or `,<expression>`.
    ///
    /// Unquote forces evaluation within a quasiquoted context.
    /// Only meaningful inside quasiquote templates.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.6: Unquote evaluates its argument within
    /// a quasiquote template.
    ///
    /// # Arguments
    ///
    /// - `expr`: The expression to unquote
    fn visit_unquote(&mut self, expr: &Spanned<Expr>) -> Self::Output;

    /// Visit an unquote-splicing expression: `(unquote-splicing <expr>)` or `,@<expr>`.
    ///
    /// Unquote-splicing evaluates its argument (which must be a list) and
    /// splices the result into the containing list structure.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.6: Unquote-splicing inserts the elements of a list
    /// into the containing template.
    ///
    /// # Arguments
    ///
    /// - `expr`: The expression to unquote and splice
    fn visit_unquote_splicing(&mut self, expr: &Spanned<Expr>) -> Self::Output;
    /// Visit an external form (SRFI-10).
    fn visit_external_form(&mut self, tag: &str, args: &[Spanned<Expr>]) -> Self::Output;

    // =========================================================================
    // LAMBDA EXPRESSIONS - R7RS Section 4.1.4
    // =========================================================================

    /// Visit a lambda expression: `(lambda <formals> <body>)`.
    ///
    /// Lambda creates anonymous procedures with lexical scoping.
    /// The formals specify the parameter structure, and the body
    /// contains the procedure's implementation.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.1.4: "Lambda expressions" create procedures.
    /// The formals can be fixed, variable, or mixed parameter lists.
    ///
    /// # Arguments
    ///
    /// - `formals`: Parameter specification (fixed, variable, or mixed)
    /// - `metadata`: Optional metadata annotations (type hints, etc.)
    /// - `body`: Sequence of expressions forming the procedure body
    fn visit_lambda(
        &mut self,
        formals: &Formals,
        metadata: &HashMap<String, Spanned<Expr>>,
        body: &[Spanned<Expr>],
    ) -> Self::Output;

    /// Visit a case-lambda expression with multiple arities.
    ///
    /// Case-lambda allows defining procedures that dispatch based on
    /// the number of arguments provided, enabling flexible APIs.
    ///
    /// # SRFI Context
    ///
    /// Defined in SRFI-16 "Syntax for procedures of variable arity".
    /// Provides pattern matching on argument count.
    ///
    /// # Arguments
    ///
    /// - `clauses`: List of (formals body...) clauses for different arities
    /// - `metadata`: Optional metadata annotations
    fn visit_case_lambda(
        &mut self,
        clauses: &[CaseLambdaClause],
        metadata: &HashMap<String, Spanned<Expr>>,
    ) -> Self::Output;

    // =========================================================================
    // CONDITIONAL EXPRESSIONS - R7RS Section 4.1.5
    // =========================================================================

    /// Visit an if expression: `(if <test> <consequent> <alternative>)`.
    ///
    /// Conditional evaluation based on the truth value of the test expression.
    /// If alternative is omitted and test is false, the result is unspecified.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.1.5: "Conditionals" - fundamental conditional construct.
    /// All values except #f are considered true in the test.
    ///
    /// # Arguments
    ///
    /// - `test`: Expression to evaluate for truth value
    /// - `consequent`: Expression to evaluate if test is true
    /// - `alternative`: Optional expression to evaluate if test is false
    fn visit_if(
        &mut self,
        test: &Spanned<Expr>,
        consequent: &Spanned<Expr>,
        alternative: Option<&Spanned<Expr>>,
    ) -> Self::Output;

    // =========================================================================
    // DEFINITION EXPRESSIONS - R7RS Section 4.1.6
    // =========================================================================

    /// Visit a definition: `(define <variable> <expression>)`.
    ///
    /// Creates new variable bindings in the current environment.
    /// The variable is bound to the value of the expression.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.1.6: "Definitions" create variable bindings.
    /// Definitions are only valid at the top level or at the beginning
    /// of a body.
    ///
    /// # Arguments
    ///
    /// - `name`: The variable name being defined
    /// - `value`: The expression whose value is bound to the variable
    /// - `metadata`: Optional metadata (type annotations, etc.)
    fn visit_define(
        &mut self,
        name: &str,
        value: &Spanned<Expr>,
        metadata: &HashMap<String, Spanned<Expr>>,
    ) -> Self::Output;

    /// Visit an assignment: `(set! <variable> <expression>)`.
    ///
    /// Modifies an existing variable binding. The variable must already
    /// exist in the current lexical environment.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.1.6: "Assignments" modify existing variable bindings.
    /// It is an error if the variable is not bound.
    ///
    /// # Arguments
    ///
    /// - `name`: The variable name to modify
    /// - `value`: The new value expression
    fn visit_set(&mut self, name: &str, value: &Spanned<Expr>) -> Self::Output;

    // =========================================================================
    // MACRO SYSTEM - R7RS Section 4.3
    // =========================================================================

    /// Visit a syntax definition: `(define-syntax <name> <transformer>)`.
    ///
    /// Defines new syntax transformers (macros) that extend the language.
    /// The transformer typically uses syntax-rules or similar mechanisms.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.3.1: "Binding constructs for syntactic keywords".
    /// Syntax definitions create macro transformers in the current environment.
    ///
    /// # Arguments
    ///
    /// - `name`: The syntax keyword being defined
    /// - `transformer`: The transformation procedure or syntax-rules
    fn visit_define_syntax(&mut self, name: &str, transformer: &Spanned<Expr>) -> Self::Output;

    /// Visit syntax-rules: `(syntax-rules <literals> <rule>...)`.
    ///
    /// Defines pattern-based macro transformations with hygenic expansion.
    /// Each rule consists of a pattern and a template.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.3.2: "Syntax-rules" provides pattern-based macro system
    /// with automatic variable capture avoidance (hygiene).
    ///
    /// # Arguments
    ///
    /// - `literals`: List of literal identifiers that match only themselves
    /// - `rules`: List of (pattern, template) transformation rules
    fn visit_syntax_rules(
        &mut self,
        literals: &[String],
        rules: &[(Spanned<Expr>, Spanned<Expr>)],
    ) -> Self::Output;

    // =========================================================================
    // CONTINUATION SYSTEM - R7RS Section 6.10
    // =========================================================================

    /// Visit call/cc: `(call-with-current-continuation <procedure>)`.
    ///
    /// Captures the current continuation and passes it to the procedure.
    /// Essential for implementing advanced control flow patterns.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 6.10: "Control features" - continuations provide
    /// first-class control abstractions.
    ///
    /// # Arguments
    ///
    /// - `expr`: The procedure to call with the current continuation
    fn visit_call_cc(&mut self, expr: &Spanned<Expr>) -> Self::Output;

    /// Visit a delay form.
    ///
    /// Delay creates a promise that defers evaluation of its expression until forced.
    /// This enables lazy evaluation and lazy data structures.
    ///
    /// # Arguments
    ///
    /// - `expression`: The expression to defer evaluation for
    fn visit_delay(&mut self, expression: &Spanned<Expr>) -> Self::Output;

    // =========================================================================
    // PRIMITIVE OPERATIONS
    // =========================================================================

    /// Visit a primitive operation call.
    ///
    /// Primitives are built-in operations implemented in Rust rather than
    /// Scheme code. They provide the foundational operations of the language.
    ///
    /// # Lambdust Implementation
    ///
    /// Primitives include arithmetic, I/O, type predicates, and other
    /// fundamental operations defined in the standard library.
    ///
    /// # Arguments
    ///
    /// - `name`: The primitive operation name
    /// - `args`: List of argument expressions
    fn visit_primitive(&mut self, name: &str, args: &[Spanned<Expr>]) -> Self::Output;

    /// Visit a type annotation: `(:: <expression> <type>)`.
    ///
    /// Provides type information for the expression, used by the type
    /// inference system and for documentation purposes.
    ///
    /// # Lambdust Extension
    ///
    /// Type annotations are a Lambdust extension that enhances the
    /// gradual typing system and improves IDE support.
    ///
    /// # Arguments
    ///
    /// - `expr`: The expression being annotated
    /// - `type_expr`: The type expression
    fn visit_type_annotation(
        &mut self,
        expr: &Spanned<Expr>,
        type_expr: &Spanned<Expr>,
    ) -> Self::Output;

    // =========================================================================
    // PROCEDURE APPLICATIONS - R7RS Section 4.1.3
    // =========================================================================

    /// Visit a procedure application: `(<operator> <operand>...)`.
    ///
    /// The fundamental operation for calling procedures. The operator
    /// is evaluated to obtain a procedure, then applied to the operands.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.1.3: "Procedure calls" describe the evaluation
    /// of procedure applications with proper tail recursion.
    ///
    /// # Arguments
    ///
    /// - `operator`: The procedure expression to call
    /// - `operands`: List of argument expressions
    fn visit_application(
        &mut self,
        operator: &Spanned<Expr>,
        operands: &[Spanned<Expr>],
    ) -> Self::Output;

    // =========================================================================
    // DERIVED FORMS - R7RS Section 4.2
    // =========================================================================

    /// Visit a begin expression: `(begin <expression>...)`.
    ///
    /// Evaluates expressions sequentially and returns the value of the last.
    /// Used for grouping side-effecting expressions.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.3: "Sequencing" - begin evaluates expressions
    /// in order and returns the value of the last expression.
    ///
    /// # Arguments
    ///
    /// - `exprs`: Sequence of expressions to evaluate
    fn visit_begin(&mut self, exprs: &[Spanned<Expr>]) -> Self::Output;

    /// Visit a let expression: `(let <bindings> <body>...)`.
    ///
    /// Creates local variable bindings with simultaneous initialization.
    /// Variables are not visible to their own initialization expressions.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.2: "Binding constructs" - let provides
    /// parallel variable binding with lexical scoping.
    ///
    /// # Arguments
    ///
    /// - `bindings`: List of (variable, expression) bindings
    /// - `body`: Sequence of expressions forming the let body
    fn visit_let(&mut self, bindings: &[Binding], body: &[Spanned<Expr>]) -> Self::Output;

    /// Visit a let* expression: `(let* <bindings> <body>...)`.
    ///
    /// Creates local variable bindings with sequential initialization.
    /// Each variable is visible to subsequent binding expressions.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.2: "Binding constructs" - let* provides
    /// sequential variable binding from left to right.
    ///
    /// # Arguments
    ///
    /// - `bindings`: List of (variable, expression) bindings (evaluated sequentially)
    /// - `body`: Sequence of expressions forming the let* body
    fn visit_let_star(&mut self, bindings: &[Binding], body: &[Spanned<Expr>]) -> Self::Output;

    /// Visit a letrec expression: `(letrec <bindings> <body>...)`.
    ///
    /// Creates local variable bindings for mutually recursive definitions.
    /// All variables are visible to all binding expressions.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.2: "Binding constructs" - letrec enables
    /// mutually recursive local definitions.
    ///
    /// # Arguments
    ///
    /// - `bindings`: List of (variable, expression) bindings (all mutually visible)
    /// - `body`: Sequence of expressions forming the letrec body
    fn visit_let_rec(&mut self, bindings: &[Binding], body: &[Spanned<Expr>]) -> Self::Output;

    /// Visit an and-let* expression: `(and-let* <clauses> <body>...)`.
    ///
    /// SRFI-2 conditional binding with short-circuit evaluation.
    /// Each clause can bind a variable or test a condition; evaluation stops
    /// at the first clause that evaluates to #f.
    ///
    /// # SRFI-2 Reference
    ///
    /// SRFI-2: "and-let*: an AND with local bindings, a guarded LET* special form"
    /// Provides conditional binding with early termination on false values.
    ///
    /// # Arguments
    ///
    /// - `clauses`: The and-let* clauses (bindings and tests)
    /// - `body`: The body expressions to evaluate if all clauses succeed
    fn visit_and_let_star(
        &mut self,
        clauses: &[AndLetClause],
        body: &[Spanned<Expr>],
    ) -> Self::Output;

    /// Visit a cond expression: `(cond <clause>...)`.
    ///
    /// Multi-branch conditional with test-result clause structure.
    /// Provides more readable alternative to nested if expressions.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.1: "Conditionals" - cond provides
    /// multi-way conditional branching with optional else clause.
    ///
    /// # Arguments
    ///
    /// - `clauses`: List of conditional clauses (test, result expressions)
    fn visit_cond(&mut self, clauses: &[CondClause]) -> Self::Output;

    /// Visit a case expression: `(case <key> <clause>...)`.
    ///
    /// Dispatches based on the value of a key expression compared
    /// against literal values using eqv?.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.1: "Conditionals" - case provides
    /// efficient dispatching based on eqv? equality.
    ///
    /// # Arguments
    ///
    /// - `expr`: The key expression to match against
    /// - `clauses`: List of case clauses with match values and result expressions
    fn visit_case(&mut self, expr: &Spanned<Expr>, clauses: &[CaseClause]) -> Self::Output;

    /// Visit an and expression: `(and <test>...)`.
    ///
    /// Logical AND with short-circuit evaluation. Returns the first
    /// false value or the last value if all are true.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.1: "Conditionals" - and provides
    /// short-circuiting logical conjunction.
    ///
    /// # Arguments
    ///
    /// - `exprs`: Sequence of test expressions
    fn visit_and(&mut self, exprs: &[Spanned<Expr>]) -> Self::Output;

    /// Visit an or expression: `(or <test>...)`.
    ///
    /// Logical OR with short-circuit evaluation. Returns the first
    /// true value or #f if all are false.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.1: "Conditionals" - or provides
    /// short-circuiting logical disjunction.
    ///
    /// # Arguments
    ///
    /// - `exprs`: Sequence of test expressions
    fn visit_or(&mut self, exprs: &[Spanned<Expr>]) -> Self::Output;

    /// Visit a when expression: `(when <test> <expression>...)`.
    ///
    /// Evaluates body expressions only if test is true. Returns
    /// unspecified value if test is false.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.1: "Conditionals" - when provides
    /// conditional execution for side effects.
    ///
    /// # Arguments
    ///
    /// - `test`: Conditional test expression
    /// - `body`: Expressions to evaluate if test is true
    fn visit_when(&mut self, test: &Spanned<Expr>, body: &[Spanned<Expr>]) -> Self::Output;

    /// Visit an unless expression: `(unless <test> <expression>...)`.
    ///
    /// Evaluates body expressions only if test is false. Returns
    /// unspecified value if test is true.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.1: "Conditionals" - unless provides
    /// conditional execution for side effects (inverse of when).
    ///
    /// # Arguments
    ///
    /// - `test`: Conditional test expression
    /// - `body`: Expressions to evaluate if test is false
    fn visit_unless(&mut self, test: &Spanned<Expr>, body: &[Spanned<Expr>]) -> Self::Output;

    // =========================================================================
    // DATA STRUCTURES AND EXCEPTIONS
    // =========================================================================

    /// Visit a pair (cons cell): `(<car> . <cdr>)`.
    ///
    /// Fundamental data structure in Scheme, used to build lists
    /// and represent compound data with two components.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 6.4: "Pairs and lists" - pairs are the basic
    /// building block for constructing compound data structures.
    ///
    /// # Arguments
    ///
    /// - `car`: The first element of the pair
    /// - `cdr`: The second element of the pair
    fn visit_pair(&mut self, car: &Spanned<Expr>, cdr: &Spanned<Expr>) -> Self::Output;

    /// Visit a guard expression: `(guard (<var> <clause>...) <body>...)`.
    ///
    /// Exception handling construct that catches exceptions and
    /// matches them against conditional clauses.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 6.11: "Exceptions" - guard provides structured
    /// exception handling with pattern matching on exception objects.
    ///
    /// # Arguments
    ///
    /// - `variable`: Variable name bound to the exception object
    /// - `clauses`: Exception handling clauses with conditions
    /// - `body`: Protected expressions that may raise exceptions
    fn visit_guard(
        &mut self,
        variable: &str,
        clauses: &[GuardClause],
        body: &[Spanned<Expr>],
    ) -> Self::Output;

    /// Visit a parameterize expression: `(parameterize <bindings> <body>...)`.
    ///
    /// Dynamically binds parameter objects to new values for the
    /// duration of the body evaluation.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 4.2.6: "Dynamic bindings" - parameterize
    /// provides dynamic scoping for parameter objects.
    ///
    /// # Arguments
    ///
    /// - `bindings`: Parameter-value pairs for dynamic binding
    /// - `body`: Expressions evaluated with new parameter values
    fn visit_parameterize(
        &mut self,
        bindings: &[ParameterBinding],
        body: &[Spanned<Expr>],
    ) -> Self::Output;

    // =========================================================================
    // MODULE SYSTEM - R7RS Section 5
    // =========================================================================

    /// Visit an import declaration: `(import <import-spec>...)`.
    ///
    /// Imports bindings from specified libraries into the current
    /// environment, enabling modular program organization.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 5.2: "Import declarations" control which
    /// library bindings are available in the current environment.
    ///
    /// # Arguments
    ///
    /// - `import_specs`: Specifications of libraries and bindings to import
    fn visit_import(&mut self, import_specs: &[Spanned<Expr>]) -> Self::Output;

    /// Visit a library definition: `(define-library <name> <library-decl>...)`.
    ///
    /// Defines a new library with specified name, imports, exports,
    /// and implementation body.
    ///
    /// # R7RS Reference
    ///
    /// R7RS Section 5.6: "Library syntax" - define-library creates
    /// new library modules with controlled visibility.
    ///
    /// # Arguments
    ///
    /// - `name`: Library name as a list of identifiers
    /// - `imports`: Import declarations for dependencies
    /// - `exports`: Export declarations specifying public interface
    /// - `body`: Library implementation expressions
    fn visit_define_library(
        &mut self,
        name: &[String],
        imports: &[Spanned<Expr>],
        exports: &[Spanned<Expr>],
        body: &[Spanned<Expr>],
    ) -> Self::Output;

    /// Visit a generic list: `(<element>...)`.
    ///
    /// Handles list structures that don't match specific special forms.
    /// Used primarily for data literals and generic list processing.
    ///
    /// # Implementation Note
    ///
    /// This method handles list expressions that aren't procedure
    /// applications or special forms.
    ///
    /// # Arguments
    ///
    /// - `elements`: List of expressions contained in the list
    fn visit_list(&mut self, elements: &[Spanned<Expr>]) -> Self::Output;

    // =========================================================================
    // CONTRACT SYSTEM - Lambdust Extension
    // =========================================================================

    /// Visit a contract definition: `(define/contract <name> <contract> <body>...)`.
    ///
    /// Defines a function with an associated behavioral contract that
    /// specifies preconditions, postconditions, and other behavioral constraints.
    ///
    /// # Contract System
    ///
    /// Lambdust's contract system provides runtime verification of
    /// function behavior, enabling design-by-contract programming.
    ///
    /// # Arguments
    ///
    /// - `name`: The function name being defined
    /// - `contract`: The behavioral contract specification
    fn visit_define_contract(
        &mut self,
        name: &str,
        contract: &Spanned<crate::contracts::ast::ContractExpr>,
    ) -> Self::Output;

    /// Visit a contract expression.
    ///
    /// Handles contract literals and contract combinators used
    /// in contract specifications.
    ///
    /// # Contract System
    ///
    /// Contracts can be simple predicates or complex compositions
    /// using contract combinators like and/c, or/c, ->c, etc.
    ///
    /// # Arguments
    ///
    /// - `contract`: The contract expression to visit
    fn visit_contract(
        &mut self,
        contract: &Spanned<crate::contracts::ast::ContractExpr>,
    ) -> Self::Output;

    /// Visit a contract application: `(contract <expr>)`.
    ///
    /// Applies a contract to an expression, providing runtime
    /// verification of the contract's constraints.
    ///
    /// # Contract System
    ///
    /// Contract application enables gradual introduction of contracts
    /// and runtime verification of behavioral properties.
    ///
    /// # Arguments
    ///
    /// - `contract`: The contract to apply
    /// - `expr`: The expression to which the contract is applied
    fn visit_contract_application(
        &mut self,
        contract: &Spanned<crate::contracts::ast::ContractExpr>,
        expr: &Spanned<Expr>,
    ) -> Self::Output;

    /// Visits a cond-expand expression (SRFI-0).
    ///
    /// This method processes conditional expansion forms that enable
    /// feature-based inclusion of code at compile time.
    ///
    /// # Arguments
    ///
    /// - `clauses`: Feature-condition clauses to evaluate
    /// - `else_clause`: Optional else clause for fallback
    fn visit_cond_expand(
        &mut self,
        clauses: &[CondExpandClause],
        else_clause: &Option<Vec<Spanned<Expr>>>,
    ) -> Self::Output;

    /// Visit a cut expression (SRFI-26).
    ///
    /// Processes cut expressions which create specialized procedures
    /// with some arguments fixed and others represented by slots.
    ///
    /// # Arguments
    ///
    /// - `procedure`: The procedure being specialized
    /// - `arguments`: Arguments containing slots and expressions
    fn visit_cut(
        &mut self,
        procedure: &Spanned<Expr>,
        arguments: &[crate::ast::CutArgument],
    ) -> Self::Output;

    /// Visit a cute expression (SRFI-26).
    ///
    /// Processes cute expressions which create specialized procedures
    /// with eager evaluation of non-slot expressions.
    ///
    /// # Arguments
    ///
    /// - `procedure`: The procedure being specialized
    /// - `arguments`: Arguments containing slots and expressions
    fn visit_cute(
        &mut self,
        procedure: &Spanned<Expr>,
        arguments: &[crate::ast::CutArgument],
    ) -> Self::Output;
}

/// Mutable visitor trait for transforming AST nodes.
///
/// This trait enables AST transformations by providing mutable access
/// to AST nodes during traversal. It's designed for implementing
/// optimizations, macro expansions, and other tree transformations.
///
/// # Use Cases
///
/// - **Macro Expansion**: Transform syntax-rules patterns into expanded forms
/// - **Optimization Passes**: Apply various optimization transformations
/// - **Desugaring**: Convert derived forms to core special forms
/// - **Type Inference**: Annotate nodes with inferred type information
/// - **Code Generation**: Transform high-level ASTs to lower-level forms
///
/// # Error Handling
///
/// Unlike the immutable `Visitor` trait, `VisitorMut` explicitly handles
/// errors through the `Error` associated type, enabling robust transformation
/// pipelines with proper error reporting.
///
/// # Implementation Strategy
///
/// The mutable visitor provides a general framework but typically requires
/// custom implementation of `visit_expr_mut` for specific transformations.
/// The default `visit_program_mut` delegates to `visit_expressions_mut`.
///
/// # Type Parameters
///
/// - `Output`: The result type of successful transformations
/// - `Error`: The error type for failed transformations
pub trait VisitorMut {
    /// The type returned by successful visit operations.
    type Output;

    /// The error type for failed transformations.
    type Error;

    /// Visit a program, potentially modifying it.
    ///
    /// Transforms the entire program by visiting and potentially modifying
    /// all top-level expressions. The default implementation delegates
    /// to `visit_expressions_mut`.
    ///
    /// # Arguments
    ///
    /// - `program`: Mutable reference to the program to transform
    ///
    /// # Returns
    ///
    /// Result containing the transformation output or an error
    fn visit_program_mut(&mut self, program: &mut Program) -> Result<Self::Output> {
        self.visit_expressions_mut(&mut program.expressions)
    }

    /// Visit a list of expressions, potentially modifying them.
    ///
    /// This is the core method for transforming sequences of expressions.
    /// Implementations must handle the possibility of expressions being
    /// added, removed, or replaced during transformation.
    ///
    /// # Implementation Requirements
    ///
    /// This method must be implemented by all mutable visitors as it
    /// defines how to process and combine results from multiple expressions.
    ///
    /// # Arguments
    ///
    /// - `expressions`: Mutable vector of expressions to transform
    ///
    /// # Returns
    ///
    /// Result containing the transformation output or an error
    fn visit_expressions_mut(
        &mut self,
        expressions: &mut Vec<Spanned<Expr>>,
    ) -> Result<Self::Output>;

    /// Visit an expression, potentially modifying it.
    ///
    /// The fundamental transformation method that handles individual
    /// expressions. Implementations typically pattern-match on expression
    /// types and apply appropriate transformations.
    ///
    /// # Implementation Requirements
    ///
    /// Most mutable visitors will need to implement this method to
    /// define their specific transformation behavior.
    ///
    /// # Arguments
    ///
    /// - `expr`: Mutable reference to the expression to transform
    ///
    /// # Returns
    ///
    /// Result containing the transformation output or an error
    fn visit_expr_mut(&mut self, expr: &mut Spanned<Expr>) -> Result<Self::Output>;
}

/// A utility visitor that counts different types of AST nodes.
///
/// `NodeCounter` provides statistical analysis of AST structure by
/// counting occurrences of each expression type. This is useful for:
///
/// - **Profiling**: Understanding code complexity and structure
/// - **Optimization**: Identifying hotspots for optimization
/// - **Analysis**: Gathering metrics about code patterns
/// - **Testing**: Verifying parser correctness and completeness
///
/// # Usage Example
///
/// ```rust,ignore
/// use lambdust::ast::{NodeCounter, Visitor, Program};
///
/// let mut counter = NodeCounter::default();
/// counter.visit_program(&program);
///
/// println!("Code statistics:");
/// println!("  Literals: {}", counter.literals);
/// println!("  Lambdas: {}", counter.lambdas);
/// println!("  Applications: {}", counter.applications);
/// println!("  Total nodes: {}", counter.total);
/// ```
///
/// # Implementation Details
///
/// The counter uses depth-first traversal and increments counters for
/// each node type visited. The `total` field provides an aggregate count
/// of all AST nodes.
#[derive(Debug, Default)]
pub struct NodeCounter {
    /// Number of literal expressions (numbers, strings, booleans, etc.)
    pub literals: usize,

    /// Number of identifier and symbol expressions
    pub identifiers: usize,

    /// Number of keyword expressions
    pub keywords: usize,

    /// Number of quote-family expressions (quote, quasiquote, unquote, unquote-splicing)
    pub quotes: usize,

    /// Number of lambda and case-lambda expressions
    pub lambdas: usize,

    /// Number of if expressions
    pub ifs: usize,

    /// Number of define expressions
    pub defines: usize,

    /// Number of set! expressions
    pub sets: usize,

    /// Number of procedure applications
    pub applications: usize,

    /// Total number of AST nodes visited
    pub total: usize,
}

impl Visitor for NodeCounter {
    type Output = ();

    fn visit_expressions(&mut self, expressions: &[Spanned<Expr>]) {
        for expr in expressions {
            self.visit_expr(expr);
        }
    }

    fn visit_literal(&mut self, _literal: &Literal) {
        self.literals += 1;
        self.total += 1;
    }

    fn visit_identifier(&mut self, _name: &str) {
        self.identifiers += 1;
        self.total += 1;
    }

    fn visit_keyword(&mut self, _name: &str) {
        self.keywords += 1;
        self.total += 1;
    }

    fn visit_quote(&mut self, expr: &Spanned<Expr>) {
        self.quotes += 1;
        self.total += 1;
        self.visit_expr(expr);
    }

    fn visit_quasiquote(&mut self, expr: &Spanned<Expr>) {
        self.quotes += 1; // Count quasiquote as a quote variant
        self.total += 1;
        self.visit_expr(expr);
    }

    fn visit_unquote(&mut self, expr: &Spanned<Expr>) {
        self.quotes += 1; // Count unquote as a quote variant
        self.total += 1;
        self.visit_expr(expr);
    }

    fn visit_unquote_splicing(&mut self, expr: &Spanned<Expr>) {
        self.quotes += 1; // Count unquote-splicing as a quote variant
        self.total += 1;
        self.visit_expr(expr);
    }

    fn visit_external_form(&mut self, _tag: &str, args: &[Spanned<Expr>]) {
        self.total += 1;
        for arg in args {
            self.visit_expr(arg);
        }
    }

    fn visit_lambda(
        &mut self,
        _formals: &Formals,
        metadata: &HashMap<String, Spanned<Expr>>,
        body: &[Spanned<Expr>],
    ) {
        self.lambdas += 1;
        self.total += 1;

        for expr in metadata.values() {
            self.visit_expr(expr);
        }
        self.visit_expressions(body);
    }

    fn visit_case_lambda(
        &mut self,
        clauses: &[CaseLambdaClause],
        metadata: &HashMap<String, Spanned<Expr>>,
    ) {
        self.lambdas += 1; // Count case-lambda as a type of lambda
        self.total += 1;

        for expr in metadata.values() {
            self.visit_expr(expr);
        }

        for clause in clauses {
            self.visit_expressions(&clause.body);
        }
    }

    fn visit_if(
        &mut self,
        test: &Spanned<Expr>,
        consequent: &Spanned<Expr>,
        alternative: Option<&Spanned<Expr>>,
    ) {
        self.ifs += 1;
        self.total += 1;

        self.visit_expr(test);
        self.visit_expr(consequent);
        if let Some(alt) = alternative {
            self.visit_expr(alt);
        }
    }

    fn visit_define(
        &mut self,
        _name: &str,
        value: &Spanned<Expr>,
        metadata: &HashMap<String, Spanned<Expr>>,
    ) {
        self.defines += 1;
        self.total += 1;

        self.visit_expr(value);
        for expr in metadata.values() {
            self.visit_expr(expr);
        }
    }

    fn visit_set(&mut self, _name: &str, value: &Spanned<Expr>) {
        self.sets += 1;
        self.total += 1;
        self.visit_expr(value);
    }

    fn visit_define_syntax(&mut self, _name: &str, transformer: &Spanned<Expr>) {
        self.total += 1;
        self.visit_expr(transformer);
    }

    fn visit_syntax_rules(
        &mut self,
        _literals: &[String],
        rules: &[(Spanned<Expr>, Spanned<Expr>)],
    ) {
        self.total += 1;
        for (pattern, template) in rules {
            self.visit_expr(pattern);
            self.visit_expr(template);
        }
    }

    fn visit_call_cc(&mut self, expr: &Spanned<Expr>) {
        self.total += 1;
        self.visit_expr(expr);
    }

    fn visit_delay(&mut self, expression: &Spanned<Expr>) {
        self.total += 1;
        self.visit_expr(expression);
    }

    fn visit_primitive(&mut self, _name: &str, args: &[Spanned<Expr>]) {
        self.total += 1;
        self.visit_expressions(args);
    }

    fn visit_type_annotation(&mut self, expr: &Spanned<Expr>, type_expr: &Spanned<Expr>) {
        self.total += 1;
        self.visit_expr(expr);
        self.visit_expr(type_expr);
    }

    fn visit_application(&mut self, operator: &Spanned<Expr>, operands: &[Spanned<Expr>]) {
        self.applications += 1;
        self.total += 1;

        self.visit_expr(operator);
        self.visit_expressions(operands);
    }

    fn visit_begin(&mut self, exprs: &[Spanned<Expr>]) {
        self.total += 1;
        self.visit_expressions(exprs);
    }

    fn visit_let(&mut self, bindings: &[Binding], body: &[Spanned<Expr>]) {
        self.total += 1;
        for binding in bindings {
            self.visit_expr(&binding.value);
        }
        self.visit_expressions(body);
    }

    fn visit_let_star(&mut self, bindings: &[Binding], body: &[Spanned<Expr>]) {
        self.total += 1;
        for binding in bindings {
            self.visit_expr(&binding.value);
        }
        self.visit_expressions(body);
    }

    fn visit_let_rec(&mut self, bindings: &[Binding], body: &[Spanned<Expr>]) {
        self.total += 1;
        for binding in bindings {
            self.visit_expr(&binding.value);
        }
        self.visit_expressions(body);
    }

    fn visit_and_let_star(&mut self, clauses: &[AndLetClause], body: &[Spanned<Expr>]) {
        self.total += 1;
        for clause in clauses {
            self.visit_expr(clause.expression());
        }
        self.visit_expressions(body);
    }

    fn visit_cond(&mut self, clauses: &[CondClause]) {
        self.total += 1;
        for clause in clauses {
            self.visit_expr(&clause.test);
            self.visit_expressions(&clause.body);
        }
    }

    fn visit_case(&mut self, expr: &Spanned<Expr>, clauses: &[CaseClause]) {
        self.total += 1;
        self.visit_expr(expr);
        for clause in clauses {
            self.visit_expressions(&clause.values);
            self.visit_expressions(&clause.body);
        }
    }

    fn visit_and(&mut self, exprs: &[Spanned<Expr>]) {
        self.total += 1;
        self.visit_expressions(exprs);
    }

    fn visit_or(&mut self, exprs: &[Spanned<Expr>]) {
        self.total += 1;
        self.visit_expressions(exprs);
    }

    fn visit_when(&mut self, test: &Spanned<Expr>, body: &[Spanned<Expr>]) {
        self.total += 1;
        self.visit_expr(test);
        self.visit_expressions(body);
    }

    fn visit_unless(&mut self, test: &Spanned<Expr>, body: &[Spanned<Expr>]) {
        self.total += 1;
        self.visit_expr(test);
        self.visit_expressions(body);
    }

    fn visit_pair(&mut self, car: &Spanned<Expr>, cdr: &Spanned<Expr>) {
        self.total += 1;
        self.visit_expr(car);
        self.visit_expr(cdr);
    }

    fn visit_guard(&mut self, _variable: &str, clauses: &[GuardClause], body: &[Spanned<Expr>]) {
        self.total += 1;
        for clause in clauses {
            self.visit_expr(&clause.test);
            self.visit_expressions(&clause.body);
            if let Some(ref arrow) = clause.arrow {
                self.visit_expr(arrow);
            }
        }
        self.visit_expressions(body);
    }

    fn visit_parameterize(&mut self, bindings: &[ParameterBinding], body: &[Spanned<Expr>]) {
        self.total += 1;
        for binding in bindings {
            self.visit_expr(&binding.parameter);
            self.visit_expr(&binding.value);
        }
        self.visit_expressions(body);
    }

    fn visit_import(&mut self, import_specs: &[Spanned<Expr>]) {
        self.total += 1;
        self.visit_expressions(import_specs);
    }

    fn visit_define_library(
        &mut self,
        _name: &[String],
        imports: &[Spanned<Expr>],
        exports: &[Spanned<Expr>],
        body: &[Spanned<Expr>],
    ) {
        self.total += 1;
        self.visit_expressions(imports);
        self.visit_expressions(exports);
        self.visit_expressions(body);
    }

    fn visit_list(&mut self, elements: &[Spanned<Expr>]) {
        self.total += 1;
        self.visit_expressions(elements);
    }

    fn visit_define_contract(
        &mut self,
        _name: &str,
        _contract: &Spanned<crate::contracts::ast::ContractExpr>,
    ) {
        self.total += 1;
    }

    fn visit_contract(&mut self, _contract: &Spanned<crate::contracts::ast::ContractExpr>) {
        self.total += 1;
    }

    fn visit_contract_application(
        &mut self,
        _contract: &Spanned<crate::contracts::ast::ContractExpr>,
        expr: &Spanned<Expr>,
    ) {
        self.total += 1;
        self.visit_expr(expr);
    }

    fn visit_cond_expand(
        &mut self,
        clauses: &[CondExpandClause],
        else_clause: &Option<Vec<Spanned<Expr>>>,
    ) {
        self.total += 1;

        // Count expressions in clauses
        for clause in clauses {
            self.visit_expressions(&clause.body);
        }

        // Count expressions in else clause if present
        if let Some(else_body) = else_clause {
            self.visit_expressions(else_body);
        }
    }

    fn visit_cut(&mut self, procedure: &Spanned<Expr>, arguments: &[crate::ast::CutArgument]) {
        self.total += 1;
        self.visit_expr(procedure);

        // Visit expressions in arguments
        for arg in arguments {
            if let crate::ast::CutArgument::Expression(expr) = arg {
                self.visit_expr(expr);
            }
        }
    }

    fn visit_cute(&mut self, procedure: &Spanned<Expr>, arguments: &[crate::ast::CutArgument]) {
        self.total += 1;
        self.visit_expr(procedure);

        // Visit expressions in arguments
        for arg in arguments {
            if let crate::ast::CutArgument::Expression(expr) = arg {
                self.visit_expr(expr);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_node_counter() {
        let span = Span::new(0, 1);
        let mut program = Program::new();

        // (define x 42)
        let define_expr = Expr::Define {
            name: "x".to_string(),
            value: Box::new(Spanned::new(Expr::Literal(Literal::integer(42)), span)),
            return_type: None,
            metadata: HashMap::new(),
        };
        program.add_expression(Spanned::new(define_expr, span));

        let mut counter = NodeCounter::default();
        counter.visit_program(&program);

        assert_eq!(counter.defines, 1);
        assert_eq!(counter.literals, 1);
        assert_eq!(counter.total, 2);
    }
}
