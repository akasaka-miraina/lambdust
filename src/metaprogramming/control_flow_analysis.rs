//! Control flow analysis components.

use crate::ast::Expr;
use crate::diagnostics::Spanned;
use std::collections::HashMap;

/// Control flow graph for a program.
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// Basic blocks
    pub blocks: HashMap<String, BasicBlock>,
    /// Entry block
    pub entry: String,
    /// Exit blocks
    pub exits: Vec<String>,
    /// Dominator tree
    pub dominators: HashMap<String, String>,
}

/// Basic block in control flow.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Block identifier
    pub id: String,
    /// Expressions in the block
    pub expressions: Vec<Spanned<Expr>>,
    /// Successor blocks
    pub successors: Vec<String>,
    /// Predecessor blocks
    pub predecessors: Vec<String>,
}

impl ControlFlowGraph {
    /// Creates a new empty control flow graph.
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            entry: String::new(),
            exits: Vec::new(),
            dominators: HashMap::new(),
        }
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}