//! Dependency analysis for detecting variable and function dependencies

use crate::ast::Expr;
use std::collections::HashMap;

/// Represents a dependency between variables or functions
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyNode {
    /// Name of the variable or function
    pub name: String,
    /// Expression that defines this node
    pub expr: Expr,
    /// Names of variables/functions this depends on
    pub depends_on: Vec<String>,
    /// Type of dependency
    pub dependency_type: DependencyType,
}

/// Type of dependency relationship
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    /// Variable definition (define x ...)
    Variable,
    /// Function definition (define (f x) ...)
    Function,
    /// Lambda expression
    Lambda,
    /// Direct reference
    Reference,
}

/// Dependency graph for static analysis
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Map of name to dependency node
    pub nodes: HashMap<String, DependencyNode>,
    /// Adjacency list representation
    pub edges: HashMap<String, Vec<String>>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Add a dependency node to the graph
    pub fn add_node(&mut self, node: DependencyNode) {
        let name = node.name.clone();
        let dependencies = node.depends_on.clone();
        
        self.nodes.insert(name.clone(), node);
        self.edges.insert(name.clone(), dependencies.clone());
        
        // Ensure all dependency targets exist in edges (even if empty)
        for dep in dependencies {
            self.edges.entry(dep).or_default();
        }
    }

    /// Get all nodes that depend on the given node
    pub fn get_dependents(&self, name: &str) -> Vec<String> {
        let mut dependents = Vec::new();
        for (node_name, deps) in &self.edges {
            if deps.contains(&name.to_string()) {
                dependents.push(node_name.clone());
            }
        }
        dependents
    }

    /// Check if there's a direct dependency between two nodes
    pub fn has_direct_dependency(&self, from: &str, to: &str) -> bool {
        if let Some(deps) = self.edges.get(from) {
            deps.contains(&to.to_string())
        } else {
            false
        }
    }

    /// Get all nodes
    pub fn get_nodes(&self) -> &HashMap<String, DependencyNode> {
        &self.nodes
    }

    /// Get all edges
    pub fn get_edges(&self) -> &HashMap<String, Vec<String>> {
        &self.edges
    }
}

/// Analyzes expressions to build dependency graphs
pub struct DependencyAnalyzer {
    /// Current dependency graph
    graph: DependencyGraph,
    /// Stack for tracking analysis context
    #[allow(dead_code)]
    context_stack: Vec<String>,
}

