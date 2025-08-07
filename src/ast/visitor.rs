//! Visitor pattern for traversing AST nodes.

#![allow(missing_docs)]

use super::*;
use crate::diagnostics::{Result, Spanned};

/// Trait for visiting AST nodes.
pub trait Visitor {
    type Output;

    /// Visit a program.
    fn visit_program(&mut self, program: &Program) -> Self::Output {
        self.visit_expressions(&program.expressions)
    }

    /// Visit a list of expressions.
    fn visit_expressions(&mut self, expressions: &[Spanned<Expr>]) -> Self::Output;

    /// Visit an expression.
    fn visit_expr(&mut self, expr: &Spanned<Expr>) -> Self::Output {
        match &expr.inner {
            Expr::Literal(lit) => self.visit_literal(lit),
            Expr::Identifier(name) => self.visit_identifier(name),
            Expr::Symbol(name) => self.visit_identifier(name), // Handle Symbol same as Identifier
            Expr::Keyword(name) => self.visit_keyword(name),
            Expr::List(elements) => self.visit_list(elements),
            Expr::Quote(expr) => self.visit_quote(expr),
            Expr::Lambda { formals, metadata, body } => {
                self.visit_lambda(formals, metadata, body)
            }
            Expr::CaseLambda { clauses, metadata } => {
                self.visit_case_lambda(clauses, metadata)
            }
            Expr::If { test, consequent, alternative } => {
                self.visit_if(test, consequent, alternative.as_ref().map(|boxed| boxed.as_ref()))
            }
            Expr::Define { name, value, metadata } => {
                self.visit_define(name, value, metadata)
            }
            Expr::Set { name, value } => self.visit_set(name, value),
            Expr::DefineSyntax { name, transformer } => {
                self.visit_define_syntax(name, transformer)
            }
            Expr::SyntaxRules { literals, rules } => {
                self.visit_syntax_rules(literals, rules)
            }
            Expr::CallCC(expr) => self.visit_call_cc(expr),
            Expr::Primitive { name, args } => self.visit_primitive(name, args),
            Expr::TypeAnnotation { expr, type_expr } => {
                self.visit_type_annotation(expr, type_expr)
            }
            Expr::Application { operator, operands } => {
                self.visit_application(operator, operands)
            }
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
            Expr::Guard { variable, clauses, body } => {
                self.visit_guard(variable, clauses, body)
            }
            Expr::Parameterize { bindings, body } => {
                self.visit_parameterize(bindings, body)
            }
            Expr::Import { import_specs } => {
                self.visit_import(import_specs)
            }
            Expr::DefineLibrary { name, imports, exports, body } => {
                self.visit_define_library(name, imports, exports, body)
            }
        }
    }

    // Individual visit methods for each expression type
    fn visit_literal(&mut self, literal: &Literal) -> Self::Output;
    fn visit_identifier(&mut self, name: &str) -> Self::Output;
    fn visit_keyword(&mut self, name: &str) -> Self::Output;
    fn visit_quote(&mut self, expr: &Spanned<Expr>) -> Self::Output;
    
    fn visit_lambda(
        &mut self,
        formals: &Formals,
        metadata: &HashMap<String, Spanned<Expr>>,
        body: &[Spanned<Expr>],
    ) -> Self::Output;
    
    fn visit_case_lambda(
        &mut self,
        clauses: &[CaseLambdaClause],
        metadata: &HashMap<String, Spanned<Expr>>,
    ) -> Self::Output;
    
    fn visit_if(
        &mut self,
        test: &Spanned<Expr>,
        consequent: &Spanned<Expr>,
        alternative: Option<&Spanned<Expr>>,
    ) -> Self::Output;
    
    fn visit_define(
        &mut self,
        name: &str,
        value: &Spanned<Expr>,
        metadata: &HashMap<String, Spanned<Expr>>,
    ) -> Self::Output;
    
    fn visit_set(&mut self, name: &str, value: &Spanned<Expr>) -> Self::Output;
    
    fn visit_define_syntax(&mut self, name: &str, transformer: &Spanned<Expr>) -> Self::Output;
    
    fn visit_syntax_rules(&mut self, literals: &[String], rules: &[(Spanned<Expr>, Spanned<Expr>)]) -> Self::Output;
    
    fn visit_call_cc(&mut self, expr: &Spanned<Expr>) -> Self::Output;
    
    fn visit_primitive(&mut self, name: &str, args: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_type_annotation(
        &mut self,
        expr: &Spanned<Expr>,
        type_expr: &Spanned<Expr>,
    ) -> Self::Output;
    
    fn visit_application(
        &mut self,
        operator: &Spanned<Expr>,
        operands: &[Spanned<Expr>],
    ) -> Self::Output;
    
    fn visit_begin(&mut self, exprs: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_let(&mut self, bindings: &[Binding], body: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_let_star(&mut self, bindings: &[Binding], body: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_let_rec(&mut self, bindings: &[Binding], body: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_cond(&mut self, clauses: &[CondClause]) -> Self::Output;
    
    fn visit_case(&mut self, expr: &Spanned<Expr>, clauses: &[CaseClause]) -> Self::Output;
    
    fn visit_and(&mut self, exprs: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_or(&mut self, exprs: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_when(&mut self, test: &Spanned<Expr>, body: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_unless(&mut self, test: &Spanned<Expr>, body: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_pair(&mut self, car: &Spanned<Expr>, cdr: &Spanned<Expr>) -> Self::Output;
    
    fn visit_guard(&mut self, variable: &str, clauses: &[GuardClause], body: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_parameterize(&mut self, bindings: &[ParameterBinding], body: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_import(&mut self, import_specs: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_define_library(&mut self, name: &[String], imports: &[Spanned<Expr>], exports: &[Spanned<Expr>], body: &[Spanned<Expr>]) -> Self::Output;
    
    fn visit_list(&mut self, elements: &[Spanned<Expr>]) -> Self::Output;
}

/// Mutable visitor trait for transforming AST nodes.
pub trait VisitorMut {
    type Output;
    type Error;

    /// Visit a program, potentially modifying it.
    fn visit_program_mut(&mut self, program: &mut Program) -> Result<Self::Output> {
        self.visit_expressions_mut(&mut program.expressions)
    }

    /// Visit a list of expressions, potentially modifying them.
    fn visit_expressions_mut(&mut self, expressions: &mut Vec<Spanned<Expr>>) -> Result<Self::Output>;

    /// Visit an expression, potentially modifying it.
    fn visit_expr_mut(&mut self, expr: &mut Spanned<Expr>) -> Result<Self::Output>;
}

/// A visitor that counts the number of nodes of each type.
#[derive(Debug, Default)]
pub struct NodeCounter {
    pub literals: usize,
    pub identifiers: usize,
    pub keywords: usize,
    pub quotes: usize,
    pub lambdas: usize,
    pub ifs: usize,
    pub defines: usize,
    pub sets: usize,
    pub applications: usize,
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

    fn visit_syntax_rules(&mut self, _literals: &[String], rules: &[(Spanned<Expr>, Spanned<Expr>)]) {
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
    
    fn visit_define_library(&mut self, _name: &[String], imports: &[Spanned<Expr>], exports: &[Spanned<Expr>], body: &[Spanned<Expr>]) {
        self.total += 1;
        self.visit_expressions(imports);
        self.visit_expressions(exports);
        self.visit_expressions(body);
    }

    fn visit_list(&mut self, elements: &[Spanned<Expr>]) {
        self.total += 1;
        self.visit_expressions(elements);
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