impl DependencyAnalyzer {
    /// Create a new dependency analyzer
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
            context_stack: Vec::new(),
        }
    }

    /// Analyze a list of expressions and build dependency graph
    pub fn analyze_expressions(&mut self, expressions: &[Expr]) -> &DependencyGraph {
        for expr in expressions {
            self.analyze_expression(expr);
        }
        &self.graph
    }

    /// Analyze a single expression
    fn analyze_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                self.analyze_list_expression(exprs);
            }
            Expr::Variable(name) => {
                self.record_variable_reference(name.clone());
            }
            Expr::Quote(_) => {
                // Quoted expressions don't create dependencies
            }
            Expr::Vector(exprs) => {
                // Analyze vector elements
                for expr in exprs {
                    self.analyze_expression(expr);
                }
            }
            Expr::DottedList(exprs, tail) => {
                // Analyze dotted list elements
                for expr in exprs {
                    self.analyze_expression(expr);
                }
                self.analyze_expression(tail);
            }
            _ => {
                // Other expressions don't create dependencies
            }
        }
    }

    /// Analyze a list expression (function call or special form)
    fn analyze_list_expression(&mut self, exprs: &[Expr]) {
        if let Some(Expr::Variable(name)) = exprs.first() {
            match name.as_str() {
                "define" => self.analyze_define_expression(exprs),
                "lambda" => self.analyze_lambda_expression(exprs),
                "let" | "let*" | "letrec" => self.analyze_let_expression(exprs),
                _ => {
                    // Regular function call - analyze all arguments
                    for expr in exprs {
                        self.analyze_expression(expr);
                    }
                }
            }
        } else {
            // Not a function call, analyze all subexpressions
            for expr in exprs {
                self.analyze_expression(expr);
            }
        }
    }

    /// Analyze a define expression
    fn analyze_define_expression(&mut self, exprs: &[Expr]) {
        if exprs.len() < 3 {
            return; // Invalid define
        }

        match &exprs[1] {
            // Variable definition: (define x expr)
            Expr::Variable(name) => {
                let dependencies = self.collect_dependencies(&exprs[2]);
                let node = DependencyNode {
                    name: name.clone(),
                    expr: exprs[2].clone(),
                    depends_on: dependencies,
                    dependency_type: DependencyType::Variable,
                };
                self.graph.add_node(node);
            }
            // Function definition: (define (f x y) body)
            Expr::List(func_def) if !func_def.is_empty() => {
                if let Expr::Variable(func_name) = &func_def[0] {
                    let dependencies = self.collect_dependencies(&exprs[2]);
                    let node = DependencyNode {
                        name: func_name.clone(),
                        expr: exprs[2].clone(),
                        depends_on: dependencies,
                        dependency_type: DependencyType::Function,
                    };
                    self.graph.add_node(node);
                }
            }
            _ => {} // Invalid define
        }
    }

    /// Analyze a lambda expression
    fn analyze_lambda_expression(&mut self, exprs: &[Expr]) {
        if exprs.len() < 3 {
            return; // Invalid lambda
        }

        // Analyze the body of the lambda
        for expr in exprs.iter().skip(2) {
            self.analyze_expression(expr);
        }
    }

    /// Analyze a let expression
    fn analyze_let_expression(&mut self, exprs: &[Expr]) {
        if exprs.len() < 3 {
            return; // Invalid let
        }

        // Analyze bindings
        if let Expr::List(bindings) = &exprs[1] {
            for binding in bindings {
                if let Expr::List(binding_pair) = binding {
                    if binding_pair.len() == 2 {
                        self.analyze_expression(&binding_pair[1]);
                    }
                }
            }
        }

        // Analyze body
        for expr in exprs.iter().skip(2) {
            self.analyze_expression(expr);
        }
    }

    /// Collect all variable references in an expression
    fn collect_dependencies(&self, expr: &Expr) -> Vec<String> {
        let mut dependencies = Vec::new();
        self.collect_dependencies_recursive(expr, &mut dependencies);
        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    /// Recursively collect dependencies
    #[allow(clippy::only_used_in_recursion)]
    fn collect_dependencies_recursive(&self, expr: &Expr, dependencies: &mut Vec<String>) {
        match expr {
            Expr::Variable(name) => {
                dependencies.push(name.clone());
            }
            Expr::List(exprs) => {
                for expr in exprs {
                    self.collect_dependencies_recursive(expr, dependencies);
                }
            }
            Expr::Vector(exprs) => {
                for expr in exprs {
                    self.collect_dependencies_recursive(expr, dependencies);
                }
            }
            Expr::DottedList(exprs, tail) => {
                for expr in exprs {
                    self.collect_dependencies_recursive(expr, dependencies);
                }
                self.collect_dependencies_recursive(tail, dependencies);
            }
            Expr::Quote(_) => {
                // Quoted expressions don't create dependencies
            }
            Expr::Quasiquote(expr) => {
                self.collect_dependencies_recursive(expr, dependencies);
            }
            Expr::Unquote(expr) => {
                self.collect_dependencies_recursive(expr, dependencies);
            }
            Expr::UnquoteSplicing(expr) => {
                self.collect_dependencies_recursive(expr, dependencies);
            }
            Expr::Literal(_) => {
                // Literals don't create dependencies
            }
        }
    }

    /// Record a variable reference in the current context
    fn record_variable_reference(&mut self, _name: String) {
        // This could be used for more sophisticated analysis
        // For now, we just track it implicitly through collect_dependencies
    }

    /// Get the final dependency graph
    pub fn get_graph(&self) -> &DependencyGraph {
        &self.graph
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_simple_variable_dependency() {
        let mut analyzer = DependencyAnalyzer::new();
        
        // (define x y)
        let define_expr = Expr::List(vec![
            Expr::Variable("define".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
        ]);
        
        let graph = analyzer.analyze_expressions(&[define_expr]);
        
        assert!(graph.nodes.contains_key("x"));
        let node = &graph.nodes["x"];
        assert_eq!(node.depends_on, vec!["y".to_string()]);
        assert_eq!(node.dependency_type, DependencyType::Variable);
    }

    #[test]
    fn test_function_dependency() {
        let mut analyzer = DependencyAnalyzer::new();
        
        // (define (foo x) (+ x y))
        let define_expr = Expr::List(vec![
            Expr::Variable("define".to_string()),
            Expr::List(vec![
                Expr::Variable("foo".to_string()),
                Expr::Variable("x".to_string()),
            ]),
            Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Variable("x".to_string()),
                Expr::Variable("y".to_string()),
            ]),
        ]);
        
        let graph = analyzer.analyze_expressions(&[define_expr]);
        
        assert!(graph.nodes.contains_key("foo"));
        let node = &graph.nodes["foo"];
        assert!(node.depends_on.contains(&"+".to_string()));
        assert!(node.depends_on.contains(&"x".to_string()));
        assert!(node.depends_on.contains(&"y".to_string()));
        assert_eq!(node.dependency_type, DependencyType::Function);
    }

    #[test]
    fn test_no_dependency_on_literals() {
        let mut analyzer = DependencyAnalyzer::new();
        
        // (define x 42)
        let define_expr = Expr::List(vec![
            Expr::Variable("define".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(42))),
        ]);
        
        let graph = analyzer.analyze_expressions(&[define_expr]);
        
        assert!(graph.nodes.contains_key("x"));
        let node = &graph.nodes["x"];
        assert!(node.depends_on.is_empty());
    }

    #[test]
    fn test_complex_dependency() {
        let mut analyzer = DependencyAnalyzer::new();
        
        // (define a (+ b c))
        // (define b (+ c 1))
        let expressions = vec![
            Expr::List(vec![
                Expr::Variable("define".to_string()),
                Expr::Variable("a".to_string()),
                Expr::List(vec![
                    Expr::Variable("+".to_string()),
                    Expr::Variable("b".to_string()),
                    Expr::Variable("c".to_string()),
                ]),
            ]),
            Expr::List(vec![
                Expr::Variable("define".to_string()),
                Expr::Variable("b".to_string()),
                Expr::List(vec![
                    Expr::Variable("+".to_string()),
                    Expr::Variable("c".to_string()),
                    Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(1))),
                ]),
            ]),
        ];
        
        let graph = analyzer.analyze_expressions(&expressions);
        
        // Check node 'a'
        assert!(graph.nodes.contains_key("a"));
        let node_a = &graph.nodes["a"];
        assert!(node_a.depends_on.contains(&"+".to_string()));
        assert!(node_a.depends_on.contains(&"b".to_string()));
        assert!(node_a.depends_on.contains(&"c".to_string()));
        
        // Check node 'b'
        assert!(graph.nodes.contains_key("b"));
        let node_b = &graph.nodes["b"];
        assert!(node_b.depends_on.contains(&"+".to_string()));
        assert!(node_b.depends_on.contains(&"c".to_string()));
    }
}